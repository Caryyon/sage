# CHANGELOG

## v0.3.2 — 2026-04-09

Network observability: new `sage-network-server` binary exposes HTTP endpoints for the whatssage.ai/network dashboard. Provides `/api/stats`, `/api/peers`, `/api/identity`, and `/api/health` routes that aggregate real-time data from running SAGE nodes. Also fixes an architecturally incoherent dream cycle that was blending hidden channels from a token-prediction NCA grid into the knowledge grid — removed pending a principled redesign.

## v0.3.1 — 2026-04-02

The `sage chat` TUI now runs retrieval through `KnowledgeLoop`, wiring in the `BinaryRelevanceReadout` feedback loop that was previously trained-but-never-applied in the chat interface. Every retrieval in the TUI now uses the trained readout to re-rank results (0.5×–2.0× scaling based on historical relevance signals), and users can type `/bad` after an unhelpful response to record a negative training signal for that query type. A new `/feedback` command shows how many retrieval events have been recorded, the current relevance rate, and how many training rounds have completed. This closes the feedback loop from ML engineer analysis 2026-03-31: the readout was training in isolation but never influencing live chat retrieval. Also fixed a double-encode bug where user input was encoded into the NCA grid twice per turn.

## v0.3.0 — 2026-03-28

Bundled LLM generation via llama.cpp — SAGE is now fully self-contained. Download a GGUF model once with `sage model download phi3-mini` and SAGE runs entirely on your machine with no external dependencies. Ollama remains supported as an alternative backend.

New features:
- **Local LLM generation**: Optional llama.cpp backend via the `local-llm` cargo feature. When enabled and a model is present at `~/.sage/model.gguf`, SAGE uses local generation with no network calls.
- **`sage model` command**: New CLI subcommand for managing local models:
  - `sage model download phi3-mini` — Downloads Phi-3-mini GGUF (~2.3GB) from HuggingFace
  - `sage model list` — Shows available preset models with sizes and descriptions
  - `sage model status` — Shows current model status, file size, and generation backend
- **Generation backend detection**: Startup now shows the active generation backend:
  - `⚡ Generation: local (phi3-mini, 2.3GB)` — when using local llama.cpp
  - `🔗 Generation: Ollama (qwen2.5:14b)` — when using Ollama
  - `📚 Generation: offline (retrieval only)` — when no LLM available
- **Model presets**: Three preset models available for download:
  - `phi3-mini` (2.3GB) — Microsoft Phi-3 Mini, fast and good for Q&A
  - `tinyllama` (600MB) — TinyLlama 1.1B, fastest but lowest quality
  - `mistral-7b` (4.1GB) — Mistral 7B, best quality, needs 8GB RAM
- **GPU acceleration**: Optional CUDA (`--features cuda`) and Metal (`--features metal`) support

Build notes:
- The `local-llm` feature requires cmake and a C++ compiler. Default build compiles cleanly without it.
- Install with local LLM support: `cargo install sage --features local-llm`
- For GPU acceleration: `cargo install sage --features local-llm,cuda` (NVIDIA) or `cargo install sage --features local-llm,metal` (Apple Silicon)

## v0.2.9 — 2026-03-28

Bundled embeddings via fastembed-rs: SAGE now generates high-quality semantic embeddings without requiring Ollama. The AllMiniLML6V2 model (22MB, 384-dim) is bundled directly into SAGE and downloaded on first use to `~/.cache/fastembed/`. Benchmark results show 96% retrieval hit rate with fastembed vs ~12% with the previous hash fallback — a massive improvement in semantic retrieval quality for offline use.

New embedding priority order: fastembed (bundled) > Ollama (if running) > hash fallback. Startup messaging now displays the active embedding backend: `Embeddings: bundled (AllMiniLML6V2, 384-dim)` when fastembed is available.

Added `src/distributed_knowledge/embedder.rs` with `OnceLock`-based lazy initialization, `embed_text()` and `is_available()` functions, and tests for dimension (384), determinism, and semantic similarity (cat/dog vs cat/car). Wired into `encoder.rs` as the primary embedding path.

## v0.2.6 — 2026-03-27

NCA Delta Attention: live grid dynamics now shape knowledge retrieval. Previously the NCA update steps ran but only touched hidden channels 4-15 — nobody read them. Now, before each retrieval, the system snapshots the knowledge channels (26-31), runs 4 NCA freerun steps across the full grid, computes per-cell L2 delta magnitude, and retrieves the top-K cells with the highest delta. These are the "activated" concepts — what the NCA grid thought about in response to the query. Delta-unique results (not found by semantic/hash retrieval) are injected into the LLM system prompt as associatively recalled concepts, implementing spreading activation memory.

Also: fixed decoder projection consistency (merged `fix/decoder-projection-consistency` — `decode_region` now uses the same strided projection as the encoder, fixing broken cosine similarity); wired memory channel 25 (recency) so it's written with 1.0 after each new encoding and decays 5% per turn (channels 22-25 were allocated but never written); added `DELTA_UNIQUE_HITS`/`TOTAL_RETRIEVALS` atomic counters for delta retrieval quality tracking; added `Grid::snapshot_knowledge_channels()` and `Grid::compute_delta_magnitude()` methods; added `AttentionDecoder::attend_with_delta()` method; added two new tests for delta retrieval correctness.

## v0.2.5 — 2026-03-26

Network sync is no longer one-sided. `NetworkManager` can now be wired to a `GossipTransport` via the new `with_transport()` constructor, enabling it to actually respond to `GridStateRequest` messages from peers. Previously the handler dropped these requests with a placeholder comment, making initial node bootstrap impossible. Now it replies with `InSync` when Merkle hashes match, sends the full grid on `full_state=true` requests (for fresh peers bootstrapping), or computes a sparse `KnowledgeDiff` from zeros otherwise. Incoming `FullState` responses now perform a proper weighted-average merge of shared channels into the local grid instead of being silently discarded. Four new unit tests cover all response paths using a mock transport.

## v0.2.4 — 2026-03-17

NCA grid comes alive. The knowledge grid now runs real cellular automata update steps during every chat turn instead of sitting as a static hash map. Three update steps before retrieval let the grid react to new input, dream steps after encoding consolidate knowledge, and freerun repair prevents drift. The AttentionDecoder (cross-attention with thalamic spatial gating) is wired into the live retrieval path, making knowledge lookup semantic when Ollama embeddings are available with cosine fallback. Also includes persistent libp2p identity for testnet nodes and general clippy/test cleanup.
