// Activity Feed Panel - Shows recent training events and activities

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};
use crate::tui::app::AppState;

pub struct ActivityFeedPanel;

impl ActivityFeedPanel {
    pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
        let mut lines = vec![
            Line::from(vec![
                Span::styled("ACTIVITY FEED", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(""),
        ];

        // Show recent training events
        if state.training_state.recent_events.is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::from("  No recent activity"));
            lines.push(Line::from(""));
            lines.push(Line::from("  Start training to see"));
            lines.push(Line::from("  real-time events"));
        } else {
            for event in state.training_state.recent_events.iter().rev().take(20) {
                let (icon, color) = get_event_icon_and_color(event);

                lines.push(Line::from(vec![
                    Span::styled(format!("{} ", icon), Style::default().fg(color)),
                    Span::raw(event),
                ]));
            }
        }

        let paragraph = Paragraph::new(lines)
            .block(Block::default().borders(Borders::ALL).title("Activity Feed [R to toggle]"))
            .wrap(Wrap { trim: true });

        frame.render_widget(paragraph, area);
    }
}

fn get_event_icon_and_color(event: &str) -> (&'static str, Color) {
    let event_lower = event.to_lowercase();

    if event_lower.contains("complete") || event_lower.contains("finished") {
        (">", Color::Green)
    } else if event_lower.contains("started") || event_lower.contains("beginning") {
        ("+", Color::Cyan)
    } else if event_lower.contains("error") || event_lower.contains("failed") {
        ("X", Color::Red)
    } else if event_lower.contains("warning") {
        ("!", Color::Yellow)
    } else if event_lower.contains("thought") || event_lower.contains("hypothesis") {
        ("*", Color::Magenta)
    } else if event_lower.contains("task") {
        ("-", Color::Cyan)
    } else if event_lower.contains("paused") {
        ("||", Color::Yellow)
    } else {
        ("•", Color::White)
    }
}
