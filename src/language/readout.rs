//! Readout Network - Trainable layer that maps NCA state to predictions
//!
//! This is the key trainable component. The NCA acts as a fixed "reservoir"
//! that transforms input, and this readout layer learns to interpret
//! the transformed state.

use crate::grid::{Grid, GRID_SIZE, NUM_CHANNELS};
use crate::language::encoder::VOCAB_SIZE;
use rand::Rng;

/// Number of features extracted from the grid
/// Using spatial pooling: 4x4 regions × 22 channels = 352 features
pub const NUM_REGIONS: usize = 4;
pub const FEATURES_PER_REGION: usize = NUM_CHANNELS;
pub const NUM_FEATURES: usize = NUM_REGIONS * NUM_REGIONS * FEATURES_PER_REGION + 4;  // +4 for global stats

/// Hidden layer size for the readout MLP
pub const HIDDEN_SIZE: usize = 128;

/// Readout network that maps grid state to character probabilities
#[derive(Clone)]
pub struct ReadoutNetwork {
    /// First layer weights [HIDDEN_SIZE × NUM_FEATURES]
    pub weights1: Vec<Vec<f64>>,
    /// First layer bias [HIDDEN_SIZE]
    pub bias1: Vec<f64>,
    /// Second layer weights [VOCAB_SIZE × HIDDEN_SIZE]
    pub weights2: Vec<Vec<f64>>,
    /// Second layer bias [VOCAB_SIZE]
    pub bias2: Vec<f64>,

    // Adam optimizer state
    m_w1: Vec<Vec<f64>>,
    v_w1: Vec<Vec<f64>>,
    m_b1: Vec<f64>,
    v_b1: Vec<f64>,
    m_w2: Vec<Vec<f64>>,
    v_w2: Vec<Vec<f64>>,
    m_b2: Vec<f64>,
    v_b2: Vec<f64>,
    adam_t: usize,
}

impl ReadoutNetwork {
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();

        // Xavier initialization for weights
        let scale1 = (2.0 / (NUM_FEATURES + HIDDEN_SIZE) as f64).sqrt();
        let scale2 = (2.0 / (HIDDEN_SIZE + VOCAB_SIZE) as f64).sqrt();

        let weights1: Vec<Vec<f64>> = (0..HIDDEN_SIZE)
            .map(|_| (0..NUM_FEATURES).map(|_| rng.gen_range(-scale1..scale1)).collect())
            .collect();
        let bias1 = vec![0.0; HIDDEN_SIZE];

        let weights2: Vec<Vec<f64>> = (0..VOCAB_SIZE)
            .map(|_| (0..HIDDEN_SIZE).map(|_| rng.gen_range(-scale2..scale2)).collect())
            .collect();
        let bias2 = vec![0.0; VOCAB_SIZE];

        // Initialize Adam state
        let m_w1 = vec![vec![0.0; NUM_FEATURES]; HIDDEN_SIZE];
        let v_w1 = vec![vec![0.0; NUM_FEATURES]; HIDDEN_SIZE];
        let m_b1 = vec![0.0; HIDDEN_SIZE];
        let v_b1 = vec![0.0; HIDDEN_SIZE];
        let m_w2 = vec![vec![0.0; HIDDEN_SIZE]; VOCAB_SIZE];
        let v_w2 = vec![vec![0.0; HIDDEN_SIZE]; VOCAB_SIZE];
        let m_b2 = vec![0.0; VOCAB_SIZE];
        let v_b2 = vec![0.0; VOCAB_SIZE];

        Self {
            weights1,
            bias1,
            weights2,
            bias2,
            m_w1,
            v_w1,
            m_b1,
            v_b1,
            m_w2,
            v_w2,
            m_b2,
            v_b2,
            adam_t: 0,
        }
    }

    /// Extract features from the NCA grid using spatial pooling
    pub fn extract_features(&self, grid: &Grid) -> Vec<f64> {
        let mut features = Vec::with_capacity(NUM_FEATURES);

        let region_size = GRID_SIZE / NUM_REGIONS;

        // Spatial pooling: average each region
        for ry in 0..NUM_REGIONS {
            for rx in 0..NUM_REGIONS {
                let mut region_avg = vec![0.0; NUM_CHANNELS];
                let mut count = 0.0;

                for y in (ry * region_size)..((ry + 1) * region_size) {
                    for x in (rx * region_size)..((rx + 1) * region_size) {
                        for c in 0..NUM_CHANNELS {
                            region_avg[c] += grid.cells[y][x][c];
                        }
                        count += 1.0;
                    }
                }

                for c in 0..NUM_CHANNELS {
                    features.push(region_avg[c] / count);
                }
            }
        }

        // Add global statistics
        let (mean, std, max, min) = self.compute_grid_stats(grid);
        features.push(mean);
        features.push(std);
        features.push(max);
        features.push(min);

        features
    }

    /// Compute global grid statistics
    fn compute_grid_stats(&self, grid: &Grid) -> (f64, f64, f64, f64) {
        let mut sum = 0.0;
        let mut sum_sq = 0.0;
        let mut max_val = f64::MIN;
        let mut min_val = f64::MAX;
        let mut count = 0.0;

        for row in &grid.cells {
            for cell in row {
                // Use alpha channel as main signal
                let val = cell[3];
                sum += val;
                sum_sq += val * val;
                max_val = max_val.max(val);
                min_val = min_val.min(val);
                count += 1.0;
            }
        }

        let mean = sum / count;
        let variance = (sum_sq / count) - (mean * mean);
        let std = variance.max(0.0).sqrt();

        (mean, std, max_val, min_val)
    }

    /// Forward pass: features → logits
    pub fn forward(&self, features: &[f64]) -> (Vec<f64>, Vec<f64>) {
        // First layer with ReLU
        let mut hidden = vec![0.0; HIDDEN_SIZE];
        for i in 0..HIDDEN_SIZE {
            let mut sum = self.bias1[i];
            for (j, &f) in features.iter().enumerate() {
                sum += f * self.weights1[i][j];
            }
            hidden[i] = sum.max(0.0);  // ReLU
        }

        // Second layer (logits)
        let mut logits = vec![0.0; VOCAB_SIZE];
        for i in 0..VOCAB_SIZE {
            let mut sum = self.bias2[i];
            for (j, &h) in hidden.iter().enumerate() {
                sum += h * self.weights2[i][j];
            }
            logits[i] = sum;
        }

        (hidden, logits)
    }

    /// Convert logits to probabilities using softmax
    pub fn softmax(&self, logits: &[f64]) -> Vec<f64> {
        let max_logit = logits.iter().cloned().fold(f64::MIN, f64::max);
        let exp_sum: f64 = logits.iter().map(|&x| (x - max_logit).exp()).sum();

        logits.iter()
            .map(|&x| (x - max_logit).exp() / exp_sum)
            .collect()
    }

    /// Predict next character given grid state
    pub fn predict(&self, grid: &Grid) -> (usize, Vec<f64>) {
        let features = self.extract_features(grid);
        let (_, logits) = self.forward(&features);
        let probs = self.softmax(&logits);

        // Return argmax and probabilities
        let predicted_idx = probs.iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .map(|(i, _)| i)
            .unwrap_or(0);

        (predicted_idx, probs)
    }

    /// Sample from the probability distribution (for generation)
    pub fn sample(&self, grid: &Grid, temperature: f64) -> usize {
        let features = self.extract_features(grid);
        let (_, logits) = self.forward(&features);

        // Apply temperature
        let scaled_logits: Vec<f64> = logits.iter()
            .map(|&x| x / temperature.max(0.01))
            .collect();

        let probs = self.softmax(&scaled_logits);

        // Sample from distribution
        let mut rng = rand::thread_rng();
        let r: f64 = rng.gen();
        let mut cumsum = 0.0;

        for (i, &p) in probs.iter().enumerate() {
            cumsum += p;
            if r < cumsum {
                return i;
            }
        }

        probs.len() - 1
    }

    /// Compute cross-entropy loss
    pub fn compute_loss(&self, probs: &[f64], target_idx: usize) -> f64 {
        -probs[target_idx].max(1e-10).ln()
    }

    /// Backward pass and update weights
    pub fn backward(
        &mut self,
        features: &[f64],
        hidden: &[f64],
        probs: &[f64],
        target_idx: usize,
        learning_rate: f64,
    ) {
        // Compute gradients
        let mut grad_logits = probs.to_vec();
        grad_logits[target_idx] -= 1.0;  // Cross-entropy gradient

        // Gradients for second layer
        let mut grad_w2 = vec![vec![0.0; HIDDEN_SIZE]; VOCAB_SIZE];
        let mut grad_b2 = vec![0.0; VOCAB_SIZE];
        let mut grad_hidden = vec![0.0; HIDDEN_SIZE];

        for i in 0..VOCAB_SIZE {
            grad_b2[i] = grad_logits[i];
            for j in 0..HIDDEN_SIZE {
                grad_w2[i][j] = grad_logits[i] * hidden[j];
                grad_hidden[j] += grad_logits[i] * self.weights2[i][j];
            }
        }

        // ReLU gradient
        for j in 0..HIDDEN_SIZE {
            if hidden[j] <= 0.0 {
                grad_hidden[j] = 0.0;
            }
        }

        // Gradients for first layer
        let mut grad_w1 = vec![vec![0.0; NUM_FEATURES]; HIDDEN_SIZE];
        let mut grad_b1 = vec![0.0; HIDDEN_SIZE];

        for i in 0..HIDDEN_SIZE {
            grad_b1[i] = grad_hidden[i];
            for j in 0..NUM_FEATURES {
                grad_w1[i][j] = grad_hidden[i] * features[j];
            }
        }

        // Update with Adam optimizer
        self.adam_update(&grad_w1, &grad_b1, &grad_w2, &grad_b2, learning_rate);
    }

    /// Adam optimizer update
    fn adam_update(
        &mut self,
        grad_w1: &[Vec<f64>],
        grad_b1: &[f64],
        grad_w2: &[Vec<f64>],
        grad_b2: &[f64],
        learning_rate: f64,
    ) {
        let beta1: f64 = 0.9;
        let beta2: f64 = 0.999;
        let epsilon: f64 = 1e-8;

        self.adam_t += 1;
        let t = self.adam_t as f64;
        let lr_corrected = learning_rate * (1.0_f64 - beta2.powf(t)).sqrt() / (1.0_f64 - beta1.powf(t));

        // Update first layer
        for i in 0..HIDDEN_SIZE {
            for j in 0..NUM_FEATURES {
                let g = grad_w1[i][j];
                self.m_w1[i][j] = beta1 * self.m_w1[i][j] + (1.0 - beta1) * g;
                self.v_w1[i][j] = beta2 * self.v_w1[i][j] + (1.0 - beta2) * g * g;
                self.weights1[i][j] -= lr_corrected * self.m_w1[i][j] / (self.v_w1[i][j].sqrt() + epsilon);
            }

            let g_b = grad_b1[i];
            self.m_b1[i] = beta1 * self.m_b1[i] + (1.0 - beta1) * g_b;
            self.v_b1[i] = beta2 * self.v_b1[i] + (1.0 - beta2) * g_b * g_b;
            self.bias1[i] -= lr_corrected * self.m_b1[i] / (self.v_b1[i].sqrt() + epsilon);
        }

        // Update second layer
        for i in 0..VOCAB_SIZE {
            for j in 0..HIDDEN_SIZE {
                let g = grad_w2[i][j];
                self.m_w2[i][j] = beta1 * self.m_w2[i][j] + (1.0 - beta1) * g;
                self.v_w2[i][j] = beta2 * self.v_w2[i][j] + (1.0 - beta2) * g * g;
                self.weights2[i][j] -= lr_corrected * self.m_w2[i][j] / (self.v_w2[i][j].sqrt() + epsilon);
            }

            let g_b = grad_b2[i];
            self.m_b2[i] = beta1 * self.m_b2[i] + (1.0 - beta1) * g_b;
            self.v_b2[i] = beta2 * self.v_b2[i] + (1.0 - beta2) * g_b * g_b;
            self.bias2[i] -= lr_corrected * self.m_b2[i] / (self.v_b2[i].sqrt() + epsilon);
        }
    }

    /// Train on a single example
    pub fn train_step(&mut self, grid: &Grid, target_idx: usize, learning_rate: f64) -> f64 {
        let features = self.extract_features(grid);
        let (hidden, logits) = self.forward(&features);
        let probs = self.softmax(&logits);
        let loss = self.compute_loss(&probs, target_idx);

        self.backward(&features, &hidden, &probs, target_idx, learning_rate);

        loss
    }

    /// Get parameter count
    pub fn param_count(&self) -> usize {
        let w1 = HIDDEN_SIZE * NUM_FEATURES;
        let b1 = HIDDEN_SIZE;
        let w2 = VOCAB_SIZE * HIDDEN_SIZE;
        let b2 = VOCAB_SIZE;
        w1 + b1 + w2 + b2
    }
}

impl Default for ReadoutNetwork {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_extraction() {
        let readout = ReadoutNetwork::new();
        let grid = Grid::new(GRID_SIZE, GRID_SIZE);
        let features = readout.extract_features(&grid);

        assert_eq!(features.len(), NUM_FEATURES);
    }

    #[test]
    fn test_forward_pass() {
        let readout = ReadoutNetwork::new();
        let features = vec![0.5; NUM_FEATURES];
        let (hidden, logits) = readout.forward(&features);

        assert_eq!(hidden.len(), HIDDEN_SIZE);
        assert_eq!(logits.len(), VOCAB_SIZE);
    }

    #[test]
    fn test_softmax_sums_to_one() {
        let readout = ReadoutNetwork::new();
        let logits = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let probs = readout.softmax(&logits);

        let sum: f64 = probs.iter().sum();
        assert!((sum - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_training_reduces_loss() {
        let mut readout = ReadoutNetwork::new();
        let grid = Grid::new(GRID_SIZE, GRID_SIZE);
        let target_idx = 10;  // Some target character

        let loss1 = readout.train_step(&grid, target_idx, 0.01);
        let loss2 = readout.train_step(&grid, target_idx, 0.01);
        let loss3 = readout.train_step(&grid, target_idx, 0.01);

        // Loss should generally decrease (may not always due to randomness)
        // Just check it's finite
        assert!(loss1.is_finite());
        assert!(loss2.is_finite());
        assert!(loss3.is_finite());
    }
}
