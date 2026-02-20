//! Reservoir Computing with Linear Readout for NCA Token Prediction
//!
//! Treats the NCA grid as a fixed dynamical reservoir. Instead of reading
//! activations directly from token-mapped cells, we:
//! 1. Run NCA steps (dynamics are fixed / pre-trained)
//! 2. Extract a feature vector from the full grid state
//! 3. Apply a trained linear readout: logits = W·features + bias
//!
//! The linear readout is trained with Adam optimizer + cross-entropy loss.
//! NCA weights stay frozen during readout training — that's the key insight.

use crate::inference::nca_predictor::{NcaPredictor, NCA_CHANNELS};
#[cfg(test)]
use crate::inference::nca_predictor::{NcaWeights, SimpleTokenizer};
use rand::Rng;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------
// Feature Extraction
// ---------------------------------------------------------------------------

/// Strategy for extracting features from the NCA grid state.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FeatureStrategy {
    /// Flatten all cell channels: grid_size² × NCA_CHANNELS features
    FlatState,
    /// Spatial statistics: per-channel mean/std/max/min + per-quadrant means
    /// Produces 4*NCA_CHANNELS + 4*NCA_CHANNELS = 64 features
    SpatialStats,
}

/// Extract features from a grid state using the given strategy.
pub fn extract_features(grid: &[Vec<[f64; NCA_CHANNELS]>], strategy: FeatureStrategy) -> Vec<f64> {
    match strategy {
        FeatureStrategy::FlatState => extract_flat(grid),
        FeatureStrategy::SpatialStats => extract_spatial_stats(grid),
    }
}

/// Compute feature dimension for a given strategy and grid size.
pub fn feature_dim(grid_size: usize, strategy: FeatureStrategy) -> usize {
    match strategy {
        FeatureStrategy::FlatState => grid_size * grid_size * NCA_CHANNELS,
        FeatureStrategy::SpatialStats => NCA_CHANNELS * 4 + NCA_CHANNELS * 4, // global stats + quadrant means
    }
}

fn extract_flat(grid: &[Vec<[f64; NCA_CHANNELS]>]) -> Vec<f64> {
    let mut features = Vec::with_capacity(grid.len() * grid[0].len() * NCA_CHANNELS);
    for row in grid {
        for cell in row {
            features.extend_from_slice(cell);
        }
    }
    features
}

fn extract_spatial_stats(grid: &[Vec<[f64; NCA_CHANNELS]>]) -> Vec<f64> {
    let gs = grid.len();
    let n = (gs * gs) as f64;
    let half = gs / 2;

    // Per-channel global stats: mean, std, max, min
    let mut means = [0.0f64; NCA_CHANNELS];
    let mut maxs = [f64::NEG_INFINITY; NCA_CHANNELS];
    let mut mins = [f64::INFINITY; NCA_CHANNELS];

    for row in grid {
        for cell in row {
            for ch in 0..NCA_CHANNELS {
                means[ch] += cell[ch];
                if cell[ch] > maxs[ch] {
                    maxs[ch] = cell[ch];
                }
                if cell[ch] < mins[ch] {
                    mins[ch] = cell[ch];
                }
            }
        }
    }
    for ch in 0..NCA_CHANNELS {
        means[ch] /= n;
    }

    // Std dev
    let mut vars = [0.0f64; NCA_CHANNELS];
    for row in grid {
        for cell in row {
            for ch in 0..NCA_CHANNELS {
                vars[ch] += (cell[ch] - means[ch]).powi(2);
            }
        }
    }
    let stds: Vec<f64> = vars.iter().map(|v| (v / n).sqrt()).collect();

    // Quadrant means (TL, TR, BL, BR)
    let mut quad_sums = [[0.0f64; NCA_CHANNELS]; 4];
    let mut quad_counts = [0usize; 4];
    for (r, row) in grid.iter().enumerate() {
        for (c, cell) in row.iter().enumerate() {
            let q = if r < half {
                if c < half {
                    0
                } else {
                    1
                }
            } else if c < half {
                2
            } else {
                3
            };
            quad_counts[q] += 1;
            for ch in 0..NCA_CHANNELS {
                quad_sums[q][ch] += cell[ch];
            }
        }
    }

    let mut features = Vec::with_capacity(NCA_CHANNELS * 8);
    for ch in 0..NCA_CHANNELS {
        features.push(means[ch]);
        features.push(stds[ch]);
        features.push(maxs[ch]);
        features.push(mins[ch]);
    }
    for q in 0..4 {
        let cnt = quad_counts[q].max(1) as f64;
        for ch in 0..NCA_CHANNELS {
            features.push(quad_sums[q][ch] / cnt);
        }
    }
    features
}

// ---------------------------------------------------------------------------
// Linear Readout
// ---------------------------------------------------------------------------

/// Linear readout layer: logits = W·features + bias
pub struct ReservoirReadout {
    /// Weight matrix: vocab_size rows × feature_dim cols (row-major)
    pub weights: Vec<Vec<f64>>,
    /// Bias vector: vocab_size
    pub bias: Vec<f64>,
    pub vocab_size: usize,
    pub feature_dim: usize,
}

impl ReservoirReadout {
    /// Create with random initialization (Xavier)
    pub fn new(vocab_size: usize, feature_dim: usize) -> Self {
        let mut rng = rand::thread_rng();
        let scale = (2.0 / (vocab_size + feature_dim) as f64).sqrt();
        let weights = (0..vocab_size)
            .map(|_| {
                (0..feature_dim)
                    .map(|_| rng.gen_range(-scale..scale))
                    .collect()
            })
            .collect();
        let bias = vec![0.0; vocab_size];
        Self {
            weights,
            bias,
            vocab_size,
            feature_dim,
        }
    }

    /// Compute logits for a feature vector
    pub fn predict(&self, features: &[f64]) -> Vec<f64> {
        assert_eq!(features.len(), self.feature_dim);
        let mut logits = self.bias.clone();
        for v in 0..self.vocab_size {
            for f in 0..self.feature_dim {
                logits[v] += self.weights[v][f] * features[f];
            }
        }
        logits
    }

    /// Softmax of logits
    pub fn softmax(logits: &[f64]) -> Vec<f64> {
        let max = logits.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let exps: Vec<f64> = logits.iter().map(|l| (l - max).exp()).collect();
        let sum: f64 = exps.iter().sum();
        exps.iter().map(|e| e / sum).collect()
    }

    /// Save readout weights to binary file
    pub fn save(&self, path: &Path) -> Result<(), Box<dyn Error>> {
        let mut data = Vec::new();
        // Header: vocab_size, feature_dim
        data.extend_from_slice(&(self.vocab_size as u64).to_le_bytes());
        data.extend_from_slice(&(self.feature_dim as u64).to_le_bytes());
        // Weights
        for row in &self.weights {
            for &w in row {
                data.extend_from_slice(&w.to_le_bytes());
            }
        }
        // Bias
        for &b in &self.bias {
            data.extend_from_slice(&b.to_le_bytes());
        }
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, data)?;
        Ok(())
    }

    /// Load readout weights from binary file
    pub fn load(path: &Path) -> Result<Self, Box<dyn Error>> {
        let data = fs::read(path)?;
        if data.len() < 16 {
            return Err("File too small for readout weights".into());
        }
        let vocab_size = u64::from_le_bytes(data[0..8].try_into()?) as usize;
        let feature_dim = u64::from_le_bytes(data[8..16].try_into()?) as usize;
        let expected = 16 + (vocab_size * feature_dim + vocab_size) * 8;
        if data.len() < expected {
            return Err(format!(
                "File too small: expected {} bytes, got {}",
                expected,
                data.len()
            )
            .into());
        }
        let mut idx = 16;
        let mut weights = Vec::with_capacity(vocab_size);
        for _ in 0..vocab_size {
            let mut row = Vec::with_capacity(feature_dim);
            for _ in 0..feature_dim {
                row.push(f64::from_le_bytes(data[idx..idx + 8].try_into()?));
                idx += 8;
            }
            weights.push(row);
        }
        let mut bias = Vec::with_capacity(vocab_size);
        for _ in 0..vocab_size {
            bias.push(f64::from_le_bytes(data[idx..idx + 8].try_into()?));
            idx += 8;
        }
        Ok(Self {
            weights,
            bias,
            vocab_size,
            feature_dim,
        })
    }
}

// ---------------------------------------------------------------------------
// Adam Optimizer
// ---------------------------------------------------------------------------

struct Adam {
    lr: f64,
    beta1: f64,
    beta2: f64,
    epsilon: f64,
    t: usize,
    // For weights: [vocab_size][feature_dim]
    m_w: Vec<Vec<f64>>,
    v_w: Vec<Vec<f64>>,
    // For bias: [vocab_size]
    m_b: Vec<f64>,
    v_b: Vec<f64>,
}

impl Adam {
    fn new(vocab_size: usize, feature_dim: usize, lr: f64) -> Self {
        Self {
            lr,
            beta1: 0.9,
            beta2: 0.999,
            epsilon: 1e-8,
            t: 0,
            m_w: vec![vec![0.0; feature_dim]; vocab_size],
            v_w: vec![vec![0.0; feature_dim]; vocab_size],
            m_b: vec![0.0; vocab_size],
            v_b: vec![0.0; vocab_size],
        }
    }

    fn step(
        &mut self,
        readout: &mut ReservoirReadout,
        grad_w: &[Vec<f64>],
        grad_b: &[f64],
        l2_lambda: f64,
    ) {
        self.t += 1;
        let bc1 = 1.0 - self.beta1.powi(self.t as i32);
        let bc2 = 1.0 - self.beta2.powi(self.t as i32);

        for v in 0..readout.vocab_size {
            for f in 0..readout.feature_dim {
                let g = grad_w[v][f] + l2_lambda * readout.weights[v][f];
                self.m_w[v][f] = self.beta1 * self.m_w[v][f] + (1.0 - self.beta1) * g;
                self.v_w[v][f] = self.beta2 * self.v_w[v][f] + (1.0 - self.beta2) * g * g;
                let m_hat = self.m_w[v][f] / bc1;
                let v_hat = self.v_w[v][f] / bc2;
                readout.weights[v][f] -= self.lr * m_hat / (v_hat.sqrt() + self.epsilon);
            }
            let g = grad_b[v];
            self.m_b[v] = self.beta1 * self.m_b[v] + (1.0 - self.beta1) * g;
            self.v_b[v] = self.beta2 * self.v_b[v] + (1.0 - self.beta2) * g * g;
            let m_hat = self.m_b[v] / bc1;
            let v_hat = self.v_b[v] / bc2;
            readout.bias[v] -= self.lr * m_hat / (v_hat.sqrt() + self.epsilon);
        }
    }
}

// ---------------------------------------------------------------------------
// Reservoir Trainer
// ---------------------------------------------------------------------------

/// Configuration for reservoir readout training
pub struct ReservoirConfig {
    pub readout_epochs: usize,
    pub learning_rate: f64,
    pub l2_lambda: f64,
    pub feature_strategy: FeatureStrategy,
    pub context_window: usize,
    pub max_examples: usize,
}

impl Default for ReservoirConfig {
    fn default() -> Self {
        Self {
            readout_epochs: 200,
            learning_rate: 0.001,
            l2_lambda: 1e-4,
            feature_strategy: FeatureStrategy::FlatState,
            context_window: 5,
            max_examples: 100,
        }
    }
}

/// Train a linear readout on top of a frozen NCA reservoir.
///
/// Returns (readout, top1_accuracy, top5_accuracy).
pub fn train_reservoir_readout(
    predictor: &mut NcaPredictor,
    corpus: &str,
    config: &ReservoirConfig,
    verbose: bool,
) -> Result<(ReservoirReadout, f64, f64), Box<dyn Error>> {
    let tokens = predictor.tokenizer.encode(corpus);
    let vocab_size = predictor.tokenizer.vocab_size();
    let grid_size = predictor.grid_size();
    let feat_dim = feature_dim(grid_size, config.feature_strategy);

    if tokens.len() < config.context_window + 1 {
        return Err("Corpus too small".into());
    }

    // Build training examples: extract features once (NCA is frozen)
    let max_ex = config
        .max_examples
        .min(tokens.len() - config.context_window);
    let step = ((tokens.len() - config.context_window) / max_ex).max(1);

    if verbose {
        eprintln!("🔬 Reservoir readout training");
        eprintln!(
            "   Feature strategy: {:?}, dim: {}",
            config.feature_strategy, feat_dim
        );
        eprintln!(
            "   Vocab size: {}, Grid: {}×{}",
            vocab_size, grid_size, grid_size
        );
    }

    let mut features_dataset: Vec<Vec<f64>> = Vec::new();
    let mut targets: Vec<usize> = Vec::new();

    for i in (0..tokens.len() - config.context_window)
        .step_by(step)
        .take(max_ex)
    {
        let ctx = &tokens[i..i + config.context_window];
        let target = tokens[i + config.context_window];
        let grid_state = predictor.run_and_get_state(ctx);
        let feats = extract_features(&grid_state, config.feature_strategy);
        features_dataset.push(feats);
        targets.push(target);
    }

    let n_examples = features_dataset.len();
    if verbose {
        eprintln!("   Training examples: {}", n_examples);
    }

    // Train linear readout with Adam
    let mut readout = ReservoirReadout::new(vocab_size, feat_dim);
    let mut optimizer = Adam::new(vocab_size, feat_dim, config.learning_rate);

    for epoch in 0..config.readout_epochs {
        let mut total_loss = 0.0;
        let mut grad_w = vec![vec![0.0; feat_dim]; vocab_size];
        let mut grad_b = vec![0.0; vocab_size];

        for (features, &target) in features_dataset.iter().zip(targets.iter()) {
            let logits = readout.predict(features);
            let probs = ReservoirReadout::softmax(&logits);

            // Cross-entropy loss
            total_loss -= probs[target].max(1e-15).ln();

            // Gradient: d_loss/d_logit = prob - one_hot(target)
            for v in 0..vocab_size {
                let d = probs[v] - if v == target { 1.0 } else { 0.0 };
                grad_b[v] += d;
                for f in 0..feat_dim {
                    grad_w[v][f] += d * features[f];
                }
            }
        }

        // Average gradients
        let inv_n = 1.0 / n_examples as f64;
        for v in 0..vocab_size {
            grad_b[v] *= inv_n;
            for f in 0..feat_dim {
                grad_w[v][f] *= inv_n;
            }
        }

        optimizer.step(&mut readout, &grad_w, &grad_b, config.l2_lambda);

        if verbose && (epoch + 1) % 20 == 0 {
            let avg_loss = total_loss / n_examples as f64;
            let (top1, top5) = evaluate_readout(&readout, &features_dataset, &targets);
            eprintln!(
                "   Epoch {}/{}: loss={:.4}, top1={:.2}%, top5={:.2}%",
                epoch + 1,
                config.readout_epochs,
                avg_loss,
                top1 * 100.0,
                top5 * 100.0
            );
        }
    }

    let (top1, top5) = evaluate_readout(&readout, &features_dataset, &targets);
    Ok((readout, top1, top5))
}

/// Evaluate readout accuracy on a dataset. Returns (top1, top5).
fn evaluate_readout(
    readout: &ReservoirReadout,
    features: &[Vec<f64>],
    targets: &[usize],
) -> (f64, f64) {
    let mut correct1 = 0;
    let mut correct5 = 0;
    for (feats, &target) in features.iter().zip(targets.iter()) {
        let logits = readout.predict(feats);
        let mut indexed: Vec<(usize, f64)> =
            logits.iter().enumerate().map(|(i, &v)| (i, v)).collect();
        indexed.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        if indexed[0].0 == target {
            correct1 += 1;
        }
        if indexed.iter().take(5).any(|(id, _)| *id == target) {
            correct5 += 1;
        }
    }
    let n = features.len() as f64;
    (correct1 as f64 / n, correct5 as f64 / n)
}

/// Default path for readout weights
pub fn default_readout_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".sage")
        .join("nca_readout.bin")
}

// ---------------------------------------------------------------------------
// Standalone Linear Readout (random NCA — the key experiment)
// ---------------------------------------------------------------------------

/// Train a linear readout on a *random* (untrained) NCA reservoir.
/// This is the core reservoir computing experiment: if a linear readout on
/// random NCA dynamics beats random baseline, it proves the NCA grid topology
/// and local update rules create useful representations even without training.
///
/// Returns (readout, top1_accuracy, top5_accuracy, random_baseline).
pub fn train_standalone_readout(
    corpus: &str,
    grid_size: usize,
    nca_steps: usize,
    config: &ReservoirConfig,
    verbose: bool,
) -> Result<(ReservoirReadout, f64, f64, f64), Box<dyn Error>> {
    use crate::inference::nca_predictor::{NcaPredictor, NcaWeights, SimpleTokenizer};

    let tokenizer = SimpleTokenizer::from_corpus(corpus, grid_size * grid_size);
    let vocab_size = tokenizer.vocab_size();
    let random_baseline = 1.0 / vocab_size as f64;

    // Use random NCA weights — the whole point is that we DON'T train these
    let weights = NcaWeights::random();
    let mut predictor = NcaPredictor::with_grid_size(tokenizer, weights, nca_steps, grid_size);

    if verbose {
        eprintln!("🔬 Standalone linear readout (random NCA reservoir)");
        eprintln!(
            "   Grid: {}×{}, NCA steps: {}, Vocab: {}",
            grid_size, grid_size, nca_steps, vocab_size
        );
        eprintln!("   Random baseline: {:.4}%", random_baseline * 100.0);
    }

    let (readout, top1, top5) = train_reservoir_readout(&mut predictor, corpus, config, verbose)?;
    Ok((readout, top1, top5, random_baseline))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_extraction_flat() {
        let grid = vec![vec![[1.0; NCA_CHANNELS]; 4]; 4];
        let feats = extract_features(&grid, FeatureStrategy::FlatState);
        assert_eq!(feats.len(), 4 * 4 * NCA_CHANNELS);
        assert!(feats.iter().all(|&v| (v - 1.0).abs() < 1e-10));
    }

    #[test]
    fn test_feature_extraction_spatial() {
        let grid = vec![vec![[0.5; NCA_CHANNELS]; 4]; 4];
        let feats = extract_features(&grid, FeatureStrategy::SpatialStats);
        assert_eq!(feats.len(), feature_dim(4, FeatureStrategy::SpatialStats));
        // Mean should be 0.5 for all channels
        for ch in 0..NCA_CHANNELS {
            assert!(
                (feats[ch * 4] - 0.5).abs() < 1e-10,
                "mean for ch {} wrong",
                ch
            );
        }
    }

    #[test]
    fn test_readout_predict_dimensions() {
        let readout = ReservoirReadout::new(10, 32);
        let features = vec![0.1; 32];
        let logits = readout.predict(&features);
        assert_eq!(logits.len(), 10);
    }

    #[test]
    fn test_softmax_sums_to_one() {
        let logits = vec![1.0, 2.0, 3.0, 4.0];
        let probs = ReservoirReadout::softmax(&logits);
        let sum: f64 = probs.iter().sum();
        assert!((sum - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_readout_save_load() {
        let readout = ReservoirReadout::new(5, 8);
        let path = std::env::temp_dir().join("test_readout.bin");
        readout.save(&path).unwrap();
        let loaded = ReservoirReadout::load(&path).unwrap();
        assert_eq!(loaded.vocab_size, 5);
        assert_eq!(loaded.feature_dim, 8);
        for v in 0..5 {
            assert!((readout.bias[v] - loaded.bias[v]).abs() < 1e-15);
            for f in 0..8 {
                assert!((readout.weights[v][f] - loaded.weights[v][f]).abs() < 1e-15);
            }
        }
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_standalone_readout_runs() {
        let corpus = "the cat sat on the mat the dog sat on the log the cat ran to the dog";
        let config = ReservoirConfig {
            readout_epochs: 10,
            max_examples: 10,
            context_window: 3,
            ..Default::default()
        };
        let result = super::train_standalone_readout(corpus, 8, 3, &config, false);
        assert!(result.is_ok());
        let (_, top1, top5, baseline) = result.unwrap();
        assert!(top1 >= 0.0 && top1 <= 1.0);
        assert!(top5 >= top1);
        assert!(baseline > 0.0);
    }

    #[test]
    fn test_reservoir_training_runs() {
        let corpus = "the cat sat on the mat the dog sat on the log the cat ran to the dog";
        let tokenizer = SimpleTokenizer::from_corpus(corpus, 64);
        let weights = NcaWeights::random();
        let mut predictor = NcaPredictor::with_grid_size(tokenizer, weights, 3, 8);

        let config = ReservoirConfig {
            readout_epochs: 10,
            max_examples: 10,
            context_window: 3,
            ..Default::default()
        };

        let result = train_reservoir_readout(&mut predictor, corpus, &config, false);
        assert!(result.is_ok());
        let (_, top1, top5) = result.unwrap();
        assert!(top1 >= 0.0 && top1 <= 1.0);
        assert!(top5 >= top1);
    }
}
