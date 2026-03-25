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
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

use diff::{merkle_hash, KnowledgeDiff, SignatureError};
use gossip::{GossipError, GossipMessage, GridStateRequest, GridStateResponse, GossipTransport, PeerAnnounce};
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
    /// Optional gossip transport for sending responses (e.g. to GridStateRequest).
    transport: Option<Arc<dyn GossipTransport>>,
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
            transport: None,
        }
    }

    /// Create a NetworkManager with a wired gossip transport.
    ///
    /// Use this constructor when you want `NetworkManager` to be able to *send*
    /// responses (e.g. reply to `GridStateRequest`).  The transport is stored as
    /// an `Arc<dyn GossipTransport>` so it can be shared with the swarm loop.
    pub fn with_transport(
        identity: NodeIdentity,
        config: NetworkConfig,
        transport: Arc<dyn GossipTransport>,
    ) -> Self {
        let mut mgr = Self::new(identity, config);
        mgr.transport = Some(transport);
        mgr
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
                self.handle_grid_state_request(req, local_grid).await;
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
                        grid,
                        confidence,
                        ..
                    } => {
                        println!(
                            "[network] Received full state from {node_id} ({} rows)",
                            grid.len()
                        );
                        self.merge_full_state(&node_id, &grid, confidence, local_grid).await;
                    }
                }
            }
        }
    }

    /// Respond to a `GridStateRequest` from a peer.
    ///
    /// Sends one of three responses via the wired transport:
    /// - `InSync`        — Merkle hashes already match, nothing to send.
    /// - `Diff`          — Hash mismatch; send a diff from the zero-baseline so the
    ///                     peer can apply it.  (A future improvement would cache each
    ///                     peer's last-known grid and diff from that instead.)
    /// - `FullState`     — Peer explicitly requested the full grid (e.g. first join).
    ///
    /// If no transport is wired, logs a warning and returns without sending.
    pub async fn handle_grid_state_request(
        &self,
        req: GridStateRequest,
        local_grid: &[Vec<Vec<f64>>],
    ) {
        let Some(ref transport) = self.transport else {
            eprintln!(
                "[network] No transport wired — cannot respond to GridStateRequest from {}",
                req.requesting_node
            );
            return;
        };

        let local_hash = merkle_hash(local_grid);

        let response = if local_hash == req.current_hash {
            // Hashes match — nothing to do.
            GossipMessage::GridStateResponse(GridStateResponse::InSync)
        } else if req.full_state {
            // Peer wants the complete grid (initial sync / large divergence).
            GossipMessage::GridStateResponse(GridStateResponse::FullState {
                node_id: self.identity.node_id.clone(),
                grid: local_grid.to_vec(),
                state_hash: local_hash,
                confidence: self.config.local_confidence,
            })
        } else {
            // Compute a diff from the zero-baseline so the peer can catch up.
            // This is a conservative approach: we send everything rather than
            // computing from the peer's actual state (which we don't have cached).
            let height = local_grid.len();
            let width = if height > 0 { local_grid[0].len() } else { 0 };
            let channels = if height > 0 && width > 0 {
                local_grid[0][0].len()
            } else {
                0
            };
            let zeros = vec![vec![vec![0.0_f64; channels]; width]; height];
            let seq = *self.sequence.lock().await;
            let mut diff = KnowledgeDiff::compute(
                &zeros,
                local_grid,
                self.identity.node_id.clone(),
                seq,
                self.config.local_confidence,
                self.config.diff_threshold,
            );
            diff.sign(&self.identity.seed_bytes());
            GossipMessage::GridStateResponse(GridStateResponse::Diff(diff))
        };

        if let Err(e) = transport
            .send_to(&req.requesting_node, response)
            .await
        {
            eprintln!(
                "[network] Failed to send GridStateResponse to {}: {e}",
                req.requesting_node
            );
        }
    }

    /// Merge a `FullState` response from a peer into the local grid.
    ///
    /// Treats the peer's entire grid as a single diff from zeros and applies it
    /// with the configured `local_confidence` weight.  This ensures the local
    /// node's own knowledge is never completely overwritten by a remote state.
    async fn merge_full_state(
        &self,
        node_id: &str,
        remote_grid: &[Vec<Vec<f64>>],
        remote_confidence: f64,
        local_grid: &mut [Vec<Vec<f64>>],
    ) {
        let height = local_grid.len();
        let width = if height > 0 { local_grid[0].len() } else { 0 };
        let channels = if height > 0 && width > 0 {
            local_grid[0][0].len()
        } else {
            0
        };

        if height == 0 || width == 0 || channels == 0 {
            eprintln!("[network] merge_full_state: local grid is empty, skipping");
            return;
        }

        // Build a diff from zeros → remote grid so we can apply it with weighting.
        let zeros = vec![vec![vec![0.0_f64; channels]; width]; height];
        let diff = KnowledgeDiff::compute(
            &zeros,
            remote_grid,
            node_id.to_string(),
            0,
            remote_confidence,
            self.config.diff_threshold,
        );

        let change_count = diff.changes.len();
        diff.apply_weighted(local_grid, self.config.local_confidence);

        println!(
            "[network] Merged full state from {node_id} ({change_count} changes, weight={:.2})",
            self.config.local_confidence
        );

        let mut stats = self.stats.write().await;
        stats.diffs_received += 1;
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
    use gossip::{GossipError, GossipMessage, GossipTransport};
    use std::sync::Arc;
    use tokio::sync::Mutex;

    /// A mock transport that captures all outgoing messages.
    struct MockTransport {
        sent: Arc<Mutex<Vec<(String, GossipMessage)>>>,
    }

    impl MockTransport {
        fn new() -> (Self, Arc<Mutex<Vec<(String, GossipMessage)>>>) {
            let sent = Arc::new(Mutex::new(Vec::new()));
            (Self { sent: Arc::clone(&sent) }, sent)
        }
    }

    #[async_trait::async_trait]
    impl GossipTransport for MockTransport {
        async fn broadcast(&self, _msg: GossipMessage) -> Result<(), GossipError> {
            Ok(())
        }
        async fn send_to(&self, peer_id: &str, msg: GossipMessage) -> Result<(), GossipError> {
            self.sent.lock().await.push((peer_id.to_string(), msg));
            Ok(())
        }
        async fn recv(&self) -> Result<(String, GossipMessage), GossipError> {
            Err(GossipError::NotStarted)
        }
        async fn connected_peers(&self) -> Vec<String> {
            Vec::new()
        }
        async fn start(&self) -> Result<(), GossipError> {
            Ok(())
        }
        async fn stop(&self) -> Result<(), GossipError> {
            Ok(())
        }
    }

    fn make_grid(h: usize, w: usize, ch: usize, val: f64) -> Vec<Vec<Vec<f64>>> {
        vec![vec![vec![val; ch]; w]; h]
    }

    fn make_manager_with_transport(
        transport: Arc<dyn GossipTransport>,
    ) -> NetworkManager {
        let identity = identity::NodeIdentity::generate();
        let config = NetworkConfig::default();
        NetworkManager::with_transport(identity, config, transport)
    }

    #[tokio::test]
    async fn test_grid_state_request_insync_when_hashes_match() {
        let (mock, sent) = MockTransport::new();
        let transport: Arc<dyn GossipTransport> = Arc::new(mock);
        let mgr = make_manager_with_transport(transport);

        let grid = make_grid(4, 4, 2, 0.5);
        let hash = merkle_hash(&grid);

        let req = gossip::GridStateRequest {
            requesting_node: "peer-1".to_string(),
            current_hash: hash,
            full_state: false,
        };

        mgr.handle_grid_state_request(req, &grid).await;

        let messages = sent.lock().await;
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].0, "peer-1");
        match &messages[0].1 {
            GossipMessage::GridStateResponse(GridStateResponse::InSync) => {}
            other => panic!("Expected InSync, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_grid_state_request_sends_diff_on_hash_mismatch() {
        let (mock, sent) = MockTransport::new();
        let transport: Arc<dyn GossipTransport> = Arc::new(mock);
        let mgr = make_manager_with_transport(transport);

        let grid = make_grid(4, 4, 2, 0.5);
        let different_hash = [0u8; 32]; // deliberately wrong

        let req = gossip::GridStateRequest {
            requesting_node: "peer-2".to_string(),
            current_hash: different_hash,
            full_state: false,
        };

        mgr.handle_grid_state_request(req, &grid).await;

        let messages = sent.lock().await;
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].0, "peer-2");
        match &messages[0].1 {
            GossipMessage::GridStateResponse(GridStateResponse::Diff(d)) => {
                assert!(!d.changes.is_empty(), "Diff should have changes");
                assert_eq!(d.source_node, mgr.identity.node_id);
            }
            other => panic!("Expected Diff, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_grid_state_request_full_state() {
        let (mock, sent) = MockTransport::new();
        let transport: Arc<dyn GossipTransport> = Arc::new(mock);
        let mgr = make_manager_with_transport(transport);

        let grid = make_grid(4, 4, 2, 0.7);
        let req = gossip::GridStateRequest {
            requesting_node: "peer-3".to_string(),
            current_hash: [0u8; 32],
            full_state: true,
        };

        mgr.handle_grid_state_request(req, &grid).await;

        let messages = sent.lock().await;
        assert_eq!(messages.len(), 1);
        match &messages[0].1 {
            GossipMessage::GridStateResponse(GridStateResponse::FullState {
                node_id,
                grid: remote_grid,
                ..
            }) => {
                assert_eq!(node_id, &mgr.identity.node_id);
                assert_eq!(remote_grid.len(), 4);
            }
            other => panic!("Expected FullState, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_merge_full_state_applies_to_local_grid() {
        let identity = identity::NodeIdentity::generate();
        let config = NetworkConfig::default();
        let mgr = NetworkManager::new(identity, config);

        let remote = make_grid(4, 4, 2, 1.0);
        let mut local = make_grid(4, 4, 2, 0.0);

        mgr.merge_full_state("peer-4", &remote, 1.0, &mut local).await;

        // apply_weighted blends: remote_weight = remote_confidence / (local_confidence + remote_confidence)
        // With remote_confidence=1.0, local_confidence=0.8:
        //   remote_weight = 1.0 / 1.8 ≈ 0.5556
        //   result = 0.0 * (0.8/1.8) + 1.0 * (1.0/1.8) ≈ 0.5556
        let lc = mgr.config.local_confidence; // 0.8
        let rc = 1.0_f64;
        let expected = rc / (lc + rc);
        for row in &local {
            for cell in row {
                for &val in cell {
                    assert!(
                        (val - expected).abs() < 1e-9,
                        "Expected {expected}, got {val}"
                    );
                }
            }
        }
    }

    #[tokio::test]
    async fn test_no_transport_does_not_panic() {
        let identity = identity::NodeIdentity::generate();
        let config = NetworkConfig::default();
        let mgr = NetworkManager::new(identity, config); // no transport

        let grid = make_grid(4, 4, 2, 0.5);
        let req = gossip::GridStateRequest {
            requesting_node: "peer-5".to_string(),
            current_hash: [0u8; 32],
            full_state: false,
        };

        // Should log a warning but not panic.
        mgr.handle_grid_state_request(req, &grid).await;
    }
}
