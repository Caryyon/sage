// Design 1: "The Social Mind" - Conversation-First Interface
// Large chat area with compact metrics and concept visualization

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Table, Row, Cell},
    Frame,
};
use crate::tui::app::AppState;
use crate::irc_sync::IrcSync;
use crate::tui::widgets::ConceptCloud;
use super::ScreenTrait;

pub struct SocialMindScreen {
    concept_cloud: ConceptCloud,
}

impl SocialMindScreen {
    pub fn new() -> Self {
        Self {
            concept_cloud: ConceptCloud::new(),
        }
    }

    pub fn update_concepts(&mut self, message: &str, baseline_concepts: &[String]) {
        self.concept_cloud.update_from_message(message, baseline_concepts);
    }
}

impl ScreenTrait for SocialMindScreen {
    fn render(&self, frame: &mut Frame, area: Rect, state: &AppState) {
        // Main layout: Top conversation (70%) + Bottom panels (30%)
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(65),  // Conversation area
                Constraint::Percentage(35),  // Metrics + Pattern progress
            ])
            .split(area);

        // Render conversation area
        render_conversation(frame, main_chunks[0], state);

        // Bottom: Split into Left (Metrics) and Right (Pattern Mastery)
        let bottom_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(50),  // Cognitive State
                Constraint::Percentage(50),  // Pattern Mastery
            ])
            .split(main_chunks[1]);

        // Split left side into Metrics + Concept Cloud
        let left_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(60),  // Cognitive State Gauges
                Constraint::Percentage(40),  // Concept Cloud
            ])
            .split(bottom_chunks[0]);

        render_cognitive_state(frame, left_chunks[0], state);
        self.concept_cloud.render(frame, left_chunks[1]);
        render_pattern_mastery(frame, bottom_chunks[1], state);
    }
}

fn render_conversation(frame: &mut Frame, area: Rect, _state: &AppState) {
    let mut feed_items: Vec<ListItem> = vec![];

    // Title
    feed_items.push(ListItem::new(Line::from(vec![
        Span::styled("💬 Live Conversation", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
    ])));
    feed_items.push(ListItem::new(Line::from("")));

    // Read live messages from IrcSync
    let messages = IrcSync::get_recent(30);

    if messages.is_empty() {
        feed_items.push(ListItem::new(Line::from(vec![
            Span::styled("  No messages yet. Start chatting on IRC!", Style::default().fg(Color::DarkGray)),
        ])));
        feed_items.push(ListItem::new(Line::from("")));
        feed_items.push(ListItem::new(Line::from(vec![
            Span::styled("  📡 Connect to: irc.libera.chat #sage-ai", Style::default().fg(Color::Cyan)),
        ])));
    } else {
        for msg in messages.iter() {
            // User message
            feed_items.push(ListItem::new(Line::from(vec![
                Span::styled(format!("[{}] ", msg.timestamp), Style::default().fg(Color::DarkGray)),
                Span::styled(format!("<{}> ", msg.sender), Style::default().fg(Color::Blue)),
                Span::raw(&msg.message),
            ])));

            // SAGE response
            if !msg.sage_response.is_empty() {
                feed_items.push(ListItem::new(Line::from(vec![
                    Span::raw("         "),
                    Span::styled("<SAGE> ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                    Span::styled(&msg.sage_response, Style::default().fg(Color::Green)),
                ])));
            }

            // Concepts mentioned
            if !msg.concepts_mentioned.is_empty() {
                let concepts_str = msg.concepts_mentioned.join(", ");
                feed_items.push(ListItem::new(Line::from(vec![
                    Span::raw("         "),
                    Span::styled("🧠 ", Style::default().fg(Color::Magenta)),
                    Span::styled(concepts_str, Style::default().fg(Color::Magenta)),
                ])));
            }

            // Blank line between conversations
            feed_items.push(ListItem::new(Line::from("")));
        }
    }

    let feed = List::new(feed_items)
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(feed, area);
}

fn render_cognitive_state(frame: &mut Frame, area: Rect, state: &AppState) {
    let mut lines = vec![];

    lines.push(Line::from(vec![
        Span::styled("🧠 Cognitive State", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
    ]));
    lines.push(Line::from(""));

    // Loss gauge
    let _loss_ratio = (1.0 - state.training_state.current_loss).max(0.0).min(1.0);
    let loss_color = if state.training_state.current_loss < 0.05 {
        Color::Green
    } else if state.training_state.current_loss < 0.15 {
        Color::Yellow
    } else {
        Color::Red
    };

    lines.push(Line::from(vec![
        Span::raw("Loss: "),
        Span::styled(format!("{:.4}", state.training_state.current_loss), Style::default().fg(loss_color)),
    ]));

    // Complexity
    lines.push(Line::from(vec![
        Span::raw("Complexity: "),
        Span::styled(format!("{:.3}", state.training_state.nca_complexity), Style::default().fg(Color::Yellow)),
    ]));

    // Diversity
    lines.push(Line::from(vec![
        Span::raw("Diversity: "),
        Span::styled(format!("{:.3}", state.training_state.nca_diversity), Style::default().fg(Color::Magenta)),
    ]));

    lines.push(Line::from(""));

    // Current status
    let pattern = if !state.training_state.nca_current_pattern.is_empty() {
        state.training_state.nca_current_pattern.clone()
    } else {
        "Idle".to_string()
    };

    lines.push(Line::from(vec![
        Span::styled("Current: ", Style::default().fg(Color::DarkGray)),
        Span::styled(pattern, Style::default().fg(Color::Yellow)),
    ]));

    lines.push(Line::from(vec![
        Span::styled(format!("Gen: {}", state.training_state.nca_generation), Style::default().fg(Color::Cyan)),
    ]));

    let paragraph = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(paragraph, area);
}

fn render_pattern_mastery(frame: &mut Frame, area: Rect, _state: &AppState) {
    // Create simple table showing pattern progress
    let header = Row::new(vec!["Pattern", "Progress"])
        .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));

    let patterns = vec![
        ("● Circle", "97%"),
        ("○ Square", "52%"),
        ("● Cross", "89%"),
        ("○ Spiral", "23%"),
    ];

    let rows: Vec<Row> = patterns.iter().map(|(name, progress)| {
        let status_color = if name.starts_with("●") {
            Color::Green
        } else {
            Color::Yellow
        };

        Row::new(vec![
            Cell::from(*name).style(Style::default().fg(status_color)),
            Cell::from(*progress),
        ])
    }).collect();

    let table = Table::new(rows, [Constraint::Percentage(60), Constraint::Percentage(40)])
        .header(header)
        .block(
            Block::default()
                .title("🎯 Pattern Mastery")
                .borders(Borders::ALL)
        );

    frame.render_widget(table, area);
}
