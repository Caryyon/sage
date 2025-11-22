//! Meta-Learning Dashboard Screen
//!
//! Displays real-time information about all 6 meta-learning phases:
//! - Phase A: Adaptive LR with warmup/cosine visualization
//! - Phase B: Reptile meta-learning status
//! - Phase C: PBT population and hyperparameter evolution
//! - Phase D: Strategy selection decisions
//! - Phase E: Architecture modification history
//! - Phase F: Learned optimizer comparison

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph},
    Frame,
};
use crate::tui::app::AppState;
use super::ScreenTrait;

pub struct MetaLearningDashboard;

impl MetaLearningDashboard {
    pub fn new() -> Self {
        Self
    }
}

impl ScreenTrait for MetaLearningDashboard {
    fn render(&self, frame: &mut Frame, area: Rect, state: &AppState) {
        // Main layout: header + 3 rows of 2 columns each
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),   // Header
                Constraint::Length(8),   // Row 1: Strategy + Adaptive LR
                Constraint::Length(8),   // Row 2: Reptile + PBT
                Constraint::Length(8),   // Row 3: Architecture + Events
                Constraint::Min(0),      // Footer/overflow
            ])
            .split(area);

        // Render header
        render_header(frame, main_chunks[0], state);

        // Row 1: Strategy Selection + Adaptive LR
        let row1 = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(main_chunks[1]);
        render_strategy_panel(frame, row1[0], state);
        render_adaptive_lr_panel(frame, row1[1], state);

        // Row 2: Reptile + PBT
        let row2 = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(main_chunks[2]);
        render_reptile_panel(frame, row2[0], state);
        render_pbt_panel(frame, row2[1], state);

        // Row 3: Architecture + Events
        let row3 = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
            .split(main_chunks[3]);
        render_architecture_panel(frame, row3[0], state);
        render_events_panel(frame, row3[1], state);
    }
}

fn render_header(frame: &mut Frame, area: Rect, state: &AppState) {
    let ts = &state.training_state;
    let gen = ts.nca_generation;
    let loss = ts.current_loss;
    let pattern = ts.nca_current_pattern.clone();

    let header_text = vec![
        Line::from(vec![
            Span::styled(" META-LEARNING DASHBOARD ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw(" | "),
            Span::styled(format!("Gen: {}", gen), Style::default().fg(Color::Yellow)),
            Span::raw(" | "),
            Span::styled(format!("Loss: {:.4}", loss), Style::default().fg(if loss < 0.1 { Color::Green } else { Color::White })),
            Span::raw(" | "),
            Span::styled(format!("Pattern: {}", pattern), Style::default().fg(Color::Magenta)),
        ]),
    ];

    let header = Paragraph::new(header_text)
        .block(Block::default().borders(Borders::BOTTOM).border_style(Style::default().fg(Color::DarkGray)));
    frame.render_widget(header, area);
}

fn render_strategy_panel(frame: &mut Frame, area: Rect, state: &AppState) {
    let ts = &state.training_state;

    let content = vec![
        Line::from(vec![
            Span::raw("Strategy: "),
            Span::styled(&ts.meta_learning_strategy, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::raw("Confidence: "),
            Span::styled(format!("{:.0}%", ts.strategy_confidence * 100.0), Style::default().fg(Color::Green)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Reasoning:", Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(vec![
            Span::raw(truncate_str(&ts.strategy_reasoning, 40)),
        ]),
    ];

    let panel = Paragraph::new(content)
        .block(Block::default()
            .title(" Phase D: Strategy Selection ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Blue)));
    frame.render_widget(panel, area);
}

fn render_adaptive_lr_panel(frame: &mut Frame, area: Rect, state: &AppState) {
    let ts = &state.training_state;
    let lr = ts.learning_rate;
    let warmup_complete = ts.warmup_complete;
    let cycle_progress = ts.lr_cycle_progress;

    // Split into info + gauge (unused for now, kept for future gauge positioning)
    let _chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(4), Constraint::Length(2)])
        .margin(1)
        .split(area);

    let content = vec![
        Line::from(vec![
            Span::raw("Learning Rate: "),
            Span::styled(format!("{:.6}", lr), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::raw("Phase: "),
            Span::styled(
                if warmup_complete { "Cosine Annealing" } else { "Warmup" },
                Style::default().fg(if warmup_complete { Color::Green } else { Color::Yellow })
            ),
        ]),
    ];

    let panel = Paragraph::new(content)
        .block(Block::default()
            .title(" Phase A: Adaptive Learning ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green)));
    frame.render_widget(panel, area);

    // Render cycle progress gauge (inside the panel area)
    if area.height > 5 {
        let gauge_area = Rect {
            x: area.x + 2,
            y: area.y + area.height - 2,
            width: area.width.saturating_sub(4),
            height: 1,
        };
        let gauge = Gauge::default()
            .ratio(cycle_progress)
            .gauge_style(Style::default().fg(Color::Cyan));
        frame.render_widget(gauge, gauge_area);
    }
}

fn render_reptile_panel(frame: &mut Frame, area: Rect, state: &AppState) {
    let ts = &state.training_state;

    let content = vec![
        Line::from(vec![
            Span::raw("Meta-Steps: "),
            Span::styled(format!("{}", ts.reptile_meta_steps), Style::default().fg(Color::Cyan)),
        ]),
        Line::from(vec![
            Span::raw("Avg Loss: "),
            Span::styled(format!("{:.4}", ts.reptile_avg_loss), Style::default().fg(Color::Yellow)),
        ]),
        Line::from(vec![
            Span::raw("Few-Shot Ready: "),
            Span::styled(
                if ts.reptile_meta_steps > 0 { "Yes" } else { "Training..." },
                Style::default().fg(if ts.reptile_meta_steps > 0 { Color::Green } else { Color::DarkGray })
            ),
        ]),
    ];

    let panel = Paragraph::new(content)
        .block(Block::default()
            .title(" Phase B: Reptile Meta-Learning ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Magenta)));
    frame.render_widget(panel, area);
}

fn render_pbt_panel(frame: &mut Frame, area: Rect, state: &AppState) {
    let ts = &state.training_state;

    let content = vec![
        Line::from(vec![
            Span::raw("Population: "),
            Span::styled(format!("{}", ts.pbt_population_size), Style::default().fg(Color::Cyan)),
        ]),
        Line::from(vec![
            Span::raw("Evolutions: "),
            Span::styled(format!("{}", ts.pbt_evolutions), Style::default().fg(Color::Yellow)),
        ]),
        Line::from(vec![
            Span::raw("Best Score: "),
            Span::styled(format!("{:.4}", ts.pbt_best_score), Style::default().fg(Color::Green)),
        ]),
        Line::from(vec![
            Span::raw("Best LR: "),
            Span::styled(format!("{:.6}", ts.pbt_best_lr), Style::default().fg(Color::White)),
        ]),
    ];

    let panel = Paragraph::new(content)
        .block(Block::default()
            .title(" Phase C: Population Training ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow)));
    frame.render_widget(panel, area);
}

fn render_architecture_panel(frame: &mut Frame, area: Rect, state: &AppState) {
    let ts = &state.training_state;

    let content = vec![
        Line::from(vec![
            Span::raw("Hidden Neurons: "),
            Span::styled(format!("{}", ts.arch_neurons), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::raw("Modifications: "),
            Span::styled(format!("{}", ts.arch_modifications), Style::default().fg(Color::Yellow)),
        ]),
        Line::from(vec![
            Span::raw("Rollbacks: "),
            Span::styled(format!("{}", ts.arch_rollbacks), Style::default().fg(if ts.arch_rollbacks > 0 { Color::Red } else { Color::Green })),
        ]),
    ];

    let panel = Paragraph::new(content)
        .block(Block::default()
            .title(" Phase E: Architecture ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Red)));
    frame.render_widget(panel, area);
}

fn render_events_panel(frame: &mut Frame, area: Rect, state: &AppState) {
    let ts = &state.training_state;

    let items: Vec<ListItem> = if ts.meta_learning_events.is_empty() {
        vec![ListItem::new("No events")]
    } else {
        ts.meta_learning_events.iter()
            .rev()
            .take(5)
            .map(|e| {
                let (icon, color) = match e.as_str() {
                    s if s.contains("Strategy") => ("", Color::Blue),
                    s if s.contains("LR") => ("", Color::Green),
                    s if s.contains("Reptile") => ("", Color::Magenta),
                    s if s.contains("PBT") => ("", Color::Yellow),
                    s if s.contains("Arch") => ("", Color::Red),
                    _ => ("", Color::White),
                };
                ListItem::new(Line::from(vec![
                    Span::styled(format!("{} ", icon), Style::default().fg(color)),
                    Span::raw(truncate_str(e, 50)),
                ]))
            })
            .collect()
    };

    let list = List::new(items)
        .block(Block::default()
            .title(" Meta-Learning Events ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::White)));
    frame.render_widget(list, area);
}

fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() > max_len {
        format!("{}...", &s[..max_len - 3])
    } else {
        s.to_string()
    }
}
