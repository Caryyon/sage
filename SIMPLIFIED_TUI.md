# 🎯 Simplified TUI - Two Screens Only

## Overview

SAGE's TUI has been streamlined to just two essential screens:
1. **Main Dashboard** - Neural training visualization
2. **Database Monitor** - SpacetimeDB state viewer

All dead code and unused screens have been removed.

## Navigation

### Simple Tab Toggle
Press **`Tab`** to switch between the two screens:
- From Main Dashboard → Database Monitor
- From Database Monitor → Main Dashboard

No more numbered keys, no more complex navigation. Just Tab!

## Keyboard Shortcuts (Works on Both Screens)

| Key | Action |
|-----|--------|
| `Tab` | Toggle between Dashboard and Database Monitor |
| `Space` | Pause/Resume training |
| `N` | Start training |
| `Q` | Quit SAGE |

## Main Dashboard

Shows:
- Neural field visualization (left)
- Training metrics (right top)
- Chat messages (right bottom)
- Current generation, loss, pattern
- Training status

**Footer:**
```
[Tab] Database Monitor  [Space] Pause/Resume  [N] Start Training  [Q] Quit
```

## Database Monitor

Shows real-time SpacetimeDB data in three columns:
- **Left**: Current state + recent metrics
- **Middle**: Pattern progress + training events
- **Right**: Conversation history

**Footer:**
```
[Tab] Main Dashboard  [Space] Pause/Resume  [N] Start Training  [Q] Quit
```

## What Was Removed

The following screens have been deleted (all dead code):
- ❌ Mission Control
- ❌ Civilization Theater
- ❌ Mind Observatory
- ❌ Learning Lab
- ❌ Timeline
- ❌ Mind Dialog

**Files deleted:**
- `src/tui/screens/mission_control.rs`
- `src/tui/screens/civilization_theater.rs`
- `src/tui/screens/mind_observatory.rs`
- `src/tui/screens/learning_lab.rs`
- `src/tui/screens/timeline.rs`
- `src/tui/screens/mind_dialog.rs`

## What Remains

**Only 2 screen files:**
- ✅ `src/tui/screens/unified_dashboard.rs` - Main training dashboard
- ✅ `src/tui/screens/database_monitor.rs` - SpacetimeDB viewer

**Core files updated:**
- `src/tui/screens/mod.rs` - Only exports 2 screens now
- `src/tui/app.rs` - Simplified keyboard handling (just Tab toggle)

## Code Changes Summary

### src/tui/screens/mod.rs
```rust
// Before: 7+ screens
pub enum ScreenType {
    UnifiedDashboard,
    MissionControl,
    CivilizationTheater,
    MindObservatory,
    LearningLab,
    Timeline,
    MindDialog,
    DatabaseMonitor,
}

// After: 2 screens
pub enum ScreenType {
    UnifiedDashboard,
    DatabaseMonitor,
}
```

### src/tui/app.rs
```rust
// Before: Numbered keys 1-7, Esc handler, Mind Dialog input mode
KeyCode::Char('1') => Action::SwitchScreen(ScreenType::MissionControl),
KeyCode::Char('2') => Action::SwitchScreen(ScreenType::CivilizationTheater),
// ... etc

// After: Simple Tab toggle
KeyCode::Tab => {
    let next_screen = match self.state.current_screen {
        ScreenType::UnifiedDashboard => ScreenType::DatabaseMonitor,
        ScreenType::DatabaseMonitor => ScreenType::UnifiedDashboard,
    };
    Action::SwitchScreen(next_screen)
}
```

## Benefits

1. **Cleaner Codebase** - Removed thousands of lines of dead code
2. **Simpler UX** - Just Tab to toggle, no numbered keys to remember
3. **Faster Builds** - Fewer files to compile
4. **Better Focus** - Two screens that actually matter
5. **Easier Maintenance** - Less code = fewer bugs

## Usage

```bash
# Start SpacetimeDB (if not running)
spacetime start --listen-addr 127.0.0.1:4000

# Run SAGE
cargo run

# Press Tab to switch between screens
# Press Q to quit
```

## Architecture

```
SAGE TUI
├── Main Dashboard (default)
│   ├── Neural field (left panel)
│   ├── Training metrics (right top)
│   └── Chat history (right bottom)
│
└── Database Monitor (Tab to access)
    ├── Current state + metrics (left)
    ├── Patterns + events (middle)
    └── Conversations (right)
```

## Future Enhancements

With the streamlined architecture, future additions could include:
- Auto-refresh timer for Database Monitor
- Sparkline loss graphs
- Pattern completion percentage
- Real-time training rate display
- Export functionality

But for now, we keep it simple: two screens, one toggle key. 🎯
