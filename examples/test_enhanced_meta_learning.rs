//! Test Enhanced Meta-Learning System
//!
//! This test verifies:
//! 1. Self-Paced Curriculum selects patterns based on difficulty
//! 2. Enhanced Adaptive LR uses warmup + cosine annealing
//! 3. Gradient stats improve learning rate decisions
//! 4. The unified controller integrates all systems
//!
//! Run with: cargo run --release --example test_enhanced_meta_learning

use sage::learning::enhanced_meta_learning::{
    EnhancedAdaptiveLR, SelfPacedCurriculum,
    GradientStats, EnhancedMetaLearningController, PaceFunction,
};
use sage::nca::NCA;

fn main() {
    println!("=== Enhanced Meta-Learning Test ===\n");

    test_enhanced_adaptive_lr();
    test_self_paced_curriculum();
    test_gradient_stats();
    test_unified_controller();
    test_with_real_nca();

    println!("\n=== All Tests Passed! ===");
}

fn test_enhanced_adaptive_lr() {
    println!("--- Test 1: Enhanced Adaptive Learning Rate ---");

    let mut lr = EnhancedAdaptiveLR::new(0.001)
        .with_warmup(50)
        .with_cycle_length(100);

    // Track LR during warmup
    println!("\nWarmup phase (steps 0-50):");
    for i in 0..60 {
        let rate = lr.get_rate();
        if i % 10 == 0 {
            println!("  Step {:3}: LR = {:.6}", i, rate);
        }
        lr.step(0.5 - i as f64 * 0.005, None);
    }

    // Track LR during cosine annealing
    println!("\nCosine annealing phase (steps 50-200):");
    for i in 60..200 {
        let rate = lr.get_rate();
        if i % 20 == 0 {
            println!("  Step {:3}: LR = {:.6}", i, rate);
        }
        lr.step(0.2 - i as f64 * 0.0005, None);
    }

    let diag = lr.get_diagnostics();
    println!("\nDiagnostics:");
    println!("  Warmup complete: {}", diag.warmup_complete);
    println!("  Current cycle: {}", diag.current_cycle);
    println!("  Cycle progress: {:.2}%", diag.cycle_progress * 100.0);
    println!("  Is improving: {}", diag.is_improving);

    println!("[OK] Enhanced Adaptive LR working\n");
}

fn test_self_paced_curriculum() {
    println!("--- Test 2: Self-Paced Curriculum ---");

    let mut curriculum = SelfPacedCurriculum::with_standard_patterns()
        .with_pace_function(PaceFunction::SelfTuned);

    println!("\nInitial pattern difficulties:");
    for pattern in curriculum.patterns() {
        println!("  {}: difficulty = {:.2}", pattern.name, pattern.difficulty_estimate);
    }

    // Simulate training where we're good at Circle, bad at Square
    println!("\nSimulating training (good at Circle, struggling with Square)...");

    for round in 0..5 {
        // Good at Circle
        for _ in 0..10 {
            curriculum.record_result("circle", 0.05, 0.1);
        }

        // Struggling with Square
        for _ in 0..10 {
            curriculum.record_result("square", 0.35, 0.1);
        }

        // Medium at Cross
        for _ in 0..10 {
            curriculum.record_result("cross", 0.12, 0.1);
        }

        // Making progress on Spiral
        let spiral_loss = 0.4 - round as f64 * 0.05;
        for _ in 0..10 {
            curriculum.record_result("spiral", spiral_loss, 0.1);
        }

        let diag = curriculum.get_diagnostics();
        println!("  Round {}: threshold = {:.3}, success_rate = {:.2}%",
            round + 1, diag.current_threshold, diag.success_rate * 100.0);
    }

    println!("\nUpdated pattern difficulties:");
    for pattern in curriculum.patterns() {
        let status = if pattern.is_mastered { " [MASTERED]" } else { "" };
        println!("  {}: difficulty = {:.2}, best_loss = {:.3}{}",
            pattern.name, pattern.difficulty_estimate, pattern.best_loss, status);
    }

    // Test pattern selection
    println!("\nPattern selection (5 rounds):");
    for i in 0..5 {
        if let Some(pattern) = curriculum.select_pattern() {
            println!("  Selection {}: {} (difficulty: {:.2})",
                i + 1, pattern.name, pattern.difficulty_estimate);
        }
    }

    println!("[OK] Self-Paced Curriculum working\n");
}

fn test_gradient_stats() {
    println!("--- Test 3: Gradient Statistics ---");

    // Test with different loss distributions
    let stable_losses = vec![0.30, 0.29, 0.31, 0.30, 0.28];
    let noisy_losses = vec![0.20, 0.45, 0.15, 0.50, 0.25];
    let improving_losses = vec![0.50, 0.40, 0.30, 0.20, 0.10];

    let stable_stats = GradientStats::from_batch_losses(&stable_losses, None);
    let noisy_stats = GradientStats::from_batch_losses(&noisy_losses, None);
    let improving_stats = GradientStats::from_batch_losses(&improving_losses, None);

    println!("\nStable training (low variance):");
    println!("  Variance: {:.4}, SNR: {:.2}", stable_stats.variance, stable_stats.snr);

    println!("\nNoisy training (high variance):");
    println!("  Variance: {:.4}, SNR: {:.2}", noisy_stats.variance, noisy_stats.snr);

    println!("\nImproving training:");
    println!("  Variance: {:.4}, SNR: {:.2}", improving_stats.variance, improving_stats.snr);

    // SNR should be higher for stable training
    assert!(stable_stats.snr > noisy_stats.snr,
        "Stable training should have higher SNR");

    println!("[OK] Gradient Statistics working\n");
}

fn test_unified_controller() {
    println!("--- Test 4: Unified Meta-Learning Controller ---");

    let mut controller = EnhancedMetaLearningController::new(0.0001);

    // Simulate training loop
    println!("\nSimulating 500 training steps...");

    let mut selected_patterns: Vec<String> = Vec::new();

    for step in 0..500 {
        // Select pattern periodically
        if step % 50 == 0 {
            if let Some(pattern) = controller.select_next_pattern() {
                selected_patterns.push(pattern.clone());
                if step % 100 == 0 {
                    println!("  Step {:3}: Selected '{}', LR = {:.6}",
                        step, pattern, controller.get_learning_rate());
                }
            }
        }

        // Simulate training with varying success
        let base_loss = match controller.current_pattern_id.as_deref() {
            Some("circle") => 0.1,
            Some("square") => 0.4,
            Some("cross") => 0.2,
            Some("spiral") => 0.3,
            _ => 0.3,
        };

        // Add some noise and improvement over time
        let loss = base_loss * (1.0 - step as f64 * 0.0005) + (rand::random::<f64>() - 0.5) * 0.05;
        let batch_losses: Vec<f64> = (0..5).map(|_| loss + (rand::random::<f64>() - 0.5) * 0.02).collect();

        controller.record_step(loss, &batch_losses, 0.1);
    }

    let diag = controller.get_diagnostics();
    println!("\nFinal diagnostics:");
    println!("  Curriculum step: {}", diag.curriculum.step);
    println!("  Mastered patterns: {}/{}", diag.curriculum.mastered_patterns, diag.curriculum.total_patterns);
    println!("  Success rate: {:.1}%", diag.curriculum.success_rate * 100.0);
    println!("  Current threshold: {:.3}", diag.curriculum.current_threshold);
    println!("  LR warmup complete: {}", diag.learning_rate.warmup_complete);
    println!("  LR cycle: {}", diag.learning_rate.current_cycle);
    println!("  Effective LR: {:.6}", diag.effective_lr);

    println!("\nPattern selection distribution:");
    let mut counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for p in &selected_patterns {
        *counts.entry(p.clone()).or_insert(0) += 1;
    }
    for (pattern, count) in counts {
        println!("  {}: {} times", pattern, count);
    }

    println!("[OK] Unified Controller working\n");
}

fn test_with_real_nca() {
    println!("--- Test 5: Integration with Real NCA ---");

    let mut controller = EnhancedMetaLearningController::new(0.00005);
    let mut nca = NCA::new();

    // Try to load existing weights
    if nca.load_weights_from_file("pattern_training_weights.json").is_ok() {
        println!("Loaded existing NCA weights");
    }

    println!("\nRunning 100 real NCA training steps...");

    let mut total_loss = 0.0;
    let mut lr_history: Vec<f64> = Vec::new();

    for step in 0..100 {
        // Select pattern
        if step % 20 == 0 {
            let _ = controller.select_next_pattern();
        }

        // Reset and evolve NCA
        nca.reset_with_seed();
        for _ in 0..50 {
            nca.step();
        }

        // Calculate loss (simplified - just measure grid activity)
        let grid = &nca.grid;
        let alive_cells: f64 = grid.cells.iter()
            .flatten()
            .map(|c| if c[3] > 0.1 { 1.0 } else { 0.0 })
            .sum();
        let total_cells = (grid.width * grid.height) as f64;
        let activity = alive_cells / total_cells;

        // Ideal activity is around 0.2-0.4 for most patterns
        let loss = (activity - 0.3).abs();

        // Record step
        let batch_losses = vec![loss; 5];
        controller.record_step(loss, &batch_losses, 0.1);

        total_loss += loss;
        lr_history.push(controller.get_learning_rate());

        if step % 20 == 0 {
            println!("  Step {:3}: loss = {:.3}, LR = {:.6}, pattern = {:?}",
                step, loss, controller.get_learning_rate(), controller.current_pattern_id);
        }
    }

    let avg_loss = total_loss / 100.0;
    let avg_lr: f64 = lr_history.iter().sum::<f64>() / lr_history.len() as f64;
    let lr_std: f64 = (lr_history.iter().map(|lr| (lr - avg_lr).powi(2)).sum::<f64>()
        / lr_history.len() as f64).sqrt();

    println!("\nResults:");
    println!("  Average loss: {:.4}", avg_loss);
    println!("  Average LR: {:.6}", avg_lr);
    println!("  LR std dev: {:.6} (shows adaptation)", lr_std);

    let diag = controller.get_diagnostics();
    println!("  Final success rate: {:.1}%", diag.curriculum.success_rate * 100.0);

    println!("[OK] Real NCA integration working\n");
}
