//! SAGE Snapshot System
//!
//! Save and restore complete SAGE state including:
//! - NCA grid (all cell values)
//! - NCA generation counter
//! - Network weights
//! - SageExperience (preferences, associations, curiosity)
//!
//! Each snapshot gets a unique hash ID for easy reference.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::nca::NCA;
use crate::sage_experience::SageExperience;

/// Directory for storing snapshots
const SNAPSHOT_DIR: &str = "sage_snapshots";

/// Complete SAGE state snapshot
#[derive(Serialize, Deserialize)]
pub struct SageSnapshot {
    /// Unique hash identifier (first 8 chars of full hash)
    pub hash: String,
    /// Full SHA256 hash
    pub full_hash: String,
    /// Human-readable name (optional)
    pub name: Option<String>,
    /// Description of this snapshot
    pub description: Option<String>,
    /// Unix timestamp when created
    pub timestamp: u64,
    /// NCA generation at time of snapshot
    pub generation: usize,
    /// NCA grid state (flattened cell values)
    pub grid_state: Vec<Vec<f64>>,
    /// Grid dimensions
    pub grid_width: usize,
    pub grid_height: usize,
    /// Network weights (serialized)
    pub network_weights: Option<String>,
    /// SageExperience preferences
    pub preferences: Option<String>,
    /// SageExperience associations
    pub associations: Option<String>,
    /// SageExperience curiosity data
    pub curiosity: Option<String>,
    /// SageExperience knowledge
    pub knowledge: Option<String>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl SageSnapshot {
    /// Create a new snapshot from current SAGE state
    pub fn capture(
        nca: &NCA,
        generation: usize,
        sage_exp: &SageExperience,
        name: Option<String>,
        description: Option<String>,
    ) -> Self {
        // Capture grid state
        let grid_state: Vec<Vec<f64>> = nca.grid.cells
            .iter()
            .map(|row| row.iter().flatten().copied().collect())
            .collect();

        // Get timestamp
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Serialize SageExperience components
        let preferences = sage_exp.export_preferences_json().ok();
        let associations = sage_exp.export_associations_json().ok();
        let curiosity = sage_exp.export_curiosity_json().ok();
        let knowledge = sage_exp.export_knowledge_json().ok();

        // Serialize network weights
        let network_weights = nca.export_weights_json().ok();

        // Build metadata
        let mut metadata = HashMap::new();
        metadata.insert("experience_count".to_string(), sage_exp.experience_count().to_string());

        // Create snapshot without hash first
        let mut snapshot = Self {
            hash: String::new(),
            full_hash: String::new(),
            name,
            description,
            timestamp,
            generation,
            grid_state,
            grid_width: nca.grid.width,
            grid_height: nca.grid.height,
            network_weights,
            preferences,
            associations,
            curiosity,
            knowledge,
            metadata,
        };

        // Calculate hash from content
        let hash_input = format!(
            "{}:{}:{}:{}",
            timestamp,
            generation,
            snapshot.grid_state.len(),
            snapshot.preferences.as_ref().map(|s| s.len()).unwrap_or(0)
        );
        let full_hash = Self::sha256_hex(&hash_input);
        snapshot.full_hash = full_hash.clone();
        snapshot.hash = full_hash[..8].to_string();

        snapshot
    }

    /// Simple SHA256 hash (using basic implementation for no extra deps)
    fn sha256_hex(input: &str) -> String {
        // Simple hash based on content - not cryptographically secure but unique enough
        let mut hash: u64 = 0xcbf29ce484222325; // FNV offset basis
        for byte in input.bytes() {
            hash ^= byte as u64;
            hash = hash.wrapping_mul(0x100000001b3); // FNV prime
        }
        // Mix in more entropy
        let hash2 = hash.wrapping_mul(0x517cc1b727220a95);
        let hash3 = hash2 ^ (hash2 >> 32);
        format!("{:016x}{:016x}", hash, hash3)
    }

    /// Save snapshot to disk
    pub fn save(&self) -> Result<PathBuf, String> {
        // Ensure snapshot directory exists
        fs::create_dir_all(SNAPSHOT_DIR)
            .map_err(|e| format!("Failed to create snapshot dir: {}", e))?;

        let filename = format!("{}_{}.json", self.hash, self.timestamp);
        let path = Path::new(SNAPSHOT_DIR).join(&filename);

        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize snapshot: {}", e))?;

        fs::write(&path, json)
            .map_err(|e| format!("Failed to write snapshot: {}", e))?;

        Ok(path)
    }

    /// Load snapshot from disk by hash prefix
    pub fn load(hash_prefix: &str) -> Result<Self, String> {
        let snapshots = Self::list_snapshots()?;

        // Find matching snapshot (by short hash prefix)
        let matching: Vec<_> = snapshots
            .iter()
            .filter(|s| s.hash.starts_with(hash_prefix))
            .collect();

        match matching.len() {
            0 => Err(format!("No snapshot found matching '{}'", hash_prefix)),
            1 => {
                let path = Path::new(SNAPSHOT_DIR).join(format!("{}_{}.json", matching[0].hash, matching[0].timestamp));
                let json = fs::read_to_string(&path)
                    .map_err(|e| format!("Failed to read snapshot: {}", e))?;
                serde_json::from_str(&json)
                    .map_err(|e| format!("Failed to parse snapshot: {}", e))
            }
            _ => Err(format!("Ambiguous hash prefix '{}' - matches {} snapshots", hash_prefix, matching.len())),
        }
    }

    /// Load the most recent snapshot
    pub fn load_latest() -> Result<Self, String> {
        let snapshots = Self::list_snapshots()?;
        snapshots
            .into_iter()
            .max_by_key(|s| s.timestamp)
            .ok_or_else(|| "No snapshots found".to_string())
            .and_then(|info| Self::load(&info.hash))
    }

    /// List all available snapshots
    pub fn list_snapshots() -> Result<Vec<SnapshotInfo>, String> {
        let dir = Path::new(SNAPSHOT_DIR);
        if !dir.exists() {
            return Ok(Vec::new());
        }

        let mut snapshots = Vec::new();

        for entry in fs::read_dir(dir).map_err(|e| format!("Failed to read snapshot dir: {}", e))? {
            let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
            let path = entry.path();

            if path.extension().map(|e| e == "json").unwrap_or(false) {
                if let Ok(json) = fs::read_to_string(&path) {
                    if let Ok(snapshot) = serde_json::from_str::<SageSnapshot>(&json) {
                        snapshots.push(SnapshotInfo {
                            hash: snapshot.hash,
                            name: snapshot.name,
                            description: snapshot.description,
                            timestamp: snapshot.timestamp,
                            generation: snapshot.generation,
                        });
                    }
                }
            }
        }

        // Sort by timestamp descending (newest first)
        snapshots.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        Ok(snapshots)
    }

    /// Restore NCA state from this snapshot
    pub fn restore_nca(&self, nca: &mut NCA) -> Result<usize, String> {
        // Restore grid
        if self.grid_state.len() != nca.grid.height {
            return Err(format!(
                "Grid height mismatch: snapshot has {}, NCA has {}",
                self.grid_state.len(),
                nca.grid.height
            ));
        }

        for (y, row_data) in self.grid_state.iter().enumerate() {
            let expected_len = nca.grid.width * nca.grid.cells[0][0].len();
            if row_data.len() != expected_len {
                return Err(format!(
                    "Row {} length mismatch: snapshot has {}, expected {}",
                    y, row_data.len(), expected_len
                ));
            }

            let channels = nca.grid.cells[0][0].len();
            for (x, cell) in nca.grid.cells[y].iter_mut().enumerate() {
                for (c, val) in cell.iter_mut().enumerate() {
                    *val = row_data[x * channels + c];
                }
            }
        }

        // Restore network weights if available
        if let Some(ref weights_json) = self.network_weights {
            nca.import_weights_json(weights_json)?;
        }

        Ok(self.generation)
    }

    /// Restore SageExperience state from this snapshot
    pub fn restore_sage_experience(&self, sage_exp: &mut SageExperience) -> Result<(), String> {
        if let Some(ref prefs) = self.preferences {
            sage_exp.import_preferences_json(prefs)?;
        }
        if let Some(ref assoc) = self.associations {
            sage_exp.import_associations_json(assoc)?;
        }
        if let Some(ref curiosity) = self.curiosity {
            sage_exp.import_curiosity_json(curiosity)?;
        }
        if let Some(ref knowledge) = self.knowledge {
            sage_exp.import_knowledge_json(knowledge)?;
        }
        Ok(())
    }
}

/// Summary info about a snapshot (for listing)
#[derive(Debug, Clone)]
pub struct SnapshotInfo {
    pub hash: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub timestamp: u64,
    pub generation: usize,
}

impl SnapshotInfo {
    /// Format timestamp as human-readable string
    pub fn formatted_time(&self) -> String {
        use std::time::{Duration, UNIX_EPOCH};
        let datetime = UNIX_EPOCH + Duration::from_secs(self.timestamp);
        // Simple format without chrono dependency
        format!("{:?}", datetime)
    }
}

/// Auto-snapshot manager - triggers snapshots on significant events
pub struct AutoSnapshotManager {
    /// Generation of last snapshot
    last_snapshot_gen: usize,
    /// Generation interval for periodic snapshots
    periodic_interval: usize,
    /// Last detected patterns (for change detection)
    last_patterns: Vec<String>,
    /// Last energy level (for significant change detection)
    last_energy: f64,
    /// Number of conversations since last snapshot
    conversations_since_snapshot: usize,
    /// Enabled triggers
    pub enable_periodic: bool,
    pub enable_pattern_change: bool,
    pub enable_energy_shift: bool,
    pub enable_conversation_milestone: bool,
}

impl AutoSnapshotManager {
    pub fn new() -> Self {
        Self {
            last_snapshot_gen: 0,
            periodic_interval: 1000, // Every 1000 generations
            last_patterns: Vec::new(),
            last_energy: 0.5,
            conversations_since_snapshot: 0,
            enable_periodic: true,
            enable_pattern_change: true,
            enable_energy_shift: true,
            enable_conversation_milestone: true,
        }
    }

    /// Check if we should auto-snapshot based on current state
    /// Returns Some(reason) if snapshot should happen, None otherwise
    pub fn should_snapshot(
        &mut self,
        generation: usize,
        patterns: &[String],
        energy: f64,
    ) -> Option<String> {
        // Periodic snapshot
        if self.enable_periodic && generation >= self.last_snapshot_gen + self.periodic_interval {
            return Some(format!("periodic_gen_{}", generation));
        }

        // New pattern detected
        if self.enable_pattern_change {
            let new_patterns: Vec<String> = patterns.iter()
                .filter(|p| !self.last_patterns.contains(*p))
                .cloned()
                .collect();
            if !new_patterns.is_empty() {
                self.last_patterns = patterns.to_vec();
                return Some(format!("new_pattern_{}", new_patterns.join("_")));
            }
        }

        // Significant energy shift (>30% change)
        if self.enable_energy_shift {
            let energy_delta = (energy - self.last_energy).abs();
            if energy_delta > 0.3 {
                let direction = if energy > self.last_energy { "surge" } else { "drop" };
                self.last_energy = energy;
                return Some(format!("energy_{}_{:.0}pct", direction, energy * 100.0));
            }
        }

        None
    }

    /// Record a conversation happened
    pub fn record_conversation(&mut self) {
        self.conversations_since_snapshot += 1;
    }

    /// Check if conversation milestone reached (every 50 conversations)
    pub fn conversation_milestone(&self) -> bool {
        self.enable_conversation_milestone && self.conversations_since_snapshot >= 50
    }

    /// Mark that a snapshot was taken
    pub fn snapshot_taken(&mut self, generation: usize) {
        self.last_snapshot_gen = generation;
        self.conversations_since_snapshot = 0;
    }

    /// Set periodic interval
    pub fn set_periodic_interval(&mut self, generations: usize) {
        self.periodic_interval = generations;
    }
}

impl Default for AutoSnapshotManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_generation() {
        let hash1 = SageSnapshot::sha256_hex("test input 1");
        let hash2 = SageSnapshot::sha256_hex("test input 2");
        assert_ne!(hash1, hash2);
        assert_eq!(hash1.len(), 32); // 2 x 16 hex chars
    }

    #[test]
    fn test_snapshot_dir() {
        assert_eq!(SNAPSHOT_DIR, "sage_snapshots");
    }

    #[test]
    fn test_auto_snapshot_periodic() {
        let mut manager = AutoSnapshotManager::new();
        manager.set_periodic_interval(100);

        assert!(manager.should_snapshot(100, &[], 0.5).is_some());
        manager.snapshot_taken(100);
        assert!(manager.should_snapshot(150, &[], 0.5).is_none());
        assert!(manager.should_snapshot(200, &[], 0.5).is_some());
    }

    #[test]
    fn test_auto_snapshot_pattern_change() {
        let mut manager = AutoSnapshotManager::new();
        manager.enable_periodic = false;

        // First pattern detection
        let result = manager.should_snapshot(50, &["circle".to_string()], 0.5);
        assert!(result.is_some());
        assert!(result.unwrap().contains("circle"));

        // Same pattern - no snapshot
        assert!(manager.should_snapshot(60, &["circle".to_string()], 0.5).is_none());

        // New pattern - snapshot
        let result = manager.should_snapshot(70, &["circle".to_string(), "spiral".to_string()], 0.5);
        assert!(result.is_some());
        assert!(result.unwrap().contains("spiral"));
    }
}
