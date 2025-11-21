// IRC Chat Screen - Shows live conversations with SAGE

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};
use super::ScreenTrait;
use crate::tui::app::AppState;

pub struct ChatScreen;

impl ScreenTrait for ChatScreen {
    fn render(&self, frame: &mut Frame, area: Rect, state: &AppState) {
        // Split screen: Header | Chat | SAGE Status | Input
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),   // Header
                Constraint::Min(10),     // Chat history
                Constraint::Length(12),  // SAGE status panel
                Constraint::Length(3),   // Input area
            ])
            .split(area);

        // Render header
        render_header(frame, chunks[0], state);

        // Render chat history
        render_chat_history(frame, chunks[1], state);

        // Render SAGE status
        render_sage_status(frame, chunks[2], state);

        // Render input area
        render_input(frame, chunks[3], state);
    }
}

fn render_header(frame: &mut Frame, area: Rect, state: &AppState) {
    let title = Paragraph::new(vec![
        Line::from(vec![
            Span::styled("SAGE IRC Chat  ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(format!("│ #sage-consciousness  │ ", ), Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{} messages", state.irc_messages.len()),
                Style::default().fg(Color::Green)
            ),
        ]),
    ])
    .alignment(Alignment::Center)
    .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Cyan)));

    frame.render_widget(title, area);
}

fn render_chat_history(frame: &mut Frame, area: Rect, state: &AppState) {
    // Show last 20 messages
    let messages: Vec<ListItem> = state
        .irc_messages
        .iter()
        .rev()
        .take(20)
        .rev()
        .map(|(sender, message, response)| {
            // Format: sender message, then SAGE's response
            let user_line = Line::from(vec![
                Span::styled(format!("{}: ", sender), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(message, Style::default().fg(Color::White)),
            ]);

            let sage_line = Line::from(vec![
                Span::styled("  SAGE: ", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
                Span::styled(response, Style::default().fg(Color::LightMagenta)),
            ]);

            ListItem::new(vec![user_line, sage_line, Line::from("")])
        })
        .collect();

    let chat_list = List::new(messages)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title(" Chat History ")
        );

    frame.render_widget(chat_list, area);
}

fn render_sage_status(frame: &mut Frame, area: Rect, state: &AppState) {
    // Split into 3 columns: Personality | Stats | Curiosity
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(34),
            Constraint::Percentage(33),
        ])
        .split(area);

    // Personality panel
    let personality_text = vec![
        Line::from(vec![
            Span::styled("Likes: ", Style::default().fg(Color::Green)),
            Span::raw(format!("{}", state.sage.get_likes().len())),
        ]),
        Line::from(vec![
            Span::styled("Dislikes: ", Style::default().fg(Color::Red)),
            Span::raw(format!("{}", state.sage.get_dislikes().len())),
        ]),
        Line::from(""),
        Line::from(
            state.sage.get_likes()
                .first()
                .map(|s| format!(" {}", s))
                .unwrap_or_else(|| "No preferences yet".to_string())
        ),
    ];

    let personality = Paragraph::new(personality_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Green))
                .title(" Personality ")
        )
        .wrap(Wrap { trim: true });

    frame.render_widget(personality, chunks[0]);

    // Stats panel
    let stats_text = vec![
        Line::from(vec![
            Span::styled("Experiences: ", Style::default().fg(Color::Cyan)),
            Span::raw(format!("{}", state.sage.experience_count())),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Associations: ", Style::default().fg(Color::Yellow)),
            Span::raw(format!("{}", state.sage.get_concept_clusters().len())),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Status: ", Style::default().fg(Color::Green)),
            Span::styled("*Online", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        ]),
    ];

    let stats = Paragraph::new(stats_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Blue))
                .title(" Stats ")
        );

    frame.render_widget(stats, chunks[1]);

    // Curiosity panel
    let curiosity_summary = state.sage.get_curiosity_summary();
    let curiosity_lines: Vec<Line> = curiosity_summary
        .lines()
        .take(8)
        .map(|line| Line::from(line.to_string()))
        .collect();

    let curiosity = Paragraph::new(curiosity_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Magenta))
                .title(" Curiosity ")
        )
        .wrap(Wrap { trim: true });

    frame.render_widget(curiosity, chunks[2]);
}

fn render_input(frame: &mut Frame, area: Rect, _state: &AppState) {
    let input_widget = Paragraph::new("Type to chat with SAGE (IRC simulation mode)")
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray))
                .title(" Input ")
        );

    frame.render_widget(input_widget, area);
}
