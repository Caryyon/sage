// Database Monitor - Meaningful ML research visualizations

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::{Line, Span},
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType, List, ListItem, Paragraph},
    Frame,
};
use crate::tui::app::AppState;
use super::ScreenTrait;
use std::process::Command;

pub struct DatabaseMonitorScreen;

impl ScreenTrait for DatabaseMonitorScreen {
    fn render(&self, frame: &mut Frame, area: Rect, state: &AppState) {
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),   // Header
                Constraint::Min(10),     // Content
                Constraint::Length(3),   // Footer
            ])
            .split(area);

        render_header(frame, main_chunks[0], state);
        render_content(frame, main_chunks[1], state);
        render_footer(frame, main_chunks[2]);
    }
}

fn render_header(frame: &mut Frame, area: Rect, state: &AppState) {
    let status_icon = if state.training_state.is_running { "[ON]" } else { "[PAUSED]" };
    let header_text = vec![
        Line::from(vec![
            Span::styled("", Style::default().fg(Color::Cyan)),
            Span::styled("Training Analytics", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw("  │  "),
            Span::styled(format!("{} Gen {}", status_icon, state.training_state.nca_generation), Style::default().fg(Color::Green)),
            Span::raw("  │  "),
            Span::styled(format!("Loss: {:.4}", state.training_state.current_loss),
                Style::default().fg(if state.training_state.current_loss < 0.1 { Color::Green } else { Color::Yellow })),
        ]),
    ];

    let header = Paragraph::new(header_text)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Cyan)));

    frame.render_widget(header, area);
}

fn render_content(frame: &mut Frame, area: Rect, state: &AppState) {
    // Responsive layout
    let is_tall = area.height >= 30;
    let is_wide = area.width >= 120;

    if is_tall && is_wide {
        render_full_layout(frame, area, state);
    } else if is_wide {
        render_wide_compact_layout(frame, area, state);
    } else {
        render_narrow_layout(frame, area, state);
    }
}

fn render_full_layout(frame: &mut Frame, area: Rect, state: &AppState) {
    // 2x3 grid: Loss curve, Learning Dynamics, Per-pattern losses, Stats, Hyperparameters, ETA
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(40),  // Top charts
            Constraint::Percentage(35),  // Middle charts
            Constraint::Percentage(25),  // Bottom stats
        ])
        .split(area);

    // Top row: Combined metrics chart (full width)
    // Middle row: Learning dynamics + Pattern mastery
    let middle_cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(rows[1]);

    let bottom_cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(34),
            Constraint::Percentage(33),
        ])
        .split(rows[2]);

    render_combined_metrics_chart(frame, rows[0], state);
    render_learning_dynamics_chart(frame, middle_cols[0], state);
    render_pattern_mastery_panel(frame, middle_cols[1], state);
    render_training_stats(frame, bottom_cols[0], state);
    render_hyperparameter_panel(frame, bottom_cols[1], state);
    render_eta_panel(frame, bottom_cols[2], state);
}

fn render_wide_compact_layout(frame: &mut Frame, area: Rect, state: &AppState) {
    // Two columns: main charts + stats
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(60),
            Constraint::Percentage(40),
        ])
        .split(area);

    let left_panel = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(cols[0]);

    render_loss_convergence_chart(frame, left_panel[0], state);
    render_learning_dynamics_chart(frame, left_panel[1], state);

    let right_panel = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(40),
            Constraint::Percentage(30),
            Constraint::Percentage(30),
        ])
        .split(cols[1]);

    render_training_stats(frame, right_panel[0], state);
    render_hyperparameter_panel(frame, right_panel[1], state);
    render_eta_panel(frame, right_panel[2], state);
}

fn render_narrow_layout(frame: &mut Frame, area: Rect, state: &AppState) {
    // Stacked: charts, then stats
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(40),
            Constraint::Percentage(30),
            Constraint::Percentage(30),
        ])
        .split(area);

    render_loss_convergence_chart(frame, chunks[0], state);
    render_learning_dynamics_chart(frame, chunks[1], state);
    render_training_stats(frame, chunks[2], state);
}

/// Combined chart showing Loss, Complexity, and Diversity on normalized scales
fn render_combined_metrics_chart(frame: &mut Frame, area: Rect, state: &AppState) {
    let metrics = &state.training_state.metrics_history;

    if metrics.is_empty() {
        let placeholder = Paragraph::new("Waiting for training data...")
            .block(Block::default()
                .title("Training Metrics")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)));
        frame.render_widget(placeholder, area);
        return;
    }

    // Use last 200 points for the chart
    let recent: Vec<&crate::tui::training::MetricSnapshot> = metrics.iter().rev().take(200).rev().collect();
    let len = recent.len();

    // Prepare data - normalize all values to 0-1 range for comparison
    let loss_data: Vec<(f64, f64)> = recent.iter()
        .enumerate()
        .map(|(i, m)| (i as f64, m.loss.min(1.0)))  // Loss already ~0-1
        .collect();

    let complexity_data: Vec<(f64, f64)> = recent.iter()
        .enumerate()
        .map(|(i, m)| (i as f64, m.complexity.min(1.0)))
        .collect();

    let diversity_data: Vec<(f64, f64)> = recent.iter()
        .enumerate()
        .map(|(i, m)| (i as f64, m.diversity.min(1.0)))
        .collect();

    // Get current values for legend
    let current_loss = recent.last().map(|m| m.loss).unwrap_or(0.0);
    let current_complexity = recent.last().map(|m| m.complexity).unwrap_or(0.0);
    let current_diversity = recent.last().map(|m| m.diversity).unwrap_or(0.0);

    let datasets = vec![
        Dataset::default()
            .name(format!("Loss: {:.4}", current_loss))
            .marker(symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(Color::Cyan))
            .data(&loss_data),
        Dataset::default()
            .name(format!("Complexity: {:.3}", current_complexity))
            .marker(symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(Color::Magenta))
            .data(&complexity_data),
        Dataset::default()
            .name(format!("Diversity: {:.3}", current_diversity))
            .marker(symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(Color::Yellow))
            .data(&diversity_data),
    ];

    let first_gen = recent.first().map(|m| m.generation).unwrap_or(0);
    let last_gen = recent.last().map(|m| m.generation).unwrap_or(0);

    let chart = Chart::new(datasets)
        .block(Block::default()
            .title(Line::from(vec![
                Span::styled("Training Metrics ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::styled(format!("(Gen {}-{}, {} pts)", first_gen, last_gen, len), Style::default().fg(Color::Gray)),
            ]))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)))
        .x_axis(
            Axis::default()
                .title("Generation")
                .style(Style::default().fg(Color::Gray))
                .bounds([0.0, (len.max(1) - 1) as f64])
                .labels(vec![
                    Span::raw(format!("{}", first_gen)),
                    Span::raw(format!("{}", (first_gen + last_gen) / 2)),
                    Span::raw(format!("{}", last_gen)),
                ]),
        )
        .y_axis(
            Axis::default()
                .title("Value")
                .style(Style::default().fg(Color::Gray))
                .bounds([0.0, 1.0])
                .labels(vec![
                    Span::styled("0.0", Style::default().fg(Color::Green)),
                    Span::styled("0.5", Style::default().fg(Color::Yellow)),
                    Span::styled("1.0", Style::default().fg(Color::Red)),
                ]),
        );

    frame.render_widget(chart, area);
}

/// Pattern mastery status panel
fn render_pattern_mastery_panel(frame: &mut Frame, area: Rect, state: &AppState) {
    let mastery = &state.pattern_mastery_status;

    let mut lines = vec![
        Line::from(vec![
            Span::styled("Pattern Mastery Status", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
    ];

    if mastery.is_empty() {
        lines.push(Line::from(Span::styled("No patterns yet...", Style::default().fg(Color::DarkGray))));
    } else {
        for (name, is_mastered, best_loss) in mastery {
            let (status, color) = if *is_mastered {
                ("*MASTERED", Color::Green)
            } else if *best_loss < 0.1 {
                ("Learning", Color::Yellow)
            } else {
                ("Pending", Color::DarkGray)
            };

            lines.push(Line::from(vec![
                Span::styled(format!("{:<12}", name), Style::default().fg(Color::White)),
                Span::styled(format!("{:<10}", status), Style::default().fg(color)),
                Span::styled(format!("Best: {:.4}", best_loss), Style::default().fg(Color::Gray)),
            ]));
        }

        // Summary
        let mastered_count = mastery.iter().filter(|(_, m, _)| *m).count();
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("Progress: ", Style::default().fg(Color::Cyan)),
            Span::styled(format!("{}/{} patterns mastered", mastered_count, mastery.len()),
                Style::default().fg(if mastered_count == mastery.len() { Color::Green } else { Color::Yellow })),
        ]));
    }

    let panel = Paragraph::new(lines)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)));
    frame.render_widget(panel, area);
}

fn render_loss_convergence_chart(frame: &mut Frame, area: Rect, state: &AppState) {
    let metrics = &state.training_state.metrics_history;

    if metrics.is_empty() {
        let placeholder = Paragraph::new("No training data yet...")
            .block(Block::default()
                .title("Loss Convergence")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)));
        frame.render_widget(placeholder, area);
        return;
    }

    // Use last 50 metrics
    let recent_metrics: Vec<&crate::tui::training::MetricSnapshot> = metrics.iter().rev().take(50).rev().collect();

    // Prepare data points for chart
    let data: Vec<(f64, f64)> = recent_metrics.iter()
        .enumerate()
        .map(|(i, m)| (i as f64, m.loss))
        .collect();

    // Calculate bounds
    let max_loss = recent_metrics.iter().map(|m| m.loss).fold(0.0, f64::max).max(0.1);
    let min_loss = recent_metrics.iter().map(|m| m.loss).fold(1.0, f64::min);

    let datasets = vec![
        Dataset::default()
            .name("Loss")
            .marker(symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(Color::Cyan))
            .data(&data),
    ];

    let x_labels = vec![
        Span::raw(format!("{}", metrics.first().map(|m| m.generation).unwrap_or(0))),
        Span::raw(format!("{}", metrics.len() / 2)),
        Span::raw(format!("{}", metrics.last().map(|m| m.generation).unwrap_or(0))),
    ];

    let y_labels = vec![
        Span::styled(format!("{:.3}", min_loss), Style::default().fg(Color::Green)),
        Span::styled(format!("{:.3}", (max_loss + min_loss) / 2.0), Style::default().fg(Color::Yellow)),
        Span::styled(format!("{:.3}", max_loss), Style::default().fg(Color::Red)),
    ];

    let chart = Chart::new(datasets)
        .block(Block::default()
            .title(Line::from(vec![
                Span::styled("", Style::default().fg(Color::Cyan)),
                Span::styled("Loss Convergence ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::styled(format!("(Last {} gens)", metrics.len()), Style::default().fg(Color::Gray)),
            ]))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)))
        .x_axis(
            Axis::default()
                .title("Generation")
                .style(Style::default().fg(Color::Gray))
                .bounds([0.0, (metrics.len() - 1) as f64])
                .labels(x_labels),
        )
        .y_axis(
            Axis::default()
                .title("Loss")
                .style(Style::default().fg(Color::Gray))
                .bounds([min_loss.min(0.0), max_loss * 1.1])
                .labels(y_labels),
        );

    frame.render_widget(chart, area);
}

#[allow(dead_code)]
fn render_complexity_diversity_chart(frame: &mut Frame, area: Rect, state: &AppState) {
    let metrics = &state.training_state.metrics_history;

    if metrics.is_empty() {
        let placeholder = Paragraph::new("No training data yet...")
            .block(Block::default()
                .title("System Dynamics")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)));
        frame.render_widget(placeholder, area);
        return;
    }

    // Use last 100 metrics for chart
    let recent_metrics: Vec<&crate::tui::training::MetricSnapshot> = metrics.iter().rev().take(100).rev().collect();

    let complexity_data: Vec<(f64, f64)> = recent_metrics.iter()
        .enumerate()
        .map(|(i, m)| (i as f64, m.complexity))
        .collect();

    let diversity_data: Vec<(f64, f64)> = recent_metrics.iter()
        .enumerate()
        .map(|(i, m)| (i as f64, m.diversity))
        .collect();

    let max_val = recent_metrics.iter()
        .map(|m| m.complexity.max(m.diversity))
        .fold(0.0, f64::max)
        .max(0.1);

    let datasets = vec![
        Dataset::default()
            .name("Complexity")
            .marker(symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(Color::Magenta))
            .data(&complexity_data),
        Dataset::default()
            .name("Diversity")
            .marker(symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(Color::Yellow))
            .data(&diversity_data),
    ];

    let chart = Chart::new(datasets)
        .block(Block::default()
            .title(Line::from(vec![
                Span::styled("System Dynamics ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::styled(format!("(Last {} gens)", recent_metrics.len()), Style::default().fg(Color::Gray)),
            ]))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)))
        .x_axis(
            Axis::default()
                .title("Generation")
                .style(Style::default().fg(Color::Gray))
                .bounds([0.0, (recent_metrics.len().max(1) - 1) as f64])
                .labels(vec![
                    Span::raw("0"),
                    Span::raw(format!("{}", recent_metrics.len() / 2)),
                    Span::raw(format!("{}", recent_metrics.len())),
                ]),
        )
        .y_axis(
            Axis::default()
                .title("Value")
                .style(Style::default().fg(Color::Gray))
                .bounds([0.0, max_val * 1.1])
                .labels(vec![
                    Span::raw("0.0"),
                    Span::styled(format!("{:.2}", max_val / 2.0), Style::default().fg(Color::Yellow)),
                    Span::styled(format!("{:.2}", max_val), Style::default().fg(Color::Magenta)),
                ]),
        );

    frame.render_widget(chart, area);
}

fn render_learning_dynamics_chart(frame: &mut Frame, area: Rect, state: &AppState) {
    // Show loss velocity and acceleration - key indicators of learning progress
    let velocity = state.training_state.loss_velocity;
    let acceleration = state.training_state.loss_acceleration;

    let velocity_color = if velocity < -0.001 {
        Color::Green  // Good: loss decreasing
    } else if velocity > 0.001 {
        Color::Red    // Bad: loss increasing
    } else {
        Color::Yellow  // Neutral: loss stable
    };

    let accel_color = if acceleration < -0.0001 {
        Color::Green   // Good: learning speeding up
    } else if acceleration > 0.0001 {
        Color::Red     // Bad: learning slowing down
    } else {
        Color::Yellow  // Neutral: stable learning rate
    };

    let lines = vec![
        Line::from(vec![
            Span::styled("Learning Rate Analysis", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Loss Velocity:", Style::default().fg(Color::Gray)),
            Span::raw("      "),
            Span::styled(
                format!("{:+.6}", velocity),
                Style::default().fg(velocity_color).add_modifier(Modifier::BOLD)
            ),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled(
                if velocity < -0.001 { "-> Improving" } else if velocity > 0.001 { "<- Worsening" } else { "-> Stable" },
                Style::default().fg(velocity_color)
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Loss Acceleration:", Style::default().fg(Color::Gray)),
            Span::raw("  "),
            Span::styled(
                format!("{:+.6}", acceleration),
                Style::default().fg(accel_color).add_modifier(Modifier::BOLD)
            ),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled(
                if acceleration < -0.0001 { " Accelerating" } else if acceleration > 0.0001 { " Decelerating" } else { "-> Steady" },
                Style::default().fg(accel_color)
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Interpretation:", Style::default().fg(Color::Cyan)),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled(
                if velocity < -0.001 && acceleration < -0.0001 {
                    "*Excellent: Fast convergence"
                } else if velocity < -0.001 {
                    "*Good: Steady improvement"
                } else if velocity.abs() < 0.001 && state.training_state.current_loss < 0.1 {
                    "-> Stable: Near optimal"
                } else if velocity.abs() < 0.001 {
                    " Stuck: May need adjustment"
                } else {
                    "x Diverging: Check hyperparameters"
                },
                Style::default().fg(
                    if velocity < -0.001 { Color::Green }
                    else if velocity.abs() < 0.001 { Color::Yellow }
                    else { Color::Red }
                )
            ),
        ]),
    ];

    let block = Paragraph::new(lines)
        .block(Block::default()
            .title(Line::from(vec![
                Span::styled("", Style::default().fg(Color::Cyan)),
                Span::styled("Learning Dynamics", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            ]))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)));

    frame.render_widget(block, area);
}

#[allow(dead_code)]
fn render_per_pattern_losses(frame: &mut Frame, area: Rect, state: &AppState) {
    // Display loss for each pattern type - critical for understanding curriculum progress
    let per_pattern = &state.training_state.per_pattern_losses;

    let items: Vec<ListItem> = if per_pattern.is_empty() {
        vec![ListItem::new(Line::from("No pattern data yet..."))]
    } else {
        per_pattern.iter().map(|(pattern, loss)| {
            let (icon, color, status) = if *loss < 0.05 {
                ("*", Color::Green, "Mastered")
            } else if *loss < 0.1 {
                ("*", Color::Yellow, "Learning")
            } else if *loss < 0.3 {
                ("o", Color::Cyan, "Training")
            } else {
                ("*", Color::Red, "Struggling")
            };

            let content = vec![
                Line::from(vec![
                    Span::styled(format!("{} ", icon), Style::default().fg(color)),
                    Span::styled(format!("{:8}", pattern), Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                    Span::raw("  "),
                    Span::styled(format!("{:10}", status), Style::default().fg(color)),
                ]),
                Line::from(vec![
                    Span::raw("  Loss: "),
                    Span::styled(
                        format!("{:.4}", loss),
                        Style::default().fg(
                            if *loss < 0.05 { Color::Green }
                            else if *loss < 0.1 { Color::Yellow }
                            else { Color::Red }
                        )
                    ),
                    Span::raw("  Progress: "),
                    Span::styled(
                        format!("{:.0}%", ((1.0 - loss.min(1.0)) * 100.0)),
                        Style::default().fg(Color::Cyan)
                    ),
                ]),
                Line::from(""),
            ];

            ListItem::new(content)
        }).collect()
    };

    let list = List::new(items)
        .block(Block::default()
            .title(Line::from(vec![
                Span::styled("", Style::default().fg(Color::Cyan)),
                Span::styled("Per-Pattern Performance", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            ]))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)));

    frame.render_widget(list, area);
}

fn render_hyperparameter_panel(frame: &mut Frame, area: Rect, state: &AppState) {
    // Display current hyperparameters - essential for AI engineering
    let lines = vec![
        Line::from(vec![
            Span::styled(" Active Configuration", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Learning Rate:", Style::default().fg(Color::Gray)),
            Span::raw("  "),
            Span::styled(
                format!("{:.6}", state.training_state.learning_rate),
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            ),
        ]),
        Line::from(vec![
            Span::styled("Batch Size:", Style::default().fg(Color::Gray)),
            Span::raw("      "),
            Span::styled(
                format!("{}", state.training_state.batch_size),
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
            ),
        ]),
        Line::from(vec![
            Span::styled("Evolution Steps:", Style::default().fg(Color::Gray)),
            Span::raw(" "),
            Span::styled(
                format!("{}", state.training_state.evolution_steps),
                Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Hot-Reload Status", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("*Active", Style::default().fg(Color::Green)),
            Span::raw(" - Press [L] to reload"),
        ]),
    ];

    let block = Paragraph::new(lines)
        .block(Block::default()
            .title(Line::from(vec![
                Span::styled(" ", Style::default().fg(Color::Cyan)),
                Span::styled("Hyperparameters", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            ]))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)));

    frame.render_widget(block, area);
}

fn render_eta_panel(frame: &mut Frame, area: Rect, state: &AppState) {
    // Predictive analytics - ETA to mastery
    let eta = state.training_state.estimated_time_to_mastery;
    let streak = state.training_state.low_loss_streak;
    let attempts = state.training_state.pattern_attempts;

    let lines = vec![
        Line::from(vec![
            Span::styled("Mastery Prediction", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Current Pattern:", Style::default().fg(Color::Gray)),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled(
                state.training_state.nca_current_pattern.clone(),
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("ETA to Mastery:", Style::default().fg(Color::Gray)),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled(
                if let Some(gens) = eta {
                    if gens < 1.0 {
                        "*Mastered!".to_string()
                    } else if gens < 100.0 {
                        format!("{:.0} generations", gens)
                    } else if gens < 1000.0 {
                        format!("{:.1}k generations", gens / 1000.0)
                    } else {
                        format!("{:.1}k+ generations", gens / 1000.0)
                    }
                } else {
                    "Calculating...".to_string()
                },
                Style::default().fg(
                    if let Some(gens) = eta {
                        if gens < 100.0 { Color::Green }
                        else if gens < 500.0 { Color::Yellow }
                        else { Color::Red }
                    } else {
                        Color::Gray
                    }
                ).add_modifier(Modifier::BOLD)
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Pattern Attempts:", Style::default().fg(Color::Gray)),
            Span::raw("  "),
            Span::styled(format!("{}", attempts), Style::default().fg(Color::Cyan)),
        ]),
        Line::from(vec![
            Span::styled("Low-Loss Streak:", Style::default().fg(Color::Gray)),
            Span::raw("  "),
            Span::styled(
                format!("{}", streak),
                Style::default().fg(if streak > 10 { Color::Green } else { Color::Yellow })
            ),
        ]),
    ];

    let block = Paragraph::new(lines)
        .block(Block::default()
            .title(Line::from(vec![
                Span::styled(" ", Style::default().fg(Color::Cyan)),
                Span::styled("Progress Tracker", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            ]))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)));

    frame.render_widget(block, area);
}

#[allow(dead_code)]
fn render_pattern_comparison(frame: &mut Frame, area: Rect) {
    let patterns = query_pattern_progress();

    let items: Vec<ListItem> = patterns.iter().map(|p| {
        let (icon, color, status) = if p.is_mastered {
            ("*", Color::Green, "Mastered")
        } else if p.best_loss < 0.1 {
            ("*", Color::Yellow, "Learning")
        } else {
            ("o", Color::Gray, "Training")
        };

        // Calculate convergence rate (inverse of loss as proxy)
        let convergence = if p.best_loss > 0.0 {
            (1.0 / p.best_loss).min(100.0)
        } else {
            100.0
        };

        let content = vec![
            Line::from(vec![
                Span::styled(format!("{} ", icon), Style::default().fg(color)),
                Span::styled(format!("{:8}", p.pattern), Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                Span::raw("  "),
                Span::styled(format!("{:10}", status), Style::default().fg(color)),
            ]),
            Line::from(vec![
                Span::raw("  Loss: "),
                Span::styled(format!("{:.4}", p.best_loss), Style::default().fg(Color::Cyan)),
                Span::raw("  Conv: "),
                Span::styled(format!("{:.1}", convergence), Style::default().fg(Color::Magenta)),
            ]),
            Line::from(""),
        ];

        ListItem::new(content)
    }).collect();

    let list = List::new(items)
        .block(Block::default()
            .title(Line::from(vec![
                Span::styled("", Style::default().fg(Color::Cyan)),
                Span::styled("Pattern Performance", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            ]))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)));

    frame.render_widget(list, area);
}

fn render_training_stats(frame: &mut Frame, area: Rect, state: &AppState) {
    let metrics = query_recent_metrics(50);

    // Calculate statistics
    let total_gens = state.training_state.nca_generation;
    let recent_loss = state.training_state.current_loss;
    let avg_loss = if !metrics.is_empty() {
        metrics.iter().map(|m| m.loss).sum::<f64>() / metrics.len() as f64
    } else {
        recent_loss
    };

    // Calculate improvement rate (loss reduction per 10 generations)
    let improvement_rate = if metrics.len() >= 10 {
        let old_avg = metrics.iter().take(10).map(|m| m.loss).sum::<f64>() / 10.0;
        let new_avg = metrics.iter().rev().take(10).map(|m| m.loss).sum::<f64>() / 10.0;
        ((old_avg - new_avg) / old_avg * 100.0).max(-100.0)
    } else {
        0.0
    };

    let mastered_count = state.training_state.patterns_mastered;
    let total_patterns = state.training_state.total_patterns;  // Dynamic - updates with hot-reload (now 5 with Hexagon)

    let lines = vec![
        Line::from(vec![
            Span::styled("Training Overview", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Generation:    ", Style::default().fg(Color::Gray)),
            Span::styled(format!("{}", total_gens), Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("Current Loss:  ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{:.4}", recent_loss),
                Style::default().fg(if recent_loss < 0.1 { Color::Green } else if recent_loss < 0.3 { Color::Yellow } else { Color::Red }).add_modifier(Modifier::BOLD)
            ),
        ]),
        Line::from(vec![
            Span::styled("Avg Loss (50): ", Style::default().fg(Color::Gray)),
            Span::styled(format!("{:.4}", avg_loss), Style::default().fg(Color::Cyan)),
        ]),
        Line::from(vec![
            Span::styled("Improvement:   ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{:+.1}%", improvement_rate),
                Style::default().fg(if improvement_rate > 0.0 { Color::Green } else { Color::Red })
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Mastery Status", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Mastered:      ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{}/{}", mastered_count, total_patterns),
                Style::default().fg(if mastered_count == total_patterns { Color::Green } else { Color::Yellow }).add_modifier(Modifier::BOLD)
            ),
        ]),
        Line::from(vec![
            Span::styled("Completion:    ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{:.0}%", (mastered_count as f64 / total_patterns as f64) * 100.0),
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
            ),
        ]),
    ];

    let block = Paragraph::new(lines)
        .block(Block::default()
            .title(Line::from(vec![
                Span::styled("", Style::default().fg(Color::Cyan)),
                Span::styled("Quick Stats", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            ]))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)));

    frame.render_widget(block, area);
}

fn render_footer(frame: &mut Frame, area: Rect) {
    let footer_text = vec![
        Line::from(vec![
            Span::styled("[Tab]", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(" Main Dashboard  "),
            Span::styled("[Space]", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(" Pause/Resume  "),
            Span::styled("[N]", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(" Start Training  "),
            Span::styled("[Q]", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(" Quit"),
        ]),
    ];

    let footer = Paragraph::new(footer_text)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Cyan)));

    frame.render_widget(footer, area);
}

// Query functions
#[derive(Debug)]
#[allow(dead_code)]
struct Metric {
    generation: u64,
    loss: f64,
    complexity: f64,
    diversity: f64,
    #[allow(dead_code)]
    pattern: String,
}

#[derive(Debug)]
#[allow(dead_code)]
struct PatternProgress {
    pattern: String,
    best_loss: f64,
    is_mastered: bool,
}

fn query_recent_metrics(limit: usize) -> Vec<Metric> {
    let mut metrics = Vec::new();

    let output = Command::new("spacetime")
        .args(&["sql", "sage-db", "SELECT * FROM training_metrics"])
        .stderr(std::process::Stdio::null())
        .output();

    if let Ok(output) = output {
        if output.status.success() {
            let text = String::from_utf8_lossy(&output.stdout);
            let lines: Vec<&str> = text.lines().collect();

            for line in lines.iter().rev().take(limit + 2) {
                if line.starts_with('+') || line.is_empty() || line.contains("generation") {
                    continue;
                }

                let parts: Vec<&str> = line.split('|').map(|s| s.trim()).collect();
                if parts.len() >= 7 {
                    metrics.push(Metric {
                        generation: parts[2].parse().unwrap_or(0),
                        loss: parts[3].parse().unwrap_or(1.0),
                        complexity: parts[4].parse().unwrap_or(0.0),
                        diversity: parts[5].parse().unwrap_or(0.0),
                        pattern: parts[6].trim_matches('"').to_string(),
                    });
                }
            }
        }
    }

    metrics.reverse();
    metrics
}

#[allow(dead_code)]
fn query_pattern_progress() -> Vec<PatternProgress> {
    let mut patterns = Vec::new();

    let output = Command::new("spacetime")
        .args(&["sql", "sage-db", "SELECT * FROM pattern_progress"])
        .stderr(std::process::Stdio::null())
        .output();

    if let Ok(output) = output {
        if output.status.success() {
            let text = String::from_utf8_lossy(&output.stdout);
            let lines: Vec<&str> = text.lines().collect();

            for line in lines.iter() {
                if line.starts_with('+') || line.is_empty() || line.contains("pattern") {
                    continue;
                }

                let parts: Vec<&str> = line.split('|').map(|s| s.trim()).collect();
                if parts.len() >= 7 {
                    patterns.push(PatternProgress {
                        pattern: parts[2].trim_matches('"').to_string(),
                        best_loss: parts[5].parse().unwrap_or(1.0),
                        is_mastered: parts[6] == "true",
                    });
                }
            }
        }
    }

    patterns
}
