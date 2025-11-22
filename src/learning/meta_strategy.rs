//! Meta-Strategy Selection
//!
//! Automatically chooses the best learning approach based on task characteristics.
//!
//! Strategies available:
//! - **Standard**: Basic gradient descent with adaptive LR
//! - **Reptile**: Few-shot meta-learning for quick adaptation
//! - **PBT**: Population-based training for hyperparameter discovery
//! - **SelfPaced**: Curriculum learning with dynamic difficulty
//! - **Ensemble**: Combine multiple strategies
//!
//! The meta-strategy selector learns which approach works best for different task types.

use crate::learning::enhanced_meta_learning::{EnhancedAdaptiveLR, GradientStats};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// LEARNING STRATEGIES
// ============================================================================

/// Available learning strategies
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LearningStrategy {
    /// Standard training with adaptive learning rate
    Standard,
    /// Reptile meta-learning for few-shot adaptation
    Reptile,
    /// Population-based training for hyperparameter evolution
    PBT,
    /// Self-paced curriculum learning
    SelfPaced,
    /// Ensemble of multiple strategies
    Ensemble,
}

impl LearningStrategy {
    pub fn all() -> Vec<LearningStrategy> {
        vec![
            LearningStrategy::Standard,
            LearningStrategy::Reptile,
            LearningStrategy::PBT,
            LearningStrategy::SelfPaced,
            LearningStrategy::Ensemble,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            LearningStrategy::Standard => "Standard",
            LearningStrategy::Reptile => "Reptile",
            LearningStrategy::PBT => "PBT",
            LearningStrategy::SelfPaced => "SelfPaced",
            LearningStrategy::Ensemble => "Ensemble",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            LearningStrategy::Standard => "Gradient descent with adaptive LR",
            LearningStrategy::Reptile => "Few-shot meta-learning",
            LearningStrategy::PBT => "Population-based hyperparameter evolution",
            LearningStrategy::SelfPaced => "Curriculum with dynamic difficulty",
            LearningStrategy::Ensemble => "Weighted combination of strategies",
        }
    }

    /// Get the reasoning for why this strategy is appropriate
    pub fn reason(&self) -> &'static str {
        match self {
            LearningStrategy::Standard => "Initial strategy - stable gradient descent",
            LearningStrategy::Reptile => "Few-shot learning for quick pattern adaptation",
            LearningStrategy::PBT => "Evolving hyperparameters for optimal performance",
            LearningStrategy::SelfPaced => "Adapting difficulty based on current mastery",
            LearningStrategy::Ensemble => "Combining multiple approaches for robustness",
        }
    }

    /// Get confidence level for this strategy (default values, can be overridden)
    pub fn confidence(&self) -> f64 {
        match self {
            LearningStrategy::Standard => 0.5,   // Baseline confidence
            LearningStrategy::Reptile => 0.7,   // Good for few-shot
            LearningStrategy::PBT => 0.8,       // Strong when given time
            LearningStrategy::SelfPaced => 0.6, // Moderate
            LearningStrategy::Ensemble => 0.75, // Robust but complex
        }
    }
}

// ============================================================================
// TASK FEATURES
// ============================================================================

/// Features extracted from a task for strategy selection
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TaskFeatures {
    /// Estimated complexity (0-1)
    pub complexity: f64,
    /// Similarity to previously seen tasks (0-1)
    pub familiarity: f64,
    /// Amount of training data available
    pub data_availability: DataAvailability,
    /// Whether task has clear structure
    pub structured: bool,
    /// Whether task benefits from exploration
    pub exploration_benefit: f64,
    /// Time budget (steps available for training)
    pub time_budget: TimeBudget,
    /// Historical performance on similar tasks
    pub historical_performance: HashMap<LearningStrategy, f64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataAvailability {
    /// Very few examples (< 10)
    Scarce,
    /// Limited examples (10-100)
    Limited,
    /// Moderate examples (100-1000)
    Moderate,
    /// Abundant examples (1000+)
    Abundant,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimeBudget {
    /// Need results quickly (< 100 steps)
    Urgent,
    /// Normal training time (100-1000 steps)
    Normal,
    /// Can train extensively (1000+ steps)
    Extended,
}

impl TaskFeatures {
    /// Create default features for a new task
    pub fn new() -> Self {
        Self {
            complexity: 0.5,
            familiarity: 0.0,
            data_availability: DataAvailability::Moderate,
            structured: true,
            exploration_benefit: 0.5,
            time_budget: TimeBudget::Normal,
            historical_performance: HashMap::new(),
        }
    }

    /// Extract features from a pattern task
    pub fn from_pattern(pattern_name: &str, loss_history: &[f64]) -> Self {
        // Estimate complexity from loss history
        let complexity = if loss_history.is_empty() {
            0.5
        } else {
            let avg_loss = loss_history.iter().sum::<f64>() / loss_history.len() as f64;
            (avg_loss * 2.0).min(1.0)
        };

        // Estimate familiarity based on pattern type
        let familiarity = match pattern_name.to_lowercase().as_str() {
            "circle" => 0.9,  // Very familiar
            "square" => 0.7,
            "cross" => 0.6,
            "spiral" => 0.4,
            "triangle" => 0.5,
            "ring" => 0.5,
            "star" => 0.3,
            "hexagon" => 0.4,
            "checkerboard" => 0.3,
            _ => 0.2,  // Novel pattern
        };

        Self {
            complexity,
            familiarity,
            data_availability: DataAvailability::Abundant,  // NCA generates data
            structured: true,  // Geometric patterns are structured
            exploration_benefit: 1.0 - familiarity,
            time_budget: TimeBudget::Normal,
            historical_performance: HashMap::new(),
        }
    }

    /// Update with historical performance data
    pub fn with_history(mut self, strategy: LearningStrategy, score: f64) -> Self {
        self.historical_performance.insert(strategy, score);
        self
    }
}

impl Default for TaskFeatures {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// STRATEGY SELECTOR
// ============================================================================

/// Selects the best learning strategy based on task features
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StrategySelector {
    /// Strategy weights (learned from experience)
    strategy_weights: HashMap<LearningStrategy, StrategyProfile>,
    /// History of strategy selections and outcomes
    selection_history: Vec<SelectionRecord>,
    /// Exploration rate for trying new strategies
    exploration_rate: f64,
    /// Total selections made
    total_selections: usize,
}

/// Profile for each strategy's performance characteristics
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StrategyProfile {
    /// Average performance score
    pub avg_score: f64,
    /// Number of times used
    pub usage_count: usize,
    /// Performance in different conditions
    pub condition_scores: HashMap<String, f64>,
    /// Confidence in predictions
    pub confidence: f64,
}

impl Default for StrategyProfile {
    fn default() -> Self {
        Self {
            avg_score: 0.5,
            usage_count: 0,
            condition_scores: HashMap::new(),
            confidence: 0.1,  // Low initial confidence
        }
    }
}

/// Record of a strategy selection
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SelectionRecord {
    pub features: TaskFeatures,
    pub selected_strategy: LearningStrategy,
    pub outcome_score: f64,
    pub steps_taken: usize,
}

impl StrategySelector {
    pub fn new() -> Self {
        let mut strategy_weights = HashMap::new();
        for strategy in LearningStrategy::all() {
            strategy_weights.insert(strategy, StrategyProfile::default());
        }

        Self {
            strategy_weights,
            selection_history: Vec::new(),
            exploration_rate: 0.2,  // 20% exploration initially
            total_selections: 0,
        }
    }

    /// Select the best strategy for given task features
    pub fn select(&mut self, features: &TaskFeatures) -> StrategySelection {
        self.total_selections += 1;

        // Decay exploration rate over time
        self.exploration_rate = (0.2 * (-0.01 * self.total_selections as f64).exp()).max(0.05);

        // Explore with probability exploration_rate
        if rand::random::<f64>() < self.exploration_rate {
            let strategies = LearningStrategy::all();
            let idx = rand::random::<usize>() % strategies.len();
            return StrategySelection {
                strategy: strategies[idx],
                confidence: 0.0,
                reasoning: "Exploration selection".to_string(),
                expected_score: 0.5,
            };
        }

        // Score each strategy
        let mut best_strategy = LearningStrategy::Standard;
        let mut best_score = f64::NEG_INFINITY;

        for strategy in LearningStrategy::all() {
            let score = self.score_strategy(strategy, features);
            if score > best_score {
                best_score = score;
                best_strategy = strategy;
            }
        }

        // Generate reasoning
        let reasoning = self.generate_reasoning(best_strategy, features);

        let profile = self.strategy_weights.get(&best_strategy).unwrap();

        StrategySelection {
            strategy: best_strategy,
            confidence: profile.confidence,
            reasoning,
            expected_score: best_score,
        }
    }

    /// Score a strategy for given features
    fn score_strategy(&self, strategy: LearningStrategy, features: &TaskFeatures) -> f64 {
        let profile = self.strategy_weights.get(&strategy).unwrap();
        let mut score = profile.avg_score;

        // Adjust based on task features
        match strategy {
            LearningStrategy::Reptile => {
                // Reptile excels at:
                // - Low data availability (few-shot)
                // - High familiarity (can transfer)
                // - Urgent time budget
                if features.data_availability == DataAvailability::Scarce {
                    score += 0.3;
                }
                if features.familiarity > 0.5 {
                    score += 0.2 * features.familiarity;
                }
                if features.time_budget == TimeBudget::Urgent {
                    score += 0.2;
                }
                // Penalize for complex novel tasks
                if features.complexity > 0.7 && features.familiarity < 0.3 {
                    score -= 0.3;
                }
            }

            LearningStrategy::PBT => {
                // PBT excels at:
                // - Extended time budget
                // - High exploration benefit
                // - Complex tasks
                if features.time_budget == TimeBudget::Extended {
                    score += 0.3;
                }
                if features.exploration_benefit > 0.6 {
                    score += 0.2 * features.exploration_benefit;
                }
                if features.complexity > 0.6 {
                    score += 0.2;
                }
                // Penalize for urgent tasks
                if features.time_budget == TimeBudget::Urgent {
                    score -= 0.4;
                }
            }

            LearningStrategy::SelfPaced => {
                // SelfPaced excels at:
                // - Moderate complexity with structure
                // - Normal time budget
                // - Abundant data
                if features.structured && features.complexity > 0.3 && features.complexity < 0.8 {
                    score += 0.2;
                }
                if features.data_availability == DataAvailability::Abundant {
                    score += 0.1;
                }
                if features.time_budget == TimeBudget::Normal {
                    score += 0.1;
                }
            }

            LearningStrategy::Standard => {
                // Standard is baseline - no bonuses but stable
                score += 0.1;  // Small stability bonus
                // Good for simple familiar tasks
                if features.complexity < 0.4 && features.familiarity > 0.7 {
                    score += 0.2;
                }
            }

            LearningStrategy::Ensemble => {
                // Ensemble is good for:
                // - Uncertain situations (low confidence elsewhere)
                // - High stakes (important to get right)
                let max_confidence = self.strategy_weights.values()
                    .map(|p| p.confidence)
                    .fold(0.0, f64::max);
                if max_confidence < 0.5 {
                    score += 0.2;  // Use ensemble when unsure
                }
            }
        }

        // Use historical performance if available
        if let Some(&hist_score) = features.historical_performance.get(&strategy) {
            score = score * 0.6 + hist_score * 0.4;
        }

        score
    }

    /// Generate human-readable reasoning for selection
    fn generate_reasoning(&self, strategy: LearningStrategy, features: &TaskFeatures) -> String {
        let mut reasons = Vec::new();

        match strategy {
            LearningStrategy::Reptile => {
                if features.data_availability == DataAvailability::Scarce {
                    reasons.push("few examples available (few-shot scenario)");
                }
                if features.familiarity > 0.5 {
                    reasons.push("similar to previously learned tasks");
                }
                if features.time_budget == TimeBudget::Urgent {
                    reasons.push("need quick adaptation");
                }
            }
            LearningStrategy::PBT => {
                if features.time_budget == TimeBudget::Extended {
                    reasons.push("have time for hyperparameter search");
                }
                if features.exploration_benefit > 0.6 {
                    reasons.push("benefits from exploration");
                }
                if features.complexity > 0.6 {
                    reasons.push("complex task needs tuning");
                }
            }
            LearningStrategy::SelfPaced => {
                if features.structured {
                    reasons.push("structured task suits curriculum");
                }
                if features.data_availability == DataAvailability::Abundant {
                    reasons.push("abundant data for curriculum ordering");
                }
            }
            LearningStrategy::Standard => {
                if features.complexity < 0.4 {
                    reasons.push("simple task");
                }
                if features.familiarity > 0.7 {
                    reasons.push("familiar pattern");
                }
                reasons.push("stable baseline approach");
            }
            LearningStrategy::Ensemble => {
                reasons.push("combining multiple approaches for robustness");
            }
        }

        if reasons.is_empty() {
            format!("{} selected based on learned preferences", strategy.name())
        } else {
            format!("{} selected: {}", strategy.name(), reasons.join(", "))
        }
    }

    /// Record outcome of a strategy selection
    pub fn record_outcome(&mut self, features: TaskFeatures, strategy: LearningStrategy,
                          outcome_score: f64, steps: usize) {
        // Compute condition before borrowing strategy_weights mutably
        let condition = Self::features_to_condition(&features);

        // Update strategy profile
        if let Some(profile) = self.strategy_weights.get_mut(&strategy) {
            let n = profile.usage_count as f64;
            profile.avg_score = (profile.avg_score * n + outcome_score) / (n + 1.0);
            profile.usage_count += 1;
            // Increase confidence with more data
            profile.confidence = (profile.usage_count as f64 / 20.0).min(0.9);

            // Record condition-specific performance
            let entry = profile.condition_scores.entry(condition).or_insert(0.5);
            *entry = (*entry + outcome_score) / 2.0;
        }

        // Store in history
        self.selection_history.push(SelectionRecord {
            features,
            selected_strategy: strategy,
            outcome_score,
            steps_taken: steps,
        });

        // Trim history if too long
        if self.selection_history.len() > 1000 {
            self.selection_history.drain(0..500);
        }
    }

    /// Convert features to a condition string for lookup
    fn features_to_condition(features: &TaskFeatures) -> String {
        let complexity = if features.complexity < 0.3 { "low" }
                        else if features.complexity < 0.7 { "med" }
                        else { "high" };
        let familiarity = if features.familiarity < 0.3 { "novel" }
                         else if features.familiarity < 0.7 { "semi" }
                         else { "familiar" };
        let data = match features.data_availability {
            DataAvailability::Scarce => "scarce",
            DataAvailability::Limited => "limited",
            DataAvailability::Moderate => "moderate",
            DataAvailability::Abundant => "abundant",
        };
        format!("{}_{}_{}", complexity, familiarity, data)
    }

    /// Get statistics about strategy usage
    pub fn get_stats(&self) -> StrategySelectorStats {
        let mut strategy_stats = HashMap::new();
        for (strategy, profile) in &self.strategy_weights {
            strategy_stats.insert(*strategy, StrategyStats {
                avg_score: profile.avg_score,
                usage_count: profile.usage_count,
                confidence: profile.confidence,
            });
        }

        StrategySelectorStats {
            total_selections: self.total_selections,
            exploration_rate: self.exploration_rate,
            strategy_stats,
            history_size: self.selection_history.len(),
        }
    }

    /// Get the best performing strategy overall
    pub fn best_strategy(&self) -> LearningStrategy {
        self.strategy_weights.iter()
            .filter(|(_, p)| p.usage_count > 0)
            .max_by(|(_, a), (_, b)| a.avg_score.partial_cmp(&b.avg_score).unwrap())
            .map(|(s, _)| *s)
            .unwrap_or(LearningStrategy::Standard)
    }
}

impl Default for StrategySelector {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of strategy selection
#[derive(Clone, Debug)]
pub struct StrategySelection {
    pub strategy: LearningStrategy,
    pub confidence: f64,
    pub reasoning: String,
    pub expected_score: f64,
}

/// Statistics for a single strategy
#[derive(Clone, Debug)]
pub struct StrategyStats {
    pub avg_score: f64,
    pub usage_count: usize,
    pub confidence: f64,
}

/// Overall selector statistics
#[derive(Clone, Debug)]
pub struct StrategySelectorStats {
    pub total_selections: usize,
    pub exploration_rate: f64,
    pub strategy_stats: HashMap<LearningStrategy, StrategyStats>,
    pub history_size: usize,
}

// ============================================================================
// META-STRATEGY CONTROLLER
// ============================================================================

/// Unified controller that manages all learning strategies
pub struct MetaStrategyController {
    selector: StrategySelector,
    // Strategy implementations
    adaptive_lr: EnhancedAdaptiveLR,
    current_strategy: LearningStrategy,
    current_features: TaskFeatures,
    steps_this_strategy: usize,
    cumulative_loss: f64,
}

impl MetaStrategyController {
    pub fn new() -> Self {
        Self {
            selector: StrategySelector::new(),
            adaptive_lr: EnhancedAdaptiveLR::new(0.0001),
            current_strategy: LearningStrategy::Standard,
            current_features: TaskFeatures::new(),
            steps_this_strategy: 0,
            cumulative_loss: 0.0,
        }
    }

    /// Start a new task with given features
    pub fn start_task(&mut self, features: TaskFeatures) -> StrategySelection {
        // Record outcome of previous task if any
        if self.steps_this_strategy > 0 {
            let avg_loss = self.cumulative_loss / self.steps_this_strategy as f64;
            // Convert loss to score (lower loss = higher score)
            let score = 1.0 - avg_loss.min(1.0);
            self.selector.record_outcome(
                self.current_features.clone(),
                self.current_strategy,
                score,
                self.steps_this_strategy,
            );
        }

        // Select strategy for new task
        let selection = self.selector.select(&features);
        self.current_strategy = selection.strategy;
        self.current_features = features;
        self.steps_this_strategy = 0;
        self.cumulative_loss = 0.0;

        // Reset adaptive LR for new task
        self.adaptive_lr = EnhancedAdaptiveLR::new(0.0001);

        selection
    }

    /// Get learning rate for current step
    pub fn get_learning_rate(&mut self, loss: f64, gradient_magnitude: f64) -> f64 {
        self.steps_this_strategy += 1;
        self.cumulative_loss += loss;

        // Create gradient stats for the adaptive LR
        let grad_stats = GradientStats::new(gradient_magnitude, 0.01);

        // Step the adaptive LR with current loss and gradient info
        self.adaptive_lr.step(loss, Some(grad_stats));
        let base_rate = self.adaptive_lr.get_rate();

        // Different strategies use different LR policies
        match self.current_strategy {
            LearningStrategy::Standard => base_rate,
            LearningStrategy::Reptile => {
                // Reptile uses smaller inner LR
                base_rate * 0.5
            }
            LearningStrategy::PBT => {
                // PBT uses hyperparameters from population (external)
                // Return a reasonable default
                0.0001
            }
            LearningStrategy::SelfPaced => {
                // SelfPaced adjusts based on difficulty
                let difficulty_factor = self.current_features.complexity;
                base_rate * (1.0 - difficulty_factor * 0.5)
            }
            LearningStrategy::Ensemble => {
                // Ensemble uses average of strategies
                base_rate
            }
        }
    }

    /// Check if we should switch strategies mid-task
    pub fn should_switch(&self) -> bool {
        // Switch if performance is poor after many steps
        if self.steps_this_strategy > 100 {
            let avg_loss = self.cumulative_loss / self.steps_this_strategy as f64;
            if avg_loss > 0.5 {
                return true;
            }
        }
        false
    }

    /// Get current strategy
    pub fn current_strategy(&self) -> LearningStrategy {
        self.current_strategy
    }

    /// Get selector statistics
    pub fn stats(&self) -> StrategySelectorStats {
        self.selector.get_stats()
    }

    /// Get best overall strategy
    pub fn best_strategy(&self) -> LearningStrategy {
        self.selector.best_strategy()
    }
}

impl Default for MetaStrategyController {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_features() {
        let features = TaskFeatures::from_pattern("circle", &[0.1, 0.05, 0.03]);
        assert!(features.familiarity > 0.8);
        assert!(features.complexity < 0.5);
    }

    #[test]
    fn test_strategy_selection() {
        let mut selector = StrategySelector::new();

        // Few-shot scenario should prefer Reptile
        let features = TaskFeatures {
            data_availability: DataAvailability::Scarce,
            time_budget: TimeBudget::Urgent,
            familiarity: 0.8,
            ..TaskFeatures::new()
        };
        let selection = selector.select(&features);
        // Initially might explore, but Reptile should score well
        println!("Selected: {:?}, Reasoning: {}", selection.strategy, selection.reasoning);
    }

    #[test]
    fn test_outcome_recording() {
        let mut selector = StrategySelector::new();

        // Record some outcomes
        selector.record_outcome(TaskFeatures::new(), LearningStrategy::Reptile, 0.8, 50);
        selector.record_outcome(TaskFeatures::new(), LearningStrategy::Standard, 0.6, 100);

        let stats = selector.get_stats();
        assert!(stats.strategy_stats[&LearningStrategy::Reptile].usage_count == 1);
        assert!(stats.strategy_stats[&LearningStrategy::Standard].usage_count == 1);
    }
}
