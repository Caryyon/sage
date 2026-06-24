//! sage-offline-qa: Benchmark offline Q&A from the NCA brain + HDC store.
//!
//! Tests whether SAGE can answer factual questions using only its
//! trained knowledge — no LLM, no API keys.
//!
//! Uses both retrieval paths:
//! 1. HDC store: 768-dim cosine similarity (high precision, episodic memory)
//! 2. NCA grid: 96-dim attractor-based retrieval (semantic memory)
//!
//! Usage: cargo run --bin sage-offline-qa
//!        cargo run --bin sage-offline-qa -- --nca-only  (NCA grid only)

use sage::distributed_knowledge::{default_brain_path, KnowledgeStore, NCAKnowledge};
use sage::hdc::{default_hdc_path, HdcStore};
use sage::grid::NUM_CHANNELS;
use std::path::Path;
use std::time::{Duration, Instant};

/// Get Ollama embedding (768-dim) for HDC store queries
fn ollama_embed(text: &str) -> Option<Vec<f32>> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .ok()?;
    let res = client
        .post("http://localhost:11434/api/embed")
        .json(&serde_json::json!({"model": "nomic-embed-text", "input": [text]}))
        .send()
        .ok()?;
    if !res.status().is_success() {
        return None;
    }
    let resp: serde_json::Value = res.json().ok()?;
    let embeddings = resp["embeddings"].as_array()?;
    let first = embeddings.first()?;
    let arr = first.as_array()?;
    Some(arr.iter().map(|v| v.as_f64().unwrap_or(0.0) as f32).collect())
}

fn ollama_available() -> bool {
    ollama_embed("test").is_some()
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let nca_only = args.iter().any(|a| a == "--nca-only");

    println!("=== SAGE Offline Q&A Benchmark ===\n");

    // Load NCA brain
    let brain_path = default_brain_path();
    let start = Instant::now();
    let mut knowledge = NCAKnowledge::new();
    match knowledge.load(&brain_path) {
        Ok(()) => println!("NCA brain loaded from {}", brain_path),
        Err(e) => {
            println!("No brain found at {}: {}", brain_path, e);
            println!("Using fresh grid");
        }
    }
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
        "Retrieval stats: {} queries, {:.1}% hit rate",
        stats.total_queries.load(std::sync::atomic::Ordering::Relaxed),
        stats.hit_rate() * 100.0
    );

    // Load HDC store (episodic memory — high-precision retrieval)
    let hdc_path = default_hdc_path();
    let use_ollama = ollama_available();
    if use_ollama {
        println!("Ollama: ✓ available (nomic-embed-text, 768-dim)");
    } else {
        println!("Ollama: ✗ not available (HDC queries will fail)");
    }
    let hdc_store = if !nca_only && Path::new(&hdc_path).exists() {
        let store = HdcStore::load(Path::new(&hdc_path)).unwrap_or_else(|_| HdcStore::new(768));
        println!("HDC store: {} entries, {}-dim\n", store.entries.len(), store.dim);
        Some(store)
    } else if nca_only {
        println!("HDC store: skipped (--nca-only)\n");
        None
    } else {
        println!("HDC store: not found at {}\n", hdc_path);
        None
    };

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

    let mut hits_nca = 0;
    let mut hits_hdc = 0;
    let mut hits_combined = 0;
    let mut total_relevance = 0.0;
    let mut total_ms = 0u128;

    println!("--- Q&A Results ---\n");

    for (question, expected_keyword) in &questions {
        let start = Instant::now();

        // 1. Query NCA grid (semantic memory)
        let nca_results = knowledge.query(question, 5);

        // 2. Query HDC store (episodic memory) via Ollama (768-dim)
        let hdc_results: Vec<(f32, &str)> = if let Some(ref hdc) = hdc_store {
            if let Some(query_emb) = ollama_embed(question) {
                hdc.query(&query_emb, 5)
                    .into_iter()
                    .map(|r| (r.0, r.1))
                    .collect()
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };

        let query_ms = start.elapsed().as_millis();
        total_ms += query_ms;

        // Check NCA results
        let nca_hit = nca_results.iter().any(|r| {
            r.text
                .as_ref()
                .map(|t| t.to_lowercase().contains(&expected_keyword.to_lowercase()))
                .unwrap_or(false)
        });

        // Check HDC results
        let hdc_hit = hdc_results.iter().any(|(_, text): &(f32, &str)| {
            text.to_lowercase().contains(&expected_keyword.to_lowercase())
        });

        let combined_hit = nca_hit || hdc_hit;

        if nca_hit {
            hits_nca += 1;
        }
        if hdc_hit {
            hits_hdc += 1;
        }
        if combined_hit {
            hits_combined += 1;
        }

        // Top relevance from NCA (primary)
        let top_relevance = nca_results.first().map(|r| r.relevance).unwrap_or(0.0);
        total_relevance += top_relevance;

        if combined_hit {
            println!("✅ Q: {}", question);
        } else {
            println!("❌ Q: {}", question);
        }
        println!("   Expected: '{}' | NCA: {} | HDC: {}", expected_keyword, if nca_hit { "✓" } else { "✗" }, if hdc_hit { "✓" } else { "✗" });

        // Show top results from each path
        if let Some(r) = nca_results.first() {
            if let Some(ref t) = r.text {
                let preview: String = t.chars().take(120).collect();
                println!("   NCA [rel={:.3}] {}", r.relevance, preview);
            }
        }
        if let Some((rel, text)) = hdc_results.first() {
            let preview: String = text.chars().take(120).collect();
            println!("   HDC [rel={:.3}] {}", rel, preview);
        }

        println!("   Time: {}ms\n", query_ms);
    }

    let n = questions.len() as f64;
    println!("=== Summary ===");
    println!("Questions: {}", questions.len());
    println!("NCA hits:       {}/{} ({:.1}%)", hits_nca, questions.len(), hits_nca as f64 / n * 100.0);
    println!("HDC hits:       {}/{} ({:.1}%)", hits_hdc, questions.len(), hits_hdc as f64 / n * 100.0);
    println!("Combined hits:  {}/{} ({:.1}%)", hits_combined, questions.len(), hits_combined as f64 / n * 100.0);
    println!("Mean top relevance (NCA): {:.4}", total_relevance / n);
    println!("Mean query time: {:.1}ms", total_ms as f64 / n);
    println!("Brain load time: {}ms", load_ms);

    let grid_bytes = grid.width * grid.height * NUM_CHANNELS * 8;
    println!(
        "Grid memory: {}×{}×{}×8B = {:.1} MB",
        grid.width,
        grid.height,
        NUM_CHANNELS,
        grid_bytes as f64 / 1_048_576.0
    );
}