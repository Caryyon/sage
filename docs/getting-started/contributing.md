# Contributing to SAGE

Welcome! SAGE is built by the community. This guide will get you from zero to a working development environment in under 10 minutes.

## Quick Start (5 minutes)

```bash
# 1. Fork and clone
git clone https://github.com/YOURNAME/sage.git && cd sage

# 2. Build (first build takes ~2-3 min)
cargo build --release

# 3. Run the fast unit tests (~1 min)
cargo test --lib

# 4. Try an example
cargo run --example simple-chat

# 5. Chat with your build
sage chat
```

## Development Environment

### Prerequisites

- **Rust** 1.75+ (`rustc --version` to check)
- **Git** (for cloning and contributing)
- **Ollama** (optional but recommended for full LLM fallback)

### Installing Rust

If you don't have Rust:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### IDE Setup

We recommend **Rust Analyzer** (VS Code extension) for code navigation and inline errors.

## Project Structure

```
sage/
├── src/
│   ├── knowledge_loop.rs           # Core chat loop: text → NCA → response
│   ├── grid.rs                     # 256×256 grid with 38 channels per cell
│   ├── query_router.rs             # Static query complexity router
│   ├── query_router_intelligent.rs # Self-improving router (v0.3.7+)
│   ├── distributed_knowledge/       # Encode, decode, attention, embedder
│   ├── network/                    # libp2p gossip, identity, security
│   ├── inference/                  # NCA predictor, Candle local LLM
│   └── miniworld.rs                # Pixel art town simulation (fun)
├── tests/                          # E2E and integration tests
├── examples/                       # Runnable code samples
│   ├── simple-chat.rs              # Basic chat loop
│   ├── custom-knowledge-source.rs  # Ingest documents
│   ├── node-federation.rs          # Run a mesh node
│   └── router-tuning.rs          # Inspect query router
├── docs/                           # Architecture docs
└── benches/                        # Performance benchmarks
```

## Running Tests

### Fast feedback loop (lib tests only)

```bash
cargo test --lib          # ~1 minute, 256 tests
```

### Full test suite

```bash
cargo test --lib          # Unit tests
cargo test --test e2e_*   # End-to-end tests (slower, need Ollama)
cargo test --test integration/*   # Integration tests
```

### Run a specific test

```bash
cargo test test_nca_predict_deterministic -- --nocapture
```

## Writing Examples

Examples live in `examples/` and are compiled with `cargo run --example NAME`.

Good examples:
- Show one concept clearly
- Include a header comment explaining what they do
- Use real API methods (check `src/lib.rs` for public exports)
- Compile cleanly with `cargo check --example NAME`

See `examples/custom-knowledge-source.rs` for a template.

## Code Style

Before submitting:

```bash
cargo fmt                 # Format all code
cargo clippy --all-targets # Check for warnings
```

Clippy warnings are treated seriously — fix them before PR.

## Making Changes

1. **Open an issue first** for large changes (new features, architecture shifts)
2. **Write tests** for new code
3. **Update docs** if you change behavior
4. **Run the full test suite** before pushing
5. **Keep commits focused** — one logical change per commit

## Common Tasks

### Add a new example

```bash
vim examples/my-example.rs        # Write the example
cargo check --example my-example  # Verify it compiles
git add examples/my-example.rs
git commit -m "feat(example): add my-example demonstrating X"
```

### Update documentation

```bash
vim docs/architecture/nca-brain.md
git add docs/architecture/nca-brain.md
git commit -m "docs: update NCA architecture for v0.5.0"
```

### Fix a slow test

If you find a test taking >10s, check if it's using unnecessarily large grids or iteration counts. The goal is fast feedback — unit tests should finish in seconds, not minutes.

## Areas Needing Help

- **Documentation** — Tutorials, API docs, architecture explainers
- **Examples** — Build something cool, show others how
- **Performance** — Profile and optimize grid operations
- **Platform support** — WASM, Raspberry Pi optimizations
- **Networking** — libp2p transport improvements, NAT traversal
- **Tests** — More edge cases, property-based tests

## Getting Help

- **Discord:** [https://discord.gg/U999zZUuUV](https://discord.gg/U999zZUuUV)
- **GitHub Issues:** Bug reports, feature requests, questions
- **Network Dashboard:** https://whatssage.ai/network

## License

MIT — your contributions are yours, the project stays free and open.

---

*Thanks for helping build The People's AI.* 🐺
