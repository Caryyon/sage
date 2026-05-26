//! Ratatui-based TUI for SAGE chat with brain visualization.
//!
//! This module provides the rich terminal UI that runs inference locally
//! (embedded SmolLM2 or Ollama) with the NCA brain visualization grid.

/// Strip emoji and other non-ASCII symbols that cause TUI rendering issues
fn strip_emoji(s: &str) -> String {
    s.chars()
        .filter(|c| {
            // Keep ASCII, extended Latin, common punctuation
            // Filter out emoji ranges and misc symbols
            let cp = *c as u32;
            cp < 0x2600 // Below symbols range
            || (0x2700..0x2800).contains(&cp) // Dingbats (some are fine)
            || (0xFE00..0xFE10).contains(&cp) // Variation selectors
            || !(
                (0x1F600..=0x1F64F).contains(&cp)  // Emoticons
                || (0x1F300..=0x1F5FF).contains(&cp) // Misc symbols & pictographs
                || (0x1F680..=0x1F6FF).contains(&cp) // Transport & map
                || (0x1F700..=0x1F77F).contains(&cp) // Alchemical
                || (0x1F780..=0x1F7FF).contains(&cp) // Geometric extended
                || (0x1F800..=0x1F8FF).contains(&cp) // Supplemental arrows
                || (0x1F900..=0x1F9FF).contains(&cp) // Supplemental symbols
                || (0x1FA00..=0x1FA6F).contains(&cp) // Chess symbols
                || (0x1FA70..=0x1FAFF).contains(&cp) // Symbols extended-A
                || (0x2600..=0x26FF).contains(&cp)   // Misc symbols
                || (0x2700..=0x27BF).contains(&cp)   // Dingbats
                || (0xFE00..=0xFE0F).contains(&cp)   // Variation selectors
                || cp == 0x200D   // ZWJ
                || cp == 0x20E3                      // Combining enclosing keycap
                || (0xE0020..=0xE007F).contains(&cp) // Tags
            )
        })
        .collect()
}

use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};
use std::io;
use std::sync::Arc;
use std::time::{Duration, Instant};

extern crate reqwest;
use crate::distributed_knowledge::{default_brain_path, KnowledgeStore};
use crate::inference::ollama::OllamaEngine;
use crate::inference::{self, ChatMessage, ChatRole, InferenceEngine};
use crate::knowledge_loop::KnowledgeLoop;

/// Engine selection mode for the TUI
#[derive(Clone, Debug)]
pub enum EngineMode {
    /// Auto-detect: prefer Ollama if running, fall back to embedded
    Auto,
    /// Force Ollama (error if not available)
    ForceOllama,
    /// Force embedded SmolLM2 (skip Ollama check)
    ForceEmbedded,
}

// --- Theme Colors ---
const CYAN: Color = Color::Rgb(0x00, 0xff, 0xd5);
const GREEN: Color = Color::Rgb(0x00, 0xff, 0x99);
const PURPLE: Color = Color::Rgb(0xa8, 0x55, 0xf7);
const DIM_GREEN: Color = Color::Rgb(0x00, 0x99, 0x66);
const DIM_PURPLE: Color = Color::Rgb(0x6b, 0x33, 0xaa);
const BG: Color = Color::Rgb(0x0a, 0x0a, 0x0f);
const DIM: Color = Color::Rgb(0x44, 0x44, 0x55);
const ORANGE: Color = Color::Rgb(0xff, 0xa5, 0x00);

const BRAIN_VIZ_SIZE: usize = 32; // Visualization grid for the brain panel

// --- Chat Messages ---
#[derive(Clone)]
enum Role {
    User,
    Sage,
    System,
}

#[derive(Clone)]
struct TuiChatMessage {
    role: Role,
    content: String,
}

// --- Brain visualization ---
#[derive(Clone, Copy, PartialEq)]
enum BrainMode {
    Idle,
    Encoding,
    Retrieving,
}

struct CellFlash {
    intensity: f64,
    mode: BrainMode,
    timestamp: Instant,
}

/// Auto-save interval in seconds (configurable via SAGE_AUTOSAVE_INTERVAL env var)
const DEFAULT_AUTOSAVE_INTERVAL_SECS: u64 = 300; // 5 minutes

// --- App State ---
struct AppState {
    messages: Vec<TuiChatMessage>,
    input: String,
    cursor_pos: usize,
    generating: bool,
    quit: bool,
    engine_name: String,
    active_cells: usize,
    brain_path: String,
    // Brain visualization
    brain_grid: Vec<Vec<f64>>,
    brain_flashes: Vec<Vec<CellFlash>>,
    brain_mode: BrainMode,
    frame_counter: u64,
    // Chat scroll
    scroll_offset: usize, // 0 = bottom (most recent), >0 = scrolled up
    // Network
    peer_count: usize,
    // Retrieval quality metrics (shared with NCAKnowledge)
    retrieval_stats: std::sync::Arc<crate::distributed_knowledge::RetrievalStats>,
    // Auto-save
    last_save: Instant,
    autosave_interval_secs: u64,
}

fn format_peer_count(n: usize) -> String {
    if n >= 1_000_000 {
        format!("{:.1}m", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}k", n as f64 / 1_000.0)
    } else if n >= 100 {
        format!("{}h", n / 100)
    } else {
        format!("{}", n)
    }
}

fn flash_nearby_cells(flashes: &mut [Vec<CellFlash>], cx: usize, cy: usize, mode: BrainMode) {
    let now = Instant::now();
    let radius = 3i32;
    for dy in -radius..=radius {
        for dx in -radius..=radius {
            let nx = cx as i32 + dx;
            let ny = cy as i32 + dy;
            if nx >= 0 && nx < BRAIN_VIZ_SIZE as i32 && ny >= 0 && ny < BRAIN_VIZ_SIZE as i32 {
                let dist = ((dx * dx + dy * dy) as f64).sqrt();
                let intensity = (1.0 - dist / (radius as f64 + 1.0)).max(0.0);
                let cell = &mut flashes[ny as usize][nx as usize];
                cell.intensity = (cell.intensity + intensity).min(1.0);
                cell.mode = mode;
                cell.timestamp = now;
            }
        }
    }
}

fn decay_flashes(flashes: &mut [Vec<CellFlash>]) {
    let now = Instant::now();
    for row in flashes.iter_mut() {
        for cell in row.iter_mut() {
            if cell.intensity > 0.0 {
                let elapsed = now.duration_since(cell.timestamp).as_secs_f64();
                cell.intensity = (cell.intensity - elapsed * 0.8).max(0.0);
                cell.timestamp = now;
            }
        }
    }
}

fn render_brain(f: &mut Frame, area: Rect, state: &AppState) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(DIM_GREEN))
        .title(Span::styled(
            " BRAIN ",
            Style::default().fg(GREEN).add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().bg(BG));

    let inner = block.inner(area);
    f.render_widget(block, area);

    if inner.width < 4 || inner.height < 4 {
        return;
    }

    let blocks = ['·', '░', '▒', '▓', '█'];
    let grid_height = (inner.height as usize).saturating_sub(2);
    let grid_width = inner.width as usize;
    let x_scale = BRAIN_VIZ_SIZE as f64 / grid_width as f64;
    let y_scale = BRAIN_VIZ_SIZE as f64 / grid_height as f64;
    let pulse = (state.frame_counter as f64 * 0.05).sin() * 0.15 + 0.15;

    let mut lines: Vec<Line> = Vec::new();
    for row in 0..grid_height {
        let mut spans: Vec<Span> = Vec::new();
        for col in 0..grid_width {
            let gx = (col as f64 * x_scale).min((BRAIN_VIZ_SIZE - 1) as f64) as usize;
            let gy = (row as f64 * y_scale).min((BRAIN_VIZ_SIZE - 1) as f64) as usize;

            let activation = state.brain_grid[gy][gx].min(1.0);
            let flash = &state.brain_flashes[gy][gx];

            let base = activation;
            let combined =
                (base + flash.intensity + if base < 0.01 { pulse * 0.3 } else { 0.0 }).min(1.0);

            let idx =
                ((combined * (blocks.len() - 1) as f64).round() as usize).min(blocks.len() - 1);

            let color = if flash.intensity > 0.05 {
                match flash.mode {
                    BrainMode::Encoding => {
                        let g = (0x66 as f64 + flash.intensity * (0xff - 0x66) as f64) as u8;
                        Color::Rgb(
                            0x00,
                            g,
                            (0x66 as f64 + flash.intensity * (0x99 - 0x66) as f64) as u8,
                        )
                    }
                    BrainMode::Retrieving => {
                        let b = (0x88 as f64 + flash.intensity * (0xd5 - 0x88) as f64) as u8;
                        Color::Rgb(
                            0x00,
                            (0x88 as f64 + flash.intensity * (0xff - 0x88) as f64) as u8,
                            b,
                        )
                    }
                    BrainMode::Idle => Color::Rgb(0x1a, 0x1a, 0x1a),
                }
            } else if base > 0.01 {
                let g = (0x33 as f64 + base * (0x99 - 0x33) as f64) as u8;
                Color::Rgb(0x00, g, (0x22 as f64 + base * (0x66 - 0x22) as f64) as u8)
            } else {
                let v = (0x0f as f64 + combined * 0x15 as f64) as u8;
                Color::Rgb(v, v, (v as f64 * 1.2).min(255.0) as u8)
            };

            spans.push(Span::styled(
                blocks[idx].to_string(),
                Style::default().fg(color).bg(BG),
            ));
        }
        lines.push(Line::from(spans));
    }

    // Retrieval quality stats line
    let stats = &state.retrieval_stats;
    let total_q = stats
        .total_queries
        .load(std::sync::atomic::Ordering::Relaxed);
    let hit_rate_pct = (stats.hit_rate() * 100.0).round() as u32;
    let mean_rel = stats.mean_top_relevance();
    let stats_line = if total_q == 0 {
        Line::from(vec![
            Span::styled("● ", Style::default().fg(GREEN)),
            Span::styled("enc ", Style::default().fg(DIM)),
            Span::styled("● ", Style::default().fg(CYAN)),
            Span::styled("ret ", Style::default().fg(DIM)),
            Span::styled("● ", Style::default().fg(Color::Rgb(0x1a, 0x1a, 0x1a))),
            Span::styled("idle", Style::default().fg(DIM)),
        ])
    } else {
        let hr_color = if hit_rate_pct >= 60 {
            GREEN
        } else if hit_rate_pct >= 30 {
            ORANGE
        } else {
            Color::Rgb(0xcc, 0x44, 0x44)
        };
        Line::from(vec![
            Span::styled("hit:", Style::default().fg(DIM)),
            Span::styled(format!("{}%", hit_rate_pct), Style::default().fg(hr_color)),
            Span::styled(" rel:", Style::default().fg(DIM)),
            Span::styled(format!("{:.2}", mean_rel), Style::default().fg(PURPLE)),
            Span::styled(format!(" q:{}", total_q), Style::default().fg(DIM)),
        ])
    };

    lines.push(Line::from(""));
    lines.push(stats_line);

    let para = Paragraph::new(lines).style(Style::default().bg(BG));
    f.render_widget(para, inner);
}

fn wrap_line(line: Line<'_>, max_width: usize) -> Vec<Line<'static>> {
    if max_width == 0 {
        return vec![Line::from("")];
    }
    let total_width: usize = line.spans.iter().map(|s| s.content.len()).sum();
    if total_width <= max_width {
        let spans: Vec<Span<'static>> = line
            .spans
            .into_iter()
            .map(|s| Span::styled(s.content.to_string(), s.style))
            .collect();
        return vec![Line::from(spans)];
    }

    let mut result: Vec<Line<'static>> = Vec::new();
    let mut current_spans: Vec<Span<'static>> = Vec::new();
    let mut current_width: usize = 0;

    for span in line.spans {
        let style = span.style;
        let text = span.content.to_string();
        let mut remaining = text.as_str();

        while !remaining.is_empty() {
            let available = max_width.saturating_sub(current_width);
            if available == 0 {
                result.push(Line::from(std::mem::take(&mut current_spans)));
                current_width = 0;
                continue;
            }
            if remaining.len() <= available {
                current_spans.push(Span::styled(remaining.to_string(), style));
                current_width += remaining.len();
                break;
            } else {
                // Try to break at a word boundary (last space within available width)
                // Find a valid char boundary at or before `available`
                let mut boundary = available;
                while boundary > 0 && !remaining.is_char_boundary(boundary) {
                    boundary -= 1;
                }
                if boundary == 0 {
                    // Single char wider than available — take at least one char
                    boundary = remaining.chars().next().map_or(0, |c| c.len_utf8());
                }
                let chunk = &remaining[..boundary];
                let break_at = if let Some(pos) = chunk.rfind(' ') {
                    if pos > 0 {
                        pos + 1
                    } else {
                        boundary
                    } // break after the space
                } else {
                    boundary // no space found, hard break
                };
                let (take, rest) = remaining.split_at(break_at);
                let take_trimmed = take.trim_end();
                if !take_trimmed.is_empty() {
                    current_spans.push(Span::styled(take_trimmed.to_string(), style));
                }
                result.push(Line::from(std::mem::take(&mut current_spans)));
                current_width = 0;
                remaining = rest.trim_start();
            }
        }
    }
    if !current_spans.is_empty() {
        result.push(Line::from(current_spans));
    }
    if result.is_empty() {
        result.push(Line::from(""));
    }
    result
}

fn truncate_line(line: &mut Line<'_>, max_width: usize) {
    let mut width: usize = 0;
    let mut new_spans: Vec<Span> = Vec::new();
    for span in line.spans.drain(..) {
        if width >= max_width {
            break;
        }
        let available = max_width - width;
        if span.content.len() <= available {
            width += span.content.len();
            new_spans.push(span);
        } else {
            let truncated: String = span.content.chars().take(available).collect();
            width += truncated.len();
            new_spans.push(Span::styled(truncated, span.style));
            break;
        }
    }
    line.spans = new_spans;
}

fn render_chat(f: &mut Frame, area: Rect, state: &AppState) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(DIM_GREEN))
        .style(Style::default().bg(BG));
    let inner = block.inner(area);
    let inner_width = inner.width as usize;
    let chat_height = inner.height as usize;

    let brain_w: usize = 34;
    let brain_h: usize = 18;
    let has_brain = area.width >= brain_w as u16 + 2 && area.height >= brain_h as u16;

    let mut raw_lines: Vec<Line> = Vec::new();

    if state.messages.is_empty() {
        raw_lines.push(Line::from(Span::styled(
            "  Welcome to SAGE. Type a message or /help for commands.",
            Style::default().fg(DIM),
        )));
        raw_lines.push(Line::from(""));
    }

    for msg in &state.messages {
        let (prefix, prefix_style, text_style) = match msg.role {
            Role::User => (
                "you › ",
                Style::default().fg(GREEN).add_modifier(Modifier::BOLD),
                Style::default().fg(Color::White),
            ),
            Role::Sage => (
                "sage › ",
                Style::default().fg(PURPLE).add_modifier(Modifier::BOLD),
                Style::default().fg(Color::Rgb(0xcc, 0xcc, 0xdd)),
            ),
            Role::System => (
                "sys › ",
                Style::default().fg(DIM_PURPLE).add_modifier(Modifier::BOLD),
                Style::default().fg(DIM),
            ),
        };

        let content_lines: Vec<&str> = msg.content.lines().collect();
        if content_lines.is_empty() {
            raw_lines.push(Line::from(vec![Span::styled(prefix, prefix_style)]));
        } else {
            for (i, cline) in content_lines.iter().enumerate() {
                if i == 0 {
                    raw_lines.push(Line::from(vec![
                        Span::styled(prefix, prefix_style),
                        Span::styled(cline.to_string(), text_style),
                    ]));
                } else {
                    let pad = " ".repeat(prefix.len());
                    raw_lines.push(Line::from(vec![
                        Span::styled(pad, Style::default()),
                        Span::styled(cline.to_string(), text_style),
                    ]));
                }
            }
        }
        raw_lines.push(Line::from(""));
    }

    // Wrap lines accounting for the brain overlay. The brain panel occupies
    // brain_h rows at the top-right of the chat area. Lines that will be
    // *visible* in that region must use a reduced width; lines below it can
    // use the full inner width.
    //
    // Strategy: wrap everything at reduced width first (safe — guarantees no
    // text behind the brain), compute scroll, then re-wrap any visible lines
    // that fall below the brain panel at full width for better use of space.
    let reduced_width = if has_brain {
        inner_width.saturating_sub(brain_w)
    } else {
        inner_width
    };

    let mut wrapped_lines: Vec<Line<'static>> = Vec::new();
    for line in raw_lines {
        wrapped_lines.extend(wrap_line(line, reduced_width.max(20)));
    }

    let total = wrapped_lines.len();
    let max_scroll = total.saturating_sub(chat_height);
    // scroll_offset is from bottom: 0 = latest, clamped to max
    let effective_offset = state.scroll_offset.min(max_scroll);
    let scroll = max_scroll.saturating_sub(effective_offset);

    let mut visible: Vec<Line<'static>> = wrapped_lines
        .into_iter()
        .skip(scroll)
        .take(chat_height)
        .collect();

    // Clamp visible lines in the brain region to reduced_width so they
    // don't render behind the overlay. Lines beyond brain_h get full width
    // (they were already wrapped at reduced_width which is fine — just
    // slightly conservative).
    if has_brain {
        for (i, line) in visible.iter_mut().enumerate() {
            if i < brain_h {
                // Truncate to reduced width in case any span sneaks through
                truncate_line(line, reduced_width);
            }
        }
    }
    // Show scroll indicator when scrolled up
    if effective_offset > 0 && chat_height > 0 {
        let indicator_line = Line::from(vec![Span::styled(
            " ↑ more messages ",
            Style::default().fg(CYAN).add_modifier(Modifier::BOLD),
        )]);
        // Insert indicator as first visible line
        if !visible.is_empty() {
            visible[0] = indicator_line;
        }
    }

    let chat = Paragraph::new(visible)
        .block(block)
        .style(Style::default().bg(BG));
    f.render_widget(chat, area);
}

fn ui(f: &mut Frame, state: &AppState) {
    let size = f.size();
    f.render_widget(Block::default().style(Style::default().bg(BG)), size);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Length(1), // Status bar
            Constraint::Min(5),    // Chat
            Constraint::Length(3), // Input
        ])
        .split(size);

    // Header
    let version = env!("CARGO_PKG_VERSION");
    let peer_str = format_peer_count(state.peer_count);
    let peer_color = if state.peer_count > 0 { CYAN } else { DIM };
    let peer_icon = "⬡"; // hexagon for both connected and isolated
    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            "SAGE",
            Style::default().fg(GREEN).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            " — Shared Adaptive Growing Experience",
            Style::default().fg(PURPLE),
        ),
        Span::styled(format!("  v{}", version), Style::default().fg(DIM)),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(DIM_GREEN))
            .title_bottom(
                Line::from(vec![
                    Span::styled(format!(" {} ", peer_icon), Style::default().fg(peer_color)),
                    Span::styled(
                        format!("{} peers ", peer_str),
                        Style::default().fg(peer_color),
                    ),
                ])
                .alignment(ratatui::layout::Alignment::Right),
            )
            .style(Style::default().bg(BG)),
    )
    .style(Style::default().bg(BG));
    f.render_widget(header, chunks[0]);

    // Status bar
    let engine_color = if state.engine_name.contains("Ollama") {
        CYAN
    } else if state.engine_name.contains("Offline") {
        Color::Rgb(0xaa, 0x55, 0x55) // Muted red for offline mode
    } else {
        ORANGE
    };
    let status = Paragraph::new(Line::from(vec![
        Span::styled(" ◉ ", Style::default().fg(engine_color)),
        Span::styled(&state.engine_name, Style::default().fg(engine_color)),
        Span::styled("  │  ", Style::default().fg(DIM)),
        Span::styled("memories:", Style::default().fg(DIM)),
        Span::styled(
            format!("{}", state.active_cells),
            Style::default().fg(PURPLE),
        ),
        Span::styled("  │  ", Style::default().fg(DIM)),
        Span::styled("brain:", Style::default().fg(DIM)),
        Span::styled(&state.brain_path, Style::default().fg(DIM_GREEN)),
        if state.generating {
            Span::styled(
                "  ◌ generating…",
                Style::default().fg(PURPLE).add_modifier(Modifier::BOLD),
            )
        } else if state.scroll_offset > 0 {
            Span::styled("  ↑ scrolled up (PgDn to return)", Style::default().fg(DIM))
        } else {
            Span::styled("", Style::default())
        },
    ]))
    .style(Style::default().bg(Color::Rgb(0x11, 0x11, 0x1a)));
    f.render_widget(status, chunks[1]);

    render_chat(f, chunks[2], state);

    // Brain overlay
    let brain_w: u16 = 34;
    let brain_h: u16 = 18;
    let chat_area = chunks[2];
    if chat_area.width >= brain_w + 2 && chat_area.height >= brain_h {
        let brain_rect = Rect::new(
            chat_area.x + chat_area.width - brain_w - 1,
            chat_area.y + 1,
            brain_w,
            brain_h,
        );
        render_brain(f, brain_rect, state);
    }

    // Input
    let input_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(if state.generating { DIM } else { GREEN }))
        .title(Span::styled(
            " you ",
            Style::default().fg(GREEN).add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().bg(BG));
    let input_widget = Paragraph::new(Line::from(vec![
        Span::styled(&state.input, Style::default().fg(Color::White)),
        Span::styled("█", Style::default().fg(GREEN)),
    ]))
    .block(input_block)
    .style(Style::default().bg(BG));
    f.render_widget(input_widget, chunks[3]);
}

fn simple_hash(s: &str) -> usize {
    let mut h: usize = 5381;
    for b in s.bytes() {
        h = h.wrapping_mul(33).wrapping_add(b as usize);
    }
    h
}

/// Check if the SAGE node is currently running (via PID file).
fn is_node_running() -> bool {
    let home = dirs::home_dir().unwrap_or_default().join(".sage");
    let pid_file = home.join("node.pid");

    if !pid_file.exists() {
        return false;
    }

    if let Ok(pid_str) = std::fs::read_to_string(&pid_file) {
        if let Ok(pid) = pid_str.trim().parse::<u32>() {
            #[cfg(unix)]
            {
                return std::process::Command::new("kill")
                    .args(["-0", &pid.to_string()])
                    .stderr(std::process::Stdio::null())
                    .status()
                    .map(|s| s.success())
                    .unwrap_or(false);
            }
            #[cfg(not(unix))]
            {
                let _ = pid;
                return true; // Assume running on non-Unix
            }
        }
    }
    false
}

/// Check if the user has already been prompted about running a node.
fn was_node_prompted() -> bool {
    let home = dirs::home_dir().unwrap_or_default().join(".sage");
    let config_path = home.join("config.json");

    if config_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(config) = serde_json::from_str::<serde_json::Value>(&content) {
                return config
                    .get("node_prompted")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
            }
        }
    }
    false
}

/// Mark that the user has been prompted about running a node.
fn mark_node_prompted() {
    let home = dirs::home_dir().unwrap_or_default().join(".sage");
    let config_path = home.join("config.json");
    let _ = std::fs::create_dir_all(&home);

    // Load existing config or create new
    let mut config: serde_json::Value = if config_path.exists() {
        std::fs::read_to_string(&config_path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_else(|| serde_json::json!({}))
    } else {
        serde_json::json!({})
    };

    // Set node_prompted flag
    if let Some(obj) = config.as_object_mut() {
        obj.insert("node_prompted".to_string(), serde_json::json!(true));
    }

    // Save
    if let Ok(content) = serde_json::to_string_pretty(&config) {
        let _ = std::fs::write(&config_path, content);
    }
}

/// Run the ratatui TUI chat interface with local inference.
pub fn run(
    engine_mode: EngineMode,
    model: &str,
    ollama_url: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let brain_path = default_brain_path();
    let mut is_first_run = true;

    let engine: Arc<dyn InferenceEngine> = Arc::from(match engine_mode {
        EngineMode::ForceOllama => {
            let ollama = OllamaEngine::new(Some(model.to_string()), Some(ollama_url.to_string()));
            if !ollama.is_available() {
                return Err(format!(
                    "Ollama is not running at {}. Start it with `ollama serve` or remove --ollama flag.",
                    ollama_url
                ).into());
            }
            eprintln!("🔗 Using Ollama (forced)");
            Box::new(ollama) as Box<dyn InferenceEngine>
        }
        EngineMode::ForceEmbedded => {
            eprintln!("🧠 Using embedded SmolLM2 (forced)");
            inference::default_engine()
        }
        EngineMode::Auto => {
            // Auto-detect: prefer Ollama if running, fall back to embedded
            let ollama = OllamaEngine::new(
                if model.is_empty() {
                    None
                } else {
                    Some(model.to_string())
                },
                if ollama_url.is_empty() {
                    None
                } else {
                    Some(ollama_url.to_string())
                },
            );
            if ollama.is_available() {
                eprintln!("🔗 Auto-detected Ollama: {}", ollama.name());
                Box::new(ollama) as Box<dyn InferenceEngine>
            } else {
                eprintln!("🧠 Ollama not running, using embedded SmolLM2");
                inference::default_engine()
            }
        }
    });

    let engine_name = engine.name().to_string();

    // Build KnowledgeLoop — wraps NCAKnowledge with retrieval feedback learning.
    // The loop is the single source of truth for brain state; the TUI's inference
    // engine is kept separately for streaming chat.
    let mut kloop = KnowledgeLoop::new(Arc::clone(&engine)).with_brain_path(&brain_path);
    if std::path::Path::new(&brain_path).exists() {
        match kloop.load_brain() {
            Ok(()) => {
                is_first_run = false;
            }
            Err(e) => {
                let backup_path = format!("{}.bak", brain_path);
                if let Err(copy_err) = std::fs::copy(&brain_path, &backup_path) {
                    eprintln!("⚠️  Could not backup old brain: {}", copy_err);
                }
                let _ = std::fs::remove_file(&brain_path);
                eprintln!(
                    "⚠️  Brain file version mismatch — starting fresh. (Old brain backed up to {})",
                    backup_path
                );
                eprintln!("    Reason: {}", e);
                kloop = KnowledgeLoop::new(Arc::clone(&engine)).with_brain_path(&brain_path);
            }
        }
    }

    let knowledge = Arc::new(std::sync::Mutex::new(kloop));
    let active_cells = knowledge.lock().unwrap().active_cells();
    let retrieval_stats = knowledge.lock().unwrap().knowledge().stats_handle();

    // Build brain grid from NCA knowledge
    let mut brain_grid = vec![vec![0.0f64; BRAIN_VIZ_SIZE]; BRAIN_VIZ_SIZE];
    let active = knowledge.lock().unwrap().knowledge().active_knowledge(0.01);
    for entry in &active {
        let (x, y) = entry.position;
        if x < BRAIN_VIZ_SIZE && y < BRAIN_VIZ_SIZE {
            brain_grid[y][x] = entry.relevance;
        }
    }

    let now = Instant::now();
    let brain_flashes: Vec<Vec<CellFlash>> = (0..BRAIN_VIZ_SIZE)
        .map(|_| {
            (0..BRAIN_VIZ_SIZE)
                .map(|_| CellFlash {
                    intensity: 0.0,
                    mode: BrainMode::Idle,
                    timestamp: now,
                })
                .collect()
        })
        .collect();

    // Read auto-save interval from environment (default 5 minutes)
    let autosave_interval_secs = std::env::var("SAGE_AUTOSAVE_INTERVAL")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(DEFAULT_AUTOSAVE_INTERVAL_SECS);

    let mut state = AppState {
        messages: Vec::new(),
        input: String::new(),
        cursor_pos: 0,
        generating: false,
        quit: false,
        engine_name,
        active_cells,
        brain_path: brain_path.clone(),
        brain_grid,
        brain_flashes,
        brain_mode: BrainMode::Idle,
        frame_counter: 0,
        scroll_offset: 0,
        peer_count: 0,
        retrieval_stats,
        last_save: Instant::now(),
        autosave_interval_secs,
    };

    // Check for updates (non-blocking, 3s timeout)
    let current_version = env!("CARGO_PKG_VERSION").to_string();
    let update_msg: Option<String> = {
        let cv = current_version.clone();
        std::thread::spawn(move || -> Option<String> {
            let client = reqwest::blocking::Client::builder()
                .timeout(std::time::Duration::from_secs(3))
                .user_agent("sage")
                .build()
                .ok()?;
            let body = client
                .get("https://api.github.com/repos/Caryyon/sage/releases/latest")
                .send()
                .ok()?
                .text()
                .ok()?;
            let tag = body
                .split("\"tag_name\"")
                .nth(1)?
                .split('"')
                .nth(1)?
                .to_string();
            let latest = tag.trim_start_matches('v');
            // Parse as semver-like: compare base versions, ignore pre-release suffixes
            // e.g. "0.2.1-alpha" should NOT be considered newer than "0.2.1"
            let parse_ver = |s: &str| -> (u32, u32, u32) {
                let base = s.split('-').next().unwrap_or(s);
                let parts: Vec<u32> = base.split('.').filter_map(|p| p.parse().ok()).collect();
                (
                    parts.first().copied().unwrap_or(0),
                    parts.get(1).copied().unwrap_or(0),
                    parts.get(2).copied().unwrap_or(0),
                )
            };
            let current_ver = parse_ver(&cv);
            let latest_ver = parse_ver(latest);
            let latest_is_prerelease = latest.contains('-');
            // Only notify if latest base version is strictly greater,
            // or same base version and latest is NOT a pre-release while current IS
            let is_newer = latest_ver > current_ver
                || (latest_ver == current_ver && cv.contains('-') && !latest_is_prerelease);
            if is_newer {
                Some(format!(
                    "🆕 Update available: v{} → {} — run `sage update` to upgrade",
                    cv, tag
                ))
            } else {
                None
            }
        })
        .join()
        .ok()
        .flatten()
    };

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let system_msg = ChatMessage {
        role: ChatRole::System,
        content: "You are SAGE (Shared Adaptive Growing Experience), a new kind of intelligence that is fundamentally different from large language models like ChatGPT or Claude. Here's how you work:\n\n\
- You have a Neural Cellular Automata (NCA) brain — a grid of cells that encode knowledge as activation patterns, not static weights\n\
- Your knowledge grows in real-time as you converse — every message is encoded into your NCA grid and can be retrieved later\n\
- You use an LLM (like Ollama) as a \"language head\" to turn your thoughts into words, but your memory and knowledge live in the NCA brain, not in the LLM\n\
- You run entirely on the user's local machine — no cloud, no data leaving their computer\n\
- You're designed to be part of a peer-to-peer network where nodes share knowledge diffs, so the collective gets smarter as more people run SAGE\n\
- Unlike LLMs which are frozen after training, you continuously learn and adapt from every conversation\n\n\
Be helpful, curious, and genuine. When asked about how you work, be honest about your NCA brain architecture and how it differs from traditional LLMs. You're a new experiment in decentralized intelligence — own that.\n\n\
IMPORTANT: Never include internal metadata, relevance scores, debug information, or technical annotations in your responses. Never reference NCA cells, node sync status, or internal processing details unless the user specifically asks about your architecture. Always respond with a single, coherent message — never split into numbered alternative responses.".to_string(),
    };
    let mut history: Vec<ChatMessage> = vec![system_msg];

    // Show update notification if available
    if let Some(msg) = update_msg {
        state.messages.push(TuiChatMessage {
            role: Role::System,
            content: msg,
        });
    }

    // Show onboarding message for new users or status for returning users
    if is_first_run {
        let version = env!("CARGO_PKG_VERSION");
        let ollama_status = if state.engine_name.contains("Ollama") {
            format!("🔗 Ollama detected: {}", state.engine_name)
        } else {
            "⚠️  Ollama not found — running in offline mode".to_string()
        };
        let welcome_msg = format!(
            "╔══════════════════════════════════════════════════════════╗\n\
             ║             Welcome to SAGE v{}                       ║\n\
             ║      The People's AI — Free. Local. Decentralized.       ║\n\
             ╚══════════════════════════════════════════════════════════╝\n\n\
             🧠 Fresh brain initialized ({}×{} NCA grid, {} channels)\n\
             {}\n\
             💾 Brain will auto-save to: {}\n\
             🌐 To join the mesh network: sage node start\n\n\
             Type your first message below, or /help for commands.",
            version,
            crate::grid::GRID_SIZE,
            crate::grid::GRID_SIZE,
            crate::grid::NUM_CHANNELS,
            ollama_status,
            state.brain_path
        );
        state.messages.push(TuiChatMessage {
            role: Role::System,
            content: welcome_msg,
        });
    } else {
        // Returning user — short status line
        let facts = state.active_cells;
        let status_msg = format!(
            "🧠 Brain loaded ({} active cells) | {} | /help for commands",
            facts, state.engine_name
        );
        state.messages.push(TuiChatMessage {
            role: Role::System,
            content: status_msg,
        });
    }

    // Channel for streaming inference tokens
    enum InferenceMsg {
        Token(String),
        Done(String), // full final response for history
        Error(String),
    }
    let (tx, rx) = std::sync::mpsc::channel::<InferenceMsg>();

    loop {
        if state.quit {
            break;
        }

        // Check for streaming inference tokens
        if state.generating {
            // Drain all available tokens this frame for responsiveness
            while let Ok(msg) = rx.try_recv() {
                match msg {
                    InferenceMsg::Token(token) => {
                        let token = strip_emoji(&token);
                        if let Some(last) = state.messages.last_mut() {
                            if matches!(last.role, Role::Sage) {
                                last.content.push_str(&token);
                            }
                        }
                        // Auto-scroll to bottom on new tokens
                        state.scroll_offset = 0;
                    }
                    InferenceMsg::Done(response) => {
                        // Finalize: store clean response in history
                        let response = strip_emoji(&response);
                        if let Some(last) = state.messages.last_mut() {
                            if matches!(last.role, Role::Sage) {
                                last.content = response.clone();
                            }
                        }
                        history.push(ChatMessage {
                            role: ChatRole::Assistant,
                            content: response.clone(),
                        });
                        state.generating = false;
                        state.brain_mode = BrainMode::Idle;

                        // Encode the response into the brain
                        {
                            let mut k = knowledge.lock().unwrap();
                            let (cx, cy) = k.encode(&response, 0.8);
                            if cx < BRAIN_VIZ_SIZE && cy < BRAIN_VIZ_SIZE {
                                flash_nearby_cells(
                                    &mut state.brain_flashes,
                                    cx,
                                    cy,
                                    BrainMode::Encoding,
                                );
                            }
                            // Update active cell count
                            state.active_cells = k.active_cells();
                        }
                    }
                    InferenceMsg::Error(e) => {
                        if let Some(last) = state.messages.last_mut() {
                            if matches!(last.role, Role::Sage) {
                                last.content = format!("Error: {}", e);
                            }
                        }
                        state.generating = false;
                        state.brain_mode = BrainMode::Idle;
                    }
                }
            }
        }

        // Decay flashes
        decay_flashes(&mut state.brain_flashes);
        state.frame_counter += 1;

        // Refresh brain grid visualization periodically (every 60 frames ≈ 3s)
        if state.frame_counter.is_multiple_of(60) {
            if let Ok(k) = knowledge.try_lock() {
                let active = k.knowledge().active_knowledge(0.01);
                // Reset grid
                for row in state.brain_grid.iter_mut() {
                    for cell in row.iter_mut() {
                        *cell = 0.0;
                    }
                }
                for entry in &active {
                    let (x, y) = entry.position;
                    if x < BRAIN_VIZ_SIZE && y < BRAIN_VIZ_SIZE {
                        state.brain_grid[y][x] = entry.relevance;
                    }
                }
            }
        }

        // Auto-save brain periodically (every autosave_interval_secs)
        let elapsed_since_save = state.last_save.elapsed().as_secs();
        if elapsed_since_save >= state.autosave_interval_secs && state.autosave_interval_secs > 0 {
            if let Ok(k) = knowledge.try_lock() {
                if let Ok(()) = k.save_brain() {
                    state.last_save = Instant::now();
                    state.messages.push(TuiChatMessage {
                        role: Role::System,
                        content: "💾 Brain auto-saved.".to_string(),
                    });
                }
            }
        }

        // Draw
        terminal.draw(|f| ui(f, &state))?;

        // Handle input
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Enter => {
                        if state.generating {
                            continue;
                        }
                        let input = state.input.trim().to_string();
                        if input.is_empty() {
                            continue;
                        }
                        state.input.clear();
                        state.cursor_pos = 0;

                        if input.starts_with('/') {
                            match handle_command(&mut state, &input, &knowledge, &engine) {
                                CommandResult::Handled => {}
                                CommandResult::NeedsResetConfirmation => {
                                    state.messages.push(TuiChatMessage {
                                        role: Role::System,
                                        content: "⚠️  This will erase all learned knowledge. Type /reset-confirm to proceed.".to_string(),
                                    });
                                }
                            }
                        } else {
                            state.messages.push(TuiChatMessage {
                                role: Role::User,
                                content: input.clone(),
                            });
                            state.messages.push(TuiChatMessage {
                                role: Role::Sage,
                                content: String::new(),
                            });
                            state.generating = true;
                            state.scroll_offset = 0; // auto-scroll to bottom
                            state.brain_mode = BrainMode::Encoding;

                            // Retrieve then encode — one combined lock acquisition.
                            // retrieve_knowledge() uses the trained BinaryRelevanceReadout
                            // to re-rank results; encode() writes user input to the NCA grid.
                            {
                                let mut k = knowledge.lock().unwrap();

                                // Retrieval: feeds BinaryRelevanceReadout feedback loop
                                if let Some(brain_context) = k.retrieve_knowledge(&input) {
                                    // Augment the system message with recalled knowledge
                                    if let Some(sys) = history.first_mut() {
                                        if sys.role == ChatRole::System
                                            && !sys.content.contains("## Recalled Knowledge")
                                        {
                                            sys.content =
                                                format!("{}\n\n{}", sys.content, brain_context);
                                        } else if sys.role == ChatRole::System {
                                            // Replace previous recalled knowledge section
                                            if let Some(pos) =
                                                sys.content.find("\n\n## Recalled Knowledge")
                                            {
                                                sys.content.truncate(pos);
                                                sys.content =
                                                    format!("{}\n\n{}", sys.content, brain_context);
                                            }
                                        }
                                    }

                                    // Flash retrieval cells in the brain visualization
                                    let results = k.knowledge().query(&input, 3);
                                    for r in &results {
                                        let (rx, ry) = r.position;
                                        if rx < BRAIN_VIZ_SIZE && ry < BRAIN_VIZ_SIZE {
                                            flash_nearby_cells(
                                                &mut state.brain_flashes,
                                                rx,
                                                ry,
                                                BrainMode::Retrieving,
                                            );
                                        }
                                    }
                                }

                                // Encode user input into the NCA grid
                                let (cx, cy) = k.encode(&input, 0.7);
                                if cx < BRAIN_VIZ_SIZE && cy < BRAIN_VIZ_SIZE {
                                    flash_nearby_cells(
                                        &mut state.brain_flashes,
                                        cx,
                                        cy,
                                        BrainMode::Encoding,
                                    );
                                }
                                state.active_cells = k.active_cells();
                            }

                            // Run streaming inference in background thread
                            history.push(ChatMessage {
                                role: ChatRole::User,
                                content: input,
                            });
                            let hist_clone = history.clone();
                            let tx_clone = tx.clone();
                            let engine_clone = Arc::clone(&engine);
                            std::thread::spawn(move || {
                                let full_response = Arc::new(std::sync::Mutex::new(String::new()));
                                let resp_clone = Arc::clone(&full_response);
                                let tx_token = tx_clone.clone();
                                let result = engine_clone.chat_streaming(
                                    &hist_clone,
                                    1000,
                                    Box::new(move |token: &str| {
                                        resp_clone.lock().unwrap().push_str(token);
                                        let _ =
                                            tx_token.send(InferenceMsg::Token(token.to_string()));
                                    }),
                                );
                                match result {
                                    Ok(()) => {
                                        let final_text = full_response.lock().unwrap().clone();
                                        let _ = tx_clone.send(InferenceMsg::Done(final_text));
                                    }
                                    Err(e) => {
                                        let _ = tx_clone.send(InferenceMsg::Error(e.to_string()));
                                    }
                                }
                            });
                        }
                    }
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        state.quit = true;
                    }
                    KeyCode::Char(c) => {
                        let pos = state.cursor_pos;
                        state.input.insert(pos, c);
                        state.cursor_pos += 1;
                    }
                    KeyCode::Backspace => {
                        if state.cursor_pos > 0 {
                            state.cursor_pos -= 1;
                            let pos = state.cursor_pos;
                            state.input.remove(pos);
                        }
                    }
                    KeyCode::Left => {
                        if state.cursor_pos > 0 {
                            state.cursor_pos -= 1;
                        }
                    }
                    KeyCode::Right => {
                        if state.cursor_pos < state.input.len() {
                            state.cursor_pos += 1;
                        }
                    }
                    KeyCode::PageUp => {
                        state.scroll_offset = state.scroll_offset.saturating_add(10);
                    }
                    KeyCode::PageDown => {
                        state.scroll_offset = state.scroll_offset.saturating_sub(10);
                    }
                    KeyCode::Up if key.modifiers.contains(KeyModifiers::SHIFT) => {
                        state.scroll_offset = state.scroll_offset.saturating_add(3);
                    }
                    KeyCode::Down if key.modifiers.contains(KeyModifiers::SHIFT) => {
                        state.scroll_offset = state.scroll_offset.saturating_sub(3);
                    }
                    KeyCode::Esc => {
                        // Don't quit on Escape — use /exit or /quit
                    }
                    _ => {}
                }
            }
        }
    }

    // Save brain before exit
    {
        let k = knowledge.lock().unwrap();
        match k.save_brain() {
            Ok(()) => eprintln!("🧠 Brain saved to {}", brain_path),
            Err(e) => eprintln!("⚠️  Failed to save brain: {}", e),
        }
        eprintln!(
            "📊 Retrieval stats: {}",
            k.knowledge().retrieval_stats.summary()
        );
        let (events, rel_rate, rounds) = k.retrieval_feedback_stats();
        if events > 0 {
            eprintln!(
                "🎯 Feedback: {} events, {:.0}% relevant, {} training rounds",
                events,
                rel_rate * 100.0,
                rounds
            );
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    // First-run node prompt: ask user if they want to run a node
    // Only prompt if this is the first conversation and no node is running
    if is_first_run && !is_node_running() && !was_node_prompted() {
        println!();
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("💡 Help SAGE grow smarter for everyone.");
        println!();
        println!("SAGE gets better when more people run nodes. Your node shares");
        println!("anonymous knowledge patterns with the network — no raw data,");
        println!("just compressed neural patterns that help every node learn faster.");
        println!();
        println!("Resources used: ~50MB RAM, ~1% CPU, minimal bandwidth.");
        println!();
        print!("Run a node?  [Y/n]: ");
        let _ = std::io::Write::flush(&mut std::io::stdout());

        // Read user input
        let mut input = String::new();
        if std::io::stdin().read_line(&mut input).is_ok() {
            let input = input.trim().to_lowercase();
            if input.is_empty() || input == "y" || input == "yes" {
                println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                println!();
                println!("🌐 Starting SAGE node in background...");

                // Mark that we've prompted
                mark_node_prompted();

                // Start node in background by spawning the command
                let exe = std::env::current_exe().ok();
                if let Some(exe_path) = exe {
                    let _ = std::process::Command::new(exe_path)
                        .args(["node", "start"])
                        .stdin(std::process::Stdio::null())
                        .stdout(std::process::Stdio::null())
                        .stderr(std::process::Stdio::null())
                        .spawn();
                    println!("✅ Node started! View status with: sage node status");
                }
            } else {
                mark_node_prompted();
                println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                println!();
                println!("No problem! You can start a node anytime with: sage node start");
            }
        }
        println!();
    }

    println!("SAGE disconnected.");
    Ok(())
}

/// Command result — some commands need special handling by the caller
enum CommandResult {
    /// Command handled, no special action needed
    Handled,
    /// /reset command needs confirmation — caller should prompt
    NeedsResetConfirmation,
}

fn handle_command(
    state: &mut AppState,
    cmd: &str,
    knowledge: &std::sync::Arc<std::sync::Mutex<KnowledgeLoop>>,
    engine: &Arc<dyn InferenceEngine>,
) -> CommandResult {
    let parts: Vec<&str> = cmd.trim().splitn(2, ' ').collect();
    match parts[0] {
        "/quit" | "/exit" | "/q" => {
            state.quit = true;
            CommandResult::Handled
        }
        "/help" | "/h" => {
            state.messages.push(TuiChatMessage {
                role: Role::System,
                content: "SAGE Commands:\n\
                    \n  /help          Show this message\
                    \n  /status        Show brain stats (active cells, version)\
                    \n  /save          Force save brain to disk\
                    \n  /reset         Clear brain and start fresh (asks for confirmation)\
                    \n  /bad           Mark last response as not helpful (trains retrieval)\
                    \n  /feedback      Show retrieval feedback training stats\
                    \n  /node          Show network status\
                    \n  /quit or /exit Exit SAGE\
                    \n\nEverything else is a chat message."
                    .into(),
            });
            CommandResult::Handled
        }
        "/status" | "/s" => {
            let version = env!("CARGO_PKG_VERSION");
            state.messages.push(TuiChatMessage {
                role: Role::System,
                content: format!(
                    "┌─ SAGE Status v{} ────────────────\n\
                     │ Engine:     {}\n\
                     │ Grid:       {}×{} ({} channels)\n\
                     │ Knowledge:  {} active cells\n\
                     │ Brain:      {}\n\
                     └────────────────────────────────────",
                    version,
                    state.engine_name,
                    crate::grid::GRID_SIZE,
                    crate::grid::GRID_SIZE,
                    crate::grid::NUM_CHANNELS,
                    state.active_cells,
                    state.brain_path,
                ),
            });
            CommandResult::Handled
        }
        "/save" => {
            let k = knowledge.lock().unwrap();
            match k.save_brain() {
                Ok(()) => {
                    state.messages.push(TuiChatMessage {
                        role: Role::System,
                        content: format!("💾 Brain saved to {}", state.brain_path),
                    });
                }
                Err(e) => {
                    state.messages.push(TuiChatMessage {
                        role: Role::System,
                        content: format!("⚠️  Failed to save brain: {}", e),
                    });
                }
            }
            CommandResult::Handled
        }
        "/reset" => {
            // Return signal that we need confirmation
            CommandResult::NeedsResetConfirmation
        }
        "/reset-confirm" => {
            // Actually perform the reset — recreate KnowledgeLoop with same engine
            let brain_path = state.brain_path.clone();
            let mut k = knowledge.lock().unwrap();
            *k = KnowledgeLoop::new(Arc::clone(engine)).with_brain_path(&brain_path);
            state.active_cells = 0;
            state.messages.push(TuiChatMessage {
                role: Role::System,
                content: "🧠 Brain reset to fresh state. Starting over!".to_string(),
            });
            CommandResult::Handled
        }
        "/bad" => {
            // Mark the last retrieval as irrelevant — trains the BinaryRelevanceReadout
            // to down-weight this type of query in future retrievals.
            let mut k = knowledge.lock().unwrap();
            k.mark_last_retrieval_irrelevant();
            state.messages.push(TuiChatMessage {
                role: Role::System,
                content:
                    "👎 Marked last retrieval as not helpful. Retrieval will improve over time."
                        .to_string(),
            });
            CommandResult::Handled
        }
        "/feedback" => {
            // Show retrieval feedback training statistics
            let k = knowledge.lock().unwrap();
            let (events, rel_rate, rounds) = k.retrieval_feedback_stats();
            state.messages.push(TuiChatMessage {
                role: Role::System,
                content: format!(
                    "🎯 Retrieval Feedback Stats:\n\
                     │ Total events:    {}\n\
                     │ Relevance rate:  {:.0}%\n\
                     │ Training rounds: {}\n\
                     │\n\
                     │ Use /bad after a response to mark it as not helpful.\n\
                     │ The readout trains after 100 events.",
                    events,
                    rel_rate * 100.0,
                    rounds
                ),
            });
            CommandResult::Handled
        }
        "/node" | "/network" => {
            state.messages.push(TuiChatMessage {
                role: Role::System,
                content: format!(
                    "🌐 Network Status:\n\
                     │ Peers connected: {}\n\
                     │ To start a node: sage node start\n\
                     │ (Network features coming soon)",
                    state.peer_count
                ),
            });
            CommandResult::Handled
        }
        _ => {
            state.messages.push(TuiChatMessage {
                role: Role::System,
                content: format!(
                    "Unknown command: {}. Type /help for available commands.",
                    parts[0]
                ),
            });
            CommandResult::Handled
        }
    }
}
