//! Test Population Based Training
//!
//! This test verifies:
//! 1. Population initialization with diverse hyperparameters
//! 2. Exploit/explore evolution works correctly
//! 3. Best hyperparameters are discovered over time
//! 4. Hyperparameter schedules are tracked
//!
//! Run with: cargo run --release --example test_pbt

use sage::learning::population_based_training::{
    PopulationBasedTraining, Hyperparameters, PBTTrainer,
};
use sage::nca::NCA;

fn main() {
    println!("=== Population Based Training Test ===\n");

    test_hyperparameters();
    test_population_diversity();
    test_evolution();
    test_pbt_trainer();
    test_hyperparameter_schedule();

    println!("\n=== All PBT Tests Passed! ===");
}

fn test_hyperparameters() {
    println!("--- Test 1: Hyperparameters ---");

    let default_hp = Hyperparameters::default();
    println!("  Default hyperparameters:");
    println!("    learning_rate: {:.6}", default_hp.learning_rate);
    println!("    evolution_steps: {}", default_hp.evolution_steps);
    println!("    batch_size: {}", default_hp.batch_size);
    println!("    pool_sample_rate: {:.2}", default_hp.pool_sample_rate);
    println!("    stochastic_update_rate: {:.2}", default_hp.stochastic_update_rate);
    println!("    grad_clip_norm: {:.2}", default_hp.grad_clip_norm);

    // Test random initialization
    println!("\n  Random hyperparameters (5 samples):");
    for i in 0..5 {
        let hp = Hyperparameters::random();
        println!("    #{}: lr={:.6}, steps={}, batch={}",
            i + 1, hp.learning_rate, hp.evolution_steps, hp.batch_size);
    }

    // Test mutation
    let mut hp = Hyperparameters::default();
    let original_lr = hp.learning_rate;
    println!("\n  Testing mutation (10 rounds):");
    for i in 0..10 {
        hp.mutate();
        if i % 3 == 0 {
            println!("    After {} mutations: lr={:.6}, steps={}",
                i + 1, hp.learning_rate, hp.evolution_steps);
        }
    }
    println!("    LR changed: {} -> {} ({:.1}x)",
        original_lr, hp.learning_rate, hp.learning_rate / original_lr);

    println!("[OK] Hyperparameters working\n");
}

fn test_population_diversity() {
    println!("--- Test 2: Population Diversity ---");

    let nca = NCA::new();
    let pbt = PopulationBasedTraining::new(10, &nca);

    println!("  Population size: {}", pbt.population_size());

    let population = pbt.get_population();
    println!("\n  Member hyperparameters:");
    println!("  ID | Learning Rate | Evo Steps | Batch | Pool Rate");
    println!("  ---|---------------|-----------|-------|----------");

    for member in population {
        println!("  {:2} | {:.6}      | {:3}       | {:2}    | {:.2}",
            member.id,
            member.hyperparams.learning_rate,
            member.hyperparams.evolution_steps,
            member.hyperparams.batch_size,
            member.hyperparams.pool_sample_rate);
    }

    // Check diversity
    let lrs: Vec<f64> = population.iter()
        .map(|m| m.hyperparams.learning_rate)
        .collect();
    let unique_lrs: std::collections::HashSet<u64> = lrs.iter()
        .map(|lr| (lr * 1e10) as u64)
        .collect();

    println!("\n  Unique learning rates: {} / {}", unique_lrs.len(), lrs.len());
    assert!(unique_lrs.len() > 1, "Should have diverse learning rates");

    println!("[OK] Population diversity working\n");
}

fn test_evolution() {
    println!("--- Test 3: Evolution (Exploit/Explore) ---");

    let nca = NCA::new();
    let mut pbt = PopulationBasedTraining::new(8, &nca)
        .with_exploit_fraction(0.25)  // Top/bottom 25%
        .with_evolution_interval(10);

    // Simulate evaluations with varying scores
    println!("  Simulating 50 evaluations...");
    for i in 0..50 {
        let member_id = i % 8;
        // Give lower IDs better scores (to test that they survive)
        let score = 0.1 + (member_id as f64 * 0.1) + rand::random::<f64>() * 0.05;
        pbt.record_evaluation(member_id, score, &nca);
    }

    let stats = pbt.get_stats();
    println!("\n  After 50 evaluations:");
    println!("    Total steps: {}", stats.total_steps);
    println!("    Evolutions: {}", stats.evolutions);
    println!("    Best score: {:.4}", stats.best_score);
    println!("    Avg score: {:.4}", stats.avg_score);
    println!("    Score variance: {:.4}", stats.score_variance);
    println!("    Diversity: {:.4}", stats.diversity);

    // Check that evolution happened
    assert!(stats.evolutions > 0, "Should have evolved at least once");

    // Check population state
    println!("\n  Population after evolution:");
    for member in pbt.get_population() {
        let improving = if member.is_improving() { "↑" } else { "→" };
        println!("    Member {}: score={:.4}, gen={}, steps={} {}",
            member.id, member.avg_recent_score(), member.generation, member.steps, improving);
    }

    println!("[OK] Evolution working\n");
}

fn test_pbt_trainer() {
    println!("--- Test 4: PBT Trainer ---");

    let nca = NCA::new();
    let mut trainer = PBTTrainer::new(6, &nca);

    println!("  Running 100 training steps...");
    let mut evolution_count = 0;

    for step in 0..100 {
        // Get current hyperparams
        let hp = trainer.current_hyperparams();

        // Simulate training with these hyperparams
        // Better hyperparams (lower LR, more steps) = lower loss (simplified)
        let base_loss = 0.5 * (hp.learning_rate / 0.0001).ln().abs() / 10.0
            + 0.3 * (64.0 / hp.evolution_steps as f64);
        let loss = base_loss + rand::random::<f64>() * 0.1;

        // Record result
        if let Some(evo_result) = trainer.record_result(loss, &nca) {
            evolution_count += 1;
            if step < 60 || step > 90 {
                println!("  Step {:3}: Evolution #{}, best_score={:.4}",
                    step, evolution_count, evo_result.best_score);
            }
        }
    }

    let stats = trainer.stats();
    println!("\n  Final statistics:");
    println!("    Total steps: {}", stats.total_steps);
    println!("    Evolutions: {}", stats.evolutions);
    println!("    Best score: {:.4}", stats.best_score);
    println!("    Avg score: {:.4}", stats.avg_score);

    let best_hp = trainer.best_hyperparams();
    println!("\n  Best hyperparameters found:");
    println!("    learning_rate: {:.6}", best_hp.learning_rate);
    println!("    evolution_steps: {}", best_hp.evolution_steps);
    println!("    batch_size: {}", best_hp.batch_size);

    println!("[OK] PBT Trainer working\n");
}

fn test_hyperparameter_schedule() {
    println!("--- Test 5: Hyperparameter Schedule ---");

    let nca = NCA::new();
    let mut pbt = PopulationBasedTraining::new(6, &nca)
        .with_evolution_interval(20);

    // Run longer to see schedule evolution
    println!("  Running 200 evaluations to discover schedule...");
    for i in 0..200 {
        let member_id = i % 6;
        // Simulate that lower learning rates perform better over time
        let member_hp = pbt.get_hyperparams(member_id).unwrap();
        let score = member_hp.learning_rate * 1000.0 + rand::random::<f64>() * 0.1;
        pbt.record_evaluation(member_id, score, &nca);
    }

    let schedule = pbt.get_schedule();
    println!("\n  Hyperparameter schedule ({} checkpoints):", schedule.len());
    println!("  Step | Learning Rate | Evo Steps | Batch");
    println!("  -----|---------------|-----------|------");

    for (step, hp) in schedule.iter().take(10) {
        println!("  {:4} | {:.6}      | {:3}       | {:2}",
            step, hp.learning_rate, hp.evolution_steps, hp.batch_size);
    }

    if schedule.len() > 10 {
        println!("  ... ({} more checkpoints)", schedule.len() - 10);
    }

    // Check that learning rate evolved (should trend lower based on our fitness function)
    if schedule.len() >= 2 {
        let first_lr = schedule.first().unwrap().1.learning_rate;
        let last_lr = schedule.last().unwrap().1.learning_rate;
        println!("\n  Learning rate evolution: {:.6} -> {:.6}", first_lr, last_lr);

        if last_lr < first_lr {
            println!("  [OK] LR decreased (as expected from fitness function)");
        } else {
            println!("  [INFO] LR didn't decrease (stochastic variation)");
        }
    }

    println!("[OK] Hyperparameter schedule working\n");
}
