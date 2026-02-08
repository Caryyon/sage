//! OpenClaw sub-agent bridge for SAGE characters
//!
//! Allows SAGE characters in the miniworld to spawn OpenClaw sub-agent tasks
//! for research, coding, and analysis work.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// OpenClaw gateway configuration
#[derive(Debug, Clone)]
pub struct OpenClawConfig {
    pub gateway_url: String,
    pub auth_token: String,
}

impl OpenClawConfig {
    /// Try to load config from ~/.openclaw/openclaw.json
    pub fn from_default_path() -> Option<Self> {
        let home = dirs::home_dir()?;
        let config_path = home.join(".openclaw").join("openclaw.json");
        let content = std::fs::read_to_string(&config_path).ok()?;
        let json: serde_json::Value = serde_json::from_str(&content).ok()?;

        let gateway = json.get("gateway")?;
        let port = gateway.get("port")?.as_u64()?;
        let token = gateway
            .get("auth")
            .and_then(|a| a.get("token"))
            .and_then(|t| t.as_str())
            .unwrap_or("")
            .to_string();

        Some(Self {
            gateway_url: format!("http://127.0.0.1:{}", port),
            auth_token: token,
        })
    }
}

/// Status of a spawned task
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

impl std::fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskStatus::Pending => write!(f, "pending"),
            TaskStatus::Running => write!(f, "running"),
            TaskStatus::Completed => write!(f, "completed"),
            TaskStatus::Failed => write!(f, "failed"),
        }
    }
}

/// A task spawned by a SAGE character via OpenClaw
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnedTask {
    /// Unique task ID (from OpenClaw session)
    pub task_id: String,
    /// Character who spawned it
    pub character_id: String,
    /// What kind of task
    pub task_type: TaskType,
    /// Human-readable description
    pub description: String,
    /// Current status
    pub status: TaskStatus,
    /// Result summary (when completed)
    pub result: Option<String>,
    /// When the task was created (tick number)
    pub created_at: u64,
    /// When the task was completed (tick number)
    pub completed_at: Option<u64>,
}

/// Types of tasks characters can spawn
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskType {
    Research { topic: String },
    Coding { project: String },
    Analysis { subject: String },
}

impl TaskType {
    pub fn description(&self) -> String {
        match self {
            TaskType::Research { topic } => format!("Researching: {}", topic),
            TaskType::Coding { project } => format!("Coding: {}", project),
            TaskType::Analysis { subject } => format!("Analyzing: {}", subject),
        }
    }

    /// Generate the prompt that would be sent to the OpenClaw sub-agent
    pub fn to_prompt(&self) -> String {
        match self {
            TaskType::Research { topic } => {
                format!(
                    "Research the following topic and provide a concise summary of key findings: {}",
                    topic
                )
            }
            TaskType::Coding { project } => {
                format!(
                    "Work on the following coding task and describe what you accomplished: {}",
                    project
                )
            }
            TaskType::Analysis { subject } => {
                format!(
                    "Analyze the following subject and provide insights: {}",
                    subject
                )
            }
        }
    }
}

/// The bridge between SAGE characters and OpenClaw gateway
#[derive(Clone)]
pub struct OpenClawBridge {
    config: Option<OpenClawConfig>,
    /// Active tasks mapped by task_id
    tasks: Arc<RwLock<HashMap<String, SpawnedTask>>>,
    /// Character -> their current task ID
    character_tasks: Arc<RwLock<HashMap<String, String>>>,
    /// Character -> last completed task result
    character_results: Arc<RwLock<HashMap<String, String>>>,
    /// Counter for generating task IDs when gateway is unavailable
    task_counter: Arc<RwLock<u64>>,
}

impl OpenClawBridge {
    pub fn new() -> Self {
        let config = OpenClawConfig::from_default_path();
        if let Some(ref c) = config {
            println!("🔗 OpenClaw bridge initialized (gateway at {})", c.gateway_url);
        } else {
            println!("⚠️  OpenClaw config not found — bridge running in simulation mode");
        }

        Self {
            config,
            tasks: Arc::new(RwLock::new(HashMap::new())),
            character_tasks: Arc::new(RwLock::new(HashMap::new())),
            character_results: Arc::new(RwLock::new(HashMap::new())),
            task_counter: Arc::new(RwLock::new(0)),
        }
    }

    /// Spawn a new sub-agent task for a character
    pub async fn spawn_task(
        &self,
        character_id: &str,
        task_type: TaskType,
        current_tick: u64,
    ) -> Option<String> {
        // Don't spawn if character already has an active task
        {
            let char_tasks = self.character_tasks.read().await;
            if let Some(existing_id) = char_tasks.get(character_id) {
                let tasks = self.tasks.read().await;
                if let Some(task) = tasks.get(existing_id) {
                    if task.status == TaskStatus::Pending || task.status == TaskStatus::Running {
                        return None; // Already busy
                    }
                }
            }
        }

        let description = task_type.description();
        let task_id = if let Some(ref config) = self.config {
            // Try to spawn via OpenClaw gateway API
            match self.spawn_via_gateway(config, &task_type).await {
                Some(id) => id,
                None => self.generate_local_task_id().await,
            }
        } else {
            self.generate_local_task_id().await
        };

        let task = SpawnedTask {
            task_id: task_id.clone(),
            character_id: character_id.to_string(),
            task_type,
            description,
            status: TaskStatus::Pending,
            result: None,
            created_at: current_tick,
            completed_at: None,
        };

        {
            let mut tasks = self.tasks.write().await;
            tasks.insert(task_id.clone(), task);
        }
        {
            let mut char_tasks = self.character_tasks.write().await;
            char_tasks.insert(character_id.to_string(), task_id.clone());
        }

        Some(task_id)
    }

    /// Attempt to spawn a sub-agent session via the OpenClaw gateway HTTP API
    async fn spawn_via_gateway(
        &self,
        config: &OpenClawConfig,
        task_type: &TaskType,
    ) -> Option<String> {
        let prompt = task_type.to_prompt();

        // TODO: The exact OpenClaw gateway API endpoint for spawning sub-agents
        // may need adjustment. This is based on the expected REST interface.
        // POST /api/v1/sessions with a JSON body containing the prompt.
        let url = format!("{}/api/v1/sessions", config.gateway_url);

        let body = serde_json::json!({
            "prompt": prompt,
            "label": format!("sage-{}", match task_type {
                TaskType::Research { .. } => "research",
                TaskType::Coding { .. } => "coding",
                TaskType::Analysis { .. } => "analysis",
            }),
        });

        let client = reqwest::Client::new();
        let resp = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", config.auth_token))
            .json(&body)
            .send()
            .await
            .ok()?;

        if !resp.status().is_success() {
            eprintln!(
                "OpenClaw gateway returned {}: {:?}",
                resp.status(),
                resp.text().await.ok()
            );
            return None;
        }

        let json: serde_json::Value = resp.json().await.ok()?;
        json.get("session_id")
            .or_else(|| json.get("id"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    }

    /// Check on a running task's status via the gateway
    async fn check_via_gateway(&self, config: &OpenClawConfig, task_id: &str) -> Option<(TaskStatus, Option<String>)> {
        let url = format!("{}/api/v1/sessions/{}", config.gateway_url, task_id);

        let client = reqwest::Client::new();
        let resp = client
            .get(&url)
            .header("Authorization", format!("Bearer {}", config.auth_token))
            .send()
            .await
            .ok()?;

        if !resp.status().is_success() {
            return None;
        }

        let json: serde_json::Value = resp.json().await.ok()?;
        let status_str = json.get("status").and_then(|v| v.as_str()).unwrap_or("unknown");
        let result = json.get("result").and_then(|v| v.as_str()).map(|s| s.to_string());

        let status = match status_str {
            "pending" => TaskStatus::Pending,
            "running" => TaskStatus::Running,
            "completed" | "done" => TaskStatus::Completed,
            "failed" | "error" => TaskStatus::Failed,
            _ => TaskStatus::Running,
        };

        Some((status, result))
    }

    /// Generate a local task ID for simulation mode
    async fn generate_local_task_id(&self) -> String {
        let mut counter = self.task_counter.write().await;
        *counter += 1;
        format!("sim-task-{}", *counter)
    }

    /// Poll and update all active tasks. Call periodically from the sim loop.
    pub async fn poll_tasks(&self, current_tick: u64) {
        let task_ids: Vec<String> = {
            let tasks = self.tasks.read().await;
            tasks
                .iter()
                .filter(|(_, t)| t.status == TaskStatus::Pending || t.status == TaskStatus::Running)
                .map(|(id, _)| id.clone())
                .collect()
        };

        for task_id in task_ids {
            let (new_status, result) = if let Some(ref config) = self.config {
                // Try gateway first
                if let Some((s, r)) = self.check_via_gateway(config, &task_id).await {
                    (s, r)
                } else {
                    // Fallback to simulation
                    self.simulate_task_progress(&task_id, current_tick).await
                }
            } else {
                // Pure simulation mode
                self.simulate_task_progress(&task_id, current_tick).await
            };

            let mut tasks = self.tasks.write().await;
            if let Some(task) = tasks.get_mut(&task_id) {
                task.status = new_status.clone();
                if let Some(ref r) = result {
                    task.result = Some(r.clone());
                }
                if new_status == TaskStatus::Completed || new_status == TaskStatus::Failed {
                    task.completed_at = Some(current_tick);
                    // Store result for character
                    if let Some(ref r) = result {
                        let mut results = self.character_results.write().await;
                        results.insert(task.character_id.clone(), r.clone());
                    }
                }
            }
        }
    }

    /// Simulate task progress when gateway is unavailable
    async fn simulate_task_progress(
        &self,
        task_id: &str,
        current_tick: u64,
    ) -> (TaskStatus, Option<String>) {
        let tasks = self.tasks.read().await;
        let Some(task) = tasks.get(task_id) else {
            return (TaskStatus::Failed, Some("Task not found".into()));
        };

        let age = current_tick.saturating_sub(task.created_at);

        // Simulate: pending for ~30 ticks, running for ~120 ticks, then complete
        if age < 30 {
            (TaskStatus::Pending, None)
        } else if age < 150 {
            (TaskStatus::Running, None)
        } else {
            let result = match &task.task_type {
                TaskType::Research { topic } => {
                    format!("Completed research on '{}'. Found 3 key insights and 5 relevant papers.", topic)
                }
                TaskType::Coding { project } => {
                    format!("Finished coding task '{}'. Implemented core logic with tests passing.", project)
                }
                TaskType::Analysis { subject } => {
                    format!("Analysis of '{}' complete. Identified 4 patterns and 2 anomalies.", subject)
                }
            };
            (TaskStatus::Completed, Some(result))
        }
    }

    /// Get the current task info for a character (for WebSocket snapshot)
    pub async fn get_character_task_info(
        &self,
        character_id: &str,
    ) -> (Option<String>, Option<String>, Option<String>) {
        let char_tasks = self.character_tasks.read().await;
        let tasks = self.tasks.read().await;
        let results = self.character_results.read().await;

        let (current_task, task_status) = if let Some(task_id) = char_tasks.get(character_id) {
            if let Some(task) = tasks.get(task_id) {
                if task.status == TaskStatus::Completed || task.status == TaskStatus::Failed {
                    (None, None)
                } else {
                    (Some(task.description.clone()), Some(task.status.to_string()))
                }
            } else {
                (None, None)
            }
        } else {
            (None, None)
        };

        let last_result = results.get(character_id).cloned();

        (current_task, task_status, last_result)
    }

    /// Check if a character has an active (non-completed) task
    pub async fn character_is_busy(&self, character_id: &str) -> bool {
        let char_tasks = self.character_tasks.read().await;
        let tasks = self.tasks.read().await;

        if let Some(task_id) = char_tasks.get(character_id) {
            if let Some(task) = tasks.get(task_id) {
                return task.status == TaskStatus::Pending || task.status == TaskStatus::Running;
            }
        }
        false
    }

    /// Clear completed/failed task from a character so they can take on new work
    pub async fn clear_completed_task(&self, character_id: &str) {
        let mut char_tasks = self.character_tasks.write().await;
        let tasks = self.tasks.read().await;

        if let Some(task_id) = char_tasks.get(character_id) {
            if let Some(task) = tasks.get(task_id) {
                if task.status == TaskStatus::Completed || task.status == TaskStatus::Failed {
                    char_tasks.remove(character_id);
                }
            }
        }
    }
}
