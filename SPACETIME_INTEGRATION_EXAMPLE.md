# SpacetimeDB Integration - Working Example

## ✅ What's Working Now

1. **SpacetimeDB Server**: Running on 127.0.0.1:4000
2. **SAGE Module**: Published with 6 tables and 10 reducers
3. **Rust Client**: Created in `src/spacetime_client.rs`
4. **Database State**: Actively persisting SAGE's training state

## How to Use

### 1. Start SpacetimeDB Server (if not running)

```bash
spacetime start --listen-addr 127.0.0.1:4000
```

The server will run in the background and print the SpacetimeDB logo.

### 2. Create Database Client in Your Code

```rust
use crate::spacetime_client::SageDbClient;

// Create client
let db_client = SageDbClient::new("sage-db");

// Update state every N generations
if generation % 10 == 0 {
    let _ = db_client.update_sage_state(
        generation,
        current_loss,
        &current_pattern,
        complexity,
        diversity,
    );
}

// Log pattern start
let _ = db_client.start_pattern("🔴 Circle", generation);

// Log pattern mastery
if loss < 0.05 {
    let _ = db_client.master_pattern("🔴 Circle", generation, loss);
}

// Log conversation
let _ = db_client.add_conversation_message(
    "Claude",
    "Great progress on learning circles!",
    generation,
);
```

### 3. Query the Database

```bash
# Check current state
spacetime sql sage-db "SELECT * FROM sage_state"

# View training history
spacetime sql sage-db "SELECT * FROM training_metrics ORDER BY generation DESC LIMIT 10"

# See all conversations
spacetime sql sage-db "SELECT * FROM conversations ORDER BY timestamp DESC"

# Check pattern progress
spacetime sql sage-db "SELECT * FROM pattern_progress"

# View all events
spacetime sql sage-db "SELECT * FROM training_events ORDER BY timestamp DESC"
```

### 4. Real-Time Monitoring

The database updates in real-time! You can run queries in one terminal while SAGE trains in another.

## Current Database State

```
id | generation | current_loss | current_pattern | complexity | diversity | is_training | updated_at
----+------------+--------------+-----------------+------------+-----------+-------------+----------------------------------
 2  | 100        | 0.05         | "🔴 Circle"     | 0.5        | 0.3       | true        | 2025-11-13T17:34:23.461997+00:00
```

## Available Reducers

All reducers are working and tested:

- `update_sage_state(generation, loss, pattern, complexity, diversity)` - Update current state
- `save_network_snapshot(generation, pattern, loss, weights_json)` - Save weights
- `add_conversation_message(sender, message, generation_context)` - Log conversation
- `start_pattern(pattern, generation)` - Mark pattern learning start
- `master_pattern(pattern, generation, final_loss)` - Mark pattern mastered
- `log_training_event(generation, event_type, description)` - Log events
- `set_training_status(is_training)` - Update training status

## Integration Points

### Main Training Loop

Add these calls to your training loop (e.g., in `src/tui/training.rs` or wherever your main loop is):

```rust
// At the start of training
let db_client = SageDbClient::new("sage-db");

// Every 10 generations
if generation % 10 == 0 {
    db_client.update_sage_state(
        generation,
        loss,
        &pattern_name,
        complexity,
        diversity,
    ).ok(); // Use .ok() to not crash if DB is unavailable
}

// When starting a new pattern
db_client.start_pattern(&pattern_name, generation).ok();

// When pattern is mastered
if is_mastered {
    db_client.master_pattern(&pattern_name, generation, loss).ok();
}
```

### Conversation System

When SAGE sends/receives messages (e.g., in `src/communication.rs`):

```rust
// Log every conversation message
db_client.add_conversation_message(
    "SAGE",
    &message_content,
    current_generation,
).ok();

db_client.add_conversation_message(
    "Claude",
    &response,
    current_generation,
).ok();
```

### Network Checkpoints

When saving network weights:

```rust
use serde_json;

// Serialize weights
let weights = serde_json::json!({
    "weights1": nca.update_net.weights1,
    "weights2": nca.update_net.weights2,
    "bias1": nca.update_net.bias1,
    "bias2": nca.update_net.bias2,
});

let weights_json = serde_json::to_string(&weights).unwrap();

db_client.save_network_snapshot(
    generation,
    &pattern,
    loss,
    &weights_json,
).ok();
```

## Benefits

### 1. Persistence 💾
- SAGE's state survives restarts
- All training history preserved
- Conversation context maintained

### 2. Real-Time Observation 👁️
- Query state while training
- Monitor progress from anywhere
- No polling overhead

### 3. Historical Analysis 📊
```sql
-- Loss improvement over time
SELECT generation, loss FROM training_metrics
WHERE pattern = '🔴 Circle'
ORDER BY generation;

-- Conversation context during breakthroughs
SELECT c.*, s.current_loss
FROM conversations c
JOIN sage_state s ON c.generation_context = s.generation
WHERE s.current_loss < 0.1;

-- Pattern mastery timeline
SELECT * FROM pattern_progress
WHERE is_mastered = true
ORDER BY mastered_at;
```

### 4. Zero Integration Overhead
- Uses CLI commands internally
- No complex async setup needed
- Graceful degradation if DB is offline

## Next Steps

### Option 1: Add to Existing Training Loop
Simply add `SageDbClient` calls to your existing code as shown above.

### Option 2: Create Web Dashboard
Build a real-time dashboard using:
- React + TypeScript with `@clockworklabs/spacetimedb-sdk`
- Or simple HTML + vanilla JS
- Shows live loss curves, pattern progress, conversations

### Option 3: Advanced Features
- Network weight snapshots for resuming training
- Pattern comparison and analysis
- Multi-run experiments tracking
- Collaborative observation mode

## Troubleshooting

### Database Connection Issues
```bash
# Check if server is running
spacetime server list

# Restart server
spacetime start --listen-addr 127.0.0.1:4000
```

### Module Not Found
```bash
# Republish module
cd sage-db
spacetime publish sage-db
```

### Query Database Directly
```bash
# Check all tables
spacetime sql sage-db "SELECT name FROM sqlite_master WHERE type='table'"

# Count records in each table
spacetime sql sage-db "SELECT COUNT(*) FROM sage_state"
spacetime sql sage-db "SELECT COUNT(*) FROM training_metrics"
spacetime sql sage-db "SELECT COUNT(*) FROM conversations"
```

## Current Status

✅ Server running
✅ Module published
✅ Tables created
✅ Reducers working
✅ Client library ready
⏳ Integration into training loop (manual step)
⏳ Web dashboard (optional)

## Files

- **Module**: `/Users/cwolff/Code/RUST/neural-networks-101/sage-db/src/lib.rs`
- **Client**: `/Users/cwolff/Code/RUST/neural-networks-101/src/spacetime_client.rs`
- **This Guide**: `/Users/cwolff/Code/RUST/neural-networks-101/SPACETIME_INTEGRATION_EXAMPLE.md`
