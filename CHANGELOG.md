# CHANGELOG

## v0.2.6 — 2026-03-27

NCA Delta Attention: live grid dynamics now shape knowledge retrieval. Previously the NCA update steps ran but only touched hidden channels 4-15 — nobody read them. Now, before each retrieval, the system snapshots the knowledge channels (26-31), runs 4 NCA freerun steps across the full grid, computes per-cell L2 delta magnitude, and retrieves the top-K cells with the highest delta. These are the "activated" concepts — what the NCA grid thought about in response to the query. Delta-unique results (not found by semantic/hash retrieval) are injected into the LLM system prompt as associatively recalled concepts, implementing spreading activation memory.

Also: fixed decoder projection consistency (merged `fix/decoder-projection-consistency` — `decode_region` now uses the same strided projection as the encoder, fixing broken cosine similarity); wired memory channel 25 (recency) so it's written with 1.0 after each new encoding and decays 5% per turn (channels 22-25 were allocated but never written); added `DELTA_UNIQUE_HITS`/`TOTAL_RETRIEVALS` atomic counters for delta retrieval quality tracking; added `Grid::snapshot_knowledge_channels()` and `Grid::compute_delta_magnitude()` methods; added `AttentionDecoder::attend_with_delta()` method; added two new tests for delta retrieval correctness.

## v0.2.5 — 2026-03-26

Network sync is no longer one-sided. `NetworkManager` can now be wired to a `GossipTransport` via the new `with_transport()` constructor, enabling it to actually respond to `GridStateRequest` messages from peers. Previously the handler dropped these requests with a placeholder comment, making initial node bootstrap impossible. Now it replies with `InSync` when Merkle hashes match, sends the full grid on `full_state=true` requests (for fresh peers bootstrapping), or computes a sparse `KnowledgeDiff` from zeros otherwise. Incoming `FullState` responses now perform a proper weighted-average merge of shared channels into the local grid instead of being silently discarded. Four new unit tests cover all response paths using a mock transport.

## v0.2.4 — 2026-03-17

NCA grid comes alive. The knowledge grid now runs real cellular automata update steps during every chat turn instead of sitting as a static hash map. Three update steps before retrieval let the grid react to new input, dream steps after encoding consolidate knowledge, and freerun repair prevents drift. The AttentionDecoder (cross-attention with thalamic spatial gating) is wired into the live retrieval path, making knowledge lookup semantic when Ollama embeddings are available with cosine fallback. Also includes persistent libp2p identity for testnet nodes and general clippy/test cleanup.
