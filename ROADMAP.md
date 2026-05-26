# SAGE Roadmap

## Vision

SAGE is **The People's AI** — a decentralized intelligence that learns and grows with its community. Think of it as **BitTorrent for intelligence**: knowledge is stored in Neural Cellular Automata (NCA) grids, distributed across a peer-to-peer mesh network, and synchronized via gossip protocols. No central servers, no data silos, no corporate gatekeepers. Every node contributes to and benefits from the collective intelligence.

## Current Status

**Version:** 0.5.2
**Phase:** 3 (Reduce LLM Dependency) — Active Development

---

## Phase Breakdown

### Phase 1: Local Intelligence
**Status:** ✅ Complete (v0.1.x - v0.2.0)

- [x] Local chat interface with NCA knowledge encoding
- [x] Hash-based and semantic (Ollama) embedding support
- [x] Attention-based knowledge retrieval (cross-attention decoder)
- [x] Delta attention for spreading activation readout
- [x] ~~Dream cycle: NCA predictor runs on response regions~~ — **Disabled** (architecturally incoherent, PR #14)
- [x] COCONUT-style continuous thought summaries
- [x] Brain persistence (save/load NCA grid state)

### Phase 2: Mesh Network
**Status:** ✅ Complete (v0.2.x - v0.3.0)

- [x] Peer-to-peer gossip protocol for knowledge synchronization
- [x] Diff-based synchronization (efficient delta updates)
- [x] Full-state fallback for new nodes joining the network
- [x] Trust tiers and validation for incoming knowledge
- [x] Two-node integration tests (roundtrip serialization, diff sync)
- [x] Network transport abstraction layer
- [x] Ed25519 identity — real cryptographic node identity (PR #2)
- [x] Signed KnowledgeDiffs — prevents poisoning attacks (PR #19, #23)
- [x] Network observability dashboard (whatssage.ai/network) (PR #15)
- [x] Direct peer messaging via request-response (PR #13)

### Phase 3: Reduce LLM Dependency
**Status:** 🔄 In Progress (v0.3.x - v0.5.x)

The goal: make the NCA grid handle more retrieval natively, reducing or eliminating the Ollama requirement for basic use cases.

- [x] Fix delta attention query-conditioning (make retrieval query-aware) — **Shipped** (query-conditioned local contrast retrieval)
- [x] Improve retrieval hit rate beyond 50% — **Achieved ~96%** with LinearProjection (PRs #10-12)
- [x] NCA-native embedding (reduce reliance on external embedding models) — **LinearProjection shipped** (learned hash→semantic embedding)
- [x] **Intelligent query router** — Shipped (v0.3.7): detects 12 query patterns, tracks NCA vs LLM accuracy, adapts over time
- [x] **NCA-style knowledge consolidation** — Shipped (v0.5.0): Hebbian reinforcement, decay, spreading activation, embedding diffusion on knowledge channels
- [x] **Recency-weighted attention** — Shipped (v0.4.x): conversational recall with time-decay weighting
- [x] **Knowledge lifecycle management** — Shipped (v0.4.x): `sage prune` command, aggregation threshold enforcement
- [x] **Reservoir benchmark** — Shipped (v0.5.2): structured NCA viability testing via `sage reservoir bench`
- [ ] Lightweight local inference for simple queries
- [ ] Graceful degradation when Ollama unavailable
- [ ] Retrieval feedback loop — train relevance readout from user signals (BinaryRelevanceReadout)

### Phase 4: Pure NCA Intelligence
**Status:** 🔮 Future (v0.6.x+)

The endgame: a fully self-contained AI that runs entirely on NCA dynamics — no LLM backend at all.

- [ ] NCA-based response generation
- [ ] Self-supervised learning from conversation context
- [ ] Emergent reasoning from cellular automata dynamics
- [ ] Fully offline, fully decentralized intelligence

---

## What's New in v0.5.2

- **Reservoir benchmark** — `sage reservoir bench` for structured NCA viability testing
- **Aggregation threshold enforcement** — Prevents low-quality knowledge from spreading in sync loop
- **NCA-style consolidation** (v0.5.0) — Real cellular automata dynamics on knowledge channels: Hebbian reinforcement, decay, spreading activation
- **Recency-weighted attention** — Conversational recall with time-decay (PR #39)
- **Knowledge pruning** — `sage prune` for lifecycle management (PR #40)
- **Signed diffs** — Ed25519 cryptographic signatures on all knowledge diffs (v0.3.4)
- **Network dashboard** — Live node stats at whatssage.ai/network
- **Retrieval metrics** — Hit rate, relevance buckets, TUI display (PR #18)
- **LinearProjection** — Learned hash→semantic embedding, 96% retrieval hit rate
- **Brain API endpoint** — `GET /v1/sage/brain` for network dashboard visualization
- **Dream cycle disabled** — Replaced with query-conditioned retrieval and real NCA consolidation

## Active Work (Next 2-4 Weeks)

1. **Local inference pipeline** — Candle-based lightweight inference for simple queries (Phase 3 completion)
2. **Documentation overhaul** — Examples, tutorials, API docs (this is actively being worked on)
3. **Contributor onboarding** — Better `cargo run --example` experience, Docker compose, dev environment
4. **Test speed** — Profile and optimize slow grid tests for faster dev feedback
5. **Miniworld + OpenClaw bridge** — Complete sub-agent spawning via web API

## Known Issues

- [ ] Integration tests can be slow (large grid operations in `test_nca_predict_deterministic`)
- [ ] ~20 clippy warnings remaining (cosmetic)
- [ ] WASM build needs CI integration
- [ ] Examples are minimal (only 2) — expanding

---

## How to Contribute

1. **Code** — Pick up an issue, submit a PR
2. **Testing** — Run a node, stress-test retrieval, report bugs
3. **Documentation** — Improve docs, write tutorials
4. **Examples** — Build something cool, show others how
5. **Ideas** — Open discussions, propose features

### Quick Start for Contributors

```bash
# 1. Fork and clone
git clone https://github.com/YOURNAME/sage.git && cd sage

# 2. Build
cargo build --release

# 3. Run tests (lib tests are fast; integration tests take ~2 min)
cargo test --lib
cargo test --test knowledge_roundtrip  # one integration test

# 4. Try an example
cargo run --example simple-chat

# 5. Chat with your build
sage chat
```

Before contributing:
- Read `ARCHITECTURE.md` and `CONTRIBUTING.md`
- Check existing issues and PRs
- For large changes, open an issue first
- Run `cargo fmt` and `cargo clippy` before submitting

## Community

- **Discord:** [https://discord.gg/U999zZUuUV](https://discord.gg/U999zZUuUV)
- **GitHub Issues:** Bug reports, feature requests, discussions
- **Network Dashboard:** https://whatssage.ai/network

---

*Last updated: 2026-05-26*
