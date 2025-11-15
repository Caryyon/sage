// Pattern generators for training phases

use super::phase_config::{PatternGenerator, Grid};
use crate::nca::NCA;

/// Generator for Phase 1: Simple expanding circles
pub struct CircleGenerator {
    size: usize,
}

impl CircleGenerator {
    pub fn new() -> Self {
        Self { size: 32 }
    }
}

impl PatternGenerator for CircleGenerator {
    fn generate(&self, _step: usize, progress: f64) -> Grid {
        let mut grid = vec![vec![0.0; self.size]; self.size];
        let center = self.size / 2;
        let radius = (progress / 100.0 * center as f64) as usize;

        for y in 0..self.size {
            for x in 0..self.size {
                let dx = x as i32 - center as i32;
                let dy = y as i32 - center as i32;
                let dist = ((dx * dx + dy * dy) as f64).sqrt();
                if dist < radius as f64 {
                    grid[y][x] = 1.0 - (dist / radius as f64) * 0.5;
                }
            }
        }

        grid
    }

    fn name(&self) -> &str {
        "CircleGenerator"
    }

    fn teaches_patterns(&self) -> Vec<String> {
        vec!["circle".to_string(), "center".to_string(), "symmetry".to_string()]
    }
}

/// Generator for Phase 2: Geometric patterns
pub struct GeometricGenerator {
    size: usize,
}

impl GeometricGenerator {
    pub fn new() -> Self {
        Self { size: 32 }
    }
}

impl PatternGenerator for GeometricGenerator {
    fn generate(&self, step: usize, progress: f64) -> Grid {
        let mut grid = vec![vec![0.0; self.size]; self.size];
        let center = self.size / 2;
        let size_param = (progress / 100.0 * 10.0) as usize + 2;

        for y in 0..self.size {
            for x in 0..self.size {
                let dx = (x as i32 - center as i32).abs() as usize;
                let dy = (y as i32 - center as i32).abs() as usize;
                if dx < size_param && dy < size_param {
                    grid[y][x] = 0.7 + ((x + y + step) as f64 * 0.1).sin() * 0.3;
                }
            }
        }

        grid
    }

    fn name(&self) -> &str {
        "GeometricGenerator"
    }

    fn teaches_patterns(&self) -> Vec<String> {
        vec!["edges".to_string(), "symmetry".to_string()]
    }
}

/// Generator for Phase 3: Settlement-like clusters
pub struct ClusterGenerator {
    size: usize,
}

impl ClusterGenerator {
    pub fn new() -> Self {
        Self { size: 32 }
    }
}

impl PatternGenerator for ClusterGenerator {
    fn generate(&self, step: usize, progress: f64) -> Grid {
        let mut grid = vec![vec![0.0; self.size]; self.size];
        let center = self.size / 2;
        let num_settlements = ((progress / 100.0) * 8.0) as usize + 1;

        for i in 0..num_settlements {
            let sx = (center + (i * 7 + step) % 20) % self.size;
            let sy = (center + (i * 11 + step / 2) % 20) % self.size;
            let cluster_size = 3 + (progress / 50.0) as usize;

            for y in 0..self.size {
                for x in 0..self.size {
                    let dx = (x as i32 - sx as i32).abs();
                    let dy = (y as i32 - sy as i32).abs();
                    let dist = ((dx * dx + dy * dy) as f64).sqrt();
                    if dist < cluster_size as f64 {
                        let value = 0.8 - dist / cluster_size as f64 * 0.6;
                        grid[y][x] = f64::max(grid[y][x], value);
                    }
                }
            }
        }

        grid
    }

    fn name(&self) -> &str {
        "ClusterGenerator"
    }

    fn teaches_patterns(&self) -> Vec<String> {
        vec!["clusters".to_string()]
    }
}

/// Generator for Phase 4: Dynamic patterns (waves, spirals, motion)
pub struct DynamicGenerator {
    size: usize,
}

impl DynamicGenerator {
    pub fn new() -> Self {
        Self { size: 32 }
    }
}

impl PatternGenerator for DynamicGenerator {
    fn generate(&self, step: usize, progress: f64) -> Grid {
        let mut grid = vec![vec![0.0; self.size]; self.size];
        let center = self.size / 2;

        for y in 0..self.size {
            for x in 0..self.size {
                // Wave patterns
                let wave1 = ((x as f64 / 3.0) + step as f64 * 0.3).sin() * 0.5 + 0.5;
                let wave2 = ((y as f64 / 3.0) + step as f64 * 0.2).cos() * 0.5 + 0.5;

                // Spiral pattern
                let dx = x as f64 - center as f64;
                let dy = y as f64 - center as f64;
                let angle = dy.atan2(dx);
                let radius = (dx * dx + dy * dy).sqrt();
                let spiral = ((angle * 3.0 + radius * 0.1 + step as f64 * 0.1).sin() * 0.5 + 0.5) * (progress / 100.0);

                // Motion (directional flow)
                let flow = ((x as f64 + step as f64) / self.size as f64) * (progress / 100.0);

                grid[y][x] = (wave1 * 0.3 + wave2 * 0.3 + spiral * 0.2 + flow * 0.2).min(1.0);
            }
        }

        grid
    }

    fn name(&self) -> &str {
        "DynamicGenerator"
    }

    fn teaches_patterns(&self) -> Vec<String> {
        vec!["waves".to_string(), "spiral".to_string(), "motion".to_string()]
    }
}

/// Generator for Phase 5: Meta-patterns (gradients, boundaries, complexity)
pub struct MetaPatternGenerator {
    size: usize,
}

impl MetaPatternGenerator {
    pub fn new() -> Self {
        Self { size: 32 }
    }
}

impl PatternGenerator for MetaPatternGenerator {
    fn generate(&self, step: usize, progress: f64) -> Grid {
        let mut grid = vec![vec![0.0; self.size]; self.size];
        let center = self.size / 2;

        for y in 0..self.size {
            for x in 0..self.size {
                let dx = x as f64 - center as f64;
                let dy = y as f64 - center as f64;
                let dist = (dx * dx + dy * dy).sqrt();

                // Smooth gradient from center
                let gradient = (1.0 - dist / center as f64).max(0.0) * (progress / 100.0);

                // Sharp boundaries (regions with distinct values)
                let region = if dist < center as f64 * 0.4 {
                    0.9
                } else if dist < center as f64 * 0.7 {
                    0.4
                } else {
                    0.1
                };

                // High complexity (varied local patterns)
                let noise = ((x as f64 * 0.7 + y as f64 * 1.3 + step as f64 * 0.5).sin() *
                            (x as f64 * 1.1 - y as f64 * 0.9).cos()) * 0.15;

                grid[y][x] = (gradient * 0.4 + region * 0.4 + noise * 0.2 + 0.5).min(1.0);
            }
        }

        grid
    }

    fn name(&self) -> &str {
        "MetaPatternGenerator"
    }

    fn teaches_patterns(&self) -> Vec<String> {
        vec!["gradient".to_string(), "boundaries".to_string(), "complexity".to_string()]
    }
}

/// Generator for NCA: Emergent patterns from Neural Cellular Automata
/// This creates truly dynamic, self-organizing patterns
pub struct NCAGenerator {
    nca: NCA,
}

impl NCAGenerator {
    pub fn new() -> Self {
        let mut nca = NCA::new();
        nca.reset_with_seed();

        Self {
            nca,
        }
    }
}

impl PatternGenerator for NCAGenerator {
    fn generate(&self, step: usize, _progress: f64) -> Grid {
        // Create a mutable copy for evolution
        let mut nca_copy = self.nca.clone();

        // Reset seed periodically to get variety
        if step % 100 == 0 {
            nca_copy.reset_with_seed();
        }

        // Evolve the NCA for several steps to get interesting patterns
        let evolution_steps = ((step % 100) / 10) + 1; // Gradually evolve more
        for _ in 0..evolution_steps {
            nca_copy.step();
        }

        // Extract the alpha channel as our 2D pattern
        // NCA has multiple channels (RGBA-like), we'll use alpha for visibility
        let height = nca_copy.grid.height;
        let width = nca_copy.grid.width;
        let mut grid = vec![vec![0.0; width]; height];

        for y in 0..height {
            for x in 0..width {
                // Use alpha channel (channel 3) for pattern intensity
                grid[y][x] = nca_copy.grid.cells[y][x][3];
            }
        }

        grid
    }

    fn name(&self) -> &str {
        "NCAGenerator"
    }

    fn teaches_patterns(&self) -> Vec<String> {
        vec![
            "emergence".to_string(),
            "self-organization".to_string(),
            "dynamic-patterns".to_string(),
            "growth".to_string(),
            "evolution".to_string(),
        ]
    }
}
