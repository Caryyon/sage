#![allow(dead_code, unused_assignments)]
// SAGE Chat — Thin TUI client that connects to a running sage-node.
//
// Usage: sage_chat [--port 19175]
//
// Connects to localhost:<port> and provides a rich terminal UI.
// All inference, knowledge, and networking happens on the sage-node daemon.

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
use std::io::{self, BufRead, BufReader, Write};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

// --- Theme Colors ---
const CYAN: Color = Color::Rgb(0x00, 0xff, 0xd5);
const GREEN: Color = Color::Rgb(0x00, 0xff, 0x99);
const PURPLE: Color = Color::Rgb(0xa8, 0x55, 0xf7);
const DIM_GREEN: Color = Color::Rgb(0x00, 0x99, 0x66);
const DIM_PURPLE: Color = Color::Rgb(0x6b, 0x33, 0xaa);
const BG: Color = Color::Rgb(0x0a, 0x0a, 0x0f);
const DIM: Color = Color::Rgb(0x44, 0x44, 0x55);
const ORANGE: Color = Color::Rgb(0xff, 0xa5, 0x00);

const GRID_SIZE: usize = 256;

// --- Chat Messages ---
#[derive(Clone)]
enum Role {
    User,
    Sage,
    System,
}

#[derive(Clone)]
struct ChatMessage {
    role: Role,
    content: String,
}

// --- Brain visualization ---
#[derive(Clone, Copy, PartialEq)]
enum BrainMode {
    Idle,
    Encoding,
    Retrieving,
    PeerSync,
}

struct CellFlash {
    intensity: f64,
    mode: BrainMode,
    timestamp: Instant,
}

// --- Node status (from STATUS command) ---
#[derive(Default, Clone)]
struct NodeStatus {
    node_id: String,
    engine: String,
    grid_health: String,
    active_cells: usize,
    peer_count: usize,
    total_activation: f64,
    avg_confidence: f64,
    brain_path: String,
    connected: bool,
    distributed_peers: usize,
    distributed: bool,
}

// --- App State ---
struct AppState {
    messages: Vec<ChatMessage>,
    input: String,
    cursor_pos: usize,
    streaming: bool,
    quit: bool,
    status: NodeStatus,
    // Brain visualization
    brain_grid: Vec<Vec<f64>>, // activation values from node
    brain_flashes: Vec<Vec<CellFlash>>,
    brain_mode: BrainMode,
    frame_counter: u64,
}

fn flash_nearby_cells(flashes: &mut [Vec<CellFlash>], cx: usize, cy: usize, mode: BrainMode) {
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
                    BrainMode::PeerSync => {
                        let r = (0xaa as f64 + flash.intensity * (0xff - 0xaa) as f64) as u8;
                        let g = (0x77 as f64 + flash.intensity * (0xdd - 0x77) as f64) as u8;
                        Color::Rgb(r, g, 0x00)
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

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("● ", Style::default().fg(GREEN)),
        Span::styled("enc ", Style::default().fg(DIM)),
        Span::styled("● ", Style::default().fg(CYAN)),
        Span::styled("ret ", Style::default().fg(DIM)),
        Span::styled("● ", Style::default().fg(ORANGE)),
        Span::styled("peer ", Style::default().fg(DIM)),
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
                let (take, rest) = remaining.split_at(available);
                if !take.is_empty() {
                    current_spans.push(Span::styled(take.to_string(), style));
                }
                result.push(Line::from(std::mem::take(&mut current_spans)));
                current_width = 0;
                remaining = rest;
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

    let mut wrapped_lines: Vec<Line<'static>> = Vec::new();
    for line in raw_lines {
        wrapped_lines.extend(wrap_line(line, inner_width));
    }

    let total = wrapped_lines.len();
    let scroll = total.saturating_sub(chat_height);

    if has_brain {
        let reduced_width = inner_width.saturating_sub(brain_w);
        let visible_start = scroll;
        let visible_end = std::cmp::min(scroll + chat_height, wrapped_lines.len());
        let mut new_visible: Vec<Line<'static>> = Vec::new();

        for i in visible_start..visible_end {
            let vis_row = new_visible.len();
            if vis_row < brain_h && reduced_width > 0 {
                let mut line = wrapped_lines[i].clone();
                truncate_line(&mut line, reduced_width);
                new_visible.push(line);
            } else {
                new_visible.push(wrapped_lines[i].clone());
            }
        }

        let chat = Paragraph::new(new_visible)
            .block(block)
            .style(Style::default().bg(BG));
        f.render_widget(chat, area);
    } else {
        let chat = Paragraph::new(wrapped_lines)
            .block(block)
            .style(Style::default().bg(BG))
            .scroll((scroll as u16, 0));
        f.render_widget(chat, area);
    }
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
    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            "SAGE",
            Style::default().fg(GREEN).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            " — Self-Adaptive General Explorer",
            Style::default().fg(PURPLE),
        ),
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
    let st = &state.status;
    let conn_indicator = if st.connected {
        Span::styled(" ◉ ", Style::default().fg(GREEN))
    } else {
        Span::styled(" ○ ", Style::default().fg(Color::Rgb(0xff, 0x33, 0x33)))
    };
    let short_id = if st.node_id.len() > 16 {
        &st.node_id[..16]
    } else if st.node_id.is_empty() {
        "disconnected"
    } else {
        &st.node_id
    };
    let peer_color = if st.peer_count > 0 { CYAN } else { PURPLE };
    let status = Paragraph::new(Line::from(vec![
        conn_indicator,
        Span::styled(short_id, Style::default().fg(DIM_GREEN)),
        Span::styled("  │  ", Style::default().fg(DIM)),
        Span::styled("grid:", Style::default().fg(DIM)),
        Span::styled(
            if st.connected {
                &st.grid_health
            } else {
                "offline"
            },
            Style::default().fg(if st.connected { GREEN } else { DIM }),
        ),
        Span::styled("  │  ", Style::default().fg(DIM)),
        Span::styled("peers:", Style::default().fg(DIM)),
        Span::styled(
            format!("{}", st.peer_count),
            Style::default().fg(peer_color),
        ),
        Span::styled("  │  ", Style::default().fg(DIM)),
        Span::styled("knowledge:", Style::default().fg(DIM)),
        Span::styled(format!("{}", st.active_cells), Style::default().fg(PURPLE)),
        if st.distributed && st.distributed_peers > 0 {
            Span::styled("  │  ", Style::default().fg(DIM))
        } else {
            Span::styled("", Style::default())
        },
        if st.distributed && st.distributed_peers > 0 {
            Span::styled(
                format!("⚡ Distributed ({} peers)", st.distributed_peers),
                Style::default().fg(CYAN).add_modifier(Modifier::BOLD),
            )
        } else {
            Span::styled("", Style::default())
        },
        if state.streaming {
            Span::styled(
                "  ◌ streaming…",
                Style::default().fg(PURPLE).add_modifier(Modifier::BOLD),
            )
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
        .border_style(Style::default().fg(if state.streaming { DIM } else { GREEN }))
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

/// Messages from background reader thread to TUI loop
enum NodeMsg {
    Token(String),
    Done,
    Status(serde_json::Value),
    PeerLine(String),
    BrainRow(usize, Vec<f64>),
    InfoLine(String),
    Error(String),
    Disconnected,
}

/// Background thread: reads lines from TCP, parses protocol, sends to UI
fn reader_thread(reader: BufReader<TcpStream>, tx: std::sync::mpsc::Sender<NodeMsg>) {
    let mut reader = reader;
    let mut line_buf = String::new();
    let mut brain_row: usize = 0;
    loop {
        line_buf.clear();
        match reader.read_line(&mut line_buf) {
            Ok(0) | Err(_) => {
                let _ = tx.send(NodeMsg::Disconnected);
                break;
            }
            Ok(_) => {
                let line = line_buf.trim_end();
                if line == "DONE" {
                    let _ = tx.send(NodeMsg::Done);
                    brain_row = 0;
                } else if let Some(token) = line.strip_prefix("TOKEN ") {
                    // Unescape newlines
                    let unescaped = token.replace("\\n", "\n");
                    let _ = tx.send(NodeMsg::Token(unescaped));
                } else if let Some(peer) = line.strip_prefix("PEER ") {
                    let _ = tx.send(NodeMsg::PeerLine(peer.to_string()));
                } else if let Some(row_data) = line.strip_prefix("ROW ") {
                    let vals: Vec<f64> = row_data
                        .split_whitespace()
                        .filter_map(|s| s.parse().ok())
                        .collect();
                    let _ = tx.send(NodeMsg::BrainRow(brain_row, vals));
                    brain_row += 1;
                } else if let Some(err) = line.strip_prefix("ERROR ") {
                    let _ = tx.send(NodeMsg::Error(err.to_string()));
                } else if line.starts_with('{') {
                    // JSON status
                    if let Ok(v) = serde_json::from_str(line) {
                        let _ = tx.send(NodeMsg::Status(v));
                    } else {
                        let _ = tx.send(NodeMsg::InfoLine(line.to_string()));
                    }
                } else {
                    let _ = tx.send(NodeMsg::InfoLine(line.to_string()));
                }
            }
        }
    }
}

/// Send a command to the node (thread-safe writer)
fn send_cmd(writer: &Mutex<TcpStream>, cmd: &str) -> io::Result<()> {
    let mut w = writer.lock().unwrap();
    w.write_all(cmd.as_bytes())?;
    w.write_all(b"\n")?;
    w.flush()
}

/// What response we're currently waiting for from the node
enum Waiting {
    None,
    Chat,
    Status,
    Peers,
    Knowledge,
    Brain,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse args
    let mut port: u16 = 19175;
    let args: Vec<String> = std::env::args().collect();
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--port" | "-p" => {
                i += 1;
                if i < args.len() {
                    port = args[i].parse().unwrap_or(19175);
                }
            }
            _ => {}
        }
        i += 1;
    }

    // Also check SAGE_PORT env
    if let Ok(p) = std::env::var("SAGE_PORT") {
        if let Ok(pn) = p.parse() {
            // Only use env if no explicit --port was given
            if !args.iter().any(|a| a == "--port" || a == "-p") {
                port = pn;
            }
        }
    }

    // Connect to sage-node
    let stream = match TcpStream::connect(format!("127.0.0.1:{}", port)) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Cannot connect to sage-node on port {}.", port);
            eprintln!(
                "Is sage-node running? Start it with: sage-node --port {}",
                port
            );
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };
    stream.set_nonblocking(false)?;
    let reader = BufReader::new(stream.try_clone()?);
    let writer = Arc::new(Mutex::new(stream));

    // Background reader thread
    let (msg_tx, msg_rx) = std::sync::mpsc::channel::<NodeMsg>();
    std::thread::spawn(move || reader_thread(reader, msg_tx));

    // Request initial status + brain
    send_cmd(&writer, "STATUS")?;

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

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
        streaming: false,
        quit: false,
        status: NodeStatus {
            connected: true,
            ..Default::default()
        },
        brain_grid: vec![vec![0.0; GRID_SIZE]; GRID_SIZE],
        brain_flashes,
        brain_mode: BrainMode::Idle,
        frame_counter: 0,
    };

    let mut waiting = Waiting::Status; // we sent STATUS on connect
    let mut peer_lines: Vec<String> = Vec::new();
    let mut info_lines: Vec<String> = Vec::new();

    // Periodic brain refresh timer
    let mut last_brain_fetch = Instant::now();
    let brain_interval = Duration::from_secs(5);

    loop {
        if state.quit {
            break;
        }

        // Process messages from node
        while let Ok(msg) = msg_rx.try_recv() {
            match msg {
                NodeMsg::Token(t) => {
                    if let Some(last) = state.messages.last_mut() {
                        if matches!(last.role, Role::Sage) {
                            last.content.push_str(&t);
                        }
                    }
                }
                NodeMsg::Done => {
                    match waiting {
                        Waiting::Chat => {
                            state.streaming = false;
                            state.brain_mode = BrainMode::Idle;
                            // After chat completes, refresh brain + status
                            let _ = send_cmd(&writer, "BRAIN");
                            waiting = Waiting::Brain;
                        }
                        Waiting::Status => {
                            // Request brain after status
                            let _ = send_cmd(&writer, "BRAIN");
                            waiting = Waiting::Brain;
                        }
                        Waiting::Brain => {
                            waiting = Waiting::None;
                        }
                        Waiting::Peers => {
                            if peer_lines.is_empty() {
                                state.messages.push(ChatMessage {
                                    role: Role::System,
                                    content: "No peers connected. Peers are discovered via mDNS."
                                        .into(),
                                });
                            } else {
                                let mut lines =
                                    vec![format!("Connected peers: {}", peer_lines.len())];
                                for p in &peer_lines {
                                    lines.push(format!("  {}", p));
                                }
                                state.messages.push(ChatMessage {
                                    role: Role::System,
                                    content: lines.join("\n"),
                                });
                            }
                            peer_lines.clear();
                            waiting = Waiting::None;
                        }
                        Waiting::Knowledge => {
                            if !info_lines.is_empty() {
                                state.messages.push(ChatMessage {
                                    role: Role::System,
                                    content: info_lines.join("\n"),
                                });
                                info_lines.clear();
                            }
                            waiting = Waiting::None;
                        }
                        Waiting::None => {}
                    }
                }
                NodeMsg::Status(v) => {
                    state.status.node_id = v["node_id"].as_str().unwrap_or("").to_string();
                    state.status.engine = v["engine"].as_str().unwrap_or("").to_string();
                    state.status.grid_health =
                        v["grid_health"].as_str().unwrap_or("healthy").to_string();
                    state.status.active_cells = v["active_cells"].as_u64().unwrap_or(0) as usize;
                    state.status.peer_count = v["peer_count"].as_u64().unwrap_or(0) as usize;
                    state.status.total_activation = v["total_activation"].as_f64().unwrap_or(0.0);
                    state.status.avg_confidence = v["avg_confidence"].as_f64().unwrap_or(0.0);
                    state.status.brain_path = v["brain_path"].as_str().unwrap_or("").to_string();
                    state.status.distributed_peers =
                        v["distributed_peers"].as_u64().unwrap_or(0) as usize;
                    state.status.distributed = v["distributed"].as_bool().unwrap_or(false);
                    state.status.connected = true;
                }
                NodeMsg::PeerLine(p) => {
                    peer_lines.push(p);
                }
                NodeMsg::BrainRow(row, vals) => {
                    if row < GRID_SIZE && vals.len() >= GRID_SIZE {
                        state.brain_grid[row] = vals[..GRID_SIZE].to_vec();
                    } else if row < GRID_SIZE && !vals.is_empty() {
                        // Handle size mismatch (e.g. migration): pad or truncate
                        let mut padded = vals;
                        padded.resize(GRID_SIZE, 0.0);
                        state.brain_grid[row] = padded;
                    }
                }
                NodeMsg::InfoLine(line) => {
                    info_lines.push(line);
                }
                NodeMsg::Error(e) => {
                    state.messages.push(ChatMessage {
                        role: Role::System,
                        content: format!("Error: {}", e),
                    });
                    state.streaming = false;
                }
                NodeMsg::Disconnected => {
                    state.status.connected = false;
                    state.streaming = false;
                    state.messages.push(ChatMessage {
                        role: Role::System,
                        content: "Disconnected from sage-node.".into(),
                    });
                }
            }
        }

        // Periodic brain refresh
        if !state.streaming
            && state.status.connected
            && last_brain_fetch.elapsed() > brain_interval
            && matches!(waiting, Waiting::None)
        {
            let _ = send_cmd(&writer, "STATUS");
            waiting = Waiting::Status;
            last_brain_fetch = Instant::now();
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
                        if state.streaming || !state.status.connected {
                            continue;
                        }
                        let input = state.input.trim().to_string();
                        if input.is_empty() {
                            continue;
                        }
                        state.input.clear();
                        state.cursor_pos = 0;

                        if input.starts_with('/') {
                            handle_local_command(&mut state, &input, &writer, &mut waiting);
                        } else {
                            // Send chat to node
                            state.messages.push(ChatMessage {
                                role: Role::User,
                                content: input.clone(),
                            });
                            state.messages.push(ChatMessage {
                                role: Role::Sage,
                                content: String::new(),
                            });
                            state.streaming = true;
                            state.brain_mode = BrainMode::Encoding;
                            // Flash some random-ish cells for encoding visual
                            let hash = simple_hash(&input);
                            let cx = hash % GRID_SIZE;
                            let cy = (hash / GRID_SIZE) % GRID_SIZE;
                            flash_nearby_cells(
                                &mut state.brain_flashes,
                                cx,
                                cy,
                                BrainMode::Encoding,
                            );

                            if send_cmd(&writer, &format!("CHAT {}", input)).is_err() {
                                state.status.connected = false;
                                state.streaming = false;
                            }
                            waiting = Waiting::Chat;
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
                        state.quit = true;
                    }
                    _ => {}
                }
            }
        }
    }

    // Send QUIT to node
    let _ = send_cmd(&writer, "QUIT");

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    println!("SAGE disconnected.");
    Ok(())
}

fn handle_local_command(
    state: &mut AppState,
    cmd: &str,
    writer: &Mutex<TcpStream>,
    waiting: &mut Waiting,
) {
    let parts: Vec<&str> = cmd.trim().splitn(2, ' ').collect();
    match parts[0] {
        "/quit" | "/q" => state.quit = true,
        "/help" | "/h" => {
            state.messages.push(ChatMessage {
                role: Role::System,
                content: "Commands:\n  /status    — Node info, peers, grid health\n  /knowledge — Query what SAGE knows\n  /peers     — Connected peer list\n  /help      — This message\n  /quit      — Exit".into(),
            });
        }
        "/status" | "/s" => {
            let st = &state.status;
            state.messages.push(ChatMessage {
                role: Role::System,
                content: format!(
                    "┌─ SAGE Node Status ─────────────\n│ Node ID:    {}\n│ Engine:     {}\n│ Grid:       {}×{} ({})\n│ Peers:      {} connected\n│ Knowledge:  {} active cells\n│ Activation: {:.2} total\n│ Confidence: {:.1}% avg\n│ Brain:      {}\n└────────────────────────────────",
                    st.node_id,
                    st.engine,
                    GRID_SIZE,
                    GRID_SIZE,
                    st.grid_health,
                    st.peer_count,
                    st.active_cells,
                    st.total_activation,
                    st.avg_confidence * 100.0,
                    st.brain_path,
                ),
            });
            // Also refresh from node
            if send_cmd(writer, "STATUS").is_ok() {
                *waiting = Waiting::Status;
            }
        }
        "/peers" | "/p" => {
            if send_cmd(writer, "PEERS").is_ok() {
                *waiting = Waiting::Peers;
            }
        }
        "/knowledge" | "/k" => {
            let query = if parts.len() > 1 { parts[1] } else { "*" };
            if send_cmd(writer, &format!("KNOWLEDGE {}", query)).is_ok() {
                *waiting = Waiting::Knowledge;
            }
        }
        _ => {
            state.messages.push(ChatMessage {
                role: Role::System,
                content: format!(
                    "Unknown command: {}. Type /help for available commands.",
                    parts[0]
                ),
            });
        }
    }
}

/// Simple hash for picking brain flash coordinates from user text
fn simple_hash(s: &str) -> usize {
    let mut h: usize = 5381;
    for b in s.bytes() {
        h = h.wrapping_mul(33).wrapping_add(b as usize);
    }
    h
}

// (Waiting enum defined above main)
