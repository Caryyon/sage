//! SAGE Learning Daemon — Constant Knowledge Ingestion
//!
//! Watches sources for new information and auto-ingests into the NCA brain.
//!
//! ## Usage
//!   sage-learn --watch ~/Documents/notes/    # Watch directory for new files
//!   sage-learn --once ~/tmp/knowledge.md       # Ingest single file
//!   sage-learn --consolidate                   # Run dream cycle on existing brain
//!   sage-learn --status                        # Show brain stats
//!
//! ## How It Works
//! 1. Scans watched directories for `.txt`, `.md`, `.json` files
//! 2. Reads new/changed files, extracts facts
//! 3. Encodes into NCA brain via KnowledgeStore
//! 4. Runs consolidation (dream cycle) to strengthen associations
//! 5. Logs everything to `~/.sage/learn.log`

use clap::{Parser, Subcommand};
use sage::distributed_knowledge::{KnowledgeStore, NCAKnowledge};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Parser)]
#[command(name = "sage-learn", about = "SAGE continuous learning daemon")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Brain path (default: ~/.sage/brain.bin)
    #[arg(short, long, env = "SAGE_BRAIN_PATH", global = true)]
    brain: Option<String>,

    /// Confidence level for ingested facts (0.0-1.0)
    #[arg(short, long, default_value_t = 0.8)]
    confidence: f64,

    /// Be verbose
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Watch directories for new files and auto-ingest
    Watch {
        /// Directories to watch (can specify multiple)
        #[arg(required = true)]
        dirs: Vec<PathBuf>,

        /// Polling interval in seconds
        #[arg(short, long, default_value_t = 60)]
        interval: u64,

        /// Also run consolidation after each batch
        #[arg(short, long)]
        consolidate: bool,
    },
    /// Ingest a single file into the brain
    Once {
        /// File to ingest
        file: PathBuf,
    },
    /// Run consolidation (dream cycle) on existing brain
    Consolidate {
        /// Number of consolidation steps
        #[arg(short, long, default_value_t = 3)]
        steps: usize,
    },
    /// Show brain statistics
    Status,
}

fn main() {
    let cli = Cli::parse();
    let brain_path = cli
        .brain
        .unwrap_or_else(|| default_brain_path());

    match cli.command {
        Commands::Watch { dirs, interval, consolidate } => {
            cmd_watch(&dirs,
                &brain_path,
                interval,
                consolidate,
                cli.confidence,
                cli.verbose,
            );
        }
        Commands::Once { file } => {
            cmd_once(&file, &brain_path, cli.confidence, cli.verbose);
        }
        Commands::Consolidate { steps } => {
            cmd_consolidate(&brain_path, steps);
        }
        Commands::Status => {
            cmd_status(&brain_path);
        }
    }
}

/// Watch directories and auto-ingest new/changed files
fn cmd_watch(
    dirs: &[PathBuf],
    brain_path: &str,
    interval: u64,
    consolidate: bool,
    confidence: f64,
    verbose: bool,
) {
    println!("👁️  Watching {} director{} for new knowledge...",
        dirs.len(),
        if dirs.len() == 1 { "y" } else { "ies" });
    println!("   Brain: {}", brain_path);
    println!("   Interval: {}s | Confidence: {} | Consolidate: {}",
        interval, confidence, consolidate);
    println!("   Press Ctrl+C to stop\n");

    let mut seen_files: HashSet<(String, u64)> = HashSet::new();

    // Initial scan — mark all existing files as "seen"
    for dir in dirs {
        for entry in walk_dir(dir) {
            if let Ok(meta) = fs::metadata(&entry) {
                if let Ok(mtime) = meta.modified() {
                    let ts = mtime.duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
                    seen_files.insert((entry.to_string_lossy().to_string(), ts));
                }
            }
        }
    }

    println!("📂 Initial scan: {} files already tracked (will skip)", seen_files.len());

    let mut round = 0;

    loop {
        round += 1;
        let mut new_count = 0;
        let mut ingest_count = 0;

        for dir in dirs {
            for path in walk_dir(dir) {
                let key = match fs::metadata(&path) {
                    Ok(m) => {
                        match m.modified() {
                            Ok(t) => {
                                let ts = t.duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
                                let k = (path.to_string_lossy().to_string(), ts);
                                if seen_files.contains(&k) {
                                    continue; // Already seen
                                }
                                k
                            }
                            Err(_) => continue,
                        }
                    }
                    Err(_) => continue,
                };

                new_count += 1;

                if let Ok(text) = fs::read_to_string(&path) {
                    if !text.trim().is_empty() {
                        ingest_text(brain_path, &text, confidence, verbose);
                        ingest_count += 1;
                        log(&format!("Ingested {} ({} chars)", path.display(), text.len()));
                    }
                }

                seen_files.insert(key);
            }
        }

        if new_count > 0 {
            println!("🔄 Round {}: {} new files, {} ingested", round, new_count, ingest_count);

            if consolidate {
                cmd_consolidate(brain_path, 2);
            }
        } else if verbose {
            println!("🔄 Round {}: no new files", round);
        }

        std::thread::sleep(Duration::from_secs(interval));
    }
}

/// Ingest a single file into the brain
fn cmd_once(file: &Path, brain_path: &str, confidence: f64, verbose: bool) {
    println!("📄 Ingesting: {}", file.display());

    let text = fs::read_to_string(file).unwrap_or_else(|e| {
        eprintln!("❌ Failed to read file: {}", e);
        std::process::exit(1);
    });

    ingest_text(brain_path, &text, confidence, verbose);

    println!("💾 Brain saved to {}", brain_path);
}

/// Run consolidation (dream cycle) on existing brain
fn cmd_consolidate(brain_path: &str, steps: usize) {
    println!("🧠 Running consolidation ({} steps)...", steps);

    let mut knowledge = load_or_create_brain(brain_path);

    let before = knowledge.grid.alive_count();

    knowledge.grid.consolidate_knowledge(steps);

    let after = knowledge.grid.alive_count();

    if let Err(e) = knowledge.save(brain_path) {
        eprintln!("❌ Failed to save brain: {}", e);
        std::process::exit(1);
    }

    println!("✅ Consolidation complete. Alive cells: {} → {}", before, after);
}

/// Show brain status
fn cmd_status(brain_path: &str) {
    println!("🧠 Brain Status: {}", brain_path);

    if !Path::new(brain_path).exists() {
        println!("   Status: 🆕 New brain (not created yet)");
        return;
    }

    let knowledge = load_or_create_brain(brain_path);
    let grid = &knowledge.grid;

    println!("   Grid: {}×{} ({} cells total)", grid.width, grid.height, grid.width * grid.height);
    println!("   Alive cells: {} ({:.1}%)", grid.alive_count(),
        grid.alive_count() as f64 / (grid.width * grid.height) as f64 * 100.0);
    println!("   Total mass: {:.2}", grid.total_mass());
    println!("   Knowledge entries: {}", knowledge.text_store.len());
}

/// Ingest text into the brain
fn ingest_text(brain_path: &str, text: &str, confidence: f64, verbose: bool) {
    let mut knowledge = load_or_create_brain(brain_path);

    // Break text into chunks (sentences/paragraphs)
    let chunks = chunk_text(text, 200); // ~200 char chunks
    let chunk_count = chunks.len();

    for chunk in chunks {
        knowledge.encode(&chunk, confidence);
    }

    if verbose {
        println!("   Encoded {} chunks ({} chars total)", chunk_count, text.len());
    }

    if let Err(e) = knowledge.save(brain_path) {
        eprintln!("❌ Failed to save brain: {}", e);
    }
}

/// Load existing brain or create new one
fn load_or_create_brain(path: &str) -> NCAKnowledge {
    let mut knowledge = NCAKnowledge::new();

    if Path::new(path).exists() {
        if let Err(e) = knowledge.load(path) {
            eprintln!("⚠️  Could not load brain (creating new): {}", e);
        }
    }

    knowledge
}

/// Walk directory, return all .txt/.md/.json files
fn walk_dir(dir: &Path) -> Vec<PathBuf> {
    let mut results = Vec::new();

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                let ext = path.extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("")
                    .to_lowercase();
                if matches!(ext.as_str(), "txt" | "md" | "json" | "rs" | "py" | "js" | "ts") {
                    results.push(path);
                }
            } else if path.is_dir() {
                results.extend(walk_dir(&path));
            }
        }
    }

    results
}

/// Chunk text into ~max_len character pieces at sentence boundaries
fn chunk_text(text: &str, max_len: usize) -> Vec<String> {
    let mut chunks = Vec::new();
    let mut current = String::new();

    for sentence in text.split(|c: char| c == '.' || c == '!' || c == '?') {
        let trimmed = sentence.trim();
        if trimmed.is_empty() { continue; }

        let sentence_with_punct = format!("{}.", trimmed);

        if current.len() + sentence_with_punct.len() > max_len && !current.is_empty() {
            chunks.push(current.trim().to_string());
            current = sentence_with_punct;
        } else {
            current.push_str(&sentence_with_punct);
            current.push(' ');
        }
    }

    if !current.trim().is_empty() {
        chunks.push(current.trim().to_string());
    }

    chunks
}

fn log(msg: &str) {
    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
    let line = format!("[{}] {}\n", now, msg);

    let log_path = dirs::home_dir()
        .unwrap_or_default()
        .join(".sage")
        .join("learn.log");

    if let Some(parent) = log_path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    let _ = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .and_then(|mut f| {
            use std::io::Write;
            f.write_all(line.as_bytes())
        });
}

fn default_brain_path() -> String {
    dirs::home_dir()
        .unwrap_or_default()
        .join(".sage")
        .join("brain.bin")
        .to_string_lossy()
        .to_string()
}
