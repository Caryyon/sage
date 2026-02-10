//! Knowledge Validation & Anti-Poisoning — Trust scoring, diff validation, and weighted merge.
//!
//! Prevents malicious or garbage data from spreading through the SAGE gossip network.

use std::collections::HashMap;
use std::time::Instant;

use super::diff::KnowledgeDiff;

/// Maximum diff size in bytes (10KB).
const MAX_DIFF_SIZE_BYTES: usize = 10 * 1024;

/// Maximum diffs per minute per peer.
const MAX_DIFFS_PER_MINUTE: usize = 10;

/// Trust tier thresholds.
const HIGH_TRUST_THRESHOLD: f64 = 0.8;
const LOW_TRUST_THRESHOLD: f64 = 0.3;

/// Default trust for new peers.
const DEFAULT_TRUST: f64 = 0.5;

/// Trust adjustment amounts.
const TRUST_INCREASE: f64 = 0.02;
const TRUST_DECREASE: f64 = 0.05;

/// Minimum entropy for activation patterns (prevents all-same diffs).
const MIN_ENTROPY: f64 = 0.01;

/// Minimum unique values for semantic coherence.
const MIN_UNIQUE_VALUES: usize = 2;

/// Trust tier for weighted merge decisions.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TrustTier {
    High,
    Medium,
    Low,
}

/// Result of validating an incoming diff.
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationResult {
    /// Diff is valid, apply at full weight.
    Accept,
    /// Diff is valid but from medium-trust peer, apply at reduced weight.
    AcceptReduced(f64),
    /// Diff is suspicious, send to quarantine.
    Quarantine(String),
    /// Diff is invalid, reject outright.
    Reject(String),
}

/// Per-peer trust score with metadata.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PeerTrust {
    pub node_id: String,
    pub trust: f64,
    pub diffs_accepted: u64,
    pub diffs_rejected: u64,
    pub last_updated_ms: u64,
}

impl PeerTrust {
    pub fn new(node_id: String) -> Self {
        Self {
            node_id,
            trust: DEFAULT_TRUST,
            diffs_accepted: 0,
            diffs_rejected: 0,
            last_updated_ms: crate::network::now_ms(),
        }
    }

    pub fn tier(&self) -> TrustTier {
        if self.trust > HIGH_TRUST_THRESHOLD {
            TrustTier::High
        } else if self.trust >= LOW_TRUST_THRESHOLD {
            TrustTier::Medium
        } else {
            TrustTier::Low
        }
    }

    pub fn increase_trust(&mut self) {
        self.trust = (self.trust + TRUST_INCREASE).min(1.0);
        self.diffs_accepted += 1;
        self.last_updated_ms = crate::network::now_ms();
    }

    pub fn decrease_trust(&mut self) {
        self.trust = (self.trust - TRUST_DECREASE).max(0.0);
        self.diffs_rejected += 1;
        self.last_updated_ms = crate::network::now_ms();
    }
}

/// Trust store — manages per-peer trust scores.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TrustStore {
    pub peers: HashMap<String, PeerTrust>,
}

impl Default for TrustStore {
    fn default() -> Self {
        Self::new()
    }
}

impl TrustStore {
    pub fn new() -> Self {
        Self {
            peers: HashMap::new(),
        }
    }

    pub fn get_or_create(&mut self, node_id: &str) -> &mut PeerTrust {
        self.peers
            .entry(node_id.to_string())
            .or_insert_with(|| PeerTrust::new(node_id.to_string()))
    }

    pub fn get_trust(&self, node_id: &str) -> f64 {
        self.peers
            .get(node_id)
            .map(|p| p.trust)
            .unwrap_or(DEFAULT_TRUST)
    }

    /// Load from ~/.sage/peers.toml (best-effort).
    pub fn load() -> Self {
        let path = dirs::home_dir()
            .map(|h| h.join(".sage").join("peers.toml"))
            .unwrap_or_default();
        if let Ok(contents) = std::fs::read_to_string(&path) {
            toml::from_str(&contents).unwrap_or_else(|_| Self::new())
        } else {
            Self::new()
        }
    }

    /// Save to ~/.sage/peers.toml (best-effort).
    pub fn save(&self) {
        let Some(home) = dirs::home_dir() else { return };
        let dir = home.join(".sage");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("peers.toml");
        if let Ok(contents) = toml::to_string(self) {
            let _ = std::fs::write(path, contents);
        }
    }
}

/// Rate limiter — tracks diff timestamps per peer.
#[derive(Debug)]
pub struct RateLimiter {
    /// peer_id -> list of timestamps of recent diffs
    windows: HashMap<String, Vec<Instant>>,
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

impl RateLimiter {
    pub fn new() -> Self {
        Self {
            windows: HashMap::new(),
        }
    }

    /// Returns true if the peer is within rate limits, and records the event.
    pub fn check_and_record(&mut self, peer_id: &str) -> bool {
        let now = Instant::now();
        let window = self.windows.entry(peer_id.to_string()).or_default();

        // Prune entries older than 60 seconds
        window.retain(|t| now.duration_since(*t).as_secs() < 60);

        if window.len() >= MAX_DIFFS_PER_MINUTE {
            return false;
        }
        window.push(now);
        true
    }
}

/// The main diff validator.
#[derive(Debug)]
pub struct DiffValidator {
    pub trust_store: TrustStore,
    pub rate_limiter: RateLimiter,
}

impl DiffValidator {
    pub fn new(trust_store: TrustStore) -> Self {
        Self {
            trust_store,
            rate_limiter: RateLimiter::new(),
        }
    }

    /// Validate an incoming diff. Returns a ValidationResult.
    pub fn validate(&mut self, diff: &KnowledgeDiff) -> ValidationResult {
        // 1. Size check
        let size = diff.to_bytes().len();
        if size > MAX_DIFF_SIZE_BYTES {
            self.trust_store
                .get_or_create(&diff.source_node)
                .decrease_trust();
            return ValidationResult::Reject(format!(
                "diff too large: {} bytes (max {})",
                size, MAX_DIFF_SIZE_BYTES
            ));
        }

        // 2. Rate limit
        if !self.rate_limiter.check_and_record(&diff.source_node) {
            return ValidationResult::Reject(format!(
                "rate limit exceeded for peer {}",
                diff.source_node
            ));
        }

        // 3. Activation bounds (all values must be 0.0-1.0)
        for change in &diff.changes {
            if change.new_value < 0.0 || change.new_value > 1.0 {
                self.trust_store
                    .get_or_create(&diff.source_node)
                    .decrease_trust();
                return ValidationResult::Reject(format!(
                    "out-of-bounds value {} at ({},{},{})",
                    change.new_value, change.row, change.col, change.channel
                ));
            }
        }

        // 4. Entropy check — reject all-same values
        if !diff.changes.is_empty() && !check_entropy(&diff.changes) {
            self.trust_store
                .get_or_create(&diff.source_node)
                .decrease_trust();
            return ValidationResult::Reject("low entropy: all values identical".into());
        }

        // 5. Semantic coherence — need some minimum complexity
        if diff.changes.len() > 3 && !check_coherence(&diff.changes) {
            return ValidationResult::Quarantine(
                "insufficient pattern complexity".into(),
            );
        }

        // 6. Trust-based decision
        let trust = self.trust_store.get_trust(&diff.source_node);
        let tier = if trust > HIGH_TRUST_THRESHOLD {
            TrustTier::High
        } else if trust >= LOW_TRUST_THRESHOLD {
            TrustTier::Medium
        } else {
            TrustTier::Low
        };

        match tier {
            TrustTier::High => ValidationResult::Accept,
            TrustTier::Medium => {
                // Reduced weight: scale confidence by trust
                let weight = trust;
                ValidationResult::AcceptReduced(weight)
            }
            TrustTier::Low => ValidationResult::Quarantine(format!(
                "low trust peer: {:.2}",
                trust
            )),
        }
    }

    /// Record that a diff was useful (increases sender trust).
    pub fn record_useful(&mut self, node_id: &str) {
        self.trust_store.get_or_create(node_id).increase_trust();
    }

    /// Record that a diff was flagged/bad (decreases sender trust).
    pub fn record_flagged(&mut self, node_id: &str) {
        self.trust_store.get_or_create(node_id).decrease_trust();
    }
}

/// Check entropy of change values. Returns false if all values are identical.
fn check_entropy(changes: &[super::diff::CellChange]) -> bool {
    if changes.len() <= 1 {
        return true; // single change is always fine
    }
    let first = changes[0].new_value;
    let variance: f64 = changes
        .iter()
        .map(|c| (c.new_value - first).powi(2))
        .sum::<f64>()
        / changes.len() as f64;
    variance > MIN_ENTROPY * MIN_ENTROPY
}

/// Check semantic coherence — values should have some diversity.
fn check_coherence(changes: &[super::diff::CellChange]) -> bool {
    // Count distinct quantized values (buckets of 0.1)
    let mut buckets = std::collections::HashSet::new();
    for c in changes {
        buckets.insert((c.new_value * 10.0).round() as i32);
    }
    buckets.len() >= MIN_UNIQUE_VALUES
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::diff::{CellChange, KnowledgeDiff};

    fn make_diff(source: &str, changes: Vec<CellChange>) -> KnowledgeDiff {
        KnowledgeDiff {
            source_node: source.to_string(),
            sequence: 0,
            width: 10,
            height: 10,
            num_channels: 3,
            changes,
            state_hash: [0u8; 32],
            confidence: 0.8,
            timestamp_ms: 0,
        }
    }

    fn change(row: usize, col: usize, ch: usize, val: f64) -> CellChange {
        CellChange::new(row, col, ch, 0.0, val)
    }

    #[test]
    fn test_new_peer_gets_default_trust() {
        let store = TrustStore::new();
        assert!((store.get_trust("unknown") - DEFAULT_TRUST).abs() < f64::EPSILON);
    }

    #[test]
    fn test_trust_increase_decrease() {
        let mut store = TrustStore::new();
        let peer = store.get_or_create("peer-a");
        let initial = peer.trust;
        peer.increase_trust();
        assert!(peer.trust > initial);
        let after_inc = peer.trust;
        peer.decrease_trust();
        assert!(peer.trust < after_inc);
    }

    #[test]
    fn test_trust_clamped() {
        let mut store = TrustStore::new();
        let peer = store.get_or_create("peer-b");
        peer.trust = 0.99;
        peer.increase_trust();
        assert!(peer.trust <= 1.0);

        peer.trust = 0.01;
        peer.decrease_trust();
        assert!(peer.trust >= 0.0);
    }

    #[test]
    fn test_trust_tiers() {
        let mut p = PeerTrust::new("x".into());
        p.trust = 0.9;
        assert_eq!(p.tier(), TrustTier::High);
        p.trust = 0.5;
        assert_eq!(p.tier(), TrustTier::Medium);
        p.trust = 0.1;
        assert_eq!(p.tier(), TrustTier::Low);
    }

    #[test]
    fn test_valid_diff_accepted() {
        let mut validator = DiffValidator::new(TrustStore::new());
        // Set peer to high trust
        validator.trust_store.get_or_create("trusted").trust = 0.9;

        let diff = make_diff(
            "trusted",
            vec![change(0, 0, 0, 0.3), change(1, 1, 1, 0.7)],
        );
        assert_eq!(validator.validate(&diff), ValidationResult::Accept);
    }

    #[test]
    fn test_out_of_bounds_rejected() {
        let mut validator = DiffValidator::new(TrustStore::new());
        let diff = make_diff("bad", vec![change(0, 0, 0, 1.5)]);
        match validator.validate(&diff) {
            ValidationResult::Reject(msg) => assert!(msg.contains("out-of-bounds")),
            other => panic!("expected Reject, got {:?}", other),
        }
    }

    #[test]
    fn test_negative_value_rejected() {
        let mut validator = DiffValidator::new(TrustStore::new());
        let diff = make_diff("bad", vec![change(0, 0, 0, -0.1)]);
        match validator.validate(&diff) {
            ValidationResult::Reject(msg) => assert!(msg.contains("out-of-bounds")),
            other => panic!("expected Reject, got {:?}", other),
        }
    }

    #[test]
    fn test_all_same_values_rejected() {
        let mut validator = DiffValidator::new(TrustStore::new());
        let diff = make_diff(
            "noisy",
            vec![
                change(0, 0, 0, 0.5),
                change(1, 0, 0, 0.5),
                change(2, 0, 0, 0.5),
            ],
        );
        match validator.validate(&diff) {
            ValidationResult::Reject(msg) => assert!(msg.contains("entropy")),
            other => panic!("expected Reject, got {:?}", other),
        }
    }

    #[test]
    fn test_low_trust_quarantined() {
        let mut validator = DiffValidator::new(TrustStore::new());
        validator.trust_store.get_or_create("sketchy").trust = 0.1;

        let diff = make_diff(
            "sketchy",
            vec![change(0, 0, 0, 0.3), change(1, 1, 1, 0.7)],
        );
        match validator.validate(&diff) {
            ValidationResult::Quarantine(msg) => assert!(msg.contains("low trust")),
            other => panic!("expected Quarantine, got {:?}", other),
        }
    }

    #[test]
    fn test_medium_trust_reduced_weight() {
        let mut validator = DiffValidator::new(TrustStore::new());
        // Default trust is 0.5 (medium)
        let diff = make_diff(
            "medium-peer",
            vec![change(0, 0, 0, 0.3), change(1, 1, 1, 0.7)],
        );
        match validator.validate(&diff) {
            ValidationResult::AcceptReduced(w) => assert!(w > 0.0 && w < 1.0),
            other => panic!("expected AcceptReduced, got {:?}", other),
        }
    }

    #[test]
    fn test_rate_limiter() {
        let mut limiter = RateLimiter::new();
        for _ in 0..MAX_DIFFS_PER_MINUTE {
            assert!(limiter.check_and_record("peer-x"));
        }
        // 11th should fail
        assert!(!limiter.check_and_record("peer-x"));
        // Different peer should be fine
        assert!(limiter.check_and_record("peer-y"));
    }

    #[test]
    fn test_record_useful_increases_trust() {
        let mut validator = DiffValidator::new(TrustStore::new());
        let initial = validator.trust_store.get_trust("peer-a");
        validator.record_useful("peer-a");
        assert!(validator.trust_store.get_trust("peer-a") > initial);
    }

    #[test]
    fn test_record_flagged_decreases_trust() {
        let mut validator = DiffValidator::new(TrustStore::new());
        let initial = validator.trust_store.get_trust("peer-a");
        validator.record_flagged("peer-a");
        assert!(validator.trust_store.get_trust("peer-a") < initial);
    }

    #[test]
    fn test_single_change_passes_entropy() {
        let mut validator = DiffValidator::new(TrustStore::new());
        validator.trust_store.get_or_create("peer").trust = 0.9;
        let diff = make_diff("peer", vec![change(0, 0, 0, 0.5)]);
        assert_eq!(validator.validate(&diff), ValidationResult::Accept);
    }
}
