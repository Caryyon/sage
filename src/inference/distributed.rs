//! Distributed Inference Module
//!
//! Accelerates SAGE inference by leveraging connected peers:
//! 1. **Parallel Knowledge Retrieval** — fan out queries to all peers, combine results
//! 2. **Speculative Decoding** — send prompts to peers, accept matching tokens
//!
//! Protocol extensions (TCP line-based):
//!   `KNOWLEDGE_QUERY <text>\n`   → `KNOWLEDGE_RESULT <json>\n`
//!   `SPECULATE <prompt_hash> <tokens>\n`

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::sync::Mutex;

/// Timeout for peer knowledge queries (500ms)
const PEER_QUERY_TIMEOUT: Duration = Duration::from_millis(500);

/// Maximum peers to use for speculative decoding
const MAX_SPECULATE_PEERS: usize = 2;

/// A knowledge item returned from a peer
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PeerKnowledgeItem {
    pub text: String,
    pub relevance: f64,
    pub confidence: f64,
    pub source_node: String,
    pub position: (usize, usize),
}

/// Result of a distributed knowledge query
#[derive(Clone, Debug, Default)]
pub struct DistributedKnowledgeResult {
    /// Combined, deduplicated, ranked knowledge items
    pub items: Vec<PeerKnowledgeItem>,
    /// Number of peers that responded
    pub peers_responded: usize,
    /// Total peers queried
    pub peers_queried: usize,
    /// Time taken for the distributed query
    pub query_time: Duration,
    /// Items from local node
    pub local_count: usize,
    /// Items from remote peers
    pub remote_count: usize,
}

/// Speculative decoding result from a peer
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SpeculationResult {
    pub prompt_hash: u64,
    pub tokens: Vec<String>,
    pub source_node: String,
}

/// Metrics for distributed inference
#[derive(Debug, Default)]
pub struct DistributedMetrics {
    /// Total queries performed
    pub total_queries: AtomicU64,
    /// Queries that used distributed retrieval
    pub distributed_queries: AtomicU64,
    /// Total knowledge items retrieved from peers
    pub peer_items_retrieved: AtomicU64,
    /// Total knowledge items retrieved locally
    pub local_items_retrieved: AtomicU64,
    /// Cumulative time saved (ms) from distributed retrieval
    pub time_saved_ms: AtomicU64,
    /// Speculation attempts
    pub speculation_attempts: AtomicU64,
    /// Successful speculation hits (tokens matched)
    pub speculation_hits: AtomicU64,
    /// Total tokens speculated
    pub tokens_speculated: AtomicU64,
}

impl DistributedMetrics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn speculation_hit_rate(&self) -> f64 {
        let attempts = self.speculation_attempts.load(Ordering::Relaxed);
        if attempts == 0 {
            return 0.0;
        }
        self.speculation_hits.load(Ordering::Relaxed) as f64 / attempts as f64
    }

    pub fn avg_peer_items_per_query(&self) -> f64 {
        let queries = self.distributed_queries.load(Ordering::Relaxed);
        if queries == 0 {
            return 0.0;
        }
        self.peer_items_retrieved.load(Ordering::Relaxed) as f64 / queries as f64
    }
}

/// Manages distributed inference across connected peers
pub struct DistributedInference {
    /// Peer addresses (host:port) for TCP connections
    peers: Arc<Mutex<Vec<PeerConnection>>>,
    /// Inference metrics
    pub metrics: Arc<DistributedMetrics>,
}

/// A connection to a peer node
struct PeerConnection {
    pub address: String,
    pub node_id: String,
    pub last_seen: Instant,
}

impl Default for DistributedInference {
    fn default() -> Self {
        Self::new()
    }
}

impl DistributedInference {
    pub fn new() -> Self {
        Self {
            peers: Arc::new(Mutex::new(Vec::new())),
            metrics: Arc::new(DistributedMetrics::new()),
        }
    }

    /// Register a peer for distributed inference
    pub async fn add_peer(&self, address: String, node_id: String) {
        let mut peers = self.peers.lock().await;
        // Update existing or add new
        if let Some(p) = peers.iter_mut().find(|p| p.node_id == node_id) {
            p.address = address;
            p.last_seen = Instant::now();
        } else {
            peers.push(PeerConnection {
                address,
                node_id,
                last_seen: Instant::now(),
            });
        }
    }

    /// Remove a peer
    pub async fn remove_peer(&self, node_id: &str) {
        let mut peers = self.peers.lock().await;
        peers.retain(|p| p.node_id != node_id);
    }

    /// Get number of connected peers
    pub async fn peer_count(&self) -> usize {
        self.peers.lock().await.len()
    }

    /// Fan out a knowledge query to all peers in parallel.
    /// Returns combined, deduplicated, relevance-ranked results.
    pub async fn query_peers(
        &self,
        query: &str,
        max_results: usize,
    ) -> DistributedKnowledgeResult {
        let start = Instant::now();
        let peers = self.peers.lock().await;
        let peer_count = peers.len();

        if peer_count == 0 {
            return DistributedKnowledgeResult {
                query_time: start.elapsed(),
                ..Default::default()
            };
        }

        // Fan out queries to all peers simultaneously
        let mut handles = Vec::new();
        for peer in peers.iter() {
            let addr = peer.address.clone();
            let node_id = peer.node_id.clone();
            let q = query.to_string();
            handles.push(tokio::spawn(async move {
                query_single_peer(&addr, &node_id, &q).await
            }));
        }
        drop(peers); // Release lock before awaiting

        // Collect results with timeout
        let mut all_items: Vec<PeerKnowledgeItem> = Vec::new();
        let mut responded = 0usize;

        for handle in handles {
            match tokio::time::timeout(PEER_QUERY_TIMEOUT, handle).await {
                Ok(Ok(Ok(items))) => {
                    responded += 1;
                    all_items.extend(items);
                }
                _ => {} // Timeout or error — skip this peer
            }
        }

        // Deduplicate by text similarity (exact match for now)
        let deduped = deduplicate_knowledge(all_items);

        // Sort by relevance descending
        let mut ranked = deduped;
        ranked.sort_by(|a, b| b.relevance.partial_cmp(&a.relevance).unwrap_or(std::cmp::Ordering::Equal));
        ranked.truncate(max_results);

        let remote_count = ranked.len();
        let query_time = start.elapsed();

        // Update metrics
        self.metrics.distributed_queries.fetch_add(1, Ordering::Relaxed);
        self.metrics.peer_items_retrieved.fetch_add(remote_count as u64, Ordering::Relaxed);
        self.metrics.total_queries.fetch_add(1, Ordering::Relaxed);

        DistributedKnowledgeResult {
            items: ranked,
            peers_responded: responded,
            peers_queried: peer_count,
            query_time,
            local_count: 0, // Caller fills this in
            remote_count,
        }
    }

    /// Send a speculative decoding request to up to MAX_SPECULATE_PEERS peers.
    /// Returns any speculation results received within timeout.
    pub async fn speculate(
        &self,
        prompt_hash: u64,
        prompt_text: &str,
    ) -> Vec<SpeculationResult> {
        let peers = self.peers.lock().await;
        if peers.is_empty() {
            return Vec::new();
        }

        let count = peers.len().min(MAX_SPECULATE_PEERS);
        let mut handles = Vec::new();

        for peer in peers.iter().take(count) {
            let addr = peer.address.clone();
            let node_id = peer.node_id.clone();
            let hash = prompt_hash;
            let text = prompt_text.to_string();
            handles.push(tokio::spawn(async move {
                speculate_on_peer(&addr, &node_id, hash, &text).await
            }));
        }
        drop(peers);

        self.metrics.speculation_attempts.fetch_add(1, Ordering::Relaxed);

        let mut results = Vec::new();
        for handle in handles {
            if let Ok(Ok(Ok(result))) = tokio::time::timeout(PEER_QUERY_TIMEOUT, handle).await {
                self.metrics.tokens_speculated.fetch_add(result.tokens.len() as u64, Ordering::Relaxed);
                results.push(result);
            }
        }

        if !results.is_empty() {
            self.metrics.speculation_hits.fetch_add(1, Ordering::Relaxed);
        }

        results
    }

    /// Get a snapshot of current metrics
    pub fn metrics_snapshot(&self) -> MetricsSnapshot {
        MetricsSnapshot {
            total_queries: self.metrics.total_queries.load(Ordering::Relaxed),
            distributed_queries: self.metrics.distributed_queries.load(Ordering::Relaxed),
            peer_items_retrieved: self.metrics.peer_items_retrieved.load(Ordering::Relaxed),
            local_items_retrieved: self.metrics.local_items_retrieved.load(Ordering::Relaxed),
            time_saved_ms: self.metrics.time_saved_ms.load(Ordering::Relaxed),
            speculation_attempts: self.metrics.speculation_attempts.load(Ordering::Relaxed),
            speculation_hits: self.metrics.speculation_hits.load(Ordering::Relaxed),
            tokens_speculated: self.metrics.tokens_speculated.load(Ordering::Relaxed),
            speculation_hit_rate: self.metrics.speculation_hit_rate(),
        }
    }
}

/// Serializable metrics snapshot
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MetricsSnapshot {
    pub total_queries: u64,
    pub distributed_queries: u64,
    pub peer_items_retrieved: u64,
    pub local_items_retrieved: u64,
    pub time_saved_ms: u64,
    pub speculation_attempts: u64,
    pub speculation_hits: u64,
    pub tokens_speculated: u64,
    pub speculation_hit_rate: f64,
}

/// Query a single peer for knowledge via TCP
async fn query_single_peer(
    address: &str,
    node_id: &str,
    query: &str,
) -> Result<Vec<PeerKnowledgeItem>, String> {
    let stream = TcpStream::connect(address)
        .await
        .map_err(|e| format!("connect to {}: {}", address, e))?;

    let (reader, mut writer) = stream.into_split();
    let mut lines = BufReader::new(reader).lines();

    // Send knowledge query
    let cmd = format!("KNOWLEDGE_QUERY {}\n", query.replace('\n', " "));
    writer.write_all(cmd.as_bytes()).await.map_err(|e| e.to_string())?;

    // Read response
    let mut items = Vec::new();
    while let Ok(Some(line)) = lines.next_line().await {
        let line = line.trim().to_string();
        if line == "DONE" {
            break;
        }
        if let Some(json_str) = line.strip_prefix("KNOWLEDGE_RESULT ") {
            if let Ok(mut item) = serde_json::from_str::<PeerKnowledgeItem>(json_str) {
                if item.source_node.is_empty() {
                    item.source_node = node_id.to_string();
                }
                items.push(item);
            }
        }
    }

    // Disconnect
    let _ = writer.write_all(b"QUIT\n").await;

    Ok(items)
}

/// Send a speculative decoding request to a peer
async fn speculate_on_peer(
    address: &str,
    node_id: &str,
    prompt_hash: u64,
    prompt_text: &str,
) -> Result<SpeculationResult, String> {
    let stream = TcpStream::connect(address)
        .await
        .map_err(|e| format!("connect to {}: {}", address, e))?;

    let (reader, mut writer) = stream.into_split();
    let mut lines = BufReader::new(reader).lines();

    // Send speculation request
    let cmd = format!(
        "SPECULATE {} {}\n",
        prompt_hash,
        prompt_text.replace('\n', "\\n")
    );
    writer.write_all(cmd.as_bytes()).await.map_err(|e| e.to_string())?;

    // Read response tokens
    let mut tokens = Vec::new();
    while let Ok(Some(line)) = lines.next_line().await {
        let line = line.trim().to_string();
        if line == "DONE" {
            break;
        }
        if let Some(token_data) = line.strip_prefix("TOKEN ") {
            tokens.push(token_data.replace("\\n", "\n"));
        }
    }

    let _ = writer.write_all(b"QUIT\n").await;

    Ok(SpeculationResult {
        prompt_hash,
        tokens,
        source_node: node_id.to_string(),
    })
}

/// Deduplicate knowledge items by exact text match, keeping highest relevance
fn deduplicate_knowledge(items: Vec<PeerKnowledgeItem>) -> Vec<PeerKnowledgeItem> {
    let mut seen: HashMap<String, PeerKnowledgeItem> = HashMap::new();
    for item in items {
        let key = item.text.clone();
        let entry = seen.entry(key).or_insert_with(|| item.clone());
        if item.relevance > entry.relevance {
            *entry = item;
        }
    }
    seen.into_values().collect()
}

/// Compute a simple hash for prompt deduplication
pub fn prompt_hash(text: &str) -> u64 {
    let mut h: u64 = 5381;
    for b in text.bytes() {
        h = h.wrapping_mul(33).wrapping_add(b as u64);
    }
    h
}

/// Handle an incoming KNOWLEDGE_QUERY from a peer.
/// Called by sage-node when it receives this protocol message.
pub fn handle_knowledge_query(
    knowledge: &crate::distributed_knowledge::NCAKnowledge,
    query: &str,
    max_results: usize,
    node_id: &str,
) -> Vec<PeerKnowledgeItem> {
    use crate::distributed_knowledge::KnowledgeStore;
    let results = knowledge.query(query, max_results);
    results
        .into_iter()
        .filter_map(|k| {
            k.text.map(|text| PeerKnowledgeItem {
                text,
                relevance: k.relevance,
                confidence: k.confidence,
                source_node: node_id.to_string(),
                position: k.position,
            })
        })
        .collect()
}

/// Stats tracking for distributed inference within a node
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct InferenceStats {
    /// Total chat requests handled
    pub total_requests: u64,
    /// Requests that used distributed knowledge
    pub distributed_requests: u64,
    /// Knowledge items served to other peers
    pub knowledge_served: u64,
    /// Speculation requests handled
    pub speculation_requests: u64,
    /// Time saved from distributed retrieval (estimated ms)
    pub estimated_time_saved_ms: u64,
}

impl InferenceStats {
    pub fn new() -> Self {
        Self::default()
    }
}
