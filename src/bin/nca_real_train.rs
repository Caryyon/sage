//! NCA training on real text - optimized 8x8 config
use sage::inference::backprop_trainer::{train_nca_backprop, BackpropConfig};
use sage::inference::nca_predictor::{NcaPredictor, SimpleTokenizer};
use std::fs;

fn main() {
    let corpus = fs::read_to_string("data/training/frankenstein.txt")
        .unwrap_or_else(|_| "the quick brown fox jumps over the lazy dog".to_string());

    let sample: String = corpus.chars().take(200_000).collect();
    println!("Corpus: {} chars", sample.len());

    let grid_size = 8;
    let config = BackpropConfig {
        learning_rate: 0.01,
        epochs: 20,
        grad_clip: 1.0,
        nca_steps: 3,
        grid_size,
        context_window: 4,
        max_examples: 200,
        lr_decay: true,
    };

    println!("Training NCA with backprop (Adam)...");
    println!("  Grid: {}×{}, Steps: {}, Epochs: {}, Examples: {}",
        config.grid_size, config.grid_size, config.nca_steps, config.epochs, config.max_examples);

    let start = std::time::Instant::now();
    let (trained_predictor, accuracy, random_accuracy) =
        train_nca_backprop(&sample, &config, true).expect("Training should succeed");
    let elapsed = start.elapsed();

    println!();
    println!("Training complete in {:.1}s", elapsed.as_secs_f64());
    println!("Accuracy: {:.1}% (random baseline: {:.1}%)",
        accuracy * 100.0, random_accuracy * 100.0);
    println!("Improvement: {:.1}× over random", accuracy / random_accuracy.max(0.001));

    // Save weights
    let weights = trained_predictor.weights();
    let save_path = std::path::PathBuf::from(
        std::env::var("HOME").unwrap_or_default()
    ).join(".sage").join("nca_lm_weights.bin");
    weights.save(&save_path).expect("Failed to save weights");
    println!("💾 Weights saved to {}", save_path.display());

    // Test generation
    let tokenizer = SimpleTokenizer::from_corpus(&sample, grid_size * grid_size);
    let mut predictor = NcaPredictor::with_grid_size(
        tokenizer,
        weights.clone(),
        3,
        grid_size,
    );

    println!("\n═══ NCA Generation Test ═══");
    let queries = [
        "the creature",
        "my dear",
        "i felt",
        "the night",
        "my father",
        "death and",
        "the monster",
        "i saw",
    ];
    for query in &queries {
        match predictor.answer(query, None, 10) {
            Ok(response) => println!("  \"{}\" → \"{}\"", query, response),
            Err(e) => println!("  \"{}\" → ERROR: {}", query, e),
        }
    }
    println!("═══════════════════════════");
}
