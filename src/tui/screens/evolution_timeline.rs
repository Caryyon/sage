// Design 3: "The Evolution Timeline" - Historical Progression View
// Shows complete learning journey with cycles, mastery events, and knowledge transfer

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Table, Row, Cell},
    Frame,
};
use crate::tui::app::AppState;
use crate::irc_sync::IrcSync;
use crate::tui::widgets::{TrainingTimeline, MetricSparklines};
use super::ScreenTrait;

pub struct EvolutionTimelineScreen {
    timeline: TrainingTimeline,
    sparklines: MetricSparklines,
}

impl EvolutionTimelineScreen {
    pub fn new() -> Self {
        Self {
            timeline: TrainingTimeline::new(),
            sparklines: MetricSparklines::new(),
        }
    }

    pub fn update_timeline(&mut self, generation: u32, pattern: String, cycle: u32) {
        self.timeline.update(generation, pattern, cycle);
    }

    pub fn push_metrics(&mut self, loss: f64, complexity: f64, diversity: f64) {
        self.sparklines.push_metrics(loss, complexity, diversity);
    }
}

impl ScreenTrait for EvolutionTimelineScreen {
    fn render(&self, frame: &mut Frame, area: Rect, state: &AppState) {
        // Main layout: Timeline + Current + Performance + History + Milestones + Feed
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(8),       // Training timeline
                Constraint::Length(3),       // Current pattern status
                Constraint::Percentage(25),  // Pattern performance comparison
                Constraint::Percentage(25),  // Metrics history sparklines
                Constraint::Percentage(25),  // Recent milestones
                Constraint::Percentage(25),  // Live feed
            ])
            .split(area);

        // Render all components
        self.timeline.render(frame, main_chunks[0]);
        render_current_status(frame, main_chunks[1], state);
        render_pattern_performance(frame, main_chunks[2], state);
        render_metrics_history(frame, main_chunks[3], state, &self.sparklines);
        render_milestones(frame, main_chunks[4], state);
        render_live_feed(frame, main_chunks[5]);
    }
}

fn render_current_status(frame: &mut Frame, area: Rect, state: &AppState) {
    let pattern = if !state.training_state.nca_current_pattern.is_empty() {
        state.training_state.nca_current_pattern.clone()
    } else {
        "Idle".to_string()
    };

    let status = Paragraph::new(Line::from(vec![
        Span::styled("Current: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::styled(format!("{} │ ", pattern), Style::default().fg(Color::Green)),
        Span::styled(format!("Gen: {} │ ", state.training_state.nca_generation), Style::default().fg(Color::Cyan)),
        Span::styled(format!("Loss: {:.4}", state.training_state.current_loss), Style::default().fg(Color::Magenta)),
    ]))
    .block(Block::default().borders(Borders::ALL));

    frame.render_widget(status, area);
}

fn render_pattern_performance(frame: &mut Frame, area: Rect, _state: &AppState) {
    let header = Row::new(vec!["Pattern", "Cycle 1", "Cycle 2", "Improvement"])
        .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));

    let patterns = vec![
        ("Circle", "97%", "97%", "Maintained", Color::Green),
        ("Square", "65%", "72%", "+7%", Color::Yellow),
        ("Cross", "89%", "—", "Not revisited", Color::DarkGray),
        ("Spiral", "23%", "—", "Not revisited", Color::DarkGray),
    ];

    let rows: Vec<Row> = patterns.iter().map(|(name, c1, c2, improvement, color)| {
        Row::new(vec![
            Cell::from(*name),
            Cell::from(*c1),
            Cell::from(*c2),
            Cell::from(*improvement).style(Style::default().fg(*color)),
        ])
    }).collect();

    let table = Table::new(rows, [
        Constraint::Percentage(25),
        Constraint::Percentage(25),
        Constraint::Percentage(25),
        Constraint::Percentage(25),
    ])
    .header(header)
    .block(
        Block::default()
            .title("📊 Pattern Performance Comparison")
            .borders(Borders::ALL)
    );

    frame.render_widget(table, area);
}

fn render_metrics_history(frame: &mut Frame, area: Rect, _state: &AppState, sparklines: &MetricSparklines) {
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

fn render_milestones(frame: &mut Frame, area: Rect, state: &AppState) {
    let items = vec![
        ListItem::new(Line::from(vec![
            Span::styled(format!("Gen {}  ", state.training_state.nca_generation), Style::default().fg(Color::Cyan)),
            Span::styled("⟳ ", Style::default().fg(Color::Yellow)),
            Span::raw("Current training in progress"),
        ])),
        ListItem::new(Line::from(vec![
            Span::styled("Gen 2200  ", Style::default().fg(Color::Cyan)),
            Span::styled("✓ ", Style::default().fg(Color::Green)),
            Span::raw("Circle revisited - Mastery maintained at 97%"),
        ])),
        ListItem::new(Line::from(vec![
            Span::styled("Gen 2000  ", Style::default().fg(Color::Cyan)),
            Span::styled("⟳ ", Style::default().fg(Color::Yellow)),
            Span::raw("Cycle 2 started - Beginning pattern revisitation"),
        ])),
        ListItem::new(Line::from(vec![
            Span::styled("Gen 1500  ", Style::default().fg(Color::Cyan)),
            Span::styled("✓ ", Style::default().fg(Color::Green)),
            Span::raw("Cross mastered - 89% final score"),
        ])),
        ListItem::new(Line::from(vec![
            Span::styled("Gen 1000  ", Style::default().fg(Color::Cyan)),
            Span::styled("✓ ", Style::default().fg(Color::Green)),
            Span::raw("Square mastered - 65% after 98 attempts"),
        ])),
    ];

    let list = List::new(items)
        .block(
            Block::default()
                .title("🏆 Recent Milestones")
                .borders(Borders::ALL)
        );

    frame.render_widget(list, area);
}

fn render_live_feed(frame: &mut Frame, area: Rect) {
    let messages = IrcSync::get_recent(4);

    let mut items: Vec<ListItem> = vec![];

    if messages.is_empty() {
        items.push(ListItem::new(Line::from(vec![
            Span::styled("  No recent activity", Style::default().fg(Color::DarkGray)),
        ])));
    } else {
        for msg in messages.iter().rev() {
            let icon = if !msg.sage_response.is_empty() {
                "💬"
            } else {
                "📊"
            };

            items.push(ListItem::new(Line::from(vec![
                Span::styled(format!("{} ", msg.timestamp), Style::default().fg(Color::DarkGray)),
                Span::raw(format!("{} ", icon)),
                Span::raw(if !msg.sage_response.is_empty() {
                    msg.sage_response.chars().take(50).collect::<String>()
                } else {
                    msg.message.chars().take(50).collect::<String>()
                }),
            ])));
        }
    }

    let list = List::new(items)
        .block(
            Block::default()
                .title("📡 Live Feed")
                .borders(Borders::ALL)
        );

    frame.render_widget(list, area);
}
