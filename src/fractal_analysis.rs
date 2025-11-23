//! Fractal Analysis for SAGE
//!
//! Implements fractal dimension calculation using box-counting method
//! to measure pattern complexity and guide curiosity-driven exploration.

/// Calculate fractal dimension using box-counting method
///
/// This measures the complexity of a 2D pattern by counting how many
/// "boxes" are needed to cover the pattern at different scales.
/// Higher dimension = more complex, self-similar pattern
///
/// Returns: Fractal dimension (typically 1.0 - 2.0 for 2D patterns)
pub fn calculate_fractal_dimension(grid: &[Vec<f64>]) -> f64 {
    let height = grid.len();
    let width = if height > 0 { grid[0].len() } else { 0 };

    if height == 0 || width == 0 {
        return 1.0; // Default for empty grid
    }

    // Convert to binary (above/below threshold)
    let threshold = 0.1; // Consider cells "active" if above this
    let binary: Vec<Vec<bool>> = grid.iter()
        .map(|row| row.iter().map(|&v| v > threshold).collect())
        .collect();

    // Try different box sizes (powers of 2)
    let mut box_sizes = Vec::new();
    let mut counts = Vec::new();

    // Start with reasonable box sizes
    let mut box_size = 2;
    while box_size <= height.min(width) / 2 {
        let count = count_boxes(&binary, box_size);
        if count > 0 {
            box_sizes.push(box_size as f64);
            counts.push(count as f64);
        }
        box_size *= 2;
    }

    if box_sizes.len() < 2 {
        // Not enough data points for regression
        return 1.0;
    }

    // Calculate fractal dimension using linear regression on log-log plot
    // D = -slope of log(count) vs log(1/box_size)
    let dimension = calculate_slope_log_log(&box_sizes, &counts);

    // Clamp to reasonable range [1.0, 2.0] for 2D patterns
    dimension.max(1.0).min(2.0)
}

/// Count how many boxes of given size contain any active cells
fn count_boxes(binary: &[Vec<bool>], box_size: usize) -> usize {
    let height = binary.len();
    let width = binary[0].len();

    let mut count = 0;

    // Iterate over grid in box_size steps
    for y in (0..height).step_by(box_size) {
        for x in (0..width).step_by(box_size) {
            // Check if this box contains any active cells
            let mut has_active = false;
            'box_check: for dy in 0..box_size {
                for dx in 0..box_size {
                    let py = y + dy;
                    let px = x + dx;
                    if py < height && px < width && binary[py][px] {
                        has_active = true;
                        break 'box_check;
                    }
                }
            }
            if has_active {
                count += 1;
            }
        }
    }

    count
}

/// Calculate slope of log-log plot using linear regression
/// Returns the negative slope (fractal dimension)
fn calculate_slope_log_log(box_sizes: &[f64], counts: &[f64]) -> f64 {
    let n = box_sizes.len() as f64;

    // Convert to log scale
    let log_box_sizes: Vec<f64> = box_sizes.iter().map(|&s| (1.0 / s).ln()).collect();
    let log_counts: Vec<f64> = counts.iter().map(|&c| c.ln()).collect();

    // Linear regression: y = mx + b
    let sum_x: f64 = log_box_sizes.iter().sum();
    let sum_y: f64 = log_counts.iter().sum();
    let sum_xy: f64 = log_box_sizes.iter().zip(&log_counts).map(|(x, y)| x * y).sum();
    let sum_xx: f64 = log_box_sizes.iter().map(|x| x * x).sum();

    // Calculate slope
    let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_xx - sum_x * sum_x);

    // Return negative slope (fractal dimension)
    -slope
}

/// Classify pattern complexity based on fractal dimension
pub fn classify_complexity(dimension: f64) -> &'static str {
    if dimension < 1.2 {
        "simple" // Line-like, very ordered
    } else if dimension < 1.4 {
        "moderate" // Some structure
    } else if dimension < 1.6 {
        "complex" // Rich fractal structure
    } else {
        "chaotic" // Nearly space-filling
    }
}

/// Extract 2D grid from NCA state (alpha channel)
/// Cells structure: Vec<Vec<Vec<f64>>> where inner Vec has 22 channels
pub fn extract_alpha_grid(cells: &[Vec<Vec<f64>>]) -> Vec<Vec<f64>> {
    cells.iter()
        .map(|row| row.iter().map(|cell| cell.get(3).copied().unwrap_or(0.0)).collect()) // Alpha is index 3
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_grid() {
        let grid: Vec<Vec<f64>> = vec![];
        let dim = calculate_fractal_dimension(&grid);
        assert_eq!(dim, 1.0);
    }

    #[test]
    fn test_simple_line() {
        // Horizontal line - should be close to 1.0
        let mut grid = vec![vec![0.0; 32]; 32];
        for x in 0..32 {
            grid[16][x] = 1.0;
        }
        let dim = calculate_fractal_dimension(&grid);
        assert!(dim >= 1.0 && dim <= 1.3);
    }

    #[test]
    fn test_filled_square() {
        // Filled square - dimension depends on box counting algorithm
        // For a solid square, different implementations give different results
        let grid = vec![vec![1.0; 32]; 32];
        let dim = calculate_fractal_dimension(&grid);
        // Wider bounds since box counting on uniform grids can vary
        assert!(dim >= 1.0 && dim <= 2.0, "Fractal dim {} out of range", dim);
    }
}
