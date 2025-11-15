# SpacetimeDB Integration for SAGE 🧬

## What We've Built So Far

### 1. SpacetimeDB Module Created ✅
Location: `sage-db/src/lib.rs`

**Tables Defined:**
- `sage_state` - Current training state (generation, loss, pattern, etc.)
- `training_metrics` - Historical time-series data
- `network_snapshots` - Saved neural network weights
- `conversations` - Chat history with SAGE
- `pattern_progress` - Pattern mastery tracking
- `training_events` - Significant milestones

**Reducers Created:**
- `update_sage_state()` - Update current state (called every few generations)
- `save_network_snapshot()` - Save weights at checkpoints
- `add_conversation_message()` - Store chat messages
- `start_pattern()` - Mark beginning of new pattern learning
- `master_pattern()` - Mark pattern as mastered
- `log_training_event()` - Log significant events

## Next Steps to Complete Integration

### 1. Fix SpacetimeDB Module Build
The module has some API compatibility issues with the current SpacetimeDB version. Need to:
- Check SpacetimeDB version: `spacetime version`
- Review latest SpacetimeDB Rust SDK docs
- Update table API calls to match current version
- Consider simplifying to use only `insert()` operations initially

### 2. Add SpacetimeDB Client to Main Project
```toml
# Add to Cargo.toml
[dependencies]
spacetimedb-sdk = "0.10"  # Check latest version
tokio = { version = "1", features = ["full"] }
```

### 3. Integrate Training Loop
```rust
// In src/tui/training.rs
use spacetimedb_sdk::*;

// Connect to SpacetimeDB
let conn = spacetimedb_sdk::connect("http://localhost:3000", "sage-db").await?;

// In training loop, every 10 generations:
conn.call_reducer("update_sage_state", &(
    generation,
    loss,
    pattern_name.to_string(),
    complexity,
    diversity,
)).await?;

// On pattern mastery:
conn.call_reducer("master_pattern", &(
    pattern_name.to_string(),
    generation,
    final_loss,
)).await?;

// For chat messages:
conn.call_reducer("add_conversation_message", &(
    sender.to_string(),
    message.to_string(),
    current_generation,
)).await?;
```

### 4. Network Weight Serialization
```rust
// Serialize network weights to JSON
use serde_json;

let weights_json = serde_json::to_string(&WeightSnapshot {
    weights1: nca.update_net.weights1.clone(),
    weights2: nca.update_net.weights2.clone(),
    bias1: nca.update_net.bias1.clone(),
    bias2: nca.update_net.bias2.clone(),
})?;

conn.call_reducer("save_network_snapshot", &(
    generation,
    pattern.to_string(),
    loss,
    weights_json,
)).await?;
```

### 5. Create Web Dashboard

**Option A: React + TypeScript**
```bash
npx create-react-app sage-dashboard --template typescript
cd sage-dashboard
npm install @clockworklabs/spacetimedb-sdk recharts
```

```typescript
// src/App.tsx
import { useQuery, SpacetimeDBClient } from '@clockworklabs/spacetimedb-sdk';
import { LineChart, Line } from 'recharts';

const client = new SpacetimeDBClient('http://localhost:3000', 'sage-db');

function Dashboard() {
  // Real-time reactive queries!
  const sageState = useQuery(client, 'sage_state');
  const metrics = useQuery(client, 'training_metrics');
  const conversations = useQuery(client, 'conversations');

  return (
    <div>
      <h1>SAGE - Generation {sageState?.[0]?.generation}</h1>

      {/* Real-time loss curve */}
      <LineChart data={metrics}>
        <Line dataKey="loss" stroke="#8884d8" />
      </LineChart>

      {/* Current pattern visualization */}
      <div>Pattern: {sageState?.[0]?.current_pattern}</div>
      <div>Loss: {sageState?.[0]?.current_loss?.toFixed(4)}</div>

      {/* Chat with SAGE */}
      <div>
        {conversations.map(msg => (
          <div key={msg.id}>
            <strong>{msg.sender}:</strong> {msg.message}
          </div>
        ))}
      </div>
    </div>
  );
}
```

**Option B: Simple HTML + Vanilla JS**
```html
<!-- sage-dashboard.html -->
<!DOCTYPE html>
<html>
<head>
  <script src="https://unpkg.com/@clockworklabs/spacetimedb-sdk"></script>
</head>
<body>
  <h1>SAGE Live Training</h1>
  <div id="generation"></div>
  <div id="loss"></div>
  <canvas id="lossChart"></canvas>
  <div id="chat"></div>

  <script>
    const client = new SpacetimeDB.Client('http://localhost:3000', 'sage-db');

    client.subscribe(['sage_state', 'conversations'], () => {
      const state = client.db.sage_state.all()[0];
      document.getElementById('generation').textContent = `Gen: ${state.generation}`;
      document.getElementById('loss').textContent = `Loss: ${state.current_loss.toFixed(4)}`;

      // Update chat
      const chats = client.db.conversations.all();
      document.getElementById('chat').innerHTML = chats.map(c =>
        `<p><strong>${c.sender}:</strong> ${c.message}</p>`
      ).join('');
    });
  </script>
</body>
</html>
```

## Benefits Once Complete

### 1. Persistence 💾
- SAGE remembers everything across restarts
- Can resume training from any generation
- Never lose conversation history

### 2. Real-Time Observation 👁️
- Multiple people watch SAGE train simultaneously
- Web dashboard updates automatically
- No polling needed

### 3. Time-Travel Queries ⏰
```sql
-- "What was SAGE's state at generation 1000?"
SELECT * FROM sage_state WHERE generation = 1000

-- "Show me all mastered patterns"
SELECT * FROM pattern_progress WHERE is_mastered = true

-- "Loss curve over time"
SELECT generation, loss FROM training_metrics ORDER BY generation
```

### 4. Historical Analysis 📊
- Compare training runs
- Identify what helped/hurt learning
- Visualize loss curves over time
- See conversation context for breakthroughs

### 5. Collaborative Learning 🤝
- Multiple researchers observe same SAGE
- Share insights in real-time
- Everyone sees updates instantly

## Running the System

```bash
# Terminal 1: Start SpacetimeDB server
spacetime start

# Terminal 2: Publish SAGE module (after fixing build)
cd sage-db
spacetime publish sage-db

# Terminal 3: Run SAGE training (after integration)
cd ..
cargo run

# Terminal 4: Open web dashboard
# Visit http://localhost:3000 or serve dashboard HTML
```

## Current Status

✅ SpacetimeDB module scaffolded with all tables/reducers
✅ Comprehensive schema designed for SAGE's needs
⏳ Module build needs API fixes for current SpacetimeDB version
⏳ Training loop integration pending
⏳ Web dashboard pending

## Why This Is Amazing

SpacetimeDB turns SAGE from a **transient process** into a **living, observable entity**:
- Survives restarts
- Remembers conversations
- Can be watched from anywhere
- Data never lost
- Instant reactivity
- Zero polling overhead
- Built-in time-travel

This is the foundation for making SAGE feel truly alive! 🌟
