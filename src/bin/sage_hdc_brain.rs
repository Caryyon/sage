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
    // First try Gutenberg structured metadata (Title: / Author: lines)
    let mut title = String::new();
    let mut author = String::new();
    
    for line in content.lines().take(80) {
        let line = line.trim();
        if line.starts_with("Title:") {
            title = line[6..].trim().to_string();
        }
        if line.starts_with("Author:") {
            author = line[7..].trim().to_string();
        }
        if !title.is_empty() && !author.is_empty() { break; }
    }
    
    // Fallback: parse first non-boilerplate lines
    if title.is_empty() {
        let lines: Vec<&str> = content.lines().take(30).collect();
        for (i, line) in lines.iter().enumerate() {
            let line = line.trim();
            if line.is_empty() { continue; }
            if line.starts_with("[") || line.contains("Project Gutenberg") || line.contains("***") || line.contains("http") {
                continue;
            }
            if line.to_lowercase().starts_with("by ") {
                if author.is_empty() { author = line[3..].trim().to_string(); }
                continue;
            }
            // Skip publisher/producer/illustration lines
            if line.contains("Produced by") || line.contains("Online Distributed")
                || line.contains("CHISWICK PRESS") || line.contains("PRINTED IN")
                || line.starts_with("RUSKIN") || line.starts_with("Ruskin")
                || line.to_lowercase().contains("publisher") || line.to_lowercase().contains("press:")
                || line.starts_with("[") || line.starts_with("_") || line.starts_with("(")
                || line.contains("SERVICE & PATON") || line.contains("HENRIETTA STREET")
                || line.contains("CHARING CROSS")
            {
                continue;
            }
            // Skip lines that are clearly addresses (contain street/road/lane + number pattern)
            if (line.contains("Street") || line.contains("Road") || line.contains("Lane") || line.contains("Square"))
                && line.chars().any(|c| c.is_ascii_digit())
            {
                continue;
            }
            // Skip single-word lines that aren't real titles
            if line.split_whitespace().count() == 1 && line.len() < 20 {
                continue;
            }
            if title.is_empty() {
                title = line.trim_end_matches(';').trim().to_string();
                for j in (i+1)..lines.len() {
                    let next = lines[j].trim();
                    if next.is_empty() { continue; }
                    if next.to_lowercase().starts_with("by ") && author.is_empty() {
                        author = next[3..].trim().to_string();
                    }
                    break;
                }
            }
        }
    }
    
    // Clean up title
    if title.contains("—") {
        title = title.split("—").next().unwrap_or(&title).trim().to_string();
    }
    
    (title, author)
}

fn strip_gutenberg_boilerplate(content: &str) -> String {
    let start_marker = "*** START OF";
    let end_marker = "*** END OF";
    let mut text = content.to_string();
    
    // Try standard Gutenberg markers first
    if let Some(idx) = content.find(start_marker) {
        if let Some(eol) = content[idx..].find('\n') {
            text = content[idx + eol + 1..].to_string();
        }
    } else {
        // No markers — strip the Gutenberg header by finding first substantial paragraph
        // Gutenberg headers are 20-50 lines of legal boilerplate
        let lines: Vec<&str> = content.lines().collect();
        let mut start = 0;
        for (i, line) in lines.iter().enumerate() {
            let line = line.trim();
            // Skip empty lines, title/author lines, boilerplate
            if line.is_empty() || line.starts_with("[") || line.starts_with("Title:") || line.starts_with("Author:") {
                continue;
            }
            if line.contains("Project Gutenberg") || line.contains("eBook") || line.contains("http") {
                continue;
            }
            if line.len() < 60 && (line.contains("CHAPTER") || line.contains("Contents") || line.contains("Illustration")) {
                continue;
            }
            // Skip TOC-like lines (dense chapter/letter listings)
            if is_toc_paragraph(line) {
                continue;
            }
            // First substantial line of prose — this is where the book starts
            if line.len() > 80 && !line.to_uppercase().eq(&line) {
                start = i;
                break;
            }
        }
        if start > 0 {
            text = lines[start..].join("\n");
        }
    }
    
    if let Some(idx) = text.find(end_marker) {
        text.truncate(idx);
    }
    text.replace("\r\n", "\n")
}

/// Check if a paragraph is a table-of-contents listing (not real content)
fn is_toc_paragraph(text: &str) -> bool {
    let trimmed = text.trim();
    
    // "Contents" / "CONTENTS" / "Table of Contents" header
    if trimmed.eq_ignore_ascii_case("contents")
        || trimmed.eq_ignore_ascii_case("table of contents")
        || trimmed.eq_ignore_ascii_case("index of chapters")
        || trimmed.eq_ignore_ascii_case("index")
    {
        return true;
    }
    
    // Single-line TOC: 3+ CHAPTER/Chapter references on one line
    // e.g. "CHAPTER I. Down the Rabbit-Hole CHAPTER II. The Pool of Tears ..."
    let chapter_matches = trimmed.match_indices("CHAPTER").count()
        + trimmed.match_indices("Chapter").count();
    if chapter_matches >= 3 {
        return true;
    }
    
    // Dense roman-numeral listing: 5+ roman numerals on one line
    // e.g. "I. Preface II. In Chancery III. A Progress IV. Telescopic..."
    let roman_count = trimmed.split_whitespace()
        .filter(|w| {
            let cleaned = w.trim_end_matches(|c: char| c == '.' || c == ',' || c == ';' || c == ':');
            is_roman_numeral(cleaned)
        })
        .count();
    if roman_count >= 5 {
        return true;
    }
    
    // Dense numeric listing: 5+ numbers on one line (e.g. "Letter 1 Letter 2 Letter 3...")
    let numeric_count = trimmed.split_whitespace()
        .filter(|w| {
            let cleaned = w.trim_end_matches(|c: char| c == '.' || c == ',' || c == ';' || c == ':');
            cleaned.parse::<u32>().is_ok()
        })
        .count();
    if numeric_count >= 5 {
        return true;
    }
    
    // "Chapter I. Chapter II. Chapter III." pattern (no CHAPTER keyword, just Chapter)
    let chapter_word_count = trimmed.match_indices("Chapter ").count();
    if chapter_word_count >= 5 {
        return true;
    }
    
    // "Letter 1 Letter 2 Letter 3 Letter 4" pattern (Frankenstein)
    let letter_count = trimmed.match_indices("Letter ").count();
    if letter_count >= 4 {
        return true;
    }
    
    // Individual TOC entry: short line starting with numeral + period
    // e.g. "I. In Chancery", "1. Introduction", "XII. Alice's Evidence"
    if trimmed.len() < 120 {
        if let Some(first_word) = trimmed.split_whitespace().next() {
            let cleaned = first_word.trim_end_matches(|c: char| c == '.' || c == ',' || c == ';' || c == ':');
            if is_roman_numeral(cleaned) || cleaned.parse::<u32>().is_ok() {
                // Rest of line is short and title-like (not prose)
                let rest: Vec<&str> = trimmed.split_whitespace().skip(1).collect();
                if rest.len() <= 8 && !trimmed.contains('.') {
                    return true;
                }
            }
        }
    }
    
    // Illustration list: "PAGE" followed by numbers
    if trimmed.contains("PAGE") && trimmed.contains("Illustration") {
        return true;
    }
    
    false
}

fn is_roman_numeral(s: &str) -> bool {
    if s.is_empty() || s.len() > 5 { return false; }
    s.chars().all(|c| matches!(c, 'I' | 'V' | 'X' | 'L' | 'C' | 'D' | 'M' | 'i' | 'v' | 'x' | 'l' | 'c' | 'd' | 'm'))
}

fn chunk_text(text: &str, title: &str, author: &str) -> Vec<String> {
    let prefix = if !title.is_empty() || !author.is_empty() {
        format!("Title: {} Author: {}", title, author)
    } else {
        String::new()
    };
    
    // Smart paragraph splitting: \n\n is the primary delimiter, but also split on
    // single \n when it separates distinct paragraphs (short header → long prose, etc.)
    let raw_paras: Vec<&str> = text.split("\n\n")
        .map(|p| p.trim())
        .filter(|p| !p.is_empty())
        .collect();
    
    // Further split large paragraphs on single newlines when they look like
    // paragraph breaks (line ends with sentence punctuation, next line is substantial)
    let mut paragraphs: Vec<String> = Vec::new();
    for para in raw_paras {
        if para.len() <= CHUNK_SIZE {
            paragraphs.push(para.to_string());
        } else {
            // Large paragraph — try splitting on single newlines
            let lines: Vec<&str> = para.lines().collect();
            let mut sub_para = String::new();
            for (_i, line) in lines.iter().enumerate() {
                let line = line.trim();
                if line.is_empty() {
                    if !sub_para.is_empty() {
                        paragraphs.push(sub_para.clone());
                        sub_para.clear();
                    }
                    continue;
                }
                
                // Check if this line starts a new paragraph:
                // - Previous line ended with sentence punctuation
                // - This line starts with capital letter or quote
                // - Previous accumulated text is substantial
                if !sub_para.is_empty() && sub_para.len() > 100 {
                    let prev_ends = sub_para.trim_end().ends_with('.')
                        || sub_para.trim_end().ends_with('!')
                        || sub_para.trim_end().ends_with('?')
                        || sub_para.trim_end().ends_with('"')
                        || sub_para.trim_end().ends_with(')')
                        || sub_para.trim_end().ends_with(':');
                    let cur_starts = line.starts_with(|c: char| c.is_uppercase())
                        || line.starts_with('"')
                        || line.starts_with('\'')
                        || line.starts_with('(');
                    
                    // Also split on chapter/section headers
                    let is_header = line.starts_with("CHAPTER")
                        || line.starts_with("Chapter")
                        || line.starts_with("Letter ")
                        || line.starts_with("VOLUME")
                        || line.starts_with("Volume")
                        || line.starts_with("Book ")
                        || line.starts_with("Part ")
                        || (line.len() < 80 && line.ends_with('.')
                            && line.split_whitespace().next()
                                .map(|w| is_roman_numeral(w.trim_end_matches('.')))
                                .unwrap_or(false));
                    
                    if (prev_ends && cur_starts) || is_header {
                        paragraphs.push(sub_para.clone());
                        sub_para.clear();
                    }
                }
                
                if !sub_para.is_empty() {
                    sub_para.push(' ');
                }
                sub_para.push_str(line);
            }
            if !sub_para.is_empty() {
                paragraphs.push(sub_para);
            }
        }
    }
    
    let mut chunks = Vec::new();
    let mut current = String::new();
    
    for para in &paragraphs {
        // Skip table-of-contents paragraphs
        if is_toc_paragraph(para) {
            continue;
        }
        
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
                    // Extract metadata from ORIGINAL content (Title:/Author: lines are in Gutenberg header)
                    let (title, author) = extract_metadata(&content);
                    let clean = strip_gutenberg_boilerplate(&content);
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
    println!("\nDone. Run 'hybrid-bench' to test retrieval + LLM synthesis.");
}