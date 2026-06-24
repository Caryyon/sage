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
    for mean in means.iter_mut().take(NCA_CHANNELS) {
        *mean /= n;
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
            for (ch, &val) in cell.iter().take(NCA_CHANNELS).enumerate() {
                quad_sums[q][ch] += val;
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
        for (v, logit) in logits.iter_mut().enumerate() {
            for (f, &feat) in features.iter().enumerate() {
                *logit += self.weights[v][f] * feat;
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
                for (f, &feat) in features.iter().enumerate() {
                    grad_w[v][f] += d * feat;
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
// Retrieval Feedback Loop
// ---------------------------------------------------------------------------

/// A single retrieval event for feedback training.
#[derive(Clone, Debug)]
pub struct RetrievalEvent {
    /// Feature vector extracted from NCA grid state after query
    pub query_features: Vec<f64>,
    /// The retrieved text (for logging)
    pub retrieved_text: String,
    /// Whether the retrieval was relevant (user/system signal)
    pub was_relevant: bool,
}

/// Accumulates retrieval quality signals and periodically fine-tunes the readout.
///
/// After `min_events` events are accumulated, calling `maybe_train()` will run
/// a mini-batch Adam update on the reservoir readout using the relevance signal as supervision.
pub struct RetrievalFeedback {
    /// Accumulated retrieval events (capped at capacity)
    events: Vec<RetrievalEvent>,
    /// Maximum events to store before rolling over
    capacity: usize,
    /// Minimum events before training triggers
    pub min_events: usize,
    /// Number of fine-tuning epochs per training trigger
    pub finetune_epochs: usize,
    /// Learning rate for fine-tuning Adam optimizer
    pub learning_rate: f64,
    /// Number of training rounds completed
    pub rounds_completed: usize,
    /// Adam optimizer state for the binary readout
    m_w: Vec<f64>, // first moment for weights
    v_w: Vec<f64>, // second moment for weights
    m_b: [f64; 2], // first moment for bias [irrelevant, relevant]
    v_b: [f64; 2], // second moment for bias
    adam_t: usize, // Adam step counter
}

impl RetrievalFeedback {
    /// Create a new feedback accumulator.
    ///
    /// `feature_dim` is the dimension of query feature vectors.
    pub fn new(feature_dim: usize) -> Self {
        Self {
            events: Vec::new(),
            capacity: 500,
            min_events: 30, // Lower threshold so feedback learning starts sooner
            finetune_epochs: 5,
            learning_rate: 1e-3,
            rounds_completed: 0,
            m_w: vec![0.0; feature_dim],
            v_w: vec![0.0; feature_dim],
            m_b: [0.0; 2],
            v_b: [0.0; 2],
            adam_t: 0,
        }
    }

    /// Record a retrieval event.
    pub fn record(&mut self, query_features: Vec<f64>, retrieved_text: String, was_relevant: bool) {
        if self.events.len() >= self.capacity {
            // Ring buffer: remove oldest
            self.events.remove(0);
        }
        self.events.push(RetrievalEvent {
            query_features,
            retrieved_text,
            was_relevant,
        });
    }

    /// Returns true when enough events are available to train.
    pub fn should_train(&self) -> bool {
        self.events.len() >= self.min_events
    }

    /// Run a fine-tuning pass on the binary relevance classifier (relevant/irrelevant).
    ///
    /// Uses a 2-class linear model: `score = w·features + b`
    /// with BCE loss and Adam updates. This tightens the readout toward the
    /// retrieval quality signal without touching NCA weights.
    ///
    /// Returns (final_loss, accuracy) on the current event buffer.
    pub fn finetune(&mut self, readout: &mut BinaryRelevanceReadout) -> (f64, f64) {
        let n = self.events.len();
        if n == 0 {
            return (0.0, 0.0);
        }

        let beta1 = 0.9_f64;
        let beta2 = 0.999_f64;
        let eps = 1e-8_f64;

        for _epoch in 0..self.finetune_epochs {
            self.adam_t += 1;
            let t = self.adam_t as f64;

            let mut g_w = vec![0.0f64; readout.weights.len()];
            let mut g_b = [0.0f64; 2];

            for event in &self.events {
                let feat = &event.query_features;
                let label = if event.was_relevant { 1 } else { 0 };

                // Forward: logit for "relevant" class
                let logit: f64 = readout
                    .weights
                    .iter()
                    .zip(feat.iter())
                    .map(|(w, f)| w * f)
                    .sum::<f64>()
                    + readout.bias[1]
                    - readout.bias[0];

                // Sigmoid
                let prob = 1.0 / (1.0 + (-logit).exp());

                // BCE gradient: (prob - label)
                let err = prob - label as f64;

                // Accumulate gradients
                for (i, f) in feat.iter().enumerate() {
                    if i < g_w.len() {
                        g_w[i] += err * f / n as f64;
                    }
                }
                g_b[1] += err / n as f64;
                g_b[0] -= err / n as f64;
            }

            // Adam update for weights
            for (i, w) in readout.weights.iter_mut().enumerate().take(g_w.len()) {
                self.m_w[i] = beta1 * self.m_w[i] + (1.0 - beta1) * g_w[i];
                self.v_w[i] = beta2 * self.v_w[i] + (1.0 - beta2) * g_w[i] * g_w[i];
                let m_hat = self.m_w[i] / (1.0 - beta1.powf(t));
                let v_hat = self.v_w[i] / (1.0 - beta2.powf(t));
                *w -= self.learning_rate * m_hat / (v_hat.sqrt() + eps);
            }

            // Adam update for bias
            for (c, b) in readout.bias.iter_mut().enumerate() {
                self.m_b[c] = beta1 * self.m_b[c] + (1.0 - beta1) * g_b[c];
                self.v_b[c] = beta2 * self.v_b[c] + (1.0 - beta2) * g_b[c] * g_b[c];
                let m_hat = self.m_b[c] / (1.0 - beta1.powf(t));
                let v_hat = self.v_b[c] / (1.0 - beta2.powf(t));
                *b -= self.learning_rate * m_hat / (v_hat.sqrt() + eps);
            }
        }

        self.rounds_completed += 1;

        // Compute final accuracy and loss on the buffer
        let mut correct = 0;
        let mut total_loss = 0.0;
        for event in &self.events {
            let logit: f64 = readout
                .weights
                .iter()
                .zip(event.query_features.iter())
                .map(|(w, f)| w * f)
                .sum::<f64>()
                + readout.bias[1]
                - readout.bias[0];
            let prob = 1.0 / (1.0 + (-logit).exp());
            let label = if event.was_relevant { 1.0 } else { 0.0 };
            total_loss -=
                label * prob.ln().max(-100.0) + (1.0 - label) * (1.0 - prob).ln().max(-100.0);
            if (prob > 0.5) == event.was_relevant {
                correct += 1;
            }
        }

        (total_loss / n as f64, correct as f64 / n as f64)
    }

    /// Call this after each retrieval. If enough events have accumulated,
    /// runs a fine-tuning pass automatically.
    ///
    /// Returns Some((loss, accuracy)) if training ran, None otherwise.
    pub fn maybe_train(&mut self, readout: &mut BinaryRelevanceReadout) -> Option<(f64, f64)> {
        if self.should_train() {
            Some(self.finetune(readout))
        } else {
            None
        }
    }

    /// Current event count.
    pub fn event_count(&self) -> usize {
        self.events.len()
    }

    /// Relevance rate in current buffer.
    pub fn relevance_rate(&self) -> f64 {
        if self.events.is_empty() {
            return 0.0;
        }
        let relevant = self.events.iter().filter(|e| e.was_relevant).count();
        relevant as f64 / self.events.len() as f64
    }
}

/// Binary linear classifier: relevant vs. irrelevant retrieval.
///
/// Weights over the query feature space; trained on retrieval feedback signals.
pub struct BinaryRelevanceReadout {
    /// Feature weights (feature_dim elements)
    pub weights: Vec<f64>,
    /// Class bias: [irrelevant, relevant]
    pub bias: [f64; 2],
    pub feature_dim: usize,
}

impl BinaryRelevanceReadout {
    /// Create a zero-initialized readout for the given feature dimension.
    pub fn new(feature_dim: usize) -> Self {
        Self {
            weights: vec![0.0; feature_dim],
            bias: [0.0, 0.0],
            feature_dim,
        }
    }

    /// Score a feature vector — higher = more likely relevant.
    pub fn score(&self, features: &[f64]) -> f64 {
        let logit: f64 = self
            .weights
            .iter()
            .zip(features.iter())
            .map(|(w, f)| w * f)
            .sum::<f64>()
            + self.bias[1]
            - self.bias[0];
        1.0 / (1.0 + (-logit).exp()) // sigmoid probability
    }

    /// Save to binary file.
    pub fn save(&self, path: &std::path::Path) -> std::io::Result<()> {
        let mut data = Vec::with_capacity((self.feature_dim + 2) * 8);
        for &w in &self.weights {
            data.extend_from_slice(&w.to_le_bytes());
        }
        for &b in &self.bias {
            data.extend_from_slice(&b.to_le_bytes());
        }
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, data)
    }

    /// Load from binary file.
    pub fn load(path: &std::path::Path, feature_dim: usize) -> std::io::Result<Self> {
        let bytes = fs::read(path)?;
        let values: Vec<f64> = bytes
            .chunks_exact(8)
            .map(|c| f64::from_le_bytes(c.try_into().unwrap()))
            .collect();
        if values.len() < feature_dim + 2 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Readout file too small",
            ));
        }
        Ok(Self {
            weights: values[..feature_dim].to_vec(),
            bias: [values[feature_dim], values[feature_dim + 1]],
            feature_dim,
        })
    }
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
        assert!((0.0..=1.0).contains(&top1));
        assert!(top5 >= top1);
        assert!(baseline > 0.0);
    }

    #[test]
    fn test_binary_readout_score() {
        let mut readout = BinaryRelevanceReadout::new(4);
        // With zero weights and equal bias, score should be 0.5
        let score = readout.score(&[1.0, 0.5, -0.5, 0.0]);
        assert!((score - 0.5).abs() < 1e-10, "Zero readout should score 0.5");

        // Set a weight and verify it shifts score
        readout.weights[0] = 1.0;
        let score_pos = readout.score(&[2.0, 0.0, 0.0, 0.0]);
        assert!(
            score_pos > 0.5,
            "Positive feature with positive weight should score > 0.5"
        );
    }

    #[test]
    fn test_retrieval_feedback_accumulates_events() {
        let mut feedback = RetrievalFeedback::new(4);
        feedback.min_events = 100; // Explicit threshold for test
        assert!(!feedback.should_train(), "Should not train with 0 events");

        for i in 0..99 {
            feedback.record(
                vec![i as f64, 0.0, 0.0, 0.0],
                format!("result {}", i),
                i % 3 == 0,
            );
        }
        assert!(
            !feedback.should_train(),
            "Should not train with 99 events (needs 100)"
        );

        feedback.record(vec![1.0, 0.0, 0.0, 0.0], "final".to_string(), true);
        assert!(feedback.should_train(), "Should train with 100 events");
    }

    #[test]
    fn test_retrieval_feedback_finetune_runs() {
        let feature_dim = 8;
        let mut feedback = RetrievalFeedback::new(feature_dim);
        feedback.min_events = 10; // Lower threshold for test

        // Add 10 events: relevant when first feature is positive, irrelevant when negative
        for i in 0..10 {
            let feat = if i < 5 {
                vec![1.0_f64, 0.5, 0.1, 0.2, 0.3, -0.1, 0.0, 0.4] // relevant
            } else {
                vec![-1.0_f64, -0.5, -0.1, -0.2, -0.3, 0.1, 0.0, -0.4] // irrelevant
            };
            feedback.record(feat, format!("result {}", i), i < 5);
        }

        assert!(feedback.should_train());
        let mut readout = BinaryRelevanceReadout::new(feature_dim);
        let (loss, accuracy) = feedback.finetune(&mut readout);

        // Loss should be finite and non-negative
        assert!(loss.is_finite(), "Loss should be finite, got {}", loss);
        assert!(loss >= 0.0, "Loss should be non-negative, got {}", loss);
        // Accuracy should be in [0, 1]
        assert!(
            (0.0..=1.0).contains(&accuracy),
            "Accuracy out of range: {}",
            accuracy
        );

        eprintln!(
            "Retrieval feedback test: loss={:.4} accuracy={:.1}%",
            loss,
            accuracy * 100.0
        );

        // After training, relevant examples should score higher than irrelevant
        let relevant_feat = vec![1.0_f64, 0.5, 0.1, 0.2, 0.3, -0.1, 0.0, 0.4];
        let irrelevant_feat = vec![-1.0_f64, -0.5, -0.1, -0.2, -0.3, 0.1, 0.0, -0.4];

        let relevant_score = readout.score(&relevant_feat);
        let irrelevant_score = readout.score(&irrelevant_feat);

        // After at least some training, the readout should have learned the pattern
        assert!(
            relevant_score > irrelevant_score,
            "Relevant score ({:.4}) should exceed irrelevant score ({:.4}) after training",
            relevant_score,
            irrelevant_score
        );
    }

    #[test]
    fn test_binary_readout_save_load() {
        let mut readout = BinaryRelevanceReadout::new(8);
        readout.weights = vec![0.1, -0.2, 0.3, -0.4, 0.5, -0.6, 0.7, -0.8];
        readout.bias = [0.1, -0.1];

        let path = std::env::temp_dir().join("test_binary_readout.bin");
        readout.save(&path).unwrap();

        let loaded = BinaryRelevanceReadout::load(&path, 8).unwrap();
        assert_eq!(loaded.weights.len(), 8);
        for (w, lw) in readout.weights.iter().zip(loaded.weights.iter()) {
            assert!((w - lw).abs() < 1e-15, "Weight mismatch: {} vs {}", w, lw);
        }
        assert!((readout.bias[0] - loaded.bias[0]).abs() < 1e-15);
        assert!((readout.bias[1] - loaded.bias[1]).abs() < 1e-15);

        let _ = fs::remove_file(&path);
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
        assert!((0.0..=1.0).contains(&top1));
        assert!(top5 >= top1);
    }
}
