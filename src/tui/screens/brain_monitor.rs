// SAGE Brain Monitor - Live NCA Grid Visualization
// Optimized for ambient monitoring on external display
// Shows real-time neural pattern formation as SAGE learns from IRC conversations

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect, Alignment},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use crate::tui::app::AppState;
use crate::irc_sync::IrcSync;
use super::ScreenTrait;

pub struct BrainMonitorScreen;

impl BrainMonitorScreen {
    pub fn new() -> Self {
        Self
    }
}

impl ScreenTrait for BrainMonitorScreen {
    fn render(&self, frame: &mut Frame, area: Rect, state: &AppState) {
        // Main layout: Title + Grid + Status
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),       // Title banner
                Constraint::Min(10),         // NCA Grid (takes remaining space)
                Constraint::Length(4),       // Status bar with metrics
            ])
            .split(area);

        render_title_banner(frame, chunks[0], state);
        render_nca_grid(frame, chunks[1], state);
        render_status_bar(frame, chunks[2], state);
    }
}

fn render_title_banner(frame: &mut Frame, area: Rect, _state: &AppState) {
    // Get IRC sync data
    let total_exp = IrcSync::get_total_experiences();
    let grid_snapshot = IrcSync::get_nca_grid();

    let opinion = grid_snapshot
        .as_ref()
        .map(|g| g.current_opinion.clone())
        .unwrap_or_else(|| "Awaiting input".to_string());

    let opinion_color = match opinion.as_str() {
        s if s.contains("Positive") => Color::Green,
        s if s.contains("Negative") => Color::Red,
        s if s.contains("Curious") => Color::Yellow,
        _ => Color::Gray,
    };

    let title = Paragraph::new(Line::from(vec![
        Span::styled("🧠 ", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
        Span::styled("SAGE BRAIN MONITOR", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw("  │  "),
        Span::styled(format!("{} Experiences", total_exp), Style::default().fg(Color::Yellow)),
        Span::raw("  │  "),
        Span::styled(format!("State: {}", opinion), Style::default().fg(opinion_color)),
    ]))
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Cyan))
    );

    frame.render_widget(title, area);
}

fn render_nca_grid(frame: &mut Frame, area: Rect, state: &AppState) {
    // Get NCA grid from IRC sync
    let grid_snapshot = IrcSync::get_nca_grid();

    if let Some(snapshot) = grid_snapshot {
        let grid_size = snapshot.grid_size;
        let alpha_values = &snapshot.alpha_values;

        // Calculate cell size based on available space
        let usable_width = area.width.saturating_sub(2) as f64;  // Account for borders
        let usable_height = (area.height.saturating_sub(2)) as f64 * 2.0;  // Double for block chars

        let _cell_width = usable_width / grid_size as f64;
        let _cell_height = usable_height / grid_size as f64;

        // Build grid visualization
        let mut grid_text = Vec::new();

        for y in 0..grid_size {
            let mut row_spans = Vec::new();

            for x in 0..grid_size {
                let idx = y * grid_size + x;
                let alpha = if idx < alpha_values.len() {
                    alpha_values[idx]
                } else {
                    0.0
                };

                // Get per-cell animation phase and activity boost from AppState
                let cell_phase = state.brain_pulse_phase + state.brain_cell_offsets.get(idx).unwrap_or(&0.0);
                let activity_boost = state.brain_activity_map.get(idx).unwrap_or(&0.0);

                // Map alpha to character and color with neural firing effects
                let (ch, color) = alpha_to_style_animated(alpha, cell_phase, *activity_boost);

                // Create styled span for each cell with its own color
                row_spans.push(Span::styled(
                    format!("{}{}", ch, ch),  // Double width for square cells
                    Style::default().fg(color)
                ));
            }

            grid_text.push(Line::from(row_spans));
        }

        // Active concepts display
        let concepts_display = if !snapshot.active_concepts.is_empty() {
            format!("Active: {}", snapshot.active_concepts.join(", "))
        } else {
            "No active concepts".to_string()
        };

        let block_title = format!("Neural Grid {}×{} │ Gen: {} │ {}",
            grid_size, grid_size, snapshot.generation, concepts_display);

        let grid = Paragraph::new(grid_text)
            .block(
                Block::default()
                    .title(block_title)
                    .borders(Borders::ALL)
                    .style(Style::default().fg(Color::Green))
            )
            .alignment(Alignment::Center);

        frame.render_widget(grid, area);
    } else {
        // No grid data available yet
        let waiting = Paragraph::new(vec![
            Line::from(""),
            Line::from(""),
            Line::from(Span::styled("⏳ Waiting for neural activity...",
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
            Line::from(""),
            Line::from(Span::styled("Start a conversation on IRC to see SAGE's brain form patterns!",
                Style::default().fg(Color::Gray))),
        ])
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .title("Neural Grid 32×32")
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::DarkGray))
        );

        frame.render_widget(waiting, area);
    }
}

fn render_status_bar(frame: &mut Frame, area: Rect, _state: &AppState) {
    let grid_snapshot = IrcSync::get_nca_grid();
    let recent_messages = IrcSync::get_recent(3);

    // Latest conversation snippet
    let latest_conv = if let Some(msg) = recent_messages.last() {
        format!("{}: {} → SAGE: {}",
            msg.sender,
            msg.message.chars().take(40).collect::<String>(),
            msg.sage_response.chars().take(40).collect::<String>())
    } else {
        "No recent conversations".to_string()
    };

    // Grid statistics
    let grid_stats = if let Some(snapshot) = grid_snapshot {
        let avg_activity = snapshot.alpha_values.iter().sum::<f64>() / snapshot.alpha_values.len() as f64;
        let max_activity = snapshot.alpha_values.iter().cloned().fold(0.0, f64::max);
        let alive_cells = snapshot.alpha_values.iter().filter(|&&a| a > 0.1).count();

        format!("Avg: {:.3} │ Max: {:.3} │ Alive: {}/{}",
            avg_activity, max_activity, alive_cells, snapshot.alpha_values.len())
    } else {
        "No grid data".to_string()
    };

    let status = Paragraph::new(vec![
        Line::from(vec![
            Span::styled("📊 Stats: ", Style::default().fg(Color::Cyan)),
            Span::raw(grid_stats),
        ]),
        Line::from(vec![
            Span::styled("💬 Latest: ", Style::default().fg(Color::Green)),
            Span::raw(latest_conv),
        ]),
    ])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title("System Status")
            .style(Style::default().fg(Color::Cyan))
    );

    frame.render_widget(status, area);
}

/// Map alpha value to visual character and color with neural firing effects
/// Heat map gradient: cold (blue/purple) → warm (yellow/orange) → hot (red/white)
fn alpha_to_style_animated(alpha: f64, cell_phase: f64, activity_boost: f64) -> (&'static str, Color) {
    // Per-cell pulse effect (each cell pulses independently)
    let pulse = (cell_phase.sin() * 0.12 + 1.0).max(0.85);

    // Add subtle shimmer (high frequency variation for organic feel)
    let shimmer = (cell_phase * 3.7).sin() * 0.05;

    // Combine effects
    let adjusted_alpha = ((alpha * pulse) + shimmer + (activity_boost * 0.4)).clamp(0.0, 1.0);

    // Neural firing spark effect (activity_boost creates bright flashes)
    let firing_intensity = activity_boost;

    if firing_intensity > 0.5 {
        // Active neural firing - brilliant white-hot spark
        ("⚡", Color::Rgb(255, 255, 255))
    } else if firing_intensity > 0.2 {
        // Mild firing - bright orange-red
        ("█", Color::Rgb(255, 180, 100))
    } else if adjusted_alpha < 0.05 {
        // Nearly dead - very dark purple/black
        ("░", Color::Rgb(20, 10, 30))
    } else if adjusted_alpha < 0.15 {
        // Very weak - dark blue
        ("▒", Color::Rgb(30, 30, 80))
    } else if adjusted_alpha < 0.3 {
        // Weak - medium blue
        ("▓", Color::Rgb(50, 80, 150))
    } else if adjusted_alpha < 0.5 {
        // Medium - cyan/teal
        ("█", Color::Rgb(80, 180, 200))
    } else if adjusted_alpha < 0.65 {
        // Medium-high - yellow-green
        ("█", Color::Rgb(180, 220, 100))
    } else if adjusted_alpha < 0.8 {
        // High - yellow
        ("█", Color::Rgb(255, 230, 80))
    } else if adjusted_alpha < 0.9 {
        // Very high - orange
        ("█", Color::Rgb(255, 160, 50))
    } else {
        // Maximum - bright red-orange (hot!)
        ("█", Color::Rgb(255, 80, 40))
    }
}
