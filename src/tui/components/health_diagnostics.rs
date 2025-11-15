// Health Diagnostics Panel - System health monitoring overlay

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use crate::tui::app::{AppState, HealthStatus};

pub struct HealthDiagnosticsPanel;

impl HealthDiagnosticsPanel {
    pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
        use ratatui::widgets::Clear;

        // Clear the area first to create modal effect
        frame.render_widget(Clear, area);

        // Split into sections
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(5),   // Header
                Constraint::Percentage(40),  // System status
                Constraint::Percentage(30),  // Performance metrics
                Constraint::Percentage(30),  // Resource usage
            ])
            .split(area);

        // Render header
        render_header(frame, chunks[0], state);

        // Render system status
        render_system_status(frame, chunks[1], state);

        // Render performance metrics
        render_performance_metrics(frame, chunks[2], state);

        // Render resource usage
        render_resource_usage(frame, chunks[3], state);
    }
}

fn render_header(frame: &mut Frame, area: Rect, state: &AppState) {
    let (health_icon, health_color) = match state.health_status {
        HealthStatus::Healthy => ("OK", Color::Green),
        HealthStatus::Warning => ("WARN", Color::Yellow),
        HealthStatus::Critical => ("CRIT", Color::Red),
    };

    let header_text = vec![
        Line::from(vec![
            Span::styled("HEALTH DIAGNOSTICS", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(format!("{} ", health_icon), Style::default().fg(health_color)),
            Span::styled(
                format!("{:?}", state.health_status),
                Style::default().fg(health_color).add_modifier(Modifier::BOLD)
            ),
        ]),
    ];

    let header = Paragraph::new(header_text)
        .block(Block::default().borders(Borders::ALL).title("Health [H to toggle]"));

    frame.render_widget(header, area);
}

fn render_system_status(frame: &mut Frame, area: Rect, state: &AppState) {
    let status_text = vec![
        Line::from(vec![
            Span::styled("SYSTEM STATUS", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw("Current Phase: "),
            Span::styled(format!("{:?}", state.current_phase), Style::default().fg(Color::Yellow)),
        ]),
        Line::from(vec![
            Span::raw("Training: "),
            Span::styled(
                if state.training_state.is_running { "Running" } else { "Stopped" },
                Style::default().fg(if state.training_state.is_running { Color::Green } else { Color::Gray })
            ),
        ]),
        Line::from(vec![
            Span::raw("Paused: "),
            Span::styled(
                if state.is_paused { "Yes" } else { "No" },
                Style::default().fg(if state.is_paused { Color::Yellow } else { Color::Green })
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw("Uptime: "),
            Span::styled(
                format!("{}h {}m {}s",
                    state.uptime_seconds / 3600,
                    (state.uptime_seconds % 3600) / 60,
                    state.uptime_seconds % 60
                ),
                Style::default().fg(Color::Gray)
            ),
        ]),
    ];

    let status = Paragraph::new(status_text)
        .block(Block::default().borders(Borders::ALL));

    frame.render_widget(status, area);
}

fn render_performance_metrics(frame: &mut Frame, area: Rect, state: &AppState) {
    let (cycles, avg_improvement, is_improving, current_perf) =
        state.autonomous_trainer.autonomous_loop.get_stats();

    let perf_text = vec![
        Line::from(vec![
            Span::styled("PERFORMANCE", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw("Improvement Cycles: "),
            Span::styled(format!("{}", cycles), Style::default().fg(Color::Yellow)),
        ]),
        Line::from(vec![
            Span::raw("Avg Improvement: "),
            Span::styled(
                format!("{:.2}%", avg_improvement * 100.0),
                Style::default().fg(if avg_improvement > 0.0 { Color::Green } else { Color::Red })
            ),
        ]),
        Line::from(vec![
            Span::raw("Status: "),
            Span::styled(
                if is_improving { "Improving UP" } else { "Stable --" },
                Style::default().fg(if is_improving { Color::Green } else { Color::Yellow })
            ),
        ]),
        Line::from(vec![
            Span::raw("Current Performance: "),
            Span::styled(
                format!("{:.1}%", current_perf * 100.0),
                Style::default().fg(get_performance_color(current_perf))
            ),
        ]),
    ];

    let perf = Paragraph::new(perf_text)
        .block(Block::default().borders(Borders::ALL));

    frame.render_widget(perf, area);
}

fn render_resource_usage(frame: &mut Frame, area: Rect, state: &AppState) {
    let active_thoughts = state.agi.mind_stream.active_thoughts.len();
    let max_thoughts = state.agi.mind_stream.max_thoughts;
    let decisions_count = state.agi.decision_stream.decisions.len();

    let resource_text = vec![
        Line::from(vec![
            Span::styled("RESOURCE USAGE", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw("Active Thoughts: "),
            Span::styled(
                format!("{} / {}", active_thoughts, max_thoughts),
                Style::default().fg(if active_thoughts < max_thoughts { Color::Green } else { Color::Yellow })
            ),
        ]),
        Line::from(vec![
            Span::raw("Decision Buffer: "),
            Span::styled(format!("{}", decisions_count), Style::default().fg(Color::Yellow)),
        ]),
        Line::from(vec![
            Span::raw("Events Logged: "),
            Span::styled(
                format!("{}", state.training_state.recent_events.len()),
                Style::default().fg(Color::Yellow)
            ),
        ]),
    ];

    let resource = Paragraph::new(resource_text)
        .block(Block::default().borders(Borders::ALL));

    frame.render_widget(resource, area);
}

fn get_performance_color(performance: f64) -> Color {
    if performance > 0.7 {
        Color::Green
    } else if performance > 0.4 {
        Color::Yellow
    } else {
        Color::Red
    }
}
