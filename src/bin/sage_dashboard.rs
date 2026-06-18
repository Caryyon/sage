//! sage-dashboard — Live Brain Visualization Server
//!
//! Serves an HTML dashboard at http://localhost:9734 showing:
//! - 256×256 NCA grid heatmap (color = cell activation)
//! - Stats: alive cells, entries, grid fill %
//! - Auto-refreshes every 5 seconds
//!
//! Usage: sage-dashboard [--port 9734]

use sage::distributed_knowledge::{KnowledgeStore, NCAKnowledge, default_brain_path};
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::sync::Arc;
use std::thread;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let port: u16 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(9734);
    let brain_path = default_brain_path();

    let listener = TcpListener::bind(format!("0.0.0.0:{}", port))
        .expect("Failed to bind");

    eprintln!("🧠 SAGE Dashboard: http://localhost:{}", port);
    eprintln!("   Brain: {}", brain_path);
    eprintln!("   Press Ctrl+C to stop");

    // Accept connections in a loop
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let brain = brain_path.clone();
                thread::spawn(move || handle_connection(stream, &brain));
            }
            Err(e) => eprintln!("Connection error: {}", e),
        }
    }
}

fn handle_connection(mut stream: TcpStream, brain_path: &str) {
    let mut reader = BufReader::new(stream.try_clone().unwrap());
    let mut request_line = String::new();
    if reader.read_line(&mut request_line).is_err() {
        return;
    }

    let path = request_line.split_whitespace().nth(1).unwrap_or("/");

    match path {
        "/" | "/index.html" => serve_html(&mut stream),
        "/api/state" => serve_json(&mut stream, brain_path),
        "/api/grid" => serve_grid_json(&mut stream, brain_path),
        "/api/history" => serve_history_json(&mut stream),
        _ => serve_404(&mut stream),
    }
}

fn serve_html(stream: &mut TcpStream) {
    let html = get_dashboard_html();
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        html.len(), html
    );
    let _ = stream.write_all(response.as_bytes());
}

fn serve_json(stream: &mut TcpStream, brain_path: &str) {
    let state = read_brain_state(brain_path);
    let json = serde_json::to_string(&state).unwrap_or_default();
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        json.len(), json
    );
    let _ = stream.write_all(response.as_bytes());
}

fn serve_grid_json(stream: &mut TcpStream, brain_path: &str) {
    let grid_data = read_grid_activation(brain_path);
    let json = serde_json::to_string(&grid_data).unwrap_or_default();
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        json.len(), json
    );
    let _ = stream.write_all(response.as_bytes());
}

fn serve_history_json(stream: &mut TcpStream) {
    let history_path = "/home/cwolff/.sage/brain_history.csv";
    let mut points: Vec<(i64, usize, usize)> = Vec::new();
    if let Ok(content) = std::fs::read_to_string(history_path) {
        for line in content.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                if let (Ok(ts), Ok(alive), Ok(entries)) = (
                    parts[0].parse::<i64>(),
                    parts[1].parse::<usize>(),
                    parts[2].parse::<usize>(),
                ) {
                    points.push((ts, alive, entries));
                }
            }
        }
        if points.len() > 200 {
            points = points.split_off(points.len() - 200);
        }
    }
    let json = serde_json::to_string(&points).unwrap_or_default();
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        json.len(), json
    );
    let _ = stream.write_all(response.as_bytes());
}

fn serve_404(stream: &mut TcpStream) {
    let body = "404 Not Found";
    let response = format!(
        "HTTP/1.1 404 Not Found\r\nContent-Type: text/plain\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(), body
    );
    let _ = stream.write_all(response.as_bytes());
}

#[derive(serde::Serialize)]
struct BrainState {
    alive_cells: usize,
    total_cells: usize,
    fill_percent: f64,
    entries: usize,
    total_mass: f64,
    grid_width: usize,
    grid_height: usize,
    file_size_mb: f64,
    file_modified: String,
}

fn read_brain_state(brain_path: &str) -> BrainState {
    let mut knowledge = NCAKnowledge::new();
    if Path::new(brain_path).exists() {
        let _ = knowledge.load(brain_path);
    }

    let grid = &knowledge.grid;
    let total = grid.width * grid.height;
    let alive = grid.alive_count();

    let (file_size, modified) = std::fs::metadata(brain_path)
        .map(|m| {
            let mod_time = m.modified().ok().and_then(|t| {
                let d = t.duration_since(std::time::UNIX_EPOCH).ok()?;
                Some(format!("{}", d.as_secs()))
            }).unwrap_or_default();
            (m.len() as f64 / 1_048_576.0, mod_time)
        })
        .unwrap_or((0.0, String::new()));

    BrainState {
        alive_cells: alive,
        total_cells: total,
        fill_percent: alive as f64 / total as f64 * 100.0,
        entries: knowledge.text_store.len(),
        total_mass: grid.total_mass(),
        grid_width: grid.width,
        grid_height: grid.height,
        file_size_mb: file_size,
        file_modified: modified,
    }
}

fn read_grid_activation(brain_path: &str) -> Vec<Vec<f64>> {
    let mut knowledge = NCAKnowledge::new();
    if Path::new(brain_path).exists() {
        let _ = knowledge.load(brain_path);
    }

    let grid = &knowledge.grid;
    let mut data = Vec::with_capacity(grid.height);
    for y in 0..grid.height {
        let mut row = Vec::with_capacity(grid.width);
        for x in 0..grid.width {
            let alpha = grid.cells[y][x][3];
            let knowledge_act = grid.cells[y][x][sage::grid::KNOWLEDGE_ACTIVATION];
            let combined = (alpha + knowledge_act).min(1.0);
            row.push(combined);
        }
        data.push(row);
    }
    data
}

fn get_dashboard_html() -> String {
    r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>SAGE — Neural Brain</title>
<style>
@import url('https://fonts.googleapis.com/css2?family=JetBrains+Mono:wght@300;400;500;600;700&display=swap');

:root {
  --bg:#06060c; --bg2:#0c0c16; --bg3:#14141f; --bg4:#1a1a2e;
  --green:#00ff88; --cyan:#00d4aa; --purple:#a855f7; --pink:#ff3366;
  --amber:#ffaa00; --blue:#3b82f6; --text:#e8e8f0; --dim:#5a5a72; --border:#1e1e2e;
}
* { margin:0; padding:0; box-sizing:border-box; }
html,body { width:100vw; height:100vh; overflow:hidden; }
body { font-family:'JetBrains Mono',monospace; background:var(--bg); color:var(--text); }

/* Ambient background */
body::before {
  content:''; position:fixed; inset:0; z-index:-1;
  background:
    radial-gradient(ellipse 80% 60% at 20% 0%, rgba(0,212,170,.06), transparent),
    radial-gradient(ellipse 60% 80% at 80% 100%, rgba(168,85,247,.04), transparent);
}

/* Header */
.header {
  display:flex; align-items:center; justify-content:space-between;
  padding:10px 20px; background:var(--bg2);
  border-bottom:1px solid var(--border); height:56px; flex-shrink:0; z-index:100;
}
.brand { display:flex; align-items:center; gap:14px; }
.logo {
  width:36px; height:36px; border-radius:8px;
  background:linear-gradient(135deg,var(--cyan),var(--purple));
  display:flex; align-items:center; justify-content:center;
  font-size:20px; filter:drop-shadow(0 0 8px rgba(0,212,170,.3));
}
.header h1 {
  font-size:18px; font-weight:700; letter-spacing:3px;
  background:linear-gradient(90deg,var(--green),var(--cyan),var(--purple));
  -webkit-background-clip:text; -webkit-text-fill-color:transparent;
}
.header .sub { font-size:10px; color:var(--dim); letter-spacing:4px; text-transform:uppercase; margin-top:2px; }

.live-indicator { display:flex; align-items:center; gap:10px; font-size:11px; color:var(--dim); }
.live-dot { width:8px; height:8px; border-radius:50%; background:var(--green); box-shadow:0 0 10px var(--green); animation:pulse 2s infinite; }
@keyframes pulse { 0%,100%{opacity:1} 50%{opacity:.3} }

/* Layout */
.dashboard {
  display:grid; grid-template-columns:1fr 320px;
  gap:12px; padding:12px;
  height:calc(100vh - 56px); /* minus header */
  max-width:100vw;
}
@media(max-width:1024px) { .dashboard { grid-template-columns:1fr; } }

/* Panels */
.panel { background:var(--bg2); border:1px solid var(--border); border-radius:10px; overflow:hidden; }
.panel-head {
  display:flex; align-items:center; justify-content:space-between;
  padding:10px 16px; border-bottom:1px solid var(--border);
  font-size:11px; font-weight:600; letter-spacing:2px; text-transform:uppercase; color:var(--dim);
}
.panel-head .badge {
  background:var(--bg4); padding:3px 10px; border-radius:10px;
  font-size:10px; color:var(--cyan); letter-spacing:1px;
}

/* Grid canvas */
.grid-panel { display:flex; flex-direction:column; min-height:0; }
.grid-wrap { flex:1; display:flex; align-items:stretch; min-height:0; overflow:hidden; }
#grid { width:100%; height:100%; display:block; image-rendering:pixelated; }

/* Grid overlay scan effect */
.scan-overlay {
  position:relative;
}
.scan-overlay::after {
  content:''; position:absolute; inset:0; pointer-events:none;
  background:repeating-linear-gradient(0deg,transparent,transparent 3px,rgba(0,212,170,.02) 4px);
}

/* Legend bar */
.legend-bar {
  display:flex; align-items:center; gap:12px; padding:10px 16px;
  background:var(--bg3); border-top:1px solid var(--border);
  font-size:10px; color:var(--dim); letter-spacing:1px;
}
.gradient-bar {
  flex:1; height:8px; border-radius:4px;
  background:linear-gradient(90deg,#06060c,#0a0a2a,#0044ff,#00ff88,#ffaa00,#ff3366);
  box-shadow:0 0 20px rgba(0,212,170,.1);
}

/* Side panel */
.side-panel { display:flex; flex-direction:column; gap:12px; min-height:0; overflow:hidden; }

.stat-grid { display:grid; grid-template-columns:1fr 1fr; gap:1px; background:var(--border); }
.stat {
  background:var(--bg2); padding:10px 12px; display:flex; flex-direction:column; gap:2px;
}
.stat-label { font-size:9px; color:var(--dim); letter-spacing:2px; text-transform:uppercase; }
.stat-value { font-size:18px; font-weight:700; color:var(--text); font-variant-numeric:tabular-nums; }
.stat-value.accent { color:var(--cyan); }
.stat-value.purple { color:var(--purple); }
.stat-value.green { color:var(--green); }
.stat-value.amber { color:var(--amber); }
.stat-unit { font-size:11px; color:var(--dim); font-weight:400; }

/* Progress ring */
.ring-wrap { display:flex; align-items:center; gap:12px; padding:10px 12px; }
.ring { position:relative; width:64px; height:64px; flex-shrink:0; }
.ring svg { transform:rotate(-90deg); }
.ring-bg { fill:none; stroke:var(--bg4); stroke-width:8; }
.ring-fg { fill:none; stroke:var(--cyan); stroke-width:8; stroke-linecap:round; transition:stroke-dashoffset .8s ease; filter:drop-shadow(0 0 6px rgba(0,212,170,.4)); }
.ring-label { position:absolute; inset:0; display:flex; align-items:center; justify-content:center; font-size:14px; font-weight:700; color:var(--text); }
.ring-info { flex:1; }
.ring-info .label { font-size:9px; color:var(--dim); letter-spacing:2px; text-transform:uppercase; }
.ring-info .desc { font-size:11px; color:var(--text); margin-top:4px; line-height:1.4; }

/* Chart */
.chart-panel { flex:0 0 auto; }
#sparkline { width:100%; height:100px; display:block; }

/* Activity log */
.log {
  flex:1; overflow-y:auto; padding:8px 0; min-height:0;
}
.log::-webkit-scrollbar { width:3px; }
.log::-webkit-scrollbar-track { background:transparent; }
.log::-webkit-scrollbar-thumb { background:var(--border); border-radius:2px; }
.log-item {
  display:flex; gap:10px; padding:6px 16px; font-size:10px;
  border-bottom:1px solid rgba(30,30,46,.5);
  animation:slideIn .3s ease;
}
.log-item:last-child { border:none; }
.log-time { color:var(--dim); white-space:nowrap; min-width:55px; }
.log-tag { min-width:45px; font-weight:600; }
.log-tag.learn { color:var(--cyan); }
.log-tag.dream { color:var(--purple); }
.log-tag.system { color:var(--amber); }
.log-msg { color:var(--dim); flex:1; }
@keyframes slideIn { from{opacity:0;transform:translateX(-8px)} to{opacity:1;transform:none} }

/* Empty state */
.empty { text-align:center; padding:24px; color:var(--dim); font-size:11px; }
</style>
</head>
<body>

<div class="header">
  <div class="brand">
    <div class="logo">🧠</div>
    <div>
      <h1>SAGE NEURAL BRAIN</h1>
      <div class="sub">Live NCA Knowledge Grid</div>
    </div>
  </div>
  <div class="live-indicator">
    <div class="live-dot"></div>
    <span id="liveStatus">CONNECTING</span>
  </div>
</div>

<div class="dashboard">
  <!-- Grid heatmap -->
  <div class="panel grid-panel scan-overlay">
    <div class="panel-head">
      <span>Knowledge Grid — 256×256</span>
      <span class="badge" id="gridBadge">— alive</span>
    </div>
    <div class="grid-wrap"><canvas id="grid" width="256" height="256"></canvas></div>
    <div class="legend-bar">
      <span>DORMANT</span>
      <div class="gradient-bar"></div>
      <span>PEAK</span>
    </div>
  </div>

  <!-- Side panel -->
  <div class="side-panel">
    <!-- Stats grid -->
    <div class="panel">
      <div class="panel-head"><span>Brain Metrics</span><span class="badge" id="fileSize">— MB</span></div>
      <div class="stat-grid">
        <div class="stat">
          <span class="stat-label">Alive Cells</span>
          <span class="stat-value cyan" id="alive" style="color:var(--cyan)">—</span>
        </div>
        <div class="stat">
          <span class="stat-label">Entries</span>
          <span class="stat-value purple" id="entries" style="color:var(--purple)">—</span>
        </div>
        <div class="stat">
          <span class="stat-label">Fill Rate</span>
          <span class="stat-value green" id="fill" style="color:var(--green)">—</span>
          <span class="stat-unit" id="fillUnit">% filled</span>
        </div>
        <div class="stat">
          <span class="stat-label">Total Mass</span>
          <span class="stat-value amber" id="mass" style="color:var(--amber)">—</span>
        </div>
      </div>
      <div class="ring-wrap">
        <div class="ring">
          <svg width="64" height="64" viewBox="0 0 80 80">
            <circle class="ring-bg" cx="40" cy="40" r="34" stroke-width="8"/>
            <circle class="ring-fg" id="ringFg" cx="40" cy="40" r="34" stroke-width="8" stroke-dasharray="213.6" stroke-dashoffset="213.6"/>
          </svg>
          <div class="ring-label" id="ringLabel">0%</div>
        </div>
        <div class="ring-info">
          <div class="label">Grid Utilization</div>
          <div class="desc" id="ringDesc">of 65,536 cells active</div>
        </div>
      </div>
    </div>

    <!-- Growth chart -->
    <div class="panel chart-panel">
      <div class="panel-head">
        <span>Neural Growth</span>
        <span class="badge" id="trendBadge">—</span>
      </div>
      <div style="padding:12px 16px">
        <canvas id="sparkline"></canvas>
      </div>
    </div>

    <!-- Activity log -->
    <div class="panel">
      <div class="panel-head">
        <span>Activity</span>
        <span class="badge" id="logCount">0</span>
      </div>
      <div class="log" id="log"></div>
    </div>
  </div>
</div>

<script>
const GRID_SIZE = 256;
const canvas = document.getElementById('grid');
const ctx = canvas.getContext('2d');
const sparkCanvas = document.getElementById('sparkline');
const sparkCtx = sparkCanvas.getContext('2d');

let history = [];
const MAX_HISTORY = 200;
let activityLog = [];
const MAX_LOG = 20;
let lastAlive = null;
let lastEntries = null;

function addLog(tag, msg) {
  const time = new Date().toLocaleTimeString('en-US', {hour12:false});
  activityLog.unshift({ time, tag, msg });
  if (activityLog.length > MAX_LOG) activityLog.pop();
  renderLog();
}

function renderLog() {
  const el = document.getElementById('log');
  document.getElementById('logCount').textContent = activityLog.length;
  if (activityLog.length === 0) {
    el.innerHTML = '<div class="empty">No activity yet</div>';
    return;
  }
  el.innerHTML = activityLog.map(l =>
    `<div class="log-item"><span class="log-time">${l.time}</span><span class="log-tag ${l.tag}">${l.tag.toUpperCase()}</span><span class="log-msg">${l.msg}</span></div>`
  ).join('');
}

async function update() {
  try {
    const [state, grid] = await Promise.all([
      fetch('/api/state').then(r => r.json()),
      fetch('/api/grid').then(r => r.json())
    ]);

    // Stats
    document.getElementById('alive').textContent = state.alive_cells.toLocaleString();
    document.getElementById('entries').textContent = state.entries.toLocaleString();
    document.getElementById('fill').textContent = state.fill_percent.toFixed(2);
    document.getElementById('fillUnit').textContent = '% filled';
    document.getElementById('mass').textContent = state.total_mass.toFixed(2);
    document.getElementById('fileSize').textContent = state.file_size_mb.toFixed(1) + ' MB';
    document.getElementById('gridBadge').textContent = state.alive_cells.toLocaleString() + ' alive';

    // Progress ring
    const circumference = 2 * Math.PI * 34; // 213.6
    const offset = circumference * (1 - state.fill_percent / 100);
    document.getElementById('ringFg').setAttribute('stroke-dashoffset', offset);
    document.getElementById('ringLabel').textContent = state.fill_percent.toFixed(1) + '%';
    document.getElementById('ringDesc').textContent = `${state.alive_cells.toLocaleString()} of ${state.total_cells.toLocaleString()} cells active`;

    // Status indicator
    document.getElementById('liveStatus').textContent = `LIVE · ${new Date().toLocaleTimeString('en-US',{hour12:false})}`;

    // Detect changes
    if (lastAlive !== null) {
      const delta = state.alive_cells - lastAlive;
      const deltaE = state.entries - lastEntries;
      if (delta !== 0 || deltaE !== 0) {
        if (deltaE > 0) addLog('learn', `+${deltaE} entries, ${delta >= 0 ? '+'+delta : delta} cells`);
        else if (delta < 0) addLog('dream', `${delta} cells (consolidation)`);
        else addLog('system', `+${delta} cells`);
      }
    }
    lastAlive = state.alive_cells;
    lastEntries = state.entries;

    // Draw grid heatmap
    const imageData = ctx.createImageData(GRID_SIZE, GRID_SIZE);
    for (let y = 0; y < GRID_SIZE; y++) {
      for (let x = 0; x < GRID_SIZE; x++) {
        const val = grid[y] ? grid[y][x] || 0 : 0;
        const idx = (y * GRID_SIZE + x) * 4;
        const [r, g, b] = heatColor(val);
        imageData.data[idx] = r;
        imageData.data[idx + 1] = g;
        imageData.data[idx + 2] = b;
        imageData.data[idx + 3] = 255;
      }
    }
    ctx.putImageData(imageData, 0, 0);

    // Track history (in-memory)
    history.push({ time: Date.now(), alive: state.alive_cells, entries: state.entries });
    if (history.length > MAX_HISTORY) history.shift();

    drawSparkline();
  } catch (e) {
    document.getElementById('liveStatus').textContent = 'OFFLINE';
  }
}

function heatColor(val) {
  if (val < 0.002) return [6, 6, 12];
  if (val < 0.005) return [20, 10, 50];
  if (val < 0.01) return [40, 20, 80];
  if (val < 0.02) return [0, 40, 120];
  if (val < 0.05) return [0, 80, 200];
  if (val < 0.1) return [0, 140, 220];
  if (val < 0.2) return [0, 200, 180];
  if (val < 0.3) return [0, 240, 120];
  if (val < 0.4) return [40, 255, 60];
  if (val < 0.5) return [120, 255, 20];
  if (val < 0.6) return [200, 255, 0];
  if (val < 0.7) return [255, 220, 0];
  if (val < 0.8) return [255, 160, 0];
  if (val < 0.9) return [255, 80, 40];
  return [255, 40, 40];
}

function drawSparkline() {
  const w = sparkCanvas.clientWidth;
  const h = sparkCanvas.clientHeight;
  sparkCanvas.width = w * window.devicePixelRatio;
  sparkCanvas.height = h * window.devicePixelRatio;
  sparkCtx.scale(window.devicePixelRatio, window.devicePixelRatio);
  sparkCtx.clearRect(0, 0, w, h);

  if (history.length < 2) return;

  const maxAlive = Math.max(...history.map(p => p.alive), 1);
  const minAlive = Math.min(...history.map(p => p.alive), 0);
  const range = maxAlive - minAlive || 1;

  // Grid lines
  sparkCtx.strokeStyle = 'rgba(30,30,46,.5)';
  sparkCtx.lineWidth = 0.5;
  for (let i = 0; i <= 4; i++) {
    const y = (h / 4) * i;
    sparkCtx.beginPath();
    sparkCtx.moveTo(0, y);
    sparkCtx.lineTo(w, y);
    sparkCtx.stroke();
  }

  // Filled area
  const gradient = sparkCtx.createLinearGradient(0, 0, 0, h);
  gradient.addColorStop(0, 'rgba(0,212,170,.25)');
  gradient.addColorStop(1, 'rgba(0,212,170,0)');
  sparkCtx.fillStyle = gradient;
  sparkCtx.beginPath();
  sparkCtx.moveTo(0, h);
  history.forEach((p, i) => {
    const x = (i / Math.max(history.length - 1, 1)) * w;
    const y = h - ((p.alive - minAlive) / range) * (h - 8) - 4;
    sparkCtx.lineTo(x, y);
  });
  sparkCtx.lineTo(w, h);
  sparkCtx.closePath();
  sparkCtx.fill();

  // Line
  sparkCtx.strokeStyle = '#00d4aa';
  sparkCtx.lineWidth = 1.5;
  sparkCtx.shadowColor = 'rgba(0,212,170,.4)';
  sparkCtx.shadowBlur = 4;
  sparkCtx.beginPath();
  history.forEach((p, i) => {
    const x = (i / Math.max(history.length - 1, 1)) * w;
    const y = h - ((p.alive - minAlive) / range) * (h - 8) - 4;
    if (i === 0) sparkCtx.moveTo(x, y);
    else sparkCtx.lineTo(x, y);
  });
  sparkCtx.stroke();
  sparkCtx.shadowBlur = 0;

  // Min/max labels
  sparkCtx.fillStyle = '#5a5a72';
  sparkCtx.font = '9px JetBrains Mono';
  sparkCtx.fillText(maxAlive.toLocaleString(), 4, 12);
  sparkCtx.fillText(minAlive.toLocaleString(), 4, h - 4);

  // Trend badge
  if (history.length >= 2) {
    const recent = history.slice(-5);
    const avg = recent.reduce((s,p) => s + p.alive, 0) / recent.length;
    const earlier = history.slice(0, Math.min(5, history.length - 5));
    const prevAvg = earlier.length > 0 ? earlier.reduce((s,p) => s + p.alive, 0) / earlier.length : avg;
    const trend = avg - prevAvg;
    const badge = document.getElementById('trendBadge');
    if (trend > 5) badge.textContent = '↑ ' + Math.round(trend);
    else if (trend < -5) badge.textContent = '↓ ' + Math.round(Math.abs(trend));
    else badge.textContent = '→ stable';
  }
}

// Load persisted history
async function loadHistory() {
  try {
    const data = await fetch('/api/history').then(r => r.json());
    if (Array.isArray(data) && data.length > 0) {
      history = data.map(p => ({ time: p[0] * 1000, alive: p[1], entries: p[2] }));
      addLog('system', `Loaded ${history.length} history points`);
      drawSparkline();
    }
  } catch(e) {}
}

// Resize sparkline on window change
window.addEventListener('resize', drawSparkline);

// Init
renderLog();
addLog('system', 'Dashboard connected');
update();
loadHistory();
setInterval(update, 5000);
</script>
</body>
</html>"#.to_string()
}