//! Integration tests for cross-attention knowledge retrieval.
//!
//! Tests the full pipeline: encode → attention retrieval → context injection.
//! Based on arXiv:2603.10055 (Lee et al.): attention layers transfer knowledge from NCA.

use sage::distributed_knowledge::attention_decoder::AttentionDecoder;
use sage::distributed_knowledge::encoder::{encode_text, write_knowledge, EncoderConfig};
use sage::distributed_knowledge::{KnowledgeStore, NCAKnowledge, TextStore};
use sage::grid::{Grid, GRID_SIZE};

fn setup_test_config() -> EncoderConfig {
    let mut config = EncoderConfig::default();
    config.ollama_url = None; // Use hash fallback in tests
    config
}

#[test]
fn test_attention_decoder_with_text_store() {
    let mut grid = Grid::new(GRID_SIZE, GRID_SIZE);
    let mut text_store = TextStore::new();
    let config = setup_test_config();
    let decoder = AttentionDecoder::new(GRID_SIZE, GRID_SIZE);

    // Encode some facts
    let facts = [
        "The capital of France is Paris",
        "Rust is a systems programming language",
        "Neural networks learn from data",
        "The sun is a star",
    ];

    for fact in &facts {
        let features = encode_text(fact, &config);
        let pos = write_knowledge(&mut grid, &features, 0.9, 0.5, &config);
        text_store.insert(pos.0, pos.1, fact.to_string());
    }

    // Query for France
    let query_features = encode_text("What is the capital of France?", &config);
    let results = decoder.attend_with_text(&query_features, &grid, 5, Some(&text_store));

    assert!(!results.is_empty(), "Should find results for query");

    // At least one result should have text
    let has_text = results.iter().any(|r| r.text.is_some());
    assert!(has_text, "Results should include text from TextStore");
}

#[test]
fn test_attention_decoder_spatial_gating() {
    let mut grid = Grid::new(GRID_SIZE, GRID_SIZE);
    let config = setup_test_config();
    let decoder = AttentionDecoder::new(GRID_SIZE, GRID_SIZE);

    // Encode texts (positions are hash-determined)
    let topics = [
        "machine learning algorithms",
        "deep neural networks",
        "cooking italian pasta",
        "gardening vegetables",
    ];

    for topic in &topics {
        let features = encode_text(topic, &config);
        write_knowledge(&mut grid, &features, 0.9, 0.5, &config);
    }

    // Query with spatial gating
    let query_features = encode_text("artificial intelligence", &config);
    let results = decoder.attend_with_spatial_gate(&query_features, &grid, 5, 32);

    assert!(!results.is_empty(), "Spatial gated query should return results");

    // Results should be sorted by attention weight
    for i in 1..results.len() {
        assert!(
            results[i - 1].attention_weight >= results[i].attention_weight,
            "Results should be sorted by attention weight"
        );
    }
}

#[test]
fn test_nca_knowledge_freerun_repair() {
    let mut knowledge = NCAKnowledge::new();

    // Encode some knowledge
    let pos = knowledge.encode("important fact to remember", 0.9);

    // Get hidden channel values before repair
    let before_hidden: Vec<f64> = (4..16)
        .map(|ch| knowledge.grid.cells[pos.1][pos.0][ch])
        .collect();

    // Run smooth_hidden_channels (formerly freerun_repair)
    knowledge.smooth_hidden_channels(pos, 5);

    // Hidden channels may change (smoothing effect)
    let after_hidden: Vec<f64> = (4..16)
        .map(|ch| knowledge.grid.cells[pos.1][pos.0][ch])
        .collect();

    // At least some channels should be finite
    assert!(
        after_hidden.iter().all(|v| v.is_finite()),
        "Hidden channels should remain finite after repair"
    );

    // Knowledge channels should be preserved
    let knowledge_act = knowledge.grid.cells[pos.1][pos.0][sage::grid::KNOWLEDGE_ACTIVATION];
    assert!(
        knowledge_act > 0.0,
        "Knowledge activation should be preserved"
    );

    let _ = (before_hidden, after_hidden); // Silence unused warning
}

#[test]
fn test_knowledge_loop_integration() {
    // This test verifies the full knowledge loop encode/retrieve cycle
    let mut knowledge = NCAKnowledge::new();

    // Encode multiple related facts
    knowledge.encode("The Eiffel Tower is in Paris", 0.9);
    knowledge.encode("Paris is the capital of France", 0.9);
    knowledge.encode("France is in Europe", 0.9);

    // Query for related information
    let results = knowledge.query("Where is the Eiffel Tower?", 10);

    // Should find some active cells (hash-based may not perfectly match semantically)
    // The important thing is the mechanism works without panicking
    assert!(
        knowledge.active_knowledge(0.01).len() > 0,
        "Should have active knowledge cells"
    );

    // Results should be finite and valid
    for r in &results {
        assert!(r.activation.is_finite());
        assert!(r.relevance.is_finite());
    }
}

/// This test requires Ollama running with nomic-embed-text model.
/// Mark as ignored since CI environments may not have Ollama.
#[test]
#[ignore]
fn test_knowledge_loop_uses_attention_for_semantic_queries() {
    // This test only runs when Ollama is available
    let config = EncoderConfig::default();

    // Test if Ollama embeddings are available
    let test_features = encode_text("test", &config);
    if !test_features.is_semantic {
        eprintln!("Ollama not available, skipping semantic attention test");
        return;
    }

    let mut grid = Grid::new(GRID_SIZE, GRID_SIZE);
    let mut text_store = TextStore::new();
    let decoder = AttentionDecoder::new(GRID_SIZE, GRID_SIZE);

    // Encode a fact with semantic embedding
    let fact = "The capital of France is Paris";
    let features = encode_text(fact, &config);
    let pos = write_knowledge(&mut grid, &features, 0.9, 0.5, &config);
    text_store.insert(pos.0, pos.1, fact.to_string());

    // Query with related semantic query
    let query = "What is the capital of France?";
    let query_features = encode_text(query, &config);

    assert!(
        query_features.is_semantic,
        "Query should use semantic embedding"
    );

    let results =
        decoder.attend_with_spatial_gate_and_text(&query_features, &grid, 5, 32, Some(&text_store));

    // With semantic embeddings, should find the France/Paris fact
    let found_france = results
        .iter()
        .any(|r| r.text.as_ref().map_or(false, |t| t.contains("France")));

    assert!(
        found_france,
        "Semantic attention should retrieve France-related knowledge"
    );
}
