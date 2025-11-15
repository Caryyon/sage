// Unified Dashboard - All SAGE information in one comprehensive view

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
        HealthStatus::Healthy => ("⚡", Color::Green),
        HealthStatus::Warning => ("⚠️ ", Color::Yellow),
        HealthStatus::Critical => ("🔥", Color::Red),
    };

    let status_text = vec![
        Line::from(vec![
            Span::styled("🧬 SAGE ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled("Neural Cellular Automata", Style::default().fg(Color::LightCyan)),
            Span::raw("  │  "),
            Span::styled(format!("{} ", health_icon), Style::default().fg(health_color)),
            Span::styled(
                format!("{:?}", state.health_status),
                Style::default().fg(health_color),
            ),
            Span::raw("  │  "),
            Span::raw(format!("Gen: {}  ", state.training_state.nca_generation)),
            Span::styled(
                if state.is_paused { "⏸ Paused" } else { "🧬 Evolving" },
                Style::default().fg(if state.is_paused { Color::Yellow } else { Color::Green }),
            ),
        ]),
    ];

    let status_bar = Paragraph::new(status_text)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));

    frame.render_widget(status_bar, area);
}

fn render_main_content(frame: &mut Frame, area: Rect, state: &AppState) {
    // Split vertically: Neural field (left, maximized) | Metrics (right, compact)
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(72),  // Neural field - FULL WIDTH
            Constraint::Percentage(28),  // Metrics sidebar
        ])
        .split(area);

    render_neural_field(frame, main_chunks[0], state);

    // Right sidebar: Metrics (top) and Loss graph (bottom)
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(12),      // Metrics
            Constraint::Min(10),         // Loss graph (expanded)
        ])
        .split(main_chunks[1]);

    render_metrics(frame, right_chunks[0], state);
    render_loss_graph(frame, right_chunks[1], state);
}

fn render_neural_field(frame: &mut Frame, area: Rect, state: &AppState) {
    use crate::tui::app::TrainingMode;

    // CT SCAN MODE: Time-based pulsing for living neural activity
    let elapsed_ms = state.training_state.grid_update_time.elapsed().as_millis() as f64;
    let pulse_phase = (elapsed_ms / 300.0).sin(); // 300ms pulse cycle
    let sparkle_phase = (elapsed_ms / 150.0).sin(); // Faster sparkle cycle

    // Use REAL grid_snapshot data directly
    let grid = &state.training_state.grid_snapshot;

    let height = area.height.saturating_sub(4) as usize;
    let width = area.width.saturating_sub(4) as usize;
    let grid_size = grid.len();

    let step_y = grid_size as f64 / height as f64;
    let step_x = grid_size as f64 / (width / 2) as f64;

    let mut lines = vec![];

    // Title with scan indicator
    let scan_indicator = if state.training_mode == TrainingMode::BaselineTraining {
        "⚡ SCANNING"
    } else {
        "IDLE"
    };

    lines.push(Line::from(vec![
        Span::styled("🧠 NEURAL CT SCAN  ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::styled(scan_indicator, Style::default().fg(if state.training_mode == TrainingMode::BaselineTraining { Color::Green } else { Color::DarkGray })),
    ]));
    lines.push(Line::from(""));

    // Render neural field with CT scan effects
    for row_idx in 0..height {
        let y = ((row_idx as f64 * step_y) as usize).min(grid_size - 1);
        let mut row_spans = vec![];

        for col_idx in 0..(width / 2) {
            let x = ((col_idx as f64 * step_x) as usize).min(grid[y].len() - 1);
            let raw_value = grid[y][x];

            // Apply CT scan effects: pulsing + spatial variation
            let spatial_offset = ((x as f64 * 0.1 + y as f64 * 0.1).sin() + 1.0) * 0.5;
            let pulse_boost = if raw_value > 0.4 {
                1.0 + (pulse_phase * 0.15)  // High activity cells pulse
            } else {
                1.0
            };
            let sparkle = if raw_value > 0.7 && sparkle_phase > 0.7 {
                0.1  // Add sparkle to very active cells
            } else {
                0.0
            };

            let value = (raw_value * pulse_boost + sparkle + spatial_offset * 0.05).min(1.0);
            let (char, color) = value_to_ct_scan_char(value, state.training_state.current_loss);

            row_spans.push(Span::styled(char, Style::default().fg(color)));
            row_spans.push(Span::styled(char, Style::default().fg(color)));
        }

        lines.push(Line::from(row_spans));
    }

    let field = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(field, area);
}

fn render_metrics(frame: &mut Frame, area: Rect, state: &AppState) {
    let metrics_text = vec![
        Line::from(vec![
            Span::styled("📊 METRICS", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
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
                Style::default().fg(
                    if state.training_state.current_loss < 0.05 { Color::Green }
                    else if state.training_state.current_loss < 0.2 { Color::Yellow }
                    else { Color::Red }
                )
            ),
        ]),
        // 🎨 Batch Progress Bar
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
                    10
                ),
                Style::default().fg(Color::Green)
            ),
        ]),
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
            Span::raw("Learning:    "),
            Span::styled(
                &state.training_state.nca_current_pattern,
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            ),
        ]),
    ];

    let metrics = Paragraph::new(metrics_text)
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(metrics, area);
}

fn render_loss_graph(frame: &mut Frame, area: Rect, state: &AppState) {
    let loss = state.training_state.current_loss;
    let gen = state.training_state.nca_generation;

    // Simple ASCII-style loss visualization
    let bar_width = area.width.saturating_sub(4) as usize;
    let bar_filled = ((1.0 - loss.min(1.0)) * bar_width as f64) as usize;

    let loss_color = if loss < 0.05 {
        Color::Green
    } else if loss < 0.15 {
        Color::Yellow
    } else {
        Color::Red
    };

    let mut lines = vec![
        Line::from(vec![
            Span::styled("📉 TRAINING PROGRESS", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw("Loss: "),
            Span::styled(format!("{:.4}", loss), Style::default().fg(loss_color).add_modifier(Modifier::BOLD)),
            Span::raw(format!("  Target: < 0.05  Gen: {}", gen)),
        ]),
        Line::from(""),
    ];

    // Progress bar
    let bar_chars: Vec<Span> = (0..bar_width).map(|i| {
        if i < bar_filled {
            Span::styled("█", Style::default().fg(Color::Green))
        } else {
            Span::styled("░", Style::default().fg(Color::DarkGray))
        }
    }).collect();
    lines.push(Line::from(bar_chars));

    // Milestones
    lines.push(Line::from(""));
    let milestone_text = if loss < 0.05 {
        "🎯 Mastery level!"
    } else if loss < 0.08 {
        "✨ Almost there!"
    } else if loss < 0.15 {
        "📈 Making progress"
    } else {
        "🔍 Exploring patterns"
    };
    lines.push(Line::from(vec![
        Span::styled(milestone_text, Style::default().fg(if loss < 0.05 { Color::Green } else { Color::Cyan })),
    ]));

    let graph = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(graph, area);
}

#[allow(dead_code)]
fn render_chat(frame: &mut Frame, area: Rect, state: &AppState) {
    // Chat history with events at bottom
    let mut chat_text = vec![
        Line::from(vec![
            Span::styled("💬 SAGE'S THOUGHTS", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
    ];

    if state.training_state.chat_history.is_empty() {
        chat_text.push(Line::from(vec![
            Span::styled("  Use ./sage-speak in terminal to talk with SAGE", Style::default().fg(Color::DarkGray)),
        ]));
    } else {
        // Show last messages that fit
        let display_count = (area.height.saturating_sub(5) as usize).min(15);
        for (sender, message) in state.training_state.chat_history.iter().rev().take(display_count).rev() {
            let (color, prefix) = if sender == "SAGE" {
                (Color::Green, "🧬 ")
            } else {
                (Color::Cyan, "💭 ")
            };

            chat_text.push(Line::from(vec![
                Span::styled(format!("{}", prefix), Style::default().fg(color)),
                Span::styled(format!("{}: ", sender), Style::default().fg(color).add_modifier(Modifier::BOLD)),
                Span::raw(message.clone()),
            ]));
            chat_text.push(Line::from(""));
        }
    }

    // Add latest event at bottom
    chat_text.push(Line::from(""));
    chat_text.push(Line::from(vec![
        Span::styled("Latest: ", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
        Span::raw(
            state.training_state.recent_events.last()
                .map(|s| s.as_str())
                .unwrap_or("No events yet")
        ),
    ]));

    let chat = Paragraph::new(chat_text)
        .block(Block::default().borders(Borders::ALL))
        .wrap(ratatui::widgets::Wrap { trim: false });
    frame.render_widget(chat, area);
}

fn render_footer(frame: &mut Frame, area: Rect, _state: &AppState) {
    let footer_text = vec![
        Line::from(vec![
            Span::styled("[Tab] ", Style::default().fg(Color::Yellow)),
            Span::raw("Database Monitor  "),
            Span::styled("[Space] ", Style::default().fg(Color::Yellow)),
            Span::raw("Pause/Resume  "),
            Span::styled("[N] ", Style::default().fg(Color::Yellow)),
            Span::raw("Start Training  "),
            Span::styled("[Q] ", Style::default().fg(Color::Yellow)),
            Span::raw("Quit"),
        ]),
    ];

    let footer = Paragraph::new(footer_text)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));

    frame.render_widget(footer, area);
}

/// CT Scan visualization: Color-coded neural activity with understanding feedback
/// - Low loss (< 0.05): Green/Cyan = "SAGE understands this pattern"
/// - Medium loss (0.05-0.2): Yellow/Magenta = "SAGE is learning"
/// - High loss (> 0.2): Red/Blue = "SAGE is confused/exploring"
fn value_to_ct_scan_char(value: f64, loss: f64) -> (&'static str, Color) {
    // Intensity levels with CT scan characters
    let char = if value > 0.8 {
        "█"  // Maximum activity
    } else if value > 0.6 {
        "▓"  // High activity
    } else if value > 0.4 {
        "▒"  // Medium activity
    } else if value > 0.2 {
        "░"  // Low activity
    } else if value > 0.05 {
        "·"  // Trace activity
    } else {
        " "  // No activity
    };

    // Color based on understanding (loss) and intensity (value)
    let color = if loss < 0.05 {
        // MASTERY: Green spectrum (SAGE understands)
        if value > 0.6 {
            Color::Green
        } else if value > 0.3 {
            Color::LightGreen
        } else {
            Color::Cyan
        }
    } else if loss < 0.15 {
        // LEARNING: Yellow/Magenta spectrum (SAGE is learning)
        if value > 0.6 {
            Color::Yellow
        } else if value > 0.3 {
            Color::LightYellow
        } else {
            Color::Magenta
        }
    } else {
        // EXPLORING: Red/Blue spectrum (SAGE is confused/exploring)
        if value > 0.6 {
            Color::Red
        } else if value > 0.3 {
            Color::LightRed
        } else {
            Color::Blue
        }
    };

    (char, color)
}

// Legacy function - keeping for reference
#[allow(dead_code)]
fn value_to_neural_char(value: f64) -> (&'static str, Color) {
    if value > 0.8 {
        ("█", Color::Red)
    } else if value > 0.6 {
        ("▓", Color::LightRed)
    } else if value > 0.4 {
        ("▒", Color::Yellow)
    } else if value > 0.2 {
        ("░", Color::Blue)
    } else if value > 0.05 {
        ("·", Color::Cyan)
    } else {
        (" ", Color::DarkGray)
    }
}

fn render_progress_bar(current: usize, total: usize, width: usize) -> String {
    if total == 0 {
        return "░".repeat(width);
    }

    let filled = ((current as f64 / total as f64) * width as f64) as usize;
    let filled = filled.min(width);

    let filled_chars = "█".repeat(filled);
    let empty_chars = "░".repeat(width.saturating_sub(filled));

    format!("{}{}", filled_chars, empty_chars)
}

#[allow(dead_code)]
fn generate_demo_pattern() -> Vec<Vec<f64>> {
    let size = 32;
    let mut grid = vec![vec![0.0; size]; size];
    let center_x = size as f64 / 2.0;
    let center_y = size as f64 / 2.0;

    for y in 0..size {
        for x in 0..size {
            let dx = x as f64 - center_x;
            let dy = y as f64 - center_y;
            let dist = (dx * dx + dy * dy).sqrt();
            let angle = dy.atan2(dx);

            let wave1 = (dist * 0.3 + angle * 2.0).sin();
            let wave2 = (dist * 0.15 - angle * 1.5).cos();
            let radial = (-dist * dist / 200.0).exp();

            grid[y][x] = (wave1 * 0.4 + wave2 * 0.3 + radial * 0.5)
                .max(0.0)
                .min(1.0);
        }
    }

    grid
}
