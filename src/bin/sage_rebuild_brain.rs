//! sage-rebuild-brain: Rebuild brain from corpus using semantic embeddings.
//!
//! Uses Ollama nomic-embed-text for fast batch embedding.
//! Chunks by paragraphs (~1000 chars) instead of sentences to reduce count.

use sage::distributed_knowledge::encoder::{write_knowledge, EncoderConfig, FeatureVector};
use sage::distributed_knowledge::{default_brain_path, KnowledgeStore, NCAKnowledge};
use sage::grid::{Grid, GRID_SIZE, KNOWLEDGE_ACTIVATION};
use std::time::Instant;

const CHUNK_SIZE: usize = 1000; // chars per chunk
const BATCH_SIZE: usize = 50;

fn main() {
    println!("=== SAGE Brain Rebuild (Ollama Semantic) ===\n");

    let corpus_dir = std::env::home_dir()
        .map(|h| h.join(".sage/corpus"))
        .unwrap_or_else(|| std::path::PathBuf::from("~/.sage/corpus"));

    // Collect chunks from corpus — paragraph-sized instead of sentence-sized
    let mut all_texts: Vec<String> = Vec::new();
    if corpus_dir.exists() {
        if let Ok(files) = std::fs::read_dir(&corpus_dir) {
            for file in files.flatten() {
                if let Ok(content) = std::fs::read_to_string(file.path()) {
                    // Split into ~1000 char chunks at paragraph boundaries
                    let mut current = String::new();
                    for para in content.split("\n\n") {
                        let para = para.trim();
                        if para.is_empty() { continue; }
                        if current.len() + para.len() + 2 > CHUNK_SIZE {
                            if !current.is_empty() {
                                all_texts.push(current.clone());
                                current.clear();
                            }
                            // If paragraph itself is long, split it
                            if para.len() > CHUNK_SIZE * 2 {
                                for chunk in para.as_bytes().chunks(CHUNK_SIZE) {
                                    let s = String::from_utf8_lossy(chunk);
                                    if s.trim().len() > 20 {
                                        all_texts.push(s.trim().to_string());
                                    }
                                }
                            } else {
                                current = para.to_string();
                            }
                        } else {
                            if !current.is_empty() { current.push_str("\n\n"); }
                            current.push_str(para);
                        }
                    }
                    if !current.is_empty() {
                        all_texts.push(current);
                    }
                }
            }
        }
    }
    
    let total = all_texts.len();
    println!("Found {} paragraph chunks from corpus", total);
    
    let config = EncoderConfig::default();
    
    // Create fresh grid and knowledge store
    let mut grid = Grid::new(GRID_SIZE, GRID_SIZE);
    let mut knowledge = NCAKnowledge::new();
    
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .unwrap();
    
    let start = Instant::now();
    let mut encoded = 0;
    let mut skipped = 0;
    
    // Process in batches via Ollama /api/embed
    for batch_start in (0..total).step_by(BATCH_SIZE) {
        let batch_end = (batch_start + BATCH_SIZE).min(total);
        let batch: Vec<&str> = all_texts[batch_start..batch_end].iter().map(|s| s.as_str()).collect();
        
        let res = client.post("http://localhost:11434/api/embed")
            .json(&serde_json::json!({"model":"nomic-embed-text","input":batch}))
            .send();
        
        match res {
            Ok(r) if r.status().is_success() => {
                let resp: serde_json::Value = r.json().unwrap_or_default();
                if let Some(embeddings) = resp["embeddings"].as_array() {
                    for (i, emb) in embeddings.iter().enumerate() {
                        if let Some(emb_arr) = emb.as_array() {
                            let emb_f64: Vec<f64> = emb_arr.iter()
                                .map(|v| v.as_f64().unwrap_or(0.0))
                                .collect();
                            
                            // Reduce 768-dim to 96-dim
                            let reduced = reduce_embedding(&emb_f64, config.num_features);
                            let mut features = FeatureVector {
                                values: reduced,
                                is_semantic: true,
                            };
                            features.normalize();
                            
                            let text_idx = batch_start + i;
                            let pos = write_knowledge(&mut grid, &features, 0.9, 0.5, &config);
                            knowledge.text_store.insert(pos.0, pos.1, all_texts[text_idx].clone());
                            encoded += 1;
                        }
                    }
                } else {
                    skipped += batch.len();
                }
            }
            Ok(r) => {
                eprintln!("HTTP {} at batch {}", r.status(), batch_start);
                skipped += batch.len();
            }
            Err(e) => {
                eprintln!("Request error at batch {}: {}", batch_start, e);
                skipped += batch.len();
            }
        }
        
        if (batch_start / BATCH_SIZE) % 20 == 0 {
            let elapsed = start.elapsed().as_secs_f64();
            let rate = encoded as f64 / elapsed.max(0.1);
            let pct = batch_end as f64 / total as f64 * 100.0;
            println!("  [{}/{}] {:.1}% — {:.0} chunks/s, {} encoded, {} skipped",
                batch_end, total, pct, rate, encoded, skipped);
        }
    }
    
    let elapsed = start.elapsed();
    println!("\nEncoded {} chunks in {:.1}s ({} skipped)", encoded, elapsed.as_secs_f64(), skipped);
    
    // Set grid
    knowledge.grid = grid;
    
    // Save
    let brain_path = default_brain_path();
    let backup_path = format!("{}.pre-rebuild", brain_path);
    if std::path::Path::new(&brain_path).exists() {
        let _ = std::fs::copy(&brain_path, &backup_path);
        println!("Backed up old brain to {}", backup_path);
    }
    
    match knowledge.save(&brain_path) {
        Ok(()) => println!("Saved rebuilt brain to {}", brain_path),
        Err(e) => eprintln!("Save failed: {}", e),
    }
    
    // Stats
    let mut alive = 0;
    for y in 0..GRID_SIZE {
        for x in 0..GRID_SIZE {
            if knowledge.grid.cells[y][x][KNOWLEDGE_ACTIVATION] > 0.01 {
                alive += 1;
            }
        }
    }
    println!("\nNew brain stats:");
    println!("  Alive cells: {}", alive);
    println!("  Text entries: {}", knowledge.text_store.len());
    println!("  Brain file: {:.1} MB",
        std::fs::metadata(&brain_path).map(|m| m.len() as f64 / 1_048_576.0).unwrap_or(0.0));
    
    // Quick sanity test
    println!("\n--- Sanity Test ---");
    let results = knowledge.query("What is the capital of France?", 5);
    println!("Query: 'What is the capital of France?'");
    for (i, r) in results.iter().enumerate() {
        let text = r.text.as_deref().unwrap_or("(none)");
        let preview = if text.len() > 100 { &text[..100] } else { text };
        println!("  #{} [rel={:.4}] {}...", i, r.relevance, preview);
    }
}

fn reduce_embedding(emb: &[f64], target_dim: usize) -> Vec<f64> {
    if emb.len() <= target_dim {
        return emb.to_vec();
    }
    (0..target_dim)
        .map(|i| {
            let idx = i * emb.len() / target_dim;
            emb[idx.min(emb.len() - 1)]
        })
        .collect()
}