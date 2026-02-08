# SAGE Distributed Knowledge Benchmarks

**Date:** 2026-02-08  
**Grid Size:** 32×32, 32 channels  
**Encoding:** Hash-based n-gram (no Ollama)

## 📝 Encoding Speed

| Items  | Total (ms) | Per Item (µs) |
|--------|-----------|---------------|
| 100    | 4.5       | 44.5          |
| 1,000  | 38.0      | 38.0          |
| 10,000 | 368.8     | 36.9          |

**Throughput:** ~27,000 items/sec. Encoding is fast and scales linearly.

## 🔍 Retrieval Accuracy

| Metric    | Value |
|-----------|-------|
| Facts     | 101   |
| Queries   | 20    |
| Hits      | 8     |
| Precision | 40.0% |
| Recall    | 40.0% |

**Analysis:** Hash-based encoding provides moderate retrieval accuracy. The n-gram hash approach maps related but not identical text to nearby grid positions, so exact-match retrieval is limited. Semantic (Ollama) embeddings would significantly improve these numbers.

## 📦 Diff Size

| Metric               | Value |
|----------------------|-------|
| Items encoded        | 100   |
| Avg changes per item | 8.5   |
| Avg bytes per delta  | 566 B |

**Analysis:** Each knowledge item produces ~566 bytes of diff data on average — very compact for network sync. The spatial spread radius of 3 creates ~8.5 changed cells per item.

## 🔀 Merge Quality

| Metric                        | Value |
|-------------------------------|-------|
| Node A items                  | 50    |
| Node B items                  | 50    |
| A retrievable after merge     | 7/50  |
| B retrievable after merge     | 0/50  |
| Degradation                   | 93.0% |

### Fill Level Degradation

| Items Encoded | Retrievable (of 50 sampled) | Quality |
|--------------|----------------------------|---------|
| 50           | 7                          | 14.0%   |
| 100          | 5                          | 10.0%   |
| 200          | 4                          | 8.0%    |
| 500          | 1                          | 2.0%    |

**Analysis:** Merge quality is poor with hash-based encoding — grid cell collisions overwrite earlier knowledge. This is the primary area for improvement: the encoder needs better spatial hashing or multi-cell encoding to prevent destructive interference. The TextStore (side-channel) preserves all text, but the grid-based retrieval path loses information as the grid fills up.

## 📊 Grid Capacity

| Items  | Retrieval Quality |
|--------|------------------|
| 10     | 20.0%            |
| 25     | 8.0%             |
| 50     | 14.0%            |
| 100    | 8.0%             |
| 200    | 6.0%             |
| 500    | 0.0%             |
| 1,000  | 0.0%             |
| 2,000  | 0.0%             |

**Analysis:** The 32×32 grid (1,024 cells) saturates quickly. Even at 10 items, hash collisions and spatial spread overlap cause only 20% exact retrieval. The grid is better suited as a "fuzzy index" — it finds *related* knowledge (network sim shows 100% for non-exact queries) but struggles with exact text matching.

## 🌐 Network Simulation

| Nodes | Total Items | Retrieval Quality | Storage  | Gossip Rounds |
|-------|------------|-------------------|----------|---------------|
| 10    | 100        | 100.0%            | 271 KB   | 4             |
| 50    | 500        | 100.0%            | 271 KB   | 6             |
| 100   | 1,000      | 100.0%            | 271 KB   | 7             |

**Analysis:** Network gossip sync works excellently. After O(log n) rounds of full merge, all nodes converge. The query-based retrieval (top-5 results, any non-empty = hit) shows 100% — the grid *does* activate in response to related queries, even if exact text matching is imprecise. Storage is constant (one 32×32×32 grid = 271 KB) regardless of node count.

## Key Takeaways

1. **Encoding is fast** — 37 µs/item at scale, suitable for real-time knowledge ingestion
2. **Grid is a fuzzy index** — excellent for "is there related knowledge?" but poor for exact retrieval
3. **Network sync works** — logarithmic gossip rounds, constant storage per node
4. **Diffs are compact** — ~566 bytes per knowledge update, efficient for bandwidth
5. **Capacity is limited** — 32×32 grid saturates quickly; exact retrieval degrades after ~10 items
6. **Merge is destructive** — knowledge collisions in the grid overwrite earlier entries

## Improvement Priorities

1. **Semantic embeddings** (Ollama) for better spatial clustering
2. **Multi-cell encoding** to spread knowledge across more cells and reduce collisions
3. **Larger grids** (64×64 or 128×128) for more capacity
4. **Collision-aware encoding** that checks for existing knowledge before overwriting
5. **Hierarchical grids** for scaling beyond single-grid capacity
