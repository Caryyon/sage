# SAGE Roadmap

## Vision

SAGE is **The People's AI** — a decentralized intelligence that learns and grows with its community. Think of it as **BitTorrent for intelligence**: knowledge is stored in Neural Cellular Automata (NCA) grids, distributed across a peer-to-peer mesh network, and synchronized via gossip protocols. No central servers, no data silos, no corporate gatekeepers. Every node contributes to and benefits from the collective intelligence.

## Current Status

**Version:** 0.2.8
**Phase:** 2 (Mesh Network) complete

---

## Phase Breakdown

### Phase 1: Local Intelligence
**Status:** Complete

- Local chat interface with NCA knowledge encoding
- Hash-based and semantic (Ollama) embedding support
- Attention-based knowledge retrieval (cross-attention decoder)
- Delta attention for spreading activation readout
- Dream cycle: NCA predictor runs on response regions
- COCONUT-style continuous thought summaries
- Brain persistence (save/load NCA grid state)

### Phase 2: Mesh Network
**Status:** Complete

- Peer-to-peer gossip protocol for knowledge synchronization
- Diff-based synchronization (efficient delta updates)
- Full-state fallback for new nodes joining the network
- Trust tiers and validation for incoming knowledge
- Two-node integration tests (roundtrip serialization, diff sync)
- Network transport abstraction layer

### Phase 3: Reduce LLM Dependency
**Status:** In Progress

The goal: make the NCA grid handle more retrieval natively, reducing or eliminating the Ollama requirement for basic use cases.

- Fix delta attention query-conditioning (make retrieval query-aware)
- Improve retrieval hit rate beyond 50%
- NCA-native embedding (reduce reliance on external embedding models)
- Lightweight local inference for simple queries
- Graceful degradation when Ollama unavailable

### Phase 4: Pure NCA Intelligence
**Status:** Future

The endgame: a fully self-contained AI that runs entirely on NCA dynamics — no LLM backend at all.

- NCA-based response generation
- Self-supervised learning from conversation context
- Emergent reasoning from cellular automata dynamics
- Fully offline, fully decentralized intelligence

---

## Near-Term Milestones (Next 4-6 Weeks)

- [ ] **Delta attention query-conditioning** — ensure retrieval is query-aware, not query-agnostic
- [ ] **Improve retrieval hit rate** — target >50% hit rate on benchmark fact-pairs
- [ ] **Persistent knowledge snapshots** — periodic checkpointing with rollback support
- [ ] **First public testnet node** — onboard early adopters to the mesh network
- [ ] **Retrieval feedback loop** — train relevance readout from user signals (BinaryRelevanceReadout)
- [ ] **Documentation** — getting started guide, architecture overview, contribution guide

---

## How to Contribute

We welcome contributions of all kinds:

1. **Code** — Pick up an issue, submit a PR. Start with `good-first-issue` labels.
2. **Testing** — Run a node, stress-test retrieval, report bugs.
3. **Documentation** — Improve docs, write tutorials, clarify confusing parts.
4. **Ideas** — Open discussions, propose features, share research papers.

Before contributing, please:
- Read the code of conduct
- Check existing issues and PRs to avoid duplicates
- For large changes, open an issue first to discuss the approach

---

## Community

Join the conversation:

- **Discord:** [https://discord.gg/U999zZUuUV](https://discord.gg/U999zZUuUV)
- **GitHub Issues:** Bug reports, feature requests, discussions

---

*Last updated: 2026-03-28*
