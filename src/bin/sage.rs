//! Unified SAGE CLI
//!
//! Single binary for all SAGE operations:
//!   sage chat       — interactive chat with your local SAGE
//!   sage node       — manage the decentralized node
//!   sage version    — print version info
//!   sage config     — show/edit configuration

use clap::{Parser, Subcommand};
use sage::distributed_knowledge::{NCAKnowledge, KnowledgeStore, default_brain_path};

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Parser)]
#[command(
    name = "sage",
    about = "SAGE — Self-Adaptive General Explorer",
    version = VERSION,
    long_about = "Decentralized AI that learns, grows, and connects.\n\nGet started:\n  sage chat          Talk to your local SAGE\n  sage node start    Join the network"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Interactive chat with your local SAGE instance
    Chat {
        /// Use Ollama backend instead of embedded LLM
        #[arg(long)]
        ollama: bool,
        /// Ollama model to use (only with --ollama)
        #[arg(short, long, default_value = "qwen2.5:14b")]
        model: String,
        /// Ollama API URL (only with --ollama)
        #[arg(long, default_value = "http://localhost:11434")]
        ollama_url: String,
    },
    /// Manage the SAGE network node
    Node {
        #[command(subcommand)]
        command: NodeCommands,
    },
    /// Print version information
    Version,
    /// Show or edit SAGE configuration
    Config {
        /// Print config file path only
        #[arg(long)]
        path: bool,
    },
    /// Update SAGE to the latest version
    Update {
        /// Skip changelog display
        #[arg(long)]
        quiet: bool,
    },
}

#[derive(Subcommand)]
enum NodeCommands {
    /// Start the SAGE node
    Start {
        /// Port for peer gossip (0 = random)
        #[arg(short, long, default_value_t = 0)]
        port: u16,
        /// Port for chat connections
        #[arg(long, default_value_t = 19175)]
        chat_port: u16,
        /// Sync interval in seconds
        #[arg(long, default_value_t = 300)]
        sync_interval: u64,
        /// Disable mDNS discovery
        #[arg(long)]
        no_mdns: bool,
    },
    /// Stop the running SAGE node
    Stop,
    /// Show node status
    Status,
}

fn sage_home() -> std::path::PathBuf {
    std::env::var("SAGE_HOME")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| dirs::home_dir().unwrap_or_default().join(".sage"))
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        None | Some(Commands::Chat { .. }) => {
            let (ollama, model, ollama_url) = match cli.command {
                Some(Commands::Chat { ollama, model, ollama_url }) => (ollama, model, ollama_url),
                _ => (false, "qwen2.5:14b".to_string(), "http://localhost:11434".to_string()),
            };
            run_chat(ollama, &model, &ollama_url);
        }
        Some(Commands::Version) => {
            println!("sage {VERSION}");
        }
        Some(Commands::Config { path }) => {
            let config_path = sage_home().join("config.toml");
            if path {
                println!("{}", config_path.display());
            } else {
                match std::fs::read_to_string(&config_path) {
                    Ok(contents) => print!("{contents}"),
                    Err(_) => eprintln!("No config found at {}", config_path.display()),
                }
            }
        }
        Some(Commands::Update { quiet }) => {
            run_update(quiet);
        }
        Some(Commands::Node { command }) => match command {
            NodeCommands::Start { port, chat_port, sync_interval, no_mdns } => {
                run_node_start(port, chat_port, sync_interval, no_mdns);
            }
            NodeCommands::Stop => {
                run_node_stop();
            }
            NodeCommands::Status => {
                run_node_status();
            }
        },
    }
}

/// Launch the interactive TUI chat with brain visualization.
fn run_chat(prefer_ollama: bool, model: &str, ollama_url: &str) {
    if let Err(e) = sage::chat_tui::run(prefer_ollama, model, ollama_url) {
        eprintln!("Chat error: {e}");
        std::process::exit(1);
    }
}

/// Start the SAGE node.
fn run_node_start(port: u16, chat_port: u16, sync_interval: u64, no_mdns: bool) {
    use sage::network::{NetworkManager, NetworkConfig};
    use sage::network::identity::NodeIdentity;
    use std::sync::Arc;
    use tokio::sync::Mutex;
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    use tokio::net::TcpListener;

    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
    rt.block_on(async {
        let brain_path = default_brain_path();
        let mut knowledge = NCAKnowledge::new();
        let _ = knowledge.load(&brain_path); // load existing if available
        let knowledge: Arc<Mutex<NCAKnowledge>> = Arc::new(Mutex::new(knowledge));

        let identity = NodeIdentity::load_or_generate(None)
            .expect("Failed to load/create node identity");
        println!("🧠 SAGE Node starting...");
        println!("   Identity: {}", identity.node_id);
        println!("   Brain: {brain_path} ({} active cells)",
                 knowledge.lock().await.active_knowledge(0.01).len());

        let net_config = NetworkConfig {
            listen_port: port,
            mdns_enabled: !no_mdns,
            sync_interval_secs: sync_interval,
            ..Default::default()
        };

        let _net = NetworkManager::new(identity.clone(), net_config);
        println!("   Gossip port: {port}");
        println!("   Chat port: {chat_port}");
        println!("   mDNS: {}", if no_mdns { "disabled" } else { "enabled" });

        // Chat listener
        let chat_listener = TcpListener::bind(format!("0.0.0.0:{chat_port}"))
            .await
            .expect("Failed to bind chat port");
        println!("\n✅ Node running. Press Ctrl+C to stop.\n");

        let k = knowledge.clone();
        tokio::spawn(async move {
            loop {
                if let Ok((stream, _addr)) = chat_listener.accept().await {
                    let k2 = k.clone();
                    tokio::spawn(async move {
                        let (reader, mut writer) = stream.into_split();
                        let mut lines = BufReader::new(reader).lines();
                        let _ = writer.write_all(b"SAGE node connected.\n").await;
                        while let Ok(Some(line)) = lines.next_line().await {
                            let line = line.trim().to_string();
                            if line.is_empty() { continue; }
                            if line == "/quit" { break; }
                            let k3: tokio::sync::MutexGuard<'_, NCAKnowledge> = k2.lock().await;
                            let results = k3.query(&line, 3);
                            if results.is_empty() {
                                let _ = writer.write_all(b"No matching knowledge.\n").await;
                            } else {
                                for r in &results {
                                    let msg = format!(
                                        "  [{},{}] relevance={:.3}\n",
                                        r.position.0, r.position.1, r.relevance
                                    );
                                    let _ = writer.write_all(msg.as_bytes()).await;
                                }
                            }
                        }
                    });
                }
            }
        });

        // Periodic brain save
        let k = knowledge.clone();
        let bp = brain_path.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(sync_interval));
            loop {
                interval.tick().await;
                let k: tokio::sync::MutexGuard<'_, NCAKnowledge> = k.lock().await;
                if let Err(e) = k.save(&bp) {
                    eprintln!("Brain save error: {e}");
                }
            }
        });

        // Wait for ctrl+c
        tokio::signal::ctrl_c().await.ok();
        println!("\n🛑 Shutting down...");
        let k: tokio::sync::MutexGuard<'_, NCAKnowledge> = knowledge.lock().await;
        if let Err(e) = k.save(&brain_path) {
            eprintln!("Final save error: {e}");
        }
        println!("Brain saved. Goodbye!");
    });
}

fn run_node_stop() {
    let pid_file = sage_home().join("node.pid");
    match std::fs::read_to_string(&pid_file) {
        Ok(pid_str) => {
            if let Ok(pid) = pid_str.trim().parse::<u32>() {
                #[cfg(unix)]
                {
                    let _ = std::process::Command::new("kill")
                        .arg(pid.to_string())
                        .status();
                }
                println!("Sent stop signal to SAGE node (PID {pid})");
                let _ = std::fs::remove_file(&pid_file);
            } else {
                eprintln!("Invalid PID file");
            }
        }
        Err(_) => eprintln!("No running SAGE node found (no PID file)"),
    }
}

/// Self-update: download latest binary, preserve data, run migration if needed.
fn run_update(quiet: bool) {
    let home = sage_home();
    let bin_path = home.join("bin/sage");
    let repo = std::env::var("SAGE_REPO").unwrap_or_else(|_| "Caryyon/sage".to_string());

    // Detect OS/arch
    let os = if cfg!(target_os = "linux") { "linux" }
             else if cfg!(target_os = "macos") { "darwin" }
             else { eprintln!("Unsupported OS"); return; };
    let arch = if cfg!(target_arch = "x86_64") { "x86_64" }
               else if cfg!(target_arch = "aarch64") { "arm64" }
               else { eprintln!("Unsupported arch"); return; };

    // Fetch latest release tag from GitHub API
    let api_url = format!("https://api.github.com/repos/{repo}/releases/latest");
    let client = reqwest::blocking::Client::builder()
        .user_agent("sage-updater")
        .build()
        .unwrap();
    let version = match client.get(&api_url).send() {
        Ok(resp) if resp.status().is_success() => {
            match resp.text() {
                Ok(body) => {
                    body.split("\"tag_name\"").nth(1)
                        .and_then(|s| s.split('"').nth(1))
                        .map(|s| s.to_string())
                        .unwrap_or_default()
                }
                _ => { eprintln!("Failed to parse release info"); return; }
            }
        }
        _ => { eprintln!("Failed to check for updates"); return; }
    };

    if version.is_empty() {
        eprintln!("No releases found"); return;
    }

    println!("📦 Latest version: {version}");

    let binary_name = format!("sage-{os}-{arch}");
    let binary_url = format!("https://github.com/{repo}/releases/download/{version}/{binary_name}");
    let changelog_url = format!("https://github.com/{repo}/releases/tag/{version}");

    println!("🔄 Checking for updates...");

    // Show changelog
    if !quiet {
        match reqwest::blocking::get(&changelog_url) {
            Ok(resp) if resp.status().is_success() => {
                if let Ok(text) = resp.text() {
                    // Show first 40 lines
                    let preview: String = text.lines().take(40).collect::<Vec<_>>().join("\n");
                    println!("\n📋 Changelog:\n{preview}\n");
                }
            }
            _ => {} // changelog is optional
        }
    }

    // Download new binary
    println!("⬇️  Downloading from {binary_url}");
    match reqwest::blocking::get(&binary_url) {
        Ok(resp) if resp.status().is_success() => {
            match resp.bytes() {
                Ok(bytes) => {
                    // Write to temp file first
                    let tmp = home.join("bin/sage.tmp");
                    if let Err(e) = std::fs::write(&tmp, &bytes) {
                        eprintln!("Write error: {e}");
                        return;
                    }
                    // Make executable
                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::PermissionsExt;
                        let _ = std::fs::set_permissions(&tmp,
                            std::fs::Permissions::from_mode(0o755));
                    }
                    // Atomic rename
                    if let Err(e) = std::fs::rename(&tmp, &bin_path) {
                        eprintln!("Rename error: {e}");
                        return;
                    }
                    println!("✅ Binary updated at {}", bin_path.display());
                }
                Err(e) => { eprintln!("Download error: {e}"); return; }
            }
        }
        Ok(resp) => { eprintln!("Download failed: HTTP {}", resp.status()); return; }
        Err(e) => { eprintln!("Connection error: {e}"); return; }
    }

    // Run brain migration if needed
    let brain_path = default_brain_path();
    if std::path::Path::new(&brain_path).exists() {
        println!("🧠 Checking brain file for migration...");
        let mut knowledge = NCAKnowledge::new();
        match knowledge.load(&brain_path) {
            Ok(()) => {
                // Re-save with current version header
                if let Err(e) = knowledge.save(&brain_path) {
                    eprintln!("Migration save error: {e}");
                } else {
                    println!("✅ Brain file up to date");
                }
            }
            Err(e) => eprintln!("Warning: brain load error: {e}"),
        }
    }

    println!("\n🎉 Update complete! Data in {} preserved.", home.display());
}

fn run_node_status() {
    let home = sage_home();
    let config_path = home.join("config.toml");
    let brain_path = default_brain_path();
    let pid_file = home.join("node.pid");

    println!("SAGE Node Status");
    println!("─────────────────");
    println!("  Home:     {}", home.display());
    println!("  Config:   {} {}", config_path.display(),
             if config_path.exists() { "✓" } else { "✗" });
    println!("  Brain:    {brain_path} {}",
             if std::path::Path::new(&brain_path).exists() { "✓" } else { "✗" });

    if pid_file.exists() {
        if let Ok(pid) = std::fs::read_to_string(&pid_file) {
            println!("  Running:  PID {}", pid.trim());
        }
    } else {
        println!("  Running:  no");
    }
}
