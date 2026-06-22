//! sage-offline-qa: Benchmark offline Q&A from the NCA brain.
//!
//! Tests whether SAGE can answer factual questions using only its
//! trained NCA knowledge — no LLM, no API keys.
//!
//! Usage: cargo run --bin sage-offline-qa

use sage::distributed_knowledge::{default_brain_path, KnowledgeStore, NCAKnowledge};
use sage::grid::NUM_CHANNELS;
use std::time::Instant;

fn main() {
    println!("=== SAGE Offline Q&A Benchmark ===\n");

    // Load brain
    let brain_path = default_brain_path();
    let start = Instant::now();
    let mut knowledge = NCAKnowledge::new();
    match knowledge.load(&brain_path) {
        Ok(()) => println!("Brain loaded from {}", brain_path),
        Err(e) => {
            println!("No brain found at {}: {}", brain_path, e);
            println!("Using fresh grid");
        }
    };
    let load_ms = start.elapsed().as_millis();
    println!("Brain loaded in {}ms", load_ms);

    let grid = &knowledge.grid;
    let stats = knowledge.stats_handle();
    println!(
        "Grid: {}×{}×{} channels, {} alive cells",
        grid.width,
        grid.height,
        NUM_CHANNELS,
        grid.alive_count()
    );
    println!(
        "Retrieval: {} queries, {:.1}% hit rate",
        stats.total_queries.load(std::sync::atomic::Ordering::Relaxed),
        stats.hit_rate() * 100.0
    );

    // Test questions — factual things that should be in the corpus
    let questions = vec![
        ("What is the capital of France?", "Paris"),
        ("Who wrote Pride and Prejudice?", "Austen"),
        ("What is the name of the monster in Frankenstein?", "Frankenstein"),
        ("Who is the author of The Great Gatsby?", "Fitzgerald"),
        ("What animal is Moby Dick?", "whale"),
        ("Who wrote The Art of War?", "Sun Tzu"),
        ("What is the main theme of The Prince?", "power"),
        ("Who is the protagonist of Don Quixote?", "Quixote"),
        ("What is the setting of Wuthering Heights?", "moors"),
        ("Who wrote The Republic?", "Plato"),
        ("What is the name of Sherlock Holmes' companion?", "Watson"),
        ("Who wrote Alice in Wonderland?", "Carroll"),
        ("What is the central concept of Tao Te Ching?", "Tao"),
        ("Who wrote Meditations?", "Marcus Aurelius"),
        ("What is the main subject of Origin of Species?", "evolution"),
    ];

    let mut hits = 0;
    let mut total_relevance = 0.0;
    let mut total_ms = 0u128;

    println!("\n--- Q&A Results ---\n");

    for (question, expected_keyword) in &questions {
        let start = Instant::now();
        let results = knowledge.query(question, 5);
        let query_ms = start.elapsed().as_millis();
        total_ms += query_ms;

        let top_texts: Vec<String> = results
            .iter()
            .filter(|r| r.text.is_some())
            .map(|r| {
                let text = r.text.as_ref().unwrap();
                format!("[rel={:.3}] {}", r.relevance, text)
            })
            .collect();

        let top_relevance = results.first().map(|r| r.relevance).unwrap_or(0.0);
        total_relevance += top_relevance;

        // Check if any retrieved text contains the expected keyword
        let found = results.iter().any(|r| {
            r.text
                .as_ref()
                .map(|t| t.to_lowercase().contains(&expected_keyword.to_lowercase()))
                .unwrap_or(false)
        });

        if found {
            hits += 1;
            println!("✅ Q: {}", question);
        } else {
            println!("❌ Q: {}", question);
        }
        println!("   Expected keyword: '{}'", expected_keyword);
        if top_texts.is_empty() {
            println!("   Retrieved: (nothing)");
        } else {
            for t in &top_texts[..top_texts.len().min(2)] {
                println!("   Retrieved: {}", t);
            }
        }
        println!("   Time: {}ms\n", query_ms);
    }

    let n = questions.len() as f64;
    println!("=== Summary ===");
    println!("Questions: {}", questions.len());
    println!("Hits: {}/{} ({:.1}%)", hits, questions.len(), hits as f64 / n * 100.0);
    println!(
        "Mean top relevance: {:.4}",
        total_relevance / n
    );
    println!("Mean query time: {:.1}ms", total_ms as f64 / n);
    println!("Brain load time: {}ms", load_ms);
    println!("Brain size: {} bytes", std::fs::metadata(&brain_path).map(|m| m.len()).unwrap_or(0));

    // Memory profile
    let grid_bytes = grid.width * grid.height * NUM_CHANNELS * 8; // f64
    println!(
        "Grid memory: {}×{}×{}×8B = {:.1} MB",
        grid.width,
        grid.height,
        NUM_CHANNELS,
        grid_bytes as f64 / 1_048_576.0
    );
}
