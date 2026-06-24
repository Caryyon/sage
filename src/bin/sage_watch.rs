//! sage-watch: Continuous Learning Daemon
//!
//! Step 11 of the v0.6.0 plan. Monitors a directory for new documents
//! and automatically ingests them into the HDC store, then runs sleep
//! cycle consolidation to transfer knowledge into the NCA brain.
//!
//! Usage:
//!   sage-watch                          # Watch ~/.sage/inbox/ for new files
//!   sage-watch --dir /path/to/docs      # Watch custom directory
//!   sage-watch --once                   # Process once, don't watch
//!   sage-watch --consolidate            # Run consolidation after ingestion
//!   sage-watch --status                 # Show learning status
//!
//! The daemon:
//! 1. Watches the inbox directory for new .txt, .md, .json files
//! 2. For each new file: chunks it, embeds each chunk via Ollama (768-dim),
//!    stores in HDC. Also encodes into NCA grid for immediate availability.
//! 3. After threshold files: runs a sleep cycle (consolidation)
//! 4. Saves HDC store and NCA brain
//! 5. Continues watching for more files

use sage::distributed_knowledge::{default_brain_path, KnowledgeStore, NCAKnowledge};
use sage::distributed_knowledge::encoder::{encode_text, EncoderConfig};
use sage::hdc::{HdcStore, default_hdc_path};
use std::collections::HashSet;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

const DEFAULT_INBOX: &str = "~/.sage/inbox/";
const POLL_INTERVAL_SECS: u64 = 5;
const CHUNK_SIZE: usize = 2000; // chars per chunk
const MIN_CHUNK_SIZE: usize = 100; // skip tiny chunks
const CONSOLIDATION_THRESHOLD: usize = 5; // min new files before consolidation
const OLLAMA_EMBED_URL: &str = "http://localhost:11434/api/embed";
const OLLAMA_EMBED_MODEL: &str = "nomic-embed-text";
const HDC_DIM: usize = 768;

/// Supported file extensions
const SUPPORTED_EXTS: &[&str] = &["txt", "md", "json", "rst", "org"];

struct WatchState {
    processed_files: HashSet<String>,
    total_chunks: usize,
    total_files: usize,
    hdc_entries_start: usize,
    #[allow(dead_code)]
    start_time: Instant,
}

impl WatchState {
    fn new(hdc_entries: usize) -> Self {
        Self {
            processed_files: HashSet::new(),
            total_chunks: 0,
            total_files: 0,
            hdc_entries_start: hdc_entries,
            start_time: Instant::now(),
        }
    }
}

fn expand_tilde(path: &str) -> String {
    if path.starts_with("~") {
        if let Some(home) = dirs::home_dir() {
            return path.replace("~", &home.to_string_lossy());
        }
    }
    path.to_string()
}

fn inbox_dir() -> PathBuf {
    PathBuf::from(expand_tilde(DEFAULT_INBOX))
}

/// Get Ollama embedding (768-dim) for a single text.
/// Returns None if Ollama is not available.
fn ollama_embed(text: &str) -> Option<Vec<f32>> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .ok()?;

    let res = client
        .post(OLLAMA_EMBED_URL)
        .json(&serde_json::json!({"model": OLLAMA_EMBED_MODEL, "input": [text]}))
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

/// Check if Ollama embedding is available
fn ollama_available() -> bool {
    ollama_embed("test").is_some()
}

/// Chunk text into learning-sized pieces. Splits on paragraphs first,
/// then by sentences if paragraphs are too long.
fn chunk_text(text: &str, chunk_size: usize) -> Vec<String> {
    let mut chunks = Vec::new();

    let paragraphs: Vec<&str> = text.split("\n\n").collect();

    for para in paragraphs {
        let para = para.trim();
        if para.len() < MIN_CHUNK_SIZE {
            continue;
        }

        if para.len() <= chunk_size {
            chunks.push(para.to_string());
        } else {
            let mut current = String::new();
            for sentence in para.split(". ") {
                if current.len() + sentence.len() + 2 > chunk_size {
                    if !current.is_empty() {
                        chunks.push(current.trim().to_string());
                    }
                    current = sentence.to_string();
                } else {
                    if !current.is_empty() {
                        current.push_str(". ");
                    }
                    current.push_str(sentence);
                }
            }
            if !current.is_empty() && current.trim().len() >= MIN_CHUNK_SIZE {
                chunks.push(current.trim().to_string());
            }
        }
    }

    chunks
}

/// Ingest a file: embed chunks via Ollama → HDC, encode via encode_text → NCA.
fn ingest_file(
    path: &Path,
    hdc: &mut HdcStore,
    nca: &mut NCAKnowledge,
    config: &EncoderConfig,
    use_ollama: bool,
) -> Result<usize, String> {
    let mut content = String::new();
    fs::File::open(path)
        .map_err(|e| format!("Failed to open {}: {}", path.display(), e))?
        .read_to_string(&mut content)
        .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;

    let chunks = chunk_text(&content, CHUNK_SIZE);
    let mut ingested = 0;

    for chunk in &chunks {
        // HDC store: use Ollama embeddings (768-dim) to match existing store
        if use_ollama {
            if let Some(emb) = ollama_embed(chunk) {
                hdc.insert(&emb, chunk, 0.8);
            }
        } else {
            // Fallback: create a new HDC store at 384-dim if needed
            let emb = sage::distributed_knowledge::embedder::embed_text(chunk);
            if let Some(e) = emb {
                hdc.insert(&e, chunk, 0.8);
            }
        }

        // NCA grid: use encode_text (reduced features) for immediate availability
        nca.encode(chunk, 0.8);
        ingested += 1;
    }

    Ok(ingested)
}

/// Run a consolidation sleep cycle
fn run_consolidation(nca: &mut NCAKnowledge, verbose: bool) -> Result<usize, String> {
    use sage::consolidation::{ConsolidationConfig, ConsolidationEngine};

    let params = sage::grid::ConsolidationParams::load_or_default();
    let config = ConsolidationConfig {
        params,
        ..ConsolidationConfig::default()
    };

    let mut engine = ConsolidationEngine::new(config);
    let report = engine.sleep_cycle(verbose)?;

    let brain_path = default_brain_path();
    nca.load(&brain_path).ok();

    eprintln!(
        "   Sleep cycle: {} clusters, {} cells, {:.1}s",
        report.clusters_encoded, report.nca_knowledge_cells, report.duration_secs
    );

    Ok(report.clusters_encoded)
}

fn cmd_status() {
    let hdc_path = default_hdc_path();
    let brain_path = default_brain_path();

    println!("═══ SAGE Continuous Learning Status ═══");
    println!();

    if Path::new(&hdc_path).exists() {
        let hdc = HdcStore::load(Path::new(&hdc_path)).unwrap_or_else(|_| HdcStore::new(HDC_DIM));
        println!("HDC Store:     {} entries, {}-dim", hdc.entries.len(), hdc.dim);
    } else {
        println!("HDC Store:     not created yet");
    }

    if Path::new(&brain_path).exists() {
        let mut nca = NCAKnowledge::new();
        nca.load(&brain_path).ok();
        let active = nca.active_knowledge(0.01).len();
        println!(
            "NCA Brain:     {} active cells, {} text entries",
            active,
            nca.text_store.len()
        );
    } else {
        println!("NCA Brain:     not created yet");
    }

    // Ollama status
    if ollama_available() {
        println!("Ollama:        ✓ available (nomic-embed-text, {}-dim)", HDC_DIM);
    } else {
        println!("Ollama:        ✗ not available (will use fastembed 384-dim)");
    }

    let inbox = inbox_dir();
    if Path::new(&inbox).exists() {
        let files: Vec<_> = fs::read_dir(&inbox)
            .into_iter()
            .flatten()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path().extension().is_some_and(|ext| {
                    SUPPORTED_EXTS.contains(&ext.to_string_lossy().to_lowercase().as_str())
                })
            })
            .collect();
        println!("Inbox ({}): {} files waiting", inbox.display(), files.len());
    } else {
        println!("Inbox ({}): does not exist", inbox.display());
        println!("  mkdir -p {}", inbox.display());
    }
}

fn process_inbox(
    hdc: &mut HdcStore,
    nca: &mut NCAKnowledge,
    config: &EncoderConfig,
    state: &mut WatchState,
    use_ollama: bool,
    run_consolidation_after: bool,
) -> Result<usize, String> {
    let inbox = inbox_dir();
    fs::create_dir_all(&inbox).map_err(|e| format!("Failed to create inbox: {}", e))?;

    let files: Vec<_> = fs::read_dir(&inbox)
        .map_err(|e| format!("Failed to read inbox: {}", e))?
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path().extension().is_some_and(|ext| {
                SUPPORTED_EXTS.contains(&ext.to_string_lossy().to_lowercase().as_str())
            })
        })
        .collect();

    let mut new_files = 0;
    let mut new_chunks = 0;

    for file in &files {
        let path = file.path();
        let name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        if state.processed_files.contains(&name) {
            continue;
        }

        eprintln!(
            "📄 Ingesting: {} ({:.1} KB)",
            name,
            fs::metadata(&path)
                .map(|m| m.len() as f64 / 1000.0)
                .unwrap_or(0.0)
        );

        match ingest_file(&path, hdc, nca, config, use_ollama) {
            Ok(chunks) => {
                new_chunks += chunks;
                new_files += 1;
                state.processed_files.insert(name.clone());
                state.total_files += 1;
                state.total_chunks += chunks;
                eprintln!("   ✓ {} chunks ingested", chunks);

                // Move processed file to archive
                let archive = inbox.join(".processed");
                fs::create_dir_all(&archive).ok();
                let dest = archive.join(&name);
                fs::rename(&path, &dest).ok();
            }
            Err(e) => {
                eprintln!("   ❌ Failed: {}", e);
            }
        }
    }

    if new_files > 0 {
        // Save HDC store
        let hdc_path = default_hdc_path();
        if let Err(e) = hdc.save(Path::new(&hdc_path)) {
            eprintln!("⚠️  Failed to save HDC store: {}", e);
        }

        // Save NCA brain
        let brain_path = default_brain_path();
        if let Err(e) = nca.save(&brain_path) {
            eprintln!("⚠️  Failed to save brain: {}", e);
        }

        eprintln!(
            "\n📊 Session total: {} files, {} chunks ingested",
            new_files, new_chunks
        );
        eprintln!(
            "   HDC: {} → {} entries",
            state.hdc_entries_start,
            hdc.entries.len()
        );
        eprintln!("   NCA: {} text entries", nca.text_store.len());

        if run_consolidation_after && new_files >= CONSOLIDATION_THRESHOLD {
            eprintln!("\n🌙 Running consolidation sleep cycle...");
            run_consolidation(nca, true)?;
        }
    }

    Ok(new_files)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.iter().any(|a| a == "--status" || a == "-s") {
        cmd_status();
        return;
    }

    let once = args.iter().any(|a| a == "--once");
    let consolidate = args.iter().any(|a| a == "--consolidate" || a == "-c");

    let hdc_path = default_hdc_path();
    let brain_path = default_brain_path();

    // Determine embedding source
    let use_ollama = ollama_available();
    let hdc_dim = if use_ollama {
        HDC_DIM
    } else {
        // Check existing store dimension
        if Path::new(&hdc_path).exists() {
            HdcStore::load(Path::new(&hdc_path))
                .map(|s| s.dim)
                .unwrap_or(sage::distributed_knowledge::embedder::EMBED_DIM)
        } else {
            sage::distributed_knowledge::embedder::EMBED_DIM
        }
    };

    eprintln!("🐺 SAGE Continuous Learning Daemon");
    eprintln!("   Inbox:    {}", inbox_dir().display());
    eprintln!("   HDC:      {} ({}-dim)", hdc_path, hdc_dim);
    eprintln!("   Brain:    {}", brain_path);
    eprintln!(
        "   Embedder: {}",
        if use_ollama {
            "Ollama nomic-embed-text (768-dim)"
        } else {
            "fastembed AllMiniLML6V2 (384-dim)"
        }
    );
    eprintln!();

    let mut hdc = if Path::new(&hdc_path).exists() {
        HdcStore::load(Path::new(&hdc_path)).unwrap_or_else(|_| HdcStore::new(hdc_dim))
    } else {
        HdcStore::new(hdc_dim)
    };
    eprintln!("   HDC entries: {}", hdc.entries.len());

    let mut nca = NCAKnowledge::new();
    if Path::new(&brain_path).exists() {
        nca.load(&brain_path).ok();
    }
    eprintln!(
        "   NCA cells:   {} active, {} text entries",
        nca.active_knowledge(0.01).len(),
        nca.text_store.len()
    );
    eprintln!();

    let config = EncoderConfig::default();
    let mut state = WatchState::new(hdc.entries.len());

    fs::create_dir_all(inbox_dir()).ok();

    if once {
        match process_inbox(&mut hdc, &mut nca, &config, &mut state, use_ollama, consolidate) {
            Ok(n) => {
                if n == 0 {
                    eprintln!("📭 No new files in inbox");
                }
                eprintln!("\n✅ Done");
            }
            Err(e) => eprintln!("❌ Error: {}", e),
        }
        return;
    }

    eprintln!("👀 Watching for new files... (Ctrl-C to stop)\n");

    loop {
        match process_inbox(&mut hdc, &mut nca, &config, &mut state, use_ollama, consolidate) {
            Ok(n) => {
                if n > 0 {
                    eprintln!("   Processed {} file(s), waiting for more...\n", n);
                }
            }
            Err(e) => {
                eprintln!("❌ Error: {}", e);
            }
        }

        std::thread::sleep(Duration::from_secs(POLL_INTERVAL_SECS));
    }
}