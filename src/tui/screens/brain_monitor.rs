// SAGE Dashboard - Primary training visualization
// Shows NCA grid, training metrics, and meta-learning status
// Renamed from Brain Monitor to be the main dashboard

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

/// Full mode with NCA grid + training metrics
fn render_full_mode(frame: &mut Frame, area: Rect, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),       // Title banner
            Constraint::Min(12),         // Main content: NCA Grid + Training Metrics
            Constraint::Length(8),       // Training progress + Events
        ])
        .split(area);

    render_title_banner(frame, chunks[0], state);
    render_main_content(frame, chunks[1], state);
    render_training_progress(frame, chunks[2], state);
}

/// Main content: NCA grid on left, training metrics on right
fn render_main_content(frame: &mut Frame, area: Rect, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(55),  // NCA Grid (larger)
            Constraint::Percentage(45),  // Training Metrics
        ])
        .split(area);

    render_nca_grid(frame, chunks[0], state);
    render_training_metrics(frame, chunks[1], state);
}

/// Training metrics panel showing generation, loss, pattern, meta-learning
fn render_training_metrics(frame: &mut Frame, area: Rect, state: &AppState) {
    // Split into: Target Pattern (top) + Metrics (bottom)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(9),   // Target pattern ASCII art (needs more space)
            Constraint::Min(7),      // Metrics
        ])
        .split(area);

    render_target_pattern(frame, chunks[0], state);
    render_metrics_content(frame, chunks[1], state);
}

/// ASCII art of the target pattern SAGE is learning
fn render_target_pattern(frame: &mut Frame, area: Rect, state: &AppState) {
    let ts = &state.training_state;
    let pattern_name = &ts.nca_current_pattern;

    // Extract just the pattern type from the full name (e.g., "Circle" from "(1/9) Circle [1] FORM...")
    let simple_name = extract_pattern_name(pattern_name);

    // Get pattern ASCII art and color
    let (art, pattern_color) = get_pattern_art(&simple_name);

    // Build display with pattern art
    let mut lines = vec![
        Line::from(vec![
            Span::styled("TARGET ", Style::default().fg(Color::Rgb(0, 255, 0)).add_modifier(Modifier::BOLD)),
            Span::styled(&simple_name, Style::default().fg(pattern_color).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),  // Spacer
    ];

    for art_line in art {
        lines.push(Line::from(Span::styled(art_line, Style::default().fg(pattern_color))));
    }

    let panel = Paragraph::new(lines)
        .alignment(Alignment::Center)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Rgb(0, 255, 0))));

    frame.render_widget(panel, area);
}

/// Extract the simple pattern name from complex training strings
fn extract_pattern_name(full_name: &str) -> String {
    let lower = full_name.to_lowercase();
    if lower.contains("circle") {
        "Circle".to_string()
    } else if lower.contains("square") {
        "Square".to_string()
    } else if lower.contains("cross") {
        "Cross".to_string()
    } else if lower.contains("spiral") {
        "Spiral".to_string()
    } else if full_name.is_empty() {
        "Initializing...".to_string()
    } else {
        // Return first word or truncated version
        full_name.split_whitespace().next().unwrap_or("Unknown").to_string()
    }
}

/// Get ASCII art for each pattern type
fn get_pattern_art(pattern_name: &str) -> (Vec<&'static str>, Color) {
    match pattern_name.to_lowercase().as_str() {
        s if s.contains("circle") => (vec![
            "   ████   ",
            " ██    ██ ",
            "██      ██",
            " ██    ██ ",
            "   ████   ",
        ], Color::Rgb(0, 255, 136)),  // eerie_green

        s if s.contains("square") => (vec![
            "██████████",
            "██      ██",
            "██      ██",
            "██      ██",
            "██████████",
        ], Color::Rgb(255, 136, 0)),  // ember_orange

        s if s.contains("cross") => (vec![
            "    ██    ",
            "    ██    ",
            "██████████",
            "    ██    ",
            "    ██    ",
        ], Color::Rgb(255, 0, 0)),  // fire_red

        s if s.contains("spiral") => (vec![
            "  ██████  ",
            " ██    ██ ",
            "██  ██  ██",
            " ██  ██   ",
            "  ████    ",
        ], Color::Rgb(136, 255, 0)),  // bright_green

        _ => (vec![
            "    ??    ",
            "   ????   ",
            "  ??????  ",
            "   ????   ",
            "    ??    ",
        ], Color::Gray),
    }
}

/// The actual metrics content with visual bars
fn render_metrics_content(frame: &mut Frame, area: Rect, state: &AppState) {
    let ts = &state.training_state;

    // Loss bar visualization (inverted - lower is better)
    let loss_ratio = (1.0 - ts.current_loss.min(1.0)).max(0.0);
    let loss_bar_width = 15;
    let filled = (loss_ratio * loss_bar_width as f64) as usize;
    let loss_bar: String = format!("[{}{}]",
        "█".repeat(filled),
        "░".repeat(loss_bar_width - filled)
    );

    let loss_color = if ts.current_loss < 0.05 {
        Color::Rgb(0, 255, 0)  // gmork_green
    } else if ts.current_loss < 0.15 {
        Color::Rgb(136, 255, 0)  // bright_green
    } else if ts.current_loss < 0.3 {
        Color::Rgb(255, 136, 0)  // ember_orange
    } else {
        Color::Rgb(255, 0, 0)  // fire_red
    };

    // Mastery progress bar
    let mastery_ratio = (ts.low_loss_streak as f64 / 50.0).min(1.0);
    let mastery_filled = (mastery_ratio * 10.0) as usize;
    let mastery_bar: String = format!("[{}{}]",
        "█".repeat(mastery_filled),
        "░".repeat(10 - mastery_filled)
    );

    let lines = vec![
        // Generation with large number
        Line::from(vec![
            Span::styled("Gen ", Style::default().fg(Color::Rgb(0, 180, 0))),
            Span::styled(format!("{}", ts.nca_generation), Style::default().fg(Color::Rgb(0, 255, 0)).add_modifier(Modifier::BOLD)),
            Span::raw("  "),
            Span::styled("LR ", Style::default().fg(Color::Rgb(0, 180, 0))),
            Span::styled(format!("{:.5}", ts.learning_rate), Style::default().fg(Color::White)),
        ]),
        // Loss with visual bar
        Line::from(vec![
            Span::styled("Loss ", Style::default().fg(Color::Rgb(0, 180, 0))),
            Span::styled(format!("{:.4} ", ts.current_loss), Style::default().fg(loss_color).add_modifier(Modifier::BOLD)),
            Span::styled(&loss_bar, Style::default().fg(loss_color)),
        ]),
        // Mastery progress
        Line::from(vec![
            Span::styled("Mastery ", Style::default().fg(Color::Rgb(0, 180, 0))),
            Span::styled(format!("{}/50 ", ts.low_loss_streak.min(50)), Style::default().fg(
                if ts.low_loss_streak >= 50 { Color::Rgb(0, 255, 0) } else { Color::Rgb(255, 136, 0) }
            )),
            Span::styled(&mastery_bar, Style::default().fg(
                if ts.low_loss_streak >= 50 { Color::Rgb(0, 255, 0) } else { Color::Rgb(255, 136, 0) }
            )),
        ]),
        // Strategy
        Line::from(vec![
            Span::styled("Strategy ", Style::default().fg(Color::Rgb(0, 180, 0))),
            Span::styled(&ts.meta_learning_strategy, Style::default().fg(Color::Rgb(0, 255, 136)).add_modifier(Modifier::BOLD)),
        ]),
        // Metrics row
        Line::from(vec![
            Span::styled("Complexity ", Style::default().fg(Color::Rgb(0, 120, 0))),
            Span::styled(format!("{:.2}", ts.nca_complexity), Style::default().fg(Color::White)),
            Span::raw(" "),
            Span::styled("Diversity ", Style::default().fg(Color::Rgb(0, 120, 0))),
            Span::styled(format!("{:.2}", ts.nca_diversity), Style::default().fg(Color::White)),
        ]),
    ];

    let panel = Paragraph::new(lines)
        .block(Block::default()
            .title(" Metrics ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Rgb(0, 255, 0))));

    frame.render_widget(panel, area);
}

/// Training progress: mastery gauge + recent events
fn render_training_progress(frame: &mut Frame, area: Rect, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(35),  // Progress bars
            Constraint::Percentage(65),  // Recent events
        ])
        .split(area);

    render_progress_bars(frame, chunks[0], state);
    render_recent_events(frame, chunks[1], state);
}

/// Progress bars with gmork theme
fn render_progress_bars(frame: &mut Frame, area: Rect, state: &AppState) {
    let ts = &state.training_state;

    // Build custom ASCII progress bars
    let mastery_ratio = (ts.low_loss_streak as f64 / 50.0).min(1.0);
    let mastery_filled = (mastery_ratio * 20.0) as usize;
    let mastery_bar = format!("[{}{}]", "█".repeat(mastery_filled), "░".repeat(20 - mastery_filled));
    let mastery_color = if mastery_ratio >= 1.0 { Color::Rgb(0, 255, 0) } else { Color::Rgb(0, 255, 136) };

    let attempts_ratio = (ts.pattern_attempts as f64 / 100.0).min(1.0);
    let attempts_filled = (attempts_ratio * 20.0) as usize;
    let attempts_bar = format!("[{}{}]", "█".repeat(attempts_filled), "░".repeat(20 - attempts_filled));
    let attempts_color = if attempts_ratio >= 1.0 { Color::Rgb(255, 0, 0) } else { Color::Rgb(255, 136, 0) };

    let lines = vec![
        Line::from(vec![
            Span::styled("Mastery ", Style::default().fg(Color::Rgb(0, 180, 0))),
            Span::styled(format!("{}/50", ts.low_loss_streak.min(50)), Style::default().fg(mastery_color).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(Span::styled(&mastery_bar, Style::default().fg(mastery_color))),
        Line::from(""),
        Line::from(vec![
            Span::styled("Attempts ", Style::default().fg(Color::Rgb(0, 180, 0))),
            Span::styled(format!("{}/100", ts.pattern_attempts.min(100)), Style::default().fg(attempts_color).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(Span::styled(&attempts_bar, Style::default().fg(attempts_color))),
    ];

    let panel = Paragraph::new(lines)
        .block(Block::default()
            .title(" Progress ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Rgb(0, 255, 0))));

    frame.render_widget(panel, area);
}

/// Recent training events with gmork colors
fn render_recent_events(frame: &mut Frame, area: Rect, state: &AppState) {
    let ts = &state.training_state;

    let mut lines: Vec<Line> = ts.recent_events.iter()
        .rev()
        .take(5)
        .map(|event| {
            let (icon, color) = if event.contains("[OK]") || event.contains("mastered") {
                ("✓", Color::Rgb(0, 255, 0))  // gmork_green
            } else if event.contains("[SYNC]") || event.contains("Strategy") {
                ("⟳", Color::Rgb(0, 255, 136))  // eerie_green
            } else if event.contains("[TARGET]") || event.contains("Pattern") {
                ("◎", Color::Rgb(136, 255, 0))  // bright_green
            } else if event.contains("LR") || event.contains("learning") {
                ("λ", Color::Rgb(255, 136, 0))  // ember_orange
            } else if event.contains("error") || event.contains("fail") {
                ("✗", Color::Rgb(255, 0, 0))  // fire_red
            } else {
                ("•", Color::Rgb(0, 180, 0))  // dim green
            };
            Line::from(vec![
                Span::styled(format!("{} ", icon), Style::default().fg(color)),
                Span::styled(event.chars().take(50).collect::<String>(), Style::default().fg(Color::Rgb(0, 200, 0))),
            ])
        })
        .collect();

    if lines.is_empty() {
        lines.push(Line::from(Span::styled(
            "Waiting for training events...",
            Style::default().fg(Color::Rgb(0, 100, 0))
        )));
    }

    let panel = Paragraph::new(lines)
        .block(Block::default()
            .title(" Events ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Rgb(0, 255, 0))));

    frame.render_widget(panel, area);
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
        crate::tui::app::ConsciousnessState::Active => ("ACTIVE", Color::Green, ""),
        crate::tui::app::ConsciousnessState::Dreaming => ("DREAMING", Color::Magenta, ""),
        crate::tui::app::ConsciousnessState::Curious => ("CURIOUS", Color::Yellow, "🔍"),
    };

    // Show vision status
    let vision_status = if state.camera_frame.is_some() {
        Span::styled(" |  VISION ACTIVE", Style::default().fg(Color::Cyan))
    } else {
        Span::styled(" |  Vision: Standby", Style::default().fg(Color::DarkGray))
    };

    // Show audio output status
    let audio_status = if state.audio_available {
        if state.audio_enabled {
            let vol_pct = (state.audio_volume * 100.0) as u8;
            Span::styled(format!(" | 🔊 {}%", vol_pct), Style::default().fg(Color::Green))
        } else {
            Span::styled(" | 🔇 Muted", Style::default().fg(Color::Red))
        }
    } else {
        Span::styled(" | 🔇 No Audio", Style::default().fg(Color::DarkGray))
    };

    // Show audio input (listening) status
    let listen_status = if state.audio_input_available {
        if state.audio_listening {
            Span::styled(" | 👂 Listening", Style::default().fg(Color::Cyan))
        } else {
            Span::styled(" | 👂 Off", Style::default().fg(Color::DarkGray))
        }
    } else {
        Span::styled("", Style::default())  // Don't show if input not available
    };

    let title = Paragraph::new(Line::from(vec![
        Span::styled("", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
        Span::styled("SAGE DASHBOARD", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw("  │  "),
        Span::styled(format!("{} Experiences", total_exp), Style::default().fg(Color::Yellow)),
        Span::raw("  │  "),
        Span::styled(format!("{} {}", consciousness_emoji, consciousness_text), Style::default().fg(consciousness_color).add_modifier(Modifier::BOLD)),
        Span::raw("  │  "),
        Span::styled(format!("{}", opinion), Style::default().fg(opinion_color)),
        vision_status,
        audio_status,
        listen_status,
    ]))
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Cyan))
    );

    frame.render_widget(title, area);
}

/// Render NCA grid showing neural processing with inline legend
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

        // Add inline legend at the bottom - VIBRANT gmork neon colors
        grid_text.push(Line::from("")); // Spacer
        grid_text.push(Line::from(vec![
            Span::styled("░░", Style::default().fg(Color::Rgb(0, 60, 0))),
            Span::styled(" Dead ", Style::default().fg(Color::Rgb(0, 255, 0))),
            Span::styled("▓▓", Style::default().fg(Color::Rgb(0, 120, 30))),
            Span::styled(" Low ", Style::default().fg(Color::Rgb(0, 255, 0))),
            Span::styled("██", Style::default().fg(Color::Rgb(0, 255, 0))),
            Span::styled(" Active ", Style::default().fg(Color::Rgb(0, 255, 0))),
            Span::styled("██", Style::default().fg(Color::Rgb(255, 136, 0))),
            Span::styled(" Firing ", Style::default().fg(Color::Rgb(0, 255, 0))),
            Span::styled("██", Style::default().fg(Color::Rgb(255, 0, 0))),
            Span::styled(" Max", Style::default().fg(Color::Rgb(0, 255, 0))),
        ]));

        // Show active concepts
        let concepts_display = if !snapshot.active_concepts.is_empty() {
            format!("Active: {}", snapshot.active_concepts.join(", "))
        } else {
            "No active concepts".to_string()
        };

        let block_title = format!("NCA Processing {}×{} │ Gen: {} │ {}",
            grid_size, grid_size, snapshot.generation, concepts_display);

        let grid = Paragraph::new(grid_text)
            .block(
                Block::default()
                    .title(block_title)
                    .borders(Borders::ALL)
                    .style(Style::default().fg(Color::Rgb(0, 255, 0))) // gmork_green border
            )
            .alignment(Alignment::Center);

        frame.render_widget(grid, area);
    } else {
        // Show legend even when waiting - VIBRANT gmork colors
        let waiting = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled("⏳ Waiting for neural activity...",
                Style::default().fg(Color::Rgb(0, 255, 0)).add_modifier(Modifier::BOLD))),
            Line::from(""),
            Line::from(Span::styled("Start a conversation on IRC to see SAGE's brain form patterns!",
                Style::default().fg(Color::Rgb(0, 180, 0)))),
            Line::from(""),
            Line::from(vec![
                Span::styled("░░", Style::default().fg(Color::Rgb(0, 60, 0))),
                Span::styled(" Dead ", Style::default().fg(Color::Rgb(0, 255, 0))),
                Span::styled("▓▓", Style::default().fg(Color::Rgb(0, 120, 30))),
                Span::styled(" Low ", Style::default().fg(Color::Rgb(0, 255, 0))),
                Span::styled("██", Style::default().fg(Color::Rgb(0, 255, 0))),
                Span::styled(" Active ", Style::default().fg(Color::Rgb(0, 255, 0))),
                Span::styled("██", Style::default().fg(Color::Rgb(255, 136, 0))),
                Span::styled(" Firing ", Style::default().fg(Color::Rgb(0, 255, 0))),
                Span::styled("██", Style::default().fg(Color::Rgb(255, 0, 0))),
                Span::styled(" Max", Style::default().fg(Color::Rgb(0, 255, 0))),
            ]),
        ])
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .title("NCA Processing 32×32")
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::Rgb(0, 255, 0)))
        );

        frame.render_widget(waiting, area);
    }
}

/// Compact status for minimal mode
fn render_compact_status(frame: &mut Frame, area: Rect, state: &AppState) {
    let consciousness_emoji = match state.consciousness_state {
        crate::tui::app::ConsciousnessState::Active => "",
        crate::tui::app::ConsciousnessState::Dreaming => "",
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
/// Uses gmork neovim theme colors - VIBRANT neon aesthetic
/// Legend: Dead → Low → Active → Firing → Max
fn alpha_to_style_animated(alpha: f64, cell_phase: f64, activity_boost: f64) -> (&'static str, Color) {
    // Gmork theme - FULL VIBRANCY:
    // void = #0d0d0d (13,13,13) - true black
    // toxic_green = #00ff44 (0,255,68) - neon green
    // gmork_green = #00ff00 (0,255,0) - PURE electric green
    // eerie_green = #00ff88 (0,255,136) - cyan-green
    // bright_green = #88ff00 (136,255,0) - lime
    // ember_orange = #ff8800 (255,136,0)
    // fire_red = #ff0000 (255,0,0)
    // bone_white = #ffffff (255,255,255)

    let pulse = (cell_phase.sin() * 0.15 + 1.0).max(0.85);
    let shimmer = (cell_phase * 3.7).sin() * 0.08;
    let adjusted_alpha = ((alpha * pulse) + shimmer + (activity_boost * 0.4)).clamp(0.0, 1.0);
    let firing_intensity = activity_boost;

    // Priority 1: Firing states (activity boost overrides alpha)
    if firing_intensity > 0.5 {
        // Max firing - bone_white with glow
        ("█", Color::Rgb(255, 255, 255))
    } else if firing_intensity > 0.2 {
        // Firing - ember_orange BRIGHT
        ("█", Color::Rgb(255, 136, 0))
    }
    // Priority 2: Alpha-based gradient - ALL NEON GREENS
    else if adjusted_alpha < 0.08 {
        // Dead - void black
        ("░", Color::Rgb(13, 13, 13))
    } else if adjusted_alpha < 0.2 {
        // Dim - very dark green (not gray!)
        ("▒", Color::Rgb(0, 60, 0))
    } else if adjusted_alpha < 0.35 {
        // Low - dark toxic green
        ("▓", Color::Rgb(0, 120, 30))
    } else if adjusted_alpha < 0.5 {
        // Emerging - toxic_green
        ("█", Color::Rgb(0, 255, 68))
    } else if adjusted_alpha < 0.65 {
        // Active - PURE gmork_green (the signature color!)
        ("█", Color::Rgb(0, 255, 0))
    } else if adjusted_alpha < 0.8 {
        // High - eerie_green (cyan tint)
        ("█", Color::Rgb(0, 255, 136))
    } else if adjusted_alpha < 0.92 {
        // Very high - bright_green (lime)
        ("█", Color::Rgb(136, 255, 0))
    } else {
        // Maximum - fire_red
        ("█", Color::Rgb(255, 0, 0))
    }
}
