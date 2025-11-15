// Self-supervised learning tasks for autonomous pattern learning

use super::phase_config::Grid;
use super::feature_extractor::FeatureExtractor;
use super::neural_network::NeuralNetwork;
use rand::Rng;

/// Types of self-supervised learning tasks
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TaskType {
    /// Predict next grid state from current state
    NextStatePrediction,
    /// Reconstruct masked regions of the grid
    MaskedReconstruction,
    /// Distinguish between similar and dissimilar grids
    ContrastiveLearning,
}

/// A self-supervised learning task
pub struct SelfSupervisedTask {
    task_type: TaskType,
    input_grid: Grid,
    target_grid: Grid,
    #[allow(dead_code)]
    mask: Option<Grid>,  // For masked reconstruction
    difficulty: f64,
}

impl SelfSupervisedTask {
    /// Create a next-state prediction task
    pub fn next_state_prediction(current: Grid, next: Grid) -> Self {
        Self {
            task_type: TaskType::NextStatePrediction,
            input_grid: current,
            target_grid: next,
            mask: None,
            difficulty: 0.5,
        }
    }

    /// Create a masked reconstruction task
    pub fn masked_reconstruction(grid: Grid, mask_ratio: f64) -> Self {
        let size = grid.len();
        let mut rng = rand::thread_rng();

        // Create random mask
        let mut mask = vec![vec![1.0; size]; size];
        let mut masked_grid = grid.clone();

        for y in 0..size {
            for x in 0..size {
                if rng.gen::<f64>() < mask_ratio {
                    mask[y][x] = 0.0;  // Masked out
                    masked_grid[y][x] = 0.0;  // Zero out in input
                }
            }
        }

        Self {
            task_type: TaskType::MaskedReconstruction,
            input_grid: masked_grid,
            target_grid: grid,
            mask: Some(mask),
            difficulty: mask_ratio,
        }
    }

    /// Create a contrastive learning task (positive pair)
    pub fn contrastive_positive(grid1: Grid, grid2: Grid) -> Self {
        Self {
            task_type: TaskType::ContrastiveLearning,
            input_grid: grid1,
            target_grid: grid2,
            mask: None,
            difficulty: 0.3,
        }
    }

    pub fn task_type(&self) -> TaskType {
        self.task_type
    }

    pub fn input_grid(&self) -> &Grid {
        &self.input_grid
    }

    pub fn target_grid(&self) -> &Grid {
        &self.target_grid
    }

    pub fn difficulty(&self) -> f64 {
        self.difficulty
    }
}

/// Generator for self-supervised learning tasks
pub struct SelfSupervisedTaskGenerator {
    feature_extractor: FeatureExtractor,
}

impl SelfSupervisedTaskGenerator {
    pub fn new() -> Self {
        Self {
            feature_extractor: FeatureExtractor::new(),
        }
    }

    /// Generate prediction tasks from a sequence of grids
    pub fn generate_prediction_tasks(&self, grid_sequence: &[Grid]) -> Vec<SelfSupervisedTask> {
        let mut tasks = Vec::new();

        for i in 0..grid_sequence.len().saturating_sub(1) {
            tasks.push(SelfSupervisedTask::next_state_prediction(
                grid_sequence[i].clone(),
                grid_sequence[i + 1].clone(),
            ));
        }

        tasks
    }

    /// Generate reconstruction tasks from grids with varying difficulty
    pub fn generate_reconstruction_tasks(&self, grids: &[Grid], num_tasks: usize) -> Vec<SelfSupervisedTask> {
        let mut tasks = Vec::new();
        let mut rng = rand::thread_rng();

        for _ in 0..num_tasks {
            if grids.is_empty() {
                break;
            }

            let grid_idx = rng.gen_range(0..grids.len());
            let mask_ratio = rng.gen_range(0.1..0.5);  // Mask 10-50% of pixels

            tasks.push(SelfSupervisedTask::masked_reconstruction(
                grids[grid_idx].clone(),
                mask_ratio,
            ));
        }

        tasks
    }

    /// Generate contrastive learning tasks
    /// Creates positive pairs (similar patterns) and negative pairs (different patterns)
    pub fn generate_contrastive_tasks(&self, grids: &[Grid], num_pairs: usize) -> Vec<(SelfSupervisedTask, bool)> {
        let mut tasks = Vec::new();
        let mut rng = rand::thread_rng();

        if grids.len() < 2 {
            return tasks;
        }

        for _ in 0..num_pairs {
            let idx1 = rng.gen_range(0..grids.len());
            let idx2 = rng.gen_range(0..grids.len());

            // Use feature similarity to determine if grids are similar
            let features1 = self.feature_extractor.extract_all(&grids[idx1]);
            let features2 = self.feature_extractor.extract_all(&grids[idx2]);

            let similarity = self.cosine_similarity(&features1, &features2);
            let is_positive = similarity > 0.7;  // High similarity = positive pair

            let task = SelfSupervisedTask::contrastive_positive(
                grids[idx1].clone(),
                grids[idx2].clone(),
            );

            tasks.push((task, is_positive));
        }

        tasks
    }

    /// Calculate cosine similarity between feature vectors
    fn cosine_similarity(&self, v1: &[f64], v2: &[f64]) -> f64 {
        assert_eq!(v1.len(), v2.len());

        let dot_product: f64 = v1.iter().zip(v2.iter()).map(|(a, b)| a * b).sum();
        let mag1: f64 = v1.iter().map(|x| x * x).sum::<f64>().sqrt();
        let mag2: f64 = v2.iter().map(|x| x * x).sum::<f64>().sqrt();

        if mag1 == 0.0 || mag2 == 0.0 {
            0.0
        } else {
            dot_product / (mag1 * mag2)
        }
    }
}

/// Self-supervised learner that trains on generated tasks
pub struct SelfSupervisedLearner {
    prediction_network: NeuralNetwork,
    reconstruction_network: NeuralNetwork,
    contrastive_network: NeuralNetwork,
    task_generator: SelfSupervisedTaskGenerator,
    feature_extractor: FeatureExtractor,
}

impl SelfSupervisedLearner {
    pub fn new() -> Self {
        let feature_extractor = FeatureExtractor::new();
        let feature_size = feature_extractor.feature_count();

        // Networks for different task types
        let prediction_network = NeuralNetwork::new(feature_size, 64, feature_size);
        let reconstruction_network = NeuralNetwork::new(feature_size, 64, feature_size);
        let contrastive_network = NeuralNetwork::new(feature_size, 32, 16);  // Embedding space

        Self {
            prediction_network,
            reconstruction_network,
            contrastive_network,
            task_generator: SelfSupervisedTaskGenerator::new(),
            feature_extractor,
        }
    }

    /// Train on next-state prediction tasks
    pub fn train_prediction(&mut self, task: &SelfSupervisedTask) -> f64 {
        assert_eq!(task.task_type(), TaskType::NextStatePrediction);

        let input_features = self.feature_extractor.extract_all(task.input_grid());
        let target_features = self.feature_extractor.extract_all(task.target_grid());

        self.prediction_network.train(&input_features, &target_features)
    }

    /// Train on masked reconstruction tasks
    pub fn train_reconstruction(&mut self, task: &SelfSupervisedTask) -> f64 {
        assert_eq!(task.task_type(), TaskType::MaskedReconstruction);

        let input_features = self.feature_extractor.extract_all(task.input_grid());
        let target_features = self.feature_extractor.extract_all(task.target_grid());

        self.reconstruction_network.train(&input_features, &target_features)
    }

    /// Train on contrastive learning tasks
    pub fn train_contrastive(&mut self, task: &SelfSupervisedTask, is_positive: bool) -> f64 {
        assert_eq!(task.task_type(), TaskType::ContrastiveLearning);

        let features1 = self.feature_extractor.extract_all(task.input_grid());
        let features2 = self.feature_extractor.extract_all(task.target_grid());

        // Get embeddings
        let embedding1 = self.contrastive_network.predict(&features1);
        let embedding2 = self.contrastive_network.predict(&features2);

        // Contrastive loss: embeddings should be close if positive, far if negative
        let distance = self.euclidean_distance(&embedding1, &embedding2);
        let target_distance = if is_positive { 0.1 } else { 2.0 };

        // Simple loss: (distance - target)^2
        let _loss = (distance - target_distance).powi(2);

        // Train both embeddings toward target
        let target1 = if is_positive {
            embedding2.clone()
        } else {
            // Push away from embedding2
            embedding1.iter().map(|&x| -x).collect()
        };

        self.contrastive_network.train(&features1, &target1)
    }

    /// Train on a batch of mixed self-supervised tasks
    pub fn train_batch(&mut self,
                       prediction_tasks: &[SelfSupervisedTask],
                       reconstruction_tasks: &[SelfSupervisedTask],
                       contrastive_tasks: &[(SelfSupervisedTask, bool)],
                       epochs: usize) {
        for _epoch in 0..epochs {
            let mut total_loss = 0.0;
            let mut task_count = 0;

            // Train prediction tasks
            for task in prediction_tasks {
                total_loss += self.train_prediction(task);
                task_count += 1;
            }

            // Train reconstruction tasks
            for task in reconstruction_tasks {
                total_loss += self.train_reconstruction(task);
                task_count += 1;
            }

            // Train contrastive tasks
            for (task, is_positive) in contrastive_tasks {
                total_loss += self.train_contrastive(task, *is_positive);
                task_count += 1;
            }

            // Silent training for TUI - no progress output
            let _avg_loss = if task_count > 0 { total_loss / task_count as f64 } else { 0.0 };
        }
    }

    /// Predict next grid state
    pub fn predict_next_state(&self, current_grid: &Grid) -> Grid {
        let input_features = self.feature_extractor.extract_all(current_grid);
        let _predicted_features = self.prediction_network.predict(&input_features);

        // This is simplified - in practice, you'd need a decoder network
        // For now, return current grid as placeholder
        current_grid.clone()
    }

    /// Get task generator
    pub fn task_generator(&self) -> &SelfSupervisedTaskGenerator {
        &self.task_generator
    }

    fn euclidean_distance(&self, v1: &[f64], v2: &[f64]) -> f64 {
        v1.iter()
            .zip(v2.iter())
            .map(|(a, b)| (a - b).powi(2))
            .sum::<f64>()
            .sqrt()
    }

    pub fn set_learning_rate(&mut self, lr: f64) {
        self.prediction_network.set_learning_rate(lr);
        self.reconstruction_network.set_learning_rate(lr);
        self.contrastive_network.set_learning_rate(lr);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_generation() {
        let generator = SelfSupervisedTaskGenerator::new();

        // Create some test grids
        let grid1 = vec![vec![0.5; 32]; 32];
        let grid2 = vec![vec![0.8; 32]; 32];
        let grids = vec![grid1, grid2];

        // Test reconstruction task generation
        let tasks = generator.generate_reconstruction_tasks(&grids, 5);
        assert_eq!(tasks.len(), 5);

        for task in tasks {
            assert_eq!(task.task_type(), TaskType::MaskedReconstruction);
        }
    }

    #[test]
    fn test_prediction_task_sequence() {
        let generator = SelfSupervisedTaskGenerator::new();

        let grids = vec![
            vec![vec![0.1; 32]; 32],
            vec![vec![0.2; 32]; 32],
            vec![vec![0.3; 32]; 32],
        ];

        let tasks = generator.generate_prediction_tasks(&grids);
        assert_eq!(tasks.len(), 2);  // N-1 tasks for N grids
    }
}
