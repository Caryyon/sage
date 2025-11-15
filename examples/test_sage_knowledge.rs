// Test SAGE with and without trained knowledge
// Shows how learning affects opinion formation

use sage::sage_experience::SageExperience;

fn main() {
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║         SAGE Knowledge Comparison Test                    ║");
    println!("║    Untrained vs Trained NCA Opinion Formation             ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");

    // Test words - mix of trained concepts and new concepts
    let test_words = vec![
        // Trained positive concepts
        "love", "joy", "peace", "hope",
        // New positive concepts (not trained on)
        "faith", "trust", "grace",
        // Negative concepts (opposite of training)
        "hate", "pain", "war",
        // Neutral/gibberish
        "table", "chair", "xqzfvp"
    ];

    println!("{}", "=".repeat(60));
    println!(" TEST 1: UNTRAINED SAGE (random NCA weights)");
    println!("{}\n", "=".repeat(60));

    let mut untrained_sage = SageExperience::new();

    for word in &test_words {
        let (_opinion, response) = untrained_sage.experience_text(word);
        println!("  {:8} → {}", word, response);
    }

    println!("\n{}", untrained_sage.get_personality());
    println!("  Likes: {:?}", untrained_sage.get_likes());
    println!("  Dislikes: {:?}", untrained_sage.get_dislikes());

    println!("\n{}", "=".repeat(60));
    println!(" TEST 2: TRAINED SAGE (learned positive concepts)");
    println!("{}\n", "=".repeat(60));

    let mut trained_sage = SageExperience::new();

    // Load trained knowledge
    let weights_path = "sage_positive_knowledge.json";
    match trained_sage.load_knowledge(weights_path) {
        Ok(_) => println!("✅ Loaded trained knowledge from: {}\n", weights_path),
        Err(e) => {
            println!("⚠️  Could not load knowledge: {}", e);
            println!("   Run: cargo run --example train_sage_knowledge first\n");
            return;
        }
    }

    for word in &test_words {
        let (_opinion, response) = trained_sage.experience_text(word);
        println!("  {:8} → {}", word, response);
    }

    println!("\n{}", trained_sage.get_personality());
    println!("  Likes: {:?}", trained_sage.get_likes());
    println!("  Dislikes: {:?}", trained_sage.get_dislikes());

    println!("\n{}", "=".repeat(60));
    println!(" ANALYSIS");
    println!("{}\n", "=".repeat(60));

    println!("Expected behavior:");
    println!("  • Untrained: Random opinions, inconsistent");
    println!("  • Trained: Should LIKE trained concepts (love, joy, peace)");
    println!("            Should be NEUTRAL/DISLIKE untrained/opposite concepts");
    println!("            Based on pattern similarity to learned knowledge\n");

    println!("💡 Key Insight:");
    println!("   Training the NCA creates a \"knowledge base\" that shapes");
    println!("   SAGE's opinions. Lower loss = concept aligns with knowledge");
    println!("   = SAGE likes it. Higher loss = conflicts = SAGE dislikes it.\n");
}
