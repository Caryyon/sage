// SAGE Brain Monitor - Enhanced with Vision and Autonomous Consciousness
// Dual-view: Camera feed + NCA grid processing
// Shows what SAGE sees and how it processes visual + conversational input

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
        // Responsive layout based on terminal size
        if area.height < 20 {
            // Minimal mode for small terminals
            render_minimal_mode(frame, area, state);
        } else {
            // Full dual-view mode
            render_full_mode(frame, area, state);
        }
    }
}

/// Minimal mode for small terminals - just show NCA grid
fn render_minimal_mode(frame: &mut Frame, area: Rect, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),       // Title
            Constraint::Min(10),         // NCA Grid
            Constraint::Length(3),       // Status
        ])
        .split(area);

    render_title_banner(frame, chunks[0], state);
    render_nca_grid(frame, chunks[1], state);
    render_compact_status(frame, chunks[2], state);
}

/// Full mode with dual-view camera + NCA grid
fn render_full_mode(frame: &mut Frame, area: Rect, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),       // Title banner
            Constraint::Min(15),         // Dual view: Camera + NCA Grid
            Constraint::Length(6),       // Autonomous Activity Feed
            Constraint::Length(5),       // Visual Memories & Status
        ])
        .split(area);

    render_title_banner(frame, chunks[0], state);
    render_dual_view(frame, chunks[1], state);
    render_autonomous_activity(frame, chunks[2], state);
    render_visual_memories_status(frame, chunks[3], state);
}

fn render_title_banner(frame: &mut Frame, area: Rect, state: &AppState) {
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

    let (consciousness_text, consciousness_color, consciousness_emoji) = match state.consciousness_state {
        crate::tui::app::ConsciousnessState::Active => ("ACTIVE", Color::Green, "⚡"),
        crate::tui::app::ConsciousnessState::Dreaming => ("DREAMING", Color::Magenta, "💭"),
        crate::tui::app::ConsciousnessState::Curious => ("CURIOUS", Color::Yellow, "🔍"),
    };

    // Show vision status
    let vision_status = if state.camera_frame.is_some() {
        Span::styled(" | 👁️  VISION ACTIVE", Style::default().fg(Color::Cyan))
    } else {
        Span::styled(" | 👁️  Vision: Standby", Style::default().fg(Color::DarkGray))
    };

    let title = Paragraph::new(Line::from(vec![
        Span::styled("🧠 ", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
        Span::styled("SAGE BRAIN MONITOR", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw("  │  "),
        Span::styled(format!("{} Experiences", total_exp), Style::default().fg(Color::Yellow)),
        Span::raw("  │  "),
        Span::styled(format!("{} {}", consciousness_emoji, consciousness_text), Style::default().fg(consciousness_color).add_modifier(Modifier::BOLD)),
        Span::raw("  │  "),
        Span::styled(format!("{}", opinion), Style::default().fg(opinion_color)),
        vision_status,
    ]))
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Cyan))
    );

    frame.render_widget(title, area);
}

/// Render dual-view: Camera feed on left, NCA processing on right
fn render_dual_view(frame: &mut Frame, area: Rect, state: &AppState) {
    let dual_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50),  // Camera view
            Constraint::Percentage(50),  // NCA grid
        ])
        .split(area);

    render_camera_view(frame, dual_chunks[0], state);
    render_nca_grid(frame, dual_chunks[1], state);
}

/// Render camera view showing what SAGE sees
fn render_camera_view(frame: &mut Frame, area: Rect, state: &AppState) {
    if let Some(ref camera_frame) = state.camera_frame {
        // Render the camera frame as colored blocks
        let height = camera_frame.len().min((area.height as usize).saturating_sub(2));
        let width = camera_frame.get(0).map(|r| r.len()).unwrap_or(0).min((area.width as usize / 2).saturating_sub(2));

        let mut camera_lines = Vec::new();
        for y in 0..height {
            let mut row_spans = Vec::new();
            for x in 0..width {
                if let Some(row) = camera_frame.get(y) {
                    if let Some(&(r, g, b)) = row.get(x) {
                        row_spans.push(Span::styled(
                            "██",
                            Style::default().fg(Color::Rgb(r, g, b))
                        ));
                    }
                }
            }
            camera_lines.push(Line::from(row_spans));
        }

        // Add visual concepts below the camera view
        if !state.visual_concepts.is_empty() {
            camera_lines.push(Line::from(""));
            camera_lines.push(Line::from(Span::styled(
                format!("Concepts: {}", state.visual_concepts.join(", ")),
                Style::default().fg(Color::Yellow).add_modifier(Modifier::ITALIC)
            )));
        }

        let camera_widget = Paragraph::new(camera_lines)
            .block(
                Block::default()
                    .title("👁️  Camera View (What SAGE Sees)")
                    .borders(Borders::ALL)
                    .style(Style::default().fg(Color::Cyan))
            )
            .alignment(Alignment::Center);

        frame.render_widget(camera_widget, area);
    } else {
        // No camera data - show placeholder
        let placeholder = Paragraph::new(vec![
            Line::from(""),
            Line::from(""),
            Line::from(Span::styled("👁️  Vision Standby",
                Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD))),
            Line::from(""),
            Line::from(Span::styled("Waiting for !see command on IRC...",
                Style::default().fg(Color::Gray))),
        ])
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .title("👁️  Camera View")
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::DarkGray))
        );

        frame.render_widget(placeholder, area);
    }
}

/// Render NCA grid showing neural processing
fn render_nca_grid(frame: &mut Frame, area: Rect, state: &AppState) {
    let grid_snapshot = IrcSync::get_nca_grid();

    if let Some(snapshot) = grid_snapshot {
        let grid_size = snapshot.grid_size;
        let alpha_values = &snapshot.alpha_values;

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

                let cell_phase = state.brain_pulse_phase + state.brain_cell_offsets.get(idx).unwrap_or(&0.0);
                let activity_boost = state.brain_activity_map.get(idx).unwrap_or(&0.0);
                let (ch, color) = alpha_to_style_animated(alpha, cell_phase, *activity_boost);

                row_spans.push(Span::styled(
                    format!("{}{}", ch, ch),
                    Style::default().fg(color)
                ));
            }

            grid_text.push(Line::from(row_spans));
        }

        // Show active concepts
        let concepts_display = if !snapshot.active_concepts.is_empty() {
            format!("Active: {}", snapshot.active_concepts.join(", "))
        } else {
            "No active concepts".to_string()
        };

        let block_title = format!("🧠 NCA Processing {}×{} │ Gen: {} │ {}",
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
                .title("🧠 NCA Processing 32×32")
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::DarkGray))
        );

        frame.render_widget(waiting, area);
    }
}

/// Render autonomous activity feed (dreams & curiosity)
fn render_autonomous_activity(frame: &mut Frame, area: Rect, state: &AppState) {
    let mut activity_lines = Vec::new();

    // Show last 5 autonomous activities
    let recent_activities: Vec<_> = state.autonomous_activities
        .iter()
        .rev()
        .take(5)
        .collect();

    if recent_activities.is_empty() {
        activity_lines.push(Line::from(Span::styled(
            "No autonomous activity yet - waiting for idle time...",
            Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC)
        )));
    } else {
        for (timestamp, activity_type, description) in recent_activities {
            let elapsed = state.uptime_seconds.saturating_sub(*timestamp);
            let mins = elapsed / 60;
            let secs = elapsed % 60;

            let (emoji, color) = match activity_type.as_str() {
                "dream" => ("💭", Color::Magenta),
                "curiosity" => ("🔍", Color::Yellow),
                _ => ("🧠", Color::Cyan),
            };

            activity_lines.push(Line::from(vec![
                Span::styled(format!("[{}m {}s ago] ", mins, secs), Style::default().fg(Color::DarkGray)),
                Span::styled(format!("{} ", emoji), Style::default().fg(color)),
                Span::styled(activity_type.to_uppercase(), Style::default().fg(color).add_modifier(Modifier::BOLD)),
                Span::raw(": "),
                Span::raw(description.chars().take(80).collect::<String>()),
            ]));
        }
    }

    let activity_widget = Paragraph::new(activity_lines)
        .block(
            Block::default()
                .title("💭 Autonomous Activity (Dreams & Curiosity)")
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::Magenta))
        );

    frame.render_widget(activity_widget, area);
}

/// Render visual memories and status bar
fn render_visual_memories_status(frame: &mut Frame, area: Rect, state: &AppState) {
    let idle_mins = state.idle_seconds / 60;
    let idle_secs = state.idle_seconds % 60;

    let mut status_lines = Vec::new();

    // Consciousness state
    let consciousness_info = match state.consciousness_state {
        crate::tui::app::ConsciousnessState::Active => {
            format!("⚡ Active learning │ Idle: {}m {}s", idle_mins, idle_secs)
        },
        crate::tui::app::ConsciousnessState::Dreaming => {
            format!("💭 Consolidating memories │ Idle: {}m {}s", idle_mins, idle_secs)
        },
        crate::tui::app::ConsciousnessState::Curious => {
            format!("🔍 Exploring autonomous goals │ Idle: {}m {}s", idle_mins, idle_secs)
        },
    };

    status_lines.push(Line::from(vec![
        Span::styled("Consciousness: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(consciousness_info),
    ]));

    // Grid stats
    let grid_snapshot = IrcSync::get_nca_grid();
    if let Some(snapshot) = grid_snapshot {
        let avg_activity = snapshot.alpha_values.iter().sum::<f64>() / snapshot.alpha_values.len() as f64;
        let alive_cells = snapshot.alpha_values.iter().filter(|&&a| a > 0.1).count();

        status_lines.push(Line::from(vec![
            Span::styled("Grid Stats: ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::raw(format!("Avg: {:.3} │ Alive: {}/1024", avg_activity, alive_cells)),
        ]));
    }

    // Visual concepts
    if !state.visual_concepts.is_empty() {
        status_lines.push(Line::from(vec![
            Span::styled("Visual Concepts: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(state.visual_concepts.join(", ")),
        ]));
    }

    // Latest conversation
    let recent_messages = IrcSync::get_recent(1);
    if let Some(msg) = recent_messages.last() {
        status_lines.push(Line::from(vec![
            Span::styled("Latest: ", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
            Span::raw(format!("{}: {} → {}",
                msg.sender,
                msg.message.chars().take(20).collect::<String>(),
                msg.sage_response.chars().take(30).collect::<String>())),
        ]));
    }

    let status = Paragraph::new(status_lines)
        .block(
            Block::default()
                .title("📊 System Status")
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::Cyan))
        );

    frame.render_widget(status, area);
}

/// Compact status for minimal mode
fn render_compact_status(frame: &mut Frame, area: Rect, state: &AppState) {
    let consciousness_emoji = match state.consciousness_state {
        crate::tui::app::ConsciousnessState::Active => "⚡",
        crate::tui::app::ConsciousnessState::Dreaming => "💭",
        crate::tui::app::ConsciousnessState::Curious => "🔍",
    };

    let status_text = format!("{} {} | Visual: {} | Autonomous: {} activities",
        consciousness_emoji,
        match state.consciousness_state {
            crate::tui::app::ConsciousnessState::Active => "Active",
            crate::tui::app::ConsciousnessState::Dreaming => "Dreaming",
            crate::tui::app::ConsciousnessState::Curious => "Curious",
        },
        if state.camera_frame.is_some() { "ON" } else { "Standby" },
        state.autonomous_activities.len()
    );

    let status = Paragraph::new(status_text)
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::Cyan))
        );

    frame.render_widget(status, area);
}

/// Map alpha value to visual character and color with neural firing effects
fn alpha_to_style_animated(alpha: f64, cell_phase: f64, activity_boost: f64) -> (&'static str, Color) {
    let pulse = (cell_phase.sin() * 0.12 + 1.0).max(0.85);
    let shimmer = (cell_phase * 3.7).sin() * 0.05;
    let adjusted_alpha = ((alpha * pulse) + shimmer + (activity_boost * 0.4)).clamp(0.0, 1.0);
    let firing_intensity = activity_boost;

    if firing_intensity > 0.5 {
        ("█", Color::Rgb(255, 255, 255))
    } else if firing_intensity > 0.2 {
        ("█", Color::Rgb(255, 180, 100))
    } else if adjusted_alpha < 0.05 {
        ("░", Color::Rgb(20, 10, 30))
    } else if adjusted_alpha < 0.15 {
        ("▒", Color::Rgb(30, 30, 80))
    } else if adjusted_alpha < 0.3 {
        ("▓", Color::Rgb(50, 80, 150))
    } else if adjusted_alpha < 0.5 {
        ("█", Color::Rgb(80, 180, 200))
    } else if adjusted_alpha < 0.65 {
        ("█", Color::Rgb(180, 220, 100))
    } else if adjusted_alpha < 0.8 {
        ("█", Color::Rgb(255, 230, 80))
    } else if adjusted_alpha < 0.9 {
        ("█", Color::Rgb(255, 160, 50))
    } else {
        ("█", Color::Rgb(255, 80, 40))
    }
}
