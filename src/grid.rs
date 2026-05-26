// Grid module - handles the cellular automata grid and cell operations

use serde::{Deserialize, Serialize};

pub const GRID_SIZE: usize = 256;
pub const NUM_BASE_CHANNELS: usize = 16; // 4 RGBA + 12 hidden
pub const NUM_PATTERN_CHANNELS: usize = 4; // One-hot encoding for 4 patterns
pub const NUM_ENV_CHANNELS: usize = 2; // Food (attract) and Toxin (repel)
pub const NUM_MEMORY_CHANNELS: usize = 4; // Memory-augmented cell channels
pub const NUM_KNOWLEDGE_CHANNELS: usize = 8; // Knowledge storage channels (6 embedding + activation + confidence)
pub const NUM_COMM_CHANNELS: usize = 2; // Cross-node communication channels
pub const NUM_META_CHANNELS: usize = 2; // Metadata channels (timestamp, legacy confidence alias)
pub const NUM_CHANNELS: usize = NUM_BASE_CHANNELS
    + NUM_PATTERN_CHANNELS
    + NUM_ENV_CHANNELS
    + NUM_MEMORY_CHANNELS
    + NUM_KNOWLEDGE_CHANNELS
    + NUM_COMM_CHANNELS
    + NUM_META_CHANNELS; // 38 total

// ── Channel Partitioning (shared vs private) ───────────────────────────────
// For p2p knowledge sharing: shared channels sync via gossip, private stay local.
// Layout: channels 0..35 are shared (synced across nodes), channels 36..37 are private.
pub const NUM_SHARED_CHANNELS: usize = 36;
pub const NUM_PRIVATE_CHANNELS: usize = 2;
pub const PRIVATE_CHANNELS_START: usize = NUM_SHARED_CHANNELS; // Channel 36

/// Returns true if the given channel index is shared (synced via gossip).
#[inline]
pub fn is_shared_channel(channel: usize) -> bool {
    channel < NUM_SHARED_CHANNELS
}

/// Returns true if the given channel index is private (local only).
#[inline]
pub fn is_private_channel(channel: usize) -> bool {
    (PRIVATE_CHANNELS_START..NUM_CHANNELS).contains(&channel)
}

// Channel indices
pub const FOOD_CHANNEL: usize = NUM_BASE_CHANNELS + NUM_PATTERN_CHANNELS; // Channel 20
pub const TOXIN_CHANNEL: usize = NUM_BASE_CHANNELS + NUM_PATTERN_CHANNELS + 1; // Channel 21

// Memory channel indices (channels 22-25)
pub const MEMORY_CHANNELS_START: usize =
    NUM_BASE_CHANNELS + NUM_PATTERN_CHANNELS + NUM_ENV_CHANNELS; // Channel 22
pub const MEMORY_ATTENTION: usize = MEMORY_CHANNELS_START; // Short-term attention weight
pub const MEMORY_GATE: usize = MEMORY_CHANNELS_START + 1; // Read/write gate (0=read, 1=write)
pub const MEMORY_VALUE: usize = MEMORY_CHANNELS_START + 2; // Memory value store
pub const MEMORY_RECENCY: usize = MEMORY_CHANNELS_START + 3; // Recency/novelty tag

// Knowledge channel indices (channels 26-33)
// Layout: 6 embedding slots + activation + confidence
pub const KNOWLEDGE_CHANNELS_START: usize = MEMORY_CHANNELS_START + NUM_MEMORY_CHANNELS; // Channel 26
pub const KNOWLEDGE_EMBEDDING_0: usize = KNOWLEDGE_CHANNELS_START; // Embedding slot 0
pub const KNOWLEDGE_EMBEDDING_1: usize = KNOWLEDGE_CHANNELS_START + 1; // Embedding slot 1
pub const KNOWLEDGE_EMBEDDING_2: usize = KNOWLEDGE_CHANNELS_START + 2; // Embedding slot 2
pub const KNOWLEDGE_EMBEDDING_3: usize = KNOWLEDGE_CHANNELS_START + 3; // Embedding slot 3
pub const KNOWLEDGE_EMBEDDING_4: usize = KNOWLEDGE_CHANNELS_START + 4; // Embedding slot 4
pub const KNOWLEDGE_EMBEDDING_5: usize = KNOWLEDGE_CHANNELS_START + 5; // Embedding slot 5
pub const KNOWLEDGE_ACTIVATION: usize = KNOWLEDGE_CHANNELS_START + 6; // Knowledge activation strength
pub const KNOWLEDGE_CONFIDENCE: usize = KNOWLEDGE_CHANNELS_START + 7; // Confidence score (0-1)

// Backward-compat aliases (KNOWLEDGE_EMBEDDING points to slot 0; META_* point into knowledge)
pub const KNOWLEDGE_EMBEDDING: usize = KNOWLEDGE_EMBEDDING_0;
pub const META_CONFIDENCE: usize = KNOWLEDGE_CONFIDENCE;
pub const META_TIMESTAMP: usize = KNOWLEDGE_EMBEDDING_5; // Slot 5 doubles as timestamp when not embedding

// Communication channel indices (channels 34-35)
pub const COMM_CHANNELS_START: usize = KNOWLEDGE_CHANNELS_START + NUM_KNOWLEDGE_CHANNELS; // Channel 34
pub const COMM_SYNC_STATE: usize = COMM_CHANNELS_START; // Sync state for cross-node communication
pub const COMM_NODE_ID: usize = COMM_CHANNELS_START + 1; // Source node identifier (hashed)

// Metadata channel indices (channels 36-37) — legacy/compat, kept for serialization stability
pub const META_CHANNELS_START: usize = COMM_CHANNELS_START + NUM_COMM_CHANNELS; // Channel 36
pub const META_TIMESTAMP_LEGACY: usize = META_CHANNELS_START; // Legacy timestamp slot
pub const META_CONFIDENCE_LEGACY: usize = META_CHANNELS_START + 1; // Legacy confidence slot

// Grid represents the cellular automata world
#[derive(Clone, Serialize, Deserialize)]
pub struct Grid {
    pub cells: Vec<Vec<Vec<f64>>>,
    pub width: usize,
    pub height: usize,
    pub death_counters: Vec<Vec<u32>>, // Track how long cells have been below threshold
    pub dead_cells: Vec<Vec<bool>>,    // Permanently dead cells
    pub species: Vec<Vec<u8>>,         // Which species owns this cell (0=none, 1=A, 2=B)
}

impl Grid {
    pub fn new(width: usize, height: usize) -> Self {
        let cells = vec![vec![vec![0.0; NUM_CHANNELS]; width]; height];
        let death_counters = vec![vec![0; width]; height];
        let dead_cells = vec![vec![false; width]; height];
        let species = vec![vec![0; width]; height];
        Grid {
            cells,
            width,
            height,
            death_counters,
            dead_cells,
            species,
        }
    }

    pub fn seed(&mut self) {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        let center_y = self.height / 2;
        let center_x = self.width / 2;

        // Create a larger, more diverse seed region with random values
        let seed_radius = 4; // Increased from 2 to 4 (9x9 region)
        for dy in -seed_radius..=seed_radius {
            for dx in -seed_radius..=seed_radius {
                let y = (center_y as i32 + dy) as usize;
                let x = (center_x as i32 + dx) as usize;

                if y < self.height && x < self.width {
                    let dist = ((dy * dy + dx * dx) as f64).sqrt();
                    let strength = (1.0 - dist / (seed_radius as f64 * 1.5)).max(0.0);

                    // RGBA channels with random variation for diversity
                    for channel in 0..4 {
                        // Random value between 0.3 and 1.0, scaled by distance from center
                        let random_val = rng.gen_range(0.3..1.0);
                        self.cells[y][x][channel] = random_val * strength;
                    }

                    // Add some random noise to hidden channels (4-21)
                    for channel in 4..MEMORY_CHANNELS_START {
                        self.cells[y][x][channel] = rng.gen_range(0.0..0.3);
                    }

                    // Initialize memory channels (22-25) with small values
                    // Memory starts "empty" but with slight noise for symmetry breaking
                    self.cells[y][x][MEMORY_ATTENTION] = rng.gen_range(0.0..0.1); // Low initial attention
                    self.cells[y][x][MEMORY_GATE] = 0.0; // Start in read mode
                    self.cells[y][x][MEMORY_VALUE] = 0.0; // Empty memory
                    self.cells[y][x][MEMORY_RECENCY] = 0.0; // No recency

                    // Initialize knowledge channels (26-33) - empty
                    for ke in
                        KNOWLEDGE_CHANNELS_START..KNOWLEDGE_CHANNELS_START + NUM_KNOWLEDGE_CHANNELS
                    {
                        self.cells[y][x][ke] = 0.0;
                    }

                    // Initialize communication channels (34-35) - empty
                    self.cells[y][x][COMM_SYNC_STATE] = 0.0;
                    self.cells[y][x][COMM_NODE_ID] = 0.0;
                }
            }
        }
    }

    pub fn get_cell(&self, y: i32, x: i32) -> &[f64] {
        let wrapped_y = ((y % self.height as i32) + self.height as i32) as usize % self.height;
        let wrapped_x = ((x % self.width as i32) + self.width as i32) as usize % self.width;
        &self.cells[wrapped_y][wrapped_x]
    }

    pub fn is_alive(&self, y: usize, x: usize) -> bool {
        self.cells[y][x][3] > 0.1
    }

    /// Calculate total mass (sum of alpha channel values)
    /// Used for tracking mass conservation
    pub fn total_mass(&self) -> f64 {
        self.cells.iter().flatten().map(|cell| cell[3]).sum()
    }

    /// Calculate mass of alive cells only (alpha > 0.1)
    pub fn alive_mass(&self) -> f64 {
        self.cells
            .iter()
            .flatten()
            .filter(|cell| cell[3] > 0.1)
            .map(|cell| cell[3])
            .sum()
    }

    /// Count alive cells
    pub fn alive_count(&self) -> usize {
        self.cells
            .iter()
            .flatten()
            .filter(|cell| cell[3] > 0.1)
            .count()
    }

    // Apply scattered random damage to the grid
    pub fn apply_damage(&mut self, damage_percent: f64) {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let total_cells = self.width * self.height;
        let cells_to_damage = (total_cells as f64 * damage_percent) as usize;

        for _ in 0..cells_to_damage {
            let x = rng.gen_range(0..self.width);
            let y = rng.gen_range(0..self.height);

            // Kill the cell
            for channel in 0..NUM_CHANNELS {
                self.cells[y][x][channel] = 0.0;
            }
        }
    }

    /// Apply localized rectangular damage (Growing CA paper style)
    /// Removes a rectangular region of cells to test regeneration
    pub fn apply_rectangular_damage(&mut self, size_fraction: f64) -> (usize, usize, usize, usize) {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        // Damage region size (fraction of grid)
        let damage_width = ((self.width as f64 * size_fraction) as usize).max(3);
        let damage_height = ((self.height as f64 * size_fraction) as usize).max(3);

        // Random position (ensure it's within bounds)
        let start_x = rng.gen_range(0..self.width.saturating_sub(damage_width));
        let start_y = rng.gen_range(0..self.height.saturating_sub(damage_height));

        // Zero out the rectangular region
        for y in start_y..(start_y + damage_height).min(self.height) {
            for x in start_x..(start_x + damage_width).min(self.width) {
                for channel in 0..NUM_CHANNELS {
                    self.cells[y][x][channel] = 0.0;
                }
            }
        }

        (start_x, start_y, damage_width, damage_height)
    }

    /// Apply circular damage centered on the pattern
    /// More challenging: removes the center of the pattern
    pub fn apply_circular_damage(&mut self, radius_fraction: f64) -> (usize, usize, f64) {
        let center_x = self.width / 2;
        let center_y = self.height / 2;
        let radius = (self.width as f64 * radius_fraction).max(2.0);

        for y in 0..self.height {
            for x in 0..self.width {
                let dx = x as f64 - center_x as f64;
                let dy = y as f64 - center_y as f64;
                let dist = (dx * dx + dy * dy).sqrt();

                if dist < radius {
                    for channel in 0..NUM_CHANNELS {
                        self.cells[y][x][channel] = 0.0;
                    }
                }
            }
        }

        (center_x, center_y, radius)
    }

    /// Apply half-pattern damage (remove left or right half)
    /// Tests if pattern can regenerate from partial state
    pub fn apply_half_damage(&mut self, remove_left: bool) {
        let mid_x = self.width / 2;

        for y in 0..self.height {
            let x_range = if remove_left {
                0..mid_x
            } else {
                mid_x..self.width
            };
            for x in x_range {
                for channel in 0..NUM_CHANNELS {
                    self.cells[y][x][channel] = 0.0;
                }
            }
        }
    }

    // Clone the grid
    pub fn clone_grid(&self) -> Self {
        Grid {
            cells: self.cells.clone(),
            width: self.width,
            height: self.height,
            death_counters: self.death_counters.clone(),
            dead_cells: self.dead_cells.clone(),
            species: self.species.clone(),
        }
    }

    // Update death tracking - cells below threshold too long become permanently dead
    pub fn update_death_tracking(&mut self, death_threshold_steps: u32) {
        for y in 0..self.height {
            for x in 0..self.width {
                if self.dead_cells[y][x] {
                    // Already dead, keep it dead
                    for channel in 0..NUM_CHANNELS {
                        self.cells[y][x][channel] = 0.0;
                    }
                    continue;
                }

                let alpha = self.cells[y][x][3];

                if alpha < 0.1 {
                    // Cell is below alive threshold
                    self.death_counters[y][x] += 1;

                    if self.death_counters[y][x] >= death_threshold_steps {
                        // Cell has been below threshold too long - permanent death
                        self.dead_cells[y][x] = true;
                        for channel in 0..NUM_CHANNELS {
                            self.cells[y][x][channel] = 0.0;
                        }
                    }
                } else {
                    // Cell is alive, reset counter
                    self.death_counters[y][x] = 0;
                }
            }
        }
    }

    // Get count of permanently dead cells
    pub fn count_dead(&self) -> usize {
        let mut count = 0;
        for y in 0..self.height {
            for x in 0..self.width {
                if self.dead_cells[y][x] {
                    count += 1;
                }
            }
        }
        count
    }

    // Count alive cells
    pub fn count_alive(&self) -> usize {
        let mut count = 0;
        for y in 0..self.height {
            for x in 0..self.width {
                if self.is_alive(y, x) {
                    count += 1;
                }
            }
        }
        count
    }

    // Set pattern conditioning (one-hot encoding in last 4 channels)
    pub fn set_pattern_condition(&mut self, pattern_id: usize) {
        for y in 0..self.height {
            for x in 0..self.width {
                // Clear all pattern channels
                for i in 0..NUM_PATTERN_CHANNELS {
                    self.cells[y][x][NUM_BASE_CHANNELS + i] = 0.0;
                }
                // Set the active pattern channel
                if pattern_id < NUM_PATTERN_CHANNELS {
                    self.cells[y][x][NUM_BASE_CHANNELS + pattern_id] = 1.0;
                }
            }
        }
    }

    // Set interpolated pattern conditioning (blended values)
    pub fn set_interpolated_condition(&mut self, weights: &[f64; 4]) {
        for y in 0..self.height {
            for x in 0..self.width {
                // Set each pattern channel to its weight
                self.cells[y][x][NUM_BASE_CHANNELS..(NUM_PATTERN_CHANNELS + NUM_BASE_CHANNELS)]
                    .copy_from_slice(&weights[..NUM_PATTERN_CHANNELS]);
            }
        }
    }

    // Place food at a location (attracts growth)
    pub fn place_food(&mut self, x: usize, y: usize, radius: f64, strength: f64) {
        let center_x = x as f64;
        let center_y = y as f64;

        for py in 0..self.height {
            for px in 0..self.width {
                let dx = px as f64 - center_x;
                let dy = py as f64 - center_y;
                let dist = (dx * dx + dy * dy).sqrt();

                if dist <= radius {
                    let falloff = 1.0 - (dist / radius);
                    self.cells[py][px][FOOD_CHANNEL] = (strength * falloff).min(1.0);
                }
            }
        }
    }

    // Place toxin at a location (repels/kills growth)
    pub fn place_toxin(&mut self, x: usize, y: usize, radius: f64, strength: f64) {
        let center_x = x as f64;
        let center_y = y as f64;

        for py in 0..self.height {
            for px in 0..self.width {
                let dx = px as f64 - center_x;
                let dy = py as f64 - center_y;
                let dist = (dx * dx + dy * dy).sqrt();

                if dist <= radius {
                    let falloff = 1.0 - (dist / radius);
                    self.cells[py][px][TOXIN_CHANNEL] = (strength * falloff).min(1.0);
                }
            }
        }
    }

    // Clear environmental channels
    pub fn clear_environment(&mut self) {
        for y in 0..self.height {
            for x in 0..self.width {
                self.cells[y][x][FOOD_CHANNEL] = 0.0;
                self.cells[y][x][TOXIN_CHANNEL] = 0.0;
            }
        }
    }

    /// Run N unconditioned NCA update steps (no new input signal).
    ///
    /// Based on rNCA (Silbernagel et al., 2025): NCA self-repair dynamics
    /// consolidate knowledge and prevent semantic drift after encoding.
    ///
    /// This is called after text encoding to let the grid "settle" its
    /// activation patterns before the next read. Cells communicate via
    /// local rules only. No external input changes the grid during repair.
    ///
    /// Only touches base hidden channels (4..NUM_BASE_CHANNELS).
    /// Knowledge channels (KNOWLEDGE_CHANNELS_START..) are left untouched.
    ///
    /// # Arguments
    /// * `cx`, `cy` - Center coordinates of the repair region
    /// * `radius` - Half-width of the repair window
    ///
    /// Snapshot the current state of knowledge channels (channels 26-31, the 6 embedding slots)
    /// across the entire grid. Returns a 2D array: snapshot[y][x] = [f32; 6].
    ///
    /// Used for NCA Delta Attention: capture state before NCA steps, compare after,
    /// and retrieve the cells with highest delta magnitude (the "activated" concepts).
    pub fn snapshot_knowledge_channels(&self) -> Vec<Vec<[f32; 6]>> {
        let mut snap = vec![vec![[0.0f32; 6]; self.width]; self.height];
        for (y, row) in snap.iter_mut().enumerate() {
            for (x, cell) in row.iter_mut().enumerate() {
                for (slot, cell_val) in cell.iter_mut().enumerate() {
                    *cell_val = self.cells[y][x][KNOWLEDGE_CHANNELS_START + slot] as f32;
                }
            }
        }
        snap
    }

    /// Compute per-cell L2 norm of the change between two knowledge channel snapshots.
    /// Returns a 2D array: delta[y][x] = L2 norm of change in the 6 embedding slots.
    pub fn compute_delta_magnitude(
        before: &[Vec<[f32; 6]>],
        after: &[Vec<[f32; 6]>],
    ) -> Vec<Vec<f32>> {
        let h = before.len();
        let w = if h > 0 { before[0].len() } else { 0 };
        let mut delta = vec![vec![0.0f32; w]; h];
        for y in 0..h.min(after.len()) {
            for x in 0..w.min(after[y].len()) {
                let mut sq_sum = 0.0f32;
                for slot in 0..6 {
                    let d = after[y][x][slot] - before[y][x][slot];
                    sq_sum += d * d;
                }
                delta[y][x] = sq_sum.sqrt();
            }
        }
        delta
    }

    /// Smooth hidden channels (4..NUM_BASE_CHANNELS) via diffusion.
    ///
    /// This is NOT a learned NCA operation — it applies trivial smoothing
    /// (70% current + 30% neighbor average) to hidden channels. Knowledge
    /// channels (26..33) are explicitly NOT modified.
    ///
    /// The name was changed from `freerun_repair` because "repair" implied
    /// more than the function actually does (diffusion, not learning).
    ///
    /// * `cx`, `cy` - Center of smoothing window
    /// * `radius` - Radius of smoothing window (cells within radius are smoothed)
    /// * `steps` - Number of smoothing iterations (more = more diffusion)
    pub fn smooth_hidden_channels(&mut self, cx: usize, cy: usize, radius: usize, steps: usize) {
        // Snapshot knowledge channels BEFORE repair to verify they're untouched
        #[cfg(debug_assertions)]
        let knowledge_snapshot: Vec<((usize, usize), Vec<f64>)> = {
            let mut snap = Vec::new();
            let r = radius as i32;
            for dy in -r..=r {
                for dx in -r..=r {
                    let nx = ((cx as i32 + dx).rem_euclid(self.width as i32)) as usize;
                    let ny = ((cy as i32 + dy).rem_euclid(self.height as i32)) as usize;
                    let k_vals: Vec<f64> = (KNOWLEDGE_CHANNELS_START..NUM_CHANNELS)
                        .map(|ch| self.cells[ny][nx][ch])
                        .collect();
                    snap.push(((nx, ny), k_vals));
                }
            }
            snap
        };

        let r = radius as i32;

        for _step in 0..steps {
            // Collect updates: for each cell in window, compute neighbor average of hidden channels
            let mut updates: Vec<(usize, usize, [f64; NUM_BASE_CHANNELS])> = Vec::new();

            for dy in -r..=r {
                for dx in -r..=r {
                    let nx = ((cx as i32 + dx).rem_euclid(self.width as i32)) as usize;
                    let ny = ((cy as i32 + dy).rem_euclid(self.height as i32)) as usize;

                    // Compute neighborhood average of hidden channels (4..16)
                    let mut neighbor_avg = [0.0f64; NUM_BASE_CHANNELS];
                    let mut neighbor_count = 0;

                    for ndy in -1i32..=1 {
                        for ndx in -1i32..=1 {
                            if ndy == 0 && ndx == 0 {
                                continue; // Skip self
                            }
                            let nnx = ((nx as i32 + ndx).rem_euclid(self.width as i32)) as usize;
                            let nny = ((ny as i32 + ndy).rem_euclid(self.height as i32)) as usize;

                            for (ch, avg) in neighbor_avg
                                .iter_mut()
                                .enumerate()
                                .skip(4)
                                .take(NUM_BASE_CHANNELS - 4)
                            {
                                *avg += self.cells[nny][nnx][ch];
                            }
                            neighbor_count += 1;
                        }
                    }

                    if neighbor_count > 0 {
                        for avg in neighbor_avg.iter_mut().skip(4).take(NUM_BASE_CHANNELS - 4) {
                            *avg /= neighbor_count as f64;
                        }
                    }

                    updates.push((nx, ny, neighbor_avg));
                }
            }

            // Apply smoothing: new_val = 0.7 * current + 0.3 * neighbor_avg
            for (nx, ny, neighbor_avg) in updates {
                for (ch, avg) in neighbor_avg
                    .iter()
                    .enumerate()
                    .skip(4)
                    .take(NUM_BASE_CHANNELS - 4)
                {
                    self.cells[ny][nx][ch] = self.cells[ny][nx][ch] * 0.7 + avg * 0.3;
                }
            }
        }

        // Verify knowledge channels were NOT modified (debug builds only)
        #[cfg(debug_assertions)]
        {
            for ((nx, ny), old_vals) in knowledge_snapshot {
                for (i, &old_val) in old_vals.iter().enumerate() {
                    let ch = KNOWLEDGE_CHANNELS_START + i;
                    let new_val = self.cells[ny][nx][ch];
                    debug_assert!(
                        (new_val - old_val).abs() < 1e-10,
                        "smooth_hidden_channels must not modify knowledge channels: \
                         channel {} at ({},{}) changed from {} to {}",
                        ch,
                        nx,
                        ny,
                        old_val,
                        new_val
                    );
                }
            }
        }
    }

    /// Consolidate knowledge via local NCA-style update rules.
    ///
    /// This implements the "dream cycle" for knowledge channels (26-33):
    /// - Strengthens frequently accessed cells (boost activation & confidence)
    /// - Decays inactive cells (reduce activation over time)
    /// - Spreads activation to neighbors (Hebbian-like association)
    ///
    /// Unlike smooth_hidden_channels (diffusion), this operates on knowledge channels
    /// and uses learned-style dynamics even though weights are fixed.
    ///
    /// # Arguments
    /// * `steps` - Number of consolidation iterations (default: 1-3)
    ///
    /// # Channel Effects
    /// - KNOWLEDGE_ACTIVATION (ch 32): Strengthened by usage, decayed by time
    /// - KNOWLEDGE_CONFIDENCE (ch 33): Increased when activation is stable
    /// - Embedding slots (ch 26-31): Light diffusion to spread associations
    pub fn consolidate_knowledge(&mut self, steps: usize) {
        // Update parameters (tunable)
        const DECAY_RATE: f64 = 0.02; // Per-step decay for inactive cells
        const STRENGTHEN_RATE: f64 = 0.05; // Boost for high-activation cells
        const SPREAD_RATE: f64 = 0.03; // How much activation spreads to neighbors
        const CONFIDENCE_BOOST: f64 = 0.02; // Confidence increase for stable cells
        const ACTIVATION_THRESHOLD: f64 = 0.3; // Minimum activation to count as "active"

        for _step in 0..steps {
            // Collect updates in a separate buffer to avoid in-place mutation artifacts
            let mut activation_updates: Vec<(usize, usize, f64)> = Vec::new();
            let mut confidence_updates: Vec<(usize, usize, f64)> = Vec::new();
            let mut embedding_updates: Vec<(usize, usize, usize, f64)> = Vec::new();

            for y in 0..self.height {
                for x in 0..self.width {
                    let activation = self.cells[y][x][KNOWLEDGE_ACTIVATION];
                    let confidence = self.cells[y][x][KNOWLEDGE_CONFIDENCE];

                    // Skip cells with no knowledge
                    if activation < 0.01 {
                        continue;
                    }

                    // Count active neighbors (Hebbian: co-activated cells strengthen together)
                    let mut active_neighbors = 0usize;
                    let mut neighbor_embedding_sum: [f64; 6] = [0.0; 6];
                    let mut neighbor_count = 0usize;

                    for dy in -1i32..=1 {
                        for dx in -1i32..=1 {
                            if dx == 0 && dy == 0 {
                                continue;
                            }
                            let ny = (y as i32 + dy) as usize;
                            let nx = (x as i32 + dx) as usize;
                            if ny < self.height && nx < self.width {
                                let n_act = self.cells[ny][nx][KNOWLEDGE_ACTIVATION];
                                if n_act > ACTIVATION_THRESHOLD {
                                    active_neighbors += 1;
                                }
                                neighbor_count += 1;
                                // Collect embedding values for diffusion
                                for slot in 0..6 {
                                    neighbor_embedding_sum[slot] +=
                                        self.cells[ny][nx][KNOWLEDGE_CHANNELS_START + slot];
                                }
                            }
                        }
                    }

                    // Rule 1: Decay for inactive cells (not recently accessed)
                    // High activation resists decay
                    if activation < ACTIVATION_THRESHOLD {
                        let decay = DECAY_RATE * (1.0 - activation);
                        activation_updates.push((x, y, -decay));
                    } else {
                        // Rule 2: Strengthen active cells with active neighbors (Hebbian)
                        if active_neighbors >= 2 {
                            let strengthen = STRENGTHEN_RATE * activation;
                            activation_updates.push((x, y, strengthen));

                            // Boost confidence for stable, well-connected knowledge
                            let conf_boost = CONFIDENCE_BOOST * confidence.min(1.0);
                            confidence_updates.push((x, y, conf_boost));
                        }
                    }

                    // Rule 3: Spread activation to neighbors (Hebbian association)
                    if activation > ACTIVATION_THRESHOLD && neighbor_count > 0 {
                        let spread = SPREAD_RATE * activation / neighbor_count as f64;
                        for dy in -1i32..=1 {
                            for dx in -1i32..=1 {
                                if dx == 0 && dy == 0 {
                                    continue;
                                }
                                let ny = (y as i32 + dy) as usize;
                                let nx = (x as i32 + dx) as usize;
                                if ny < self.height && nx < self.width {
                                    // Only spread to cells with some knowledge (avoid noise)
                                    if self.cells[ny][nx][KNOWLEDGE_ACTIVATION] > 0.01 {
                                        activation_updates.push((nx, ny, spread));
                                    }
                                }
                            }
                        }
                    }

                    // Rule 4: Light embedding diffusion for semantic spreading
                    if activation > 0.1 && neighbor_count > 0 {
                        for slot in 0..6 {
                            let avg_neighbor = neighbor_embedding_sum[slot] / neighbor_count as f64;
                            let current = self.cells[y][x][KNOWLEDGE_CHANNELS_START + slot];
                            // 95% current + 5% neighbor average (very light diffusion)
                            let new_val = current * 0.95 + avg_neighbor * 0.05;
                            embedding_updates.push((x, y, slot, new_val));
                        }
                    }
                }
            }

            // Apply all updates
            for (x, y, delta) in &activation_updates {
                self.cells[*y][*x][KNOWLEDGE_ACTIVATION] =
                    (self.cells[*y][*x][KNOWLEDGE_ACTIVATION] + delta).clamp(0.0, 1.0);
            }
            for (x, y, delta) in &confidence_updates {
                self.cells[*y][*x][KNOWLEDGE_CONFIDENCE] =
                    (self.cells[*y][*x][KNOWLEDGE_CONFIDENCE] + delta).clamp(0.0, 1.0);
            }
            for (x, y, slot, val) in &embedding_updates {
                self.cells[*y][*x][KNOWLEDGE_CHANNELS_START + slot] = *val;
            }
        }
    }
}

// Perception function - computes Sobel gradients for each channel
pub fn perceive(grid: &Grid, y: usize, x: usize) -> Vec<f64> {
    let mut perception = Vec::new();
    let sobel_x = [[-1.0, 0.0, 1.0], [-2.0, 0.0, 2.0], [-1.0, 0.0, 1.0]];
    let sobel_y = [[-1.0, -2.0, -1.0], [0.0, 0.0, 0.0], [1.0, 2.0, 1.0]];

    for channel in 0..NUM_CHANNELS {
        let mut dx = 0.0;
        let mut dy = 0.0;
        let mut center_val = 0.0;

        for dy_offset in -1..=1 {
            for dx_offset in -1..=1 {
                let ny = y as i32 + dy_offset;
                let nx = x as i32 + dx_offset;
                let cell = grid.get_cell(ny, nx);
                let val = cell[channel];

                let ky = (dy_offset + 1) as usize;
                let kx = (dx_offset + 1) as usize;

                dx += val * sobel_x[ky][kx];
                dy += val * sobel_y[ky][kx];

                if dy_offset == 0 && dx_offset == 0 {
                    center_val = val;
                }
            }
        }

        perception.push(center_val);
        perception.push(dx);
        perception.push(dy);
    }

    perception
}

// Convert grid to ASCII for debugging
pub fn grid_to_ascii(grid: &Grid) -> String {
    let mut result = String::new();
    for y in 0..grid.height {
        for x in 0..grid.width {
            let alpha = grid.cells[y][x][3];
            let ch = if alpha > 0.8 {
                '█'
            } else if alpha > 0.5 {
                '▓'
            } else if alpha > 0.2 {
                '▒'
            } else if alpha > 0.1 {
                '░'
            } else {
                ' '
            };
            result.push(ch);
        }
        result.push('\n');
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_channel_counts() {
        assert_eq!(NUM_CHANNELS, 38, "Total channels should be 38");
        assert_eq!(
            NUM_SHARED_CHANNELS + NUM_PRIVATE_CHANNELS,
            NUM_CHANNELS,
            "Shared + private should equal total"
        );
    }

    #[test]
    fn test_channel_partitioning() {
        // Channels 0..23 are shared
        for ch in 0..NUM_SHARED_CHANNELS {
            assert!(is_shared_channel(ch), "Channel {} should be shared", ch);
            assert!(
                !is_private_channel(ch),
                "Channel {} should not be private",
                ch
            );
        }
        // Channels 24..31 are private
        for ch in PRIVATE_CHANNELS_START..NUM_CHANNELS {
            assert!(is_private_channel(ch), "Channel {} should be private", ch);
            assert!(
                !is_shared_channel(ch),
                "Channel {} should not be shared",
                ch
            );
        }
    }

    #[test]
    fn test_grid_new_dimensions() {
        let grid = Grid::new(64, 64);
        assert_eq!(grid.width, 64);
        assert_eq!(grid.height, 64);
        assert_eq!(grid.cells[0][0].len(), NUM_CHANNELS);
    }

    #[test]
    fn test_freerun_does_not_touch_knowledge_channels() {
        let mut grid = Grid::new(64, 64);
        let cx = 32;
        let cy = 32;

        // Set some knowledge channel values
        for ch in KNOWLEDGE_CHANNELS_START..NUM_CHANNELS {
            grid.cells[cy][cx][ch] = 0.5;
        }

        // Snapshot before
        let before: Vec<f64> = (KNOWLEDGE_CHANNELS_START..NUM_CHANNELS)
            .map(|ch| grid.cells[cy][cx][ch])
            .collect();

        // Run freerun repair
        grid.smooth_hidden_channels(cx, cy, 4, 5);

        // Verify unchanged
        for (i, ch) in (KNOWLEDGE_CHANNELS_START..NUM_CHANNELS).enumerate() {
            assert!(
                (grid.cells[cy][cx][ch] - before[i]).abs() < 1e-10,
                "Knowledge channel {} should be unchanged after smooth_hidden_channels",
                ch
            );
        }
    }

    #[test]
    fn test_freerun_smooths_hidden_channels() {
        let mut grid = Grid::new(64, 64);
        let cx = 32;
        let cy = 32;

        // Set center cell hidden channel 5 to 1.0, neighbors to 0.0
        grid.cells[cy][cx][5] = 1.0;

        // All neighbors at 0.0 (default)
        // After smoothing: new = 0.7 * 1.0 + 0.3 * 0.0 = 0.7 (first step)

        let before = grid.cells[cy][cx][5];
        grid.smooth_hidden_channels(cx, cy, 2, 3);
        let after = grid.cells[cy][cx][5];

        assert!(
            after < before,
            "Hidden channel should decrease after smoothing: before={}, after={}",
            before,
            after
        );
    }

    #[test]
    fn test_freerun_stays_in_bounds() {
        let mut grid = Grid::new(64, 64);

        // Set some values at the edge
        grid.cells[0][0][5] = 1.0;
        grid.cells[0][0][KNOWLEDGE_CHANNELS_START] = 0.9;

        // Should not panic when center is at grid edge
        grid.smooth_hidden_channels(0, 0, 4, 3);

        // Grid should still be valid
        assert!(grid.cells[0][0][5].is_finite());
        assert!(grid.cells[0][0][KNOWLEDGE_CHANNELS_START].is_finite());
    }

    #[test]
    fn test_freerun_with_large_radius() {
        let mut grid = Grid::new(32, 32);

        // Set values across the grid
        for y in 0..32 {
            for x in 0..32 {
                grid.cells[y][x][5] = ((x + y) % 10) as f64 / 10.0;
            }
        }

        // Radius larger than half grid should wrap correctly
        grid.smooth_hidden_channels(16, 16, 20, 2);

        // Should complete without panic
        assert!(grid.cells[16][16][5].is_finite());
    }

    #[test]
    fn test_consolidate_knowledge_decays_inactive() {
        let mut grid = Grid::new(64, 64);

        // Set low-activation knowledge (below threshold)
        grid.cells[32][32][KNOWLEDGE_ACTIVATION] = 0.1; // Below ACTIVATION_THRESHOLD (0.3)
        grid.cells[32][32][KNOWLEDGE_CONFIDENCE] = 0.5;

        let before = grid.cells[32][32][KNOWLEDGE_ACTIVATION];
        grid.consolidate_knowledge(1);
        let after = grid.cells[32][32][KNOWLEDGE_ACTIVATION];

        assert!(
            after < before,
            "Low-activation cell should decay: before={}, after={}",
            before,
            after
        );
    }

    #[test]
    fn test_consolidate_knowledge_strengthens_active_with_neighbors() {
        let mut grid = Grid::new(64, 64);

        // Create a cluster of active cells (center + 4 neighbors above threshold)
        grid.cells[32][32][KNOWLEDGE_ACTIVATION] = 0.8; // Center
        grid.cells[31][32][KNOWLEDGE_ACTIVATION] = 0.6; // Above
        grid.cells[33][32][KNOWLEDGE_ACTIVATION] = 0.6; // Below
        grid.cells[32][31][KNOWLEDGE_ACTIVATION] = 0.6; // Left
        grid.cells[32][33][KNOWLEDGE_ACTIVATION] = 0.6; // Right
        grid.cells[32][32][KNOWLEDGE_CONFIDENCE] = 0.5;

        let before = grid.cells[32][32][KNOWLEDGE_ACTIVATION];
        grid.consolidate_knowledge(1);
        let after = grid.cells[32][32][KNOWLEDGE_ACTIVATION];

        assert!(
            after >= before,
            "Active cell with active neighbors should strengthen or stay stable: before={}, after={}",
            before,
            after
        );
    }

    #[test]
    fn test_consolidate_knowledge_stays_in_bounds() {
        let mut grid = Grid::new(64, 64);

        // Set high activation at edge
        grid.cells[0][0][KNOWLEDGE_ACTIVATION] = 0.9;
        grid.cells[0][0][KNOWLEDGE_CONFIDENCE] = 0.8;
        grid.cells[0][1][KNOWLEDGE_ACTIVATION] = 0.7; // Neighbor
        grid.cells[1][0][KNOWLEDGE_ACTIVATION] = 0.7; // Neighbor

        // Should not panic
        grid.consolidate_knowledge(3);

        // All channels should remain in [0, 1]
        for ch in KNOWLEDGE_CHANNELS_START..NUM_CHANNELS {
            let val = grid.cells[0][0][ch];
            assert!(
                val >= 0.0 && val <= 1.0,
                "Knowledge channel {} out of bounds: {}",
                ch,
                val
            );
        }
    }

    // ── Integration Tests: Knowledge Consolidation with Retrieval ────────────────
    //
    // These tests verify that consolidate_knowledge() works correctly with
    // actual knowledge encoding and retrieval, not just raw grid cells.

    /// Test that consolidation preserves retrieval recall for encoded facts.
    ///
    /// The consolidation algorithm decays inactive cells and strengthens
    /// active cells with active neighbors. This should not destroy encoded
    /// knowledge that was recently accessed.
    #[test]
    fn test_consolidation_preserves_recently_accessed_knowledge() {
        use crate::distributed_knowledge::{KnowledgeStore, NCAKnowledge};

        let mut knowledge = NCAKnowledge::new();
        knowledge.config.ollama_url = None; // Use hash-based embeddings

        // Encode several related facts about Python
        knowledge.encode("Python is a programming language", 0.9);
        knowledge.encode("Python has dynamic typing", 0.85);
        knowledge.encode("Python supports object-oriented programming", 0.8);

        // Query to establish access patterns
        let before = knowledge.query("Python programming", 10);
        let before_count = before.iter().filter(|r| r.relevance > 0.3).count();

        // Run consolidation (simulates "dream cycle")
        knowledge.grid.consolidate_knowledge(3);

        // Query after consolidation
        let after = knowledge.query("Python programming", 10);
        let after_count = after.iter().filter(|r| r.relevance > 0.3).count();

        // Consolidation should preserve at least half of recall quality
        assert!(
            after_count >= before_count / 2,
            "Consolidation destroyed too much recall: before={}, after={}",
            before_count,
            after_count
        );
    }

    /// Test that consolidation survives save/load persistence.
    ///
    /// Knowledge should remain retrievable after consolidation + serialization.
    #[test]
    fn test_consolidation_survives_persistence() {
        use crate::distributed_knowledge::{KnowledgeStore, NCAKnowledge};

        let path = "/tmp/sage_test_consolidation_persistence.bin";
        let _ = std::fs::remove_file(path);

        let mut knowledge = NCAKnowledge::new();
        knowledge.config.ollama_url = None;

        // Encode a fact with high confidence
        knowledge.encode("Rust has ownership semantics", 0.9);

        // Run consolidation
        knowledge.grid.consolidate_knowledge(2);

        // Save
        knowledge.save(path).expect("Save should succeed");

        // Load into fresh instance
        let mut loaded = NCAKnowledge::new();
        loaded.load(path).expect("Load should succeed");

        // Query should still find the knowledge
        let results = loaded.query("Rust ownership", 5);
        assert!(
            !results.is_empty(),
            "Knowledge should survive consolidation + save/load"
        );

        // Verify relevance is reasonable
        assert!(
            results[0].relevance > 0.1,
            "Retrieved knowledge should have meaningful relevance: {:?}",
            results[0]
        );

        let _ = std::fs::remove_file(path);
    }

    /// Test that consolidation respects activation thresholds.
    ///
    /// Cells below ACTIVATION_THRESHOLD should decay, cells above with
    /// active neighbors should strengthen.
    #[test]
    fn test_consolidation_threshold_behavior() {
        let mut grid = Grid::new(64, 64);

        // High-activation cell with active neighbors (should strengthen)
        grid.cells[32][32][KNOWLEDGE_ACTIVATION] = 0.8;
        grid.cells[32][32][KNOWLEDGE_CONFIDENCE] = 0.9;
        grid.cells[31][32][KNOWLEDGE_ACTIVATION] = 0.6;
        grid.cells[33][32][KNOWLEDGE_ACTIVATION] = 0.6;
        grid.cells[32][31][KNOWLEDGE_ACTIVATION] = 0.6;
        grid.cells[32][33][KNOWLEDGE_ACTIVATION] = 0.6;

        // Low-activation cell (should decay)
        grid.cells[10][10][KNOWLEDGE_ACTIVATION] = 0.1;
        grid.cells[10][10][KNOWLEDGE_CONFIDENCE] = 0.5;

        let high_before = grid.cells[32][32][KNOWLEDGE_ACTIVATION];
        let low_before = grid.cells[10][10][KNOWLEDGE_ACTIVATION];

        grid.consolidate_knowledge(1);

        let high_after = grid.cells[32][32][KNOWLEDGE_ACTIVATION];
        let low_after = grid.cells[10][10][KNOWLEDGE_ACTIVATION];

        // High-activation should stay high or increase
        assert!(
            high_after >= high_before * 0.9,
            "High-activation cell should not decay significantly: before={}, after={}",
            high_before,
            high_after
        );

        // Low-activation should have decayed
        assert!(
            low_after < low_before,
            "Low-activation cell should decay: before={}, after={}",
            low_before,
            low_after
        );
    }

    /// Test that multiple consolidation steps don't destroy knowledge.
    ///
    /// Repeated consolidation should converge, not collapse to zero.
    #[test]
    fn test_repeated_consolidation_converges() {
        use crate::distributed_knowledge::{KnowledgeStore, NCAKnowledge};

        let mut knowledge = NCAKnowledge::new();
        knowledge.config.ollama_url = None;

        // Encode multiple facts
        knowledge.encode("Neural networks learn from data", 0.9);
        knowledge.encode("Training requires gradient descent", 0.85);
        knowledge.encode("Backpropagation computes gradients", 0.8);

        // Initial query
        let initial = knowledge.query("neural network training", 5);
        let initial_relevance = initial.first().map(|r| r.relevance).unwrap_or(0.0);

        // Run multiple consolidation rounds
        for _ in 0..5 {
            knowledge.grid.consolidate_knowledge(2);
        }

        // Query after repeated consolidation
        let final_result = knowledge.query("neural network training", 5);
        let final_relevance = final_result.first().map(|r| r.relevance).unwrap_or(0.0);

        // Should maintain at least 30% of original relevance
        assert!(
            final_relevance >= initial_relevance * 0.3,
            "Repeated consolidation destroyed too much: initial={}, final={}",
            initial_relevance,
            final_relevance
        );
    }
}
