//! Text Store
//!
//! Simple key-value store mapping grid coordinates to original text.
//! Persisted alongside brain.bin as text_store.bin.
//! LRU eviction at ~10MB to keep memory bounded.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Maximum total text bytes before LRU eviction kicks in (~10MB)
const MAX_STORE_BYTES: usize = 10 * 1024 * 1024;

/// Key for the text store: grid coordinates
#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct GridKey {
    pub x: usize,
    pub y: usize,
}

/// Entry in the text store
#[derive(Clone, Debug, Serialize, Deserialize)]
struct TextEntry {
    pub text: String,
    /// Monotonically increasing access counter for LRU
    pub last_access: u64,
}

/// Text store mapping grid coordinates to original text
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

    /// Store text at a grid coordinate
    pub fn insert(&mut self, x: usize, y: usize, text: String) {
        let key = GridKey { x, y };
        let text_len = text.len();

        // Remove old entry if exists
        if let Some(old) = self.entries.remove(&key) {
            self.total_bytes = self.total_bytes.saturating_sub(old.text.len());
        }

        self.access_counter += 1;
        self.entries.insert(
            key,
            TextEntry {
                text,
                last_access: self.access_counter,
            },
        );
        self.total_bytes += text_len;

        // Evict LRU entries if over budget
        while self.total_bytes > MAX_STORE_BYTES && !self.entries.is_empty() {
            self.evict_lru();
        }
    }

    /// Retrieve text at a grid coordinate
    pub fn get(&mut self, x: usize, y: usize) -> Option<&str> {
        let key = GridKey { x, y };
        self.access_counter += 1;
        let counter = self.access_counter;
        if let Some(entry) = self.entries.get_mut(&key) {
            entry.last_access = counter;
            Some(&entry.text)
        } else {
            None
        }
    }

    /// Retrieve text without updating LRU (for read-only queries)
    pub fn peek(&self, x: usize, y: usize) -> Option<&str> {
        let key = GridKey { x, y };
        self.entries.get(&key).map(|e| e.text.as_str())
    }

    /// Number of entries
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if store is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Total bytes stored
    pub fn total_bytes(&self) -> usize {
        self.total_bytes
    }

    /// Save to disk
    pub fn save(&self, path: &str) -> Result<(), String> {
        let data =
            bincode::serialize(self).map_err(|e| format!("TextStore serialize error: {}", e))?;
        if let Some(parent) = std::path::Path::new(path).parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("Dir creation error: {}", e))?;
        }
        std::fs::write(path, &data).map_err(|e| format!("TextStore write error: {}", e))?;
        Ok(())
    }

    /// Load from disk
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
                self.total_bytes = self.total_bytes.saturating_sub(removed.text.len());
            }
        }
    }
}

/// Get the default text store persistence path
pub fn default_text_store_path() -> String {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    format!("{}/.sage/text_store.bin", home)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_get() {
        let mut store = TextStore::new();
        store.insert(5, 10, "hello world".into());
        assert_eq!(store.peek(5, 10), Some("hello world"));
        assert_eq!(store.peek(0, 0), None);
    }

    #[test]
    fn test_overwrite() {
        let mut store = TextStore::new();
        store.insert(1, 1, "first".into());
        store.insert(1, 1, "second".into());
        assert_eq!(store.peek(1, 1), Some("second"));
        assert_eq!(store.len(), 1);
    }

    #[test]
    fn test_save_and_load() {
        let mut store = TextStore::new();
        store.insert(3, 7, "persistent text".into());
        let path = format!(
            "/tmp/sage_test_text_store_{:?}.bin",
            std::thread::current().id()
        );
        store.save(&path).unwrap();
        let loaded = TextStore::load(&path).unwrap();
        assert_eq!(loaded.peek(3, 7), Some("persistent text"));
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_lru_get_updates_access() {
        let mut store = TextStore::new();
        store.insert(0, 0, "a".into());
        store.insert(1, 1, "b".into());
        // Access (0,0) to make it more recent
        let _ = store.get(0, 0);
        assert_eq!(store.len(), 2);
    }
}
