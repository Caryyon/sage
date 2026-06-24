//! sage-nca-train-text: Train the NCA Language Model on real book text
//!
//! Step 8 of the v0.6.0 plan: Replace demo curriculum with actual corpus text.
//! Uses BPE tokenization for subword coverage (zero <unk> tokens on unseen words).
//!
//! Usage:
//!   sage-nca-train-text --corpus-dir ~/.sage/corpus/ [--bpe] [--epochs 50] [--grid 32]
//!   sage-nca-train-text --text-file path/to/book.txt [--bpe] [--epochs 30]
//!   sage-nca-train-text --corpus-dir ~/.sage/corpus/ --max-files 10 --epochs 20
//!
//! The trained model can then be used for next-token prediction via sage-nca-generate.

use sage::inference::nca_lm::{NcaLmConfig, NcaLmTrainingConfig};
use sage::inference::nca_lm_trainer::{load_corpus_text, train_on_text};
use std::path::PathBuf;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let mut corpus_dir: Option<String> = None;
    let mut text_file: Option<String> = None;
    let mut use_bpe = false;
    let mut epochs = 50;
    let mut grid_size = 32;
    let mut nca_steps = 5;
    let mut vocab_size = 4096;
    let mut learning_rate = 0.001;
    let mut max_examples = 0;
    let mut max_files = 0;
    let mut batch_size = 8;
    let mut eval_interval = 5;
    let mut checkpoint_interval = 0;
    let mut context_window = 64;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--corpus-dir" | "-d" => {
                i += 1;
                corpus_dir = Some(args[i].clone());
            }
            "--text-file" | "-f" => {
                i += 1;
                text_file = Some(args[i].clone());
            }
            "--bpe" | "-b" => use_bpe = true,
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
            "--max-files" => {
                i += 1;
                max_files = args[i].parse().unwrap_or(0);
            }
            "--batch-size" => {
                i += 1;
                batch_size = args[i].parse().unwrap_or(8);
            }
            "--eval-interval" => {
                i += 1;
                eval_interval = args[i].parse().unwrap_or(5);
            }
            "--checkpoint-interval" => {
                i += 1;
                checkpoint_interval = args[i].parse().unwrap_or(0);
            }
            "--context-window" | "-w" => {
                i += 1;
                context_window = args[i].parse().unwrap_or(64);
            }
            "--help" | "-h" => {
                print_help();
                return;
            }
            other => {
                eprintln!("Unknown argument: {}", other);
                print_help();
                std::process::exit(1);
            }
        }
        i += 1;
    }

    // Load corpus text
    let corpus_text = if let Some(dir) = corpus_dir {
        let dir_path = PathBuf::from(&dir);
        match load_corpus_text(&dir_path, max_files) {
            Ok(text) => text,
            Err(e) => {
                eprintln!("❌ Failed to load corpus from {}: {}", dir, e);
                std::process::exit(1);
            }
        }
    } else if let Some(file) = text_file {
        let path = PathBuf::from(&file);
        match std::fs::read_to_string(&path) {
            Ok(text) => {
                eprintln!("📄 Loaded text file: {} ({} chars)", file, text.len());
                text
            }
            Err(e) => {
                eprintln!("❌ Failed to read {}: {}", file, e);
                std::process::exit(1);
            }
        }
    } else {
        eprintln!("❌ Must specify --corpus-dir or --text-file");
        print_help();
        std::process::exit(1);
    };

    // Build training config
    let model_config = NcaLmConfig {
        grid_size,
        nca_steps,
        vocab_size,
        max_tokens: 256,
        temperature: 0.8,
        top_k: 40,
        top_p: 0.9,
        repeat_penalty: 1.1,
        context_window,
    };

    let training_config = NcaLmTrainingConfig {
        model: model_config,
        learning_rate,
        epochs,
        grad_clip: 1.0,
        max_examples,
        lr_decay: true,
        batch_size,
        eval_interval,
        checkpoint_interval,
        checkpoint_dir: None,
    };

    // Train!
    match train_on_text(&corpus_text, use_bpe, &training_config) {
        Ok((model, stats)) => {
            println!("\n{}", stats.summary());

            // Save the trained model
            let weights_path = sage::inference::nca_lm::default_lm_weights_path();
            let vocab_path = sage::inference::nca_lm::default_lm_vocab_path();
            let config_path = sage::inference::nca_lm::default_lm_config_path();

            if let Err(e) = model.save(Some(&weights_path), Some(&vocab_path), Some(&config_path)) {
                eprintln!("⚠️  Failed to save model: {}", e);
            } else {
                eprintln!("💾 Model saved to {}", weights_path.display());
            }
        }
        Err(e) => {
            eprintln!("❌ Training failed: {}", e);
            std::process::exit(1);
        }
    }
}

fn print_help() {
    eprintln!(
        "sage-nca-train-text: Train NCA Language Model on real book text\n\n\
         Usage:\n  \
         sage-nca-train-text --corpus-dir <dir> [options]\n  \
         sage-nca-train-text --text-file <file> [options]\n\n\
         Options:\n  \
         --bpe / -b           Use BPE subword tokenizer (eliminates <unk> tokens)\n  \
         --epochs / -e <n>    Number of training epochs (default: 50)\n  \
         --grid / -g <n>      Grid size (default: 32)\n  \
         --steps / -s <n>     NCA update steps (default: 5)\n  \
         --vocab / -v <n>     Vocabulary size (default: 4096)\n  \
         --lr <f>             Learning rate (default: 0.001)\n  \
         --max-examples / -m <n>  Max training examples (0 = all)\n  \
         --max-files <n>      Max corpus files to load (0 = all)\n  \
         --batch-size <n>     Batch size (default: 8)\n  \
         --eval-interval <n>  Epochs between eval (default: 5)\n  \
         --checkpoint-interval <n>  Epochs between checkpoints (default: 0 = off)\n  \
         --context-window / -w <n>  Context window size (default: 64)\n  \
         --help / -h          Show this help\n"
    );
}