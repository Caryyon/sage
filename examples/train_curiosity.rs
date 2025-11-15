// Demonstrate curiosity-driven exploration and learning

use sage::learning::{
    phase_config::load_default_phases,
    feature_extractor::FeatureExtractor,
    curiosity::{CuriositySystem, ExplorationStrategy},
};

fn main() {
    println!("=== Curiosity-Driven Exploration ===\n");

    let phases = load_default_phases();
    let feature_extractor = FeatureExtractor::new();
    let feature_size = feature_extractor.feature_count();

    let mut curiosity = CuriositySystem::new(feature_size);
    curiosity.set_learning_rate(0.01);

    println!("Initializing curiosity system with {} features", feature_size);
    println!("Strategy: {:?}\n", ExplorationStrategy::Balanced);

    // Simulate exploration across different phases
    println!("=== Exploration Phase 1: Initial Discovery ===");
    let mut all_grids = Vec::new();

    for phase_idx in 0..phases.len() {
        println!("\nExploring Phase {}: {}", phase_idx + 1, phases[phase_idx].name);

        let mut phase_grids = Vec::new();
        let mut phase_interest_scores = Vec::new();

        for step in 0..20 {
            let progress = (step as f64 / 20.0) * 100.0;
            let grid = phases[phase_idx].generate_pattern(step, progress);

            // Evaluate how interesting this observation is
            let interest = curiosity.evaluate_interest(&grid, step);
            phase_interest_scores.push(interest);

            phase_grids.push(grid.clone());
            all_grids.push(grid);
        }

        let avg_interest: f64 = phase_interest_scores.iter().sum::<f64>() / phase_interest_scores.len() as f64;
        println!("  Average interest score: {:.3}", avg_interest);
    }

    // Show initial stats
    let stats = curiosity.get_stats();
    println!("\n=== Initial Exploration Stats ===");
    println!("Total explorations: {}", stats.total_explorations);
    println!("Total intrinsic reward: {:.2}", stats.total_intrinsic_reward);
    println!("Average novelty: {:.3}", stats.average_novelty);
    println!("Average intrinsic reward: {:.3}", stats.average_intrinsic_reward);

    // Learn from observed sequences
    println!("\n=== Learning Phase: Pattern Prediction ===");
    let mut learning_loss = 0.0;
    let mut sequence_count = 0;

    for phase_idx in 0..phases.len() {
        let mut sequence = Vec::new();
        for step in 0..20 {
            let progress = (step as f64 / 20.0) * 100.0;
            sequence.push(phases[phase_idx].generate_pattern(step, progress));
        }

        let loss = curiosity.learn_from_sequence(&sequence);
        learning_loss += loss;
        sequence_count += 1;

        if phase_idx % 2 == 0 {
            println!("Phase {} learning loss: {:.6}", phase_idx + 1, loss);
        }
    }

    let avg_loss = learning_loss / sequence_count as f64;
    println!("\nAverage learning loss: {:.6}", avg_loss);

    // Re-explore with learned model
    println!("\n=== Exploration Phase 2: After Learning ===");
    let mut post_learning_interest = Vec::new();

    for phase_idx in 0..phases.len() {
        for step in (0..20).step_by(5) {
            let progress = (step as f64 / 20.0) * 100.0;
            let grid = phases[phase_idx].generate_pattern(step, progress);
            let interest = curiosity.evaluate_interest(&grid, 100 + step);
            post_learning_interest.push(interest);
        }
    }

    let avg_post_interest: f64 = post_learning_interest.iter().sum::<f64>() / post_learning_interest.len() as f64;
    println!("Average interest after learning: {:.3}", avg_post_interest);

    // Test different exploration strategies
    println!("\n=== Testing Exploration Strategies ===");

    let strategies = [
        ExplorationStrategy::NoveltyDriven,
        ExplorationStrategy::UncertaintyDriven,
        ExplorationStrategy::Balanced,
    ];

    for strategy in &strategies {
        curiosity.set_strategy(*strategy);

        let mut strategy_rewards = Vec::new();
        for phase_idx in 0..3 {
            for step in 0..10 {
                let progress = (step as f64 / 10.0) * 100.0;
                let grid = phases[phase_idx].generate_pattern(step, progress);
                let interest = curiosity.evaluate_interest(&grid, 200 + step);
                strategy_rewards.push(interest);
            }
        }

        let avg_reward: f64 = strategy_rewards.iter().sum::<f64>() / strategy_rewards.len() as f64;
        println!("{:?}: Average reward = {:.3}", strategy, avg_reward);
    }

    // Get suggestion for next strategy
    if let Some(suggested) = curiosity.suggest_exploration_adjustment() {
        println!("\nSuggested strategy based on progress: {:?}", suggested);
    }

    // Final stats
    let final_stats = curiosity.get_stats();
    println!("\n=== Final Curiosity Stats ===");
    println!("Total explorations: {}", final_stats.total_explorations);
    println!("Total intrinsic reward: {:.2}", final_stats.total_intrinsic_reward);
    println!("Average novelty (last 50): {:.3}", final_stats.average_novelty);
    println!("Average intrinsic reward (last 50): {:.3}", final_stats.average_intrinsic_reward);

    println!("\n=== Curiosity System Ready ===");
    println!("The system can now:");
    println!("  - Detect novel patterns automatically");
    println!("  - Calculate intrinsic motivation for learning");
    println!("  - Adapt exploration strategy based on progress");
    println!("  - Learn to predict future observations");
}
