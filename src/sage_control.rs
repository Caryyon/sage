// SAGE Control Protocol - Unified instance management and monitoring
//
// Architecture:
// - Each SAGE instance (TUI, IRC bot, Discord bot) registers itself on startup
// - Heartbeat updates every 2-3 seconds
// - Control Center TUI screen reads registry and displays status
// - Control commands sent via shared file
//
// Files:
// - /tmp/sage_instances.json - Instance registry with heartbeats
// - /tmp/sage_control_commands.json - Control commands queue

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

const INSTANCES_FILE: &str = "/tmp/sage_instances.json";
const COMMANDS_FILE: &str = "/tmp/sage_control_commands.json";

/// Types of SAGE instances
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum InstanceType {
    MainTui,
    IrcBot,
    DiscordBot,
    BackgroundLearner,
    Other(String),
}

impl InstanceType {
    pub fn name(&self) -> &str {
        match self {
            InstanceType::MainTui => "Main TUI",
            InstanceType::IrcBot => "IRC Bot",
            InstanceType::DiscordBot => "Discord Bot",
            InstanceType::BackgroundLearner => "Background Learner",
            InstanceType::Other(name) => name,
        }
    }

    pub fn emoji(&self) -> &str {
        match self {
            InstanceType::MainTui => "🖥️",
            InstanceType::IrcBot => "💬",
            InstanceType::DiscordBot => "🎮",
            InstanceType::BackgroundLearner => "🧠",
            InstanceType::Other(_) => "⚙️",
        }
    }
}

/// Instance status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum InstanceStatus {
    Starting,
    Running,
    Reconnecting,
    Stopping,
    Error(String),
}

impl InstanceStatus {
    pub fn indicator(&self) -> &str {
        match self {
            InstanceStatus::Starting => "🟡",
            InstanceStatus::Running => "🟢",
            InstanceStatus::Reconnecting => "🟡",
            InstanceStatus::Stopping => "🔴",
            InstanceStatus::Error(_) => "❌",
        }
    }

    pub fn label(&self) -> String {
        match self {
            InstanceStatus::Starting => "Starting".to_string(),
            InstanceStatus::Running => "Running".to_string(),
            InstanceStatus::Reconnecting => "Reconnecting".to_string(),
            InstanceStatus::Stopping => "Stopping".to_string(),
            InstanceStatus::Error(e) => format!("Error: {}", e),
        }
    }
}

/// Information about a running SAGE instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceInfo {
    /// Type of instance
    pub instance_type: InstanceType,

    /// Process ID
    pub pid: u32,

    /// Current status
    pub status: InstanceStatus,

    /// When instance started (Unix timestamp)
    pub start_time: u64,

    /// Last heartbeat timestamp
    pub last_heartbeat: u64,

    /// Path to log file
    pub log_path: String,

    /// Optional metadata
    pub metadata: HashMap<String, String>,
}

impl InstanceInfo {
    pub fn new(instance_type: InstanceType, pid: u32, log_path: String) -> Self {
        let now = current_timestamp();
        Self {
            instance_type,
            pid,
            status: InstanceStatus::Starting,
            start_time: now,
            last_heartbeat: now,
            log_path,
            metadata: HashMap::new(),
        }
    }

    /// Check if instance is alive based on heartbeat
    pub fn is_alive(&self) -> bool {
        let now = current_timestamp();
        let elapsed = now.saturating_sub(self.last_heartbeat);
        elapsed < 10 // Consider dead if no heartbeat for 10 seconds
    }

    /// Get uptime in seconds
    pub fn uptime(&self) -> u64 {
        let now = current_timestamp();
        now.saturating_sub(self.start_time)
    }

    /// Format uptime as human-readable string
    pub fn uptime_string(&self) -> String {
        let uptime = self.uptime();
        let hours = uptime / 3600;
        let minutes = (uptime % 3600) / 60;
        let seconds = uptime % 60;

        if hours > 0 {
            format!("{}h {}m", hours, minutes)
        } else if minutes > 0 {
            format!("{}m {}s", minutes, seconds)
        } else {
            format!("{}s", seconds)
        }
    }

    /// Update heartbeat
    pub fn heartbeat(&mut self) {
        self.last_heartbeat = current_timestamp();
    }

    /// Update status
    pub fn set_status(&mut self, status: InstanceStatus) {
        self.status = status;
        self.heartbeat();
    }
}

/// Control commands
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ControlCommand {
    Start(InstanceType),
    Stop(InstanceType),
    Restart(InstanceType),
    Reload(InstanceType),
}

/// Instance registry manager
pub struct InstanceRegistry {
    instances: HashMap<InstanceType, InstanceInfo>,
}

impl InstanceRegistry {
    /// Load registry from file
    pub fn load() -> Self {
        let instances = if Path::new(INSTANCES_FILE).exists() {
            fs::read_to_string(INSTANCES_FILE)
                .ok()
                .and_then(|content| serde_json::from_str(&content).ok())
                .unwrap_or_default()
        } else {
            HashMap::new()
        };

        Self { instances }
    }

    /// Save registry to file
    pub fn save(&self) -> std::io::Result<()> {
        let content = serde_json::to_string_pretty(&self.instances)?;
        fs::write(INSTANCES_FILE, content)?;
        Ok(())
    }

    /// Register a new instance
    pub fn register(&mut self, info: InstanceInfo) -> std::io::Result<()> {
        self.instances.insert(info.instance_type.clone(), info);
        self.save()
    }

    /// Unregister an instance
    pub fn unregister(&mut self, instance_type: &InstanceType) -> std::io::Result<()> {
        self.instances.remove(instance_type);
        self.save()
    }

    /// Update instance heartbeat
    pub fn heartbeat(&mut self, instance_type: &InstanceType) -> std::io::Result<()> {
        if let Some(info) = self.instances.get_mut(instance_type) {
            info.heartbeat();
            self.save()?;
        }
        Ok(())
    }

    /// Update instance status
    pub fn update_status(&mut self, instance_type: &InstanceType, status: InstanceStatus) -> std::io::Result<()> {
        if let Some(info) = self.instances.get_mut(instance_type) {
            info.set_status(status);
            self.save()?;
        }
        Ok(())
    }

    /// Get all instances
    pub fn get_all(&self) -> Vec<InstanceInfo> {
        self.instances.values().cloned().collect()
    }

    /// Get specific instance
    pub fn get(&self, instance_type: &InstanceType) -> Option<&InstanceInfo> {
        self.instances.get(instance_type)
    }

    /// Remove dead instances (no heartbeat for 30+ seconds)
    pub fn cleanup_dead(&mut self) -> std::io::Result<()> {
        let now = current_timestamp();
        self.instances.retain(|_, info| {
            let elapsed = now.saturating_sub(info.last_heartbeat);
            elapsed < 30 // Remove if dead for 30+ seconds
        });
        self.save()
    }

    /// Stop an instance by sending SIGTERM
    pub fn stop_instance(&mut self, instance_type: &InstanceType) -> Result<String, String> {
        let info = self.instances.get(instance_type)
            .ok_or_else(|| format!("Instance {:?} not found", instance_type))?;

        let pid = info.pid;

        // Send SIGTERM to process
        #[cfg(unix)]
        {
            use std::process::Command;
            let output = Command::new("kill")
                .arg("-TERM")
                .arg(pid.to_string())
                .output()
                .map_err(|e| format!("Failed to send SIGTERM: {}", e))?;

            if output.status.success() {
                // Update status to Stopping
                self.update_status(instance_type, InstanceStatus::Stopping)
                    .map_err(|e| format!("Failed to update status: {}", e))?;
                Ok(format!("Sent SIGTERM to {} (PID {})", instance_type.name(), pid))
            } else {
                Err(format!("Failed to stop process {}: {}", pid,
                    String::from_utf8_lossy(&output.stderr)))
            }
        }

        #[cfg(not(unix))]
        {
            Err("Process control not supported on this platform".to_string())
        }
    }

    /// Start an instance by spawning a new process
    pub fn start_instance(&mut self, instance_type: &InstanceType) -> Result<String, String> {
        use std::process::Command;

        // Check if already running
        if let Some(info) = self.instances.get(instance_type) {
            if info.is_alive() {
                return Err(format!("{} is already running (PID {})",
                    instance_type.name(), info.pid));
            }
        }

        // Build command based on instance type
        let (example_name, log_path, env_vars) = match instance_type {
            InstanceType::IrcBot => (
                "sage_irc_autonomous",
                "/tmp/sage_irc_LATEST.log",
                Vec::new(),
            ),
            InstanceType::DiscordBot => {
                let discord_token = std::env::var("DISCORD_TOKEN")
                    .unwrap_or_else(|_| "MTQzOTA4MDQzMzA2MDE1NTQwMg.GHAqOl.XfjIdxA9BhbBQI6gfH_3K-B8uflqyiWBfUs6tc".to_string());
                (
                    "sage_discord_autonomous",
                    "/tmp/sage_discord_LATEST.log",
                    vec![("DISCORD_TOKEN", discord_token)],
                )
            },
            _ => return Err(format!("Cannot start instance type: {:?}", instance_type)),
        };

        // Spawn process in background
        let mut cmd = Command::new("cargo");
        cmd.arg("run")
            .arg("--release")
            .arg("--example")
            .arg(example_name);

        // Add environment variables
        for (key, value) in env_vars {
            cmd.env(key, value);
        }

        // Redirect output to log file
        let log_file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_path)
            .map_err(|e| format!("Failed to open log file: {}", e))?;

        cmd.stdout(log_file.try_clone().unwrap())
            .stderr(log_file);

        let child = cmd.spawn()
            .map_err(|e| format!("Failed to spawn process: {}", e))?;

        let pid = child.id();

        Ok(format!("Started {} (PID {})", instance_type.name(), pid))
    }

    /// Restart an instance (stop then start)
    pub fn restart_instance(&mut self, instance_type: &InstanceType) -> Result<String, String> {
        // Stop if running
        if let Some(info) = self.instances.get(instance_type) {
            if info.is_alive() {
                self.stop_instance(instance_type)?;
                // Wait for process to stop
                std::thread::sleep(std::time::Duration::from_secs(2));
            }
        }

        // Remove from registry to allow fresh registration
        let _ = self.unregister(instance_type);

        // Start new instance
        self.start_instance(instance_type)
    }
}

/// Control commands manager
pub struct ControlCommands {
    commands: Vec<ControlCommand>,
}

impl ControlCommands {
    /// Load commands from file
    pub fn load() -> Self {
        let commands = if Path::new(COMMANDS_FILE).exists() {
            fs::read_to_string(COMMANDS_FILE)
                .ok()
                .and_then(|content| serde_json::from_str(&content).ok())
                .unwrap_or_default()
        } else {
            Vec::new()
        };

        Self { commands }
    }

    /// Save commands to file
    pub fn save(&self) -> std::io::Result<()> {
        let content = serde_json::to_string_pretty(&self.commands)?;
        fs::write(COMMANDS_FILE, content)?;
        Ok(())
    }

    /// Send a command
    pub fn send(&mut self, command: ControlCommand) -> std::io::Result<()> {
        self.commands.push(command);
        self.save()
    }

    /// Poll for commands directed at this instance
    pub fn poll(&mut self, instance_type: &InstanceType) -> std::io::Result<Vec<ControlCommand>> {
        let mut my_commands = Vec::new();

        self.commands.retain(|cmd| {
            let is_mine = match cmd {
                ControlCommand::Start(t) => t == instance_type,
                ControlCommand::Stop(t) => t == instance_type,
                ControlCommand::Restart(t) => t == instance_type,
                ControlCommand::Reload(t) => t == instance_type,
            };

            if is_mine {
                my_commands.push(cmd.clone());
                false // Remove from queue
            } else {
                true // Keep in queue
            }
        });

        self.save()?;
        Ok(my_commands)
    }

    /// Clear all commands
    pub fn clear(&mut self) -> std::io::Result<()> {
        self.commands.clear();
        self.save()
    }
}

/// Get current Unix timestamp
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_instance_registration() {
        let mut registry = InstanceRegistry::load();

        let info = InstanceInfo::new(
            InstanceType::IrcBot,
            12345,
            "/tmp/test.log".to_string(),
        );

        registry.register(info.clone()).unwrap();

        let loaded = registry.get(&InstanceType::IrcBot);
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().pid, 12345);
    }

    #[test]
    fn test_heartbeat() {
        let mut info = InstanceInfo::new(
            InstanceType::MainTui,
            99999,
            "/tmp/test.log".to_string(),
        );

        let old_heartbeat = info.last_heartbeat;
        std::thread::sleep(std::time::Duration::from_secs(1));
        info.heartbeat();

        assert!(info.last_heartbeat > old_heartbeat);
        assert!(info.is_alive());
    }
}
