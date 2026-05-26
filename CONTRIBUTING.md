# Contributing to SAGE

Thank you for your interest! SAGE is built by the community.

## Quick Start

See [docs/getting-started/contributing.md](docs/getting-started/contributing.md) for the full guide — environment setup, project structure, test commands, example authoring, and areas needing help.

## TL;DR

```bash
# Build
cargo build --release

# Test (fast)
cargo test --lib

# Check style
cargo fmt
cargo clippy --all-targets

# Try an example
cargo run --example simple-chat
```

## Before Contributing

- Read [ARCHITECTURE.md](ARCHITECTURE.md) for system overview
- Read [ROADMAP.md](ROADMAP.md) for current direction
- Open an issue first for large changes
- Write tests for new code
- Update docs if behavior changes

## Areas Needing Help

- **Documentation** — Tutorials, API docs, architecture explainers
- **Examples** — Build something cool, show others how
- **Performance** — Profile and optimize grid operations  
- **Platform support** — WASM, Raspberry Pi optimizations
- **Networking** — libp2p improvements, NAT traversal
- **Tests** — More edge cases, property-based tests

## Community

- **Discord:** https://discord.gg/U999zZUuUV
- **GitHub Issues:** Bug reports, feature requests, discussions
- **Network Dashboard:** https://whatssage.ai/network

## License

MIT — your contributions are yours, the project stays free and open.
