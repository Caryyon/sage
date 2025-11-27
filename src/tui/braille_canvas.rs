//! Braille Canvas - High-resolution grid rendering with Unicode Braille characters
//!
//! Converts 2D grids to Braille characters for beautiful terminal visualization.
//! Each Braille character represents a 2×4 pixel block, giving 4× higher resolution
//! than traditional block characters.
//!
//! Supports color highlighting based on cell activity/intensity.

use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
};
use crate::grid::Grid;

/// Convert a grid to colored Braille lines with activity highlighting
///
/// # Arguments
/// * `grid` - The grid to render (typically 32×32)
/// * `activity_map` - Optional activity values for color highlighting (0.0 = dead, 1.0 = max)
/// * `pulse_phase` - Optional animation phase for pulsing effect
///
/// # Returns
/// Vector of Lines suitable for rendering in ratatui
pub fn grid_to_braille_colored(
    grid: &Grid,
    activity_map: Option<&[f64]>,
    pulse_phase: Option<f64>,
) -> Vec<Line<'static>> {
    let height = grid.cells.len();
    let width = if height > 0 { grid.cells[0].len() } else { 0 };

    if height == 0 || width == 0 {
        return vec![Line::from("[No data]")];
    }

    let mut lines = Vec::new();

    // Process grid in 4-row chunks (each Braille char is 4 pixels tall)
    for row in (0..height).step_by(4) {
        let mut spans = Vec::new();

        // Process each pair of columns (each Braille char is 2 pixels wide)
        for col in (0..width).step_by(2) {
            let (braille, avg_activity) = encode_braille_char_colored(
                grid,
                row,
                col,
                activity_map,
            );

            // Choose color based on activity level
            let color = if let Some(phase) = pulse_phase {
                activity_to_color_animated(avg_activity, phase)
            } else {
                activity_to_color(avg_activity)
            };

            spans.push(Span::styled(
                braille.to_string(),
                Style::default().fg(color),
            ));
        }

        lines.push(Line::from(spans));
    }

    lines
}

/// Encode a 2×4 pixel block as a Braille character with color info
///
/// Returns: (braille_char, average_activity)
fn encode_braille_char_colored(
    grid: &Grid,
    y: usize,
    x: usize,
    activity_map: Option<&[f64]>,
) -> (char, f64) {
    let height = grid.cells.len();
    let width = grid.cells[0].len();

    // Braille Unicode: U+2800 + dot pattern
    let positions = [
        (0, 0, 0x01), // Dot 1
        (1, 0, 0x02), // Dot 2
        (2, 0, 0x04), // Dot 3
        (3, 0, 0x40), // Dot 7
        (0, 1, 0x08), // Dot 4
        (1, 1, 0x10), // Dot 5
        (2, 1, 0x20), // Dot 6
        (3, 1, 0x80), // Dot 8
    ];

    let mut dots = 0u8;
    let mut total_activity = 0.0;
    let mut pixel_count = 0;

    for (dy, dx, bit) in positions {
        let py = y + dy;
        let px = x + dx;

        if py < height && px < width {
            // Use alpha channel to determine if pixel is "on"
            let alpha = grid.cells[py][px][3];

            if alpha > 0.3 {
                dots |= bit;
            }

            // Collect activity if available
            if let Some(activities) = activity_map {
                let idx = py * width + px;
                if idx < activities.len() {
                    total_activity += activities[idx];
                    pixel_count += 1;
                }
            } else {
                // Use alpha as activity proxy
                total_activity += alpha;
                pixel_count += 1;
            }
        }
    }

    let avg_activity = if pixel_count > 0 {
        total_activity / pixel_count as f64
    } else {
        0.0
    };

    let braille = char::from_u32(0x2800 + dots as u32).unwrap_or('?');

    (braille, avg_activity)
}

/// Map activity level to color (static)
fn activity_to_color(activity: f64) -> Color {
    match activity {
        a if a < 0.1 => Color::Rgb(0, 60, 0),       // Dark green (dead/dormant)
        a if a < 0.3 => Color::Rgb(0, 120, 30),     // Medium green (low activity)
        a if a < 0.5 => Color::Rgb(0, 255, 0),      // Bright green (moderate activity)
        a if a < 0.7 => Color::Rgb(100, 255, 50),   // Yellow-green (high activity)
        a if a < 0.85 => Color::Rgb(255, 200, 0),   // Orange (firing)
        _ => Color::Rgb(255, 50, 0),                // Red (maximum activity)
    }
}

/// Map activity level to color with animation pulse
fn activity_to_color_animated(activity: f64, phase: f64) -> Color {
    let base_color = activity_to_color(activity);

    // Add subtle pulse to active cells
    if activity > 0.3 {
        let pulse = (phase.sin() * 0.5 + 0.5) * 30.0; // 0-30 brightness boost

        match base_color {
            Color::Rgb(r, g, b) => {
                let r = (r as f64 + pulse).min(255.0) as u8;
                let g = (g as f64 + pulse).min(255.0) as u8;
                let b = (b as f64 + pulse).min(255.0) as u8;
                Color::Rgb(r, g, b)
            }
            other => other,
        }
    } else {
        base_color
    }
}

/// Create activity legend for display
pub fn create_activity_legend() -> Line<'static> {
    Line::from(vec![
        Span::styled("⠀", Style::default().fg(Color::Rgb(0, 60, 0))),
        Span::styled(" Dead ", Style::default().fg(Color::DarkGray)),
        Span::styled("⣿", Style::default().fg(Color::Rgb(0, 120, 30))),
        Span::styled(" Low ", Style::default().fg(Color::DarkGray)),
        Span::styled("⣿", Style::default().fg(Color::Rgb(0, 255, 0))),
        Span::styled(" Active ", Style::default().fg(Color::DarkGray)),
        Span::styled("⣿", Style::default().fg(Color::Rgb(255, 200, 0))),
        Span::styled(" Firing ", Style::default().fg(Color::DarkGray)),
        Span::styled("⣿", Style::default().fg(Color::Rgb(255, 50, 0))),
        Span::styled(" Max", Style::default().fg(Color::DarkGray)),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_braille_encoding() {
        let mut grid = Grid::new(4, 2);

        // Create a pattern: all on
        for y in 0..4 {
            for x in 0..2 {
                grid.cells[y][x][3] = 1.0; // Alpha
            }
        }

        let (braille, activity) = encode_braille_char_colored(&grid, 0, 0, None);
        assert_eq!(braille, '⣿'); // All 8 dots on
        assert!(activity > 0.9); // High activity
    }

    #[test]
    fn test_activity_colors() {
        assert_eq!(activity_to_color(0.0), Color::Rgb(0, 60, 0));   // Dead
        assert_eq!(activity_to_color(0.5), Color::Rgb(0, 255, 0));  // Active
        assert_eq!(activity_to_color(1.0), Color::Rgb(255, 50, 0)); // Max
    }

    #[test]
    fn test_colored_rendering() {
        let grid = Grid::new(8, 8);
        let lines = grid_to_braille_colored(&grid, None, None);
        assert_eq!(lines.len(), 2); // 8 rows / 4 = 2 Braille lines
    }
}
