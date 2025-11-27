//! Test Language Learning - Demonstrates SAGE's language capabilities
//!
//! This example trains SAGE to predict characters and generate text.
//!
//! Run with:
//!   cargo run --release --example test_language_learning

use sage::language::{
    CharacterEncoder, CharacterPredictor, LanguageTrainer, TrainingConfig,
};
use std::io::{self, Write};

fn main() {
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║           SAGE Language Learning Test                      ║");
    println!("║     Training SAGE to understand and generate text          ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");

    // Parse command line args
    let args: Vec<String> = std::env::args().collect();
    let mode = args.get(1).map(|s| s.as_str()).unwrap_or("train");

    match mode {
        "train" => run_training(),
        "quick" => run_quick_training(),
        "test" => run_encoder_test(),
        "interactive" => run_interactive(),
        _ => {
            println!("Usage: test_language_learning [mode]");
            println!("Modes:");
            println!("  train       - Full training run");
            println!("  quick       - Quick training (5 epochs)");
            println!("  test        - Test encoder only");
            println!("  interactive - Interactive generation");
        }
    }
}

fn run_training() {
    println!("Starting full training...\n");

    let config = TrainingConfig {
        learning_rate: 0.001,
        epochs_per_level: 50,
        batch_size: 32,
        advance_threshold: 3.0,  // Perplexity threshold
        eval_temperature: 0.8,
        log_interval: 100,
        eval_interval: 10,
    };

    let mut trainer = LanguageTrainer::new(config);
    let metrics = trainer.train();

    println!("\nFinal Results:");
    if let Some(last) = metrics.last() {
        println!("  Final loss: {:.4}", last.loss);
        println!("  Final accuracy: {:.2}%", last.accuracy * 100.0);
        println!("  Final perplexity: {:.2}", last.perplexity);
    }

    // Interactive mode after training
    println!("\n--- Interactive Generation ---");
    println!("Enter a prompt (or 'quit' to exit):\n");

    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();

        if input == "quit" || input == "exit" {
            break;
        }

        let generated = trainer.interactive_generate(input, 30);
        println!("SAGE: {}\n", generated);
    }
}

fn run_quick_training() {
    println!("Running quick training (5 epochs)...\n");

    let config = TrainingConfig {
        learning_rate: 0.001,
        epochs_per_level: 5,
        batch_size: 16,
        advance_threshold: 5.0,  // Easier threshold for quick test
        eval_temperature: 1.0,
        log_interval: 20,
        eval_interval: 1,
    };

    let mut trainer = LanguageTrainer::new(config);

    println!("Level: {} ({})", trainer.corpus.level, trainer.corpus.level_name());
    println!("Training pairs: {}\n", trainer.corpus.get_training_pairs().len());

    for epoch in 1..=5 {
        let metrics = trainer.train_epoch();
        println!(
            "Epoch {}: loss={:.4}, acc={:.2}%, perplexity={:.2}",
            epoch, metrics.loss, metrics.accuracy * 100.0, metrics.perplexity
        );

        // Generate sample
        let sample = trainer.generate_sample();
        println!("  Sample: \"{}\"\n", sample);
    }

    println!("\n✓ Quick training complete!");
}

fn run_encoder_test() {
    println!("Testing character encoder...\n");

    let encoder = CharacterEncoder::new();

    // Test character indexing
    println!("Character index tests:");
    for c in ['a', 'b', 'z', 'A', '0', ' ', '!'] {
        let idx = encoder.char_to_index(c);
        let back = encoder.index_to_char(idx);
        println!("  '{}' -> {} -> '{}'", c, idx, back);
    }

    // Test embedding
    println!("\nEmbedding tests:");
    let embed_a = encoder.get_embedding('a', 0);
    let embed_b = encoder.get_embedding('b', 0);
    let embed_a_pos5 = encoder.get_embedding('a', 5);

    println!("  'a' at pos 0: first 5 values = {:?}", &embed_a[..5]);
    println!("  'b' at pos 0: first 5 values = {:?}", &embed_b[..5]);
    println!("  'a' at pos 5: first 5 values = {:?}", &embed_a_pos5[..5]);

    // Test grid encoding
    println!("\nGrid encoding test:");
    let grid = encoder.encode("hello");

    let mut alive_cells = 0;
    let mut total_activation = 0.0;
    for row in &grid.cells {
        for cell in row {
            if cell[3] > 0.1 {
                alive_cells += 1;
                total_activation += cell[3];
            }
        }
    }

    println!("  Text: \"hello\"");
    println!("  Alive cells: {}", alive_cells);
    println!("  Total activation: {:.2}", total_activation);

    println!("\n✓ Encoder test complete!");
}

fn run_interactive() {
    println!("Interactive mode (untrained model)...\n");
    println!("Note: Without training, output will be random.\n");

    let mut predictor = CharacterPredictor::new();

    loop {
        print!("Enter text (or 'quit'): ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();

        if input == "quit" || input == "exit" {
            break;
        }

        // Process and predict
        predictor.process_input(input);
        let (predicted, confidence) = predictor.predict_next();

        println!("Predicted next char: '{}' (confidence: {:.2}%)", predicted, confidence * 100.0);

        // Generate continuation
        predictor.reset();
        let generated = predictor.generate(input, 20, 1.0);
        println!("Generated: \"{}\"\n", generated);
    }
}
