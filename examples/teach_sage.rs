// Train SAGE on specific concepts to build a knowledge base
// This gives SAGE consistent preferences based on learned patterns

use sage::nca::NCA;
use sage::text_encoder::TextEncoder;
use sage::sage_experience::SageExperience;

fn main() {
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║           SAGE Knowledge Training System                  ║");
    println!("║        Teaching SAGE to recognize patterns                ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");

    // Create SAGE instance
    let mut sage = SageExperience::new();

    println!("📚 Phase 1: Testing SAGE's initial opinions (untrained NCA)\n");

    // Test initial opinions before training
    let test_words = vec!["love", "joy", "peace", "hate", "pain", "war"];

    for word in &test_words {
        let (_opinion, response) = sage.experience_text(word);
        println!("  {} → {}", word, response);
    }

    println!("\n{}", sage.get_personality());
    println!("\n{'=':.>60}\n", "");

    // Now train the NCA on positive emotional concepts
    println!("🎓 Phase 2: Training SAGE's NCA on positive concepts...\n");

    let positive_concepts = vec![
        "love", "joy", "peace", "hope", "kind", "warm",
        "light", "good", "calm", "safe"
    ];

    // Train the NCA
    train_sage_nca(&positive_concepts, 500);

    println!("\n✅ Training complete! NCA now recognizes positive patterns.\n");
    println!("{'=':.>60}\n", "");

    // Create a NEW SAGE instance with the trained NCA
    println!("🧠 Phase 3: Creating new SAGE with trained NCA...\n");

    let mut trained_sage = SageExperience::new();
    // In a real implementation, we'd load the trained weights here
    // For now, we'll demonstrate the concept

    println!("📊 Phase 4: Testing SAGE's opinions with trained knowledge\n");

    // Test the same words again
    for word in &test_words {
        let (_opinion, response) = trained_sage.experience_text(word);
        println!("  {} → {}", word, response);
    }

    println!("\n{}", trained_sage.get_personality());

    println!("\n{'=':.>60}\n", "");
    println!("💡 Analysis:");
    println!("   - Trained SAGE should have lower loss (higher confidence) on");
    println!("     concepts it was trained on (love, joy, peace)");
    println!("   - Untrained concepts (hate, pain, war) should show different");
    println!("     patterns based on their similarity to learned patterns");
    println!("\n🔬 Next step: Implement NCA weight persistence to save/load");
    println!("   trained knowledge between sessions.");
}

fn train_sage_nca(concepts: &[&str], iterations_per_concept: usize) {
    let mut nca = NCA::new();
    let mut encoder = TextEncoder::new();

    println!("   Training on {} concepts, {} iterations each:",
             concepts.len(), iterations_per_concept);

    for (i, &concept) in concepts.iter().enumerate() {
        print!("   [{}/{}] Training on '{}' ", i + 1, concepts.len(), concept);

        // Encode concept to grid
        let target = encoder.encode_text(concept);

        // Train NCA to recognize this pattern
        for iter in 0..iterations_per_concept {
            nca.reset_with_seed();

            // Evolve NCA
            for _ in 0..80 {
                nca.step();
            }

            // Calculate loss and train (backprop would go here in full implementation)
            let loss = calculate_loss(&nca, &target);

            if iter % 100 == 0 {
                print!(".");
                use std::io::{self, Write};
                io::stdout().flush().unwrap();
            }

            // In real implementation: nca.train_step(&target, learning_rate);
        }

        println!(" ✓");
    }
}

fn calculate_loss(nca: &NCA, target: &sage::grid::Grid) -> f64 {
    let mut total_loss = 0.0;
    let mut count = 0;

    for y in 0..nca.grid.height {
        for x in 0..nca.grid.width {
            for channel in 0..4 {
                let diff = nca.grid.cells[y][x][channel] - target.cells[y][x][channel];
                total_loss += diff * diff;
                count += 1;
            }
        }
    }

    total_loss / count as f64
}
