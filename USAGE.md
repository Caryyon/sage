# SAGE Unified System - Usage Guide

The new unified SAGE system combines all functionality into a single command with easy-to-use flags.

## Quick Start

```bash
# Just TUI training (default)
cargo run --release

# TUI + IRC bot with full consciousness
cargo run --release --all

# TUI + IRC bot only
cargo run --release --irc

# TUI + Vision system
cargo run --release --vision

# TUI + Autonomous consciousness (dreams + curiosity)
cargo run --release --autonomous

# Everything combined
cargo run --release --irc --vision --autonomous
# or simply:
cargo run --release --all
```

## Detailed Usage

### TUI Only (Default)
```bash
cargo run --release
```
Launches:
- ✅ TUI Mission Control
- ✅ NCA Training
- ✅ Real-time visualization
- ✅ SpacetimeDB integration

### With IRC Bot
```bash
cargo run --release --irc
```
Adds:
- ✅ IRC conversational interface
- ✅ LLM-enhanced responses (Ollama)
- ✅ Tool use (search, weather, wiki, etc.)
- ✅ A/B testing (NCA vs baseline)
- ✅ IRC tab in TUI showing chat logs

### With Vision
```bash
cargo run --release --vision
```
Adds:
- ✅ Camera capture
- ✅ Visual feature extraction
- ✅ Visual memory storage
- ✅ Cross-modal learning
- ✅ Vision tab in TUI showing camera feed

###With Autonomous Consciousness
```bash
cargo run --release --autonomous
```
Adds:
- ✅ Dream Mode (memory consolidation during idle)
- ✅ Curiosity Mode (proactive question asking)
- ✅ Vision→Dream→Learn loop
- ✅ Dreams tab in TUI showing autonomous thoughts

### Full System
```bash
cargo run --release --all
```
Launches everything:
- ✅ TUI Mission Control
- ✅ IRC Bot (LLM + tools)
- ✅ Vision System (camera + memory)
- ✅ Autonomous Consciousness (dreams + curiosity)
- ✅ All subsystems running in parallel threads

## Subcommands (Alternative Usage)

### IRC Only (No TUI)
```bash
# Basic IRC bot
cargo run --release irc

# IRC bot with vision
cargo run --release irc --vision

# IRC bot with autonomous mode
cargo run --release irc --autonomous

# IRC bot with everything
cargo run --release irc --vision --autonomous
```

### Vision Testing
```bash
cargo run --release vision
```
Launches standalone vision test (camera preview + feature extraction)

### Autonomous Only
```bash
cargo run --release autonomous
```
Launches autonomous consciousness without TUI (logs to `/tmp/sage_autonomous_thoughts.log`)

## Architecture

```
┌─────────────────────────────────────────┐
│         TUI Mission Control (Main Thread)│
│  ┌─────────┬─────────┬────────┬────────┐│
│  │Training │IRC Chat │ Vision │Dreams  ││
│  │(Default)│  (Tab)  │ (Tab)  │ (Tab)  ││
│  └─────────┴─────────┴────────┴────────┘│
└─────────────────────────────────────────┘
         │          │         │        │
    ┌────┴────┐┌────┴───┐┌───┴───┐┌───┴───────┐
    │NCA      ││IRC Bot ││Vision ││Autonomous │
    │Training ││Thread  ││Thread ││Thread     │
    │Thread   ││(LLM)   ││(Cam)  ││(Dreams)   │
    └─────────┘└────────┘└───────┘└───────────┘
```

All threads communicate via shared state (`Arc<Mutex<T>>`) and channel messages.

## Examples

### Development (Training Only)
```bash
cargo run --release
```
Focus on NCA training, monitoring loss/metrics

### Full Deployment (Production Mode)
```bash
cargo run --release --all
```
Run complete autonomous AI with all capabilities

### IRC Bot Deployment
```bash
cargo run --release irc --vision --autonomous
```
Run IRC bot with vision and consciousness (no TUI overhead)

### Vision Research
```bash
cargo run --release --vision
```
Focus on visual perception experiments

## Environment Variables

```bash
# Ollama API endpoint (default: http://localhost:11434)
export OLLAMA_API_URL=http://localhost:11434

# IRC server (default: silver.libera.chat)
export IRC_SERVER=irc.libera.chat

# IRC channel (default: #sage-ai)
export IRC_CHANNEL=#my-channel

# SpacetimeDB endpoint (default: 127.0.0.1:4000)
export SPACETIME_ADDR=127.0.0.1:4000
```

## Keyboard Shortcuts

### TUI Navigation
- `Tab` - Switch between tabs (Training → IRC → Vision → Dreams)
- `q` - Quit
- `r` - Refresh current view
- `d` - Toggle debug mode
- `Ctrl+C` - Emergency shutdown (saves state)

### IRC Tab
- `i` - Focus input field
- `Esc` - Unfocus input
- `↑/↓` - Scroll chat history

### Vision Tab
- `s` - Capture snapshot
- `f` - Toggle feature overlay
- `m` - View visual memory

### Dreams Tab
- `Space` - Trigger dream cycle manually
- `c` - Toggle curiosity mode

## Migration from Examples

**Old way (fragmented):**
```bash
# Terminal 1
cargo run --release

# Terminal 2
cargo run --release --example sage_irc_autonomous

# Terminal 3
./sage-listen
```

**New way (unified):**
```bash
# Single terminal
cargo run --release --all
```

Everything in one TUI with multiple tabs!

## Troubleshooting

### Camera Lock Error
If you get "Camera locked by another process":
```bash
# Kill all SAGE processes
pkill -9 sage

# Restart with vision
cargo run --release --all
```

### IRC Connection Issues
```bash
# Check Ollama is running
curl http://localhost:11434/api/tags

# Test IRC connection
cargo run --release irc
```

### SpacetimeDB Not Running
```bash
# Start SpacetimeDB
spacetime start --listen-addr 127.0.0.1:4000

# Verify
spacetime server list
```

## Next Steps

1. Run default TUI: `cargo run --release`
2. Try with IRC: `cargo run --release --irc`
3. Add vision: `cargo run --release --irc --vision`
4. Full system: `cargo run --release --all`
5. Explore tabs with `Tab` key
6. Check logs in `/tmp/sage_*.log`
