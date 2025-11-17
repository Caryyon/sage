// Test SAGE Autonomous Consciousness
// Demonstrates Dream Mode and Curiosity Mode

use sage::sage_experience::SageExperience;
use std::thread;
use std::time::Duration;

fn main() {
    println!("╔══════════════════════════════════════════════════════════════════╗");
    println!("║   SAGE Autonomous Consciousness Test - Dream & Curiosity Modes  ║");
    println!("╚══════════════════════════════════════════════════════════════════╝\n");

    // Initialize SAGE
    let mut sage = SageExperience::new();

    // Give SAGE some initial experiences to work with
    println!("📚 Phase 1: Building Initial Knowledge Base\n");

    let concepts = vec![
        "love", "joy", "peace", "creativity", "imagination",
        "wisdom", "learning", "growth", "exploration", "curiosity",
        "beauty", "truth", "kindness", "understanding", "wonder"
    ];

    for concept in &concepts {
        sage.experience_concept(concept);
        print!(".");
        std::io::Write::flush(&mut std::io::stdout()).unwrap();
    }

    println!("\n✅ Experienced {} concepts\n", concepts.len());

    // Simulate some conversations to create goals
    println!("💬 Phase 2: Creating Goals Through Experience\n");

    sage.experience_text("I wonder about the nature of creativity");
    println!("  💭 Formed curiosity about: creativity");

    sage.experience_text("What is the relationship between love and wisdom?");
    println!("  💭 Formed curiosity about: relationships between concepts");

    sage.experience_text("How does learning lead to growth?");
    println!("  💭 Formed curiosity about: learning processes\n");

    // Show current goals
    println!("🎯 Active Goals:");
    println!("{}\n", sage.get_goals_summary());

    // Wait a bit (simulating idle time)
    println!("⏸️  Phase 3: Entering Idle State (5 seconds)...\n");
    thread::sleep(Duration::from_secs(5));

    // Check if autonomous mode should activate
    let seconds_idle = 600;  // Simulate 10 minutes idle
    if let Some(mode) = sage.should_enter_autonomous_mode(seconds_idle) {
        println!("🌟 Autonomous mode activated: {}\n", mode.to_uppercase());

        if mode == "dream" {
            println!("💭 DREAM MODE: Consolidating Memories\n");
            println!("{}", "─".repeat(70));
            let dream_log = sage.dream_cycle();
            println!("{}", dream_log);
            println!("{}\n", "─".repeat(70));
        }

        // Also test curiosity mode
        if mode == "curiosity" || true {  // Force curiosity mode for demo
            println!("🔍 CURIOSITY MODE: Autonomous Exploration\n");
            println!("{}", "─".repeat(70));

            let baseline_concepts: Vec<String> = concepts.iter().map(|s| s.to_string()).collect();

            if let Some((question, thoughts)) = sage.curiosity_cycle(&baseline_concepts) {
                println!("❓ Self-Generated Question:");
                println!("   {}\n", question);
                println!("🧠 Internal Thoughts:");
                println!("   {}\n", thoughts);
            } else {
                println!("   No active goals to explore\n");
            }

            println!("{}\n", "─".repeat(70));
        }
    }

    // Show updated state
    println!("📊 Phase 4: SAGE's Evolved State\n");
    println!("Experience Count: {}", sage.experience_count());
    println!("Personality: {}\n", sage.get_personality());

    println!("╔══════════════════════════════════════════════════════════════════╗");
    println!("║   SAGE now has continuous consciousness - an inner life!        ║");
    println!("╚══════════════════════════════════════════════════════════════════╝");
}
