//! Knowledge Quarantine — Holds suspicious diffs for review before applying.
//!
//! Diffs from low-trust peers or those failing coherence checks are quarantined.
//! They can be promoted if corroborated by multiple trusted peers, or auto-expired.

use std::collections::HashMap;

use super::diff::KnowledgeDiff;
use super::validation::TrustStore;

/// How long quarantined items live before auto-expiry (24 hours in ms).
const QUARANTINE_TTL_MS: u64 = 24 * 60 * 60 * 1000;

/// Minimum trusted corroborations to promote from quarantine.
const MIN_CORROBORATIONS: usize = 2;

/// Minimum trust to count as a corroborating peer.
const CORROBORATION_TRUST_THRESHOLD: f64 = 0.5;

/// Similarity threshold for considering two diffs as corroborating.
const SIMILARITY_THRESHOLD: f64 = 0.8;

/// A quarantined diff entry.
#[derive(Debug, Clone)]
pub struct QuarantinedDiff {
    pub diff: KnowledgeDiff,
    pub reason: String,
    pub quarantined_at_ms: u64,
    /// Node IDs that sent similar diffs (corroborations).
    pub corroborated_by: Vec<String>,
}

/// The quarantine store.
#[derive(Debug)]
pub struct Quarantine {
    items: Vec<QuarantinedDiff>,
}

impl Default for Quarantine {
    fn default() -> Self {
        Self::new()
    }
}

impl Quarantine {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    /// Add a diff to quarantine.
    pub fn add(&mut self, diff: KnowledgeDiff, reason: String) {
        let now = crate::network::now_ms();

        // Check if we already have a similar diff — if so, add corroboration
        for item in &mut self.items {
            if diffs_similar(&item.diff, &diff) >= SIMILARITY_THRESHOLD {
                if !item.corroborated_by.contains(&diff.source_node) {
                    item.corroborated_by.push(diff.source_node.clone());
                }
                return;
            }
        }

        self.items.push(QuarantinedDiff {
            diff,
            reason,
            quarantined_at_ms: now,
            corroborated_by: Vec::new(),
        });
    }

    /// Remove expired items (older than 24 hours).
    pub fn expire(&mut self) {
        let now = crate::network::now_ms();
        self.items
            .retain(|item| now - item.quarantined_at_ms < QUARANTINE_TTL_MS);
    }

    /// Check for items that have enough corroborations from trusted peers.
    /// Returns promoted diffs and removes them from quarantine.
    pub fn promote(&mut self, trust_store: &TrustStore) -> Vec<KnowledgeDiff> {
        let mut promoted = Vec::new();
        let mut to_remove = Vec::new();

        for (i, item) in self.items.iter().enumerate() {
            let trusted_corroborations = item
                .corroborated_by
                .iter()
                .filter(|node| trust_store.get_trust(node) >= CORROBORATION_TRUST_THRESHOLD)
                .count();

            if trusted_corroborations >= MIN_CORROBORATIONS {
                promoted.push(item.diff.clone());
                to_remove.push(i);
            }
        }

        // Remove in reverse order to preserve indices
        for i in to_remove.into_iter().rev() {
            self.items.remove(i);
        }

        promoted
    }

    /// Current quarantine size.
    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

/// Compute similarity between two diffs (0.0-1.0) based on overlapping changes.
fn diffs_similar(a: &KnowledgeDiff, b: &KnowledgeDiff) -> f64 {
    if a.changes.is_empty() && b.changes.is_empty() {
        return 1.0;
    }
    if a.changes.is_empty() || b.changes.is_empty() {
        return 0.0;
    }

    // Build a map of (row,col,ch) -> new_value for diff a
    let a_map: HashMap<(usize, usize, usize), f64> = a
        .changes
        .iter()
        .map(|c| ((c.row, c.col, c.channel), c.new_value))
        .collect();

    let mut matches = 0usize;
    for c in &b.changes {
        if let Some(&val) = a_map.get(&(c.row, c.col, c.channel)) {
            if (val - c.new_value).abs() < 0.1 {
                matches += 1;
            }
        }
    }

    let max_len = a.changes.len().max(b.changes.len());
    matches as f64 / max_len as f64
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
    fn test_add_and_len() {
        let mut q = Quarantine::new();
        assert!(q.is_empty());
        q.add(
            make_diff("a", vec![change(0, 0, 0, 0.5)]),
            "test".into(),
        );
        assert_eq!(q.len(), 1);
    }

    #[test]
    fn test_similar_diffs_corroborate() {
        let mut q = Quarantine::new();
        let changes = vec![change(0, 0, 0, 0.5), change(1, 1, 1, 0.7)];
        q.add(make_diff("a", changes.clone()), "low trust".into());
        q.add(make_diff("b", changes.clone()), "low trust".into());
        // Similar diff from b should corroborate, not add new entry
        assert_eq!(q.len(), 1);
        assert_eq!(q.items[0].corroborated_by.len(), 1);
        assert_eq!(q.items[0].corroborated_by[0], "b");
    }

    #[test]
    fn test_promote_with_enough_corroborations() {
        let mut q = Quarantine::new();
        let mut trust = TrustStore::new();

        let changes = vec![change(0, 0, 0, 0.5), change(1, 1, 1, 0.7)];
        q.add(make_diff("a", changes.clone()), "low trust".into());

        // Add corroborations from trusted peers
        trust.get_or_create("b").trust = 0.7;
        trust.get_or_create("c").trust = 0.7;
        q.add(make_diff("b", changes.clone()), "low trust".into());
        q.add(make_diff("c", changes.clone()), "low trust".into());

        let promoted = q.promote(&trust);
        assert_eq!(promoted.len(), 1);
        assert!(q.is_empty());
    }

    #[test]
    fn test_no_promote_without_trusted_corroborations() {
        let mut q = Quarantine::new();
        let trust = TrustStore::new();

        let changes = vec![change(0, 0, 0, 0.5), change(1, 1, 1, 0.7)];
        q.add(make_diff("a", changes.clone()), "low trust".into());
        // One corroboration from default trust (0.5) peer
        q.add(make_diff("b", changes.clone()), "low trust".into());

        let promoted = q.promote(&trust);
        // Only 1 corroboration, need 2
        assert!(promoted.is_empty());
        assert_eq!(q.len(), 1);
    }

    #[test]
    fn test_different_diffs_separate() {
        let mut q = Quarantine::new();
        q.add(
            make_diff("a", vec![change(0, 0, 0, 0.5)]),
            "reason".into(),
        );
        q.add(
            make_diff("b", vec![change(5, 5, 2, 0.9)]),
            "reason".into(),
        );
        assert_eq!(q.len(), 2);
    }

    #[test]
    fn test_similarity() {
        let a = make_diff("a", vec![change(0, 0, 0, 0.5), change(1, 1, 1, 0.7)]);
        let b = make_diff("b", vec![change(0, 0, 0, 0.5), change(1, 1, 1, 0.7)]);
        assert!(diffs_similar(&a, &b) >= SIMILARITY_THRESHOLD);

        let c = make_diff("c", vec![change(5, 5, 2, 0.1)]);
        assert!(diffs_similar(&a, &c) < SIMILARITY_THRESHOLD);
    }
}
