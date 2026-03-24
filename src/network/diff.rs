//! Knowledge Diff — Sparse grid state diffs with Merkle hashing.
//!
//! Computes, serializes, and applies diffs between grid states.
//! Only changed cells/channels are included (sparse representation).
//!
//! # Signed Diffs
//!
//! Each outgoing `KnowledgeDiff` carries an Ed25519 signature and the sender's
//! public key.  Receivers can call `verify_signature()` before trusting the diff.
//! The signed payload is: `source_node bytes || sequence (8 LE) || state_hash (32)`.
//! This is compact, covers authenticity (node id + sequence) and integrity (hash).

use crate::grid::is_shared_channel;
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};

/// Error variants for `KnowledgeDiff::verify_signature()`.
#[derive(Debug, PartialEq, Eq)]
pub enum SignatureError {
    /// Diff has no signature or public key set.
    Missing,
    /// The signer_public_key bytes are not a valid Ed25519 point.
    InvalidPublicKey,
    /// Signature does not match the payload.
    Invalid,
}

impl std::fmt::Display for SignatureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Missing => write!(f, "signature missing"),
            Self::InvalidPublicKey => write!(f, "invalid signer public key"),
            Self::Invalid => write!(f, "signature verification failed"),
        }
    }
}

/// A single cell-channel change using normalized coordinates (0.0–1.0).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CellChange {
    /// Normalized row coordinate (0.0–1.0). Used for cross-grid-size sync.
    #[serde(default)]
    pub norm_row: f32,
    /// Normalized column coordinate (0.0–1.0). Used for cross-grid-size sync.
    #[serde(default)]
    pub norm_col: f32,
    /// Absolute row in the source grid.
    pub row: usize,
    /// Absolute column in the source grid.
    pub col: usize,
    pub channel: usize,
    pub old_value: f64,
    pub new_value: f64,
}

impl CellChange {
    /// Create a CellChange with absolute coords only (norm coords default to 0.0).
    /// Prefer `KnowledgeDiff::compute` which sets normalized coords automatically.
    pub fn new(row: usize, col: usize, channel: usize, old_value: f64, new_value: f64) -> Self {
        Self {
            norm_row: 0.0,
            norm_col: 0.0,
            row,
            col,
            channel,
            old_value,
            new_value,
        }
    }
}

/// A sparse diff between two grid states.
///
/// Coordinates are stored both as absolute indices (for same-size grids)
/// and as normalized 0.0–1.0 values (for cross-grid-size sync).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeDiff {
    /// The node that produced this diff.
    pub source_node: String,
    /// Monotonic sequence number for ordering.
    pub sequence: u64,
    /// Grid dimensions of the *source* grid that produced this diff.
    pub width: usize,
    pub height: usize,
    pub num_channels: usize,
    /// The actual changes (sparse).
    pub changes: Vec<CellChange>,
    /// Merkle hash of the source grid state AFTER applying this diff.
    pub state_hash: [u8; 32],
    /// Confidence weight (0.0–1.0) — how confident the source is in these changes.
    pub confidence: f64,
    /// Timestamp (unix epoch millis).
    pub timestamp_ms: u64,
    /// Ed25519 public key of the source node (32 bytes), base64-encoded for serde compat.
    /// Set when the diff is signed; absent for legacy/test diffs.
    #[serde(default)]
    pub signer_public_key: Option<Vec<u8>>,
    /// Ed25519 signature over the canonical payload (64 bytes), base64-encoded for serde compat.
    /// Payload: `source_node_bytes || sequence_le8 || state_hash32`.
    #[serde(default)]
    pub signature: Option<Vec<u8>>,
}

impl KnowledgeDiff {
    /// Compute a diff between two grid states (3D: [height][width][channels]).
    /// Only includes cells where the absolute difference exceeds `threshold`.
    pub fn compute(
        old: &[Vec<Vec<f64>>],
        new: &[Vec<Vec<f64>>],
        source_node: String,
        sequence: u64,
        confidence: f64,
        threshold: f64,
    ) -> Self {
        let height = old.len();
        let width = if height > 0 { old[0].len() } else { 0 };
        let num_channels = if height > 0 && width > 0 {
            old[0][0].len()
        } else {
            0
        };

        let mut changes = Vec::new();

        for row in 0..height {
            for col in 0..width {
                for ch in 0..num_channels {
                    // Only sync shared channels — private channels stay local
                    if !is_shared_channel(ch) {
                        continue;
                    }
                    let ov = old[row][col][ch];
                    let nv = new[row][col][ch];
                    if (nv - ov).abs() > threshold {
                        changes.push(CellChange {
                            norm_row: row as f32 / height as f32,
                            norm_col: col as f32 / width as f32,
                            row,
                            col,
                            channel: ch,
                            old_value: ov,
                            new_value: nv,
                        });
                    }
                }
            }
        }

        let state_hash = merkle_hash(new);
        let timestamp_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        Self {
            signer_public_key: None,
            signature: None,
            source_node,
            sequence,
            width,
            height,
            num_channels,
            changes,
            state_hash,
            confidence,
            timestamp_ms,
        }
    }

    /// Resolve the target row/col for a change, using normalized coords
    /// when the local grid differs in size from the source.
    fn resolve_coords(
        &self,
        change: &CellChange,
        local_h: usize,
        local_w: usize,
    ) -> (usize, usize) {
        if self.height == local_h && self.width == local_w {
            (change.row, change.col)
        } else {
            let r = (change.norm_row * local_h as f32) as usize;
            let c = (change.norm_col * local_w as f32) as usize;
            (
                r.min(local_h.saturating_sub(1)),
                c.min(local_w.saturating_sub(1)),
            )
        }
    }

    /// Apply this diff to a grid state using weighted-average merge.
    /// `local_confidence` is the confidence in the current local state.
    /// Supports cross-grid-size sync via normalized coordinates.
    pub fn apply_weighted(&self, grid: &mut [Vec<Vec<f64>>], local_confidence: f64) {
        let total = local_confidence + self.confidence;
        if total <= 0.0 {
            return;
        }
        let remote_weight = self.confidence / total;
        let local_weight = local_confidence / total;

        let local_h = grid.len();
        let local_w = if local_h > 0 { grid[0].len() } else { 0 };

        for change in &self.changes {
            let (r, c) = self.resolve_coords(change, local_h, local_w);
            if r < local_h && c < local_w && change.channel < grid[0][0].len() {
                let current = grid[r][c][change.channel];
                grid[r][c][change.channel] =
                    current * local_weight + change.new_value * remote_weight;
            }
        }
    }

    /// Apply this diff directly (overwrite, no merge).
    /// Supports cross-grid-size sync via normalized coordinates.
    pub fn apply_direct(&self, grid: &mut [Vec<Vec<f64>>]) {
        let local_h = grid.len();
        let local_w = if local_h > 0 { grid[0].len() } else { 0 };

        for change in &self.changes {
            let (r, c) = self.resolve_coords(change, local_h, local_w);
            if r < local_h && c < local_w && change.channel < grid[0][0].len() {
                grid[r][c][change.channel] = change.new_value;
            }
        }
    }

    /// Serialize to binary (bincode + optional compression placeholder).
    pub fn to_bytes(&self) -> Vec<u8> {
        bincode::serialize(self).expect("failed to serialize diff")
    }

    /// Deserialize from binary.
    pub fn from_bytes(data: &[u8]) -> Result<Self, bincode::Error> {
        bincode::deserialize(data)
    }

    /// Build the canonical signed payload for this diff.
    ///
    /// Payload: `source_node_bytes || sequence_le8 || state_hash32`
    /// This covers node identity, ordering, and content integrity.
    pub fn signing_payload(&self) -> Vec<u8> {
        let mut payload = Vec::with_capacity(self.source_node.len() + 8 + 32);
        payload.extend_from_slice(self.source_node.as_bytes());
        payload.extend_from_slice(&self.sequence.to_le_bytes());
        payload.extend_from_slice(&self.state_hash);
        payload
    }

    /// Sign this diff with the given 32-byte Ed25519 seed.
    /// Sets `signer_public_key` and `signature` in place.
    pub fn sign(&mut self, seed: &[u8; 32]) {
        let signing_key = SigningKey::from_bytes(seed);
        let payload = self.signing_payload();
        let sig = signing_key.sign(&payload);
        self.signer_public_key = Some(signing_key.verifying_key().as_bytes().to_vec());
        self.signature = Some(sig.to_bytes().to_vec());
    }

    /// Verify the diff's Ed25519 signature.
    ///
    /// Returns `Ok(())` if the signature is present and valid.
    /// Returns `Err` if the signature is missing, the public key is invalid/wrong length,
    /// or the signature does not match the payload.
    pub fn verify_signature(&self) -> Result<(), SignatureError> {
        let pk_vec = self.signer_public_key.as_ref().ok_or(SignatureError::Missing)?;
        let sig_vec = self.signature.as_ref().ok_or(SignatureError::Missing)?;

        let pk_bytes: [u8; 32] = pk_vec
            .as_slice()
            .try_into()
            .map_err(|_| SignatureError::InvalidPublicKey)?;
        let sig_bytes: [u8; 64] = sig_vec
            .as_slice()
            .try_into()
            .map_err(|_| SignatureError::Invalid)?;

        let vk = VerifyingKey::from_bytes(&pk_bytes).map_err(|_| SignatureError::InvalidPublicKey)?;
        let sig = Signature::from_bytes(&sig_bytes);
        let payload = self.signing_payload();
        vk.verify(&payload, &sig).map_err(|_| SignatureError::Invalid)
    }

    /// Returns true if this diff has no changes.
    pub fn is_empty(&self) -> bool {
        self.changes.is_empty()
    }
}

/// Compute a simple Merkle-like hash of a 3D grid state.
/// Uses XOR-folding — placeholder for a real Merkle tree.
pub fn merkle_hash(grid: &[Vec<Vec<f64>>]) -> [u8; 32] {
    let mut hash = [0u8; 32];
    for (ri, row) in grid.iter().enumerate() {
        for (ci, col) in row.iter().enumerate() {
            for (ch, &val) in col.iter().enumerate() {
                let bytes = val.to_le_bytes();
                let idx = (ri.wrapping_mul(7) ^ ci.wrapping_mul(13) ^ ch.wrapping_mul(31)) % 32;
                for (i, &b) in bytes.iter().enumerate() {
                    hash[(idx + i) % 32] ^= b;
                }
            }
        }
    }
    hash
}

/// Quick check: do two grids have the same Merkle hash?
pub fn grids_match(a: &[Vec<Vec<f64>>], b: &[Vec<Vec<f64>>]) -> bool {
    merkle_hash(a) == merkle_hash(b)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_grid(h: usize, w: usize, ch: usize, fill: f64) -> Vec<Vec<Vec<f64>>> {
        vec![vec![vec![fill; ch]; w]; h]
    }

    #[test]
    fn test_empty_diff_for_identical_grids() {
        let g = make_grid(4, 4, 3, 0.5);
        let diff = KnowledgeDiff::compute(&g, &g, "test".into(), 0, 1.0, 1e-9);
        assert!(diff.is_empty());
    }

    #[test]
    fn test_diff_detects_changes() {
        let old = make_grid(4, 4, 3, 0.0);
        let mut new = make_grid(4, 4, 3, 0.0);
        new[1][2][0] = 1.0;
        new[3][3][2] = -0.5;

        let diff = KnowledgeDiff::compute(&old, &new, "test".into(), 1, 1.0, 1e-9);
        assert_eq!(diff.changes.len(), 2);
    }

    #[test]
    fn test_threshold_filtering() {
        let old = make_grid(2, 2, 1, 0.0);
        let mut new = make_grid(2, 2, 1, 0.0);
        new[0][0][0] = 0.001; // below threshold
        new[1][1][0] = 0.1; // above threshold

        let diff = KnowledgeDiff::compute(&old, &new, "test".into(), 1, 1.0, 0.01);
        assert_eq!(diff.changes.len(), 1);
        assert_eq!(diff.changes[0].row, 1);
    }

    #[test]
    fn test_apply_direct() {
        let old = make_grid(2, 2, 1, 0.0);
        let mut new = make_grid(2, 2, 1, 0.0);
        new[0][1][0] = 5.0;

        let diff = KnowledgeDiff::compute(&old, &new, "test".into(), 1, 1.0, 1e-9);
        let mut target = make_grid(2, 2, 1, 0.0);
        diff.apply_direct(&mut target);
        assert!((target[0][1][0] - 5.0).abs() < 1e-9);
    }

    #[test]
    fn test_apply_weighted() {
        let old = make_grid(2, 2, 1, 0.0);
        let mut new = make_grid(2, 2, 1, 0.0);
        new[0][0][0] = 1.0;

        let diff = KnowledgeDiff::compute(&old, &new, "test".into(), 1, 0.5, 1e-9);
        let mut target = make_grid(2, 2, 1, 0.0);
        target[0][0][0] = 0.0; // local value

        diff.apply_weighted(&mut target, 0.5);
        // (0.0 * 0.5 + 1.0 * 0.5) / 1.0 = 0.5
        assert!((target[0][0][0] - 0.5).abs() < 1e-9);
    }

    // ---- Signature tests ----

    #[test]
    fn test_sign_and_verify() {
        let seed = [42u8; 32];
        let old = make_grid(2, 2, 1, 0.0);
        let mut new = make_grid(2, 2, 1, 0.0);
        new[0][0][0] = 1.0;

        let mut diff = KnowledgeDiff::compute(&old, &new, "node-sign-test".into(), 1, 0.9, 1e-9);
        diff.sign(&seed);

        assert!(diff.signer_public_key.is_some(), "public key should be set after sign()");
        assert!(diff.signature.is_some(), "signature should be set after sign()");
        assert!(
            diff.verify_signature().is_ok(),
            "freshly signed diff must verify"
        );
    }

    #[test]
    fn test_unsigned_diff_returns_missing() {
        let old = make_grid(2, 2, 1, 0.0);
        let new = make_grid(2, 2, 1, 0.5);
        let diff = KnowledgeDiff::compute(&old, &new, "node-x".into(), 0, 1.0, 1e-9);
        assert_eq!(diff.verify_signature(), Err(SignatureError::Missing));
    }

    #[test]
    fn test_tampered_signature_rejected() {
        let seed = [7u8; 32];
        let old = make_grid(2, 2, 1, 0.0);
        let mut new = make_grid(2, 2, 1, 0.0);
        new[1][1][0] = 0.8;

        let mut diff = KnowledgeDiff::compute(&old, &new, "node-tamper".into(), 5, 0.8, 1e-9);
        diff.sign(&seed);

        // Corrupt the signature (first byte XOR flip)
        if let Some(ref mut sig) = diff.signature {
            if let Some(b) = sig.get_mut(0) {
                *b ^= 0xFF;
            }
        }
        assert_eq!(diff.verify_signature(), Err(SignatureError::Invalid));
    }

    #[test]
    fn test_tampered_sequence_rejected() {
        let seed = [99u8; 32];
        let old = make_grid(2, 2, 1, 0.0);
        let mut new = make_grid(2, 2, 1, 0.0);
        new[0][1][0] = 0.5;

        let mut diff = KnowledgeDiff::compute(&old, &new, "node-seq".into(), 10, 1.0, 1e-9);
        diff.sign(&seed);

        // Replay attack: bump the sequence number after signing
        diff.sequence += 1;
        assert_eq!(
            diff.verify_signature(),
            Err(SignatureError::Invalid),
            "mutated sequence should fail verification"
        );
    }

    #[test]
    fn test_sign_survives_serialize_roundtrip() {
        let seed = [55u8; 32];
        let old = make_grid(3, 3, 2, 0.0);
        let mut new = make_grid(3, 3, 2, 0.0);
        new[1][2][1] = 0.3;

        let mut diff = KnowledgeDiff::compute(&old, &new, "node-rt".into(), 7, 0.75, 1e-9);
        diff.sign(&seed);

        let bytes = diff.to_bytes();
        let restored = KnowledgeDiff::from_bytes(&bytes).expect("roundtrip");
        assert_eq!(
            restored.verify_signature(),
            Ok(()),
            "signature must survive bincode roundtrip"
        );
    }

    #[test]
    fn test_serialize_roundtrip() {
        let old = make_grid(2, 2, 2, 0.0);
        let mut new = make_grid(2, 2, 2, 0.0);
        new[1][0][1] = std::f64::consts::PI;

        let diff = KnowledgeDiff::compute(&old, &new, "node-a".into(), 42, 0.9, 1e-9);
        let bytes = diff.to_bytes();
        let restored = KnowledgeDiff::from_bytes(&bytes).unwrap();

        assert_eq!(restored.changes.len(), 1);
        assert_eq!(restored.source_node, "node-a");
        assert_eq!(restored.sequence, 42);
    }

    #[test]
    fn test_merkle_hash_deterministic() {
        let g = make_grid(3, 3, 2, 0.7);
        assert_eq!(merkle_hash(&g), merkle_hash(&g));
    }

    #[test]
    fn test_merkle_hash_differs() {
        let a = make_grid(3, 3, 2, 0.0);
        let mut b = make_grid(3, 3, 2, 0.0);
        b[0][0][0] = 1.0;
        assert_ne!(merkle_hash(&a), merkle_hash(&b));
    }

    #[test]
    fn test_normalized_coords_in_diff() {
        let old = make_grid(4, 4, 1, 0.0);
        let mut new = make_grid(4, 4, 1, 0.0);
        new[2][3][0] = 1.0; // row=2, col=3 in a 4×4 grid

        let diff = KnowledgeDiff::compute(&old, &new, "test".into(), 1, 1.0, 1e-9);
        assert_eq!(diff.changes.len(), 1);

        let change = &diff.changes[0];
        assert!(
            (change.norm_row - 0.5).abs() < 1e-6,
            "norm_row should be 2/4=0.5"
        );
        assert!(
            (change.norm_col - 0.75).abs() < 1e-6,
            "norm_col should be 3/4=0.75"
        );
    }

    #[test]
    fn test_cross_grid_size_apply() {
        // Create diff on a 4×4 grid
        let old = make_grid(4, 4, 1, 0.0);
        let mut new = make_grid(4, 4, 1, 0.0);
        new[2][2][0] = 1.0; // center-ish: norm=(0.5, 0.5)

        let diff = KnowledgeDiff::compute(&old, &new, "test".into(), 1, 1.0, 1e-9);

        // Apply to an 8×8 grid — should map (0.5,0.5) → (4,4)
        let mut target = make_grid(8, 8, 1, 0.0);
        diff.apply_direct(&mut target);

        assert!(
            (target[4][4][0] - 1.0).abs() < 1e-9,
            "Cross-grid apply should map to (4,4) in 8×8, got value at (4,4)={}",
            target[4][4][0]
        );
    }

    #[test]
    fn test_cross_grid_size_weighted() {
        let old = make_grid(8, 8, 1, 0.0);
        let mut new = make_grid(8, 8, 1, 0.0);
        new[4][4][0] = 1.0; // norm=(0.5, 0.5)

        let diff = KnowledgeDiff::compute(&old, &new, "test".into(), 1, 0.5, 1e-9);

        // Apply to a 4×4 grid — should map (0.5,0.5) → (2,2)
        let mut target = make_grid(4, 4, 1, 0.0);
        target[2][2][0] = 0.0;
        diff.apply_weighted(&mut target, 0.5);

        assert!(
            (target[2][2][0] - 0.5).abs() < 1e-9,
            "Weighted cross-grid: expected 0.5, got {}",
            target[2][2][0]
        );
    }
}
