# NCA Brain Architecture

SAGE stores knowledge in a grid-based system with semantic retrieval and NCA-style consolidation.

## Current State (v0.5.0)

The knowledge grid now has real NCA-style dynamics through `consolidate_knowledge()`, which operates
on knowledge channels (26-33) with local update rules: Hebbian reinforcement, decay, spreading
activation, and embedding diffusion.

The token-prediction NCA grid (181×181×16) remains separate and is used by the `NcaPredictor` for
routing decisions. The dream cycle that attempted to blend these two grids is disabled—the
consolidation now operates directly on knowledge channels with grounded semantics.

See [CHANGELOG.md](../../CHANGELOG.md) for the v0.5.0 release details.

## Grid Structure
- 256×256 cells
- 38 channels per cell
- Channels 0-3: RGBA (base layer)
- Channels 4-15: hidden channels (smoothed via neighbor averaging)
- Channels 16-19: pattern channels (one-hot encoding)
- Channels 20-21: environment channels (food/toxin)
- Channels 22-25: memory channels (attention, gate, value, recency)
- Channels 26-33: **knowledge channels** (6 embedding slots + activation + confidence)
- Channels 34-35: P2P communication
- Channels 36-37: private metadata

## Encoding Process
1. Text → embedding (fastembed or Ollama) → 64-dim feature vector
2. Hash features → grid position + neighborhood spread
3. Write embedding into knowledge channels
4. Track access for consolidation

## Consolidation (Dream Cycle)

`consolidate_knowledge(steps)` runs NCA-style updates on knowledge channels:

1. **Hebbian reinforcement**: Frequently accessed cells strengthen activation
2. **Decay**: Inactive cells fade (forgetting)
3. **Spreading activation**: Neighbors gain activation (association formation)
4. **Embedding diffusion**: Semantic vectors spread locally (clustering)

This replaces the previous disabled dream cycle. The algorithm:
- Uses local 3×3 neighborhood (NCA-style)
- Maintains channel bounds [0, 1] for stability
- Operates directly on knowledge channels (grounded semantics)

## Retrieval Process
1. Hash query → find candidate cells
2. Semantic search via cross-attention decoder
3. Delta attention for spreading activation
4. Feedback learning via BinaryRelevanceReadout
5. Return top-K results

## Architecture Status

| Feature | Status |
|---------|--------|
| Grid-based knowledge store | ✅ Implemented |
| Attention-based retrieval | ✅ Implemented |
| NCA-style consolidation | ✅ Implemented (v0.5.0) |
| Retrieval feedback learning | ✅ Implemented |
| P2P knowledge sync | ✅ Implemented |
| Trained grid dynamics | 🔄 Future work |
| Unified NCA architecture | 🔄 Future work (NcaPredictor separate) |

## References
- [`src/grid.rs`](../../src/grid.rs) — Grid constants, channel layout, and `consolidate_knowledge()`
- [`src/knowledge_loop.rs`](../../src/knowledge_loop.rs) — Core orchestration
- [`src/distributed_knowledge/encoder.rs`](../../src/distributed_knowledge/encoder.rs) — Embedding
- [`src/distributed_knowledge/decoder.rs`](../../src/distributed_knowledge/decoder.rs) — Retrieval
