//! Vision Monitor - High-resolution Braille rendering of SAGE's visual perception
//!
//! This screen shows:
//! - Live camera feed in Braille characters (high-res)
//! - Learned visual patterns
//! - Visual memory recall
//! - Feature extraction visualization

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::tui::app::AppState;
use crate::tui::screens::ScreenTrait;

/// Vision Monitor screen - dedicated view for SAGE's visual perception
pub struct VisionMonitor {
    // Future: camera feed, visual memory, etc.
}

impl VisionMonitor {
    pub fn new() -> Self {
        Self {}
    }
}

impl ScreenTrait for VisionMonitor {
    fn render(&self, frame: &mut Frame, area: Rect, state: &AppState) {
        // Main layout: Top (camera feed) and Bottom (stats)
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(20),      // Camera feed (Braille)
                Constraint::Length(8),    // Visual metrics
            ])
            .split(area);

        // Render camera feed in Braille
        self.render_camera_braille(frame, chunks[0], state);

        // Render visual learning metrics
        self.render_visual_metrics(frame, chunks[1], state);
    }
}

impl VisionMonitor {
    /// Render camera feed using high-resolution Braille characters
    fn render_camera_braille(
        &self,
        f: &mut Frame,
        area: Rect,
        state: &AppState,
    ) {
        // Access the NCA grid from SAGE's neural substrate
        let grid = state.sage.get_grid();

        // Check if grid is mostly empty (< 5% cells active)
        let active_count: usize = grid.cells.iter()
            .flatten()
            .filter(|cell| cell[3] > 0.3)
            .count();
        let total_cells = grid.cells.len() * grid.cells[0].len();
        let is_mostly_empty = (active_count as f64 / total_cells as f64) < 0.05;

        // If empty, create a demo pattern to showcase Braille rendering
        let demo_grid;
        let display_grid = if is_mostly_empty {
            demo_grid = create_demo_pattern();
            &demo_grid
        } else {
            grid
        };

        // Convert grid to Braille
        let braille_lines = grid_to_braille(display_grid);

        // Create colored text with visual features
        let mut text_lines = Vec::new();

        // Header
        let header_suffix = if is_mostly_empty {
            " [Demo Pattern]"
        } else {
            " [Live NCA]"
        };

        text_lines.push(Line::from(vec![
            Span::styled("👁️  SAGE Vision Feed ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(
                format!("({}×{} → Braille)", display_grid.cells.len(), display_grid.cells[0].len()),
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(
                header_suffix,
                Style::default().fg(if is_mostly_empty { Color::Yellow } else { Color::Green }),
            ),
        ]));
        text_lines.push(Line::from(""));

        // Braille rendering
        for line in braille_lines {
            text_lines.push(Line::from(Span::styled(
                line,
                Style::default().fg(Color::White),
            )));
        }

        let paragraph = Paragraph::new(text_lines)
            .block(
                Block::default()
                    .title("Camera Feed (High-Res Braille)")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan)),
            )
            .wrap(Wrap { trim: true });

        f.render_widget(paragraph, area);
    }

    /// Render visual learning metrics
    fn render_visual_metrics(
        &self,
        f: &mut Frame,
        area: Rect,
        _state: &AppState,
    ) {
        let text = vec![
            Line::from(vec![
                Span::styled("Visual Learning Progress", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::raw("  Pattern Recognition: "),
                Span::styled(
                    format!("{:.1}%", 0.0), // TODO: Wire up actual metrics
                    Style::default().fg(Color::Green),
                ),
            ]),
            Line::from(vec![
                Span::raw("  Visual Memory: "),
                Span::styled("0 patterns stored", Style::default().fg(Color::Cyan)),
            ]),
            Line::from(vec![
                Span::raw("  Feature Extraction: "),
                Span::styled("Enabled", Style::default().fg(Color::Green)),
            ]),
        ];

        let paragraph = Paragraph::new(text)
            .block(
                Block::default()
                    .title("Visual Metrics")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Yellow)),
            );

        f.render_widget(paragraph, area);
    }
}

impl Default for VisionMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a demo pattern to showcase Braille rendering
fn create_demo_pattern() -> crate::grid::Grid {
    use crate::grid::Grid;

    let mut grid = Grid::new(32, 32);

    // Create a circle pattern in the center
    let center_x = 16.0;
    let center_y = 16.0;
    let radius = 10.0;

    for y in 0..32 {
        for x in 0..32 {
            let dx = x as f64 - center_x;
            let dy = y as f64 - center_y;
            let distance = (dx * dx + dy * dy).sqrt();

            if distance < radius {
                // Inside circle - set alpha channel
                let intensity = 1.0 - (distance / radius).powi(2);
                grid.cells[y][x][3] = intensity;  // Alpha
            }
        }
    }

    grid
}

/// Convert a 32×32 grid to Braille characters (16×8 output)
///
/// Each Braille character represents a 2×4 pixel block:
/// ```
/// Dot pattern:
/// 1 4
/// 2 5
/// 3 6
/// 7 8
/// ```
fn grid_to_braille(grid: &crate::grid::Grid) -> Vec<String> {
    let height = grid.cells.len();
    let width = if height > 0 { grid.cells[0].len() } else { 0 };

    if height == 0 || width == 0 {
        return vec!["[No visual data]".to_string()];
    }

    let mut lines = Vec::new();

    // Process grid in 4-row chunks (each Braille char is 4 pixels tall)
    for row in (0..height).step_by(4) {
        let mut line = String::new();

        // Process each pair of columns (each Braille char is 2 pixels wide)
        for col in (0..width).step_by(2) {
            let braille = encode_braille_char(grid, row, col);
            line.push(braille);
        }

        lines.push(line);
    }

    lines
}

/// Encode a 2×4 pixel block as a single Braille character
fn encode_braille_char(grid: &crate::grid::Grid, y: usize, x: usize) -> char {
    let height = grid.cells.len();
    let width = grid.cells[0].len();

    // Braille Unicode: U+2800 + dot pattern
    // Dot positions and their bit values:
    // Position (dy, dx) -> Bit
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

    for (dy, dx, bit) in positions {
        let py = y + dy;
        let px = x + dx;

        if py < height && px < width {
            // Use alpha channel to determine if pixel is "on"
            let alpha = grid.cells[py][px][3];

            // Threshold: consider pixel visible if alpha > 0.3
            if alpha > 0.3 {
                dots |= bit;
            }
        }
    }

    // Convert to Braille Unicode character
    char::from_u32(0x2800 + dots as u32).unwrap_or('?')
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::Grid;

    #[test]
    fn test_grid_to_braille_empty() {
        let grid = Grid::new(32, 32);
        let braille = grid_to_braille(&grid);
        assert_eq!(braille.len(), 8); // 32 rows / 4 = 8 lines
    }

    #[test]
    fn test_encode_braille_all_on() {
        let mut grid = Grid::new(32, 32);

        // Set a 2×4 block to all white (alpha = 1.0)
        for y in 0..4 {
            for x in 0..2 {
                grid.cells[y][x][3] = 1.0; // Alpha channel
            }
        }

        let braille = encode_braille_char(&grid, 0, 0);
        // All 8 dots on = 0xFF = '⣿'
        assert_eq!(braille, '⣿');
    }

    #[test]
    fn test_encode_braille_all_off() {
        let grid = Grid::new(32, 32);
        let braille = encode_braille_char(&grid, 0, 0);
        // All dots off = 0x00 = '⠀' (blank Braille)
        assert_eq!(braille, '⠀');
    }

    #[test]
    fn test_encode_braille_pattern() {
        let mut grid = Grid::new(32, 32);

        // Create diagonal pattern:
        // 1 0
        // 0 1
        // 0 0
        // 0 0
        grid.cells[0][0][3] = 1.0; // Dot 1
        grid.cells[1][1][3] = 1.0; // Dot 5

        let braille = encode_braille_char(&grid, 0, 0);
        // Dot 1 (0x01) + Dot 5 (0x10) = 0x11 = '⠑'
        assert_eq!(braille, '⠑');
    }
}
