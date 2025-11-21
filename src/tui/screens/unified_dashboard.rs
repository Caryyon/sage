// Unified Dashboard - Visual Training Process Focus
// Shows how SAGE learns patterns through NCA evolution

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use crate::tui::app::{AppState, HealthStatus};
use super::ScreenTrait;

pub struct UnifiedDashboardScreen;

impl ScreenTrait for UnifiedDashboardScreen {
    fn render(&self, frame: &mut Frame, area: Rect, state: &AppState) {
        // Main layout: Top bar + Content area + Footer
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),   // Top status bar
                Constraint::Min(10),     // Main content area
                Constraint::Length(3),   // Footer controls
            ])
            .split(area);

        render_status_bar(frame, main_chunks[0], state);
        render_main_content(frame, main_chunks[1], state);
        render_footer(frame, main_chunks[2], state);
    }
}

fn render_status_bar(frame: &mut Frame, area: Rect, state: &AppState) {
    let (health_icon, health_color) = match state.health_status {
        HealthStatus::Healthy => ("OK", Color::Green),
        HealthStatus::Warning => ("!!", Color::Yellow),
        HealthStatus::Critical => ("XX", Color::Red),
    };

    // Performance mode color based on CPU usage
    let perf_color = match state.performance_mode {
        crate::tui::app::PerformanceMode::Eco => Color::Green,
        crate::tui::app::PerformanceMode::Low => Color::Cyan,
        crate::tui::app::PerformanceMode::Medium => Color::Yellow,
        crate::tui::app::PerformanceMode::High => Color::Magenta,
        crate::tui::app::PerformanceMode::Max => Color::Red,
    };

    let status_text = vec![
        Line::from(vec![
            Span::styled("SAGE NCA Training", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw("  |  "),
            Span::styled(format!("{} ", health_icon), Style::default().fg(health_color)),
            Span::styled(
                format!("{:?}", state.health_status),
                Style::default().fg(health_color),
            ),
            Span::raw("  |  "),
            Span::raw(format!("Gen: {}  ", state.training_state.nca_generation)),
            Span::styled(
                if state.is_paused { "[Paused]" } else { "[Training]" },
                Style::default().fg(if state.is_paused { Color::Yellow } else { Color::Green }),
            ),
            Span::raw("  |  "),
            Span::styled(
                format!("{}", state.performance_mode.label()),
                Style::default().fg(perf_color),
            ),
        ]),
    ];

    let status_bar = Paragraph::new(status_text)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));

    frame.render_widget(status_bar, area);
}

fn render_main_content(frame: &mut Frame, area: Rect, state: &AppState) {
    // Main layout: Pattern visualization (top half) | Metrics + diagnostics (bottom half)
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(50),  // Top: 3 pattern panels
            Constraint::Percentage(50),  // Bottom: metrics + diagnostics
        ])
        .split(area);

    render_pattern_comparison(frame, main_chunks[0], state);

    // Bottom half: Metrics (left) and Training diagnostics (right)
    let bottom_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50),  // Metrics
            Constraint::Percentage(50),  // Training diagnostics
        ])
        .split(main_chunks[1]);

    render_metrics(frame, bottom_chunks[0], state);
    render_training_diagnostics(frame, bottom_chunks[1], state);
}

fn render_pattern_comparison(frame: &mut Frame, area: Rect, state: &AppState) {
    // Split into three sections: Target | Current | Difference
    let sections = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(33),
            Constraint::Percentage(34),
        ])
        .split(area);

    render_pattern_panel(frame, sections[0], state, PatternView::Target);
    render_pattern_panel(frame, sections[1], state, PatternView::Current);
    render_pattern_panel(frame, sections[2], state, PatternView::Difference);
}

#[derive(Clone, Copy)]
enum PatternView {
    Target,
    Current,
    Difference,
}

fn render_pattern_panel(frame: &mut Frame, area: Rect, state: &AppState, view: PatternView) {
    let grid = &state.training_state.grid_snapshot;
    let grid_size = grid.len();

    let available_height = area.height.saturating_sub(4) as usize;
    let available_width = area.width.saturating_sub(4) as usize;

    // Terminal characters are ~2:1 (height:width), so adjust for square aspect ratio
    // We want width_chars = height_lines * 2 for a square visual
    let adjusted_width = available_height * 2;

    // Use the smaller dimension to ensure square display
    let display_size = available_width.min(adjusted_width);
    let height = (display_size / 2).max(1);  // Divide by 2 to account for char aspect ratio
    let width = display_size.max(1);

    // Calculate step size for downsampling
    let step_y = grid_size as f64 / height as f64;
    let step_x = grid_size as f64 / width as f64;

    let mut lines = vec![];

    // Title with color coding
    let (title, title_color) = match view {
        PatternView::Target => ("TARGET", Color::Yellow),
        PatternView::Current => ("CURRENT", Color::Cyan),
        PatternView::Difference => ("DIFFERENCE", Color::Magenta),
    };

    lines.push(Line::from(vec![
        Span::styled(title, Style::default().fg(title_color).add_modifier(Modifier::BOLD)),
    ]));
    lines.push(Line::from(""));

    // Render pattern grid
    for row_idx in 0..height {
        let y = ((row_idx as f64 * step_y) as usize).min(grid_size - 1);
        let mut row_spans = vec![];

        for col_idx in 0..width {
            let x = ((col_idx as f64 * step_x) as usize).min(grid[y].len() - 1);

            let value = match view {
                PatternView::Target => {
                    // Use actual target grid from AppState if available (PatternTraining mode)
                    if let Some(ref target_grid) = state.pattern_target_grid {
                        target_grid.cells[y][x][3]  // Use alpha channel
                    } else {
                        // Fall back to calculated target (BaselineTraining mode)
                        calculate_target_value(x, y, grid_size, &state.training_state.nca_current_pattern)
                    }
                },
                PatternView::Current => grid[y][x],
                PatternView::Difference => {
                    let target = if let Some(ref target_grid) = state.pattern_target_grid {
                        target_grid.cells[y][x][3]
                    } else {
                        calculate_target_value(x, y, grid_size, &state.training_state.nca_current_pattern)
                    };
                    (grid[y][x] - target).abs()
                },
            };

            let (char, color) = value_to_rich_char(value, view, state.training_state.current_loss);
            row_spans.push(Span::styled(format!("{}", char), Style::default().fg(color)));
        }

        lines.push(Line::from(row_spans));
    }

    let panel = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(panel, area);
}

fn render_metrics(frame: &mut Frame, area: Rect, state: &AppState) {
    let mut metrics_text = vec![
        Line::from(vec![
            Span::styled("TRAINING METRICS", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw("Generation:  "),
            Span::styled(
                format!("{}", state.training_state.nca_generation),
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
            ),
        ]),
        Line::from(vec![
            Span::raw("Loss:        "),
            Span::styled(
                format!("{:.4}", state.training_state.current_loss),
                Style::default().fg(loss_to_color(state.training_state.current_loss))
            ),
            Span::raw(format!("  {}", loss_to_indicator(state.training_state.current_loss))),
        ]),
        Line::from(vec![
            Span::raw("Batch:       "),
            Span::styled(
                format!("{}/{}",
                    state.training_state.current_batch,
                    state.training_state.total_batches
                ),
                Style::default().fg(Color::Cyan)
            ),
            Span::raw(" "),
            Span::styled(
                render_progress_bar(
                    state.training_state.current_batch,
                    state.training_state.total_batches,
                    8
                ),
                Style::default().fg(Color::Green)
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw("Diversity:   "),
            Span::styled(
                format!("{:.3}", state.training_state.nca_diversity),
                Style::default().fg(Color::Magenta)
            ),
        ]),
        Line::from(vec![
            Span::raw("Complexity:  "),
            Span::styled(
                format!("{:.3}", state.training_state.nca_complexity),
                Style::default().fg(Color::Blue)
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw("Pattern:     "),
            Span::styled(
                &state.training_state.nca_current_pattern,
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("PATTERN MASTERY", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
    ];

    // Add pattern mastery status
    for (i, (pattern_name, is_mastered, best_loss)) in state.pattern_mastery_status.iter().enumerate() {
        let is_current = i == state.pattern_current_index;
        let (status_icon, status_color) = if *is_mastered {
            ("*", Color::Green)
        } else if is_current {
            ("⟳", Color::Yellow)
        } else {
            ("o", Color::DarkGray)
        };

        metrics_text.push(Line::from(vec![
            Span::styled(format!("{} ", status_icon), Style::default().fg(status_color).add_modifier(Modifier::BOLD)),
            Span::raw(format!("{:<8}", pattern_name)),
            if *is_mastered {
                Span::styled("MASTERED", Style::default().fg(Color::Green))
            } else if is_current {
                Span::styled(format!("Training ({:.4})", best_loss), Style::default().fg(Color::Yellow))
            } else {
                Span::styled(format!("Best: {:.4}", best_loss), Style::default().fg(Color::DarkGray))
            },
        ]));
    }

    // Add summary
    let mastered_count = state.pattern_mastery_status.iter().filter(|(_, m, _)| *m).count();
    metrics_text.push(Line::from(""));
    metrics_text.push(Line::from(vec![
        Span::raw("Progress:    "),
        Span::styled(
            format!("{}/{} patterns mastered", mastered_count, state.pattern_mastery_status.len()),
            Style::default().fg(if mastered_count == state.pattern_mastery_status.len() {
                Color::Green
            } else {
                Color::Cyan
            }).add_modifier(Modifier::BOLD)
        ),
    ]));

    let metrics = Paragraph::new(metrics_text)
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(metrics, area);
}

fn render_training_diagnostics(frame: &mut Frame, area: Rect, state: &AppState) {
    let loss = state.training_state.current_loss;
    let gen = state.training_state.nca_generation;

    // Loss progress bar
    let bar_width = area.width.saturating_sub(4) as usize;
    let max_loss = 1.0;
    let bar_filled = ((1.0 - (loss.min(max_loss) / max_loss)) * bar_width as f64) as usize;

    let mut lines = vec![
        Line::from(vec![
            Span::styled("CONVERGENCE", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw(format!("Progress: {:.1}%  ", (1.0 - loss.min(1.0)) * 100.0)),
            Span::raw(format!("Gen: {}", gen)),
        ]),
        Line::from(""),
    ];

    // ASCII progress bar with gradient
    let bar_chars: Vec<Span> = (0..bar_width).map(|i| {
        if i < bar_filled {
            let intensity = i as f64 / bar_filled.max(1) as f64;
            let char = if intensity > 0.75 { "=" }
                      else if intensity > 0.5 { "=" }
                      else if intensity > 0.25 { "-" }
                      else { "-" };
            let color = if loss < 0.05 { Color::Green }
                       else if loss < 0.15 { Color::Yellow }
                       else { Color::Red };
            Span::styled(char, Style::default().fg(color))
        } else {
            Span::styled(".", Style::default().fg(Color::DarkGray))
        }
    }).collect();
    lines.push(Line::from(bar_chars));

    // Learning stage indicator
    lines.push(Line::from(""));
    let stage = learning_stage(loss);
    lines.push(Line::from(vec![
        Span::styled("Stage: ", Style::default().fg(Color::DarkGray)),
        Span::styled(stage, Style::default().fg(stage_color(loss))),
    ]));

    // Pattern understanding
    lines.push(Line::from(""));
    let understanding = pattern_understanding_bar(loss, bar_width.min(20));
    lines.push(Line::from(vec![
        Span::raw("Understanding: "),
        Span::raw(understanding),
    ]));

    // What this training does
    lines.push(Line::from(""));
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("WHAT THIS DOES", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
    ]));
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("->", Style::default().fg(Color::Yellow)),
        Span::raw(" Training NCA to self-organize"),
    ]));
    lines.push(Line::from(vec![
        Span::styled("->", Style::default().fg(Color::Yellow)),
        Span::raw(" Each cell learns local rules"),
    ]));
    lines.push(Line::from(vec![
        Span::styled("->", Style::default().fg(Color::Yellow)),
        Span::raw(" Emergent global patterns"),
    ]));
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("FUTURE USE", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
    ]));
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("•", Style::default().fg(Color::Green)),
        Span::raw(" Pattern memory foundation"),
    ]));
    lines.push(Line::from(vec![
        Span::styled("•", Style::default().fg(Color::Green)),
        Span::raw(" Distributed representations"),
    ]));
    lines.push(Line::from(vec![
        Span::styled("•", Style::default().fg(Color::Green)),
        Span::raw(" Self-repair & adaptation"),
    ]));

    let diagnostics = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(diagnostics, area);
}

fn render_footer(frame: &mut Frame, area: Rect, _state: &AppState) {
    let footer_text = vec![
        Line::from(vec![
            Span::styled("[Tab] ", Style::default().fg(Color::Yellow)),
            Span::raw("Database  "),
            Span::styled("[Space] ", Style::default().fg(Color::Yellow)),
            Span::raw("Pause  "),
            Span::styled("[P] ", Style::default().fg(Color::Yellow)),
            Span::raw("Perf  "),
            Span::styled("[N] ", Style::default().fg(Color::Yellow)),
            Span::raw("Train  "),
            Span::styled("[Q] ", Style::default().fg(Color::Yellow)),
            Span::raw("Quit"),
        ]),
    ];

    let footer = Paragraph::new(footer_text)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));

    frame.render_widget(footer, area);
}

/// Rich ASCII character set for fine gradations (no emojis)
fn value_to_rich_char(value: f64, view: PatternView, loss: f64) -> (char, Color) {
    let char = match view {
        PatternView::Difference => {
            // For difference view, show error magnitude
            if value > 0.5 { '#' }
            else if value > 0.3 { '@' }
            else if value > 0.15 { '%' }
            else if value > 0.08 { 'o' }
            else if value > 0.04 { '.' }
            else { ' ' }
        },
        _ => {
            // For target/current, show intensity
            if value > 0.85 { '#' }
            else if value > 0.70 { '@' }
            else if value > 0.55 { '&' }
            else if value > 0.40 { '*' }
            else if value > 0.30 { '+' }
            else if value > 0.20 { '=' }
            else if value > 0.12 { '-' }
            else if value > 0.06 { '.' }
            else { ' ' }
        }
    };

    // Color coding based on view type and learning progress
    let color = match view {
        PatternView::Target => {
            if value > 0.6 { Color::Yellow }
            else if value > 0.3 { Color::LightYellow }
            else { Color::DarkGray }
        },
        PatternView::Current => {
            if loss < 0.05 {
                // Mastery - green spectrum
                if value > 0.6 { Color::Green }
                else if value > 0.3 { Color::LightGreen }
                else { Color::Cyan }
            } else if loss < 0.15 {
                // Learning - cyan spectrum
                if value > 0.6 { Color::Cyan }
                else if value > 0.3 { Color::LightCyan }
                else { Color::Blue }
            } else {
                // Exploring - blue/gray spectrum
                if value > 0.6 { Color::Blue }
                else if value > 0.3 { Color::LightBlue }
                else { Color::DarkGray }
            }
        },
        PatternView::Difference => {
            // Error visualization
            if value > 0.3 { Color::Red }
            else if value > 0.15 { Color::LightRed }
            else if value > 0.08 { Color::Magenta }
            else if value > 0.04 { Color::LightMagenta }
            else { Color::DarkGray }
        },
    };

    (char, color)
}

/// Calculate target pattern value (simplified - should ideally come from training state)
fn calculate_target_value(x: usize, y: usize, grid_size: usize, pattern: &str) -> f64 {
    let cx = grid_size as f64 / 2.0;
    let cy = grid_size as f64 / 2.0;
    let dx = x as f64 - cx;
    let dy = y as f64 - cy;
    let dist = (dx * dx + dy * dy).sqrt();
    let radius = grid_size as f64 / 3.0;

    match pattern {
        s if s.contains("Circle") => {
            let circle = (radius - dist).max(0.0) / 5.0;
            circle.min(1.0)
        },
        s if s.contains("Square") => {
            let half_size = grid_size as f64 / 3.0;
            if dx.abs() < half_size && dy.abs() < half_size {
                1.0
            } else {
                0.0
            }
        },
        s if s.contains("Cross") => {
            let thickness = grid_size as f64 / 8.0;
            if dx.abs() < thickness || dy.abs() < thickness {
                1.0
            } else {
                0.0
            }
        },
        s if s.contains("Spiral") => {
            let angle = dy.atan2(dx);
            let spiral_dist = angle * radius / (2.0 * std::f64::consts::PI);
            let diff = (dist - spiral_dist.abs()).abs();
            (1.0 - diff / 5.0).max(0.0)
        },
        _ => 0.0,
    }
}

fn loss_to_color(loss: f64) -> Color {
    if loss < 0.05 { Color::Green }
    else if loss < 0.10 { Color::LightGreen }
    else if loss < 0.15 { Color::Yellow }
    else if loss < 0.25 { Color::LightRed }
    else { Color::Red }
}

fn loss_to_indicator(loss: f64) -> &'static str {
    if loss < 0.05 { "[MASTERED]" }
    else if loss < 0.10 { "[CONVERGING]" }
    else if loss < 0.15 { "[LEARNING]" }
    else if loss < 0.30 { "[EXPLORING]" }
    else { "[INITIALIZING]" }
}

fn learning_stage(loss: f64) -> &'static str {
    if loss < 0.05 { "Pattern Mastered" }
    else if loss < 0.10 { "Fine Tuning" }
    else if loss < 0.15 { "Rapid Learning" }
    else if loss < 0.30 { "Exploration" }
    else { "Initialization" }
}

fn stage_color(loss: f64) -> Color {
    if loss < 0.05 { Color::Green }
    else if loss < 0.15 { Color::Cyan }
    else if loss < 0.30 { Color::Yellow }
    else { Color::Red }
}

fn pattern_understanding_bar(loss: f64, width: usize) -> String {
    let filled = ((1.0 - loss.min(1.0)) * width as f64) as usize;
    let filled = filled.min(width);

    let mut bar = String::new();
    for i in 0..width {
        if i < filled {
            bar.push('=');
        } else {
            bar.push(' ');
        }
    }

    format!("[{}] {:.0}%", bar, (1.0 - loss.min(1.0)) * 100.0)
}

fn render_progress_bar(current: usize, total: usize, width: usize) -> String {
    if total == 0 {
        return ".".repeat(width);
    }

    let filled = ((current as f64 / total as f64) * width as f64) as usize;
    let filled = filled.min(width);

    let filled_chars = "=".repeat(filled);
    let empty_chars = ".".repeat(width.saturating_sub(filled));

    format!("{}{}", filled_chars, empty_chars)
}
