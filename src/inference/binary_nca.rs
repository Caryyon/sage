//! Discrete-State Neural Cellular Automaton (Binary NCA)
//!
//! Solves the fundamental problem of continuous NCA: homogenization/saturation.
//!
//! ## Architecture
//! - Each cell is binary: 0 or 1 (like Conway's Game of Life)
//! - Shared MLP: 3×3 neighborhood → hidden → flip probability
//! - Hebbian learning: memories are stored by reinforcing connections that
//!   produce the correct cell state for each neighborhood pattern
//!
//! ## Why This Works
//! - Binary states prevent gradual drift — no saturation to 5.0
//! - Discrete flips create visible, differentiated patterns
//! - Each memory is a unique stable attractor in binary space
//! - The MLP learns to "restore" any stored pattern from partial/noisy input
//!
//! ## Biological Plausibility
//! - Neurons fire or don't fire (binary)
//! - Hebbian: "cells that fire together, wire together"
//! - Local learning: each cell's rule depends only on its neighbors

use rand::Rng;
use std::f64;

// ── Constants ──────────────────────────────────────────────────────────────

/// Grid side length (32×32 = 1024 cells)
pub const GRID_SIZE: usize = 32;

/// MLP hidden layer size
const HIDDEN_SIZE: usize = 32;

/// Neighborhood size: 3×3 = 9 cells
const NEIGHBORHOOD_SIZE: usize = 9;

/// Temperature for stochastic updates (0 = deterministic, higher = more noise)
const TEMPERATURE: f64 = 0.3;

/// Learning rate for Hebbian updates
const LEARNING_RATE: f64 = 0.05;

/// Number of training epochs per memory
const TRAIN_EPOCHS: usize = 20;

// ── Binary NCA Grid ───────────────────────────────────────────────────────

/// A binary cell state: true = ON (1), false = OFF (0)
pub type CellState = bool;

/// A memory pattern: GRID_SIZE × GRID_SIZE of binary values
pub type Pattern = Vec<Vec<CellState>>;

/// Discrete-state NCA with shared MLP update rule
pub struct BinaryNCA {
    /// Current grid state
    pub grid: Vec<Vec<CellState>>,
    /// Grid size (square)
    pub size: usize,
    /// Stored memory patterns
    pub memories: Vec<Pattern>,
    /// Memory labels/names
    pub memory_names: Vec<String>,
    
    // MLP weights (shared across all cells)
    // Layer 1: NEIGHBORHOOD_SIZE → HIDDEN_SIZE
    w1: Vec<Vec<f64>>, // HIDDEN_SIZE × NEIGHBORHOOD_SIZE
    b1: Vec<f64>,      // HIDDEN_SIZE
    // Layer 2: HIDDEN_SIZE → 1 (output: probability of being ON)
    w2: Vec<f64>,      // HIDDEN_SIZE
    b2: f64,           // scalar bias
}

impl BinaryNCA {
    /// Create a new Binary NCA with random weights
    pub fn new(size: usize) -> Self {
        let mut rng = rand::thread_rng();
        
        // Xavier initialization for the weights
        let scale1 = (2.0 / NEIGHBORHOOD_SIZE as f64).sqrt();
        let scale2 = (2.0 / HIDDEN_SIZE as f64).sqrt();
        
        let w1: Vec<Vec<f64>> = (0..HIDDEN_SIZE)
            .map(|_| (0..NEIGHBORHOOD_SIZE)
                .map(|_| rng.gen_range(-scale1..scale1))
                .collect())
            .collect();
        let b1 = vec![0.0; HIDDEN_SIZE];
        let w2: Vec<f64> = (0..HIDDEN_SIZE)
            .map(|_| rng.gen_range(-scale2..scale2))
            .collect();
        let b2 = 0.0;
        
        // Initialize grid with random noise
        let grid: Vec<Vec<bool>> = (0..size)
            .map(|_| (0..size)
                .map(|_| rng.gen_bool(0.5))
                .collect())
            .collect();
        
        Self {
            grid,
            size,
            memories: Vec::new(),
            memory_names: Vec::new(),
            w1,
            b1,
            w2,
            b2,
        }
    }
    
    /// Get the 3×3 neighborhood of cell (r, c) as a flat vector of 9 f64 values
    /// Order: top-left, top, top-right, left, center, right, bottom-left, bottom, bottom-right
    fn get_neighborhood(&self, r: usize, c: usize) -> Vec<f64> {
        let mut neigh = Vec::with_capacity(NEIGHBORHOOD_SIZE);
        for dr in [-1i32, 0, 1] {
            for dc in [-1i32, 0, 1] {
                let nr = ((r as i32 + dr).rem_euclid(self.size as i32)) as usize;
                let nc = ((c as i32 + dc).rem_euclid(self.size as i32)) as usize;
                neigh.push(if self.grid[nr][nc] { 1.0 } else { 0.0 });
            }
        }
        neigh
    }
    
    /// Forward pass: neighborhood → probability of cell being ON
    /// Returns (probability, hidden_activations) for learning
    fn forward(&self, neighborhood: &[f64]) -> (f64, Vec<f64>) {
        // Layer 1: hidden = relu(W1·x + b1)
        let mut hidden = Vec::with_capacity(HIDDEN_SIZE);
        for h in 0..HIDDEN_SIZE {
            let mut sum = self.b1[h];
            for (i, &n) in neighborhood.iter().enumerate() {
                sum += self.w1[h][i] * n;
            }
            // ReLU with small leak for gradients
            hidden.push(if sum > 0.0 { sum } else { sum * 0.01 });
        }
        
        // Layer 2: y = sigmoid(W2·h + b2)
        let mut sum = self.b2;
        for (h, &h_val) in hidden.iter().enumerate() {
            sum += self.w2[h] * h_val;
        }
        let prob = sigmoid(sum);
        
        (prob, hidden)
    }
    
    /// Apply one NCA update step with stochastic flips
    pub fn step(&mut self) {
        let mut new_grid = self.grid.clone();
        let mut rng = rand::thread_rng();
        
        for r in 0..self.size {
            for c in 0..self.size {
                let neighborhood = self.get_neighborhood(r, c);
                let (prob, _) = self.forward(&neighborhood);
                
                // Stochastic flip: cell becomes ON with probability `prob`
                // Current state biases the flip (hysteresis)
                let current = self.grid[r][c];
                let flip_prob = if current {
                    // Currently ON: stay ON with prob = prob, flip OFF with prob = 1-prob
                    // Add temperature for exploration
                    let stay_prob = prob * (1.0 - TEMPERATURE) + 0.5 * TEMPERATURE;
                    1.0 - stay_prob // probability of flipping OFF
                } else {
                    // Currently OFF: flip ON with prob
                    let turn_on_prob = prob * (1.0 - TEMPERATURE) + 0.5 * TEMPERATURE;
                    turn_on_prob
                };
                
                new_grid[r][c] = if rng.gen::<f64>() < flip_prob {
                    !current // flip
                } else {
                    current  // stay
                };
            }
        }
        
        self.grid = new_grid;
    }
    
    /// Apply NCA step deterministically (no noise, for pattern recovery)
    pub fn step_deterministic(&mut self) {
        let mut new_grid = self.grid.clone();
        
        for r in 0..self.size {
            for c in 0..self.size {
                let neighborhood = self.get_neighborhood(r, c);
                let (prob, _) = self.forward(&neighborhood);
                
                // Threshold at 0.5
                new_grid[r][c] = prob > 0.5;
            }
        }
        
        self.grid = new_grid;
    }
    
    /// Run multiple NCA steps
    pub fn run_steps(&mut self, steps: usize, deterministic: bool) {
        for _ in 0..steps {
            if deterministic {
                self.step_deterministic();
            } else {
                self.step();
            }
        }
    }
    
    /// Store the current grid state as a memory pattern.
    /// Retrains the MLP on ALL stored memories to prevent catastrophic forgetting.
    pub fn store_memory(&mut self, name: &str) {
        let pattern = self.grid.clone();
        self.memories.push(pattern);
        self.memory_names.push(name.to_string());
        
        // Retrain from scratch on ALL memories (batch training prevents forgetting)
        self.retrain_all_memories();
    }
    
    /// Batch training: train the MLP on ALL stored memories simultaneously.
    /// This prevents catastrophic forgetting by learning a single update rule
    /// that works for all memories.
    fn retrain_all_memories(&mut self) {
        if self.memories.is_empty() { return; }
        
        let mut rng = rand::thread_rng();
        let scale1 = (2.0 / NEIGHBORHOOD_SIZE as f64).sqrt();
        let scale2 = (2.0 / HIDDEN_SIZE as f64).sqrt();
        
        // Reset weights
        for h in 0..HIDDEN_SIZE {
            for i in 0..NEIGHBORHOOD_SIZE {
                self.w1[h][i] = rng.gen_range(-scale1..scale1);
            }
            self.b1[h] = 0.0;
            self.w2[h] = rng.gen_range(-scale2..scale2);
        }
        self.b2 = 0.0;
        
        // Collect all clean examples from ALL memories
        let mut clean_examples: Vec<(Vec<f64>, f64)> = Vec::new();
        
        for pattern in &self.memories {
            for r in 0..self.size {
                for c in 0..self.size {
                    let mut neigh = Vec::with_capacity(NEIGHBORHOOD_SIZE);
                    for dr in [-1i32, 0, 1] {
                        for dc in [-1i32, 0, 1] {
                            let nr = ((r as i32 + dr).rem_euclid(self.size as i32)) as usize;
                            let nc = ((c as i32 + dc).rem_euclid(self.size as i32)) as usize;
                            neigh.push(if pattern[nr][nc] { 1.0 } else { 0.0 });
                        }
                    }
                    let target = if pattern[r][c] { 1.0 } else { 0.0 };
                    clean_examples.push((neigh, target));
                }
            }
        }
        
        // Train: first on clean examples (learn fixed points), then with noise (learn repair)
        let epochs = TRAIN_EPOCHS * self.memories.len().max(1);
        
        for phase in 0..2 {
            for _epoch in 0..epochs {
                use rand::seq::SliceRandom;
                
                if phase == 0 {
                    // Phase 1: Clean training — learn the fixed points
                    clean_examples.shuffle(&mut rng);
                    for (neighborhood, target) in &clean_examples {
                        self.sgd_step(neighborhood, *target);
                    }
                } else {
                    // Phase 2: Noisy training — learn to repair corrupted neighborhoods
                    // Generate noisy variants on-the-fly
                    let mut noisy_examples = clean_examples.clone();
                    for (neigh, _) in &mut noisy_examples {
                        for i in 0..NEIGHBORHOOD_SIZE {
                            // Skip center cell (index 4) — that's what we're predicting
                            if i != 4 && rng.gen::<f64>() < 0.15 {
                                neigh[i] = if neigh[i] > 0.5 { 0.0 } else { 1.0 };
                            }
                        }
                    }
                    noisy_examples.shuffle(&mut rng);
                    for (neighborhood, target) in &noisy_examples {
                        self.sgd_step(neighborhood, *target);
                    }
                }
            }
        }
    }
    
    /// Single SGD update step
    fn sgd_step(&mut self, neighborhood: &[f64], target: f64) {
        let (prob, hidden) = self.forward(neighborhood);
        let error = target - prob;
        
        // Output layer
        for h in 0..HIDDEN_SIZE {
            self.w2[h] += LEARNING_RATE * error * hidden[h];
        }
        self.b2 += LEARNING_RATE * error;
        
        // Hidden layer
        for h in 0..HIDDEN_SIZE {
            let hidden_grad = error * self.w2[h];
            let relu_deriv = if hidden[h] > 0.0 { 1.0 } else { 0.01 };
            let grad = hidden_grad * relu_deriv;
            
            for i in 0..NEIGHBORHOOD_SIZE {
                self.w1[h][i] += LEARNING_RATE * grad * neighborhood[i];
            }
            self.b1[h] += LEARNING_RATE * grad;
        }
    }
    
    /// Store a new pattern by generating a random one
    pub fn store_random_memory(&mut self, name: &str) {
        let mut rng = rand::thread_rng();
        let pattern: Vec<Vec<bool>> = (0..self.size)
            .map(|_| (0..self.size)
                .map(|_| rng.gen_bool(0.3)) // 30% density
                .collect())
            .collect();
        
        self.memories.push(pattern);
        self.memory_names.push(name.to_string());
        self.retrain_all_memories();
    }
    
    /// Load a memory pattern into the grid
    pub fn load_memory(&mut self, idx: usize) {
        if idx < self.memories.len() {
            self.grid = self.memories[idx].clone();
        }
    }
    
    /// Inject noise: randomly flip some cells
    pub fn inject_noise(&mut self, fraction: f64) {
        let mut rng = rand::thread_rng();
        let total = self.size * self.size;
        let flips = (total as f64 * fraction) as usize;
        
        for _ in 0..flips {
            let r = rng.gen_range(0..self.size);
            let c = rng.gen_range(0..self.size);
            self.grid[r][c] = !self.grid[r][c];
        }
    }
    
    /// Clear the grid (all cells OFF)
    pub fn clear(&mut self) {
        for row in &mut self.grid {
            for cell in row.iter_mut() {
                *cell = false;
            }
        }
    }
    
    /// Randomize the grid
    pub fn randomize(&mut self) {
        let mut rng = rand::thread_rng();
        for r in 0..self.size {
            for c in 0..self.size {
                self.grid[r][c] = rng.gen_bool(0.5);
            }
        }
    }
    
    /// Set a specific pattern (e.g., a test pattern like a smiley face)
    pub fn set_pattern(&mut self, pattern: &[&str]) {
        let mut r = 0;
        for line in pattern {
            let mut c = 0;
            for ch in line.chars() {
                if c < self.size {
                    self.grid[r][c] = ch != ' ' && ch != '.';
                    c += 1;
                }
            }
            r += 1;
            if r >= self.size { break; }
        }
    }
    
    /// Count how many cells match a given memory pattern
    pub fn match_score(&self, idx: usize) -> usize {
        if idx >= self.memories.len() { return 0; }
        let mut score = 0;
        for r in 0..self.size {
            for c in 0..self.size {
                if self.grid[r][c] == self.memories[idx][r][c] {
                    score += 1;
                }
            }
        }
        score
    }
    
    /// Count how many cells differ between current grid and a memory
    pub fn count_changed_from_memory(&self, idx: usize) -> usize {
        if idx >= self.memories.len() { return 0; }
        let mut changed = 0;
        for r in 0..self.size {
            for c in 0..self.size {
                if self.grid[r][c] != self.memories[idx][r][c] {
                    changed += 1;
                }
            }
        }
        changed
    }
    
    /// Find which memory the current grid is closest to
    pub fn closest_memory(&self) -> Option<(usize, f64)> {
        if self.memories.is_empty() { return None; }
        
        let mut best_idx = 0;
        let mut best_score = 0;
        
        for (idx, _) in self.memories.iter().enumerate() {
            let score = self.match_score(idx);
            if score > best_score {
                best_score = score;
                best_idx = idx;
            }
        }
        
        let fraction = best_score as f64 / (self.size * self.size) as f64;
        Some((best_idx, fraction))
    }
    
    /// Get the number of ON cells
    pub fn count_on(&self) -> usize {
        self.grid.iter()
            .flat_map(|row| row.iter())
            .filter(|&&x| x)
            .count()
    }
    
    /// Render grid as a string using Unicode block characters
    pub fn render(&self) -> String {
        let mut result = String::new();
        for row in &self.grid {
            for &cell in row {
                result.push(if cell { '█' } else { '░' });
            }
            result.push('\n');
        }
        result
    }
    
    /// Render grid compactly (2 cells per char using braille or half-blocks)
    pub fn render_compact(&self) -> String {
        let mut result = String::new();
        // Use half-height blocks: each char represents 2×1 cells
        for r in (0..self.size).step_by(2) {
            for c in 0..self.size {
                let top = self.grid[r][c];
                let bottom = if r + 1 < self.size { self.grid[r + 1][c] } else { false };
                let ch = match (top, bottom) {
                    (false, false) => ' ',
                    (false, true) => '▄',
                    (true, false) => '▀',
                    (true, true) => '█',
                };
                result.push(ch);
            }
            result.push('\n');
        }
        result
    }
}

// ── Helper Functions ───────────────────────────────────────────────────────

fn sigmoid(x: f64) -> f64 {
    1.0 / (1.0 + (-x).exp())
}

/// Generate a structured pattern (e.g., a block, stripe, checkerboard)
pub fn generate_pattern(kind: PatternKind, size: usize) -> Vec<Vec<bool>> {
    let mut grid = vec![vec![false; size]; size];
    
    match kind {
        PatternKind::Block => {
            let start = size / 4;
            let end = size * 3 / 4;
            for r in start..end {
                for c in start..end {
                    grid[r][c] = true;
                }
            }
        }
        PatternKind::HorizontalStripes => {
            for r in (0..size).step_by(4) {
                for c in 0..size {
                    grid[r][c] = true;
                }
            }
        }
        PatternKind::VerticalStripes => {
            for c in (0..size).step_by(4) {
                for r in 0..size {
                    grid[r][c] = true;
                }
            }
        }
        PatternKind::Checkerboard => {
            for r in 0..size {
                for c in 0..size {
                    grid[r][c] = (r + c) % 2 == 0;
                }
            }
        }
        PatternKind::Ring => {
            let cx = size / 2;
            let cy = size / 2;
            let inner_r = (size / 6) as i32;
            let outer_r = (size / 3) as i32;
            for r in 0..size {
                for c in 0..size {
                    let dx = r as i32 - cx as i32;
                    let dy = c as i32 - cy as i32;
                    let dist_sq = dx * dx + dy * dy;
                    grid[r][c] = (dist_sq >= inner_r * inner_r) && (dist_sq <= outer_r * outer_r);
                }
            }
        }
        PatternKind::Cross => {
            let mid = size / 2;
            let thickness = size / 8;
            for r in 0..size {
                for c in 0..size {
                    let h = (r as i32 - mid as i32).abs() < thickness as i32;
                    let v = (c as i32 - mid as i32).abs() < thickness as i32;
                    grid[r][c] = h || v;
                }
            }
        }
        PatternKind::RandomSparse => {
            let mut rng = rand::thread_rng();
            for r in 0..size {
                for c in 0..size {
                    grid[r][c] = rng.gen_bool(0.2);
                }
            }
        }
    }
    
    grid
}

#[derive(Clone, Copy, Debug)]
pub enum PatternKind {
    Block,
    HorizontalStripes,
    VerticalStripes,
    Checkerboard,
    Ring,
    Cross,
    RandomSparse,
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_binary_nca_creation() {
        let nca = BinaryNCA::new(8);
        assert_eq!(nca.size, 8);
        assert_eq!(nca.grid.len(), 8);
        assert_eq!(nca.grid[0].len(), 8);
    }
    
    #[test]
    fn test_pattern_generation() {
        let patterns = vec![
            PatternKind::Block,
            PatternKind::Checkerboard,
            PatternKind::Ring,
        ];
        for kind in patterns {
            let pattern = generate_pattern(kind, 16);
            assert_eq!(pattern.len(), 16);
            assert_eq!(pattern[0].len(), 16);
        }
    }
    
    #[test]
    fn test_fixed_points() {
        let mut nca = BinaryNCA::new(8);
        
        // Create and store a simple block pattern
        let pattern = generate_pattern(PatternKind::Block, 8);
        nca.grid = pattern.clone();
        nca.store_memory("block");
        
        // Load the memory
        nca.load_memory(0);
        
        // Run one deterministic step
        nca.step_deterministic();
        
        // Should be unchanged (fixed point)
        let changed = nca.count_changed_from_memory(0);
        assert_eq!(changed, 0, "Memory should be a fixed point: {} cells changed", changed);
    }
    
    #[test]
    fn test_multiple_memories() {
        let mut nca = BinaryNCA::new(16);
        
        // Store several patterns
        let kinds = vec![
            PatternKind::Block,
            PatternKind::Ring,
            PatternKind::Cross,
        ];
        
        for (i, kind) in kinds.iter().enumerate() {
            let pattern = generate_pattern(*kind, 16);
            nca.grid = pattern;
            nca.store_memory(&format!("pattern{}", i));
        }
        
        assert_eq!(nca.memories.len(), 3);
        assert_eq!(nca.memory_names.len(), 3);
        
        // Each memory should be a fixed point
        for idx in 0..3 {
            nca.load_memory(idx);
            nca.step_deterministic();
            let changed = nca.count_changed_from_memory(idx);
            assert_eq!(changed, 0, "Memory {} should be a fixed point", idx);
        }
    }
    
    #[test]
    fn test_batch_training_no_forgetting() {
        let mut nca = BinaryNCA::new(16);
        
        // Store pattern A
        let pattern_a = generate_pattern(PatternKind::Block, 16);
        nca.grid = pattern_a;
        nca.store_memory("block");
        
        // Verify pattern A is still a fixed point
        nca.load_memory(0);
        nca.step_deterministic();
        let changed_a = nca.count_changed_from_memory(0);
        
        // Store pattern B
        let pattern_b = generate_pattern(PatternKind::Ring, 16);
        nca.grid = pattern_b;
        nca.store_memory("ring");
        
        // Verify BOTH patterns are still fixed points
        nca.load_memory(0);
        nca.step_deterministic();
        let changed_a_after = nca.count_changed_from_memory(0);
        
        nca.load_memory(1);
        nca.step_deterministic();
        let changed_b = nca.count_changed_from_memory(1);
        
        assert_eq!(changed_a_after, 0, "Pattern A should remain a fixed point after storing B");
        assert_eq!(changed_b, 0, "Pattern B should be a fixed point");
    }
}
