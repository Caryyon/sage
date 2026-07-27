//! sage-specialist-qa: Test specialist brains end-to-end through the KnowledgeLoop.
//!
//! Loads a specialist template, runs queries through the full chat pipeline,
//! and reports whether each answer came from local synthesis (NCA) or LLM.
//!
//! Usage: sage-specialist-qa --template accounting-specialist
//!        sage-specialist-qa --template accounting-specialist --no-llm

use clap::Parser;
use sage::brain_templates::{default_templates_dir, find_template};
use sage::distributed_knowledge::{default_brain_path, KnowledgeStore, NCAKnowledge};
use sage::inference::{InferenceEngine, OllamaEngine};
use sage::knowledge_loop::KnowledgeLoop;
use std::sync::Arc;

#[derive(Parser)]
#[command(name = "sage-specialist-qa", about = "Test specialist brains end-to-end")]
struct Cli {
    /// Template name to test
    #[arg(long)]
    template: String,

    /// Brain path override
    #[arg(long)]
    brain: Option<String>,

    /// Don't use LLM at all (local synthesis only)
    #[arg(long)]
    no_llm: bool,

    /// Ollama model (ignored if --no-llm)
    #[arg(short, long, default_value = "qwen2.5:14b")]
    model: String,

    /// Ollama URL
    #[arg(long, default_value = "http://localhost:11434")]
    ollama_url: String,

    /// Queries to test (or omit for default test set)
    queries: Vec<String>,
}

fn main() {
    let cli = Cli::parse();
    let brain_path = cli.brain.unwrap_or_else(default_brain_path);

    // Load the specialist template into the brain
    println!("📦 Loading template: {}", cli.template);
    let templates_dir = default_templates_dir();
    let bundle = match find_template(&cli.template, &templates_dir) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("❌ {}", e);
            std::process::exit(1);
        }
    };

    // Import template to brain
    let mut knowledge = bundle.to_knowledge();
    let n_cells = knowledge.active_knowledge(0.01).len();
    let n_texts = knowledge.text_store.len();
    knowledge.save(&brain_path).expect("save brain");
    println!("   {} active cells, {} text entries", n_cells, n_texts);

    // Create the inference engine
    // Use Ollama directly — the NCA LM has 0% accuracy and pollutes results
    let engine: Arc<dyn sage::inference::InferenceEngine> = if cli.no_llm {
        Arc::new(sage::inference::LocalSynthesizer::new())
    } else {
        // Try Ollama directly
        let ollama = OllamaEngine::new(Some(cli.model.clone()), Some(cli.ollama_url.clone()));
        if ollama.is_available() {
            println!("🧠 Using Ollama ({})", cli.model);
            Arc::new(ollama)
        } else {
            println!("⚠️  Ollama not available, using local synthesis");
            Arc::new(sage::inference::LocalSynthesizer::new())
        }
    };

    // Create KnowledgeLoop and load brain
    let system_prompt = format!(
        "You are a SAGE specialist. Answer questions based on your trained knowledge. Be concise and factual."
    );

    let mut loop_ = KnowledgeLoop::new(engine)
        .with_system_prompt(&system_prompt)
        .with_brain_path(&brain_path);
    loop_.load_brain().expect("load brain");

    // Default test queries if none provided
    let queries: Vec<String> = if cli.queries.is_empty() {
        vec![
            "What is the accounting equation?".into(),
            "Define photosynthesis".into(),
            "What is the Pythagorean theorem?".into(),
            "What is double-entry bookkeeping?".into(),
            "What is a prime number?".into(),
            "What is HTTP?".into(),
            "What is GDP?".into(),
            "What is SQL?".into(),
        ]
    } else {
        cli.queries
    };

    println!("\n═══════════════════════════════════════════════════");
    println!("  Testing {} queries", queries.len());
    println!("  LLM: {}", if cli.no_llm { "DISABLED (local synthesis only)" } else { "enabled (with NCA local synthesis first)" });
    println!("═══════════════════════════════════════════════════\n");

    // Also load a separate brain for direct query comparison
    let mut debug_knowledge = NCAKnowledge::new();
    debug_knowledge.load(&brain_path).expect("load brain for debug");

    let mut nca_count = 0;
    let mut llm_count = 0;
    let mut err_count = 0;

    for (i, query) in queries.iter().enumerate() {
        print!("━━━ Q{}: {} ━━━\n", i + 1, query);

        // Debug: direct query to brain
        let debug_results = debug_knowledge.query(query, 5);
        println!("   📚 Direct query: {} results", debug_results.len());
        for (j, r) in debug_results.iter().enumerate() {
            let text = r.text.as_deref().unwrap_or("(no text)");
            let truncated = if text.len() > 80 { format!("{}...", &text[..80]) } else { text.to_string() };
            println!("     {}. [rel={:.3}] {}", j+1, r.relevance, truncated);
        }

        // Debug: try LocalSynthesizer directly with brain query results
        let passages: Vec<String> = debug_results.iter()
            .filter_map(|r| r.text.clone())
            .collect();
        match sage::inference::LocalSynthesizer::synthesize(query, &passages) {
            Some(answer) => println!("   🔬 Direct synthesis: {}", &answer[..answer.len().min(120)]),
            None => println!("   🔬 Direct synthesis: None (no confident answer)"),
        }

        let start = std::time::Instant::now();
        match loop_.chat(query) {
            Ok(response) => {
                let elapsed = start.elapsed();

                // Heuristic: local synthesis responses are short and extractive
                // LLM responses are longer and more conversational
                let is_local = response.len() < 400
                    && !response.contains("I'd be happy to")
                    && !response.contains("Great question")
                    && !response.contains("Certainly!");

                if is_local {
                    nca_count += 1;
                    println!("🟢 [NCA/Local Synthesis] ({:.1}s)", elapsed.as_secs_f64());
                } else {
                    llm_count += 1;
                    println!("🔵 [LLM] ({:.1}s)", elapsed.as_secs_f64());
                }
                // Truncate long responses for display
                if response.len() > 300 {
                    println!("   {}...", &response[..300]);
                } else {
                    println!("   {}", response);
                }
            }
            Err(e) => {
                err_count += 1;
                println!("❌ [ERROR] {}", e);
            }
        }
        println!();
    }

    println!("═══════════════════════════════════════════════════");
    println!("  Results: {} NCA/Local, {} LLM, {} errors (out of {})", nca_count, llm_count, err_count, queries.len());
    let total = queries.len() as f64;
    if total > 0.0 {
        println!("  LLM call reduction: {:.0}%", (nca_count as f64 / total) * 100.0);
    }
    println!("═══════════════════════════════════════════════════");
}