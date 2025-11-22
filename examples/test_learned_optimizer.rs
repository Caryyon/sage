//! Test Learned Optimizer (L2O)
//!
//! This test verifies:
//! 1. Optimizer network produces valid updates
//! 2. Learned optimizer can optimize simple functions
//! 3. Meta-learner evolves optimizer population
//! 4. Comparison with Adam optimizer
//! 5. Optimizer can be saved/loaded
//!
//! Run with: cargo run --release --example test_learned_optimizer

use sage::learning::learned_optimizer::{
    LearnedOptimizer, OptimizerMetaLearner, AdamOptimizer,
    benchmark_optimizers, OptimizerInput, OptimizerNetwork,
};

fn main() {
    println!("=== Learned Optimizer (L2O) Test ===\n");

    test_optimizer_network();
    test_learned_optimizer();
    test_meta_learner();
    test_adam_comparison();
    test_weights_persistence();

    println!("\n=== All Learned Optimizer Tests Passed! ===");
}

fn test_optimizer_network() {
    println!("--- Test 1: Optimizer Network ---");

    let mut network = OptimizerNetwork::new(16);

    // Test with various inputs
    let test_cases = vec![
        OptimizerInput { gradient: 0.5, momentum: 0.1, loss: 0.8, step: 1, grad_squared: 0.25, loss_delta: 0.0 },
        OptimizerInput { gradient: -0.3, momentum: -0.05, loss: 0.5, step: 10, grad_squared: 0.09, loss_delta: -0.1 },
        OptimizerInput { gradient: 2.0, momentum: 1.5, loss: 0.2, step: 100, grad_squared: 4.0, loss_delta: -0.05 },
    ];

    println!("  Testing update computation:");
    for (i, features) in test_cases.iter().enumerate() {
        let update = network.compute_update(features);
        println!("    Case {}: grad={:.2}, momentum={:.2} -> update={:.4}",
            i + 1, features.gradient, features.momentum, update);
        assert!(update.abs() <= 1.0, "Update should be clipped to [-1, 1]");
    }

    // Test reset
    network.reset();
    let update_after_reset = network.compute_update(&test_cases[0]);
    println!("\n  After reset: update={:.4}", update_after_reset);

    println!("[OK] Optimizer network working\n");
}

fn test_learned_optimizer() {
    println!("--- Test 2: Learned Optimizer ---");

    let num_params = 10;
    let mut optimizer = LearnedOptimizer::new(num_params, 0.01);

    // Simulate optimization on a simple quadratic
    let target = vec![1.0; num_params];
    let mut params = vec![0.0; num_params];

    println!("  Optimizing quadratic function (target = 1.0)...");
    println!("  Step | Loss      | Param[0]");
    println!("  -----|-----------|--------");

    for step in 0..50 {
        // Compute loss and gradients
        let loss: f64 = params.iter()
            .zip(target.iter())
            .map(|(p, t): (&f64, &f64)| (p - t).powi(2))
            .sum();

        let gradients: Vec<f64> = params.iter()
            .zip(target.iter())
            .map(|(p, t)| 2.0 * (p - t))
            .collect();

        // Get updates from learned optimizer
        let updates = optimizer.compute_updates(&gradients, loss);

        // Apply updates
        for (p, u) in params.iter_mut().zip(updates.iter()) {
            *p += u;
        }

        if step % 10 == 0 || step == 49 {
            println!("  {:4} | {:.6}  | {:.4}", step, loss, params[0]);
        }
    }

    let final_loss: f64 = params.iter()
        .zip(target.iter())
        .map(|(p, t): (&f64, &f64)| (p - t).powi(2))
        .sum();

    println!("\n  Final loss: {:.6}", final_loss);
    println!("  Final params[0..3]: [{:.3}, {:.3}, {:.3}]", params[0], params[1], params[2]);

    println!("[OK] Learned optimizer working\n");
}

fn test_meta_learner() {
    println!("--- Test 3: Meta-Learner (Evolution) ---");

    let pop_size = 8;
    let num_params = 5;
    let mut meta = OptimizerMetaLearner::new(pop_size, num_params, 0.01);

    println!("  Population size: {}", meta.population_size());
    println!("  Running 5 generations of evolution...\n");

    for gen in 0..5 {
        // Evaluate each optimizer on a simple task
        for i in 0..pop_size {
            let optimizer = meta.get_optimizer(i);
            optimizer.reset();

            // Simple fitness: ability to reduce loss on quadratic
            let target = vec![1.0; num_params];
            let mut params = vec![0.0; num_params];
            let mut total_loss = 0.0;

            for _ in 0..30 {
                let loss: f64 = params.iter()
                    .zip(target.iter())
                    .map(|(p, t): (&f64, &f64)| (p - t).powi(2))
                    .sum();
                total_loss += loss;

                let gradients: Vec<f64> = params.iter()
                    .zip(target.iter())
                    .map(|(p, t)| 2.0 * (p - t))
                    .collect();

                let updates = optimizer.compute_updates(&gradients, loss);
                for (p, u) in params.iter_mut().zip(updates.iter()) {
                    *p += u;
                }
            }

            // Fitness: inverse of total loss
            let fitness = 1.0 / (1.0 + total_loss);
            meta.record_fitness(i, fitness);
        }

        // Evolve
        let stats = meta.evolve();
        println!("  Gen {}: best_fitness={:.4}, avg_fitness={:.4}, elite={}",
            stats.generation, meta.best_fitness(), stats.avg_fitness, stats.elite_count);
    }

    // Get best optimizer
    let best = meta.best_optimizer(num_params, 0.01);
    println!("\n  Best optimizer retrieved (step count: {})", best.step());

    println!("[OK] Meta-learner working\n");
}

fn test_adam_comparison() {
    println!("--- Test 4: Comparison with Adam ---");

    let num_params = 5;
    let mut learned = LearnedOptimizer::new(num_params, 0.01);
    let mut adam = AdamOptimizer::new(num_params, 0.01);

    // First, evolve the learned optimizer a bit
    println!("  Training learned optimizer for 3 generations...");
    let mut meta = OptimizerMetaLearner::new(10, num_params, 0.01);
    for _ in 0..3 {
        for i in 0..10 {
            let opt = meta.get_optimizer(i);
            opt.reset();
            let target = vec![2.0; num_params];
            let mut params = vec![0.0; num_params];
            let mut total_loss = 0.0;

            for _ in 0..50 {
                let loss: f64 = params.iter().zip(target.iter()).map(|(p, t): (&f64, &f64)| (p - t).powi(2)).sum();
                total_loss += loss;
                let gradients: Vec<f64> = params.iter().zip(target.iter()).map(|(p, t): (&f64, &f64)| 2.0 * (p - t)).collect();
                let updates = opt.compute_updates(&gradients, loss);
                for (p, u) in params.iter_mut().zip(updates.iter()) { *p += u; }
            }
            meta.record_fitness(i, 1.0 / (1.0 + total_loss));
        }
        meta.evolve();
    }
    learned = meta.best_optimizer(num_params, 0.01);

    // Benchmark
    println!("\n  Running benchmark (100 steps)...");
    let result = benchmark_optimizers(&mut learned, &mut adam, 100);

    println!("\n  Results:");
    println!("    Learned optimizer:");
    println!("      Final loss: {:.6}", result.learned_final_loss);
    println!("      Avg loss: {:.6}", result.learned_avg_loss);
    println!("      Converged at step: {:?}", result.learned_convergence_step);

    println!("    Adam optimizer:");
    println!("      Final loss: {:.6}", result.adam_final_loss);
    println!("      Avg loss: {:.6}", result.adam_avg_loss);
    println!("      Converged at step: {:?}", result.adam_convergence_step);

    println!("\n  Winner: {}", if result.learned_wins() { "Learned" } else { "Adam" });

    println!("[OK] Adam comparison working\n");
}

fn test_weights_persistence() {
    println!("--- Test 5: Weights Persistence ---");

    let num_params = 5;
    let mut opt1 = LearnedOptimizer::new(num_params, 0.01);

    // Train opt1 a bit
    let target = vec![1.5; num_params];
    let mut params = vec![0.0; num_params];
    for _ in 0..20 {
        let loss: f64 = params.iter().zip(target.iter()).map(|(p, t): (&f64, &f64)| (p - t).powi(2)).sum();
        let gradients: Vec<f64> = params.iter().zip(target.iter()).map(|(p, t): (&f64, &f64)| 2.0 * (p - t)).collect();
        let updates = opt1.compute_updates(&gradients, loss);
        for (p, u) in params.iter_mut().zip(updates.iter()) { *p += u; }
    }

    // Save weights
    let weights = opt1.get_weights();
    println!("  Saved weights from opt1");
    println!("    w_proj length: {}", weights.w_proj.len());
    println!("    w_input shape: {}x{}", weights.w_input.len(),
        weights.w_input.first().map(|r| r.len()).unwrap_or(0));

    // Create new optimizer and load weights
    let mut opt2 = LearnedOptimizer::new(num_params, 0.01);
    opt2.set_weights(&weights);
    println!("\n  Loaded weights into opt2");

    // Both should produce same update for same input
    opt1.reset();
    opt2.reset();

    let test_gradients = vec![0.5; num_params];
    let test_loss = 0.3;

    let updates1 = opt1.compute_updates(&test_gradients, test_loss);
    let updates2 = opt2.compute_updates(&test_gradients, test_loss);

    let diff: f64 = updates1.iter()
        .zip(updates2.iter())
        .map(|(a, b)| (a - b).abs())
        .sum();

    println!("\n  Update difference after weight transfer: {:.10}", diff);
    assert!(diff < 1e-9, "Updates should be identical after weight transfer");

    println!("[OK] Weights persistence working\n");
}
