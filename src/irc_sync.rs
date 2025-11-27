//! IRC Sync - Stub module (IRC functionality removed)
//!
//! This is a placeholder to maintain compatibility while IRC is disabled.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IrcMessage {
    pub sender: String,
    pub content: String,
    pub message: String,
    pub timestamp: u64,
    pub channel: String,
    pub sage_response: String,
    pub concepts_mentioned: Vec<String>,
    pub emotional_tone: f64,
}

/// NCA Grid snapshot for TUI display
#[derive(Debug, Clone, Default)]
pub struct GridSnapshot {
    pub cells: Vec<Vec<Vec<f64>>>,
    pub grid_size: usize,
    pub generation: usize,
    pub active_concepts: Vec<String>,
    pub current_opinion: String,
}

/// Autonomous activity record
#[derive(Debug, Clone, Default)]
pub struct AutonomousActivity {
    pub timestamp: u64,
    pub activity_type: String,
    pub description: String,
}

/// Camera snapshot for visual processing
#[derive(Debug, Clone, Default)]
pub struct CameraSnapshot {
    pub frame: Vec<Vec<(u8, u8, u8)>>,
    pub visual_concepts: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IrcSync {
    pub messages: Vec<IrcMessage>,
}

impl IrcSync {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn load() -> Result<Self, std::io::Error> {
        Ok(Self::default())
    }

    pub fn get_recent(_count: usize) -> Vec<IrcMessage> {
        Vec::new()
    }

    pub fn get_camera_snapshot() -> Option<CameraSnapshot> {
        None
    }

    pub fn get_autonomous_activities() -> Vec<AutonomousActivity> {
        Vec::new()
    }

    pub fn get_total_experiences() -> usize {
        0
    }

    pub fn get_nca_grid() -> Option<GridSnapshot> {
        None
    }
}
