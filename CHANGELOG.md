# CHANGELOG

## v0.2.5 — 2026-03-26

Network sync is no longer one-sided. `NetworkManager` can now be wired to a `GossipTransport` via the new `with_transport()` constructor, enabling it to actually respond to `GridStateRequest` messages from peers. Previously the handler dropped these requests with a placeholder comment, making initial node bootstrap impossible. Now it replies with `InSync` when Merkle hashes match, sends the full grid on `full_state=true` requests (for fresh peers bootstrapping), or computes a sparse `KnowledgeDiff` from zeros otherwise. Incoming `FullState` responses now perform a proper weighted-average merge of shared channels into the local grid instead of being silently discarded. Four new unit tests cover all response paths using a mock transport.

## v0.2.4 — 2026-03-17

NCA grid comes alive. The knowledge grid now runs real cellular automata update steps during every chat turn instead of sitting as a static hash map. Three update steps before retrieval let the grid react to new input, dream steps after encoding consolidate knowledge, and freerun repair prevents drift. The AttentionDecoder (cross-attention with thalamic spatial gating) is wired into the live retrieval path, making knowledge lookup semantic when Ollama embeddings are available with cosine fallback. Also includes persistent libp2p identity for testnet nodes and general clippy/test cleanup.
