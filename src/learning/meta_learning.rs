// Meta-learning: Learning how to learn more efficiently

use std::collections::VecDeque;

/// Tracks learning performance over time
#[derive(Debug, Clone)]
pub struct LearningPerformance {
    pub step: usize,
    pub loss: f64,
    pub accuracy: f64,
    pub learning_rate: f64,
    pub timestamp: std::time::Instant,
}

/// Adaptive learning rate controller
pub struct AdaptiveLearningRate {
    current_rate: f64,
    min_rate: f64,
    max_rate: f64,
    recent_losses: VecDeque<f64>,
    patience: usize,
    patience_counter: usize,
    improvement_threshold: f64,
}

impl AdaptiveLearningRate {
    pub fn new(initial_rate: f64, min_rate: f64, max_rate: f64) -> Self {
        Self {
            current_rate: initial_rate,
            min_rate,
            max_rate,
            recent_losses: VecDeque::new(),
            patience: 10,
            patience_counter: 0,
            improvement_threshold: 0.001,
        }
    }

    /// Update learning rate based on loss trend
    pub fn update(&mut self, loss: f64) -> f64 {
        self.recent_losses.push_back(loss);
        if self.recent_losses.len() > 20 {
            self.recent_losses.pop_front();
        }

        if self.recent_losses.len() < 10 {
            return self.current_rate;
        }

        // Check if we're improving
        let recent_avg: f64 = self.recent_losses.iter().rev().take(5).sum::<f64>() / 5.0;
        let old_avg: f64 = self.recent_losses.iter().take(5).sum::<f64>() / 5.0;

        let improvement = old_avg - recent_avg;

        if improvement < self.improvement_threshold {
            // Not improving enough
            self.patience_counter += 1;

            if self.patience_counter >= self.patience {
                // Reduce learning rate (silent mode for TUI)
                self.current_rate *= 0.5;
                self.current_rate = self.current_rate.max(self.min_rate);
                self.patience_counter = 0;
            }
        } else {
            // Good improvement, reset patience
            self.patience_counter = 0;

            // Optionally increase learning rate slightly if consistently improving
            if improvement > self.improvement_threshold * 2.0 {
                self.current_rate *= 1.05;
                self.current_rate = self.current_rate.min(self.max_rate);
            }
        }

        self.current_rate
    }

    pub fn get_rate(&self) -> f64 {
        self.current_rate
    }

    pub fn reset(&mut self, new_rate: f64) {
        self.current_rate = new_rate;
        self.recent_losses.clear();
        self.patience_counter = 0;
    }
}

/// Estimates task difficulty based on learning progress
pub struct TaskDifficultyEstimator {
    task_history: Vec<TaskPerformance>,
}

#[derive(Debug, Clone)]
pub struct TaskPerformance {
    pub task_id: String,
    pub initial_loss: f64,
    pub final_loss: f64,
    pub iterations: usize,
    pub convergence_rate: f64,
    pub difficulty_score: f64,
}

impl TaskDifficultyEstimator {
    pub fn new() -> Self {
        Self {
            task_history: Vec::new(),
        }
    }

    /// Estimate difficulty of a task based on learning characteristics
    pub fn estimate_difficulty(&self, initial_loss: f64, _learning_rate: f64) -> f64 {
        // Higher initial loss = harder task
        let loss_component = (initial_loss / 2.0).min(1.0);

        // If we have history, compare to similar tasks
        let history_component = if !self.task_history.is_empty() {
            let avg_initial: f64 = self.task_history.iter()
                .map(|t| t.initial_loss)
                .sum::<f64>() / self.task_history.len() as f64;

            (initial_loss / avg_initial.max(0.1)).min(2.0) / 2.0
        } else {
            0.5
        };

        // Combine components
        0.6 * loss_component + 0.4 * history_component
    }

    /// Record task performance
    pub fn record_task(&mut self, task_id: String, initial_loss: f64, final_loss: f64, iterations: usize) {
        let convergence_rate = if iterations > 0 {
            (initial_loss - final_loss) / iterations as f64
        } else {
            0.0
        };

        let difficulty_score = self.estimate_difficulty(initial_loss, 0.01);

        self.task_history.push(TaskPerformance {
            task_id,
            initial_loss,
            final_loss,
            iterations,
            convergence_rate,
            difficulty_score,
        });

        // Keep last 100 tasks
        if self.task_history.len() > 100 {
            self.task_history.remove(0);
        }
    }

    /// Get average difficulty of recent tasks
    pub fn average_recent_difficulty(&self, count: usize) -> f64 {
        if self.task_history.is_empty() {
            return 0.5;
        }

        let recent_count = count.min(self.task_history.len());
        let recent: Vec<_> = self.task_history.iter().rev().take(recent_count).collect();

        recent.iter().map(|t| t.difficulty_score).sum::<f64>() / recent.len() as f64
    }
}

/// Curriculum learning - orders tasks from easy to hard
pub struct CurriculumLearner {
    difficulty_estimator: TaskDifficultyEstimator,
    current_difficulty_target: f64,
    adaptation_rate: f64,
}

impl CurriculumLearner {
    pub fn new() -> Self {
        Self {
            difficulty_estimator: TaskDifficultyEstimator::new(),
            current_difficulty_target: 0.3,  // Start with easier tasks
            adaptation_rate: 0.05,
        }
    }

    /// Determine if a task is appropriate for current curriculum level
    pub fn should_attempt_task(&self, estimated_difficulty: f64) -> bool {
        // Allow tasks within a range around current target
        let lower_bound = (self.current_difficulty_target - 0.2).max(0.0);
        let upper_bound = (self.current_difficulty_target + 0.2).min(1.0);

        estimated_difficulty >= lower_bound && estimated_difficulty <= upper_bound
    }

    /// Update curriculum based on recent performance
    pub fn update_curriculum(&mut self, recent_success_rate: f64) {
        // If doing well, increase difficulty target
        if recent_success_rate > 0.7 {
            self.current_difficulty_target += self.adaptation_rate;
            self.current_difficulty_target = self.current_difficulty_target.min(1.0);
        } else if recent_success_rate < 0.3 {
            // If struggling, decrease difficulty target
            self.current_difficulty_target -= self.adaptation_rate;
            self.current_difficulty_target = self.current_difficulty_target.max(0.1);
        }
    }

    pub fn get_target_difficulty(&self) -> f64 {
        self.current_difficulty_target
    }

    pub fn record_task_result(&mut self, task_id: String, initial_loss: f64, final_loss: f64, iterations: usize) {
        self.difficulty_estimator.record_task(task_id, initial_loss, final_loss, iterations);
    }

    pub fn estimate_difficulty(&self, initial_loss: f64, learning_rate: f64) -> f64 {
        self.difficulty_estimator.estimate_difficulty(initial_loss, learning_rate)
    }
}

/// Learning strategy that adapts based on meta-learning
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LearningStrategy {
    /// High learning rate, fast but potentially unstable
    Aggressive,
    /// Moderate learning rate, balanced
    Balanced,
    /// Low learning rate, slow but stable
    Conservative,
    /// Adaptive - changes based on performance
    Adaptive,
}

/// Meta-learning system that optimizes learning process
pub struct MetaLearner {
    adaptive_lr: AdaptiveLearningRate,
    curriculum: CurriculumLearner,
    strategy: LearningStrategy,
    performance_history: VecDeque<LearningPerformance>,
    total_iterations: usize,
}

impl MetaLearner {
    pub fn new(initial_lr: f64) -> Self {
        Self {
            adaptive_lr: AdaptiveLearningRate::new(initial_lr, 0.0001, 0.1),
            curriculum: CurriculumLearner::new(),
            strategy: LearningStrategy::Adaptive,
            performance_history: VecDeque::new(),
            total_iterations: 0,
        }
    }

    /// Record a learning step
    pub fn record_step(&mut self, step: usize, loss: f64, accuracy: f64) {
        let lr = self.adaptive_lr.update(loss);

        let perf = LearningPerformance {
            step,
            loss,
            accuracy,
            learning_rate: lr,
            timestamp: std::time::Instant::now(),
        };

        self.performance_history.push_back(perf);
        if self.performance_history.len() > 100 {
            self.performance_history.pop_front();
        }

        self.total_iterations += 1;
    }

    /// Get current recommended learning rate
    pub fn get_learning_rate(&self) -> f64 {
        match self.strategy {
            LearningStrategy::Aggressive => self.adaptive_lr.get_rate() * 2.0,
            LearningStrategy::Conservative => self.adaptive_lr.get_rate() * 0.5,
            LearningStrategy::Balanced => self.adaptive_lr.get_rate(),
            LearningStrategy::Adaptive => self.adaptive_lr.get_rate(),
        }
    }

    /// Suggest optimal learning strategy based on recent performance
    pub fn suggest_strategy(&self) -> LearningStrategy {
        if self.performance_history.len() < 20 {
            return LearningStrategy::Balanced;
        }

        let recent: Vec<_> = self.performance_history.iter().rev().take(20).collect();

        // Calculate loss variance
        let avg_loss: f64 = recent.iter().map(|p| p.loss).sum::<f64>() / recent.len() as f64;
        let variance: f64 = recent.iter()
            .map(|p| (p.loss - avg_loss).powi(2))
            .sum::<f64>() / recent.len() as f64;

        // Calculate improvement rate
        let old_losses: Vec<_> = recent.iter().take(10).map(|p| p.loss).collect();
        let new_losses: Vec<_> = recent.iter().skip(10).map(|p| p.loss).collect();
        let old_avg: f64 = old_losses.iter().sum::<f64>() / old_losses.len() as f64;
        let new_avg: f64 = new_losses.iter().sum::<f64>() / new_losses.len() as f64;
        let improvement = old_avg - new_avg;

        // Decide strategy
        if variance > 0.1 {
            // High variance = reduce learning rate
            LearningStrategy::Conservative
        } else if improvement < 0.001 && avg_loss > 0.1 {
            // Not improving much = try more aggressive
            LearningStrategy::Aggressive
        } else {
            // Doing well = stay balanced/adaptive
            LearningStrategy::Adaptive
        }
    }

    /// Set learning strategy
    pub fn set_strategy(&mut self, strategy: LearningStrategy) {
        self.strategy = strategy;
    }

    /// Get meta-learning statistics
    pub fn get_stats(&self) -> MetaLearningStats {
        let recent_count = 20.min(self.performance_history.len());
        let recent: Vec<_> = self.performance_history.iter().rev().take(recent_count).collect();

        let avg_loss = if !recent.is_empty() {
            recent.iter().map(|p| p.loss).sum::<f64>() / recent.len() as f64
        } else {
            0.0
        };

        let avg_accuracy = if !recent.is_empty() {
            recent.iter().map(|p| p.accuracy).sum::<f64>() / recent.len() as f64
        } else {
            0.0
        };

        let learning_efficiency = if self.total_iterations > 0 {
            avg_accuracy / self.total_iterations as f64
        } else {
            0.0
        };

        MetaLearningStats {
            current_learning_rate: self.get_learning_rate(),
            average_recent_loss: avg_loss,
            average_recent_accuracy: avg_accuracy,
            current_strategy: self.strategy,
            curriculum_difficulty: self.curriculum.get_target_difficulty(),
            total_iterations: self.total_iterations,
            learning_efficiency,
        }
    }

    /// Update curriculum based on success
    pub fn update_curriculum(&mut self, success_rate: f64) {
        self.curriculum.update_curriculum(success_rate);
    }

    /// Check if should attempt a task given its difficulty
    pub fn should_attempt_task(&self, initial_loss: f64) -> bool {
        let difficulty = self.curriculum.estimate_difficulty(initial_loss, self.get_learning_rate());
        self.curriculum.should_attempt_task(difficulty)
    }

    /// Record completed task
    pub fn record_task(&mut self, task_id: String, initial_loss: f64, final_loss: f64, iterations: usize) {
        self.curriculum.record_task_result(task_id, initial_loss, final_loss, iterations);
    }
}

#[derive(Debug, Clone)]
pub struct MetaLearningStats {
    pub current_learning_rate: f64,
    pub average_recent_loss: f64,
    pub average_recent_accuracy: f64,
    pub current_strategy: LearningStrategy,
    pub curriculum_difficulty: f64,
    pub total_iterations: usize,
    pub learning_efficiency: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adaptive_learning_rate() {
        let mut alr = AdaptiveLearningRate::new(0.01, 0.0001, 0.1);

        // Simulate improving loss
        for i in 0..20 {
            let loss = 1.0 - (i as f64 * 0.04);
            alr.update(loss);
        }

        // Should maintain or increase rate when improving
        assert!(alr.get_rate() >= 0.005);
    }

    #[test]
    fn test_meta_learner() {
        let mut meta = MetaLearner::new(0.01);

        for i in 0..50 {
            let loss = 1.0 / (i as f64 + 1.0);
            let accuracy = 1.0 - loss;
            meta.record_step(i, loss, accuracy);
        }

        let stats = meta.get_stats();
        assert!(stats.average_recent_accuracy > 0.5);
    }
}
