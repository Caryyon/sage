//! Architecture Self-Modification
//!
//! Allows SAGE to modify its own network structure with safety constraints.
//!
//! Safety measures:
//! 1. All modifications logged with timestamp and reason
//! 2. Automatic rollback if performance degrades
//! 3. Bounds on network size (min/max neurons)
//! 4. Approval required for major changes
//! 5. Gradual changes only (no drastic restructuring)
//!
//! Modification types:
//! - Add hidden neurons (increase capacity)
//! - Remove unused neurons (pruning)
//! - Adjust layer widths
//! - Modify activation scaling

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

// ============================================================================
// SAFETY CONFIGURATION
// ============================================================================

/// Safety configuration for architecture modifications
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SafetyConfig {
    /// Minimum number of hidden neurons
    pub min_hidden_neurons: usize,
    /// Maximum number of hidden neurons
    pub max_hidden_neurons: usize,
    /// Maximum change per modification (neurons)
    pub max_change_per_step: usize,
    /// Minimum performance threshold (below = rollback)
    pub min_performance: f64,
    /// Performance drop that triggers rollback
    pub rollback_threshold: f64,
    /// Require approval for changes beyond this size
    pub approval_threshold: usize,
    /// Cooldown steps between modifications
    pub modification_cooldown: usize,
    /// Maximum modifications per session
    pub max_modifications_per_session: usize,
}

impl Default for SafetyConfig {
    fn default() -> Self {
        Self {
            min_hidden_neurons: 8,
            max_hidden_neurons: 64,
            max_change_per_step: 4,
            min_performance: 0.3,
            rollback_threshold: 0.2,
            approval_threshold: 8,
            modification_cooldown: 100,
            max_modifications_per_session: 10,
        }
    }
}

impl SafetyConfig {
    /// Conservative settings (very safe)
    pub fn conservative() -> Self {
        Self {
            min_hidden_neurons: 12,
            max_hidden_neurons: 32,
            max_change_per_step: 2,
            min_performance: 0.4,
            rollback_threshold: 0.1,
            approval_threshold: 4,
            modification_cooldown: 200,
            max_modifications_per_session: 5,
        }
    }

    /// Exploratory settings (more freedom)
    pub fn exploratory() -> Self {
        Self {
            min_hidden_neurons: 4,
            max_hidden_neurons: 128,
            max_change_per_step: 8,
            min_performance: 0.2,
            rollback_threshold: 0.3,
            approval_threshold: 16,
            modification_cooldown: 50,
            max_modifications_per_session: 20,
        }
    }
}

// ============================================================================
// MODIFICATION TYPES
// ============================================================================

/// Types of architecture modifications
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum ModificationType {
    /// Add neurons to hidden layer
    AddNeurons { count: usize, layer: usize },
    /// Remove neurons from hidden layer
    RemoveNeurons { count: usize, layer: usize },
    /// Scale activation function
    ScaleActivation { factor: f64 },
    /// Adjust dropout rate
    AdjustDropout { new_rate: f64 },
    /// No modification (observation only)
    None,
}

impl ModificationType {
    pub fn magnitude(&self) -> usize {
        match self {
            ModificationType::AddNeurons { count, .. } => *count,
            ModificationType::RemoveNeurons { count, .. } => *count,
            ModificationType::ScaleActivation { .. } => 1,
            ModificationType::AdjustDropout { .. } => 1,
            ModificationType::None => 0,
        }
    }

    pub fn is_major(&self, threshold: usize) -> bool {
        self.magnitude() >= threshold
    }

    pub fn description(&self) -> String {
        match self {
            ModificationType::AddNeurons { count, layer } =>
                format!("Add {} neurons to layer {}", count, layer),
            ModificationType::RemoveNeurons { count, layer } =>
                format!("Remove {} neurons from layer {}", count, layer),
            ModificationType::ScaleActivation { factor } =>
                format!("Scale activations by {:.2}", factor),
            ModificationType::AdjustDropout { new_rate } =>
                format!("Set dropout rate to {:.2}", new_rate),
            ModificationType::None => "No modification".to_string(),
        }
    }
}

// ============================================================================
// MODIFICATION LOG
// ============================================================================

/// Record of a modification attempt
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ModificationRecord {
    /// Unique ID
    pub id: usize,
    /// Timestamp (step number)
    pub timestamp: usize,
    /// Type of modification
    pub modification: ModificationType,
    /// Reason for modification
    pub reason: String,
    /// Performance before modification
    pub performance_before: f64,
    /// Performance after modification
    pub performance_after: Option<f64>,
    /// Whether modification was applied
    pub applied: bool,
    /// Whether modification was rolled back
    pub rolled_back: bool,
    /// Architecture state before (for rollback)
    pub state_before: ArchitectureState,
}

/// Snapshot of architecture state
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ArchitectureState {
    pub hidden_neurons: usize,
    pub activation_scale: f64,
    pub dropout_rate: f64,
    pub total_parameters: usize,
}

impl ArchitectureState {
    pub fn new(hidden_neurons: usize) -> Self {
        Self {
            hidden_neurons,
            activation_scale: 1.0,
            dropout_rate: 0.0,
            // Approximate: input * hidden + hidden * output
            total_parameters: 48 * hidden_neurons + hidden_neurons * 16,
        }
    }
}

// ============================================================================
// MODIFICATION PROPOSER
// ============================================================================

/// Proposes architecture modifications based on training metrics
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ModificationProposer {
    /// Current architecture state
    current_state: ArchitectureState,
    /// Recent performance history
    performance_history: VecDeque<f64>,
    /// Gradient magnitude history
    gradient_history: VecDeque<f64>,
    /// Whether currently underfitting
    underfitting_detected: bool,
    /// Whether currently overfitting
    overfitting_detected: bool,
    /// Steps since last modification
    steps_since_modification: usize,
}

impl ModificationProposer {
    pub fn new(initial_hidden: usize) -> Self {
        Self {
            current_state: ArchitectureState::new(initial_hidden),
            performance_history: VecDeque::with_capacity(100),
            gradient_history: VecDeque::with_capacity(100),
            underfitting_detected: false,
            overfitting_detected: false,
            steps_since_modification: 0,
        }
    }

    /// Update with new training metrics
    pub fn update(&mut self, loss: f64, gradient_magnitude: f64, validation_loss: Option<f64>) {
        self.steps_since_modification += 1;

        // Track performance (inverse of loss)
        let performance = 1.0 - loss.min(1.0);
        self.performance_history.push_back(performance);
        if self.performance_history.len() > 100 {
            self.performance_history.pop_front();
        }

        self.gradient_history.push_back(gradient_magnitude);
        if self.gradient_history.len() > 100 {
            self.gradient_history.pop_front();
        }

        // Detect underfitting: high training loss, not improving
        if self.performance_history.len() >= 20 {
            let recent: Vec<f64> = self.performance_history.iter()
                .rev().take(20).cloned().collect();
            let old: Vec<f64> = self.performance_history.iter()
                .rev().skip(20).take(20).cloned().collect();

            if !old.is_empty() {
                let recent_avg = recent.iter().sum::<f64>() / recent.len() as f64;
                let old_avg = old.iter().sum::<f64>() / old.len() as f64;

                // Underfitting: low performance, not improving
                self.underfitting_detected = recent_avg < 0.5 && recent_avg <= old_avg + 0.02;
            }
        }

        // Detect overfitting: training improves but validation doesn't
        if let Some(val_loss) = validation_loss {
            let train_perf = 1.0 - loss.min(1.0);
            let val_perf = 1.0 - val_loss.min(1.0);
            self.overfitting_detected = train_perf > val_perf + 0.15;
        }
    }

    /// Propose a modification based on current state
    pub fn propose(&self, config: &SafetyConfig) -> ProposedModification {
        // Check cooldown
        if self.steps_since_modification < config.modification_cooldown {
            return ProposedModification {
                modification: ModificationType::None,
                reason: format!("Cooldown: {} steps remaining",
                    config.modification_cooldown - self.steps_since_modification),
                confidence: 0.0,
                requires_approval: false,
            };
        }

        // Check if we have enough data
        if self.performance_history.len() < 50 {
            return ProposedModification {
                modification: ModificationType::None,
                reason: "Insufficient data for modification decision".to_string(),
                confidence: 0.0,
                requires_approval: false,
            };
        }

        // Calculate current average performance
        let avg_perf: f64 = self.performance_history.iter().sum::<f64>()
            / self.performance_history.len() as f64;

        // Calculate gradient variance (indicator of learning dynamics)
        let grad_mean = self.gradient_history.iter().sum::<f64>()
            / self.gradient_history.len() as f64;
        let grad_var = self.gradient_history.iter()
            .map(|g| (g - grad_mean).powi(2))
            .sum::<f64>() / self.gradient_history.len() as f64;

        // Decision logic
        if self.underfitting_detected && self.current_state.hidden_neurons < config.max_hidden_neurons {
            // Underfitting: add capacity
            let neurons_to_add = (config.max_change_per_step / 2).max(2)
                .min(config.max_hidden_neurons - self.current_state.hidden_neurons);

            return ProposedModification {
                modification: ModificationType::AddNeurons {
                    count: neurons_to_add,
                    layer: 0,
                },
                reason: format!("Underfitting detected (avg_perf={:.3}), increasing capacity", avg_perf),
                confidence: 0.7,
                requires_approval: neurons_to_add >= config.approval_threshold,
            };
        }

        if self.overfitting_detected && self.current_state.hidden_neurons > config.min_hidden_neurons {
            // Overfitting: reduce capacity or add regularization
            let neurons_to_remove = (config.max_change_per_step / 2).max(1)
                .min(self.current_state.hidden_neurons - config.min_hidden_neurons);

            return ProposedModification {
                modification: ModificationType::RemoveNeurons {
                    count: neurons_to_remove,
                    layer: 0,
                },
                reason: "Overfitting detected, reducing capacity".to_string(),
                confidence: 0.6,
                requires_approval: neurons_to_remove >= config.approval_threshold,
            };
        }

        // High gradient variance: scale down activations
        if grad_var > 0.1 {
            return ProposedModification {
                modification: ModificationType::ScaleActivation { factor: 0.9 },
                reason: format!("High gradient variance ({:.4}), stabilizing", grad_var),
                confidence: 0.5,
                requires_approval: false,
            };
        }

        // Low gradient variance with low performance: scale up activations
        if grad_var < 0.001 && avg_perf < 0.7 {
            return ProposedModification {
                modification: ModificationType::ScaleActivation { factor: 1.1 },
                reason: format!("Low gradients ({:.4}), increasing sensitivity", grad_var),
                confidence: 0.4,
                requires_approval: false,
            };
        }

        // No modification needed
        ProposedModification {
            modification: ModificationType::None,
            reason: format!("Stable performance (avg={:.3}), no modification needed", avg_perf),
            confidence: 1.0,
            requires_approval: false,
        }
    }

    /// Reset steps since modification
    pub fn reset_cooldown(&mut self) {
        self.steps_since_modification = 0;
    }

    /// Get current state
    pub fn state(&self) -> &ArchitectureState {
        &self.current_state
    }

    /// Update current state after modification
    pub fn update_state(&mut self, new_state: ArchitectureState) {
        self.current_state = new_state;
    }
}

/// A proposed modification with reasoning
#[derive(Clone, Debug)]
pub struct ProposedModification {
    pub modification: ModificationType,
    pub reason: String,
    pub confidence: f64,
    pub requires_approval: bool,
}

// ============================================================================
// ARCHITECTURE MODIFIER
// ============================================================================

/// Manages architecture modifications with safety constraints
pub struct ArchitectureModifier {
    /// Safety configuration
    config: SafetyConfig,
    /// Modification proposer
    proposer: ModificationProposer,
    /// Modification history log
    history: Vec<ModificationRecord>,
    /// Next modification ID
    next_id: usize,
    /// Current step
    current_step: usize,
    /// Modifications this session
    modifications_this_session: usize,
    /// Pending approval queue
    pending_approval: Option<ProposedModification>,
    /// Last performance (for rollback check)
    last_performance: f64,
}

impl ArchitectureModifier {
    pub fn new(initial_hidden: usize, config: SafetyConfig) -> Self {
        Self {
            config,
            proposer: ModificationProposer::new(initial_hidden),
            history: Vec::new(),
            next_id: 0,
            current_step: 0,
            modifications_this_session: 0,
            pending_approval: None,
            last_performance: 0.5,
        }
    }

    /// Update with training metrics
    pub fn update(&mut self, loss: f64, gradient_magnitude: f64, validation_loss: Option<f64>) {
        self.current_step += 1;
        self.proposer.update(loss, gradient_magnitude, validation_loss);
        self.last_performance = 1.0 - loss.min(1.0);
    }

    /// Try to get a modification (may require approval)
    pub fn try_modification(&mut self) -> ModificationResult {
        // Check session limit
        if self.modifications_this_session >= self.config.max_modifications_per_session {
            return ModificationResult::Blocked {
                reason: "Session modification limit reached".to_string(),
            };
        }

        // Check for pending approval
        if let Some(pending) = &self.pending_approval {
            return ModificationResult::NeedsApproval {
                modification: pending.modification.clone(),
                reason: pending.reason.clone(),
            };
        }

        // Get proposal
        let proposal = self.proposer.propose(&self.config);

        if proposal.modification == ModificationType::None {
            return ModificationResult::NoChange {
                reason: proposal.reason,
            };
        }

        // Check if needs approval
        if proposal.requires_approval {
            self.pending_approval = Some(proposal.clone());
            return ModificationResult::NeedsApproval {
                modification: proposal.modification,
                reason: proposal.reason,
            };
        }

        // Apply modification
        self.apply_modification(proposal)
    }

    /// Approve pending modification
    pub fn approve(&mut self) -> ModificationResult {
        if let Some(proposal) = self.pending_approval.take() {
            self.apply_modification(proposal)
        } else {
            ModificationResult::NoChange {
                reason: "No pending modification".to_string(),
            }
        }
    }

    /// Reject pending modification
    pub fn reject(&mut self) -> ModificationResult {
        if let Some(proposal) = self.pending_approval.take() {
            ModificationResult::Rejected {
                modification: proposal.modification,
                reason: "Rejected by user".to_string(),
            }
        } else {
            ModificationResult::NoChange {
                reason: "No pending modification".to_string(),
            }
        }
    }

    /// Apply a modification
    fn apply_modification(&mut self, proposal: ProposedModification) -> ModificationResult {
        let state_before = self.proposer.state().clone();

        // Create new state based on modification
        let new_state = match &proposal.modification {
            ModificationType::AddNeurons { count, .. } => {
                ArchitectureState::new(state_before.hidden_neurons + count)
            }
            ModificationType::RemoveNeurons { count, .. } => {
                ArchitectureState::new(state_before.hidden_neurons.saturating_sub(*count))
            }
            ModificationType::ScaleActivation { factor } => {
                let mut state = state_before.clone();
                state.activation_scale *= factor;
                state
            }
            ModificationType::AdjustDropout { new_rate } => {
                let mut state = state_before.clone();
                state.dropout_rate = *new_rate;
                state
            }
            ModificationType::None => state_before.clone(),
        };

        // Validate new state
        if new_state.hidden_neurons < self.config.min_hidden_neurons {
            return ModificationResult::Blocked {
                reason: format!("Would go below minimum neurons ({})",
                    self.config.min_hidden_neurons),
            };
        }
        if new_state.hidden_neurons > self.config.max_hidden_neurons {
            return ModificationResult::Blocked {
                reason: format!("Would exceed maximum neurons ({})",
                    self.config.max_hidden_neurons),
            };
        }

        // Create record
        let record = ModificationRecord {
            id: self.next_id,
            timestamp: self.current_step,
            modification: proposal.modification.clone(),
            reason: proposal.reason.clone(),
            performance_before: self.last_performance,
            performance_after: None,
            applied: true,
            rolled_back: false,
            state_before,
        };

        self.history.push(record);
        self.next_id += 1;
        self.modifications_this_session += 1;
        self.proposer.reset_cooldown();
        self.proposer.update_state(new_state.clone());

        ModificationResult::Applied {
            modification: proposal.modification,
            reason: proposal.reason,
            new_state,
        }
    }

    /// Check if we should rollback the last modification
    pub fn check_rollback(&mut self, current_performance: f64) -> Option<RollbackResult> {
        // Get last applied, non-rolled-back modification
        let last_idx = self.history.iter().rposition(|r| r.applied && !r.rolled_back)?;
        let last_record = &mut self.history[last_idx];

        // Update performance_after
        last_record.performance_after = Some(current_performance);

        // Check if performance dropped too much
        let perf_drop = last_record.performance_before - current_performance;
        if perf_drop > self.config.rollback_threshold {
            // Perform rollback
            let old_state = last_record.state_before.clone();
            last_record.rolled_back = true;
            self.proposer.update_state(old_state.clone());

            return Some(RollbackResult {
                modification_id: last_record.id,
                modification: last_record.modification.clone(),
                performance_before: last_record.performance_before,
                performance_after: current_performance,
                restored_state: old_state,
            });
        }

        None
    }

    /// Get modification history
    pub fn history(&self) -> &[ModificationRecord] {
        &self.history
    }

    /// Get current state
    pub fn state(&self) -> &ArchitectureState {
        self.proposer.state()
    }

    /// Get statistics
    pub fn stats(&self) -> ModifierStats {
        let applied = self.history.iter().filter(|r| r.applied).count();
        let rolled_back = self.history.iter().filter(|r| r.rolled_back).count();

        ModifierStats {
            total_proposals: self.history.len(),
            applied,
            rolled_back,
            current_neurons: self.proposer.state().hidden_neurons,
            session_modifications: self.modifications_this_session,
            has_pending_approval: self.pending_approval.is_some(),
        }
    }

    /// Reset session counter (call at session start)
    pub fn new_session(&mut self) {
        self.modifications_this_session = 0;
    }
}

/// Result of a modification attempt
#[derive(Clone, Debug)]
pub enum ModificationResult {
    Applied {
        modification: ModificationType,
        reason: String,
        new_state: ArchitectureState,
    },
    NeedsApproval {
        modification: ModificationType,
        reason: String,
    },
    Blocked {
        reason: String,
    },
    Rejected {
        modification: ModificationType,
        reason: String,
    },
    NoChange {
        reason: String,
    },
}

/// Result of a rollback
#[derive(Clone, Debug)]
pub struct RollbackResult {
    pub modification_id: usize,
    pub modification: ModificationType,
    pub performance_before: f64,
    pub performance_after: f64,
    pub restored_state: ArchitectureState,
}

/// Statistics about modifications
#[derive(Clone, Debug)]
pub struct ModifierStats {
    pub total_proposals: usize,
    pub applied: usize,
    pub rolled_back: usize,
    pub current_neurons: usize,
    pub session_modifications: usize,
    pub has_pending_approval: bool,
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safety_config() {
        let config = SafetyConfig::default();
        assert!(config.min_hidden_neurons < config.max_hidden_neurons);
        assert!(config.max_change_per_step > 0);
    }

    #[test]
    fn test_modification_magnitude() {
        let add = ModificationType::AddNeurons { count: 5, layer: 0 };
        assert_eq!(add.magnitude(), 5);

        let scale = ModificationType::ScaleActivation { factor: 1.1 };
        assert_eq!(scale.magnitude(), 1);
    }

    #[test]
    fn test_proposer_cooldown() {
        let proposer = ModificationProposer::new(16);
        let config = SafetyConfig::default();
        let proposal = proposer.propose(&config);

        // Should be in cooldown initially
        assert_eq!(proposal.modification, ModificationType::None);
    }
}
