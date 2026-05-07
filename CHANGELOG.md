# Changelog

## [0.4.0] - 2026-05-04

### Added
- **User Feedback System** — Track query patterns, routing decisions, and satisfaction
  - New module: `src/feedback.rs`
  - CLI commands: `sage feedback stats/submit/export`
  - Data persists to `~/.sage/feedback.json`
  - Tracks NCA vs LLM routing outcomes
- **Router Learning Loop** — Self-improving query routing
  - Router syncs with feedback data before each query
  - Pattern-specific satisfaction tracking
  - Preferences evolve based on actual usage
- **Multi-Node Sync Validation** — Verified knowledge propagation
  - 14 integration tests passing
  - Knowledge diffs propagate correctly between nodes
  - Cross-grid-size sync works
- **Demo Script** — Interactive walkthrough (`./demo.sh`)
  - Shows personal knowledge store
  - Demonstrates contextual retrieval
  - Displays feedback statistics
  - Shows decentralized sync

### Changed
- Updated README with clearer value proposition
- Focus on user benefits rather than technical implementation

### Fixed
- Clippy warnings in feedback module
- Unused variable in query_router_intelligent

**Total: 262 tests passing (248 lib + 14 integration)**

---

## [0.3.8] - 2026-04-30

### Added
- Bootstrap peer configuration via `~/.sage/config.toml` — users can now set
  WAN bootstrap peers without editing source code or managing separate files.
  Config merges: hardcoded defaults → config.toml → bootstrap_peers.txt.
- Example config with documentation (`config.example.toml`)

### Fixed
- Removed unused imports in query_router_intelligent
- Removed unreachable pattern match arm in QueryPattern

This completes the Phase 2 network configuration ergonomics milestone.

## [0.3.7] - 2026-04-25

### Added
- Query routing: Simple→NCA, Complex→LLM
- NCA predictor training scripts
- Integration tests for routing
- Docker support
- CI/CD via GitHub Actions
- Multi-platform builds (Linux, macOS, Windows, ARM)
- Comprehensive documentation
- Tutorials and examples

### Changed
- Logo simplified to green S on dark background
- Version bumped from 0.3.4 → 0.3.7
- Build now clean (0 warnings)

### Fixed
- Dead code warnings in integration tests
- Compiler warnings across codebase

## [0.3.5] - 2026-04-24

### Added
- Install script (curl | bash)
- GitHub Actions release workflow
- MANIFESTO.md
- Brain API endpoint

## [0.3.4] - 2026-04-23

### Added
- Signed knowledge diffs (Ed25519)
- Network observability dashboard
- LinearProjection for retrieval
- COCONUT continuous thought layer
