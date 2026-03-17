# SAGE Knowledge Loop

The Knowledge Loop is the core intelligence cycle of SAGE. It orchestrates the flow from user input through the NCA brain to response generation.

## Pipeline Overview

```
User Input
    │
    ▼
┌─────────────────────────────────────────────────────────────┐
│ 1. ENCODE INPUT                                             │
│    Text → FeatureVector → NCA Grid (knowledge channels)     │
│    Position determined by feature hashing/semantic mapping  │
└──────────────────────────┬──────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────┐
│ 2. RETRIEVE KNOWLEDGE                                        │
│    Query Grid → Cross-Attention (semantic) or Cosine (hash) │
│    Returns top-K relevant text snippets from TextStore      │
└──────────────────────────┬──────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────┐
│ 3. INJECT CONTEXT                                           │
│    Append retrieved knowledge to system prompt              │
│    Build message history for LLM                            │
└──────────────────────────┬──────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────┐
│ 4. GENERATE RESPONSE                                        │
│    LLM (Ollama) generates response with knowledge context   │
└──────────────────────────┬──────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────┐
│ 5. ENCODE RESPONSE                                          │
│    Response text → NCA Grid (creates new knowledge cell)    │
│    Also encodes User+Assistant exchange for associative     │
│    recall                                                   │
└──────────────────────────┬──────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────┐
│ 6. DREAM CYCLE                                              │
│    NcaPredictor runs N steps on the activated region        │
│    Updates hidden channels (4..16) with learned dynamics    │
│    Knowledge channels (26+) preserved                       │
└──────────────────────────┬──────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────┐
│ 7. FREERUN REPAIR                                           │
│    Unconditioned NCA steps (no new input)                   │
│    Local neighbor averaging on hidden channels              │
│    Consolidates activation patterns, prevents drift         │
└──────────────────────────┬──────────────────────────────────┘
                           │
                           ▼
                      Response
```

## Knowledge Retrieval

The retrieval step uses one of two strategies:

### Cross-Attention (Semantic Queries)

When Ollama embeddings are available, the query is encoded semantically and cross-attention is used to find relevant cells:

- **Query (Q)**: Task context embedding from Ollama
- **Keys (K)**: Per-cell embedding vectors (6 slots at channels 26-31)
- **Values (V)**: Same per-cell embeddings

Attention scores are computed as:
```
score = softmax(Q · K^T / sqrt(d_k))
```

Spatial gating (thalamic routing) weights attention by grid quadrant relevance before fine-grained cell selection.

### Cosine Similarity (Hash-Based Queries)

When Ollama is unavailable, hash-based feature encoding is used:

- Search radius around hash-mapped position
- Score = 70% cosine similarity + 30% spatial proximity + 10% confidence
- Faster but less semantically aware

## Freerun Repair

Based on rNCA (Silbernagel et al., 2025), freerun repair consolidates knowledge after encoding:

1. After encoding text, run N unconditioned update steps
2. Each step: compute 8-neighbor average of hidden channels
3. Apply smoothing: `new = 0.7 * current + 0.3 * neighbor_avg`
4. Knowledge channels are NOT modified (verified in debug builds)

This lets the grid "settle" before the next read, preventing semantic drift.

## Configuration

Key KnowledgeLoop parameters:

| Parameter | Default | Description |
|-----------|---------|-------------|
| `relevance_threshold` | 0.3 | Minimum score to include retrieved knowledge |
| `max_results` | 5 | Maximum knowledge snippets per query |
| `user_encode_confidence` | 0.7 | Confidence when encoding user messages |
| `response_encode_confidence` | 0.8 | Confidence when encoding responses |

## Related

- [ARCHITECTURE.md](ARCHITECTURE.md) - Full system architecture
- [DISTRIBUTED.md](DISTRIBUTED.md) - P2P knowledge sync
- arXiv:2603.10055 (Lee et al.) - NCA-to-LLM attention transfer
- rNCA paper (Silbernagel et al., 2025) - Self-repair dynamics
