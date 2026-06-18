//! NCA Brain Processor — Self-organizing language map on the 256×256 grid.
//!
//! The brain IS the NCA grid. Text is encoded at hashed positions, then
//! local dynamics propagate and transform the information. The grid
//! self-organizes because the dynamics are dissipative — activation
//! decays unless reinforced by new input or by neighbors.
//!
//! Architecture:
//!   - 54 processing channels per cell (memory + knowledge)
//!   - Local update rule: diffusion + decay + nonlinear competition
//!   - Energy-dissipative: total activation decreases unless reinforced
//!   - Structured: similar text hashes to nearby cells, creating clusters
//!
//! The update rule has two modes:
//!   1. "Passive" (no new input): diffusion + decay — patterns settle
//!   2. "Active" (new text encoded): injected activation propagates outward
//!
//! Over time, the grid develops regions that specialize in different
//! language patterns. Querying a region activates the text stored there.

use crate::grid::{
    Grid, KNOWLEDGE_ACTIVATION, KNOWLEDGE_CHANNELS_START, KNOWLEDGE_CONFIDENCE,
    MEMORY_ATTENTION, MEMORY_CHANNELS_START, MEMORY_GATE, MEMORY_RECENCY, MEMORY_VALUE,
    NUM_KNOWLEDGE_CHANNELS, NUM_MEMORY_CHANNELS,
};
use std::path::Path;

// ── Processing channel layout ──────────────────────────────────────────────
const PROC_CHANNELS: usize = NUM_MEMORY_CHANNELS + NUM_KNOWLEDGE_CHANNELS; // 4 + 50 = 54

fn proc_channel_indices() -> Vec<usize> {
    (MEMORY_CHANNELS_START..MEMORY_CHANNELS_START + NUM_MEMORY_CHANNELS)
        .chain(KNOWLEDGE_CHANNELS_START..KNOWLEDGE_CHANNELS_START + NUM_KNOWLEDGE_CHANNELS)
        .collect()
}

// ── NCA Dynamics Parameters ────────────────────────────────────────────────

/// Parameters controlling the NCA brain dynamics.
/// These are the "physics" of the grid — they determine how activation
/// spreads, decays, and self-organizes.
///
/// Key principle: the system must be DISSIPATIVE.
/// Total energy must decrease unless reinforced by new input.
/// This prevents saturation and creates structure.
#[derive(Clone, Debug)]
pub struct BrainDynamics {
    /// Self-decay rate: each step, activation *= (1 - self_decay)
    /// Higher = patterns fade faster (more forgetting, less saturation)
    pub self_decay: f64,        // 0.02 — slow forgetting

    /// Neighbor diffusion: fraction of activation that spreads to neighbors
    /// Each neighbor gets diffusion * activation / 8 (8 neighbors in 3×3)
    /// Higher = information spreads faster but fades sooner
    pub diffusion: f64,         // 0.15 — moderate spreading

    /// Nonlinear competition: cells with high activation suppress nearby
    /// low-activation cells (winner-take-all). Creates sharp clusters.
    pub competition: f64,       // 0.05 — mild competition

    /// Noise injection: random perturbation per step (for exploration)
    pub noise: f64,              // 0.001 — tiny noise for exploration

    /// Activation floor: cells below this are set to zero each step
    pub floor: f64,              // 0.005 — clean up noise

    /// Embedding diffusion: how much embedding slots spread to neighbors
    /// (much slower than activation — embeddings should be stable)
    pub embedding_diffusion: f64, // 0.02 — very slow embedding spread

    /// Confidence boost: how much confidence increases for stable cells
    pub confidence_boost: f64,   // 0.01 — slow confidence building
}

impl Default for BrainDynamics {
        fn default() -> Self {
        Self {
            self_decay: 0.0006,      // Very slow decay — clusters survive ~1000 chunks
            diffusion: 0.03,         // Low spreading — keeps clusters tight
            competition: 0.02,      // Mild competition — sharpens boundaries
            noise: 0.0003,           // Tiny noise for exploration
            floor: 0.001,            // Clean up sub-threshold noise
            embedding_diffusion: 0.005, // Very slow embedding spread
            confidence_boost: 0.005, // Slow confidence building
        }
    }
}

/// Apply one NCA update step using local dynamics.
///
/// For each active cell:
///   1. Self-decay: activation *= (1 - self_decay)
///   2. Diffusion: spread activation to 8 neighbors (each gets diffusion/8)
///   3. Competition: high-activation cells suppress low-activation neighbors
///   4. Noise: add tiny random perturbation
///   5. Floor: remove sub-threshold values
///
/// Energy conservation check:
///   self_decay + 8 * (diffusion/8) = self_decay + diffusion
///   With self_decay=0.02, diffusion=0.15: total loss = 0.17 per step
///   This means ~17% of activation "evaporates" per step unless reinforced.
///   Reinforcement comes from: (a) new text encoding, (b) Hebbian consolidation.
///
/// This creates a dissipative system where:
///   - Fresh input creates bright clusters
///   - Old patterns slowly fade
///   - Competition sharpens cluster boundaries
///   - The grid develops stable structure from recurring patterns
pub fn nca_brain_step(grid: &mut Grid, dynamics: &BrainDynamics) {
    let w = grid.width;
    let h = grid.height;

    // Snapshot current activation for double-buffering
    let mut next_activation = vec![vec![0.0f64; w]; h];
    let mut next_embeddings: Vec<(usize, usize, [f64; NUM_KNOWLEDGE_CHANNELS - 2])> = Vec::new();

    // First pass: compute diffusion and decay
    for y in 0..h {
        for x in 0..w {
            let act = grid.cells[y][x][KNOWLEDGE_ACTIVATION];
            if act < 0.001 {
                continue;
            }

            // Self-decay
            let decayed = act * (1.0 - dynamics.self_decay);

            // Keep (1 - diffusion) fraction locally
            let kept = decayed * (1.0 - dynamics.diffusion);
            next_activation[y][x] += kept;

            // Spread diffusion/8 to each neighbor
            let spread = decayed * dynamics.diffusion / 8.0;
            for dy in -1i32..=1 {
                for dx in -1i32..=1 {
                    if dx == 0 && dy == 0 { continue; }
                    let ny = ((y as i32 + dy).rem_euclid(h as i32)) as usize;
                    let nx = ((x as i32 + dx).rem_euclid(w as i32)) as usize;
                    next_activation[ny][nx] += spread;
                }
            }
        }
    }

    // Second pass: competition, noise, floor, and apply
    for y in 0..h {
        for x in 0..w {
            let mut new_act = next_activation[y][x];

            // Competition: if a neighbor has much higher activation, suppress this cell
            if new_act > 0.01 {
                let mut max_neighbor = 0.0;
                for dy in -1i32..=1 {
                    for dx in -1i32..=1 {
                        if dx == 0 && dy == 0 { continue; }
                        let ny = ((y as i32 + dy).rem_euclid(h as i32)) as usize;
                        let nx = ((x as i32 + dx).rem_euclid(w as i32)) as usize;
                        let n_act = grid.cells[ny][nx][KNOWLEDGE_ACTIVATION];
                        if n_act > max_neighbor { max_neighbor = n_act; }
                    }
                }
                // If this cell is much weaker than a neighbor, apply suppression
                if max_neighbor > new_act * 3.0 {
                    new_act *= (1.0 - dynamics.competition);
                }
            }

            // Noise injection (tiny)
            if new_act > 0.01 {
                use rand::Rng;
                let mut rng = rand::thread_rng();
                new_act += rng.gen_range(-dynamics.noise..dynamics.noise);
            }

            // Floor: clean up near-zero values
            if new_act < dynamics.floor {
                new_act = 0.0;
            }

            // Clamp
            new_act = new_act.clamp(0.0, 1.0);

            // Apply: blend with current (smooth update to avoid oscillation)
            let current = grid.cells[y][x][KNOWLEDGE_ACTIVATION];
            grid.cells[y][x][KNOWLEDGE_ACTIVATION] = new_act;

            // Embedding diffusion: slowly spread embedding values to neighbors
            if new_act > 0.05 {
                let mut emb_vals = [0.0f64; NUM_KNOWLEDGE_CHANNELS - 2];
                for (i, slot) in emb_vals.iter_mut().enumerate() {
                    *slot = grid.cells[y][x][KNOWLEDGE_CHANNELS_START + i];
                }
                next_embeddings.push((y, x, emb_vals));
            }
        }
    }

    // Apply embedding diffusion (very slow — keeps embeddings stable)
    let emb_spread = dynamics.embedding_diffusion;
    for &(y, x, emb) in &next_embeddings {
        for dy in -1i32..=1 {
            for dx in -1i32..=1 {
                if dx == 0 && dy == 0 { continue; }
                let ny = ((y as i32 + dy).rem_euclid(h as i32)) as usize;
                let nx = ((x as i32 + dx).rem_euclid(w as i32)) as usize;
                // Only spread to cells that already have some activation
                if grid.cells[ny][nx][KNOWLEDGE_ACTIVATION] > 0.01 {
                    for (i, &val) in emb.iter().enumerate() {
                        let ch = KNOWLEDGE_CHANNELS_START + i;
                        let current = grid.cells[ny][nx][ch];
                        // Blend: 98% current + 2% neighbor
                        grid.cells[ny][nx][ch] = current * (1.0 - emb_spread) + val * emb_spread;
                    }
                }
            }
        }
    }

    // Confidence dynamics: stable cells slowly gain confidence
    for y in 0..h {
        for x in 0..w {
            let act = grid.cells[y][x][KNOWLEDGE_ACTIVATION];
            let conf = grid.cells[y][x][KNOWLEDGE_CONFIDENCE];
            if act > 0.1 {
                grid.cells[y][x][KNOWLEDGE_CONFIDENCE] =
                    (conf + dynamics.confidence_boost).clamp(0.0, 1.0);
            } else if act < 0.01 && conf > 0.0 {
                // Decay confidence for inactive cells
                grid.cells[y][x][KNOWLEDGE_CONFIDENCE] =
                    (conf - dynamics.self_decay).max(0.0);
            }
        }
    }
}

/// Run N NCA update steps on the brain grid.
pub fn process_brain(grid: &mut Grid, dynamics: &BrainDynamics, steps: usize) {
    for _ in 0..steps {
        nca_brain_step(grid, dynamics);
    }
}

/// Process text: encode it into the brain, then run NCA steps to propagate.
///
/// This is the core learning operation:
/// 1. Encode text features into the grid at the hashed position
/// 2. Run NCA dynamics steps so the activation propagates and decays
/// 3. The grid self-organizes as repeated patterns create stable clusters
pub fn process_text(
    grid: &mut Grid,
    features: &crate::distributed_knowledge::encoder::FeatureVector,
    dynamics: &BrainDynamics,
    confidence: f64,
    config: &crate::distributed_knowledge::encoder::EncoderConfig,
    nca_steps: usize,
) -> (usize, usize) {
    // Step 1: Encode text into the grid
    let pos = crate::distributed_knowledge::encoder::write_knowledge(
        grid, features, confidence, 0.0, config,
    );

    // Step 2: Run NCA dynamics to propagate and decay
    process_brain(grid, dynamics, nca_steps);

    pos
}

// ── Weight-based NCA (for future trained update rule) ──────────────────────
// This section is kept for when we want to train the NCA update rule
// with a prediction objective. For now, the dynamics-based approach above
// is simpler and produces structured results without training.

pub struct BrainNcaWeights {
    pub w1: Vec<Vec<f64>>,
    pub b1: Vec<f64>,
    pub w2: Vec<Vec<f64>>,
    pub b2: Vec<f64>,
    pub w3: Vec<Vec<f64>>,
    pub b3: Vec<f64>,
}

impl BrainNcaWeights {
    pub fn random() -> Self {
        // Smaller network: 486 → 128 → 32 → 54
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let scale1 = (2.0 / 486.0_f64).sqrt();
        let scale2 = (2.0 / 128.0_f64).sqrt();
        let scale3 = (2.0 / 32.0_f64).sqrt();
        Self {
            w1: (0..128).map(|_| (0..486).map(|_| rng.gen_range(-scale1..scale1)).collect()).collect(),
            b1: vec![0.0; 128],
            w2: (0..32).map(|_| (0..128).map(|_| rng.gen_range(-scale2..scale2)).collect()).collect(),
            b2: vec![0.0; 32],
            w3: (0..54).map(|_| (0..32).map(|_| rng.gen_range(-scale3..scale3)).collect()).collect(),
            b3: vec![0.0; 54],
        }
    }

    pub fn param_count(&self) -> usize {
        128 * 486 + 128 + 32 * 128 + 32 + 54 * 32 + 54
    }

    pub fn save(&self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let data: Vec<f64> = self.to_vec();
        let bytes: Vec<u8> = data.iter().flat_map(|f| f.to_le_bytes()).collect();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, &bytes)?;
        Ok(())
    }

    pub fn load(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let bytes = std::fs::read(path)?;
        let data: Vec<f64> = bytes.chunks_exact(8)
            .map(|c| f64::from_le_bytes(c.try_into().unwrap()))
            .collect();
        Ok(Self::from_vec(&data))
    }

    pub fn to_vec(&self) -> Vec<f64> {
        let mut v = Vec::with_capacity(self.param_count());
        for row in &self.w1 { v.extend(row); }
        v.extend(&self.b1);
        for row in &self.w2 { v.extend(row); }
        v.extend(&self.b2);
        for row in &self.w3 { v.extend(row); }
        v.extend(&self.b3);
        v
    }

    pub fn from_vec(params: &[f64]) -> Self {
        let mut idx = 0;
        let mut w1 = Vec::with_capacity(128);
        for _ in 0..128 {
            w1.push(params[idx..idx+486].to_vec());
            idx += 486;
        }
        let b1 = params[idx..idx+128].to_vec();
        idx += 128;
        let mut w2 = Vec::with_capacity(32);
        for _ in 0..32 {
            w2.push(params[idx..idx+128].to_vec());
            idx += 128;
        }
        let b2 = params[idx..idx+32].to_vec();
        idx += 32;
        let mut w3 = Vec::with_capacity(54);
        for _ in 0..54 {
            w3.push(params[idx..idx+32].to_vec());
            idx += 32;
        }
        let b3 = params[idx..idx+54].to_vec();
        Self { w1, b1, w2, b2, w3, b3 }
    }

    pub fn default_path() -> std::path::PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        std::path::PathBuf::from(format!("{}/.sage/brain_nca_weights.bin", home))
    }
}

/// NCA step using trained MLP weights (operates on knowledge channels only).
/// This is the learned update rule — the weights were trained via ES to
/// transform prefix embeddings toward target word embeddings.
///
/// Only processes active cells + their neighbors. Uses 50 knowledge channels
/// (not all 54) for speed — the 4 memory channels are handled by dynamics.
pub fn nca_brain_step_weighted(grid: &mut Grid, weights: &BrainNcaWeights) {
    use crate::grid::{KNOWLEDGE_ACTIVATION, KNOWLEDGE_CHANNELS_START};
    let w = grid.width;
    let h = grid.height;

    // Collect active cells with their activation levels, then cap to top-N
    // to prevent OOM on dense grids. The MLP is only useful on the most
    // active cells anyway — dim cells contribute little signal.
    const MAX_WEIGHTED_CELLS: usize = 200;

    let mut active_cells: Vec<(usize, usize, f64)> = Vec::new();
    for y in 0..h {
        for x in 0..w {
            let act = grid.cells[y][x][KNOWLEDGE_ACTIVATION];
            if act > 0.01 {
                active_cells.push((y, x, act));
            }
        }
    }
    if active_cells.is_empty() { return; }

    // Sort by activation descending, take top MAX_WEIGHTED_CELLS
    active_cells.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
    active_cells.truncate(MAX_WEIGHTED_CELLS);

    // Build the set of cells to process (active cells + their neighbors)
    let mut process_set: Vec<(usize, usize)> = Vec::new();
    {
        let mut seen = vec![vec![false; w]; h];
        for &(y, x, _) in &active_cells {
            for dy in -1i32..=1 {
                for dx in -1i32..=1 {
                    let ny = ((y as i32 + dy).rem_euclid(h as i32)) as usize;
                    let nx = ((x as i32 + dx).rem_euclid(w as i32)) as usize;
                    if !seen[ny][nx] { seen[ny][nx] = true; process_set.push((ny, nx)); }
                }
            }
        }
    }

    if process_set.is_empty() { return; }

    const NCH: usize = 50;
    const PERC: usize = 9 * NCH;
    let mut deltas: Vec<(usize, usize, Vec<f64>)> = Vec::with_capacity(process_set.len());

    for (cy, cx) in &process_set {
        let y = *cy; let x = *cx;
        let mut input = vec![0.0f64; PERC];
        let mut idx = 0;
        for dy in -1i32..=1 {
            for dx in -1i32..=1 {
                let ny = ((y as i32 + dy).rem_euclid(h as i32)) as usize;
                let nx = ((x as i32 + dx).rem_euclid(w as i32)) as usize;
                for ch in 0..NCH { input[idx] = grid.cells[ny][nx][KNOWLEDGE_CHANNELS_START + ch]; idx += 1; }
            }
        }
        let mut h1 = vec![0.0f64; 128];
        for i in 0..128 { let mut sum = weights.b1[i]; let row = &weights.w1[i]; for j in 0..PERC { sum += row[j] * input[j]; } h1[i] = sum.max(0.0); }
        let mut h2 = vec![0.0f64; 32];
        for i in 0..32 { let mut sum = weights.b2[i]; let row = &weights.w2[i]; for j in 0..128 { sum += row[j] * h1[j]; } h2[i] = sum.max(0.0); }
        let mut out = vec![0.0f64; NCH];
        for i in 0..NCH { let mut sum = weights.b3[i]; let row = &weights.w3[i]; for j in 0..32 { sum += row[j] * h2[j]; } out[i] = sum.tanh() * 0.1; }
        deltas.push((y, x, out));
    }

    let decay = 0.005;
    for (y, x, out) in &deltas {
        for (i, &delta) in out.iter().enumerate() {
            let ch = KNOWLEDGE_CHANNELS_START + i;
            if ch == KNOWLEDGE_ACTIVATION {
                grid.cells[*y][*x][ch] = (grid.cells[*y][*x][ch] * (1.0 - decay) + delta).clamp(0.0, 1.0);
            } else {
                grid.cells[*y][*x][ch] = (grid.cells[*y][*x][ch] + delta).clamp(-5.0, 5.0);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::Grid;

    #[test]
    fn test_dynamics_decay_prevents_saturation() {
        let mut grid = Grid::new(32, 32);
        let dynamics = BrainDynamics::default();

        // Put activation in one cell
        grid.cells[16][16][KNOWLEDGE_ACTIVATION] = 1.0;

        // Run 100 steps with no new input
        for _ in 0..100 {
            nca_brain_step(&mut grid, &dynamics);
        }

        // Activation should have decayed significantly
        let final_act = grid.cells[16][16][KNOWLEDGE_ACTIVATION];
        assert!(
            final_act < 0.1,
            "Activation should decay without reinforcement, got {}",
            final_act
        );
    }

    #[test]
    fn test_dynamics_spread_to_neighbors() {
        let mut grid = Grid::new(32, 32);
        let dynamics = BrainDynamics::default();

        grid.cells[16][16][KNOWLEDGE_ACTIVATION] = 1.0;

        // One step should spread to neighbors
        nca_brain_step(&mut grid, &dynamics);

        let neighbor_act = grid.cells[15][16][KNOWLEDGE_ACTIVATION];
        assert!(
            neighbor_act > 0.001,
            "Activation should spread to neighbors, got {}",
            neighbor_act
        );
    }

    #[test]
    fn test_dynamics_energy_decreases() {
        let mut grid = Grid::new(32, 32);
        let dynamics = BrainDynamics::default();

        // Start with some activation
        grid.cells[16][16][KNOWLEDGE_ACTIVATION] = 1.0;
        grid.cells[10][10][KNOWLEDGE_ACTIVATION] = 0.8;

        let total_before: f64 = {
            let cells = &grid.cells;
            (0..32).flat_map(|y| (0..32).map(move |x| cells[y][x][KNOWLEDGE_ACTIVATION])).sum()
        };

        // Run 10 steps
        for _ in 0..10 {
            nca_brain_step(&mut grid, &dynamics);
        }

        let total_after: f64 = {
            let cells = &grid.cells;
            (0..32).flat_map(|y| (0..32).map(move |x| cells[y][x][KNOWLEDGE_ACTIVATION])).sum()
        };

        assert!(
            total_after < total_before,
            "Total energy should decrease (dissipative): before={}, after={}",
            total_before, total_after
        );
    }

    #[test]
    fn test_reinforcement_maintains_activation() {
        let mut grid = Grid::new(32, 32);
        let dynamics = BrainDynamics::default();

        // Repeatedly encode at the same position
        for _ in 0..50 {
            grid.cells[16][16][KNOWLEDGE_ACTIVATION] += 0.3;
            grid.cells[16][16][KNOWLEDGE_ACTIVATION] = grid.cells[16][16][KNOWLEDGE_ACTIVATION].min(1.0);
            nca_brain_step(&mut grid, &dynamics);
        }

        // With reinforcement, the cell should still have meaningful activation
        let act = grid.cells[16][16][KNOWLEDGE_ACTIVATION];
        assert!(
            act > 0.05,
            "Reinforced cell should maintain activation: {}",
            act
        );
    }

    #[test]
    fn test_competition_creates_structure() {
        let mut grid = Grid::new(32, 32);
        let dynamics = BrainDynamics {
            competition: 0.3, // Strong competition
            ..Default::default()
        };

        // Two nearby cells with different activation
        grid.cells[15][15][KNOWLEDGE_ACTIVATION] = 1.0; // Strong
        grid.cells[16][16][KNOWLEDGE_ACTIVATION] = 0.1; // Weak neighbor

        // Run several steps
        for _ in 0..20 {
            nca_brain_step(&mut grid, &dynamics);
        }

        // The strong cell should dominate, weak cell should be suppressed
        let strong = grid.cells[15][15][KNOWLEDGE_ACTIVATION];
        let weak = grid.cells[16][16][KNOWLEDGE_ACTIVATION];
        assert!(
            strong > weak,
            "Competition should favor stronger cell: strong={}, weak={}",
            strong, weak
        );
    }

    #[test]
    fn test_process_text_creates_cluster() {
        use crate::distributed_knowledge::encoder::{encode_text, EncoderConfig};
        let mut grid = Grid::new(64, 64);
        let dynamics = BrainDynamics::default();
        let config = EncoderConfig { ollama_url: None, ..Default::default() };

        // Process the same text multiple times (reinforcement)
        for _ in 0..10 {
            let features = encode_text("the cat sat on the mat", &config);
            process_text(&mut grid, &features, &dynamics, 0.8, &config, 3);
        }

        // There should be a cluster of activation
        let total_act: f64 = {
            let cells = &grid.cells;
            (0..64).flat_map(|y| (0..64).map(move |x| cells[y][x][KNOWLEDGE_ACTIVATION])).sum()
        };
        assert!(
            total_act > 0.5,
            "Multiple encodings should create activation cluster: total={}",
            total_act
        );

        // The grid should NOT be saturated
        let max_possible = 64.0 * 64.0;
        let fill_ratio = total_act / max_possible;
        assert!(
            fill_ratio < 0.1,
            "Grid should not be saturated: fill_ratio={}",
            fill_ratio
        );
    }
}