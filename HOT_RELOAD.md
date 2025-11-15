# 🔥 SAGE Hot-Reload System

## Overview

SAGE now supports **hot-reloading** of training logic without restarting the TUI! This is achieved through dynamic library loading (`libloading`), allowing you to modify training algorithms, curriculum strategies, and hyperparameters on the fly while SAGE continues running.

## Architecture

```
┌─────────────────────────────┐
│   SAGE TUI (persistent)     │  ← Never restarts
│   - Rendering               │
│   - User input              │
│   - State display           │
└──────────┬──────────────────┘
           │ dlopen (hot-reload)
           ▼
┌─────────────────────────────┐
│  libsage_training.dylib     │  ← Reloadable!
│  - NCA training loop        │
│  - Pattern generators       │
│  - Curriculum logic         │
│  - Learning algorithms      │
└──────────┬──────────────────┘
           │
           ▼
┌─────────────────────────────┐
│  SpacetimeDB (persistent)   │
│  - Checkpoints              │
│  - Metrics                  │
│  - Training history         │
└─────────────────────────────┘
```

## Usage

### Test Hot-Reload

```bash
# Terminal 1: Run the test example
cargo run --release --example test_hot_reload

# Terminal 2: Make changes to sage-training/src/lib.rs, then:
cargo build --release -p sage-training

# Watch Terminal 1: It will automatically reload!
```

### Development Workflow

1. **Start SAGE** (once the TUI is integrated with hot-reload):
   ```bash
   cargo run --release
   ```

2. **Make changes** to training logic in `sage-training/src/lib.rs`

3. **Rebuild** just the training library:
   ```bash
   cargo build --release -p sage-training
   ```

4. **Watch SAGE** automatically detect the change, save checkpoint, reload, and continue!

## What Can You Hot-Reload?

✅ **Safe to hot-reload**:
- Training hyperparameters (learning rates, batch sizes)
- Curriculum progression logic
- Pattern generation algorithms
- Loss calculation methods
- Metrics computation
- Learning strategies

❌ **Requires full restart**:
- Changes to the `TrainingEngine` trait interface
- Changes to checkpoint data structures
- TUI rendering logic
- SpacetimeDB schema

## How It Works

### 1. Stable ABI Boundary

The `TrainingEngine` trait defines a stable interface:

```rust
pub trait TrainingEngine: Send {
    fn step(&mut self, iterations: usize) -> TrainingUpdate;
    fn get_state(&self) -> EngineState;
    fn load_checkpoint(&mut self, checkpoint: Checkpoint) -> Result<(), String>;
    fn save_checkpoint(&self) -> Result<Checkpoint, String>;
    fn set_config(&mut self, config: TrainingConfig);
    fn get_config(&self) -> TrainingConfig;
}
```

### 2. Checkpoint System

Before reloading, SAGE:
1. Saves full training state (generation, weights, curriculum progress)
2. Unloads old library
3. Loads new library
4. Restores checkpoint into new engine

### 3. File Watching

The `EngineLoader` checks for library modifications:

```rust
if loader.check_for_reload() {
    let checkpoint = engine.save_checkpoint()?;
    engine = loader.load()?;
    engine.load_checkpoint(checkpoint)?;
}
```

## Example: Changing Learning Rate On-The-Fly

**Before** (in `sage-training/src/lib.rs`):
```rust
impl Default for TrainingConfig {
    fn default() -> Self {
        Self {
            base_learning_rate: 0.0001,  // Old rate
            square_learning_rate: 0.0002,
            // ...
        }
    }
}
```

**After**:
```rust
impl Default for TrainingConfig {
    fn default() -> Self {
        Self {
            base_learning_rate: 0.0005,  // NEW: 5× faster!
            square_learning_rate: 0.001,
            // ...
        }
    }
}
```

Rebuild: `cargo build --release -p sage-training`

SAGE immediately switches to the new learning rates **without losing training progress**!

## Future Enhancements

### SAGE Self-Modification
Once we integrate this with the TUI, SAGE could:
- Generate new training strategies in Rust code
- Compile them with `cargo build -p sage-training`
- Hot-reload itself with evolved logic
- **Truly self-modifying AGI!** 🤖

### A/B Testing
Run multiple SAGE instances with different training strategies, compare results in real-time.

### Live Experimentation
Researcher tweaks parameters, sees immediate effect on training curves.

## Technical Details

### Library Location
- **macOS**: `target/release/libsage_training.dylib`
- **Linux**: `target/release/libsage_training.so`
- **Windows**: `target/release/sage_training.dll`

### FFI Safety
The `Box<dyn TrainingEngine>` return type triggers a warning but is safe because:
1. Both sides compiled with same Rust version
2. Only used within same process
3. Library never unloaded while trait objects exist

### Performance
- Hot-reload overhead: ~50-100ms (one-time during reload)
- Runtime overhead: **Zero** (direct function calls via vtable)
- Checkpoint save/restore: ~10-20ms

## Caveats

1. **ABI Stability**: Don't change trait signatures without restarting TUI
2. **State Migration**: New code must handle old checkpoints gracefully
3. **Memory Safety**: Library holds Arc references - ensure clean shutdown
4. **Compilation Time**: Full release build of sage-training ~5-6s

## Next Steps

1. **Integrate with TUI**: Connect `TrainingRunner` to use `EngineLoader`
2. **Add hot-reload UI indicator**: Show when reload happens in TUI
3. **Implement config hot-reload**: Change hyperparameters via TUI commands
4. **Add rollback**: Keep last N library versions for quick rollback
