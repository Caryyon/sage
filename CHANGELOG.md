# Changelog

## [0.5.2] - 2026-05-21

### Added
- **Reservoir Computing Benchmark Command** — New `sage-reservoir bench` command
  provides structured benchmark comparing:
  - Random NCA + linear readout (tests if topology alone provides signal)
  - Random NCA + spatial stats features (alternative feature extraction)
  - Trained NCA + linear readout (baseline for comparison)
  
  Outputs JSON results to `~/.sage/reservoir_bench.json` with verdict:
  - `nca_viable`: Random NCA + readout beats baseline by 1.5× or more
  - `nca_requires_training`: Trained NCA works but random doesn't
  - `nca_not_viable`: Neither provides significant signal
  
  This gives researchers concrete data to evaluate whether NCA topology
  provides useful signal for token prediction, informing the Phase 3→4
  decision (whether to invest in NCA research or pivot to attention-only).

## [0.5.1] - 2026-05-21

### Fixed
- **Aggregation Threshold Enforcement in Sync Loop** — Privacy fix that
  defers knowledge sync until minimum conversations accumulated. Previously,
  every chat message immediately broadcast knowledge diffs to peers, leaking
  user data and enabling reconstruction attacks. Now enforces the
  `min_conversations_before_sync` threshold (default: 5) before any sync.

  This completes the privacy infrastructure started in v0.5.0. The
  NetworkManager's `AggregationTracker` now integrates with the periodic
  sync loop, ensuring privacy-preserving knowledge sharing.

## [0.5.0] - 2026-05-14

### Added
- **NCA-Style Knowledge Consolidation** — Implements principled dream cycle for
  knowledge channels (26-33) using local NCA update rules:
  - Hebbian reinforcement: frequently accessed cells strengthen over time
  - Decay: inactive cells fade, preventing stale knowledge accumulation
  - Spreading activation: neighbors gain activation (association formation)
  - Embedding diffusion: semantic vectors spread locally for clustering

  This replaces the disabled dream cycle that attempted to blend the token-
  prediction NCA with knowledge channels (architecturally incoherent). The new
  `consolidate_knowledge()` function operates directly on knowledge channels
  with grounded, local 3x3 neighborhood rules.

### Changed
- `step_knowledge()` now calls `consolidate_knowledge(2)` instead of being
  disabled entirely. Knowledge grids now have real NCA dynamics.

### Tests
- 5 new integration tests verify consolidation preserves recall, survives
  persistence, respects thresholds, and converges over multiple rounds.

This release addresses the sage-ml-engineer recommendation to design knowledge
update rules that modify activation/confidence channels based on usage, decay,
and Hebbian-like spreading.

## [0.4.1] - 2026-05-07

### Fixed
- **Direct Protocol Peer ID Registration** — PeerAnnounce messages received via the
  direct protocol now register the SAGE node ID → libp2p PeerId mapping, ensuring
  subsequent send_to() calls route directly instead of falling back to broadcast.
  This was already working for GossipSub-delivered PeerAnnounce, but was missing
  from the direct protocol handler.

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
