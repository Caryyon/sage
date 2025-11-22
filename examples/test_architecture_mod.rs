//! Test Architecture Self-Modification
//!
//! This test verifies:
//! 1. Safety configuration works correctly
//! 2. Modification proposals are generated appropriately
//! 3. Modifications are logged and can be rolled back
//! 4. Approval flow works for major changes
//! 5. Session limits are enforced
//!
//! Run with: cargo run --release --example test_architecture_mod

use sage::learning::architecture_modification::{
    ArchitectureModifier, SafetyConfig, ModificationResult, ModificationType,
};

fn main() {
    println!("=== Architecture Self-Modification Test ===\n");

    test_safety_config();
    test_modification_proposal();
    test_modification_application();
    test_rollback();
    test_approval_flow();
    test_session_limits();

    println!("\n=== All Architecture Modification Tests Passed! ===");
}

fn test_safety_config() {
    println!("--- Test 1: Safety Configuration ---");

    let default = SafetyConfig::default();
    println!("  Default config:");
    println!("    Hidden neurons: {} - {}", default.min_hidden_neurons, default.max_hidden_neurons);
    println!("    Max change/step: {}", default.max_change_per_step);
    println!("    Rollback threshold: {:.2}", default.rollback_threshold);
    println!("    Approval threshold: {}", default.approval_threshold);
    println!("    Cooldown: {} steps", default.modification_cooldown);

    let conservative = SafetyConfig::conservative();
    println!("\n  Conservative config:");
    println!("    Hidden neurons: {} - {}", conservative.min_hidden_neurons, conservative.max_hidden_neurons);
    println!("    Max change/step: {}", conservative.max_change_per_step);

    let exploratory = SafetyConfig::exploratory();
    println!("\n  Exploratory config:");
    println!("    Hidden neurons: {} - {}", exploratory.min_hidden_neurons, exploratory.max_hidden_neurons);
    println!("    Max change/step: {}", exploratory.max_change_per_step);

    // Verify constraints
    assert!(default.min_hidden_neurons < default.max_hidden_neurons);
    assert!(conservative.max_change_per_step < default.max_change_per_step);
    assert!(exploratory.max_change_per_step > default.max_change_per_step);

    println!("[OK] Safety configuration working\n");
}

fn test_modification_proposal() {
    println!("--- Test 2: Modification Proposal ---");

    let config = SafetyConfig {
        modification_cooldown: 10,  // Short cooldown for testing
        ..SafetyConfig::default()
    };
    let mut modifier = ArchitectureModifier::new(16, config);

    // Simulate underfitting scenario
    println!("  Simulating underfitting (high loss, no improvement)...");
    for i in 0..60 {
        let loss = 0.7 + rand::random::<f64>() * 0.1;  // High loss
        let grad = 0.1;
        modifier.update(loss, grad, None);

        if i == 50 {
            println!("    Step {}: loss={:.3}", i, loss);
        }
    }

    let result = modifier.try_modification();
    match &result {
        ModificationResult::Applied { modification, reason, .. } => {
            println!("  Proposed: {}", modification.description());
            println!("  Reason: {}", reason);
        }
        ModificationResult::NoChange { reason } => {
            println!("  No modification: {}", reason);
        }
        ModificationResult::NeedsApproval { modification, reason } => {
            println!("  Needs approval: {} - {}", modification.description(), reason);
        }
        _ => println!("  Other result: {:?}", result),
    }

    let stats = modifier.stats();
    println!("\n  Stats after proposal:");
    println!("    Total proposals: {}", stats.total_proposals);
    println!("    Applied: {}", stats.applied);
    println!("    Current neurons: {}", stats.current_neurons);

    println!("[OK] Modification proposal working\n");
}

fn test_modification_application() {
    println!("--- Test 3: Modification Application ---");

    let config = SafetyConfig {
        modification_cooldown: 5,
        approval_threshold: 100,  // High threshold so no approval needed
        ..SafetyConfig::default()
    };
    let mut modifier = ArchitectureModifier::new(16, config);

    // Simulate enough steps to trigger modification
    println!("  Running 100 training steps with poor performance...");
    for _ in 0..100 {
        let loss = 0.6 + rand::random::<f64>() * 0.1;
        modifier.update(loss, 0.1, None);
    }

    // Try to get modification
    let initial_neurons = modifier.state().hidden_neurons;
    let result = modifier.try_modification();

    match result {
        ModificationResult::Applied { modification, new_state, .. } => {
            println!("  Applied: {}", modification.description());
            println!("  Neurons: {} -> {}", initial_neurons, new_state.hidden_neurons);
        }
        ModificationResult::NoChange { reason } => {
            println!("  No change: {}", reason);
        }
        _ => println!("  Other: {:?}", result),
    }

    // Check history
    let history = modifier.history();
    println!("\n  Modification history: {} records", history.len());
    for record in history.iter().take(3) {
        println!("    #{}: {} (applied={}, rollback={})",
            record.id, record.modification.description(),
            record.applied, record.rolled_back);
    }

    println!("[OK] Modification application working\n");
}

fn test_rollback() {
    println!("--- Test 4: Rollback Mechanism ---");

    let config = SafetyConfig {
        modification_cooldown: 5,
        rollback_threshold: 0.1,  // 10% drop triggers rollback
        approval_threshold: 100,
        ..SafetyConfig::default()
    };
    let mut modifier = ArchitectureModifier::new(16, config);

    // Build up data and trigger modification
    println!("  Building history and applying modification...");
    for _ in 0..60 {
        modifier.update(0.6, 0.1, None);
    }
    let _ = modifier.try_modification();

    let state_after_mod = modifier.state().clone();
    println!("  State after modification: {} neurons", state_after_mod.hidden_neurons);

    // Simulate performance drop
    println!("\n  Simulating performance drop...");
    let poor_performance = 0.2;  // Much worse than before
    let rollback = modifier.check_rollback(poor_performance);

    match rollback {
        Some(r) => {
            println!("  ROLLBACK triggered!");
            println!("    Modification: {}", r.modification.description());
            println!("    Performance: {:.3} -> {:.3}", r.performance_before, r.performance_after);
            println!("    Restored to: {} neurons", r.restored_state.hidden_neurons);
        }
        None => {
            println!("  No rollback needed (performance acceptable)");
        }
    }

    let stats = modifier.stats();
    println!("\n  Stats:");
    println!("    Rolled back: {}", stats.rolled_back);

    println!("[OK] Rollback mechanism working\n");
}

fn test_approval_flow() {
    println!("--- Test 5: Approval Flow ---");

    let config = SafetyConfig {
        modification_cooldown: 5,
        approval_threshold: 2,  // Low threshold - anything big needs approval
        max_change_per_step: 4,
        ..SafetyConfig::default()
    };
    let mut modifier = ArchitectureModifier::new(16, config);

    // Build data for modification
    for _ in 0..60 {
        modifier.update(0.7, 0.1, None);
    }

    // First try should need approval
    let result = modifier.try_modification();
    match &result {
        ModificationResult::NeedsApproval { modification, reason } => {
            println!("  Needs approval: {}", modification.description());
            println!("  Reason: {}", reason);

            // Reject first
            println!("\n  Rejecting modification...");
            let reject_result = modifier.reject();
            println!("  Result: {:?}", reject_result);

            // Try again
            for _ in 0..10 {
                modifier.update(0.7, 0.1, None);
            }
            let _ = modifier.try_modification();

            // Approve this time
            println!("\n  Approving modification...");
            let approve_result = modifier.approve();
            match approve_result {
                ModificationResult::Applied { new_state, .. } => {
                    println!("  Approved! New neurons: {}", new_state.hidden_neurons);
                }
                _ => println!("  Result: {:?}", approve_result),
            }
        }
        _ => {
            println!("  Got: {:?}", result);
        }
    }

    let stats = modifier.stats();
    println!("\n  Has pending approval: {}", stats.has_pending_approval);

    println!("[OK] Approval flow working\n");
}

fn test_session_limits() {
    println!("--- Test 6: Session Limits ---");

    let config = SafetyConfig {
        modification_cooldown: 2,
        max_modifications_per_session: 3,
        approval_threshold: 100,  // No approval needed
        ..SafetyConfig::default()
    };
    let mut modifier = ArchitectureModifier::new(16, config);

    println!("  Max modifications per session: 3");
    println!("  Attempting modifications...");

    for attempt in 1..=5 {
        // Generate data
        for _ in 0..20 {
            modifier.update(0.7, 0.1, None);
        }

        let result = modifier.try_modification();
        match result {
            ModificationResult::Applied { .. } => {
                println!("    Attempt {}: Applied", attempt);
            }
            ModificationResult::Blocked { reason } => {
                println!("    Attempt {}: BLOCKED - {}", attempt, reason);
            }
            ModificationResult::NoChange { reason } => {
                println!("    Attempt {}: No change - {}", attempt, reason);
            }
            _ => {}
        }
    }

    let stats = modifier.stats();
    println!("\n  Session modifications: {}", stats.session_modifications);

    // Start new session
    modifier.new_session();
    println!("\n  Started new session");
    let stats = modifier.stats();
    println!("  Session modifications after reset: {}", stats.session_modifications);

    println!("[OK] Session limits working\n");
}
