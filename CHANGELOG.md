# CHANGELOG

## v0.2.4 — 2026-03-17

NCA grid comes alive. The knowledge grid now runs real cellular automata update steps during every chat turn instead of sitting as a static hash map. Three update steps before retrieval let the grid react to new input, dream steps after encoding consolidate knowledge, and freerun repair prevents drift. The AttentionDecoder (cross-attention with thalamic spatial gating) is wired into the live retrieval path, making knowledge lookup semantic when Ollama embeddings are available with cosine fallback. Also includes persistent libp2p identity for testnet nodes and general clippy/test cleanup.
