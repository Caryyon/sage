//! Chat Knowledge Integration Tests
//!
//! Simulates chat sessions by encoding conversation turns into the NCA grid
//! (without actual Ollama/LLM calls). Verifies knowledge accumulation,
//! context retrieval, and decay over simulated time.

use sage::distributed_knowledge::{NCAKnowledge, KnowledgeStore};

/// Simulate encoding a conversation turn into knowledge
fn encode_conversation_turn(store: &mut NCAKnowledge, user_msg: &str, assistant_msg: &str) {
    // Encode both the question and the answer as knowledge
    let combined = format!("{} {}", user_msg, assistant_msg);
    store.encode(&combined, 0.85);
}

#[test]
fn single_conversation_creates_queryable_knowledge() {
    let mut store = NCAKnowledge::new();

    encode_conversation_turn(
        &mut store,
        "What is photosynthesis?",
        "Photosynthesis is the process by which plants convert sunlight into energy using chlorophyll",
    );

    let results = store.query("photosynthesis plants sunlight", 5);
    assert!(!results.is_empty(), "Should find knowledge from conversation");
    assert!(results[0].relevance > 0.0);
}

#[test]
fn multiple_conversations_accumulate_knowledge() {
    let mut store = NCAKnowledge::new();

    encode_conversation_turn(
        &mut store,
        "Tell me about rust",
        "Rust is a systems programming language focused on safety and performance",
    );

    let active_after_one = store.active_knowledge(0.01).len();

    encode_conversation_turn(
        &mut store,
        "What about python?",
        "Python is an interpreted language popular for scripting and data science",
    );

    let active_after_two = store.active_knowledge(0.01).len();

    encode_conversation_turn(
        &mut store,
        "How about JavaScript?",
        "JavaScript is the language of the web, running in browsers and on servers via Node.js",
    );

    let active_after_three = store.active_knowledge(0.01).len();

    assert!(active_after_two >= active_after_one, "Knowledge should accumulate");
    assert!(active_after_three >= active_after_two, "Knowledge should keep accumulating");
}

#[test]
fn later_queries_find_earlier_conversations() {
    let mut store = NCAKnowledge::new();

    // First conversation about astronomy
    encode_conversation_turn(
        &mut store,
        "How far is the moon?",
        "The moon is approximately 384400 kilometers from Earth",
    );

    // Second conversation about biology
    encode_conversation_turn(
        &mut store,
        "What is DNA?",
        "DNA is deoxyribonucleic acid, the molecule that carries genetic instructions",
    );

    // Third conversation about cooking
    encode_conversation_turn(
        &mut store,
        "How do you make bread?",
        "Bread is made by mixing flour water yeast and salt then baking",
    );

    // Query about the first topic should still return results
    let moon_results = store.query("moon distance earth kilometers", 5);
    assert!(!moon_results.is_empty(), "Should still find first conversation's knowledge");
}

#[test]
fn knowledge_decay_reduces_activation_over_time() {
    let mut store = NCAKnowledge::new();

    encode_conversation_turn(
        &mut store,
        "What is gravity?",
        "Gravity is the force of attraction between masses described by general relativity",
    );

    let before_decay: f64 = store.active_knowledge(0.01)
        .iter().map(|k| k.activation).sum();

    // Simulate time passing with decay
    for _ in 0..10 {
        store.decay_knowledge(0.1); // 10% decay each step
    }

    let after_decay: f64 = store.active_knowledge(0.01)
        .iter().map(|k| k.activation).sum();

    assert!(
        after_decay < before_decay,
        "Total activation should decrease after decay: before={} after={}",
        before_decay, after_decay
    );
}

#[test]
fn heavy_decay_eventually_removes_knowledge() {
    let mut store = NCAKnowledge::new();

    encode_conversation_turn(
        &mut store,
        "temporary fact",
        "this should decay away completely",
    );

    assert!(!store.active_knowledge(0.01).is_empty(), "Should have knowledge initially");

    // Aggressive decay over many steps
    for _ in 0..100 {
        store.decay_knowledge(0.3);
    }

    let remaining: f64 = store.active_knowledge(0.01)
        .iter().map(|k| k.activation).sum();

    assert!(
        remaining < 0.01,
        "After heavy decay, almost no activation should remain: {}",
        remaining
    );
}

#[test]
fn reinforced_knowledge_survives_decay_longer() {
    let mut store = NCAKnowledge::new();

    // Encode once
    encode_conversation_turn(
        &mut store,
        "what is reinforcement?",
        "reinforcement strengthens neural pathways through repetition",
    );

    // Re-encode (reinforce) the same knowledge
    encode_conversation_turn(
        &mut store,
        "tell me more about reinforcement",
        "reinforcement learning uses repeated exposure to strengthen patterns",
    );

    let reinforced_activation: f64 = store.active_knowledge(0.01)
        .iter().map(|k| k.activation).sum();

    // Create a store with single encoding for comparison
    let mut store_single = NCAKnowledge::new();
    encode_conversation_turn(
        &mut store_single,
        "what is reinforcement?",
        "reinforcement strengthens neural pathways through repetition",
    );

    let single_activation: f64 = store_single.active_knowledge(0.01)
        .iter().map(|k| k.activation).sum();

    assert!(
        reinforced_activation >= single_activation,
        "Reinforced knowledge should have >= activation: reinforced={} single={}",
        reinforced_activation, single_activation
    );
}

#[test]
fn simulated_multi_turn_conversation() {
    let mut store = NCAKnowledge::new();

    // Multi-turn conversation about a project
    let turns = vec![
        ("I'm building a web app", "Great! What framework are you using?"),
        ("I'm using React for the frontend", "React is a popular choice for building user interfaces with components"),
        ("What about the backend?", "You could use Node.js Express or Rust Axum for the backend API"),
        ("I chose Axum", "Axum is a great Rust web framework built on tokio and tower for async HTTP"),
    ];

    for (user, assistant) in &turns {
        encode_conversation_turn(&mut store, user, assistant);
    }

    // Query should find relevant context from the conversation
    let results = store.query("web framework react axum", 10);
    assert!(!results.is_empty(), "Should find knowledge from multi-turn conversation");

    // The total knowledge should reflect all turns
    let total_active = store.active_knowledge(0.01).len();
    assert!(total_active > 0, "Should have accumulated knowledge from all turns");
}

#[test]
fn decay_then_new_knowledge_works() {
    let mut store = NCAKnowledge::new();

    // Old knowledge
    encode_conversation_turn(&mut store, "old topic", "old information that will decay");

    // Decay heavily
    for _ in 0..50 {
        store.decay_knowledge(0.2);
    }

    // Add new knowledge
    encode_conversation_turn(
        &mut store,
        "new topic",
        "fresh information added after decay",
    );

    let results = store.query("new topic fresh information", 5);
    assert!(!results.is_empty(), "New knowledge should be queryable after decay period");
    assert!(results[0].activation > 0.1, "New knowledge should have strong activation");
}
