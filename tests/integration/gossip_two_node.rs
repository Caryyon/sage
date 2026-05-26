//! Gossip Two-Node Integration Test
//!
//! Spawns two in-process "nodes" using mock transports.
//! Node A encodes 5 facts, triggers a sync, and verifies Node B receives them.
//!
//! This tests the full gossip protocol path:
//!   NCAKnowledge::encode → KnowledgeDiff → NetworkManager::handle_message
//!   → apply_weighted (merge) → verified via active_knowledge()

use async_trait::async_trait;
use sage::distributed_knowledge::{KnowledgeStore, NCAKnowledge};
use sage::network::diff::KnowledgeDiff;
use sage::network::gossip::{
    GossipError, GossipMessage, GossipTransport, GridStateRequest, GridStateResponse,
};
use sage::network::identity::NodeIdentity;
use sage::network::{NetworkConfig, NetworkManager};
use std::sync::Arc;
use tokio::sync::Mutex;

// ---------------------------------------------------------------------------
// Mock transport
// ---------------------------------------------------------------------------

struct MockTransport {
    sent: Arc<Mutex<Vec<(String, GossipMessage)>>>,
}

impl MockTransport {
    fn new() -> (Self, Arc<Mutex<Vec<(String, GossipMessage)>>>) {
        let sent = Arc::new(Mutex::new(Vec::new()));
        (Self { sent: sent.clone() }, sent)
    }
}

#[async_trait]
impl GossipTransport for MockTransport {
    async fn broadcast(&self, msg: GossipMessage) -> Result<(), GossipError> {
        self.sent
            .lock()
            .await
            .push(("*broadcast*".to_string(), msg));
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

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn nca_grid_to_vec(store: &NCAKnowledge) -> Vec<Vec<Vec<f64>>> {
    let g = &store.grid;
    let ch_count = g.cells[0][0].len();
    let mut out = vec![vec![vec![0.0f64; ch_count]; g.width]; g.height];
    for y in 0..g.height {
        for x in 0..g.width {
            for ch in 0..ch_count {
                out[y][x][ch] = g.cells[y][x][ch];
            }
        }
    }
    out
}

fn vec_to_nca_grid(store: &mut NCAKnowledge, grid_vec: &Vec<Vec<Vec<f64>>>) {
    let ch_count = store.grid.cells[0][0].len();
    for y in 0..store.grid.height {
        for x in 0..store.grid.width {
            for ch in 0..ch_count.min(grid_vec[y][x].len()) {
                store.grid.cells[y][x][ch] = grid_vec[y][x][ch];
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Test 1: Two-node sync via KnowledgeDiff
// ---------------------------------------------------------------------------

#[test]
fn two_node_diff_sync() {
    // --- Build node A with knowledge (sync/blocking work — outside tokio runtime) ---
    let mut node_a = NCAKnowledge::new().with_node_id(1.0);
    let facts = [
        "rust is a systems programming language with memory safety guarantees",
        "machine learning uses neural networks to learn patterns from data",
        "the mitochondria is the powerhouse of the cell biology",
        "Paris is the capital city of France in western Europe",
        "quantum entanglement is a phenomenon in quantum physics research",
    ];
    for fact in &facts {
        node_a.encode(fact, 0.9);
    }

    // Verify A can query
    let a_count = node_a.active_knowledge(0.01).len();
    assert!(
        a_count > 0,
        "Node A should have active knowledge after encoding"
    );

    // Node B starts empty
    let mut node_b = NCAKnowledge::new().with_node_id(2.0);
    let b_before = node_b.active_knowledge(0.01).len();

    // Compute diff: what A has that empty doesn't
    let a_grid = nca_grid_to_vec(&node_a);
    let empty = vec![
        vec![vec![0.0f64; node_a.grid.cells[0][0].len()]; node_a.grid.width];
        node_a.grid.height
    ];
    let diff = KnowledgeDiff::compute(
        &empty,
        &a_grid,
        "node-a".to_string(),
        1,
        0.9,
        0.01, // threshold
    );

    assert!(
        !diff.changes.is_empty(),
        "Diff from A's encoded knowledge should have changes (got 0)"
    );

    // Apply diff to B's grid representation
    let mut b_grid = nca_grid_to_vec(&node_b);
    diff.apply_weighted(&mut b_grid, 0.8);
    vec_to_nca_grid(&mut node_b, &b_grid);

    // Verify B now has knowledge
    let b_after = node_b.active_knowledge(0.01).len();
    assert!(
        b_after > b_before,
        "Node B should have more active knowledge after sync: before={} after={}",
        b_before,
        b_after
    );

    println!(
        "✅ Two-node diff sync: A={} active, B before={} after={}",
        a_count, b_before, b_after
    );
}

// ---------------------------------------------------------------------------
// Test 2: NetworkManager handles GridStateRequest → sends FullState response
// ---------------------------------------------------------------------------

#[test]
fn network_manager_full_state_request() {
    // Build node A grid (sync, outside tokio)
    let mut node_a = NCAKnowledge::new().with_node_id(1.0);
    node_a.encode("deep learning transformer attention mechanism", 0.9);
    node_a.encode(
        "evolutionary algorithms genetic programming optimization",
        0.85,
    );
    node_a.encode(
        "cryptography elliptic curves public key infrastructure",
        0.9,
    );
    node_a.encode("quantum computing superposition entanglement qubits", 0.8);
    node_a.encode("blockchain distributed ledger consensus proof of work", 0.9);

    let a_grid = nca_grid_to_vec(&node_a);

    // Create NetworkManager for A wired with mock transport
    let identity_a = NodeIdentity::generate();
    let (transport_a, sent_a) = MockTransport::new();
    let mgr_a =
        NetworkManager::with_transport(identity_a, NetworkConfig::default(), Arc::new(transport_a));

    // Build request: B wants full state
    let req = GridStateRequest {
        requesting_node: "node-b-test".to_string(),
        current_hash: [0u8; 32], // empty hash → mismatch
        full_state: true,
    };

    // Run async handler in a fresh single-threaded runtime
    let rt = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();

    rt.block_on(async {
        mgr_a.handle_grid_state_request(req, &a_grid).await;
    });

    // Verify the response was sent
    let sent = rt.block_on(async { sent_a.lock().await.clone() });
    assert_eq!(sent.len(), 1, "A should have sent exactly one reply");
    assert_eq!(sent[0].0, "node-b-test");

    match &sent[0].1 {
        GossipMessage::GridStateResponse(GridStateResponse::FullState { grid, .. }) => {
            assert_eq!(grid.len(), node_a.grid.height, "Grid rows should match");
            assert!(grid[0].len() > 0, "Grid columns should be non-empty");
            println!(
                "✅ NetworkManager FullState: sent {}×{} grid to node-b",
                grid.len(),
                grid[0].len()
            );
        }
        other => panic!("Expected FullState response, got {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// Test 3: Knowledge diff roundtrip through GossipMessage serialization
// ---------------------------------------------------------------------------

#[test]
fn knowledge_diff_gossip_roundtrip() {
    // Create a non-trivial diff (sync, no async needed)
    let mut store = NCAKnowledge::new();
    store.encode("test knowledge for gossip roundtrip serialization", 0.9);
    store.encode("another fact about networking and protocols", 0.8);

    let grid = nca_grid_to_vec(&store);
    let empty =
        vec![vec![vec![0.0f64; store.grid.cells[0][0].len()]; store.grid.width]; store.grid.height];

    let diff = KnowledgeDiff::compute(&empty, &grid, "test-node".to_string(), 42, 0.9, 0.01);
    let original_changes = diff.changes.len();
    assert!(original_changes > 0, "Diff should have changes");

    // Serialize through GossipMessage
    let msg = GossipMessage::KnowledgeDiff(diff.clone());
    let bytes = msg.to_bytes();
    assert!(!bytes.is_empty(), "Serialized message should not be empty");

    // Deserialize
    let decoded = GossipMessage::from_bytes(&bytes).expect("Should deserialize cleanly");
    match decoded {
        GossipMessage::KnowledgeDiff(d) => {
            assert_eq!(
                d.changes.len(),
                original_changes,
                "Changes should be preserved through serialization"
            );
            assert_eq!(d.source_node, "test-node");
            assert_eq!(d.sequence, 42);

            // Apply the deserialized diff to an empty grid
            let mut target = vec![
                vec![vec![0.0f64; store.grid.cells[0][0].len()]; store.grid.width];
                store.grid.height
            ];
            d.apply_weighted(&mut target, 0.8);

            // Verify some cells were modified
            let modified: usize = target
                .iter()
                .flat_map(|row| row.iter())
                .flat_map(|cell| cell.iter())
                .filter(|&&v| v.abs() > 1e-10)
                .count();
            assert!(
                modified > 0,
                "Applying deserialized diff should modify cells"
            );

            println!(
                "✅ Gossip roundtrip: {} changes → {} bytes → {} cells modified",
                original_changes,
                bytes.len(),
                modified
            );
        }
        other => panic!("Expected KnowledgeDiff, got {}", other.type_name()),
    }
}

// ---------------------------------------------------------------------------
// Test 4: InSync response when hashes match
// ---------------------------------------------------------------------------

#[test]
fn network_manager_insync_when_hashes_match() {
    use sage::network::diff::merkle_hash;

    let mut store = NCAKnowledge::new();
    store.encode("some knowledge to hash", 0.9);
    let grid = nca_grid_to_vec(&store);
    let hash = merkle_hash(&grid);

    let identity = NodeIdentity::generate();
    let (transport, sent) = MockTransport::new();
    let mgr =
        NetworkManager::with_transport(identity, NetworkConfig::default(), Arc::new(transport));

    let req = GridStateRequest {
        requesting_node: "peer-xyz".to_string(),
        current_hash: hash, // same hash
        full_state: false,
    };

    let rt = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    rt.block_on(async { mgr.handle_grid_state_request(req, &grid).await });

    let sent_msgs = rt.block_on(async { sent.lock().await.clone() });
    assert_eq!(sent_msgs.len(), 1, "Should send one reply");
    assert!(
        matches!(
            sent_msgs[0].1,
            GossipMessage::GridStateResponse(GridStateResponse::InSync)
        ),
        "Should reply InSync when hashes match"
    );
    println!("✅ NetworkManager: correctly replies InSync when hashes match");
}
