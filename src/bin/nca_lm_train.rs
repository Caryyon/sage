//! sage-nca-train: Train the NCA Language Model on a specialist curriculum
//!
//! Usage:
//!   sage-nca-train --curriculum curricula/junior-react-dev.json [--epochs 50] [--grid 32]
//!   sage-nca-train --demo   # Quick demo on built-in corpus
//!   sage-nca-train --list   # List available curricula

use sage::inference::nca_lm::{NcaLanguageModel, NcaLmConfig, NcaLmTrainingConfig};
use sage::inference::nca_lm_trainer::{train_nca_lm, TrainingStats};
use std::fs;
use std::path::PathBuf;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let mut curriculum_path: Option<String> = None;
    let mut epochs = 50;
    let mut grid_size = 32;
    let mut nca_steps = 5;
    let mut vocab_size = 4096;
    let mut learning_rate = 0.001;
    let mut max_examples = 0;
    let mut demo = false;
    let mut list = false;
    let mut fast = false;
    let mut production = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--curriculum" | "-c" => {
                i += 1;
                curriculum_path = Some(args[i].clone());
            }
            "--epochs" | "-e" => {
                i += 1;
                epochs = args[i].parse().unwrap_or(50);
            }
            "--grid" | "-g" => {
                i += 1;
                grid_size = args[i].parse().unwrap_or(32);
            }
            "--steps" | "-s" => {
                i += 1;
                nca_steps = args[i].parse().unwrap_or(5);
            }
            "--vocab" | "-v" => {
                i += 1;
                vocab_size = args[i].parse().unwrap_or(4096);
            }
            "--lr" => {
                i += 1;
                learning_rate = args[i].parse().unwrap_or(0.001);
            }
            "--max-examples" | "-m" => {
                i += 1;
                max_examples = args[i].parse().unwrap_or(0);
            }
            "--demo" => demo = true,
            "--list" => list = true,
            "--fast" => fast = true,
            "--production" | "--prod" => production = true,
            "--help" | "-h" => {
                print_help();
                return;
            }
            _ => {
                eprintln!("Unknown arg: {}", args[i]);
                print_help();
                std::process::exit(1);
            }
        }
        i += 1;
    }

    if list {
        list_curricula();
        return;
    }

    // Determine config
    let model_config = if production {
        NcaLmConfig::production()
    } else if fast {
        NcaLmConfig::fast()
    } else {
        NcaLmConfig {
            grid_size,
            nca_steps,
            vocab_size,
            ..NcaLmConfig::default()
        }
    };

    let training_config = NcaLmTrainingConfig {
        model: model_config,
        learning_rate,
        epochs,
        max_examples,
        ..NcaLmTrainingConfig::default()
    };

    // Get curriculum
    let curriculum = if demo {
        // Use built-in demo corpus
        let demo_path = std::env::temp_dir().join("sage_demo_curriculum.json");
        let demo_json = r#"{
            "name": "demo-assistant",
            "domain": "general-knowledge",
            "topics": [
                {
                    "name": "sage-architecture",
                    "facts": [
                        {"fact": "SAGE uses Neural Cellular Automata for knowledge storage and retrieval"},
                        {"fact": "The NCA grid is 256 by 256 cells with 16 channels per cell"},
                        {"fact": "Knowledge is encoded as activation patterns across the grid"},
                        {"fact": "Cross-attention decoding retrieves relevant knowledge for queries"},
                        {"fact": "The NCA language model generates text using cellular automata dynamics"},
                        {"fact": "SAGE runs entirely locally with no cloud dependencies"},
                        {"fact": "Peer-to-peer gossip protocol enables decentralized knowledge sharing"},
                        {"fact": "The knowledge loop encodes every conversation into the NCA grid"},
                        {"fact": "Retrieval feedback training improves relevance over time"},
                        {"fact": "SAGE specialists are domain-specific AI assistants"}
                    ]
                },
                {
                    "name": "programming-basics",
                    "facts": [
                        {"fact": "Variables store data that can be referenced and manipulated"},
                        {"fact": "Functions are reusable blocks of code that perform specific tasks"},
                        {"fact": "Loops repeat a block of code while a condition is true"},
                        {"fact": "Conditionals execute different code based on boolean expressions"},
                        {"fact": "Arrays store ordered collections of elements"},
                        {"fact": "Objects group related data and functions together"},
                        {"fact": "Recursion is when a function calls itself to solve a problem"},
                        {"fact": "Algorithms are step-by-step procedures for solving problems"},
                        {"fact": "Data structures organize and store data efficiently"},
                        {"fact": "Debugging is the process of finding and fixing errors in code"}
                    ]
                }
            ]
        }"#;
        fs::write(&demo_path, demo_json).expect("Failed to write demo curriculum");
        demo_path
    } else if let Some(ref path) = curriculum_path {
        PathBuf::from(path)
    } else {
        eprintln!("Error: Provide --curriculum <path> or --demo. See --help.");
        std::process::exit(1);
    };

    eprintln!("🧬 NCA Language Model Training");
    eprintln!("   Curriculum: {}", curriculum.display());

    match train_nca_lm(&curriculum, &training_config) {
        Ok((model, stats)) => {
            eprintln!("\n{}", stats.summary());

            // Save the trained model
            match model.save(None, None, None) {
                Ok(()) => {
                    eprintln!("\n💾 Model saved:");
                    eprintln!("   Weights: ~/.sage/nca_lm_weights.bin");
                    eprintln!("   Vocab:   ~/.sage/nca_lm_vocab.json");
                    eprintln!("   Config:  ~/.sage/nca_lm_config.json");
                }
                Err(e) => {
                    eprintln!("⚠️  Failed to save model: {}", e);
                }
            }

            // Quick test generation
            eprintln!("\n🧪 Testing generation...");
            match model.generate_text("What is SAGE?") {
                Ok(response) => {
                    eprintln!("   Prompt: 'What is SAGE?'");
                    eprintln!("   Response: '{}'", response);
                }
                Err(e) => {
                    eprintln!("   Generation failed: {}", e);
                }
            }
        }
        Err(e) => {
            eprintln!("❌ Training failed: {}", e);
            std::process::exit(1);
        }
    }

    // Clean up demo file
    if demo {
        let _ = fs::remove_file(&curriculum);
    }
}

fn print_help() {
    eprintln!("sage-nca-train: Train the NCA Language Model");
    eprintln!();
    eprintln!("USAGE:");
    eprintln!("  sage-nca-train --curriculum <path> [OPTIONS]");
    eprintln!("  sage-nca-train --demo     Quick demo training");
    eprintln!("  sage-nca-train --list     List available curricula");
    eprintln!();
    eprintln!("OPTIONS:");
    eprintln!("  -c, --curriculum <path>  Curriculum JSON file");
    eprintln!("  -e, --epochs <n>         Training epochs (default: 50)");
    eprintln!("  -g, --grid <n>           Grid size (default: 32)");
    eprintln!("  -s, --steps <n>          NCA steps per forward pass (default: 5)");
    eprintln!("  -v, --vocab <n>          Vocabulary size (default: 4096)");
    eprintln!("  --lr <f>                 Learning rate (default: 0.001)");
    eprintln!("  --fast                   Use fast config (16×16, 1K vocab, 3 steps)");
    eprintln!("  --production, --prod     Use production config (64×64, 8K vocab, 8 steps)");
    eprintln!("  --demo                   Train on built-in demo curriculum");
    eprintln!("  --list                   List available curricula");
    eprintln!("  -h, --help               Show this help");
}

fn list_curricula() {
    let curricula_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("curricula");
    if curricula_dir.exists() {
        eprintln!("Available curricula:");
        if let Ok(entries) = fs::read_dir(&curricula_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map_or(false, |e| e == "json") {
                    if let Ok(content) = fs::read_to_string(&path) {
                        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                            let name = json["name"].as_str().unwrap_or("unknown");
                            let domain = json["domain"].as_str().unwrap_or("unknown");
                            let topics = json["topics"].as_array().map(|a| a.len()).unwrap_or(0);
                            eprintln!(
                                "  {} — {} specialist ({} topics, {} domain)",
                                path.file_name().unwrap().to_string_lossy(),
                                name,
                                topics,
                                domain
                            );
                        }
                    }
                }
            }
        }
    } else {
        eprintln!("No curricula directory found at {}", curricula_dir.display());
    }
}
