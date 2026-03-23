//! Text Store
//!
//! Maps grid coordinates to original text snippets.
//! A single grid cell can store multiple distinct texts (up to MAX_TEXTS_PER_CELL),
//! because hash collisions naturally cause multiple encoded items to land at or near
//! the same cell. Storing only the last text per cell caused the capacity benchmark
//! to crater: early items were silently evicted by later ones sharing the same position.
//!
//! Persisted alongside brain.bin as text_store.bin.
//! LRU eviction at ~10MB to keep memory bounded.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Maximum total text bytes before LRU eviction kicks in (~10MB)
const MAX_STORE_BYTES: usize = 10 * 1024 * 1024;

/// Maximum distinct texts stored per cell. Keeps memory bounded even if many items
/// collide at the same grid position. Oldest entry is dropped when this limit is reached.
const MAX_TEXTS_PER_CELL: usize = 8;

/// Key for the text store: grid coordinates
#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct GridKey {
    pub x: usize,
    pub y: usize,
}

/// Multi-text entry: all original texts encoded to this cell, plus LRU metadata.
#[derive(Clone, Debug, Serialize, Deserialize)]
struct TextEntry {
    /// Up to MAX_TEXTS_PER_CELL texts, ordered oldest→newest.
    pub texts: Vec<String>,
    /// Monotonically increasing access counter for LRU eviction at the cell level.
    pub last_access: u64,
}

impl TextEntry {
    fn byte_count(&self) -> usize {
        self.texts.iter().map(|t| t.len()).sum()
    }
}

/// Text store mapping grid coordinates to original text(s).
///
/// Multiple texts can coexist at a single (x, y) coordinate because the spatial hash
/// encoder maps many distinct texts to overlapping grid regions.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TextStore {
    entries: HashMap<GridKey, TextEntry>,
    access_counter: u64,
    total_bytes: usize,
}

impl Default for TextStore {
    fn default() -> Self {
        Self::new()
    }
}

impl TextStore {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            access_counter: 0,
            total_bytes: 0,
        }
    }

    /// Store text at a grid coordinate.
    ///
    /// If `text` is already present at this cell (exact match), this is a no-op.
    /// If the cell already has MAX_TEXTS_PER_CELL entries, the oldest is evicted to
    /// make room for the new one.
    pub fn insert(&mut self, x: usize, y: usize, text: String) {
        let key = GridKey { x, y };

        self.access_counter += 1;
        let counter = self.access_counter;

        let entry = self.entries.entry(key).or_insert_with(|| TextEntry {
            texts: Vec::new(),
            last_access: counter,
        });

        // Deduplicate — don't store the same text twice at the same cell
        if entry.texts.iter().any(|t| t == &text) {
            entry.last_access = counter;
            return;
        }

        // Evict oldest text if at capacity
        if entry.texts.len() >= MAX_TEXTS_PER_CELL {
            let evicted = entry.texts.remove(0);
            self.total_bytes = self.total_bytes.saturating_sub(evicted.len());
        }

        self.total_bytes += text.len();
        entry.texts.push(text);
        entry.last_access = counter;

        // Evict LRU cells if over total budget
        while self.total_bytes > MAX_STORE_BYTES && !self.entries.is_empty() {
            self.evict_lru();
        }
    }

    /// Retrieve the most-recently-written text at a grid coordinate (legacy single-text API).
    ///
    /// Prefer `peek_all` when iterating over multiple results in the decoder.
    pub fn get(&mut self, x: usize, y: usize) -> Option<&str> {
        let key = GridKey { x, y };
        self.access_counter += 1;
        let counter = self.access_counter;
        if let Some(entry) = self.entries.get_mut(&key) {
            entry.last_access = counter;
            entry.texts.last().map(|s| s.as_str())
        } else {
            None
        }
    }

    /// Retrieve the most-recently-written text without updating LRU (single-text compat).
    pub fn peek(&self, x: usize, y: usize) -> Option<&str> {
        let key = GridKey { x, y };
        self.entries
            .get(&key)
            .and_then(|e| e.texts.last().map(|s| s.as_str()))
    }

    /// Retrieve ALL texts stored at a grid coordinate, oldest first.
    ///
    /// Returns an empty slice if no texts exist at this position.
    pub fn peek_all(&self, x: usize, y: usize) -> &[String] {
        let key = GridKey { x, y };
        self.entries
            .get(&key)
            .map(|e| e.texts.as_slice())
            .unwrap_or(&[])
    }

    /// Number of distinct cells with stored text.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Total number of text entries across all cells.
    pub fn total_texts(&self) -> usize {
        self.entries.values().map(|e| e.texts.len()).sum()
    }

    /// Total bytes stored.
    pub fn total_bytes(&self) -> usize {
        self.total_bytes
    }

    /// Save to disk.
    pub fn save(&self, path: &str) -> Result<(), String> {
        let data =
            bincode::serialize(self).map_err(|e| format!("TextStore serialize error: {}", e))?;
        if let Some(parent) = std::path::Path::new(path).parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("Dir creation error: {}", e))?;
        }
        std::fs::write(path, &data).map_err(|e| format!("TextStore write error: {}", e))?;
        Ok(())
    }

    /// Load from disk.
    pub fn load(path: &str) -> Result<Self, String> {
        let data = std::fs::read(path).map_err(|e| format!("TextStore read error: {}", e))?;
        bincode::deserialize(&data).map_err(|e| format!("TextStore deserialize error: {}", e))
    }

    fn evict_lru(&mut self) {
        if let Some(lru_key) = self
            .entries
            .iter()
            .min_by_key(|(_, v)| v.last_access)
            .map(|(k, _)| k.clone())
        {
            if let Some(removed) = self.entries.remove(&lru_key) {
                self.total_bytes = self.total_bytes.saturating_sub(removed.byte_count());
            }
        }
    }
}

/// Get the default text store persistence path.
pub fn default_text_store_path() -> String {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    format!("{}/.sage/text_store.bin", home)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_peek() {
        let mut store = TextStore::new();
        store.insert(5, 10, "hello world".into());
        assert_eq!(store.peek(5, 10), Some("hello world"));
        assert_eq!(store.peek(0, 0), None);
    }

    #[test]
    fn test_multi_text_at_same_cell() {
        let mut store = TextStore::new();
        store.insert(1, 1, "first".into());
        store.insert(1, 1, "second".into());
        store.insert(1, 1, "third".into());
        // peek() returns the most recent
        assert_eq!(store.peek(1, 1), Some("third"));
        // peek_all() returns all of them
        let all = store.peek_all(1, 1);
        assert_eq!(all.len(), 3);
        assert!(all.contains(&"first".to_string()));
        assert!(all.contains(&"second".to_string()));
        assert!(all.contains(&"third".to_string()));
        // len() counts cells, not texts
        assert_eq!(store.len(), 1);
        assert_eq!(store.total_texts(), 3);
    }

    #[test]
    fn test_deduplication_within_cell() {
        let mut store = TextStore::new();
        store.insert(2, 3, "duplicate".into());
        store.insert(2, 3, "duplicate".into());
        assert_eq!(store.peek_all(2, 3).len(), 1);
        assert_eq!(store.total_texts(), 1);
    }

    #[test]
    fn test_capacity_cap_evicts_oldest() {
        let mut store = TextStore::new();
        for i in 0..MAX_TEXTS_PER_CELL + 2 {
            store.insert(0, 0, format!("item {}", i));
        }
        // Should never exceed MAX_TEXTS_PER_CELL
        assert_eq!(store.peek_all(0, 0).len(), MAX_TEXTS_PER_CELL);
        // Oldest items should be gone, newest should be present
        let all = store.peek_all(0, 0);
        assert!(!all.contains(&"item 0".to_string()));
        assert!(!all.contains(&"item 1".to_string()));
        assert!(all.contains(&format!("item {}", MAX_TEXTS_PER_CELL + 1)));
    }

    #[test]
    fn test_save_and_load_multi_text() {
        let mut store = TextStore::new();
        store.insert(3, 7, "persistent text A".into());
        store.insert(3, 7, "persistent text B".into());
        let path = format!(
            "/tmp/sage_test_text_store_{:?}.bin",
            std::thread::current().id()
        );
        store.save(&path).unwrap();
        let loaded = TextStore::load(&path).unwrap();
        assert_eq!(loaded.peek(3, 7), Some("persistent text B"));
        assert_eq!(loaded.peek_all(3, 7).len(), 2);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_lru_get_updates_access() {
        let mut store = TextStore::new();
        store.insert(0, 0, "a".into());
        store.insert(1, 1, "b".into());
        let _ = store.get(0, 0);
        assert_eq!(store.len(), 2);
    }

    #[test]
    fn test_peek_all_empty() {
        let store = TextStore::new();
        assert_eq!(store.peek_all(99, 99).len(), 0);
    }
}
