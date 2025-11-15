// Full training implementation with spiral curriculum

use crate::*;

/// Generate target pattern for training
pub fn generate_target_pattern(pattern: TargetPattern) -> Grid {
    use sage::grid::GRID_SIZE;

    let mut grid = Grid::new(GRID_SIZE, GRID_SIZE);
    let center_x = GRID_SIZE / 2;
    let center_y = GRID_SIZE / 2;

    match pattern {
        TargetPattern::Circle => {
            let radius = 8.0;
            for y in 0..GRID_SIZE {
                for x in 0..GRID_SIZE {
                    let dx = x as f64 - center_x as f64;
                    let dy = y as f64 - center_y as f64;
                    let dist = (dx * dx + dy * dy).sqrt();

                    if dist < radius {
                        grid.cells[y][x][3] = 1.0;
                        grid.cells[y][x][0] = 1.0;
                    }
                }
            }
        }
        TargetPattern::Square => {
            let size = 12;
            for y in (center_y.saturating_sub(size))..(center_y + size).min(GRID_SIZE) {
                for x in (center_x.saturating_sub(size))..(center_x + size).min(GRID_SIZE) {
                    grid.cells[y][x][3] = 1.0;
                    grid.cells[y][x][1] = 1.0;
                }
            }
        }
        TargetPattern::Cross => {
            let arm_width = 4;
            let arm_length = 10;

            // Horizontal
            for x in (center_x.saturating_sub(arm_length))..(center_x + arm_length).min(GRID_SIZE) {
                for dy in 0..arm_width {
                    if let Some(y) = center_y.checked_sub(arm_width/2).and_then(|cy| cy.checked_add(dy)) {
                        if y < GRID_SIZE {
                            grid.cells[y][x][3] = 1.0;
                            grid.cells[y][x][2] = 1.0;
                        }
                    }
                }
            }

            // Vertical
            for y in (center_y.saturating_sub(arm_length))..(center_y + arm_length).min(GRID_SIZE) {
                for dx in 0..arm_width {
                    if let Some(x) = center_x.checked_sub(arm_width/2).and_then(|cx| cx.checked_add(dx)) {
                        if x < GRID_SIZE {
                            grid.cells[y][x][3] = 1.0;
                            grid.cells[y][x][2] = 1.0;
                        }
                    }
                }
            }
        }
        TargetPattern::Spiral => {
            for i in 0..100 {
                let angle = i as f64 * 0.3;
                let radius = i as f64 * 0.1;
                let x = (center_x as f64 + radius * angle.cos()) as usize;
                let y = (center_y as f64 + radius * angle.sin()) as usize;

                if x < GRID_SIZE && y < GRID_SIZE {
                    grid.cells[y][x][3] = 1.0;
                    grid.cells[y][x][0] = 1.0;
                    grid.cells[y][x][1] = 1.0;
                }
            }
        }
        TargetPattern::Hexagon => {
            // Regular hexagon with 6 sides
            let radius = 10.0;
            let line_thickness = 2;

            // Draw 6 edges connecting vertices
            for edge in 0..6 {
                let angle1 = (edge as f64) * std::f64::consts::PI / 3.0;
                let angle2 = ((edge + 1) as f64) * std::f64::consts::PI / 3.0;

                let x1 = center_x as f64 + radius * angle1.cos();
                let y1 = center_y as f64 + radius * angle1.sin();
                let x2 = center_x as f64 + radius * angle2.cos();
                let y2 = center_y as f64 + radius * angle2.sin();

                // Draw line between vertices using Bresenham-like approach
                let steps = 50;
                for step in 0..=steps {
                    let t = step as f64 / steps as f64;
                    let x = (x1 + (x2 - x1) * t) as isize;
                    let y = (y1 + (y2 - y1) * t) as isize;

                    // Draw with thickness
                    for dy in -(line_thickness as isize)..=(line_thickness as isize) {
                        for dx in -(line_thickness as isize)..=(line_thickness as isize) {
                            let px = (x + dx) as usize;
                            let py = (y + dy) as usize;

                            if px < GRID_SIZE && py < GRID_SIZE {
                                grid.cells[py][px][3] = 1.0;
                                grid.cells[py][px][0] = 0.5;  // Purple-ish
                                grid.cells[py][px][2] = 0.8;
                            }
                        }
                    }
                }
            }
        }
    }

    grid
}

/// Calculate diversity metric (only alive cells)
pub fn calculate_diversity(nca: &NCA) -> f64 {
    let alive_threshold = 0.1;
    let mut alive_values = Vec::new();

    for row in &nca.grid.cells {
        for cell in row {
            let alpha = cell[3];
            if alpha > alive_threshold {
                alive_values.push(alpha);
            }
        }
    }

    if alive_values.len() > 1 {
        let mean = alive_values.iter().sum::<f64>() / alive_values.len() as f64;
        let variance = alive_values.iter()
            .map(|&v| (v - mean).powi(2))
            .sum::<f64>() / alive_values.len() as f64;
        variance.sqrt()
    } else {
        0.0
    }
}

/// Calculate complexity metric (spatial variation)
pub fn calculate_complexity(nca: &NCA) -> f64 {
    let mut total_diff = 0.0;
    let mut diff_count = 0;

    for y in 0..nca.grid.height {
        for x in 0..nca.grid.width {
            let val = nca.grid.cells[y][x][3];

            if x + 1 < nca.grid.width {
                total_diff += (val - nca.grid.cells[y][x + 1][3]).abs();
                diff_count += 1;
            }

            if y + 1 < nca.grid.height {
                total_diff += (val - nca.grid.cells[y + 1][x][3]).abs();
                diff_count += 1;
            }
        }
    }

    if diff_count > 0 { total_diff / diff_count as f64 } else { 0.0 }
}

/// Train NCA on target pattern using batch processing
/// Returns (average_loss, individual_batch_losses)
pub fn train_nca_batch(
    nca: &mut NCA,
    target: &Grid,
    learning_rate: f64,
    num_steps: usize,
    batch_size: usize,
) -> (f64, Vec<f64>) {
    let mut batch_losses = Vec::with_capacity(batch_size);

    for _ in 0..batch_size {
        // Reset and evolve
        nca.reset_with_seed();

        // Evolve NCA
        for _ in 0..num_steps {
            nca.step();
        }

        // Calculate loss
        let loss = nca.train_step(target, learning_rate);
        batch_losses.push(loss);
    }

    // Return average loss AND individual losses for progress tracking
    let avg_loss = batch_losses.iter().sum::<f64>() / batch_losses.len() as f64;
    (avg_loss, batch_losses)
}
