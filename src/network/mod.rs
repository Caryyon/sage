//! Network Module — SAGE gossip networking and peer synchronization.
//!
//! Provides node identity, knowledge diff computation, gossip message types,
//! and the NetworkManager that ties it all together.

pub mod contribution;
pub mod diff;
pub mod direct_protocol;
pub mod gossip;
pub mod identity;
pub mod invite;
pub mod libp2p_transport;
pub mod privacy;
pub mod quarantine;
pub mod security;
pub mod validation;

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

use diff::{merkle_hash, KnowledgeDiff};
use gossip::{
    GossipError, GossipMessage, GossipTransport, GridStateRequest, GridStateResponse, PeerAnnounce,
};
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
    /// Human-readable name like "swift-harbor"
    pub human_name: String,
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
    peers: Arc<RwLock<HashMap<String, PeerInfo>>>,
    /// The local grid state snapshot (for diff computation).
    /// Stored as the last-broadcast state so we can compute incremental diffs.
    last_broadcast_state: Mutex<Option<Vec<Vec<Vec<f64>>>>>,
    /// Monotonic sequence counter for outgoing diffs.
    sequence: Mutex<u64>,
    /// Whether the manager is currently running.
    running: Arc<RwLock<bool>>,
    /// Stats.
    stats: Arc<RwLock<NetworkStats>>,
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
    /// Optional transport — used to respond to GridStateRequest.
    transport: Option<Arc<dyn GossipTransport>>,
    /// Background task handle for the recv loop.
    background_handle: Arc<RwLock<Option<tokio::task::JoinHandle<()>>>>,
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
            peers: Arc::new(RwLock::new(HashMap::new())),
            last_broadcast_state: Mutex::new(None),
            sequence: Mutex::new(0),
            running: Arc::new(RwLock::new(false)),
            stats: Arc::new(RwLock::new(NetworkStats::default())),
            validator: Mutex::new(DiffValidator::new(trust_store)),
            quarantine: Mutex::new(Quarantine::new()),
            privacy_config,
            aggregation: Mutex::new(AggregationTracker::new(aggregation_threshold)),
            pii_filter: Mutex::new(PiiFilter::new()),
            ban_list: Mutex::new(BanList::load()),
            rate_limiter: Mutex::new(RateLimiter::new(RateLimitConfig::default())),
            transport: None,
            background_handle: Arc::new(RwLock::new(None)),
        }
    }

    /// Create a new NetworkManager wired to a transport for responding to sync requests.
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

    /// Start networking.
    ///
    /// Initializes the wired transport and spawns a background task that:
    /// - Loops while `running` is true
    /// - Calls `transport.recv()` to receive incoming messages
    /// - Handles PeerAnnounce messages (registering peers)
    /// - Logs other message types for visibility
    ///
    /// Returns `Err(GossipError::NotStarted)` if no transport is wired.
    pub async fn start(&self) -> Result<(), GossipError> {
        let mut running = self.running.write().await;
        if *running {
            return Ok(());
        }

        // Check transport is wired
        let transport = self.transport.as_ref().ok_or(GossipError::NotStarted)?;

        // Start the underlying transport (libp2p swarm, etc.)
        transport.start().await?;

        *running = true;
        drop(running); // Release lock before spawning task

        // Clone Arc fields for the background task
        let transport = Arc::clone(transport);
        let running_flag = Arc::clone(&self.running);
        let peers = Arc::clone(&self.peers);
        let node_id = self.identity.node_id.clone();
        let node_id_log = node_id.clone();

        let handle = tokio::spawn(async move {
            println!("[network] Background recv loop started for node {}", node_id_log);
            loop {
                // Check if still running
                if !*running_flag.read().await {
                    println!("[network] Background task exiting (stopped)");
                    break;
                }

                match transport.recv().await {
                    Ok((peer_id, msg)) => {
                        let msg_type = msg.type_name();
                        println!("[network] Received {} from {}", msg_type, peer_id);

                        // Handle PeerAnnounce to populate peer map
                        if let GossipMessage::PeerAnnounce(announce) = msg {
                            let info = PeerInfo {
                                node_id: announce.node_id.clone(),
                                human_name: announce.human_name.clone(),
                                public_key: announce.public_key,
                                state_hash: announce.state_hash,
                                last_seen_ms: announce.timestamp_ms,
                                diff_count: announce.diff_count,
                                protocol_version: announce.protocol_version,
                            };
                            peers.write().await.insert(announce.node_id, info);
                            println!("[network] Registered peer: {}", peer_id);
                        }
                    }
                    Err(GossipError::NotStarted) => {
                        // Transport stopped
                        println!("[network] Background task exiting (transport not started)");
                        break;
                    }
                    Err(e) => {
                        eprintln!("[network] recv error: {}", e);
                        // Continue looping on transient errors
                    }
                }
            }
        });

        *self.background_handle.write().await = Some(handle);
        println!("[network] Node {} started", node_id);
        Ok(())
    }

    /// Stop networking.
    pub async fn stop(&self) -> Result<(), GossipError> {
        {
            let mut running = self.running.write().await;
            if !*running {
                return Ok(());
            }
            *running = false;
        }

        // Abort background task if present
        if let Some(handle) = self.background_handle.write().await.take() {
            handle.abort();
        }

        // Stop the transport
        if let Some(ref transport) = self.transport {
            transport.stop().await?;
        }

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

    /// Broadcast a gossip message via the transport.
    pub async fn broadcast(&self, message: GossipMessage) -> Result<(), GossipError> {
        let Some(ref transport) = self.transport else {
            return Err(GossipError::NotStarted);
        };
        transport.broadcast(message).await
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
            human_name: announce.human_name.clone(),
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
            human_name: self.identity.human_name.clone(),
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

        // Sign the diff with this node's Ed25519 key
        diff.sign(self.identity.seed_bytes());

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

        // Verify Ed25519 signature (if present)
        if let Err(e) = diff.verify_signature() {
            println!(
                "[network] Rejected diff from {}: signature verification failed: {}",
                diff.source_node, e
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

    /// Check if aggregation threshold is met (ready to sync).
    /// Does NOT increment the counter.
    pub async fn is_ready_to_sync(&self) -> bool {
        let agg = self.aggregation.lock().await;
        agg.is_ready()
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
            GossipMessage::GridStateResponse(resp) => match resp {
                GridStateResponse::InSync => {
                    println!("[network] Peer confirmed in-sync");
                }
                GridStateResponse::Diff(diff) => {
                    self.handle_incoming_diff(diff, local_grid).await;
                }
                GridStateResponse::FullState {
                    ref node_id,
                    ref grid,
                    confidence,
                    ..
                } => {
                    println!(
                        "[network] Received full state from {node_id} ({} rows)",
                        grid.len()
                    );
                    self.merge_full_state(grid, confidence, local_grid);
                }
            },
        }
    }

    /// Respond to a GridStateRequest using the wired transport.
    ///
    /// - If hashes match: reply InSync.
    /// - If `full_state=true`: reply with our full grid.
    /// - Otherwise: reply with a diff (diff from zeros as best-effort when we don't
    ///   have the peer's actual state).
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
            // Hashes match — nothing to send.
            GossipMessage::GridStateResponse(GridStateResponse::InSync)
        } else if req.full_state {
            // Peer wants our full grid (initial bootstrap).
            GossipMessage::GridStateResponse(GridStateResponse::FullState {
                node_id: self.identity.node_id.clone(),
                grid: local_grid.to_vec(),
                state_hash: local_hash,
                confidence: self.config.local_confidence,
            })
        } else {
            // Send a diff. We don't track per-peer state, so diff from an empty
            // baseline (zeros). This over-sends but is always correct.
            let height = local_grid.len();
            let width = if height > 0 { local_grid[0].len() } else { 0 };
            let channels = if height > 0 && width > 0 {
                local_grid[0][0].len()
            } else {
                0
            };
            let zeros = vec![vec![vec![0.0f64; channels]; width]; height];
            let seq = *self.sequence.lock().await;
            let diff = KnowledgeDiff::compute(
                &zeros,
                local_grid,
                self.identity.node_id.clone(),
                seq,
                self.config.local_confidence,
                self.config.diff_threshold,
            );
            GossipMessage::GridStateResponse(GridStateResponse::Diff(diff))
        };

        if let Err(e) = transport.send_to(&req.requesting_node, response).await {
            eprintln!(
                "[network] Failed to respond to GridStateRequest from {}: {e}",
                req.requesting_node
            );
        }
    }

    /// Merge a full remote grid state into the local grid using weighted averaging.
    /// Only shared channels are merged; private channels remain unchanged.
    fn merge_full_state(
        &self,
        remote_grid: &[Vec<Vec<f64>>],
        remote_confidence: f64,
        local_grid: &mut [Vec<Vec<f64>>],
    ) {
        let total = self.config.local_confidence + remote_confidence;
        if total <= 0.0 {
            return;
        }
        let remote_weight = remote_confidence / total;
        let local_weight = self.config.local_confidence / total;

        let local_h = local_grid.len();
        let local_w = if local_h > 0 { local_grid[0].len() } else { 0 };
        let remote_h = remote_grid.len();
        let remote_w = if remote_h > 0 {
            remote_grid[0].len()
        } else {
            0
        };

        if local_h == 0 || local_w == 0 || remote_h == 0 || remote_w == 0 {
            return;
        }

        use crate::grid::is_shared_channel;

        for (local_row, row) in local_grid.iter_mut().enumerate().take(local_h) {
            for (local_col, cell) in row.iter_mut().enumerate().take(local_w) {
                // Map local coords to remote coords (handles cross-grid-size sync).
                let remote_row = (local_row * remote_h / local_h).min(remote_h - 1);
                let remote_col = (local_col * remote_w / local_w).min(remote_w - 1);

                let num_channels = cell
                    .len()
                    .min(remote_grid[remote_row][remote_col].len());
                for (ch, val) in cell.iter_mut().enumerate().take(num_channels) {
                    if !is_shared_channel(ch) {
                        continue;
                    }
                    let rv = remote_grid[remote_row][remote_col][ch];
                    *val = (*val * local_weight + rv * remote_weight).clamp(-5.0, 5.0);
                }
            }
        }

        let merged_cells: usize = local_h * local_w;
        println!(
            "[network] Merged full state ({remote_h}×{remote_w}) into local grid ({local_h}×{local_w}): {merged_cells} cells updated (shared channels only)"
        );
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
    use crate::network::gossip::{GossipError, GossipMessage, GossipTransport};
    use async_trait::async_trait;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    type GossipMessageQueue = Arc<Mutex<Vec<(String, GossipMessage)>>>;

    /// A mock transport that captures all sent messages.
    struct MockTransport {
        sent: GossipMessageQueue,
    }

    impl MockTransport {
        fn new() -> (Self, GossipMessageQueue) {
            let sent = Arc::new(Mutex::new(Vec::new()));
            (Self { sent: sent.clone() }, sent)
        }
    }

    #[async_trait]
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

    fn make_manager_with_mock() -> (NetworkManager, GossipMessageQueue) {
        let identity = NodeIdentity::generate();
        let config = NetworkConfig::default();
        let (transport, captured) = MockTransport::new();
        let mgr = NetworkManager::with_transport(identity, config, Arc::new(transport));
        (mgr, captured)
    }

    #[tokio::test]
    async fn test_grid_state_request_insync_when_hashes_match() {
        let (mgr, captured) = make_manager_with_mock();
        // Build a trivial grid and compute its hash.
        let grid = make_grid(4, 4, 38, 0.5);
        let hash = merkle_hash(&grid);

        let req = GridStateRequest {
            requesting_node: "peer-aabbcc".to_string(),
            current_hash: hash,
            full_state: false,
        };
        mgr.handle_grid_state_request(req, &grid).await;

        let sent = captured.lock().await;
        assert_eq!(sent.len(), 1, "Should have sent exactly one reply");
        assert_eq!(sent[0].0, "peer-aabbcc");
        assert!(
            matches!(
                sent[0].1,
                GossipMessage::GridStateResponse(GridStateResponse::InSync)
            ),
            "Should reply InSync when hashes match"
        );
    }

    #[tokio::test]
    async fn test_grid_state_request_full_state_when_requested() {
        let (mgr, captured) = make_manager_with_mock();
        let grid = make_grid(4, 4, 38, 0.3);
        let zeros_hash = [0u8; 32]; // different from actual hash

        let req = GridStateRequest {
            requesting_node: "peer-112233".to_string(),
            current_hash: zeros_hash,
            full_state: true,
        };
        mgr.handle_grid_state_request(req, &grid).await;

        let sent = captured.lock().await;
        assert_eq!(sent.len(), 1);
        assert_eq!(sent[0].0, "peer-112233");
        match &sent[0].1 {
            GossipMessage::GridStateResponse(GridStateResponse::FullState { grid: g, .. }) => {
                assert_eq!(g.len(), 4, "Full state grid should have 4 rows");
            }
            other => panic!("Expected FullState, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_grid_state_request_sends_diff_when_hash_differs() {
        let (mgr, captured) = make_manager_with_mock();
        let grid = make_grid(4, 4, 38, 0.7);
        let stale_hash = [0u8; 32];

        let req = GridStateRequest {
            requesting_node: "peer-445566".to_string(),
            current_hash: stale_hash,
            full_state: false,
        };
        mgr.handle_grid_state_request(req, &grid).await;

        let sent = captured.lock().await;
        assert_eq!(sent.len(), 1);
        assert_eq!(sent[0].0, "peer-445566");
        assert!(
            matches!(
                sent[0].1,
                GossipMessage::GridStateResponse(GridStateResponse::Diff(_))
            ),
            "Should reply with a Diff when hashes differ and full_state=false"
        );
    }

    #[tokio::test]
    async fn test_no_transport_does_not_panic() {
        // NetworkManager without transport should not panic on GridStateRequest.
        let identity = NodeIdentity::generate();
        let config = NetworkConfig::default();
        let mgr = NetworkManager::new(identity, config);
        let grid = make_grid(4, 4, 38, 0.1);
        let req = GridStateRequest {
            requesting_node: "peer-000".to_string(),
            current_hash: [0u8; 32],
            full_state: false,
        };
        // Should log an error but not panic.
        mgr.handle_grid_state_request(req, &grid).await;
    }

    #[tokio::test]
    async fn test_merge_full_state_blends_shared_channels() {
        let identity = NodeIdentity::generate();
        let config = NetworkConfig {
            local_confidence: 0.8,
            ..Default::default()
        };
        let mgr = NetworkManager::new(identity, config);

        // Local grid: 0.0 in all channels. Remote: 1.0. remote_confidence=0.2.
        // Expected blend: local_weight=0.8/1.0=0.8, remote_weight=0.2/1.0=0.2.
        let remote = make_grid(4, 4, 38, 1.0);
        let mut local = make_grid(4, 4, 38, 0.0);

        mgr.merge_full_state(&remote, 0.2, &mut local);

        // Channel 0 is shared (RGBA). Check value is ~0.2.
        let blended = local[0][0][0];
        assert!(
            (blended - 0.2).abs() < 1e-9,
            "Shared channel should be blended: expected 0.2, got {blended}"
        );
    }
}
