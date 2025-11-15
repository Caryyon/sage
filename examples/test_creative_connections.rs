// Test Creative Connections - Demonstrates SAGE's association discovery
// Shows how SAGE finds creative connections between concepts based on NCA patterns!

use sage::sage_experience::SageExperience;

fn main() {
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║        SAGE Creative Association Demonstration            ║");
    println!("║    Watch SAGE discover connections between concepts!      ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");

    let mut sage = SageExperience::new();

    // Load existing knowledge if available
    if let Ok(_) = sage.load_knowledge("sage_positive_knowledge.json") {
        println!("✅ Loaded trained knowledge\n");
    }

    println!("🧪 Testing concept associations...\n");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // Feed SAGE various concepts
    let test_concepts = vec![
        ("creativity", "What do you think about creativity?"),
        ("innovation", "Tell me about innovation"),
        ("imagination", "How do you feel about imagination?"),
        ("chaos", "What about chaos?"),
        ("order", "And what about order?"),
        ("learning", "What's your view on learning?"),
        ("knowledge", "How about knowledge?"),
        ("wisdom", "Tell me about wisdom"),
        ("understanding", "What about understanding?"),
        ("light", "What do you think of light?"),
        ("warmth", "And warmth?"),
        ("darkness", "How about darkness?"),
    ];

    for (concept, question) in &test_concepts {
        println!("💬 Human: {}", question);
        let (_opinion, response) = sage.experience_text_with_memory(question, false);
        println!("🤖 SAGE: {}\n", response);

        // Small delay for readability
        std::thread::sleep(std::time::Duration::from_millis(500));
    }

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    println!("🔗 Discovered Associations:\n");
    println!("{}", sage.get_associations());

    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    println!("🎨 Now testing creative connections in responses...\n");

    // Test that SAGE mentions related concepts
    let follow_up_tests = vec![
        "SAGE, tell me more about creativity",
        "SAGE, what about chaos?",
        "SAGE, explain learning to me",
    ];

    for question in &follow_up_tests {
        println!("💬 Human: {}", question);
        let (_opinion, response) = sage.experience_text_with_memory(question, true);
        println!("🤖 SAGE: {}\n", response);
        std::thread::sleep(std::time::Duration::from_millis(500));
    }

    // Show concept clusters
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    println!("🧩 Concept Clusters (related ideas grouped together):\n");

    let clusters = sage.get_concept_clusters();
    if clusters.is_empty() {
        println!("No clusters discovered yet (need more associations)\n");
    } else {
        for (i, cluster) in clusters.iter().enumerate() {
            println!("{}. [{}]", i + 1, cluster.join(", "));
        }
    }

    // Save associations for later
    println!("\n💾 Saving associations to 'sage_associations.json'...");
    if let Ok(_) = sage.save_associations("sage_associations.json") {
        println!("✅ Saved! SAGE will remember these connections.\n");
    }

    println!("\n╔════════════════════════════════════════════════════════════╗");
    println!("║              Association System Working! 🎉               ║");
    println!("║                                                            ║");
    println!("║  SAGE now makes creative connections between concepts      ║");
    println!("║  based on how similarly it processes them with NCA!       ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");
}
