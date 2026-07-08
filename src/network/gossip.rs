//! Gossip Protocol — Message types and protocol definitions for SAGE networking.
//!
//! Defines the message types exchanged between SAGE nodes.
//! The actual libp2p transport is stubbed — this module provides the types and traits.

use super::diff::KnowledgeDiff;
use serde::{Deserialize, Serialize};

/// Topic names for GossipSub channels.
pub const TOPIC_KNOWLEDGE: &str = "/sage/knowledge/1.0";
pub const TOPIC_ANNOUNCE: &str = "/sage/announce/1.0";
pub const TOPIC_GRID_SYNC: &str = "/sage/grid-sync/1.0";
pub const TOPIC_HDC_SYNC: &str = "/sage/hdc-sync/1.0";

/// All message types that can be sent between SAGE nodes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GossipMessage {
    /// A knowledge diff to be merged into the receiving node's grid.
    KnowledgeDiff(KnowledgeDiff),

    /// Announce presence on the network.
    PeerAnnounce(PeerAnnounce),

    /// Request another node's grid state (or Merkle hash for comparison).
    GridStateRequest(GridStateRequest),

    /// Response with grid state information.
    GridStateResponse(GridStateResponse),

    /// HDC knowledge sync — share episodic memory entries with peers.
    /// Each entry is an embedding + text + metadata. The receiver merges
    /// with dedup. This is the "BitTorrent for intelligence" — nodes share
    /// what they've learned without sharing raw source documents.
    HdcSync(HdcSyncBatch),

    /// Request HDC entries from a peer (for initial sync or catch-up).
    HdcSyncRequest(HdcSyncRequest),
}

/// Peer announcement — broadcast when a node joins or periodically.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerAnnounce {
    pub node_id: String,
    /// Human-readable name like "swift-harbor"
    #[serde(default)]
    pub human_name: String,
    pub public_key: [u8; 32],
    /// Merkle hash of this node's current grid state.
    pub state_hash: [u8; 32],
    /// Grid dimensions.
    pub grid_width: usize,
    pub grid_height: usize,
    pub grid_channels: usize,
    /// How many diffs this node has applied.
    pub diff_count: u64,
    /// Unix timestamp millis.
    pub timestamp_ms: u64,
    /// Supported protocol version.
    pub protocol_version: u32,
}

impl PeerAnnounce {
    pub const CURRENT_PROTOCOL_VERSION: u32 = 1;
}

/// A batch of HDC entries to share with peers.
///
/// Each entry contains the full embedding vector, the source text, and metadata.
/// The receiver merges these into its own HDC store with deduplication.
///
/// **Privacy note:** Only the embedding (math) and text excerpt are shared —
/// not the original source document. This is federated learning: nodes share
/// what they've learned, not what they've read.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HdcSyncBatch {
    /// Source node ID
    pub source_node: String,
    /// Embedding dimension
    pub dim: usize,
    /// The entries to share (embedding + text + metadata)
    pub entries: Vec<HdcSyncEntry>,
    /// Sequence number for ordering
    pub sequence: u64,
    /// Timestamp (Unix epoch ms)
    pub timestamp_ms: u64,
}

/// A single HDC entry for network sync.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HdcSyncEntry {
    /// Full-dimensional embedding vector
    pub embedding: Vec<f32>,
    /// Source text excerpt
    pub text: String,
    /// Confidence score (0.0-1.0)
    pub confidence: f32,
    /// Timestamp (Unix epoch seconds)
    pub timestamp: u64,
}

/// Request HDC entries from a peer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HdcSyncRequest {
    /// Requesting node ID
    pub requesting_node: String,
    /// Optional: only send entries newer than this timestamp
    pub since_timestamp: u64,
    /// Maximum number of entries to send
    pub max_entries: usize,
}

/// Request for grid state — used for initial sync or after detecting hash mismatch.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridStateRequest {
    pub requesting_node: String,
    /// The requester's current state hash (so responder can compute a diff).
    pub current_hash: [u8; 32],
    /// If true, send the full grid state instead of a diff.
    pub full_state: bool,
}

/// Response to a grid state request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GridStateResponse {
    /// Hashes match — no sync needed.
    InSync,
    /// Here's a diff to bring you up to date.
    Diff(KnowledgeDiff),
    /// Full grid state (for initial sync or large divergence).
    FullState {
        node_id: String,
        grid: Vec<Vec<Vec<f64>>>,
        state_hash: [u8; 32],
        confidence: f64,
    },
}

impl GossipMessage {
    /// Serialize to bytes for network transmission.
    pub fn to_bytes(&self) -> Vec<u8> {
        bincode::serialize(self).expect("failed to serialize gossip message")
    }

    /// Deserialize from bytes.
    pub fn from_bytes(data: &[u8]) -> Result<Self, bincode::Error> {
        bincode::deserialize(data)
    }

    /// Get a human-readable type name.
    pub fn type_name(&self) -> &'static str {
        match self {
            Self::KnowledgeDiff(_) => "KnowledgeDiff",
            Self::PeerAnnounce(_) => "PeerAnnounce",
            Self::GridStateRequest(_) => "GridStateRequest",
            Self::GridStateResponse(_) => "GridStateResponse",
            Self::HdcSync(_) => "HdcSync",
            Self::HdcSyncRequest(_) => "HdcSyncRequest",
        }
    }
}

/// Trait for a gossip transport backend.
/// This allows swapping out the actual networking (libp2p, mock, etc.).
#[async_trait::async_trait]
pub trait GossipTransport: Send + Sync {
    /// Broadcast a message to all connected peers.
    async fn broadcast(&self, message: GossipMessage) -> Result<(), GossipError>;

    /// Send a message to a specific peer.
    async fn send_to(&self, peer_id: &str, message: GossipMessage) -> Result<(), GossipError>;

    /// Receive the next incoming message (blocks until available).
    async fn recv(&self) -> Result<(String, GossipMessage), GossipError>;

    /// Get list of currently connected peer IDs.
    async fn connected_peers(&self) -> Vec<String>;

    /// Start the transport (begin listening and discovering peers).
    async fn start(&self) -> Result<(), GossipError>;

    /// Stop the transport.
    async fn stop(&self) -> Result<(), GossipError>;
}

/// Errors that can occur during gossip operations.
#[derive(Debug, Clone)]
pub enum GossipError {
    /// Serialization/deserialization failed.
    SerdeError(String),
    /// Network transport error.
    TransportError(String),
    /// Peer not found.
    PeerNotFound(String),
    /// Transport not started.
    NotStarted,
}

impl std::fmt::Display for GossipError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SerdeError(e) => write!(f, "serialization error: {e}"),
            Self::TransportError(e) => write!(f, "transport error: {e}"),
            Self::PeerNotFound(id) => write!(f, "peer not found: {id}"),
            Self::NotStarted => write!(f, "transport not started"),
        }
    }
}

impl std::error::Error for GossipError {}
