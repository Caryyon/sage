//! sage-curriculum — Bulk curriculum ingestion CLI
//!
//! Feeds structured domain knowledge into SAGE grids for domain-expert training.
//!
//! Usage:
//!   sage-curriculum ingest <curriculum.json>              — load curriculum, report quality
//!   sage-curriculum ingest <curriculum.json> --template   — also export brain template
//!   sage-curriculum sample --name cs-fundamentals          — generate sample curriculum
//!   sage-curriculum verify <curriculum.json>               — test retrieval on existing brain

use clap::{Parser, Subcommand};
use sage::brain_templates::{default_templates_dir, BrainTemplateBundle};
use sage::curriculum::{
    ingest_curriculum, load_curriculum, sample_curriculum, Curriculum, IngestionConfig,
};
use sage::distributed_knowledge::{KnowledgeStore, NCAKnowledge};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "sage-curriculum", about = "Bulk curriculum ingestion for SAGE domain-expert training")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Brain path (default: ~/.sage/brain.bin)
    #[arg(short, long, env = "SAGE_BRAIN_PATH", global = true)]
    brain: Option<String>,

    /// Confidence for injected facts (0.0-1.0)
    #[arg(short, long, default_value_t = 0.95, global = true)]
    confidence: f64,

    /// Consolidation steps between topics (default: 5)
    #[arg(long, default_value_t = 5, global = true)]
    consolidation_steps: usize,

    /// Skip verification after each topic
    #[arg(long, global = true)]
    no_verify: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Ingest a curriculum JSON into the brain
    Ingest {
        /// Path to curriculum JSON file
        curriculum: PathBuf,

        /// Also export a brain template after ingestion
        #[arg(short, long)]
        template: bool,

        /// Template name (defaults to curriculum name)
        #[arg(long)]
        template_name: Option<String>,

        /// Template description
        #[arg(long)]
        template_desc: Option<String>,

        /// Comma-separated tags
        #[arg(long, value_delimiter = ',')]
        template_tags: Vec<String>,
    },

    /// Generate a sample curriculum file
    Sample {
        /// Output file (default: stdout)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Curriculum name
        #[arg(short, long, default_value = "cs-fundamentals")]
        name: String,
    },

    /// Verify a curriculum's facts against an existing brain
    Verify {
        /// Path to curriculum JSON file
        curriculum: PathBuf,
    },

    /// Show curriculum structure without ingesting
    Show {
        /// Path to curriculum JSON file
        curriculum: PathBuf,
    },
}

fn main() {
    let cli = Cli::parse();
    let brain_path = cli
        .brain
        .unwrap_or_else(|| default_brain_path());

    match cli.command {
        Commands::Ingest {
            curriculum,
            template,
            template_name,
            template_desc,
            template_tags,
        } => cmd_ingest(
            &curriculum,
            &brain_path,
            template,
            template_name,
            template_desc,
            template_tags,
            cli.confidence,
            cli.consolidation_steps,
            cli.no_verify,
        ),
        Commands::Sample { output, name } => cmd_sample(output, &name),
        Commands::Verify { curriculum } => cmd_verify(&curriculum, &brain_path),
        Commands::Show { curriculum } => cmd_show(&curriculum),
    }
}

fn cmd_ingest(
    path: &PathBuf,
    brain_path: &str,
    export_template: bool,
    template_name: Option<String>,
    template_desc: Option<String>,
    template_tags: Vec<String>,
    confidence: f64,
    consolidation_steps: usize,
    no_verify: bool,
) {
    let curriculum = match load_curriculum(path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("❌ Failed to load curriculum: {}", e);
            std::process::exit(1);
        }
    };

    println!("📚 Loading curriculum: {}", curriculum.name);
    println!("   Domain: {}", curriculum.domain);
    println!("   Topics: {} | Facts: {}", curriculum.topics.len(), count_facts(&curriculum));

    // Load or create brain
    let mut knowledge = NCAKnowledge::new();
    let brain_exists = std::path::Path::new(brain_path).exists();
    if brain_exists {
        println!("\n📂 Loading existing brain from {}", brain_path);
        if let Err(e) = knowledge.load(brain_path) {
            eprintln!("⚠️  Could not load brain (will create new): {}", e);
        }
    } else {
        println!("\n🆕 Creating new brain");
    }

    let config = IngestionConfig {
        confidence,
        consolidation_steps,
        verify_each_topic: !no_verify,
        ..Default::default()
    };

    println!("\n⚙️  Encoding...");
    let report = ingest_curriculum(&mut knowledge, &curriculum, &config);

    println!("\n{}", report.summary());

    // Save brain
    if let Err(e) = knowledge.save(brain_path) {
        eprintln!("❌ Failed to save brain: {}", e);
        std::process::exit(1);
    }
    println!("💾 Saved brain to {}", brain_path);

    // Export template if requested
    if export_template {
        let name = template_name.unwrap_or_else(|| curriculum.name.clone());
        let desc = template_desc.unwrap_or_else(|| {
            format!(
                "Domain expert: {} ({} facts, {:.0}% hit rate)",
                curriculum.name,
                report.total_facts,
                report.overall_hit_rate * 100.0
            )
        });
        let tags = if template_tags.is_empty() {
            vec![curriculum.domain.clone()]
        } else {
            template_tags
        };

        let bundle = BrainTemplateBundle::from_knowledge(
            &knowledge,
            &name,
            &desc,
            tags,
            Some(curriculum.domain.clone()),
        );

        let templates_dir = default_templates_dir();
        match bundle.save(&templates_dir) {
            Ok(path) => println!("📦 Exported template '{}' → {}", name, path),
            Err(e) => eprintln!("⚠️  Failed to export template: {}", e),
        }
    }
}

fn cmd_sample(output: Option<PathBuf>, name: &str) {
    let mut curriculum = sample_curriculum();
    curriculum.name = name.to_string();

    let json = serde_json::to_string_pretty(&curriculum).unwrap();

    if let Some(path) = output {
        std::fs::write(&path, &json).unwrap_or_else(|e| {
            eprintln!("Failed to write: {}", e);
            std::process::exit(1);
        });
        println!("📄 Sample curriculum written to {}", path.display());
    } else {
        println!("{}", json);
    }
}

fn cmd_verify(path: &PathBuf, brain_path: &str) {
    let curriculum = match load_curriculum(path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("❌ Failed to load curriculum: {}", e);
            std::process::exit(1);
        }
    };

    let mut knowledge = NCAKnowledge::new();
    if let Err(e) = knowledge.load(brain_path) {
        eprintln!("❌ Cannot load brain: {}", e);
        std::process::exit(1);
    }

    println!("🔍 Verifying curriculum '{}' against brain...", curriculum.name);
    println!("   Topics: {} | Facts: {}", curriculum.topics.len(), count_facts(&curriculum));

    let config = IngestionConfig::default();
    let report = ingest_curriculum(&mut knowledge, &curriculum, &config);

    println!("\n{}", report.summary());

    // Don't save — verify-only mode
}

fn cmd_show(path: &PathBuf) {
    let curriculum = match load_curriculum(path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("❌ Failed to load curriculum: {}", e);
            std::process::exit(1);
        }
    };

    println!("📋 Curriculum: {}", curriculum.name);
    println!("   Domain: {}", curriculum.domain);
    if let Some(desc) = &curriculum.description {
        println!("   Description: {}", desc);
    }
    println!("\n{} topic(s), {} facts total", curriculum.topics.len(), count_facts(&curriculum));

    for topic in &curriculum.topics {
        println!("\n  📘 {} ({} facts)", topic.name, topic.facts.len());
        if let Some(region) = &topic.region {
            println!("     Region: {:?}", region);
        }
        for fact in &topic.facts {
            let query_str = fact
                .query
                .as_ref()
                .map(|q| format!(" [query: \"{}\"]", q))
                .unwrap_or_default();
            println!("     • {}{}", truncate(&fact.fact, 80), query_str);
        }
    }
}

fn count_facts(curriculum: &Curriculum) -> usize {
    curriculum.topics.iter().map(|t| t.facts.len()).sum()
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max])
    }
}

fn default_brain_path() -> String {
    dirs::home_dir()
        .unwrap_or_default()
        .join(".sage")
        .join("brain.bin")
        .to_string_lossy()
        .to_string()
}
