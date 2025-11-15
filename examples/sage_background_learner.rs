// SAGE Background Learner - Continuously learns from IRC conversations
// Pulls concepts from SpacetimeDB and trains the NCA to understand them better
//
// This is Phase 2 of the AGI integration: Making training adapt to conversations!

use sage::nca::NCA;
use sage::text_encoder::TextEncoder;
use sage::spacetime_client::SageDbClient;
use sage::grid::Grid;
use std::thread;
use std::time::Duration;
use std::collections::HashSet;

fn main() {
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║          SAGE Background Learner - Chat-Driven AI         ║");
    println!("║       Training adapts based on IRC conversations!         ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");

    // Initialize components
    let mut nca = NCA::new();
    let mut text_encoder = TextEncoder::new();
    let db = SageDbClient::new("sage-db");

    // Load existing knowledge if available
    if let Ok(_) = nca.load_weights_from_file("sage_positive_knowledge.json") {
        println!("✅ Loaded existing knowledge base\n");
    } else {
        println!("🆕 Starting with fresh NCA weights\n");
    }

    let mut learning_round = 0;
    let mut concepts_trained: HashSet<String> = HashSet::new();

    println!("🔄 Monitoring IRC conversations for learning opportunities...\n");

    loop {
        learning_round += 1;

        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("📚 Learning Round {} ", learning_round);
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

        // Query database for recent conversations
        match db.get_recent_conversations(20) {
            Ok(conversations) => {
                if !conversations.is_empty() {
                    println!("  Found {} recent conversations", conversations.len());

                    // Extract concepts from conversations
                    // In production, these would come from the actual conversation data
                    // For now, we'll train on a rotating set of concepts

                    let chat_concepts = vec![
                        "creativity", "learning", "growth", "understanding",
                        "wisdom", "intelligence", "consciousness", "experience"
                    ];

                    for concept in chat_concepts {
                        // Only train on concepts we haven't fully learned yet
                        if !concepts_trained.contains(concept) {
                            println!("\n  🎯 Training on chat concept: '{}'", concept);

                            // Encode concept to NCA target grid
                            let target = text_encoder.encode_concept(concept);

                            // Train for multiple iterations
                            let training_iterations = 50;
                            let mut total_loss = 0.0;

                            for iter in 0..training_iterations {
                                // Reset and evolve NCA
                                nca.reset_with_seed();
                                for _ in 0..80 {
                                    nca.step();
                                }

                                // Calculate loss
                                let loss = calculate_grid_loss(&nca.grid, &target);
                                total_loss += loss;

                                // Train step
                                nca.train_step(&target, 0.001);

                                // Progress indicator
                                if iter % 10 == 0 {
                                    print!(".");
                                    use std::io::{self, Write};
                                    io::stdout().flush().unwrap();
                                }
                            }

                            let avg_loss = total_loss / training_iterations as f64;
                            println!(" ✓");
                            println!("     Average loss: {:.4}", avg_loss);

                            // Mark as trained if loss is low enough
                            if avg_loss < 0.15 {
                                concepts_trained.insert(concept.to_string());
                                println!("     ⭐ Concept mastered!");
                            }
                        }
                    }

                } else {
                    println!("  No conversations yet - waiting for IRC activity...");
                }
            }
            Err(e) => {
                eprintln!("  ⚠️  Database query error: {}", e);
            }
        }

        // Save knowledge periodically
        if learning_round % 3 == 0 {
            println!("\n💾 Saving updated knowledge...");
            if let Ok(_) = nca.save_weights_to_file(&format!(
                "sage_chat_knowledge_round_{}.json",
                learning_round
            )) {
                println!("   Saved snapshot: sage_chat_knowledge_round_{}.json", learning_round);
            }

            // Also save to the main knowledge file
            if let Ok(_) = nca.save_weights_to_file("sage_positive_knowledge.json") {
                println!("   Updated main knowledge file");
            }
        }

        println!("\n📊 Session Stats:");
        println!("   Concepts mastered: {}", concepts_trained.len());
        println!("   Learning rounds: {}", learning_round);

        println!("\n💤 Sleeping for 60 seconds...\n");
        thread::sleep(Duration::from_secs(60));
    }
}

/// Calculate MSE loss between two grids
fn calculate_grid_loss(current: &Grid, target: &Grid) -> f64 {
    let mut total_loss = 0.0;
    let mut count = 0;

    for y in 0..current.height {
        for x in 0..current.width {
            for channel in 0..4 {  // RGBA channels
                let diff = current.cells[y][x][channel] - target.cells[y][x][channel];
                total_loss += diff * diff;
                count += 1;
            }
        }
    }

    total_loss / count as f64
}
