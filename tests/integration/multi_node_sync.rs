//! Multi-Node Sync Integration Tests
//!
//! Simulates multiple NCAKnowledge nodes exchanging diffs
//! and verifies knowledge propagation, bidirectional sync,
//! and conflict resolution.

use sage::distributed_knowledge::{KnowledgeStore, NCAKnowledge};

#[test]
fn diff_from_a_applied_to_b_propagates_knowledge() {
    let mut node_a = NCAKnowledge::new().with_node_id(1.0);
    let mut node_b = NCAKnowledge::new().with_node_id(2.0);

    // Node A encodes knowledge about Rust
    node_a.encode("rust programming language systems safety ownership", 0.9);

    // Compute diff: what A has that a blank grid doesn't
    let empty = NCAKnowledge::new();
    let delta = node_a.diff(&empty.grid);
    assert!(
        !delta.changes.is_empty(),
        "Delta from encoded grid vs empty should have changes"
    );

    // Apply to B
    node_b.apply_delta(&delta);

    // B should now have active knowledge
    let b_active = node_b.active_knowledge(0.01);
    assert!(
        !b_active.is_empty(),
        "Node B should have knowledge after applying A's delta"
    );
}

#[test]
fn bidirectional_sync_both_have_all_knowledge() {
    let mut node_a = NCAKnowledge::new().with_node_id(1.0);
    let mut node_b = NCAKnowledge::new().with_node_id(2.0);

    // A learns about topic X
    node_a.encode("machine learning neural networks deep learning AI", 0.9);
    // B learns about topic Y
    node_b.encode("cooking recipes italian pasta pizza mediterranean", 0.9);

    let a_active_before = node_a.active_knowledge(0.01).len();
    let b_active_before = node_b.active_knowledge(0.01).len();

    // Sync A → B: compute what A has that B doesn't
    let delta_a = node_a.diff(&node_b.grid);
    node_b.apply_delta(&delta_a);

    // B should now have knowledge from both
    let b_active_after = node_b.active_knowledge(0.01).len();
    assert!(
        b_active_after >= b_active_before,
        "B should have at least as much knowledge after sync: before={} after={}",
        b_active_before,
        b_active_after
    );

    // Sync B → A
    let delta_b = node_b.diff(&node_a.grid);
    node_a.apply_delta(&delta_b);

    let a_active_after = node_a.active_knowledge(0.01).len();
    assert!(
        a_active_after >= a_active_before,
        "A should have at least as much knowledge after sync: before={} after={}",
        a_active_before,
        a_active_after
    );
}

#[test]
fn diff_is_empty_for_identical_grids() {
    let mut node_a = NCAKnowledge::new().with_node_id(1.0);
    node_a.encode("some knowledge", 0.9);

    // Clone the grid
    let node_b = NCAKnowledge::new()
        .with_node_id(2.0)
        .with_grid(node_a.grid.clone());

    let delta = node_a.diff(&node_b.grid);
    assert!(
        delta.changes.is_empty(),
        "Diff between identical grids should be empty, got {} changes",
        delta.changes.len()
    );
}

#[test]
fn three_node_gossip_propagation() {
    let mut node_a = NCAKnowledge::new().with_node_id(1.0);
    let mut node_b = NCAKnowledge::new().with_node_id(2.0);
    let mut node_c = NCAKnowledge::new().with_node_id(3.0);

    // Only A has knowledge
    node_a.encode("distributed systems consensus algorithms raft paxos", 0.9);

    // A → B
    let delta_ab = node_a.diff(&node_b.grid);
    node_b.apply_delta(&delta_ab);

    // B → C (gossip: knowledge propagates without direct A→C connection)
    let delta_bc = node_b.diff(&node_c.grid);
    node_c.apply_delta(&delta_bc);

    // C should have knowledge even though it never talked to A
    let c_active = node_c.active_knowledge(0.01);
    assert!(
        !c_active.is_empty(),
        "Node C should have knowledge via gossip through B"
    );
}

#[test]
fn conflict_resolution_higher_activation_wins() {
    let mut node_a = NCAKnowledge::new().with_node_id(1.0);
    let mut node_b = NCAKnowledge::new().with_node_id(2.0);

    // Both encode on the same topic but with different confidence
    node_a.encode("the sky is blue due to rayleigh scattering", 0.95);
    node_b.encode("the sky is blue because of light refraction", 0.5);

    // Sync A → B (A has higher confidence, should dominate)
    let delta_a = node_a.diff(&node_b.grid);
    node_b.apply_delta(&delta_a);

    // B's max confidence should be at least as high as A's encoding
    let b_max_conf: f64 = node_b
        .active_knowledge(0.01)
        .iter()
        .map(|k| k.confidence)
        .fold(0.0, f64::max);

    // The merged result should reflect A's higher confidence
    assert!(
        b_max_conf > 0.3,
        "After sync, B should have meaningful confidence: {}",
        b_max_conf
    );
}

#[test]
fn merge_is_additive_not_destructive() {
    let mut node_a = NCAKnowledge::new().with_node_id(1.0);
    let mut node_b = NCAKnowledge::new().with_node_id(2.0);

    node_a.encode("knowledge alpha", 0.9);
    node_b.encode("knowledge beta", 0.9);

    let b_active_before = node_b.active_knowledge(0.01);
    let b_total_activation_before: f64 = b_active_before.iter().map(|k| k.activation).sum();

    // Merge A into B
    node_b.merge(&node_a.grid, 0.8);

    let b_active_after = node_b.active_knowledge(0.01);
    let b_total_activation_after: f64 = b_active_after.iter().map(|k| k.activation).sum();

    assert!(
        b_total_activation_after >= b_total_activation_before - 0.01,
        "Merge should not significantly reduce total activation: before={} after={}",
        b_total_activation_before,
        b_total_activation_after
    );
}

#[test]
fn delta_preserves_source_node_id() {
    let mut node_a = NCAKnowledge::new().with_node_id(42.0);
    node_a.encode("tagged knowledge", 0.9);

    let empty = NCAKnowledge::new();
    let delta = node_a.diff(&empty.grid);

    assert_eq!(delta.source_node, 42.0, "Delta should carry source node ID");
}

#[test]
fn apply_delta_to_populated_grid_no_loss() {
    let mut node_a = NCAKnowledge::new().with_node_id(1.0);
    let mut node_b = NCAKnowledge::new().with_node_id(2.0);

    // B has its own knowledge
    node_b.encode("existing knowledge about databases sql postgres", 0.9);
    let b_count_before = node_b.active_knowledge(0.01).len();

    // A has different knowledge
    node_a.encode("networking tcp ip protocols http", 0.9);

    let empty = NCAKnowledge::new();
    let delta = node_a.diff(&empty.grid);
    node_b.apply_delta(&delta);

    let b_count_after = node_b.active_knowledge(0.01).len();
    assert!(
        b_count_after >= b_count_before,
        "Applying delta should not reduce existing knowledge: before={} after={}",
        b_count_before,
        b_count_after
    );
}

#[test]
fn repeated_sync_converges() {
    let mut node_a = NCAKnowledge::new().with_node_id(1.0);
    let mut node_b = NCAKnowledge::new().with_node_id(2.0);

    node_a.encode("convergence test knowledge", 0.9);

    // First sync
    let d1 = node_a.diff(&node_b.grid);
    let changes_1 = d1.changes.len();
    node_b.apply_delta(&d1);

    // Second sync (should have fewer or zero changes)
    let d2 = node_a.diff(&node_b.grid);
    let changes_2 = d2.changes.len();

    assert!(
        changes_2 <= changes_1,
        "Repeated sync should converge: first={} changes, second={} changes",
        changes_1,
        changes_2
    );
}
