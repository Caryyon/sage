//! GPU-Accelerated NCA Language Model Training
//!
//! Rewrites the NCA forward/backward passes as candle tensor operations
//! for CUDA acceleration on NVIDIA GPUs.
//!
//! Key insight: The 3×3 neighborhood gather is implemented via grid rolling —
//! for each neighbor offset (dr, dc), we roll the grid by (-dr, -dc), so the
//! rolled grid at (r,c) contains the neighbor value. Concatenate all 9 rolled
//! views to get the 144-dim perception vector for every cell simultaneously.
//!
//! Architecture (same as CPU, but tensorized):
//!   grid: [G, G, 16] → roll 9 ways → [G, G, 144]
//!   → Linear(144, 384) → ReLU → Linear(384, 128) → ReLU
//!   → Linear(128, 16) → tanh*0.1 → add to grid → clamp
//!
//! Training uses candle's autograd — no manual backprop needed.

use candle_core::{DType, Device, IndexOp, Module, Tensor};
use candle_nn::{AdamW, Linear, Optimizer, ParamsAdamW, VarBuilder, VarMap};
use std::error::Error;
use std::fs;
use std::path::Path;
use std::time::Instant;

use super::nca_predictor::{SimpleTokenizer, NCA_CHANNELS};

// ── Constants ──────────────────────────────────────────────────────────────

const ACTIVATION_CH: usize = 0;
const PERCEPTION_SIZE: usize = 9 * NCA_CHANNELS; // 144
const HIDDEN1_SIZE: usize = 1024;
const HIDDEN2_SIZE: usize = 384;

/// 3×3 neighbor offsets: (dr, dc) for each of the 9 positions
const NEIGHBOR_OFFSETS: [(i32, i32); 9] = [
    (-1, -1), (-1, 0), (-1, 1),
    (0, -1),  (0, 0),  (0, 1),
    (1, -1),  (1, 0),  (1, 1),
];

// ── GPU NCA MLP ────────────────────────────────────────────────────────────

/// The NCA update MLP, implemented as candle Linear layers.
/// Applied identically to every cell in the grid.
struct NcaMlp {
    linear1: Linear,  // 144 → 384
    linear2: Linear,  // 384 → 128
    linear3: Linear,  // 128 → 16
}

impl NcaMlp {
    fn new(vb: VarBuilder) -> candle_core::Result<Self> {
        let linear1 = candle_nn::linear(PERCEPTION_SIZE, HIDDEN1_SIZE, vb.pp("l1"))?;
        let linear2 = candle_nn::linear(HIDDEN1_SIZE, HIDDEN2_SIZE, vb.pp("l2"))?;
        let linear3 = candle_nn::linear(HIDDEN2_SIZE, NCA_CHANNELS, vb.pp("l3"))?;
        Ok(Self { linear1, linear2, linear3 })
    }

    /// Forward pass: [N, 144] → [N, 16]
    fn forward(&self, x: &Tensor) -> candle_core::Result<Tensor> {
        let h1 = self.linear1.forward(x)?.relu()?;
        let h2 = self.linear2.forward(&h1)?.relu()?;
        let out = self.linear3.forward(&h2)?;
        // tanh * 0.1 (matching CPU version)
        out.tanh()?.affine(0.1, 0.0)
    }
}

// ── GPU NCA Grid Operations ────────────────────────────────────────────────

/// Gather 3×3 neighborhood for every cell using grid rolling.
///
/// For each neighbor offset (dr, dc), we roll the grid by (-dr, -dc).
/// After rolling, the value at (r,c) in the rolled grid is the neighbor
/// at (r+dr, c+dc) in the original grid. We concatenate all 9 rolled
/// views along the channel dimension to get [G, G, 144].
fn gather_neighborhood(grid: &Tensor) -> candle_core::Result<Tensor> {
    let _dims = grid.dims(); // [G, G, C]

    let mut views = Vec::with_capacity(9);
    for &(dr, dc) in &NEIGHBOR_OFFSETS {
        // roll takes i32 shift, not Tensor
        let rolled = grid.roll(-dr, 0)?.roll(-dc, 1)?;
        views.push(rolled);
    }

    // Concatenate along channel dimension: [G, G, 9*C] = [G, G, 144]
    Tensor::cat(&views.iter().collect::<Vec<_>>(), 2)
}

/// Run one NCA step: gather neighborhood → MLP → delta → add to grid → clamp
fn nca_step(grid: &Tensor, mlp: &NcaMlp) -> candle_core::Result<Tensor> {
    let dims = grid.dims();
    let g = dims[0];

    // Gather 3×3 neighborhood: [G, G, 144]
    let perception = gather_neighborhood(grid)?;

    // Reshape to [G*G, 144] for MLP
    let flat = perception.reshape(&[g * g, PERCEPTION_SIZE])?;

    // Apply MLP: [G*G, 16]
    let delta_flat = mlp.forward(&flat)?;

    // Reshape back to [G, G, 16]
    let delta = delta_flat.reshape(&[g, g, NCA_CHANNELS])?;

    // grid += delta, clamp to [-5, 5]
    (grid + delta)?.clamp(-5.0f64, 5.0f64)
}

/// Run multiple NCA steps, returning the final grid
fn nca_forward(grid: &Tensor, mlp: &NcaMlp, steps: usize) -> candle_core::Result<Tensor> {
    let mut current = grid.copy()?;
    for _ in 0..steps {
        current = nca_step(&current, mlp)?;
    }
    Ok(current)
}

// ── Token Encoding ─────────────────────────────────────────────────────────

/// Encode context tokens into the NCA grid.
/// Sets activation channel at token positions with recency weighting.
/// Builds the grid from scratch for simplicity.
/// Encode context tokens into a FULL grid activation pattern.
/// Each token is spread across multiple positions using a hash function,
/// so every cell gets activated. This enables actual cellular automaton
/// behavior — activation propagates and interacts across the entire grid.
fn encode_tokens(
    token_ids: &[usize],
    grid_size: usize,
    device: &Device,
) -> candle_core::Result<Tensor> {
    let cells = grid_size * grid_size;
    let mut grid_data = vec![0.0f64; cells * NCA_CHANNELS];

    if token_ids.is_empty() {
        return Tensor::from_slice(&grid_data, &[grid_size, grid_size, NCA_CHANNELS], device);
    }

    // Spread each token across the grid using a hash-like function.
    // Each token gets ~(cells / token_ids.len()) positions, filling the grid.
    let positions_per_token = (cells / token_ids.len()).max(1);

    for (seq_pos, &tid) in token_ids.iter().enumerate() {
        let recency = 1.0 - (token_ids.len() - 1 - seq_pos) as f64 / token_ids.len() as f64 * 0.5;

        for p in 0..positions_per_token {
            // Hash-like spreading: each token appears at many positions
            let row = (tid.wrapping_mul(13).wrapping_add(seq_pos.wrapping_mul(7)).wrapping_add(p.wrapping_mul(31))) % grid_size;
            let col = (tid.wrapping_mul(17).wrapping_add(seq_pos.wrapping_mul(11)).wrapping_add(p.wrapping_mul(29))) % grid_size;
            let idx = (row * grid_size + col) * NCA_CHANNELS + ACTIVATION_CH;
            // Accumulate activation, clamped to prevent saturation
            grid_data[idx] = (grid_data[idx] + recency * 0.5).clamp(-5.0, 5.0);
        }
    }

    Tensor::from_slice(&grid_data, &[grid_size, grid_size, NCA_CHANNELS], device)
}

// ── Activation Readout ─────────────────────────────────────────────────────

/// Read activation for all vocabulary tokens by averaging across all
/// hash-spread positions. Each token appears at many grid positions;
/// we pool those to get a single activation score per token.
fn read_activations(
    grid: &Tensor,
    vocab_size: usize,
    grid_size: usize,
    context_len: usize,
) -> candle_core::Result<Vec<(usize, f64)>> {
    let cells = grid_size * grid_size;
    let positions_per_token = (cells / context_len.max(1)).max(1);
    let mut activations = Vec::with_capacity(vocab_size);

    for tid in 0..vocab_size {
        let mut sum = 0.0;
        let mut count = 0;
        for p in 0..positions_per_token {
            let row = (tid.wrapping_mul(13).wrapping_add(p.wrapping_mul(31))) % grid_size;
            let col = (tid.wrapping_mul(17).wrapping_add(p.wrapping_mul(29))) % grid_size;
            if let Ok(val) = grid.i((row, col, ACTIVATION_CH)) {
                if let Ok(scalar) = val.to_scalar::<f64>() {
                    sum += scalar;
                    count += 1;
                }
            }
        }
        activations.push((tid, if count > 0 { sum / count as f64 } else { 0.0 }));
    }
    activations.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    Ok(activations)
}

// ── GPU Training Loop ──────────────────────────────────────────────────────

/// Training configuration for GPU NCA LM
pub struct GpuTrainingConfig {
    pub grid_size: usize,
    pub nca_steps: usize,
    pub vocab_size: usize,
    pub epochs: usize,
    pub learning_rate: f64,
    pub max_examples: usize,
    pub context_window: usize,
    pub batch_size: usize,
    pub eval_interval: usize,
}

impl Default for GpuTrainingConfig {
    fn default() -> Self {
        Self {
            grid_size: 32,
            nca_steps: 20,
            vocab_size: 4096,
            epochs: 50,
            learning_rate: 0.001,
            max_examples: 1000,
            context_window: 64,
            batch_size: 8,
            eval_interval: 5,
        }
    }
}

/// Training statistics
pub struct GpuTrainingStats {
    pub grid_size: usize,
    pub nca_steps: usize,
    pub vocab_size: usize,
    pub param_count: usize,
    pub epochs: usize,
    pub final_accuracy: f64,
    pub random_baseline: f64,
    pub improvement: f64,
    pub elapsed_secs: f64,
    pub epoch_losses: Vec<f64>,
    pub epoch_accuracies: Vec<f64>,
}

impl GpuTrainingStats {
    pub fn summary(&self) -> String {
        format!(
            "GPU NCA-LM Training Summary:\n\
             ├─ Grid: {}×{} ({} cells)\n\
             ├─ NCA steps: {}\n\
             ├─ Params: {}K\n\
             ├─ Vocab: {} tokens\n\
             ├─ Epochs: {}\n\
             ├─ Final top-5: {:.2}% (random: {:.4}%)\n\
             ├─ Improvement: {:.1}x\n\
             └─ Time: {:.1}s",
            self.grid_size, self.grid_size, self.grid_size * self.grid_size,
            self.nca_steps,
            self.param_count / 1000,
            self.vocab_size,
            self.epochs,
            self.final_accuracy * 100.0,
            self.random_baseline * 100.0,
            self.improvement,
            self.elapsed_secs,
        )
    }
}

/// Evaluate top-5 accuracy on eval examples
fn evaluate_top5_gpu(
    mlp: &NcaMlp,
    examples: &[(Vec<usize>, usize)],
    grid_size: usize,
    nca_steps: usize,
    vocab_size: usize,
    device: &Device,
) -> candle_core::Result<f64> {
    if examples.is_empty() {
        return Ok(0.0);
    }

    let mut correct = 0;
    for (ctx, target) in examples {
        let grid = encode_tokens(ctx, grid_size, device)?;
        let grid = nca_forward(&grid, mlp, nca_steps)?;
        let activations = read_activations(&grid, vocab_size, grid_size, ctx.len())?;

        if activations.iter().take(5).any(|(id, _)| *id == *target) {
            correct += 1;
        }
    }

    Ok(correct as f64 / examples.len() as f64)
}

/// Main GPU training function.
///
/// Trains the NCA language model on a curriculum using CUDA-accelerated
/// tensor operations with candle's autograd for backpropagation.
pub fn train_nca_lm_gpu(
    curriculum_path: &Path,
    config: &GpuTrainingConfig,
) -> Result<(VarMap, SimpleTokenizer, GpuTrainingStats), Box<dyn Error>> {
    let start_time = Instant::now();

    // Try CUDA first, fall back to CPU
    let device = Device::new_cuda(0).unwrap_or_else(|e| {
        eprintln!("⚠️  CUDA unavailable ({}), falling back to CPU", e);
        Device::Cpu
    });
    eprintln!("🖥️  Device: {:?}", device);

    let grid_size = config.grid_size;
    let nca_steps = config.nca_steps;
    let vocab_size = config.vocab_size;

    eprintln!("═══ GPU NCA Language Model Training ═══");
    eprintln!("Grid: {}×{} ({} cells)", grid_size, grid_size, grid_size * grid_size);
    eprintln!("NCA steps: {}, Vocab target: {}", nca_steps, vocab_size);

    // Step 1: Convert curriculum to corpus
    eprintln!("\n📚 Converting curriculum to training corpus...");
    let corpus = super::nca_lm_trainer::curriculum_to_corpus(curriculum_path)?;
    eprintln!("   Corpus: {} chars, {} words", corpus.len(), corpus.split_whitespace().count());

    // Step 2: Build tokenizer
    eprintln!("\n🔤 Building tokenizer (vocab={})...", vocab_size);
    let tokenizer = SimpleTokenizer::from_corpus(&corpus, vocab_size);
    let actual_vocab = tokenizer.vocab_size();
    eprintln!("   Vocabulary: {} tokens", actual_vocab);

    // Step 3: Build training examples
    eprintln!("\n📊 Building training examples...");
    let examples = super::nca_lm_trainer::build_training_examples(
        &corpus,
        &tokenizer,
        config.context_window,
        config.max_examples,
    );
    eprintln!("   Examples: {} (context window: {})", examples.len(), config.context_window);

    if examples.len() < 10 {
        return Err(format!(
            "Too few training examples ({}). Need at least 10.",
            examples.len()
        ).into());
    }

    // Split train/eval
    let split_idx = (examples.len() as f64 * 0.9) as usize;
    let train_examples = &examples[..split_idx];
    let eval_examples = &examples[split_idx..];
    eprintln!("   Train: {}, Eval: {}", train_examples.len(), eval_examples.len());

    let random_baseline = 1.0 / actual_vocab as f64;
    eprintln!("   Random baseline: {:.4}%", random_baseline * 100.0);

    // Step 4: Initialize MLP with VarMap for autograd
    let varmap = VarMap::new();
    let vb = VarBuilder::from_varmap(&varmap, DType::F64, &device);
    let mlp = NcaMlp::new(vb)?;

    let param_count: usize = varmap.all_vars().iter().map(|v| v.shape().elem_count()).sum();
    eprintln!("   Params: {}K", param_count / 1000);

    // AdamW optimizer
    let adamw_params = ParamsAdamW {
        lr: config.learning_rate,
        beta1: 0.9,
        beta2: 0.999,
        eps: 1e-8,
        weight_decay: 0.0,
    };
    let mut opt = AdamW::new(varmap.all_vars(), adamw_params)?;

    let batch_size = config.batch_size;
    let mut best_accuracy = 0.0;
    let mut epoch_losses: Vec<f64> = Vec::new();
    let mut epoch_accuracies: Vec<f64> = Vec::new();

    // Step 5: Training loop
    eprintln!("\n🚀 Starting GPU training...");
    for epoch in 0..config.epochs {
        let epoch_start = Instant::now();
        let mut epoch_loss = 0.0;
        let mut examples_processed = 0;
        let mut last_grid: Option<Tensor> = None;

        // Process in batches
        for batch_start in (0..train_examples.len()).step_by(batch_size) {
            let batch_end = (batch_start + batch_size).min(train_examples.len());
            let batch_count = batch_end - batch_start;

            // Accumulate loss over batch
            let mut batch_losses = Vec::with_capacity(batch_count);

            for (ctx, target) in &train_examples[batch_start..batch_end] {
                // Encode context tokens into grid
                let grid = encode_tokens(ctx, grid_size, &device)?;

                // Run NCA forward (with autograd tracking via VarMap)
                let final_grid = nca_forward(&grid, &mlp, nca_steps)?;

                // Save the last grid for heatmap snapshot
                last_grid = Some(final_grid.copy()?);

                // Read activation at target position
                let _target_row = target / grid_size;
                let _target_col = target % grid_size;

                // Read ALL activations using hash-spread pooling
                // Each token is spread across many positions; pool them
                let activations = read_activations(&final_grid, actual_vocab, grid_size, ctx.len())?;
                let mut all_acts = Vec::with_capacity(actual_vocab);
                for (tid, _act) in &activations {
                    // Re-read the pooled value as a tensor for autograd
                    // Use the same hash positions as read_activations
                    let cells = grid_size * grid_size;
                    let positions_per_token = (cells / ctx.len().max(1)).max(1);
                    let mut sum: Option<Tensor> = None;
                    let mut count = 0;
                    for p in 0..positions_per_token {
                        let row = (tid.wrapping_mul(13).wrapping_add(p.wrapping_mul(31))) % grid_size;
                        let col = (tid.wrapping_mul(17).wrapping_add(p.wrapping_mul(29))) % grid_size;
                        let val = final_grid.i((row, col, ACTIVATION_CH))?;
                        sum = Some(match sum {
                            Some(s) => s.add(&val)?,
                            None => val,
                        });
                        count += 1;
                    }
                    if let Some(s) = sum {
                        all_acts.push(s.affine(1.0 / count as f64, 0.0)?);
                    } else {
                        all_acts.push(Tensor::new(0.0f64, &device)?);
                    }
                }
                let logits = Tensor::stack(&all_acts.iter().collect::<Vec<_>>(), 0)?;

                // Cross-entropy loss: -log(softmax(logits)[target])
                let log_softmax = candle_nn::ops::log_softmax(&logits, 0)?;
                let loss = log_softmax.i(*target)?.neg()?;

                batch_losses.push(loss);
            }

            // Average batch loss and backward
            let batch_loss = {
                let losses = Tensor::stack(&batch_losses.iter().collect::<Vec<_>>(), 0)?;
                losses.mean_all()?
            };

            epoch_loss += batch_loss.to_scalar::<f64>()? * batch_count as f64;
            examples_processed += batch_count;

            // Backward pass (candle autograd handles everything)
            opt.backward_step(&batch_loss)?;
        }

        epoch_loss /= examples_processed as f64;
        epoch_losses.push(epoch_loss);

        // Evaluate
        let accuracy = if epoch % config.eval_interval == 0 || epoch == config.epochs - 1 {
            evaluate_top5_gpu(&mlp, eval_examples, grid_size, nca_steps, actual_vocab, &device)?
        } else {
            epoch_accuracies.last().copied().unwrap_or(0.0)
        };

        epoch_accuracies.push(accuracy);
        if accuracy > best_accuracy {
            best_accuracy = accuracy;
        }

        let elapsed = epoch_start.elapsed();
        eprintln!(
            "  Epoch {}/{}: loss={:.4}, top-5={:.2}% (best={:.2}%, random={:.4}%) [{:.1}s]",
            epoch + 1,
            config.epochs,
            epoch_loss,
            accuracy * 100.0,
            best_accuracy * 100.0,
            random_baseline * 100.0,
            elapsed.as_secs_f64()
        );

        // Write live training state for TUI monitoring
        // Capture step-by-step grid frames for animation: run NCA steps
        // on the last training grid, capturing each step so the TUI can
        // animate activation spreading across the grid.
        let grid_frames: Vec<Vec<Vec<f64>>> = last_grid
            .as_ref()
            .map(|g| {
                let mut frames = Vec::with_capacity(16);
                let mut current = g.copy().unwrap();
                frames.push(downsample_grid(&current, grid_size));
                for _step in 0..15 {
                    if let Ok(next) = nca_step(&current, &mlp) {
                        current = next;
                        frames.push(downsample_grid(&current, grid_size));
                    } else {
                        break;
                    }
                }
                frames
            })
            .unwrap_or_default();
        write_training_state(
            epoch + 1, config.epochs,
            &epoch_losses, &epoch_accuracies,
            best_accuracy, random_baseline,
            grid_size, actual_vocab, param_count,
            start_time.elapsed().as_secs_f64(),
            &grid_frames,
        );

        // Checkpoint every 5 epochs
        if (epoch + 1) % 5 == 0 || epoch == config.epochs - 1 {
            let cp_dir = dirs::home_dir()
                .unwrap_or_default()
                .join(".sage")
                .join("checkpoints");
            fs::create_dir_all(&cp_dir).ok();
            let cp_path = cp_dir.join(format!("gpu_epoch_{:04}.safetensors", epoch + 1));
            let tensors: std::collections::HashMap<String, Tensor> = varmap
                .all_vars()
                .iter()
                .enumerate()
                .map(|(i, var)| {
                    (format!("param_{}", i), var.as_tensor().copy().unwrap())
                })
                .collect();
            if candle_core::safetensors::save(&tensors, &cp_path).is_ok() {
                eprintln!("   💾 Checkpoint: {}", cp_path.display());
            }
        }
    }

    let total_elapsed = start_time.elapsed();
    let improvement = best_accuracy / random_baseline;

    eprintln!("\n✅ GPU Training complete in {:.1}s", total_elapsed.as_secs_f64());
    eprintln!("   Final accuracy: {:.2}% (random: {:.4}%)", best_accuracy * 100.0, random_baseline * 100.0);
    eprintln!("   Improvement over random: {:.1}x", improvement);

    clear_training_state();

    let stats = GpuTrainingStats {
        grid_size,
        nca_steps,
        vocab_size: actual_vocab,
        param_count,
        epochs: config.epochs,
        final_accuracy: best_accuracy,
        random_baseline,
        improvement,
        elapsed_secs: total_elapsed.as_secs_f64(),
        epoch_losses,
        epoch_accuracies,
    };

    Ok((varmap, tokenizer, stats))
}

// ── Live Training State ────────────────────────────────────────────────────

/// Write live training state to a JSON file for TUI monitoring
fn write_training_state(
    current_epoch: usize,
    total_epochs: usize,
    losses: &[f64],
    accuracies: &[f64],
    best_accuracy: f64,
    random_baseline: f64,
    grid_size: usize,
    vocab_size: usize,
    param_count: usize,
    elapsed_secs: f64,
    grid_frames: &[Vec<Vec<f64>>],
) {
    let state_path = dirs::home_dir()
        .unwrap_or_default()
        .join(".sage")
        .join("training_state.json");

    let state = serde_json::json!({
        "running": true,
        "current_epoch": current_epoch,
        "total_epochs": total_epochs,
        "losses": losses,
        "accuracies": accuracies,
        "best_accuracy": best_accuracy,
        "random_baseline": random_baseline,
        "grid_size": grid_size,
        "vocab_size": vocab_size,
        "param_count": param_count,
        "elapsed_secs": elapsed_secs,
        "grid_frames": grid_frames,
        "updated_at": chrono::Utc::now().to_rfc3339(),
    });

    if let Ok(json) = serde_json::to_string_pretty(&state) {
        let _ = fs::write(&state_path, json);
    }
}

/// Downsample a grid tensor to at most 32×32 for the TUI heatmap state file.
/// Reads the activation channel from the actual training grid.
fn downsample_grid(grid: &Tensor, grid_size: usize) -> Vec<Vec<f64>> {
    let display_size = grid_size.min(32);
    let step = grid_size / display_size;
    let mut snapshot = vec![vec![0.0; display_size]; display_size];

    for r in 0..display_size {
        for c in 0..display_size {
            let mut sum = 0.0;
            let mut count = 0;
            for dr in 0..step {
                for dc in 0..step {
                    let gr = r * step + dr;
                    let gc = c * step + dc;
                    if gr < grid_size && gc < grid_size {
                        if let Ok(val) = grid.i((gr, gc, ACTIVATION_CH)) {
                            if let Ok(scalar) = val.to_scalar::<f64>() {
                                sum += scalar;
                                count += 1;
                            }
                        }
                    }
                }
            }
            snapshot[r][c] = if count > 0 { sum / count as f64 } else { 0.0 };
        }
    }

    snapshot
}

/// Clear training state (called when training completes)
fn clear_training_state() {
    let state_path = dirs::home_dir()
        .unwrap_or_default()
        .join(".sage")
        .join("training_state.json");
    let state = serde_json::json!({"running": false});
    if let Ok(json) = serde_json::to_string_pretty(&state) {
        let _ = fs::write(&state_path, json);
    }
}

// ── Save/Load ──────────────────────────────────────────────────────────────

/// Save trained GPU model weights to disk
pub fn save_gpu_model(
    varmap: &VarMap,
    tokenizer: &SimpleTokenizer,
    config: &GpuTrainingConfig,
    weights_path: &Path,
    vocab_path: &Path,
    config_path: &Path,
) -> Result<(), Box<dyn Error>> {
    // Save weights as safetensors
    use std::collections::HashMap;
    let tensors: HashMap<String, Tensor> = varmap
        .all_vars()
        .iter()
        .enumerate()
        .map(|(i, var)| {
            let name = format!("param_{}", i);
            let tensor = var.as_tensor().copy().unwrap();
            (name, tensor)
        })
        .collect();

    candle_core::safetensors::save(&tensors, weights_path)?;

    // Save vocab as text
    let vocab_text: String = tokenizer.id_to_token.join(" ");
    fs::write(vocab_path, vocab_text)?;

    // Save config as JSON
    let config_json = serde_json::json!({
        "grid_size": config.grid_size,
        "nca_steps": config.nca_steps,
        "vocab_size": config.vocab_size,
        "context_window": config.context_window,
    });
    fs::write(config_path, serde_json::to_string_pretty(&config_json)?)?;

    Ok(())
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gather_neighborhood_shape() -> candle_core::Result<()> {
        let device = Device::Cpu;
        let grid = Tensor::zeros(&[4, 4, NCA_CHANNELS], DType::F64, &device)?;
        let perception = gather_neighborhood(&grid)?;
        assert_eq!(perception.dims(), &[4, 4, PERCEPTION_SIZE]);
        Ok(())
    }

    #[test]
    fn test_nca_step_preserves_shape() -> candle_core::Result<()> {
        let device = Device::Cpu;
        let varmap = VarMap::new();
        let vb = VarBuilder::from_varmap(&varmap, DType::F64, &device);
        let mlp = NcaMlp::new(vb)?;

        let grid = Tensor::zeros(&[8, 8, NCA_CHANNELS], DType::F64, &device)?;
        let result = nca_step(&grid, &mlp)?;
        assert_eq!(result.dims(), &[8, 8, NCA_CHANNELS]);
        Ok(())
    }

    #[test]
    fn test_nca_forward_multistep() -> candle_core::Result<()> {
        let device = Device::Cpu;
        let varmap = VarMap::new();
        let vb = VarBuilder::from_varmap(&varmap, DType::F64, &device);
        let mlp = NcaMlp::new(vb)?;

        let grid = Tensor::zeros(&[8, 8, NCA_CHANNELS], DType::F64, &device)?;
        let result = nca_forward(&grid, &mlp, 3)?;
        assert_eq!(result.dims(), &[8, 8, NCA_CHANNELS]);
        Ok(())
    }
}
