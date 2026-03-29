//! Unified SAGE CLI
//!
//! Single binary for all SAGE operations:
//!   sage chat       — interactive chat with your local SAGE
//!   sage node       — manage the decentralized node
//!   sage version    — print version info
//!   sage config     — show/edit configuration

use clap::{Parser, Subcommand};
use sage::distributed_knowledge::{default_brain_path, KnowledgeStore, NCAKnowledge};

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
        /// Force Ollama backend (error if not running)
        #[arg(long, conflicts_with = "embedded")]
        ollama: bool,
        /// Force embedded SmolLM2 (skip Ollama auto-detection)
        #[arg(long, conflicts_with = "ollama")]
        embedded: bool,
        /// Ollama model to use
        #[arg(short, long, default_value = "qwen2.5:14b")]
        model: String,
        /// Ollama API URL
        #[arg(long, default_value = "http://localhost:11434")]
        ollama_url: String,
    },
    /// Manage local LLM models for bundled generation
    Model {
        #[command(subcommand)]
        command: ModelCommands,
    },
    /// Manage the SAGE network node
    Node {
        #[command(subcommand)]
        command: NodeCommands,
    },
    /// Run a dream cycle — let SAGE consolidate and explore its knowledge
    Dream {
        /// Show last N dream summaries instead of running a new dream
        #[arg(long)]
        show: bool,
        /// Number of dream steps to run
        #[arg(short, long, default_value_t = 5)]
        steps: usize,
    },
    /// Show knowledge statistics and insights
    Insights,
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
    /// Show SAGE system status dashboard
    Status,
    /// Export knowledge state to a .sage file
    Export {
        /// Output file path (e.g., my_knowledge.sage)
        file: String,
        /// Export only the diff since last export (for "sneakernet" sharing)
        #[arg(long)]
        share: bool,
    },
    /// Import knowledge from a .sage file
    Import {
        /// Input file path (e.g., my_knowledge.sage)
        file: String,
        /// Merge weight for incoming knowledge (0.0-1.0, default 0.5)
        #[arg(short, long, default_value_t = 0.5)]
        weight: f64,
        /// Merge a diff file (from --share export)
        #[arg(long)]
        merge: bool,
    },
    /// Search knowledge base (non-interactive)
    Search {
        /// Query string
        query: String,
        /// Maximum results to return
        #[arg(short = 'n', long, default_value_t = 5)]
        max_results: usize,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand)]
enum ModelCommands {
    /// Download a preset model from HuggingFace
    Download {
        /// Model preset name (phi3-mini, tinyllama, mistral-7b)
        name: String,
    },
    /// List available model presets
    List,
    /// Show current model status
    Status,
}

#[derive(Subcommand)]
enum NodeCommands {
    /// Start the SAGE node
    Start {
        /// Port for peer gossip (default: 4001, 0 = random)
        #[arg(short, long, default_value_t = 4001)]
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
        /// Run in foreground (don't daemonize)
        #[arg(short, long)]
        foreground: bool,
        /// Run as background daemon
        #[arg(long)]
        daemon: bool,
        /// Internal flag: running as daemon (hidden)
        #[arg(long, hide = true)]
        daemon_internal: bool,
        /// Disable stats beacon (don't post to whatssage.ai)
        #[arg(long)]
        no_beacon: bool,
    },
    /// Stop the running SAGE node
    Stop,
    /// Show node status
    Status,
    /// Check node health
    Health,
    /// Generate an invite link for direct peer connection
    Invite,
    /// Join a peer using an invite code
    Join {
        /// The invite code to join
        code: String,
    },
    /// Show the last 50 lines of the node log
    Logs {
        /// Follow the log output (like tail -f)
        #[arg(short, long)]
        follow: bool,
        /// Number of lines to show
        #[arg(short, long, default_value_t = 50)]
        lines: usize,
    },
    /// Export node stats as JSON for whatssage.ai
    Stats {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
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
            let (ollama, embedded, model, ollama_url) = match cli.command {
                Some(Commands::Chat {
                    ollama,
                    embedded,
                    model,
                    ollama_url,
                }) => (ollama, embedded, model, ollama_url),
                _ => (
                    false,
                    false,
                    "qwen2.5:14b".to_string(),
                    "http://localhost:11434".to_string(),
                ),
            };

            // Determine engine mode: force-ollama, force-embedded, or auto-detect
            let engine_mode = if ollama {
                sage::chat_tui::EngineMode::ForceOllama
            } else if embedded {
                sage::chat_tui::EngineMode::ForceEmbedded
            } else {
                sage::chat_tui::EngineMode::Auto
            };
            run_chat(engine_mode, &model, &ollama_url);
        }
        Some(Commands::Dream { show, steps }) => {
            if show {
                run_dream_show();
            } else {
                run_dream(steps);
            }
        }
        Some(Commands::Insights) => {
            run_insights();
        }
        Some(Commands::Model { command }) => match command {
            ModelCommands::Download { name } => {
                run_model_download(&name);
            }
            ModelCommands::List => {
                run_model_list();
            }
            ModelCommands::Status => {
                run_model_status();
            }
        },
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
        Some(Commands::Status) => {
            run_status();
        }
        Some(Commands::Export { file, share }) => {
            if share {
                run_export_share(&file);
            } else {
                run_export(&file);
            }
        }
        Some(Commands::Import { file, weight, merge }) => {
            if merge {
                run_import_merge(&file, weight);
            } else {
                run_import(&file, weight);
            }
        }
        Some(Commands::Search { query, max_results, json }) => {
            run_search(&query, max_results, json);
        }
        Some(Commands::Node { command }) => match command {
            NodeCommands::Start {
                port,
                chat_port,
                sync_interval,
                no_mdns,
                foreground,
                daemon,
                daemon_internal,
                no_beacon,
            } => {
                if foreground || daemon_internal {
                    // Run directly (foreground mode or we ARE the daemon)
                    if daemon_internal {
                        // Write PID file
                        let pid_file = sage_home().join("node.pid");
                        let _ = std::fs::create_dir_all(sage_home());
                        let _ = std::fs::write(&pid_file, std::process::id().to_string());
                    }
                    run_node_start(port, chat_port, sync_interval, no_mdns, foreground, no_beacon);
                    if daemon_internal {
                        // Clean up PID file on exit
                        let _ = std::fs::remove_file(sage_home().join("node.pid"));
                    }
                } else if daemon {
                    // Explicit --daemon flag: daemonize
                    daemonize_node(port, chat_port, sync_interval, no_mdns, no_beacon);
                } else {
                    // Default: daemonize (original behavior)
                    daemonize_node(port, chat_port, sync_interval, no_mdns, no_beacon);
                }
            }
            NodeCommands::Stop => {
                run_node_stop();
            }
            NodeCommands::Status => {
                run_node_status();
            }
            NodeCommands::Health => {
                run_node_health();
            }
            NodeCommands::Invite => {
                run_node_invite();
            }
            NodeCommands::Join { code } => {
                run_node_join(&code);
            }
            NodeCommands::Logs { follow, lines } => {
                run_node_logs(follow, lines);
            }
            NodeCommands::Stats { json } => {
                run_node_stats(json);
            }
        },
    }
}

/// Launch the interactive TUI chat with brain visualization.
fn run_chat(engine_mode: sage::chat_tui::EngineMode, model: &str, ollama_url: &str) {
    if let Err(e) = sage::chat_tui::run(engine_mode, model, ollama_url) {
        eprintln!("Chat error: {e}");
        std::process::exit(1);
    }
}

/// Daemonize: re-exec self with --daemon-internal, redirect output to log file.
fn daemonize_node(port: u16, chat_port: u16, sync_interval: u64, no_mdns: bool, no_beacon: bool) {
    use sage::network::identity::NodeIdentity;

    let home = sage_home();
    let _ = std::fs::create_dir_all(&home);
    let pid_file = home.join("node.pid");
    let log_file = home.join("node.log");

    // Check if already running
    if pid_file.exists() {
        if let Ok(pid_str) = std::fs::read_to_string(&pid_file) {
            if let Ok(pid) = pid_str.trim().parse::<u32>() {
                // Check if process is alive
                #[cfg(unix)]
                {
                    let status = std::process::Command::new("kill")
                        .args(["-0", &pid.to_string()])
                        .stderr(std::process::Stdio::null())
                        .status();
                    if status.map(|s| s.success()).unwrap_or(false) {
                        eprintln!("SAGE node is already running (PID {pid})");
                        eprintln!("Run `sage node stop` first, or `sage node status` to check.");
                        std::process::exit(1);
                    }
                }
                // Stale PID file, remove it
                let _ = std::fs::remove_file(&pid_file);
            }
        }
    }

    // Check if port is already in use before daemonizing
    if port != 0 {
        let addr = format!("0.0.0.0:{}", port);
        match std::net::TcpListener::bind(&addr) {
            Ok(_listener) => {
                // Port is available, listener will be dropped and released
            }
            Err(e) => {
                eprintln!("Error: Port {} is already in use: {}", port, e);
                eprintln!("Run `sage node stop` first, or choose a different port with --port.");
                std::process::exit(1);
            }
        }
    }

    // Load identity to show the node name
    let identity = NodeIdentity::load_or_generate(None).ok();
    let node_name = identity.as_ref().map(|i| i.human_name.as_str()).unwrap_or("unknown");
    let node_id = identity.as_ref().map(|i| i.node_id.as_str()).unwrap_or("unknown");

    let exe = std::env::current_exe().expect("Failed to determine executable path");
    let log = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_file)
        .expect("Failed to open log file");
    let log_err = log.try_clone().expect("Failed to clone log file handle");

    let mut cmd = std::process::Command::new(exe);
    cmd.args(["node", "start", "--daemon-internal"])
        .arg("--port")
        .arg(port.to_string())
        .arg("--chat-port")
        .arg(chat_port.to_string())
        .arg("--sync-interval")
        .arg(sync_interval.to_string());
    if no_mdns {
        cmd.arg("--no-mdns");
    }
    if no_beacon {
        cmd.arg("--no-beacon");
    }
    cmd.stdout(log)
        .stderr(log_err)
        .stdin(std::process::Stdio::null());

    // Detach from parent process group on Unix
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        cmd.process_group(0);
    }

    match cmd.spawn() {
        Ok(child) => {
            // Wait a moment to check if daemon started successfully
            std::thread::sleep(std::time::Duration::from_millis(500));

            println!("🌐 Starting SAGE node...");
            println!("🔑 Node ID: {} ({})", node_name, node_id);
            println!("📡 Listening on: /ip4/0.0.0.0/tcp/{}", port);
            println!("🔍 Searching for peers...");
            println!("✅ Node running (PID {}). Press Ctrl+C to stop.", child.id());
            println!();
            println!("   Log: {}", log_file.display());
            println!("   Stop: sage node stop");
        }
        Err(e) => {
            eprintln!("Failed to start daemon: {e}");
            std::process::exit(1);
        }
    }
}

/// Default bootstrap nodes for SAGE network discovery.
const BOOTSTRAP_NODES: &[&str] = &[
    // Placeholder bootstrap node - replace with actual when deployed
    // "/ip4/bootstrap.whatssage.ai/tcp/4001/p2p/12D3KooWDpJ7As7BWAwRMfu1VU2WCqNjvq387JEYKDBj4kx6nXTN",
];

/// Live network stats for the node UI
#[derive(Clone, Default)]
struct LiveNodeStats {
    peers_connected: usize,
    #[allow(dead_code)]
    patterns_received: u64,
    #[allow(dead_code)]
    patterns_sent: u64,
    start_time: Option<std::time::Instant>,
}

impl LiveNodeStats {
    fn uptime_str(&self) -> String {
        match self.start_time {
            Some(start) => {
                let secs = start.elapsed().as_secs();
                if secs < 60 {
                    format!("{}s", secs)
                } else if secs < 3600 {
                    format!("{}m {}s", secs / 60, secs % 60)
                } else {
                    let h = secs / 3600;
                    let m = (secs % 3600) / 60;
                    format!("{}h {}m", h, m)
                }
            }
            None => "0s".to_string(),
        }
    }
}

/// Start the SAGE node with live knowledge feed.
fn run_node_start(port: u16, chat_port: u16, sync_interval: u64, no_mdns: bool, foreground: bool, no_beacon: bool) {
    use sage::network::contribution::ContributionStats;
    use sage::network::identity::NodeIdentity;
    use sage::network::libp2p_transport::{Libp2pConfig, Libp2pTransport};
    use sage::network::gossip::GossipTransport;
    use sage::network::{NetworkConfig, NetworkManager};
    use std::sync::Arc;
    use std::io::Write;
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    use tokio::net::TcpListener;
    use tokio::sync::Mutex;

    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
    rt.block_on(async {
        let brain_path = default_brain_path();
        let mut knowledge = NCAKnowledge::new();
        let _ = knowledge.load(&brain_path); // load existing if available
        let initial_cells = knowledge.active_knowledge(0.01).len();
        let knowledge: Arc<Mutex<NCAKnowledge>> = Arc::new(Mutex::new(knowledge));

        // Load contribution stats
        let contrib = Arc::new(Mutex::new(ContributionStats::load_or_create()));

        let identity =
            NodeIdentity::load_or_generate(None).expect("Failed to load/create node identity");

        // Check if gossip port is available
        if port != 0 {
            let addr = format!("0.0.0.0:{}", port);
            match std::net::TcpListener::bind(&addr) {
                Ok(listener) => {
                    drop(listener); // Release the port immediately
                }
                Err(e) => {
                    eprintln!();
                    eprintln!("❌ Error: Port {} is already in use: {}", port, e);
                    eprintln!();
                    eprintln!("Try one of these:");
                    eprintln!("  • Run `sage node stop` to stop an existing node");
                    eprintln!("  • Use a different port: `sage node start --port 4002`");
                    eprintln!("  • Use a random port: `sage node start --port 0`");
                    std::process::exit(1);
                }
            }
        }

        // Check if chat port is available
        let chat_addr = format!("0.0.0.0:{}", chat_port);
        match std::net::TcpListener::bind(&chat_addr) {
            Ok(listener) => {
                drop(listener); // Release the port
            }
            Err(e) => {
                eprintln!();
                eprintln!("❌ Error: Chat port {} is already in use: {}", chat_port, e);
                eprintln!();
                eprintln!("Try one of these:");
                eprintln!("  • Run `sage node stop` to stop an existing node");
                eprintln!("  • Use a different chat port: `sage node start --chat-port 19176`");
                std::process::exit(1);
            }
        }

        // Get contribution tier
        let tier = {
            let c = contrib.lock().await;
            c.tier()
        };

        // Print the live knowledge feed header
        println!();
        println!("🌐 SAGE Node — {}", identity.human_name);
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("✅ Listening on /ip4/0.0.0.0/tcp/{}", port);
        println!("💾 Brain: {} active cells", initial_cells);
        println!("{} Contribution tier: {}", tier.emoji(), tier.name());
        println!("🔍 Searching for peers on local network...");
        println!();

        let net_config = NetworkConfig {
            listen_port: port,
            mdns_enabled: !no_mdns,
            sync_interval_secs: sync_interval,
            ..Default::default()
        };

        let _net = NetworkManager::new(identity.clone(), net_config);

        // Start libp2p transport for real peer discovery
        let bootstrap_nodes: Vec<String> = BOOTSTRAP_NODES.iter().map(|s| s.to_string()).collect();

        // Also load custom bootstrap peers from file
        let bootstrap_path = sage_home().join("bootstrap_peers.txt");
        let mut all_bootstrap: Vec<String> = bootstrap_nodes;
        if bootstrap_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&bootstrap_path) {
                for line in content.lines() {
                    let line = line.trim();
                    if !line.is_empty() && !all_bootstrap.contains(&line.to_string()) {
                        all_bootstrap.push(line.to_string());
                    }
                }
            }
        }

        let libp2p_config = Libp2pConfig {
            listen_port: port,
            mdns_enabled: !no_mdns,
            bootstrap_nodes: all_bootstrap,
        };
        let transport = Arc::new(Libp2pTransport::new(libp2p_config));

        match transport.start().await {
            Ok(()) => {}
            Err(e) => {
                eprintln!();
                eprintln!("❌ Error starting network transport: {}", e);
                std::process::exit(1);
            }
        }

        // Chat listener
        let chat_listener = match TcpListener::bind(format!("0.0.0.0:{chat_port}")).await {
            Ok(l) => l,
            Err(e) => {
                eprintln!();
                eprintln!("❌ Error: Failed to bind chat port {}: {}", chat_port, e);
                std::process::exit(1);
            }
        };

        // Live stats tracking
        let live_stats = Arc::new(Mutex::new(LiveNodeStats {
            start_time: Some(std::time::Instant::now()),
            ..Default::default()
        }));

        // Track connected peer names for display
        let peer_names: Arc<Mutex<std::collections::HashMap<String, String>>> =
            Arc::new(Mutex::new(std::collections::HashMap::new()));

        // Peer discovery and live feed task
        let transport_clone = transport.clone();
        let live_stats_clone = live_stats.clone();
        let peer_names_clone = peer_names.clone();
        let contrib_clone = contrib.clone();
        let is_foreground = foreground;
        let discovery_task = tokio::spawn(async move {
            let mut last_peers: std::collections::HashSet<String> = std::collections::HashSet::new();
            let mut last_stats_line_len = 0usize;

            loop {
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;

                let current_peers = transport_clone.connected_peers().await;
                let current_set: std::collections::HashSet<String> = current_peers.iter().cloned().collect();

                // Check for new connections
                for peer_id in current_set.difference(&last_peers) {
                    // Generate a human-readable name for the peer
                    let peer_name = generate_peer_display_name(peer_id);
                    let short_addr = peer_id.chars().take(20).collect::<String>();

                    let timestamp = chrono::Local::now().format("%H:%M:%S");
                    println!("[{}] 🤝 {} connected    ({}...)", timestamp, peer_name, short_addr);

                    // Store peer name
                    peer_names_clone.lock().await.insert(peer_id.clone(), peer_name.clone());

                    // Update contribution stats
                    {
                        let mut c = contrib_clone.lock().await;
                        c.unique_peers.insert(peer_id.clone());
                    }
                }

                // Check for disconnections
                for peer_id in last_peers.difference(&current_set) {
                    let peer_name = peer_names_clone.lock().await.get(peer_id).cloned()
                        .unwrap_or_else(|| "unknown".to_string());
                    let timestamp = chrono::Local::now().format("%H:%M:%S");
                    println!("[{}] 👋 {} disconnected", timestamp, peer_name);
                }

                // Update live stats
                {
                    let mut stats = live_stats_clone.lock().await;
                    stats.peers_connected = current_set.len();
                }

                // Update stats line (only in foreground mode)
                if is_foreground {
                    let stats = live_stats_clone.lock().await;
                    let contrib = contrib_clone.lock().await;

                    let stats_line = format!(
                        "\rNetwork stats: {} peers | {} received | {} sent | uptime {}",
                        stats.peers_connected,
                        contrib.patterns_received,
                        contrib.patterns_sent,
                        stats.uptime_str()
                    );

                    // Clear previous line and print new one
                    let padding = " ".repeat(last_stats_line_len.saturating_sub(stats_line.len()));
                    print!("{}{}", stats_line, padding);
                    let _ = std::io::stdout().flush();
                    last_stats_line_len = stats_line.len();
                }

                last_peers = current_set;
            }
        });

        // Log sync events
        let sync_log_path = sage_home().join("sync.log");

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
                            if line.is_empty() {
                                continue;
                            }
                            if line == "/quit" {
                                break;
                            }
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

        // Periodic brain save + sync logging + contribution save
        let k = knowledge.clone();
        let bp = brain_path.clone();
        let slp = sync_log_path.clone();
        let contrib_save = contrib.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(sync_interval));
            loop {
                interval.tick().await;
                let k: tokio::sync::MutexGuard<'_, NCAKnowledge> = k.lock().await;
                if let Err(e) = k.save(&bp) {
                    eprintln!("Brain save error: {e}");
                } else {
                    // Log the auto-save
                    if let Ok(mut file) = std::fs::OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open(&slp)
                    {
                        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
                        let _ = writeln!(file, "[{}] Auto-save: {} active cells", timestamp, k.active_knowledge(0.01).len());
                    }
                }

                // Save contribution stats
                {
                    let mut c = contrib_save.lock().await;
                    c.update_uptime();
                    let _ = c.save();
                }
            }
        });

        // Stats beacon task - posts to whatssage.ai every 60 seconds
        let beacon_task = if !no_beacon {
            let identity_beacon = identity.clone();
            let contrib_beacon = contrib.clone();
            let knowledge_beacon = knowledge.clone();
            Some(tokio::spawn(async move {
                const BEACON_URL: &str = "https://api.whatssage.ai/api/stats";
                let client = reqwest::Client::new();

                // Wait 10 seconds before first beacon to let node stabilize
                tokio::time::sleep(std::time::Duration::from_secs(10)).await;

                loop {
                    // Gather stats
                    let (active_cells, grid_utilization) = {
                        let k = knowledge_beacon.lock().await;
                        let active = k.active_knowledge(0.01).len();
                        let total = k.grid.width * k.grid.height;
                        let util = active as f64 / total as f64;
                        (active, util)
                    };

                    let (patterns_sent, patterns_received, peer_count, tier_name) = {
                        let c = contrib_beacon.lock().await;
                        (c.patterns_sent, c.patterns_received, c.peer_count(), c.tier().name().to_string())
                    };

                    // Calculate uptime from PID file
                    let uptime_secs: u64 = {
                        let pid_file = sage_home().join("node.pid");
                        if pid_file.exists() {
                            if let Ok(pid_str) = std::fs::read_to_string(&pid_file) {
                                if let Ok(pid) = pid_str.trim().parse::<u32>() {
                                    #[cfg(target_os = "linux")]
                                    {
                                        std::fs::metadata(format!("/proc/{}", pid))
                                            .ok()
                                            .and_then(|stat| stat.modified().ok())
                                            .and_then(|t| t.elapsed().ok())
                                            .map(|d| d.as_secs())
                                            .unwrap_or(0)
                                    }
                                    #[cfg(not(target_os = "linux"))]
                                    {
                                        let _ = pid;
                                        0
                                    }
                                } else { 0 }
                            } else { 0 }
                        } else { 0 }
                    };

                    let stats = serde_json::json!({
                        "node_name": identity_beacon.human_name,
                        "node_id": identity_beacon.node_id,
                        "version": VERSION,
                        "uptime_seconds": uptime_secs,
                        "patterns_sent": patterns_sent,
                        "patterns_received": patterns_received,
                        "peer_count": peer_count,
                        "active_cells": active_cells,
                        "grid_utilization": grid_utilization,
                        "contribution_tier": tier_name
                    });

                    // POST to beacon endpoint - fail silently
                    let _ = client.post(BEACON_URL)
                        .json(&stats)
                        .timeout(std::time::Duration::from_secs(10))
                        .send()
                        .await;

                    // Wait 60 seconds before next beacon
                    tokio::time::sleep(std::time::Duration::from_secs(60)).await;
                }
            }))
        } else {
            None
        };

        // Wait for ctrl+c
        tokio::signal::ctrl_c().await.ok();
        discovery_task.abort();
        if let Some(task) = beacon_task {
            task.abort();
        }

        println!();
        println!();
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("🛑 Shutting down...");

        // Stop transport
        if let Err(e) = transport.stop().await {
            eprintln!("Transport stop error: {e}");
        }

        // Save brain
        let k: tokio::sync::MutexGuard<'_, NCAKnowledge> = knowledge.lock().await;
        if let Err(e) = k.save(&brain_path) {
            eprintln!("Final save error: {e}");
        } else {
            // Log shutdown
            if let Ok(mut file) = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&sync_log_path)
            {
                let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
                let _ = writeln!(file, "[{}] Shutdown: {} active cells saved", timestamp, k.active_knowledge(0.01).len());
            }
        }

        // Save final contribution stats
        {
            let mut c = contrib.lock().await;
            c.update_uptime();
            let _ = c.save();
        }

        let _final_stats = live_stats.lock().await;
        let final_contrib = contrib.lock().await;
        println!("💾 Brain saved.");
        println!("📊 Session: {} peers | {} patterns received | {} sent",
            final_contrib.peer_count(),
            final_contrib.patterns_received,
            final_contrib.patterns_sent
        );
        println!("👋 Goodbye!");
    });
}

/// Generate a human-readable display name for a peer from its ID.
fn generate_peer_display_name(peer_id: &str) -> String {
    // Use first few bytes of peer ID to generate a consistent name
    let bytes: Vec<u8> = peer_id.bytes().take(4).collect();
    if bytes.len() >= 2 {
        let adj_idx = bytes[0] as usize % PEER_ADJECTIVES.len();
        let noun_idx = bytes[1] as usize % PEER_NOUNS.len();
        format!("{}-{}", PEER_ADJECTIVES[adj_idx], PEER_NOUNS[noun_idx])
    } else {
        "unknown-peer".to_string()
    }
}

const PEER_ADJECTIVES: &[&str] = &[
    "swift", "calm", "bold", "bright", "deep", "fair", "glad", "keen", "mild", "pure",
    "rare", "sage", "warm", "wise", "wild", "amber", "azure", "coral", "dusty", "golden",
];

const PEER_NOUNS: &[&str] = &[
    "harbor", "ridge", "creek", "grove", "haven", "brook", "cliff", "delta", "forge", "glade",
    "hill", "inlet", "knoll", "lake", "marsh", "nexus", "oasis", "peak", "quay", "river",
];

fn run_node_stop() {
    let pid_file = sage_home().join("node.pid");

    // Also check old sage.pid for backwards compatibility
    let old_pid_file = sage_home().join("sage.pid");
    let actual_pid_file = if pid_file.exists() {
        pid_file.clone()
    } else if old_pid_file.exists() {
        old_pid_file.clone()
    } else {
        eprintln!("No node is running.");
        eprintln!("Start one with: sage node start");
        return;
    };

    match std::fs::read_to_string(&actual_pid_file) {
        Ok(pid_str) => {
            if let Ok(pid) = pid_str.trim().parse::<u32>() {
                #[cfg(unix)]
                {
                    // Check if process exists first
                    let exists = std::process::Command::new("kill")
                        .args(["-0", &pid.to_string()])
                        .stderr(std::process::Stdio::null())
                        .status()
                        .map(|s| s.success())
                        .unwrap_or(false);

                    if !exists {
                        println!("Node was not running, cleaned up.");
                        let _ = std::fs::remove_file(&actual_pid_file);
                        // Also clean up old file if different
                        if actual_pid_file != pid_file {
                            let _ = std::fs::remove_file(&pid_file);
                        }
                        return;
                    }

                    // Send SIGTERM for graceful shutdown
                    println!("🛑 Stopping SAGE node (PID {})...", pid);
                    let _ = std::process::Command::new("kill")
                        .args(["-TERM", &pid.to_string()])
                        .status();

                    // Wait up to 5 seconds for graceful shutdown
                    for i in 0..10 {
                        std::thread::sleep(std::time::Duration::from_millis(500));
                        let still_running = std::process::Command::new("kill")
                            .args(["-0", &pid.to_string()])
                            .stderr(std::process::Stdio::null())
                            .status()
                            .map(|s| s.success())
                            .unwrap_or(false);

                        if !still_running {
                            println!("✅ Node stopped gracefully.");
                            let _ = std::fs::remove_file(&actual_pid_file);
                            if actual_pid_file != pid_file {
                                let _ = std::fs::remove_file(&pid_file);
                            }
                            return;
                        }

                        if i == 4 {
                            println!("   Waiting for graceful shutdown...");
                        }
                    }

                    // Still running? Send SIGKILL
                    println!("   Forcing shutdown (SIGKILL)...");
                    let _ = std::process::Command::new("kill")
                        .args(["-KILL", &pid.to_string()])
                        .status();

                    std::thread::sleep(std::time::Duration::from_millis(200));
                    println!("✅ Node stopped.");
                }

                #[cfg(not(unix))]
                {
                    println!("Cannot stop process on this platform. PID was: {}", pid);
                }

                let _ = std::fs::remove_file(&actual_pid_file);
                if actual_pid_file != pid_file {
                    let _ = std::fs::remove_file(&pid_file);
                }
            } else {
                eprintln!("Invalid PID file contents");
                let _ = std::fs::remove_file(&actual_pid_file);
            }
        }
        Err(_) => {
            eprintln!("No node is running.");
            eprintln!("Start one with: sage node start");
        }
    }
}

/// Check node health - detailed status with exit codes.
/// Exit codes: 0 = healthy, 1 = degraded, 2 = down
#[allow(unused_imports)]
fn run_node_health() {
    use sage::network::identity::NodeIdentity;

    let home = sage_home();
    let brain_path = default_brain_path();
    let pid_file = home.join("node.pid");
    let old_pid_file = home.join("sage.pid");
    let sync_log = home.join("sync.log");

    let mut health_issues: Vec<String> = Vec::new();
    let mut health_checks: Vec<(bool, &str)> = Vec::new();

    // Check 1: Process running
    let actual_pid_file = if pid_file.exists() {
        Some(pid_file.clone())
    } else if old_pid_file.exists() {
        Some(old_pid_file.clone())
    } else {
        None
    };

    let (process_running, pid) = if let Some(ref pf) = actual_pid_file {
        if let Ok(pid_str) = std::fs::read_to_string(pf) {
            if let Ok(pid) = pid_str.trim().parse::<u32>() {
                #[cfg(unix)]
                {
                    let alive = std::process::Command::new("kill")
                        .args(["-0", &pid.to_string()])
                        .stderr(std::process::Stdio::null())
                        .status()
                        .map(|s| s.success())
                        .unwrap_or(false);
                    (alive, Some(pid))
                }
                #[cfg(not(unix))]
                {
                    (true, Some(pid)) // Assume running on non-unix
                }
            } else {
                (false, None)
            }
        } else {
            (false, None)
        }
    } else {
        (false, None)
    };

    health_checks.push((process_running, "Process running"));
    if !process_running {
        health_issues.push("Node process not running".to_string());
    }

    // Check 2: Port bound (try to bind, if we can't, something is listening)
    let port_bound = {
        let port = 4001; // Default port
        match std::net::TcpListener::bind(format!("0.0.0.0:{}", port)) {
            Ok(_) => {
                // We could bind, so port is NOT in use (bad if node should be running)
                if process_running {
                    false // Node running but not listening? Bad
                } else {
                    true // Node not running, port not in use - expected
                }
            }
            Err(_) => {
                // Port in use - good if node is running
                true
            }
        }
    };
    health_checks.push((port_bound || !process_running, "Port 4001 bound"));
    if process_running && !port_bound {
        health_issues.push("Port 4001 not bound (node may have crashed)".to_string());
    }

    // Check 3: Recent sync activity
    let recent_sync = if sync_log.exists() {
        if let Ok(metadata) = std::fs::metadata(&sync_log) {
            if let Ok(modified) = metadata.modified() {
                if let Ok(elapsed) = modified.elapsed() {
                    // Consider healthy if synced in last hour
                    elapsed.as_secs() < 3600
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            false
        }
    } else {
        false
    };

    let last_sync_str = if sync_log.exists() {
        if let Ok(content) = std::fs::read_to_string(&sync_log) {
            content.lines().last()
                .and_then(|line| line.strip_prefix('[').and_then(|s| s.split(']').next()))
                .map(|s| s.to_string())
        } else {
            None
        }
    } else {
        None
    };

    // Only check sync if process is running
    if process_running {
        health_checks.push((recent_sync, "Last sync: recent"));
        if !recent_sync {
            health_issues.push("No recent sync activity".to_string());
        }
    }

    // Check 4: Brain file accessible
    let brain_accessible = std::path::Path::new(&brain_path).exists();
    health_checks.push((brain_accessible, "Brain file accessible"));
    if !brain_accessible {
        health_issues.push("Brain file not found".to_string());
    }

    // Determine overall health
    let (health_status, exit_code) = if !process_running {
        ("DOWN", 2)
    } else if health_issues.is_empty() {
        ("HEALTHY", 0)
    } else {
        ("DEGRADED", 1)
    };

    // Output
    println!();
    match health_status {
        "HEALTHY" => println!("Node health: ✅ HEALTHY"),
        "DEGRADED" => println!("Node health: ⚠️  DEGRADED"),
        "DOWN" => println!("Node health: ❌ DOWN"),
        _ => {}
    }
    println!();

    for (ok, check) in &health_checks {
        if *ok {
            println!("✅ {}", check);
        } else {
            println!("❌ {}", check);
        }
    }

    if let Some(pid) = pid {
        if process_running {
            println!("✅ Process running (PID {})", pid);
        }
    }

    if let Some(sync_time) = last_sync_str {
        println!("ℹ️  Last sync: {}", sync_time);
    }

    if !health_issues.is_empty() {
        println!();
        println!("Issues:");
        for issue in &health_issues {
            println!("  • {}", issue);
        }
    }

    println!();
    std::process::exit(exit_code);
}

/// Generate an invite link for direct peer connection.
fn run_node_invite() {
    use sage::network::identity::NodeIdentity;
    use sage::network::invite::InviteCode;

    let home = sage_home();
    let key_path = home.join("identity.key");

    // Load or create identity
    let identity = match NodeIdentity::load_or_generate(Some(&key_path)) {
        Ok(id) => id,
        Err(e) => {
            eprintln!("Failed to load node identity: {}", e);
            std::process::exit(1);
        }
    };

    // Generate multiaddr for this node (using default port 4001)
    // In a real implementation, we'd read the actual listening port from config
    let multiaddr = format!("/ip4/0.0.0.0/tcp/4001/p2p/{}", identity.node_id);

    let invite = InviteCode::generate(&multiaddr);

    println!();
    println!("🔗 Invite Code for {} ({})", identity.human_name, identity.node_id);
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();
    println!("  {}", invite.code);
    println!();
    println!("Share this code with a friend. They can run:");
    println!("  sage node join {}", invite.code);
    println!();
    println!("Valid for: {} (expires in {})", "24 hours", invite.time_remaining());
    println!();
}

/// Join a peer using an invite code.
fn run_node_join(code: &str) {
    use sage::network::invite::InviteCode;

    match InviteCode::parse(code) {
        Ok(invite) => {
            if !invite.is_valid() {
                eprintln!("❌ This invite code has expired.");
                std::process::exit(1);
            }

            println!("🔗 Invite code valid (expires in {})", invite.time_remaining());
            println!("   Connecting to: {}", invite.multiaddr);
            println!();

            // Save the peer address to bootstrap file for next node start
            let home = sage_home();
            let bootstrap_path = home.join("bootstrap_peers.txt");

            let mut peers: Vec<String> = if bootstrap_path.exists() {
                std::fs::read_to_string(&bootstrap_path)
                    .unwrap_or_default()
                    .lines()
                    .map(|s| s.to_string())
                    .collect()
            } else {
                Vec::new()
            };

            if !peers.contains(&invite.multiaddr) {
                peers.push(invite.multiaddr.clone());
                let _ = std::fs::write(&bootstrap_path, peers.join("\n"));
            }

            println!("✅ Added {} to bootstrap peers.", invite.multiaddr);
            println!();
            println!("Run `sage node start` to connect to this peer.");
        }
        Err(e) => {
            eprintln!("❌ Invalid invite code: {}", e);
            std::process::exit(1);
        }
    }
}

/// Show the node log file.
fn run_node_logs(follow: bool, lines: usize) {
    let log_file = sage_home().join("node.log");

    if !log_file.exists() {
        eprintln!("No node log found at {}", log_file.display());
        eprintln!("Start the node with: sage node start");
        return;
    }

    if follow {
        // Use tail -f for live following
        #[cfg(unix)]
        {
            let status = std::process::Command::new("tail")
                .args(["-f", "-n", &lines.to_string()])
                .arg(&log_file)
                .status();

            if let Err(e) = status {
                eprintln!("Failed to follow log: {}", e);
            }
        }
        #[cfg(not(unix))]
        {
            eprintln!("Live log following not supported on this platform.");
            eprintln!("Log file: {}", log_file.display());
        }
    } else {
        // Read and show last N lines
        match std::fs::read_to_string(&log_file) {
            Ok(content) => {
                let all_lines: Vec<&str> = content.lines().collect();
                let start = if all_lines.len() > lines {
                    all_lines.len() - lines
                } else {
                    0
                };

                for line in &all_lines[start..] {
                    println!("{}", line);
                }
            }
            Err(e) => {
                eprintln!("Failed to read log: {}", e);
            }
        }
    }
}

/// Export node stats as JSON (for whatssage.ai dashboard).
fn run_node_stats(json_output: bool) {
    use sage::network::contribution::ContributionStats;
    use sage::network::identity::NodeIdentity;

    let home = sage_home();
    let brain_path = default_brain_path();
    let key_path = home.join("identity.key");
    let pid_file = home.join("node.pid");

    // Load identity
    let identity = if key_path.exists() {
        NodeIdentity::load(&key_path).ok()
    } else {
        None
    };

    // Load contribution stats
    let contrib = ContributionStats::load_or_create();

    // Load brain for grid stats
    let mut knowledge = NCAKnowledge::new();
    let (active_cells, grid_utilization) = if std::path::Path::new(&brain_path).exists() {
        if knowledge.load(&brain_path).is_ok() {
            let active = knowledge.active_knowledge(0.01).len();
            let total = knowledge.grid.width * knowledge.grid.height;
            let util = active as f64 / total as f64;
            (active, util)
        } else {
            (0, 0.0)
        }
    } else {
        (0, 0.0)
    };

    // Check uptime
    let uptime_secs = if pid_file.exists() {
        if let Ok(pid_str) = std::fs::read_to_string(&pid_file) {
            if let Ok(pid) = pid_str.trim().parse::<u32>() {
                #[cfg(target_os = "linux")]
                {
                    if let Ok(stat) = std::fs::metadata(format!("/proc/{}", pid)) {
                        if let Ok(created) = stat.modified() {
                            if let Ok(elapsed) = created.elapsed() {
                                elapsed.as_secs()
                            } else {
                                0
                            }
                        } else {
                            0
                        }
                    } else {
                        0
                    }
                }
                #[cfg(not(target_os = "linux"))]
                {
                    let _ = pid;
                    0
                }
            } else {
                0
            }
        } else {
            0
        }
    } else {
        0
    };

    let tier = contrib.tier();

    if json_output {
        let stats = serde_json::json!({
            "node_name": identity.as_ref().map(|i| i.human_name.as_str()).unwrap_or("unknown"),
            "node_id": identity.as_ref().map(|i| i.node_id.as_str()).unwrap_or("unknown"),
            "version": VERSION,
            "uptime_seconds": uptime_secs,
            "patterns_sent": contrib.patterns_sent,
            "patterns_received": contrib.patterns_received,
            "peer_count": contrib.peer_count(),
            "active_cells": active_cells,
            "grid_utilization": grid_utilization,
            "contribution_tier": tier.name()
        });

        println!("{}", serde_json::to_string_pretty(&stats).unwrap_or_else(|_| "{}".to_string()));
    } else {
        println!();
        println!("SAGE Node Stats");
        println!("═══════════════");
        println!();

        if let Some(ref id) = identity {
            println!("Node:            {} ({})", id.human_name, id.node_id);
        }

        println!("Version:         v{}", VERSION);
        println!("Tier:            {}", tier);
        println!();
        println!("Patterns sent:   {}", contrib.patterns_sent);
        println!("Patterns recv:   {}", contrib.patterns_received);
        println!("Unique peers:    {}", contrib.peer_count());
        println!("Network age:     {}", contrib.network_age());
        println!("Total uptime:    {}", contrib.uptime_str());
        println!();
        println!("Active cells:    {}", active_cells);
        println!("Grid util:       {:.2}%", grid_utilization * 100.0);
        println!();

        if uptime_secs > 0 {
            let h = uptime_secs / 3600;
            let m = (uptime_secs % 3600) / 60;
            println!("Current session: {}h {}m", h, m);
        } else {
            println!("Current session: Node not running");
        }
    }
}

/// Self-update: download latest binary, preserve data, run migration if needed.
fn run_update(quiet: bool) {
    let home = sage_home();
    let bin_path = home.join("bin/sage");
    let repo = std::env::var("SAGE_REPO").unwrap_or_else(|_| "Caryyon/sage".to_string());

    // Detect OS/arch
    let os = if cfg!(target_os = "linux") {
        "linux"
    } else if cfg!(target_os = "macos") {
        "darwin"
    } else {
        eprintln!("Unsupported OS");
        return;
    };
    let arch = if cfg!(target_arch = "x86_64") {
        "x86_64"
    } else if cfg!(target_arch = "aarch64") {
        "arm64"
    } else {
        eprintln!("Unsupported arch");
        return;
    };

    // Fetch latest release tag from GitHub API
    let api_url = format!("https://api.github.com/repos/{repo}/releases/latest");
    let client = reqwest::blocking::Client::builder()
        .user_agent("sage-updater")
        .build()
        .unwrap();
    let version = match client.get(&api_url).send() {
        Ok(resp) if resp.status().is_success() => match resp.text() {
            Ok(body) => body
                .split("\"tag_name\"")
                .nth(1)
                .and_then(|s| s.split('"').nth(1))
                .map(|s| s.to_string())
                .unwrap_or_default(),
            _ => {
                eprintln!("Failed to parse release info");
                return;
            }
        },
        _ => {
            eprintln!("Failed to check for updates");
            return;
        }
    };

    if version.is_empty() {
        eprintln!("No releases found");
        return;
    }

    println!("📦 Latest version: {version}");

    let binary_name = format!("sage-{os}-{arch}");
    let binary_url = format!("https://github.com/{repo}/releases/download/{version}/{binary_name}");
    let changelog_url = format!("https://api.github.com/repos/{repo}/releases/tags/{version}");

    println!("🔄 Checking for updates...");

    // Show changelog
    if !quiet {
        match client.get(&changelog_url).send() {
            Ok(resp) if resp.status().is_success() => {
                if let Ok(text) = resp.text() {
                    // Extract "body" field from GitHub API JSON
                    if let Some(body) = text.split("\"body\":\"").nth(1) {
                        if let Some(body) = body.split("\",\"").next() {
                            let body = body.replace("\\r\\n", "\n").replace("\\n", "\n");
                            let preview: String =
                                body.lines().take(20).collect::<Vec<_>>().join("\n");
                            if !preview.is_empty() {
                                println!("\n📋 Release notes:\n{preview}\n");
                            }
                        }
                    }
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
                        let _ =
                            std::fs::set_permissions(&tmp, std::fs::Permissions::from_mode(0o755));
                    }
                    // Atomic rename
                    if let Err(e) = std::fs::rename(&tmp, &bin_path) {
                        eprintln!("Rename error: {e}");
                        return;
                    }
                    println!("✅ Binary updated at {}", bin_path.display());
                }
                Err(e) => {
                    eprintln!("Download error: {e}");
                    return;
                }
            }
        }
        Ok(resp) => {
            eprintln!("Download failed: HTTP {}", resp.status());
            return;
        }
        Err(e) => {
            eprintln!("Connection error: {e}");
            return;
        }
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

    println!(
        "\n🎉 Update complete! Data in {} preserved.",
        home.display()
    );
}

fn run_node_status() {
    use sage::network::identity::NodeIdentity;

    let home = sage_home();
    let brain_path = default_brain_path();
    let pid_file = home.join("node.pid");
    let old_pid_file = home.join("sage.pid");
    let log_file = home.join("node.log");
    let sync_log = home.join("sync.log");
    let key_path = home.join("identity.key");

    // Check both pid files
    let actual_pid_file = if pid_file.exists() {
        Some(pid_file.clone())
    } else if old_pid_file.exists() {
        Some(old_pid_file.clone())
    } else {
        None
    };

    // Load or create node identity to get the name
    let identity = if key_path.exists() {
        NodeIdentity::load(&key_path).ok()
    } else {
        None
    };

    // Check if running
    let running_info: Option<(u32, String)> = actual_pid_file.as_ref().and_then(|pf| {
        std::fs::read_to_string(pf).ok().and_then(|pid_str| {
            pid_str.trim().parse::<u32>().ok().and_then(|pid| {
                #[cfg(unix)]
                {
                    let alive = std::process::Command::new("kill")
                        .args(["-0", &pid.to_string()])
                        .stderr(std::process::Stdio::null())
                        .status()
                        .map(|s| s.success())
                        .unwrap_or(false);
                    if alive {
                        // Get uptime
                        let mut uptime_str = String::new();
                        #[cfg(target_os = "linux")]
                        {
                            if let Ok(stat) = std::fs::metadata(format!("/proc/{pid}")) {
                                if let Ok(created) = stat.modified() {
                                    if let Ok(elapsed) = created.elapsed() {
                                        let secs = elapsed.as_secs();
                                        let h = secs / 3600;
                                        let m = (secs % 3600) / 60;
                                        let s = secs % 60;
                                        if h > 0 {
                                            uptime_str = format!("{}h {}m", h, m);
                                        } else if m > 0 {
                                            uptime_str = format!("{}m {}s", m, s);
                                        } else {
                                            uptime_str = format!("{}s", s);
                                        }
                                    }
                                }
                            }
                        }
                        Some((pid, uptime_str))
                    } else {
                        // Stale PID file
                        let _ = std::fs::remove_file(pf);
                        None
                    }
                }
                #[cfg(not(unix))]
                {
                    Some((pid, String::new()))
                }
            })
        })
    });

    if running_info.is_none() {
        println!("No node running. Start with: sage node start");
        return;
    }

    let (pid, uptime_str) = running_info.unwrap();

    println!();
    println!("SAGE Node Status");
    println!("════════════════");
    println!();

    // Show node identity with human name
    if let Some(id) = &identity {
        println!("Node:     {}", id.human_name);
        let uptime_display = if uptime_str.is_empty() { "running" } else { &uptime_str };
        println!("State:    running (PID {}, uptime {})", pid, uptime_display);
    } else {
        println!("State:    running (PID {})", pid);
    }

    // Read sync log for peer info
    #[allow(unused_variables)]
    let peers_info: Vec<String> = Vec::new();
    let mut synced_received: usize = 0;
    let mut synced_sent: usize = 0;
    let mut last_sync: Option<String> = None;

    if sync_log.exists() {
        if let Ok(content) = std::fs::read_to_string(&sync_log) {
            for line in content.lines().rev().take(100) {
                // Parse sync log entries
                if line.contains("Synced") && line.contains("from") {
                    synced_received += 1;
                    if last_sync.is_none() {
                        // Extract timestamp
                        if let Some(ts) = line.strip_prefix('[').and_then(|s| s.split(']').next()) {
                            last_sync = Some(ts.to_string());
                        }
                    }
                }
                if line.contains("patterns") && line.contains("sent") {
                    synced_sent += 1;
                }
            }
        }
    }

    // Show peer count (we'd need IPC to get actual live peers, show placeholder)
    println!("Peers:    (run `sage node status` with node logs for live peers)");

    // Show sync stats from log
    if synced_received > 0 || synced_sent > 0 {
        println!("Synced:   {} patterns received, {} sent", synced_received, synced_sent);
    }

    // Brain stats
    let mut knowledge = NCAKnowledge::new();
    if std::path::Path::new(&brain_path).exists() {
        if knowledge.load(&brain_path).is_ok() {
            let active = knowledge.active_knowledge(0.01);
            let total_cells = knowledge.grid.width * knowledge.grid.height;
            let utilization = (active.len() as f64 / total_cells as f64) * 100.0;
            println!("Brain:    {} active cells ({:.1}% of grid)", active.len(), utilization);
        }
    }

    // Network address (from config or default)
    println!("Network:  /ip4/0.0.0.0/tcp/4001");
    println!();

    // Show log file location
    if log_file.exists() {
        println!("Log file: {}", log_file.display());
        // Show last few log lines
        if let Ok(content) = std::fs::read_to_string(&log_file) {
            let lines: Vec<_> = content.lines().rev().take(3).collect();
            if !lines.is_empty() {
                println!();
                println!("Recent log:");
                for line in lines.iter().rev() {
                    println!("  {}", line);
                }
            }
        }
    }
}

/// Download a model preset from HuggingFace
fn run_model_download(name: &str) {
    use indicatif::{ProgressBar, ProgressStyle};
    use sage::inference::local_llm::{get_preset, LocalLLM, MODEL_PRESETS};

    let preset = match get_preset(name) {
        Some(p) => p,
        None => {
            eprintln!("Unknown model: {}", name);
            eprintln!("\nAvailable presets:");
            for p in MODEL_PRESETS {
                eprintln!("  {:<12} {:>6}  {}", p.name, p.size, p.description);
            }
            std::process::exit(1);
        }
    };

    let dest_path = LocalLLM::default_path();
    let dest_dir = dest_path.parent().unwrap();

    // Create directory if needed
    if let Err(e) = std::fs::create_dir_all(dest_dir) {
        eprintln!("Failed to create directory: {}", e);
        std::process::exit(1);
    }

    // Check if model already exists
    if dest_path.exists() {
        println!("⚠️  Model already exists at: {}", dest_path.display());
        println!("   Delete it first if you want to re-download.");
        return;
    }

    println!("📥 Downloading {} ({})...", preset.name, preset.size);
    println!("   {}", preset.description);
    println!("   From: {}", preset.url);
    println!("   To:   {}", dest_path.display());
    println!();

    // Download with progress bar
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(3600)) // 1 hour timeout for large files
        .build()
        .expect("Failed to create HTTP client");

    let response = match client.get(preset.url).send() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Download failed: {}", e);
            std::process::exit(1);
        }
    };

    if !response.status().is_success() {
        eprintln!("Download failed: HTTP {}", response.status());
        std::process::exit(1);
    }

    let total_size = response.content_length().unwrap_or(0);
    let pb = ProgressBar::new(total_size);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
            .expect("Invalid progress bar template")
            .progress_chars("#>-"),
    );

    // Download to temp file first
    let temp_path = dest_dir.join(format!("{}.download", preset.filename));
    let mut file = match std::fs::File::create(&temp_path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Failed to create file: {}", e);
            std::process::exit(1);
        }
    };

    use std::io::{Read, Write};
    let mut downloaded: u64 = 0;
    let mut buffer = [0u8; 8192];
    let mut reader = response;

    loop {
        match reader.read(&mut buffer) {
            Ok(0) => break,
            Ok(n) => {
                if let Err(e) = file.write_all(&buffer[..n]) {
                    let _ = std::fs::remove_file(&temp_path);
                    eprintln!("Write error: {}", e);
                    std::process::exit(1);
                }
                downloaded += n as u64;
                pb.set_position(downloaded);
            }
            Err(e) => {
                let _ = std::fs::remove_file(&temp_path);
                eprintln!("Download error: {}", e);
                std::process::exit(1);
            }
        }
    }

    pb.finish_with_message("Download complete");

    // Rename to final location
    if let Err(e) = std::fs::rename(&temp_path, &dest_path) {
        let _ = std::fs::remove_file(&temp_path);
        eprintln!("Failed to move file: {}", e);
        std::process::exit(1);
    }

    println!();
    println!("✅ Model downloaded successfully!");
    println!("   Path: {}", dest_path.display());
    println!();
    println!("SAGE will now use local generation by default.");
    println!("Run `sage chat` to start chatting!");
}

/// List available model presets
fn run_model_list() {
    use sage::inference::local_llm::MODEL_PRESETS;

    println!("Available model presets:");
    println!();
    println!("  {:<12} {:>6}  {}", "NAME", "SIZE", "DESCRIPTION");
    println!("  {}", "-".repeat(60));
    for p in MODEL_PRESETS {
        println!("  {:<12} {:>6}  {}", p.name, p.size, p.description);
    }
    println!();
    println!("Download with: sage model download <name>");
    println!("Example:       sage model download phi3-mini");
}

/// Show current model status
fn run_model_status() {
    use sage::inference::local_llm::LocalLLM;
    use sage::inference::{detect_generation_backend, GenerationBackend};

    let model_path = LocalLLM::default_path();
    let backend = detect_generation_backend();

    println!("SAGE Model Status");
    println!("─────────────────");
    println!();

    // Model file status
    if model_path.exists() {
        if let Ok(metadata) = std::fs::metadata(&model_path) {
            let size = metadata.len();
            let gb = size as f64 / 1_073_741_824.0;
            let size_str = if gb >= 1.0 {
                format!("{:.1} GB", gb)
            } else {
                format!("{:.0} MB", size as f64 / 1_048_576.0)
            };
            println!("  Model file: {} ✓", model_path.display());
            println!("  Size:       {}", size_str);
        }
    } else {
        println!("  Model file: {} ✗ (not found)", model_path.display());
    }

    println!();

    // Backend status
    match backend {
        GenerationBackend::Local { model_name, model_size } => {
            println!("  ⚡ Generation: local ({}, {})", model_name, model_size);
        }
        GenerationBackend::Ollama { model } => {
            println!("  🔗 Generation: Ollama ({})", model);
        }
        GenerationBackend::Embedded { model_name } => {
            println!("  🧠 Generation: embedded ({})", model_name);
        }
        GenerationBackend::Offline => {
            println!("  📚 Generation: offline (retrieval only)");
        }
    }

    println!();

    // Feature flag status
    #[cfg(feature = "local-llm")]
    println!("  local-llm feature: enabled ✓");
    #[cfg(not(feature = "local-llm"))]
    println!("  local-llm feature: disabled ✗");

    #[cfg(feature = "cuda")]
    println!("  CUDA acceleration: enabled ✓");
    #[cfg(feature = "metal")]
    println!("  Metal acceleration: enabled ✓");

    println!();

    if !model_path.exists() {
        println!("To enable local generation without Ollama:");
        println!("  sage model download phi3-mini");
        println!();
    }
}

/// Run a dream cycle — find activated cells, run curiosity probes, log results.
fn run_dream(steps: usize) {
    use sage::distributed_knowledge::{default_brain_path, KnowledgeStore, NCAKnowledge};
    use sage::grid::KNOWLEDGE_ACTIVATION;
    use std::collections::HashMap;
    use std::io::Write;

    let brain_path = default_brain_path();
    let mut knowledge = NCAKnowledge::new();

    // Load brain
    if std::path::Path::new(&brain_path).exists() {
        if let Err(e) = knowledge.load(&brain_path) {
            eprintln!("Failed to load brain: {}", e);
            std::process::exit(1);
        }
    } else {
        eprintln!("No brain found at {}. Run `sage chat` first to build knowledge.", brain_path);
        std::process::exit(1);
    }

    println!("💭 Starting dream cycle ({} steps)...", steps);
    println!();

    // Step 1: Find top-20 most activated cells
    let mut activations: Vec<((usize, usize), f64)> = Vec::new();
    for y in 0..knowledge.grid.height {
        for x in 0..knowledge.grid.width {
            let act = knowledge.grid.cells[y][x][KNOWLEDGE_ACTIVATION];
            if act > 0.01 {
                activations.push(((x, y), act));
            }
        }
    }
    activations.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    let top_cells: Vec<_> = activations.iter().take(20).collect();

    if top_cells.is_empty() {
        println!("No active knowledge cells found. Dream cycle skipped.");
        return;
    }

    println!("🔍 Found {} active cells, examining top {}...", activations.len(), top_cells.len());

    // Step 2: Run curiosity probes on each top cell
    // For each top cell, look at nearby cells and see what related concepts exist
    let mut topics: Vec<String> = Vec::new();
    let mut topic_counts: HashMap<String, usize> = HashMap::new();

    for ((x, y), activation) in &top_cells {
        // Get text at this position
        if let Some(text) = knowledge.text_store.peek(*x, *y) {
            // Extract key topic (first ~40 chars or first sentence)
            let topic = extract_topic(text);
            *topic_counts.entry(topic.clone()).or_insert(0) += 1;
            if !topics.contains(&topic) && topics.len() < 10 {
                topics.push(topic);
            }
        }

        // Curiosity probe: check nearby cells (within radius 5)
        let radius = 5i32;
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let nx = (*x as i32 + dx).clamp(0, (knowledge.grid.width - 1) as i32) as usize;
                let ny = (*y as i32 + dy).clamp(0, (knowledge.grid.height - 1) as i32) as usize;
                let nearby_act = knowledge.grid.cells[ny][nx][KNOWLEDGE_ACTIVATION];

                // If nearby cell is also activated, it's related knowledge
                if nearby_act > *activation * 0.3 {
                    if let Some(nearby_text) = knowledge.text_store.peek(nx, ny) {
                        let nearby_topic = extract_topic(nearby_text);
                        *topic_counts.entry(nearby_topic.clone()).or_insert(0) += 1;
                    }
                }
            }
        }
    }

    // Step 3: Run NCA freerun repair steps to let knowledge consolidate
    for ((x, y), _) in top_cells.iter().take(5) {
        knowledge.freerun_repair((*x, *y), steps);
    }

    // Save the brain after dream consolidation
    if let Err(e) = knowledge.save(&brain_path) {
        eprintln!("Warning: failed to save brain after dream: {}", e);
    }

    // Step 4: Print dream summary
    let top_topics: Vec<_> = {
        let mut sorted: Vec<_> = topic_counts.into_iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(&a.1));
        sorted.into_iter().take(5).map(|(t, _)| t).collect()
    };

    if top_topics.is_empty() {
        println!("💭 Dream completed, but no topics identified.");
    } else {
        println!("💭 Dreamed about: {}", top_topics.join(", "));
    }

    // Step 5: Log to dreams.log
    let dreams_log_path = sage_home().join("dreams.log");
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&dreams_log_path)
    {
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
        let line = format!("[{}] Dreamed about: {}\n", timestamp, top_topics.join(", "));
        let _ = file.write_all(line.as_bytes());
    }

    println!();
    println!("Dream log saved to: {}", dreams_log_path.display());
}

/// Extract a short topic from text (first ~50 chars or first sentence).
fn extract_topic(text: &str) -> String {
    let text = text.trim();
    // Try to get first sentence
    if let Some(idx) = text.find(|c| c == '.' || c == '!' || c == '?') {
        if idx < 60 {
            return text[..=idx].to_string();
        }
    }
    // Otherwise, take first 50 chars
    if text.len() > 50 {
        format!("{}...", &text[..47])
    } else {
        text.to_string()
    }
}

/// Show last 10 dream summaries from dreams.log
fn run_dream_show() {
    let dreams_log_path = sage_home().join("dreams.log");

    if !dreams_log_path.exists() {
        println!("No dream history found. Run `sage dream` to start dreaming.");
        return;
    }

    match std::fs::read_to_string(&dreams_log_path) {
        Ok(content) => {
            let lines: Vec<_> = content.lines().collect();
            let recent: Vec<_> = lines.iter().rev().take(10).collect();

            if recent.is_empty() {
                println!("Dream log is empty.");
                return;
            }

            println!("💭 Recent Dreams");
            println!("─────────────────");
            for line in recent.iter().rev() {
                println!("{}", line);
            }
        }
        Err(e) => {
            eprintln!("Failed to read dream log: {}", e);
        }
    }
}

/// Show knowledge statistics and insights
fn run_insights() {
    use sage::distributed_knowledge::{default_brain_path, KnowledgeStore, NCAKnowledge};
    use sage::grid::KNOWLEDGE_ACTIVATION;
    use std::collections::HashMap;

    let brain_path = default_brain_path();
    let mut knowledge = NCAKnowledge::new();

    // Load brain
    if std::path::Path::new(&brain_path).exists() {
        if let Err(e) = knowledge.load(&brain_path) {
            eprintln!("Failed to load brain: {}", e);
            std::process::exit(1);
        }
    } else {
        eprintln!("No brain found. Run `sage chat` first to build knowledge.");
        std::process::exit(1);
    }

    println!("📊 SAGE Knowledge Insights");
    println!("══════════════════════════");
    println!();

    // 1. Basic stats
    let active_cells = knowledge.active_knowledge(0.01).len();
    let total_texts = knowledge.text_store.len();
    let text_bytes = knowledge.text_store.total_bytes();

    println!("📈 Overview");
    println!("   Active cells:    {}", active_cells);
    println!("   Stored texts:    {}", total_texts);
    println!("   Text memory:     {:.1} KB", text_bytes as f64 / 1024.0);
    println!();

    // 2. Topic clusters - find top activated cells and cluster by text similarity
    let mut activations: Vec<((usize, usize), f64, Option<String>)> = Vec::new();
    for y in 0..knowledge.grid.height {
        for x in 0..knowledge.grid.width {
            let act = knowledge.grid.cells[y][x][KNOWLEDGE_ACTIVATION];
            if act > 0.1 {
                let text = knowledge.text_store.peek(x, y).map(|s| s.to_string());
                activations.push(((x, y), act, text));
            }
        }
    }
    activations.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    // Simple clustering: extract first word of each text as category
    let mut word_counts: HashMap<String, usize> = HashMap::new();
    for (_, _, text_opt) in &activations {
        if let Some(text) = text_opt {
            // Get first significant word (skip common words)
            let words: Vec<_> = text.split_whitespace().collect();
            for word in words.iter().take(3) {
                let word_lower = word.to_lowercase().trim_matches(|c: char| !c.is_alphanumeric()).to_string();
                if word_lower.len() > 3 && !is_stop_word(&word_lower) {
                    *word_counts.entry(word_lower).or_insert(0) += 1;
                    break;
                }
            }
        }
    }

    let mut top_words: Vec<_> = word_counts.into_iter().collect();
    top_words.sort_by(|a, b| b.1.cmp(&a.1));

    println!("🏷️  Top Topics (by keyword frequency)");
    let max_count = top_words.first().map(|(_, c)| *c).unwrap_or(1);
    for (word, count) in top_words.iter().take(10) {
        let bar_len = (count * 20 / max_count.max(1)).max(1);
        let bar: String = "█".repeat(bar_len);
        println!("   {:12} {:3} {}", word, count, bar);
    }
    println!();

    // 3. Knowledge growth over time - read stats.json
    let stats_path = sage_home().join("stats.json");
    if stats_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&stats_path) {
            if let Ok(stats) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(daily) = stats.get("daily_facts").and_then(|d| d.as_object()) {
                    println!("📅 Knowledge Growth (facts encoded per day)");
                    let mut days: Vec<_> = daily.iter().collect();
                    days.sort_by(|a, b| a.0.cmp(b.0));
                    let recent: Vec<_> = days.iter().rev().take(7).collect();
                    let max_facts = recent.iter()
                        .map(|(_, v)| v.as_u64().unwrap_or(0))
                        .max()
                        .unwrap_or(1) as usize;

                    for (date, count) in recent.iter().rev() {
                        let c = count.as_u64().unwrap_or(0) as usize;
                        let bar_len = (c * 20 / max_facts.max(1)).max(1);
                        let bar: String = "█".repeat(bar_len);
                        println!("   {} {:4} {}", date, c, bar);
                    }
                    println!();
                }
            }
        }
    }

    // 4. Retrieval quality stats
    let stats = knowledge.stats_handle();
    let total_queries = stats.total_queries.load(std::sync::atomic::Ordering::Relaxed);
    let hits = stats.hits.load(std::sync::atomic::Ordering::Relaxed);
    let misses = stats.misses.load(std::sync::atomic::Ordering::Relaxed);

    if total_queries > 0 {
        println!("🎯 Retrieval Quality");
        println!("   Total queries:   {}", total_queries);
        println!("   Hit rate:        {:.1}%", stats.hit_rate() * 100.0);
        println!("   Hits / Misses:   {} / {}", hits, misses);
        println!("   Mean relevance:  {:.3}", stats.mean_top_relevance());
        println!();

        // Relevance distribution bar chart
        let low = stats.relevance_low.load(std::sync::atomic::Ordering::Relaxed);
        let mid = stats.relevance_mid.load(std::sync::atomic::Ordering::Relaxed);
        let good = stats.relevance_good.load(std::sync::atomic::Ordering::Relaxed);
        let exc = stats.relevance_excellent.load(std::sync::atomic::Ordering::Relaxed);
        let max_rel = [low, mid, good, exc].into_iter().max().unwrap_or(1) as usize;

        println!("   Relevance Distribution:");
        for (label, count) in [("  low (0-0.3)", low), ("  mid (0.3-0.6)", mid), ("  good (0.6-0.8)", good), ("  exc (0.8+)", exc)] {
            let c = count as usize;
            let bar_len = (c * 15 / max_rel.max(1)).max(if c > 0 { 1 } else { 0 });
            let bar: String = "█".repeat(bar_len);
            println!("   {:14} {:4} {}", label, count, bar);
        }
    } else {
        println!("🎯 Retrieval Quality");
        println!("   No retrieval data yet. Chat with SAGE to build statistics.");
    }
}

/// Check if a word is a common stop word
fn is_stop_word(word: &str) -> bool {
    const STOP_WORDS: &[&str] = &[
        "the", "and", "for", "are", "but", "not", "you", "all", "can", "had",
        "her", "was", "one", "our", "out", "has", "have", "been", "were", "this",
        "that", "with", "they", "from", "what", "there", "their", "will", "when",
        "your", "user", "assistant", "echo", "test",
    ];
    STOP_WORDS.contains(&word)
}

// ═══════════════════════════════════════════════════════════════════════════
// v0.3.0 commands: status, export, import, search
// ═══════════════════════════════════════════════════════════════════════════

/// Show SAGE system status dashboard
fn run_status() {
    use sage::distributed_knowledge::encoder::{detect_embedding_status, EmbeddingStatus, EncoderConfig};
    use sage::inference::{detect_generation_backend, GenerationBackend};
    use std::sync::atomic::Ordering;

    let home = sage_home();
    let brain_path = default_brain_path();
    let text_store_path = brain_path.replace("brain.bin", "text_store.bin");
    let pid_file = home.join("node.pid");
    let old_pid_file = home.join("sage.pid");

    println!();
    println!("╔══════════════════════════════════════════════════════════════════╗");
    println!("║                      SAGE Status Dashboard                        ║");
    println!("╚══════════════════════════════════════════════════════════════════╝");
    println!();

    // Version
    println!("  Version:          v{}", VERSION);
    println!();

    // Embedding backend
    let embed_status = detect_embedding_status(&EncoderConfig::default());
    let embed_str = match &embed_status {
        EmbeddingStatus::Fastembed { model, dim } => format!("FastEmbed bundled ({}, {}-dim) ✓", model, dim),
        EmbeddingStatus::Ollama { model } => format!("Ollama ({}) ✓", model),
        EmbeddingStatus::HashFallback => "Hash fallback (reduced quality) ⚠".to_string(),
    };
    println!("  Embedding:        {}", embed_str);

    // Generation backend
    let gen_backend = detect_generation_backend();
    let gen_str = match &gen_backend {
        GenerationBackend::Local { model_name, model_size } => format!("local llama.cpp ({}, {}) ⚡", model_name, model_size),
        GenerationBackend::Ollama { model } => format!("Ollama ({}) 🔗", model),
        GenerationBackend::Embedded { model_name } => format!("embedded ({}) 🧠", model_name),
        GenerationBackend::Offline => "offline (retrieval only) 📚".to_string(),
    };
    println!("  Generation:       {}", gen_str);
    println!();

    // Load brain for stats
    let mut knowledge = NCAKnowledge::new();
    let brain_loaded = if std::path::Path::new(&brain_path).exists() {
        knowledge.load(&brain_path).is_ok()
    } else {
        false
    };

    if brain_loaded {
        // Grid utilization
        let total_cells = knowledge.grid.width * knowledge.grid.height;
        let active = knowledge.active_knowledge(0.01);
        let active_count = active.len();
        let utilization = (active_count as f64 / total_cells as f64) * 100.0;
        println!("  Grid size:        {}×{} ({} channels)",
            knowledge.grid.width, knowledge.grid.height, knowledge.grid.cells[0][0].len());
        println!("  Grid utilization: {:.2}% ({} / {} cells active)",
            utilization, active_count, total_cells);

        // Knowledge stats
        let text_count = knowledge.text_store.len();
        let unique_positions: std::collections::HashSet<_> = active.iter()
            .map(|a| a.position)
            .collect();
        println!("  Facts encoded:    {}", text_count);
        println!("  Unique positions: {}", unique_positions.len());

        // Retrieval stats
        let stats = knowledge.stats_handle();
        let total_queries = stats.total_queries.load(Ordering::Relaxed);
        if total_queries > 0 {
            let hit_rate = stats.hit_rate() * 100.0;
            let mean_rel = stats.mean_top_relevance();
            println!("  Queries (session): {} (hit rate: {:.1}%, mean relevance: {:.3})",
                total_queries, hit_rate, mean_rel);
        }
        println!();

        // Brain file info
        let brain_size = std::fs::metadata(&brain_path)
            .map(|m| format_size(m.len()))
            .unwrap_or_else(|_| "unknown".to_string());
        let brain_modified = std::fs::metadata(&brain_path)
            .ok()
            .and_then(|m| m.modified().ok())
            .map(|t| format_time(t))
            .unwrap_or_else(|| "unknown".to_string());
        println!("  Brain file:       {}", brain_path);
        println!("  Brain size:       {}", brain_size);
        println!("  Last saved:       {}", brain_modified);

        // Text store info
        if std::path::Path::new(&text_store_path).exists() {
            let ts_size = std::fs::metadata(&text_store_path)
                .map(|m| format_size(m.len()))
                .unwrap_or_else(|_| "unknown".to_string());
            println!("  Text store:       {} ({})", text_store_path, ts_size);
        }
    } else {
        println!("  Brain:            Not loaded (no brain.bin found)");
        println!("  Brain path:       {}", brain_path);
    }
    println!();

    // Network status - check both pid files
    let actual_pid_file = if pid_file.exists() {
        Some(&pid_file)
    } else if old_pid_file.exists() {
        Some(&old_pid_file)
    } else {
        None
    };

    let node_running = if let Some(pf) = actual_pid_file {
        if let Ok(pid_str) = std::fs::read_to_string(pf) {
            if let Ok(pid) = pid_str.trim().parse::<u32>() {
                #[cfg(unix)]
                {
                    std::process::Command::new("kill")
                        .args(["-0", &pid.to_string()])
                        .stderr(std::process::Stdio::null())
                        .status()
                        .map(|s| s.success())
                        .unwrap_or(false)
                }
                #[cfg(not(unix))]
                { true }
            } else { false }
        } else { false }
    } else { false };

    if node_running {
        println!("  Network node:     Running ✓");
        println!("  Peers:            (check `sage node status` for details)");
    } else {
        println!("  Network node:     Not running");
        println!("                    Run `sage node start` to join the mesh");
    }
    println!();
}

/// Export knowledge state to a .sage file
fn run_export(file: &str) {
    let brain_path = default_brain_path();
    let mut knowledge = NCAKnowledge::new();

    // Load existing brain
    if !std::path::Path::new(&brain_path).exists() {
        eprintln!("No brain found at {}. Nothing to export.", brain_path);
        std::process::exit(1);
    }

    if let Err(e) = knowledge.load(&brain_path) {
        eprintln!("Failed to load brain: {}", e);
        std::process::exit(1);
    }

    // Create export header with SAGE export magic
    let export_header = SageExportHeader {
        magic: *b"SGEX",  // SAGE EXport
        version: 1,
        sage_version: VERSION.to_string(),
        grid_size: knowledge.grid.width as u32,
        channels: knowledge.grid.cells[0][0].len() as u32,
        text_entries: knowledge.text_store.len() as u32,
        created_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0),
    };

    // Serialize header
    let header_data = match bincode::serialize(&export_header) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Failed to serialize export header: {}", e);
            std::process::exit(1);
        }
    };

    // Serialize grid
    let grid_data = match bincode::serialize(&knowledge.grid) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Failed to serialize grid: {}", e);
            std::process::exit(1);
        }
    };

    // Serialize text store
    let text_data = match bincode::serialize(&knowledge.text_store) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Failed to serialize text store: {}", e);
            std::process::exit(1);
        }
    };

    // Write to file: header_len (u32) + header + grid_len (u32) + grid + text_len (u32) + text
    let mut output = Vec::new();
    output.extend_from_slice(&(header_data.len() as u32).to_le_bytes());
    output.extend_from_slice(&header_data);
    output.extend_from_slice(&(grid_data.len() as u32).to_le_bytes());
    output.extend_from_slice(&grid_data);
    output.extend_from_slice(&(text_data.len() as u32).to_le_bytes());
    output.extend_from_slice(&text_data);

    // Ensure file has .sage extension
    let file_path = if file.ends_with(".sage") {
        file.to_string()
    } else {
        format!("{}.sage", file)
    };

    if let Err(e) = std::fs::write(&file_path, &output) {
        eprintln!("Failed to write export file: {}", e);
        std::process::exit(1);
    }

    let active_count = knowledge.active_knowledge(0.01).len();
    println!("✅ Exported to: {}", file_path);
    println!("   Grid: {}×{}", knowledge.grid.width, knowledge.grid.height);
    println!("   Active cells: {}", active_count);
    println!("   Text entries: {}", knowledge.text_store.len());
    println!("   File size: {}", format_size(output.len() as u64));
}

/// Import knowledge from a .sage file
fn run_import(file: &str, weight: f64) {
    let brain_path = default_brain_path();

    // Read export file
    let data = match std::fs::read(file) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Failed to read file: {}", e);
            std::process::exit(1);
        }
    };

    // Parse header
    if data.len() < 4 {
        eprintln!("Invalid .sage file: too short");
        std::process::exit(1);
    }

    let header_len = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
    if data.len() < 4 + header_len {
        eprintln!("Invalid .sage file: header truncated");
        std::process::exit(1);
    }

    let header: SageExportHeader = match bincode::deserialize(&data[4..4+header_len]) {
        Ok(h) => h,
        Err(e) => {
            eprintln!("Failed to parse export header: {}", e);
            std::process::exit(1);
        }
    };

    // Verify magic
    if &header.magic != b"SGEX" {
        eprintln!("Invalid .sage file: bad magic bytes");
        std::process::exit(1);
    }

    println!("📦 Importing from SAGE export v{} (created by SAGE {})", header.version, header.sage_version);
    println!("   Grid: {}×{}, {} channels", header.grid_size, header.grid_size, header.channels);
    println!("   Text entries: {}", header.text_entries);

    // Parse grid
    let grid_offset = 4 + header_len;
    if data.len() < grid_offset + 4 {
        eprintln!("Invalid .sage file: grid length missing");
        std::process::exit(1);
    }
    let grid_len = u32::from_le_bytes([
        data[grid_offset], data[grid_offset+1], data[grid_offset+2], data[grid_offset+3]
    ]) as usize;

    let grid_data_offset = grid_offset + 4;
    if data.len() < grid_data_offset + grid_len {
        eprintln!("Invalid .sage file: grid data truncated");
        std::process::exit(1);
    }

    let import_grid: sage::grid::Grid = match bincode::deserialize(&data[grid_data_offset..grid_data_offset+grid_len]) {
        Ok(g) => g,
        Err(e) => {
            eprintln!("Failed to parse grid: {}", e);
            std::process::exit(1);
        }
    };

    // Parse text store
    let text_offset = grid_data_offset + grid_len;
    if data.len() < text_offset + 4 {
        eprintln!("Invalid .sage file: text store length missing");
        std::process::exit(1);
    }
    let text_len = u32::from_le_bytes([
        data[text_offset], data[text_offset+1], data[text_offset+2], data[text_offset+3]
    ]) as usize;

    let text_data_offset = text_offset + 4;
    if data.len() < text_data_offset + text_len {
        eprintln!("Invalid .sage file: text store truncated");
        std::process::exit(1);
    }

    let _import_text: sage::distributed_knowledge::TextStore = match bincode::deserialize(&data[text_data_offset..text_data_offset+text_len]) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Failed to parse text store: {}", e);
            std::process::exit(1);
        }
    };

    // Load existing brain (or create new)
    let mut knowledge = NCAKnowledge::new();
    let had_existing = if std::path::Path::new(&brain_path).exists() {
        knowledge.load(&brain_path).is_ok()
    } else {
        false
    };

    let before_active = knowledge.active_knowledge(0.01).len();

    // Merge grid with weighted average
    println!("   Merging with weight {:.2}...", weight);
    knowledge.merge(&import_grid, weight);

    let after_active = knowledge.active_knowledge(0.01).len();

    // Save merged brain
    if let Err(e) = knowledge.save(&brain_path) {
        eprintln!("Failed to save merged brain: {}", e);
        std::process::exit(1);
    }

    println!("✅ Import complete!");
    if had_existing {
        println!("   Before: {} active cells", before_active);
    }
    println!("   After:  {} active cells", after_active);
    println!("   Saved to: {}", brain_path);
}

/// Search knowledge base non-interactively
fn run_search(query: &str, max_results: usize, json_output: bool) {
    let brain_path = default_brain_path();
    let mut knowledge = NCAKnowledge::new();

    if !std::path::Path::new(&brain_path).exists() {
        if json_output {
            println!("{{\"error\": \"No brain found\", \"results\": []}}");
        } else {
            eprintln!("No brain found at {}. Run `sage chat` first to learn some facts.", brain_path);
        }
        std::process::exit(1);
    }

    if let Err(e) = knowledge.load(&brain_path) {
        if json_output {
            println!("{{\"error\": \"Failed to load brain: {}\", \"results\": []}}", e);
        } else {
            eprintln!("Failed to load brain: {}", e);
        }
        std::process::exit(1);
    }

    // Run query
    let results = knowledge.query(query, max_results);

    if json_output {
        // JSON output
        let json_results: Vec<serde_json::Value> = results.iter()
            .filter(|r| r.text.is_some())
            .map(|r| {
                serde_json::json!({
                    "text": r.text.as_ref().unwrap(),
                    "relevance": r.relevance,
                    "position": [r.position.0, r.position.1],
                    "activation": r.activation,
                    "confidence": r.confidence
                })
            })
            .collect();

        let output = serde_json::json!({
            "query": query,
            "results": json_results,
            "total": json_results.len()
        });

        println!("{}", serde_json::to_string_pretty(&output).unwrap_or_else(|_| "{}".to_string()));
    } else {
        // Human-readable output
        let relevant: Vec<_> = results.iter()
            .filter(|r| r.text.is_some())
            .collect();

        if relevant.is_empty() {
            println!("No results found for: {}", query);
            std::process::exit(1);
        }

        println!("Query: {}", query);
        println!("Found {} result(s):\n", relevant.len());

        for (i, r) in relevant.iter().enumerate() {
            let text = r.text.as_ref().unwrap();
            println!("{}. [relevance: {:.3}] {}", i + 1, r.relevance, text);
        }
    }

    // Exit 0 if results found, 1 if not
    let has_results = results.iter().any(|r| r.text.is_some());
    if !has_results {
        std::process::exit(1);
    }
}

// ─── Helper functions ────────────────────────────────────────────────────────

/// Format file size in human-readable form
fn format_size(bytes: u64) -> String {
    if bytes >= 1_073_741_824 {
        format!("{:.2} GB", bytes as f64 / 1_073_741_824.0)
    } else if bytes >= 1_048_576 {
        format!("{:.2} MB", bytes as f64 / 1_048_576.0)
    } else if bytes >= 1024 {
        format!("{:.2} KB", bytes as f64 / 1024.0)
    } else {
        format!("{} bytes", bytes)
    }
}

/// Format system time as human-readable string
fn format_time(time: std::time::SystemTime) -> String {
    match time.duration_since(std::time::UNIX_EPOCH) {
        Ok(duration) => {
            let secs = duration.as_secs();
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            let age_secs = now.saturating_sub(secs);
            if age_secs < 60 {
                "just now".to_string()
            } else if age_secs < 3600 {
                format!("{} min ago", age_secs / 60)
            } else if age_secs < 86400 {
                format!("{} hours ago", age_secs / 3600)
            } else {
                format!("{} days ago", age_secs / 86400)
            }
        }
        Err(_) => "unknown".to_string(),
    }
}

/// SAGE export file header
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
struct SageExportHeader {
    /// Magic bytes "SGEX" (SAGE EXport)
    magic: [u8; 4],
    /// Export format version
    version: u32,
    /// SAGE version that created this export
    sage_version: String,
    /// Grid dimensions
    grid_size: u32,
    /// Number of channels per cell
    channels: u32,
    /// Number of text entries
    text_entries: u32,
    /// Unix timestamp of export
    created_at: u64,
}

/// SAGE diff export file header (for --share)
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
struct SageDiffHeader {
    /// Magic bytes "SGDF" (SAGE DiFF)
    magic: [u8; 4],
    /// Diff format version
    version: u32,
    /// SAGE version that created this diff
    sage_version: String,
    /// Source node ID
    source_node: String,
    /// Number of changed cells
    num_changes: u32,
    /// Unix timestamp of diff creation
    created_at: u64,
}

/// Export only the diff since last export (for "sneakernet" sharing).
/// Diffs are small: only changed cells, compressed.
fn run_export_share(file: &str) {
    use sage::network::diff::KnowledgeDiff;

    let brain_path = default_brain_path();
    let home = sage_home();
    let last_export_path = home.join("last_export.bin");

    let mut knowledge = NCAKnowledge::new();

    // Load existing brain
    if !std::path::Path::new(&brain_path).exists() {
        eprintln!("No brain found at {}. Nothing to export.", brain_path);
        std::process::exit(1);
    }

    if let Err(e) = knowledge.load(&brain_path) {
        eprintln!("Failed to load brain: {}", e);
        std::process::exit(1);
    }

    // Load last export state (if exists)
    let last_state: Vec<Vec<Vec<f64>>> = if last_export_path.exists() {
        match std::fs::read(&last_export_path) {
            Ok(data) => bincode::deserialize(&data).unwrap_or_else(|_| {
                // Create empty grid with same dimensions
                vec![vec![vec![0.0; knowledge.grid.cells[0][0].len()]; knowledge.grid.width]; knowledge.grid.height]
            }),
            Err(_) => vec![vec![vec![0.0; knowledge.grid.cells[0][0].len()]; knowledge.grid.width]; knowledge.grid.height],
        }
    } else {
        // No previous export — all cells are "new"
        vec![vec![vec![0.0; knowledge.grid.cells[0][0].len()]; knowledge.grid.width]; knowledge.grid.height]
    };

    // Compute diff
    let node_id = {
        use sage::network::identity::NodeIdentity;
        let key_path = home.join("identity.key");
        if key_path.exists() {
            NodeIdentity::load(&key_path)
                .map(|id| id.node_id)
                .unwrap_or_else(|_| "unknown".to_string())
        } else {
            "unknown".to_string()
        }
    };

    let diff = KnowledgeDiff::compute(
        &last_state,
        &knowledge.grid.cells,
        node_id.clone(),
        0, // sequence
        0.9, // confidence
        0.01, // threshold
    );

    if diff.changes.is_empty() {
        println!("No changes since last export. Nothing to share.");
        return;
    }

    // Create diff header
    let header = SageDiffHeader {
        magic: *b"SGDF",
        version: 1,
        sage_version: VERSION.to_string(),
        source_node: node_id,
        num_changes: diff.changes.len() as u32,
        created_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0),
    };

    // Serialize header and diff
    let header_data = match bincode::serialize(&header) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Failed to serialize diff header: {}", e);
            std::process::exit(1);
        }
    };

    let diff_data = diff.to_bytes();

    // Combine: header_len (u32) + header + diff_len (u32) + diff
    let mut output = Vec::new();
    output.extend_from_slice(&(header_data.len() as u32).to_le_bytes());
    output.extend_from_slice(&header_data);
    output.extend_from_slice(&(diff_data.len() as u32).to_le_bytes());
    output.extend_from_slice(&diff_data);

    // Ensure file has .sage-diff extension
    let file_path = if file.ends_with(".sage-diff") {
        file.to_string()
    } else {
        format!("{}.sage-diff", file)
    };

    if let Err(e) = std::fs::write(&file_path, &output) {
        eprintln!("Failed to write diff file: {}", e);
        std::process::exit(1);
    }

    // Save current state as last export
    if let Ok(state_data) = bincode::serialize(&knowledge.grid.cells) {
        let _ = std::fs::write(&last_export_path, &state_data);
    }

    println!("✅ Exported diff to: {}", file_path);
    println!("   Changed cells: {}", diff.changes.len());
    println!("   File size: {}", format_size(output.len() as u64));
    println!();
    println!("Share this file via Discord, USB, or email.");
    println!("Recipients can import with: sage import --merge {}", file_path);
}

/// Import a diff file (from --share export) and merge it.
fn run_import_merge(file: &str, weight: f64) {
    use sage::network::diff::KnowledgeDiff;

    let brain_path = default_brain_path();

    // Read diff file
    let data = match std::fs::read(file) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Failed to read file: {}", e);
            std::process::exit(1);
        }
    };

    // Parse header
    if data.len() < 4 {
        eprintln!("Invalid .sage-diff file: too short");
        std::process::exit(1);
    }

    let header_len = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
    if data.len() < 4 + header_len {
        eprintln!("Invalid .sage-diff file: header truncated");
        std::process::exit(1);
    }

    let header: SageDiffHeader = match bincode::deserialize(&data[4..4+header_len]) {
        Ok(h) => h,
        Err(e) => {
            eprintln!("Failed to parse diff header: {}", e);
            std::process::exit(1);
        }
    };

    // Verify magic
    if &header.magic != b"SGDF" {
        eprintln!("Invalid .sage-diff file: bad magic bytes (expected SGDF)");
        eprintln!("Hint: This might be a full export. Try: sage import {}", file);
        std::process::exit(1);
    }

    println!("📦 Importing diff from {} (SAGE {})", header.source_node, header.sage_version);
    println!("   Changed cells: {}", header.num_changes);

    // Parse diff
    let diff_offset = 4 + header_len;
    if data.len() < diff_offset + 4 {
        eprintln!("Invalid .sage-diff file: diff length missing");
        std::process::exit(1);
    }
    let diff_len = u32::from_le_bytes([
        data[diff_offset], data[diff_offset+1], data[diff_offset+2], data[diff_offset+3]
    ]) as usize;

    let diff_data_offset = diff_offset + 4;
    if data.len() < diff_data_offset + diff_len {
        eprintln!("Invalid .sage-diff file: diff data truncated");
        std::process::exit(1);
    }

    let diff: KnowledgeDiff = match KnowledgeDiff::from_bytes(&data[diff_data_offset..diff_data_offset+diff_len]) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Failed to parse diff: {}", e);
            std::process::exit(1);
        }
    };

    // Load existing brain (or create new)
    let mut knowledge = NCAKnowledge::new();
    let had_existing = if std::path::Path::new(&brain_path).exists() {
        knowledge.load(&brain_path).is_ok()
    } else {
        false
    };

    let before_active = knowledge.active_knowledge(0.01).len();

    // Apply diff with weighted merge
    println!("   Merging with weight {:.2}...", weight);
    diff.apply_weighted(&mut knowledge.grid.cells, weight);

    let after_active = knowledge.active_knowledge(0.01).len();

    // Save merged brain
    if let Err(e) = knowledge.save(&brain_path) {
        eprintln!("Failed to save merged brain: {}", e);
        std::process::exit(1);
    }

    println!("✅ Merge complete!");
    if had_existing {
        println!("   Before: {} active cells", before_active);
    }
    println!("   After:  {} active cells", after_active);
    println!("   Saved to: {}", brain_path);
}
