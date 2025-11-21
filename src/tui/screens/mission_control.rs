// Mission Control - Full-width neural visualization + IRC feed + metrics
// This is the main monitoring dashboard for SAGE with LLM integration

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};
use crate::tui::app::AppState;
use crate::irc_sync::IrcSync;
use super::ScreenTrait;

pub struct MissionControlScreen;

impl ScreenTrait for MissionControlScreen {
    fn render(&self, frame: &mut Frame, area: Rect, state: &AppState) {
        // Main layout: Top (80%) and Bottom footer (20%)
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(80),  // Main content
                Constraint::Percentage(20),  // Footer with controls
            ])
            .split(area);

        // Top section: Neural field (70%) | Sidebar (30%)
        let top_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(70),  // Neural CT scan (full width!)
                Constraint::Percentage(30),  // Sidebar (metrics + IRC feed)
            ])
            .split(main_chunks[0]);

        // Render full-width neural field
        render_neural_ct_scan(frame, top_chunks[0], state);

        // Right sidebar: Metrics (30%) + IRC Feed (70%)
        let sidebar_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(30),  // Metrics compact
                Constraint::Percentage(70),  // IRC feed
            ])
            .split(top_chunks[1]);

        render_metrics(frame, sidebar_chunks[0], state);
        render_irc_feed(frame, sidebar_chunks[1], state);

        // Bottom section: Emotional state + controls
        render_emotional_state(frame, main_chunks[1], state);
    }
}

fn render_neural_ct_scan(frame: &mut Frame, area: Rect, state: &AppState) {
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
        " SCANNING"
    } else if state.training_mode == TrainingMode::IrcLearning {
        "IRC LEARNING"
    } else {
        "IDLE"
    };

    let pattern_name = if !state.training_state.nca_current_pattern.is_empty() {
        state.training_state.nca_current_pattern.clone()
    } else {
        "No pattern".to_string()
    };

    let pattern_text = format!("  │  Pattern: {}", pattern_name);

    lines.push(Line::from(vec![
        Span::styled("NEURAL CT SCAN  ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::styled(scan_indicator, Style::default().fg(
            if state.training_mode != crate::tui::app::TrainingMode::Idle { Color::Green } else { Color::DarkGray }
        )),
        Span::styled(pattern_text, Style::default().fg(Color::Yellow)),
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
    // Get live IRC message count
    let irc_count = IrcSync::load().ok().map(|s| s.messages.len()).unwrap_or(0);

    let metrics_text = vec![
        Line::from(vec![
            Span::styled("METRICS", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw("Gen: "),
            Span::styled(format!("{}", state.training_state.nca_generation), Style::default().fg(Color::Yellow)),
        ]),
        Line::from(vec![
            Span::raw("Loss: "),
            Span::styled(
                format!("{:.4}", state.training_state.current_loss),
                Style::default().fg(if state.training_state.current_loss < 0.05 {
                    Color::Green
                } else if state.training_state.current_loss < 0.15 {
                    Color::Yellow
                } else {
                    Color::Red
                })
            ),
        ]),
        Line::from(vec![
            Span::raw("Diversity: "),
            Span::styled(format!("{:.3}", state.training_state.nca_diversity), Style::default().fg(Color::Cyan)),
        ]),
        Line::from(vec![
            Span::raw("Complexity: "),
            Span::styled(format!("{:.3}", state.training_state.nca_complexity), Style::default().fg(Color::Magenta)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw("IRC Messages: "),
            Span::styled(format!("{}", irc_count), Style::default().fg(Color::Green)),
        ]),
    ];

    let metrics = Paragraph::new(metrics_text)
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(metrics, area);
}

fn render_irc_feed(frame: &mut Frame, area: Rect, _state: &AppState) {
    let mut feed_items: Vec<ListItem> = vec![];

    // Title
    feed_items.push(ListItem::new(Line::from(vec![
        Span::styled("IRC FEED", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
    ])));

    // Read live messages from IrcSync (shared file between IRC bot and TUI)
    let messages = IrcSync::get_recent(20);

    if messages.is_empty() {
        feed_items.push(ListItem::new(Line::from(vec![
            Span::styled("  No messages yet", Style::default().fg(Color::DarkGray)),
        ])));
        feed_items.push(ListItem::new(Line::from(vec![
            Span::raw("  Start IRC bot with:"),
        ])));
        feed_items.push(ListItem::new(Line::from(vec![
            Span::styled("  make irc", Style::default().fg(Color::Cyan)),
        ])));
    } else {
        for msg in messages.iter() {
            // Timestamp + User message
            feed_items.push(ListItem::new(Line::from(vec![
                Span::styled(format!("[{}] ", msg.timestamp), Style::default().fg(Color::DarkGray)),
                Span::styled(format!("<{}> ", msg.sender), Style::default().fg(Color::Blue)),
                Span::raw(msg.message.chars().take(40).collect::<String>()),
            ])));

            // SAGE response
            if !msg.sage_response.is_empty() {
                feed_items.push(ListItem::new(Line::from(vec![
                    Span::raw("         "),
                    Span::styled("<SAGE> ", Style::default().fg(Color::Green)),
                    Span::raw(msg.sage_response.chars().take(40).collect::<String>()),
                ])));
            }

            // Show concepts mentioned (if any)
            if !msg.concepts_mentioned.is_empty() {
                let concepts_str = msg.concepts_mentioned.join(", ");
                feed_items.push(ListItem::new(Line::from(vec![
                    Span::raw("         "),
                    Span::styled("", Style::default().fg(Color::Magenta)),
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

fn render_emotional_state(frame: &mut Frame, area: Rect, state: &AppState) {
    // Infer emotional state from loss
    let (mood_emoji, mood_text) = if state.training_state.current_loss < 0.05 {
        ("😊", "Confident & Clear")
    } else if state.training_state.current_loss < 0.15 {
        ("", "Learning & Growing")
    } else {
        ("🔍", "Curious & Exploring")
    };

    let uptime_text = format!("Uptime: {}s", state.uptime_seconds);

    let status_lines = vec![
        Line::from(vec![
            Span::styled("🎭 EMOTIONAL STATE  ", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
            Span::styled(format!("{} {}", mood_emoji, mood_text), Style::default().fg(Color::Yellow)),
            Span::raw("  │  "),
            Span::styled(uptime_text, Style::default().fg(Color::Cyan)),
            Span::raw("  │  "),
            Span::styled("[Tab] Screens  [N] Train  [Q] Quit", Style::default().fg(Color::DarkGray)),
        ]),
    ];

    let status = Paragraph::new(status_lines)
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(status, area);
}

/// CT Scan visualization: Color-coded neural activity with understanding feedback
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
