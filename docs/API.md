# SAGE API Reference

## REST Endpoints (Web Dashboard)

The web dashboard is served by `src/web_dashboard/` using Axum, default port configurable via `start_server(port, db)`.

All API responses use a standard envelope:

```json
{
  "success": true|false,
  "data": <T> | null,
  "error": "message" | null
}
```

### Instance Management

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/api/instances` | List all registered SAGE instances with status, role, expertise level, task counts, success rate |
| `GET` | `/api/instances/:id` | Get details for a specific instance (includes pending approval count) |

**Response: `InstanceInfo`**
```json
{
  "instance_id": "lumin-1",
  "name": "lumin-1",
  "role": "content_creator",
  "status": "online",
  "expertise_level": "intermediate",
  "total_tasks": 42,
  "success_rate": 85.7,
  "pending_approvals": 3
}
```

### Human-in-the-Loop Approvals

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/api/pending` | List all pending action drafts awaiting approval |
| `POST` | `/api/approve/:id` | Approve a pending action draft |
| `POST` | `/api/reject/:id` | Reject a pending action draft. Query param: `?reason=<text>` |

**Response: `PendingApproval`**
```json
{
  "id": 1,
  "instance_id": "lumin-1",
  "action_type": "post_tweet",
  "description": "Post about new feature launch",
  "risk_level": "medium"
}
```

### Expertise Tracking

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/api/expertise/:id` | Get expertise details: role, level, skill scores, milestones |

**Response: `ExpertiseInfo`**
```json
{
  "instance_id": "lumin-1",
  "role": "content_creator",
  "level": "intermediate",
  "overall_score": 72.5,
  "skills": [{"name": "writing", "score": 0.8, "examples_seen": 150}],
  "milestones": [{"id": "first_post", "name": "First Published Post"}]
}
```

### Task History

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/api/tasks/:id` | Get last 50 tasks for an instance |

**Response: `TaskInfo[]`**
```json
{
  "id": 1,
  "task_type": "write_blog",
  "input_summary": "Write about Rust async",
  "output_summary": "1200 word blog post",
  "success": true,
  "human_approved": true,
  "execution_time_ms": 4500
}
```

### Health & Static

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/api/health` | Returns `"OK"` |
| `GET` | `/` | Dashboard HTML (embedded from `static/dashboard/index.html`) |
| `GET` | `/static/*` | Static files from `static/dashboard/` |

---

## WebSocket Endpoints

### Dashboard Real-time Updates

**Endpoint:** `GET /api/ws` (upgrade to WebSocket)

Pushes `DashboardUpdate` every 5 seconds:

```json
{
  "timestamp": 1706000000,
  "instances": [<InstanceInfo>, ...],
  "pending_count": 3
}
```

The server reads incoming messages but currently only handles `Close`.

### Miniworld Server

**Binary:** `miniworld_server` (or `sage_city_server` example)  
**Endpoint:** `GET /ws` — default port `8888` (env `PORT`)

Pushes `WorldStateMessage` every 100ms (10 ticks/sec):

```json
{
  "type": "world_state",
  "world": {
    "config": {"width": 64, "height": 64, "name": "SAGE Village"},
    "tiles": [[{"ground": "Grass", "overlay": null, "sprite_col": 0, "sprite_row": 0, "team_color": null}]],
    "characters": {
      "sage-1": {
        "id": "sage-1", "name": "Lumin", "x": 10, "y": 20,
        "direction": "down", "state": "walking",
        "sprite": "Sage1", "anim_frame": 0
      }
    },
    "time_of_day": 720,
    "tick": 1000
  }
}
```

Incoming text messages are logged (for future interactivity).

### NCA Dashboard (`sage_web_dashboard` example)

**Port:** `3030`  
**Endpoint:** `GET /ws`

Broadcasts NCA grid state updates for real-time neural visualization.

---

## Available Binaries & Examples

### Main Binary: `sage`

```bash
cargo run --release
```

TUI Mission Control using ratatui. Launches the NCA training dashboard with optional subsystems controlled via CLI flags (IRC, vision, autonomous mode). Registers with the instance control center.

### Binary: `miniworld_server`

```bash
cargo run --release --bin miniworld_server
# PORT=8888 by default
```

Standalone WebSocket server for the SAGE Village 2D tile simulation. Serves static HTML/JS from `static/miniworld/`.

### Binary: `sage_vision`

```bash
cargo run --release --bin sage_vision
```

Real-time camera perception loop. Captures at ~10 FPS, extracts visual features (brightness, color, edges), generates concepts, stores to visual memory, converts frames to NCA grids.

### Key Examples

| Example | Description |
|---------|-------------|
| `sage_discord_autonomous` | **Primary production bot.** Discord bot with Ollama LLM, NCA personality, inner world, semantic memory, human-in-loop approvals. Commands: `/state`, `/evolve`, `/ask`, `/save`, `/load`, `/snapshots`, `/role`, `/expertise`, `/pending`, `/approve`, `/reject` |
| `sage_city_server` | Web UI for Miniworld simulation (port 8743) |
| `sage_web_dashboard` | NCA real-time visualization dashboard (port 3030) |
| `sage_job_dashboard` | Job instance monitoring dashboard |
| `sage_job_instance` | CLI to spawn/list/stop role-specialized instances |
| `sage_control_cli` | Instance registry CLI management |
| `sage_headless` | Run SAGE without TUI |
| `sage_background_learner` | Background NCA training |
| `db_explorer` | SpacetimeDB data browser |

---

## Deployment Architecture

### Docker Compose Services

```
┌─────────────────────────────────────────────────────┐
│                   sage-network                       │
│                                                      │
│  ┌──────────────┐  ┌──────────────┐                 │
│  │ spacetimedb  │  │   ollama     │                 │
│  │  :3001→3000  │  │ :11434→11434 │                 │
│  │  (database)  │  │  (LLM/GPU)   │                 │
│  └──────┬───────┘  └──────┬───────┘                 │
│         │                  │                         │
│  ┌──────┴──────────────────┴───────┐                │
│  │         sage-discord            │                 │
│  │  (sage_discord_autonomous)      │                 │
│  │  Instances: Lumin, Nova, Echo   │                 │
│  │  + job roles via profiles       │                 │
│  └─────────────────────────────────┘                │
└─────────────────────────────────────────────────────┘
```

**Core services (always started):**

| Service | Image | Port | Purpose |
|---------|-------|------|---------|
| `spacetimedb` | `clockworklabs/spacetime:latest` | `3001→3000` | SpacetimeDB server (WASM module host) |
| `spacetimedb-init` | Custom (`Dockerfile.spacetimedb-init`) | — | One-shot: publishes `sage-db` WASM module |
| `ollama` | `ollama/ollama:latest` | `11434→11434` | LLM inference (NVIDIA GPU). Hosts custom `sage` model + `nomic-embed-text` |
| `ollama-init` | `ollama/ollama:latest` | — | One-shot: pulls models, creates custom `sage` model from `Modelfile.sage` |
| `sage-discord` (Lumin) | Custom (`Dockerfile`) | — | Primary Discord bot |

**Profile-gated instances:**

| Profile | Service | Name | Role |
|---------|---------|------|------|
| `nova` | `sage-nova` | Nova | General SAGE |
| `echo` | `sage-echo` | Echo | General SAGE |
| `content` | `sage-content` | Content-Maya | Content Creator |
| `analyst` | `sage-analyst` | Data-Alex | Data Analyst |
| `support` | `sage-support` | Support | Customer Support |
| `marketer` | `sage-marketer` | Marketer | Ad Marketer |
| `all-sages` | All above | — | Everything |
| `job-instances` | All role instances | — | All job roles |

Each instance gets isolated persistent data in `data/<name>/` (inner world, memory, NCA grid, etc.).

### Databases

- **SpacetimeDB** — WASM-based database. Module source in `sage-db/`. Stores expertise records, action drafts, task history, milestones. Queried via `SageDbClient` (REST/SDK).
- **JSON files** — Per-instance persistence: inner world, semantic memory, associations, curiosity, preferences, NCA grid.
- **In-memory** — Vector memory (embeddings), conversation context, sentiment history.

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `DISCORD_TOKEN` | — | Bot token for Lumin |
| `DISCORD_TOKEN_NOVA` | — | Bot token for Nova |
| `DISCORD_TOKEN_ECHO` | — | Bot token for Echo |
| `OLLAMA_HOST` | `http://ollama:11434` | Ollama API endpoint |
| `SPACETIMEDB_URI` | `http://spacetimedb:3000` | SpacetimeDB endpoint |
| `SAGE_NAME` | — | Instance display name |
| `SAGE_INSTANCE_ID` | — | Unique instance identifier |
| `SAGE_ROLE` | — | Role specialization |
| `PORT` | `8888` | Miniworld server port |
| `RUST_LOG` | `info` | Log level |

---

## Running Locally

### Prerequisites

```bash
# Rust toolchain
rustup update stable
rustup target add wasm32-unknown-unknown  # For sage-db module

# System deps (Ubuntu/Debian)
sudo apt install libssl-dev libasound2-dev libclang-dev pkg-config

# Ollama
curl -fsSL https://ollama.com/install.sh | sh
ollama pull fluffy/l3-8b-stheno-v3.2:latest
ollama pull nomic-embed-text
ollama create sage -f Modelfile.sage
```

### Run the TUI

```bash
cargo run --release
```

### Run Discord bot

```bash
DISCORD_TOKEN=<token> cargo run --release --example sage_discord_autonomous
# With role specialization:
DISCORD_TOKEN=<token> cargo run --release --example sage_discord_autonomous -- --role content_creator
```

### Run Miniworld

```bash
cargo run --release --bin miniworld_server
# Open http://localhost:8888
```

### Full Docker stack

```bash
docker compose up -d                              # Core + Lumin
docker compose --profile all-sages up -d          # All instances
docker compose --profile job-instances up -d       # All role instances
```
