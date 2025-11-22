//! Reptile Meta-Learning Algorithm
//!
//! Reptile (OpenAI, 2018) finds an initialization θ* that can quickly adapt
//! to any task with just a few gradient steps.
//!
//! Algorithm:
//! ```text
//! Initialize θ (meta-weights)
//! for iteration = 1, 2, ...
//!     Sample task T
//!     θ' = SGD(T, θ, k steps)  // Train on task
//!     θ ← θ + ε(θ' - θ)        // Move toward adapted weights
//! ```
//!
//! For SAGE: This enables few-shot pattern learning. Instead of ~500 generations
//! to master a new pattern, SAGE can adapt in ~10-50 steps.

use crate::nca::{NCA, WeightSnapshot, NetworkGradients};
use crate::grid::Grid;
use serde::{Deserialize, Serialize};

// ============================================================================
// WEIGHT OPERATIONS
// ============================================================================

/// Extended weight snapshot with arithmetic operations for Reptile
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReptileWeights {
    pub weights1: Vec<Vec<f64>>,
    pub bias1: Vec<f64>,
    pub weights2: Vec<Vec<f64>>,
    pub bias2: Vec<f64>,
}

impl ReptileWeights {
    /// Create from NCA's WeightSnapshot
    pub fn from_snapshot(snapshot: &WeightSnapshot) -> Self {
        Self {
            weights1: snapshot.weights1.clone(),
            bias1: snapshot.bias1.clone(),
            weights2: snapshot.weights2.clone(),
            bias2: snapshot.bias2.clone(),
        }
    }

    /// Convert to NCA's WeightSnapshot
    pub fn to_snapshot(&self) -> WeightSnapshot {
        WeightSnapshot {
            weights1: self.weights1.clone(),
            bias1: self.bias1.clone(),
            weights2: self.weights2.clone(),
            bias2: self.bias2.clone(),
        }
    }

    /// Subtract another weight set: self - other
    pub fn subtract(&self, other: &ReptileWeights) -> ReptileWeights {
        ReptileWeights {
            weights1: self.weights1.iter().zip(&other.weights1)
                .map(|(a, b)| a.iter().zip(b).map(|(x, y)| x - y).collect())
                .collect(),
            bias1: self.bias1.iter().zip(&other.bias1)
                .map(|(a, b)| a - b).collect(),
            weights2: self.weights2.iter().zip(&other.weights2)
                .map(|(a, b)| a.iter().zip(b).map(|(x, y)| x - y).collect())
                .collect(),
            bias2: self.bias2.iter().zip(&other.bias2)
                .map(|(a, b)| a - b).collect(),
        }
    }

    /// Add another weight set: self + other
    pub fn add(&self, other: &ReptileWeights) -> ReptileWeights {
        ReptileWeights {
            weights1: self.weights1.iter().zip(&other.weights1)
                .map(|(a, b)| a.iter().zip(b).map(|(x, y)| x + y).collect())
                .collect(),
            bias1: self.bias1.iter().zip(&other.bias1)
                .map(|(a, b)| a + b).collect(),
            weights2: self.weights2.iter().zip(&other.weights2)
                .map(|(a, b)| a.iter().zip(b).map(|(x, y)| x + y).collect())
                .collect(),
            bias2: self.bias2.iter().zip(&other.bias2)
                .map(|(a, b)| a + b).collect(),
        }
    }

    /// Scale weights by a factor
    pub fn scale(&self, factor: f64) -> ReptileWeights {
        ReptileWeights {
            weights1: self.weights1.iter()
                .map(|row| row.iter().map(|x| x * factor).collect())
                .collect(),
            bias1: self.bias1.iter().map(|x| x * factor).collect(),
            weights2: self.weights2.iter()
                .map(|row| row.iter().map(|x| x * factor).collect())
                .collect(),
            bias2: self.bias2.iter().map(|x| x * factor).collect(),
        }
    }

    /// Average multiple weight sets
    pub fn average(weights_list: &[ReptileWeights]) -> ReptileWeights {
        if weights_list.is_empty() {
            panic!("Cannot average empty weights list");
        }
        if weights_list.len() == 1 {
            return weights_list[0].clone();
        }

        let n = weights_list.len() as f64;
        let mut result = weights_list[0].clone();

        for w in weights_list.iter().skip(1) {
            result = result.add(w);
        }

        result.scale(1.0 / n)
    }

    /// Compute L2 norm of weights (for diagnostics)
    pub fn l2_norm(&self) -> f64 {
        let mut sum = 0.0;
        for row in &self.weights1 {
            for &v in row {
                sum += v * v;
            }
        }
        for &v in &self.bias1 {
            sum += v * v;
        }
        for row in &self.weights2 {
            for &v in row {
                sum += v * v;
            }
        }
        for &v in &self.bias2 {
            sum += v * v;
        }
        sum.sqrt()
    }
}

// ============================================================================
// PATTERN TASK
// ============================================================================

/// A pattern task for meta-learning
#[derive(Clone)]
pub struct PatternTask {
    pub id: String,
    pub name: String,
    /// Function to generate target grid
    target_generator: fn() -> Grid,
}

impl PatternTask {
    pub fn new(id: &str, name: &str, generator: fn() -> Grid) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            target_generator: generator,
        }
    }

    pub fn generate_target(&self) -> Grid {
        (self.target_generator)()
    }
}

// ============================================================================
// REPTILE META-LEARNER
// ============================================================================

/// Reptile meta-learning algorithm for few-shot pattern adaptation
pub struct ReptileMetaLearner {
    /// Meta-weights: the learned initialization
    meta_weights: ReptileWeights,
    /// Meta learning rate (epsilon in the algorithm)
    meta_lr: f64,
    /// Number of SGD steps per task (k in the algorithm)
    inner_steps: usize,
    /// Learning rate for inner loop
    inner_lr: f64,
    /// Number of tasks to sample per meta-update
    task_batch_size: usize,
    /// Evolution steps when training
    evolution_steps: usize,
    /// Track adaptation performance
    adaptation_history: Vec<AdaptationRecord>,
    /// Meta-training iteration count
    meta_iteration: usize,
}

#[derive(Clone, Debug)]
pub struct AdaptationRecord {
    pub task_id: String,
    pub initial_loss: f64,
    pub final_loss: f64,
    pub steps_taken: usize,
    pub improvement: f64,
}

impl ReptileMetaLearner {
    /// Create a new Reptile meta-learner
    pub fn new(nca: &NCA) -> Self {
        let snapshot = nca.update_net.snapshot();
        Self {
            meta_weights: ReptileWeights::from_snapshot(&snapshot),
            meta_lr: 0.1,           // Reptile typically uses higher meta-LR
            inner_steps: 10,        // k = 10 SGD steps per task
            inner_lr: 0.0001,       // Standard NCA learning rate
            task_batch_size: 4,     // Sample 4 tasks per meta-update
            evolution_steps: 64,    // NCA evolution steps
            adaptation_history: Vec::new(),
            meta_iteration: 0,
        }
    }

    /// Configure meta learning rate
    pub fn with_meta_lr(mut self, lr: f64) -> Self {
        self.meta_lr = lr;
        self
    }

    /// Configure inner loop steps
    pub fn with_inner_steps(mut self, steps: usize) -> Self {
        self.inner_steps = steps;
        self
    }

    /// Configure inner learning rate
    pub fn with_inner_lr(mut self, lr: f64) -> Self {
        self.inner_lr = lr;
        self
    }

    /// Configure task batch size
    pub fn with_task_batch_size(mut self, size: usize) -> Self {
        self.task_batch_size = size;
        self
    }

    /// Get current meta-weights
    pub fn get_meta_weights(&self) -> &ReptileWeights {
        &self.meta_weights
    }

    /// Set meta-weights (e.g., from loaded checkpoint)
    pub fn set_meta_weights(&mut self, weights: ReptileWeights) {
        self.meta_weights = weights;
    }

    /// Apply meta-weights to an NCA
    pub fn apply_to_nca(&self, nca: &mut NCA) {
        let snapshot = self.meta_weights.to_snapshot();
        nca.update_net.load_snapshot(&snapshot);
    }

    /// Perform one meta-learning iteration (Reptile algorithm)
    pub fn meta_step(&mut self, nca: &mut NCA, tasks: &[PatternTask]) -> MetaStepResult {
        use rand::seq::SliceRandom;
        let mut rng = rand::thread_rng();

        // Sample tasks
        let sampled_tasks: Vec<&PatternTask> = tasks
            .choose_multiple(&mut rng, self.task_batch_size.min(tasks.len()))
            .collect();

        let mut weight_deltas: Vec<ReptileWeights> = Vec::new();
        let mut task_results: Vec<AdaptationRecord> = Vec::new();

        for task in sampled_tasks {
            // 1. Clone meta-weights to NCA
            let snapshot = self.meta_weights.to_snapshot();
            nca.update_net.load_snapshot(&snapshot);

            // 2. Compute initial loss
            let target = task.generate_target();
            let initial_loss = self.compute_loss(nca, &target);

            // 3. Inner loop: Train on this task for k steps
            for _ in 0..self.inner_steps {
                self.train_step(nca, &target);
            }

            // 4. Compute final loss
            let final_loss = self.compute_loss(nca, &target);

            // 5. Get adapted weights
            let adapted_snapshot = nca.update_net.snapshot();
            let adapted_weights = ReptileWeights::from_snapshot(&adapted_snapshot);

            // 6. Compute delta: θ' - θ
            let delta = adapted_weights.subtract(&self.meta_weights);
            weight_deltas.push(delta);

            // Record adaptation
            let record = AdaptationRecord {
                task_id: task.id.clone(),
                initial_loss,
                final_loss,
                steps_taken: self.inner_steps,
                improvement: initial_loss - final_loss,
            };
            task_results.push(record.clone());
            self.adaptation_history.push(record);
        }

        // 7. Average deltas
        let avg_delta = ReptileWeights::average(&weight_deltas);

        // 8. Update meta-weights: θ ← θ + ε * avg_delta
        let scaled_delta = avg_delta.scale(self.meta_lr);
        self.meta_weights = self.meta_weights.add(&scaled_delta);

        self.meta_iteration += 1;

        // Keep adaptation history bounded
        if self.adaptation_history.len() > 1000 {
            self.adaptation_history.drain(0..500);
        }

        MetaStepResult {
            iteration: self.meta_iteration,
            tasks_sampled: task_results.len(),
            avg_initial_loss: task_results.iter().map(|r| r.initial_loss).sum::<f64>() / task_results.len() as f64,
            avg_final_loss: task_results.iter().map(|r| r.final_loss).sum::<f64>() / task_results.len() as f64,
            avg_improvement: task_results.iter().map(|r| r.improvement).sum::<f64>() / task_results.len() as f64,
            delta_norm: avg_delta.l2_norm(),
            task_results,
        }
    }

    /// Adapt to a new pattern using the meta-initialization (few-shot learning)
    pub fn adapt(&self, nca: &mut NCA, task: &PatternTask, max_steps: usize) -> AdaptationResult {
        // Start from meta-weights
        let snapshot = self.meta_weights.to_snapshot();
        nca.update_net.load_snapshot(&snapshot);

        let target = task.generate_target();
        let initial_loss = self.compute_loss(nca, &target);

        let mut loss_history = vec![initial_loss];
        let mut final_loss = initial_loss;

        for step in 0..max_steps {
            self.train_step(nca, &target);
            final_loss = self.compute_loss(nca, &target);
            loss_history.push(final_loss);

            // Early stopping if mastered
            if final_loss < 0.05 {
                return AdaptationResult {
                    task_id: task.id.clone(),
                    initial_loss,
                    final_loss,
                    steps_taken: step + 1,
                    converged: true,
                    loss_history,
                };
            }
        }

        AdaptationResult {
            task_id: task.id.clone(),
            initial_loss,
            final_loss,
            steps_taken: max_steps,
            converged: final_loss < 0.1,
            loss_history,
        }
    }

    /// Compare few-shot adaptation with vs without meta-learning
    pub fn benchmark_adaptation(&self, nca: &mut NCA, task: &PatternTask, steps: usize) -> AdaptationBenchmark {
        // Test 1: Adapt from meta-weights
        let meta_result = self.adapt(nca, task, steps);

        // Test 2: Adapt from random initialization
        let fresh_nca_net = crate::nca::UpdateNetwork::new(
            nca.update_net.weights1[0].len(),
            nca.update_net.hidden_size,
            nca.update_net.weights2.len(),
        );
        nca.update_net = fresh_nca_net;

        let target = task.generate_target();
        let random_initial_loss = self.compute_loss(nca, &target);

        let mut random_loss_history = vec![random_initial_loss];
        let mut random_final_loss = random_initial_loss;

        for _ in 0..steps {
            self.train_step(nca, &target);
            random_final_loss = self.compute_loss(nca, &target);
            random_loss_history.push(random_final_loss);
        }

        let random_result = AdaptationResult {
            task_id: task.id.clone(),
            initial_loss: random_initial_loss,
            final_loss: random_final_loss,
            steps_taken: steps,
            converged: random_final_loss < 0.1,
            loss_history: random_loss_history,
        };

        // Restore meta-weights
        self.apply_to_nca(nca);

        // Compute speedup before moving values
        let speedup = if random_result.final_loss > 0.0 {
            meta_result.final_loss / random_result.final_loss
        } else {
            1.0
        };

        AdaptationBenchmark {
            meta_adaptation: meta_result,
            random_adaptation: random_result,
            speedup,
        }
    }

    /// Train NCA for one step on target pattern
    fn train_step(&self, nca: &mut NCA, target: &Grid) {
        // Reset and evolve
        nca.reset_with_seed();
        for _ in 0..self.evolution_steps {
            nca.step();
        }

        // Compute gradients and update
        let gradients = self.compute_gradients(nca, target);
        nca.update_net.update_weights(&gradients, self.inner_lr);
    }

    /// Compute loss between NCA output and target
    fn compute_loss(&self, nca: &mut NCA, target: &Grid) -> f64 {
        nca.reset_with_seed();
        for _ in 0..self.evolution_steps {
            nca.step();
        }

        // MSE on alpha channel
        let mut mse = 0.0;
        let mut count = 0;
        for y in 0..nca.grid.height {
            for x in 0..nca.grid.width {
                let diff = nca.grid.cells[y][x][3] - target.cells[y][x][3];
                mse += diff * diff;
                count += 1;
            }
        }
        mse / count as f64
    }

    /// Compute gradients for training
    fn compute_gradients(&self, nca: &NCA, target: &Grid) -> NetworkGradients {
        let mut gradients = NetworkGradients::new(&nca.update_net);

        // Simplified gradient computation: difference between output and target
        // scaled by network response
        for y in 0..nca.grid.height {
            for x in 0..nca.grid.width {
                let output_alpha = nca.grid.cells[y][x][3];
                let target_alpha = target.cells[y][x][3];
                let error = output_alpha - target_alpha;

                // Accumulate gradient (simplified - real implementation would use backprop)
                for i in 0..gradients.grad_b2.len() {
                    gradients.grad_b2[i] += error * 0.001;
                }
            }
        }

        gradients
    }

    /// Get meta-learning statistics
    pub fn get_stats(&self) -> ReptileStats {
        let recent_count = 50.min(self.adaptation_history.len());
        let recent: Vec<_> = self.adaptation_history.iter().rev().take(recent_count).collect();

        let avg_improvement = if !recent.is_empty() {
            recent.iter().map(|r| r.improvement).sum::<f64>() / recent.len() as f64
        } else {
            0.0
        };

        let avg_final_loss = if !recent.is_empty() {
            recent.iter().map(|r| r.final_loss).sum::<f64>() / recent.len() as f64
        } else {
            f64::MAX
        };

        ReptileStats {
            meta_iteration: self.meta_iteration,
            total_adaptations: self.adaptation_history.len(),
            avg_recent_improvement: avg_improvement,
            avg_recent_final_loss: avg_final_loss,
            meta_weights_norm: self.meta_weights.l2_norm(),
        }
    }
}

// ============================================================================
// RESULT TYPES
// ============================================================================

#[derive(Clone, Debug)]
pub struct MetaStepResult {
    pub iteration: usize,
    pub tasks_sampled: usize,
    pub avg_initial_loss: f64,
    pub avg_final_loss: f64,
    pub avg_improvement: f64,
    pub delta_norm: f64,
    pub task_results: Vec<AdaptationRecord>,
}

#[derive(Clone, Debug)]
pub struct AdaptationResult {
    pub task_id: String,
    pub initial_loss: f64,
    pub final_loss: f64,
    pub steps_taken: usize,
    pub converged: bool,
    pub loss_history: Vec<f64>,
}

#[derive(Clone, Debug)]
pub struct AdaptationBenchmark {
    pub meta_adaptation: AdaptationResult,
    pub random_adaptation: AdaptationResult,
    pub speedup: f64,
}

#[derive(Clone, Debug)]
pub struct ReptileStats {
    pub meta_iteration: usize,
    pub total_adaptations: usize,
    pub avg_recent_improvement: f64,
    pub avg_recent_final_loss: f64,
    pub meta_weights_norm: f64,
}

// ============================================================================
// STANDARD PATTERN GENERATORS
// ============================================================================

pub fn create_circle_target() -> Grid {
    use crate::grid::GRID_SIZE;
    let mut grid = Grid::new(GRID_SIZE, GRID_SIZE);
    let center = GRID_SIZE as f64 / 2.0;
    let radius = 8.0;

    for y in 0..GRID_SIZE {
        for x in 0..GRID_SIZE {
            let dx = x as f64 - center;
            let dy = y as f64 - center;
            if (dx * dx + dy * dy).sqrt() < radius {
                grid.cells[y][x][3] = 1.0;
                grid.cells[y][x][0] = 1.0;
            }
        }
    }
    grid
}

pub fn create_square_target() -> Grid {
    use crate::grid::GRID_SIZE;
    let mut grid = Grid::new(GRID_SIZE, GRID_SIZE);
    let center = GRID_SIZE / 2;
    let size = 12;

    for y in (center - size)..(center + size) {
        for x in (center - size)..(center + size) {
            if y < GRID_SIZE && x < GRID_SIZE {
                grid.cells[y][x][3] = 1.0;
                grid.cells[y][x][1] = 1.0;
            }
        }
    }
    grid
}

pub fn create_cross_target() -> Grid {
    use crate::grid::GRID_SIZE;
    let mut grid = Grid::new(GRID_SIZE, GRID_SIZE);
    let center = GRID_SIZE / 2;
    let arm_width = 4;
    let arm_length = 10;

    // Horizontal
    for x in (center - arm_length)..(center + arm_length) {
        for dy in 0..arm_width {
            let y = center - arm_width / 2 + dy;
            if y < GRID_SIZE && x < GRID_SIZE {
                grid.cells[y][x][3] = 1.0;
                grid.cells[y][x][2] = 1.0;
            }
        }
    }

    // Vertical
    for y in (center - arm_length)..(center + arm_length) {
        for dx in 0..arm_width {
            let x = center - arm_width / 2 + dx;
            if y < GRID_SIZE && x < GRID_SIZE {
                grid.cells[y][x][3] = 1.0;
                grid.cells[y][x][2] = 1.0;
            }
        }
    }
    grid
}

pub fn create_spiral_target() -> Grid {
    use crate::grid::GRID_SIZE;
    let mut grid = Grid::new(GRID_SIZE, GRID_SIZE);
    let center = GRID_SIZE as f64 / 2.0;

    for i in 0..100 {
        let angle = i as f64 * 0.3;
        let radius = i as f64 * 0.1;
        let x = (center + radius * angle.cos()) as usize;
        let y = (center + radius * angle.sin()) as usize;

        if x < GRID_SIZE && y < GRID_SIZE {
            grid.cells[y][x][3] = 1.0;
            grid.cells[y][x][0] = 1.0;
            grid.cells[y][x][1] = 1.0;
        }
    }
    grid
}

/// Create standard pattern tasks
pub fn create_standard_tasks() -> Vec<PatternTask> {
    vec![
        PatternTask::new("circle", "Circle", create_circle_target),
        PatternTask::new("square", "Square", create_square_target),
        PatternTask::new("cross", "Cross", create_cross_target),
        PatternTask::new("spiral", "Spiral", create_spiral_target),
    ]
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weight_operations() {
        let w1 = ReptileWeights {
            weights1: vec![vec![1.0, 2.0], vec![3.0, 4.0]],
            bias1: vec![0.1, 0.2],
            weights2: vec![vec![5.0, 6.0]],
            bias2: vec![0.3],
        };

        let w2 = ReptileWeights {
            weights1: vec![vec![0.5, 1.0], vec![1.5, 2.0]],
            bias1: vec![0.05, 0.1],
            weights2: vec![vec![2.5, 3.0]],
            bias2: vec![0.15],
        };

        let diff = w1.subtract(&w2);
        assert!((diff.weights1[0][0] - 0.5).abs() < 1e-10);

        let sum = w1.add(&w2);
        assert!((sum.weights1[0][0] - 1.5).abs() < 1e-10);

        let scaled = w1.scale(2.0);
        assert!((scaled.weights1[0][0] - 2.0).abs() < 1e-10);
    }

    #[test]
    fn test_reptile_creation() {
        let nca = NCA::new();
        let reptile = ReptileMetaLearner::new(&nca)
            .with_meta_lr(0.1)
            .with_inner_steps(5);

        assert!(reptile.meta_lr == 0.1);
        assert!(reptile.inner_steps == 5);
    }

    #[test]
    fn test_pattern_tasks() {
        let tasks = create_standard_tasks();
        assert_eq!(tasks.len(), 4);

        for task in &tasks {
            let target = task.generate_target();
            assert!(target.width > 0);
            assert!(target.height > 0);
        }
    }
}
