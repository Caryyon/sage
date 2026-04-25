# SAGE Roadmap

## Vision

SAGE is **The People's AI** — a decentralized intelligence that learns and grows with its community. Think of it as **BitTorrent for intelligence**: knowledge is stored in Neural Cellular Automata (NCA) grids, distributed across a peer-to-peer mesh network, and synchronized via gossip protocols. No central servers, no data silos, no corporate gatekeepers. Every node contributes to and benefits from the collective intelligence.

## Current Status

**Version:** 0.3.4
**Phase:** 3 (Reduce LLM Dependency) — In Progress

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
**Status:** 🔄 In Progress (v0.3.x)

The goal: make the NCA grid handle more retrieval natively, reducing or eliminating the Ollama requirement for basic use cases.

- [x] Fix delta attention query-conditioning (make retrieval query-aware) — **Shipped** (query-conditioned local contrast retrieval)
- [x] Improve retrieval hit rate beyond 50% — **Achieved ~96%** with LinearProjection (PRs #10-12)
- [x] NCA-native embedding (reduce reliance on external embedding models) — **LinearProjection shipped** (learned hash→semantic embedding)
- [ ] Lightweight local inference for simple queries
- [ ] Graceful degradation when Ollama unavailable
- [ ] Retrieval feedback loop — train relevance readout from user signals (BinaryRelevanceReadout)

### Phase 4: Pure NCA Intelligence
**Status:** 🔮 Future (v0.4.x+)

The endgame: a fully self-contained AI that runs entirely on NCA dynamics — no LLM backend at all.

- [ ] NCA-based response generation
- [ ] Self-supervised learning from conversation context
- [ ] Emergent reasoning from cellular automata dynamics
- [ ] Fully offline, fully decentralized intelligence

---

## What's New in v0.3.4

- **Signed diffs** — Ed25519 cryptographic signatures on all knowledge diffs
- **Network dashboard** — Live node stats at whatssage.ai/network
- **Retrieval metrics** — Hit rate, relevance buckets, TUI display (PR #18)
- **LinearProjection** — Learned hash→semantic embedding, 96% retrieval hit rate
- **Brain API endpoint** — `GET /v1/sage/brain` for network dashboard visualization
- **Dream cycle disabled** — Architecturally incoherent, replaced with query-conditioned retrieval

## Active Work (Next 2-4 Weeks)

1. **Retrieval feedback loop** — BinaryRelevanceReadout with Adam fine-tuning from user signals
2. **Local inference pipeline** — Candle-based lightweight inference for simple queries
3. **Miniworld + OpenClaw bridge** — Complete the TODO for sub-agent spawning
4. **Documentation catch-up** — README, API docs, architecture overview all stale

## Known Issues

- [ ] Test `test_nca_predict_deterministic` uses large grid (fixed in main)
- [ ] ~70 clippy warnings (cosmetic: loop variable indexing patterns)
- [ ] ROADMAP drift — this file was stale since v0.2.8 (fixed now)

---

## How to Contribute

1. **Code** — Pick up an issue, submit a PR
2. **Testing** — Run a node, stress-test retrieval, report bugs
3. **Documentation** — Improve docs, write tutorials
4. **Ideas** — Open discussions, propose features

Before contributing:
- Read `ARCHITECTURE.md` and `CONTRIBUTING.md`
- Check existing issues and PRs
- For large changes, open an issue first

## Community

- **Discord:** [https://discord.gg/U999zZUuUV](https://discord.gg/U999zUuUV)
- **GitHub Issues:** Bug reports, feature requests, discussions
- **Network Dashboard:** https://whatssage.ai/network

---

*Last updated: 2026-04-24*
