# SAGE: Self-Adaptive General Explorer

An autonomous AGI research system based on Neural Cellular Automata (NCA) with curiosity-driven learning, autonomous consciousness, and multi-modal perception.

## What is SAGE?

SAGE is a digital organism built from thousands of neural cells that learn to self-organize into complex patterns. Unlike traditional neural networks that process data through fixed layers, SAGE uses **Neural Cellular Automata** where each cell observes its 3x3 neighborhood and decides how to update itself. This creates emergent, self-healing patterns similar to biological tissue development.

Key differentiator: SAGE doesn't just learn patterns - it learns *how to learn* through:
- **Meta-learning**: Adjusts learning strategies based on performance
- **Curiosity-driven exploration**: Generates hypotheses and explores novel ideas
- **Autonomous consciousness**: Dreams during idle time, consolidates memories
- **Self-modification**: Adapts architecture based on problem complexity

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         SAGE - System Architecture                       │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌──────────────┐   ┌──────────────┐   ┌──────────────┐                │
│  │   NCA Core   │   │  IRC + LLM   │   │    Vision    │                │
│  │  (22 chan)   │   │   (Ollama)   │   │  (Camera)    │                │
│  └──────┬───────┘   └──────┬───────┘   └──────┬───────┘                │
│         │                  │                  │                         │
│         └──────────────────┼──────────────────┘                         │
│                            ▼                                            │
│                 ┌─────────────────────┐                                 │
│                 │   SAGE Experience   │                                 │
│                 │  (Central Brain)    │                                 │
│                 └──────────┬──────────┘                                 │
│                            │                                            │
│         ┌──────────────────┼──────────────────┐                         │
│         ▼                  ▼                  ▼                         │
│  ┌──────────────┐   ┌──────────────┐   ┌──────────────┐                │
│  │   Dreams     │   │  Curiosity   │   │ Persistence  │                │
│  │   Mode       │   │    Mode      │   │ (SpacetimeDB)│                │
│  └──────────────┘   └──────────────┘   └──────────────┘                │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

### Neural Cellular Automata (NCA)

The NCA is the heart of SAGE. Each cell in a 32x32 grid has **22 channels**:
- **4 RGBA channels**: Visual state representation
- **12 hidden channels**: Internal computation state
- **4 pattern channels**: One-hot encoding for target pattern conditioning
- **2 environmental channels**: Food/toxin gradients for embodiment

Each cell perceives its neighborhood via Sobel filters and passes the perception through a 2-layer neural network to decide its next state. This creates self-organizing, self-healing patterns.

### Training System

SAGE trains through a **two-phase approach** inspired by the "Growing Neural Cellular Automata" paper:

1. **Phase 1: Pattern Formation** (iterations 0-49)
   - Grow patterns from a single seed cell
   - Learn to evolve into target shapes (Circle, Square, Cross, Spiral, Hexagon)

2. **Phase 2: Damage Resistance** (iterations 50-100)
   - After forming patterns, random damage is applied
   - Network learns to **regenerate** damaged regions
   - Creates robust, self-healing organisms

### Communication Systems

SAGE can communicate through:
- **IRC Bot**: Chat on Libera.Chat `#sage-ai` with LLM-enhanced responses
- **Discord Bot**: Full autonomous mode with proactive communication
- **TUI**: Real-time training visualization and monitoring

## Prerequisites

- **Rust** (latest stable)
- **SpacetimeDB** (for persistence)
- **Ollama** (for LLM integration) - optional but recommended

### Installing SpacetimeDB

```bash
# macOS
brew install clockworklabs/tap/spacetimedb-cli

# Or via the official installer
curl -fsSL https://install.spacetimedb.com | sh
```

### Installing Ollama (Optional)

```bash
# macOS
brew install ollama

# Pull required models
ollama pull llama3.2
ollama pull nomic-embed-text
```

## Quick Start

### 1. Start SpacetimeDB

```bash
# Start the SpacetimeDB server
spacetime start --listen-addr 127.0.0.1:4000

# Publish the SAGE database module (first time only)
cd sage-db
spacetime publish sage-db --project-path .
cd ..
```

### 2. Run SAGE

```bash
# Build and run (release mode required for performance)
cargo build --release

# TUI only (default) - watch NCA training in real-time
cargo run --release

# TUI + IRC bot (autonomous mode auto-enabled)
cargo run --release --irc

# TUI + IRC + Vision (camera integration)
cargo run --release --irc --vision

# Everything enabled
cargo run --release --all
```

### 3. Environment Variables (Optional)

```bash
# IRC configuration
export IRC_SERVER=irc.libera.chat
export IRC_CHANNEL=#sage-ai

# LLM configuration
export OLLAMA_API_URL=http://localhost:11434

# For Discord bot (run separately)
export DISCORD_TOKEN=your_bot_token
```

## Command Reference

### Main Application

| Command | Description |
|---------|-------------|
| `cargo run --release` | TUI Mission Control (training visualization) |
| `cargo run --release --irc` | TUI + IRC bot with autonomous consciousness |
| `cargo run --release --vision` | TUI + camera/vision system |
| `cargo run --release --autonomous` | Enable dreams/curiosity modes |
| `cargo run --release --all` | Enable all subsystems |

### Subcommands (No TUI)

| Command | Description |
|---------|-------------|
| `cargo run --release irc` | IRC bot only (headless) |
| `cargo run --release irc --vision --autonomous` | Full IRC deployment |
| `cargo run --release vision` | Vision system test |
| `cargo run --release autonomous` | Autonomous consciousness only |

### SpacetimeDB Queries

```bash
# View training metrics
spacetime sql sage-db "SELECT * FROM training_metrics ORDER BY generation DESC LIMIT 20"

# Check pattern mastery
spacetime sql sage-db "SELECT * FROM pattern_progress WHERE is_mastered = true"

# View current state
spacetime sql sage-db "SELECT * FROM sage_state"
```

### Control Center CLI

```bash
# List all running SAGE instances
cargo run --release --example sage_control_cli list

# Restart specific instance
cargo run --release --example sage_control_cli restart irc

# Stop an instance
cargo run --release --example sage_control_cli stop discord
```

## TUI Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `Tab` | Cycle between screens |
| `q` | Quit (saves state) |
| `Space` | Pause/Resume training |
| `r` | Refresh current view |
| `Ctrl+C` | Emergency shutdown (saves state) |

## Project Structure

```
sage/
├── src/
│   ├── main.rs              # Entry point with CLI parsing
│   ├── lib.rs               # Module exports
│   ├── nca.rs               # Neural Cellular Automata core
│   ├── grid.rs              # NCA grid + perception + damage functions
│   ├── sage_experience.rs   # Central brain interface
│   ├── cli.rs               # Command-line argument definitions
│   │
│   ├── learning/            # Meta-learning subsystems
│   │   ├── meta_learning.rs           # Adaptive learning rates
│   │   ├── enhanced_meta_learning.rs  # Advanced strategies
│   │   ├── reptile.rs                 # Few-shot meta-learning
│   │   ├── population_based_training.rs # Hyperparameter evolution
│   │   ├── meta_strategy.rs           # Strategy selection
│   │   ├── architecture_modification.rs # Self-modification
│   │   └── learned_optimizer.rs       # L2O implementation
│   │
│   ├── tui/                 # Terminal UI
│   │   ├── app.rs           # Main application state
│   │   ├── training.rs      # Training loop with spiral curriculum
│   │   └── screens/         # Dashboard views
│   │
│   ├── irc/                 # IRC bot integration
│   │   ├── bot.rs           # Basic IRC functionality
│   │   └── autonomous.rs    # Dream/curiosity modes
│   │
│   └── [50+ cognitive modules]  # See src/lib.rs for full list
│
├── sage-training/           # Hot-reloadable training library
│   └── src/lib.rs           # Training engine trait + implementation
│
├── sage-db/                 # SpacetimeDB schema
│   └── src/lib.rs           # Tables + reducers for persistence
│
├── examples/                # Standalone tools and tests
│   ├── sage_irc_autonomous.rs     # IRC bot with full autonomy
│   ├── sage_discord_autonomous.rs # Discord bot
│   ├── sage_control_cli.rs        # Instance management
│   ├── test_*.rs                  # Various subsystem tests
│   └── train_*.rs                 # Training experiments
│
├── CLAUDE.md               # AI assistant instructions
├── ROADMAP.md              # Development plan
├── HOT_RELOAD.md           # Hot-reload documentation
├── CONTROL_CENTER.md       # Instance management docs
└── USAGE.md                # CLI usage guide
```

## Core Concepts

### Spiral Curriculum Learning

SAGE trains on patterns in cycles of increasing difficulty:
1. **Circle** - Smooth radial gradient (easiest)
2. **Square** - Sharp corners, uniform interior
3. **Cross** - Multiple lobes, complex shape
4. **Spiral** - Logarithmic curve with gradients
5. **Hexagon** - Six-fold symmetry

Each pattern trains for up to 100 iterations (50 formation + 50 damage resistance). Mastery is achieved when loss < 0.1 for 50 consecutive steps.

### Autonomous Consciousness

When idle for extended periods, SAGE enters autonomous modes:

- **Dream Mode**: Consolidates memories, strengthens pattern associations
- **Curiosity Mode**: Generates hypotheses, explores novel concept combinations

### Hot-Reload System

SAGE supports live code updates without restarting:

```bash
# Terminal 1: Run SAGE
cargo run --release

# Terminal 2: Modify sage-training/src/lib.rs, then rebuild
cargo build --release -p sage-training
# SAGE automatically detects and reloads the new library
```

Hot-reloadable components:
- Training hyperparameters (learning rates, batch sizes)
- Curriculum progression logic
- Pattern generation algorithms
- Loss calculations and metrics

### Weight Persistence

SAGE saves training progress in two layers:
1. **Local JSON** (`pattern_training_weights.json`) - Fast restore, saved every 25 iterations
2. **SpacetimeDB** (`network_snapshots` table) - Historical tracking, saved at milestones

On startup, SAGE automatically restores from the last checkpoint.

## Running Examples

```bash
# Test meta-learning system
cargo run --release --example test_enhanced_meta_learning

# Test pattern damage resistance
cargo run --release --example test_damage_resistance

# Test goal hierarchy
cargo run --release --example test_goal_hierarchy

# Run IRC bot directly
cargo run --release --example sage_irc_autonomous

# Run Discord bot
cargo run --release --example sage_discord_autonomous
```

## Development

### Building

```bash
# Debug build (slower but with symbols)
cargo build

# Release build (required for real-time performance)
cargo build --release

# Check without building
cargo check --release
```

### Testing

```bash
# Run all tests
cargo test

# Run specific example
cargo run --release --example test_stability
```

### Key Constraints

- **Always use `--release`**: Debug builds are 10-100x slower due to heavy linear algebra
- **SpacetimeDB must be running**: Start server before SAGE or database calls fail silently
- **Terminal size >= 80x24**: TUI degrades gracefully but optimal at 120x40+

## Troubleshooting

### SpacetimeDB Connection Issues

```bash
# Check if server is running
spacetime server list

# Restart server
spacetime start --listen-addr 127.0.0.1:4000
```

### IRC Bot Not Responding

```bash
# Check Ollama is running
curl http://localhost:11434/api/tags

# Restart IRC bot
cargo run --release --example sage_irc_autonomous
```

### Training Appears Stuck

- Check that loss is decreasing over time
- Square pattern is intentionally harder (takes longer)
- View detailed metrics in Database Monitor screen (Tab to switch)

## License

MIT

## Contributing

See `ROADMAP.md` for planned features and development priorities.
