//! Distributed Knowledge Module
//!
//! NCA-based knowledge storage for decentralized AI. The NCA grid IS the knowledge store.
//! Each node runs locally, learns from interactions, and shares knowledge across a network.
//!
//! ## Channel Layout (38 total)
//! - Channels 0-15: Self-organization (existing NCA behavior)
//! - Channels 16-19: Pattern conditioning (one-hot)
//! - Channels 20-21: Environment (food, toxin)
//! - Channels 22-25: Memory (attention/gate/value/recency)
//! - Channels 26-33: Knowledge (6 embedding slots + activation + confidence)
//! - Channels 34-35: Communication (sync state + node ID)
//! - Channels 36-37: Metadata (legacy timestamp + confidence)

pub mod attention_decoder;
pub mod decoder;
pub mod encoder;
pub mod text_store;

use crate::grid::{
    Grid, COMM_NODE_ID, COMM_SYNC_STATE, GRID_SIZE, KNOWLEDGE_ACTIVATION, KNOWLEDGE_CHANNELS_START,
    KNOWLEDGE_CONFIDENCE, NUM_CHANNELS, NUM_KNOWLEDGE_CHANNELS,
};
use decoder::{scan_active_knowledge, KnowledgeActivation};
use encoder::{encode_text, write_knowledge, EncoderConfig};
use serde::{Deserialize, Serialize};
pub use text_store::{default_text_store_path, TextStore};

// ── Brain file versioning ──────────────────────────────────────────────────

/// Current brain file schema version.
/// v1: raw grid, no header
/// v2: BrainHeader + grid (basic versioning)
/// v3: channel partitioning (24 shared + 8 private)
pub const BRAIN_VERSION: u32 = 3;
/// Magic bytes identifying a SAGE brain file.
pub const BRAIN_MAGIC: [u8; 4] = *b"SAGE";

/// Header written at the start of every brain.bin file.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BrainHeader {
    /// Magic bytes – must be `b"SAGE"`.
    pub magic: [u8; 4],
    /// Schema version (currently 2).
    pub version: u32,
    /// Grid width/height when this file was written.
    pub grid_size: u32,
    /// Number of channels per cell.
    pub channels: u32,
    /// Unix timestamp (seconds) when the brain was first created.
    pub created_at: u64,
    /// 32-byte node identity (SHA-256 or random).
    pub node_id: [u8; 32],
}

impl BrainHeader {
    /// Build a new header for the current runtime parameters.
    pub fn new(grid_size: usize, channels: usize, node_id: [u8; 32]) -> Self {
        let created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        Self {
            magic: BRAIN_MAGIC,
            version: BRAIN_VERSION,
            grid_size: grid_size as u32,
            channels: channels as u32,
            created_at,
            node_id,
        }
    }

    /// Byte-size of the serialized header (bincode, fixed).
    pub fn serialized_size() -> usize {
        // 4 + 4 + 4 + 4 + 8 + 32 = 56, but bincode adds length prefixes
        // We use bincode::serialized_size for accuracy.
        bincode::serialized_size(&BrainHeader {
            magic: BRAIN_MAGIC,
            version: 0,
            grid_size: 0,
            channels: 0,
            created_at: 0,
            node_id: [0u8; 32],
        })
        .unwrap_or(64) as usize
    }
}

/// Migrate a v1 grid (assumed 32×32) to a v2 grid of `target_size`.
/// Uses bilinear interpolation to expand cell values.
pub fn migrate_v1_to_v2(old_grid: &Grid, target_size: usize) -> Grid {
    let old_w = old_grid.width;
    let old_h = old_grid.height;
    let mut new_grid = Grid::new(target_size, target_size);

    for ny in 0..target_size {
        for nx in 0..target_size {
            // Map new coords back to old grid (normalized)
            let src_x = (nx as f64 / target_size as f64) * old_w as f64;
            let src_y = (ny as f64 / target_size as f64) * old_h as f64;

            let x0 = (src_x.floor() as usize).min(old_w.saturating_sub(1));
            let y0 = (src_y.floor() as usize).min(old_h.saturating_sub(1));
            let x1 = (x0 + 1).min(old_w.saturating_sub(1));
            let y1 = (y0 + 1).min(old_h.saturating_sub(1));

            let fx = src_x - x0 as f64;
            let fy = src_y - y0 as f64;

            let num_ch = old_grid.cells[0][0].len().min(new_grid.cells[0][0].len());
            for ch in 0..num_ch {
                let v00 = old_grid.cells[y0][x0][ch];
                let v10 = old_grid.cells[y0][x1][ch];
                let v01 = old_grid.cells[y1][x0][ch];
                let v11 = old_grid.cells[y1][x1][ch];
                let val = v00 * (1.0 - fx) * (1.0 - fy)
                    + v10 * fx * (1.0 - fy)
                    + v01 * (1.0 - fx) * fy
                    + v11 * fx * fy;
                new_grid.cells[ny][nx][ch] = val;
            }
        }
    }
    new_grid
}

/// Trait for knowledge storage operations on an NCA grid
pub trait KnowledgeStore {
    /// Encode text into the grid's knowledge channels
    fn encode(&mut self, text: &str, confidence: f64) -> (usize, usize);

    /// Query the grid for knowledge matching the given text
    fn query(&self, query: &str, max_results: usize) -> Vec<KnowledgeActivation>;

    /// Merge another grid's knowledge into this one
    fn merge(&mut self, other: &Grid, merge_strength: f64);

    /// Compute a delta between this grid and another (for sync)
    fn diff(&self, other: &Grid) -> GridDelta;

    /// Apply a delta to this grid
    fn apply_delta(&mut self, delta: &GridDelta);

    /// Save grid state to disk
    fn save(&self, path: &str) -> Result<(), String>;

    /// Load grid state from disk
    fn load(&mut self, path: &str) -> Result<(), String>;
}

/// Delta between two grid states, for efficient network sync
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GridDelta {
    /// Changed cells: (x, y, channel_values for knowledge/comm/meta channels only)
    pub changes: Vec<CellDelta>,
    /// Source node identifier
    pub source_node: f64,
    /// Timestamp of this delta
    pub timestamp: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CellDelta {
    pub x: usize,
    pub y: usize,
    /// Values for knowledge channels (26-33) + comm channels (34-35) = 10 total
    pub values: [f64; 10],
}

/// NCA-based knowledge store implementation
pub struct NCAKnowledge {
    pub grid: Grid,
    pub config: EncoderConfig,
    pub node_id: f64,
    pub text_store: TextStore,
    timestamp_counter: f64,
}

impl Default for NCAKnowledge {
    fn default() -> Self {
        Self::new()
    }
}

impl NCAKnowledge {
    pub fn new() -> Self {
        Self {
            grid: Grid::new(GRID_SIZE, GRID_SIZE),
            config: EncoderConfig::default(),
            node_id: 0.0,
            text_store: TextStore::new(),
            timestamp_counter: 0.0,
        }
    }

    pub fn with_node_id(mut self, node_id: f64) -> Self {
        self.node_id = node_id;
        self
    }

    pub fn with_grid(mut self, grid: Grid) -> Self {
        self.grid = grid;
        self
    }

    fn next_timestamp(&mut self) -> f64 {
        self.timestamp_counter += 0.001;
        // Normalize to [0, 1] using sigmoid-like wrapping
        self.timestamp_counter % 1.0
    }

    /// Get all active knowledge cells above a threshold
    pub fn active_knowledge(&self, min_activation: f64) -> Vec<KnowledgeActivation> {
        scan_active_knowledge(&self.grid, min_activation)
    }

    /// Decay all knowledge activations (call periodically)
    pub fn decay_knowledge(&mut self, decay_rate: f64) {
        let factor = 1.0 - decay_rate.clamp(0.0, 1.0);
        for y in 0..self.grid.height {
            for x in 0..self.grid.width {
                self.grid.cells[y][x][KNOWLEDGE_ACTIVATION] *= factor;
                // Don't decay embedding or metadata — those persist
            }
        }
    }

    /// Run freerun repair steps on a region of the grid.
    ///
    /// Based on rNCA (Silbernagel et al., 2025): NCA self-repair dynamics
    /// consolidate knowledge and prevent semantic drift after encoding.
    ///
    /// This lets the grid "settle" its activation patterns via local
    /// communication rules before the next read. Only touches hidden
    /// channels (4..16); knowledge channels are preserved.
    ///
    /// # Arguments
    /// * `center` - (x, y) coordinates of the repair region center
    /// * `steps` - Number of freerun repair steps to run
    pub fn freerun_repair(&mut self, center: (usize, usize), steps: usize) {
        const DEFAULT_RADIUS: usize = 8;
        self.grid
            .freerun_repair(center.0, center.1, DEFAULT_RADIUS, steps);
    }
}

impl KnowledgeStore for NCAKnowledge {
    fn encode(&mut self, text: &str, confidence: f64) -> (usize, usize) {
        let features = encode_text(text, &self.config);
        let ts = self.next_timestamp();
        let pos = write_knowledge(&mut self.grid, &features, confidence, ts, &self.config);
        // Store original text for later retrieval
        self.text_store.insert(pos.0, pos.1, text.to_string());
        pos
    }

    fn query(&self, query: &str, max_results: usize) -> Vec<KnowledgeActivation> {
        decoder::query_knowledge_with_text(
            &self.grid,
            query,
            &self.config,
            max_results,
            Some(&self.text_store),
        )
    }

    fn merge(&mut self, other: &Grid, merge_strength: f64) {
        let s = merge_strength.clamp(0.0, 1.0);
        for y in 0..self.grid.height.min(other.height) {
            for x in 0..self.grid.width.min(other.width) {
                // For knowledge channels: take the max activation (union merge)
                let other_act = other.cells[y][x][KNOWLEDGE_ACTIVATION];
                let self_act = self.grid.cells[y][x][KNOWLEDGE_ACTIVATION];

                if other_act > self_act {
                    // Blend all embedding slots
                    for slot in 0..NUM_KNOWLEDGE_CHANNELS {
                        let ch = KNOWLEDGE_CHANNELS_START + slot;
                        self.grid.cells[y][x][ch] = self.grid.cells[y][x][ch] * (1.0 - s)
                            + other.cells[y][x][ch] * s;
                    }
                    // Activation
                    self.grid.cells[y][x][KNOWLEDGE_ACTIVATION] =
                        self_act * (1.0 - s) + other_act * s;
                    // Confidence
                    self.grid.cells[y][x][KNOWLEDGE_CONFIDENCE] = self.grid.cells[y][x]
                        [KNOWLEDGE_CONFIDENCE]
                        .max(other.cells[y][x][KNOWLEDGE_CONFIDENCE] * s);
                }
            }
        }
    }

    fn diff(&self, other: &Grid) -> GridDelta {
        let mut changes = Vec::new();
        let threshold = 0.01;

        // channels: all NUM_KNOWLEDGE_CHANNELS knowledge + 2 comm = 10
        let mut channels = [0usize; 10];
        for i in 0..NUM_KNOWLEDGE_CHANNELS {
            channels[i] = KNOWLEDGE_CHANNELS_START + i;
        }
        channels[NUM_KNOWLEDGE_CHANNELS] = COMM_SYNC_STATE;
        channels[NUM_KNOWLEDGE_CHANNELS + 1] = COMM_NODE_ID;

        for y in 0..self.grid.height.min(other.height) {
            for x in 0..self.grid.width.min(other.width) {
                let mut has_diff = false;
                let mut values = [0.0f64; 10];
                for (i, &ch) in channels.iter().enumerate() {
                    let diff = self.grid.cells[y][x][ch] - other.cells[y][x][ch];
                    if diff.abs() > threshold {
                        has_diff = true;
                    }
                    values[i] = self.grid.cells[y][x][ch];
                }

                if has_diff {
                    changes.push(CellDelta { x, y, values });
                }
            }
        }

        GridDelta {
            changes,
            source_node: self.node_id,
            timestamp: self.timestamp_counter,
        }
    }

    fn apply_delta(&mut self, delta: &GridDelta) {
        let mut channels = [0usize; 10];
        for i in 0..NUM_KNOWLEDGE_CHANNELS {
            channels[i] = KNOWLEDGE_CHANNELS_START + i;
        }
        channels[NUM_KNOWLEDGE_CHANNELS] = COMM_SYNC_STATE;
        channels[NUM_KNOWLEDGE_CHANNELS + 1] = COMM_NODE_ID;

        // Index of KNOWLEDGE_ACTIVATION within our channels array
        let act_idx = KNOWLEDGE_ACTIVATION - KNOWLEDGE_CHANNELS_START;

        for change in &delta.changes {
            if change.x < self.grid.width && change.y < self.grid.height {
                for (i, &ch) in channels.iter().enumerate() {
                    let incoming = change.values[i];
                    let current = self.grid.cells[change.y][change.x][ch];

                    // For activation: take max; for others: take incoming if activation is higher
                    if ch == KNOWLEDGE_ACTIVATION {
                        self.grid.cells[change.y][change.x][ch] = current.max(incoming);
                    } else if change.values[act_idx]
                        > self.grid.cells[change.y][change.x][KNOWLEDGE_ACTIVATION]
                    {
                        self.grid.cells[change.y][change.x][ch] = incoming;
                    }
                }
                // Mark the source node
                self.grid.cells[change.y][change.x][COMM_NODE_ID] = delta.source_node;
            }
        }
    }

    fn save(&self, path: &str) -> Result<(), String> {
        // Build header
        let node_id_bytes = {
            let mut buf = [0u8; 32];
            let bytes = self.node_id.to_le_bytes();
            buf[..bytes.len()].copy_from_slice(&bytes);
            buf
        };
        let header = BrainHeader::new(self.grid.width, NUM_CHANNELS, node_id_bytes);
        let header_data = bincode::serialize(&header)
            .map_err(|e| format!("Header serialization error: {}", e))?;
        let grid_data =
            bincode::serialize(&self.grid).map_err(|e| format!("Serialization error: {}", e))?;

        // Create parent directories
        if let Some(parent) = std::path::Path::new(path).parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("Dir creation error: {}", e))?;
        }

        let mut data = Vec::with_capacity(header_data.len() + grid_data.len());
        data.extend_from_slice(&header_data);
        data.extend_from_slice(&grid_data);
        std::fs::write(path, &data).map_err(|e| format!("Write error: {}", e))?;

        // Save text store alongside brain
        let text_store_path = text_store_path_for(path);
        self.text_store
            .save(&text_store_path)
            .unwrap_or_else(|e| eprintln!("Warning: failed to save text store: {}", e));

        Ok(())
    }

    fn load(&mut self, path: &str) -> Result<(), String> {
        let data = std::fs::read(path).map_err(|e| format!("Read error: {}", e))?;

        // Try to read header
        let header_size = BrainHeader::serialized_size();
        if data.len() > header_size {
            if let Ok(header) = bincode::deserialize::<BrainHeader>(&data[..header_size]) {
                if header.magic == BRAIN_MAGIC {
                    // Versioned file — deserialize grid from after header
                    let grid: Grid = bincode::deserialize(&data[header_size..])
                        .map_err(|e| format!("Grid deserialization error: {}", e))?;

                    if header.version < BRAIN_VERSION
                        || (header.grid_size as usize) < self.grid.width
                    {
                        // Need migration
                        eprintln!(
                            "Migrating brain v{} ({}×{}) → v{} ({}×{})",
                            header.version,
                            header.grid_size,
                            header.grid_size,
                            BRAIN_VERSION,
                            self.grid.width,
                            self.grid.width
                        );
                        self.grid = migrate_v1_to_v2(&grid, self.grid.width);
                    } else {
                        self.grid = grid;
                    }

                    // Load text store if it exists
                    let text_store_path = text_store_path_for(path);
                    if let Ok(ts) = TextStore::load(&text_store_path) {
                        self.text_store = ts;
                    }
                    return Ok(());
                }
            }
        }

        // Fallback: legacy v1 file (no header)
        let grid: Grid =
            bincode::deserialize(&data).map_err(|e| format!("Deserialization error: {}", e))?;

        if grid.width != self.grid.width || grid.height != self.grid.height {
            eprintln!(
                "Migrating legacy brain ({}×{}) → ({}×{})",
                grid.width, grid.height, self.grid.width, self.grid.height
            );
            self.grid = migrate_v1_to_v2(&grid, self.grid.width);
        } else {
            self.grid = grid;
        }

        // Load text store if it exists
        let text_store_path = text_store_path_for(path);
        if let Ok(ts) = TextStore::load(&text_store_path) {
            self.text_store = ts;
        }

        Ok(())
    }
}

/// Derive the text store path from a brain path.
/// Replaces the filename with `text_store.bin`, or appends `.text_store` if no known suffix.
fn text_store_path_for(brain_path: &str) -> String {
    if brain_path.contains("brain.bin") {
        brain_path.replace("brain.bin", "text_store.bin")
    } else {
        let p = std::path::Path::new(brain_path);
        let parent = p.parent().unwrap_or(std::path::Path::new("."));
        parent.join("text_store.bin").to_string_lossy().to_string()
    }
}

/// Inspect a brain file and return its header info without loading the full grid.
pub fn brain_info(path: &str) -> Result<BrainHeader, String> {
    let data = std::fs::read(path).map_err(|e| format!("Read error: {}", e))?;
    let header_size = BrainHeader::serialized_size();
    if data.len() < header_size {
        return Err("File too small for brain header (possibly legacy v1)".into());
    }
    let header: BrainHeader = bincode::deserialize(&data[..header_size])
        .map_err(|e| format!("Header parse error: {}", e))?;
    if header.magic != BRAIN_MAGIC {
        return Err("Not a SAGE brain file (bad magic bytes, possibly legacy v1)".into());
    }
    Ok(header)
}

/// Get the default brain persistence path
pub fn default_brain_path() -> String {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    format!("{}/.sage/brain.bin", home)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nca_knowledge_encode_and_query() {
        let mut store = NCAKnowledge::new();

        store.encode("rust programming systems", 0.9);
        store.encode("machine learning neural networks", 0.8);

        let results = store.query("rust programming", 5);
        assert!(!results.is_empty(), "Should find encoded knowledge");
    }

    #[test]
    fn test_nca_knowledge_merge() {
        let mut store_a = NCAKnowledge::new().with_node_id(1.0);
        let mut store_b = NCAKnowledge::new().with_node_id(2.0);

        store_a.encode("knowledge from node A", 0.9);
        store_b.encode("knowledge from node B", 0.8);

        let before_active = store_a.active_knowledge(0.01).len();
        store_a.merge(&store_b.grid, 0.8);
        let after_active = store_a.active_knowledge(0.01).len();

        assert!(
            after_active >= before_active,
            "Merge should not decrease active knowledge: before={} after={}",
            before_active,
            after_active
        );
    }

    #[test]
    fn test_nca_knowledge_diff_and_apply() {
        let mut store_a = NCAKnowledge::new().with_node_id(1.0);
        let store_b = NCAKnowledge::new().with_node_id(2.0);

        store_a.encode("some new knowledge", 0.9);

        let delta = store_a.diff(&store_b.grid);
        assert!(
            !delta.changes.is_empty(),
            "Should have changes after encoding"
        );

        let mut store_c = NCAKnowledge::new().with_node_id(3.0);
        store_c.apply_delta(&delta);

        let active = store_c.active_knowledge(0.01);
        assert!(
            !active.is_empty(),
            "Applied delta should create active knowledge"
        );
    }

    #[test]
    fn test_knowledge_decay() {
        let mut store = NCAKnowledge::new();
        store.encode("decaying knowledge", 0.9);

        let before: f64 = store
            .active_knowledge(0.01)
            .iter()
            .map(|k| k.activation)
            .sum();

        store.decay_knowledge(0.5);

        let after: f64 = store
            .active_knowledge(0.01)
            .iter()
            .map(|k| k.activation)
            .sum();

        assert!(after < before, "Decay should reduce activation");
    }

    #[test]
    fn test_save_and_load() {
        let mut store = NCAKnowledge::new();
        store.encode("persistent knowledge", 0.9);

        let path = "/tmp/sage_test_brain.bin";
        store.save(path).expect("Save should succeed");

        let mut loaded = NCAKnowledge::new();
        loaded.load(path).expect("Load should succeed");

        let results = loaded.active_knowledge(0.01);
        assert!(
            !results.is_empty(),
            "Loaded grid should have active knowledge"
        );

        // Cleanup
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn test_default_brain_path() {
        let path = default_brain_path();
        assert!(path.ends_with(".sage/brain.bin"));
    }

    #[test]
    fn test_brain_header_size() {
        let header = BrainHeader::new(256, 32, [0u8; 32]);
        let data = bincode::serialize(&header).unwrap();
        let expected = BrainHeader::serialized_size();
        assert_eq!(
            data.len(),
            expected,
            "serialized_size() must match actual bincode output: actual={}, expected={}",
            data.len(),
            expected
        );
    }

    #[test]
    fn test_brain_header_roundtrip() {
        let header = BrainHeader::new(256, 38, [42u8; 32]);
        assert_eq!(header.magic, BRAIN_MAGIC);
        assert_eq!(header.version, BRAIN_VERSION);
        assert_eq!(header.grid_size, 256);
        assert_eq!(header.channels, 38);

        let data = bincode::serialize(&header).unwrap();
        let restored: BrainHeader = bincode::deserialize(&data).unwrap();
        assert_eq!(restored.magic, BRAIN_MAGIC);
        assert_eq!(restored.version, BRAIN_VERSION);
        assert_eq!(restored.node_id, [42u8; 32]);
    }

    #[test]
    fn test_migrate_v1_to_v2() {
        // Create a small "v1" grid (32×32)
        let mut old_grid = Grid::new(32, 32);
        // Put some knowledge in the center
        old_grid.cells[16][16][KNOWLEDGE_CHANNELS_START] = 0.9; // embedding slot 0
        old_grid.cells[16][16][KNOWLEDGE_ACTIVATION] = 0.8;

        let new_grid = migrate_v1_to_v2(&old_grid, 256);
        assert_eq!(new_grid.width, 256);
        assert_eq!(new_grid.height, 256);

        // The center of the new grid should have interpolated values
        // Check that knowledge was spread via interpolation (center ≈ 128)
        let has_knowledge = (120..136)
            .any(|y| (120..136).any(|x| new_grid.cells[y][x][KNOWLEDGE_ACTIVATION] > 0.01));
        assert!(
            has_knowledge,
            "Migrated grid should have knowledge near center"
        );
    }

    #[test]
    fn test_save_load_with_header() {
        let mut store = NCAKnowledge::new();
        store.encode("versioned knowledge", 0.9);

        let path = "/tmp/sage_test_brain_v2.bin";
        let _ = std::fs::remove_file(path); // clean up from previous runs
        store.save(path).expect("Save with header should succeed");

        // Verify file starts with header
        let data = std::fs::read(path).unwrap();
        let header_size = BrainHeader::serialized_size();
        let header: BrainHeader = bincode::deserialize(&data[..header_size]).unwrap();
        assert_eq!(header.magic, BRAIN_MAGIC);
        assert_eq!(header.version, BRAIN_VERSION);

        // Load it back
        let mut loaded = NCAKnowledge::new();
        loaded.load(path).expect("Load with header should succeed");
        let results = loaded.active_knowledge(0.01);
        assert!(!results.is_empty());

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn test_load_legacy_v1_brain() {
        // Simulate a legacy v1 brain (no header, just raw grid)
        let mut old_grid = Grid::new(32, 32);
        old_grid.cells[16][16][KNOWLEDGE_ACTIVATION] = 0.7;

        let path = "/tmp/sage_test_legacy_v1.bin";
        let data = bincode::serialize(&old_grid).unwrap();
        std::fs::write(path, &data).unwrap();

        // Load into a 256×256 store — should auto-migrate
        let mut store = NCAKnowledge::new(); // default 256×256
        store.load(path).expect("Legacy load should succeed");
        assert_eq!(store.grid.width, 256);
        assert_eq!(store.grid.height, 256);

        let _ = std::fs::remove_file(path);
    }
}
