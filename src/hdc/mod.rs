//! HDC Store — Hyperdimensional Computing for Knowledge Storage
//!
//! Full-dimensional vector store for exact semantic retrieval.
//! No grid, no hashing, no dimension reduction. Each entry is a full
//! embedding vector paired with its source text. Retrieval is cosine
//! similarity over all vectors — O(n) but n=10K is trivial.
//!
//! This is the episodic memory store in the SAGE dual-store architecture.
//! The NCA grid handles semantic dynamics; the HDC store handles facts.

use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::Instant;

// Re-export HDC sync types from network module
pub use crate::network::gossip::{HdcSyncBatch, HdcSyncEntry, HdcSyncRequest};

/// A single HDC entry: an embedding vector and its associated text.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HdcEntry {
    /// Full-dimensional embedding (768-dim from nomic, 384-dim from fastembed)
    pub embedding: Vec<f32>,
    /// The source text this embedding represents
    pub text: String,
    /// Confidence score (0.0-1.0), typically from encoding
    pub confidence: f32,
    /// Timestamp for recency weighting (Unix epoch seconds)
    pub timestamp: u64,
}

/// The HDC store: a flat array of entries with fast cosine-similarity retrieval.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HdcStore {
    pub entries: Vec<HdcEntry>,
    /// Dimensionality of the embeddings (768 for nomic, 384 for fastembed)
    pub dim: usize,
    /// Precomputed magnitudes for fast cosine sim
    #[serde(skip)]
    magnitudes: Vec<f32>,
}

impl HdcStore {
    /// Create a new empty HDC store with the given embedding dimension
    pub fn new(dim: usize) -> Self {
        Self {
            entries: Vec::new(),
            dim,
            magnitudes: Vec::new(),
        }
    }

    /// Insert a new entry. Optionally dedup against existing entries
    /// (skip if cosine similarity > threshold to any existing entry).
    pub fn insert(&mut self, embedding: &[f32], text: &str, confidence: f32) -> bool {
        if embedding.len() != self.dim {
            // Auto-resize dimension on first insert
            if self.entries.is_empty() {
                self.dim = embedding.len();
            } else {
                return false; // Dimension mismatch
            }
        }

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let entry = HdcEntry {
            embedding: embedding.to_vec(),
            text: text.to_string(),
            confidence,
            timestamp,
        };

        self.magnitudes.push(magnitude(embedding));
        self.entries.push(entry);
        true
    }

    /// Insert with deduplication — skip if too similar to existing entries.
    pub fn insert_dedup(&mut self, embedding: &[f32], text: &str, confidence: f32, threshold: f32) -> bool {
        if !self.entries.is_empty() && embedding.len() == self.dim {
            // Quick check: is this too similar to anything already stored?
            let query_mag = magnitude(embedding);
            for (i, entry) in self.entries.iter().enumerate() {
                let cos = cosine_similarity(embedding, query_mag, &entry.embedding, self.magnitudes[i]);
                if cos > threshold {
                    return false; // Duplicate, skip
                }
            }
        }
        self.insert(embedding, text, confidence)
    }

    /// Find the most similar existing entry. Returns (cosine_similarity, text) or None.
    pub fn find_most_similar(&self, embedding: &[f32]) -> Option<(f32, &str)> {
        if self.entries.is_empty() || embedding.len() != self.dim {
            return None;
        }
        let query_mag = magnitude(embedding);
        let mut best_cos = -1.0f32;
        let mut best_idx = 0;
        for (i, entry) in self.entries.iter().enumerate() {
            let cos = cosine_similarity(embedding, query_mag, &entry.embedding, self.magnitudes[i]);
            if cos > best_cos {
                best_cos = cos;
                best_idx = i;
            }
        }
        Some((best_cos, self.entries[best_idx].text.as_str()))
    }

    /// Query the store for the top-K most similar entries.
    /// Returns (cosine_similarity, text) pairs sorted by similarity descending.
    pub fn query(&self, embedding: &[f32], k: usize) -> Vec<(f32, &str)> {
        if self.entries.is_empty() || embedding.len() != self.dim {
            return Vec::new();
        }

        let query_mag = magnitude(embedding);

        // Compute all similarities
        let mut scored: Vec<(f32, usize)> = self
            .entries
            .iter()
            .enumerate()
            .map(|(i, entry)| {
                let cos = cosine_similarity(embedding, query_mag, &entry.embedding, self.magnitudes[i]);
                (cos, i)
            })
            .collect();

        // Sort by similarity descending
        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(k);

        scored
            .into_iter()
            .map(|(cos, i)| (cos, self.entries[i].text.as_str()))
            .collect()
    }

    /// Query with full metadata (confidence, timestamp, position)
    pub fn query_detailed(&self, embedding: &[f32], k: usize) -> Vec<HdcQueryResult> {
        if self.entries.is_empty() || embedding.len() != self.dim {
            return Vec::new();
        }

        let query_mag = magnitude(embedding);

        let mut scored: Vec<HdcQueryResult> = self
            .entries
            .iter()
            .enumerate()
            .map(|(i, entry)| {
                let cos = cosine_similarity(embedding, query_mag, &entry.embedding, self.magnitudes[i]);
                HdcQueryResult {
                    relevance: cos,
                    text: entry.text.as_str(),
                    confidence: entry.confidence,
                    timestamp: entry.timestamp,
                    index: i,
                }
            })
            .collect();

        scored.sort_by(|a, b| b.relevance.partial_cmp(&a.relevance).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(k);
        scored
    }

    /// Number of entries in the store
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Is the store empty?
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Memory usage in bytes (approximate)
    pub fn memory_bytes(&self) -> usize {
        self.entries.len() * (self.dim * 4 + 200) // f32 * dim + ~200 bytes for text + metadata
    }

    /// Save to a binary file using bincode
    pub fn save(&self, path: &Path) -> Result<(), String> {
        // Recompute magnitudes (they're skipped in serde)
        let mut to_save = self.clone();
        to_save.magnitudes = to_save.entries.iter().map(|e| magnitude(&e.embedding)).collect();
        
        let data = bincode::serialize(&to_save).map_err(|e| format!("Serialize error: {}", e))?;
        std::fs::write(path, &data).map_err(|e| format!("Write error: {}", e))?;
        Ok(())
    }

    /// Load from a binary file
    pub fn load(path: &Path) -> Result<Self, String> {
        let data = std::fs::read(path).map_err(|e| format!("Read error: {}", e))?;
        let mut store: Self = bincode::deserialize(&data).map_err(|e| format!("Deserialize error: {}", e))?;
        // Recompute magnitudes
        store.magnitudes = store.entries.iter().map(|e| magnitude(&e.embedding)).collect();
        Ok(store)
    }

    /// Get all texts (for dedup checking, export, etc.)
    pub fn texts(&self) -> Vec<&str> {
        self.entries.iter().map(|e| e.text.as_str()).collect()
    }

    /// Clear all entries
    pub fn clear(&mut self) {
        self.entries.clear();
        self.magnitudes.clear();
    }

    /// Merge another store into this one (with optional dedup)
    pub fn merge(&mut self, other: &HdcStore, threshold: f32) -> usize {
        let mut merged = 0;
        for entry in &other.entries {
            if entry.embedding.len() == self.dim {
                if self.insert_dedup(&entry.embedding, &entry.text, entry.confidence, threshold) {
                    merged += 1;
                }
            }
        }
        merged
    }

    /// Create a batch of HDC entries for network sync.
    ///
    /// Returns up to `max_entries` entries starting from `since_index`.
    /// Only includes entries with the correct dimension.
    pub fn create_sync_batch(
        &self,
        source_node: &str,
        since_index: usize,
        max_entries: usize,
        sequence: u64,
    ) -> HdcSyncBatch {
        let entries: Vec<HdcSyncEntry> = self
            .entries
            .iter()
            .skip(since_index)
            .take(max_entries)
            .map(|e| HdcSyncEntry {
                embedding: e.embedding.clone(),
                text: e.text.clone(),
                confidence: e.confidence,
                timestamp: e.timestamp,
            })
            .collect();

        let timestamp_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        HdcSyncBatch {
            source_node: source_node.to_string(),
            dim: self.dim,
            entries,
            sequence,
            timestamp_ms,
        }
    }

    /// Create a sync batch of entries newer than the given timestamp.
    pub fn create_sync_batch_since(
        &self,
        source_node: &str,
        since_timestamp: u64,
        max_entries: usize,
        sequence: u64,
    ) -> HdcSyncBatch {
        let entries: Vec<HdcSyncEntry> = self
            .entries
            .iter()
            .filter(|e| e.timestamp > since_timestamp)
            .take(max_entries)
            .map(|e| HdcSyncEntry {
                embedding: e.embedding.clone(),
                text: e.text.clone(),
                confidence: e.confidence,
                timestamp: e.timestamp,
            })
            .collect();

        let timestamp_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        HdcSyncBatch {
            source_node: source_node.to_string(),
            dim: self.dim,
            entries,
            sequence,
            timestamp_ms,
        }
    }

    /// Merge a received HDC sync batch into this store.
    ///
    /// Uses deduplication to avoid storing duplicate entries.
    /// Returns the number of new entries actually merged.
    pub fn merge_sync_batch(&mut self, batch: &HdcSyncBatch, dedup_threshold: f32) -> usize {
        // Dimension must match (or store must be empty for first sync)
        if self.entries.is_empty() && self.dim == 0 {
            self.dim = batch.dim;
        }
        if batch.dim != self.dim {
            return 0;
        }

        let mut merged = 0;
        for entry in &batch.entries {
            if self.insert_dedup(&entry.embedding, &entry.text, entry.confidence, dedup_threshold) {
                merged += 1;
            }
        }
        merged
    }

    /// Get the latest timestamp in the store (for incremental sync).
    pub fn latest_timestamp(&self) -> u64 {
        self.entries
            .iter()
            .map(|e| e.timestamp)
            .max()
            .unwrap_or(0)
    }
}

/// A query result with full metadata
#[derive(Debug, Clone)]
pub struct HdcQueryResult<'a> {
    pub relevance: f32,
    pub text: &'a str,
    pub confidence: f32,
    pub timestamp: u64,
    pub index: usize,
}

/// Compute the L2 magnitude of a vector
#[inline]
fn magnitude(v: &[f32]) -> f32 {
    v.iter().map(|x| x * x).sum::<f32>().sqrt()
}

/// Cosine similarity between two vectors (with precomputed magnitudes)
#[inline]
fn cosine_similarity(a: &[f32], a_mag: f32, b: &[f32], b_mag: f32) -> f32 {
    if a_mag < 1e-10 || b_mag < 1e-10 {
        return 0.0;
    }
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    (dot / (a_mag * b_mag)).clamp(-1.0, 1.0)
}

/// Default HDC store path
pub fn default_hdc_path() -> String {
    dirs::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("~"))
        .join(".sage/hdc_store.bin")
        .to_string_lossy()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::gossip::{HdcSyncBatch, HdcSyncEntry};

    #[test]
    fn test_insert_and_query() {
        let mut store = HdcStore::new(3);
        
        store.insert(&[1.0, 0.0, 0.0], "red", 1.0);
        store.insert(&[0.0, 1.0, 0.0], "green", 1.0);
        store.insert(&[0.0, 0.0, 1.0], "blue", 1.0);
        
        assert_eq!(store.len(), 3);
        
        // Query for "reddish" should find "red" first
        let results = store.query(&[0.9, 0.1, 0.0], 1);
        assert_eq!(results.len(), 1);
        assert!((results[0].0 - 0.9934).abs() < 0.01);
        assert_eq!(results[0].1, "red");
    }

    #[test]
    fn test_dedup() {
        let mut store = HdcStore::new(3);
        
        store.insert(&[1.0, 0.0, 0.0], "red item 1", 1.0);
        
        // Very similar — should be rejected
        let inserted = store.insert_dedup(&[0.99, 0.01, 0.0], "red item 2", 1.0, 0.98);
        assert!(!inserted);
        assert_eq!(store.len(), 1);
        
        // Different — should be accepted
        let inserted = store.insert_dedup(&[0.0, 1.0, 0.0], "green", 1.0, 0.98);
        assert!(inserted);
        assert_eq!(store.len(), 2);
    }

    #[test]
    fn test_save_load() {
        let mut store = HdcStore::new(3);
        store.insert(&[1.0, 0.0, 0.0], "red", 0.9);
        store.insert(&[0.0, 1.0, 0.0], "green", 0.8);
        
        let tmp = std::env::temp_dir().join("sage_hdc_test.bin");
        store.save(&tmp).unwrap();
        
        let loaded = HdcStore::load(&tmp).unwrap();
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded.dim, 3);
        
        // Query still works after load
        let results = loaded.query(&[1.0, 0.0, 0.0], 1);
        assert_eq!(results[0].1, "red");
        
        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn test_empty_query() {
        let store = HdcStore::new(128);
        let results = store.query(&[0.0; 128], 5);
        assert!(results.is_empty());
    }

    #[test]
    fn test_large_store() {
        // Simulate 10K entries
        let mut store = HdcStore::new(768);
        for i in 0..1000 {
            let mut emb = vec![0.0f32; 768];
            emb[i % 768] = 1.0;
            store.insert(&emb, &format!("entry_{}", i), 1.0);
        }
        
        let start = Instant::now();
        let results = store.query(&[1.0; 768].repeat(1)[..768], 5);
        let elapsed = start.elapsed();
        
        assert_eq!(results.len(), 5);
        // Should be fast — 1K entries in <10ms
        assert!(elapsed.as_millis() < 100, "Query took {}ms", elapsed.as_millis());
    }

    #[test]
    fn test_merge() {
        let mut store_a = HdcStore::new(3);
        store_a.insert(&[1.0, 0.0, 0.0], "red", 1.0);
        
        let mut store_b = HdcStore::new(3);
        store_b.insert(&[0.0, 1.0, 0.0], "green", 1.0);
        store_b.insert(&[1.0, 0.0, 0.0], "red duplicate", 1.0); // Will be deduped
        
        let merged = store_a.merge(&store_b, 0.98);
        assert_eq!(merged, 1); // Only green was new
        assert_eq!(store_a.len(), 2);
    }

    #[test]
    fn test_create_sync_batch() {
        let mut store = HdcStore::new(3);
        store.insert(&[1.0, 0.0, 0.0], "red", 1.0);
        store.insert(&[0.0, 1.0, 0.0], "green", 0.9);
        store.insert(&[0.0, 0.0, 1.0], "blue", 0.8);

        let batch = store.create_sync_batch("node-alice", 0, 10, 1);
        assert_eq!(batch.source_node, "node-alice");
        assert_eq!(batch.dim, 3);
        assert_eq!(batch.entries.len(), 3);
        assert_eq!(batch.sequence, 1);
        assert!(batch.timestamp_ms > 0);
    }

    #[test]
    fn test_create_sync_batch_limit() {
        let mut store = HdcStore::new(3);
        for i in 0..10 {
            let mut emb = vec![0.0f32; 3];
            emb[i % 3] = 1.0;
            store.insert(&emb, &format!("entry_{}", i), 1.0);
        }

        let batch = store.create_sync_batch("node-alice", 0, 5, 0);
        assert_eq!(batch.entries.len(), 5);
    }

    #[test]
    fn test_create_sync_batch_since() {
        let mut store = HdcStore::new(3);
        store.insert(&[1.0, 0.0, 0.0], "old", 1.0);
        // Manually set a high timestamp for the next entry
        store.insert(&[0.0, 1.0, 0.0], "new", 1.0);
        // Set the first entry's timestamp to 0 (simulate old entry)
        store.entries[0].timestamp = 100;
        store.entries[1].timestamp = 200;

        let batch = store.create_sync_batch_since("node-alice", 150, 10, 0);
        assert_eq!(batch.entries.len(), 1);
        assert_eq!(batch.entries[0].text, "new");
    }

    #[test]
    fn test_merge_sync_batch() {
        let mut store_a = HdcStore::new(3);
        store_a.insert(&[1.0, 0.0, 0.0], "red", 1.0);

        let batch = HdcSyncBatch {
            source_node: "node-bob".to_string(),
            dim: 3,
            entries: vec![
                HdcSyncEntry {
                    embedding: vec![0.0, 1.0, 0.0],
                    text: "green".to_string(),
                    confidence: 1.0,
                    timestamp: 100,
                },
                HdcSyncEntry {
                    embedding: vec![1.0, 0.0, 0.0],
                    text: "red duplicate".to_string(),
                    confidence: 1.0,
                    timestamp: 100,
                },
            ],
            sequence: 1,
            timestamp_ms: 1000,
        };

        let merged = store_a.merge_sync_batch(&batch, 0.95);
        assert_eq!(merged, 1); // Only "green" was new
        assert_eq!(store_a.len(), 2);
    }

    #[test]
    fn test_merge_sync_batch_dim_mismatch() {
        let mut store = HdcStore::new(3);
        store.insert(&[1.0, 0.0, 0.0], "red", 1.0);

        let batch = HdcSyncBatch {
            source_node: "node-bob".to_string(),
            dim: 768,
            entries: vec![HdcSyncEntry {
                embedding: vec![0.0; 768],
                text: "wrong dim".to_string(),
                confidence: 1.0,
                timestamp: 100,
            }],
            sequence: 1,
            timestamp_ms: 1000,
        };

        let merged = store.merge_sync_batch(&batch, 0.95);
        assert_eq!(merged, 0); // Dimension mismatch — nothing merged
    }

    #[test]
    fn test_merge_sync_batch_empty_store() {
        let mut store = HdcStore::new(0);

        let batch = HdcSyncBatch {
            source_node: "node-bob".to_string(),
            dim: 3,
            entries: vec![
                HdcSyncEntry {
                    embedding: vec![1.0, 0.0, 0.0],
                    text: "red".to_string(),
                    confidence: 1.0,
                    timestamp: 100,
                },
            ],
            sequence: 1,
            timestamp_ms: 1000,
        };

        let merged = store.merge_sync_batch(&batch, 0.95);
        assert_eq!(merged, 1); // First entry into empty store
        assert_eq!(store.len(), 1);
        assert_eq!(store.dim, 3); // Auto-set from batch
    }

    #[test]
    fn test_latest_timestamp() {
        let mut store = HdcStore::new(3);
        store.insert(&[1.0, 0.0, 0.0], "old", 1.0);
        store.entries[0].timestamp = 100;
        store.insert(&[0.0, 1.0, 0.0], "new", 1.0);
        store.entries[1].timestamp = 200;

        assert_eq!(store.latest_timestamp(), 200);
    }

    #[test]
    fn test_latest_timestamp_empty() {
        let store = HdcStore::new(3);
        assert_eq!(store.latest_timestamp(), 0);
    }
}