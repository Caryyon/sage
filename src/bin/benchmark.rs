//! SAGE Retrieval Quality Benchmark
//!
//! Encodes 50 fact-pairs (question→answer style) and queries each back.
//! Measures:
//!   - Hit rate (exact answer in top-5 results)
//!   - Mean relevance score
//!   - Semantic-only vs delta-attention vs combined retrieval
//!
//! Usage: cargo run --bin benchmark
//! Results saved to ~/clawd/sage-team/sage-daily-dev/<date>-benchmark-results.md

use sage::distributed_knowledge::{KnowledgeStore, NCAKnowledge};
use sage::distributed_knowledge::decoder::{KnowledgeActivation, query_knowledge_with_text};
use sage::distributed_knowledge::encoder::EncoderConfig;
use sage::distributed_knowledge::attention_decoder::AttentionDecoder;
use std::time::Instant;

/// The 50 fact-pairs used for benchmarking.
/// Format: (query, answer) — answer must appear in the encoded text to count as a hit.
const FACTS: &[(&str, &str)] = &[
    ("What is the capital of France?", "Paris"),
    ("What is the capital of Germany?", "Berlin"),
    ("What is the capital of Japan?", "Tokyo"),
    ("What is the capital of Brazil?", "Brasilia"),
    ("What is the capital of Australia?", "Canberra"),
    ("What is the capital of Canada?", "Ottawa"),
    ("What is the capital of Italy?", "Rome"),
    ("What is the capital of Spain?", "Madrid"),
    ("What element has atomic number 1?", "hydrogen"),
    ("What element has atomic number 6?", "carbon"),
    ("What element has atomic number 8?", "oxygen"),
    ("What element has atomic number 79?", "gold"),
    ("What is the speed of light in vacuum?", "299792458"),
    ("What planet is closest to the Sun?", "Mercury"),
    ("What planet is largest in the solar system?", "Jupiter"),
    ("What is the largest ocean on Earth?", "Pacific"),
    ("What is the longest river in the world?", "Nile"),
    ("What is the highest mountain on Earth?", "Everest"),
    ("Who wrote Hamlet?", "Shakespeare"),
    ("Who wrote Romeo and Juliet?", "Shakespeare"),
    ("Who wrote 1984?", "Orwell"),
    ("Who wrote Pride and Prejudice?", "Austen"),
    ("What language does Python code run in?", "Python"),
    ("What does CPU stand for?", "processing"),
    ("What does RAM stand for?", "memory"),
    ("What does HTTP stand for?", "hypertext"),
    ("What is the boiling point of water in Celsius?", "100"),
    ("What is the freezing point of water in Celsius?", "zero"),
    ("How many sides does a hexagon have?", "six"),
    ("How many sides does a triangle have?", "three"),
    ("What is the square root of 144?", "12"),
    ("What is 7 times 8?", "56"),
    ("What is Pi approximately equal to?", "3.14"),
    ("What gas do plants absorb from air?", "carbon dioxide"),
    ("What gas do humans exhale?", "carbon dioxide"),
    ("What organ pumps blood in the human body?", "heart"),
    ("What is the powerhouse of the cell?", "mitochondria"),
    ("What is DNA short for?", "deoxyribonucleic"),
    ("What year did World War 2 end?", "1945"),
    ("What year did the Berlin Wall fall?", "1989"),
    ("What year did Neil Armstrong walk on the Moon?", "1969"),
    ("What is the chemical formula of water?", "H2O"),
    ("What is the chemical formula of salt?", "NaCl"),
    ("What is the hardest natural material?", "diamond"),
    ("What is the most abundant gas in Earth atmosphere?", "nitrogen"),
    ("How many continents are on Earth?", "seven"),
    ("How many bones are in the adult human body?", "206"),
    ("What is the SI unit of force?", "Newton"),
    ("What is the SI unit of energy?", "Joule"),
    ("What is the SI unit of electric current?", "Ampere"),
];

/// Encoded text for each fact (full sentence to encode, not just the answer).
/// We encode the full Q→A sentence so the grid stores rich semantic content.
fn fact_text(query: &str, answer: &str) -> String {
    format!("{} The answer is {}.", query, answer)
}

#[derive(Debug, Clone)]
struct MethodResult {
    name: &'static str,
    hits: usize,
    total: usize,
    mean_relevance: f64,
    mean_result_count: f64,
    elapsed_ms: f64,
}

impl MethodResult {
    fn hit_rate(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            self.hits as f64 / self.total as f64
        }
    }
}

/// Check if the answer keyword appears in any of the retrieved results.
fn check_hit(results: &[KnowledgeActivation], answer: &str) -> bool {
    let answer_lower = answer.to_lowercase();
    results.iter().any(|r| {
        if let Some(ref text) = r.text {
            text.to_lowercase().contains(answer_lower.as_str())
        } else {
            false
        }
    })
}

/// Compute mean relevance from results.
fn mean_relevance(results: &[KnowledgeActivation]) -> f64 {
    if results.is_empty() {
        return 0.0;
    }
    results.iter().map(|r| r.relevance).sum::<f64>() / results.len() as f64
}

fn run_semantic_benchmark(store: &NCAKnowledge) -> MethodResult {
    let mut hits = 0;
    let mut total_relevance = 0.0;
    let mut total_results = 0.0;
    let start = Instant::now();

    let mut config = EncoderConfig::default();
    config.ollama_url = None; // Force hash fallback for reproducibility

    for &(query, answer) in FACTS {
        let results = query_knowledge_with_text(
            &store.grid,
            query,
            &config,
            5,
            Some(&store.text_store),
        );
        if check_hit(&results, answer) {
            hits += 1;
        }
        total_relevance += mean_relevance(&results);
        total_results += results.len() as f64;
    }

    let elapsed = start.elapsed().as_secs_f64() * 1000.0;

    MethodResult {
        name: "Semantic (hash)",
        hits,
        total: FACTS.len(),
        mean_relevance: total_relevance / FACTS.len() as f64,
        mean_result_count: total_results / FACTS.len() as f64,
        elapsed_ms: elapsed,
    }
}

fn run_delta_benchmark(store: &mut NCAKnowledge) -> MethodResult {
    let mut hits = 0;
    let mut total_relevance = 0.0;
    let mut total_results = 0.0;
    let start = Instant::now();

    let decoder = AttentionDecoder::new(store.grid.width, store.grid.height);

    for &(query, answer) in FACTS {
        // Use delta-attention retrieval (NCA freerun + delta magnitude)
        let results = decoder.attend_with_delta(
            &mut store.grid,
            &[], // no Ollama embedding — use pure delta
            5,
            Some(&store.text_store),
        );
        if check_hit(&results, answer) {
            hits += 1;
        }
        total_relevance += mean_relevance(&results);
        total_results += results.len() as f64;
    }

    let elapsed = start.elapsed().as_secs_f64() * 1000.0;

    MethodResult {
        name: "Delta Attention (NCA)",
        hits,
        total: FACTS.len(),
        mean_relevance: total_relevance / FACTS.len() as f64,
        mean_result_count: total_results / FACTS.len() as f64,
        elapsed_ms: elapsed,
    }
}

fn run_combined_benchmark(store: &mut NCAKnowledge) -> MethodResult {
    // Combined: union of semantic results and delta results (deduped)
    let mut hits = 0;
    let mut total_relevance = 0.0;
    let mut total_results = 0.0;
    let start = Instant::now();

    let mut config = EncoderConfig::default();
    config.ollama_url = None;
    let decoder = AttentionDecoder::new(store.grid.width, store.grid.height);

    for &(query, answer) in FACTS {
        let mut semantic = query_knowledge_with_text(
            &store.grid,
            query,
            &config,
            5,
            Some(&store.text_store),
        );

        let delta = decoder.attend_with_delta(
            &mut store.grid,
            &[],
            5,
            Some(&store.text_store),
        );

        // Combine: merge delta results not already in semantic results
        let mut seen_texts: std::collections::HashSet<String> = semantic
            .iter()
            .filter_map(|r| r.text.clone())
            .collect();

        for d_result in delta {
            if let Some(ref t) = d_result.text {
                if !seen_texts.contains(t) {
                    seen_texts.insert(t.clone());
                    semantic.push(d_result);
                }
            }
        }

        // Sort by relevance, take top 5
        semantic.sort_by(|a, b| b.relevance.partial_cmp(&a.relevance).unwrap_or(std::cmp::Ordering::Equal));
        semantic.truncate(5);

        if check_hit(&semantic, answer) {
            hits += 1;
        }
        total_relevance += mean_relevance(&semantic);
        total_results += semantic.len() as f64;
    }

    let elapsed = start.elapsed().as_secs_f64() * 1000.0;

    MethodResult {
        name: "Combined (Semantic + Delta)",
        hits,
        total: FACTS.len(),
        mean_relevance: total_relevance / FACTS.len() as f64,
        mean_result_count: total_results / FACTS.len() as f64,
        elapsed_ms: elapsed,
    }
}

fn print_table(results: &[MethodResult]) {
    println!("\n## SAGE Retrieval Quality Benchmark\n");
    println!("| Method                      | Hit Rate | Mean Relevance | Mean Results | Time (ms) |");
    println!("|------------------------------|----------|----------------|--------------|-----------|");
    for r in results {
        println!(
            "| {:<28} | {:>7.1}% | {:>13.4} | {:>12.1} | {:>9.1} |",
            r.name,
            r.hit_rate() * 100.0,
            r.mean_relevance,
            r.mean_result_count,
            r.elapsed_ms
        );
    }
    println!();
}

fn save_results(results: &[MethodResult], store_stats: &str) {
    let date = chrono::Local::now().format("%Y-%m-%d").to_string();
    let dir = std::path::PathBuf::from(format!(
        "{}/clawd/sage-team/sage-daily-dev",
        dirs::home_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .display()
    ));
    std::fs::create_dir_all(&dir).ok();

    let path = dir.join(format!("{}-benchmark-results.md", date));

    let mut md = String::new();
    md.push_str(&format!("# SAGE Retrieval Quality Benchmark — {}\n\n", date));
    md.push_str("## Setup\n\n");
    md.push_str(&format!("{}\n\n", store_stats));
    md.push_str("## Results\n\n");
    md.push_str("| Method                      | Hit Rate | Mean Relevance | Mean Results | Time (ms) |\n");
    md.push_str("|------------------------------|----------|----------------|--------------|----------|\n");
    for r in results {
        md.push_str(&format!(
            "| {:<28} | {:>7.1}% | {:>13.4} | {:>12.1} | {:>9.1} |\n",
            r.name,
            r.hit_rate() * 100.0,
            r.mean_relevance,
            r.mean_result_count,
            r.elapsed_ms
        ));
    }

    md.push_str("\n## Detailed Results\n\n");
    for r in results {
        md.push_str(&format!(
            "### {}\n- Hits: {}/{} ({:.1}%)\n- Mean Relevance: {:.4}\n- Mean Results: {:.1}\n- Time: {:.1}ms\n\n",
            r.name, r.hits, r.total, r.hit_rate() * 100.0,
            r.mean_relevance, r.mean_result_count, r.elapsed_ms
        ));
    }

    md.push_str("## Notes\n\n");
    md.push_str("- Ollama embeddings disabled (hash fallback) for reproducibility\n");
    md.push_str("- 50 fact-pairs encoded as full Q→A sentences\n");
    md.push_str("- Hit = answer keyword found in top-5 retrieved results\n");
    md.push_str("- Delta Attention uses NCA freerun + per-cell L2 delta magnitude\n");
    md.push_str("- Combined = union of semantic + delta, deduped, top-5 by relevance\n");

    match std::fs::write(&path, &md) {
        Ok(()) => println!("📄 Results saved to: {}", path.display()),
        Err(e) => eprintln!("⚠  Failed to save results: {}", e),
    }
}

fn main() {
    println!("🔬 SAGE Retrieval Quality Benchmark");
    println!("   Facts: {}", FACTS.len());
    println!("   Grid: 256×256");
    println!("   Embeddings: hash fallback (Ollama disabled for reproducibility)");
    println!();

    // Build the knowledge store
    let mut config = EncoderConfig::default();
    config.ollama_url = None; // reproducible benchmark

    println!("📝 Encoding {} fact-pairs...", FACTS.len());
    let encode_start = Instant::now();

    let mut store = NCAKnowledge::new();

    for &(query, answer) in FACTS {
        let text = fact_text(query, answer);
        store.encode(&text, 0.9);
    }

    let encode_ms = encode_start.elapsed().as_secs_f64() * 1000.0;
    println!("   Encoded in {:.1}ms ({:.1}ms/fact)", encode_ms, encode_ms / FACTS.len() as f64);

    let store_stats = format!(
        "- Facts encoded: {}\n- Grid size: {}×{}\n- Encoding time: {:.1}ms",
        FACTS.len(),
        store.grid.width,
        store.grid.height,
        encode_ms
    );

    // Run benchmarks
    println!("\n🏃 Running semantic benchmark...");
    let semantic_result = run_semantic_benchmark(&store);

    println!("🏃 Running delta-attention benchmark...");
    let delta_result = run_delta_benchmark(&mut store);

    println!("🏃 Running combined benchmark...");
    let combined_result = run_combined_benchmark(&mut store);

    let results = vec![semantic_result, delta_result, combined_result];

    // Print table
    print_table(&results);

    // Detailed breakdown
    println!("## Per-fact hit report (Combined method)\n");
    {
        let mut combined_store = NCAKnowledge::new();
        for &(query, answer) in FACTS {
            let text = fact_text(query, answer);
            combined_store.encode(&text, 0.9);
        }

        let decoder = AttentionDecoder::new(combined_store.grid.width, combined_store.grid.height);
        let enc_config = {
            let mut c = EncoderConfig::default();
            c.ollama_url = None;
            c
        };

        let mut semantic_hits = 0;
        let mut combined_hits = 0;

        for &(query, answer) in FACTS {
            let sem = query_knowledge_with_text(
                &combined_store.grid,
                query,
                &enc_config,
                5,
                Some(&combined_store.text_store),
            );
            let delta = decoder.attend_with_delta(
                &mut combined_store.grid,
                &[],
                5,
                Some(&combined_store.text_store),
            );

            let sem_hit = check_hit(&sem, answer);
            let delta_hit = check_hit(&delta, answer);
            let combined_hit = sem_hit || delta_hit;

            if sem_hit { semantic_hits += 1; }
            if combined_hit { combined_hits += 1; }

            let flag = match (sem_hit, delta_hit) {
                (true, true) => "✅✅",
                (true, false) => "✅❌",
                (false, true) => "❌✅",
                (false, false) => "❌❌",
            };
            println!("  {} sem={} delta={} | {:40} → {}", flag, sem_hit as u8, delta_hit as u8, query, answer);
        }
        println!();
        println!("  Semantic hits:  {}/{}", semantic_hits, FACTS.len());
        println!("  Combined hits:  {}/{}", combined_hits, FACTS.len());
        println!("  Delta-unique:   {} (found by delta only)", combined_hits - semantic_hits);
    }

    save_results(&results, &store_stats);
}
