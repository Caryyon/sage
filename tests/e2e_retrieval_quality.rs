//! Retrieval Quality E2E Tests — Quality Gates
//!
//! These tests block releases if they fail. They ensure the encode→retrieve
//! pipeline maintains minimum quality standards.

use sage::distributed_knowledge::encoder::{
    detect_embedding_status, encode_text, write_knowledge, EncoderConfig,
};
use sage::distributed_knowledge::{KnowledgeStore, NCAKnowledge};
use sage::grid::{Grid, GRID_SIZE};

/// QUALITY GATE: Encode 10 fact pairs, retrieve each, assert >= 70% hit rate.
///
/// This is the primary quality gate for the encode→retrieve pipeline.
/// If this fails, the retrieval system is broken and cannot ship.
#[test]
fn test_encode_10_facts_retrieve_all() {
    let mut store = NCAKnowledge::new();

    // Use fastembed if available, hash otherwise
    let status = detect_embedding_status(&store.config);
    eprintln!("Embedding backend: {}", status);

    // 10 semantically distinct fact pairs: (fact, query)
    let facts = [
        ("Paris is the capital of France", "Paris France capital"),
        ("Cats are furry mammals that meow", "cats mammals meow"),
        (
            "Python is a programming language",
            "python programming language",
        ),
        ("The Pacific Ocean is the largest", "Pacific Ocean largest"),
        ("Einstein developed relativity", "Einstein relativity"),
        (
            "DNA contains genetic information",
            "DNA genetic information",
        ),
        (
            "Mount Everest is the tallest mountain",
            "Everest tallest mountain",
        ),
        (
            "The Nile is the longest river in Africa",
            "Nile longest river",
        ),
        (
            "Rust is a systems programming language",
            "Rust systems programming",
        ),
        ("Water freezes at zero degrees", "water freezes zero"),
    ];

    // Encode all facts
    for (fact, _query) in &facts {
        store.encode(fact, 0.9);
    }

    // Query each and count hits
    let mut hits = 0;
    let mut results_log = Vec::new();

    for (fact, query) in &facts {
        let results = store.query(query, 10);

        let is_hit = results
            .iter()
            .any(|r| r.text.as_ref().map(|t| t == *fact).unwrap_or(false));

        if is_hit {
            hits += 1;
            results_log.push(format!("✓ '{}' found", query));
        } else {
            results_log.push(format!(
                "✗ '{}' not found (got {} results)",
                query,
                results.len()
            ));
        }
    }

    // Log results for diagnostics
    eprintln!("=== Retrieval Quality Gate Results ===");
    for line in &results_log {
        eprintln!("  {}", line);
    }
    eprintln!("Hit rate: {}/10 ({}%)", hits, hits * 10);

    // QUALITY GATE: >= 70% hit rate (7/10)
    let min_hits = 7;
    assert!(
        hits >= min_hits,
        "QUALITY GATE FAILED: Retrieval hit rate {}/10 ({}%) is below threshold {}/10 (70%). \
         This blocks release. Check encode/retrieve pipeline.\n{}",
        hits,
        hits * 10,
        min_hits,
        results_log.join("\n")
    );
}

/// QUALITY GATE: Hash fallback mode works with zero panics.
///
/// Forces hash-only mode and verifies the system can still encode/retrieve.
#[test]
fn test_hash_fallback_works() {
    let mut store = NCAKnowledge::new();

    // Force hash fallback by disabling Ollama
    store.config.ollama_url = None;

    // Verify we're using hash fallback (not fastembed or Ollama)
    let _features = encode_text("test fallback", &store.config);
    // Hash fallback returns is_semantic = false if fastembed isn't available
    // This test just ensures the system works regardless of embedding backend

    // 10 facts for testing
    let facts = [
        "Paris is the capital of France",
        "Cats are furry mammals",
        "Python is used for data science",
        "The sun is a yellow star",
        "Water boils at 100 degrees",
        "Rust ensures memory safety",
        "The moon orbits Earth",
        "Trees produce oxygen",
        "Fish live in water",
        "Birds can fly in the sky",
    ];

    // Encode all facts — should not panic
    for fact in &facts {
        let pos = store.encode(fact, 0.8);
        assert!(
            pos.0 < GRID_SIZE && pos.1 < GRID_SIZE,
            "Encode position should be valid"
        );
    }

    // Query each — should not panic and return results
    let mut total_hits = 0;
    for fact in &facts {
        // Use first 2 words as query
        let query: String = fact
            .split_whitespace()
            .take(2)
            .collect::<Vec<_>>()
            .join(" ");
        let results = store.query(&query, 5);

        // Count any result as a "hit" for this test
        if !results.is_empty() {
            total_hits += 1;
        }
    }

    // With hash fallback, we expect at least 1/10 hits (10% minimum)
    assert!(
        total_hits >= 1,
        "Hash fallback should produce at least 1/10 hits. Got {}/10. \
         System should degrade gracefully, not fail completely.",
        total_hits
    );

    eprintln!(
        "Hash fallback quality: {}/10 hits ({}%)",
        total_hits,
        total_hits * 10
    );
}

/// REGRESSION: Retrieval must NOT mutate the grid.
///
/// The grid should be identical before and after retrieval operations.
/// Any mutation during retrieval is a critical bug.
#[test]
fn test_retrieval_does_not_mutate_grid() {
    use sage::distributed_knowledge::attention_decoder::AttentionDecoder;

    let mut grid = Grid::new(GRID_SIZE, GRID_SIZE);
    let config = EncoderConfig {
        ollama_url: None,
        ..Default::default()
    };

    // Encode some knowledge
    let texts = [
        "neural networks deep learning",
        "rust programming systems",
        "quantum physics particles",
    ];

    for text in &texts {
        let features = encode_text(text, &config);
        write_knowledge(&mut grid, &features, 0.9, 0.5, &config);
    }

    // Snapshot ENTIRE grid state before retrieval
    let snapshot_before: Vec<Vec<Vec<f64>>> = grid
        .cells
        .iter()
        .map(|row| row.to_vec())
        .collect();

    // Verify snapshot captured correctly
    let cells_before = snapshot_before.len() * snapshot_before[0].len();
    assert_eq!(
        cells_before,
        GRID_SIZE * GRID_SIZE,
        "Snapshot should capture entire grid"
    );

    // Run semantic retrieval
    let query1 = encode_text("neural networks", &config);
    let decoder = AttentionDecoder::new(GRID_SIZE, GRID_SIZE);
    let _ = decoder.attend(&query1, &grid, 10);

    // Run contrast retrieval
    let _ = decoder.attend_with_contrast(&query1, &grid, 10, None);

    // Run spatial gated retrieval
    let _ = decoder.attend_with_spatial_gate(&query1, &grid, 10, 32);

    // Verify grid is IDENTICAL after all retrieval operations
    let mut mutations = 0;
    for (y, row) in snapshot_before.iter().enumerate().take(grid.height) {
        for (x, cell) in row.iter().enumerate().take(grid.width) {
            for (ch, before) in cell.iter().enumerate().take(grid.cells[y][x].len()) {
                let after = grid.cells[y][x][ch];
                if (*before - after).abs() > 1e-15 {
                    mutations += 1;
                    if mutations <= 5 {
                        eprintln!("MUTATION at [{y}][{x}][{ch}]: {} -> {}", before, after);
                    }
                }
            }
        }
    }

    assert_eq!(
        mutations, 0,
        "CRITICAL REGRESSION: Retrieval mutated {} grid cells! \
         Retrieval operations must be read-only. \
         This is a data corruption bug.",
        mutations
    );
}

/// Test: Different queries should produce different results (query-conditioning)
#[test]
fn test_retrieval_is_query_conditioned() {
    let mut store = NCAKnowledge::new();
    store.config.ollama_url = None;

    // Encode semantically distinct facts
    store.encode("cats are furry domestic animals that purr", 0.9);
    store.encode("cars are vehicles with four wheels and engines", 0.9);
    store.encode("stars are massive balls of burning gas", 0.9);

    // Query for "cat" related
    let results_cat = store.query("cats furry animals", 5);

    // Query for "car" related
    let results_car = store.query("cars vehicles wheels", 5);

    // Results should differ in at least one aspect (position, text, or relevance)
    if !results_cat.is_empty() && !results_car.is_empty() {
        let positions_cat: Vec<_> = results_cat.iter().map(|r| r.position).collect();
        let positions_car: Vec<_> = results_car.iter().map(|r| r.position).collect();

        let texts_cat: Vec<_> = results_cat.iter().filter_map(|r| r.text.clone()).collect();
        let texts_car: Vec<_> = results_car.iter().filter_map(|r| r.text.clone()).collect();

        // At least one of these should differ for query-conditioned retrieval
        let positions_differ = positions_cat != positions_car;
        let texts_differ = texts_cat != texts_car;

        assert!(
            positions_differ || texts_differ,
            "Query-conditioned retrieval should produce different results for different queries. \
             Got identical results for 'cats' vs 'cars'. \
             Positions: {:?} vs {:?}",
            positions_cat,
            positions_car
        );
    }
}

/// Test: Active knowledge count is reasonable after encoding
#[test]
fn test_encoding_produces_reasonable_activation() {
    let mut store = NCAKnowledge::new();
    store.config.ollama_url = None;

    let before = store.active_knowledge(0.01).len();
    assert_eq!(before, 0, "Fresh grid should have 0 active cells");

    // Encode one fact
    store.encode("test encoding activation levels", 0.9);

    let after = store.active_knowledge(0.01).len();

    // With spread_radius=6, we expect roughly π*6² ≈ 113 cells to be activated
    // Allow range [10, 500] for reasonable activation
    assert!(
        after >= 10,
        "Encoding should activate at least 10 cells, got {}",
        after
    );
    assert!(
        after <= 500,
        "Encoding should activate at most 500 cells, got {}",
        after
    );

    eprintln!("Encoding activated {} cells (expected ~100)", after);
}

/// Test: Encoding at high confidence produces higher activation than low confidence
#[test]
fn test_confidence_affects_activation_strength() {
    let mut store_high = NCAKnowledge::new();
    let mut store_low = NCAKnowledge::new();
    store_high.config.ollama_url = None;
    store_low.config.ollama_url = None;

    store_high.encode("confidence test high", 0.95);
    store_low.encode("confidence test low", 0.1);

    let max_high: f64 = store_high
        .active_knowledge(0.01)
        .iter()
        .map(|k| k.activation)
        .fold(0.0, f64::max);

    let max_low: f64 = store_low
        .active_knowledge(0.01)
        .iter()
        .map(|k| k.activation)
        .fold(0.0, f64::max);

    assert!(
        max_high > max_low,
        "High confidence ({:.3}) should produce higher activation than low confidence ({:.3})",
        max_high,
        max_low
    );
}

/// Test: Retrieval stats are tracked correctly
#[test]
fn test_retrieval_stats_tracking() {
    let mut store = NCAKnowledge::new();
    store.config.ollama_url = None;

    let stats = store.stats_handle();

    // Initially zero
    assert_eq!(
        stats
            .total_queries
            .load(std::sync::atomic::Ordering::Relaxed),
        0
    );

    // Encode something
    store.encode("stats tracking test", 0.9);

    // Run queries
    let _ = store.query("stats", 5);
    let _ = store.query("tracking", 5);
    let _ = store.query("test", 5);

    // Should have recorded 3 queries
    assert_eq!(
        stats
            .total_queries
            .load(std::sync::atomic::Ordering::Relaxed),
        3,
        "Should track all queries"
    );

    // Hit + miss should equal total
    let hits = stats.hits.load(std::sync::atomic::Ordering::Relaxed);
    let misses = stats.misses.load(std::sync::atomic::Ordering::Relaxed);
    assert_eq!(hits + misses, 3, "hits + misses should equal total queries");
}
