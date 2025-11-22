//! Enhanced Meta-Learning System
//!
//! Phase A of the meta-learning roadmap:
//! - SelfPacedCurriculum: Dynamic difficulty-based pattern selection
//! - EnhancedAdaptiveLR: Warmup + cosine annealing + gradient-aware adaptation
//! - GradientStats: Learning signal quality tracking
//!
//! These systems enable SAGE to learn *how to learn* more effectively.

use std::collections::{HashMap, VecDeque};
use std::f64::consts::PI;
use serde::{Deserialize, Serialize};

// ============================================================================
// GRADIENT STATISTICS
// ============================================================================

/// Statistics about gradient quality for adaptive learning rate
#[derive(Debug, Clone, Default)]
pub struct GradientStats {
    /// Gradient magnitude: ||nabla L||
    pub magnitude: f64,
    /// Variance of gradient across batch
    pub variance: f64,
    /// Signal-to-noise ratio (higher = clearer learning signal)
    pub snr: f64,
    /// Direction consistency with previous gradient
    pub direction_consistency: f64,
}

impl GradientStats {
    pub fn new(magnitude: f64, variance: f64) -> Self {
        let snr = if variance > 1e-10 {
            magnitude / variance.sqrt()
        } else {
            10.0 // Default high SNR if no variance
        };

        Self {
            magnitude,
            variance,
            snr,
            direction_consistency: 1.0,
        }
    }

    /// Compute stats from a batch of loss values
    pub fn from_batch_losses(losses: &[f64], prev_losses: Option<&[f64]>) -> Self {
        if losses.is_empty() {
            return Self::default();
        }

        let mean_loss: f64 = losses.iter().sum::<f64>() / losses.len() as f64;
        let variance: f64 = losses.iter()
            .map(|l| (l - mean_loss).powi(2))
            .sum::<f64>() / losses.len() as f64;

        // Approximate gradient magnitude from loss change
        let magnitude = if let Some(prev) = prev_losses {
            let prev_mean: f64 = prev.iter().sum::<f64>() / prev.len() as f64;
            (prev_mean - mean_loss).abs()
        } else {
            mean_loss
        };

        Self::new(magnitude, variance)
    }
}

// ============================================================================
// ENHANCED ADAPTIVE LEARNING RATE
// ============================================================================

/// Enhanced learning rate scheduler with warmup, cosine annealing, and gradient-awareness
#[derive(Debug, Clone)]
pub struct EnhancedAdaptiveLR {
    /// Base learning rate (target after warmup)
    base_rate: f64,
    /// Current effective learning rate
    current_rate: f64,
    /// Minimum learning rate floor
    min_rate: f64,
    /// Maximum learning rate ceiling
    max_rate: f64,

    // Warmup configuration
    warmup_steps: usize,
    warmup_complete: bool,

    // Cosine annealing with warm restarts (SGDR)
    cycle_length: usize,
    cycle_mult: f64,      // Multiply cycle length after each restart
    current_cycle: usize,
    steps_in_cycle: usize,

    // Gradient-aware adaptation
    grad_history: VecDeque<GradientStats>,
    grad_history_size: usize,

    // Loss-based adaptation (fallback)
    loss_history: VecDeque<f64>,
    patience: usize,
    patience_counter: usize,

    // State
    current_step: usize,
}

impl EnhancedAdaptiveLR {
    pub fn new(base_rate: f64) -> Self {
        Self {
            base_rate,
            current_rate: base_rate,
            min_rate: base_rate * 0.01,
            max_rate: base_rate * 10.0,

            warmup_steps: 100,
            warmup_complete: false,

            cycle_length: 500,
            cycle_mult: 1.5,
            current_cycle: 0,
            steps_in_cycle: 0,

            grad_history: VecDeque::new(),
            grad_history_size: 20,

            loss_history: VecDeque::new(),
            patience: 15,
            patience_counter: 0,

            current_step: 0,
        }
    }

    /// Configure warmup period
    pub fn with_warmup(mut self, steps: usize) -> Self {
        self.warmup_steps = steps;
        self
    }

    /// Configure cycle length for cosine annealing
    pub fn with_cycle_length(mut self, length: usize) -> Self {
        self.cycle_length = length;
        self
    }

    /// Get current learning rate with all scheduling applied
    pub fn get_rate(&self) -> f64 {
        // Warmup phase: linear increase from 0 to base_rate
        if !self.warmup_complete && self.current_step < self.warmup_steps {
            let warmup_progress = self.current_step as f64 / self.warmup_steps as f64;
            return self.base_rate * warmup_progress;
        }

        // Cosine annealing with warm restarts
        let effective_cycle_length = (self.cycle_length as f64
            * self.cycle_mult.powi(self.current_cycle as i32)) as usize;
        let cycle_progress = self.steps_in_cycle as f64 / effective_cycle_length as f64;

        // Cosine factor: 1.0 at start of cycle, 0.0 at end
        let cosine_factor = 0.5 * (1.0 + (PI * cycle_progress).cos());

        // Apply to current rate (which may have been adapted by gradient stats)
        self.min_rate + (self.current_rate - self.min_rate) * cosine_factor
    }

    /// Step the scheduler and optionally update based on gradient stats
    pub fn step(&mut self, loss: f64, grad_stats: Option<GradientStats>) {
        self.current_step += 1;
        self.steps_in_cycle += 1;

        // Track loss history
        self.loss_history.push_back(loss);
        if self.loss_history.len() > 30 {
            self.loss_history.pop_front();
        }

        // Check warmup completion
        if !self.warmup_complete && self.current_step >= self.warmup_steps {
            self.warmup_complete = true;
        }

        // Check for cycle restart
        let effective_cycle_length = (self.cycle_length as f64
            * self.cycle_mult.powi(self.current_cycle as i32)) as usize;
        if self.steps_in_cycle >= effective_cycle_length {
            self.steps_in_cycle = 0;
            self.current_cycle += 1;
            // On restart, optionally boost the base rate if making progress
            if self.is_improving() {
                self.current_rate = (self.current_rate * 1.1).min(self.max_rate);
            }
        }

        // Gradient-aware adaptation
        if let Some(stats) = grad_stats {
            self.adapt_from_gradients(stats);
        } else {
            // Fallback: loss-based adaptation
            self.adapt_from_loss();
        }
    }

    /// Adapt learning rate based on gradient signal quality
    fn adapt_from_gradients(&mut self, stats: GradientStats) {
        self.grad_history.push_back(stats);
        if self.grad_history.len() > self.grad_history_size {
            self.grad_history.pop_front();
        }

        if self.grad_history.len() < 10 {
            return;
        }

        // Calculate average SNR
        let avg_snr: f64 = self.grad_history.iter()
            .map(|s| s.snr)
            .sum::<f64>() / self.grad_history.len() as f64;

        // High SNR = clear gradient signal = can use higher LR
        // Low SNR = noisy gradients = need lower LR
        if avg_snr > 5.0 {
            self.current_rate = (self.current_rate * 1.05).min(self.max_rate);
        } else if avg_snr < 1.0 {
            self.current_rate = (self.current_rate * 0.9).max(self.min_rate);
        }

        // Also check gradient magnitude trend
        let recent_mags: Vec<f64> = self.grad_history.iter()
            .rev().take(5)
            .map(|s| s.magnitude)
            .collect();
        let older_mags: Vec<f64> = self.grad_history.iter()
            .skip(self.grad_history.len().saturating_sub(10))
            .take(5)
            .map(|s| s.magnitude)
            .collect();

        if !recent_mags.is_empty() && !older_mags.is_empty() {
            let recent_avg: f64 = recent_mags.iter().sum::<f64>() / recent_mags.len() as f64;
            let older_avg: f64 = older_mags.iter().sum::<f64>() / older_mags.len() as f64;

            // If gradients are shrinking (approaching minimum), reduce LR
            if recent_avg < older_avg * 0.5 {
                self.current_rate = (self.current_rate * 0.95).max(self.min_rate);
            }
        }
    }

    /// Fallback: adapt based on loss trend
    fn adapt_from_loss(&mut self) {
        if self.loss_history.len() < 15 {
            return;
        }

        let recent: Vec<f64> = self.loss_history.iter().rev().take(7).copied().collect();
        let older: Vec<f64> = self.loss_history.iter().rev().skip(7).take(7).copied().collect();

        let recent_avg: f64 = recent.iter().sum::<f64>() / recent.len() as f64;
        let older_avg: f64 = older.iter().sum::<f64>() / older.len() as f64;

        let improvement = older_avg - recent_avg;

        if improvement < 0.0001 {
            // Not improving
            self.patience_counter += 1;
            if self.patience_counter >= self.patience {
                self.current_rate = (self.current_rate * 0.7).max(self.min_rate);
                self.patience_counter = 0;
            }
        } else {
            self.patience_counter = 0;
            // Good improvement - slightly increase LR
            if improvement > 0.01 {
                self.current_rate = (self.current_rate * 1.05).min(self.max_rate);
            }
        }
    }

    /// Check if learning is improving
    fn is_improving(&self) -> bool {
        if self.loss_history.len() < 10 {
            return true;
        }

        let recent: Vec<f64> = self.loss_history.iter().rev().take(5).copied().collect();
        let older: Vec<f64> = self.loss_history.iter().rev().skip(5).take(5).copied().collect();

        let recent_avg: f64 = recent.iter().sum::<f64>() / recent.len() as f64;
        let older_avg: f64 = older.iter().sum::<f64>() / older.len() as f64;

        recent_avg < older_avg
    }

    /// Reset for new pattern
    pub fn reset_for_pattern(&mut self, pattern_difficulty: f64) {
        self.loss_history.clear();
        self.grad_history.clear();
        self.patience_counter = 0;

        // Adjust base rate based on pattern difficulty
        // Harder patterns might need different starting rates
        let difficulty_factor = 1.0 + (pattern_difficulty - 0.5) * 0.5;
        self.current_rate = self.base_rate * difficulty_factor;
    }

    /// Get diagnostic info
    pub fn get_diagnostics(&self) -> LRDiagnostics {
        let avg_snr = if !self.grad_history.is_empty() {
            self.grad_history.iter().map(|s| s.snr).sum::<f64>() / self.grad_history.len() as f64
        } else {
            0.0
        };

        let warmup_progress = if self.warmup_complete {
            1.0
        } else {
            self.current_step as f64 / self.warmup_steps.max(1) as f64
        };

        LRDiagnostics {
            current_rate: self.get_rate(),
            base_rate: self.base_rate,
            warmup_complete: self.warmup_complete,
            warmup_progress,
            current_cycle: self.current_cycle,
            cycle_progress: self.steps_in_cycle as f64 / self.cycle_length as f64,
            avg_gradient_snr: avg_snr,
            is_improving: self.is_improving(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct LRDiagnostics {
    pub current_rate: f64,
    pub base_rate: f64,
    pub warmup_complete: bool,
    pub warmup_progress: f64,  // Progress through warmup phase (0-1)
    pub current_cycle: usize,
    pub cycle_progress: f64,
    pub avg_gradient_snr: f64,
    pub is_improving: bool,
}

// ============================================================================
// SELF-PACED CURRICULUM
// ============================================================================

/// A pattern task for curriculum learning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternTask {
    pub id: String,
    pub name: String,
    pub difficulty_estimate: f64,  // 0.0 = easy, 1.0 = hard
    pub times_attempted: usize,
    pub times_mastered: usize,
    pub best_loss: f64,
    pub recent_losses: VecDeque<f64>,
    pub is_mastered: bool,
}

impl PatternTask {
    pub fn new(id: &str, name: &str, initial_difficulty: f64) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            difficulty_estimate: initial_difficulty,
            times_attempted: 0,
            times_mastered: 0,
            best_loss: f64::MAX,
            recent_losses: VecDeque::new(),
            is_mastered: false,
        }
    }

    /// Update with a training result
    pub fn record_result(&mut self, loss: f64) {
        self.times_attempted += 1;
        self.best_loss = self.best_loss.min(loss);

        self.recent_losses.push_back(loss);
        if self.recent_losses.len() > 20 {
            self.recent_losses.pop_front();
        }

        // Update difficulty estimate based on actual performance
        self.update_difficulty();
    }

    /// Get current average loss
    pub fn get_current_loss(&self) -> f64 {
        if self.recent_losses.is_empty() {
            return f64::MAX;
        }
        self.recent_losses.iter().sum::<f64>() / self.recent_losses.len() as f64
    }

    /// Update difficulty estimate based on learning progress
    fn update_difficulty(&mut self) {
        if self.recent_losses.len() < 5 {
            return;
        }

        let avg_loss = self.get_current_loss();

        // Difficulty is based on:
        // 1. Current loss (higher loss = harder)
        // 2. Improvement rate (slower improvement = harder)
        // 3. Attempt count (more attempts needed = harder)

        let loss_factor = (avg_loss * 2.0).min(1.0);

        let improvement_factor = if self.recent_losses.len() >= 10 {
            let old: f64 = self.recent_losses.iter().take(5).sum::<f64>() / 5.0;
            let new: f64 = self.recent_losses.iter().rev().take(5).sum::<f64>() / 5.0;
            let improvement = old - new;
            // Low or negative improvement = harder
            (1.0 - improvement * 10.0).clamp(0.0, 1.0)
        } else {
            0.5
        };

        let attempt_factor = (self.times_attempted as f64 / 1000.0).min(1.0);

        // Weighted combination
        self.difficulty_estimate =
            0.5 * loss_factor +
            0.3 * improvement_factor +
            0.2 * attempt_factor;
    }

    /// Check if pattern is "learnable" at current threshold
    pub fn is_learnable(&self, threshold: f64) -> bool {
        self.get_current_loss() < threshold || self.recent_losses.is_empty()
    }
}

/// Pace function for curriculum progression
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum PaceFunction {
    /// Linear threshold increase
    Linear { rate: f64 },
    /// Logarithmic (fast start, slow later)
    Logarithmic { base: f64, scale: f64 },
    /// Self-tuned based on success rate
    SelfTuned,
}

impl Default for PaceFunction {
    fn default() -> Self {
        PaceFunction::SelfTuned
    }
}

/// Self-paced curriculum learning system
#[derive(Debug, Clone)]
pub struct SelfPacedCurriculum {
    /// All available patterns
    patterns: Vec<PatternTask>,
    /// Current difficulty threshold (lambda)
    difficulty_threshold: f64,
    /// Initial threshold
    initial_threshold: f64,
    /// Maximum threshold
    max_threshold: f64,
    /// Pace function for threshold progression
    pace_function: PaceFunction,
    /// Success rate window
    success_history: VecDeque<bool>,
    /// Current step
    step: usize,
    /// Recently selected patterns (to avoid repetition)
    recent_selections: VecDeque<String>,
}

impl SelfPacedCurriculum {
    pub fn new() -> Self {
        Self {
            patterns: Vec::new(),
            difficulty_threshold: 0.3,
            initial_threshold: 0.3,
            max_threshold: 2.0,
            pace_function: PaceFunction::SelfTuned,
            success_history: VecDeque::new(),
            step: 0,
            recent_selections: VecDeque::new(),
        }
    }

    /// Add a pattern to the curriculum
    pub fn add_pattern(&mut self, pattern: PatternTask) {
        self.patterns.push(pattern);
    }

    /// Initialize with standard NCA patterns
    pub fn with_standard_patterns() -> Self {
        let mut curriculum = Self::new();
        curriculum.add_pattern(PatternTask::new("circle", "Circle", 0.2));
        curriculum.add_pattern(PatternTask::new("square", "Square", 0.6));
        curriculum.add_pattern(PatternTask::new("cross", "Cross", 0.5));
        curriculum.add_pattern(PatternTask::new("spiral", "Spiral", 0.7));
        curriculum
    }

    /// Set pace function
    pub fn with_pace_function(mut self, pace: PaceFunction) -> Self {
        self.pace_function = pace;
        self
    }

    /// Select next pattern based on self-paced learning
    pub fn select_pattern(&mut self) -> Option<&PatternTask> {
        if self.patterns.is_empty() {
            return None;
        }

        // Find patterns that are "learnable" (loss < threshold)
        let learnable_indices: Vec<usize> = self.patterns.iter()
            .enumerate()
            .filter(|(_, p)| p.is_learnable(self.difficulty_threshold))
            .map(|(i, _)| i)
            .collect();

        if learnable_indices.is_empty() {
            // No patterns below threshold - pick the easiest one
            let easiest_idx = self.patterns.iter()
                .enumerate()
                .min_by(|(_, a), (_, b)| {
                    a.get_current_loss().partial_cmp(&b.get_current_loss()).unwrap()
                })
                .map(|(i, _)| i)?;

            self.recent_selections.push_back(self.patterns[easiest_idx].id.clone());
            if self.recent_selections.len() > 5 {
                self.recent_selections.pop_front();
            }

            return Some(&self.patterns[easiest_idx]);
        }

        // Among learnable patterns, prefer:
        // 1. Not recently selected
        // 2. Higher difficulty (at the edge of ability)
        // 3. Not yet mastered

        let mut scored: Vec<(usize, f64)> = learnable_indices.iter()
            .map(|&idx| {
                let pattern = &self.patterns[idx];
                let mut score = pattern.difficulty_estimate;

                // Bonus for not recently selected
                if !self.recent_selections.contains(&pattern.id) {
                    score += 0.2;
                }

                // Bonus for not yet mastered
                if !pattern.is_mastered {
                    score += 0.3;
                }

                // Penalty for too many attempts without progress
                if pattern.times_attempted > 500 && pattern.get_current_loss() > 0.2 {
                    score -= 0.2;
                }

                (idx, score)
            })
            .collect();

        // Sort by score descending
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        // Pick the highest scoring pattern
        let selected_idx = scored[0].0;

        self.recent_selections.push_back(self.patterns[selected_idx].id.clone());
        if self.recent_selections.len() > 5 {
            self.recent_selections.pop_front();
        }

        Some(&self.patterns[selected_idx])
    }

    /// Get pattern by ID (mutable)
    pub fn get_pattern_mut(&mut self, id: &str) -> Option<&mut PatternTask> {
        self.patterns.iter_mut().find(|p| p.id == id)
    }

    /// Record training result
    pub fn record_result(&mut self, pattern_id: &str, loss: f64, mastery_threshold: f64) {
        let was_success = loss < mastery_threshold;
        self.success_history.push_back(was_success);
        if self.success_history.len() > 50 {
            self.success_history.pop_front();
        }

        if let Some(pattern) = self.get_pattern_mut(pattern_id) {
            pattern.record_result(loss);
            if loss < mastery_threshold {
                pattern.times_mastered += 1;
                if pattern.times_mastered >= 10 {
                    pattern.is_mastered = true;
                }
            }
        }

        self.step += 1;
        self.update_threshold();
    }

    /// Update difficulty threshold based on pace function
    fn update_threshold(&mut self) {
        match self.pace_function {
            PaceFunction::Linear { rate } => {
                self.difficulty_threshold = (self.initial_threshold + rate * self.step as f64)
                    .min(self.max_threshold);
            }
            PaceFunction::Logarithmic { base, scale } => {
                self.difficulty_threshold = (self.initial_threshold
                    + scale * (1.0 + self.step as f64 / base).ln())
                    .min(self.max_threshold);
            }
            PaceFunction::SelfTuned => {
                let success_rate = self.get_success_rate();

                // High success rate -> increase threshold (harder patterns)
                // Low success rate -> decrease threshold (easier patterns)
                if success_rate > 0.7 {
                    self.difficulty_threshold = (self.difficulty_threshold * 1.02)
                        .min(self.max_threshold);
                } else if success_rate < 0.3 {
                    self.difficulty_threshold = (self.difficulty_threshold * 0.98)
                        .max(self.initial_threshold * 0.5);
                }
            }
        }
    }

    /// Get current success rate
    pub fn get_success_rate(&self) -> f64 {
        if self.success_history.is_empty() {
            return 0.5;
        }
        let successes = self.success_history.iter().filter(|&&s| s).count();
        successes as f64 / self.success_history.len() as f64
    }

    /// Get curriculum diagnostics
    pub fn get_diagnostics(&self) -> CurriculumDiagnostics {
        let mastered_count = self.patterns.iter().filter(|p| p.is_mastered).count();
        let avg_difficulty: f64 = if !self.patterns.is_empty() {
            self.patterns.iter().map(|p| p.difficulty_estimate).sum::<f64>()
                / self.patterns.len() as f64
        } else {
            0.0
        };

        CurriculumDiagnostics {
            total_patterns: self.patterns.len(),
            mastered_patterns: mastered_count,
            current_threshold: self.difficulty_threshold,
            success_rate: self.get_success_rate(),
            avg_pattern_difficulty: avg_difficulty,
            step: self.step,
        }
    }

    /// Get all patterns
    pub fn patterns(&self) -> &[PatternTask] {
        &self.patterns
    }

    /// Get current threshold
    pub fn threshold(&self) -> f64 {
        self.difficulty_threshold
    }
}

impl Default for SelfPacedCurriculum {
    fn default() -> Self {
        Self::with_standard_patterns()
    }
}

#[derive(Debug, Clone)]
pub struct CurriculumDiagnostics {
    pub total_patterns: usize,
    pub mastered_patterns: usize,
    pub current_threshold: f64,
    pub success_rate: f64,
    pub avg_pattern_difficulty: f64,
    pub step: usize,
}

// ============================================================================
// UNIFIED CONTROLLER
// ============================================================================

/// Unified controller combining all Level 1 meta-learning systems
pub struct EnhancedMetaLearningController {
    pub curriculum: SelfPacedCurriculum,
    pub adaptive_lr: EnhancedAdaptiveLR,
    pub pattern_lr_multipliers: HashMap<String, f64>,
    pub current_pattern_id: Option<String>,
}

impl EnhancedMetaLearningController {
    pub fn new(base_lr: f64) -> Self {
        let mut multipliers = HashMap::new();
        multipliers.insert("circle".to_string(), 1.0);
        multipliers.insert("square".to_string(), 2.0);    // Square needs higher LR
        multipliers.insert("cross".to_string(), 1.6);
        multipliers.insert("spiral".to_string(), 1.2);

        Self {
            curriculum: SelfPacedCurriculum::with_standard_patterns(),
            adaptive_lr: EnhancedAdaptiveLR::new(base_lr)
                .with_warmup(100)
                .with_cycle_length(500),
            pattern_lr_multipliers: multipliers,
            current_pattern_id: None,
        }
    }

    /// Select next pattern and prepare learning rate
    pub fn select_next_pattern(&mut self) -> Option<String> {
        let pattern = self.curriculum.select_pattern()?;
        let pattern_id = pattern.id.clone();
        let difficulty = pattern.difficulty_estimate;

        // Reset LR for new pattern
        self.adaptive_lr.reset_for_pattern(difficulty);
        self.current_pattern_id = Some(pattern_id.clone());

        Some(pattern_id)
    }

    /// Get current learning rate (with pattern multiplier)
    pub fn get_learning_rate(&self) -> f64 {
        let base_rate = self.adaptive_lr.get_rate();

        if let Some(ref pattern_id) = self.current_pattern_id {
            let multiplier = self.pattern_lr_multipliers
                .get(pattern_id)
                .copied()
                .unwrap_or(1.0);
            base_rate * multiplier
        } else {
            base_rate
        }
    }

    /// Record training step result
    pub fn record_step(&mut self, loss: f64, batch_losses: &[f64], mastery_threshold: f64) {
        // Compute gradient stats from batch
        let grad_stats = GradientStats::from_batch_losses(batch_losses, None);

        // Update adaptive LR
        self.adaptive_lr.step(loss, Some(grad_stats));

        // Update curriculum
        if let Some(ref pattern_id) = self.current_pattern_id {
            self.curriculum.record_result(pattern_id, loss, mastery_threshold);
        }
    }

    /// Get combined diagnostics
    pub fn get_diagnostics(&self) -> MetaLearningDiagnostics {
        MetaLearningDiagnostics {
            curriculum: self.curriculum.get_diagnostics(),
            learning_rate: self.adaptive_lr.get_diagnostics(),
            current_pattern: self.current_pattern_id.clone(),
            effective_lr: self.get_learning_rate(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MetaLearningDiagnostics {
    pub curriculum: CurriculumDiagnostics,
    pub learning_rate: LRDiagnostics,
    pub current_pattern: Option<String>,
    pub effective_lr: f64,
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enhanced_adaptive_lr() {
        let mut lr = EnhancedAdaptiveLR::new(0.001);

        // Warmup phase
        for i in 0..50 {
            lr.step(1.0 - i as f64 * 0.01, None);
        }

        let rate = lr.get_rate();
        assert!(rate < 0.001, "Should be in warmup phase: {}", rate);

        // Complete warmup
        for i in 50..150 {
            lr.step(0.5 - i as f64 * 0.001, None);
        }

        assert!(lr.warmup_complete, "Warmup should be complete");
    }

    #[test]
    fn test_self_paced_curriculum() {
        let mut curriculum = SelfPacedCurriculum::with_standard_patterns();

        // Initially should select easier patterns
        let pattern = curriculum.select_pattern().unwrap();
        assert!(!pattern.id.is_empty());

        // Record some results
        for _ in 0..20 {
            curriculum.record_result("circle", 0.05, 0.1);  // Good at circle
            curriculum.record_result("square", 0.4, 0.1);   // Struggling with square
        }

        // Curriculum should adapt
        let diag = curriculum.get_diagnostics();
        assert!(diag.success_rate > 0.0);
    }

    #[test]
    fn test_gradient_stats() {
        let losses = vec![0.5, 0.48, 0.52, 0.47, 0.49];
        let stats = GradientStats::from_batch_losses(&losses, None);

        assert!(stats.snr > 0.0, "SNR should be positive");
        assert!(stats.variance > 0.0, "Variance should be positive");
    }

    #[test]
    fn test_unified_controller() {
        let mut controller = EnhancedMetaLearningController::new(0.001);

        // Select pattern
        let pattern = controller.select_next_pattern();
        assert!(pattern.is_some());

        // Record steps
        for _ in 0..100 {
            let batch_losses = vec![0.3, 0.28, 0.32, 0.29, 0.31];
            controller.record_step(0.3, &batch_losses, 0.1);
        }

        let diag = controller.get_diagnostics();
        assert!(diag.effective_lr > 0.0);
    }
}
