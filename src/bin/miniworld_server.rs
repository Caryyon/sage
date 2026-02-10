//! Miniworld WebSocket Server
//!
//! Runs the SAGE village simulation and broadcasts state via WebSocket.
//! Supports delta updates: sends full_state on connect, then delta messages
//! containing only changed characters/tiles for reduced bandwidth.

use axum::{
    Router,
    extract::{State, WebSocketUpgrade},
    extract::ws::{Message, WebSocket},
    response::IntoResponse,
    routing::get,
};
use futures::{SinkExt, StreamExt};
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tower_http::services::{ServeDir, ServeFile};
use tower_http::cors::{CorsLayer, Any};

use sage::miniworld::{World, create_default_town, town::add_default_sages, OpenClawBridge};
use sage::miniworld::character::CharacterState;
use sage::miniworld::openclaw_bridge::TaskType;

/// Shared world state
struct AppState {
    world: RwLock<World>,
    _tx: broadcast::Sender<String>,
    /// Broadcast channel for delta updates (sent after first full_state)
    delta_tx: broadcast::Sender<String>,
    openclaw: OpenClawBridge,
}

/// Full world state message sent to clients on connect
#[derive(Serialize)]
struct WorldStateMessage {
    r#type: String,
    world: WorldSnapshot,
}

/// Delta update message — only changed data
#[derive(Serialize)]
struct DeltaMessage {
    r#type: String,
    time_of_day: u32,
    tick: u64,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    characters_changed: HashMap<String, CharacterSnapshot>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    characters_removed: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tiles_changed: Vec<TileChange>,
}

#[derive(Serialize)]
struct TileChange {
    x: u32,
    y: u32,
    tile: TileSnapshot,
}

/// Snapshot of world state for JSON serialization
#[derive(Serialize, Clone)]
struct WorldSnapshot {
    config: ConfigSnapshot,
    tiles: Vec<Vec<TileSnapshot>>,
    characters: HashMap<String, CharacterSnapshot>,
    time_of_day: u32,
    tick: u64,
}

#[derive(Serialize, Clone)]
struct ConfigSnapshot {
    width: u32,
    height: u32,
    name: String,
}

#[derive(Serialize, Clone, PartialEq)]
struct TileSnapshot {
    ground: String,
    overlay: Option<String>,
    sprite_col: u8,
    sprite_row: u8,
    team_color: Option<String>,
}

#[derive(Serialize, Clone, PartialEq)]
struct CharacterSnapshot {
    id: String,
    name: String,
    x: u32,
    y: u32,
    direction: String,
    state: String,
    sprite: String,
    anim_frame: u8,
    current_task: Option<String>,
    task_status: Option<String>,
    last_result: Option<String>,
}

/// Pre-computed task info for characters (gathered async before snapshot)
type CharacterTaskInfo = HashMap<String, (Option<String>, Option<String>, Option<String>)>;

fn snapshot_world(world: &World) -> WorldSnapshot {
    snapshot_world_with_tasks(world, &HashMap::new())
}

fn snapshot_world_with_tasks(world: &World, task_info: &CharacterTaskInfo) -> WorldSnapshot {
    let tiles: Vec<Vec<TileSnapshot>> = world.tiles.iter().map(|row| {
        row.iter().map(|tile| {
            TileSnapshot {
                ground: format!("{:?}", tile.ground),
                overlay: tile.overlay.map(|o| format!("{:?}", o)),
                sprite_col: tile.sprite_col,
                sprite_row: tile.sprite_row,
                team_color: if tile.team_color != sage::miniworld::TeamColor::Wood {
                    Some(format!("{:?}", tile.team_color))
                } else {
                    None
                },
            }
        }).collect()
    }).collect();

    let characters: HashMap<String, CharacterSnapshot> = world.characters.iter()
        .map(|(id, c)| {
            let (current_task, task_status, last_result) = task_info
                .get(id)
                .cloned()
                .unwrap_or((None, None, None));
            (id.clone(), CharacterSnapshot {
                id: c.id.clone(),
                name: c.name.clone(),
                x: c.x,
                y: c.y,
                direction: format!("{:?}", c.direction).to_lowercase(),
                state: match &c.state {
                    CharacterState::Idle => "idle".to_string(),
                    CharacterState::Walking => "walking".to_string(),
                    CharacterState::Working => "working".to_string(),
                    CharacterState::Talking { with } => format!("talking:{}", with),
                    CharacterState::Sleeping => "sleeping".to_string(),
                    CharacterState::Eating => "eating".to_string(),
                    CharacterState::Shopping => "shopping".to_string(),
                    CharacterState::Researching { topic } => format!("researching:{}", topic),
                    CharacterState::Coding { project } => format!("coding:{}", project),
                    CharacterState::Analyzing { subject } => format!("analyzing:{}", subject),
                },
                sprite: format!("{:?}", c.sprite),
                anim_frame: c.anim_frame,
                current_task,
                task_status,
                last_result,
            })
        })
        .collect();

    WorldSnapshot {
        config: ConfigSnapshot {
            width: world.config.width,
            height: world.config.height,
            name: world.config.name.clone(),
        },
        tiles,
        characters,
        time_of_day: world.time_of_day,
        tick: world.tick,
    }
}

/// Compute delta between previous and current snapshots
fn compute_delta(prev: &WorldSnapshot, curr: &WorldSnapshot) -> DeltaMessage {
    let mut characters_changed = HashMap::new();
    let mut characters_removed = Vec::new();

    // Find changed or new characters
    for (id, char_snap) in &curr.characters {
        match prev.characters.get(id) {
            Some(prev_char) if prev_char == char_snap => {} // unchanged
            _ => { characters_changed.insert(id.clone(), char_snap.clone()); }
        }
    }

    // Find removed characters
    for id in prev.characters.keys() {
        if !curr.characters.contains_key(id) {
            characters_removed.push(id.clone());
        }
    }

    // Tile changes (rare — only check if we suspect changes)
    let tiles_changed = Vec::new();
    // NOTE: tiles rarely change at runtime, so we skip diff for now.
    // If tile mutations are added, compare prev.tiles vs curr.tiles here.

    DeltaMessage {
        r#type: "delta".to_string(),
        time_of_day: curr.time_of_day,
        tick: curr.tick,
        characters_changed,
        characters_removed,
        tiles_changed,
    }
}

async fn api_instances(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let world = state.world.read().await;
    let chars: Vec<serde_json::Value> = world.characters.iter().map(|(id, c)| {
        serde_json::json!({
            "instance_id": id,
            "name": c.name,
            "role": format!("{:?}", c.sprite),
            "status": "online",
            "total_tasks": 0,
            "success_rate": 100.0,
            "pending_approvals": 0,
            "expertise_level": "Apprentice"
        })
    }).collect();
    axum::Json(serde_json::json!({
        "success": true,
        "data": chars
    }))
}

async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    let (mut sender, mut receiver) = socket.split();

    // Send initial full state
    {
        let world = state.world.read().await;
        let snapshot = snapshot_world(&world);
        let msg = WorldStateMessage {
            r#type: "full_state".to_string(),
            world: snapshot,
        };
        if let Ok(json) = serde_json::to_string(&msg) {
            let _ = sender.send(Message::Text(json)).await;
        }
    }

    // Subscribe to delta updates (not full state broadcasts)
    let mut rx = state.delta_tx.subscribe();

    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if sender.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });

    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Close(_) => break,
                Message::Text(text) => {
                    println!("Received: {}", text);
                }
                _ => {}
            }
        }
    });

    tokio::select! {
        _ = &mut send_task => recv_task.abort(),
        _ = &mut recv_task => send_task.abort(),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let port = std::env::var("PORT").unwrap_or_else(|_| "8888".to_string())
        .parse::<u16>().unwrap_or(8888);

    let mut world = create_default_town();
    add_default_sages(&mut world);

    println!("🏘️  Created SAGE Village with {} characters", world.characters.len());

    let (tx, _) = broadcast::channel::<String>(100);
    let (delta_tx, _) = broadcast::channel::<String>(100);

    let openclaw = OpenClawBridge::new();

    let state = Arc::new(AppState {
        world: RwLock::new(world),
        _tx: tx.clone(),
        delta_tx: delta_tx.clone(),
        openclaw,
    });

    let research_topics = ["neural architecture search", "self-supervised learning", "meta-learning strategies",
        "curiosity-driven exploration", "emergent behavior in multi-agent systems",
        "transformer attention mechanisms", "reinforcement learning from human feedback"];
    let coding_projects = ["pattern recognition module", "memory consolidation system", "adaptive learning rate scheduler",
        "distributed training pipeline", "knowledge graph builder", "autonomous code reviewer"];
    let analysis_subjects = ["training loss convergence patterns", "agent interaction dynamics",
        "resource allocation efficiency", "learning transfer between domains",
        "population diversity metrics", "novelty search effectiveness"];

    // Simulation loop
    let sim_state = state.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(100));
        let mut task_poll_counter: u64 = 0;
        let mut prev_snapshot: Option<WorldSnapshot> = None;

        loop {
            interval.tick().await;
            task_poll_counter += 1;

            // Update world
            let current_tick = {
                let mut world = sim_state.world.write().await;
                world.tick(0.1);

                let w = world.config.width;
                let h = world.config.height;
                let char_ids: Vec<String> = world.characters.keys().cloned().collect();
                for id in char_ids {
                    if let Some(character) = world.characters.get_mut(&id) {
                        if character.state == CharacterState::Idle && character.destination.is_none()
                            && rand::random::<f32>() < 0.05 {
                                character.wander(w, h);
                            }
                    }
                }
                world.tick
            };

            // Periodically spawn OpenClaw tasks
            if task_poll_counter.is_multiple_of(10) {
                let char_states: Vec<(String, CharacterState, String)> = {
                    let world = sim_state.world.read().await;
                    world.characters.iter().map(|(id, c)| {
                        (id.clone(), c.state.clone(), format!("{:?}", c.sprite))
                    }).collect()
                };

                for (id, state, sprite) in &char_states {
                    if !matches!(state, CharacterState::Idle | CharacterState::Working) {
                        continue;
                    }
                    if sim_state.openclaw.character_is_busy(id).await {
                        continue;
                    }
                    if rand::random::<f32>() > 0.02 {
                        continue;
                    }

                    let task_type = if sprite.contains("Mage") {
                        let topic = research_topics[rand::random::<usize>() % research_topics.len()].to_string();
                        TaskType::Research { topic }
                    } else if sprite.contains("Swordsman") {
                        let project = coding_projects[rand::random::<usize>() % coding_projects.len()].to_string();
                        TaskType::Coding { project }
                    } else {
                        let subject = analysis_subjects[rand::random::<usize>() % analysis_subjects.len()].to_string();
                        TaskType::Analysis { subject }
                    };

                    let new_state = match &task_type {
                        TaskType::Research { topic } => CharacterState::Researching { topic: topic.clone() },
                        TaskType::Coding { project } => CharacterState::Coding { project: project.clone() },
                        TaskType::Analysis { subject } => CharacterState::Analyzing { subject: subject.clone() },
                    };

                    if let Some(_task_id) = sim_state.openclaw.spawn_task(id, task_type, current_tick).await {
                        let mut world = sim_state.world.write().await;
                        if let Some(character) = world.characters.get_mut(id) {
                            character.state = new_state;
                            character.destination = None;
                        }
                    }
                }

                sim_state.openclaw.poll_tasks(current_tick).await;

                for (id, state, _) in &char_states {
                    if state.is_openclaw_task() && !sim_state.openclaw.character_is_busy(id).await {
                        let mut world = sim_state.world.write().await;
                        if let Some(character) = world.characters.get_mut(id) {
                            if character.state.is_openclaw_task() {
                                character.state = CharacterState::Idle;
                            }
                        }
                        sim_state.openclaw.clear_completed_task(id).await;
                    }
                }
            }

            // Build task info map
            let task_info: CharacterTaskInfo = {
                let world = sim_state.world.read().await;
                let mut info = HashMap::new();
                for id in world.characters.keys() {
                    let ti = sim_state.openclaw.get_character_task_info(id).await;
                    info.insert(id.clone(), ti);
                }
                info
            };

            // Create current snapshot
            let curr_snapshot = {
                let world = sim_state.world.read().await;
                snapshot_world_with_tasks(&world, &task_info)
            };

            // Compute and broadcast delta (or full state if no previous)
            match &prev_snapshot {
                Some(prev) => {
                    let delta = compute_delta(prev, &curr_snapshot);
                    // Only send if something changed
                    if !delta.characters_changed.is_empty()
                        || !delta.characters_removed.is_empty()
                        || !delta.tiles_changed.is_empty()
                        || delta.time_of_day != prev.time_of_day
                    {
                        if let Ok(json) = serde_json::to_string(&delta) {
                            let _ = sim_state.delta_tx.send(json);
                        }
                    }
                }
                None => {
                    // First tick — no previous state, send nothing via delta
                    // (new clients get full_state on connect)
                }
            }

            prev_snapshot = Some(curr_snapshot);
        }
    });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/ws", get(websocket_handler))
        .route("/api/instances", get(api_instances))
        .route("/api/ws", get(websocket_handler))
        .nest_service("/city", ServeDir::new("static/miniworld").fallback(ServeFile::new("static/miniworld/index.html")))
        .nest_service("/dashboard", ServeDir::new("static/dashboard").fallback(ServeFile::new("static/dashboard/index.html")))
        .nest_service("/journals", ServeDir::new("static/journals").fallback(ServeFile::new("static/journals/index.html")))
        .nest_service("/sprites", ServeDir::new("static/miniworld/sprites"))
        .route("/install.sh", get(|| async {
            match tokio::fs::read_to_string("static/install.sh").await {
                Ok(script) => axum::response::Response::builder()
                    .header("content-type", "text/plain; charset=utf-8")
                    .body(axum::body::Body::from(script))
                    .unwrap(),
                Err(_) => axum::response::Response::builder()
                    .status(404)
                    .body(axum::body::Body::from("install.sh not found"))
                    .unwrap(),
            }
        }))
        .fallback_service(ServeDir::new("static").fallback(ServeFile::new("static/index.html")))
        .layer(cors)
        .with_state(state);

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));
    println!("🌐 Miniworld server running at http://localhost:{}", port);
    println!("   WebSocket at ws://localhost:{}/ws", port);
    println!("   Delta updates enabled — reduced bandwidth after initial connect");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
