// Hot-reload training runner - wraps EngineLoader for live code updates

use std::sync::{Arc, Mutex};
use std::thread;
use super::app::AppState;
use super::engine_loader::{EngineLoader, TrainingUpdate};
use crate::spacetime_client::SageDbClient;

pub struct HotReloadRunner {
    pub app_state: Arc<Mutex<AppState>>,
}

impl HotReloadRunner {
    pub fn new(app_state: Arc<Mutex<AppState>>) -> Self {
        Self { app_state }
    }

    /// Start training with hot-reload support
    pub fn start_training(&self) {
        let app_state = Arc::clone(&self.app_state);

        thread::spawn(move || {
            // Initialize
            {
                let mut state = app_state.lock().unwrap();
                state.training_state.add_event("Hot-reload training engine starting...".to_string());
            }

            // Load engine
            let mut loader = match EngineLoader::new() {
                Ok(l) => l,
                Err(e) => {
                    let mut state = app_state.lock().unwrap();
                    state.training_state.add_event(format!("❌ Failed to create loader: {}", e));
                    return;
                }
            };

            let mut engine = match loader.load() {
                Ok(e) => e,
                Err(e) => {
                    let mut state = app_state.lock().unwrap();
                    state.training_state.add_event(format!("❌ Failed to load engine: {}", e));
                    return;
                }
            };

            {
                let mut state = app_state.lock().unwrap();
                state.training_state.add_event(format!("[OK] Engine loaded! Version: {}", loader.version()));
            }

            // Initialize SpacetimeDB
            let mut db_client = SageDbClient::new("sage-db");
            let _ = db_client.connect();

            // Training loop
            let mut iteration = 0;
            let batch_size = 1;  // Process 1 step per iteration for smooth 60fps grid updates

            loop {
                // Check if should stop
                {
                    let state = app_state.lock().unwrap();
                    if !state.training_state.is_running {
                        break;
                    }
                }

                // Check for hot-reload
                if loader.check_for_reload() {
                    let mut state = app_state.lock().unwrap();
                    state.training_state.add_event("[SYNC] LIBRARY CHANGED! Hot-reloading...".to_string());

                    // Save checkpoint
                    match engine.save_checkpoint() {
                        Ok(checkpoint) => {
                            state.training_state.add_event(format!("   Checkpoint saved: Gen {}", checkpoint.generation));

                            // Reload engine
                            match loader.load() {
                                Ok(new_engine) => {
                                    engine = new_engine;
                                    state.training_state.add_event(format!("   📚 New engine loaded! Version: {}", loader.version()));

                                    // Restore checkpoint
                                    match engine.load_checkpoint(checkpoint) {
                                        Ok(_) => {
                                            state.training_state.add_event("   [OK] HOT RELOAD COMPLETE!".to_string());
                                        }
                                        Err(e) => {
                                            state.training_state.add_event(format!("   [WARN]  Checkpoint restore failed: {}", e));
                                        }
                                    }
                                }
                                Err(e) => {
                                    state.training_state.add_event(format!("   ❌ Failed to reload engine: {}", e));
                                }
                            }
                        }
                        Err(e) => {
                            state.training_state.add_event(format!("   ❌ Failed to save checkpoint: {}", e));
                        }
                    }
                }

                // Run training iteration
                let update: TrainingUpdate = engine.step(batch_size);

                // Update app state
                {
                    let mut state = app_state.lock().unwrap();

                    // Save previous grid for interpolation
                    state.training_state.previous_grid_snapshot = state.training_state.grid_snapshot.clone();
                    state.training_state.grid_update_time = std::time::Instant::now();

                    // Update core metrics (accessing training_state)
                    state.training_state.nca_generation = update.generation;
                    state.training_state.current_loss = update.loss;
                    state.training_state.nca_complexity = update.complexity;
                    state.training_state.nca_diversity = update.diversity;
                    state.training_state.nca_current_pattern = update.current_pattern.clone();
                    state.training_state.grid_snapshot = update.grid_snapshot.clone();
                    state.training_state.total_patterns = update.total_patterns;  // [SYNC] Updates dynamically with hot-reload

                    // Add events from engine
                    for event in update.events {
                        state.training_state.add_event(event);
                    }

                    // Update patterns mastered
                    // (derived from curriculum progress in engine)
                }

                // Update SpacetimeDB every 10 iterations
                if iteration % 10 == 0 {
                    let state = app_state.lock().unwrap();
                    let _ = db_client.update_sage_state(
                        state.training_state.nca_generation,
                        state.training_state.current_loss,
                        &state.training_state.nca_current_pattern,
                        state.training_state.nca_complexity,
                        state.training_state.nca_diversity,
                    );
                }

                iteration += 1;

                // No sleep - let training run at full speed while UI samples at 60fps
                // The 60fps render loop will show the latest grid_snapshot
            }

            let mut state = app_state.lock().unwrap();
            state.training_state.add_event("🛑 Training stopped".to_string());
        });
    }
}
