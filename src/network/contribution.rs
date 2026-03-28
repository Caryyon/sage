//! Node Contribution Tracking — track patterns shared, tiers, and network participation.
//!
//! This module tracks each node's contribution to the network:
//! - Patterns shared/received
//! - Unique peers connected
//! - Network age (first seen timestamp)
//! - Contribution tier based on patterns shared

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

/// Contribution tier based on patterns shared
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContributionTier {
    /// 0-99 patterns shared
    Seedling,
    /// 100-999 patterns shared
    Sprout,
    /// 1,000-9,999 patterns shared
    Grove,
    /// 10,000+ patterns shared
    Forest,
}

impl ContributionTier {
    /// Determine tier from patterns shared count
    pub fn from_patterns(patterns_shared: u64) -> Self {
        match patterns_shared {
            0..=99 => ContributionTier::Seedling,
            100..=999 => ContributionTier::Sprout,
            1000..=9999 => ContributionTier::Grove,
            _ => ContributionTier::Forest,
        }
    }

    /// Get the emoji for this tier
    pub fn emoji(&self) -> &'static str {
        match self {
            ContributionTier::Seedling => "🌱",
            ContributionTier::Sprout => "🌿",
            ContributionTier::Grove => "🌳",
            ContributionTier::Forest => "🌲",
        }
    }

    /// Get the name for this tier
    pub fn name(&self) -> &'static str {
        match self {
            ContributionTier::Seedling => "Seedling",
            ContributionTier::Sprout => "Sprout",
            ContributionTier::Grove => "Grove",
            ContributionTier::Forest => "Forest",
        }
    }
}

impl std::fmt::Display for ContributionTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.emoji(), self.name())
    }
}

/// Persistent contribution statistics for a node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContributionStats {
    /// Total patterns sent to peers
    pub patterns_sent: u64,
    /// Total patterns received from peers
    pub patterns_received: u64,
    /// Total bytes sent
    pub bytes_sent: u64,
    /// Total bytes received
    pub bytes_received: u64,
    /// Unique peer node IDs that have connected
    pub unique_peers: HashSet<String>,
    /// Unix timestamp of first network participation
    pub first_seen_ms: u64,
    /// Total uptime in seconds across all sessions
    pub total_uptime_secs: u64,
    /// Current session start time (not persisted, set on load)
    #[serde(skip)]
    pub session_start_ms: u64,
}

impl Default for ContributionStats {
    fn default() -> Self {
        Self::new()
    }
}

impl ContributionStats {
    /// Create new empty contribution stats
    pub fn new() -> Self {
        let now_ms = now_ms();
        Self {
            patterns_sent: 0,
            patterns_received: 0,
            bytes_sent: 0,
            bytes_received: 0,
            unique_peers: HashSet::new(),
            first_seen_ms: now_ms,
            total_uptime_secs: 0,
            session_start_ms: now_ms,
        }
    }

    /// Default path for contribution stats file
    pub fn default_path() -> PathBuf {
        let home = dirs::home_dir().expect("could not determine home directory");
        home.join(".sage").join("contribution.json")
    }

    /// Load contribution stats from disk, or create new if not found
    pub fn load_or_create() -> Self {
        let path = Self::default_path();
        if path.exists() {
            match fs::read_to_string(&path) {
                Ok(content) => {
                    match serde_json::from_str::<ContributionStats>(&content) {
                        Ok(mut stats) => {
                            stats.session_start_ms = now_ms();
                            return stats;
                        }
                        Err(_) => {
                            // Corrupted file, start fresh
                        }
                    }
                }
                Err(_) => {
                    // Can't read, start fresh
                }
            }
        }
        Self::new()
    }

    /// Save contribution stats to disk
    pub fn save(&self) -> std::io::Result<()> {
        let path = Self::default_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        fs::write(&path, content)
    }

    /// Record patterns sent to a peer
    pub fn record_sent(&mut self, peer_id: &str, pattern_count: u64, bytes: u64) {
        self.patterns_sent += pattern_count;
        self.bytes_sent += bytes;
        self.unique_peers.insert(peer_id.to_string());
    }

    /// Record patterns received from a peer
    pub fn record_received(&mut self, peer_id: &str, pattern_count: u64, bytes: u64) {
        self.patterns_received += pattern_count;
        self.bytes_received += bytes;
        self.unique_peers.insert(peer_id.to_string());
    }

    /// Update uptime (call before saving)
    pub fn update_uptime(&mut self) {
        let now = now_ms();
        let session_duration_ms = now.saturating_sub(self.session_start_ms);
        self.total_uptime_secs += session_duration_ms / 1000;
        self.session_start_ms = now;
    }

    /// Get current contribution tier
    pub fn tier(&self) -> ContributionTier {
        ContributionTier::from_patterns(self.patterns_sent)
    }

    /// Get network age in human-readable format
    pub fn network_age(&self) -> String {
        let now = now_ms();
        let age_ms = now.saturating_sub(self.first_seen_ms);
        let age_secs = age_ms / 1000;

        if age_secs < 60 {
            "just joined".to_string()
        } else if age_secs < 3600 {
            format!("{} min", age_secs / 60)
        } else if age_secs < 86400 {
            format!("{} hours", age_secs / 3600)
        } else {
            format!("{} days", age_secs / 86400)
        }
    }

    /// Get total uptime in human-readable format
    pub fn uptime_str(&self) -> String {
        let secs = self.total_uptime_secs;
        if secs < 60 {
            format!("{}s", secs)
        } else if secs < 3600 {
            format!("{}m {}s", secs / 60, secs % 60)
        } else if secs < 86400 {
            format!("{}h {}m", secs / 3600, (secs % 3600) / 60)
        } else {
            format!("{}d {}h", secs / 86400, (secs % 86400) / 3600)
        }
    }

    /// Get unique peer count
    pub fn peer_count(&self) -> usize {
        self.unique_peers.len()
    }
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tier_from_patterns() {
        assert_eq!(ContributionTier::from_patterns(0), ContributionTier::Seedling);
        assert_eq!(ContributionTier::from_patterns(99), ContributionTier::Seedling);
        assert_eq!(ContributionTier::from_patterns(100), ContributionTier::Sprout);
        assert_eq!(ContributionTier::from_patterns(999), ContributionTier::Sprout);
        assert_eq!(ContributionTier::from_patterns(1000), ContributionTier::Grove);
        assert_eq!(ContributionTier::from_patterns(9999), ContributionTier::Grove);
        assert_eq!(ContributionTier::from_patterns(10000), ContributionTier::Forest);
        assert_eq!(ContributionTier::from_patterns(100000), ContributionTier::Forest);
    }

    #[test]
    fn test_contribution_stats_new() {
        let stats = ContributionStats::new();
        assert_eq!(stats.patterns_sent, 0);
        assert_eq!(stats.patterns_received, 0);
        assert_eq!(stats.tier(), ContributionTier::Seedling);
    }

    #[test]
    fn test_record_sent_received() {
        let mut stats = ContributionStats::new();
        stats.record_sent("peer-1", 50, 1000);
        stats.record_received("peer-2", 30, 500);

        assert_eq!(stats.patterns_sent, 50);
        assert_eq!(stats.patterns_received, 30);
        assert_eq!(stats.peer_count(), 2);
        assert!(stats.unique_peers.contains("peer-1"));
        assert!(stats.unique_peers.contains("peer-2"));
    }
}
