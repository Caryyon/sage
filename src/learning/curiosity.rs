// Curiosity-driven exploration system for autonomous learning

use super::phase_config::Grid;
use super::feature_extractor::FeatureExtractor;
use super::neural_network::NeuralNetwork;
use std::collections::VecDeque;

/// Measures how novel an observation is
pub struct NoveltyDetector {
    feature_extractor: FeatureExtractor,
    observation_history: VecDeque<Vec<f64>>,
    max_history: usize,
}

impl NoveltyDetector {
    pub fn new(max_history: usize) -> Self {
        Self {
            feature_extractor: FeatureExtractor::new(),
            observation_history: VecDeque::new(),
            max_history,
        }
    }

    /// Calculate novelty score for a grid (higher = more novel)
    pub fn calculate_novelty(&mut self, grid: &Grid) -> f64 {
        let features = self.feature_extractor.extract_all(grid);

        if self.observation_history.is_empty() {
            // First observation is maximally novel
            self.add_to_history(features);
            return 1.0;
        }

        // Calculate minimum distance to any previous observation
        let min_distance = self.observation_history
            .iter()
            .map(|prev_features| self.euclidean_distance(&features, prev_features))
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(1.0);

        // Normalize distance to 0-1 range (novelty score)
        let novelty = (min_distance / 10.0).min(1.0);

        self.add_to_history(features);

        novelty
    }

    fn add_to_history(&mut self, features: Vec<f64>) {
        self.observation_history.push_back(features);
        if self.observation_history.len() > self.max_history {
            self.observation_history.pop_front();
        }
    }

    fn euclidean_distance(&self, v1: &[f64], v2: &[f64]) -> f64 {
        v1.iter()
            .zip(v2.iter())
            .map(|(a, b)| (a - b).powi(2))
            .sum::<f64>()
            .sqrt()
    }

    /// Get average novelty of recent observations
    pub fn average_recent_novelty(&self, recent_count: usize) -> f64 {
        if self.observation_history.len() < 2 {
            return 0.5;
        }

        let count = recent_count.min(self.observation_history.len() - 1);
        let recent: Vec<_> = self.observation_history
            .iter()
            .rev()
            .take(count)
            .collect();

        if recent.is_empty() {
            return 0.5;
        }

        let mut total_novelty = 0.0;
        for i in 0..recent.len() {
            let min_dist = self.observation_history
                .iter()
                .filter(|obs| !std::ptr::eq(*obs, recent[i]))
                .map(|prev| self.euclidean_distance(recent[i], prev))
                .min_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap_or(1.0);

            total_novelty += (min_dist / 10.0).min(1.0);
        }

        total_novelty / recent.len() as f64
    }
}

/// Intrinsic motivation system - rewards learning and exploration
pub struct IntrinsicMotivation {
    prediction_network: NeuralNetwork,
    prediction_errors: VecDeque<f64>,
    novelty_detector: NoveltyDetector,
    max_error_history: usize,
}

impl IntrinsicMotivation {
    pub fn new(feature_size: usize) -> Self {
        Self {
            prediction_network: NeuralNetwork::new(feature_size, 32, feature_size),
            prediction_errors: VecDeque::new(),
            novelty_detector: NoveltyDetector::new(100),
            max_error_history: 50,
        }
    }

    /// Calculate intrinsic reward for a grid
    /// Higher reward = more interesting for learning
    pub fn calculate_intrinsic_reward(&mut self, grid: &Grid) -> f64 {
        // Component 1: Novelty (how different from previous observations)
        let novelty = self.novelty_detector.calculate_novelty(grid);

        // Component 2: Learning progress (how well we can predict this)
        let features = self.novelty_detector.feature_extractor.extract_all(grid);
        let prediction = self.prediction_network.predict(&features);
        let prediction_error = self.calculate_prediction_error(&features, &prediction);

        // Component 3: Improvement rate (are we getting better at prediction?)
        let improvement = self.calculate_improvement_rate();

        // Combine components
        let intrinsic_reward =
            0.4 * novelty +           // Novel observations are interesting
            0.3 * prediction_error +  // Hard-to-predict patterns are interesting
            0.3 * improvement;        // Making progress is interesting

        // Track prediction error
        self.prediction_errors.push_back(prediction_error);
        if self.prediction_errors.len() > self.max_error_history {
            self.prediction_errors.pop_front();
        }

        intrinsic_reward
    }

    /// Train the prediction model on observed patterns
    pub fn learn_to_predict(&mut self, current_grid: &Grid, next_grid: &Grid) -> f64 {
        let current_features = self.novelty_detector.feature_extractor.extract_all(current_grid);
        let next_features = self.novelty_detector.feature_extractor.extract_all(next_grid);

        self.prediction_network.train(&current_features, &next_features)
    }

    fn calculate_prediction_error(&self, actual: &[f64], predicted: &[f64]) -> f64 {
        let mse: f64 = actual.iter()
            .zip(predicted.iter())
            .map(|(a, p)| (a - p).powi(2))
            .sum::<f64>() / actual.len() as f64;

        mse.sqrt().min(1.0)  // Normalize to 0-1
    }

    fn calculate_improvement_rate(&self) -> f64 {
        if self.prediction_errors.len() < 10 {
            return 0.0;
        }

        let recent_errors: Vec<_> = self.prediction_errors.iter().rev().take(10).copied().collect();
        let old_errors: Vec<_> = self.prediction_errors.iter().take(10).copied().collect();

        let recent_avg: f64 = recent_errors.iter().sum::<f64>() / recent_errors.len() as f64;
        let old_avg: f64 = old_errors.iter().sum::<f64>() / old_errors.len() as f64;

        // Improvement = reduction in error (clamped to 0-1)
        ((old_avg - recent_avg) / old_avg.max(0.01)).clamp(0.0, 1.0)
    }

    pub fn set_learning_rate(&mut self, lr: f64) {
        self.prediction_network.set_learning_rate(lr);
    }
}

/// Exploration strategy that decides what to explore next
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExplorationStrategy {
    /// Explore areas with high novelty
    NoveltyDriven,
    /// Explore areas where predictions are uncertain
    UncertaintyDriven,
    /// Balance between exploitation and exploration
    Balanced,
    /// Random exploration
    Random,
}

/// Curiosity system that drives autonomous exploration
pub struct CuriositySystem {
    intrinsic_motivation: IntrinsicMotivation,
    exploration_strategy: ExplorationStrategy,
    exploration_history: Vec<ExplorationRecord>,
    total_intrinsic_reward: f64,
}

#[derive(Clone)]
pub struct ExplorationRecord {
    pub step: usize,
    pub novelty: f64,
    pub intrinsic_reward: f64,
    pub strategy_used: ExplorationStrategy,
}

impl CuriositySystem {
    pub fn new(feature_size: usize) -> Self {
        Self {
            intrinsic_motivation: IntrinsicMotivation::new(feature_size),
            exploration_strategy: ExplorationStrategy::Balanced,
            exploration_history: Vec::new(),
            total_intrinsic_reward: 0.0,
        }
    }

    /// Evaluate how interesting a grid is for learning
    pub fn evaluate_interest(&mut self, grid: &Grid, step: usize) -> f64 {
        let intrinsic_reward = self.intrinsic_motivation.calculate_intrinsic_reward(grid);

        // Record exploration
        let record = ExplorationRecord {
            step,
            novelty: self.intrinsic_motivation.novelty_detector.calculate_novelty(grid),
            intrinsic_reward,
            strategy_used: self.exploration_strategy,
        };

        self.exploration_history.push(record);
        self.total_intrinsic_reward += intrinsic_reward;

        // Keep last 1000 records
        if self.exploration_history.len() > 1000 {
            self.exploration_history.remove(0);
        }

        intrinsic_reward
    }

    /// Learn from observed sequence
    pub fn learn_from_sequence(&mut self, grid_sequence: &[Grid]) -> f64 {
        let mut total_loss = 0.0;
        let mut count = 0;

        for i in 0..grid_sequence.len().saturating_sub(1) {
            total_loss += self.intrinsic_motivation.learn_to_predict(
                &grid_sequence[i],
                &grid_sequence[i + 1],
            );
            count += 1;
        }

        if count > 0 {
            total_loss / count as f64
        } else {
            0.0
        }
    }

    /// Suggest exploration strategy based on recent progress
    pub fn suggest_exploration_adjustment(&self) -> Option<ExplorationStrategy> {
        if self.exploration_history.len() < 50 {
            return None;
        }

        let recent_rewards: Vec<_> = self.exploration_history
            .iter()
            .rev()
            .take(50)
            .map(|r| r.intrinsic_reward)
            .collect();

        let avg_reward: f64 = recent_rewards.iter().sum::<f64>() / recent_rewards.len() as f64;

        // If rewards are too low, try more exploration
        if avg_reward < 0.2 {
            Some(ExplorationStrategy::NoveltyDriven)
        } else if avg_reward > 0.7 {
            // If rewards are high, we're finding good stuff
            Some(ExplorationStrategy::UncertaintyDriven)
        } else {
            Some(ExplorationStrategy::Balanced)
        }
    }

    pub fn set_strategy(&mut self, strategy: ExplorationStrategy) {
        self.exploration_strategy = strategy;
    }

    pub fn get_total_intrinsic_reward(&self) -> f64 {
        self.total_intrinsic_reward
    }

    pub fn get_average_novelty(&self, recent_count: usize) -> f64 {
        if self.exploration_history.is_empty() {
            return 0.0;
        }

        let count = recent_count.min(self.exploration_history.len());
        let recent: Vec<_> = self.exploration_history
            .iter()
            .rev()
            .take(count)
            .collect();

        if recent.is_empty() {
            return 0.0;
        }

        recent.iter().map(|r| r.novelty).sum::<f64>() / recent.len() as f64
    }

    /// Get statistics about exploration
    pub fn get_stats(&self) -> CuriosityStats {
        let total_steps = self.exploration_history.len();

        if total_steps == 0 {
            return CuriosityStats::default();
        }

        let recent_count = 50.min(total_steps);
        let recent: Vec<_> = self.exploration_history
            .iter()
            .rev()
            .take(recent_count)
            .collect();

        let avg_novelty = recent.iter().map(|r| r.novelty).sum::<f64>() / recent.len() as f64;
        let avg_reward = recent.iter().map(|r| r.intrinsic_reward).sum::<f64>() / recent.len() as f64;

        CuriosityStats {
            total_explorations: total_steps,
            total_intrinsic_reward: self.total_intrinsic_reward,
            average_novelty: avg_novelty,
            average_intrinsic_reward: avg_reward,
            current_strategy: self.exploration_strategy,
        }
    }

    pub fn set_learning_rate(&mut self, lr: f64) {
        self.intrinsic_motivation.set_learning_rate(lr);
    }
}

#[derive(Debug, Clone)]
pub struct CuriosityStats {
    pub total_explorations: usize,
    pub total_intrinsic_reward: f64,
    pub average_novelty: f64,
    pub average_intrinsic_reward: f64,
    pub current_strategy: ExplorationStrategy,
}

impl Default for CuriosityStats {
    fn default() -> Self {
        Self {
            total_explorations: 0,
            total_intrinsic_reward: 0.0,
            average_novelty: 0.0,
            average_intrinsic_reward: 0.0,
            current_strategy: ExplorationStrategy::Balanced,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_novelty_detection() {
        let mut detector = NoveltyDetector::new(10);

        let grid1 = vec![vec![0.5; 32]; 32];
        let grid2 = vec![vec![0.8; 32]; 32];
        let grid3 = vec![vec![0.5; 32]; 32];  // Same as grid1

        let novelty1 = detector.calculate_novelty(&grid1);
        let novelty2 = detector.calculate_novelty(&grid2);
        let novelty3 = detector.calculate_novelty(&grid3);

        assert!(novelty1 > 0.5, "First observation should be novel");
        assert!(novelty2 > novelty3, "Different grid should be more novel than similar grid");
    }

    #[test]
    fn test_intrinsic_motivation() {
        let feature_extractor = FeatureExtractor::new();
        let feature_size = feature_extractor.feature_count();
        let mut motivation = IntrinsicMotivation::new(feature_size);

        let grid = vec![vec![0.5; 32]; 32];
        let reward = motivation.calculate_intrinsic_reward(&grid);

        assert!(reward >= 0.0 && reward <= 1.0, "Reward should be normalized");
    }
}
