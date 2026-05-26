//! SAGE API — OpenAI-compatible HTTP server.
//!
//! Connects to a running sage-node via TCP and exposes endpoints that
//! any OpenAI-compatible client (Cursor, Continue, aider, etc.) can use.
//!
//! Usage: sage-api [--port 19175] [--api-port 19176]

use axum::{
    extract::State,
    http::StatusCode,
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse, Json,
    },
    routing::{get, post},
    Router,
};
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tower_http::cors::{Any, CorsLayer};

// ─── CLI ─────────────────────────────────────────────────────────────

#[derive(Parser)]
#[command(name = "sage-api", about = "OpenAI-compatible API server for SAGE")]
struct Cli {
    /// sage-node TCP port
    #[arg(short, long, default_value_t = 19175, env = "SAGE_PORT")]
    port: u16,

    /// HTTP API port
    #[arg(long, default_value_t = 19176, env = "SAGE_API_PORT")]
    api_port: u16,
}

// ─── Shared state ────────────────────────────────────────────────────

struct AppState {
    node_port: u16,
}

type SharedState = Arc<AppState>;

/// Send a command to sage-node and collect response lines until DONE.
fn node_request(port: u16, cmd: &str) -> Result<Vec<String>, String> {
    let mut stream = TcpStream::connect(format!("127.0.0.1:{}", port))
        .map_err(|e| format!("Cannot connect to sage-node on port {}: {}", port, e))?;
    stream
        .set_read_timeout(Some(std::time::Duration::from_secs(120)))
        .ok();
    stream
        .write_all(format!("{}\n", cmd).as_bytes())
        .map_err(|e| format!("Write error: {}", e))?;
    stream.flush().map_err(|e| format!("Flush error: {}", e))?;

    let reader = BufReader::new(stream);
    let mut lines = Vec::new();
    for line in reader.lines() {
        let line = line.map_err(|e| format!("Read error: {}", e))?;
        if line.trim() == "DONE" {
            break;
        }
        lines.push(line);
    }
    Ok(lines)
}

// node_chat_streaming removed — streaming done inline in handler

// ─── OpenAI request/response types ──────────────────────────────────

#[derive(Deserialize)]
struct ChatCompletionRequest {
    #[serde(default)]
    model: Option<String>,
    messages: Vec<MessageInput>,
    #[serde(default)]
    stream: Option<bool>,
    #[serde(default)]
    #[allow(dead_code)]
    temperature: Option<f64>,
    #[serde(default)]
    #[allow(dead_code)]
    max_tokens: Option<u32>,
}

#[derive(Deserialize)]
struct MessageInput {
    role: String,
    #[serde(deserialize_with = "deserialize_content")]
    content: String,
}

/// Accept content as either a plain string or an array of content parts
/// (OpenAI multi-modal format: [{"type": "text", "text": "..."}])
fn deserialize_content<'de, D: serde::Deserializer<'de>>(d: D) -> Result<String, D::Error> {
    use serde::de::{self, SeqAccess, Visitor};
    use std::fmt;

    struct ContentVisitor;

    impl<'de> Visitor<'de> for ContentVisitor {
        type Value = String;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a string or array of content parts")
        }

        fn visit_str<E: de::Error>(self, v: &str) -> Result<String, E> {
            Ok(v.to_string())
        }

        fn visit_string<E: de::Error>(self, v: String) -> Result<String, E> {
            Ok(v)
        }

        fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<String, A::Error> {
            #[derive(serde::Deserialize)]
            struct ContentPart {
                #[serde(rename = "type")]
                kind: String,
                text: Option<String>,
            }
            let mut parts = Vec::new();
            while let Some(part) = seq.next_element::<ContentPart>()? {
                if part.kind == "text" {
                    if let Some(text) = part.text {
                        parts.push(text);
                    }
                }
            }
            Ok(parts.join(" "))
        }
    }

    d.deserialize_any(ContentVisitor)
}

#[derive(Serialize)]
struct ChatCompletionResponse {
    id: String,
    object: String,
    created: u64,
    model: String,
    choices: Vec<Choice>,
    usage: Usage,
}

#[derive(Serialize)]
struct Choice {
    index: u32,
    message: MessageOutput,
    finish_reason: String,
}

#[derive(Serialize)]
struct MessageOutput {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct Usage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

#[derive(Serialize)]
struct StreamChunk {
    id: String,
    object: String,
    created: u64,
    model: String,
    choices: Vec<StreamChoice>,
}

#[derive(Serialize)]
struct StreamChoice {
    index: u32,
    delta: Delta,
    finish_reason: Option<String>,
}

#[derive(Serialize)]
struct Delta {
    #[serde(skip_serializing_if = "Option::is_none")]
    role: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<String>,
}

#[derive(Deserialize)]
struct EmbeddingRequest {
    input: EmbeddingInput,
    #[serde(default)]
    model: Option<String>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum EmbeddingInput {
    Single(String),
    Multiple(Vec<String>),
}

#[derive(Serialize)]
struct EmbeddingResponse {
    object: String,
    data: Vec<EmbeddingData>,
    model: String,
    usage: Usage,
}

#[derive(Serialize)]
struct EmbeddingData {
    object: String,
    embedding: Vec<f64>,
    index: usize,
}

#[derive(Deserialize)]
struct KnowledgeQuery {
    query: String,
    #[serde(default = "default_limit")]
    #[allow(dead_code)]
    limit: usize,
}

fn default_limit() -> usize {
    10
}

// ─── Helpers ─────────────────────────────────────────────────────────

fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn chat_id() -> String {
    format!("chatcmpl-sage-{:x}", now_unix())
}

/// Flatten messages into a single user message for sage-node CHAT protocol.
/// sage-node manages its own system prompt, so we just send the last user message
/// (or concatenate for multi-turn context).
fn flatten_messages(messages: &[MessageInput]) -> String {
    // If there's just one user message, send it directly.
    // For multi-turn, send last user message (sage-node keeps conversation per-connection).
    // For best results with single-connection-per-request, concatenate context.
    let user_msgs: Vec<&MessageInput> = messages.iter().filter(|m| m.role == "user").collect();
    if let Some(last) = user_msgs.last() {
        // If there's conversation history, provide it as context
        if messages.len() > 2 {
            let mut context = String::new();
            for msg in messages.iter() {
                if msg.role == "system" {
                    continue; // sage-node has its own system prompt
                }
                let prefix = match msg.role.as_str() {
                    "user" => "User",
                    "assistant" => "Assistant",
                    _ => &msg.role,
                };
                context.push_str(&format!("{}: {}\n", prefix, msg.content));
            }
            context.trim().to_string()
        } else {
            last.content.clone()
        }
    } else {
        messages
            .last()
            .map(|m| m.content.clone())
            .unwrap_or_default()
    }
}

// ─── Route handlers ──────────────────────────────────────────────────

async fn models_list() -> Json<serde_json::Value> {
    let now = now_unix();
    Json(serde_json::json!({
        "object": "list",
        "data": [
            {
                "id": "sage",
                "object": "model",
                "created": now,
                "owned_by": "sage-local",
                "permission": [],
                "root": "sage",
                "parent": null,
            },
            {
                "id": "sage-1.7b",
                "object": "model",
                "created": now,
                "owned_by": "sage-local",
                "permission": [],
                "root": "sage-1.7b",
                "parent": null,
            }
        ]
    }))
}

async fn chat_completions(
    State(state): State<SharedState>,
    Json(req): Json<ChatCompletionRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let message = flatten_messages(&req.messages);
    let model = req.model.as_deref().unwrap_or("sage").to_string();
    let stream = req.stream.unwrap_or(false);

    if stream {
        // SSE streaming response
        let (tx, rx) = mpsc::channel::<Result<Event, axum::Error>>(256);
        let port = state.node_port;
        let id = chat_id();
        let created = now_unix();
        let model_c = model.clone();

        tokio::task::spawn_blocking(move || {
            // Send initial role chunk
            let initial = StreamChunk {
                id: id.clone(),
                object: "chat.completion.chunk".into(),
                created,
                model: model_c.clone(),
                choices: vec![StreamChoice {
                    index: 0,
                    delta: Delta {
                        role: Some("assistant".into()),
                        content: None,
                    },
                    finish_reason: None,
                }],
            };
            let _ = tx.blocking_send(Ok(
                Event::default().data(serde_json::to_string(&initial).unwrap())
            ));

            // Do the TCP in this blocking thread directly
            let mut stream = match TcpStream::connect(format!("127.0.0.1:{}", port)) {
                Ok(s) => s,
                Err(e) => {
                    let _ = tx.blocking_send(Ok(
                        Event::default().data(format!(r#"{{"error":"{}"}}"#, e))
                    ));
                    return;
                }
            };
            stream
                .set_read_timeout(Some(std::time::Duration::from_secs(120)))
                .ok();
            let _ = stream.write_all(format!("CHAT {}\n", message).as_bytes());
            let _ = stream.flush();

            let reader = BufReader::new(stream);
            for line in reader.lines() {
                let line = match line {
                    Ok(l) => l,
                    Err(_) => break,
                };
                let trimmed = line.trim();
                if trimmed == "DONE" {
                    break;
                } else if let Some(token) = trimmed.strip_prefix("TOKEN ") {
                    let unescaped = token.replace("\\n", "\n");
                    let chunk = StreamChunk {
                        id: id.clone(),
                        object: "chat.completion.chunk".into(),
                        created,
                        model: model_c.clone(),
                        choices: vec![StreamChoice {
                            index: 0,
                            delta: Delta {
                                role: None,
                                content: Some(unescaped),
                            },
                            finish_reason: None,
                        }],
                    };
                    if tx
                        .blocking_send(Ok(
                            Event::default().data(serde_json::to_string(&chunk).unwrap())
                        ))
                        .is_err()
                    {
                        break;
                    }
                } else if let Some(_err) = trimmed.strip_prefix("ERROR ") {
                    break;
                }
            }

            // Send final chunk with finish_reason
            let final_chunk = StreamChunk {
                id: id.clone(),
                object: "chat.completion.chunk".into(),
                created,
                model: model_c,
                choices: vec![StreamChoice {
                    index: 0,
                    delta: Delta {
                        role: None,
                        content: None,
                    },
                    finish_reason: Some("stop".into()),
                }],
            };
            let _ = tx.blocking_send(Ok(
                Event::default().data(serde_json::to_string(&final_chunk).unwrap())
            ));
            let _ = tx.blocking_send(Ok(Event::default().data("[DONE]")));
        });

        let stream = ReceiverStream::new(rx);
        Ok(Sse::new(stream)
            .keep_alive(KeepAlive::default())
            .into_response())
    } else {
        // Non-streaming: collect full response
        let port = state.node_port;
        let result = tokio::task::spawn_blocking(move || {
            let mut stream = TcpStream::connect(format!("127.0.0.1:{}", port))
                .map_err(|e| format!("Cannot connect to sage-node: {}", e))?;
            stream
                .set_read_timeout(Some(std::time::Duration::from_secs(120)))
                .ok();
            stream
                .write_all(format!("CHAT {}\n", message).as_bytes())
                .map_err(|e| format!("Write: {}", e))?;
            stream.flush().map_err(|e| format!("Flush: {}", e))?;

            let reader = BufReader::new(stream);
            let mut full = String::new();
            for line in reader.lines() {
                let line = line.map_err(|e| format!("Read: {}", e))?;
                let trimmed = line.trim();
                if trimmed == "DONE" {
                    break;
                } else if let Some(token) = trimmed.strip_prefix("TOKEN ") {
                    full.push_str(&token.replace("\\n", "\n"));
                } else if let Some(err) = trimmed.strip_prefix("ERROR ") {
                    return Err(err.to_string());
                }
            }
            Ok::<String, String>(full)
        })
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": {"message": e.to_string()}})),
            )
        })?
        .map_err(|e| {
            (
                StatusCode::BAD_GATEWAY,
                Json(serde_json::json!({"error": {"message": e}})),
            )
        })?;

        let prompt_tokens = req
            .messages
            .iter()
            .map(|m| m.content.len() / 4)
            .sum::<usize>() as u32;
        let completion_tokens = (result.len() / 4) as u32;

        Ok(Json(ChatCompletionResponse {
            id: chat_id(),
            object: "chat.completion".into(),
            created: now_unix(),
            model,
            choices: vec![Choice {
                index: 0,
                message: MessageOutput {
                    role: "assistant".into(),
                    content: result,
                },
                finish_reason: "stop".into(),
            }],
            usage: Usage {
                prompt_tokens,
                completion_tokens,
                total_tokens: prompt_tokens + completion_tokens,
            },
        })
        .into_response())
    }
}

async fn embeddings(
    State(_state): State<SharedState>,
    Json(req): Json<EmbeddingRequest>,
) -> Result<Json<EmbeddingResponse>, (StatusCode, Json<serde_json::Value>)> {
    // Use the local embedding engine
    let texts = match req.input {
        EmbeddingInput::Single(s) => vec![s],
        EmbeddingInput::Multiple(v) => v,
    };

    let data: Vec<EmbeddingData> = texts
        .iter()
        .enumerate()
        .map(|(i, text)| {
            let engine = sage::inference::embeddings::EmbeddingEngine::new();
            let embedding = engine.embed(text).unwrap_or_else(|_| vec![0.0; 384]);
            EmbeddingData {
                object: "embedding".into(),
                embedding,
                index: i,
            }
        })
        .collect();

    let total_tokens = texts.iter().map(|t| t.len() / 4).sum::<usize>() as u32;

    Ok(Json(EmbeddingResponse {
        object: "list".into(),
        data,
        model: req.model.unwrap_or_else(|| "sage".into()),
        usage: Usage {
            prompt_tokens: total_tokens,
            completion_tokens: 0,
            total_tokens,
        },
    }))
}

async fn sage_status(
    State(state): State<SharedState>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let port = state.node_port;
    let lines = tokio::task::spawn_blocking(move || node_request(port, "STATUS"))
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
        })?
        .map_err(|e| {
            (
                StatusCode::BAD_GATEWAY,
                Json(serde_json::json!({"error": e})),
            )
        })?;

    // First line should be JSON status
    if let Some(first) = lines.first() {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(first) {
            return Ok(Json(v));
        }
    }
    Ok(Json(serde_json::json!({"status": "unknown", "raw": lines})))
}

async fn sage_peers(
    State(state): State<SharedState>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let port = state.node_port;
    let lines = tokio::task::spawn_blocking(move || node_request(port, "PEERS"))
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
        })?
        .map_err(|e| {
            (
                StatusCode::BAD_GATEWAY,
                Json(serde_json::json!({"error": e})),
            )
        })?;

    let peers: Vec<&str> = lines
        .iter()
        .filter_map(|l| l.strip_prefix("PEER "))
        .collect();

    Ok(Json(serde_json::json!({
        "peers": peers,
        "count": peers.len(),
    })))
}

async fn sage_knowledge(
    State(state): State<SharedState>,
    Json(req): Json<KnowledgeQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let port = state.node_port;
    let query = req.query;
    let lines =
        tokio::task::spawn_blocking(move || node_request(port, &format!("KNOWLEDGE {}", query)))
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"error": e.to_string()})),
                )
            })?
            .map_err(|e| {
                (
                    StatusCode::BAD_GATEWAY,
                    Json(serde_json::json!({"error": e})),
                )
            })?;

    Ok(Json(serde_json::json!({
        "results": lines,
    })))
}

async fn sage_brain(
    State(state): State<SharedState>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let port = state.node_port;
    let lines = tokio::task::spawn_blocking(move || node_request(port, "BRAIN"))
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
        })?
        .map_err(|e| {
            (
                StatusCode::BAD_GATEWAY,
                Json(serde_json::json!({"error": e})),
            )
        })?;

    // Parse "ROW y: v0 v1 v2 ..." lines into a 2D grid
    let mut grid: Vec<Vec<f64>> = Vec::new();
    let mut width = 0usize;
    for line in &lines {
        if let Some(rest) = line.strip_prefix("ROW ") {
            if let Some(colon) = rest.find(':') {
                let vals: Vec<f64> = rest[colon + 1..]
                    .split_whitespace()
                    .filter_map(|v| v.parse().ok())
                    .collect();
                if !vals.is_empty() {
                    width = width.max(vals.len());
                    grid.push(vals);
                }
            }
        }
    }

    // Normalize to [0, 1] for visualization
    let flat: Vec<f64> = grid.iter().flatten().copied().collect();
    let max_val = flat.iter().copied().fold(0.0f64, |a, b| a.max(b));
    let min_val = flat.iter().copied().fold(1.0f64, |a, b| a.min(b));
    let range = (max_val - min_val).max(1e-6);

    let normalized: Vec<Vec<f64>> = grid
        .iter()
        .map(|row| row.iter().map(|v| (v - min_val) / range).collect())
        .collect();

    Ok(Json(serde_json::json!({
        "grid": normalized,
        "raw": lines,
        "dimensions": { "rows": grid.len(), "cols": width },
    })))
}

async fn health() -> &'static str {
    "ok"
}

// ─── Main ────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let state = Arc::new(AppState {
        node_port: cli.port,
    });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/health", get(health))
        .route("/v1/models", get(models_list))
        .route("/v1/chat/completions", post(chat_completions))
        .route("/v1/embeddings", post(embeddings))
        .route("/v1/sage/status", get(sage_status))
        .route("/v1/sage/peers", get(sage_peers))
        .route("/v1/sage/knowledge", post(sage_knowledge))
        .route("/v1/sage/brain", get(sage_brain))
        .layer(cors)
        .with_state(state);

    let addr = format!("0.0.0.0:{}", cli.api_port);
    println!("🌐 SAGE API server starting on {}", addr);
    println!("   Node port:  {}", cli.port);
    println!("   API port:   {}", cli.api_port);
    println!();
    println!("   OpenAI-compatible endpoint:");
    println!(
        "   POST http://localhost:{}/v1/chat/completions",
        cli.api_port
    );
    println!();
    println!("   Use with any OpenAI client:");
    println!("   OPENAI_API_BASE=http://localhost:{}/v1", cli.api_port);
    println!();

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind API port");

    axum::serve(listener, app).await.unwrap();
}
