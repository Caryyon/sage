// Grid module - handles the cellular automata grid and cell operations

use serde::{Serialize, Deserialize};

pub const GRID_SIZE: usize = 32;
pub const NUM_BASE_CHANNELS: usize = 16;  // 4 RGBA + 12 hidden
pub const NUM_PATTERN_CHANNELS: usize = 4;  // One-hot encoding for 4 patterns
pub const NUM_ENV_CHANNELS: usize = 2;  // Food (attract) and Toxin (repel)
pub const NUM_CHANNELS: usize = NUM_BASE_CHANNELS + NUM_PATTERN_CHANNELS + NUM_ENV_CHANNELS;  // 22 total

// Channel indices
pub const FOOD_CHANNEL: usize = NUM_BASE_CHANNELS + NUM_PATTERN_CHANNELS;  // Channel 20
pub const TOXIN_CHANNEL: usize = NUM_BASE_CHANNELS + NUM_PATTERN_CHANNELS + 1;  // Channel 21

// Grid represents the cellular automata world
#[derive(Clone, Serialize, Deserialize)]
pub struct Grid {
    pub cells: Vec<Vec<Vec<f64>>>,
    pub width: usize,
    pub height: usize,
    pub death_counters: Vec<Vec<u32>>,  // Track how long cells have been below threshold
    pub dead_cells: Vec<Vec<bool>>,     // Permanently dead cells
    pub species: Vec<Vec<u8>>,          // Which species owns this cell (0=none, 1=A, 2=B)
}

impl Grid {
    pub fn new(width: usize, height: usize) -> Self {
        let cells = vec![vec![vec![0.0; NUM_CHANNELS]; width]; height];
        let death_counters = vec![vec![0; width]; height];
        let dead_cells = vec![vec![false; width]; height];
        let species = vec![vec![0; width]; height];
        Grid { cells, width, height, death_counters, dead_cells, species }
    }

    pub fn seed(&mut self) {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        let center_y = self.height / 2;
        let center_x = self.width / 2;

        // Create a larger, more diverse seed region with random values
        let seed_radius = 4;  // Increased from 2 to 4 (9x9 region)
        for dy in -(seed_radius as i32)..=(seed_radius as i32) {
            for dx in -(seed_radius as i32)..=(seed_radius as i32) {
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

                    // Add some random noise to hidden channels too
                    for channel in 4..NUM_CHANNELS {
                        self.cells[y][x][channel] = rng.gen_range(0.0..0.3);
                    }
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

    // Apply damage to the grid
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
                for i in 0..NUM_PATTERN_CHANNELS {
                    self.cells[y][x][NUM_BASE_CHANNELS + i] = weights[i];
                }
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
                let ny = (y as i32 + dy_offset) as i32;
                let nx = (x as i32 + dx_offset) as i32;
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
            let ch = if alpha > 0.8 { '█' }
                    else if alpha > 0.5 { '▓' }
                    else if alpha > 0.2 { '▒' }
                    else if alpha > 0.1 { '░' }
                    else { ' ' };
            result.push(ch);
        }
        result.push('\n');
    }
    result
}
