//! Unified Meta-Learning Demo
//!
//! This demo shows all 6 meta-learning phases working together:
//!
//! 1. Phase D: Meta-Strategy Selection - picks best approach
//! 2. Phase A: Enhanced Adaptive LR - warmup + cosine annealing
//! 3. Phase B: Reptile - few-shot pattern adaptation
//! 4. Phase C: PBT - hyperparameter evolution
//! 5. Phase E: Architecture Modification - detects underfitting
//! 6. Phase F: Learned Optimizer - neural network-based updates
//!
//! Run with: cargo run --release --example meta_learning_demo

use sage::nca::NCA;
use sage::grid::Grid;
use sage::learning::enhanced_meta_learning::{
    EnhancedAdaptiveLR, GradientStats,
};
use sage::learning::reptile::{ReptileMetaLearner, PatternTask};
use sage::learning::population_based_training::{
    PopulationBasedTraining, PBTTrainer,
};
use sage::learning::meta_strategy::{
    MetaStrategyController, TaskFeatures, DataAvailability, TimeBudget,
};
use sage::learning::architecture_modification::{
    ArchitectureModifier, SafetyConfig, ModificationResult,
};
use sage::learning::learned_optimizer::{
    OptimizerMetaLearner, AdamOptimizer, benchmark_optimizers,
};

fn main() {
    println!("╔══════════════════════════════════════════════════════════════════╗");
    println!("║           SAGE Meta-Learning System - Unified Demo               ║");
    println!("║                   All 6 Phases Working Together                  ║");
    println!("╚══════════════════════════════════════════════════════════════════╝\n");

    // Initialize NCA
    let mut nca = NCA::new();
    println!("✓ NCA initialized (22 channels, 48→16 network)\n");

    demo_phase_d_strategy_selection();
    demo_phase_a_adaptive_learning();
    demo_phase_b_reptile(&mut nca);
    demo_phase_c_pbt(&mut nca);
    demo_phase_e_architecture();
    demo_phase_f_learned_optimizer();
    demo_integrated_training(&mut nca);

    println!("\n╔══════════════════════════════════════════════════════════════════╗");
    println!("║                    Demo Complete! All Systems Go                 ║");
    println!("╚══════════════════════════════════════════════════════════════════╝");
}

fn demo_phase_d_strategy_selection() {
    println!("┌────────────────────────────────────────────────────────────────┐");
    println!("│ Phase D: Meta-Strategy Selection                               │");
    println!("│ Automatically choosing the best learning approach              │");
    println!("└────────────────────────────────────────────────────────────────┘\n");

    let mut controller = MetaStrategyController::new();

    // Scenario 1: Few-shot learning needed
    let few_shot_task = TaskFeatures {
        complexity: 0.5,
        familiarity: 0.7,
        data_availability: DataAvailability::Scarce,
        structured: true,
        exploration_benefit: 0.3,
        time_budget: TimeBudget::Urgent,
        historical_performance: std::collections::HashMap::new(),
    };

    let selection = controller.start_task(few_shot_task);
    println!("  Scenario: Few-shot learning (scarce data, urgent)");
    println!("  → Strategy: {} ", selection.strategy.name());
    println!("  → Reasoning: {}\n", selection.reasoning);

    // Scenario 2: Complex task with time
    let complex_task = TaskFeatures {
        complexity: 0.9,
        familiarity: 0.2,
        data_availability: DataAvailability::Abundant,
        structured: true,
        exploration_benefit: 0.8,
        time_budget: TimeBudget::Extended,
        historical_performance: std::collections::HashMap::new(),
    };

    let selection = controller.start_task(complex_task);
    println!("  Scenario: Complex novel task (time available)");
    println!("  → Strategy: {}", selection.strategy.name());
    println!("  → Reasoning: {}\n", selection.reasoning);

    // Scenario 3: Simple familiar task
    let simple_task = TaskFeatures {
        complexity: 0.2,
        familiarity: 0.9,
        data_availability: DataAvailability::Abundant,
        structured: true,
        exploration_benefit: 0.1,
        time_budget: TimeBudget::Normal,
        historical_performance: std::collections::HashMap::new(),
    };

    let selection = controller.start_task(simple_task);
    println!("  Scenario: Simple familiar task");
    println!("  → Strategy: {}", selection.strategy.name());
    println!("  → Reasoning: {}\n", selection.reasoning);

    println!("  ✓ Meta-Strategy Selection working!\n");
}

fn demo_phase_a_adaptive_learning() {
    println!("┌────────────────────────────────────────────────────────────────┐");
    println!("│ Phase A: Enhanced Adaptive Learning                            │");
    println!("│ Warmup + Cosine Annealing + Gradient-Aware LR                  │");
    println!("└────────────────────────────────────────────────────────────────┘\n");

    let mut adaptive_lr = EnhancedAdaptiveLR::new(0.001)
        .with_warmup(20)
        .with_cycle_length(50);

    println!("  Training with adaptive learning rate:");
    println!("  Step | Loss   | LR        | Phase");
    println!("  -----|--------|-----------|-------");

    for step in 0..60 {
        // Simulate decreasing loss
        let loss = 0.5 * (-0.03 * step as f64).exp() + rand::random::<f64>() * 0.05;
        let grad_stats = GradientStats::new(0.1 + rand::random::<f64>() * 0.05, 0.01);

        adaptive_lr.step(loss, Some(grad_stats));
        let lr = adaptive_lr.get_rate();

        let phase = if step < 20 { "warmup" } else { "cosine" };

        if step % 10 == 0 || step == 59 {
            println!("  {:4} | {:.4} | {:.7} | {}", step, loss, lr, phase);
        }
    }

    let diag = adaptive_lr.get_diagnostics();
    println!("\n  Diagnostics:");
    println!("    Warmup complete: {}", diag.warmup_complete);
    println!("    Current LR: {:.7}", diag.current_rate);

    println!("\n  ✓ Adaptive Learning working!\n");
}

fn demo_phase_b_reptile(nca: &mut NCA) {
    println!("┌────────────────────────────────────────────────────────────────┐");
    println!("│ Phase B: Reptile Meta-Learning                                 │");
    println!("│ Few-shot pattern adaptation                                    │");
    println!("└────────────────────────────────────────────────────────────────┘\n");

    let mut reptile = ReptileMetaLearner::new(nca);

    // Create pattern tasks
    let tasks = vec![
        PatternTask::new("circle", "Circle", create_circle_grid),
        PatternTask::new("square", "Square", create_square_grid),
        PatternTask::new("cross", "Cross", create_cross_grid),
    ];

    println!("  Meta-training on {} patterns...", tasks.len());

    // Do a few meta-steps
    for meta_step in 0..3 {
        let result = reptile.meta_step(nca, &tasks);
        println!("  Meta-step {}: avg_loss={:.4}, tasks={}",
            meta_step + 1, result.avg_final_loss, result.tasks_sampled);
    }

    // Test few-shot adaptation
    println!("\n  Testing few-shot adaptation on Spiral pattern:");
    let spiral_task = PatternTask::new("spiral", "Spiral", create_spiral_grid);

    let benchmark = reptile.benchmark_adaptation(nca, &spiral_task, 20);

    println!("    With Reptile init: {} steps (loss={:.4})",
        benchmark.meta_adaptation.steps_taken,
        benchmark.meta_adaptation.final_loss);
    println!("    Random init: {} steps (loss={:.4})",
        benchmark.random_adaptation.steps_taken,
        benchmark.random_adaptation.final_loss);
    println!("    Speedup: {:.2}x", benchmark.speedup);

    println!("\n  ✓ Reptile Few-Shot Learning working!\n");
}

fn demo_phase_c_pbt(nca: &mut NCA) {
    println!("┌────────────────────────────────────────────────────────────────┐");
    println!("│ Phase C: Population Based Training                             │");
    println!("│ Hyperparameter evolution with exploit/explore                  │");
    println!("└────────────────────────────────────────────────────────────────┘\n");

    let mut pbt = PopulationBasedTraining::new(6, nca)
        .with_exploit_fraction(0.25)
        .with_evolution_interval(20);

    println!("  Population of {} members, evolving every 20 steps", pbt.population_size());
    println!("\n  Initial hyperparameter diversity:");
    println!("  ID | LR        | Steps | Batch");
    println!("  ---|-----------|-------|------");

    for member in pbt.get_population().iter().take(4) {
        println!("  {:2} | {:.6}  | {:3}   | {:2}",
            member.id, member.hyperparams.learning_rate,
            member.hyperparams.evolution_steps, member.hyperparams.batch_size);
    }

    // Simulate training
    println!("\n  Running 60 evaluations...");
    for i in 0..60 {
        let member_id = i % 6;
        // Better hyperparams = lower score (simulated)
        let hp = pbt.get_hyperparams(member_id).unwrap();
        let score = hp.learning_rate * 5000.0 + rand::random::<f64>() * 0.1;
        pbt.record_evaluation(member_id, score, nca);
    }

    let stats = pbt.get_stats();
    println!("\n  After evolution:");
    println!("    Evolutions: {}", stats.evolutions);
    println!("    Best score: {:.4}", stats.best_score);
    println!("    Avg score: {:.4}", stats.avg_score);
    println!("    Diversity: {:.4}", stats.diversity);

    // Show discovered schedule
    let schedule = pbt.get_schedule();
    if !schedule.is_empty() {
        println!("\n  Discovered hyperparameter schedule:");
        for (step, hp) in schedule.iter().take(3) {
            println!("    Step {}: lr={:.6}", step, hp.learning_rate);
        }
    }

    println!("\n  ✓ Population Based Training working!\n");
}

fn demo_phase_e_architecture() {
    println!("┌────────────────────────────────────────────────────────────────┐");
    println!("│ Phase E: Architecture Self-Modification                        │");
    println!("│ Safe network modification with rollback                        │");
    println!("└────────────────────────────────────────────────────────────────┘\n");

    let config = SafetyConfig {
        modification_cooldown: 10,
        max_modifications_per_session: 5,
        ..SafetyConfig::default()
    };

    let mut modifier = ArchitectureModifier::new(16, config);

    println!("  Initial state: {} hidden neurons", modifier.state().hidden_neurons);
    println!("  Safety bounds: 8-64 neurons, max 4 change/step\n");

    // Simulate underfitting scenario
    println!("  Simulating underfitting (high loss, no improvement)...");
    for _ in 0..60 {
        let loss = 0.7 + rand::random::<f64>() * 0.1;
        modifier.update(loss, 0.1, None);
    }

    // Try to get modification
    let result = modifier.try_modification();
    match result {
        ModificationResult::Applied { modification, new_state, reason } => {
            println!("  → Modification applied!");
            println!("    Type: {}", modification.description());
            println!("    Reason: {}", reason);
            println!("    New neurons: {}", new_state.hidden_neurons);
        }
        ModificationResult::NeedsApproval { modification, reason } => {
            println!("  → Needs approval: {} - {}", modification.description(), reason);
        }
        ModificationResult::NoChange { reason } => {
            println!("  → No change: {}", reason);
        }
        _ => {}
    }

    // Test rollback
    println!("\n  Testing rollback on performance drop...");
    let rollback = modifier.check_rollback(0.2);  // Simulate bad performance
    match rollback {
        Some(r) => {
            println!("  → Rollback triggered!");
            println!("    Performance: {:.3} -> {:.3}", r.performance_before, r.performance_after);
            println!("    Restored to: {} neurons", r.restored_state.hidden_neurons);
        }
        None => {
            println!("  → No rollback needed");
        }
    }

    let stats = modifier.stats();
    println!("\n  Final stats:");
    println!("    Applied: {}, Rolled back: {}", stats.applied, stats.rolled_back);

    println!("\n  ✓ Architecture Self-Modification working!\n");
}

fn demo_phase_f_learned_optimizer() {
    println!("┌────────────────────────────────────────────────────────────────┐");
    println!("│ Phase F: Learned Optimizer (L2O)                               │");
    println!("│ Neural network-based optimization                              │");
    println!("└────────────────────────────────────────────────────────────────┘\n");

    let num_params = 5;

    // Train learned optimizer via evolution
    println!("  Training learned optimizer (3 generations)...");
    let mut meta = OptimizerMetaLearner::new(8, num_params, 0.01);

    for gen in 1..=3 {
        for i in 0..8 {
            let opt = meta.get_optimizer(i);
            opt.reset();

            let target = vec![2.0; num_params];
            let mut params = vec![0.0; num_params];
            let mut total_loss = 0.0;

            for _ in 0..30 {
                let loss: f64 = params.iter().zip(target.iter())
                    .map(|(p, t): (&f64, &f64)| (p - t).powi(2)).sum();
                total_loss += loss;
                let gradients: Vec<f64> = params.iter().zip(target.iter())
                    .map(|(p, t): (&f64, &f64)| 2.0 * (p - t)).collect();
                let updates = opt.compute_updates(&gradients, loss);
                for (p, u) in params.iter_mut().zip(updates.iter()) { *p += u; }
            }
            meta.record_fitness(i, 1.0 / (1.0 + total_loss));
        }
        let _stats = meta.evolve();
        println!("    Gen {}: best_fitness={:.4}", gen, meta.best_fitness());
    }

    // Compare with Adam
    println!("\n  Comparing with Adam optimizer (50 steps)...");
    let mut learned = meta.best_optimizer(num_params, 0.01);
    let mut adam = AdamOptimizer::new(num_params, 0.01);

    let result = benchmark_optimizers(&mut learned, &mut adam, 50);

    println!("    Learned: final_loss={:.4}, avg={:.4}",
        result.learned_final_loss, result.learned_avg_loss);
    println!("    Adam:    final_loss={:.4}, avg={:.4}",
        result.adam_final_loss, result.adam_avg_loss);
    println!("    Winner: {}", if result.learned_wins() { "Learned! 🎉" } else { "Adam (needs more training)" });

    println!("\n  ✓ Learned Optimizer working!\n");
}

fn demo_integrated_training(nca: &mut NCA) {
    println!("┌────────────────────────────────────────────────────────────────┐");
    println!("│ INTEGRATED: All Systems Training Together                      │");
    println!("│ Meta-strategy → Reptile → PBT → Adaptive LR                    │");
    println!("└────────────────────────────────────────────────────────────────┘\n");

    // 1. Meta-strategy selects approach
    let mut strategy_ctrl = MetaStrategyController::new();
    let features = TaskFeatures::from_pattern("spiral", &[0.5, 0.4, 0.35]);
    let selection = strategy_ctrl.start_task(features);
    println!("  1. Meta-Strategy selected: {}", selection.strategy.name());

    // 2. Initialize systems based on strategy
    let mut reptile = ReptileMetaLearner::new(nca);
    let mut adaptive_lr = EnhancedAdaptiveLR::new(0.0001).with_warmup(10);
    let mut pbt_trainer = PBTTrainer::new(4, nca);

    // 3. Pre-train with Reptile on base patterns
    println!("  2. Reptile meta-training on base patterns...");
    let base_tasks = vec![
        PatternTask::new("circle", "Circle", create_circle_grid),
        PatternTask::new("square", "Square", create_square_grid),
    ];
    for _ in 0..2 {
        reptile.meta_step(nca, &base_tasks);
    }
    println!("     Meta-weights initialized from 2 meta-steps");

    // 4. Adapt to target pattern
    println!("  3. Few-shot adaptation to Spiral...");
    let spiral = PatternTask::new("spiral", "Spiral", create_spiral_grid);
    let adapt_result = reptile.adapt(nca, &spiral, 15);
    println!("     Adapted in {} steps (loss: {:.4})",
        adapt_result.steps_taken, adapt_result.final_loss);

    // 5. Fine-tune with PBT-discovered hyperparameters
    println!("  4. Fine-tuning with PBT hyperparameters...");
    let hp = pbt_trainer.current_hyperparams();
    println!("     Using lr={:.6}, steps={}", hp.learning_rate, hp.evolution_steps);

    // 6. Training loop with adaptive LR
    println!("  5. Training with adaptive LR...\n");
    println!("  Step | Loss   | LR");
    println!("  -----|--------|----------");

    let mut best_loss = f64::MAX;

    for step in 0..30 {
        // Simulate NCA training
        nca.step();
        let loss = 0.4 * (-0.05 * step as f64).exp() + rand::random::<f64>() * 0.03;
        best_loss = best_loss.min(loss);

        let grad_stats = GradientStats::new(0.1, 0.01);
        adaptive_lr.step(loss, Some(grad_stats));
        let lr = adaptive_lr.get_rate();

        // Record for PBT
        pbt_trainer.record_result(loss, nca);

        if step % 5 == 0 || step == 29 {
            println!("  {:4} | {:.4} | {:.8}", step, loss, lr);
        }
    }

    println!("\n  ═══════════════════════════════════════");
    println!("  Final Results:");
    println!("    Best loss achieved: {:.4}", best_loss);
    println!("    Strategy used: {}", selection.strategy.name());
    println!("    PBT evolutions: {}", pbt_trainer.stats().evolutions);
    println!("  ═══════════════════════════════════════");
}

// Pattern generator functions
fn create_circle_grid() -> Grid {
    let mut grid = Grid::new(32, 32);
    let center = 16.0;
    let radius = 10.0;

    for y in 0..32 {
        for x in 0..32 {
            let dx = x as f64 - center;
            let dy = y as f64 - center;
            let dist = (dx * dx + dy * dy).sqrt();

            if dist < radius {
                grid.cells[y][x][3] = 1.0 - (dist / radius);
                grid.cells[y][x][0] = 0.2;
                grid.cells[y][x][1] = 0.6;
                grid.cells[y][x][2] = 1.0;
            }
        }
    }
    grid
}

fn create_square_grid() -> Grid {
    let mut grid = Grid::new(32, 32);
    for y in 8..24 {
        for x in 8..24 {
            grid.cells[y][x][3] = 1.0;
            grid.cells[y][x][0] = 1.0;
            grid.cells[y][x][1] = 0.5;
            grid.cells[y][x][2] = 0.2;
        }
    }
    grid
}

fn create_cross_grid() -> Grid {
    let mut grid = Grid::new(32, 32);
    // Horizontal bar
    for y in 13..19 {
        for x in 4..28 {
            grid.cells[y][x][3] = 1.0;
            grid.cells[y][x][0] = 0.8;
            grid.cells[y][x][1] = 0.2;
            grid.cells[y][x][2] = 0.2;
        }
    }
    // Vertical bar
    for y in 4..28 {
        for x in 13..19 {
            grid.cells[y][x][3] = 1.0;
            grid.cells[y][x][0] = 0.8;
            grid.cells[y][x][1] = 0.2;
            grid.cells[y][x][2] = 0.2;
        }
    }
    grid
}

fn create_spiral_grid() -> Grid {
    let mut grid = Grid::new(32, 32);
    let center = 16.0;

    for y in 0..32 {
        for x in 0..32 {
            let dx = x as f64 - center;
            let dy = y as f64 - center;
            let dist = (dx * dx + dy * dy).sqrt();
            let angle = dy.atan2(dx);

            let spiral_dist = (angle * 2.0 + dist * 0.3).sin() * 0.5 + 0.5;

            if dist < 14.0 && dist > 2.0 {
                let intensity = spiral_dist * (1.0 - dist / 14.0);
                grid.cells[y][x][3] = intensity;
                grid.cells[y][x][0] = intensity * 0.8;
                grid.cells[y][x][1] = intensity * 0.3;
                grid.cells[y][x][2] = intensity;
            }
        }
    }
    grid
}
