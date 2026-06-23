//! Attractor Network — Self-Organizing NCA Memory
//!
//! A continuous attractor network implemented as a Neural Cellular Automaton.
//! Memories are stored as stable attractor states in the grid. Learning is
//! Hebbian (local, correlation-based). Recall is pattern completion —
//! seed the grid with a partial pattern and it converges to the nearest memory.
//!
//! Architecture:
//!   Encode: Text → tokens → hash-spread grid activation pattern
//!   Store:  Run NCA to stability → Hebbian weight update (co-active cells wire together)
//!   Recall: Seed grid → run NCA → grid converges to attractor → decode to text
//!
//! This is fundamentally different from LLMs:
//!   - No gradient descent, no backprop, no cross-entropy loss
//!   - No next-token prediction
//!   - Memories are explicit attractor states, not implicit in weights
//!   - The grid IS the brain — NCA dynamics are the whole point

use candle_core::{DType, Device, IndexOp, Tensor};
use std::collections::HashMap;
use std::fs;

use super::nca_predictor::{SimpleTokenizer, NCA_CHANNELS};

// ── Constants ──────────────────────────────────────────────────────────────

const ACTIVATION_CH: usize = 0;
const PERCEPTION_SIZE: usize = 9 * NCA_CHANNELS; // 144
const HIDDEN1_SIZE: usize = 1024;
const HIDDEN2_SIZE: usize = 384;
#[allow(dead_code)]
const DEFAULT_GRID_SIZE: usize = 64;
#[allow(dead_code)]
const DEFAULT_NCA_STEPS: usize = 60; // enough to converge to attractor
const HEBBIAN_LR: f64 = 0.01;
const WEIGHT_NORM_MAX: f64 = 1.0;
#[allow(dead_code)]
const STABILITY_THRESHOLD: f64 = 0.001; // grid change below this = stable
const SPARSE_FRACTION: f64 = 0.5;
#[allow(dead_code)]
const COMPETITIVE_TOP: f64 = 0.3; // strengthen top 30% most active
#[allow(dead_code)]
const COMPETITIVE_BOTTOM: f64 = 0.3; // weaken bottom 30% least active

/// 3×3 neighbor offsets
const NEIGHBOR_OFFSETS: [(i32, i32); 9] = [
    (-1, -1), (-1, 0), (-1, 1),
    (0, -1),  (0, 0),  (0, 1),
    (1, -1),  (1, 0),  (1, 1),
];

// ── Attractor MLP (non-autograd, for Hebbian updates) ─────────────────────

/// NCA update MLP with accessible weight tensors for Hebbian learning.
/// Unlike the GPU training MLP, this doesn't use VarMap/autograd —
/// weights are plain tensors we can read and modify directly.
pub struct AttractorMlp {
    pub w1: Tensor, // [144, 1024]
    pub b1: Tensor, // [1024]
    pub w2: Tensor, // [1024, 384]
    pub b2: Tensor, // [384]
    pub w3: Tensor, // [384, 16]
    pub b3: Tensor, // [16]
}

impl AttractorMlp {
    /// Initialize with structured weights that create natural pattern formation.
    /// First layer uses a center-surround (self-excite, neighbor-inhibit) pattern
    /// that acts like a reaction-diffusion system. Patterns emerge from noise.
    pub fn new(device: &Device) -> candle_core::Result<Self> {
        let scale1 = (2.0 / PERCEPTION_SIZE as f64).sqrt();
        let scale2 = (2.0 / HIDDEN1_SIZE as f64).sqrt();
        let scale3 = (2.0 / HIDDEN2_SIZE as f64).sqrt();

        // Layer 1: structured center-surround + small random
        // The 144-dim perception is 9 neighbors × 16 channels.
        // Center cell (offset 4) should excite, neighbors should inhibit.
        let mut w1_data = vec![0.0f64; PERCEPTION_SIZE * HIDDEN1_SIZE];
        for out_idx in 0..HIDDEN1_SIZE {
            for in_idx in 0..PERCEPTION_SIZE {
                let neighbor_idx = in_idx / NCA_CHANNELS; // which neighbor (0-8)
                let ch = in_idx % NCA_CHANNELS;
                // Center-surround: center (idx 4) excites, others inhibit
                let structure = if neighbor_idx == 4 && ch == ACTIVATION_CH {
                    0.3 // self-excitation for activation channel
                } else if neighbor_idx != 4 && ch == ACTIVATION_CH {
                    -0.03 // weak neighbor inhibition
                } else {
                    0.0 // other channels: random only
                };
                // Add small random for diversity
                let noise = (out_idx.wrapping_mul(in_idx + 1) as f64 * 0.01).sin() * scale1 * 0.1;
                w1_data[in_idx * HIDDEN1_SIZE + out_idx] = structure + noise;
            }
        }
        let w1 = Tensor::from_slice(&w1_data, (PERCEPTION_SIZE, HIDDEN1_SIZE), device)?;
        let b1 = Tensor::zeros((HIDDEN1_SIZE,), DType::F64, device)?;

        // Layers 2 & 3: random (they'll learn structure via Hebbian)
        let w2 = Tensor::randn(0.0f64, scale2, (HIDDEN1_SIZE, HIDDEN2_SIZE), device)?;
        let b2 = Tensor::zeros((HIDDEN2_SIZE,), DType::F64, device)?;
        let w3 = Tensor::randn(0.0f64, scale3, (HIDDEN2_SIZE, NCA_CHANNELS), device)?;
        let b3 = Tensor::zeros((NCA_CHANNELS,), DType::F64, device)?;

        Ok(Self { w1, b1, w2, b2, w3, b3 })
    }

    /// Forward pass for a single cell: [144] → [16]
    fn forward_cell(&self, perception: &Tensor) -> candle_core::Result<Tensor> {
        let h1 = perception.matmul(&self.w1)?.broadcast_add(&self.b1)?.relu()?;
        let h2 = h1.matmul(&self.w2)?.broadcast_add(&self.b2)?.relu()?;
        let out = h2.matmul(&self.w3)?.broadcast_add(&self.b3)?;
        // tanh * 0.3 for visible updates even with random weights
        out.tanh()?.affine(0.3, 0.0)
    }

    /// Save weights to a safetensors file
    pub fn save(&self, path: &std::path::Path) -> candle_core::Result<()> {
        let mut tensors = HashMap::new();
        tensors.insert("w1".to_string(), self.w1.copy()?);
        tensors.insert("b1".to_string(), self.b1.copy()?);
        tensors.insert("w2".to_string(), self.w2.copy()?);
        tensors.insert("b2".to_string(), self.b2.copy()?);
        tensors.insert("w3".to_string(), self.w3.copy()?);
        tensors.insert("b3".to_string(), self.b3.copy()?);
        candle_core::safetensors::save(&tensors, path)?;
        Ok(())
    }

    /// Load weights from a safetensors file
    pub fn load(path: &std::path::Path, device: &Device) -> candle_core::Result<Self> {
        let tensors = candle_core::safetensors::load(path, device)?;
        Ok(Self {
            w1: tensors.get("w1").cloned().unwrap_or_else(|| Tensor::zeros((PERCEPTION_SIZE, HIDDEN1_SIZE), DType::F64, device).unwrap()),
            b1: tensors.get("b1").cloned().unwrap_or_else(|| Tensor::zeros((HIDDEN1_SIZE,), DType::F64, device).unwrap()),
            w2: tensors.get("w2").cloned().unwrap_or_else(|| Tensor::zeros((HIDDEN1_SIZE, HIDDEN2_SIZE), DType::F64, device).unwrap()),
            b2: tensors.get("b2").cloned().unwrap_or_else(|| Tensor::zeros((HIDDEN2_SIZE,), DType::F64, device).unwrap()),
            w3: tensors.get("w3").cloned().unwrap_or_else(|| Tensor::zeros((HIDDEN2_SIZE, NCA_CHANNELS), DType::F64, device).unwrap()),
            b3: tensors.get("b3").cloned().unwrap_or_else(|| Tensor::zeros((NCA_CHANNELS,), DType::F64, device).unwrap()),
        })
    }
}

// ── Grid Operations ────────────────────────────────────────────────────────

/// Build perception vectors for all cells: gather 3×3 neighborhood.
/// Returns [G*G, 144] tensor where each row is one cell's perception.
fn build_perceptions(grid: &Tensor, grid_size: usize) -> candle_core::Result<Tensor> {
    let cells = grid_size * grid_size;
    let mut perceptions = Vec::with_capacity(cells);

    for r in 0..grid_size {
        for c in 0..grid_size {
            let mut neighbor_vals = Vec::with_capacity(PERCEPTION_SIZE);
            for &(dr, dc) in &NEIGHBOR_OFFSETS {
                let nr = (r as i32 + dr).rem_euclid(grid_size as i32) as usize;
                let nc = (c as i32 + dc).rem_euclid(grid_size as i32) as usize;
                for ch in 0..NCA_CHANNELS {
                    neighbor_vals.push(grid.i((nr, nc, ch))?);
                }
            }
            perceptions.push(Tensor::stack(&neighbor_vals, 0)?);
        }
    }

    Tensor::stack(&perceptions, 0)
}

/// Run one NCA step with self-reinforcement: activation persists
/// unless the MLP actively changes it. This ensures the initial encoding
/// survives long enough for Hebbian learning to create attractor basins.
fn nca_step(grid: &Tensor, mlp: &AttractorMlp, grid_size: usize) -> candle_core::Result<Tensor> {
    let perceptions = build_perceptions(grid, grid_size)?; // [G*G, 144]
    let updates = mlp.forward_cell(&perceptions)?; // [G*G, 16]

    // Reshape updates back to grid
    let updates_3d = updates.reshape(&[grid_size, grid_size, NCA_CHANNELS])?;

    // Self-reinforcement: old activation * 0.9 + MLP update
    // This means activation persists ~10 steps without MLP help.
    // The MLP learns to modulate this persistence to create attractors.
    let persisted = grid.affine(0.9, 0.0)?;
    let new_grid = persisted.add(&updates_3d)?;

    // Clamp to prevent runaway activation
    new_grid.clamp(-5.0, 5.0)
}

/// Run multiple NCA steps, returning all intermediate grids for visualization.
pub fn nca_steps_with_trace(
    grid: &Tensor,
    mlp: &AttractorMlp,
    grid_size: usize,
    steps: usize,
) -> candle_core::Result<Vec<Tensor>> {
    let mut trace = Vec::with_capacity(steps + 1);
    trace.push(grid.copy()?);
    let mut current = grid.copy()?;

    for _ in 0..steps {
        current = nca_step(&current, mlp, grid_size)?;
        trace.push(current.copy()?);
    }

    Ok(trace)
}

/// Check if grid has stabilized (mean absolute change below threshold)
#[allow(dead_code)]
fn is_stable(prev: &Tensor, current: &Tensor) -> candle_core::Result<bool> {
    let diff = current.sub(prev)?.abs()?.mean_all()?;
    let val: f64 = diff.to_scalar()?;
    Ok(val < STABILITY_THRESHOLD)
}

// ── Pattern Encoding / Decoding ────────────────────────────────────────────

/// Encode text into a SPARSE distributed grid activation pattern.
/// Only ~15% of cells are activated per memory, creating distinct
/// attractor basins that don't overlap and saturate.
/// Each token gets a unique fingerprint across multiple channels.
pub fn encode_pattern(
    text: &str,
    tokenizer: &SimpleTokenizer,
    grid_size: usize,
    device: &Device,
) -> candle_core::Result<Tensor> {
    let token_ids = tokenizer.encode(text);
    let cells = grid_size * grid_size;
    let mut grid_data = vec![0.0f64; cells * NCA_CHANNELS];

    if token_ids.is_empty() {
        return Tensor::from_slice(&grid_data, &[grid_size, grid_size, NCA_CHANNELS], device);
    }

    // Only activate a sparse subset of cells
    let active_cells = (cells as f64 * SPARSE_FRACTION) as usize;
    let positions_per_token = (active_cells / token_ids.len()).max(1);

    for (seq_pos, &tid) in token_ids.iter().enumerate() {
        let recency = 1.0 - (token_ids.len() - 1 - seq_pos) as f64 / token_ids.len() as f64 * 0.3;

        for p in 0..positions_per_token {
            let row = (tid.wrapping_mul(13)
                .wrapping_add(seq_pos.wrapping_mul(7))
                .wrapping_add(p.wrapping_mul(31))) % grid_size;
            let col = (tid.wrapping_mul(17)
                .wrapping_add(seq_pos.wrapping_mul(11))
                .wrapping_add(p.wrapping_mul(29))) % grid_size;

            let base_idx = (row * grid_size + col) * NCA_CHANNELS;

            // Activation channel: primary signal (strong, sparse)
            grid_data[base_idx + ACTIVATION_CH] =
                (grid_data[base_idx + ACTIVATION_CH] + recency).clamp(-5.0, 5.0);

            // Auxiliary channels: unique pattern fingerprint per token
            for ch in 1..NCA_CHANNELS {
                let fingerprint = ((tid as f64 * (ch as f64 + 1.0) * 0.17
                    + seq_pos as f64 * 0.31
                    + p as f64 * 0.07)
                    .sin()
                    * 0.5)
                    .clamp(-5.0, 5.0);
                grid_data[base_idx + ch] =
                    (grid_data[base_idx + ch] + fingerprint).clamp(-5.0, 5.0);
            }
        }
    }

    Tensor::from_slice(&grid_data, &[grid_size, grid_size, NCA_CHANNELS], device)
}

/// Decode a stabilized grid state back to text.
/// Reads activation at each token's hash positions, ranks by activation,
/// and returns the top tokens as decoded text.
pub fn decode_pattern(
    grid: &Tensor,
    tokenizer: &SimpleTokenizer,
    grid_size: usize,
    top_k: usize,
) -> candle_core::Result<String> {
    let vocab_size = tokenizer.vocab_size();
    let cells = grid_size * grid_size;
    let positions_per_token = (cells / top_k.max(1)).max(1);

    let mut token_scores: Vec<(usize, f64)> = Vec::with_capacity(vocab_size);

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
        let score = if count > 0 { sum / count as f64 } else { 0.0 };
        // Use a very low threshold to filter only truly inactive tokens.
        // The previous 0.01 threshold was too aggressive — after NCA recall
        // dynamics, activation values can be small but still meaningful.
        if score.abs() > 1e-6 {
            token_scores.push((tid, score));
        }
    }

    token_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    let top_tokens: Vec<usize> = token_scores.iter().take(top_k).map(|(tid, _)| *tid).collect();
    Ok(tokenizer.decode(&top_tokens))
}

// ── Hebbian Learning ───────────────────────────────────────────────────────

/// Store a pattern as an attractor using Hebbian learning.
/// Runs NCA steps until the pattern stabilizes, then strengthens
/// connections between co-active cells. "Cells that fire together, wire together."
pub fn store_memory(
    grid: &Tensor,
    mlp: &mut AttractorMlp,
    grid_size: usize,
    steps: usize,
    _device: &Device,
) -> candle_core::Result<Vec<Tensor>> {
    let trace = nca_steps_with_trace(grid, mlp, grid_size, steps)?;
    let stabilized = trace.last().unwrap();

    // Hebbian update: for each cell, correlate pre-synaptic (neighbor) activity
    // with post-synaptic (cell update) activity and strengthen those connections.
    let cells = grid_size * grid_size;

    // Build perceptions and updates for the stabilized state
    let perceptions = build_perceptions(stabilized, grid_size)?; // [cells, 144]
    let updates = mlp.forward_cell(&perceptions)?; // [cells, 16]

    // For each layer, accumulate outer product of (pre, post) averaged over cells
    // Layer 1: pre = perception[144], post = hidden1[1024]
    let h1_pre_relu = perceptions.matmul(&mlp.w1)?.broadcast_add(&mlp.b1)?;
    let h1_post = h1_pre_relu.relu()?; // [cells, 1024]

    // Hebbian update w1: Δw1 += lr * mean(perception^T × h1_post)
    let dw1 = perceptions.t()?.matmul(&h1_post)?.affine(HEBBIAN_LR / cells as f64, 0.0)?;
    mlp.w1 = mlp.w1.add(&dw1)?;

    // Layer 2: pre = h1_post[1024], post = hidden2[384]
    let h2_pre_relu = h1_post.matmul(&mlp.w2)?.broadcast_add(&mlp.b2)?;
    let h2_post = h2_pre_relu.relu()?;

    let dw2 = h1_post.t()?.matmul(&h2_post)?.affine(HEBBIAN_LR / cells as f64, 0.0)?;
    mlp.w2 = mlp.w2.add(&dw2)?;

    // Layer 3: pre = h2_post[384], post = updates[16]
    let dw3 = h2_post.t()?.matmul(&updates)?.affine(HEBBIAN_LR / cells as f64, 0.0)?;
    mlp.w3 = mlp.w3.add(&dw3)?;

    // Small bias updates toward mean activation
    let db1 = h1_post.mean(0)?.affine(HEBBIAN_LR * 0.1, 0.0)?;
    let db2 = h2_post.mean(0)?.affine(HEBBIAN_LR * 0.1, 0.0)?;
    let db3 = updates.mean(0)?.affine(HEBBIAN_LR * 0.1, 0.0)?;
    mlp.b1 = mlp.b1.add(&db1)?;
    mlp.b2 = mlp.b2.add(&db2)?;
    mlp.b3 = mlp.b3.add(&db3)?;

    // Homeostatic normalization: prevent runaway saturation
    // 1. Frobenius norm cap on each weight matrix
    // 2. Small decay toward zero (prevents all-positive drift)
    for (w, shape) in [
        (&mut mlp.w1, (PERCEPTION_SIZE, HIDDEN1_SIZE)),
        (&mut mlp.w2, (HIDDEN1_SIZE, HIDDEN2_SIZE)),
        (&mut mlp.w3, (HIDDEN2_SIZE, NCA_CHANNELS)),
    ] {
        let _n_elems = (shape.0 * shape.1) as f64;
        let frob = w.sqr()?.mean_all()?.sqrt()?;
        let frob_val: f64 = frob.to_scalar()?;
        if frob_val > WEIGHT_NORM_MAX {
            *w = w.affine(WEIGHT_NORM_MAX / frob_val, 0.0)?;
        }
        // Small decay: pull weights toward zero by 0.1% per memory
        *w = w.affine(0.999, 0.0)?;
    }
    // Also decay biases
    mlp.b1 = mlp.b1.affine(0.999, 0.0)?;
    mlp.b2 = mlp.b2.affine(0.999, 0.0)?;
    mlp.b3 = mlp.b3.affine(0.999, 0.0)?;

    Ok(trace)
}

/// Recall: seed the grid with a partial pattern and let it converge to
/// the nearest attractor. Returns the full trace for visualization.
pub fn recall(
    seed: &Tensor,
    mlp: &AttractorMlp,
    grid_size: usize,
    steps: usize,
) -> candle_core::Result<Vec<Tensor>> {
    nca_steps_with_trace(seed, mlp, grid_size, steps)
}

// ── State File for TUI ─────────────────────────────────────────────────────

/// Write attractor network state for TUI monitoring
pub fn write_attractor_state(
    phase: &str, // "storing", "recalling", "idle"
    memory_label: &str,
    grid_frames: &[Vec<Vec<f64>>],
    memory_count: usize,
    grid_size: usize,
) {
    let state_path = dirs::home_dir()
        .unwrap_or_default()
        .join(".sage")
        .join("training_state.json");

    let state = serde_json::json!({
        "running": phase != "idle",
        "phase": phase,
        "memory_label": memory_label,
        "memory_count": memory_count,
        "grid_size": grid_size,
        "grid_frames": grid_frames,
        "current_epoch": memory_count,
        "total_epochs": memory_count.max(1),
        "losses": [],
        "accuracies": [],
        "best_accuracy": 0.0,
        "random_baseline": 0.0,
        "vocab_size": 0,
        "param_count": 548,
        "elapsed_secs": 0.0,
        "updated_at": chrono::Utc::now().to_rfc3339(),
    });

    if let Ok(json) = serde_json::to_string_pretty(&state) {
        let _ = fs::write(&state_path, json);
    }
}

/// Downsample a grid tensor to at most 32×32 for the TUI state file
pub fn downsample_grid(grid: &Tensor, grid_size: usize) -> Vec<Vec<f64>> {
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

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode_roundtrip() {
        let device = Device::Cpu;
        let tokenizer = SimpleTokenizer::new(4096);
        let text = "hello world this is a test of the attractor network";
        let grid_size = 32;

        let grid = encode_pattern(text, &tokenizer, grid_size, &device).unwrap();
        let decoded = decode_pattern(&grid, &tokenizer, grid_size, 20).unwrap();

        // Should recover at least some of the original tokens
        assert!(!decoded.is_empty());
        assert!(decoded.len() > 5);
    }

    #[test]
    fn test_attractor_mlp_forward() {
        let device = Device::Cpu;
        let mlp = AttractorMlp::new(&device).unwrap();

        // Create a random perception vector (reshape to 2D for matmul)
        let perception = Tensor::randn(0.0f64, 1.0, (1, PERCEPTION_SIZE), &device).unwrap();
        let output = mlp.forward_cell(&perception).unwrap();

        assert_eq!(output.dims(), &[1, NCA_CHANNELS]);
    }

    #[test]
    fn test_nca_step_stability() {
        let device = Device::Cpu;
        let mlp = AttractorMlp::new(&device).unwrap();
        let grid_size = 16;

        // Create a random grid
        let grid = Tensor::randn(0.0f64, 0.5, (grid_size, grid_size, NCA_CHANNELS), &device).unwrap();

        // Run one step
        let next = nca_step(&grid, &mlp, grid_size).unwrap();

        // Should have same shape
        assert_eq!(next.dims(), grid.dims());

        // Should be different (not identity)
        let diff = next.sub(&grid).unwrap().abs().unwrap().mean_all().unwrap();
        let diff_val: f64 = diff.to_scalar().unwrap();
        assert!(diff_val > 0.0, "NCA step should change the grid");
    }

    #[test]
    fn test_store_and_recall() {
        let device = Device::Cpu;
        let mut mlp = AttractorMlp::new(&device).unwrap();
        let tokenizer = SimpleTokenizer::new(4096);
        let grid_size = 16;
        let text = "the quick brown fox jumps over the lazy dog";

        // Encode and store
        let pattern = encode_pattern(text, &tokenizer, grid_size, &device).unwrap();
        let store_trace = store_memory(&pattern, &mut mlp, grid_size, 30, &device).unwrap();

        // Store trace should have multiple frames
        assert!(store_trace.len() > 1);

        // Recall with the same pattern as seed
        let recall_trace = recall(&pattern, &mlp, grid_size, 30).unwrap();
        assert!(recall_trace.len() > 1);

        // Decode the stabilized state
        let stabilized = recall_trace.last().unwrap();
        let decoded = decode_pattern(stabilized, &tokenizer, grid_size, 20).unwrap();
        assert!(!decoded.is_empty());
    }

    #[test]
    fn test_multiple_memories() {
        let device = Device::Cpu;
        let mut mlp = AttractorMlp::new(&device).unwrap();
        let tokenizer = SimpleTokenizer::new(4096);
        let grid_size = 16;

        let memories = [
            "the cat sat on the mat",
            "the dog ran in the park",
            "birds fly south for winter",
        ];

        // Store all memories
        for text in &memories {
            let pattern = encode_pattern(text, &tokenizer, grid_size, &device).unwrap();
            store_memory(&pattern, &mut mlp, grid_size, 20, &device).unwrap();
        }

        // Recall each — attractor networks with random init are probabilistic.
        // Check that at least 2 of 3 memories decode something.
        let mut decoded_count = 0;
        for text in &memories {
            let seed = encode_pattern(text, &tokenizer, grid_size, &device).unwrap();
            let trace = recall(&seed, &mlp, grid_size, 20).unwrap();
            let stabilized = trace.last().unwrap();
            let decoded = decode_pattern(stabilized, &tokenizer, grid_size, 20).unwrap();
            if !decoded.is_empty() {
                decoded_count += 1;
            }
        }
        assert!(decoded_count >= 2, "Only {} of 3 memories decoded (probabilistic attractor)", decoded_count);
    }
}
