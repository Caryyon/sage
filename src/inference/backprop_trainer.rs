//! Differentiable NCA Training via Backpropagation
//!
//! Implements manual gradient computation through unrolled NCA steps.
//! No external autograd — hand-derived gradients for the 3-layer MLP update rule.
//!
//! Architecture per cell per step:
//!   input = perceive(3×3 neighborhood) → [144]
//!   h1 = relu(W1 * input + b1)         → [384]
//!   h2 = relu(W2 * h1 + b2)            → [128]
//!   delta = tanh(W3 * h2 + b3) * 0.1   → [16]
//!   grid[r][c] += delta                 (clamped to [-5, 5])
//!
//! Loss: cross-entropy on softmax of activation channel values at token positions.
//! Optimizer: Adam with gradient clipping.

use super::nca_predictor::{
    NcaPredictor, NcaWeights, SimpleTokenizer, TrainingConfig, NCA_CHANNELS,
};

const PERCEPTION_SIZE: usize = 9 * NCA_CHANNELS; // 144
const HIDDEN1_SIZE: usize = 384;
const HIDDEN2_SIZE: usize = 128;
const ACTIVATION_CH: usize = 0;

// ---------------------------------------------------------------------------
// Gradient accumulator for NcaWeights
// ---------------------------------------------------------------------------

/// Stores gradients with same shape as NcaWeights
#[derive(Clone)]
struct NcaGradients {
    dw1: Vec<Vec<f64>>, // HIDDEN1_SIZE × PERCEPTION_SIZE
    db1: Vec<f64>,
    dw2: Vec<Vec<f64>>, // HIDDEN2_SIZE × HIDDEN1_SIZE
    db2: Vec<f64>,
    dw3: Vec<Vec<f64>>, // NCA_CHANNELS × HIDDEN2_SIZE
    db3: Vec<f64>,
}

impl NcaGradients {
    fn zeros() -> Self {
        Self {
            dw1: vec![vec![0.0; PERCEPTION_SIZE]; HIDDEN1_SIZE],
            db1: vec![0.0; HIDDEN1_SIZE],
            dw2: vec![vec![0.0; HIDDEN1_SIZE]; HIDDEN2_SIZE],
            db2: vec![0.0; HIDDEN2_SIZE],
            dw3: vec![vec![0.0; HIDDEN2_SIZE]; NCA_CHANNELS],
            db3: vec![0.0; NCA_CHANNELS],
        }
    }

    /// Add another gradient into this one
    fn accumulate(&mut self, other: &NcaGradients) {
        for h in 0..HIDDEN1_SIZE {
            for i in 0..PERCEPTION_SIZE {
                self.dw1[h][i] += other.dw1[h][i];
            }
            self.db1[h] += other.db1[h];
        }
        for h in 0..HIDDEN2_SIZE {
            for i in 0..HIDDEN1_SIZE {
                self.dw2[h][i] += other.dw2[h][i];
            }
            self.db2[h] += other.db2[h];
        }
        for ch in 0..NCA_CHANNELS {
            for h in 0..HIDDEN2_SIZE {
                self.dw3[ch][h] += other.dw3[ch][h];
            }
            self.db3[ch] += other.db3[ch];
        }
    }

    /// Flatten to vec (same order as NcaWeights::to_vec)
    fn to_vec(&self) -> Vec<f64> {
        let mut v = Vec::new();
        for row in &self.dw1 {
            v.extend(row);
        }
        v.extend(&self.db1);
        for row in &self.dw2 {
            v.extend(row);
        }
        v.extend(&self.db2);
        for row in &self.dw3 {
            v.extend(row);
        }
        v.extend(&self.db3);
        v
    }

    /// Clip gradient norm
    fn clip_norm(&mut self, max_norm: f64) {
        let v = self.to_vec();
        let norm: f64 = v.iter().map(|x| x * x).sum::<f64>().sqrt();
        if norm > max_norm {
            let scale = max_norm / norm;
            for row in &mut self.dw1 {
                for x in row.iter_mut() {
                    *x *= scale;
                }
            }
            for x in &mut self.db1 {
                *x *= scale;
            }
            for row in &mut self.dw2 {
                for x in row.iter_mut() {
                    *x *= scale;
                }
            }
            for x in &mut self.db2 {
                *x *= scale;
            }
            for row in &mut self.dw3 {
                for x in row.iter_mut() {
                    *x *= scale;
                }
            }
            for x in &mut self.db3 {
                *x *= scale;
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Adam optimizer state
// ---------------------------------------------------------------------------

struct AdamState {
    m: Vec<f64>, // first moment
    v: Vec<f64>, // second moment
    t: usize,    // timestep
    lr: f64,
    beta1: f64,
    beta2: f64,
    eps: f64,
}

impl AdamState {
    fn new(n_params: usize, lr: f64) -> Self {
        Self {
            m: vec![0.0; n_params],
            v: vec![0.0; n_params],
            t: 0,
            lr,
            beta1: 0.9,
            beta2: 0.999,
            eps: 1e-8,
        }
    }

    /// Update parameters in-place given gradients. Returns updated params.
    fn step(&mut self, params: &mut [f64], grads: &[f64]) {
        self.t += 1;
        let bc1 = 1.0 - self.beta1.powi(self.t as i32);
        let bc2 = 1.0 - self.beta2.powi(self.t as i32);

        for i in 0..params.len() {
            self.m[i] = self.beta1 * self.m[i] + (1.0 - self.beta1) * grads[i];
            self.v[i] = self.beta2 * self.v[i] + (1.0 - self.beta2) * grads[i] * grads[i];
            let m_hat = self.m[i] / bc1;
            let v_hat = self.v[i] / bc2;
            params[i] -= self.lr * m_hat / (v_hat.sqrt() + self.eps);
        }
    }

    fn set_lr(&mut self, lr: f64) {
        self.lr = lr;
    }
}

// ---------------------------------------------------------------------------
// Intermediate values stored during forward pass for backprop
// ---------------------------------------------------------------------------

/// Per-cell intermediate values for one NCA step
#[allow(dead_code)]
struct CellTrace {
    input: Vec<f64>,     // [PERCEPTION_SIZE] - perception input
    pre_h1: Vec<f64>,    // [HIDDEN1_SIZE] - before relu
    h1: Vec<f64>,        // [HIDDEN1_SIZE] - after relu
    pre_h2: Vec<f64>,    // [HIDDEN2_SIZE] - before relu
    h2: Vec<f64>,        // [HIDDEN2_SIZE] - after relu
    pre_out: Vec<f64>,   // [NCA_CHANNELS] - before tanh
    delta: Vec<f64>,     // [NCA_CHANNELS] - after tanh * 0.1
    pre_clamp: Vec<f64>, // [NCA_CHANNELS] - grid + delta before clamp
}

/// All traces for one NCA step across the whole grid
#[allow(dead_code)]
struct StepTrace {
    cells: Vec<Vec<CellTrace>>,                 // [grid_size][grid_size]
    grid_before: Vec<Vec<[f64; NCA_CHANNELS]>>, // grid state before this step
}

// ---------------------------------------------------------------------------
// Forward pass with recording
// ---------------------------------------------------------------------------

fn forward_with_trace(
    weights: &NcaWeights,
    grid: &mut Vec<Vec<[f64; NCA_CHANNELS]>>,
    grid_size: usize,
    nca_steps: usize,
) -> Vec<StepTrace> {
    let mut traces = Vec::with_capacity(nca_steps);

    for _step in 0..nca_steps {
        let grid_before = grid.clone();
        let mut step_cells: Vec<Vec<CellTrace>> = Vec::with_capacity(grid_size);

        // Compute deltas and traces
        let mut deltas = vec![vec![[0.0; NCA_CHANNELS]; grid_size]; grid_size];

        for r in 0..grid_size {
            let mut row_traces = Vec::with_capacity(grid_size);
            for c in 0..grid_size {
                // Perceive
                let mut input = vec![0.0; PERCEPTION_SIZE];
                let mut idx = 0;
                for dr in [-1i32, 0, 1] {
                    for dc in [-1i32, 0, 1] {
                        let nr = (r as i32 + dr).rem_euclid(grid_size as i32) as usize;
                        let nc = (c as i32 + dc).rem_euclid(grid_size as i32) as usize;
                        for ch in 0..NCA_CHANNELS {
                            input[idx] = grid[nr][nc][ch];
                            idx += 1;
                        }
                    }
                }

                // Layer 1: input → h1 (relu)
                let mut pre_h1 = vec![0.0; HIDDEN1_SIZE];
                let mut h1 = vec![0.0; HIDDEN1_SIZE];
                for h in 0..HIDDEN1_SIZE {
                    let mut sum = weights.b1[h];
                    for i in 0..PERCEPTION_SIZE {
                        sum += weights.w1[h][i] * input[i];
                    }
                    pre_h1[h] = sum;
                    h1[h] = sum.max(0.0);
                }

                // Layer 2: h1 → h2 (relu)
                let mut pre_h2 = vec![0.0; HIDDEN2_SIZE];
                let mut h2 = vec![0.0; HIDDEN2_SIZE];
                for h in 0..HIDDEN2_SIZE {
                    let mut sum = weights.b2[h];
                    for i in 0..HIDDEN1_SIZE {
                        sum += weights.w2[h][i] * h1[i];
                    }
                    pre_h2[h] = sum;
                    h2[h] = sum.max(0.0);
                }

                // Layer 3: h2 → delta (tanh * 0.1)
                let mut pre_out = vec![0.0; NCA_CHANNELS];
                let mut delta = vec![0.0; NCA_CHANNELS];
                for ch in 0..NCA_CHANNELS {
                    let mut sum = weights.b3[ch];
                    for h in 0..HIDDEN2_SIZE {
                        sum += weights.w3[ch][h] * h2[h];
                    }
                    pre_out[ch] = sum;
                    delta[ch] = sum.tanh() * 0.1;
                }

                // Pre-clamp values
                let mut pre_clamp = [0.0; NCA_CHANNELS];
                for ch in 0..NCA_CHANNELS {
                    pre_clamp[ch] = grid[r][c][ch] + delta[ch];
                    deltas[r][c][ch] = delta[ch];
                }

                row_traces.push(CellTrace {
                    input,
                    pre_h1,
                    h1,
                    pre_h2,
                    h2,
                    pre_out,
                    delta,
                    pre_clamp: pre_clamp.to_vec(),
                });
            }
            step_cells.push(row_traces);
        }

        // Apply deltas with clamp
        for r in 0..grid_size {
            for c in 0..grid_size {
                for ch in 0..NCA_CHANNELS {
                    grid[r][c][ch] = (grid[r][c][ch] + deltas[r][c][ch]).clamp(-5.0, 5.0);
                }
            }
        }

        traces.push(StepTrace {
            cells: step_cells,
            grid_before,
        });
    }

    traces
}

// ---------------------------------------------------------------------------
// Backward pass
// ---------------------------------------------------------------------------

/// Compute gradients by backpropagating through the unrolled NCA steps.
///
/// `d_grid` is dL/d(final_grid) — the gradient of loss w.r.t. the final grid state.
/// Returns accumulated weight gradients.
fn backward_through_steps(
    weights: &NcaWeights,
    traces: &[StepTrace],
    mut d_grid: Vec<Vec<[f64; NCA_CHANNELS]>>,
    grid_size: usize,
) -> NcaGradients {
    let mut total_grads = NcaGradients::zeros();

    // Process steps in reverse
    for step_trace in traces.iter().rev() {
        let mut d_grid_prev = vec![vec![[0.0; NCA_CHANNELS]; grid_size]; grid_size];

        for r in 0..grid_size {
            for c in 0..grid_size {
                let trace = &step_trace.cells[r][c];

                // dL/d(grid_after_clamp) = d_grid[r][c]
                // Clamp: if pre_clamp was in [-5, 5], gradient passes through; else 0
                let mut d_post_add = [0.0; NCA_CHANNELS];
                for ch in 0..NCA_CHANNELS {
                    if trace.pre_clamp[ch] >= -5.0 && trace.pre_clamp[ch] <= 5.0 {
                        d_post_add[ch] = d_grid[r][c][ch];
                    }
                }

                // grid_after = grid_before + delta
                // dL/d(grid_before) += d_post_add (residual connection)
                // dL/d(delta) = d_post_add
                for ch in 0..NCA_CHANNELS {
                    d_grid_prev[r][c][ch] += d_post_add[ch];
                }
                let d_delta = d_post_add;

                // delta = tanh(pre_out) * 0.1
                // d(delta)/d(pre_out) = 0.1 * (1 - tanh(pre_out)^2)
                let mut d_pre_out = vec![0.0; NCA_CHANNELS];
                for ch in 0..NCA_CHANNELS {
                    let t = trace.pre_out[ch].tanh();
                    d_pre_out[ch] = d_delta[ch] * 0.1 * (1.0 - t * t);
                }

                // pre_out = W3 * h2 + b3
                let mut d_h2 = vec![0.0; HIDDEN2_SIZE];
                for ch in 0..NCA_CHANNELS {
                    total_grads.db3[ch] += d_pre_out[ch];
                    for h in 0..HIDDEN2_SIZE {
                        total_grads.dw3[ch][h] += d_pre_out[ch] * trace.h2[h];
                        d_h2[h] += weights.w3[ch][h] * d_pre_out[ch];
                    }
                }

                // h2 = relu(pre_h2)
                let mut d_pre_h2 = vec![0.0; HIDDEN2_SIZE];
                for h in 0..HIDDEN2_SIZE {
                    d_pre_h2[h] = if trace.pre_h2[h] > 0.0 { d_h2[h] } else { 0.0 };
                }

                // pre_h2 = W2 * h1 + b2
                let mut d_h1 = vec![0.0; HIDDEN1_SIZE];
                for h in 0..HIDDEN2_SIZE {
                    total_grads.db2[h] += d_pre_h2[h];
                    for i in 0..HIDDEN1_SIZE {
                        total_grads.dw2[h][i] += d_pre_h2[h] * trace.h1[i];
                        d_h1[i] += weights.w2[h][i] * d_pre_h2[h];
                    }
                }

                // h1 = relu(pre_h1)
                let mut d_pre_h1 = vec![0.0; HIDDEN1_SIZE];
                for i in 0..HIDDEN1_SIZE {
                    d_pre_h1[i] = if trace.pre_h1[i] > 0.0 { d_h1[i] } else { 0.0 };
                }

                // pre_h1 = W1 * input + b1
                let mut d_input = vec![0.0; PERCEPTION_SIZE];
                for h in 0..HIDDEN1_SIZE {
                    total_grads.db1[h] += d_pre_h1[h];
                    for i in 0..PERCEPTION_SIZE {
                        total_grads.dw1[h][i] += d_pre_h1[h] * trace.input[i];
                        d_input[i] += weights.w1[h][i] * d_pre_h1[h];
                    }
                }

                // Perception: input is a flattened 3×3 neighborhood read
                // Distribute d_input back to the corresponding grid cells
                let mut idx = 0;
                for dr in [-1i32, 0, 1] {
                    for dc in [-1i32, 0, 1] {
                        let nr = (r as i32 + dr).rem_euclid(grid_size as i32) as usize;
                        let nc = (c as i32 + dc).rem_euclid(grid_size as i32) as usize;
                        for ch in 0..NCA_CHANNELS {
                            d_grid_prev[nr][nc][ch] += d_input[idx];
                            idx += 1;
                        }
                    }
                }
            }
        }

        d_grid = d_grid_prev;
    }

    total_grads
}

// ---------------------------------------------------------------------------
// Loss computation
// ---------------------------------------------------------------------------

/// Cross-entropy loss with softmax.
/// Returns (loss, d_activations) where d_activations[i] = softmax(activations)[i] - target_one_hot[i]
fn cross_entropy_loss(activations: &[f64], target: usize) -> (f64, Vec<f64>) {
    // Softmax
    let max_val = activations
        .iter()
        .cloned()
        .fold(f64::NEG_INFINITY, f64::max);
    let exps: Vec<f64> = activations.iter().map(|&a| (a - max_val).exp()).collect();
    let sum: f64 = exps.iter().sum();
    let probs: Vec<f64> = exps.iter().map(|e| e / sum).collect();

    // Loss = -log(prob[target])
    let loss = -probs[target].max(1e-30).ln();

    // Gradient: softmax - one_hot
    let mut d_act = probs;
    d_act[target] -= 1.0;

    (loss, d_act)
}

// ---------------------------------------------------------------------------
// Token coordinate mapping (same as nca_predictor)
// ---------------------------------------------------------------------------

fn token_to_coord(token_id: usize, grid_size: usize) -> (usize, usize) {
    let row = token_id / grid_size;
    let col = token_id % grid_size;
    (row.min(grid_size - 1), col.min(grid_size - 1))
}

// ---------------------------------------------------------------------------
// Public training function
// ---------------------------------------------------------------------------

/// Backprop training configuration
pub struct BackpropConfig {
    pub learning_rate: f64,
    pub epochs: usize,
    pub grad_clip: f64,
    pub nca_steps: usize,
    pub grid_size: usize,
    pub context_window: usize,
    pub max_examples: usize,
    pub lr_decay: bool, // cosine decay
}

impl Default for BackpropConfig {
    fn default() -> Self {
        Self {
            learning_rate: 0.001,
            epochs: 50,
            grad_clip: 1.0,
            nca_steps: 3,
            grid_size: 8,
            context_window: 5,
            max_examples: 30,
            lr_decay: true,
        }
    }
}

impl From<&TrainingConfig> for BackpropConfig {
    fn from(tc: &TrainingConfig) -> Self {
        Self {
            learning_rate: tc.learning_rate,
            epochs: tc.epochs,
            grad_clip: 1.0,
            nca_steps: tc.nca_steps,
            grid_size: tc.grid_size,
            context_window: tc.context_window,
            max_examples: tc.max_examples,
            lr_decay: true,
        }
    }
}

/// Train the NCA predictor using backpropagation through unrolled NCA steps.
/// Returns (trained_predictor, final_top5_accuracy, random_baseline)
pub fn train_nca_backprop(
    corpus: &str,
    config: &BackpropConfig,
    verbose: bool,
) -> Result<(NcaPredictor, f64, f64), Box<dyn std::error::Error>> {
    let grid_size = config.grid_size;
    let tokenizer = SimpleTokenizer::from_corpus(corpus, grid_size * grid_size);
    let tokens = tokenizer.encode(corpus);
    let vocab_size = tokenizer.vocab_size();

    if tokens.len() < config.context_window + 1 {
        return Err("Corpus too small for training".into());
    }

    let random_accuracy = 1.0 / vocab_size as f64;

    // Build training examples
    let max_examples = config
        .max_examples
        .min(tokens.len() - config.context_window);
    let step = ((tokens.len() - config.context_window) / max_examples).max(1);
    let examples: Vec<(Vec<usize>, usize)> = (0..tokens.len() - config.context_window)
        .step_by(step)
        .take(max_examples)
        .map(|i| {
            let ctx = tokens[i..i + config.context_window].to_vec();
            let target = tokens[i + config.context_window];
            (ctx, target)
        })
        .collect();

    if verbose {
        eprintln!("📊 Optimizer: backprop (Adam)");
        eprintln!(
            "📊 Vocab: {}, Tokens: {}, Grid: {}×{}, Random: {:.4}%",
            vocab_size,
            tokens.len(),
            grid_size,
            grid_size,
            random_accuracy * 100.0
        );
        let pc = NcaWeights::random().param_count();
        eprintln!("📊 NCA params: {} ({:.1} KB)", pc, pc as f64 * 8.0 / 1024.0);
        eprintln!(
            "📊 Training examples: {}, NCA steps: {}, LR: {}, Epochs: {}",
            examples.len(),
            config.nca_steps,
            config.learning_rate,
            config.epochs
        );
        eprintln!(
            "📊 Grad clip: {}, LR decay: {}",
            config.grad_clip, config.lr_decay
        );
    }

    let mut weights = NcaWeights::random();
    let n_params = weights.param_count();
    let mut adam = AdamState::new(n_params, config.learning_rate);

    let mut best_accuracy = 0.0;
    let mut best_weights = weights.clone();

    for epoch in 0..config.epochs {
        // Cosine learning rate decay
        if config.lr_decay {
            let progress = epoch as f64 / config.epochs as f64;
            let lr = config.learning_rate * 0.5 * (1.0 + (std::f64::consts::PI * progress).cos());
            adam.set_lr(lr);
        }

        let mut epoch_loss = 0.0;
        let mut epoch_grads = NcaGradients::zeros();

        // Accumulate gradients over all examples (full batch)
        for (ctx, target) in &examples {
            // Initialize grid
            let mut grid = vec![vec![[0.0f64; NCA_CHANNELS]; grid_size]; grid_size];

            // Activate context tokens
            for (pos, &tid) in ctx.iter().enumerate() {
                let (r, c) = token_to_coord(tid, grid_size);
                grid[r][c][ACTIVATION_CH] = 1.0;
                let pos_norm = if ctx.len() > 1 {
                    pos as f64 / (ctx.len() - 1) as f64
                } else {
                    1.0
                };
                grid[r][c][1] = pos_norm;
                grid[r][c][2] = (pos + 1) as f64 / ctx.len() as f64;
            }

            // Forward with trace
            let traces = forward_with_trace(&weights, &mut grid, grid_size, config.nca_steps);

            // Read activations for all vocab tokens
            let mut activations = vec![0.0; vocab_size];
            for tid in 0..vocab_size {
                let (r, c) = token_to_coord(tid, grid_size);
                activations[tid] = grid[r][c][ACTIVATION_CH];
            }

            // Compute loss
            let (loss, d_activations) = cross_entropy_loss(&activations, *target);
            epoch_loss += loss;

            // Convert d_activations to d_grid
            let mut d_grid = vec![vec![[0.0; NCA_CHANNELS]; grid_size]; grid_size];
            for tid in 0..vocab_size {
                let (r, c) = token_to_coord(tid, grid_size);
                // Multiple tokens might map to same cell — accumulate
                d_grid[r][c][ACTIVATION_CH] += d_activations[tid];
            }

            // Backward
            let grads = backward_through_steps(&weights, &traces, d_grid, grid_size);
            epoch_grads.accumulate(&grads);
        }

        // Average gradients
        let n_ex = examples.len() as f64;
        epoch_loss /= n_ex;

        // Scale gradients by 1/n_examples
        for row in &mut epoch_grads.dw1 {
            for x in row.iter_mut() {
                *x /= n_ex;
            }
        }
        for x in &mut epoch_grads.db1 {
            *x /= n_ex;
        }
        for row in &mut epoch_grads.dw2 {
            for x in row.iter_mut() {
                *x /= n_ex;
            }
        }
        for x in &mut epoch_grads.db2 {
            *x /= n_ex;
        }
        for row in &mut epoch_grads.dw3 {
            for x in row.iter_mut() {
                *x /= n_ex;
            }
        }
        for x in &mut epoch_grads.db3 {
            *x /= n_ex;
        }

        // Clip gradients
        epoch_grads.clip_norm(config.grad_clip);

        // Adam update
        let mut params = weights.to_vec();
        let grad_vec = epoch_grads.to_vec();
        adam.step(&mut params, &grad_vec);
        weights = NcaWeights::from_vec(&params);

        // Evaluate top-5 accuracy
        let accuracy = evaluate_top5(&tokenizer, &weights, &examples, grid_size, config.nca_steps);

        if accuracy > best_accuracy {
            best_accuracy = accuracy;
            best_weights = weights.clone();
        }

        if verbose {
            eprintln!(
                "  Epoch {}/{}: loss = {:.4}, top-5 = {:.2}% (best = {:.2}%, random = {:.4}%)",
                epoch + 1,
                config.epochs,
                epoch_loss,
                accuracy * 100.0,
                best_accuracy * 100.0,
                random_accuracy * 100.0
            );
        }
    }

    let predictor = NcaPredictor::with_grid_size(tokenizer, best_weights, 10, grid_size);
    Ok((predictor, best_accuracy, random_accuracy))
}

/// Evaluate top-5 accuracy (same metric as CMA-ES for fair comparison)
fn evaluate_top5(
    tokenizer: &SimpleTokenizer,
    weights: &NcaWeights,
    examples: &[(Vec<usize>, usize)],
    grid_size: usize,
    nca_steps: usize,
) -> f64 {
    let vocab_size = tokenizer.vocab_size();
    let mut correct = 0;

    for (ctx, target) in examples {
        let mut grid = vec![vec![[0.0f64; NCA_CHANNELS]; grid_size]; grid_size];
        for (pos, &tid) in ctx.iter().enumerate() {
            let (r, c) = token_to_coord(tid, grid_size);
            grid[r][c][ACTIVATION_CH] = 1.0;
            let pos_norm = if ctx.len() > 1 {
                pos as f64 / (ctx.len() - 1) as f64
            } else {
                1.0
            };
            grid[r][c][1] = pos_norm;
            grid[r][c][2] = (pos + 1) as f64 / ctx.len() as f64;
        }

        // Forward (no trace needed for eval)
        for _ in 0..nca_steps {
            let mut deltas = vec![vec![[0.0; NCA_CHANNELS]; grid_size]; grid_size];
            for r in 0..grid_size {
                for c in 0..grid_size {
                    let mut input = [0.0; PERCEPTION_SIZE];
                    let mut idx = 0;
                    for dr in [-1i32, 0, 1] {
                        for dc in [-1i32, 0, 1] {
                            let nr = (r as i32 + dr).rem_euclid(grid_size as i32) as usize;
                            let nc = (c as i32 + dc).rem_euclid(grid_size as i32) as usize;
                            for ch in 0..NCA_CHANNELS {
                                input[idx] = grid[nr][nc][ch];
                                idx += 1;
                            }
                        }
                    }
                    let mut h1 = vec![0.0; HIDDEN1_SIZE];
                    for h in 0..HIDDEN1_SIZE {
                        let mut sum = weights.b1[h];
                        for i in 0..PERCEPTION_SIZE {
                            sum += weights.w1[h][i] * input[i];
                        }
                        h1[h] = sum.max(0.0);
                    }
                    let mut h2 = vec![0.0; HIDDEN2_SIZE];
                    for h in 0..HIDDEN2_SIZE {
                        let mut sum = weights.b2[h];
                        for i in 0..HIDDEN1_SIZE {
                            sum += weights.w2[h][i] * h1[i];
                        }
                        h2[h] = sum.max(0.0);
                    }
                    for ch in 0..NCA_CHANNELS {
                        let mut sum = weights.b3[ch];
                        for h in 0..HIDDEN2_SIZE {
                            sum += weights.w3[ch][h] * h2[h];
                        }
                        deltas[r][c][ch] = sum.tanh() * 0.1;
                    }
                }
            }
            for r in 0..grid_size {
                for c in 0..grid_size {
                    for ch in 0..NCA_CHANNELS {
                        grid[r][c][ch] = (grid[r][c][ch] + deltas[r][c][ch]).clamp(-5.0, 5.0);
                    }
                }
            }
        }

        let mut indexed: Vec<(usize, f64)> = (0..vocab_size.min(grid_size * grid_size))
            .map(|id| {
                let r = id / grid_size;
                let c = id % grid_size;
                (id, grid[r][c][ACTIVATION_CH])
            })
            .collect();
        indexed.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        if indexed.iter().take(5).any(|(id, _)| id == target) {
            correct += 1;
        }
    }

    correct as f64 / examples.len() as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cross_entropy_loss() {
        let activations = vec![2.0, 1.0, 0.1];
        let (loss, grads) = cross_entropy_loss(&activations, 0);
        // Loss should be positive
        assert!(loss > 0.0);
        // Gradient for target should be negative (softmax - 1)
        assert!(grads[0] < 0.0);
        // Other gradients should be positive (softmax - 0)
        assert!(grads[1] > 0.0);
        assert!(grads[2] > 0.0);
        // Gradients should sum to ~0
        let sum: f64 = grads.iter().sum();
        assert!(sum.abs() < 1e-10, "Grad sum: {}", sum);
    }

    #[test]
    fn test_gradient_clip() {
        let mut grads = NcaGradients::zeros();
        grads.db1[0] = 100.0;
        grads.clip_norm(1.0);
        let v = grads.to_vec();
        let norm: f64 = v.iter().map(|x| x * x).sum::<f64>().sqrt();
        assert!((norm - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_adam_step() {
        let mut adam = AdamState::new(3, 0.01);
        let mut params = vec![1.0, 2.0, 3.0];
        let grads = vec![0.1, 0.2, 0.3];
        adam.step(&mut params, &grads);
        // Params should have decreased (positive gradient → decrease)
        assert!(params[0] < 1.0);
        assert!(params[1] < 2.0);
        assert!(params[2] < 3.0);
    }

    #[test]
    fn test_backprop_smoke() {
        // Tiny training run to make sure nothing panics
        let corpus = "the cat sat on the mat the dog sat on the log the cat sat on the dog";
        let config = BackpropConfig {
            learning_rate: 0.001,
            epochs: 2,
            grad_clip: 1.0,
            nca_steps: 1,
            grid_size: 4,
            context_window: 3,
            max_examples: 3,
            lr_decay: false,
        };
        let result = train_nca_backprop(corpus, &config, false);
        assert!(result.is_ok());
    }
}
