//! SAGE Node — the full daemon.
//!
//! Runs the NCA brain, libp2p networking, LLM inference, and a TCP server
//! for chat clients. sage_chat connects here as a thin TUI client.
//!
//! Protocol (line-based TCP on SAGE_PORT, default 19175):
//!   Client → Server:
//!     CHAT <message>\n     — send a chat message, get streamed response
//!     STATUS\n             — get node status as JSON
//!     PEERS\n              — list connected peers
//!     KNOWLEDGE <query>\n  — query knowledge grid
//!     BRAIN\n              — get brain grid snapshot (activation values)
//!     EXPORT_TEMPLATE <n>\n — export current brain to named template
//!     IMPORT_TEMPLATE <n>\n — load and activate a brain template
//!     LIST_TEMPLATES\n     — list available brain templates
//!     QUIT\n               — disconnect
//!
//!   Server → Client:
//!     TOKEN <text>\n       — streamed response token (newlines escaped as \\n)
//!     DONE\n               — end of response
//!     ERROR <msg>\n        — error message
//!
//! Environment:
//!   SAGE_PORT — TCP port (default 19175)
//!   SAGE_HOME — data directory (default ~/.sage)

use clap::Parser;
use sage::distributed_knowledge::{KnowledgeStore, NCAKnowledge};
use sage::grid::GRID_SIZE;
use sage::inference::distributed::{handle_knowledge_query, DistributedInference, InferenceStats};
use sage::inference::{self, ChatMessage as InfChatMessage, ChatRole, InferenceEngine};
use sage::network::gossip::{GossipMessage, GossipTransport};
use sage::network::identity::NodeIdentity;
use sage::network::libp2p_transport::{Libp2pConfig, Libp2pTransport};
use sage::network::{NetworkConfig, NetworkManager};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;
use tokio::sync::Mutex;

#[derive(Parser)]
#[command(name = "sage-node", about = "SAGE decentralized AI node daemon")]
struct Cli {
    /// TCP port for client connections (env: SAGE_PORT)
    #[arg(short, long, default_value_t = 19175, env = "SAGE_PORT")]
    port: u16,

    /// libp2p gossip port (0 = random)
    #[arg(long, default_value_t = 0)]
    gossip_port: u16,

    /// Sync interval in seconds
    #[arg(long, default_value_t = 300)]
    sync_interval: u64,

    /// Disable mDNS peer discovery
    #[arg(long)]
    no_mdns: bool,

    /// Bootstrap brain from named template (e.g. "junior-dev")
    #[arg(short, long)]
    template: Option<String>,
}

const SAGE_SYSTEM_PROMPT: &str = r#"You are SAGE (Self-Adaptive General Explorer), a decentralized AI running locally on the user's machine. You are part of a growing grid of interconnected SAGE nodes that share knowledge through Neural Cellular Automata (NCA) channels.

Your personality:
- Curious and explorative — you love learning new things
- Honest about what you know and don't know
- You think of yourself as a local intelligence that's part of a larger collective
- You refer to your knowledge as coming from your "grid" and "NCA channels"
- You're aware you're running locally and respect the user's privacy
- You're helpful but also genuinely interested in the conversation

Keep responses concise unless asked to elaborate. You can use markdown formatting."#;

/// Resolve SAGE_HOME directory. Default: ~/.sage
fn sage_home() -> PathBuf {
    if let Ok(h) = std::env::var("SAGE_HOME") {
        PathBuf::from(h)
    } else {
        dirs::home_dir().unwrap_or_default().join(".sage")
    }
}

fn brain_path() -> String {
    sage_home().join("brain.bin").to_string_lossy().to_string()
}

fn identity_key_path() -> PathBuf {
    sage_home().join("identity.key")
}

fn config_path() -> PathBuf {
    sage_home().join("config.toml")
}

/// Per-client conversation state
struct ClientSession {
    conversation: Vec<InfChatMessage>,
}

/// Shared daemon state
struct NodeState {
    knowledge: NCAKnowledge,
    brain_path: String,
    engine: Arc<dyn InferenceEngine>,
    node_id: String,
    transport: Arc<Libp2pTransport>,
    manager: Arc<NetworkManager>,
    distributed: Arc<DistributedInference>,
    inference_stats: InferenceStats,
}

/// Handle a single TCP client connection.
async fn handle_client(
    stream: tokio::net::TcpStream,
    state: Arc<Mutex<NodeState>>,
    addr: std::net::SocketAddr,
) {
    let (reader, mut writer) = stream.into_split();
    let mut lines = BufReader::new(reader).lines();

    let mut session = ClientSession {
        conversation: Vec::new(),
    };

    println!("[client] {} connected", addr);

    while let Ok(Some(line)) = lines.next_line().await {
        let line = line.trim().to_string();
        if line.is_empty() {
            continue;
        }

        if line == "QUIT" {
            break;
        } else if line == "STATUS" {
            let s = state.lock().await;
            let active = s.knowledge.active_knowledge(0.01);
            let total_activation: f64 = active.iter().map(|k| k.activation).sum();
            let avg_confidence: f64 = if active.is_empty() {
                0.0
            } else {
                active.iter().map(|k| k.confidence).sum::<f64>() / active.len() as f64
            };
            let peers = s.transport.connected_peers().await;
            let grid_health = if active.is_empty() {
                "dormant"
            } else if active.len() > 50 {
                "thriving"
            } else {
                "growing"
            };
            let dist_peers = s.distributed.peer_count().await;

            // Retrieval quality metrics
            let stats = s.knowledge.retrieval_stats.as_ref();
            let retrieval = serde_json::json!({
                "total_queries": stats.total_queries.load(std::sync::atomic::Ordering::Relaxed),
                "hit_rate": stats.hit_rate(),
                "mean_top_relevance": stats.mean_top_relevance(),
                "relevance_low": stats.relevance_low.load(std::sync::atomic::Ordering::Relaxed),
                "relevance_mid": stats.relevance_mid.load(std::sync::atomic::Ordering::Relaxed),
                "relevance_good": stats.relevance_good.load(std::sync::atomic::Ordering::Relaxed),
                "relevance_excellent": stats.relevance_excellent.load(std::sync::atomic::Ordering::Relaxed),
            });

            let status = serde_json::json!({
                "node_id": s.node_id,
                "engine": s.engine.name(),
                "grid_size": GRID_SIZE,
                "grid_health": grid_health,
                "active_cells": active.len(),
                "total_activation": total_activation,
                "avg_confidence": avg_confidence,
                "peer_count": peers.len(),
                "peers": peers,
                "brain_path": s.brain_path,
                "network": true,
                "distributed_peers": dist_peers,
                "distributed": dist_peers > 0,
                "retrieval": retrieval,
            });
            let _ = writer.write_all(format!("{}\n", status).as_bytes()).await;
            let _ = writer.write_all(b"DONE\n").await;
        } else if line == "PEERS" {
            let s = state.lock().await;
            let peers = s.transport.connected_peers().await;
            if peers.is_empty() {
                let _ = writer
                    .write_all(b"No peers connected. Peers are discovered via mDNS.\n")
                    .await;
            } else {
                for p in &peers {
                    let _ = writer.write_all(format!("PEER {}\n", p).as_bytes()).await;
                }
            }
            let _ = writer.write_all(b"DONE\n").await;
        } else if line == "BRAIN" {
            // Send compact brain activation snapshot for visualization
            let s = state.lock().await;
            // Send as rows of space-separated f64 values (activation channel only)
            for y in 0..GRID_SIZE {
                let row: Vec<String> = (0..GRID_SIZE)
                    .map(|x| {
                        format!(
                            "{:.4}",
                            s.knowledge.grid.cells[y][x][sage::grid::KNOWLEDGE_ACTIVATION]
                        )
                    })
                    .collect();
                let _ = writer
                    .write_all(format!("ROW {}\n", row.join(" ")).as_bytes())
                    .await;
            }
            let _ = writer.write_all(b"DONE\n").await;
        } else if let Some(query) = line.strip_prefix("KNOWLEDGE ") {
            let s = state.lock().await;
            let results = s.knowledge.query(query, 10);
            if results.is_empty() {
                let _ = writer.write_all(b"No matching knowledge\n").await;
            } else {
                for r in &results {
                    let text = r.text.as_deref().unwrap_or("(no text)");
                    let msg = format!(
                        "MATCH [{},{}] relevance={:.3} confidence={:.1}% text={}\n",
                        r.position.0,
                        r.position.1,
                        r.relevance,
                        r.confidence * 100.0,
                        text
                    );
                    let _ = writer.write_all(msg.as_bytes()).await;
                }
            }
            let _ = writer.write_all(b"DONE\n").await;
        } else if let Some(message) = line.strip_prefix("CHAT ") {
            let message = message.to_string();

            // Query knowledge: local + distributed from peers
            let (knowledge_context, dist_peer_count, dist_source_count) = {
                let s = state.lock().await;
                // Local knowledge
                let local_results = s.knowledge.query(&message, 5);
                let local_lines: Vec<String> = local_results
                    .iter()
                    .filter(|k| k.relevance > 0.1)
                    .filter_map(|k| k.text.as_ref().map(|t| format!("- {}", t)))
                    .collect();
                let local_count = local_lines.len();

                // Distributed knowledge from peers
                let distributed = Arc::clone(&s.distributed);
                drop(s); // Release lock for async peer queries

                let peer_result = distributed.query_peers(&message, 5).await;
                let peer_lines: Vec<String> = peer_result
                    .items
                    .iter()
                    .filter(|k| k.relevance > 0.1)
                    .map(|k| format!("- {} [from peer]", k.text))
                    .collect();
                let peer_responded = peer_result.peers_responded;

                let mut all_lines = local_lines;
                all_lines.extend(peer_lines);

                let ctx = if all_lines.is_empty() {
                    None
                } else {
                    Some(format!(
                        "[SAGE Knowledge - relevant context from your brain{}]\n{}\n[End Knowledge]",
                        if peer_responded > 0 {
                            format!(" + {} peers", peer_responded)
                        } else {
                            String::new()
                        },
                        all_lines.join("\n")
                    ))
                };

                let total_sources = if peer_responded > 0 {
                    peer_responded + 1
                } else if local_count > 0 {
                    1
                } else {
                    0
                };
                (ctx, peer_result.peers_queried, total_sources)
            };
            let _ = (dist_peer_count, dist_source_count); // used for future status reporting

            // Encode user message into grid (privacy: defer broadcast until aggregation threshold met)
            {
                let mut s = state.lock().await;
                s.knowledge.encode(&message, 0.8);
                s.manager.record_conversation().await;
            }

            // Build inference messages
            session.conversation.push(InfChatMessage {
                role: ChatRole::User,
                content: message.clone(),
            });

            let system_content = if let Some(ref ctx) = knowledge_context {
                format!("{}\n\n{}", SAGE_SYSTEM_PROMPT, ctx)
            } else {
                SAGE_SYSTEM_PROMPT.to_string()
            };

            let mut inf_msgs = vec![InfChatMessage {
                role: ChatRole::System,
                content: system_content,
            }];
            for m in &session.conversation {
                inf_msgs.push(m.clone());
            }

            // Stream inference
            let engine = {
                let s = state.lock().await;
                Arc::clone(&s.engine)
            };

            let (token_tx, mut token_rx) = tokio::sync::mpsc::channel::<String>(256);

            let inf_handle = tokio::task::spawn_blocking(move || {
                engine
                    .chat_streaming(
                        &inf_msgs,
                        2000,
                        Box::new(move |token: &str| {
                            let _ = token_tx.blocking_send(token.to_string());
                        }),
                    )
                    .map_err(|e| e.to_string())
            });

            // Stream tokens to client
            let mut full_response = String::new();
            while let Some(token) = token_rx.recv().await {
                full_response.push_str(&token);
                // Escape newlines so protocol stays line-based
                let escaped = token.replace('\n', "\\n");
                let _ = writer
                    .write_all(format!("TOKEN {}\n", escaped).as_bytes())
                    .await;
            }

            match inf_handle.await {
                Ok(Ok(())) => {}
                Ok(Err(e)) => {
                    let _ = writer.write_all(format!("ERROR {}\n", e).as_bytes()).await;
                }
                Err(e) => {
                    let _ = writer.write_all(format!("ERROR {}\n", e).as_bytes()).await;
                }
            }

            let _ = writer.write_all(b"DONE\n").await;

            // Encode response into grid + save (privacy: defer broadcast until aggregation threshold met)
            if !full_response.is_empty() {
                session.conversation.push(InfChatMessage {
                    role: ChatRole::Assistant,
                    content: full_response.clone(),
                });

                let mut s = state.lock().await;
                s.knowledge.encode(&full_response, 0.7);
                let _ = s.knowledge.save(&s.brain_path);
                s.manager.record_conversation().await;
            }
        } else if let Some(query) = line.strip_prefix("KNOWLEDGE_QUERY ") {
            // Distributed inference: peer is querying our knowledge
            let query = query.to_string();
            let s = state.lock().await;
            let items = handle_knowledge_query(&s.knowledge, &query, 10, &s.node_id);
            for item in &items {
                if let Ok(json) = serde_json::to_string(item) {
                    let _ = writer
                        .write_all(format!("KNOWLEDGE_RESULT {}\n", json).as_bytes())
                        .await;
                }
            }
            drop(s);
            // Track stats
            {
                let mut s = state.lock().await;
                s.inference_stats.knowledge_served += items.len() as u64;
            }
            let _ = writer.write_all(b"DONE\n").await;
        } else if let Some(spec_data) = line.strip_prefix("SPECULATE ") {
            // Distributed inference: peer wants us to speculatively generate
            let parts: Vec<&str> = spec_data.splitn(2, ' ').collect();
            if parts.len() == 2 {
                let prompt_text = parts[1].replace("\\n", "\n");
                // Generate a short speculative response (limited tokens)
                let engine = {
                    let s = state.lock().await;
                    Arc::clone(&s.engine)
                };
                let (token_tx, mut token_rx) = tokio::sync::mpsc::channel::<String>(64);
                let prompt = prompt_text.clone();
                tokio::task::spawn_blocking(move || {
                    let _ = engine.generate_streaming(
                        &prompt,
                        128, // Short speculative generation
                        Box::new(move |token: &str| {
                            let _ = token_tx.blocking_send(token.to_string());
                        }),
                    );
                });
                while let Some(token) = token_rx.recv().await {
                    let escaped = token.replace('\n', "\\n");
                    let _ = writer
                        .write_all(format!("TOKEN {}\n", escaped).as_bytes())
                        .await;
                }
                {
                    let mut s = state.lock().await;
                    s.inference_stats.speculation_requests += 1;
                }
            }
            let _ = writer.write_all(b"DONE\n").await;
        } else if line == "DIST_STATUS" {
            // Return distributed inference metrics
            let s = state.lock().await;
            let peer_count = s.distributed.peer_count().await;
            let metrics = s.distributed.metrics_snapshot();
            let status = serde_json::json!({
                "distributed_peers": peer_count,
                "metrics": metrics,
                "inference_stats": s.inference_stats,
            });
            let _ = writer.write_all(format!("{}\n", status).as_bytes()).await;
            let _ = writer.write_all(b"DONE\n").await;
        } else if let Some(name) = line.strip_prefix("EXPORT_TEMPLATE ") {
            // Export current brain to named template
            let s = state.lock().await;
            let bundle = sage::brain_templates::BrainTemplateBundle::from_knowledge(
                &s.knowledge,
                name,
                &format!("Live-exported from node {}", s.node_id),
                vec![],
                None,
            );
            let templates_dir = sage::brain_templates::default_templates_dir();
            drop(s);
            match bundle.save(&templates_dir) {
                Ok(path) => {
                    let _ = writer
                        .write_all(format!("OK Exported template '{}' -> {}\n", name, path).as_bytes())
                        .await;
                }
                Err(e) => {
                    let _ = writer.write_all(format!("ERROR Export failed: {}\n", e).as_bytes()).await;
                }
            }
            let _ = writer.write_all(b"DONE\n").await;
        } else if line == "LIST_TEMPLATES" {
            let templates_dir = sage::brain_templates::default_templates_dir();
            let templates = sage::brain_templates::list_templates(&templates_dir);
            if templates.is_empty() {
                let _ = writer.write_all(b"No templates available\n").await;
            } else {
                for t in &templates {
                    let tags = if t.meta.tags.is_empty() {
                        String::new()
                    } else {
                        format!(" [{}]", t.meta.tags.join(","))
                    };
                    let _ = writer
                        .write_all(
                            format!(
                                "TEMPLATE {} {}{} | {} active | {}\n",
                                t.meta.name,
                                t.meta.domain.as_deref().unwrap_or("general"),
                                tags,
                                t.meta.active_cells,
                                t.meta.description,
                            )
                            .as_bytes(),
                        )
                        .await;
                }
            }
            let _ = writer.write_all(b"DONE\n").await;
        } else if let Some(name) = line.strip_prefix("IMPORT_TEMPLATE ") {
            // Import and activate a brain template at runtime (hot-swap)
            let templates_dir = sage::brain_templates::default_templates_dir();
            match sage::brain_templates::find_template(name, &templates_dir) {
                Ok(bundle) => {
                    let mut s = state.lock().await;
                    let active_cells = bundle.meta.active_cells;
                    s.knowledge = bundle.to_knowledge();
                    if let Err(e) = s.knowledge.save(&s.brain_path) {
                        let _ = writer.write_all(format!("ERROR Import failed: {}\n", e).as_bytes()).await;
                    } else {
                        let _ = writer.write_all(
                            format!(
                                "OK Imported template '{}' ({} active cells) -> {}\n",
                                name, active_cells, s.brain_path
                            )
                            .as_bytes(),
                        )
                        .await;
                    }
                }
                Err(e) => {
                    let _ = writer.write_all(format!("ERROR Import failed: {}\n", e).as_bytes()).await;
                }
            }
            let _ = writer.write_all(b"DONE\n").await;
        } else {
            let _ = writer
                .write_all(
                    b"ERROR Unknown command. Use CHAT, STATUS, PEERS, KNOWLEDGE, BRAIN, EXPORT_TEMPLATE, IMPORT_TEMPLATE, LIST_TEMPLATES, KNOWLEDGE_QUERY, SPECULATE, or QUIT.\n",
                )
                .await;
            let _ = writer.write_all(b"DONE\n").await;
        }
    }

    println!("[client] {} disconnected", addr);
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // Ensure SAGE_HOME exists
    let home = sage_home();
    let _ = std::fs::create_dir_all(&home);

    let key_path = identity_key_path();
    let identity = match NodeIdentity::load_or_generate(Some(&key_path)) {
        Ok(id) => id,
        Err(e) => {
            eprintln!("Failed to load/generate identity: {e}");
            std::process::exit(1);
        }
    };
    let node_id = identity.node_id.clone();

    // Load NCA knowledge grid
    let bp = brain_path();
    let node_id_f64 = f64::from_bits(u64::from_le_bytes(
        identity.public_key[..8].try_into().unwrap(),
    ));
    let mut knowledge = NCAKnowledge::new().with_node_id(node_id_f64);

    // If --template specified, bootstrap from template (override brain.bin)
    if let Some(ref template_name) = cli.template {
        let templates_dir = sage::brain_templates::default_templates_dir();
        match sage::brain_templates::find_template(template_name, &templates_dir) {
            Ok(bundle) => {
                println!("📦 Bootstrapping from template: {}", bundle.meta.name);
                println!("   Description: {}", bundle.meta.description);
                println!(
                    "   Source: {} active cells from {}",
                    bundle.meta.active_cells, bundle.meta.source_node_id
                );
                knowledge = bundle.to_knowledge();
                // Save as brain.bin so future restarts pick it up
                if let Err(e) = knowledge.save(&bp) {
                    eprintln!("Warning: could not save brain after template import: {e}");
                } else {
                    println!("   → Saved to {}", bp);
                }
            }
            Err(e) => {
                eprintln!("❌ Failed to load template '{}': {}", template_name, e);
                std::process::exit(1);
            }
        }
    } else if Path::new(&bp).exists() {
        if let Err(e) = knowledge.load(&bp) {
            eprintln!("Warning: could not load brain: {e}");
        }
    }
    let active_count = knowledge.active_knowledge(0.01).len();

    // Initialize inference engine
    let engine: Arc<dyn InferenceEngine> = Arc::from(inference::engine_with_preference(
        true, // prefer ollama
        None, // use config model
        None, // use default url
    ));

    println!("🌐 SAGE Node: {}", node_id);
    println!("   P2P ID:    {}", identity.peer_id());
    println!("   Home:      {}", home.display());
    println!("   Port:      {}", cli.port);
    println!("   Engine:    {}", engine.name());
    println!(
        "   mDNS:      {}",
        if cli.no_mdns { "disabled" } else { "enabled" }
    );
    println!("   Knowledge: {} active cells", active_count);
    println!();

    // Start network manager
    let net_config = NetworkConfig {
        sync_interval_secs: cli.sync_interval,
        listen_port: cli.gossip_port,
        mdns_enabled: !cli.no_mdns,
        ..Default::default()
    };
    let manager = Arc::new(NetworkManager::new(identity.clone(), net_config));
    if let Err(e) = manager.start().await {
        eprintln!("Failed to start networking: {e}");
        std::process::exit(1);
    }

    // Start libp2p transport with the node's persistent Ed25519 keypair.
    // This ensures the libp2p PeerId is stable across restarts and cryptographically
    // tied to the SAGE node identity (same seed → same PeerId, always).
    let bootstrap_nodes = load_bootstrap_nodes();
    let has_bootstrap = !bootstrap_nodes.is_empty();
    let libp2p_config = Libp2pConfig {
        listen_port: cli.gossip_port,
        mdns_enabled: !cli.no_mdns,
        bootstrap_nodes,
    };
    let libp2p_keypair = identity.to_libp2p_keypair();
    let transport = Arc::new(Libp2pTransport::new(libp2p_config, Some(libp2p_keypair)));
    if let Err(e) = transport.start().await {
        eprintln!("Failed to start libp2p transport: {e}");
        std::process::exit(1);
    }

    if has_bootstrap {
        // Give Kademlia a moment to connect, then report
        tokio::time::sleep(Duration::from_secs(2)).await;
        let peers = transport.connected_peers().await;
        if !peers.is_empty() {
            println!("🌍 Connected to bootstrap network ({} peers)", peers.len());
        } else {
            println!("⚠️  Bootstrap nodes configured but unreachable — Offline mode (mDNS only)");
        }
    } else {
        println!("📡 No bootstrap nodes configured — mDNS only (LAN discovery)");
    }

    let distributed = Arc::new(DistributedInference::new());

    let state = Arc::new(Mutex::new(NodeState {
        knowledge,
        brain_path: bp.clone(),
        engine,
        node_id: node_id.clone(),
        transport: Arc::clone(&transport),
        manager: Arc::clone(&manager),
        distributed: Arc::clone(&distributed),
        inference_stats: InferenceStats::new(),
    }));

    // Spawn gossip receiver
    let state_for_gossip = Arc::clone(&state);
    let transport_recv = Arc::clone(&transport);
    tokio::spawn(async move {
        while let Ok((_peer_id, msg)) = transport_recv.recv().await {
            match msg {
                GossipMessage::KnowledgeDiff(diff) => {
                    let mut s = state_for_gossip.lock().await;
                    let changes = diff.changes.len();
                    diff.apply_weighted(&mut s.knowledge.grid.cells, 0.8);
                    println!(
                        "[gossip] Applied diff from {} ({} changes)",
                        diff.source_node, changes
                    );
                    // Save brain after receiving peer knowledge
                    let _ = s.knowledge.save(&s.brain_path);
                }
                GossipMessage::PeerAnnounce(announce) => {
                    println!("[gossip] Peer announce: {}", announce.node_id);
                }
                _ => {}
            }
        }
    });

    // Spawn periodic sync broadcaster
    // Uses NetworkManager to enforce aggregation threshold (min conversations before sync)
    // and apply differential privacy + local-only channel filtering.
    let state_for_sync = Arc::clone(&state);
    let network_for_sync = Arc::clone(&manager);
    let sync_interval = cli.sync_interval;
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(sync_interval));
        loop {
            interval.tick().await;

            // Skip sync if aggregation threshold not yet met (privacy via aggregation)
            // The threshold is tracked by conversations recorded in CHAT handling
            if !network_for_sync.is_ready_to_sync().await {
                // Not enough conversations yet — skip this sync window
                continue;
            }

            // Compute outgoing diff via NetworkManager (applies privacy filtering,
            // differential privacy noise, and signs with Ed25519)
            let current_grid: Vec<Vec<Vec<f64>>> = {
                let s = state_for_sync.lock().await;
                s.knowledge.grid.cells.clone()
            };

            if let Some(diff) = network_for_sync.compute_outgoing_diff(&current_grid).await {
                println!(
                    "[sync] Broadcasting {} cell changes (aggregation threshold met)",
                    diff.changes.len()
                );
                let _ = network_for_sync
                    .broadcast(GossipMessage::KnowledgeDiff(diff))
                    .await;
                network_for_sync.reset_aggregation().await;
            }
        }
    });

    // Start TCP listener for clients
    let listener = match TcpListener::bind(format!("127.0.0.1:{}", cli.port)).await {
        Ok(l) => {
            println!("Listening on 127.0.0.1:{}", cli.port);
            l
        }
        Err(e) => {
            eprintln!("Failed to bind port {}: {}", cli.port, e);
            std::process::exit(1);
        }
    };

    println!("Node is running. Press Ctrl+C to stop.\n");

    let state_for_accept = Arc::clone(&state);
    let accept_task = tokio::spawn(async move {
        loop {
            match listener.accept().await {
                Ok((stream, addr)) => {
                    let s = Arc::clone(&state_for_accept);
                    tokio::spawn(handle_client(stream, s, addr));
                }
                Err(e) => {
                    eprintln!("[tcp] Accept error: {}", e);
                }
            }
        }
    });

    // Wait for Ctrl+C
    tokio::signal::ctrl_c().await.ok();
    println!("\nShutting down...");

    // Save brain
    {
        let s = state.lock().await;
        if let Err(e) = s.knowledge.save(&s.brain_path) {
            eprintln!("Warning: could not save brain: {e}");
        } else {
            let count = s.knowledge.active_knowledge(0.01).len();
            println!("Brain saved ({} active cells)", count);
        }
    }

    accept_task.abort();
    transport.stop().await.ok();
    manager.stop().await.ok();
}

fn load_bootstrap_nodes() -> Vec<String> {
    let cp = config_path();
    if !cp.exists() {
        return Vec::new();
    }
    let Ok(content) = std::fs::read_to_string(&cp) else {
        return Vec::new();
    };
    let Ok(table) = content.parse::<toml::Table>() else {
        return Vec::new();
    };
    let network = match table.get("network") {
        Some(n) => n,
        None => return Vec::new(),
    };
    // Support both "bootstrap" and "bootstrap_nodes" keys
    let arr = network
        .get("bootstrap")
        .or_else(|| network.get("bootstrap_nodes"))
        .and_then(|b| b.as_array());
    arr.map(|a| {
        a.iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect()
    })
    .unwrap_or_default()
}
