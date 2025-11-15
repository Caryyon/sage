// Demonstrate hierarchical concept learning

use sage::learning::{
    phase_config::load_default_phases,
    feature_extractor::FeatureExtractor,
    hierarchical_concepts::HierarchicalLearner,
};

fn main() {
    println!("=== Hierarchical Concept Learning ===\n");

    let phases = load_default_phases();
    let feature_extractor = FeatureExtractor::new();

    // Create hierarchical learner (3 levels, threshold of 10 observations)
    let mut learner = HierarchicalLearner::new(3, 10);

    println!("Observing patterns from {} phases...\n", phases.len());

    // Observe patterns from each phase
    let mut total_observations = 0;

    for phase_idx in 0..phases.len() {
        println!("Phase {}: {}", phase_idx + 1, phases[phase_idx].name);

        for step in 0..50 {
            let progress = (step as f64 / 50.0) * 100.0;
            let grid = phases[phase_idx].generate_pattern(step, progress);

            // Detect patterns
            let patterns = feature_extractor.detect_patterns(&grid, 0.5);

            // Learn from this observation
            learner.observe(patterns.clone());
            total_observations += 1;
        }

        // Show stats after each phase
        let stats = learner.get_stats();
        println!("  Level 0 (primitives): {} concepts", stats.level_counts[0]);
        println!("  Level 1 (composites): {} concepts", stats.level_counts.get(1).unwrap_or(&0));
        println!("  Level 2 (abstractions): {} concepts", stats.level_counts.get(2).unwrap_or(&0));
        println!();
    }

    // Final statistics
    let stats = learner.get_stats();
    println!("=== Final Hierarchy Statistics ===");
    println!("Total observations: {}", total_observations);
    println!("Total concepts discovered: {}", stats.total_concepts);
    for (level, count) in stats.level_counts.iter().enumerate() {
        println!("  Level {}: {} concepts", level, count);
    }

    // Show learned composite concepts
    println!("\n=== Learned Composite Concepts (sorted by observations) ===");
    let learned = learner.get_learned_concepts();

    if learned.is_empty() {
        println!("No composite concepts learned yet (need more observations)");
    } else {
        for (i, concept) in learned.iter().take(15).enumerate() {
            println!("{}. {} (Level {})", i + 1, concept.name, concept.level);
            println!("   Components: [{}]", concept.components.join(" + "));
            println!("   Observations: {}, Confidence: {:.2}", concept.observations, concept.confidence);
        }
    }

    println!("\n=== Hierarchical Learning Complete ===");
    println!("The system has learned to:");
    println!("  - Recognize primitive patterns (level 0)");
    println!("  - Discover pattern combinations (level 1)");
    println!("  - Abstract higher-level concepts (level 2+)");
    println!("  - Build a concept hierarchy from observations");
}
