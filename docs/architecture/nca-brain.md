# NCA Brain Architecture

SAGE stores knowledge in a grid-based system with semantic retrieval.

## Current State (v0.3.8)

**Note:** The "NCA" (Neural Cellular Automata) branding is aspirational. The current
implementation is a **grid-based knowledge store with attention-based retrieval**,
not a trained cellular automata system. Knowledge channels are static storage.

See the [ML Engineer analysis (2026-04-28)](../../../sage-team/sage-ml-engineer/analysis/2026-04-28.md)
for details on what's working vs. aspirational.

## Grid Structure
- 256×256 cells
- 38 channels per cell
- Channels 26-33: knowledge storage (6 embedding slots + activation + confidence)
- Channels 4-15: hidden channels (smoothed via neighbor averaging)
- Channels 34-35: P2P communication
- Channels 36-37: private metadata

## Encoding Process
1. Text → embedding (fastembed or Ollama) → 64-dim feature vector
2. Hash features → grid position + neighborhood spread
3. Write embedding into knowledge channels
4. Run `freerun_repair`: neighbor averaging on hidden channels (NOT trained NCA)

## Retrieval Process
1. Hash query → find candidate cells
2. Semantic search via cross-attention decoder
3. Delta attention for spreading activation
4. Feedback learning via BinaryRelevanceReadout
5. Return top-K results

## What's NOT Implemented Yet
- **Trained NCA dynamics**: Grid uses fixed neighbor-averaging, not learned update rules
- **Knowledge propagation**: Channels 26-33 are static storage, never updated by dynamics
- **Dream cycle**: Disabled since 2026-04-06 (two incompatible grids)
- **Unified architecture**: NCA Predictor (181×181×16) is separate from knowledge grid (256×256×38)

## Roadmap

| Feature | Status | Estimated Effort |
|---------|--------|------------------|
| Train grid dynamics | Planned | 2-3 weeks |
| Query-conditioned updates | Planned | 1-2 weeks |
| Unified architecture | Planned | 4-6 weeks |
| Task-conditioned readout | Planned | 1-2 weeks |

## References
- [`src/grid.rs`](../../src/grid.rs) — Grid constants and channel layout
- [`src/knowledge_loop.rs`](../../src/knowledge_loop.rs) — Core orchestration
- [`src/distributed_knowledge/encoder.rs`](../../src/distributed_knowledge/encoder.rs) — Embedding
- [`src/distributed_knowledge/decoder.rs`](../../src/distributed_knowledge/decoder.rs) — Retrieval
