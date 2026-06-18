//! sage-autolearn — Autonomous Learning Binary
//!
//! Single binary that discovers knowledge from all projects and ingests
//! into the NCA brain. No shell scripts, no GPU training, no corruption.
//! Called directly by cron every 10 minutes.
//!
//! Usage:
//!   sage-autolearn                    # Discover + ingest + report
//!   sage-autolearn --status           # Just show brain status

use sage::distributed_knowledge::{KnowledgeStore, NCAKnowledge, default_brain_path};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use regex::Regex;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 && args[1] == "--status" {
        cmd_status();
        return;
    }

    let brain_path = default_brain_path();
    let start = std::time::Instant::now();

    eprintln!("[{}] SAGE autolearn starting...",
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S"));

    // Phase 1: Discover
    let discoveries = discover_all();
    let facts: Vec<String> = discoveries.iter().map(|d| d.fact.clone()).collect();
    eprintln!("  Discovered: {} facts from {} topics", facts.len(),
        discoveries.iter().map(|d| &d.topic).collect::<std::collections::HashSet<_>>().len());

    // Phase 2: Ingest
    let mut knowledge = load_or_create_brain(&brain_path);
    let before_entries = knowledge.text_store.len();
    let before_alive = knowledge.grid.alive_count();

    for fact in &facts {
        knowledge.encode(fact, 0.85);
    }

    if let Err(e) = knowledge.save(&brain_path) {
        eprintln!("❌ Failed to save brain: {}", e);
        std::process::exit(1);
    }

    let after_entries = knowledge.text_store.len();
    let after_alive = knowledge.grid.alive_count();

    eprintln!("  Ingested: {} → {} entries, {} → {} alive cells",
        before_entries, after_entries, before_alive, after_alive);

    // Phase 3: Report
    let elapsed = start.elapsed().as_secs_f64();
    eprintln!("[{}] Complete. Brain: {} alive, {} entries ({:.1}s)",
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
        after_alive, after_entries, elapsed);

    // Append to history file for dashboard sparkline
    let history_line = format!("{} {} {}\n",
        chrono::Local::now().timestamp(), after_alive, after_entries);
    let _ = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(std::path::Path::new("/home/cwolff/.sage/brain_history.csv"))
        .and_then(|mut f| std::io::Write::write_all(&mut f, history_line.as_bytes()));
}

fn cmd_status() {
    let brain_path = default_brain_path();
    if !Path::new(&brain_path).exists() {
        println!("Brain: not created yet");
        return;
    }
    let knowledge = load_or_create_brain(&brain_path);
    let grid = &knowledge.grid;
    println!("Grid: {}×{} | Alive: {} ({:.1}%) | Entries: {}",
        grid.width, grid.height,
        grid.alive_count(),
        grid.alive_count() as f64 / (grid.width * grid.height) as f64 * 100.0,
        knowledge.text_store.len());
}

// ── Discovery (same logic as sage-discover, inlined) ───────────────────────

struct Discovery {
    fact: String,
    topic: String,
}

fn discover_all() -> Vec<Discovery> {
    let mut discoveries = Vec::new();

    // 1. SAGE's own codebase
    let sage_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    discoveries.extend(scan_codebase(&sage_dir, "sage"));

    // 2. User's projects
    if let Some(home) = dirs::home_dir() {
        let code_dir = home.join("Code");
        if code_dir.exists() {
            discoveries.extend(scan_project_overviews(&code_dir));
        }
    }

    // 3. System knowledge
    discoveries.extend(system_knowledge());

    // Deduplicate by fact text
    let mut seen = std::collections::HashSet::new();
    discoveries.retain(|d| seen.insert(d.fact.clone()));

    // Limit per topic
    let mut groups: HashMap<String, Vec<Discovery>> = HashMap::new();
    for d in discoveries {
        groups.entry(d.topic.clone()).or_default().push(d);
    }
    let mut result = Vec::new();
    for (_, mut facts) in groups {
        facts.truncate(30);
        result.extend(facts);
    }

    result
}

fn scan_codebase(dir: &Path, domain: &str) -> Vec<Discovery> {
    let mut discoveries = Vec::new();
    for entry in walk_dir(dir, 3) {
        let path = entry.path();
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
        let file_stem = path.file_stem().unwrap_or_default().to_string_lossy().to_string();

        match ext.as_str() {
            "rs" => discoveries.extend(scan_rust(&path, domain, &file_stem)),
            "js" | "ts" | "tsx" => discoveries.extend(scan_js(&path, domain, &file_stem)),
            "py" => discoveries.extend(scan_python(&path, domain, &file_stem)),
            "md" | "txt" => discoveries.extend(scan_markdown(&path, domain, &file_stem)),
            "toml" => discoveries.extend(scan_toml(&path, domain, &file_stem)),
            _ => {}
        }
    }
    discoveries
}

fn scan_project_overviews(code_dir: &Path) -> Vec<Discovery> {
    let mut discoveries = Vec::new();
    if let Ok(entries) = fs::read_dir(code_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let name = path.file_name().unwrap_or_default().to_string_lossy();
                if name.starts_with('.') { continue; }

                // README
                let readme = path.join("README.md");
                if readme.exists() {
                    if let Ok(content) = fs::read_to_string(&readme) {
                        for fact in extract_md_facts(&content) {
                            discoveries.push(Discovery {
                                fact: format!("Project {}: {}", name, fact),
                                topic: format!("project-{}", name),
                            });
                        }
                    }
                }

                // Cargo.toml / package.json
                for config_file in &["Cargo.toml", "package.json"] {
                    let config = path.join(config_file);
                    if config.exists() {
                        if let Ok(content) = fs::read_to_string(&config) {
                            discoveries.push(Discovery {
                                fact: format!("Project {} has config: {}",
                                    name, content.lines().take(5).collect::<Vec<_>>().join("; ")),
                                topic: format!("project-{}", name),
                            });
                        }
                    }
                }
            }
        }
    }
    discoveries
}

fn scan_rust(path: &Path, domain: &str, file_stem: &str) -> Vec<Discovery> {
    let mut discoveries = Vec::new();
    let content = match fs::read_to_string(path) { Ok(c) => c, Err(_) => return discoveries };

    // Doc comments
    let doc_re = Regex::new(r"(?m)^\s*///\s*(.+)$").unwrap();
    for cap in doc_re.captures_iter(&content) {
        let doc = cap[1].trim().to_string();
        if doc.len() > 20 && doc.len() < 300 && !doc.contains("TODO") && !doc.contains("FIXME") {
            discoveries.push(Discovery {
                fact: format!("In Rust {}: {}", file_stem, doc),
                topic: format!("{}-rust-{}", domain, file_stem),
            });
        }
    }

    // Struct definitions
    let struct_re = Regex::new(r"(?m)^\s*pub\s+struct\s+(\w+)").unwrap();
    for cap in struct_re.captures_iter(&content) {
        discoveries.push(Discovery {
            fact: format!("Rust struct {} defined in {}", &cap[1], file_stem),
            topic: format!("{}-rust-types", domain),
        });
    }

    // Function signatures
    let fn_re = Regex::new(r"(?m)^\s*pub\s+fn\s+(\w+)\s*\(([^)]*)\)(?:\s*->\s*([^\{]+))?").unwrap();
    for cap in fn_re.captures_iter(&content) {
        let name = &cap[1];
        let ret = cap.get(3).map(|m| m.as_str().trim()).unwrap_or("()");
        discoveries.push(Discovery {
            fact: format!("Function {}(...) returns {} in {}", name, ret, file_stem),
            topic: format!("{}-rust-api", domain),
        });
    }

    discoveries
}

fn scan_js(path: &Path, domain: &str, file_stem: &str) -> Vec<Discovery> {
    let mut discoveries = Vec::new();
    let content = match fs::read_to_string(path) { Ok(c) => c, Err(_) => return discoveries };

    let jsdoc_re = Regex::new(r"/\*\*\s*\n(.*?)\*/").unwrap();
    for cap in jsdoc_re.captures_iter(&content) {
        let doc = cap[1].replace('*', "").replace('/', "").trim().to_string();
        if doc.len() > 20 && doc.len() < 300 {
            discoveries.push(Discovery {
                fact: format!("In JS/TS {}: {}", file_stem, doc),
                topic: format!("{}-js-{}", domain, file_stem),
            });
        }
    }
    discoveries
}

fn scan_python(path: &Path, domain: &str, file_stem: &str) -> Vec<Discovery> {
    let mut discoveries = Vec::new();
    let content = match fs::read_to_string(path) { Ok(c) => c, Err(_) => return discoveries };

    let doc_re = Regex::new(r#"""([^"]{20,500})"""#).unwrap();
    for cap in doc_re.captures_iter(&content) {
        let doc = cap[1].trim().replace('\n', " ");
        if doc.len() > 20 {
            discoveries.push(Discovery {
                fact: format!("In Python {}: {}", file_stem, doc),
                topic: format!("{}-python-{}", domain, file_stem),
            });
        }
    }
    discoveries
}

fn scan_markdown(path: &Path, domain: &str, file_stem: &str) -> Vec<Discovery> {
    let mut discoveries = Vec::new();
    let content = match fs::read_to_string(path) { Ok(c) => c, Err(_) => return discoveries };
    discoveries.extend(extract_md_facts(&content).into_iter().map(|f| Discovery {
        fact: format!("In {} docs: {}", file_stem, f),
        topic: format!("{}-docs-{}", domain, file_stem),
    }));
    discoveries
}

fn extract_md_facts(content: &str) -> Vec<String> {
    let mut facts = Vec::new();
    let header_re = Regex::new(r"(?m)^#{1,3}\s+(.+)$\n\n?(.{20,500}?)(?:\n\n|$)").unwrap();
    for cap in header_re.captures_iter(content) {
        let header = cap[1].trim();
        let paragraph = cap[2].trim().replace('\n', " ");
        facts.push(format!("{}: {}", header, paragraph));
    }
    facts
}

fn scan_toml(path: &Path, domain: &str, file_stem: &str) -> Vec<Discovery> {
    let mut discoveries = Vec::new();
    let content = match fs::read_to_string(path) { Ok(c) => c, Err(_) => return discoveries };

    // Extract dependency names
    let dep_re = Regex::new(r"(?m)^\[dependencies\](.*?)(?:^\[|\z)").unwrap();
    if let Some(cap) = dep_re.captures(&content) {
        let deps: Vec<String> = cap[1].lines()
            .filter(|l| l.contains('=') && !l.starts_with('#'))
            .take(10)
            .map(|l| l.split('=').next().unwrap_or("").trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        if !deps.is_empty() {
            discoveries.push(Discovery {
                fact: format!("Project {} depends on: {}", file_stem, deps.join(", ")),
                topic: format!("{}-deps", domain),
            });
        }
    }
    discoveries
}

fn system_knowledge() -> Vec<Discovery> {
    vec![
        Discovery { fact: "SAGE uses Neural Cellular Automata for knowledge storage".into(), topic: "sage-core".into() },
        Discovery { fact: "The NCA grid is 256×256 cells with 38 channels per cell".into(), topic: "sage-core".into() },
        Discovery { fact: "Knowledge is encoded as activation patterns across the cellular grid".into(), topic: "sage-core".into() },
        Discovery { fact: "SAGE retrieves knowledge using cross-attention decoding".into(), topic: "sage-core".into() },
        Discovery { fact: "SAGE runs entirely locally with no cloud dependencies".into(), topic: "sage-core".into() },
        Discovery { fact: "Peer-to-peer gossip protocol enables decentralized knowledge sharing".into(), topic: "sage-core".into() },
        Discovery { fact: "SAGE is written in Rust for memory safety and performance".into(), topic: "sage-core".into() },
        Discovery { fact: "Rust ownership ensures memory safety without garbage collection".into(), topic: "rust-concepts".into() },
        Discovery { fact: "Rust borrowing allows temporary access without taking ownership".into(), topic: "rust-concepts".into() },
        Discovery { fact: "Rust lifetimes ensure references are valid for their entire use".into(), topic: "rust-concepts".into() },
        Discovery { fact: "Neural Cellular Automata use local rules to produce global patterns".into(), topic: "ml-concepts".into() },
        Discovery { fact: "Gradient descent minimizes loss by adjusting weights iteratively".into(), topic: "ml-concepts".into() },
        Discovery { fact: "Attention mechanisms let neural networks focus on relevant input".into(), topic: "ml-concepts".into() },
        Discovery { fact: "CUDA enables GPU-accelerated tensor operations for ML training".into(), topic: "ml-concepts".into() },
        Discovery { fact: "Embeddings map discrete tokens to continuous vector spaces".into(), topic: "ml-concepts".into() },
        Discovery { fact: "Linux process management uses signals like SIGKILL and SIGTERM".into(), topic: "systems".into() },
        Discovery { fact: "SSH key authentication is more secure than password-based auth".into(), topic: "systems".into() },
        Discovery { fact: "NFS allows remote filesystems to be mounted as local directories".into(), topic: "systems".into() },
        Discovery { fact: "Docker containers provide isolated environments for applications".into(), topic: "systems".into() },
        Discovery { fact: "cron jobs schedule recurring tasks on Unix-like systems".into(), topic: "systems".into() },
    ]
}

// ── Helpers ────────────────────────────────────────────────────────────────

fn load_or_create_brain(path: &str) -> NCAKnowledge {
    let mut knowledge = NCAKnowledge::new();
    if Path::new(path).exists() {
        match knowledge.load(path) {
            Ok(()) => {
                let alive = knowledge.grid.alive_count();
                let entries = knowledge.text_store.len();
                eprintln!("  Loaded brain: {} alive cells, {} entries", alive, entries);
            }
            Err(e) => {
                eprintln!("⚠️  Could not load brain (creating new): {}", e);
                // Don't abort — start fresh with empty brain
            }
        }
    }
    knowledge
}

fn walk_dir(dir: &Path, max_depth: usize) -> Vec<fs::DirEntry> {
    let mut results = Vec::new();
    if max_depth == 0 { return results; }
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                results.push(entry);
            } else if path.is_dir() {
                let name = path.file_name().unwrap_or_default().to_string_lossy();
                if !name.starts_with('.') && name != "target" && name != "node_modules" {
                    results.extend(walk_dir(&path, max_depth - 1));
                }
            }
        }
    }
    results
}
