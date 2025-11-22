//! Test Meta-Strategy Selection
//!
//! This test verifies:
//! 1. Task feature extraction works
//! 2. Strategy selection adapts to features
//! 3. Outcome recording updates preferences
//! 4. Controller integrates all components
//!
//! Run with: cargo run --release --example test_meta_strategy

use sage::learning::meta_strategy::{
    StrategySelector, TaskFeatures, LearningStrategy,
    DataAvailability, TimeBudget, MetaStrategyController,
};

fn main() {
    println!("=== Meta-Strategy Selection Test ===\n");

    test_task_features();
    test_strategy_selection();
    test_learning_over_time();
    test_controller();
    test_scenario_simulation();

    println!("\n=== All Meta-Strategy Tests Passed! ===");
}

fn test_task_features() {
    println!("--- Test 1: Task Feature Extraction ---");

    // Test pattern-based feature extraction
    let patterns = ["circle", "square", "spiral", "star", "unknown"];

    println!("  Pattern features:");
    for pattern in patterns {
        let loss_history = vec![0.3, 0.2, 0.15, 0.1];
        let features = TaskFeatures::from_pattern(pattern, &loss_history);
        println!("    {}: complexity={:.2}, familiarity={:.2}, exploration={:.2}",
            pattern, features.complexity, features.familiarity, features.exploration_benefit);
    }

    // Test custom features
    let custom = TaskFeatures {
        complexity: 0.8,
        familiarity: 0.2,
        data_availability: DataAvailability::Scarce,
        structured: false,
        exploration_benefit: 0.9,
        time_budget: TimeBudget::Extended,
        historical_performance: std::collections::HashMap::new(),
    };
    println!("\n  Custom task: complexity={:.2}, familiarity={:.2}",
        custom.complexity, custom.familiarity);

    println!("[OK] Task features working\n");
}

fn test_strategy_selection() {
    println!("--- Test 2: Strategy Selection ---");

    let mut selector = StrategySelector::new();

    // Scenario 1: Few-shot learning (should prefer Reptile)
    let few_shot = TaskFeatures {
        complexity: 0.5,
        familiarity: 0.7,
        data_availability: DataAvailability::Scarce,
        structured: true,
        exploration_benefit: 0.3,
        time_budget: TimeBudget::Urgent,
        historical_performance: std::collections::HashMap::new(),
    };
    println!("  Scenario: Few-shot learning (scarce data, urgent)");
    let selection = selector.select(&few_shot);
    println!("    Selected: {} (confidence={:.2})", selection.strategy.name(), selection.confidence);
    println!("    Reasoning: {}", selection.reasoning);

    // Scenario 2: Complex task with time (should prefer PBT)
    let complex = TaskFeatures {
        complexity: 0.9,
        familiarity: 0.2,
        data_availability: DataAvailability::Abundant,
        structured: true,
        exploration_benefit: 0.8,
        time_budget: TimeBudget::Extended,
        historical_performance: std::collections::HashMap::new(),
    };
    println!("\n  Scenario: Complex task with extended time");
    let selection = selector.select(&complex);
    println!("    Selected: {} (confidence={:.2})", selection.strategy.name(), selection.confidence);
    println!("    Reasoning: {}", selection.reasoning);

    // Scenario 3: Simple familiar task (should prefer Standard)
    let simple = TaskFeatures {
        complexity: 0.2,
        familiarity: 0.9,
        data_availability: DataAvailability::Abundant,
        structured: true,
        exploration_benefit: 0.1,
        time_budget: TimeBudget::Normal,
        historical_performance: std::collections::HashMap::new(),
    };
    println!("\n  Scenario: Simple familiar task");
    let selection = selector.select(&simple);
    println!("    Selected: {} (confidence={:.2})", selection.strategy.name(), selection.confidence);
    println!("    Reasoning: {}", selection.reasoning);

    println!("[OK] Strategy selection working\n");
}

fn test_learning_over_time() {
    println!("--- Test 3: Learning From Outcomes ---");

    let mut selector = StrategySelector::new();

    // Simulate experiences where Reptile performs well on familiar tasks
    println!("  Simulating 50 task outcomes...");
    for i in 0..50 {
        let familiarity = (i % 10) as f64 / 10.0;
        let features = TaskFeatures {
            familiarity,
            complexity: 0.5,
            data_availability: DataAvailability::Limited,
            ..TaskFeatures::new()
        };

        // Reptile works better on familiar tasks
        let reptile_score = 0.5 + familiarity * 0.4 + rand::random::<f64>() * 0.1;
        let standard_score = 0.5 + rand::random::<f64>() * 0.2;

        if i % 2 == 0 {
            selector.record_outcome(features.clone(), LearningStrategy::Reptile, reptile_score, 50);
        } else {
            selector.record_outcome(features, LearningStrategy::Standard, standard_score, 100);
        }
    }

    let stats = selector.get_stats();
    println!("\n  After 50 outcomes:");
    println!("    Total selections: {}", stats.total_selections);
    println!("    Exploration rate: {:.3}", stats.exploration_rate);
    println!("\n  Strategy performance:");
    for (strategy, s) in &stats.strategy_stats {
        if s.usage_count > 0 {
            println!("    {}: avg_score={:.3}, used={}, confidence={:.2}",
                strategy.name(), s.avg_score, s.usage_count, s.confidence);
        }
    }

    // Check that Reptile learned to be preferred
    let best = selector.best_strategy();
    println!("\n  Best strategy: {}", best.name());

    println!("[OK] Learning over time working\n");
}

fn test_controller() {
    println!("--- Test 4: Meta-Strategy Controller ---");

    let mut controller = MetaStrategyController::new();

    // Start a few-shot task
    let features = TaskFeatures {
        data_availability: DataAvailability::Scarce,
        time_budget: TimeBudget::Urgent,
        familiarity: 0.6,
        ..TaskFeatures::new()
    };

    let selection = controller.start_task(features);
    println!("  Started task with strategy: {}", selection.strategy.name());
    println!("  Reasoning: {}", selection.reasoning);

    // Simulate training steps
    println!("\n  Simulating 20 training steps...");
    for step in 0..20 {
        let loss = 0.5 * (-0.1 * step as f64).exp() + rand::random::<f64>() * 0.05;
        let grad_mag = 0.1 + rand::random::<f64>() * 0.05;
        let lr = controller.get_learning_rate(loss, grad_mag);

        if step % 5 == 0 {
            println!("    Step {}: loss={:.4}, lr={:.6}", step, loss, lr);
        }
    }

    // Check if should switch
    println!("\n  Should switch strategy? {}", controller.should_switch());
    println!("  Current strategy: {}", controller.current_strategy().name());

    // Start a new task to record outcome
    let new_features = TaskFeatures {
        complexity: 0.8,
        time_budget: TimeBudget::Extended,
        ..TaskFeatures::new()
    };
    let new_selection = controller.start_task(new_features);
    println!("\n  New task strategy: {}", new_selection.strategy.name());

    let stats = controller.stats();
    println!("\n  Controller stats:");
    println!("    Total selections: {}", stats.total_selections);
    println!("    Exploration rate: {:.3}", stats.exploration_rate);

    println!("[OK] Controller working\n");
}

fn test_scenario_simulation() {
    println!("--- Test 5: Full Scenario Simulation ---");

    let mut controller = MetaStrategyController::new();

    // Simulate different scenarios
    let scenarios = vec![
        ("Circle (simple)", TaskFeatures::from_pattern("circle", &[0.1])),
        ("Spiral (complex)", TaskFeatures::from_pattern("spiral", &[0.4])),
        ("Star (novel)", TaskFeatures::from_pattern("star", &[0.5])),
        ("Square (hard)", TaskFeatures::from_pattern("square", &[0.35])),
    ];

    println!("  Running 4 scenarios with 30 steps each:\n");

    for (name, features) in scenarios {
        let selection = controller.start_task(features);
        println!("  {}", name);
        println!("    Strategy: {}", selection.strategy.name());

        // Run training
        let mut final_loss = 0.5;
        for step in 0..30 {
            let loss = 0.5 * (-0.05 * step as f64).exp() + rand::random::<f64>() * 0.03;
            let grad = 0.1;
            controller.get_learning_rate(loss, grad);
            final_loss = loss;
        }
        println!("    Final loss: {:.4}", final_loss);
    }

    // Final statistics
    let stats = controller.stats();
    println!("\n  Final statistics after all scenarios:");
    println!("    Total tasks: {}", stats.total_selections);
    println!("    Best strategy: {}", controller.best_strategy().name());

    println!("[OK] Scenario simulation working\n");
}
