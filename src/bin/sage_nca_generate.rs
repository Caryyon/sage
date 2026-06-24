//! sage-nca-generate: NCA pattern generation from consolidated knowledge
//!
//! Step 7 of the v0.6.0 plan: Given a prompt, activate related cells in the
//! NCA grid, run propagation steps, and read out generated patterns.
//!
//! This produces:
//! - Relevant keyword suggestions
//! - Topic clustering
//! - "Intuition" about what the brain knows
//!
//! Usage:
//!   cargo run --bin sage_nca_generate -- "philosophy"
//!   cargo run --bin sage_nca_generate -- "science evolution"
//!   cargo run --bin sage_nca_generate -- --verbose "war strategy"

use sage::distributed_knowledge::decoder::{
    query_knowledge_with_text, scan_active_knowledge, KnowledgeActivation,
};
use sage::distributed_knowledge::encoder::EncoderConfig;
use sage::distributed_knowledge::{NCAKnowledge, default_brain_path, KnowledgeStore};
use sage::grid::{ConsolidationParams, GRID_SIZE, KNOWLEDGE_ACTIVATION, KNOWLEDGE_CONFIDENCE};
use std::collections::HashMap;
use std::time::Instant;

fn main() -> Result<(), String> {
    let args: Vec<String> = std::env::args().collect();
    let verbose = args.iter().any(|a| a == "--verbose" || a == "-v");
    let query: String = args
        .iter()
        .filter(|a| !a.starts_with('-'))
        .skip(1) // skip binary name
        .cloned()
        .collect::<Vec<_>>()
        .join(" ");

    if query.is_empty() {
        eprintln!("Usage: sage_nca_generate [--verbose] \"your query\"");
        eprintln!();
        eprintln!("Examples:");
        eprintln!("  sage_nca_generate \"philosophy\"");
        eprintln!("  sage_nca_generate \"science evolution\"");
        eprintln!("  sage_nca_generate --verbose \"war strategy\"");
        return Ok(());
    }

    let brain_path = default_brain_path();
    if !std::path::Path::new(&brain_path).exists() {
        return Err(format!(
            "No NCA brain found at {}. Run sage_consolidate first.",
            brain_path
        ));
    }

    eprintln!("🧠 SAGE NCA Generation Engine");
    eprintln!("   Step 7: Pattern generation from consolidated knowledge");
    eprintln!();

    // Load the NCA brain
    let mut knowledge = NCAKnowledge::new();
    knowledge.load(&brain_path)?;

    let active_before = knowledge.active_knowledge(0.01).len();
    eprintln!("   Brain loaded: {} active cells", active_before);

    let config = EncoderConfig::default();
    let start = Instant::now();

    // ── Phase 1: Query the NCA grid ──────────────────────────────────────
    eprintln!("   Query: \"{}\"", query);
    let results = query_knowledge_with_text(
        &knowledge.grid,
        &query,
        &config,
        20,
        Some(&knowledge.text_store),
    );

    if verbose {
        eprintln!("   Top matches:");
        for (i, r) in results.iter().take(10).enumerate() {
            let text_preview = r
                .text
                .as_ref()
                .map(|t| {
                    if t.len() > 80 {
                        format!("{}...", &t[..80])
                    } else {
                        t.clone()
                    }
                })
                .unwrap_or_else(|| "(no text)".to_string());
            eprintln!(
                "     {}. [{:.3}] ({},{}) {}",
                i + 1,
                r.relevance,
                r.position.0,
                r.position.1,
                text_preview
            );
        }
    }

    // ── Phase 2: Activate related cells ──────────────────────────────────
    // For each top match, boost activation and spread to neighbors
    let top_n = results.iter().take(5);
    let mut activated_positions: Vec<(usize, usize)> = Vec::new();

    for r in top_n {
        let (x, y) = r.position;
        // Boost the matched cell
        knowledge.grid.cells[y][x][KNOWLEDGE_ACTIVATION] =
            (knowledge.grid.cells[y][x][KNOWLEDGE_ACTIVATION] + 0.2).min(1.0);
        knowledge.grid.cells[y][x][KNOWLEDGE_CONFIDENCE] =
            (knowledge.grid.cells[y][x][KNOWLEDGE_CONFIDENCE] + 0.1).min(1.0);
        activated_positions.push((x, y));
    }

    // ── Phase 3: Run NCA propagation steps ──────────────────────────────
    // Spread activation from activated cells to neighbors (Hebbian-like)
    let spread_steps = 3;
    let spread_radius = 3;
    let spread_strength = 0.15;

    for _step in 0..spread_steps {
        let mut spread_updates: Vec<(usize, usize, f64)> = Vec::new();

        for &(cx, cy) in &activated_positions {
            let r = spread_radius as i32;
            for dy in -r..=r {
                for dx in -r..=r {
                    if dx == 0 && dy == 0 {
                        continue;
                    }
                    let nx = ((cx as i32 + dx).rem_euclid(GRID_SIZE as i32)) as usize;
                    let ny = ((cy as i32 + dy).rem_euclid(GRID_SIZE as i32)) as usize;

                    let source_act = knowledge.grid.cells[cy][cx][KNOWLEDGE_ACTIVATION];
                    let dist = ((dx * dx + dy * dy) as f64).sqrt();
                    let boost = source_act * spread_strength / (1.0 + dist);

                    spread_updates.push((nx, ny, boost));
                }
            }
        }

        // Apply spread updates
        for (nx, ny, boost) in &spread_updates {
            knowledge.grid.cells[*ny][*nx][KNOWLEDGE_ACTIVATION] =
                (knowledge.grid.cells[*ny][*nx][KNOWLEDGE_ACTIVATION] + boost).min(1.0);
        }

        // Add newly activated cells to the set for next step
        let threshold = 0.15;
        for (nx, ny, _) in &spread_updates {
            if knowledge.grid.cells[*ny][*nx][KNOWLEDGE_ACTIVATION] >= threshold {
                if !activated_positions.contains(&(*nx, *ny)) {
                    activated_positions.push((*nx, *ny));
                }
            }
        }
    }

    // ── Phase 4: Read out generated patterns ─────────────────────────────
    // Collect all cells above activation threshold after propagation
    let min_activation = 0.1;
    let generated = scan_active_knowledge(&knowledge.grid, min_activation);

    // Enrich with text from the text store (use peek for read-only access)
    let generated_with_text: Vec<KnowledgeActivation> = generated
        .into_iter()
        .map(|mut cell| {
            cell.text = knowledge.text_store.peek(cell.position.0, cell.position.1).map(|s| s.to_string());
            cell
        })
        .collect();

    // Extract keywords from BOTH query results AND generated cells
    let mut keyword_freq: HashMap<String, usize> = HashMap::new();
    let mut all_texts: Vec<String> = Vec::new();

    // First: extract from query results (these have text from text_store)
    for r in &results {
        if let Some(ref text) = r.text {
            all_texts.push(text.clone());
            extract_keywords_from_text(text, &mut keyword_freq);
        }
    }

    // Then: extract from generated cells (cluster labels + propagated)
    for cell in &generated_with_text {
        if let Some(ref text) = cell.text {
            all_texts.push(text.clone());
            extract_keywords_from_text(text, &mut keyword_freq);
        }
    }

    // Sort keywords by frequency
    let mut keywords: Vec<(String, usize)> = keyword_freq.into_iter().collect();
    keywords.sort_by(|a, b| b.1.cmp(&a.1));

    // ── Phase 5: Topic clustering ────────────────────────────────────────
    // Group generated cells by common themes based on keyword overlap
    let mut topics: Vec<TopicCluster> = Vec::new();
    let top_keywords: Vec<&str> = keywords.iter().take(15).map(|(k, _)| k.as_str()).collect();

    for cell in &generated_with_text {
        if let Some(ref text) = cell.text {
            let text_lower = text.to_lowercase();
            let mut matched_keywords: Vec<String> = Vec::new();
            for kw in &top_keywords {
                if text_lower.contains(kw) {
                    matched_keywords.push(kw.to_string());
                }
            }
            if !matched_keywords.is_empty() {
                // Find or create topic cluster
                let topic_key = matched_keywords.join("+");
                if let Some(cluster) = topics.iter_mut().find(|t| t.key == topic_key) {
                    cluster.count += 1;
                    cluster.total_activation += cell.activation;
                    if !cluster.samples.contains(&text_lower) && cluster.samples.len() < 3 {
                        cluster.samples.push(text_lower.clone());
                    }
                } else {
                    topics.push(TopicCluster {
                        key: topic_key,
                        keywords: matched_keywords,
                        count: 1,
                        total_activation: cell.activation,
                        samples: vec![text_lower],
                    });
                }
            }
        }
    }

    topics.sort_by(|a, b| {
        b.total_activation
            .partial_cmp(&a.total_activation)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let elapsed = start.elapsed();

    // ── Output ───────────────────────────────────────────────────────────
    println!();
    println!("═══════════════════════════════════════════════");
    println!("NCA Generation Results for: \"{}\"", query);
    println!("═══════════════════════════════════════════════");
    println!();
    println!("📊 Stats:");
    println!("   Active cells before:  {}", active_before);
    println!("   Cells after propagation: {}", generated_with_text.len());
    println!("   Propagation steps:    {}", spread_steps);
    println!("   Time:                 {:.1}ms", elapsed.as_secs_f64() * 1000.0);
    println!();

    println!("🔑 Top Keywords (NCA intuition):");
    for (i, (kw, freq)) in keywords.iter().take(20).enumerate() {
        println!("   {:2}. {} (×{})", i + 1, kw, freq);
    }
    println!();

    if !topics.is_empty() {
        println!("🧩 Topic Clusters:");
        for (i, topic) in topics.iter().take(10).enumerate() {
            println!(
                "   {:2}. [{}] {} cells, act={:.3}",
                i + 1,
                topic.keywords.join(", "),
                topic.count,
                topic.total_activation
            );
            if verbose {
                for sample in &topic.samples {
                    let preview = if sample.len() > 100 {
                        format!("{}...", &sample[..100])
                    } else {
                        sample.clone()
                    };
                    println!("       └─ {}", preview);
                }
            }
        }
        println!();
    }

    if verbose {
        println!("📝 All Generated Cells ({}):", generated_with_text.len());
        for (i, cell) in generated_with_text.iter().enumerate() {
            let text_preview = cell
                .text
                .as_ref()
                .map(|t| {
                    if t.len() > 100 {
                        format!("{}...", &t[..100])
                    } else {
                        t.clone()
                    }
                })
                .unwrap_or_else(|| "(no text)".to_string());
            println!(
                "   {:3}. [{:.3}] ({},{}) {}",
                i + 1,
                cell.activation,
                cell.position.0,
                cell.position.1,
                text_preview
            );
        }
    }

    println!("═══════════════════════════════════════════════");
    println!("This is NCA generation — zero LLM, zero API keys.");
    println!("The grid produces keyword associations and topic");
    println!("clusters from its consolidated knowledge patterns.");
    println!("═══════════════════════════════════════════════");

    Ok(())
}

struct TopicCluster {
    key: String,
    keywords: Vec<String>,
    count: usize,
    total_activation: f64,
    samples: Vec<String>,
}

fn extract_keywords_from_text(text: &str, freq: &mut HashMap<String, usize>) {
    // Parse cluster labels: "[CLUSTER:N] ... keywords=kw1, kw2, ..."
    if text.starts_with("[CLUSTER:") {
        if let Some(kw_start) = text.find("keywords=") {
            let kw_part = &text[kw_start + 9..];
            let kw_end = kw_part.find('|').unwrap_or(kw_part.len());
            for kw in kw_part[..kw_end].split(',') {
                let w = kw.trim().to_lowercase();
                if w.len() > 2 && !is_stop_word(&w) && !is_cluster_meta(&w) {
                    *freq.entry(w).or_insert(0) += 1;
                }
            }
        }
    }
    // Also extract words from the text itself (for non-cluster entries)
    for word in text.split(|c: char| !c.is_alphanumeric() && c != '\'') {
        let w = word.trim().to_lowercase();
        if w.len() > 2
            && !is_stop_word(&w)
            && !is_cluster_meta(&w)
            && !w.chars().all(|c| c.is_numeric())
        {
            *freq.entry(w).or_insert(0) += 1;
        }
    }
}

fn is_cluster_meta(word: &str) -> bool {
    matches!(word, "size" | "samples" | "keywords" | "coherence" | "cluster")
}

fn is_stop_word(word: &str) -> bool {
    matches!(
        word,
        "the" | "and" | "for" | "are" | "but" | "not" | "you" | "all"
            | "can" | "had" | "her" | "was" | "one" | "our" | "out" | "has"
            | "have" | "been" | "being" | "who" | "whom" | "which"
            | "what" | "when" | "where" | "why" | "how" | "this" | "that"
            | "these" | "those" | "then" | "than" | "with" | "from" | "they"
            | "will" | "would" | "shall" | "should" | "may" | "might" | "must"
            | "his" | "him" | "she" | "its" | "their" | "them" | "there"
            | "here" | "into" | "over" | "such" | "only" | "other" | "some"
            | "more" | "most" | "very" | "much" | "many" | "each" | "every"
            | "both" | "few" | "own" | "same" | "too" | "does" | "did"
            | "just" | "like" | "also" | "well" | "back" | "even" | "still"
            | "make" | "made" | "said" | "upon" | "come" | "came" | "take"
            | "took" | "know" | "knew" | "seen" | "see" | "saw" | "get"
            | "got" | "put" | "let" | "set" | "use" | "used" | "way"
            | "now" | "new" | "old" | "any" | "yet" | "nor" | "though"
            | "through" | "after" | "before" | "between" | "under" | "again"
            | "further" | "once" | "without" | "about" | "above" | "below"
            | "down" | "while" | "during" | "until" | "since" | "among"
            | "within" | "along" | "across" | "behind" | "beyond" | "beside"
            | "could" | "great" | "little" | "long" | "part" | "place"
            | "thing" | "things" | "life" | "world" | "man" | "men"
            | "day" | "time" | "year" | "years" | "hand" | "hands"
            | "head" | "eyes" | "face" | "heart" | "body" | "mind"
            | "first" | "last" | "next" | "thus" | "far" | "near"
            | "always" | "never" | "often" | "ever" | "else" | "whole"
            | "found" | "find" | "give" | "gave" | "given" | "left"
            | "right" | "large" | "small" | "high" | "low" | "full"
            | "true" | "good" | "bad" | "best" | "better" | "less"
            | "enough" | "almost" | "rather" | "quite" | "really"
            | "perhaps" | "certain" | "seemed" | "nothing"
            | "something" | "everything" | "anything" | "everyone"
            | "someone" | "anyone" | "nobody" | "somebody" | "anybody"
            | "himself" | "herself" | "itself" | "themselves" | "yourself"
            | "myself" | "ourselves" | "yourselves" | "because" | "therefore"
            | "however" | "moreover" | "nevertheless" | "otherwise"
            | "according" | "whether" | "already" | "together" | "another"
            | "against" | "toward" | "towards" | "around" | "amongst"
            | "chapter" | "project" | "gutenberg" | "ebook" | "www"
            | "http" | "https" | "org" | "com" | "net" | "title"
            | "author" | "page" | "pages" | "volume" | "vol" | "edition"
            | "published" | "publisher" | "copyright" | "rights" | "reserved"
            | "electronic" | "work" | "works" | "literary"
            | "archive" | "etext" | "text" | "texts" | "file" | "files"
            | "format" | "formats" | "plain" | "vanilla" | "ascii"
            | "character" | "characters" | "line" | "lines" | "letter"
            | "letters" | "word" | "words" | "number" | "numbers"
            | "series" | "parts" | "section" | "sections"
            | "contents" | "table" | "list" | "lists" | "index"
            | "preface" | "introduction" | "conclusion" | "appendix"
            | "footnote" | "footnotes" | "endnote" | "endnotes"
            | "bibliography" | "reference" | "references" | "glossary"
            | "illustration" | "illustrations" | "figure" | "figures"
            | "plate" | "plates" | "map" | "maps" | "diagram" | "diagrams"
    )
}
