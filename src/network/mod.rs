//! Network Module — SAGE gossip networking and peer synchronization.
//!
//! Provides node identity, knowledge diff computation, gossip message types,
//! and the NetworkManager that ties it all together.

pub mod diff;
pub mod gossip;
pub mod identity;
pub mod libp2p_transport;
pub mod privacy;
pub mod quarantine;
pub mod security;
pub mod validation;

use std::collections::HashMap;
use tokio::sync::{Mutex, RwLock};

use diff::{merkle_hash, KnowledgeDiff, SignatureError};
use gossip::{GossipError, GossipMessage, GridStateResponse, PeerAnnounce};
use identity::NodeIdentity;
use privacy::{
    apply_differential_privacy, filter_local_only_channels, AggregationTracker, PiiFilter,
    PrivacyConfig,
};
use quarantine::Quarantine;
use security::{BanList, RateLimitConfig, RateLimitResult, RateLimiter};
use validation::{DiffValidator, TrustStore, ValidationResult};

/// Information about a connected peer.
#[derive(Debug, Clone)]
pub struct PeerInfo {
    pub node_id: String,
    pub public_key: [u8; 32],
    pub state_hash: [u8; 32],
    pub last_seen_ms: u64,
    pub diff_count: u64,
    pub protocol_version: u32,
}

/// Configuration for the NetworkManager.
#[derive(Debug, Clone)]
pub struct NetworkConfig {
    /// How often to broadcast local diffs (seconds).
    pub sync_interval_secs: u64,
    /// Minimum change threshold for diff computation.
    pub diff_threshold: f64,
    /// Local confidence weight when merging incoming diffs.
    pub local_confidence: f64,
    /// Maximum peers to track.
    pub max_peers: usize,
    /// Port to listen on (0 = random).
    pub listen_port: u16,
    /// Enable mDNS for local discovery.
    pub mdns_enabled: bool,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            sync_interval_secs: 300, // 5 minutes
            diff_threshold: 1e-6,
            local_confidence: 0.8,
            max_peers: 50,
            listen_port: 0,
            mdns_enabled: true,
        }
    }
}

/// The main network manager — coordinates identity, peers, and sync.
pub struct NetworkManager {
    /// This node's identity.
    pub identity: NodeIdentity,
    /// Configuration.
    pub config: NetworkConfig,
    /// Known peers.
    peers: RwLock<HashMap<String, PeerInfo>>,
    /// The local grid state snapshot (for diff computation).
    /// Stored as the last-broadcast state so we can compute incremental diffs.
    last_broadcast_state: Mutex<Option<Vec<Vec<Vec<f64>>>>>,
    /// Monotonic sequence counter for outgoing diffs.
    sequence: Mutex<u64>,
    /// Whether the manager is currently running.
    running: RwLock<bool>,
    /// Stats.
    stats: RwLock<NetworkStats>,
    /// Diff validator (trust scoring + validation rules).
    validator: Mutex<DiffValidator>,
    /// Knowledge quarantine for suspicious diffs.
    quarantine: Mutex<Quarantine>,
    /// Privacy configuration.
    privacy_config: PrivacyConfig,
    /// Aggregation tracker — enforces min conversations before sync.
    aggregation: Mutex<AggregationTracker>,
    /// PII filter.
    pii_filter: Mutex<PiiFilter>,
    /// Banned peers list.
    ban_list: Mutex<BanList>,
    /// Rate limiter.
    rate_limiter: Mutex<RateLimiter>,
}

/// Network statistics.
#[derive(Debug, Clone, Default)]
pub struct NetworkStats {
    pub diffs_sent: u64,
    pub diffs_received: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub sync_rounds: u64,
}

impl NetworkManager {
    /// Create a new NetworkManager with the given identity and config.
    pub fn new(identity: NodeIdentity, config: NetworkConfig) -> Self {
        let trust_store = TrustStore::load();
        let privacy_config = PrivacyConfig::default();
        let aggregation_threshold = privacy_config.min_conversations_before_sync;
        Self {
            identity,
            config,
            peers: RwLock::new(HashMap::new()),
            last_broadcast_state: Mutex::new(None),
            sequence: Mutex::new(0),
            running: RwLock::new(false),
            stats: RwLock::new(NetworkStats::default()),
            validator: Mutex::new(DiffValidator::new(trust_store)),
            quarantine: Mutex::new(Quarantine::new()),
            privacy_config,
            aggregation: Mutex::new(AggregationTracker::new(aggregation_threshold)),
            pii_filter: Mutex::new(PiiFilter::new()),
            ban_list: Mutex::new(BanList::load()),
            rate_limiter: Mutex::new(RateLimiter::new(RateLimitConfig::default())),
        }
    }

    /// Create with default config, loading or generating identity.
    pub fn with_defaults() -> std::io::Result<Self> {
        let identity = NodeIdentity::load_or_generate(None)?;
        Ok(Self::new(identity, NetworkConfig::default()))
    }

    /// Start networking (placeholder — would start libp2p swarm).
    pub async fn start(&self) -> Result<(), GossipError> {
        let mut running = self.running.write().await;
        if *running {
            return Ok(());
        }
        *running = true;
        println!("[network] Node {} started", self.identity.node_id);
        Ok(())
    }

    /// Stop networking.
    pub async fn stop(&self) -> Result<(), GossipError> {
        let mut running = self.running.write().await;
        *running = false;
        println!("[network] Node {} stopped", self.identity.node_id);
        Ok(())
    }

    /// Check if the manager is running.
    pub async fn is_running(&self) -> bool {
        *self.running.read().await
    }

    /// Get the current peer count.
    pub async fn peer_count(&self) -> usize {
        self.peers.read().await.len()
    }

    /// Get info about all known peers.
    pub async fn get_peers(&self) -> Vec<PeerInfo> {
        self.peers.read().await.values().cloned().collect()
    }

    /// Get current stats.
    pub async fn get_stats(&self) -> NetworkStats {
        self.stats.read().await.clone()
    }

    /// Register or update a peer from an announcement.
    pub async fn handle_announce(&self, announce: PeerAnnounce) {
        let mut peers = self.peers.write().await;
        let info = PeerInfo {
            node_id: announce.node_id.clone(),
            public_key: announce.public_key,
            state_hash: announce.state_hash,
            last_seen_ms: announce.timestamp_ms,
            diff_count: announce.diff_count,
            protocol_version: announce.protocol_version,
        };

        if peers.len() < self.config.max_peers || peers.contains_key(&announce.node_id) {
            peers.insert(announce.node_id, info);
        }
    }

    /// Create a PeerAnnounce message for this node.
    pub async fn create_announce(&self, grid: &[Vec<Vec<f64>>]) -> GossipMessage {
        let height = grid.len();
        let width = if height > 0 { grid[0].len() } else { 0 };
        let channels = if height > 0 && width > 0 {
            grid[0][0].len()
        } else {
            0
        };
        let seq = *self.sequence.lock().await;

        GossipMessage::PeerAnnounce(PeerAnnounce {
            node_id: self.identity.node_id.clone(),
            public_key: self.identity.public_key,
            state_hash: merkle_hash(grid),
            grid_width: width,
            grid_height: height,
            grid_channels: channels,
            diff_count: seq,
            timestamp_ms: now_ms(),
            protocol_version: PeerAnnounce::CURRENT_PROTOCOL_VERSION,
        })
    }

    /// Compute a diff from the last broadcast state to the current grid.
    /// Updates the stored state and returns the diff (or None if empty).
    pub async fn compute_outgoing_diff(
        &self,
        current_grid: &[Vec<Vec<f64>>],
    ) -> Option<KnowledgeDiff> {
        let mut last_state = self.last_broadcast_state.lock().await;
        let mut seq = self.sequence.lock().await;

        let diff = if let Some(ref old) = *last_state {
            KnowledgeDiff::compute(
                old,
                current_grid,
                self.identity.node_id.clone(),
                *seq,
                self.config.local_confidence,
                self.config.diff_threshold,
            )
        } else {
            // First broadcast — diff against zeros
            let height = current_grid.len();
            let width = if height > 0 { current_grid[0].len() } else { 0 };
            let channels = if height > 0 && width > 0 {
                current_grid[0][0].len()
            } else {
                0
            };
            let zeros = vec![vec![vec![0.0; channels]; width]; height];
            KnowledgeDiff::compute(
                &zeros,
                current_grid,
                self.identity.node_id.clone(),
                *seq,
                self.config.local_confidence,
                self.config.diff_threshold,
            )
        };

        if diff.is_empty() {
            return None;
        }

        // Apply privacy protections before broadcasting
        let mut diff = diff;

        // Filter out local-only channels
        filter_local_only_channels(&mut diff, &self.privacy_config.local_only_channels);
        if diff.is_empty() {
            return None;
        }

        // Add differential privacy noise
        apply_differential_privacy(&mut diff, &self.privacy_config);

        // Sign the diff with our Ed25519 identity before broadcasting.
        // Recipients can call verify_signature() to authenticate the source.
        diff.sign(&self.identity.seed_bytes());

        // Update stored state
        *last_state = Some(current_grid.to_vec());
        *seq += 1;

        let mut stats = self.stats.write().await;
        stats.diffs_sent += 1;
        stats.sync_rounds += 1;

        Some(diff)
    }

    /// Handle an incoming diff — validate via trust/anti-poisoning and merge into the local grid.
    pub async fn handle_incoming_diff(
        &self,
        diff: KnowledgeDiff,
        local_grid: &mut [Vec<Vec<f64>>],
    ) {
        // Check if peer is banned
        {
            let ban_list = self.ban_list.lock().await;
            if ban_list.is_banned(&diff.source_node) {
                println!(
                    "[network] Rejected diff from banned peer {}",
                    diff.source_node
                );
                return;
            }
        }

        // Signature verification — reject unsigned or tampered diffs.
        // A missing signature is allowed for legacy/test peers but logged.
        // An invalid signature (present but wrong) is always rejected.
        match diff.verify_signature() {
            Ok(()) => {
                // Valid signature — node is authenticated.
            }
            Err(SignatureError::Missing) => {
                // No signature present — tolerate for now (legacy nodes / tests),
                // but note it for future enforcement.
                println!(
                    "[network] Warning: unsigned diff from {} (seq={}) — tolerated (legacy peer)",
                    diff.source_node, diff.sequence
                );
            }
            Err(e) => {
                // Signature present but invalid — could be tampering or impersonation.
                println!(
                    "[network] Rejected diff from {} (seq={}): {e}",
                    diff.source_node, diff.sequence
                );
                return;
            }
        }

        // Rate limit check
        {
            let mut limiter = self.rate_limiter.lock().await;
            match limiter.check_diff(&diff.source_node) {
                RateLimitResult::Allowed => {}
                RateLimitResult::Limited(secs) => {
                    println!(
                        "[network] Rate limited diff from {} (retry in {}s)",
                        diff.source_node, secs
                    );
                    return;
                }
                RateLimitResult::BackedOff(secs) => {
                    println!(
                        "[network] Backed off peer {} ({}s remaining)",
                        diff.source_node, secs
                    );
                    return;
                }
            }
        }

        // Basic confidence check
        if diff.confidence <= 0.0 || diff.confidence > 1.0 {
            println!(
                "[network] Rejected diff from {}: invalid confidence {}",
                diff.source_node, diff.confidence
            );
            return;
        }

        let source = diff.source_node.clone();
        let seq = diff.sequence;
        let change_count = diff.changes.len();

        // Run through validation pipeline
        let result = {
            let mut validator = self.validator.lock().await;
            validator.validate(&diff)
        };

        match result {
            ValidationResult::Accept => {
                diff.apply_weighted(local_grid, self.config.local_confidence);
                let mut validator = self.validator.lock().await;
                validator.record_useful(&source);
                println!(
                    "[network] Applied diff from {} (seq={}, {} changes, full weight)",
                    source, seq, change_count
                );
            }
            ValidationResult::AcceptReduced(weight) => {
                // Create a modified diff with reduced confidence
                let mut reduced_diff = diff;
                reduced_diff.confidence *= weight;
                reduced_diff.apply_weighted(local_grid, self.config.local_confidence);
                println!(
                    "[network] Applied diff from {} (seq={}, {} changes, weight={:.2})",
                    source, seq, change_count, weight
                );
            }
            ValidationResult::Quarantine(reason) => {
                println!(
                    "[network] Quarantined diff from {} (seq={}): {}",
                    source, seq, reason
                );
                let mut quarantine = self.quarantine.lock().await;
                quarantine.add(diff, reason);

                // Check if any quarantined items can be promoted
                let trust_store = &self.validator.lock().await.trust_store;
                let promoted = quarantine.promote(trust_store);
                drop(quarantine);
                for promoted_diff in promoted {
                    println!(
                        "[network] Promoting quarantined diff from {} (corroborated)",
                        promoted_diff.source_node
                    );
                    promoted_diff.apply_weighted(local_grid, self.config.local_confidence);
                }
            }
            ValidationResult::Reject(reason) => {
                println!(
                    "[network] Rejected diff from {} (seq={}): {}",
                    source, seq, reason
                );
                // Record failure for rate limiting
                let mut limiter = self.rate_limiter.lock().await;
                limiter.record_failure(&source);

                // Check if peer should be banned
                let trust = {
                    let validator = self.validator.lock().await;
                    validator.trust_store.get_trust(&source)
                };
                let mut ban_list = self.ban_list.lock().await;
                if ban_list.check_and_ban(&source, trust) {
                    println!("[network] Auto-banned peer {} (trust={:.3})", source, trust);
                }
                return;
            }
        }

        // Record success for rate limiting
        {
            let mut limiter = self.rate_limiter.lock().await;
            limiter.record_success(&source);
        }

        let mut stats = self.stats.write().await;
        stats.diffs_received += 1;
    }

    /// Run periodic quarantine maintenance (expire old items, promote corroborated ones).
    pub async fn maintain_quarantine(&self, local_grid: &mut [Vec<Vec<f64>>]) {
        let mut quarantine = self.quarantine.lock().await;
        quarantine.expire();
        let trust_store = &self.validator.lock().await.trust_store;
        let promoted = quarantine.promote(trust_store);
        drop(quarantine);
        for diff in promoted {
            println!(
                "[network] Promoting quarantined diff from {} (corroborated)",
                diff.source_node
            );
            diff.apply_weighted(local_grid, self.config.local_confidence);
        }
    }

    /// Record a conversation for aggregation tracking.
    /// Returns true if enough conversations have accumulated to allow a sync.
    pub async fn record_conversation(&self) -> bool {
        let mut agg = self.aggregation.lock().await;
        agg.record_conversation()
    }

    /// Reset aggregation counter (call after successful sync).
    pub async fn reset_aggregation(&self) {
        let mut agg = self.aggregation.lock().await;
        agg.reset();
    }

    /// Filter PII from text using the configured filter.
    pub async fn filter_pii(&self, text: &str) -> String {
        if !self.privacy_config.filter_pii {
            return text.to_string();
        }
        let filter = self.pii_filter.lock().await;
        filter.filter(text)
    }

    /// Check if a peer is banned.
    pub async fn is_peer_banned(&self, node_id: &str) -> bool {
        let ban_list = self.ban_list.lock().await;
        ban_list.is_banned(node_id)
    }

    /// Save trust scores to disk.
    pub async fn save_trust(&self) {
        let validator = self.validator.lock().await;
        validator.trust_store.save();
    }

    /// Handle an incoming gossip message.
    pub async fn handle_message(&self, message: GossipMessage, local_grid: &mut [Vec<Vec<f64>>]) {
        match message {
            GossipMessage::PeerAnnounce(announce) => {
                self.handle_announce(announce).await;
            }
            GossipMessage::KnowledgeDiff(diff) => {
                self.handle_incoming_diff(diff, local_grid).await;
            }
            GossipMessage::GridStateRequest(req) => {
                println!(
                    "[network] Grid state request from {} (full={})",
                    req.requesting_node, req.full_state
                );
                // Response would be sent via transport — placeholder
            }
            GossipMessage::GridStateResponse(resp) => {
                match resp {
                    GridStateResponse::InSync => {
                        println!("[network] Peer confirmed in-sync");
                    }
                    GridStateResponse::Diff(diff) => {
                        self.handle_incoming_diff(diff, local_grid).await;
                    }
                    GridStateResponse::FullState {
                        node_id,
                        grid: _,
                        confidence: _,
                        ..
                    } => {
                        println!("[network] Received full state from {node_id}");
                        // For full state, we'd do a weighted merge of the entire grid
                    }
                }
            }
        }
    }
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}
