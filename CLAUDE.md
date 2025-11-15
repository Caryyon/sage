# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**SAGE (Self-Adaptive General Explorer)** is an autonomous AGI research system based on Neural Cellular Automata (NCA) with 13 cognitive abilities. It learns to create complex patterns through self-organization and demonstrates meta-learning, curiosity-driven exploration, and self-modification.

Key differentiator: SAGE doesn't just learn patterns—it learns *how to learn* through curriculum learning, self-supervised tasks, and autonomous experimentation.

## Build & Run Commands

### Main Development
```bash
# Build release version (optimized, required for real-time performance)
cargo build --release

# Run SAGE Mission Control TUI
cargo run --release
```

### Hot-Reload System (NEW!)

SAGE now supports **hot-reloading** training logic without restarting! See `HOT_RELOAD.md` for full documentation.

```bash
# Test hot-reload
cargo run --release --example test_hot_reload

# In another terminal, make changes to sage-training/src/lib.rs, then:
cargo build --release -p sage-training
# Watch the first terminal automatically reload!
```

**What you can hot-reload**:
- Training hyperparameters (learning rates, batch sizes)
- Curriculum progression logic
- Pattern generation algorithms
- Loss/metrics calculations

**Note**: TUI integration is in progress. Current workflow still requires manual restart, but infrastructure is ready.

### SpacetimeDB Integration
SAGE uses SpacetimeDB for persistent state, metrics, and training history.

```bash
# Start SpacetimeDB server (required before running SAGE)
spacetime start --listen-addr 127.0.0.1:4000

# Publish the SAGE database module
cd sage-db
spacetime publish sage-db --project-path .

# Query training data
spacetime sql sage-db "SELECT * FROM training_metrics ORDER BY generation DESC LIMIT 20"

# View specific tables
spacetime sql sage-db "SELECT * FROM sage_state"
spacetime sql sage-db "SELECT * FROM pattern_progress WHERE is_mastered = true"
```

## Architecture Overview

### Core System: Neural Cellular Automata (NCA)

**NCA (`src/nca.rs`)**: The heart of SAGE. Each cell has 22 channels:
- 4 RGBA channels (visual state)
- 12 hidden channels (internal computation)
- 4 pattern condition channels (one-hot encoding for target patterns)
- 2 environmental channels (food/toxin gradients)

The NCA uses a 2-layer neural network (`UpdateNetwork`) with Adam optimizer. Each cell observes its 3×3 neighborhood via Sobel filters (perceive function in `src/grid.rs`) and decides how to update.

**Critical NCA Concepts:**
- **Stochastic updates**: Only 50% of cells update per step (prevents synchronization artifacts)
- **Living cell masking**: Dead cells (alpha < 0.1) stay dead
- **Batch processing**: Training processes 5 generations at once via Rayon parallelization
- **Weight snapshots**: Networks save/restore weights for pattern-specific training

### Training System (`src/tui/training.rs`)

**Spiral Curriculum Learning**:
- SAGE cycles through patterns: Circle → Square → Cross → Spiral → (repeat)
- Moves on after mastery (50 low-loss steps) OR 100 attempts (500 generations)
- Revisits previously learned patterns with new knowledge (transfer learning)
- Pattern-specific learning rates: Square gets 2× higher rate due to difficulty

**Pattern Types** (`TargetPattern` enum):
- **Circle**: Smooth radial gradient (easiest)
- **Square**: Sharp corners, uniform interior (hardest—edge discontinuities)
- **Cross**: Complex shape with multiple lobes (medium)
- **Spiral**: Logarithmic curve with gradients (complex)

### Metrics System

**Diversity**: Standard deviation of alpha channel values in *living cells only* (not empty space). Measures pattern complexity within the organism.
- Low (0.0-0.1): Uniform patterns (solid circle)
- Medium (0.1-0.2): Gradients, varied intensities
- High (0.2-0.4+): Complex patterns (spirals with intricate details)

**Complexity**: Average absolute difference between neighboring cells. Measures spatial variation and edge sharpness.

**Loss**: Mean squared error between NCA output and target pattern, calculated only on alpha channel.

### TUI (Terminal User Interface)

**Multi-threaded Architecture**:
- Main thread: Ratatui rendering and user input
- Background thread: NCA training loop (`TrainingRunner::run`)
- Shared state via `Arc<Mutex<AppState>>`

**Two Screens**:
1. **Unified Dashboard** (`src/tui/screens/unified_dashboard.rs`): Real-time training view with NCA grid visualization, loss/metrics, activity feed, and health diagnostics
2. **Database Monitor** (`src/tui/screens/database_monitor.rs`): Historical analytics with line charts (Loss Convergence, System Dynamics), pattern performance comparison, improvement rates

**Responsive Design**: TUI adapts to terminal size with breakpoints (height < 20, width < 80, etc.)

### 🔥 Hot-Reload System

SAGE now supports **live code updates** without restarting! The training logic has been extracted into a separate dynamic library that can be recompiled and reloaded on-the-fly.

**Architecture**:
- **`sage-training/`**: Separate crate containing hot-reloadable training logic
  - Compiles to `libsage_training.dylib` (dynamic library)
  - Stable `TrainingEngine` trait interface for ABI compatibility
  - Complete spiral curriculum implementation with all patterns
  - FFI exports: `create_engine()` and `engine_version()`

- **`src/tui/engine_loader.rs`**: Dynamic library loader using `libloading`
  - Loads `.dylib` files at runtime
  - File modification detection via timestamps
  - Version compatibility checking

- **`src/tui/hot_reload_runner.rs`**: Training runner with hot-reload support
  - Background thread for continuous training
  - Automatic reload detection
  - Checkpoint save/restore to preserve state across reloads

**Testing Hot-Reload**:
```bash
# Terminal 1: Run the hot-reload test
cargo run --release --example test_hot_reload

# Terminal 2: Make changes and rebuild
# Edit sage-training/src/lib.rs (change learning rates, curriculum logic, etc.)
cargo build --release -p sage-training

# Back in Terminal 1, you'll see:
# 🔄 LIBRARY CHANGED! Hot-reloading...
# 💾 Checkpoint saved: Gen 450
# 📚 New engine loaded! Version: 0.1.0
# ✅ HOT RELOAD COMPLETE!
```

**What You Can Hot-Reload**:
- ✅ Training hyperparameters (learning rates, batch size, evolution steps)
- ✅ Curriculum progression logic (mastery thresholds, patience limits)
- ✅ Pattern generation algorithms
- ✅ Loss calculation and metrics
- ✅ Event messages and logging

**What Requires Restart**:
- ❌ TrainingEngine trait interface changes (ABI breaking)
- ❌ TUI layout and rendering code
- ❌ Database schema changes

**Key Files**:
- `sage-training/src/lib.rs:174-279` - Main training step() implementation with spiral curriculum
- `sage-training/src/training_impl.rs` - Helper functions (pattern generation, metrics, batch training)
- `examples/test_hot_reload.rs` - Standalone hot-reload test harness
- `HOT_RELOAD.md` - Complete hot-reload system documentation

### Learning Subsystems (`src/learning/`)

**Meta-Learning** (`meta_learning.rs`):
- `AdaptiveLearningRate`: Adjusts learning rate based on loss trends (reduces by 0.5× if no improvement for 10 steps)
- `CurriculumLearner`: Estimates task difficulty and decides curriculum progression
- `LearningStrategy` enum: Aggressive/Balanced/Conservative/Adaptive

**Self-Supervised Learning** (`self_supervised.rs`):
- Next-state prediction: Learn temporal dynamics
- Masked reconstruction: Predict missing regions
- Contrastive learning: Distinguish similar/dissimilar patterns

**Pattern Generators** (`pattern_generators.rs`):
- Phase-based curriculum: Circles → Geometric → Clusters → Dynamic → Meta-patterns
- `NCAGenerator`: Uses evolved NCA to generate training data (meta!)

**Feature Extraction** (`feature_extractor.rs`):
- Extracts 13+ features: density, center of mass, symmetry, edge strength, gradient magnitude, etc.
- Used by curiosity engine and pattern classifier

### SpacetimeDB Schema (`sage-db/src/lib.rs`)

**Tables**:
- `sage_state`: Current SAGE state (single row, frequently updated)
- `training_metrics`: Time-series data (generation, loss, complexity, diversity)
- `network_snapshots`: Saved network weights at milestones
- `conversations`: Chat history with SAGE
- `pattern_progress`: Pattern mastery tracking
- `training_events`: Significant milestones ("pattern_mastered", "pattern_deferred", etc.)

**Reducers** (called via CLI with `spacetime call sage-db <reducer_name> <args>`):
- `update_sage_state`: Update current state
- `start_pattern`/`master_pattern`: Pattern lifecycle
- `save_network_snapshot`: Persist weights
- `log_training_event`: Log milestones
- `add_conversation_message`: SAGE communication

### Communication System (`src/communication.rs`)

SAGE has three "voices" based on training state:
- **Status updates**: Current generation/loss/pattern (filtered by loss thresholds)
- **Deep introspection**: Reflections on learning process (triggered at milestones)
- **Philosophical musings**: Existential thoughts about consciousness (rare)

Messages stored in `AppState.conversation_log` and persisted to SpacetimeDB.

## Key Implementation Details

### Pattern-Specific Training
Located in `src/tui/training.rs` around line 420-460:
```rust
// Different learning rates per pattern
let learning_rate = match current_pattern_type {
    TargetPattern::Square => 0.0002,  // 2× higher for hard pattern
    _ => 0.0001,
};

// Different evolution steps per pattern
let evolution_steps = match current_pattern_type {
    TargetPattern::Spiral => 120,
    TargetPattern::Square => 100,
    _ => 100,
};
```

### Metrics Calculation
Diversity fixed in training.rs lines 497-519 to only consider alive cells (alpha > 0.1):
```rust
let alive_values: Vec<f64> = grid.cells.iter()
    .flatten()
    .map(|cell| cell[3])
    .filter(|&alpha| alpha > 0.1)
    .collect();
```

### Spiral Curriculum Logic
Lines 639-672 in `src/tui/training.rs`:
- Pattern mastery: 50 consecutive steps with loss < 0.1
- Patience threshold: 100 attempts (500 generations with batch_size=5)
- Progress indicators shown every 25 attempts
- Revisiting on cycle 2+ with messages like "🔄 Revisiting: Square (Cycle 2)"

### State Persistence
`AppState` automatically saves to `/tmp/sage_state.json` on Ctrl+C and restores on startup. Network weights stored separately via SpacetimeDB.

## Development Workflow

1. **Make code changes** to core NCA, training loop, or TUI
2. **Ctrl+C** to stop SAGE (auto-saves state to /tmp/sage_state.json)
3. **Rebuild**: `cargo build --release`
4. **Restart SAGE**: `cargo run --release` (auto-restores state)
5. **Monitor in Database Monitor** (Tab key to switch screens)
6. **Query SpacetimeDB** for detailed metrics: `spacetime sql sage-db "SELECT * FROM training_metrics WHERE pattern = 'Square' ORDER BY generation DESC LIMIT 10"`

### Future Architecture Goal: Hot Reload

The original vision was to make the TUI a persistent client that could hot-reload SAGE's training logic/configs without restart - similar to module federation. This would enable:
- Live code updates without interrupting training
- SAGE self-modifying its own code
- Dynamic curriculum/pattern injection

**Potential Implementation**: Separate SAGE training into a dynamic library (cdylib) that the TUI loads via `dlopen`. Use file watchers to detect recompilation and reload the library. SpacetimeDB already provides the persistence layer needed for this architecture.

## Testing Strategy

No formal test suite currently. Testing is experiential:
- Run SAGE for 2000+ generations
- Verify spiral curriculum progresses through all 4 patterns
- Check diversity metric changes between patterns (Circle: ~0.05-0.15, Spiral: ~0.20-0.40)
- Confirm loss convergence (Square should reach < 0.1 eventually)
- Validate TUI responsiveness at different terminal sizes

## Common Patterns

### Adding a New Pattern
1. Add variant to `TargetPattern` enum in `src/tui/training.rs`
2. Create target grid generator function (follow `create_circle_target` pattern)
3. Add to `curriculum_progress` initialization with mastery flag
4. Add pattern-specific learning rate/evolution steps in training loop
5. Update match arms for pattern names (emoji + name)

### Adding a New Metric
1. Calculate in training loop around line 489-534
2. Add field to `AppState` struct in `src/tui/app.rs`
3. Update SpacetimeDB schema in `sage-db/src/lib.rs` (add to `TrainingMetrics` table)
4. Display in TUI screens (unified dashboard or database monitor)
5. Republish database: `cd sage-db && spacetime publish sage-db --project-path .`

### Modifying TUI Layouts
- Use responsive breakpoints: check `area.height` and `area.width`
- All charts use `ratatui::widgets::Chart` with `Marker::Braille` for smooth lines
- Color coding: Cyan for primary, Yellow for secondary, Red for errors, Green for success
- Keep ultra-compact mode (height < 10) in mind

## Important Constraints

- **Always build with `--release`**: Debug builds are 10-100× slower due to heavy linear algebra
- **SpacetimeDB must be running**: Start server before SAGE or database calls fail silently
- **Metrics calculated every 10 steps**: Performance optimization (see `should_update_metrics` in training.rs line 488)
- **Diversity only for alive cells**: Changed from measuring entire grid to measuring pattern complexity
- **Terminal size ≥ 80×24**: TUI degrades gracefully but optimal at 120×40+

## SpacetimeDB Client Notes

Currently uses CLI (`std::process::Command`) for simplicity. Client code in `src/spacetime_client.rs` is designed for easy migration to SDK (websocket-based) in the future.

Polling strategy: TUI polls SpacetimeDB every render (~60fps) via SQL queries. This is fine for local development but would need WebSocket subscriptions for production.
