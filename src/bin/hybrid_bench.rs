//! Quick hybrid benchmark — loads existing HDC store, builds keyword index, tests
use sage::hdc::{default_hdc_path, HdcStore};
use std::collections::HashMap;
use std::path::Path;
use std::time::Instant;

type KeywordIndex = HashMap<String, Vec<(usize, u32)>>;

fn tokenize(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.len() >= 2)
        .map(|w| w.to_string())
        .collect()
}

fn build_keyword_index(store: &HdcStore) -> KeywordIndex {
    let mut index: KeywordIndex = HashMap::new();
    for (i, entry) in store.entries.iter().enumerate() {
        let tokens = tokenize(&entry.text);
        let mut counts: HashMap<String, u32> = HashMap::new();
        for t in tokens {
            *counts.entry(t).or_insert(0) += 1;
        }
        for (word, count) in counts {
            index.entry(word).or_default().push((i, count));
        }
    }
    index
}

fn hybrid_query<'a>(
    store: &'a HdcStore,
    keyword_index: &KeywordIndex,
    query_embedding: &[f32],
    query_text: &str,
    k: usize,
) -> Vec<(f32, &'a str)> {
    let query_tokens = tokenize(query_text);
    
    // Step 1: Find all entry indices that contain at least one query keyword
    let mut candidate_indices: HashMap<usize, u32> = HashMap::new();
    for token in &query_tokens {
        if let Some(entries) = keyword_index.get(token) {
            for (idx, count) in entries {
                *candidate_indices.entry(*idx).or_insert(0) += count;
            }
        }
    }
    
    // Step 2: Also include top HDC results (union, not intersection)
    let hdc_results = store.query(query_embedding, 500.min(store.len()));
    for (_score, text) in &hdc_results {
        if let Some(idx) = store.entries.iter().position(|e| e.text.as_str() == *text) {
            candidate_indices.entry(idx).or_insert(0);
        }
    }
    
    if candidate_indices.is_empty() {
        return store.query(query_embedding, k);
    }
    
    // Step 3: Score all candidates with HDC cosine similarity + keyword boost
    let query_mag: f32 = query_embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
    
    let mut scored: Vec<(f32, usize)> = Vec::with_capacity(candidate_indices.len());
    for (&idx, &kw_count) in &candidate_indices {
        let entry = &store.entries[idx];
        let entry_mag: f32 = entry.embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        let dot: f32 = query_embedding.iter().zip(entry.embedding.iter()).map(|(a, b)| a * b).sum();
        let hdc_score = if query_mag > 1e-10 && entry_mag > 1e-10 {
            (dot / (query_mag * entry_mag)).clamp(-1.0, 1.0)
        } else {
            0.0
        };
        
        // Keyword boost: bonus for matching query terms
        let kw_boost = if kw_count > 0 {
            (kw_count as f32 / query_tokens.len().max(1) as f32) * 0.3
        } else {
            0.0
        };
        
        // Fuse: HDC score + keyword boost
        let fused = hdc_score + kw_boost;
        scored.push((fused, idx));
    }
    
    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    scored.truncate(k);
    
    scored.into_iter()
        .map(|(score, idx)| (score, store.entries[idx].text.as_str()))
        .collect()
}

fn main() {
    println!("=== Hybrid Q&A Benchmark v2 ===\n");
    
    let store = HdcStore::load(Path::new(&default_hdc_path())).unwrap();
    println!("Loaded {} entries", store.len());
    
    println!("Building keyword index...");
    let kw_start = Instant::now();
    let keyword_index = build_keyword_index(&store);
    println!("  {} unique words in {:.1}s\n", keyword_index.len(), kw_start.elapsed().as_secs_f64());
    
    let client = reqwest::blocking::Client::new();
    
    let questions = vec![
        ("What is the capital of France?", "paris"),
        ("Who wrote Pride and Prejudice?", "austen"),
        ("What is the name of the monster in Frankenstein?", "frankenstein"),
        ("Who is the author of The Great Gatsby?", "fitzgerald"),
        ("What animal is Moby Dick?", "whale"),
        ("Who wrote The Art of War?", "sun tzu"),
        ("What is the main theme of The Prince?", "power"),
        ("Who is the protagonist of Don Quixote?", "quixote"),
        ("What is the setting of Wuthering Heights?", "moor"),
        ("Who wrote The Republic?", "plato"),
        ("What is the name of Sherlock Holmes companion?", "watson"),
        ("Who wrote Alice in Wonderland?", "carroll"),
        ("What is the central concept of Tao Te Ching?", "tao"),
        ("Who wrote Meditations?", "marcus aurelius"),
        ("What is the main subject of Origin of Species?", "evolution"),
    ];
    
    let mut hits = 0;
    let mut total_relevance = 0.0;
    let mut total_ms = 0u128;
    
    for (question, expected_keyword) in &questions {
        let q_start = Instant::now();
        
        let res = client.post("http://localhost:11434/api/embeddings")
            .json(&serde_json::json!({"model":"nomic-embed-text","prompt":question}))
            .send();
        
        match res {
            Ok(r) if r.status().is_success() => {
                let resp: serde_json::Value = r.json().unwrap_or_default();
                if let Some(emb) = resp["embedding"].as_array() {
                    let q_emb: Vec<f32> = emb.iter()
                        .map(|v| v.as_f64().unwrap_or(0.0) as f32)
                        .collect();
                    
                    let results = hybrid_query(&store, &keyword_index, &q_emb, question, 5);
                    let query_ms = q_start.elapsed().as_millis();
                    total_ms += query_ms;
                    
                    let top_relevance = results.first().map(|(r, _)| *r).unwrap_or(0.0);
                    total_relevance += top_relevance;
                    
                    let found = results.iter().any(|(_, text): &(f32, &str)| {
                        text.to_lowercase().contains(&expected_keyword.to_lowercase())
                    });
                    
                    if found {
                        hits += 1;
                        println!("✅ Q: {}", question);
                    } else {
                        println!("❌ Q: {}", question);
                    }
                    
                    println!("   Expected: '{}'", expected_keyword);
                    for (i, (rel, text)) in results.iter().take(3).enumerate() {
                        let preview: &str = if text.len() > 150 { &text[..150] } else { text };
                        println!("   #{} [rel={:.4}] {}...", i, rel, preview);
                    }
                    println!("   Time: {}ms\n", query_ms);
                }
            }
            _ => { println!("❌ Q: {} (embedding failed)\n", question); }
        }
    }
    
    let n = questions.len() as f64;
    println!("=== Summary ===");
    println!("Questions: {}", questions.len());
    println!("Hits: {}/{} ({:.1}%)", hits, questions.len(), hits as f64 / n * 100.0);
    println!("Mean top relevance: {:.4}", total_relevance / n as f32);
    println!("Mean query time: {:.1}ms (embedding + retrieval)", total_ms as f64 / n);
}