// Terrain module - procedural terrain generation functions

use crate::grid::Grid;
use rand::Rng;

// ========== PRIMITIVE PATTERNS (Phase 1: Foundation) ==========
// These are the building blocks that NCA learns first

// Create horizontal gradient (left to right)
pub fn create_gradient_horizontal(size: usize) -> Grid {
    let mut grid = Grid::new(size, size);

    for y in 0..size {
        for x in 0..size {
            let intensity = x as f64 / size as f64;
            grid.cells[y][x][0] = intensity;
            grid.cells[y][x][1] = intensity;
            grid.cells[y][x][2] = intensity;
            grid.cells[y][x][3] = 1.0;
        }
    }

    grid
}

// Create vertical gradient (top to bottom)
pub fn create_gradient_vertical(size: usize) -> Grid {
    let mut grid = Grid::new(size, size);

    for y in 0..size {
        for x in 0..size {
            let intensity = y as f64 / size as f64;
            grid.cells[y][x][0] = intensity;
            grid.cells[y][x][1] = intensity;
            grid.cells[y][x][2] = intensity;
            grid.cells[y][x][3] = 1.0;
        }
    }

    grid
}

// Create radial gradient (from center)
pub fn create_gradient_radial(size: usize) -> Grid {
    let mut grid = Grid::new(size, size);
    let center = size as f64 / 2.0;
    let max_dist = center * 1.414;  // Diagonal

    for y in 0..size {
        for x in 0..size {
            let dx = x as f64 - center;
            let dy = y as f64 - center;
            let dist = (dx * dx + dy * dy).sqrt();
            let intensity = (1.0 - (dist / max_dist)).max(0.0);

            grid.cells[y][x][0] = intensity;
            grid.cells[y][x][1] = intensity;
            grid.cells[y][x][2] = intensity;
            grid.cells[y][x][3] = 1.0;
        }
    }

    grid
}

// Create simple dot pattern (single cell activation)
pub fn create_dot(size: usize) -> Grid {
    let mut grid = Grid::new(size, size);
    let center = size / 2;

    grid.cells[center][center][0] = 1.0;
    grid.cells[center][center][1] = 1.0;
    grid.cells[center][center][2] = 1.0;
    grid.cells[center][center][3] = 1.0;

    grid
}

// ========== TERRAIN PATTERNS (Phase 3: Complex Application) ==========
// These use combinations of primitives learned in Phase 1

// Create mountain terrain with multiple peaks
// Uses: Radial gradients (peaks) + vertical gradients (elevation)
pub fn create_mountain_terrain(size: usize, peak_height: f64, num_peaks: usize) -> Grid {
    let mut grid = Grid::new(size, size);
    let mut rng = rand::thread_rng();

    // Generate multiple peaks
    for _ in 0..num_peaks {
        let peak_x = rng.gen_range(size / 4..3 * size / 4) as f64;
        let peak_y = rng.gen_range(size / 4..3 * size / 4) as f64;
        let radius = rng.gen_range(8.0..16.0);

        for y in 0..size {
            for x in 0..size {
                let dx = x as f64 - peak_x;
                let dy = y as f64 - peak_y;
                let dist = (dx * dx + dy * dy).sqrt();

                // Radial falloff from peak
                if dist < radius {
                    let height = peak_height * (1.0 - (dist / radius).powi(2));
                    grid.cells[y][x][0] = (grid.cells[y][x][0] + height).min(1.0);
                }
            }
        }
    }

    // Set alpha channel to 1.0 for all terrain
    for y in 0..size {
        for x in 0..size {
            let height = grid.cells[y][x][0];
            grid.cells[y][x][3] = 1.0;

            // Color based on elevation (will be used for visualization)
            if height > 0.7 {
                // High mountains - white/snow
                grid.cells[y][x][1] = height * 0.9;
                grid.cells[y][x][2] = height * 0.9;
            } else if height > 0.4 {
                // Mid elevation - brown/rock
                grid.cells[y][x][1] = height * 0.6;
                grid.cells[y][x][2] = height * 0.3;
            } else {
                // Low elevation - green
                grid.cells[y][x][1] = height * 0.8;
                grid.cells[y][x][2] = height * 0.2;
            }
        }
    }

    grid
}

// Create rolling hills with sine wave patterns
pub fn create_hills_terrain(size: usize, amplitude: f64, frequency: f64) -> Grid {
    let mut grid = Grid::new(size, size);

    for y in 0..size {
        for x in 0..size {
            let nx = x as f64 / size as f64;
            let ny = y as f64 / size as f64;

            // Combine multiple sine waves for natural-looking hills
            let height = amplitude * (
                (nx * frequency * std::f64::consts::PI * 2.0).sin() * 0.5 +
                (ny * frequency * std::f64::consts::PI * 2.0).sin() * 0.5 +
                ((nx + ny) * frequency * std::f64::consts::PI * 1.5).sin() * 0.3
            ).abs().min(1.0);

            grid.cells[y][x][0] = height;
            grid.cells[y][x][3] = 1.0;

            // Green for hills
            grid.cells[y][x][1] = 0.5 + height * 0.5;
            grid.cells[y][x][2] = 0.2 + height * 0.2;
        }
    }

    grid
}

// Create flat plains with gentle variation
pub fn create_plains_terrain(size: usize, variation: f64) -> Grid {
    let mut grid = Grid::new(size, size);
    let mut rng = rand::thread_rng();

    // Base elevation with small random variation
    let base_height = 0.3;

    for y in 0..size {
        for x in 0..size {
            let noise = rng.gen_range(-variation..variation);
            let height = (base_height + noise).clamp(0.0, 1.0);

            grid.cells[y][x][0] = height;
            grid.cells[y][x][3] = 1.0;

            // Yellowish-green for plains
            grid.cells[y][x][1] = 0.7;
            grid.cells[y][x][2] = 0.4;
        }
    }

    // Smooth the terrain
    let smoothed = grid.cells.clone();
    for y in 1..size-1 {
        for x in 1..size-1 {
            let mut sum = 0.0;
            let mut count = 0;
            for dy in -1..=1 {
                for dx in -1..=1 {
                    sum += smoothed[(y as i32 + dy) as usize][(x as i32 + dx) as usize][0];
                    count += 1;
                }
            }
            grid.cells[y][x][0] = sum / count as f64;
        }
    }

    grid
}

// Create valley terrain (inverted mountains)
pub fn create_valley_terrain(size: usize, depth: f64) -> Grid {
    let mut grid = Grid::new(size, size);
    let center_x = size as f64 / 2.0;
    let center_y = size as f64 / 2.0;

    for y in 0..size {
        for x in 0..size {
            let dx = x as f64 - center_x;
            let dy = y as f64 - center_y;
            let dist = (dx * dx + dy * dy).sqrt();
            let max_dist = (size as f64 / 2.0) * 1.414; // Diagonal distance

            // Higher at edges, lower in center
            let height = (dist / max_dist).min(1.0) * (1.0 - depth) + depth;

            grid.cells[y][x][0] = height;
            grid.cells[y][x][3] = 1.0;

            // Color based on depth
            if height < 0.3 {
                // Deep valley - darker green/blue
                grid.cells[y][x][1] = height * 0.5;
                grid.cells[y][x][2] = 0.6;
            } else {
                // Valley sides - green
                grid.cells[y][x][1] = height * 0.7;
                grid.cells[y][x][2] = height * 0.3;
            }
        }
    }

    grid
}

// ========== GEOMETRIC PATTERNS (Phase 2: Composition) ==========
// These combine primitives into recognizable shapes

// Circle - uses radial gradient
pub fn create_circle_target(size: usize, radius: f64) -> Grid {
    let mut grid = Grid::new(size, size);
    let center = size as f64 / 2.0;

    for y in 0..size {
        for x in 0..size {
            let dy = y as f64 - center;
            let dx = x as f64 - center;
            let dist = (dx * dx + dy * dy).sqrt();

            if dist <= radius {
                grid.cells[y][x][0] = 0.0;
                grid.cells[y][x][1] = 0.8;
                grid.cells[y][x][2] = 0.0;
                grid.cells[y][x][3] = 1.0;
            }
        }
    }

    grid
}

pub fn create_triangle_target(size: usize, height: f64) -> Grid {
    let mut grid = Grid::new(size, size);
    let center_x = size as f64 / 2.0;
    let base_y = size as f64 / 2.0 + height / 2.0;
    let top_y = size as f64 / 2.0 - height / 2.0;

    for y in 0..size {
        for x in 0..size {
            let y_pos = y as f64;
            let x_pos = x as f64;

            if y_pos >= top_y && y_pos <= base_y {
                let progress = (y_pos - top_y) / height;
                let half_width = progress * height / 2.0;

                if (x_pos - center_x).abs() <= half_width {
                    grid.cells[y][x][0] = 0.8;
                    grid.cells[y][x][1] = 0.0;
                    grid.cells[y][x][2] = 0.8;
                    grid.cells[y][x][3] = 1.0;
                }
            }
        }
    }

    grid
}

pub fn create_square_target(size: usize, side_length: f64) -> Grid {
    let mut grid = Grid::new(size, size);
    let center = size as f64 / 2.0;
    let half_side = side_length / 2.0;

    for y in 0..size {
        for x in 0..size {
            let dy = (y as f64 - center).abs();
            let dx = (x as f64 - center).abs();

            if dx <= half_side && dy <= half_side {
                grid.cells[y][x][0] = 0.8;
                grid.cells[y][x][1] = 0.8;
                grid.cells[y][x][2] = 0.0;
                grid.cells[y][x][3] = 1.0;
            }
        }
    }

    grid
}

pub fn create_cross_target(size: usize, thickness: f64, length: f64) -> Grid {
    let mut grid = Grid::new(size, size);
    let center = size as f64 / 2.0;
    let half_thickness = thickness / 2.0;
    let half_length = length / 2.0;

    for y in 0..size {
        for x in 0..size {
            let dy = (y as f64 - center).abs();
            let dx = (x as f64 - center).abs();

            // Horizontal bar
            let in_horizontal = dy <= half_thickness && dx <= half_length;
            // Vertical bar
            let in_vertical = dx <= half_thickness && dy <= half_length;

            if in_horizontal || in_vertical {
                grid.cells[y][x][0] = 0.0;
                grid.cells[y][x][1] = 0.8;
                grid.cells[y][x][2] = 0.8;
                grid.cells[y][x][3] = 1.0;
            }
        }
    }

    grid
}
