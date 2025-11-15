# 💾 Database Monitor TUI Screen

## Overview

A new TUI screen has been added to SAGE that displays real-time SpacetimeDB state in a beautiful terminal interface.

## Access

Press **`7`** from any screen to open the Database Monitor.

## Layout

The screen is divided into three columns:

### Left Column: Current State & Recent Metrics

**Current State (Top)**
- Generation number
- Current loss (color-coded: green < 0.05, yellow < 0.1, red > 0.3)
- Pattern being learned
- Complexity metric
- Diversity metric
- Training status (🧬 Active / ⏸ Paused)
- Last update timestamp

**Recent Metrics (Bottom)**
- Last 8 training metric records
- Shows generation, loss, and pattern
- Helps visualize training progression

### Middle Column: Patterns & Events

**Pattern Progress (Top)**
- All patterns with their status
- ✓ = Mastered (green)
- ○ = In Progress (yellow)
- Shows best loss achieved for each pattern

**Training Events (Bottom)**
- Recent significant events
- Icons:
  - ▶ Pattern start
  - 🎯 Pattern mastered
  - ⭐ Milestone
  - • Other events
- Shows generation and description

### Right Column: Conversations

**Full Conversation History**
- All messages between Claude and SAGE
- Color-coded by sender:
  - SAGE: Cyan
  - Claude: Magenta
  - Others: Yellow
- Shows generation context for each message
- Messages auto-truncate to fit

## Features

### Real-Time Updates
- Queries SpacetimeDB directly via CLI
- Updates on every render (when you switch to the screen)
- No caching - always fresh data

### Color Coding
- **Loss values**: Green (excellent) → Yellow (good) → Red (needs work)
- **Training status**: Green (active) / Yellow (paused)
- **Senders**: Distinct colors for easy identification

### Smart Truncation
- Long messages automatically truncated with "..."
- Optimized for terminal width
- All data readable at a glance

## Keyboard Shortcuts

- **`7`** - Switch to Database Monitor
- **`1-6`** - Switch to other screens
- **`R`** - Refresh (automatically refreshes on render)
- **`Q`** - Quit SAGE
- **`Esc`** - Return to Mission Control

## Sample View

```
┌─ 💾 SpacetimeDB Database Monitor │ 127.0.0.1:4000 │ sage-db ────────────────┐
│                                                                              │
│ ┌─ Current State ────┐ ┌─ Pattern Progress ──┐ ┌─ Conversations ─────────┐│
│ │ Generation: 100    │ │ ✓ 🔴 Circle  0.0500 │ │ Claude (gen 100)        ││
│ │ Loss: 0.0500       │ │ ○ 🟦 Square  0.1200 │ │ Hi SAGE! I have just... ││
│ │ Pattern: 🔴 Circle │ │                     │ │                          ││
│ │ Complexity: 0.500  │ │                     │ │ SAGE (gen 120)           ││
│ │ Diversity: 0.300   │ │                     │ │ This is amazing! I ca... ││
│ │ Training: 🧬 Active│ │                     │ │                          ││
│ │                    │ │                     │ │                          ││
│ │ Updated: 2025-11-13│ └─────────────────────┘ └──────────────────────────┘│
│ └────────────────────┘ ┌─ Training Events ───┐                             │
│ ┌─ Recent Metrics ───┐ │ 100 ⭐ Reached gen..│                             │
│ │  90 │ 0.0800 🔴... │ │                     │                             │
│ │  95 │ 0.0650 🔴... │ │                     │                             │
│ │ 100 │ 0.0500 🔴... │ │                     │                             │
│ └────────────────────┘ └─────────────────────┘                             │
└──────────────────────────────────────────────────────────────────────────────┘
[←/→] Navigate  [R] Refresh  [Q] Quit  [Tab] Change Screen
```

## Data Sources

All data comes directly from SpacetimeDB tables:

- **sage_state** - Current training state (single row, constantly updated)
- **training_metrics** - Historical time-series data
- **pattern_progress** - Pattern learning tracking
- **training_events** - Significant milestones
- **conversations** - Full chat history

## Performance

- Queries execute in milliseconds
- No polling overhead (only queries on render)
- Gracefully handles database unavailability
- Shows default/fallback data if DB is offline

## Integration with Training

To populate this screen with real data, integrate the SpacetimeDB client into your training loop:

```rust
use sage::spacetime_client::SageDbClient;

let db = SageDbClient::new("sage-db");

// Every 10 generations
if generation % 10 == 0 {
    db.update_sage_state(generation, loss, &pattern, complexity, diversity).ok();
}

// On pattern start
db.start_pattern(&pattern, generation).ok();

// On conversation
db.add_conversation_message("SAGE", &message, generation).ok();
```

## Troubleshooting

### "N/A" or Empty Data
- SpacetimeDB server may not be running
- Start with: `spacetime start --listen-addr 127.0.0.1:4000`

### No Recent Metrics
- Data hasn't been inserted yet
- Manually test with: `spacetime call sage-db update_sage_state -- 100 0.05 "🔴 Circle" 0.5 0.3`

### Conversations Not Showing
- Add test data: `spacetime call sage-db add_conversation_message -- "Claude" "Test message" 100`

## Future Enhancements

Potential improvements:
- Live refresh timer (auto-update every N seconds)
- Scrollable conversation history
- Loss curve sparkline graphs
- Pattern completion percentage
- Database connection status indicator
- Export/snapshot functionality

## Files

- **Screen**: `src/tui/screens/database_monitor.rs`
- **Module**: `src/tui/screens/mod.rs`
- **App**: `src/tui/app.rs` (keyboard shortcut `7` added)
- **Database Module**: `sage-db/src/lib.rs`
- **Client**: `src/spacetime_client.rs`
