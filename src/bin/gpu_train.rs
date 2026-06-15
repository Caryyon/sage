//! sage-gpu-train: Train the NCA Language Model on GPU (CUDA)
//!
//! Usage:
//!   sage-gpu-train --curriculum curricula/junior-react-dev.json [--epochs 50] [--grid 64]
//!   sage-gpu-train --demo     Quick demo training
//!   sage-gpu-train --list     List available curricula

use sage::inference::nca_lm_gpu::{train_nca_lm_gpu, save_gpu_model, GpuTrainingConfig};
use std::fs;
use std::path::PathBuf;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let mut curriculum_path: Option<String> = None;
    let mut epochs = 50;
    let mut grid_size = 32;
    let mut nca_steps = 20;
    let mut vocab_size = 4096;
    let mut learning_rate = 0.001;
    let mut max_examples = 1000;
    let mut batch_size = 8;
    let mut demo = false;
    let mut list = false;
    let mut fast = false;

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
                grid_size = args[i].parse().unwrap_or(64);
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
                max_examples = args[i].parse().unwrap_or(1000);
            }
            "--batch" | "-b" => {
                i += 1;
                batch_size = args[i].parse().unwrap_or(8);
            }
            "--demo" => demo = true,
            "--list" => list = true,
            "--fast" => fast = true,
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

    let config = if fast {
        GpuTrainingConfig {
            grid_size: 16,
            nca_steps: 3,
            vocab_size: 1024,
            epochs: 10,
            max_examples: 500,
            batch_size: 4,
            ..GpuTrainingConfig::default()
        }
    } else {
        GpuTrainingConfig {
            grid_size,
            nca_steps,
            vocab_size,
            epochs,
            learning_rate,
            max_examples,
            batch_size,
            ..GpuTrainingConfig::default()
        }
    };

    // Get curriculum
    let curriculum = if demo {
        let demo_path = std::env::temp_dir().join("sage_gpu_demo_curriculum.json");
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

    eprintln!("🧬 GPU NCA Language Model Training");
    eprintln!("   Curriculum: {}", curriculum.display());

    match train_nca_lm_gpu(&curriculum, &config) {
        Ok((varmap, tokenizer, stats)) => {
            eprintln!("\n{}", stats.summary());

            // Save the trained model
            let weights_path = dirs::home_dir()
                .unwrap_or_default()
                .join(".sage")
                .join("nca_lm_gpu_weights.safetensors");
            let vocab_path = dirs::home_dir()
                .unwrap_or_default()
                .join(".sage")
                .join("nca_lm_gpu_vocab.txt");
            let config_path = dirs::home_dir()
                .unwrap_or_default()
                .join(".sage")
                .join("nca_lm_gpu_config.json");

            match save_gpu_model(&varmap, &tokenizer, &config, &weights_path, &vocab_path, &config_path) {
                Ok(()) => {
                    eprintln!("\n💾 Model saved:");
                    eprintln!("   Weights: {}", weights_path.display());
                    eprintln!("   Vocab:   {}", vocab_path.display());
                    eprintln!("   Config:  {}", config_path.display());
                }
                Err(e) => {
                    eprintln!("⚠️  Failed to save model: {}", e);
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
    eprintln!("sage-gpu-train: Train the NCA Language Model on GPU (CUDA)");
    eprintln!();
    eprintln!("USAGE:");
    eprintln!("  sage-gpu-train --curriculum <path> [OPTIONS]");
    eprintln!("  sage-gpu-train --demo     Quick demo training");
    eprintln!("  sage-gpu-train --list     List available curricula");
    eprintln!();
    eprintln!("OPTIONS:");
    eprintln!("  -c, --curriculum <path>  Curriculum JSON file");
    eprintln!("  -e, --epochs <n>         Training epochs (default: 50)");
    eprintln!("  -g, --grid <n>           Grid size (default: 64)");
    eprintln!("  -s, --steps <n>          NCA steps per forward pass (default: 5)");
    eprintln!("  -v, --vocab <n>          Vocabulary size (default: 4096)");
    eprintln!("  --lr <f>                 Learning rate (default: 0.001)");
    eprintln!("  -m, --max-examples <n>   Max training examples (default: 1000)");
    eprintln!("  -b, --batch <n>          Batch size (default: 8)");
    eprintln!("  --fast                   Use fast config (16×16, 1K vocab, 10 epochs)");
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
