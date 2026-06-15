//! sage-tui — Live SAGE Training & Node Monitor
//!
//! A Ratatui-based terminal dashboard that shows:
//! - Training progress: loss sparkline, accuracy gauge, epoch progress
//! - Grid heatmap: live visualization of NCA cell activations
//! - Node status: peers, knowledge stats, specialists
//!
//! Reads live state from ~/.sage/training_state.json (written by gpu-train).
//! Auto-detects whether training is running and switches views.

use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::{Line, Span, Text},
    widgets::{Block, Borders, Gauge, Paragraph, Sparkline},
    Frame, Terminal,
};
use std::fs;
use std::io;
use std::path::PathBuf;
use std::time::{Duration, Instant};

// ── Training State ─────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
struct TrainingState {
    running: bool,
    current_epoch: usize,
    total_epochs: usize,
    losses: Vec<f64>,
    accuracies: Vec<f64>,
    best_accuracy: f64,
    random_baseline: f64,
    grid_size: usize,
    vocab_size: usize,
    param_count: usize,
    elapsed_secs: f64,
    grid_frames: Vec<Vec<Vec<f64>>>,
    updated_at: String,
}

fn read_training_state() -> Option<TrainingState> {
    let path = state_path();
    let json = fs::read_to_string(&path).ok()?;
    let v: serde_json::Value = serde_json::from_str(&json).ok()?;
    Some(TrainingState {
        running: v["running"].as_bool().unwrap_or(false),
        current_epoch: v["current_epoch"].as_u64().unwrap_or(0) as usize,
        total_epochs: v["total_epochs"].as_u64().unwrap_or(0) as usize,
        losses: v["losses"].as_array().map(|a| a.iter().filter_map(|x| x.as_f64()).collect()).unwrap_or_default(),
        accuracies: v["accuracies"].as_array().map(|a| a.iter().filter_map(|x| x.as_f64()).collect()).unwrap_or_default(),
        best_accuracy: v["best_accuracy"].as_f64().unwrap_or(0.0),
        random_baseline: v["random_baseline"].as_f64().unwrap_or(0.0),
        grid_size: v["grid_size"].as_u64().unwrap_or(0) as usize,
        vocab_size: v["vocab_size"].as_u64().unwrap_or(0) as usize,
        param_count: v["param_count"].as_u64().unwrap_or(0) as usize,
        elapsed_secs: v["elapsed_secs"].as_f64().unwrap_or(0.0),
        grid_frames: v["grid_frames"].as_array().map(|frames| {
            frames.iter().map(|frame| {
                frame.as_array().map(|rows| {
                    rows.iter().map(|row| {
                        row.as_array().map(|cells| {
                            cells.iter().filter_map(|c| c.as_f64()).collect()
                        }).unwrap_or_default()
                    }).collect()
                }).unwrap_or_default()
            }).collect()
        }).unwrap_or_default(),
        updated_at: v["updated_at"].as_str().unwrap_or("").to_string(),
    })
}

fn state_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_default()
        .join(".sage")
        .join("training_state.json")
}

// ── Grid Heatmap ───────────────────────────────────────────────────────────

/// Render a mini NCA grid heatmap using half-block characters (▀).
/// This is the standard ratatui technique for pixel rendering — each terminal
/// cell shows two rows of data via fg/bg colors on the half-block glyph.
/// Falls back to a synthetic wave pattern when no snapshot data is available.
fn render_grid_heatmap(area: Rect, f: &mut Frame, snapshot: &[Vec<f64>]) {
    let buf = f.buffer_mut();
    let inner_w = area.width as usize;
    let inner_h = area.height as usize;
    if inner_w == 0 || inner_h == 0 {
        return;
    }

    if snapshot.is_empty() || snapshot[0].is_empty() {
        // Synthetic wave fallback
        let t = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH).unwrap().as_secs_f64();
        for y in 0..inner_h {
            for x in 0..inner_w {
                let px = area.x + x as u16;
                let py = area.y + y as u16;
                // Two rows per cell: top = y*2, bottom = y*2+1
                let top_wave = ((y * 2) as f64 * 0.3 + x as f64 * 0.2 + t * 0.5).sin() * 0.5 + 0.5;
                let bot_wave = ((y * 2 + 1) as f64 * 0.3 + x as f64 * 0.2 + t * 0.5).sin() * 0.5 + 0.5;
                let fg = wave_to_color(top_wave.clamp(0.0, 1.0));
                let bg = wave_to_color(bot_wave.clamp(0.0, 1.0));
                buf.get_mut(px, py).set_char('▀').set_fg(fg).set_bg(bg);
            }
        }
        return;
    }

    let src_h = snapshot.len();
    let src_w = snapshot[0].len();

    // Find min/max for color scaling
    let mut min_val = f64::MAX;
    let mut max_val = f64::MIN;
    for row in snapshot {
        for &val in row {
            if val.is_finite() {
                min_val = min_val.min(val);
                max_val = max_val.max(val);
            }
        }
    }
    if max_val <= min_val { max_val = min_val + 1.0; }

    // Use the known clamp bounds (-5.0, 5.0) as reference when data is uniform
    let ref_min = -5.0f64;
    let ref_max = 5.0f64;
    let ref_range = ref_max - ref_min;
    // Blend: use data range when it's wide enough, otherwise lean on reference
    let data_range = max_val - min_val;
    let blend = (data_range / ref_range).min(1.0);
    let eff_min = min_val * blend + ref_min * (1.0 - blend);
    let eff_max = max_val * blend + ref_max * (1.0 - blend);
    let eff_range = (eff_max - eff_min).max(0.001);

    // Each terminal row shows 2 data rows via half-block (fg=top, bg=bottom)
    let data_rows = inner_h * 2;
    let h_step = src_h / data_rows.max(1);
    let w_step = src_w / inner_w.max(1);

    for y in 0..inner_h {
        for x in 0..inner_w {
            let px = area.x + x as u16;
            let py = area.y + y as u16;

            // Top row of this cell
            let top_sr = ((y * 2) * h_step).min(src_h - 1);
            let top_sc = (x * w_step).min(src_w - 1);
            let top_val = snapshot[top_sr][top_sc];
            let top_act = ((top_val - eff_min) / eff_range).clamp(0.0, 1.0);

            // Bottom row of this cell
            let bot_sr = ((y * 2 + 1) * h_step).min(src_h - 1);
            let bot_sc = (x * w_step).min(src_w - 1);
            let bot_val = snapshot[bot_sr][bot_sc];
            let bot_act = ((bot_val - eff_min) / eff_range).clamp(0.0, 1.0);

            let fg = wave_to_color(top_act);
            let bg = wave_to_color(bot_act);
            buf.get_mut(px, py).set_char('▀').set_fg(fg).set_bg(bg);
        }
    }
}

/// Convert normalized activation [0,1] to a color.
/// 0.0 = dark blue (coldest in data), 1.0 = bright red (hottest in data).
/// The caller already normalizes using the actual min/max range.
fn wave_to_color(normalized: f64) -> Color {
    let t = normalized.clamp(0.0, 1.0);

    if t < 0.15 {
        // Coldest: very dark blue
        let b = (16.0 + 40.0 * (t / 0.15)) as u8;
        Color::Rgb(4, 4, b)
    } else if t < 0.35 {
        // Cool: blue → cyan
        let s = (t - 0.15) / 0.2;
        let g = (180.0 * s) as u8;
        let b = (56.0 + 199.0 * (1.0 - s)) as u8;
        Color::Rgb(0, g, b)
    } else if t < 0.55 {
        // Neutral: cyan → green → yellow
        let s = (t - 0.35) / 0.2;
        let r = (255.0 * s) as u8;
        let g = 255u8;
        let b = (255.0 * (1.0 - s)) as u8;
        Color::Rgb(r, g, b)
    } else if t < 0.75 {
        // Warm: yellow → orange
        let s = (t - 0.55) / 0.2;
        let r = 255u8;
        let g = (255.0 * (1.0 - s * 0.7)) as u8;
        Color::Rgb(r, g, 0)
    } else {
        // Hot: orange → red
        let s = (t - 0.75) / 0.25;
        let r = 255u8;
        let g = (179.0 * (1.0 - s)) as u8;
        Color::Rgb(r, g, 0)
    }
}

// ── Main TUI ───────────────────────────────────────────────────────────────

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let tick_rate = Duration::from_millis(150);  // ~7 fps for smooth animation
    let mut last_tick = Instant::now();
    let mut state: Option<TrainingState> = None;
    let mut last_update = Instant::now();
    let mut frame_idx: usize = 0;
    let mut frame_count: usize = 0;

    loop {
        // Poll for events
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    KeyCode::Char('r') => {
                        // Force refresh
                        state = read_training_state();
                        last_update = Instant::now();
                    }
                    _ => {}
                }
            }
        }

        // Tick: refresh state and animate frames
        if last_tick.elapsed() >= tick_rate {
            let new_state = read_training_state();
            let epoch_changed = match (&state, &new_state) {
                (Some(old), Some(new)) => old.current_epoch != new.current_epoch,
                _ => new_state.is_some(),
            };
            if new_state.is_some() {
                state = new_state;
                if epoch_changed {
                    frame_idx = 0; // reset animation on new epoch
                }
                last_update = Instant::now();
            }
            // Advance animation frame
            if let Some(ref s) = state {
                frame_count = s.grid_frames.len();
                if frame_count > 0 {
                    frame_idx = (frame_idx + 1) % frame_count;
                }
            }
            last_tick = Instant::now();
        }

        terminal.draw(|f| {
            let size = f.size();

            // Main layout: header, body (split), footer
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),  // header
                    Constraint::Min(10),    // body
                    Constraint::Length(1),  // footer
                ])
                .split(size);

            // ── Header ──────────────────────────────────────────────
            let header_text = if let Some(ref s) = state {
                if s.running {
                    format!(
                        " 🧬 SAGE Training Monitor — Epoch {}/{} — {}×{} Grid — {}K Params — {} Tokens ",
                        s.current_epoch, s.total_epochs,
                        s.grid_size, s.grid_size,
                        s.param_count / 1000,
                        s.vocab_size,
                    )
                } else {
                    " 🧬 SAGE Training Monitor — Idle (no training running) ".to_string()
                }
            } else {
                " 🧬 SAGE Training Monitor — Waiting for training to start... ".to_string()
            };

            let header = Paragraph::new(header_text)
                .style(Style::default().fg(Color::White).bg(Color::Rgb(20, 20, 40)))
                .block(Block::default().borders(Borders::NONE));
            f.render_widget(header, chunks[0]);

            // ── Body ────────────────────────────────────────────────
            let body_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(50),  // left: charts
                    Constraint::Percentage(50),  // right: grid + stats
                ])
                .split(chunks[1]);

            // Left side: loss sparkline + accuracy gauge + epoch progress
            let left_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(8),   // loss sparkline
                    Constraint::Length(4),   // accuracy gauge
                    Constraint::Length(3),   // epoch progress
                    Constraint::Min(0),      // stats
                ])
                .split(body_chunks[0]);

            // Right side: grid heatmap + info
            let right_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(12),     // grid heatmap
                    Constraint::Length(6),   // info panel
                ])
                .split(body_chunks[1]);

            if let Some(ref s) = state {
                // ── Loss Sparkline ──────────────────────────────────
                let loss_data: Vec<u64> = s.losses.iter()
                    .map(|l| (l * 1000.0) as u64)
                    .collect();

                // Zoom into actual loss range for visible variation
                let data_min = loss_data.iter().min().copied().unwrap_or(0);
                let data_max = loss_data.iter().max().copied().unwrap_or(1);
                // Add 5% padding so the line doesn't touch edges
                let range = (data_max - data_min).max(1);
                let pad = (range as f64 * 0.05) as u64;
                let spark_min = data_min.saturating_sub(pad);
                let spark_max = data_max + pad;

                // Shift data so spark_min is the baseline
                let shifted: Vec<u64> = loss_data.iter()
                    .map(|l| l.saturating_sub(spark_min))
                    .collect();

                let sparkline = Sparkline::default()
                    .block(
                        Block::default()
                            .title(format!(
                                " Loss Curve [{:.4} → {:.4}] ",
                                s.losses.first().copied().unwrap_or(0.0),
                                s.losses.last().copied().unwrap_or(0.0),
                            ))
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(Color::Cyan))
                    )
                    .data(&shifted)
                    .max(spark_max - spark_min)
                    .style(Style::default().fg(Color::Yellow));
                f.render_widget(sparkline, left_chunks[0]);

                // ── Accuracy Gauge ──────────────────────────────────
                let acc_pct = s.best_accuracy * 100.0;
                let random_pct = s.random_baseline * 100.0;
                let improvement = if s.random_baseline > 0.0 {
                    s.best_accuracy / s.random_baseline
                } else {
                    0.0
                };

                let gauge_label = format!(
                    " Top-5 Accuracy: {:.1}% (random: {:.2}%, {:.0}x improvement) ",
                    acc_pct, random_pct, improvement
                );
                let gauge = Gauge::default()
                    .block(
                        Block::default()
                            .title(" Accuracy ")
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(Color::Green))
                    )
                    .gauge_style(Style::default().fg(Color::Green).bg(Color::Rgb(20, 20, 40)))
                    .percent((acc_pct.min(100.0)) as u16)
                    .label(gauge_label);
                f.render_widget(gauge, left_chunks[1]);

                // ── Epoch Progress ──────────────────────────────────
                let epoch_pct = if s.total_epochs > 0 {
                    (s.current_epoch as f64 / s.total_epochs as f64 * 100.0) as u16
                } else {
                    0
                };

                let elapsed_min = s.elapsed_secs / 60.0;
                let est_total = if s.current_epoch > 0 {
                    s.elapsed_secs / s.current_epoch as f64 * s.total_epochs as f64 / 60.0
                } else {
                    0.0
                };

                let progress_label = format!(
                    " {}/{} epochs | {:.0}m elapsed | ~{:.0}m remaining ",
                    s.current_epoch, s.total_epochs, elapsed_min, est_total - elapsed_min
                );
                let progress = Gauge::default()
                    .block(
                        Block::default()
                            .title(" Progress ")
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(Color::Magenta))
                    )
                    .gauge_style(Style::default().fg(Color::Magenta).bg(Color::Rgb(20, 20, 40)))
                    .percent(epoch_pct)
                    .label(progress_label);
                f.render_widget(progress, left_chunks[2]);

                // ── Stats Panel ─────────────────────────────────────
                let stats_text = vec![
                    Line::from(vec![
                        Span::styled(" Grid:     ", Style::default().fg(Color::Gray)),
                        Span::styled(
                            format!("{}×{} ({} cells)", s.grid_size, s.grid_size, s.grid_size * s.grid_size),
                            Style::default().fg(Color::White),
                        ),
                    ]),
                    Line::from(vec![
                        Span::styled(" Params:   ", Style::default().fg(Color::Gray)),
                        Span::styled(
                            format!("{}K", s.param_count / 1000),
                            Style::default().fg(Color::White),
                        ),
                    ]),
                    Line::from(vec![
                        Span::styled(" Vocab:    ", Style::default().fg(Color::Gray)),
                        Span::styled(
                            format!("{} tokens", s.vocab_size),
                            Style::default().fg(Color::White),
                        ),
                    ]),
                    Line::from(vec![
                        Span::styled(" Best:     ", Style::default().fg(Color::Gray)),
                        Span::styled(
                            format!("{:.2}% top-5", s.best_accuracy * 100.0),
                            Style::default().fg(Color::Green),
                        ),
                    ]),
                    Line::from(vec![
                        Span::styled(" Updated:  ", Style::default().fg(Color::Gray)),
                        Span::styled(
                            &s.updated_at,
                            Style::default().fg(Color::DarkGray),
                        ),
                    ]),
                ];

                let stats = Paragraph::new(Text::from(stats_text))
                    .block(
                        Block::default()
                            .title(" Model Stats ")
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(Color::Blue))
                    );
                f.render_widget(stats, left_chunks[3]);

                // ── Grid Heatmap ────────────────────────────────────
                // Animate through NCA step frames
                let current_frame = if s.grid_frames.is_empty() {
                    &[][..]
                } else {
                    &s.grid_frames[frame_idx % s.grid_frames.len()]
                };
                let grid_block = Block::default()
                    .title(format!(
                        " NCA Grid — Step {}/{} — Activation Heatmap ",
                        if s.grid_frames.is_empty() { 0 } else { frame_idx + 1 },
                        s.grid_frames.len().max(1),
                    ))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Rgb(255, 128, 0)));
                let grid_area = grid_block.inner(right_chunks[0]);
                f.render_widget(grid_block, right_chunks[0]);
                render_grid_heatmap(grid_area, f, current_frame);

                // ── Info Panel ──────────────────────────────────────
                let info_text = vec![
                    Line::from(vec![
                        Span::styled(" 🟦 Cold ", Style::default().fg(Color::Blue)),
                        Span::raw(" → "),
                        Span::styled("⬜ Neutral", Style::default().fg(Color::Gray)),
                        Span::raw(" → "),
                        Span::styled("🟥 Hot", Style::default().fg(Color::Red)),
                    ]),
                    Line::from(""),
                    Line::from(vec![
                        Span::styled(" q/Esc ", Style::default().fg(Color::Yellow)),
                        Span::raw(" quit  "),
                        Span::styled(" r ", Style::default().fg(Color::Yellow)),
                        Span::raw(" refresh"),
                    ]),
                    Line::from(""),
                    Line::from(vec![
                        Span::styled(
                            " Each cell represents a token position in the NCA grid.",
                            Style::default().fg(Color::DarkGray),
                        ),
                    ]),
                    Line::from(vec![
                        Span::styled(
                            " Hot cells = high activation (model is 'thinking' about those tokens).",
                            Style::default().fg(Color::DarkGray),
                        ),
                    ]),
                ];

                let info = Paragraph::new(Text::from(info_text))
                    .block(
                        Block::default()
                            .title(" Legend ")
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(Color::DarkGray))
                    );
                f.render_widget(info, right_chunks[1]);
            } else {
                // No training state — show waiting screen
                let waiting = Paragraph::new(
                    "Waiting for training to start...\n\n\
                     Run: gpu-train --curriculum curricula/junior-react-dev.json\n\
                     The TUI will auto-detect and display live progress."
                )
                .style(Style::default().fg(Color::DarkGray))
                .block(
                    Block::default()
                        .title(" SAGE Training Monitor ")
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::DarkGray))
                );
                f.render_widget(waiting, body_chunks[0]);

                // Still show a grid on the right for visual interest
                let grid_block = Block::default()
                    .title(" NCA Grid (idle) ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::DarkGray));
                let grid_area = grid_block.inner(body_chunks[1]);
                f.render_widget(grid_block, body_chunks[1]);
                render_grid_heatmap(grid_area, f, &[]);
            }

            // ── Footer ──────────────────────────────────────────────
            let footer = Paragraph::new(
                Line::from(vec![
                    Span::styled(" q ", Style::default().fg(Color::Black).bg(Color::Yellow)),
                    Span::raw(" quit  "),
                    Span::styled(" r ", Style::default().fg(Color::Black).bg(Color::Yellow)),
                    Span::raw(" refresh  "),
                    Span::raw(" | "),
                    Span::styled(
                        format!(" {}", chrono::Local::now().format("%H:%M:%S")),
                        Style::default().fg(Color::DarkGray),
                    ),
                ])
            )
            .style(Style::default().bg(Color::Rgb(20, 20, 40)));
            f.render_widget(footer, chunks[2]);
        })?;
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}
