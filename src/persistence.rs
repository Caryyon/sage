// State persistence - keep SAGE alive across restarts

use serde::{Serialize, Deserialize};
use std::fs;
use std::path::Path;

const STATE_FILE: &str = "/tmp/sage_state.json";

#[derive(Serialize, Deserialize, Clone)]
pub struct PersistedState {
    pub uptime_seconds: u64,
    pub phase1_progress: f64,
    pub phase2_progress: f64,
    pub phase3_progress: f64,
    pub phase4_progress: f64,
    pub phase5_progress: f64,
    pub current_phase_name: String,
    pub chat_history: Vec<(String, String)>,
    pub grid_snapshot: Vec<Vec<f64>>,
    pub current_loss: f64,
    pub learning_rate: f64,
    pub is_running: bool,
    pub total_thoughts_generated: usize,
    pub total_decisions_made: usize,
}

impl PersistedState {
    pub fn save(&self) -> Result<(), String> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize state: {}", e))?;

        fs::write(STATE_FILE, json)
            .map_err(|e| format!("Failed to write state file: {}", e))?;

        Ok(())
    }

    pub fn load() -> Option<Self> {
        if !Path::new(STATE_FILE).exists() {
            return None;
        }

        let json = fs::read_to_string(STATE_FILE).ok()?;
        serde_json::from_str(&json).ok()
    }

    pub fn clear() -> Result<(), String> {
        if Path::new(STATE_FILE).exists() {
            fs::remove_file(STATE_FILE)
                .map_err(|e| format!("Failed to remove state file: {}", e))?;
        }
        Ok(())
    }

    pub fn snapshot_exists() -> bool {
        Path::new(STATE_FILE).exists()
    }
}

pub struct StateManager;

impl StateManager {
    /// Create a snapshot of the current state
    pub fn create_snapshot(
        uptime_seconds: u64,
        training_state: &crate::tui::training::TrainingState,
        current_phase: crate::tui::app::Phase,
        chat_history: &[(String, String)],
        active_thoughts: usize,
        active_decisions: usize,
    ) -> PersistedState {
        PersistedState {
            uptime_seconds,
            phase1_progress: training_state.phase1_progress,
            phase2_progress: training_state.phase2_progress,
            phase3_progress: training_state.phase3_progress,
            phase4_progress: training_state.phase4_progress,
            phase5_progress: training_state.phase5_progress,
            current_phase_name: format!("{:?}", current_phase),
            chat_history: chat_history.to_vec(),
            grid_snapshot: training_state.grid_snapshot.clone(),
            current_loss: training_state.current_loss,
            learning_rate: training_state.learning_rate,
            is_running: training_state.is_running,
            total_thoughts_generated: active_thoughts,
            total_decisions_made: active_decisions,
        }
    }

    /// Auto-save state every N seconds
    pub fn auto_save(
        uptime_seconds: u64,
        last_save_time: u64,
        training_state: &crate::tui::training::TrainingState,
        current_phase: crate::tui::app::Phase,
        chat_history: &[(String, String)],
        active_thoughts: usize,
        active_decisions: usize,
    ) -> Result<bool, String> {
        const SAVE_INTERVAL: u64 = 10; // Save every 10 seconds

        if uptime_seconds - last_save_time >= SAVE_INTERVAL {
            let snapshot = Self::create_snapshot(
                uptime_seconds,
                training_state,
                current_phase,
                chat_history,
                active_thoughts,
                active_decisions,
            );
            snapshot.save()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}
