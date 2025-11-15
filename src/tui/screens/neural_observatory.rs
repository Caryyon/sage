// Design 2: "The Neural Observatory" - Real-Time Metrics Dashboard
// Mission control style with sparklines, gauges, and system vitals

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph},
    Frame,
};
use crate::tui::app::AppState;
use crate::irc_sync::IrcSync;
use crate::tui::widgets::MetricSparklines;
use super::ScreenTrait;

pub struct NeuralObservatoryScreen {
    sparklines: MetricSparklines,
}

impl NeuralObservatoryScreen {
    pub fn new() -> Self {
        Self {
            sparklines: MetricSparklines::new(),
        }
    }

    pub fn push_metrics(&mut self, loss: f64, complexity: f64, diversity: f64) {
        self.sparklines.push_metrics(loss, complexity, diversity);
    }
}

impl ScreenTrait for NeuralObservatoryScreen {
    fn render(&self, frame: &mut Frame, area: Rect, state: &AppState) {
        // Main layout: Header + Content + Footer
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),       // System vitals
                Constraint::Percentage(50),  // Neural activity + Pattern progress
                Constraint::Percentage(30),  // Conversation stream
                Constraint::Percentage(20),  // Cognitive metrics gauges
            ])
            .split(area);

        // Render components
        render_system_vitals(frame, main_chunks[0], state);

        // Middle section: Neural activity (left) + Pattern progress (right)
        let middle_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(60),  // Sparklines
                Constraint::Percentage(40),  // Pattern progress
            ])
            .split(main_chunks[1]);

        render_neural_activity(frame, middle_chunks[0], state, &self.sparklines);
        render_pattern_progress(frame, middle_chunks[1], state);

        render_conversation_stream(frame, main_chunks[2]);
        render_metric_gauges(frame, main_chunks[3], state);
    }
}

fn render_system_vitals(frame: &mut Frame, area: Rect, state: &AppState) {
    let status_indicator = if state.training_mode != crate::tui::app::TrainingMode::Idle {
        ("◉", Color::Green)
    } else {
        ("○", Color::DarkGray)
    };

    let pattern = if !state.training_state.nca_current_pattern.is_empty() {
        state.training_state.nca_current_pattern.clone()
    } else {
        "Idle".to_string()
    };

    let vitals = Paragraph::new(Line::from(vec![
        Span::styled(status_indicator.0, Style::default().fg(status_indicator.1).add_modifier(Modifier::BOLD)),
        Span::raw(" ACTIVE  │  "),
        Span::styled(format!("Gen: {}  │  ", state.training_state.nca_generation), Style::default().fg(Color::Yellow)),
        Span::styled(format!("Uptime: {}s  │  ", state.uptime_seconds), Style::default().fg(Color::Cyan)),
        Span::styled(format!("Pattern: {}", pattern), Style::default().fg(Color::Green)),
    ]))
    .block(
        Block::default()
            .title("🔬 SAGE NEURAL OBSERVATORY")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Cyan))
    );

    frame.render_widget(vitals, area);
}

fn render_neural_activity(frame: &mut Frame, area: Rect, _state: &AppState, sparklines: &MetricSparklines) {
    // Split into three sparkline panels
    let sparkline_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(33),
            Constraint::Percentage(34),
        ])
        .split(area);

    sparklines.render_loss(frame, sparkline_chunks[0]);
    sparklines.render_complexity(frame, sparkline_chunks[1]);
    sparklines.render_diversity(frame, sparkline_chunks[2]);
}

fn render_pattern_progress(frame: &mut Frame, area: Rect, state: &AppState) {
    let mut lines = vec![];

    lines.push(Line::from(vec![
        Span::styled("🎯 Pattern Progress", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
    ]));
    lines.push(Line::from(""));

    // Pattern list with progress bars
    let patterns = [
        ("Circle", 0.97, Color::Green),
        ("Square", 0.52, Color::Yellow),
        ("Cross", 0.89, Color::Green),
        ("Spiral", 0.23, Color::Red),
    ];

    for (name, progress, color) in patterns {
        let bar_length = 10;
        let filled = (progress * bar_length as f64) as usize;
        let empty = bar_length - filled;

        let bar = format!("{}{}",
            "█".repeat(filled),
            "░".repeat(empty)
        );

        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(bar, Style::default().fg(color)),
            Span::raw("  "),
            Span::styled(name, Style::default().fg(Color::White)),
            Span::raw(format!("  {:.0}%", progress * 100.0)),
        ]));
    }

    lines.push(Line::from(""));

    // Current pattern info
    let pattern = if !state.training_state.nca_current_pattern.is_empty() {
        state.training_state.nca_current_pattern.clone()
    } else {
        "Idle".to_string()
    };

    lines.push(Line::from(vec![
        Span::styled("Current: ", Style::default().fg(Color::DarkGray)),
        Span::styled(pattern, Style::default().fg(Color::Yellow)),
    ]));

    let paragraph = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(paragraph, area);
}

fn render_conversation_stream(frame: &mut Frame, area: Rect) {
    let messages = IrcSync::get_recent(10);

    let mut items: Vec<ListItem> = vec![];

    if messages.is_empty() {
        items.push(ListItem::new(Line::from(vec![
            Span::styled("  No IRC activity yet", Style::default().fg(Color::DarkGray)),
        ])));
    } else {
        for msg in messages.iter().rev().take(5) {  // Show last 5, newest first
            items.push(ListItem::new(Line::from(vec![
                Span::styled(format!("{} ", msg.timestamp), Style::default().fg(Color::DarkGray)),
                Span::styled("<SAGE> ", Style::default().fg(Color::Green)),
                Span::raw(msg.sage_response.chars().take(60).collect::<String>()),
            ])));
        }
    }

    let list = List::new(items)
        .block(
            Block::default()
                .title("💬 Conversation Stream (SAGE only)")
                .borders(Borders::ALL)
        );

    frame.render_widget(list, area);
}

fn render_metric_gauges(frame: &mut Frame, area: Rect, state: &AppState) {
    // Split into 3 gauge columns
    let gauge_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(33),
            Constraint::Percentage(34),
        ])
        .split(area);

    // Loss gauge (inverted - lower is better)
    let loss_ratio = (1.0 - state.training_state.current_loss).max(0.0).min(1.0);
    let loss_gauge = Gauge::default()
        .block(
            Block::default()
                .title(format!("Loss: {:.4}", state.training_state.current_loss))
                .borders(Borders::ALL)
        )
        .gauge_style(Style::default().fg(Color::Cyan))
        .ratio(loss_ratio);
    frame.render_widget(loss_gauge, gauge_chunks[0]);

    // Complexity gauge
    let complexity_ratio = state.training_state.nca_complexity.min(1.0);
    let complexity_gauge = Gauge::default()
        .block(
            Block::default()
                .title(format!("Complexity: {:.3}", state.training_state.nca_complexity))
                .borders(Borders::ALL)
        )
        .gauge_style(Style::default().fg(Color::Yellow))
        .ratio(complexity_ratio);
    frame.render_widget(complexity_gauge, gauge_chunks[1]);

    // Diversity gauge
    let diversity_ratio = state.training_state.nca_diversity.min(1.0);
    let diversity_gauge = Gauge::default()
        .block(
            Block::default()
                .title(format!("Diversity: {:.3}", state.training_state.nca_diversity))
                .borders(Borders::ALL)
        )
        .gauge_style(Style::default().fg(Color::Magenta))
        .ratio(diversity_ratio);
    frame.render_widget(diversity_gauge, gauge_chunks[2]);
}
