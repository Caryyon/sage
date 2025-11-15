// Self-Modification System - SAGE analyzing and improving itself

use crate::learning::meta_learning::MetaLearner;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Performance diagnosis - what's working and what's not
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PerformanceDiagnosis {
    /// Learning is progressing well
    Healthy { reason: String },
    /// Learning has plateaued
    Plateau { reason: String, suggestion: String },
    /// Learning is unstable
    Unstable { reason: String, suggestion: String },
    /// Learning is stuck/failing
    Stuck { reason: String, suggestion: String },
}

/// Hyperparameter configuration
#[derive(Debug, Clone)]
pub struct HyperparameterConfig {
    pub learning_rate: f64,
    pub evolution_steps: usize,
    pub batch_size: usize,
    pub mastery_threshold: f64,
    pub patience_limit: usize,
}

impl Default for HyperparameterConfig {
    fn default() -> Self {
        Self {
            learning_rate: 0.0001,
            evolution_steps: 100,
            batch_size: 5,
            mastery_threshold: 0.1,
            patience_limit: 100,
        }
    }
}

/// Performance introspector - SAGE analyzing its own performance
pub struct PerformanceIntrospector {
    loss_history: Vec<f64>,
    complexity_history: Vec<f64>,
    diversity_history: Vec<f64>,
    generation_history: Vec<u64>,
    pattern_performance: HashMap<String, PatternMetrics>,
}

#[derive(Debug, Clone)]
pub struct PatternMetrics {
    pub pattern_name: String,
    pub attempts: usize,
    pub best_loss: f64,
    pub avg_loss: f64,
    pub improvement_rate: f64,
    pub is_mastered: bool,
}

impl PerformanceIntrospector {
    pub fn new() -> Self {
        Self {
            loss_history: Vec::new(),
            complexity_history: Vec::new(),
            diversity_history: Vec::new(),
            generation_history: Vec::new(),
            pattern_performance: HashMap::new(),
        }
    }

    /// Record a training step
    pub fn record_step(&mut self, generation: u64, loss: f64, complexity: f64, diversity: f64, pattern: &str) {
        self.loss_history.push(loss);
        self.complexity_history.push(complexity);
        self.diversity_history.push(diversity);
        self.generation_history.push(generation);

        // Update pattern metrics
        let entry = self.pattern_performance.entry(pattern.to_string())
            .or_insert(PatternMetrics {
                pattern_name: pattern.to_string(),
                attempts: 0,
                best_loss: f64::MAX,
                avg_loss: loss,
                improvement_rate: 0.0,
                is_mastered: false,
            });

        entry.attempts += 1;
        entry.best_loss = entry.best_loss.min(loss);
        entry.avg_loss = ((entry.avg_loss * (entry.attempts - 1) as f64) + loss) / entry.attempts as f64;

        // Keep last 1000 steps
        if self.loss_history.len() > 1000 {
            self.loss_history.remove(0);
            self.complexity_history.remove(0);
            self.diversity_history.remove(0);
            self.generation_history.remove(0);
        }
    }

    /// Diagnose current performance state
    pub fn diagnose(&self) -> PerformanceDiagnosis {
        if self.loss_history.len() < 20 {
            return PerformanceDiagnosis::Healthy {
                reason: "Just started training, gathering data".to_string()
            };
        }

        let recent_count = 50.min(self.loss_history.len());
        let recent_losses: Vec<f64> = self.loss_history.iter().rev().take(recent_count).copied().collect();

        // Calculate loss trend
        let first_half_avg = recent_losses.iter().skip(recent_count / 2).sum::<f64>() / (recent_count / 2) as f64;
        let second_half_avg = recent_losses.iter().take(recent_count / 2).sum::<f64>() / (recent_count / 2) as f64;
        let improvement = first_half_avg - second_half_avg;

        // Calculate variance (instability)
        let avg_loss = recent_losses.iter().sum::<f64>() / recent_losses.len() as f64;
        let variance = recent_losses.iter()
            .map(|l| (l - avg_loss).powi(2))
            .sum::<f64>() / recent_losses.len() as f64;

        // Diagnose based on patterns
        if variance > 0.05 {
            PerformanceDiagnosis::Unstable {
                reason: format!("High loss variance ({:.4}), learning is oscillating", variance),
                suggestion: "Reduce learning rate to stabilize convergence".to_string()
            }
        } else if improvement.abs() < 0.001 && avg_loss > 0.1 {
            PerformanceDiagnosis::Plateau {
                reason: format!("Loss stuck at {:.4}, no improvement in {} steps", avg_loss, recent_count),
                suggestion: "Try increasing learning rate or switching patterns to build new connections".to_string()
            }
        } else if improvement < -0.01 {
            PerformanceDiagnosis::Stuck {
                reason: format!("Loss increasing (diverging), getting worse not better"),
                suggestion: "Significantly reduce learning rate or reset to last checkpoint".to_string()
            }
        } else if improvement > 0.01 {
            PerformanceDiagnosis::Healthy {
                reason: format!("Loss improving steadily ({:.4} → {:.4})", first_half_avg, second_half_avg)
            }
        } else {
            PerformanceDiagnosis::Healthy {
                reason: "Loss stable and low, performing well".to_string()
            }
        }
    }

    /// Get performance summary
    pub fn get_performance_summary(&self) -> String {
        if self.loss_history.is_empty() {
            return "No performance data yet.".to_string();
        }

        let recent_count = 20.min(self.loss_history.len());
        let recent_avg = self.loss_history.iter().rev().take(recent_count).sum::<f64>() / recent_count as f64;

        let mut summary = format!("Recent avg loss: {:.4}. ", recent_avg);

        // Pattern performance
        let mut patterns: Vec<_> = self.pattern_performance.values().collect();
        patterns.sort_by(|a, b| a.best_loss.partial_cmp(&b.best_loss).unwrap());

        if !patterns.is_empty() {
            summary.push_str("Best patterns: ");
            for (i, pattern) in patterns.iter().take(3).enumerate() {
                if i > 0 {
                    summary.push_str(", ");
                }
                summary.push_str(&format!("{} ({:.3})", pattern.pattern_name, pattern.best_loss));
            }
            summary.push('.');
        }

        summary
    }

    /// Get weakest pattern (needs most work)
    pub fn get_weakest_pattern(&self) -> Option<String> {
        self.pattern_performance.values()
            .filter(|p| !p.is_mastered)
            .max_by(|a, b| a.avg_loss.partial_cmp(&b.avg_loss).unwrap())
            .map(|p| p.pattern_name.clone())
    }

    /// Get strongest pattern
    pub fn get_strongest_pattern(&self) -> Option<String> {
        self.pattern_performance.values()
            .min_by(|a, b| a.best_loss.partial_cmp(&b.best_loss).unwrap())
            .map(|p| p.pattern_name.clone())
    }

    /// Mark pattern as mastered
    pub fn mark_mastered(&mut self, pattern: &str) {
        if let Some(entry) = self.pattern_performance.get_mut(pattern) {
            entry.is_mastered = true;
        }
    }
}

/// Hyperparameter optimizer - SAGE tuning itself
pub struct HyperparameterOptimizer {
    current_config: HyperparameterConfig,
    performance_with_config: Vec<(HyperparameterConfig, f64)>,
    experiment_count: usize,
}

impl HyperparameterOptimizer {
    pub fn new() -> Self {
        Self {
            current_config: HyperparameterConfig::default(),
            performance_with_config: Vec::new(),
            experiment_count: 0,
        }
    }

    /// Suggest hyperparameter adjustment based on diagnosis
    pub fn suggest_adjustment(&mut self, diagnosis: &PerformanceDiagnosis) -> Option<HyperparameterConfig> {
        let mut new_config = self.current_config.clone();

        match diagnosis {
            PerformanceDiagnosis::Unstable { .. } => {
                // Reduce learning rate by 30%
                new_config.learning_rate *= 0.7;
                new_config.learning_rate = new_config.learning_rate.max(0.00001);
                Some(new_config)
            }
            PerformanceDiagnosis::Plateau { .. } => {
                // Try increasing learning rate by 50%
                new_config.learning_rate *= 1.5;
                new_config.learning_rate = new_config.learning_rate.min(0.001);
                Some(new_config)
            }
            PerformanceDiagnosis::Stuck { .. } => {
                // Drastically reduce learning rate
                new_config.learning_rate *= 0.3;
                new_config.learning_rate = new_config.learning_rate.max(0.00001);
                Some(new_config)
            }
            PerformanceDiagnosis::Healthy { .. } => {
                // No change needed
                None
            }
        }
    }

    /// Record performance with current config
    pub fn record_performance(&mut self, avg_loss: f64) {
        self.performance_with_config.push((self.current_config.clone(), avg_loss));
        self.experiment_count += 1;

        // Keep best 20 configurations
        if self.performance_with_config.len() > 20 {
            self.performance_with_config.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
            self.performance_with_config.truncate(20);
        }
    }

    /// Apply new configuration
    pub fn apply_config(&mut self, config: HyperparameterConfig) {
        self.current_config = config;
    }

    /// Get current config
    pub fn get_config(&self) -> &HyperparameterConfig {
        &self.current_config
    }

    /// Get best config found so far
    pub fn get_best_config(&self) -> Option<&HyperparameterConfig> {
        self.performance_with_config.first().map(|(config, _)| config)
    }
}

/// Self-Modification Engine - Combines introspection with optimization
pub struct SelfModificationEngine {
    pub introspector: PerformanceIntrospector,
    pub optimizer: HyperparameterOptimizer,
    pub meta_learner: MetaLearner,
    modification_count: usize,
}

impl SelfModificationEngine {
    pub fn new() -> Self {
        Self {
            introspector: PerformanceIntrospector::new(),
            optimizer: HyperparameterOptimizer::new(),
            meta_learner: MetaLearner::new(0.0001),
            modification_count: 0,
        }
    }

    /// Analyze and potentially modify training approach
    pub fn analyze_and_adapt(&mut self) -> Option<SelfModificationAction> {
        // Diagnose performance
        let diagnosis = self.introspector.diagnose();

        // Get suggested adjustment
        if let Some(new_config) = self.optimizer.suggest_adjustment(&diagnosis) {
            self.modification_count += 1;

            Some(SelfModificationAction {
                action_type: match diagnosis {
                    PerformanceDiagnosis::Unstable { .. } => "Stabilize".to_string(),
                    PerformanceDiagnosis::Plateau { .. } => "Breakthrough".to_string(),
                    PerformanceDiagnosis::Stuck { .. } => "Reset".to_string(),
                    PerformanceDiagnosis::Healthy { .. } => "Optimize".to_string(),
                },
                diagnosis: format!("{:?}", diagnosis),
                old_learning_rate: self.optimizer.get_config().learning_rate,
                new_learning_rate: new_config.learning_rate,
                new_config,
            })
        } else {
            None
        }
    }

    /// Get self-modification summary for LLM context
    pub fn get_modification_summary(&self) -> String {
        let diagnosis = self.introspector.diagnose();
        let perf_summary = self.introspector.get_performance_summary();

        format!(
            "Self-modification: {} adaptations made. Current state: {:?}. {}",
            self.modification_count,
            match diagnosis {
                PerformanceDiagnosis::Healthy { .. } => "Healthy",
                PerformanceDiagnosis::Plateau { .. } => "Plateau",
                PerformanceDiagnosis::Unstable { .. } => "Unstable",
                PerformanceDiagnosis::Stuck { .. } => "Stuck",
            },
            perf_summary
        )
    }

    /// Get weaknesses for introspection
    pub fn get_weaknesses(&self) -> Vec<String> {
        let mut weaknesses = Vec::new();

        if let Some(weak_pattern) = self.introspector.get_weakest_pattern() {
            weaknesses.push(format!("Struggle with {} patterns", weak_pattern));
        }

        let diagnosis = self.introspector.diagnose();
        match diagnosis {
            PerformanceDiagnosis::Plateau { reason, .. } |
            PerformanceDiagnosis::Unstable { reason, .. } |
            PerformanceDiagnosis::Stuck { reason, .. } => {
                weaknesses.push(reason);
            }
            _ => {}
        }

        weaknesses
    }

    /// Get strengths for introspection
    pub fn get_strengths(&self) -> Vec<String> {
        let mut strengths = Vec::new();

        if let Some(strong_pattern) = self.introspector.get_strongest_pattern() {
            strengths.push(format!("Excel at {} patterns", strong_pattern));
        }

        if self.modification_count > 5 {
            strengths.push(format!("Adapted {} times to improve learning", self.modification_count));
        }

        strengths
    }
}

impl Default for SelfModificationEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct SelfModificationAction {
    pub action_type: String,
    pub diagnosis: String,
    pub old_learning_rate: f64,
    pub new_learning_rate: f64,
    pub new_config: HyperparameterConfig,
}
