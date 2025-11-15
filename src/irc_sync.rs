// IRC message synchronization via shared file
// Simple file-based sync between IRC bot and TUI

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

const SYNC_FILE: &str = "/tmp/sage_irc_messages.json";

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct IrcMessage {
    pub timestamp: String,
    pub sender: String,
    pub message: String,
    pub sage_response: String,
    pub concepts_mentioned: Vec<String>,
}

/// NCA grid state snapshot for visualization
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NcaGridSnapshot {
    pub timestamp: String,
    pub generation: u64,
    pub grid_size: usize,
    /// Flattened alpha channel values (0-1) from 32×32 grid
    pub alpha_values: Vec<f64>,
    /// Active concepts currently being processed
    pub active_concepts: Vec<String>,
    /// Current opinion (Positive/Negative/Neutral/Curious)
    pub current_opinion: String,
    /// Loss value if applicable
    pub loss: f64,
}

#[derive(Serialize, Deserialize)]
pub struct IrcSync {
    pub messages: Vec<IrcMessage>,
    #[serde(default)]
    pub nca_grid: Option<NcaGridSnapshot>,
    #[serde(default)]
    pub total_experiences: u64,
}

impl Default for IrcSync {
    fn default() -> Self {
        Self {
            messages: Vec::new(),
            nca_grid: None,
            total_experiences: 0,
        }
    }
}

impl IrcSync {
    /// Write IRC message to sync file (called by IRC bot)
    pub fn write_message(
        sender: &str,
        message: &str,
        sage_response: &str,
        concepts: Vec<String>,
    ) -> Result<(), String> {
        let mut sync = Self::load().unwrap_or_default();

        let timestamp = chrono::Local::now().format("%H:%M:%S").to_string();

        sync.messages.push(IrcMessage {
            timestamp,
            sender: sender.to_string(),
            message: message.to_string(),
            sage_response: sage_response.to_string(),
            concepts_mentioned: concepts,
        });

        // Keep only last 50 messages
        if sync.messages.len() > 50 {
            sync.messages = sync.messages.split_off(sync.messages.len() - 50);
        }

        let json = serde_json::to_string_pretty(&sync)
            .map_err(|e| format!("Serialization error: {}", e))?;

        fs::write(SYNC_FILE, json)
            .map_err(|e| format!("Write error: {}", e))?;

        Ok(())
    }

    /// Read IRC messages from sync file (called by TUI)
    pub fn load() -> Result<Self, String> {
        if !Path::new(SYNC_FILE).exists() {
            return Ok(Self::default());
        }

        let contents = fs::read_to_string(SYNC_FILE)
            .map_err(|e| format!("Read error: {}", e))?;

        serde_json::from_str(&contents)
            .map_err(|e| format!("Parse error: {}", e))
    }

    /// Get recent messages (last N)
    pub fn get_recent(n: usize) -> Vec<IrcMessage> {
        Self::load()
            .ok()
            .map(|sync| {
                let len = sync.messages.len();
                if len > n {
                    sync.messages[len - n..].to_vec()
                } else {
                    sync.messages
                }
            })
            .unwrap_or_default()
    }

    /// Get all mentioned concepts from recent messages
    pub fn get_active_concepts() -> Vec<String> {
        Self::load()
            .ok()
            .map(|sync| {
                sync.messages
                    .iter()
                    .flat_map(|m| m.concepts_mentioned.clone())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Update NCA grid snapshot (called by IRC bot after learning)
    pub fn update_nca_grid(
        generation: u64,
        alpha_values: Vec<f64>,
        active_concepts: Vec<String>,
        current_opinion: String,
        loss: f64,
    ) -> Result<(), String> {
        let mut sync = Self::load().unwrap_or_default();

        let timestamp = chrono::Local::now().format("%H:%M:%S").to_string();
        let grid_size = (alpha_values.len() as f64).sqrt() as usize;

        sync.nca_grid = Some(NcaGridSnapshot {
            timestamp,
            generation,
            grid_size,
            alpha_values,
            active_concepts,
            current_opinion,
            loss,
        });

        sync.total_experiences += 1;

        let json = serde_json::to_string_pretty(&sync)
            .map_err(|e| format!("Serialization error: {}", e))?;

        fs::write(SYNC_FILE, json)
            .map_err(|e| format!("Write error: {}", e))?;

        Ok(())
    }

    /// Get current NCA grid snapshot (called by TUI)
    pub fn get_nca_grid() -> Option<NcaGridSnapshot> {
        Self::load()
            .ok()
            .and_then(|sync| sync.nca_grid)
    }

    /// Get total experiences count
    pub fn get_total_experiences() -> u64 {
        Self::load()
            .ok()
            .map(|sync| sync.total_experiences)
            .unwrap_or(0)
    }
}
