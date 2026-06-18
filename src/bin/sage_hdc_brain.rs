//! sage-hdc-brain: Build brain from corpus using HDC store + keyword index.
//!
//! Hybrid retrieval: HDC semantic search + keyword filtering.
//! The answer IS in the store — we just need the right retrieval mechanism.

use sage::hdc::{default_hdc_path, HdcStore};
use std::collections::HashMap;
use std::time::Instant;

const CHUNK_SIZE: usize = 800;
const OVERLAP: usize = 200;
const BATCH_SIZE: usize = 50;
const MIN_CHUNK_LEN: usize = 200;

// ── Keyword Index ──

/// Simple inverted index: word → list of (entry_index, count)
type KeywordIndex = HashMap<String, Vec<(usize, u32)>>;

fn tokenize(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.len() >= 2)
        .map(|w| w.to_string())
        .collect()
}

fn build_keyword_index(entries: &[String]) -> KeywordIndex {
    let mut index: KeywordIndex = HashMap::new();
    for (i, text) in entries.iter().enumerate() {
        let tokens = tokenize(text);
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

/// Hybrid query: keyword-first retrieval fused with HDC semantic scores.
/// 1. Find all chunks containing query keywords (from inverted index)
/// 2. Score them with HDC cosine similarity
/// 3. Fuse: keyword_match_score * 0.5 + hdc_score * 0.5
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
    
    if candidate_indices.is_empty() {
        // Fallback: pure HDC if no keyword matches
        return store.query(query_embedding, k);
    }
    
    // Step 2: Score candidates with HDC cosine similarity
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
        
        // Keyword match score: how many query tokens matched, normalized
        let kw_score = kw_count as f32 / query_tokens.len().max(1) as f32;
        
        // Fuse: equal weight
        let fused = hdc_score * 0.5 + kw_score * 0.5;
        scored.push((fused, idx));
    }
    
    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    scored.truncate(k);
    
    scored.into_iter()
        .map(|(score, idx)| (score, store.entries[idx].text.as_str()))
        .collect()
}

// ── Metadata Extraction ──

fn extract_metadata(content: &str) -> (String, String) {
    let lines: Vec<&str> = content.lines().take(15).collect();
    let mut title = String::new();
    let mut author = String::new();
    
    for (i, line) in lines.iter().enumerate() {
        let line = line.trim();
        if line.is_empty() { continue; }
        if line.starts_with("[") || line.contains("Project Gutenberg") || line.contains("***") || line.contains("http") {
            continue;
        }
        if line.to_lowercase().starts_with("by ") {
            author = line[3..].trim().to_string();
            continue;
        }
        if title.is_empty() {
            title = line.to_string();
            for j in (i+1)..lines.len() {
                let next = lines[j].trim();
                if next.is_empty() { continue; }
                if next.to_lowercase().starts_with("by ") {
                    author = next[3..].trim().to_string();
                }
                break;
            }
        }
    }
    
    if title.contains("—") {
        title = title.split("—").next().unwrap_or(&title).trim().to_string();
    }
    
    (title, author)
}

fn strip_gutenberg_boilerplate(content: &str) -> String {
    let start_marker = "*** START OF";
    let end_marker = "*** END OF";
    let mut text = content.to_string();
    if let Some(idx) = content.find(start_marker) {
        if let Some(eol) = content[idx..].find('\n') {
            text = content[idx + eol + 1..].to_string();
        }
    }
    if let Some(idx) = text.find(end_marker) {
        text.truncate(idx);
    }
    text.replace("\r\n", "\n")
}

fn chunk_text(text: &str, title: &str, author: &str) -> Vec<String> {
    let prefix = if !title.is_empty() || !author.is_empty() {
        format!("Title: {} Author: {}", title, author)
    } else {
        String::new()
    };
    
    let paragraphs: Vec<&str> = text.split("\n\n")
        .map(|p| p.trim())
        .filter(|p| !p.is_empty())
        .collect();
    
    let mut chunks = Vec::new();
    let mut current = String::new();
    
    for para in paragraphs {
        if para.len() < 50 && para.chars().filter(|c| c.is_uppercase()).count() > para.len() / 3 {
            continue;
        }
        
        if current.len() + para.len() + 2 > CHUNK_SIZE {
            if current.len() >= MIN_CHUNK_LEN {
                let chunk = if prefix.is_empty() { current.clone() } else { format!("{}\n\n{}", prefix, current) };
                chunks.push(chunk);
            }
            if current.len() > OVERLAP {
                let overlap_start = current.len() - OVERLAP;
                let mut safe_start = overlap_start;
                while safe_start < current.len() && !current.is_char_boundary(safe_start) { safe_start += 1; }
                let overlap_start = current[safe_start..].find(' ')
                    .map(|i| safe_start + i + 1)
                    .unwrap_or(safe_start);
                current = current[overlap_start..].to_string();
            } else {
                current.clear();
            }
        }
        
        if !current.is_empty() { current.push_str("\n\n"); }
        current.push_str(para);
    }
    
    if current.len() >= MIN_CHUNK_LEN {
        let chunk = if prefix.is_empty() { current } else { format!("{}\n\n{}", prefix, current) };
        chunks.push(chunk);
    }
    
    chunks
}

// ── Main ──

fn main() {
    println!("=== SAGE HDC Brain Build v3 (Hybrid) ===\n");

    let corpus_dir = std::env::home_dir()
        .map(|h| h.join(".sage/corpus"))
        .unwrap_or_else(|| std::path::PathBuf::from("~/.sage/corpus"));

    let mut all_texts: Vec<String> = Vec::new();
    let mut book_count = 0;

    if corpus_dir.exists() {
        if let Ok(files) = std::fs::read_dir(&corpus_dir) {
            let mut file_list: Vec<_> = files.flatten().collect();
            file_list.sort_by_key(|f| f.path());
            
            for file in file_list {
                if let Ok(content) = std::fs::read_to_string(file.path()) {
                    let clean = strip_gutenberg_boilerplate(&content);
                    let (title, author) = extract_metadata(&clean);
                    let chunks = chunk_text(&clean, &title, &author);
                    if !chunks.is_empty() {
                        if !title.is_empty() {
                            println!("  {}: {} chunks", title, chunks.len());
                        }
                        all_texts.extend(chunks);
                        book_count += 1;
                    }
                }
            }
        }
    }

    let total = all_texts.len();
    println!("\nFound {} chunks from {} books", total, book_count);

    // Build keyword index (fast, in-memory)
    println!("Building keyword index...");
    let kw_start = Instant::now();
    let keyword_index = build_keyword_index(&all_texts);
    let unique_words = keyword_index.len();
    println!("  {} unique words indexed in {:.1}s", unique_words, kw_start.elapsed().as_secs_f64());

    // Create HDC store
    let mut store = HdcStore::new(768);

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .unwrap();

    let start = Instant::now();
    let mut encoded = 0;

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
                            let emb_f32: Vec<f32> = emb_arr.iter()
                                .map(|v| v.as_f64().unwrap_or(0.0) as f32)
                                .collect();
                            let text_idx = batch_start + i;
                            store.insert(&emb_f32, &all_texts[text_idx], 0.9);
                            encoded += 1;
                        }
                    }
                }
            }
            _ => { eprintln!("Batch error at {}", batch_start); }
        }

        if (batch_start / BATCH_SIZE) % 20 == 0 {
            let elapsed = start.elapsed().as_secs_f64();
            let rate = encoded as f64 / elapsed.max(0.1);
            let pct = batch_end as f64 / total as f64 * 100.0;
            println!("  [{}/{}] {:.1}% — {:.0}/s, {} stored",
                batch_end, total, pct, rate, store.len());
        }
    }

    let elapsed = start.elapsed();
    println!("\nProcessed {} chunks in {:.1}s", encoded, elapsed.as_secs_f64());
    println!("  HDC store: {} entries, {} dim", store.len(), store.dim);
    println!("  Keyword index: {} unique words", unique_words);
    println!("  Memory: {:.1} MB", store.memory_bytes() as f64 / 1_048_576.0);

    // Save HDC store
    let hdc_path = default_hdc_path();
    match store.save(std::path::Path::new(&hdc_path)) {
        Ok(()) => println!("\nSaved HDC store to {}", hdc_path),
        Err(e) => eprintln!("Save failed: {}", e),
    }

    let file_size = std::fs::metadata(&hdc_path).map(|m| m.len()).unwrap_or(0);
    println!("HDC file: {:.1} MB", file_size as f64 / 1_048_576.0);

    // ── Hybrid Q&A Benchmark ──
    println!("\n=== Hybrid Q&A Benchmark ===\n");

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

                    // Hybrid query
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
    println!("\nHDC store: {} entries, {:.1} MB", store.len(), file_size as f64 / 1_048_576.0);
    println!("Keyword index: {} unique words", unique_words);
    println!("\nThis is SAGE v0.6.0 — a brain that runs on anything, knows what it reads,");
    println!("and belongs to whoever runs it. No API keys. No cloud. No GPU.");
}