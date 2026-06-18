//! sage-learn: Continuous Language Learning for the 256×256 NCA Brain
//!
//! This is the core learning loop. It feeds English text into the single
//! 256×256 brain, lets NCA dynamics self-organize the grid, and tracks
//! retrieval quality over time.
//!
//! Architecture:
//!   1. Load existing brain (or create fresh 256×256)
//!   2. Read corpus in sentence/paragraph chunks
//!   3. Encode each chunk into the grid (hash-based position, full 256×256)
//!   4. Run NCA consolidation steps (Hebbian learning between chunks)
//!   5. Periodically test retrieval quality
//!   6. Save brain state
//!   7. Report progress
//!
//! The grid self-organizes because:
//!   - Similar text lands in nearby regions (hash-based spatial addressing)
//!   - Hebbian consolidation strengthens co-activated cells
//!   - Decay removes noise
//!   - Over time, regions specialize (nouns, verbs, syntax patterns, etc.)
//!
//! Usage:
//!   sage-learn --corpus /path/to/corpus.txt [--chunk-size 500] [--consolidate-every 50]
//!   sage-learn --download-gutenberg  # Download public domain books
//!   sage-learn --status             # Show learning progress

use sage::distributed_knowledge::brain_processor::{process_text, BrainDynamics, BrainNcaWeights};
use sage::distributed_knowledge::decoder::query_knowledge_with_text;
use sage::distributed_knowledge::encoder::{
    encode_text, EncoderConfig,
};
use sage::distributed_knowledge::text_store::TextStore;
use sage::distributed_knowledge::{default_brain_path, NCAKnowledge, KnowledgeStore};
use sage::grid::{Grid, GRID_SIZE};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Instant;

// ── Configuration ──────────────────────────────────────────────────────────

const DEFAULT_CHUNK_SIZE: usize = 500; // chars per chunk
const DEFAULT_CONSOLIDATE_EVERY: usize = 50; // chunks between consolidation
const DEFAULT_CONSO_STEPS: usize = 3; // consolidation iterations
const DEFAULT_TEST_EVERY: usize = 200; // chunks between retrieval tests
const DEFAULT_NCA_STEPS: usize = 8; // NCA dynamics steps per chunk
const CORPUS_DIR: &str = "~/.sage/corpus/";
const PROGRESS_FILE: &str = "~/.sage/learn_progress.json";

// ── Test queries for quality tracking ──────────────────────────────────────

const TEST_QUERIES: &[(&str, &str)] = &[
    // Basic English structure
    ("the cat sat on the mat", "cat sat mat"),
    ("she walked to the store", "walked store"),
    ("he opened the door slowly", "opened door"),
    ("they were happy together", "happy together"),
    ("the sun rose over the mountains", "sun mountains"),
    // Grammar patterns
    ("running quickly through the forest", "running quickly"),
    ("the beautiful old house on the hill", "beautiful house"),
    ("if it rains tomorrow we will stay home", "rains tomorrow"),
    ("because she was tired she went to bed", "tired bed"),
    ("although it was cold they went outside", "cold outside"),
    // Semantic associations
    ("the doctor examined the patient carefully", "doctor patient"),
    ("she cooked dinner for her family", "cooked dinner family"),
    ("the teacher explained the lesson to the students", "teacher students"),
    ("he drove his car to work every morning", "drove car work"),
    ("the musician played a beautiful melody", "musician melody"),
];

// ── Progress tracking ──────────────────────────────────────────────────────

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
struct LearnProgress {
    total_chunks_processed: u64,
    total_chars_processed: u64,
    corpus_files_processed: Vec<String>,
    last_corpus_position: u64, // byte offset in current corpus
    current_corpus: String,
    sessions_completed: u64,
    best_hit_rate: f64,
    best_mean_relevance: f64,
    alive_cells: usize,
    text_entries: usize,
    last_session_time: String,
    nca_steps_per_chunk: usize,
    nca_weight_version: u64,
}

impl Default for LearnProgress {
    fn default() -> Self {
        Self {
            total_chunks_processed: 0,
            total_chars_processed: 0,
            corpus_files_processed: vec![],
            last_corpus_position: 0,
            current_corpus: String::new(),
            sessions_completed: 0,
            best_hit_rate: 0.0,
            best_mean_relevance: 0.0,
            alive_cells: 0,
            text_entries: 0,
            last_session_time: String::new(),
            nca_steps_per_chunk: DEFAULT_NCA_STEPS,
            nca_weight_version: 0,
        }
    }
}

fn load_progress() -> LearnProgress {
    let path = expand_tilde(PROGRESS_FILE);
    if let Ok(data) = fs::read_to_string(&path) {
        serde_json::from_str(&data).unwrap_or_default()
    } else {
        LearnProgress::default()
    }
}

fn save_progress(progress: &LearnProgress) {
    let path = expand_tilde(PROGRESS_FILE);
    if let Some(parent) = Path::new(&path).parent() {
        let _ = fs::create_dir_all(parent);
    }
    let json = serde_json::to_string_pretty(progress).unwrap_or_default();
    let _ = fs::write(&path, json);
}

// ── Corpus management ──────────────────────────────────────────────────────

fn expand_tilde(path: &str) -> String {
    if let Some(stripped) = path.strip_prefix("~/") {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/root".to_string());
        format!("{}/{}", home, stripped)
    } else {
        path.to_string()
    }
}

fn corpus_dir() -> PathBuf {
    PathBuf::from(expand_tilde(CORPUS_DIR))
}

/// Download a Project Gutenberg book by ID.
/// Returns the path to the downloaded file.
fn download_gutenberg(book_id: u32) -> Option<PathBuf> {
    let dir = corpus_dir();
    let _ = fs::create_dir_all(&dir);

    let filename = format!("pg{}.txt", book_id);
    let path = dir.join(&filename);

    if path.exists() {
        eprintln!("  Already downloaded: {}", path.display());
        return Some(path);
    }

    let url = format!("https://www.gutenberg.org/cache/epub/{}/pg{}.txt", book_id, book_id);
    eprintln!("  Downloading {}...", url);

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .ok()?;

    let resp = client.get(&url).send().ok()?;
    if !resp.status().is_success() {
        eprintln!("  Failed: HTTP {}", resp.status());
        return None;
    }

    let text = resp.text().ok()?;
    let cleaned = clean_gutenberg(&text);

    if let Err(e) = fs::write(&path, &cleaned) {
        eprintln!("  Failed to write: {}", e);
        return None;
    }

    eprintln!("  Downloaded {} ({} chars)", filename, cleaned.len());
    Some(path)
}

/// Strip Project Gutenberg header/footer boilerplate.
fn clean_gutenberg(raw: &str) -> String {
    // Find start of actual content
    let start_markers = [
        "*** START OF THE PROJECT GUTENBERG",
        "*** START OF THIS PROJECT GUTENBERG",
        "***START OF THE PROJECT GUTENBERG",
        "*** START OF PROJECT GUTENBERG",
    ];

    let mut start_idx = 0usize;
    for marker in &start_markers {
        if let Some(idx) = raw.find(marker) {
            // Find the end of the line containing the marker
            if let Some(line_end) = raw[idx..].find('\n') {
                start_idx = idx + line_end + 1;
                break;
            }
        }
    }

    // Find end of content
    let end_markers = [
        "*** END OF THE PROJECT GUTENBERG",
        "*** END OF THIS PROJECT GUTENBERG",
        "***END OF THE PROJECT GUTENBERG",
        "*** END OF PROJECT GUTENBERG",
    ];

    let mut end_idx = raw.len();
    for marker in &end_markers {
        if let Some(idx) = raw.rfind(marker) {
            end_idx = idx;
            break;
        }
    }

    let content = &raw[start_idx..end_idx];

    // Clean up: remove excessive blank lines, normalize whitespace
    let lines: Vec<&str> = content
        .lines()
        .filter(|l| !l.trim().is_empty() || l.chars().all(|c| c.is_whitespace()))
        .collect();

    // Rejoin with single newlines, skip truly empty lines
    let mut result = String::new();
    let mut prev_empty = false;
    for line in &lines {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            if !prev_empty {
                result.push('\n');
                prev_empty = true;
            }
        } else {
            result.push_str(trimmed);
            result.push(' ');
            prev_empty = false;
        }
    }

    result
}

/// Download a curated set of public domain books for English language training.
fn download_training_corpus() -> Vec<PathBuf> {
    // Well-known public domain books with good English prose
    let books: &[(u32, &str)] = &[
        (1342, "Pride and Prejudice - Jane Austen"),
        (84, "Frankenstein - Mary Shelley"),
        (11, "Alice's Adventures in Wonderland - Lewis Carroll"),
        (1661, "The Adventures of Sherlock Holmes - Arthur Conan Doyle"),
        (2701, "Moby Dick - Herman Melville"),
        (98, "A Tale of Two Cities - Charles Dickens"),
        (43, "The Strange Case of Dr Jekyll and Mr Hyde - R.L. Stevenson"),
        (345, "Dracula - Bram Stoker"),
        (174, "The Picture of Dorian Gray - Oscar Wilde"),
        (76, "Adventures of Huckleberry Finn - Mark Twain"),
        (1260, "Jane Eyre - Charlotte Bronte"),
        (768, "Wuthering Heights - Emily Bronte"),
        (5200, "Metamorphosis - Franz Kafka"),
        (1184, "The Count of Monte Cristo - Alexandre Dumas"),
        (1232, "The Prince - Niccolo Machiavelli"),
        (244, "A Study in Scarlet - Arthur Conan Doyle"),
        (1400, "Great Expectations - Charles Dickens"),
        (4300, "Ulysses - James Joyce"),
        (2814, "Dubliners - James Joyce"),
        (2554, "Crime and Punishment - Fyodor Dostoevsky"),
    ];

    let mut paths = Vec::new();
    for (id, title) in books {
        eprintln!("📖 {}", title);
        if let Some(path) = download_gutenberg(*id) {
            paths.push(path);
        }
        // Be polite to the server
        std::thread::sleep(std::time::Duration::from_millis(500));
    }

    paths
}

/// List available corpus files.
fn list_corpus_files() -> Vec<PathBuf> {
    let dir = corpus_dir();
    if !dir.exists() {
        return vec![];
    }

    let mut files: Vec<PathBuf> = fs::read_dir(&dir)
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().map(|e| e == "txt").unwrap_or(false))
        .collect();

    files.sort();
    files
}

// ── Text chunking ──────────────────────────────────────────────────────────

/// Split text into sentence/paragraph chunks of roughly `chunk_size` chars.
/// Tries to break at sentence boundaries (., !, ?, newlines).
fn chunk_text(text: &str, chunk_size: usize) -> Vec<String> {
    let mut chunks = Vec::new();
    let mut current = String::new();
    let mut char_count = 0;

    for ch in text.chars() {
        current.push(ch);
        char_count += 1;

        // Break at sentence endings when we're near chunk_size
        if char_count >= chunk_size && (ch == '.' || ch == '!' || ch == '?') {
            let trimmed = current.trim().to_string();
            if trimmed.len() >= 20 {
                // Skip tiny fragments
                chunks.push(trimmed);
            }
            current.clear();
            char_count = 0;
        }
    }

    // Don't waste the remainder
    let trimmed = current.trim().to_string();
    if trimmed.len() >= 20 {
        chunks.push(trimmed);
    }

    chunks
}

// ── Retrieval testing ──────────────────────────────────────────────────────

#[derive(Debug, Clone)]
struct TestResults {
    hit_rate: f64,
    mean_relevance: f64,
    hits: usize,
    total: usize,
}

fn run_retrieval_tests(grid: &Grid, config: &EncoderConfig, text_store: &TextStore) -> TestResults {
    let mut hits = 0usize;
    let mut total_relevance = 0.0f64;
    let total = TEST_QUERIES.len();

    for (_fact, query) in TEST_QUERIES {
        let results = query_knowledge_with_text(grid, query, config, 5, Some(text_store));

        if !results.is_empty() {
            let top_rel = results[0].relevance;
            total_relevance += top_rel;
            if top_rel > 0.3 && results[0].text.is_some() {
                hits += 1;
            }
        }
    }

    TestResults {
        hit_rate: if total > 0 {
            hits as f64 / total as f64
        } else {
            0.0
        },
        mean_relevance: if total > 0 {
            total_relevance / total as f64
        } else {
            0.0
        },
        hits,
        total,
    }
}

// ── Main learning loop ─────────────────────────────────────────────────────

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let mut corpus_path: Option<String> = None;
    let mut chunk_size = DEFAULT_CHUNK_SIZE;
    let mut consolidate_every = DEFAULT_CONSOLIDATE_EVERY;
    let mut max_chunks: Option<u64> = None;
    let mut download = false;
    let mut show_status = false;
    let mut list_corpus = false;
    let mut fresh = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--corpus" | "-c" => {
                i += 1;
                corpus_path = Some(args[i].clone());
            }
            "--chunk-size" => {
                i += 1;
                chunk_size = args[i].parse().unwrap_or(DEFAULT_CHUNK_SIZE);
            }
            "--consolidate-every" => {
                i += 1;
                consolidate_every = args[i].parse().unwrap_or(DEFAULT_CONSOLIDATE_EVERY);
            }
            "--max-chunks" => {
                i += 1;
                max_chunks = Some(args[i].parse().unwrap_or(1000));
            }
            "--download-gutenberg" | "--download" => {
                download = true;
            }
            "--status" | "-s" => {
                show_status = true;
            }
            "--list-corpus" => {
                list_corpus = true;
            }
            "--fresh" => {
                fresh = true;
            }
            "--help" | "-h" => {
                print_help();
                return;
            }
            _ => {
                eprintln!("Unknown arg: {}", args[i]);
                print_help();
                return;
            }
        }
        i += 1;
    }

    if show_status {
        cmd_status();
        return;
    }

    if list_corpus {
        cmd_list_corpus();
        return;
    }

    if download {
        cmd_download();
        return;
    }

    // ── Main learning session ──────────────────────────────────────────
    cmd_learn(corpus_path, chunk_size, consolidate_every, max_chunks, fresh);
}

fn print_help() {
    eprintln!("sage-learn: Continuous Language Learning for the 256×256 NCA Brain");
    eprintln!();
    eprintln!("Usage:");
    eprintln!("  sage-learn --corpus <file>     Learn from a text corpus");
    eprintln!("  sage-learn --download-gutenberg Download training corpus");
    eprintln!("  sage-learn --list-corpus       List available corpus files");
    eprintln!("  sage-learn --status            Show learning progress");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  --chunk-size <n>        Chars per text chunk (default: {})", DEFAULT_CHUNK_SIZE);
    eprintln!("  --consolidate-every <n> Chunks between consolidation (default: {})", DEFAULT_CONSOLIDATE_EVERY);
    eprintln!("  --max-chunks <n>        Max chunks this session (default: unlimited)");
    eprintln!("  --fresh                 Start with a fresh brain (discard existing)");
}

fn cmd_status() {
    let progress = load_progress();
    let brain_path = default_brain_path();

    println!("═══ SAGE Learning Status ═══");
    println!();

    if Path::new(&brain_path).exists() {
        match sage::distributed_knowledge::brain_info(&brain_path) {
            Ok(header) => {
                println!("Brain: {}×{} grid, {} channels, v{}",
                    header.grid_size, header.grid_size, header.channels, header.version);
            }
            Err(e) => {
                println!("Brain: {} (header: {})", brain_path, e);
            }
        }

        // Quick load to get alive count
        let mut store = NCAKnowledge::new();
        if store.load(&brain_path).is_ok() {
            let alive = store.grid.alive_count();
            let entries = store.text_store.len();
            println!("  Alive cells: {} ({:.1}% of grid)", alive,
                alive as f64 / (GRID_SIZE * GRID_SIZE) as f64 * 100.0);
            println!("  Text entries: {}", entries);
        }
    } else {
        println!("Brain: not yet created");
    }

    println!();
    println!("Learning Progress:");
    println!("  Sessions completed: {}", progress.sessions_completed);
    println!("  Total chunks: {}", progress.total_chunks_processed);
    println!("  Total chars: {} ({:.1} MB)", progress.total_chars_processed,
        progress.total_chars_processed as f64 / 1_000_000.0);
    println!("  Corpus files: {}", progress.corpus_files_processed.len());
    for f in &progress.corpus_files_processed {
        println!("    - {}", f);
    }
    println!("  Best hit rate: {:.1}%", progress.best_hit_rate * 100.0);
    println!("  Best mean relevance: {:.3}", progress.best_mean_relevance);
    if !progress.last_session_time.is_empty() {
        println!("  Last session: {}", progress.last_session_time);
    }

    // Show corpus files available
    let files = list_corpus_files();
    if !files.is_empty() {
        println!();
        println!("Corpus files available:");
        for f in &files {
            let size = fs::metadata(f).map(|m| m.len()).unwrap_or(0);
            println!("  {} ({:.1} KB)", f.file_name().unwrap().to_string_lossy(),
                size as f64 / 1000.0);
        }
    }
}

fn cmd_list_corpus() {
    let files = list_corpus_files();
    if files.is_empty() {
        println!("No corpus files. Run --download-gutenberg to fetch books.");
        return;
    }
    println!("Available corpus files:");
    for f in &files {
        let size = fs::metadata(f).map(|m| m.len()).unwrap_or(0);
        println!("  {} ({:.1} KB)", f.file_name().unwrap().to_string_lossy(),
            size as f64 / 1000.0);
    }
}

fn cmd_download() {
    println!("═══ Downloading Training Corpus ═══");
    println!();
    let paths = download_training_corpus();
    println!();
    println!("Downloaded {} books:", paths.len());
    for p in &paths {
        let size = fs::metadata(p).map(|m| m.len()).unwrap_or(0);
        println!("  {} ({:.1} KB)", p.file_name().unwrap().to_string_lossy(),
            size as f64 / 1000.0);
    }
}

fn cmd_learn(
    corpus_path: Option<String>,
    chunk_size: usize,
    consolidate_every: usize,
    max_chunks: Option<u64>,
    fresh: bool,
) {
    let start_time = Instant::now();
    let mut progress = load_progress();

    // ── Load or create brain ────────────────────────────────────────────
    let brain_path = default_brain_path();
    let mut store = NCAKnowledge::new();

    if fresh {
        eprintln!("🧠 Creating fresh 256×256 brain");
        // Remove old brain if it exists
        let _ = fs::remove_file(&brain_path);
        let _ = fs::remove_file(brain_path.replace("brain.bin", "text_store.bin"));
    } else if Path::new(&brain_path).exists() {
        eprintln!("🧠 Loading existing brain...");
        match store.load(&brain_path) {
            Ok(()) => {
                let alive = store.grid.alive_count();
                let entries = store.text_store.len();
                eprintln!("   Loaded: {} alive cells, {} entries", alive, entries);
            }
            Err(e) => {
                eprintln!("   Failed to load brain: {}", e);
                eprintln!("   Starting fresh.");
                store = NCAKnowledge::new();
            }
        }
    } else {
        eprintln!("🧠 No existing brain — creating fresh 256×256");
    }

    // ── Find corpus ─────────────────────────────────────────────────────
    let corpus_file = if let Some(ref path) = corpus_path {
        PathBuf::from(path)
    } else {
        // Auto-select: pick the next unprocessed corpus file
        let all_files = list_corpus_files();
        let processed: std::collections::HashSet<String> = progress
            .corpus_files_processed
            .iter()
            .cloned()
            .collect();

        // First, check if there's a current in-progress corpus
        if !progress.current_corpus.is_empty() {
            let p = PathBuf::from(&progress.current_corpus);
            if p.exists() {
                // Check if we're near the end — if so, mark as finished and move on
                let file_len = fs::metadata(&p).map(|m| m.len()).unwrap_or(0) as usize;
                if progress.last_corpus_position as usize + 100 >= file_len {
                    let name = p.file_name().unwrap_or_default().to_string_lossy().to_string();
                    if !progress.corpus_files_processed.contains(&name) {
                        progress.corpus_files_processed.push(name.clone());
                    }
                    progress.last_corpus_position = 0;
                    progress.current_corpus = String::new();
                    save_progress(&progress);
                    eprintln!("✅ Finished corpus: {} (detected on load)", name);
                    // Pick next unprocessed
                    match all_files.iter().find(|f| {
                        let n = f.file_name().unwrap_or_default().to_string_lossy().to_string();
                        !processed.contains(&n) && n != name
                    }) {
                        Some(f) => f.clone(),
                        None => {
                            eprintln!("⚠️  All {} corpus files processed!", all_files.len());
                            return;
                        }
                    }
                } else {
                    p
                }
            } else {
                // Current corpus disappeared, pick next unprocessed
                match all_files.iter().find(|f| {
                    let name = f.file_name().unwrap_or_default().to_string_lossy().to_string();
                    !processed.contains(&name)
                }) {
                    Some(f) => f.clone(),
                    None => {
                        eprintln!("⚠️  All {} corpus files processed!", all_files.len());
                        return;
                    }
                }
            }
        } else {
            // No current corpus — pick the next unprocessed one
            match all_files.iter().find(|f| {
                let name = f.file_name().unwrap_or_default().to_string_lossy().to_string();
                !processed.contains(&name)
            }) {
                Some(f) => f.clone(),
                None => {
                    eprintln!("⚠️  All {} corpus files processed!", all_files.len());
                    eprintln!("   Consider: sage-learn --download-gutenberg");
                    eprintln!("   Or run with --corpus to specify a new file.");
                    return;
                }
            }
        }
    };

    let corpus_name = corpus_file
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    eprintln!("📖 Corpus: {} ({:.1} KB)", corpus_name,
        fs::metadata(&corpus_file).map(|m| m.len() as f64 / 1000.0).unwrap_or(0.0));

    // ── Read corpus ─────────────────────────────────────────────────────
    let corpus_text = match fs::read_to_string(&corpus_file) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("❌ Failed to read corpus: {}", e);
            return;
        }
    };

    // Resume from last position if continuing same corpus
    let raw_offset = if progress.current_corpus == corpus_file.to_string_lossy() {
        progress.last_corpus_position as usize
    } else {
        0
    };

    // Snap to nearest valid UTF-8 char boundary (byte offset may land mid-char)
    let start_offset = if raw_offset > 0 && raw_offset < corpus_text.len() {
        if corpus_text.is_char_boundary(raw_offset) {
            raw_offset
        } else {
            // Walk backward to find the start of the multi-byte char we're inside
            let mut snapped = raw_offset;
            while snapped > 0 && !corpus_text.is_char_boundary(snapped) {
                snapped -= 1;
            }
            eprintln!("   Snapped resume offset {} → {} (was mid-char)", raw_offset, snapped);
            snapped
        }
    } else {
        raw_offset
    };

    let text_to_process = if start_offset > 0 && start_offset < corpus_text.len() {
        eprintln!("   Resuming from offset {} ({:.1}%)", start_offset,
            start_offset as f64 / corpus_text.len() as f64 * 100.0);
        corpus_text[start_offset..].to_string()
    } else {
        corpus_text.clone()
    };

    // ── Chunk the text ───────────────────────────────────────────────────
    let chunks = chunk_text(&text_to_process, chunk_size);
    let total_chunks = chunks.len();
    let effective_max = max_chunks.unwrap_or(total_chunks as u64).min(total_chunks as u64);

    eprintln!("   {} chunks ({} chars), processing up to {}",
        total_chunks, text_to_process.len(), effective_max);

    if chunks.is_empty() {
        eprintln!("⚠️  No valid chunks extracted. Corpus may be too short or malformed.");
        return;
    }

    // ── Configure encoder ───────────────────────────────────────────────
    let config = EncoderConfig::default();

    // ── Configure NCA dynamics ──────────────────────────────────────────
    let dynamics = BrainDynamics::default();
    eprintln!("⚙️  NCA dynamics: decay={}, diffusion={}, competition={}",
        dynamics.self_decay, dynamics.diffusion, dynamics.competition);

    // ── Load trained NCA weights (if available) ─────────────────────────
    let weights_path = BrainNcaWeights::default_path();
    let trained_weights: Option<BrainNcaWeights> = if std::path::Path::new(&weights_path).exists() {
        match BrainNcaWeights::load(&weights_path) {
            Ok(w) => {
                eprintln!("⚙️  Loaded trained weights ({} params)", w.param_count());
                Some(w)
            }
            Err(e) => {
                eprintln!("⚠️  Failed to load trained weights: {}", e);
                None
            }
        }
    } else {
        eprintln!("ℹ️  No trained weights — using dynamics only");
        None
    };

    // ── Learning loop ───────────────────────────────────────────────────
    eprintln!();
    eprintln!("🔄 Learning session starting...");

    let mut session_chunks: u64 = 0;
    let mut session_chars: u64 = 0;
    let mut _last_test_results: Option<TestResults> = None;

    for (chunk_idx, chunk) in chunks.iter().enumerate() {
        if chunk_idx as u64 >= effective_max {
            break;
        }

        // Process chunk: encode into grid & run NCA dynamics
        let features = encode_text(chunk, &config);
        let pos = process_text(
            &mut store.grid,
            &features,
            &dynamics,
            0.7, // confidence — moderate for language learning
            &config,
            DEFAULT_NCA_STEPS,
        );

        // If trained weights available, also run a weighted NCA step
        // This applies the learned prediction routing on top of the dynamics
        if let Some(ref w) = trained_weights {
            sage::distributed_knowledge::brain_processor::nca_brain_step_weighted(
                &mut store.grid, w,
            );
        }
        store.text_store.insert(pos.0, pos.1, chunk.clone());

        session_chunks += 1;
        session_chars += chunk.len() as u64;

        // Run consolidation periodically
        if session_chunks % consolidate_every as u64 == 0 {
            store.grid.consolidate_knowledge(DEFAULT_CONSO_STEPS);
        }

        // Run retrieval tests periodically
        if session_chunks % DEFAULT_TEST_EVERY as u64 == 0 {
            let results = run_retrieval_tests(&store.grid, &config, &store.text_store);
            _last_test_results = Some(results.clone());

            let fill_pct = store.grid.alive_count() as f64
                / (GRID_SIZE * GRID_SIZE) as f64
                * 100.0;

            eprintln!(
                "   📊 chunk {} | alive {:.1}% | entries {} | hit_rate {:.1}% | mean_rel {:.3}",
                progress.total_chunks_processed + session_chunks,
                fill_pct,
                store.text_store.len(),
                results.hit_rate * 100.0,
                results.mean_relevance,
            );
        }

        // Progress dot every 100 chunks
        if session_chunks % 100 == 0 {
            eprint!(".");
            let _ = std::io::stderr().flush();
        }
    }

    eprintln!(); // newline after dots

    // ── Final consolidation ─────────────────────────────────────────────
    eprintln!("🧹 Final consolidation (5 steps)...");
    store.grid.consolidate_knowledge(5);

    // ── Final retrieval test ────────────────────────────────────────────
    eprintln!("📊 Final retrieval test...");
    let final_results = run_retrieval_tests(&store.grid, &config, &store.text_store);

    // ── Update progress ─────────────────────────────────────────────────
    let new_offset = start_offset
        + chunks
            .iter()
            .take(effective_max as usize)
            .map(|c| c.len() + 1) // +1 for the space/newline between chunks
            .sum::<usize>();

    progress.total_chunks_processed += session_chunks;
    progress.total_chars_processed += session_chars;
    progress.sessions_completed += 1;
    progress.last_corpus_position = new_offset as u64;
    progress.current_corpus = corpus_file.to_string_lossy().to_string();
    progress.alive_cells = store.grid.alive_count();
    progress.text_entries = store.text_store.len();
    progress.last_session_time = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    // Track best scores
    if final_results.hit_rate > progress.best_hit_rate {
        progress.best_hit_rate = final_results.hit_rate;
    }
    if final_results.mean_relevance > progress.best_mean_relevance {
        progress.best_mean_relevance = final_results.mean_relevance;
    }

    // Mark corpus as processed if we finished it (within 10 bytes of end)
    if new_offset + 100 >= corpus_text.len() {
        if !progress.corpus_files_processed.contains(&corpus_name) {
            progress.corpus_files_processed.push(corpus_name.clone());
        }
        progress.last_corpus_position = 0;
        progress.current_corpus = String::new();
        eprintln!("✅ Finished corpus: {}", corpus_name);
    }

    // ── Save brain ──────────────────────────────────────────────────────
    eprintln!("💾 Saving brain...");
    match store.save(&brain_path) {
        Ok(()) => eprintln!("   Saved: {} alive cells, {} entries",
            store.grid.alive_count(), store.text_store.len()),
        Err(e) => eprintln!("   ❌ Save failed: {}", e),
    }

    save_progress(&progress);

    // ── Report ───────────────────────────────────────────────────────────
    let elapsed = start_time.elapsed();
    let fill_pct = store.grid.alive_count() as f64
        / (GRID_SIZE * GRID_SIZE) as f64
        * 100.0;

    println!();
    println!("═══ Learning Session Complete ═══");
    println!("  Duration:       {:.1}s", elapsed.as_secs_f64());
    println!("  Chunks:         {} ({} chars, {:.1} KB)",
        session_chunks, session_chars, session_chars as f64 / 1000.0);
    println!("  Grid fill:      {:.1}% ({} / {} cells)",
        fill_pct, store.grid.alive_count(), GRID_SIZE * GRID_SIZE);
    println!("  Text entries:   {}", store.text_store.len());
    println!("  Hit rate:       {:.1}% ({}/{})",
        final_results.hit_rate * 100.0, final_results.hits, final_results.total);
    println!("  Mean relevance: {:.3}", final_results.mean_relevance);
    println!("  Best ever:      hit_rate={:.1}%, mean_rel={:.3}",
        progress.best_hit_rate * 100.0, progress.best_mean_relevance);
    println!("  Total progress: {} chunks, {} sessions",
        progress.total_chunks_processed, progress.sessions_completed);
    println!("  Brain saved:    {}", brain_path);
}
