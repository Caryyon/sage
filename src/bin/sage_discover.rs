//! SAGE Content Discovery — Autonomous Knowledge Mining
//!
//! Scans codebases, documentation, and project files to extract
//! trainable knowledge. Turns implicit expertise into explicit
//! curriculum facts that feed the GPU NCA trainer.
//!
//! ## Discovery Sources
//! - Source code comments & docstrings (Rust, JS, TS, Python)
//! - README files & markdown documentation
//! - Dependency manifests (Cargo.toml, package.json)  
//! - Type definitions & function signatures
//! - Error messages & log patterns
//!
//! ## Usage
//!   sage-discover --scan ~/Code/          # Scan all projects
//!   sage-discover --scan ~/Code/sage      # Scan SAGE itself
//!   sage-discover --auto                  # Auto-discover everything
//!   sage-discover --output curriculum.json

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use clap::Parser;
use regex::Regex;
use serde_json::json;

#[derive(Parser)]
#[command(name = "sage-discover", about = "Autonomous knowledge discovery for SAGE")]
struct Cli {
    /// Directories to scan
    #[arg(short, long)]
    scan: Vec<PathBuf>,

    /// Auto-discover: scan SAGE source + well-known system paths
    #[arg(short, long)]
    auto: bool,

    /// Output curriculum JSON path
    #[arg(short, long, default_value = ".sage/auto-curriculum.json")]
    output: PathBuf,

    /// Minimum fact quality score (0-1)
    #[arg(long, default_value_t = 0.3)]
    min_quality: f64,

    /// Max facts per topic
    #[arg(long, default_value_t = 50)]
    max_facts: usize,

    /// Be verbose
    #[arg(short, long)]
    verbose: bool,
}

fn main() {
    let cli = Cli::parse();

    let mut discoveries: Vec<Discovery> = Vec::new();

    if cli.auto {
        // Auto-discover mode: scan SAGE + system knowledge
        println!("🔍 Auto-discovery mode");
        
        // 1. SAGE's own codebase
        let sage_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        println!("  Scanning SAGE source: {}", sage_dir.display());
        discoveries.extend(scan_codebase(&sage_dir, "sage-architecture", &cli));

        // 2. Rust standard library patterns (if available)
        if let Ok(rust_src) = std::env::var("RUST_SRC_PATH") {
            println!("  Scanning Rust stdlib: {}", rust_src);
            discoveries.extend(scan_codebase(Path::new(&rust_src), "rust-stdlib", &cli));
        }

        // 3. System knowledge (man pages, docs)
        println!("  Scanning system documentation...");
        discoveries.extend(discover_system_knowledge(&cli));

        // 4. User's projects
        let home = dirs::home_dir().unwrap_or_default();
        let code_dir = home.join("Code");
        if code_dir.exists() {
            println!("  Scanning user projects: {}", code_dir.display());
            // Only scan top-level READMEs and configs, not all source
            discoveries.extend(scan_project_overviews(&code_dir, &cli));
        }
    }

    // User-specified scans
    for dir in &cli.scan {
        println!("  Scanning: {}", dir.display());
        discoveries.extend(scan_codebase(dir, "discovered", &cli));
    }

    // Deduplicate and cluster into topics
    let curriculum = build_curriculum(discoveries, &cli);

    // Write output
    let output_path = if cli.output.is_absolute() {
        cli.output.clone()
    } else {
        dirs::home_dir().unwrap_or_default().join(&cli.output)
    };

    if let Some(parent) = output_path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    match fs::write(&output_path, serde_json::to_string_pretty(&curriculum).unwrap()) {
        Ok(_) => {
            let topics = curriculum["topics"].as_array().unwrap_or(&vec![]).len();
            let facts: usize = curriculum["topics"]
                .as_array()
                .unwrap_or(&vec![])
                .iter()
                .map(|t| t["facts"].as_array().unwrap_or(&vec![]).len())
                .sum();
            println!("\n💾 Curriculum saved: {}", output_path.display());
            println!("   Topics: {} | Facts: {}", topics, facts);
            println!("\n🏋️ Train with:");
            println!("   cargo run --bin gpu-train -- --curriculum {}", output_path.display());
        }
        Err(e) => eprintln!("❌ Failed to write curriculum: {}", e),
    }
}

/// A discovered fact with metadata
#[derive(Clone, Debug)]
struct Discovery {
    fact: String,
    domain: String,
    topic: String,
    source_file: String,
    quality: f64, // 0-1 score
}

/// Scan a codebase for knowledge
fn scan_codebase(dir: &Path, domain: &str, cli: &Cli) -> Vec<Discovery> {
    let mut discoveries = Vec::new();

    for entry in walk_dir(dir, 3) { // max depth 3
        let path = entry.path();
        let ext = path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        match ext.as_str() {
            "rs" => discoveries.extend(scan_rust_file(&path, domain, cli)),
            "js" | "ts" | "tsx" => discoveries.extend(scan_js_file(&path, domain, cli)),
            "py" => discoveries.extend(scan_python_file(&path, domain, cli)),
            "md" | "txt" => discoveries.extend(scan_markdown_file(&path, domain, cli)),
            "toml" => discoveries.extend(scan_toml_file(&path, domain, cli)),
            "json" => discoveries.extend(scan_json_file(&path, domain, cli)),
            _ => {}
        }
    }

    discoveries
}

/// Scan Rust source for doc comments and type knowledge
fn scan_rust_file(path: &Path, domain: &str, cli: &Cli) -> Vec<Discovery> {
    let mut discoveries = Vec::new();
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return discoveries,
    };

    let file_name = path.file_name().unwrap_or_default().to_string_lossy();

    // Extract doc comments (/// or //!)
    let doc_re = Regex::new(r"(?m)^\s*///\s*(.+)$").unwrap();
    for cap in doc_re.captures_iter(&content) {
        let doc = cap[1].trim();
        if doc.len() > 20 && doc.len() < 300 {
            discoveries.push(Discovery {
                fact: format!("In Rust: {}", doc),
                domain: domain.to_string(),
                topic: format!("rust-{}", file_name.trim_end_matches(".rs")),
                source_file: path.display().to_string(),
                quality: quality_score(doc, "doc"),
            });
        }
    }

    // Extract struct definitions with fields
    let struct_re = Regex::new(r"(?m)^\s*pub\s+struct\s+(\w+)\s*\{([^}]+)\}").unwrap();
    for cap in struct_re.captures_iter(&content) {
        let name = &cap[1];
        let fields = &cap[2];
        let field_count = fields.lines().filter(|l| !l.trim().is_empty()).count();
        discoveries.push(Discovery {
            fact: format!(
                "Rust struct {} has {} fields including: {}",
                name,
                field_count,
                fields.lines().take(3).map(|l| l.trim()).collect::<Vec<_>>().join(", ")
            ),
            domain: domain.to_string(),
            topic: format!("rust-types-{}", file_name.trim_end_matches(".rs")),
            source_file: path.display().to_string(),
            quality: 0.7,
        });
    }

    // Extract function signatures
    let fn_re = Regex::new(r"(?m)^\s*pub\s+fn\s+(\w+)\s*\(([^)]*)\)(?:\s*->\s*([^\{]+))?").unwrap();
    for cap in fn_re.captures_iter(&content) {
        let name = &cap[1];
        let params = &cap[2];
        let ret = cap.get(3).map(|m| m.as_str().trim()).unwrap_or("()");
        discoveries.push(Discovery {
            fact: format!(
                "Function {}({}) returns {}",
                name,
                params.split(',').take(3).map(|s| s.trim()).collect::<Vec<_>>().join(", "),
                ret
            ),
            domain: domain.to_string(),
            topic: format!("rust-api-{}", file_name.trim_end_matches(".rs")),
            source_file: path.display().to_string(),
            quality: 0.6,
        });
    }

    discoveries
}

/// Scan JS/TS for JSDoc and function knowledge
fn scan_js_file(path: &Path, domain: &str, cli: &Cli) -> Vec<Discovery> {
    let mut discoveries = Vec::new();
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return discoveries,
    };

    let file_name = path.file_name().unwrap_or_default().to_string_lossy();

    // JSDoc comments (simpler regex without lookaheads)
    let jsdoc_re = Regex::new(r"/\*\*\s*\n(.*?)\*/").unwrap();
    for cap in jsdoc_re.captures_iter(&content) {
        let doc = cap[1].replace("*", "").replace("/", "").trim().to_string();
        if doc.len() > 20 && doc.len() < 300 {
            discoveries.push(Discovery {
                fact: format!("In JavaScript/TypeScript: {}", doc),
                domain: domain.to_string(),
                topic: format!("js-{}", file_name.trim_end_matches(".js").trim_end_matches(".ts")),
                source_file: path.display().to_string(),
                quality: quality_score(&doc, "doc"),
            });
        }
    }

    discoveries
}

/// Scan Python for docstrings
fn scan_python_file(path: &Path, domain: &str, cli: &Cli) -> Vec<Discovery> {
    let mut discoveries = Vec::new();
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return discoveries,
    };

    let file_name = path.file_name().unwrap_or_default().to_string_lossy();

    // Docstrings
    let doc_re = Regex::new(r#"""([^"]{20,500})"""#).unwrap();
    for cap in doc_re.captures_iter(&content) {
        let doc = cap[1].trim().replace('\n', " ");
        if doc.len() > 20 {
            discoveries.push(Discovery {
                fact: format!("In Python: {}", doc),
                domain: domain.to_string(),
                topic: format!("python-{}", file_name.trim_end_matches(".py")),
                source_file: path.display().to_string(),
                quality: quality_score(&doc, "doc"),
            });
        }
    }

    discoveries
}

/// Scan markdown for section headers and content
fn scan_markdown_file(path: &Path, domain: &str, cli: &Cli) -> Vec<Discovery> {
    let mut discoveries = Vec::new();
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return discoveries,
    };

    let file_name = path.file_name().unwrap_or_default().to_string_lossy();

    // Extract headers and their following paragraph
    let header_re = Regex::new(r"(?m)^#{1,3}\s+(.+)$\n\n?(.{20,500}?)(?:\n\n|$)").unwrap();
    for cap in header_re.captures_iter(&content) {
        let header = cap[1].trim();
        let paragraph = cap[2].trim().replace('\n', " ");
        discoveries.push(Discovery {
            fact: format!("{}: {}", header, paragraph),
            domain: domain.to_string(),
            topic: format!("docs-{}", file_name.trim_end_matches(".md")),
            source_file: path.display().to_string(),
            quality: quality_score(&format!("{} {}", header, paragraph), "doc"),
        });
    }

    // Extract code blocks
    let code_re = Regex::new(r"```\w*\n([^`]{10,300})\n```").unwrap();
    for cap in code_re.captures_iter(&content) {
        let code = cap[1].trim().replace('\n', " ");
        if code.len() > 20 {
            discoveries.push(Discovery {
                fact: format!("Code example: {}", code.chars().take(200).collect::<String>()),
                domain: domain.to_string(),
                topic: format!("code-{}", file_name.trim_end_matches(".md")),
                source_file: path.display().to_string(),
                quality: 0.5,
            });
        }
    }

    discoveries
}

/// Scan Cargo.toml for dependency knowledge
fn scan_toml_file(path: &Path, domain: &str, cli: &Cli) -> Vec<Discovery> {
    let mut discoveries = Vec::new();
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return discoveries,
    };

    // Extract dependencies
    let dep_re = Regex::new(r#"^([\w-]+)\s*=\s*"([^"]+)""#).unwrap();
    for cap in dep_re.captures_iter(&content) {
        let name = &cap[1];
        let version = &cap[2];
        discoveries.push(Discovery {
            fact: format!("Project uses {} version {} as a dependency", name, version),
            domain: domain.to_string(),
            topic: format!("deps-{}", name),
            source_file: path.display().to_string(),
            quality: 0.5,
        });
    }

    discoveries
}

/// Scan JSON for schemas and configs
fn scan_json_file(path: &Path, domain: &str, cli: &Cli) -> Vec<Discovery> {
    let mut discoveries = Vec::new();
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return discoveries,
    };

    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
        // Extract top-level keys as facts
        if let Some(obj) = json.as_object() {
            for (key, value) in obj {
                let fact = match value {
                    serde_json::Value::String(s) => format!("{} is configured as '{}'", key, s),
                    serde_json::Value::Number(n) => format!("{} has value {}", key, n),
                    serde_json::Value::Bool(b) => format!("{} is {}", key, if *b { "enabled" } else { "disabled" }),
                    serde_json::Value::Array(a) => format!("{} contains {} items", key, a.len()),
                    _ => continue,
                };
                discoveries.push(Discovery {
                    fact,
                    domain: domain.to_string(),
                    topic: format!("config-{}", key),
                    source_file: path.display().to_string(),
                    quality: 0.4,
                });
            }
        }
    }

    discoveries
}

/// Scan only project-level files (READMEs, configs)
fn scan_project_overviews(dir: &Path, cli: &Cli) -> Vec<Discovery> {
    let mut discoveries = Vec::new();

    // Read top-level READMEs
    for readme_name in &["README.md", "README.rst", "readme.md"] {
        let readme = dir.join(readme_name);
        if readme.exists() {
            discoveries.extend(scan_markdown_file(&readme, "user-projects", cli));
        }
    }

    // Read package manifests
    let cargo = dir.join("Cargo.toml");
    if cargo.exists() {
        discoveries.extend(scan_toml_file(&cargo, "user-projects", cli));
    }

    let package = dir.join("package.json");
    if package.exists() {
        discoveries.extend(scan_json_file(&package, "user-projects", cli));
    }

    // Recurse into subdirectories (one level)
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                for readme_name in &["README.md", "readme.md"] {
                    let readme = path.join(readme_name);
                    if readme.exists() {
                        discoveries.extend(scan_markdown_file(&readme, "user-projects", cli));
                    }
                }
            }
        }
    }

    discoveries
}

/// Discover system-level knowledge
fn discover_system_knowledge(cli: &Cli) -> Vec<Discovery> {
    let mut discoveries = Vec::new();

    // Linux commands from man pages
    let commands = vec![
        ("systemd", "systemd is a system and service manager for Linux operating systems"),
        ("docker", "Docker is a platform for developing, shipping, and running applications in containers"),
        ("git", "Git is a distributed version control system for tracking code changes"),
        ("ssh", "SSH is a cryptographic network protocol for secure remote login and command execution"),
        ("nginx", "nginx is a high-performance HTTP server and reverse proxy"),
        ("postgres", "PostgreSQL is an advanced open-source relational database management system"),
        ("redis", "Redis is an in-memory data structure store used as a database, cache, and message broker"),
        ("curl", "curl is a command-line tool for transferring data with URLs using various protocols"),
        ("jq", "jq is a lightweight command-line JSON processor"),
        ("tmux", "tmux is a terminal multiplexer that enables multiple terminal sessions in a single window"),
        ("cargo", "Cargo is the Rust package manager and build system"),
        ("npm", "npm is the Node.js package manager for JavaScript dependencies"),
    ];

    for (name, desc) in commands {
        discoveries.push(Discovery {
            fact: desc.to_string(),
            domain: "system-tools".to_string(),
            topic: format!("tools-{}", name),
            source_file: "system".to_string(),
            quality: 0.8,
        });
    }

    // Rust concepts
    let rust_concepts = vec![
        "Rust ownership ensures memory safety without garbage collection by tracking who owns each value",
        "Borrowing in Rust allows temporary access to data without taking ownership",
        "Traits in Rust define shared behavior across types, similar to interfaces in other languages",
        "Lifetimes in Rust ensure references never outlive the data they point to",
        "The Result type in Rust is used for error handling with Ok and Err variants",
        "The Option type in Rust represents values that may be Some or None",
        "Pattern matching in Rust with match is exhaustive and checks all possible cases",
        "Iterators in Rust are lazy and can be chained with map, filter, and collect",
        "Closures in Rust can capture their environment by reference or value",
        "Unsafe Rust allows dereferencing raw pointers and calling unsafe functions",
    ];

    for concept in rust_concepts {
        discoveries.push(Discovery {
            fact: concept.to_string(),
            domain: "rust-fundamentals".to_string(),
            topic: "rust-concepts".to_string(),
            source_file: "builtin".to_string(),
            quality: 0.85,
        });
    }

    // Neural network / ML concepts
    let ml_concepts = vec![
        "Neural Cellular Automata use local rules to produce global patterns through iterative updates",
        "Gradient descent minimizes loss by adjusting weights in the direction of negative gradient",
        "Attention mechanisms allow neural networks to focus on relevant parts of the input",
        "Backpropagation computes gradients by applying the chain rule backwards through the network",
        "Autoencoders learn compressed representations by encoding input to a bottleneck then decoding",
        "Transformers use self-attention to process sequences in parallel without recurrence",
        "ReLU activation outputs the input if positive, otherwise zero, enabling efficient training",
        "Batch normalization stabilizes training by normalizing layer inputs across mini-batches",
        "Dropout prevents overfitting by randomly setting neurons to zero during training",
        "Cross-entropy loss measures the difference between predicted and true probability distributions",
    ];

    for concept in ml_concepts {
        discoveries.push(Discovery {
            fact: concept.to_string(),
            domain: "machine-learning".to_string(),
            topic: "ml-concepts".to_string(),
            source_file: "builtin".to_string(),
            quality: 0.85,
        });
    }

    discoveries
}

/// Build a curriculum JSON from discoveries
fn build_curriculum(discoveries: Vec<Discovery>, cli: &Cli) -> serde_json::Value {
    // Group by (domain, topic)
    let mut groups: HashMap<(String, String), Vec<Discovery>> = HashMap::new();
    for d in discoveries {
        groups.entry((d.domain.clone(), d.topic.clone())).or_default().push(d);
    }

    // Sort by quality and build topics
    let mut topics = Vec::new();
    for ((domain, topic_name), mut facts) in groups {
        facts.sort_by(|a, b| b.quality.partial_cmp(&a.quality).unwrap());
        facts.truncate(cli.max_facts);

        if facts.is_empty() {
            continue;
        }

        let fact_objects: Vec<serde_json::Value> = facts.into_iter().map(|f| {
            json!({
                "fact": f.fact,
                "source": f.source_file,
                "quality": f.quality,
            })
        }).collect();

        topics.push(json!({
            "name": topic_name.replace(" ", "-").to_lowercase(),
            "region": [0, 0, 64, 64],
            "facts": fact_objects,
        }));
    }

    json!({
        "name": "auto-discovered",
        "domain": "mixed-knowledge",
        "discovered_at": chrono::Utc::now().to_rfc3339(),
        "total_facts": topics.iter().map(|t| t["facts"].as_array().unwrap().len()).sum::<usize>(),
        "topics": topics,
    })
}

/// Walk directory with depth limit
fn walk_dir(dir: &Path, max_depth: usize) -> Vec<fs::DirEntry> {
    let mut results = Vec::new();
    if max_depth == 0 {
        return results;
    }

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                results.push(entry);
            } else if path.is_dir() {
                // Skip hidden, target, node_modules
                let name = path.file_name().unwrap_or_default().to_string_lossy();
                if !name.starts_with('.') && name != "target" && name != "node_modules" {
                    results.extend(walk_dir(&path, max_depth - 1));
                }
            }
        }
    }

    results
}

/// Calculate quality score for a fact
fn quality_score(text: &str, kind: &str) -> f64 {
    let mut score = 0.5;

    // Length bonus (sweet spot: 50-200 chars)
    let len = text.len();
    if len > 30 && len < 300 {
        score += 0.2;
    }
    if len > 50 && len < 200 {
        score += 0.1;
    }

    // Specificity indicators
    if text.contains("is a") || text.contains("is an") {
        score += 0.1; // Definitional
    }
    if text.contains("function") || text.contains("method") || text.contains("struct") {
        score += 0.1; // Technical
    }
    if text.contains("returns") || text.contains("takes") || text.contains("accepts") {
        score += 0.1; // API detail
    }

    // Penalties
    if text.contains("TODO") || text.contains("FIXME") || text.contains("XXX") {
        score -= 0.2;
    }
    if text.len() < 20 {
        score -= 0.3;
    }

    (score as f64).clamp(0.0, 1.0)
}
