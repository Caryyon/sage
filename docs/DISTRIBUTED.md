# SAGE Distributed Intelligence

## The Living Network That Gets Smarter With Every Node

> *"What if AI wasn't a product you paid for, but a commons that grew with every person who joined?"*

---

## Table of Contents

1. [Vision & Mission](#1-vision--mission)
2. [Core Architecture](#2-core-architecture)
3. [Distribution Protocol](#3-distribution-protocol)
4. [Comparison to Existing Systems](#4-comparison-to-existing-systems)
5. [Phase Roadmap](#5-phase-roadmap)
6. [Technical Deep Dive](#6-technical-deep-dive)
7. [Research Novelty](#7-research-novelty)

---

## 1. Vision & Mission

### The Problem

AI today is a feudal system. A handful of corporations control the models, the data, and the access. You rent intelligence by the token. Your conversations train their models, but you never see the benefit. When the API goes down, your AI goes dark. When the price goes up, you pay or lose access.

Meanwhile, billions of CPU cycles sit idle on laptops, desktops, and servers around the world.

### The SAGE Vision

**SAGE Distributed Intelligence** is a decentralized AI network where:

- **Anyone can run a node** — download, run, chat. No account, no API key, no subscription.
- **Every conversation makes the network smarter** — knowledge learned on one node propagates to all nodes via gossip protocol.
- **More nodes = better AI for everyone** — the network's collective intelligence scales with participation, not with corporate GPU budgets.
- **CPU-first** — Neural Cellular Automata are tiny operations (additions, multiplications on small grids). No GPU required. Your laptop is enough.
- **Alive, not frozen** — unlike traditional models with static weights, SAGE nodes continuously learn and adapt. The NCA grid is a living substrate that evolves with use.

### Mission Statement

Build the world's first truly decentralized, community-trained AI system. Free as in freedom, free as in beer. An intelligence commons that belongs to everyone and gets smarter the more people use it.

### Design Principles

1. **Zero barriers** — `curl -sSL sage.run | sh` and you're part of the network
2. **Privacy by default** — your conversations stay local; only distilled knowledge (abstract NCA state diffs) gets shared
3. **No central authority** — no master server, no single point of failure, no kill switch
4. **CPU-friendly** — runs on a Raspberry Pi, screams on a desktop
5. **Incrementally useful** — a single node is a capable local AI; the network just makes it better

---

## 2. Core Architecture

### 2.1 Overview

```
┌──────────────────────────────────────────────────────────────────┐
│                         SAGE Node                                 │
│                                                                   │
│  ┌─────────────┐    ┌──────────────────┐    ┌────────────────┐  │
│  │  User Input  │───▶│  Text Encoder    │───▶│   NCA Grid     │  │
│  │  (chat/API)  │    │  (text → grid)   │    │  (32 channels) │  │
│  └─────────────┘    └──────────────────┘    │  (32×32 cells) │  │
│                                              └───────┬────────┘  │
│                                                      │           │
│                              ┌────────────────────────┘           │
│                              ▼                                    │
│  ┌─────────────┐    ┌──────────────────┐    ┌────────────────┐  │
│  │  Response    │◀───│  Local LM Head   │◀───│  Knowledge     │  │
│  │  (text out)  │    │  (~100M params)  │    │  Extractor     │  │
│  └─────────────┘    └──────────────────┘    │  (grid → ctx)  │  │
│                                              └────────────────┘  │
│         │                                           │            │
│         │              ┌────────────────┐           │            │
│         │              │  Gossip Layer  │◀──────────┘            │
│         │              │  (libp2p)      │                        │
│         │              └───────┬────────┘                        │
│         │                      │                                 │
└─────────┼──────────────────────┼─────────────────────────────────┘
          │                      │
          ▼                      ▼
     User sees              Other SAGE
     response               nodes on
                             the network
```

### 2.2 NCA Grid as Knowledge Store

The NCA grid is SAGE's "brain" — a 2D grid of cells where each cell holds a multi-channel state vector. Currently SAGE uses 26 channels on a 32×32 grid. For distributed intelligence, we expand to **32+ channels** with dedicated roles:

| Channel Range | Name | Purpose |
|--------------|------|---------|
| 0–3 | **Visual/RGBA** | Pattern visualization, alive masking |
| 4–7 | **Structural** | Gram structure, syntax patterns |
| 8–11 | **Semantic Core** | Topic embeddings, concept representations |
| 12–15 | **Association** | Cross-concept links, analogy patterns |
| 16–19 | **Environmental** | Context signals, conversation state |
| 20–21 | **Confidence** | Per-cell certainty scores, source reliability |
| 22–25 | **Memory** | Attention, gate, value, recency (existing) |
| 26–27 | **Temporal** | Knowledge age, decay rate |
| 28–29 | **Provenance** | Origin node fingerprint, sync generation |
| 30–31 | **Distributed** | Merge conflict markers, consensus weight |

**Why 32×32×32?** That's 32,768 floats — **128 KB**. An entire knowledge state that fits in L1 cache. Compare to GPT-3's 175 billion parameters (700 GB). The NCA grid isn't storing raw text; it's storing *compressed conceptual representations* that emerge from the cellular automata dynamics.

### 2.3 Knowledge Encoding: Text → Grid

When a user has a conversation with SAGE, the knowledge from that conversation gets encoded into the NCA grid through a multi-stage pipeline:

```
Input Text
    │
    ▼
┌─────────────────────────────┐
│  Tokenizer + Embedding      │  Learned embeddings (~32K vocab)
│  "Rust is memory-safe"      │  → [0.23, -0.11, 0.87, ...]
└─────────────┬───────────────┘
              │
              ▼
┌─────────────────────────────┐
│  Spatial Encoder             │  Project embedding → 2D grid patch
│  (learned linear + reshape)  │  Semantically similar concepts
│                              │  → nearby grid regions
└─────────────┬───────────────┘
              │
              ▼
┌─────────────────────────────┐
│  NCA Integration Steps       │  Run N update steps (N=8..16)
│  Grid absorbs new knowledge  │  New info merges with existing
│  through local update rules  │  state via learned kernels
└─────────────────────────────┘
```

**Key insight:** The NCA doesn't memorize text. It develops *patterns* that represent conceptual knowledge. "Rust is memory-safe" doesn't get stored as tokens — it creates activation patterns in the semantic channels (8–11) that represent the *concept* of memory safety associated with the *concept* of Rust. These patterns interact with existing knowledge through the automata dynamics, forming associative links automatically.

#### Spatial Encoding Strategy

Concepts are mapped to grid regions via **learned spatial hashing**:

```rust
fn encode_to_grid(embedding: &[f32; D], grid: &mut NCAGrid) {
    // Compute 2D position from semantic embedding
    let (cx, cy) = spatial_hash(embedding);  // Learned projection

    // Create activation patch (Gaussian-weighted)
    for dy in -R..=R {
        for dx in -R..=R {
            let weight = gaussian(dx, dy, sigma=2.0);
            let (gx, gy) = wrap(cx + dx, cy + dy);  // Toroidal wrap

            // Write to semantic channels with gating
            let gate = grid[gy][gx][MEMORY_GATE];
            for ch in SEMANTIC_START..SEMANTIC_END {
                grid[gy][gx][ch] += weight * embedding[ch - SEMANTIC_START] * gate;
            }

            // Update provenance
            grid[gy][gx][PROVENANCE_NODE] = self.node_id_hash;
            grid[gy][gx][TEMPORAL_AGE] = 0.0;  // Fresh knowledge
        }
    }

    // Run integration steps — let the NCA settle
    for _ in 0..INTEGRATION_STEPS {
        grid.step();  // NCA update rule
    }
}
```

### 2.4 Knowledge Extraction: Grid → Text

To generate a response, SAGE extracts relevant knowledge from the NCA grid and feeds it as context to a small local transformer:

```
User Query: "Is Rust memory safe?"
        │
        ▼
┌──────────────────────────────┐
│  Query Encoder               │
│  Same spatial hash as write  │
│  → identifies relevant       │
│    grid region               │
└─────────────┬────────────────┘
              │
              ▼
┌──────────────────────────────┐
│  Attention Readout           │
│  Multi-head attention over   │
│  grid cells, weighted by     │
│  relevance to query          │
│  → knowledge vector (1024d)  │
└─────────────┬────────────────┘
              │
              ▼
┌──────────────────────────────┐
│  Context Formatter           │
│  knowledge vector → natural  │
│  language context prefix     │
│  "Known: Rust provides       │
│   memory safety through      │
│   ownership and borrowing"   │
└─────────────┬────────────────┘
              │
              ▼
┌──────────────────────────────┐
│  Local Transformer (~100M)   │
│  Generates fluent response   │
│  conditioned on NCA context  │
│  + conversation history      │
└──────────────────────────────┘
```

**Why a hybrid approach?** NCA grids are excellent at storing, compressing, and associating knowledge — but they're not language models. A small transformer (TinyLlama-scale, ~100M parameters) handles the actual text generation, using NCA-extracted knowledge as its context window. This gives us:

- **NCA**: Long-term, ever-growing knowledge store (grows with the network)
- **Transformer**: Fluent text generation (static, small, CPU-friendly)

The transformer doesn't need to *know* everything — it just needs to speak well. The NCA provides the knowledge; the transformer provides the voice.

### 2.5 The Hybrid Pipeline in Detail

```
┌────────────────────────────────────────────────────────────┐
│                    Response Generation                       │
│                                                              │
│  1. Encode query → grid coordinates                         │
│  2. Read grid neighborhood (semantic + association channels)│
│  3. Run attention over read cells → knowledge embedding     │
│  4. Concatenate: [knowledge_emb | query_tokens]             │
│  5. Feed to transformer decoder                              │
│  6. Generate tokens autoregressively                         │
│  7. Post-process: confidence gating (skip if NCA uncertain) │
│  8. If NCA confidence < threshold → fall back to base LM    │
│                                                              │
│  Latency budget (on CPU):                                    │
│  Steps 1-3: ~2ms (grid lookup + small attention)            │
│  Steps 4-6: ~200ms/token (100M transformer)                 │
│  Total first token: ~50ms                                    │
│  Throughput: ~5 tokens/sec on 4-core laptop                 │
└────────────────────────────────────────────────────────────┘
```

---

## 3. Distribution Protocol

### 3.1 Network Topology

SAGE uses a **peer-to-peer gossip network** built on [libp2p](https://libp2p.io/) — the same networking stack that powers IPFS, Filecoin, and Ethereum 2.0. No central servers. No master nodes.

```
        Node A (Portland)
       / gossip \
      /          \
Node B ─────── Node C (Berlin)
(Tokyo)  gossip   │
      \          │ gossip
       \        │
        Node D (São Paulo)
         │
         │ gossip
         │
        Node E (Lagos)
```

#### Discovery & Connectivity

1. **Bootstrap nodes** — A small set of well-known, community-operated nodes for initial peer discovery. Not authorities — just address books.
2. **mDNS** — Automatic local network discovery (find SAGE nodes on your LAN without internet)
3. **DHT (Kademlia)** — Distributed hash table for peer discovery beyond bootstraps
4. **Relay circuit** — NAT traversal for nodes behind firewalls (libp2p relay v2)

```rust
// Node startup
let transport = libp2p::tcp::tokio::Transport::new(tcp::Config::default())
    .upgrade(upgrade::Version::V1)
    .authenticate(noise::Config::new(&keypair)?)
    .multiplex(yamux::Config::default());

let behaviour = SageBehaviour {
    gossipsub: Gossipsub::new(peer_id, gossipsub_config),
    kademlia: Kademlia::new(peer_id, MemoryStore::new(peer_id)),
    mdns: Mdns::new(mdns::Config::default())?,
};
```

### 3.2 Knowledge Sync via Gossip

Nodes don't sync raw conversations. They sync **NCA state diffs** — compressed representations of what the grid *learned*, not what was said.

#### What Gets Shared

```
NOT shared:                    Shared:
─────────────                  ──────
"Hey SAGE, my password         Grid diff: semantic channels
 is hunter2"                   for region around "security"
                               concept updated +0.03
"I'm feeling depressed         Grid diff: association channels
 about my divorce"             linking "coping" ↔ "strategies"
                               strengthened
```

**Privacy guarantee:** NCA state diffs are lossy compressions of conceptual knowledge. You cannot reconstruct the original conversation from a grid diff any more than you can reconstruct a photograph from a description of its mood.

#### Diff Format

```rust
/// A knowledge diff — the atomic unit of distribution
#[derive(Serialize, Deserialize)]
struct GridDiff {
    /// Unique diff identifier
    id: DiffId,                          // [u8; 32] — blake3 hash

    /// Source node (pseudonymous)
    source: PeerId,

    /// Which grid generation this applies to
    base_generation: u64,

    /// Sparse cell updates (only changed cells)
    updates: Vec<CellUpdate>,

    /// Confidence score (source node's self-assessment)
    confidence: f32,

    /// Signature (proves source node created this)
    signature: Ed25519Signature,

    /// Timestamp
    created_at: u64,
}

#[derive(Serialize, Deserialize)]
struct CellUpdate {
    x: u8,
    y: u8,
    /// Only the channels that changed, delta-encoded
    channel_deltas: SmallVec<[(u8, f32); 8]>,
}
```

Typical diff size: **200–2000 bytes** for a conversation's worth of learning. A node on a 1 Mbps connection can process ~500 diffs/second.

### 3.3 Merkle DAG for Knowledge Versioning

Every node maintains a **Merkle DAG** (directed acyclic graph) of its knowledge history:

```
    ┌──────────┐
    │ Gen 0    │  ← Initial grid state (shipped with binary)
    │ hash: a1 │
    └────┬─────┘
         │
    ┌────▼─────┐
    │ Gen 1    │  ← After local conversations
    │ hash: b3 │
    │ parent:a1│
    └────┬─────┘
         │
    ┌────▼─────┐     ┌──────────┐
    │ Gen 2    │◀────│ Merge    │  ← Incorporated diffs from peer
    │ hash: c7 │     │ from     │
    │ parent:b3│     │ Node B   │
    └────┬─────┘     └──────────┘
         │
        ...
```

**Why Merkle DAG?**

- **Integrity** — Any tampering with historical knowledge is detectable via hash chain
- **Efficient sync** — Nodes compare DAG heads to find divergence points, then only send missing diffs
- **Fork resolution** — When two nodes have conflicting knowledge, the DAG makes the conflict explicit and resolvable
- **No blockchain overhead** — No proof-of-work, no consensus mechanism, no mining. Just content-addressed data

### 3.4 Gossip Protocol Details

SAGE uses **GossipSub** (libp2p's topic-based pub/sub) with custom topic structure:

| Topic | Purpose | Frequency |
|-------|---------|-----------|
| `/sage/knowledge/v1` | Grid diffs | Event-driven (after learning) |
| `/sage/heartbeat/v1` | Node liveness, generation heads | Every 30s |
| `/sage/reputation/v1` | Peer quality scores | Every 5min |
| `/sage/bootstrap/v1` | New node onboarding | On join |

#### Sync Protocol

```
New Node Joins:
1. Connect to bootstrap peers
2. Announce on /sage/bootstrap/v1
3. Receive DAG heads from neighbors
4. Request missing diff chain back to common ancestor
5. Apply diffs in causal order
6. Begin participating in gossip

Ongoing Sync:
1. After local learning event → create GridDiff
2. Validate diff (confidence > threshold, not too large)
3. Publish to /sage/knowledge/v1
4. Receiving nodes validate, apply if passes checks
5. Gossip naturally propagates to full network in O(log N) rounds
```

### 3.5 Anti-Poisoning & Reputation

A decentralized learning network is a target for adversarial actors. SAGE employs defense in depth:

#### Layer 1: Cryptographic Identity

Every node has an Ed25519 keypair. All diffs are signed. You can't impersonate another node or forge diffs.

#### Layer 2: Diff Validation

Before applying any received diff, a node checks:

```rust
fn validate_diff(diff: &GridDiff, current_grid: &NCAGrid) -> ValidationResult {
    // 1. Signature verification
    if !verify_signature(&diff.source, &diff.signature, &diff.payload()) {
        return Reject("Invalid signature");
    }

    // 2. Magnitude bounds — no single diff should wildly change the grid
    for update in &diff.updates {
        for (_, delta) in &update.channel_deltas {
            if delta.abs() > MAX_DELTA {
                return Reject("Delta too large");
            }
        }
    }

    // 3. Coverage bounds — no diff should touch too many cells
    if diff.updates.len() > MAX_CELLS_PER_DIFF {
        return Reject("Too many cell updates");
    }

    // 4. Coherence check — apply tentatively, verify grid stays stable
    let mut test_grid = current_grid.clone();
    apply_diff(&mut test_grid, diff);
    for _ in 0..STABILITY_STEPS {
        test_grid.step();
    }
    if !is_stable(&test_grid) {
        return Reject("Diff destabilizes grid");
    }

    Accept
}
```

#### Layer 3: Reputation System

Nodes build reputation over time based on the quality of their contributions:

```rust
struct PeerReputation {
    peer_id: PeerId,
    /// Number of diffs accepted by this node
    accepted_diffs: u64,
    /// Number of diffs rejected
    rejected_diffs: u64,
    /// Average quality score of accepted diffs (peer-assessed)
    quality_score: f32,
    /// How long this peer has been active
    uptime_days: u32,
    /// Reputation score: [0.0, 1.0]
    score: f32,
}

impl PeerReputation {
    fn update_score(&mut self) {
        let acceptance_rate = self.accepted_diffs as f32
            / (self.accepted_diffs + self.rejected_diffs).max(1) as f32;
        let longevity_bonus = (self.uptime_days as f32 / 365.0).min(1.0) * 0.1;
        self.score = (acceptance_rate * 0.7 + self.quality_score * 0.2 + longevity_bonus)
            .clamp(0.0, 1.0);
    }
}
```

**Reputation affects diff acceptance:**
- New nodes (score < 0.3): Diffs are quarantined, applied only after corroboration by 2+ reputable nodes
- Established nodes (0.3–0.7): Standard validation pipeline
- Trusted nodes (> 0.7): Fast-path acceptance with lighter validation

#### Layer 4: Knowledge Consensus

For contentious knowledge (where nodes disagree), SAGE uses **soft consensus**:

- Each cell in the grid has a **confidence channel** (20–21)
- When conflicting diffs arrive, the update with higher aggregate confidence wins
- Confidence decays over time — stale uncorroborated knowledge fades
- Widely corroborated knowledge (many independent nodes agree) gets boosted

This is explicitly **not** blockchain consensus. There's no finality, no voting. It's more like how scientific knowledge works: widely reproduced results become accepted; outlier claims fade unless confirmed.

---

## 4. Comparison to Existing Systems

### 4.1 SAGE vs. Ollama

| Dimension | Ollama | SAGE |
|-----------|--------|------|
| **Model** | Static pre-trained LLMs (7B–70B) | Living NCA grid + small LM head |
| **Learning** | None — frozen weights | Continuous — learns from every conversation |
| **Hardware** | GPU strongly recommended for 7B+ | CPU-first — NCA is add/multiply on 32K floats |
| **Network** | None — isolated instances | Gossip network — every node benefits from all nodes |
| **Size** | 4–40 GB per model | ~50 MB total (NCA grid + small transformer) |
| **Knowledge** | Locked at training cutoff | Continuously updated via network |

**Ollama gives you a frozen brain in a jar. SAGE gives you a living brain that's connected to a global nervous system.**

### 4.2 SAGE vs. ChatGPT / Claude / Commercial APIs

| Dimension | ChatGPT/Claude | SAGE |
|-----------|---------------|------|
| **Cost** | $20/month, $0.01+/request | Free forever |
| **Privacy** | Your data on their servers | Everything local, only abstract diffs shared |
| **Availability** | Depends on their servers | Works offline, network enhances |
| **Control** | They choose what you get | You run the code, you control the node |
| **Censorship** | Corporate content policies | Community-governed knowledge |
| **Learning** | You train their model for free | You train YOUR model, everyone benefits |

### 4.3 SAGE vs. Federated Learning

| Dimension | Federated Learning | SAGE |
|-----------|-------------------|------|
| **Architecture** | Central parameter server | No central anything |
| **Communication** | Gradient uploads (large) | Grid diffs (tiny, ~1 KB) |
| **Model** | Same big model on all nodes | NCA grid (unique per node, converges) |
| **Privacy** | Gradients can leak data | Grid diffs are lossy — no reconstruction |
| **Coordination** | Synchronized rounds | Asynchronous gossip |
| **Failure mode** | Server dies = everything stops | Any node can die, network continues |

### 4.4 The CPU Advantage

Modern AI is trapped in a GPU monoculture. Training and running LLMs requires expensive, power-hungry GPUs. This creates artificial scarcity — only well-funded companies can play.

NCA computation is fundamentally different:

```
LLM inference (7B model):
  - Matrix multiply: [4096 × 4096] × [4096 × 1]
  - Per token: ~14 billion FLOPs
  - Memory: 14 GB (fp16)
  - Hardware: GPU required

NCA update step (32×32×32 grid):
  - Perception: 3×3 convolution × 32 channels = 9,216 multiply-adds
  - Update: small MLP (96 → 384 → 32) = ~73,728 multiply-adds
  - Per cell: ~83K FLOPs × 1024 cells = ~85M FLOPs total
  - Memory: 128 KB
  - Hardware: Any CPU made after 2010
```

**NCA is 165,000× cheaper per step than a single LLM token.** Even accounting for multiple NCA steps per query and the small transformer for text generation, SAGE runs comfortably on hardware that can't even load a 7B model.

---

## 5. Phase Roadmap

### Phase 1: Local SAGE Chat
**Status: In Progress** | **Target: Q1 2026**

The foundation. A single-node SAGE that can chat, learn, and remember.

- [x] NCA grid with 26 channels (expanding to 32)
- [x] Text encoder (text → grid)
- [x] Memory channels (attention, gate, value, recency)
- [x] TUI interface
- [ ] Knowledge extraction (grid → context vector)
- [ ] Local transformer head (~100M params, Candle-based)
- [ ] Conversation learning loop (chat → encode → integrate → better responses)
- [ ] Persistent grid state (save/load across sessions)

**Deliverable:** `sage chat` — a local AI that learns from your conversations and gets better over time.

### Phase 2: Knowledge Sync
**Target: Q2 2026**

Connect the nodes. Every SAGE instance can now share what it learns.

- [ ] libp2p networking layer
- [ ] GridDiff format and serialization
- [ ] Gossip protocol (publish/subscribe diffs)
- [ ] Merkle DAG for knowledge versioning
- [ ] Basic diff validation (magnitude, coverage, stability)
- [ ] mDNS for LAN discovery
- [ ] Bootstrap node support

**Deliverable:** Two SAGE nodes on the same network automatically discover each other and share knowledge.

### Phase 3: Community Network
**Target: Q3–Q4 2026**

Scale to the internet. A global network of SAGE nodes forming a collective intelligence.

- [ ] Public bootstrap nodes (community-operated)
- [ ] Reputation system
- [ ] NAT traversal (libp2p relay)
- [ ] Knowledge consensus mechanism
- [ ] Anti-poisoning defenses (quarantine, corroboration)
- [ ] Network statistics dashboard
- [ ] Node operator CLI tools

**Deliverable:** A public SAGE network that anyone can join. Running a node makes the whole network smarter.

### Phase 4: API Compatibility
**Target: Q1 2027**

Make SAGE a drop-in replacement for OpenAI's API.

- [ ] OpenAI-compatible HTTP endpoint (`/v1/chat/completions`)
- [ ] Streaming responses (SSE)
- [ ] Model listing endpoint (reports SAGE capabilities)
- [ ] Token counting compatibility
- [ ] Integration guides for popular frameworks (LangChain, etc.)

**Deliverable:** `OPENAI_BASE_URL=http://localhost:8088/v1` — existing apps work with SAGE, backed by collective intelligence, for free.

### Phase 5: Beyond (2027+)

- **Specialized knowledge domains** — medical, legal, scientific sub-networks
- **Multi-modal** — NCA grids already handle visual patterns; extend to image understanding
- **Mobile nodes** — SAGE on phones (NCA is light enough)
- **Incentive layer** — optional token-based incentives for high-quality contributors (opt-in, not required)

---

## 6. Technical Deep Dive

### 6.1 NCA Update Rules for Knowledge Integration

The NCA update rule is the heart of SAGE. Each cell updates based on its 3×3 neighborhood using a learned function:

```
perception(cell) = [
    sobel_x ⊛ channels,   // Horizontal gradients
    sobel_y ⊛ channels,   // Vertical gradients
    identity ⊛ channels    // Raw neighbor averages
]
// Result: 3 × 32 channels = 96-dimensional perception vector

update(cell) = MLP(perception(cell))
// MLP: 96 → 384 (ReLU) → 32 (channels)
// Stochastic: each cell has 50% chance of updating (async dynamics)

cell_new = cell + update(cell) × dt  // Residual update, dt=0.1
```

**For knowledge integration**, we add a **gated write mechanism**:

```rust
fn knowledge_integration_step(
    grid: &mut NCAGrid,
    knowledge_patch: &KnowledgePatch,
) {
    // Phase 1: Write knowledge to target region
    for (x, y, channels) in knowledge_patch.cells() {
        let write_gate = sigmoid(grid[y][x][MEMORY_GATE]);
        for (ch, val) in channels {
            grid[y][x][ch] += val * write_gate * LEARNING_RATE;
        }
        grid[y][x][CONFIDENCE] = knowledge_patch.confidence;
        grid[y][x][TEMPORAL_AGE] = 0.0;
    }

    // Phase 2: Let NCA dynamics integrate the new knowledge
    // This is where the magic happens — the automata rules
    // cause the new knowledge to form associations with
    // existing patterns in neighboring cells
    for _ in 0..INTEGRATION_STEPS {
        grid.nca_step();  // Standard NCA update

        // Additional: temporal decay on all cells
        for y in 0..GRID_SIZE {
            for x in 0..GRID_SIZE {
                grid[y][x][TEMPORAL_AGE] += DECAY_RATE;
                grid[y][x][CONFIDENCE] *= CONFIDENCE_DECAY;
            }
        }
    }
}
```

The key insight is that NCA integration steps cause **emergent knowledge association**. When new knowledge is written near existing related knowledge, the NCA dynamics create activation patterns that bridge them — forming connections that neither piece of knowledge had in isolation.

### 6.2 Encoder/Decoder Architecture

#### Text Encoder (text → grid)

```
Input: "The Rust borrow checker prevents data races"

Tokenize → [The, Rust, borrow, checker, prevents, data, races]

Per-token embedding (32K vocab, 256-dim):
  → 7 × 256 matrix

Sentence encoding (bidirectional GRU, 2 layers):
  → 512-dim sentence vector

Spatial projection (learned linear: 512 → 2):
  → (x, y) = (14.3, 22.7)  // Grid coordinates for this concept

Channel projection (learned linear: 512 → 32):
  → Channel activations for semantic content

Write to grid at (14, 23) with Gaussian spread (σ=2)
```

The spatial projection is trained to cluster semantically similar concepts nearby:
- Programming languages → upper-left quadrant
- Emotions → lower-right
- Science → center-right
- etc.

This topology emerges from training, not manual assignment. It's analogous to how the brain develops specialized regions through experience.

#### Knowledge Decoder (grid → context)

```
Query: "Tell me about memory safety in Rust"

1. Encode query → (x, y) = (14.1, 22.9)  // Near where Rust concepts live

2. Attention readout:
   for each cell in grid:
     relevance = dot(query_embedding, cell[SEMANTIC_CHANNELS])
     weight = softmax(relevance / sqrt(d))
   knowledge_vector = weighted_sum(cell_states × weights)
   // 1024-dim vector summarizing relevant grid knowledge

3. Context generation:
   Feed knowledge_vector through small decoder MLP:
   → "Relevant knowledge: Rust uses ownership and borrowing to
      guarantee memory safety at compile time. The borrow checker
      enforces that references follow strict aliasing rules."

4. This context prefix is prepended to the transformer's input:
   [NCA context] + [conversation history] + [user query]
   → Transformer generates fluent response
```

### 6.3 Conflict Resolution

When two nodes develop conflicting knowledge (e.g., Node A learns "Python is slow" while Node B learns "Python is fast for NumPy operations"), the conflict resolution works at the grid level:

```rust
fn merge_diffs(
    local: &NCAGrid,
    incoming: &GridDiff,
    peer_reputation: f32,
) -> MergeResult {
    let mut conflicts = Vec::new();

    for update in &incoming.updates {
        let (x, y) = (update.x as usize, update.y as usize);

        // Check for conflict: both local and incoming modified this cell recently
        let local_age = local[y][x][TEMPORAL_AGE];
        let local_confidence = local[y][x][CONFIDENCE];

        if local_age < FRESH_THRESHOLD && incoming.confidence > 0.5 {
            // Both have recent, confident knowledge for this cell
            // → Weighted merge based on confidence and reputation
            let local_weight = local_confidence;
            let remote_weight = incoming.confidence * peer_reputation;
            let total = local_weight + remote_weight;

            for (ch, delta) in &update.channel_deltas {
                let blended = local[y][x][*ch as usize] * (local_weight / total)
                    + (local[y][x][*ch as usize] + delta) * (remote_weight / total);
                conflicts.push(CellConflict {
                    x, y,
                    channel: *ch,
                    resolution: blended,
                });
            }
        } else {
            // No conflict — apply directly
            apply_update(local, update);
        }
    }

    // Mark conflict cells for extra NCA integration steps
    // The automata dynamics help smooth out merged knowledge
    MergeResult { conflicts, integration_steps: conflicts.len() * 2 }
}
```

**Philosophy:** Conflicts aren't failures — they're *information*. Two nodes having different knowledge about the same concept means the network has observed that concept in different contexts. The merge preserves nuance rather than picking a winner.

### 6.4 Privacy Architecture

SAGE's privacy model is built on the **lossy compression** inherent in NCA encoding:

#### What CANNOT be recovered from grid diffs:

1. **Exact text** — The encoder is many-to-one; infinite texts map to the same grid state
2. **Personal details** — Names, addresses, etc. are stripped during encoding (pre-filter) and further destroyed by the lossy projection
3. **Conversation flow** — Diffs are per-concept, not per-conversation. The temporal sequence is lost
4. **Minority opinions** — A single conversation shifts the grid minimally; only patterns that recur across many conversations create significant diffs

#### What CAN be inferred from grid diffs:

1. **Topics discussed** — A diff touching the "medicine" region of the grid reveals medical topics were discussed (but not what specifically)
2. **Relative interest** — A node producing many diffs in one area is clearly focused on that topic
3. **Temporal patterns** — When diffs are published reveals usage patterns

#### Privacy Controls

```toml
# sage.toml — per-node configuration
[privacy]
# Which channels to share (default: semantic only, not memory/personal)
shared_channels = [8, 9, 10, 11, 12, 13, 14, 15]

# Minimum aggregation — don't share until N conversations contribute
min_conversations_before_share = 5

# Differential privacy noise (ε parameter)
dp_epsilon = 1.0

# Topics to never share (keyword blocklist → grid region masking)
private_topics = ["health", "finance", "relationships"]

# Share diffs with specific peers only
trusted_peers = ["peer_id_1", "peer_id_2"]
```

#### Differential Privacy

Grid diffs include calibrated noise to provide formal differential privacy guarantees:

```rust
fn add_dp_noise(diff: &mut GridDiff, epsilon: f32) {
    let sensitivity = MAX_DELTA;  // Maximum effect of one conversation
    let scale = sensitivity / epsilon;  // Laplace noise scale

    for update in &mut diff.updates {
        for (_, delta) in &mut update.channel_deltas {
            *delta += laplace_noise(scale);
        }
    }
}
```

With ε=1.0, an observer cannot determine with meaningful confidence whether any specific conversation contributed to a diff.

---

## 7. Research Novelty

SAGE Distributed Intelligence presents several novel contributions to the field:

### 7.1 NCA as a Distributed Knowledge Store

**No one has used Neural Cellular Automata as a knowledge representation that syncs across a peer-to-peer network.** NCA has been explored for:
- Texture generation (Mordvintsev et al., 2020)
- Growing neural networks (self-organizing systems)
- Image regeneration and robustness

Using NCA as a **semantic knowledge substrate** that accepts textual input, develops associative structure through automata dynamics, and produces knowledge context for language generation — this is novel.

### 7.2 Gossip-Based Knowledge Distribution

Federated learning syncs model gradients. SAGE syncs **knowledge state diffs** — a fundamentally different primitive. This is:
- **Orders of magnitude smaller** (KB vs MB)
- **Content-addressable** (Merkle DAG versioning)
- **Asynchronous** (no synchronization rounds)
- **Robust to heterogeneity** (nodes can have different grid sizes, different histories)

### 7.3 Hybrid NCA-Transformer Architecture

The combination of NCA (for knowledge storage/retrieval) with a small transformer (for language generation) creates a novel architecture class where:
- The knowledge store is **alive** — it evolves through automata dynamics, forming new associations without explicit training
- The language model is **small and static** — it doesn't need to store world knowledge, just generate fluent text
- The two components communicate through a **learned attention interface**

This separation of knowledge from generation is philosophically aligned with how biological brains work (long-term memory vs. working memory / language production).

### 7.4 Emergent Collective Intelligence

The most ambitious claim: a network of SAGE nodes exhibits **emergent intelligence** beyond what any single node possesses. This emerges from:
- **Knowledge diversity** — different nodes learn from different conversations, covering more of the knowledge space
- **Association amplification** — when Node A's knowledge about "X" and Node B's knowledge about "Y" merge on Node C, the NCA dynamics can discover X↔Y associations that neither A nor B had
- **Error correction** — incorrect knowledge from one node is outvoted/diluted by correct knowledge from many nodes
- **Specialization** — nodes that focus on specific domains develop deep expertise that benefits the whole network

This is analogous to how a city is smarter than any individual — the collective intelligence emerges from the network of interactions between specialized agents.

### 7.5 Publishable Contributions

1. **"Neural Cellular Automata as Distributed Knowledge Stores"** — Core NCA architecture paper
2. **"Gossip-Based Knowledge Synchronization for Decentralized AI"** — Distribution protocol paper
3. **"Hybrid NCA-Transformer Systems for CPU-Efficient Language AI"** — Systems paper
4. **"Emergent Collective Intelligence in Peer-to-Peer AI Networks"** — Theory/analysis paper
5. **"Privacy-Preserving Knowledge Distribution via Lossy Grid Compression"** — Privacy paper

---

## Appendix A: Wire Format

### GridDiff Binary Format

```
Offset  Size    Field
0       1       Version (0x01)
1       32      Diff ID (blake3 hash)
33      32      Source PeerId
65      8       Base generation (u64 LE)
73      4       Num updates (u32 LE)
77      4       Confidence (f32 LE)
81      64      Ed25519 signature
145     ...     Updates (variable length)

Per update:
0       1       X coordinate (u8)
1       1       Y coordinate (u8)
2       1       Num channel deltas (u8)
3       ...     Channel deltas: (u8 channel, f32 delta) pairs
```

### Gossip Message Envelope

```
Offset  Size    Field
0       1       Message type (0x01=diff, 0x02=heartbeat, 0x03=reputation)
1       4       Payload length (u32 LE)
5       ...     Payload (type-specific)
```

## Appendix B: Performance Estimates

| Metric | Value | Notes |
|--------|-------|-------|
| Grid memory | 128 KB | 32×32×32 × f32 |
| NCA step latency | 0.1 ms | Single core, no SIMD |
| NCA step (SIMD) | 0.02 ms | AVX2 on modern x86 |
| Diff creation | 0.5 ms | Sparse encoding |
| Diff size (typical) | 500 B | 10 cell updates, 3 channels each |
| Diff validation | 2 ms | Includes stability check |
| Network gossip round | ~2 sec | To reach ~1000 nodes |
| Transformer (100M) | 200 ms/token | CPU, 4 cores |
| Full response (50 tokens) | ~10 sec | CPU-only, no GPU |
| Knowledge merge | 5 ms | Per incoming diff |
| Cold start (join network) | ~30 sec | Download diff chain from peers |

## Appendix C: Threat Model

| Threat | Mitigation |
|--------|-----------|
| **Sybil attack** (flood network with fake nodes) | Reputation system, proof-of-contribution (must produce valid diffs to gain reputation) |
| **Poisoning** (inject false knowledge) | Multi-layer validation, confidence consensus, quarantine for new nodes |
| **Eclipse attack** (isolate a node) | Multiple bootstrap peers, DHT-based discovery, connection diversity requirements |
| **Data exfiltration** (extract private info from diffs) | Lossy encoding, differential privacy, channel filtering, min-aggregation |
| **Denial of service** (flood with invalid diffs) | Rate limiting per peer, reputation-based prioritization |
| **Model theft** (steal the trained grid) | Grid is public by design — this is a feature, not a bug. The commons model. |

---

*SAGE Distributed Intelligence is open source. The network belongs to everyone.*

*Join us: [github.com/sage-ai/sage](https://github.com/sage-ai/sage)*
