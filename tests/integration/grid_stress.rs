//! Grid Stress Integration Tests
//!
//! Tests NCA knowledge grid under heavy load: 1000+ items,
//! retrieval quality, collision detection, and encode/decode benchmarks.

use sage::distributed_knowledge::encoder::{encode_text, EncoderConfig};
use sage::distributed_knowledge::{KnowledgeStore, NCAKnowledge};
use std::time::Instant;

#[test]
fn encode_1000_items_without_panic() {
    let mut store = NCAKnowledge::new();

    for i in 0..1000 {
        let text = format!("knowledge item number {} about topic area {}", i, i % 50);
        store.encode(&text, 0.8);
    }

    let active = store.active_knowledge(0.01);
    assert!(
        !active.is_empty(),
        "Should have active knowledge after 1000 encodes"
    );
}

#[test]
fn retrieval_quality_with_many_items() {
    let mut store = NCAKnowledge::new();

    // Encode items across 10 distinct topic areas
    let topics = [
        "astronomy stars planets galaxies universe",
        "biology cells dna genes evolution",
        "chemistry molecules reactions elements bonds",
        "mathematics algebra calculus geometry proofs",
        "physics quantum gravity relativity forces",
        "history ancient civilizations empires wars",
        "literature novels poetry authors fiction",
        "music harmony rhythm melody instruments",
        "geography continents oceans mountains rivers",
        "computer science algorithms data structures programming",
    ];

    for (i, topic) in topics.iter().enumerate() {
        for j in 0..20 {
            let text = format!("{} detail number {} variant {}", topic, j, j % 7);
            store.encode(&text, 0.8);
        }
        eprintln!("Encoded topic {} ({}/10)", i, i + 1);
    }

    // Query each topic and verify we get results
    let mut found_count = 0;
    for topic in &topics {
        let query_word = topic.split_whitespace().next().unwrap();
        let results = store.query(query_word, 5);
        if !results.is_empty() {
            found_count += 1;
        }
    }

    assert!(
        found_count >= 5,
        "Should find results for at least half the topics: found {}/{}",
        found_count,
        topics.len()
    );
}

#[test]
fn grid_capacity_and_collision_detection() {
    let mut store = NCAKnowledge::new();
    let config = EncoderConfig::default();

    let mut position_counts = std::collections::HashMap::new();

    // Track where each item lands
    for i in 0..500 {
        let text = format!("unique item {}", i);
        let features = encode_text(&text, &config);
        let pos = sage::distributed_knowledge::encoder::feature_to_position(
            &features,
            store.grid.width,
            store.grid.height,
        );
        *position_counts.entry(pos).or_insert(0usize) += 1;
        store.encode(&text, 0.8);
    }

    let total_positions = position_counts.len();
    let max_collisions = position_counts.values().max().copied().unwrap_or(0);
    let avg_collisions = 500.0 / total_positions as f64;

    eprintln!("Grid capacity report:");
    eprintln!("  Total unique positions used: {}", total_positions);
    eprintln!("  Max items at one position: {}", max_collisions);
    eprintln!("  Average items per position: {:.2}", avg_collisions);
    eprintln!(
        "  Grid utilization: {:.1}% of {}x{} = {} cells",
        total_positions as f64 / (store.grid.width * store.grid.height) as f64 * 100.0,
        store.grid.width,
        store.grid.height,
        store.grid.width * store.grid.height
    );

    // With 500 items and a 32x32=1024 cell grid, we expect some collisions
    // but not everything in one cell
    assert!(
        total_positions > 1,
        "Should use more than 1 grid position for 500 items"
    );
}

#[test]
fn benchmark_encode_speed() {
    let mut store = NCAKnowledge::new();
    let n = 100;

    let start = Instant::now();
    for i in 0..n {
        let text = format!("benchmark encode item {} with some padding text here", i);
        store.encode(&text, 0.8);
    }
    let elapsed = start.elapsed();

    let per_item_us = elapsed.as_micros() as f64 / n as f64;
    eprintln!(
        "Encode benchmark: {} items in {:?} ({:.1} µs/item, {:.0} items/sec)",
        n,
        elapsed,
        per_item_us,
        n as f64 / elapsed.as_secs_f64()
    );

    // Sanity: encoding should complete (generous limit for debug builds)
    assert!(
        per_item_us < 500_000.0,
        "Encoding too slow: {:.1} µs/item",
        per_item_us
    );
}

#[test]
fn benchmark_query_speed() {
    let mut store = NCAKnowledge::new();

    // Pre-populate
    for i in 0..200 {
        store.encode(&format!("preloaded knowledge item {}", i), 0.8);
    }

    let n = 50;
    let start = Instant::now();
    for i in 0..n {
        let query = format!("query number {}", i % 50);
        let _ = store.query(&query, 5);
    }
    let elapsed = start.elapsed();

    let per_query_us = elapsed.as_micros() as f64 / n as f64;
    eprintln!(
        "Query benchmark: {} queries in {:?} ({:.1} µs/query, {:.0} queries/sec)",
        n,
        elapsed,
        per_query_us,
        n as f64 / elapsed.as_secs_f64()
    );

    assert!(
        per_query_us < 5_000_000.0,
        "Querying too slow: {:.1} µs/query",
        per_query_us
    );
}

#[test]
fn benchmark_diff_speed() {
    let mut node_a = NCAKnowledge::new().with_node_id(1.0);
    let node_b = NCAKnowledge::new().with_node_id(2.0);

    // Populate A
    for i in 0..200 {
        node_a.encode(&format!("diff benchmark item {}", i), 0.8);
    }

    let n = 100;
    let start = Instant::now();
    for _ in 0..n {
        let _ = node_a.diff(&node_b.grid);
    }
    let elapsed = start.elapsed();

    let per_diff_us = elapsed.as_micros() as f64 / n as f64;
    eprintln!(
        "Diff benchmark: {} diffs in {:?} ({:.1} µs/diff)",
        n, elapsed, per_diff_us
    );

    assert!(
        per_diff_us < 5_000_000.0,
        "Diff too slow: {:.1} µs/diff",
        per_diff_us
    );
}

#[test]
fn activation_saturation_under_load() {
    let mut store = NCAKnowledge::new();

    // Track max activation as we add more items
    let mut max_activations = Vec::new();

    for batch in 0..20 {
        for i in 0..50 {
            let text = format!("batch {} item {} saturation test content", batch, i);
            store.encode(&text, 0.9);
        }

        let max_act: f64 = store
            .active_knowledge(0.01)
            .iter()
            .map(|k| k.activation)
            .fold(0.0, f64::max);
        max_activations.push(max_act);
    }

    // Activations should be clamped to [0, 1]
    for (i, &act) in max_activations.iter().enumerate() {
        assert!(
            act <= 1.0 + 1e-10,
            "Activation should not exceed 1.0 at batch {}: got {}",
            i,
            act
        );
    }

    eprintln!("Max activation over 20 batches (50 items each):");
    for (i, act) in max_activations.iter().enumerate() {
        eprintln!("  Batch {}: {:.4}", i, act);
    }
}

#[test]
fn many_items_active_knowledge_count() {
    let mut store = NCAKnowledge::new();

    let mut counts = Vec::new();
    for batch in 0..10 {
        for i in 0..100 {
            store.encode(&format!("item {}_{}", batch, i), 0.7);
        }
        counts.push(store.active_knowledge(0.01).len());
    }

    eprintln!("Active cell counts after each batch of 100:");
    for (i, c) in counts.iter().enumerate() {
        eprintln!("  After {} items: {} active cells", (i + 1) * 100, c);
    }

    // Should have active cells throughout
    assert!(
        counts.last().copied().unwrap_or(0) > 0,
        "Should have active cells after 1000 items"
    );
}

#[test]
fn save_load_large_grid() {
    let mut store = NCAKnowledge::new();

    for i in 0..500 {
        store.encode(&format!("persistence stress item {}", i), 0.8);
    }

    let path = "/tmp/sage_stress_brain.bin";
    let start = Instant::now();
    store.save(path).expect("Save should succeed");
    let save_time = start.elapsed();

    let start = Instant::now();
    let mut loaded = NCAKnowledge::new();
    loaded.load(path).expect("Load should succeed");
    let load_time = start.elapsed();

    eprintln!(
        "Large grid persistence: save={:?} load={:?}",
        save_time, load_time
    );

    let orig_active = store.active_knowledge(0.01).len();
    let loaded_active = loaded.active_knowledge(0.01).len();
    assert_eq!(
        orig_active, loaded_active,
        "Active count should match after load"
    );

    let _ = std::fs::remove_file(path);
}
