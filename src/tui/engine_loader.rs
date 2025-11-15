// Dynamic training engine loader with hot-reload support

use libloading::{Library, Symbol};
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use std::fs;

// Import types from sage-training (will be available via rlib during development)
// These types define the stable ABI
use serde::{Deserialize, Serialize};

/// Training engine interface - matches sage-training/src/lib.rs
pub trait TrainingEngine: Send {
    fn step(&mut self, iterations: usize) -> TrainingUpdate;
    fn get_state(&self) -> EngineState;
    fn load_checkpoint(&mut self, checkpoint: Checkpoint) -> Result<(), String>;
    fn save_checkpoint(&self) -> Result<Checkpoint, String>;
    fn set_config(&mut self, config: TrainingConfig);
    fn get_config(&self) -> TrainingConfig;
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TrainingUpdate {
    pub generation: u64,
    pub loss: f64,
    pub grid_snapshot: Vec<Vec<f64>>,
    pub complexity: f64,
    pub diversity: f64,
    pub current_pattern: String,
    pub patterns_mastered: usize,
    pub events: Vec<String>,

    // Advanced Analytics
    pub loss_velocity: f64,
    pub loss_acceleration: f64,
    pub per_pattern_losses: Vec<(String, f64)>,
    pub pattern_attempts: usize,
    pub estimated_time_to_mastery: Option<f64>,
    pub learning_rate: f64,
    pub batch_size: usize,
    pub evolution_steps: usize,
    pub low_loss_streak: usize,

    // Batch Progress (for smooth UI updates)
    pub current_batch: usize,
    pub total_batches: usize,
    pub batch_losses: Vec<f64>,

    // Curriculum info (dynamic - updates with hot-reload)
    pub total_patterns: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EngineState {
    pub generation: u64,
    pub current_loss: f64,
    pub current_pattern: String,
    pub learning_rate: f64,
    pub patterns_mastered: usize,
    pub curriculum_cycle: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Checkpoint {
    pub generation: u64,
    pub nca_weights: NetworkWeights,
    pub current_pattern: usize,
    pub curriculum_progress: Vec<(String, bool)>,
    pub curriculum_cycle: usize,
    pub pattern_attempts: usize,
    pub low_loss_streak: usize,
    pub config: TrainingConfig,
    // Adaptive LR fields
    pub best_loss_this_pattern: f64,
    pub attempts_since_improvement: usize,
    pub current_lr_multiplier: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NetworkWeights {
    pub weights1: Vec<Vec<f64>>,
    pub bias1: Vec<f64>,
    pub weights2: Vec<Vec<f64>>,
    pub bias2: Vec<f64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TrainingConfig {
    pub base_learning_rate: f64,
    pub square_learning_rate: f64,
    pub batch_size: usize,
    pub evolution_steps: usize,
    pub mastery_threshold: usize,
    pub patience_threshold: usize,
    pub mastery_loss_threshold: f64,
}

impl Default for TrainingConfig {
    fn default() -> Self {
        Self {
            base_learning_rate: 0.0005,
            square_learning_rate: 0.001,
            batch_size: 16,  // Larger batches for stable gradients & better learning
            evolution_steps: 100,
            mastery_threshold: 10,
            patience_threshold: 500,
            mastery_loss_threshold: 0.08,
        }
    }
}

/// Hot-reloadable training engine loader
pub struct EngineLoader {
    library_path: PathBuf,
    library: Option<Library>,
    last_modified: SystemTime,
    version: String,
}

impl EngineLoader {
    /// Create a new engine loader
    pub fn new() -> Result<Self, String> {
        let library_path = if cfg!(target_os = "macos") {
            PathBuf::from("target/release/libsage_training.dylib")
        } else if cfg!(target_os = "windows") {
            PathBuf::from("target/release/sage_training.dll")
        } else {
            PathBuf::from("target/release/libsage_training.so")
        };

        let last_modified = Self::get_modification_time(&library_path)?;

        Ok(Self {
            library_path,
            library: None,
            last_modified,
            version: String::from("unknown"),
        })
    }

    /// Load or reload the training engine
    pub fn load(&mut self) -> Result<Box<dyn TrainingEngine>, String> {
        // Unload old library first (if exists)
        if self.library.is_some() {
            // Silent unload - events handled by training runner
            self.library = None;  // Drop triggers unload
        }

        // Check library exists
        if !self.library_path.exists() {
            return Err(format!(
                "Training library not found at: {}. Run: cargo build --release -p sage-training",
                self.library_path.display()
            ));
        }

        // Silent load - events handled by training runner

        // Load new library
        let lib = unsafe {
            Library::new(&self.library_path)
                .map_err(|e| format!("Failed to load library: {}", e))?
        };

        // Check version compatibility
        let version_fn: Symbol<unsafe extern "C" fn() -> *const u8> = unsafe {
            lib.get(b"engine_version")
                .map_err(|e| format!("Failed to find engine_version symbol: {}", e))?
        };

        let version_ptr = unsafe { version_fn() };
        let version_cstr = unsafe { std::ffi::CStr::from_ptr(version_ptr as *const i8) };
        self.version = version_cstr.to_string_lossy().to_string();

        // Version logged by training runner via events

        // Get constructor function
        let constructor: Symbol<unsafe extern "C" fn() -> Box<dyn TrainingEngine>> = unsafe {
            lib.get(b"create_engine")
                .map_err(|e| format!("Failed to find create_engine symbol: {}", e))?
        };

        // Create engine instance
        let engine = unsafe { constructor() };

        // Store library reference (keeps it loaded)
        self.library = Some(lib);

        Ok(engine)
    }

    /// Check if library file has been modified
    pub fn check_for_reload(&mut self) -> bool {
        match Self::get_modification_time(&self.library_path) {
            Ok(current_time) => {
                if current_time > self.last_modified {
                    self.last_modified = current_time;
                    true
                } else {
                    false
                }
            }
            Err(_) => false,
        }
    }

    /// Get file modification time
    fn get_modification_time(path: &Path) -> Result<SystemTime, String> {
        fs::metadata(path)
            .map_err(|e| format!("Failed to get metadata: {}", e))?
            .modified()
            .map_err(|e| format!("Failed to get modification time: {}", e))
    }

    /// Get current engine version
    pub fn version(&self) -> &str {
        &self.version
    }
}

impl Drop for EngineLoader {
    fn drop(&mut self) {
        if self.library.is_some() {
            // Silent unload - library automatically unloaded when dropped
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore] // Only run when library is built
    fn test_load_engine() {
        let mut loader = EngineLoader::new().unwrap();
        let engine = loader.load().unwrap();
        println!("Engine version: {}", loader.version());
    }
}
