//! Learned Optimizer (L2O)
//!
//! Instead of hand-designed optimizers (Adam, SGD), learn an optimizer
//! that discovers update rules suited to NCA training dynamics.
//!
//! The learned optimizer is a small neural network that:
//! - Takes: gradient, momentum, loss history, step number
//! - Outputs: weight update direction and magnitude
//!
//! Trained via meta-learning: optimize the optimizer to minimize
//! loss over training trajectories.
//!
//! Based on: "Learning to Learn by Gradient Descent by Gradient Descent" (Andrychowicz et al.)

use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

// ============================================================================
// OPTIMIZER NETWORK
// ============================================================================

/// A small LSTM-like network that outputs weight updates
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OptimizerNetwork {
    /// Hidden state size
    hidden_size: usize,
    /// Input features per parameter
    input_size: usize,

    // LSTM-like gates (simplified)
    /// Input gate weights
    w_input: Vec<Vec<f64>>,
    /// Forget gate weights
    w_forget: Vec<Vec<f64>>,
    /// Output gate weights
    w_output: Vec<Vec<f64>>,
    /// Cell gate weights
    w_cell: Vec<Vec<f64>>,
    /// Output projection
    w_proj: Vec<f64>,

    /// Hidden state
    hidden: Vec<f64>,
    /// Cell state
    cell: Vec<f64>,
}

impl OptimizerNetwork {
    pub fn new(hidden_size: usize) -> Self {
        let input_size = 6;  // gradient, momentum, loss, step, grad_squared, loss_delta
        let mut rng = rand::thread_rng();

        // Initialize weights with small random values
        let mut init_weight = |rows: usize, cols: usize| -> Vec<Vec<f64>> {
            (0..rows).map(|_| {
                (0..cols).map(|_| rng.gen_range(-0.1..0.1)).collect()
            }).collect()
        };

        Self {
            hidden_size,
            input_size,
            w_input: init_weight(hidden_size, input_size + hidden_size),
            w_forget: init_weight(hidden_size, input_size + hidden_size),
            w_output: init_weight(hidden_size, input_size + hidden_size),
            w_cell: init_weight(hidden_size, input_size + hidden_size),
            w_proj: (0..hidden_size).map(|_| rng.gen_range(-0.1..0.1)).collect(),
            hidden: vec![0.0; hidden_size],
            cell: vec![0.0; hidden_size],
        }
    }

    /// Reset hidden state
    pub fn reset(&mut self) {
        self.hidden = vec![0.0; self.hidden_size];
        self.cell = vec![0.0; self.hidden_size];
    }

    /// Compute update for a single parameter
    pub fn compute_update(&mut self, features: &OptimizerInput) -> f64 {
        // Prepare input: concatenate features with hidden state
        let input = self.prepare_input(features);

        // LSTM forward pass (simplified)
        let input_gate = self.gate_forward(&self.w_input, &input);
        let forget_gate = self.gate_forward(&self.w_forget, &input);
        let output_gate = self.gate_forward(&self.w_output, &input);
        let cell_candidate = self.gate_forward_tanh(&self.w_cell, &input);

        // Update cell and hidden state
        for i in 0..self.hidden_size {
            self.cell[i] = forget_gate[i] * self.cell[i] + input_gate[i] * cell_candidate[i];
            self.hidden[i] = output_gate[i] * self.cell[i].tanh();
        }

        // Project to scalar update
        let update: f64 = self.hidden.iter()
            .zip(self.w_proj.iter())
            .map(|(h, w)| h * w)
            .sum();

        // Clip update to prevent explosions
        update.max(-1.0).min(1.0)
    }

    fn prepare_input(&self, features: &OptimizerInput) -> Vec<f64> {
        let mut input = vec![
            features.gradient,
            features.momentum,
            features.loss,
            features.step as f64 / 1000.0,  // Normalize step
            features.grad_squared,
            features.loss_delta,
        ];
        input.extend_from_slice(&self.hidden);
        input
    }

    fn gate_forward(&self, weights: &[Vec<f64>], input: &[f64]) -> Vec<f64> {
        weights.iter().map(|row| {
            let sum: f64 = row.iter().zip(input.iter()).map(|(w, x)| w * x).sum();
            sigmoid(sum)
        }).collect()
    }

    fn gate_forward_tanh(&self, weights: &[Vec<f64>], input: &[f64]) -> Vec<f64> {
        weights.iter().map(|row| {
            let sum: f64 = row.iter().zip(input.iter()).map(|(w, x)| w * x).sum();
            sum.tanh()
        }).collect()
    }

    /// Get optimizer weights for saving/loading
    pub fn get_weights(&self) -> OptimizerWeights {
        OptimizerWeights {
            w_input: self.w_input.clone(),
            w_forget: self.w_forget.clone(),
            w_output: self.w_output.clone(),
            w_cell: self.w_cell.clone(),
            w_proj: self.w_proj.clone(),
        }
    }

    /// Set optimizer weights
    pub fn set_weights(&mut self, weights: &OptimizerWeights) {
        self.w_input = weights.w_input.clone();
        self.w_forget = weights.w_forget.clone();
        self.w_output = weights.w_output.clone();
        self.w_cell = weights.w_cell.clone();
        self.w_proj = weights.w_proj.clone();
    }

    /// Mutate weights slightly (for evolution)
    pub fn mutate(&mut self, rate: f64) {
        let mut rng = rand::thread_rng();
        let mut mutate_vec = |v: &mut Vec<f64>| {
            for x in v.iter_mut() {
                if rng.gen::<f64>() < rate {
                    *x += rng.gen_range(-0.05..0.05);
                }
            }
        };

        // Mutate all weight matrices
        for row in self.w_input.iter_mut() { mutate_vec(row); }
        for row in self.w_forget.iter_mut() { mutate_vec(row); }
        for row in self.w_output.iter_mut() { mutate_vec(row); }
        for row in self.w_cell.iter_mut() { mutate_vec(row); }
        mutate_vec(&mut self.w_proj);
    }
}

fn sigmoid(x: f64) -> f64 {
    1.0 / (1.0 + (-x).exp())
}

/// Weights snapshot for the optimizer network
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OptimizerWeights {
    pub w_input: Vec<Vec<f64>>,
    pub w_forget: Vec<Vec<f64>>,
    pub w_output: Vec<Vec<f64>>,
    pub w_cell: Vec<Vec<f64>>,
    pub w_proj: Vec<f64>,
}

/// Input features for the optimizer
#[derive(Clone, Debug)]
pub struct OptimizerInput {
    pub gradient: f64,
    pub momentum: f64,
    pub loss: f64,
    pub step: usize,
    pub grad_squared: f64,
    pub loss_delta: f64,
}

impl Default for OptimizerInput {
    fn default() -> Self {
        Self {
            gradient: 0.0,
            momentum: 0.0,
            loss: 1.0,
            step: 0,
            grad_squared: 0.0,
            loss_delta: 0.0,
        }
    }
}

// ============================================================================
// LEARNED OPTIMIZER
// ============================================================================

/// Learned optimizer that uses a neural network to compute updates
#[derive(Clone, Serialize, Deserialize)]
pub struct LearnedOptimizer {
    /// The optimizer network
    network: OptimizerNetwork,
    /// Base learning rate (scales network output)
    base_lr: f64,
    /// Momentum terms per parameter
    momentum: Vec<f64>,
    /// Squared gradient accumulator (for normalization)
    grad_squared_acc: Vec<f64>,
    /// Loss history
    loss_history: VecDeque<f64>,
    /// Current step
    step: usize,
    /// Number of parameters
    num_params: usize,
}

impl LearnedOptimizer {
    pub fn new(num_params: usize, base_lr: f64) -> Self {
        Self {
            network: OptimizerNetwork::new(16),
            base_lr,
            momentum: vec![0.0; num_params],
            grad_squared_acc: vec![0.0; num_params],
            loss_history: VecDeque::with_capacity(100),
            step: 0,
            num_params,
        }
    }

    /// Compute updates for all parameters
    pub fn compute_updates(&mut self, gradients: &[f64], loss: f64) -> Vec<f64> {
        self.step += 1;

        // Update loss history
        self.loss_history.push_back(loss);
        if self.loss_history.len() > 100 {
            self.loss_history.pop_front();
        }

        let loss_delta = if self.loss_history.len() >= 2 {
            let prev = self.loss_history[self.loss_history.len() - 2];
            loss - prev
        } else {
            0.0
        };

        // Reset network state for new optimization step
        self.network.reset();

        let mut updates = Vec::with_capacity(gradients.len());

        for (i, &grad) in gradients.iter().enumerate() {
            // Update momentum (exponential moving average)
            self.momentum[i] = 0.9 * self.momentum[i] + 0.1 * grad;

            // Update squared gradient accumulator
            self.grad_squared_acc[i] = 0.99 * self.grad_squared_acc[i] + 0.01 * grad * grad;

            // Prepare input features
            let features = OptimizerInput {
                gradient: grad.max(-10.0).min(10.0),  // Clip gradients
                momentum: self.momentum[i].max(-10.0).min(10.0),
                loss,
                step: self.step,
                grad_squared: self.grad_squared_acc[i].sqrt(),
                loss_delta,
            };

            // Get update from network
            let raw_update = self.network.compute_update(&features);

            // Scale by base learning rate
            let update = raw_update * self.base_lr;
            updates.push(update);
        }

        updates
    }

    /// Get optimizer weights
    pub fn get_weights(&self) -> OptimizerWeights {
        self.network.get_weights()
    }

    /// Set optimizer weights
    pub fn set_weights(&mut self, weights: &OptimizerWeights) {
        self.network.set_weights(weights);
    }

    /// Reset optimizer state (for new training run)
    pub fn reset(&mut self) {
        self.network.reset();
        self.momentum = vec![0.0; self.num_params];
        self.grad_squared_acc = vec![0.0; self.num_params];
        self.loss_history.clear();
        self.step = 0;
    }

    /// Mutate for evolution-based meta-learning
    pub fn mutate(&mut self, rate: f64) {
        self.network.mutate(rate);
    }

    /// Current step
    pub fn step(&self) -> usize {
        self.step
    }
}

// ============================================================================
// META-LEARNER FOR OPTIMIZER
// ============================================================================

/// Trains the learned optimizer via evolution
pub struct OptimizerMetaLearner {
    /// Population of optimizers
    population: Vec<LearnedOptimizer>,
    /// Fitness scores
    fitness: Vec<f64>,
    /// Best optimizer weights found
    best_weights: Option<OptimizerWeights>,
    /// Best fitness
    best_fitness: f64,
    /// Generation count
    generation: usize,
    /// Elite fraction (top performers kept)
    elite_fraction: f64,
    /// Mutation rate
    mutation_rate: f64,
}

impl OptimizerMetaLearner {
    pub fn new(population_size: usize, num_params: usize, base_lr: f64) -> Self {
        let population: Vec<LearnedOptimizer> = (0..population_size)
            .map(|_| LearnedOptimizer::new(num_params, base_lr))
            .collect();

        Self {
            population,
            fitness: vec![0.0; population_size],
            best_weights: None,
            best_fitness: f64::NEG_INFINITY,
            generation: 0,
            elite_fraction: 0.2,
            mutation_rate: 0.1,
        }
    }

    /// Get an optimizer from the population for evaluation
    pub fn get_optimizer(&mut self, index: usize) -> &mut LearnedOptimizer {
        &mut self.population[index]
    }

    /// Record fitness for an optimizer
    pub fn record_fitness(&mut self, index: usize, fitness: f64) {
        self.fitness[index] = fitness;

        // Track best
        if fitness > self.best_fitness {
            self.best_fitness = fitness;
            self.best_weights = Some(self.population[index].get_weights());
        }
    }

    /// Evolve the population (call after all fitness recorded)
    pub fn evolve(&mut self) -> EvolutionStats {
        self.generation += 1;

        let pop_size = self.population.len();
        let elite_count = (pop_size as f64 * self.elite_fraction).ceil() as usize;

        // Sort by fitness (descending)
        let mut indices: Vec<usize> = (0..pop_size).collect();
        indices.sort_by(|&a, &b| {
            self.fitness[b].partial_cmp(&self.fitness[a]).unwrap()
        });

        let avg_fitness = self.fitness.iter().sum::<f64>() / pop_size as f64;
        let best_idx = indices[0];
        let worst_idx = indices[pop_size - 1];

        // Create new population
        let mut new_population = Vec::with_capacity(pop_size);

        // Keep elite
        for &idx in indices.iter().take(elite_count) {
            new_population.push(self.population[idx].clone());
        }

        // Fill rest with mutated copies of elite
        while new_population.len() < pop_size {
            let parent_idx = indices[rand::random::<usize>() % elite_count];
            let mut child = self.population[parent_idx].clone();
            child.mutate(self.mutation_rate);
            child.reset();  // Reset state for fresh evaluation
            new_population.push(child);
        }

        self.population = new_population;
        self.fitness = vec![0.0; pop_size];

        EvolutionStats {
            generation: self.generation,
            best_fitness: self.fitness[best_idx],
            worst_fitness: self.fitness[worst_idx],
            avg_fitness,
            elite_count,
        }
    }

    /// Get the best optimizer found
    pub fn best_optimizer(&self, num_params: usize, base_lr: f64) -> LearnedOptimizer {
        let mut opt = LearnedOptimizer::new(num_params, base_lr);
        if let Some(weights) = &self.best_weights {
            opt.set_weights(weights);
        }
        opt
    }

    /// Population size
    pub fn population_size(&self) -> usize {
        self.population.len()
    }

    /// Current generation
    pub fn generation(&self) -> usize {
        self.generation
    }

    /// Best fitness found
    pub fn best_fitness(&self) -> f64 {
        self.best_fitness
    }
}

/// Statistics from evolution step
#[derive(Clone, Debug)]
pub struct EvolutionStats {
    pub generation: usize,
    pub best_fitness: f64,
    pub worst_fitness: f64,
    pub avg_fitness: f64,
    pub elite_count: usize,
}

// ============================================================================
// COMPARISON WITH STANDARD OPTIMIZERS
// ============================================================================

/// Standard Adam optimizer for comparison
#[derive(Clone)]
pub struct AdamOptimizer {
    lr: f64,
    beta1: f64,
    beta2: f64,
    epsilon: f64,
    m: Vec<f64>,  // First moment
    v: Vec<f64>,  // Second moment
    t: usize,     // Timestep
}

impl AdamOptimizer {
    pub fn new(num_params: usize, lr: f64) -> Self {
        Self {
            lr,
            beta1: 0.9,
            beta2: 0.999,
            epsilon: 1e-8,
            m: vec![0.0; num_params],
            v: vec![0.0; num_params],
            t: 0,
        }
    }

    pub fn compute_updates(&mut self, gradients: &[f64]) -> Vec<f64> {
        self.t += 1;
        let t = self.t as f64;

        gradients.iter().enumerate().map(|(i, &g)| {
            self.m[i] = self.beta1 * self.m[i] + (1.0 - self.beta1) * g;
            self.v[i] = self.beta2 * self.v[i] + (1.0 - self.beta2) * g * g;

            let m_hat = self.m[i] / (1.0 - self.beta1.powf(t));
            let v_hat = self.v[i] / (1.0 - self.beta2.powf(t));

            -self.lr * m_hat / (v_hat.sqrt() + self.epsilon)
        }).collect()
    }

    pub fn reset(&mut self) {
        self.m = vec![0.0; self.m.len()];
        self.v = vec![0.0; self.v.len()];
        self.t = 0;
    }
}

/// Compare learned optimizer with Adam
pub fn benchmark_optimizers(
    learned: &mut LearnedOptimizer,
    adam: &mut AdamOptimizer,
    num_steps: usize,
) -> BenchmarkResult {
    // Simple quadratic loss for benchmarking
    let target = vec![1.0, 2.0, 3.0, 4.0, 5.0];
    let num_params = target.len();

    // Learned optimizer run
    let mut params_l = vec![0.0; num_params];
    let mut losses_l = Vec::with_capacity(num_steps);
    learned.reset();

    for _ in 0..num_steps {
        let gradients: Vec<f64> = params_l.iter()
            .zip(target.iter())
            .map(|(p, t)| 2.0 * (p - t))
            .collect();
        let loss: f64 = params_l.iter()
            .zip(target.iter())
            .map(|(p, t)| (p - t).powi(2))
            .sum();

        losses_l.push(loss);
        let updates = learned.compute_updates(&gradients, loss);
        for (p, u) in params_l.iter_mut().zip(updates.iter()) {
            *p += u;
        }
    }

    // Adam run
    let mut params_a = vec![0.0; num_params];
    let mut losses_a = Vec::with_capacity(num_steps);
    adam.reset();

    for _ in 0..num_steps {
        let gradients: Vec<f64> = params_a.iter()
            .zip(target.iter())
            .map(|(p, t)| 2.0 * (p - t))
            .collect();
        let loss: f64 = params_a.iter()
            .zip(target.iter())
            .map(|(p, t)| (p - t).powi(2))
            .sum();

        losses_a.push(loss);
        let updates = adam.compute_updates(&gradients);
        for (p, u) in params_a.iter_mut().zip(updates.iter()) {
            *p += u;
        }
    }

    BenchmarkResult {
        learned_final_loss: *losses_l.last().unwrap_or(&f64::MAX),
        adam_final_loss: *losses_a.last().unwrap_or(&f64::MAX),
        learned_convergence_step: losses_l.iter().position(|&l| l < 0.01),
        adam_convergence_step: losses_a.iter().position(|&l| l < 0.01),
        learned_avg_loss: losses_l.iter().sum::<f64>() / losses_l.len() as f64,
        adam_avg_loss: losses_a.iter().sum::<f64>() / losses_a.len() as f64,
    }
}

/// Benchmark comparison result
#[derive(Clone, Debug)]
pub struct BenchmarkResult {
    pub learned_final_loss: f64,
    pub adam_final_loss: f64,
    pub learned_convergence_step: Option<usize>,
    pub adam_convergence_step: Option<usize>,
    pub learned_avg_loss: f64,
    pub adam_avg_loss: f64,
}

impl BenchmarkResult {
    pub fn learned_wins(&self) -> bool {
        self.learned_final_loss < self.adam_final_loss
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optimizer_network() {
        let mut net = OptimizerNetwork::new(8);
        let features = OptimizerInput {
            gradient: 0.5,
            momentum: 0.1,
            loss: 0.3,
            step: 10,
            grad_squared: 0.25,
            loss_delta: -0.05,
        };
        let update = net.compute_update(&features);
        assert!(update.abs() <= 1.0, "Update should be clipped");
    }

    #[test]
    fn test_learned_optimizer() {
        let mut opt = LearnedOptimizer::new(10, 0.001);
        let gradients = vec![0.1; 10];
        let updates = opt.compute_updates(&gradients, 0.5);
        assert_eq!(updates.len(), 10);
    }

    #[test]
    fn test_meta_learner() {
        let mut meta = OptimizerMetaLearner::new(5, 10, 0.001);
        assert_eq!(meta.population_size(), 5);

        // Record some fitness
        for i in 0..5 {
            meta.record_fitness(i, i as f64);
        }

        let stats = meta.evolve();
        assert_eq!(stats.generation, 1);
    }
}
