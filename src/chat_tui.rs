//! Ratatui-based TUI for SAGE chat with brain visualization.
//!
//! This module provides the rich terminal UI that runs inference locally
//! (embedded SmolLM2 or Ollama) with the NCA brain visualization grid.

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

use crate::distributed_knowledge::{default_brain_path, KnowledgeStore, NCAKnowledge};
use crate::inference::{self, ChatMessage, ChatRole, InferenceEngine};
use crate::inference::ollama::OllamaEngine;

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

const GRID_SIZE: usize = 32;

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
}

fn flash_nearby_cells(flashes: &mut Vec<Vec<CellFlash>>, cx: usize, cy: usize, mode: BrainMode) {
    let now = Instant::now();
    let radius = 3i32;
    for dy in -radius..=radius {
        for dx in -radius..=radius {
            let nx = cx as i32 + dx;
            let ny = cy as i32 + dy;
            if nx >= 0 && nx < GRID_SIZE as i32 && ny >= 0 && ny < GRID_SIZE as i32 {
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

fn decay_flashes(flashes: &mut Vec<Vec<CellFlash>>) {
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
    let x_scale = GRID_SIZE as f64 / grid_width as f64;
    let y_scale = GRID_SIZE as f64 / grid_height as f64;
    let pulse = (state.frame_counter as f64 * 0.05).sin() * 0.15 + 0.15;

    let mut lines: Vec<Line> = Vec::new();
    for row in 0..grid_height {
        let mut spans: Vec<Span> = Vec::new();
        for col in 0..grid_width {
            let gx = (col as f64 * x_scale).min((GRID_SIZE - 1) as f64) as usize;
            let gy = (row as f64 * y_scale).min((GRID_SIZE - 1) as f64) as usize;

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
                Color::Rgb(
                    0x00,
                    g,
                    (0x22 as f64 + base * (0x66 - 0x22) as f64) as u8,
                )
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

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("● ", Style::default().fg(GREEN)),
        Span::styled("enc ", Style::default().fg(DIM)),
        Span::styled("● ", Style::default().fg(CYAN)),
        Span::styled("ret ", Style::default().fg(DIM)),
        Span::styled("● ", Style::default().fg(Color::Rgb(0x1a, 0x1a, 0x1a))),
        Span::styled("idle", Style::default().fg(DIM)),
    ]));

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
                    if pos > 0 { pos + 1 } else { boundary } // break after the space
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
    let reduced_width = if has_brain { inner_width.saturating_sub(brain_w) } else { inner_width };

    let mut wrapped_lines: Vec<Line<'static>> = Vec::new();
    for line in raw_lines {
        wrapped_lines.extend(wrap_line(line, reduced_width.max(20)));
    }

    let total = wrapped_lines.len();
    let scroll = if total > chat_height { total - chat_height } else { 0 };

    let mut visible: Vec<Line<'static>> = wrapped_lines.into_iter().skip(scroll).take(chat_height).collect();

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
            Constraint::Min(5),   // Chat
            Constraint::Length(3), // Input
        ])
        .split(size);

    // Header
    let version = env!("CARGO_PKG_VERSION");
    let header = Paragraph::new(Line::from(vec![
        Span::styled("SAGE", Style::default().fg(GREEN).add_modifier(Modifier::BOLD)),
        Span::styled(" — Shared Adaptive Growing Experience", Style::default().fg(PURPLE)),
        Span::styled(format!("  v{}", version), Style::default().fg(DIM)),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(DIM_GREEN))
            .style(Style::default().bg(BG)),
    )
    .style(Style::default().bg(BG));
    f.render_widget(header, chunks[0]);

    // Status bar
    let engine_color = if state.engine_name.contains("Ollama") { CYAN } else { ORANGE };
    let status = Paragraph::new(Line::from(vec![
        Span::styled(" ◉ ", Style::default().fg(engine_color)),
        Span::styled(&state.engine_name, Style::default().fg(engine_color)),
        Span::styled("  │  ", Style::default().fg(DIM)),
        Span::styled("knowledge:", Style::default().fg(DIM)),
        Span::styled(format!("{}", state.active_cells), Style::default().fg(PURPLE)),
        Span::styled("  │  ", Style::default().fg(DIM)),
        Span::styled("brain:", Style::default().fg(DIM)),
        Span::styled(&state.brain_path, Style::default().fg(DIM_GREEN)),
        if state.generating {
            Span::styled("  ◌ thinking…", Style::default().fg(PURPLE).add_modifier(Modifier::BOLD))
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

/// Run the ratatui TUI chat interface with local inference.
pub fn run(engine_mode: EngineMode, model: &str, ollama_url: &str) -> Result<(), Box<dyn std::error::Error>> {
    let brain_path = default_brain_path();
    let knowledge = NCAKnowledge::new();

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
            // Default to embedded SmolLM2 — use --ollama to switch to Ollama
            eprintln!("🧠 Using embedded SmolLM2 (use --ollama for other models)");
            inference::default_engine()
        }
    });

    let engine_name = engine.name().to_string();
    let active_cells = knowledge.active_knowledge(0.01).len();

    // Build brain grid from NCA knowledge
    let mut brain_grid = vec![vec![0.0f64; GRID_SIZE]; GRID_SIZE];
    let active = knowledge.active_knowledge(0.01);
    for entry in &active {
        let (x, y) = entry.position;
        if x < GRID_SIZE && y < GRID_SIZE {
            brain_grid[y][x] = entry.relevance as f64;
        }
    }

    let now = Instant::now();
    let brain_flashes: Vec<Vec<CellFlash>> = (0..GRID_SIZE)
        .map(|_| {
            (0..GRID_SIZE)
                .map(|_| CellFlash {
                    intensity: 0.0,
                    mode: BrainMode::Idle,
                    timestamp: now,
                })
                .collect()
        })
        .collect();

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
    };

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let system_msg = ChatMessage {
        role: ChatRole::System,
        content: "You are SAGE, a decentralized AI that learns and grows. You're running on the user's local machine as part of a peer-to-peer network of AI nodes. Be helpful, curious, and thoughtful.".to_string(),
    };
    let mut history: Vec<ChatMessage> = vec![system_msg];

    // Channel for async inference results
    let (tx, rx) = std::sync::mpsc::channel::<String>();

    loop {
        if state.quit {
            break;
        }

        // Check for inference result
        if state.generating {
            if let Ok(response) = rx.try_recv() {
                if let Some(last) = state.messages.last_mut() {
                    if matches!(last.role, Role::Sage) {
                        last.content = response.clone();
                    }
                }
                history.push(ChatMessage {
                    role: ChatRole::Assistant,
                    content: response,
                });
                state.generating = false;
                state.brain_mode = BrainMode::Idle;

                // Flash retrieval cells
                let hash = simple_hash(&state.messages.last().map(|m| m.content.as_str()).unwrap_or(""));
                let cx = (hash / 7) % GRID_SIZE;
                let cy = (hash / 13) % GRID_SIZE;
                flash_nearby_cells(&mut state.brain_flashes, cx, cy, BrainMode::Retrieving);
            }
        }

        // Decay flashes
        decay_flashes(&mut state.brain_flashes);
        state.frame_counter += 1;

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
                            handle_command(&mut state, &input);
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
                            state.brain_mode = BrainMode::Encoding;

                            // Flash encoding cells
                            let hash = simple_hash(&input);
                            let cx = hash % GRID_SIZE;
                            let cy = (hash / GRID_SIZE) % GRID_SIZE;
                            flash_nearby_cells(&mut state.brain_flashes, cx, cy, BrainMode::Encoding);

                            // Run inference in background thread
                            history.push(ChatMessage {
                                role: ChatRole::User,
                                content: input,
                            });
                            let hist_clone = history.clone();
                            let tx_clone = tx.clone();
                            let engine_clone = Arc::clone(&engine);
                            std::thread::spawn(move || {
                                match engine_clone.chat(&hist_clone, 1000) {
                                    Ok(response) => { let _ = tx_clone.send(response); }
                                    Err(e) => { let _ = tx_clone.send(format!("Error: {}", e)); }
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
                    KeyCode::Esc => {
                        // Don't quit on Escape — use /exit or /quit
                    }
                    _ => {}
                }
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    println!("SAGE disconnected.");
    Ok(())
}

fn handle_command(state: &mut AppState, cmd: &str) {
    let parts: Vec<&str> = cmd.trim().splitn(2, ' ').collect();
    match parts[0] {
        "/quit" | "/q" => state.quit = true,
        "/help" | "/h" => {
            state.messages.push(TuiChatMessage {
                role: Role::System,
                content: "Commands:\n  /status  — Engine and brain info\n  /help    — This message\n  /quit    — Exit".into(),
            });
        }
        "/status" | "/s" => {
            state.messages.push(TuiChatMessage {
                role: Role::System,
                content: format!(
                    "┌─ SAGE Status ──────────────────\n│ Engine:     {}\n│ Grid:       {}×{}\n│ Knowledge:  {} active cells\n│ Brain:      {}\n└────────────────────────────────",
                    state.engine_name,
                    GRID_SIZE, GRID_SIZE,
                    state.active_cells,
                    state.brain_path,
                ),
            });
        }
        _ => {
            state.messages.push(TuiChatMessage {
                role: Role::System,
                content: format!("Unknown command: {}. Type /help for available commands.", parts[0]),
            });
        }
    }
}
