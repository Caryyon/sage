//! Autonomous Specialist Worker
//!
//! The engine that turns a SpecialistProfile into a working employee.
//! Wraps the NCA brain, inference engine, and specialist prompt into a
//! self-contained loop that accepts tasks, retrieves knowledge, executes,
//! and reports results — all without human intervention.
//!
//! Architecture:
//!   Task arrives → Worker retrieves NCA knowledge → builds augmented prompt
//!   → runs inference → encodes results back into brain → reports completion
//!
//! The worker runs continuously: it polls for new tasks, processes them
//! one at a time (or up to max_concurrent), and grows smarter with each
//! completed task as results are encoded back into the NCA grid.

use crate::distributed_knowledge::default_brain_path;
use crate::inference::{ChatMessage, ChatRole, InferenceEngine};
use crate::knowledge_loop::KnowledgeLoop;
use crate::specialist::SpecialistProfile;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Task priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TaskPriority {
    Low,
    Normal,
    High,
    Urgent,
}

impl TaskPriority {
    pub fn label(&self) -> &str {
        match self {
            TaskPriority::Low => "low",
            TaskPriority::Normal => "normal",
            TaskPriority::High => "high",
            TaskPriority::Urgent => "urgent",
        }
    }
}

/// The lifecycle of a task through the worker
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskState {
    /// Waiting in queue
    Queued,
    /// Worker is retrieving relevant NCA knowledge
    Retrieving,
    /// Worker is planning the approach
    Planning,
    /// Worker is executing via inference engine
    Executing,
    /// Worker is validating output against requirements
    Validating,
    /// Task completed successfully
    Completed,
    /// Task failed (with reason)
    Failed,
}

impl TaskState {
    pub fn label(&self) -> &str {
        match self {
            TaskState::Queued => "queued",
            TaskState::Retrieving => "retrieving",
            TaskState::Planning => "planning",
            TaskState::Executing => "executing",
            TaskState::Validating => "validating",
            TaskState::Completed => "completed",
            TaskState::Failed => "failed",
        }
    }

    pub fn is_terminal(&self) -> bool {
        matches!(self, TaskState::Completed | TaskState::Failed)
    }

    pub fn is_active(&self) -> bool {
        matches!(
            self,
            TaskState::Retrieving | TaskState::Planning | TaskState::Executing | TaskState::Validating
        )
    }
}

/// A single work task assigned to the specialist
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerTask {
    /// Unique task ID
    pub id: String,
    /// The task description / requirements
    pub description: String,
    /// Which capability this maps to (if any)
    pub capability: Option<String>,
    /// Priority level
    pub priority: TaskPriority,
    /// Current state
    pub state: TaskState,
    /// The final output / deliverable
    pub result: Option<String>,
    /// Error message if failed
    pub error: Option<String>,
    /// When the task was created (unix timestamp)
    pub created_at: u64,
    /// When the task entered each state (unix timestamps)
    pub state_timestamps: Vec<(TaskState, u64)>,
    /// Number of inference tokens used
    pub tokens_used: usize,
    /// Quality self-assessment (0.0-1.0)
    pub self_assessed_quality: Option<f64>,
}

impl WorkerTask {
    pub fn new(id: String, description: String, capability: Option<String>, priority: TaskPriority) -> Self {
        Self {
            id,
            description,
            capability,
            priority,
            state: TaskState::Queued,
            result: None,
            error: None,
            created_at: now_ms(),
            state_timestamps: vec![(TaskState::Queued, now_ms())],
            tokens_used: 0,
            self_assessed_quality: None,
        }
    }

    pub fn transition(&mut self, new_state: TaskState) {
        self.state = new_state;
        self.state_timestamps.push((new_state, now_ms()));
    }

    pub fn elapsed(&self) -> Duration {
        let now = now_ms();
        Duration::from_millis(now.saturating_sub(self.created_at))
    }

    pub fn elapsed_in_state(&self, state: TaskState) -> Option<Duration> {
        self.state_timestamps
            .iter()
            .find(|(s, _)| *s == state)
            .map(|(_, t)| {
                let now = now_ms();
                Duration::from_millis(now.saturating_sub(*t))
            })
    }
}

/// Statistics about the worker's performance
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WorkerStats {
    pub tasks_completed: u64,
    pub tasks_failed: u64,
    pub total_tokens_used: u64,
    pub avg_completion_secs: f64,
    pub avg_quality: f64,
    pub active_cells: usize,
    pub brain_saves: u64,
    pub uptime_secs: u64,
}

/// Configuration for the worker loop
#[derive(Debug, Clone)]
pub struct WorkerConfig {
    /// Max tokens per inference call
    pub max_tokens: usize,
    /// Brain auto-save interval in seconds
    pub autosave_interval_secs: u64,
    /// Consolidation steps after each task
    pub consolidation_steps: usize,
    /// Whether to encode task results back into the brain
    pub encode_results: bool,
    /// Minimum confidence for encoding results
    pub encode_confidence: f64,
}

impl Default for WorkerConfig {
    fn default() -> Self {
        Self {
            max_tokens: 2000,
            autosave_interval_secs: 300,
            consolidation_steps: 2,
            encode_results: true,
            encode_confidence: 0.7,
        }
    }
}

/// The autonomous specialist worker
pub struct SpecialistWorker {
    /// The specialist profile (role, capabilities, prompt, hiring info)
    pub profile: SpecialistProfile,
    /// The knowledge loop (NCA brain + retrieval)
    pub knowledge: Arc<Mutex<KnowledgeLoop>>,
    /// The inference engine
    pub engine: Arc<dyn InferenceEngine>,
    /// Task queue (pending tasks)
    pub task_queue: Arc<Mutex<Vec<WorkerTask>>>,
    /// Completed tasks history
    pub task_history: Arc<Mutex<Vec<WorkerTask>>>,
    /// Worker statistics
    pub stats: Arc<Mutex<WorkerStats>>,
    /// Configuration
    pub config: WorkerConfig,
    /// Stop signal
    stop: Arc<AtomicBool>,
    /// Brain path
    brain_path: String,
    /// Start time
    start_time: Instant,
}

impl SpecialistWorker {
    /// Create a new worker from a specialist profile
    pub fn new(
        profile: SpecialistProfile,
        engine: Arc<dyn InferenceEngine>,
        brain_path: Option<String>,
        config: Option<WorkerConfig>,
    ) -> Self {
        let bp = brain_path.unwrap_or_else(default_brain_path);
        let cfg = config.unwrap_or_default();

        let mut kloop = KnowledgeLoop::new(Arc::clone(&engine)).with_brain_path(&bp);
        if std::path::Path::new(&bp).exists() {
            let _ = kloop.load_brain();
        }

        Self {
            profile,
            knowledge: Arc::new(Mutex::new(kloop)),
            engine,
            task_queue: Arc::new(Mutex::new(Vec::new())),
            task_history: Arc::new(Mutex::new(Vec::new())),
            stats: Arc::new(Mutex::new(WorkerStats::default())),
            config: cfg,
            stop: Arc::new(AtomicBool::new(false)),
            brain_path: bp,
            start_time: Instant::now(),
        }
    }

    /// Submit a task to the worker's queue
    pub fn submit_task(&self, description: &str, capability: Option<&str>, priority: TaskPriority) -> String {
        let id = format!("task-{}", uuid_simple());
        let task = WorkerTask::new(
            id.clone(),
            description.to_string(),
            capability.map(|s| s.to_string()),
            priority,
        );

        let mut queue = self.task_queue.lock().unwrap();
        queue.push(task);
        // Sort by priority (highest first)
        queue.sort_by(|a, b| b.priority.cmp(&a.priority));

        id
    }

    /// Get the current queue status
    pub fn queue_status(&self) -> Vec<WorkerTask> {
        self.task_queue.lock().unwrap().clone()
    }

    /// Get completed task history
    pub fn completed_tasks(&self) -> Vec<WorkerTask> {
        self.task_history.lock().unwrap().clone()
    }

    /// Get current worker stats
    pub fn current_stats(&self) -> WorkerStats {
        let mut s = self.stats.lock().unwrap().clone();
        s.uptime_secs = self.start_time.elapsed().as_secs();
        if let Ok(k) = self.knowledge.lock() {
            s.active_cells = k.active_cells();
        }
        s
    }

    /// Start the autonomous worker loop. Blocks until stop() is called.
    pub fn run(&self) {
        let max_concurrent = self.profile.hiring.max_concurrent_tasks;
        let mut last_save = Instant::now();

        println!("🧠 Specialist Worker started: {} ({} {})",
            self.profile.display_name,
            self.profile.role.level.label(),
            self.profile.role.title,
        );
        println!("   Brain: {} | Max concurrent tasks: {} | Rate: ${}/hr",
            self.brain_path,
            max_concurrent,
            self.profile.hiring.suggested_rate_usd,
        );
        println!("   {} capabilities registered", self.profile.capabilities.len());
        println!();

        loop {
            if self.stop.load(Ordering::Relaxed) {
                break;
            }

            // Check for new tasks
            let mut active_count;
            {
                let mut queue = self.task_queue.lock().unwrap();
                active_count = queue.iter().filter(|t| t.state.is_active()).count();

                // Start new tasks if we have capacity
                while active_count < max_concurrent {
                    if let Some(pos) = queue.iter().position(|t| t.state == TaskState::Queued) {
                        queue[pos].transition(TaskState::Retrieving);
                        active_count += 1;
                    } else {
                        break;
                    }
                }
            }

            // Process active tasks
            let active_tasks: Vec<WorkerTask> = {
                let queue = self.task_queue.lock().unwrap();
                queue.iter().filter(|t| t.state.is_active()).cloned().collect()
            };

            for mut task in active_tasks {
                let result = self.process_task_step(&mut task);

                match result {
                    TaskStepResult::Continue => {
                        // Update task in queue
                        let mut queue = self.task_queue.lock().unwrap();
                        if let Some(t) = queue.iter_mut().find(|t| t.id == task.id) {
                            *t = task;
                        }
                    }
                    TaskStepResult::Completed(output, quality) => {
                        self.complete_task(task, Ok(output), Some(quality));
                        active_count = active_count.saturating_sub(1);
                    }
                    TaskStepResult::Failed(error) => {
                        self.complete_task(task, Err(error), None);
                        active_count = active_count.saturating_sub(1);
                    }
                }
            }

            // Auto-save brain periodically
            if last_save.elapsed().as_secs() >= self.config.autosave_interval_secs {
                if let Ok(k) = self.knowledge.lock() {
                    if k.save_brain().is_ok() {
                        let mut s = self.stats.lock().unwrap();
                        s.brain_saves += 1;
                        s.active_cells = k.active_cells();
                    }
                }
                last_save = Instant::now();
            }

            // Sleep to avoid busy-waiting
            std::thread::sleep(Duration::from_millis(500));
        }

        // Final save on shutdown
        if let Ok(k) = self.knowledge.lock() {
            let _ = k.save_brain();
        }
        println!("👋 Worker shut down. {} tasks completed, {} failed.",
            self.stats.lock().unwrap().tasks_completed,
            self.stats.lock().unwrap().tasks_failed,
        );
    }

    /// Process one step of a task's lifecycle. Returns the next action.
    fn process_task_step(&self, task: &mut WorkerTask) -> TaskStepResult {
        match task.state {
            TaskState::Retrieving => {
                // Validate capability match
                if let Some(ref cap_name) = task.capability {
                    let cap = self.profile.capabilities.iter().find(|c| &c.name == cap_name);
                    if cap.is_none() {
                        task.error = Some(format!(
                            "Capability '{}' not in specialist profile. Available: {}",
                            cap_name,
                            self.profile.capabilities.iter().map(|c| c.name.as_str()).collect::<Vec<_>>().join(", ")
                        ));
                        task.transition(TaskState::Failed);
                        return TaskStepResult::Failed(task.error.clone().unwrap());
                    }
                }

                task.transition(TaskState::Executing);
                TaskStepResult::Continue
            }

            TaskState::Executing => {
                // Run inference with the augmented prompt
                let knowledge_context = {
                    let mut k = self.knowledge.lock().unwrap();
                    k.retrieve_knowledge(&task.description)
                };

                let messages = self.build_messages(task, knowledge_context.as_deref());

                match self.engine.chat(&messages, self.config.max_tokens) {
                    Ok(response) => {
                        task.result = Some(response.clone());
                        task.tokens_used = response.split_whitespace().count();
                        task.transition(TaskState::Validating);
                        TaskStepResult::Continue
                    }
                    Err(e) => {
                        task.error = Some(format!("Inference error: {}", e));
                        task.transition(TaskState::Failed);
                        TaskStepResult::Failed(task.error.clone().unwrap())
                    }
                }
            }

            TaskState::Validating => {
                // Self-assess quality
                let quality = self.self_assess(task);
                task.self_assessed_quality = Some(quality);

                let threshold = if let Some(ref cap_name) = task.capability {
                    self.profile.capabilities.iter()
                        .find(|c| &c.name == cap_name)
                        .map(|c| c.quality_threshold)
                        .unwrap_or(0.5)
                } else {
                    0.5
                };

                if quality >= threshold {
                    // Encode result back into brain
                    if self.config.encode_results {
                        if let Some(ref result) = task.result {
                            let mut k = self.knowledge.lock().unwrap();
                            k.encode(result, self.config.encode_confidence);
                            // Run consolidation
                            k.knowledge_mut().grid.consolidate_knowledge(self.config.consolidation_steps);
                        }
                    }

                    task.transition(TaskState::Completed);
                    TaskStepResult::Completed(
                        task.result.clone().unwrap_or_default(),
                        quality,
                    )
                } else {
                    task.error = Some(format!(
                        "Self-assessed quality {:.2} below threshold {:.2}",
                        quality, threshold
                    ));
                    task.transition(TaskState::Failed);
                    TaskStepResult::Failed(task.error.clone().unwrap())
                }
            }

            _ => TaskStepResult::Continue,
        }
    }

    /// Build the chat messages for inference, augmented with NCA knowledge
    fn build_messages(&self, task: &WorkerTask, knowledge_context: Option<&str>) -> Vec<ChatMessage> {
        let system_prompt = if let Some(ctx) = knowledge_context {
            format!(
                "{}\n\n## Recalled Knowledge from NCA Brain\n{}",
                self.profile.prompt.assemble(),
                ctx
            )
        } else {
            self.profile.prompt.assemble()
        };

        let task_prompt = if let Some(ref cap_name) = task.capability {
            format!(
                "Task (capability: {}): {}\n\nExecute this task following your task instructions. \
                 Deliver the complete result. Do not ask clarifying questions unless absolutely blocked.",
                cap_name, task.description
            )
        } else {
            format!(
                "Task: {}\n\nExecute this task following your task instructions. \
                 Deliver the complete result. Do not ask clarifying questions unless absolutely blocked.",
                task.description
            )
        };

        vec![
            ChatMessage {
                role: ChatRole::System,
                content: system_prompt,
            },
            ChatMessage {
                role: ChatRole::User,
                content: task_prompt,
            },
        ]
    }

    /// Self-assess the quality of a completed task (0.0-1.0)
    fn self_assess(&self, task: &WorkerTask) -> f64 {
        let result = match task.result {
            Some(ref r) => r,
            None => return 0.0,
        };

        let mut score = 0.5; // Start neutral

        // Length heuristic: very short responses likely incomplete
        if result.len() < 50 {
            score -= 0.2;
        } else if result.len() > 500 {
            score += 0.1;
        }

        // Structure heuristic: look for organized output
        let has_sections = result.contains("##") || result.contains("```") || result.contains("1.");
        if has_sections {
            score += 0.15;
        }

        // Completeness heuristic: check if task description keywords appear in result
        let task_words: Vec<&str> = task.description.split_whitespace().collect();
        let result_lower = result.to_lowercase();
        let mut keyword_hits = 0;
        for word in &task_words {
            if word.len() > 3 && result_lower.contains(&word.to_lowercase()) {
                keyword_hits += 1;
            }
        }
        if !task_words.is_empty() {
            let hit_ratio = keyword_hits as f64 / task_words.len() as f64;
            score += hit_ratio * 0.15;
        }

        // Clamp
        score.clamp(0.0, 1.0)
    }

    /// Mark a task as completed or failed, move to history, update stats
    fn complete_task(&self, task: WorkerTask, result: Result<String, String>, quality: Option<f64>) {
        let mut queue = self.task_queue.lock().unwrap();
        if let Some(pos) = queue.iter().position(|t| t.id == task.id) {
            let mut completed = queue.remove(pos);
            completed.result = result.as_ref().ok().cloned();
            completed.error = result.as_ref().err().cloned();
            completed.self_assessed_quality = quality;

            let mut history = self.task_history.lock().unwrap();
            let mut stats = self.stats.lock().unwrap();

            match result {
                Ok(_) => {
                    stats.tasks_completed += 1;
                    if let Some(q) = quality {
                        let n = stats.tasks_completed as f64;
                        stats.avg_quality = (stats.avg_quality * (n - 1.0) + q) / n;
                    }
                    println!("✅ Task {} completed ({:.1}s, quality: {:.2})",
                        completed.id,
                        completed.elapsed().as_secs_f64(),
                        quality.unwrap_or(0.0),
                    );
                }
                Err(ref e) => {
                    stats.tasks_failed += 1;
                    println!("❌ Task {} failed: {}", completed.id, e);
                }
            }

            stats.total_tokens_used += completed.tokens_used as u64;
            if stats.tasks_completed > 0 {
                stats.avg_completion_secs = history.iter()
                    .filter(|t| t.state == TaskState::Completed)
                    .map(|t| t.elapsed().as_secs_f64())
                    .sum::<f64>() / stats.tasks_completed as f64;
            }

            history.push(completed);
        }
    }

    /// Signal the worker to stop
    pub fn stop(&self) {
        self.stop.store(true, Ordering::Relaxed);
    }

    /// Check if the worker is running
    pub fn is_running(&self) -> bool {
        !self.stop.load(Ordering::Relaxed)
    }
}

/// Result of processing one task step
enum TaskStepResult {
    /// Task needs more processing (state advanced, continue loop)
    Continue,
    /// Task completed with output and quality score
    Completed(String, f64),
    /// Task failed with error message
    Failed(String),
}

/// Generate a simple unique ID (not cryptographically secure, just for task tracking)
fn uuid_simple() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("{:x}", now)
}

/// Current time in milliseconds since epoch
fn now_ms() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inference::{ChatMessage, ChatRole, InferenceEngine};
    use crate::specialist::presets;
    use std::error::Error;
    use std::sync::Mutex;

    /// Mock engine that returns predefined responses
    struct MockEngine {
        responses: Mutex<Vec<String>>,
        call_count: Mutex<usize>,
    }

    impl MockEngine {
        fn new(responses: Vec<String>) -> Self {
            Self {
                responses: Mutex::new(responses),
                call_count: Mutex::new(0),
            }
        }
    }

    impl InferenceEngine for MockEngine {
        fn generate(&self, _prompt: &str, _max_tokens: usize) -> Result<String, Box<dyn Error>> {
            let mut count = self.call_count.lock().unwrap();
            let responses = self.responses.lock().unwrap();
            let idx = *count % responses.len();
            *count += 1;
            Ok(responses[idx].clone())
        }

        fn chat(&self, _messages: &[ChatMessage], _max_tokens: usize) -> Result<String, Box<dyn Error>> {
            self.generate("", 0)
        }

        fn generate_streaming(
            &self,
            _prompt: &str,
            _max_tokens: usize,
            mut cb: Box<dyn FnMut(&str) + Send>,
        ) -> Result<(), Box<dyn Error>> {
            let resp = self.generate("", 0)?;
            cb(&resp);
            Ok(())
        }

        fn chat_streaming(
            &self,
            messages: &[ChatMessage],
            max_tokens: usize,
            mut cb: Box<dyn FnMut(&str) + Send>,
        ) -> Result<(), Box<dyn Error>> {
            let _ = self.chat(messages, max_tokens)?;
            cb("ok");
            Ok(())
        }

        fn name(&self) -> &str { "mock" }
        fn is_available(&self) -> bool { true }
    }

    fn make_test_profile() -> SpecialistProfile {
        let role = presets::junior_react_developer();
        SpecialistProfile {
            name: "test-worker".to_string(),
            display_name: "Test Worker".to_string(),
            tagline: "test".to_string(),
            description: "test".to_string(),
            version: "0.1.0".to_string(),
            role: role.clone(),
            capabilities: presets::default_capabilities(&role),
            quality: crate::specialist::QualityMetrics {
                hit_rate: 0.8,
                mean_relevance: 0.7,
                topics_verified: 5,
                facts_encoded: 100,
                active_cells: 500,
                grid_utilization: 0.01,
                topic_hit_rates: vec![],
            },
            prompt: presets::default_prompt(&role),
            hiring: presets::default_hiring(&role),
            template_name: "test".to_string(),
            created_at: 0,
            author_node_id: "test".to_string(),
            tags: vec![],
        }
    }

    #[test]
    fn test_task_lifecycle() {
        let mut task = WorkerTask::new(
            "task-1".to_string(),
            "Build a login form".to_string(),
            Some("component-development".to_string()),
            TaskPriority::Normal,
        );

        assert_eq!(task.state, TaskState::Queued);
        assert!(task.result.is_none());

        task.transition(TaskState::Retrieving);
        assert_eq!(task.state, TaskState::Retrieving);

        task.transition(TaskState::Executing);
        task.result = Some("<form>...</form>".to_string());
        assert!(task.result.is_some());

        task.transition(TaskState::Completed);
        assert!(task.state.is_terminal());
    }

    #[test]
    fn test_worker_submit_and_process() {
        let profile = make_test_profile();
        let engine = Arc::new(MockEngine::new(vec![
            "Here is the login form component:\n\n```tsx\nconst LoginForm = () => {\n  return <form>...</form>;\n};\n```\n\n## Summary\nBuilt a React login form with email/password fields and validation.".to_string(),
        ]));

        let worker = SpecialistWorker::new(
            profile,
            engine,
            Some("/tmp/sage_test_worker.bin".to_string()),
            Some(WorkerConfig {
                max_tokens: 500,
                autosave_interval_secs: 3600,
                consolidation_steps: 1,
                encode_results: false,
                encode_confidence: 0.7,
            }),
        );

        // Clean up any previous test brain
        let _ = std::fs::remove_file("/tmp/sage_test_worker.bin");

        let task_id = worker.submit_task("Build a login form component", Some("component-development"), TaskPriority::Normal);
        assert!(task_id.starts_with("task-"));

        // Process the task manually (one step at a time, since run() blocks)
        let mut queue = worker.task_queue.lock().unwrap();
        assert_eq!(queue.len(), 1);
        assert_eq!(queue[0].state, TaskState::Queued);
        drop(queue);

        // Simulate what run() does: start the queued task
        {
            let mut queue = worker.task_queue.lock().unwrap();
            queue[0].transition(TaskState::Retrieving);
        }

        // Process through the steps
        let mut task = worker.task_queue.lock().unwrap().remove(0);
        let result = worker.process_task_step(&mut task);
        assert!(matches!(result, TaskStepResult::Continue));
        assert_eq!(task.state, TaskState::Executing);

        let result = worker.process_task_step(&mut task);
        assert!(matches!(result, TaskStepResult::Continue));
        assert_eq!(task.state, TaskState::Validating);
        assert!(task.result.is_some());

        let result = worker.process_task_step(&mut task);
        match result {
            TaskStepResult::Completed(output, quality) => {
                assert!(output.contains("LoginForm"));
                assert!(quality > 0.5);
            }
            _ => panic!("Expected Completed, got something else"),
        }

        // Clean up
        let _ = std::fs::remove_file("/tmp/sage_test_worker.bin");
    }

    #[test]
    fn test_self_assessment() {
        let profile = make_test_profile();
        let engine = Arc::new(MockEngine::new(vec!["ok".to_string()]));
        let worker = SpecialistWorker::new(profile, engine, None, None);

        let task = WorkerTask::new(
            "task-1".to_string(),
            "Build a React login form component with email and password validation".to_string(),
            Some("component-development".to_string()),
            TaskPriority::Normal,
        );

        // Good response
        let mut good_task = task.clone();
        good_task.result = Some(
            "## Login Form Component\n\n```tsx\nconst LoginForm = () => {\n  const [email, setEmail] = useState('');\n  const [password, setPassword] = useState('');\n  \n  const handleSubmit = (e) => {\n    e.preventDefault();\n    // validate email and password\n  };\n  \n  return (\n    <form onSubmit={handleSubmit}>\n      <input type=\"email\" value={email} />\n      <input type=\"password\" value={password} />\n      <button type=\"submit\">Login</button>\n    </form>\n  );\n};\n```\n\n## Summary\nComplete login form with validation.".to_string(),
        );
        let good_score = worker.self_assess(&good_task);
        assert!(good_score > 0.6, "Good response should score high, got {}", good_score);

        // Bad response
        let mut bad_task = task.clone();
        bad_task.result = Some("ok".to_string());
        let bad_score = worker.self_assess(&bad_task);
        assert!(bad_score < 0.5, "Bad response should score low, got {}", bad_score);
    }

    #[test]
    fn test_worker_stats() {
        let profile = make_test_profile();
        let engine = Arc::new(MockEngine::new(vec!["done".to_string()]));
        let worker = SpecialistWorker::new(profile, engine, None, None);

        let stats = worker.current_stats();
        assert_eq!(stats.tasks_completed, 0);
        assert_eq!(stats.tasks_failed, 0);
        assert!(stats.uptime_secs < 5);
    }
}
