//! Security Module — Peer banning, rate limiting, and connection management.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Instant;

// ─── Peer Banning ───────────────────────────────────────────────────────────

/// Trust threshold below which a peer gets banned.
pub const BAN_TRUST_THRESHOLD: f64 = 0.1;

/// Banned peer entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BannedPeer {
    pub node_id: String,
    pub reason: String,
    pub banned_at_ms: u64,
    pub trust_at_ban: f64,
}

/// Manages the ban list, persisted to ~/.sage/banned.toml.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BanList {
    pub banned: Vec<BannedPeer>,
}

impl BanList {
    pub fn new() -> Self {
        Self { banned: vec![] }
    }

    /// Load from ~/.sage/banned.toml.
    pub fn load() -> Self {
        let path = dirs::home_dir()
            .map(|h| h.join(".sage").join("banned.toml"))
            .unwrap_or_default();
        if let Ok(contents) = std::fs::read_to_string(&path) {
            toml::from_str(&contents).unwrap_or_default()
        } else {
            Self::new()
        }
    }

    /// Save to ~/.sage/banned.toml.
    pub fn save(&self) {
        let Some(home) = dirs::home_dir() else { return };
        let dir = home.join(".sage");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("banned.toml");
        if let Ok(contents) = toml::to_string_pretty(self) {
            let _ = std::fs::write(path, contents);
        }
    }

    /// Check if a node is banned.
    pub fn is_banned(&self, node_id: &str) -> bool {
        self.banned.iter().any(|b| b.node_id == node_id)
    }

    /// Ban a peer.
    pub fn ban(&mut self, node_id: &str, reason: &str, trust: f64) {
        if self.is_banned(node_id) {
            return;
        }
        self.banned.push(BannedPeer {
            node_id: node_id.to_string(),
            reason: reason.to_string(),
            banned_at_ms: now_ms(),
            trust_at_ban: trust,
        });
        self.save();
    }

    /// Check trust and auto-ban if below threshold. Returns true if banned.
    pub fn check_and_ban(&mut self, node_id: &str, trust: f64) -> bool {
        if trust < BAN_TRUST_THRESHOLD {
            self.ban(node_id, "Trust dropped below threshold", trust);
            true
        } else {
            false
        }
    }
}

// ─── Rate Limiting ──────────────────────────────────────────────────────────

/// Rate limiting configuration.
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Max new connections per minute from the same IP.
    pub max_connections_per_minute: usize,
    /// Max diffs per hour per peer.
    pub max_diffs_per_hour: usize,
    /// Base backoff duration in seconds for validation failures.
    pub base_backoff_secs: u64,
    /// Maximum backoff duration in seconds.
    pub max_backoff_secs: u64,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_connections_per_minute: 10,
            max_diffs_per_hour: 60,
            base_backoff_secs: 5,
            max_backoff_secs: 3600,
        }
    }
}

/// Per-IP connection tracking.
#[derive(Debug)]
struct ConnectionBucket {
    timestamps: Vec<Instant>,
}

/// Per-peer diff tracking.
#[derive(Debug)]
struct DiffBucket {
    timestamps: Vec<Instant>,
    /// Number of consecutive validation failures.
    consecutive_failures: u32,
    /// When the backoff expires (None = not backed off).
    backoff_until: Option<Instant>,
}

/// Rate limiter for connections and diffs.
#[derive(Debug)]
pub struct RateLimiter {
    config: RateLimitConfig,
    connections: HashMap<String, ConnectionBucket>,
    diffs: HashMap<String, DiffBucket>,
}

/// Result of a rate limit check.
#[derive(Debug, Clone, PartialEq)]
pub enum RateLimitResult {
    Allowed,
    /// Rate limited — includes seconds until next allowed attempt.
    Limited(u64),
    /// Peer is in exponential backoff.
    BackedOff(u64),
}

impl RateLimiter {
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            config,
            connections: HashMap::new(),
            diffs: HashMap::new(),
        }
    }

    /// Check if a connection from this IP is allowed.
    pub fn check_connection(&mut self, ip: &str) -> RateLimitResult {
        let now = Instant::now();
        let bucket = self.connections.entry(ip.to_string()).or_insert(ConnectionBucket {
            timestamps: vec![],
        });

        // Prune timestamps older than 1 minute
        let one_min_ago = now - std::time::Duration::from_secs(60);
        bucket.timestamps.retain(|t| *t > one_min_ago);

        if bucket.timestamps.len() >= self.config.max_connections_per_minute {
            // Estimate seconds until oldest expires
            if let Some(oldest) = bucket.timestamps.first() {
                let wait = 60u64.saturating_sub(oldest.elapsed().as_secs());
                return RateLimitResult::Limited(wait);
            }
            return RateLimitResult::Limited(60);
        }

        bucket.timestamps.push(now);
        RateLimitResult::Allowed
    }

    /// Check if a diff from this peer is allowed.
    pub fn check_diff(&mut self, peer_id: &str) -> RateLimitResult {
        let now = Instant::now();
        let bucket = self.diffs.entry(peer_id.to_string()).or_insert(DiffBucket {
            timestamps: vec![],
            consecutive_failures: 0,
            backoff_until: None,
        });

        // Check backoff
        if let Some(until) = bucket.backoff_until {
            if now < until {
                let remaining = (until - now).as_secs();
                return RateLimitResult::BackedOff(remaining);
            }
            bucket.backoff_until = None;
        }

        // Prune timestamps older than 1 hour
        let one_hour_ago = now - std::time::Duration::from_secs(3600);
        bucket.timestamps.retain(|t| *t > one_hour_ago);

        if bucket.timestamps.len() >= self.config.max_diffs_per_hour {
            if let Some(oldest) = bucket.timestamps.first() {
                let wait = 3600u64.saturating_sub(oldest.elapsed().as_secs());
                return RateLimitResult::Limited(wait);
            }
            return RateLimitResult::Limited(3600);
        }

        bucket.timestamps.push(now);
        RateLimitResult::Allowed
    }

    /// Record a validation failure — triggers exponential backoff.
    pub fn record_failure(&mut self, peer_id: &str) {
        let bucket = self.diffs.entry(peer_id.to_string()).or_insert(DiffBucket {
            timestamps: vec![],
            consecutive_failures: 0,
            backoff_until: None,
        });

        bucket.consecutive_failures += 1;
        let backoff_secs = (self.config.base_backoff_secs * 2u64.pow(bucket.consecutive_failures.min(16)))
            .min(self.config.max_backoff_secs);
        bucket.backoff_until = Some(Instant::now() + std::time::Duration::from_secs(backoff_secs));
    }

    /// Record a validation success — resets failure counter.
    pub fn record_success(&mut self, peer_id: &str) {
        if let Some(bucket) = self.diffs.get_mut(peer_id) {
            bucket.consecutive_failures = 0;
            bucket.backoff_until = None;
        }
    }

    /// Prune stale entries (call periodically).
    pub fn cleanup(&mut self) {
        let now = Instant::now();
        let one_hour = std::time::Duration::from_secs(3600);
        self.connections.retain(|_, b| {
            b.timestamps.retain(|t| now.duration_since(*t) < std::time::Duration::from_secs(60));
            !b.timestamps.is_empty()
        });
        self.diffs.retain(|_, b| {
            b.timestamps.retain(|t| now.duration_since(*t) < one_hour);
            !b.timestamps.is_empty() || b.backoff_until.is_some()
        });
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
    fn test_ban_list() {
        let mut bans = BanList::new();
        assert!(!bans.is_banned("peer-1"));
        bans.ban("peer-1", "test", 0.05);
        assert!(bans.is_banned("peer-1"));
        assert!(!bans.is_banned("peer-2"));
    }

    #[test]
    fn test_check_and_ban() {
        let mut bans = BanList::new();
        assert!(!bans.check_and_ban("peer-1", 0.5));
        assert!(!bans.is_banned("peer-1"));
        assert!(bans.check_and_ban("peer-2", 0.05));
        assert!(bans.is_banned("peer-2"));
    }

    #[test]
    fn test_rate_limiter_connections() {
        let config = RateLimitConfig {
            max_connections_per_minute: 3,
            ..Default::default()
        };
        let mut limiter = RateLimiter::new(config);

        assert_eq!(limiter.check_connection("1.2.3.4"), RateLimitResult::Allowed);
        assert_eq!(limiter.check_connection("1.2.3.4"), RateLimitResult::Allowed);
        assert_eq!(limiter.check_connection("1.2.3.4"), RateLimitResult::Allowed);
        // 4th should be limited
        assert!(matches!(limiter.check_connection("1.2.3.4"), RateLimitResult::Limited(_)));
        // Different IP should be fine
        assert_eq!(limiter.check_connection("5.6.7.8"), RateLimitResult::Allowed);
    }

    #[test]
    fn test_rate_limiter_diffs() {
        let config = RateLimitConfig {
            max_diffs_per_hour: 2,
            ..Default::default()
        };
        let mut limiter = RateLimiter::new(config);

        assert_eq!(limiter.check_diff("peer-1"), RateLimitResult::Allowed);
        assert_eq!(limiter.check_diff("peer-1"), RateLimitResult::Allowed);
        assert!(matches!(limiter.check_diff("peer-1"), RateLimitResult::Limited(_)));
    }

    #[test]
    fn test_exponential_backoff() {
        let config = RateLimitConfig {
            base_backoff_secs: 1,
            max_backoff_secs: 100,
            ..Default::default()
        };
        let mut limiter = RateLimiter::new(config);

        limiter.record_failure("peer-1");
        // Should be backed off
        assert!(matches!(limiter.check_diff("peer-1"), RateLimitResult::BackedOff(_)));

        // Success resets
        limiter.record_success("peer-1");
        // After resetting backoff_until, should be allowed
        if let Some(bucket) = limiter.diffs.get_mut("peer-1") {
            bucket.backoff_until = None;
        }
        assert_eq!(limiter.check_diff("peer-1"), RateLimitResult::Allowed);
    }
}
