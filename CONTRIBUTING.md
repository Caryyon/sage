# Contributing to SAGE

Thanks for your interest in contributing to SAGE. This guide covers setup, workflow, and guidelines.

## Development Environment

### Prerequisites

- **Rust 1.75+** — Install via [rustup](https://rustup.rs)
- **Ollama** (optional) — For semantic embeddings during testing. Without it, tests use hash fallback.

### System Dependencies

```bash
# Ubuntu/Debian
sudo apt install libssl-dev pkg-config libasound2-dev libclang-dev

# macOS
brew install openssl pkg-config
```

### Clone and Build

```bash
git clone https://github.com/caryyon/sage.git
cd sage
cargo build
```

### Running Tests

```bash
# Run all tests (use /tmp to avoid locking issues with IDE)
CARGO_TARGET_DIR=/tmp/sage-target cargo test --lib -- --skip test_nca_predict_deterministic --skip embedding

# Run a specific test
CARGO_TARGET_DIR=/tmp/sage-target cargo test test_name

# Run with logging
RUST_LOG=debug CARGO_TARGET_DIR=/tmp/sage-target cargo test test_name -- --nocapture
```

**Note:** Some NCA tests are skipped by default because they require deterministic seeding or Ollama.

### Running Locally

```bash
cargo run -- chat                    # Chat mode
cargo run -- node start              # Start network node
cargo run -- status                  # Check health
```

## Code Style

### Formatting

```bash
cargo fmt
```

All code must be formatted with `rustfmt`. CI will reject unformatted code.

### Linting

```bash
cargo clippy -- -D warnings
```

All clippy warnings are treated as errors in CI. Fix them before submitting.

### Guidelines

- No `unwrap()` in library code except tests — use `?` or explicit error handling
- Document public APIs with doc comments
- Keep functions focused and small
- Prefer composition over inheritance
- Use meaningful variable names

## Git Workflow

### Branch Naming

- `feat/description` — New features
- `fix/description` — Bug fixes
- `daily/YYYY-MM-DD` — Daily work branches
- `weekly/YYYY-WXX` — Weekly work branches

### Commit Messages

Write clear, descriptive commit messages:

```
feat: add delta attention for spreading activation retrieval

- Snapshot knowledge channels before NCA freerun
- Compute per-cell L2 delta magnitude
- Retrieve top-K cells with highest activation
- Inject delta-unique results into LLM context
```

Format: `type: description`

Types: `feat`, `fix`, `docs`, `test`, `refactor`, `perf`, `chore`

### Pull Request Checklist

Before submitting a PR:

- [ ] `cargo build` succeeds
- [ ] `cargo test --lib` passes (with skips noted above)
- [ ] `cargo clippy -- -D warnings` has no warnings
- [ ] `cargo fmt` was run
- [ ] CHANGELOG.md updated if user-facing changes
- [ ] Documentation updated if API changes

### PR Process

1. Fork the repo and create your branch from `main`
2. Make your changes
3. Ensure all checks pass locally
4. Push and open a PR against `main`
5. Respond to review feedback
6. Once approved, we'll merge

## What to Contribute

### Good First Issues

Look for issues tagged [`good-first-issue`](https://github.com/caryyon/sage/labels/good-first-issue) on GitHub. These are scoped, well-defined tasks suitable for newcomers.

### Areas We Need Help

- **Testing** — More test coverage, especially for edge cases
- **Documentation** — Improve docs, add examples, clarify confusing parts
- **Performance** — Profile and optimize hot paths
- **Platform support** — Windows compatibility, ARM testing
- **UX** — Error messages, CLI ergonomics, onboarding

### Proposing Features

For non-trivial features:

1. Open an issue first to discuss the approach
2. Reference which phase (1-4) it relates to
3. Describe the problem and proposed solution
4. Wait for feedback before implementing

## Architecture Overview

See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for a full breakdown. Key modules:

- `src/distributed_knowledge/` — NCA grid, encoding, retrieval
- `src/gossip/` — Peer-to-peer protocol
- `src/chat/` — Chat interface and LLM client
- `src/api/` — OpenAI-compatible HTTP API

## Getting Help

- **Discord** — [#dev channel](https://discord.gg/U999zZUuUV) for real-time discussion
- **GitHub Issues** — Bug reports and feature requests
- **Discussions** — Longer-form design conversations

## Code of Conduct

Be respectful. We're building something together. Harassment, trolling, and unconstructive behavior won't be tolerated.

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
