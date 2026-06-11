/**
 * SAGE Network Stats Server
 *
 * HTTP API for whatssage.ai/network dashboard
 * Aggregates stats from SAGE nodes and serves them to the web UI
 *
 * Usage: sage-network-server [--port 3001] [--sage-api http://localhost:19176]
 */
use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use sage::specialist::SpecialistProfile;
use tokio::time::{interval, Duration};
use tower_http::cors::CorsLayer;

const DEFAULT_PORT: u16 = 3001;
const DEFAULT_SAGE_API: &str = "http://localhost:19176";

#[derive(Debug, Clone, Serialize, Deserialize)]
struct NodeStats {
    id: String,
    name: String,
    version: String,
    status: String, // "online", "offline", "syncing"
    peers: usize,
    patterns: usize,
    grid_utilization: f32, // 0.0 - 1.0
    last_seen: u64,        // unix timestamp
    uptime_seconds: u64,
    contribution_tier: String, // "seedling", "sprout", "grove", "forest"
    /// Retrieval quality metrics (optional, from /STATUS endpoint)
    pub retrieval: Option<RetrievalMetrics>,
}

/// Knowledge retrieval quality metrics for dashboard display.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct RetrievalMetrics {
    total_queries: u64,
    hit_rate: f64,
    mean_top_relevance: f64,
    relevance_low: u64,
    relevance_mid: u64,
    relevance_good: u64,
    relevance_excellent: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct NetworkStats {
    total_nodes: usize,
    online_nodes: usize,
    total_patterns: usize,
    active_peers: usize,
    last_updated: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ActivityEvent {
    timestamp: u64,
    node_id: String,
    event_type: String, // "sync", "dream", "import", "error"
    details: String,
    severity: String, // "info", "warn", "error"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PendingApproval {
    id: String,
    node_id: String,
    node_name: String,
    requested_at: u64,
    action: String,
}

// Shared state
#[derive(Clone)]
struct AppState {
    nodes: Arc<Mutex<HashMap<String, NodeStats>>>,
    activities: Arc<Mutex<Vec<ActivityEvent>>>,
    pending: Arc<Mutex<Vec<PendingApproval>>>,
    specialists: Arc<Mutex<HashMap<String, SpecialistProfile>>>,
    sage_api_url: String,
}

#[derive(Parser, Debug)]
#[command(name = "sage-network-server")]
#[command(about = "SAGE Network Stats Server for whatssage.ai dashboard")]
struct Args {
    /// Port to serve on
    #[arg(short, long, default_value_t = DEFAULT_PORT)]
    port: u16,

    /// SAGE API URL to poll for stats
    #[arg(short, long, default_value = DEFAULT_SAGE_API)]
    sage_api: String,

    /// Poll interval in seconds
    #[arg(long, default_value_t = 30)]
    poll_interval: u64,
}

// GET /health - Health check
async fn health() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "ok",
        "service": "sage-network-stats"
    }))
}

// GET /api/v1/network/stats - Aggregate network stats
async fn get_network_stats(State(state): State<AppState>) -> impl IntoResponse {
    let nodes = state.nodes.lock().unwrap();

    let online = nodes.values().filter(|n| n.status == "online").count();
    let total_patterns: usize = nodes.values().map(|n| n.patterns).sum();
    let active_peers: usize = nodes.values().map(|n| n.peers).sum();

    let stats = NetworkStats {
        total_nodes: nodes.len(),
        online_nodes: online,
        total_patterns,
        active_peers,
        last_updated: now(),
    };

    Json(stats)
}

// GET /api/v1/network/nodes - List all nodes
async fn get_nodes(State(state): State<AppState>) -> impl IntoResponse {
    let nodes = state.nodes.lock().unwrap();
    let node_list: Vec<NodeStats> = nodes.values().cloned().collect();
    Json(node_list)
}

// GET /api/v1/network/nodes/:id - Get specific node
async fn get_node(
    axum::extract::Path(node_id): axum::extract::Path<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let nodes = state.nodes.lock().unwrap();

    match nodes.get(&node_id) {
        Some(node) => (StatusCode::OK, Json(serde_json::json!(node))),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": "Node not found"
            })),
        ),
    }
}

// GET /api/v1/network/activity - Recent activity feed
async fn get_activity(State(state): State<AppState>) -> impl IntoResponse {
    let activities = state.activities.lock().unwrap();
    // Return last 50 events, newest first
    let recent: Vec<ActivityEvent> = activities.iter().rev().take(50).cloned().collect();
    Json(recent)
}

// GET /api/v1/network/pending - Pending approvals
async fn get_pending(State(state): State<AppState>) -> impl IntoResponse {
    let pending = state.pending.lock().unwrap();
    Json(pending.clone())
}

// GET /api/v1/router/stats - Intelligent router statistics
async fn get_router_stats() -> impl IntoResponse {
    // Try to load router statistics from persistence
    let router_path = dirs::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join(".sage")
        .join("intelligent_router.json");

    match std::fs::read_to_string(&router_path) {
        Ok(json) => match serde_json::from_str::<serde_json::Value>(&json) {
            Ok(stats) => (StatusCode::OK, Json(stats)),
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": format!("Failed to parse router stats: {}", e)
                })),
            ),
        },
        Err(e) => {
            // No router stats yet - return empty structure
            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "pattern_stats": {},
                    "min_attempts_for_learning": 10,
                    "use_learning": true,
                    "default_complexity": "Moderate",
                    "nca_available": false,
                    "exploration_rate": 0.1,
                    "status": "no_data",
                    "message": format!("No router stats found yet: {}", e)
                })),
            )
        }
    }
}

// POST /api/v1/network/ping - Node heartbeat
#[derive(Deserialize)]
struct PingRequest {
    node_id: String,
    name: String,
    version: String,
    peers: usize,
    patterns: usize,
    grid_utilization: f32,
    uptime_seconds: u64,
}

async fn node_ping(
    State(state): State<AppState>,
    Json(req): Json<PingRequest>,
) -> impl IntoResponse {
    let mut nodes = state.nodes.lock().unwrap();

    // Determine contribution tier based on patterns
    let tier = if req.patterns > 10000 {
        "forest"
    } else if req.patterns > 1000 {
        "grove"
    } else if req.patterns > 100 {
        "sprout"
    } else {
        "seedling"
    };

    let node = NodeStats {
        id: req.node_id.clone(),
        name: req.name.clone(),
        version: req.version.clone(),
        status: "online".to_string(),
        peers: req.peers,
        patterns: req.patterns,
        grid_utilization: req.grid_utilization,
        last_seen: now(),
        uptime_seconds: req.uptime_seconds,
        contribution_tier: tier.to_string(),
        retrieval: None,
    };

    nodes.insert(req.node_id.clone(), node);

    // Log activity
    let mut activities = state.activities.lock().unwrap();
    activities.push(ActivityEvent {
        timestamp: now(),
        node_id: req.node_id.clone(),
        event_type: "ping".to_string(),
        details: format!(
            "Node {} checked in with {} patterns",
            req.name, req.patterns
        ),
        severity: "info".to_string(),
    });

    // Keep only last 1000 events
    if activities.len() > 1000 {
        activities.remove(0);
    }

    Json(serde_json::json!({
        "status": "ok",
        "tier": tier
    }))
}

// Background task: Poll local SAGE node for stats
async fn poll_sage_node(state: AppState, poll_interval_secs: u64) {
    let client = reqwest::Client::new();
    let mut interval = interval(Duration::from_secs(poll_interval_secs));

    loop {
        interval.tick().await;

        // Try to get status from local SAGE API
        match client
            .get(format!("{}/v1/sage/status", state.sage_api_url))
            .send()
            .await
        {
            Ok(resp) => {
                if let Ok(json) = resp.json::<serde_json::Value>().await {
                    // Extract stats from response
                    let node_id = json
                        .get("node_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("local")
                        .to_string();

                    let patterns = json
                        .get("patterns_matched")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0) as usize;

                    let grid_active = json
                        .get("grid_active_cells")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0) as usize;

                    let grid_total = 65536; // 256x256
                    let utilization = grid_active as f32 / grid_total as f32;

                    // Extract retrieval quality metrics if present
                    let retrieval = json.get("retrieval").and_then(|r| {
                        Some(RetrievalMetrics {
                            total_queries: r.get("total_queries")?.as_u64()?,
                            hit_rate: r.get("hit_rate")?.as_f64()?,
                            mean_top_relevance: r.get("mean_top_relevance")?.as_f64()?,
                            relevance_low: r.get("relevance_low")?.as_u64()?,
                            relevance_mid: r.get("relevance_mid")?.as_u64()?,
                            relevance_good: r.get("relevance_good")?.as_u64()?,
                            relevance_excellent: r.get("relevance_excellent")?.as_u64()?,
                        })
                    });

                    let mut nodes = state.nodes.lock().unwrap();
                    nodes.insert(
                        node_id.clone(),
                        NodeStats {
                            id: node_id.clone(),
                            name: "Local Node".to_string(),
                            version: json
                                .get("version")
                                .and_then(|v| v.as_str())
                                .unwrap_or("unknown")
                                .to_string(),
                            status: "online".to_string(),
                            peers: json.get("peer_count").and_then(|v| v.as_u64()).unwrap_or(0)
                                as usize,
                            patterns,
                            grid_utilization: utilization,
                            last_seen: now(),
                            uptime_seconds: 0,
                            contribution_tier: if patterns > 1000 {
                                "grove".to_string()
                            } else {
                                "seedling".to_string()
                            },
                            retrieval,
                        },
                    );

                    // Log successful poll
                    let mut activities = state.activities.lock().unwrap();
                    activities.push(ActivityEvent {
                        timestamp: now(),
                        node_id: node_id.clone(),
                        event_type: "poll".to_string(),
                        details: format!("Polled local node: {} patterns", patterns),
                        severity: "info".to_string(),
                    });

                    if activities.len() > 1000 {
                        activities.remove(0);
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to poll SAGE node: {}", e);

                // Log error
                let mut activities = state.activities.lock().unwrap();
                activities.push(ActivityEvent {
                    timestamp: now(),
                    node_id: "system".to_string(),
                    event_type: "error".to_string(),
                    details: format!("Failed to connect to stats API: {}", e),
                    severity: "error".to_string(),
                });

                if activities.len() > 1000 {
                    activities.remove(0);
                }
            }
        }

        // Clean up stale nodes (not seen in 5 minutes)
        let mut nodes = state.nodes.lock().unwrap();
        let cutoff = now() - 300;
        nodes.retain(|_, node| node.last_seen > cutoff);
    }
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    println!("🌐 SAGE Network Stats Server");
    println!("   Port: {}", args.port);
    println!("   SAGE API: {}", args.sage_api);
    println!("   Poll interval: {}s", args.poll_interval);
    println!();

    let state = AppState {
        nodes: Arc::new(Mutex::new(HashMap::new())),
        activities: Arc::new(Mutex::new(Vec::new())),
        pending: Arc::new(Mutex::new(Vec::new())),
        sage_api_url: args.sage_api.clone(),
    };

    // Start background polling task
    let poll_state = state.clone();
    tokio::spawn(async move {
        poll_sage_node(poll_state, args.poll_interval).await;
    });

    // Build router with CORS
    let app = Router::new()
        .route("/health", get(health))
        .route("/api/v1/network/stats", get(get_network_stats))
        .route("/api/v1/network/nodes", get(get_nodes))
        .route("/api/v1/network/nodes/:id", get(get_node))
        .route("/api/v1/network/activity", get(get_activity))
        .route("/api/v1/network/pending", get(get_pending))
        .route("/api/v1/network/ping", post(node_ping))
        .route("/api/v1/router/stats", get(get_router_stats))
        .route("/api/v1/specialists", get(list_specialists_handler))
        .route("/api/v1/specialists/:name", get(get_specialist_handler))
        .route("/api/v1/specialists", post(publish_specialist_handler))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], args.port));
    println!("🚀 Server running on http://{}", addr);

    // Start server using tokio
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    println!("🚀 Server running on http://{}", addr);
    axum::serve(listener, app).await.unwrap();
}

// ─── Specialist Catalog Handlers ─────────────────────────────────────

/// GET /api/v1/specialists — list all published specialists
async fn list_specialists_handler(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let specialists = state.specialists.lock().unwrap();
    let list: Vec<&SpecialistProfile> = specialists.values().collect();
    Json(serde_json::json!({
        "specialists": list,
        "count": list.len(),
    }))
}

/// GET /api/v1/specialists/:name — get a specific specialist
async fn get_specialist_handler(
    State(state): State<AppState>,
    axum::extract::Path(name): axum::extract::Path<String>,
) -> impl IntoResponse {
    let specialists = state.specialists.lock().unwrap();
    match specialists.get(&name) {
        Some(profile) => Json(serde_json::json!(profile)).into_response(),
        None => (StatusCode::NOT_FOUND, format!("Specialist '{}' not found", name)).into_response(),
    }
}

/// POST /api/v1/specialists — publish a new specialist
async fn publish_specialist_handler(
    State(state): State<AppState>,
    Json(profile): Json<SpecialistProfile>,
) -> impl IntoResponse {
    let mut specialists = state.specialists.lock().unwrap();

    if specialists.contains_key(&profile.name) {
        return (
            StatusCode::CONFLICT,
            format!("Specialist '{}' already exists. Use PUT to update.", profile.name),
        )
            .into_response();
    }

    let name = profile.name.clone();
    specialists.insert(name.clone(), profile);

    println!("📦 Specialist published: {}", name);

    (
        StatusCode::CREATED,
        Json(serde_json::json!({
            "status": "published",
            "name": name,
            "url": format!("/api/v1/specialists/{}", name),
        })),
    )
        .into_response()
}
