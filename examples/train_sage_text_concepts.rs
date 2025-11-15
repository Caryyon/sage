// Train SAGE specifically on text concepts for IRC conversations
// This gives SAGE baseline knowledge of positive concepts

use sage::nca::NCA;
use sage::text_encoder::TextEncoder;
use sage::grid::Grid;

fn main() {
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║        SAGE Text Concept Training                         ║");
    println!("║    Teaching SAGE to understand positive concepts          ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");

    let mut nca = NCA::new();
    let mut text_encoder = TextEncoder::new();

    // Positive concepts SAGE should understand well (low loss)
    let positive_concepts = vec![
        "love", "joy", "peace", "harmony", "beauty",
        "truth", "wisdom", "kindness", "compassion", "courage",
        "creativity", "innovation", "learning", "growth", "understanding",
        "hope", "faith", "trust", "friendship", "connection",
        "music", "art", "poetry", "dance", "song",
        "light", "warmth", "comfort", "safety", "home",
        "hello", "hi", "greetings", "welcome", "thanks",
    ];

    let total_concepts = positive_concepts.len();
    let iterations_per_concept = 100;
    let learning_rate = 0.001;

    println!("Training on {} positive concepts...\n", total_concepts);

    for (idx, concept) in positive_concepts.iter().enumerate() {
        println!("[{}/{}] Training on '{}'", idx + 1, total_concepts, concept);

        // Encode concept to grid
        let target = text_encoder.encode_concept(concept);

        // Train for multiple iterations
        for iter in 0..iterations_per_concept {
            // Reset and evolve NCA
            nca.reset_with_seed();
            for _ in 0..80 {
                nca.step();
            }

            // Calculate loss
            let loss = calculate_grid_loss(&nca.grid, &target);

            // Train step (gradient descent)
            nca.train_step(&target, learning_rate);

            // Print progress every 20 iterations
            if iter % 20 == 0 {
                println!("  Iteration {}/{}: loss = {:.4}", iter, iterations_per_concept, loss);
            }
        }

        // Final test
        nca.reset_with_seed();
        for _ in 0..80 {
            nca.step();
        }
        let final_loss = calculate_grid_loss(&nca.grid, &target);
        println!("  ✓ Final loss: {:.4}\n", final_loss);
    }

    // Save trained weights
    println!("💾 Saving trained knowledge to 'sage_positive_knowledge.json'...");
    match nca.save_weights_to_file("sage_positive_knowledge.json") {
        Ok(_) => println!("✅ Training complete! SAGE now understands positive concepts."),
        Err(e) => eprintln!("❌ Error saving: {}", e),
    }

    println!("\n🎉 SAGE is ready to chat! Run the IRC bot now:");
    println!("   cargo run --release --example sage_irc_bot");
}

/// Calculate MSE loss between two grids
fn calculate_grid_loss(current: &Grid, target: &Grid) -> f64 {
    let mut total_loss = 0.0;
    let mut count = 0;

    for y in 0..current.height {
        for x in 0..current.width {
            for channel in 0..4 {
                let diff = current.cells[y][x][channel] - target.cells[y][x][channel];
                total_loss += diff * diff;
                count += 1;
            }
        }
    }

    total_loss / count as f64
}
