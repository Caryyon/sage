// Train SAGE on specific concepts to build a knowledge base
// This creates persistent, consistent preferences based on learned patterns

use sage::nca::NCA;
use sage::text_encoder::TextEncoder;
use sage::grid::Grid;

fn main() {
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║           SAGE Knowledge Training System                  ║");
    println!("║     Building SAGE's understanding of positive concepts    ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");

    let positive_concepts = vec![
        "love", "joy", "peace", "hope", "kind",
        "warm", "light", "good", "calm", "safe",
        "happy", "smile", "beauty", "truth", "wisdom"
    ];

    println!("🎓 Training NCA on {} positive concepts...\n", positive_concepts.len());

    let mut nca = NCA::new();
    let mut encoder = TextEncoder::new();

    // Training parameters
    let iterations_per_concept = 200;
    let evolution_steps = 80;

    for (i, &concept) in positive_concepts.iter().enumerate() {
        print!("   [{:2}/{}] Training on '{:8}' ", i + 1, positive_concepts.len(), concept);
        use std::io::{self, Write};
        io::stdout().flush().unwrap();

        // Encode concept to grid
        let target = encoder.encode_text(concept);

        let mut total_loss = 0.0;

        // Train on this concept multiple times
        for iter in 0..iterations_per_concept {
            nca.reset_with_seed();

            // Evolve NCA toward target
            for _ in 0..evolution_steps {
                nca.step();
            }

            // Calculate loss
            let loss = calculate_loss(&nca.grid, &target);
            total_loss += loss;

            // Train the NCA (gradient descent)
            nca.train_step(&target, 0.001);

            if iter % 50 == 0 {
                print!(".");
                io::stdout().flush().unwrap();
            }
        }

        let avg_loss = total_loss / iterations_per_concept as f64;
        println!(" ✓ (avg loss: {:.4})", avg_loss);
    }

    // Save trained weights
    let weights_path = "sage_positive_knowledge.json";
    match nca.save_weights_to_file(weights_path) {
        Ok(_) => println!("\n💾 Saved trained weights to: {}", weights_path),
        Err(e) => println!("\n⚠️  Error saving weights: {}", e),
    }

    println!("\n{}", "=".repeat(60));
    println!("\n✅ Training complete!");
    println!("\n📚 SAGE now has knowledge of:");
    for concept in &positive_concepts {
        print!("   {} ", concept);
    }
    println!("\n");

    println!("💡 Next steps:");
    println!("   1. Run: cargo run --example test_sage_knowledge");
    println!("   2. Watch SAGE form opinions based on learned patterns");
    println!("   3. Compare to untrained SAGE's random opinions\n");
}

fn calculate_loss(current: &Grid, target: &Grid) -> f64 {
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
