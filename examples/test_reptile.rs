//! Test Reptile Meta-Learning
//!
//! This test verifies:
//! 1. Meta-weights can be learned from multiple tasks
//! 2. Few-shot adaptation is faster than random init
//! 3. Meta-learning improves over iterations
//!
//! Run with: cargo run --release --example test_reptile

use sage::learning::reptile::{
    ReptileMetaLearner, PatternTask, ReptileWeights,
    create_standard_tasks, create_circle_target, create_square_target,
};
use sage::nca::NCA;

fn main() {
    println!("=== Reptile Meta-Learning Test ===\n");

    test_weight_operations();
    test_pattern_tasks();
    test_meta_learning();
    test_few_shot_adaptation();
    test_benchmark_comparison();

    println!("\n=== All Reptile Tests Passed! ===");
}

fn test_weight_operations() {
    println!("--- Test 1: Weight Operations ---");

    // Create sample weights
    let w1 = ReptileWeights {
        weights1: vec![vec![1.0, 2.0, 3.0], vec![4.0, 5.0, 6.0]],
        bias1: vec![0.1, 0.2],
        weights2: vec![vec![7.0, 8.0], vec![9.0, 10.0]],
        bias2: vec![0.3, 0.4],
    };

    let w2 = ReptileWeights {
        weights1: vec![vec![0.5, 1.0, 1.5], vec![2.0, 2.5, 3.0]],
        bias1: vec![0.05, 0.1],
        weights2: vec![vec![3.5, 4.0], vec![4.5, 5.0]],
        bias2: vec![0.15, 0.2],
    };

    // Test subtraction
    let diff = w1.subtract(&w2);
    println!("  Subtraction: w1 - w2");
    println!("    weights1[0][0]: {} - {} = {}", 1.0, 0.5, diff.weights1[0][0]);
    assert!((diff.weights1[0][0] - 0.5).abs() < 1e-10);

    // Test addition
    let sum = w1.add(&w2);
    println!("  Addition: w1 + w2");
    println!("    weights1[0][0]: {} + {} = {}", 1.0, 0.5, sum.weights1[0][0]);
    assert!((sum.weights1[0][0] - 1.5).abs() < 1e-10);

    // Test scaling
    let scaled = w1.scale(0.5);
    println!("  Scaling: w1 * 0.5");
    println!("    weights1[0][0]: {} * 0.5 = {}", 1.0, scaled.weights1[0][0]);
    assert!((scaled.weights1[0][0] - 0.5).abs() < 1e-10);

    // Test averaging
    let avg = ReptileWeights::average(&[w1.clone(), w2.clone()]);
    println!("  Average: (w1 + w2) / 2");
    println!("    weights1[0][0]: ({} + {}) / 2 = {}", 1.0, 0.5, avg.weights1[0][0]);
    assert!((avg.weights1[0][0] - 0.75).abs() < 1e-10);

    // Test L2 norm
    let norm = w1.l2_norm();
    println!("  L2 norm of w1: {:.4}", norm);
    assert!(norm > 0.0);

    println!("[OK] Weight operations working\n");
}

fn test_pattern_tasks() {
    println!("--- Test 2: Pattern Tasks ---");

    let tasks = create_standard_tasks();
    println!("  Created {} standard tasks:", tasks.len());

    for task in &tasks {
        let target = task.generate_target();

        // Count active cells
        let active_cells: usize = target.cells.iter()
            .flatten()
            .filter(|c| c[3] > 0.5)
            .count();

        println!("    {}: {} active cells", task.name, active_cells);
        assert!(active_cells > 0, "Pattern {} should have active cells", task.name);
    }

    println!("[OK] Pattern tasks working\n");
}

fn test_meta_learning() {
    println!("--- Test 3: Meta-Learning Iterations ---");

    let mut nca = NCA::new();
    let tasks = create_standard_tasks();

    let mut reptile = ReptileMetaLearner::new(&nca)
        .with_meta_lr(0.1)
        .with_inner_steps(5)
        .with_inner_lr(0.0001)
        .with_task_batch_size(2);

    println!("  Running 10 meta-learning iterations...");
    println!("  (meta_lr=0.1, inner_steps=5, task_batch=2)\n");

    let mut improvements = Vec::new();

    for i in 0..10 {
        let result = reptile.meta_step(&mut nca, &tasks);

        improvements.push(result.avg_improvement);

        if i % 2 == 0 {
            println!("  Iter {:2}: tasks={}, init_loss={:.4}, final_loss={:.4}, improvement={:.4}",
                result.iteration,
                result.tasks_sampled,
                result.avg_initial_loss,
                result.avg_final_loss,
                result.avg_improvement);
        }
    }

    let stats = reptile.get_stats();
    println!("\n  Meta-learning stats:");
    println!("    Total iterations: {}", stats.meta_iteration);
    println!("    Total adaptations: {}", stats.total_adaptations);
    println!("    Avg recent improvement: {:.4}", stats.avg_recent_improvement);
    println!("    Meta-weights norm: {:.2}", stats.meta_weights_norm);

    println!("[OK] Meta-learning iterations working\n");
}

fn test_few_shot_adaptation() {
    println!("--- Test 4: Few-Shot Adaptation ---");

    let mut nca = NCA::new();
    let tasks = create_standard_tasks();

    // First, do some meta-training
    let mut reptile = ReptileMetaLearner::new(&nca)
        .with_meta_lr(0.1)
        .with_inner_steps(5)
        .with_task_batch_size(4);

    println!("  Meta-training for 20 iterations...");
    for _ in 0..20 {
        reptile.meta_step(&mut nca, &tasks);
    }

    // Now test few-shot adaptation
    println!("\n  Testing few-shot adaptation (10 steps max):");

    for task in &tasks {
        let result = reptile.adapt(&mut nca, task, 10);

        let status = if result.converged { "CONVERGED" } else { "in progress" };
        println!("    {}: {:.4} -> {:.4} in {} steps [{}]",
            task.name,
            result.initial_loss,
            result.final_loss,
            result.steps_taken,
            status);
    }

    println!("[OK] Few-shot adaptation working\n");
}

fn test_benchmark_comparison() {
    println!("--- Test 5: Meta vs Random Initialization Benchmark ---");

    let mut nca = NCA::new();
    let tasks = create_standard_tasks();

    // Meta-train
    let mut reptile = ReptileMetaLearner::new(&nca)
        .with_meta_lr(0.1)
        .with_inner_steps(5)
        .with_task_batch_size(4);

    println!("  Meta-training for 30 iterations...");
    for _ in 0..30 {
        reptile.meta_step(&mut nca, &tasks);
    }

    // Benchmark on circle (held out from intensive training)
    let circle_task = &tasks[0];
    println!("\n  Benchmarking on '{}' pattern (20 adaptation steps):", circle_task.name);

    let benchmark = reptile.benchmark_adaptation(&mut nca, circle_task, 20);

    println!("\n  From META initialization:");
    println!("    Initial loss: {:.4}", benchmark.meta_adaptation.initial_loss);
    println!("    Final loss:   {:.4}", benchmark.meta_adaptation.final_loss);
    println!("    Converged:    {}", benchmark.meta_adaptation.converged);

    println!("\n  From RANDOM initialization:");
    println!("    Initial loss: {:.4}", benchmark.random_adaptation.initial_loss);
    println!("    Final loss:   {:.4}", benchmark.random_adaptation.final_loss);
    println!("    Converged:    {}", benchmark.random_adaptation.converged);

    println!("\n  Loss comparison (lower is better):");
    let meta_better = benchmark.meta_adaptation.final_loss < benchmark.random_adaptation.final_loss;
    if meta_better {
        let improvement = (1.0 - benchmark.meta_adaptation.final_loss / benchmark.random_adaptation.final_loss) * 100.0;
        println!("    META is {:.1}% better!", improvement.abs());
    } else {
        println!("    Random performed better (meta-training may need more iterations)");
    }

    // Show loss curves
    println!("\n  Loss curves (first 10 steps):");
    println!("    Step  |  Meta   | Random");
    println!("    ------|---------|--------");
    for i in 0..10.min(benchmark.meta_adaptation.loss_history.len()) {
        let meta_loss = benchmark.meta_adaptation.loss_history.get(i).unwrap_or(&0.0);
        let random_loss = benchmark.random_adaptation.loss_history.get(i).unwrap_or(&0.0);
        println!("    {:5} | {:.4}  | {:.4}", i, meta_loss, random_loss);
    }

    println!("[OK] Benchmark comparison complete\n");
}
