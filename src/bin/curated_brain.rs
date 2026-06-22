use sage::distributed_knowledge::encoder::{write_knowledge, EncoderConfig, FeatureVector};
use sage::distributed_knowledge::{default_brain_path, KnowledgeStore, NCAKnowledge};
use sage::grid::{Grid, GRID_SIZE, KNOWLEDGE_ACTIVATION};
use std::time::Instant;

fn main() {
    println!("=== SAGE Curated Brain Build ===\n");

    let corpus_dir = std::env::home_dir()
        .map(|h| h.join(".sage/corpus"))
        .unwrap_or_else(|| std::path::PathBuf::from("~/.sage/corpus"));

    // Collect ONE representative paragraph from each book
    let mut texts: Vec<String> = Vec::new();
    if corpus_dir.exists() {
        if let Ok(files) = std::fs::read_dir(&corpus_dir) {
            for file in files.flatten() {
                if let Ok(content) = std::fs::read_to_string(file.path()) {
                    // Normalize line endings and split into paragraphs
                    let content = content.replace("\r\n", "\n");
                    // Find first substantial paragraph (skip headers/legal text)
                    let paras: Vec<&str> = content
                        .split("\n\n")
                        .map(|p| p.trim())
                        .filter(|p| p.len() > 200 && p.len() < 1500)
                        .filter(|p| !p.contains("Project Gutenberg") && !p.contains("Copyright"))
                        .take(1)
                        .collect();
                    if let Some(p) = paras.first() {
                        texts.push(p.to_string());
                    }
                }
            }
        }
    }
    
    println!("Curated {} paragraphs from {} books", texts.len(), texts.len());
    
    let config = EncoderConfig::default();
    let mut grid = Grid::new(GRID_SIZE, GRID_SIZE);
    let mut knowledge = NCAKnowledge::new();
    
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build().unwrap();
    
    let start = Instant::now();
    for (i, text) in texts.iter().enumerate() {
        let res = client.post("http://localhost:11434/api/embeddings")
            .json(&serde_json::json!({"model":"nomic-embed-text","prompt":text}))
            .send();
        
        if let Ok(r) = res {
            if let Ok(resp) = r.json::<serde_json::Value>() {
                if let Some(emb) = resp["embedding"].as_array() {
                    let emb_f64: Vec<f64> = emb.iter().map(|v| v.as_f64().unwrap_or(0.0)).collect();
                    let reduced = reduce_embedding(&emb_f64, config.num_features);
                    let mut features = FeatureVector { values: reduced, is_semantic: true };
                    features.normalize();
                    
                    let pos = write_knowledge(&mut grid, &features, 0.9, 0.5, &config);
                    knowledge.text_store.insert(pos.0, pos.1, text.clone());
                }
            }
        }
        
        if i % 10 == 0 {
            println!("  [{}/{}] encoded", i, texts.len());
        }
    }
    
    knowledge.grid = grid;
    let brain_path = default_brain_path();
    match knowledge.save(&brain_path) {
        Ok(()) => println!("\nSaved curated brain ({} texts) to {}", texts.len(), brain_path),
        Err(e) => eprintln!("Save failed: {}", e),
    }
    
    let mut alive = 0;
    for y in 0..GRID_SIZE {
        for x in 0..GRID_SIZE {
            if knowledge.grid.cells[y][x][KNOWLEDGE_ACTIVATION] > 0.01 { alive += 1; }
        }
    }
    println!("Alive cells: {} / {} ({:.1}%)", alive, GRID_SIZE*GRID_SIZE, alive as f64 / (GRID_SIZE*GRID_SIZE) as f64 * 100.0);
    println!("Text entries: {}", knowledge.text_store.len());
    println!("Time: {:.1}s", start.elapsed().as_secs_f64());
    
    // Sanity test
    println!("\n--- Sanity Test ---");
    let results = knowledge.query("What is the capital of France?", 5);
    for (i, r) in results.iter().enumerate() {
        let text = r.text.as_deref().unwrap_or("(none)");
        let preview = if text.len() > 100 { &text[..100] } else { text };
        println!("  #{} [rel={:.4}] {}...", i, r.relevance, preview);
    }
}

fn reduce_embedding(emb: &[f64], target_dim: usize) -> Vec<f64> {
    if emb.len() <= target_dim { return emb.to_vec(); }
    (0..target_dim).map(|i| {
        let idx = (i * emb.len() / target_dim).min(emb.len() - 1);
        emb[idx]
    }).collect()
}
