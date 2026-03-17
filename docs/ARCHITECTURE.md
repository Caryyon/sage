# SAGE Architecture Overview

**SAGE** — Shared Adaptive Growing Experience: A decentralized intelligence network written in Rust.

## High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        Entry Points                              │
│  sage (TUI)  │  sage_discord_autonomous  │  miniworld_server    │
│              │  sage_vision              │  sage_city_server     │
└──────┬───────┴──────────┬───────────────┴──────────┬────────────┘
       │                  │                           │
┌──────▼──────────────────▼───────────────────────────▼────────────┐
│                    Core Cognitive Systems                          │
│  ┌────────────┐  ┌──────────────┐  ┌─────────────────────────┐  │
│  │  NCA Grid  │  │  LLM Client  │  │  Personality Engine     │  │
│  │  (neural   │  │  (Ollama)    │  │  (humanization,         │  │
│  │  substrate)│  │              │  │   context, modulation)  │  │
│  └─────┬──────┘  └──────┬───────┘  └────────────┬────────────┘  │
│        │                │                        │               │
│  ┌─────▼────────────────▼────────────────────────▼───────────┐  │
│  │              Cognitive State (unified)                      │  │
│  │  NCA state + Inner World + Sentiment → personality params  │  │
│  └────────────────────────────┬───────────────────────────────┘  │
│                               │                                  │
│  ┌────────────────────────────▼───────────────────────────────┐  │
│  │                    Memory Systems                           │  │
│  │  episodic │ semantic (vectors) │ fact │ visual │ temporal  │  │
│  │  conversation_context │ concept_associations │ embeddings  │  │
│  └────────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────────┘
       │                                          │
┌──────▼──────────────────────────────────────────▼────────────────┐
│                    Interfaces & Simulations                       │
│  Discord (serenity) │ IRC (stub) │ Inner World │ Miniworld/City │
│  Web Dashboard      │ TUI        │ Vision      │ Tool System    │
└──────────────────────────────────────────────────────────────────┘
       │
┌──────▼──────────────────────────────────────────────────────────┐
│                    Persistence & Storage                          │
│  SpacetimeDB (expertise, tasks, approvals)                       │
│  JSON files (inner world, NCA grid, memory, preferences)         │
│  In-memory vectors (embeddings, conversation context)            │
└──────────────────────────────────────────────────────────────────┘
```

## Module Reference

### Neural Substrate

| Module | Purpose |
|--------|---------|
| `nca` | Neural Cellular Automata — the core neural grid. 2D grid of cells with continuous state values that evolve via local rules. SAGE's "brain substrate." |
| `nca_state` | Extract cognitive state from NCA grid (energy, complexity, stability metrics) |
| `nca_memory` | NCA memory channels (channels 22-25) for persistent neural patterns |
| `grid` | Low-level 2D grid data structure |
| `pattern_semantics` | Map NCA patterns to semantic meanings |

### Knowledge Retrieval: Attention vs. Cosine

SAGE supports two knowledge retrieval strategies, selected automatically based on query type:

**Cosine Similarity (Hash-Based Queries)**
- Fast O(active_cells) scan of grid neighborhood
- Uses hash-based feature encoding when Ollama embeddings unavailable
- Combines 70% cosine similarity + 30% spatial proximity
- Good for exact keyword matching

**Cross-Attention (Semantic Queries)**
- Based on arXiv:2603.10055 (Lee et al., MIT CSAIL): attention layers are the most transferable mechanism from NCA to LLM
- Query = task context embedding; Keys/Values = NCA cell embedding slots
- Scaled dot-product attention: `softmax(QK^T / sqrt(d_k)) V`
- Spatial gating (thalamic routing): routes attention to most relevant grid quadrant first
- Complexity: O(active_cells * d_k) where d_k = 6 embedding slots
- Used when Ollama embeddings are available (semantic understanding)

**Freerun Repair**
- Based on rNCA (Silbernagel et al., 2025): self-repair dynamics prevent semantic drift
- After encoding new knowledge, run unconditioned NCA steps (no new input)
- Smooths hidden channels (4..16) via local neighbor averaging
- Knowledge channels (26+) are preserved unchanged
- Consolidates activation patterns before the next read

### Language & Learning

| Module | Purpose |
|--------|---------|
| `language` | Grounded reservoir language model (corpus, encoder, predictor, training) |
| `language_learning` | Language acquisition pipeline |
| `hybrid_language` | Hybrid NCA + small language model system |
| `local_llm` | Local LLM inference via Candle (TinyLlama) |
| `llm_client` | Ollama API client for conversational AI |
| `text_encoder` | Text → NCA grid encoding |
| `word_pattern_mapper` | Bidirectional word ↔ NCA pattern grounding |
| `learning/` | Scalable learning subsystem: meta-learning (MAML, Reptile), neural networks, novelty search, population-based training, self-supervised learning, architecture modification |
| `embeddings` | Semantic memory via Ollama `nomic-embed-text` embeddings |

### Cognitive Architecture

| Module | Purpose |
|--------|---------|
| `sage_experience` | Main interface — orchestrates all cognitive systems |
| `cognitive_state` | Unified cognitive state merging NCA + inner world |
| `consciousness` | Core consciousness loop |
| `attention` | Selective focus and salience-based attention |
| `introspection` | Phenomenological self-awareness |
| `theory_of_mind` | Model other agents' mental states |
| `inner_thoughts` | Autonomous thought evaluation and classification |
| `curiosity` | Proactive curiosity and question generation |
| `self_modification` | Performance introspection and self-optimization |
| `emergent_goals` | Autonomous goal formation |
| `goal_hierarchy` | Multi-step goal decomposition and planning |

### Memory Systems

| Module | Purpose |
|--------|---------|
| `episodic_memory` | Narrative conversation memory |
| `temporal_memory` | Short-term → long-term memory consolidation |
| `vector_memory` | In-memory vector store for semantic RAG |
| `fact_memory` | Structured fact extraction and storage |
| `visual_memory` | Cross-modal learning, dream-vision integration |
| `memory_manager` | Unified interface coordinating all memory systems |
| `conversation_context` | Per-user conversation history |
| `concept_associations` | Creative connections between concepts |
| `sentiment_history` | Sentiment trend tracking for NCA modulation |

### Personality & Emotion

| Module | Purpose |
|--------|---------|
| `personality/` | Centralized personality engine: humanization, context building, response modulation |
| `emotional_gradients` | PAD model (Pleasure-Arousal-Dominance) emotional modeling |
| `preferences` | Opinion formation and personality traits |
| `personality_evolution` | Personality drift tracking over time |
| `persona_templates` | Archetype-based persona templates |
| `persona_generator` | Persona generation (templates + Claude API) |

### Inner World Simulation

| Module | Purpose |
|--------|---------|
| `inner_world/` | Rich inner world simulation — SAGE's subjective experience |
| `inner_world/simulation` | Core simulation loop |
| `inner_world/city` | City behaviors and social dynamics |
| `inner_world/rooms` | House rooms (bedroom, kitchen, study, garden) |
| `inner_world/dreams` | Dream generation during sleep cycles |
| `inner_world/garden` | Virtual garden tending |
| `inner_world/cooking` | Cooking simulation |
| `inner_world/library` | Reading and book collection |
| `inner_world/music` | Music listening and creation |
| `inner_world/pet` | Virtual pet companion |
| `inner_world/journal` | Journal writing |
| `inner_world/research` | Research activities |
| `inner_world/visitors` | NPC visitor events |
| `inner_world/holidays` | Holiday/seasonal events |
| `inner_world/real_weather` | Real weather integration |
| `inner_world/sun` / `moon` | Day/night cycle |
| `inner_world/outreach` | Proactive outreach behaviors |
| `inner_world/f2f_conversation` | Face-to-face conversation simulation |

### Miniworld (SAGE City)

| Module | Purpose |
|--------|---------|
| `miniworld/` | 2D pixel-art tile-based town simulation |
| `miniworld/world` | World state: tiles grid, characters, buildings, time, tick loop |
| `miniworld/tiles` | Tile types: ground (grass, path, water, stone), overlays (trees, rocks, buildings), team colors |
| `miniworld/character` | Character state machine: Idle, Walking, Working, Talking, Sleeping, Eating, Shopping. Pathfinding, wandering. |
| `miniworld/renderer` | PNG rendering with painter's algorithm (ground → overlay → character layers) |
| `miniworld/town` | Default town layout generator, building placement, SAGE character spawning |

Rendering uses a 3-layer painter's algorithm: ground tiles → overlay tiles → character sprites. The `miniworld_server` binary and `sage_city_server` example serve this via WebSocket at 10 ticks/sec.

### Job Instance System

| Module | Purpose |
|--------|---------|
| `agentic/` | Agentic task loop: observe → plan → act → reflect |
| `hil/` | Human-in-the-loop approval system for external actions |
| `roles/` | Role templates: Content Creator, Ad Marketer, Data Analyst, Customer Support |
| `expertise/` | Skill mastery tracking, milestones, skill decay |
| `orchestration/` | Instance orchestration and task execution |
| `external_apis/` | External integrations: Facebook, Analytics, Helpdesk |
| `sage_control` | Instance registry, heartbeats, process management |

### Interface Modules

| Module | Purpose |
|--------|---------|
| `tui/` | Ratatui TUI: NCA visualization, training dashboard, braille canvas, hot reload |
| `web_dashboard/` | Axum web dashboard: REST API + WebSocket for instance monitoring |
| `dashboard/` | Discord bot dashboard TUI with reducer-based state |
| `cli` | CLI argument parsing (clap) |
| `irc` / `irc_manager` / `irc_sync` | IRC support (currently stubbed/disabled) |
| `proactive_communication` | AGI-level proactive conversation initiation |
| `response_pipeline` | Multi-stage LLM pipeline for grounded responses |
| `tool_system` | Real-world interaction tools |

### Perception

| Module | Purpose |
|--------|---------|
| `vision` | Camera capture, feature extraction (brightness, color, edges) |
| `visual_training` | Visual curriculum for sensorimotor integration |
| `audio_input` | Microphone → grid conversion (inverse sonification) |
| `sonification` | Neural patterns → audio |

### Misc

| Module | Purpose |
|--------|---------|
| `agi` | AGI-level reasoning primitives |
| `civilization` | Multi-agent civilization simulation |
| `arc_tasks` | ARC-style grid transformation reasoning |
| `self_play` | Multi-NCA self-play arena |
| `dsl` | Domain-specific language for grid transformations (program synthesis) |
| `fractal_analysis` | Fractal dimension for pattern complexity |
| `ab_test` | A/B testing framework for NCA validation |
| `persistence` | General serialization/deserialization |
| `sage_snapshot` | Complete state snapshot save/load |
| `spacetime_client` | SpacetimeDB SDK client wrapper |
| `terrain` | Terrain generation for NCA environments |
| `display` | Terminal display utilities |
| `message_queue` | Internal message passing |
| `communication` | SAGE inter-instance communication |

## Data Flow: Discord Message → Response

```
User message on Discord
  → serenity event handler (sage_discord_autonomous)
  → ConversationContextManager (load user history)
  → SageExperience (consciousness update)
  → CognitiveState (NCA metrics + inner world state)
  → PersonalityEngine (humanization + context building)
  → ResponsePipeline (multi-stage LLM)
    → LlmClient → Ollama API (sage model)
    → Response modulation (inner world events, mood, personality)
  → EpisodicMemory (store interaction)
  → SemanticMemory (embed & index)
  → Discord reply
```

## Workspace Layout

```
sage/
├── Cargo.toml              # Workspace root
├── src/                    # Main crate (sage)
│   ├── main.rs             # TUI entry point
│   ├── lib.rs              # All module declarations
│   ├── bin/                # Additional binaries
│   │   ├── miniworld_server.rs
│   │   └── sage_vision.rs
│   ├── web_dashboard/      # REST API + WebSocket dashboard
│   ├── miniworld/          # 2D tile world simulation
│   ├── inner_world/        # Rich inner experience simulation
│   ├── learning/           # Meta-learning subsystem
│   ├── language/           # Grounded language model
│   ├── personality/        # Personality engine
│   ├── agentic/            # Observe-plan-act-reflect loop
│   ├── tui/                # Terminal UI
│   └── ...                 # ~80+ other modules
├── sage-db/                # SpacetimeDB WASM module
├── sage-training/          # Training utilities crate
├── examples/               # ~50+ examples and production binaries
├── static/                 # Web assets (dashboard, miniworld)
├── books/                  # Book corpus for reading/learning
├── data/                   # Per-instance persistent data
├── scripts/                # Deployment scripts
├── Dockerfile              # Multi-stage build for Discord bot
├── Dockerfile.spacetimedb-init  # DB module publisher
├── docker-compose.yml      # Full stack deployment
├── docker-compose.multi.yml    # Role-specialized instances
├── docker-compose.native-ollama.yml  # Host Ollama variant
└── Modelfile.sage          # Custom Ollama model definition
```

## Build Notes

**Current build issue:** Missing `libssl-dev` (OpenSSL development headers). Fix:

```bash
sudo apt install libssl-dev pkg-config
```

Other build dependencies: `libasound2-dev` (audio), `libclang-dev` (bindgen).
