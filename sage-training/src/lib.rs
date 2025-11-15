// SAGE Training Engine - Hot-reloadable training logic
// This library can be recompiled and reloaded without restarting the TUI

use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

// Re-export core types from sage
pub use sage::{
    nca::NCA,
    grid::Grid,
};

mod training_impl;
use training_impl::*;

/// Target patterns for training
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub enum TargetPattern {
    Circle,
    Square,
    Cross,
    Spiral,
    Hexagon,  // 🆕 Level 4: Six-sided polygon - tests arbitrary n-gons
}

/// Training engine interface - stable ABI boundary
///
/// This trait defines the contract between the TUI and the hot-reloadable training logic.
/// Changes to this interface require careful versioning.
pub trait TrainingEngine: Send {
    /// Execute training iterations
    fn step(&mut self, iterations: usize) -> TrainingUpdate;

    /// Get current engine state (for display)
    fn get_state(&self) -> EngineState;

    /// Load from checkpoint (for hot reload)
    fn load_checkpoint(&mut self, checkpoint: Checkpoint) -> Result<(), String>;

    /// Save checkpoint (before hot reload)
    fn save_checkpoint(&self) -> Result<Checkpoint, String>;

    /// Update configuration dynamically
    fn set_config(&mut self, config: TrainingConfig);

    /// Get current configuration
    fn get_config(&self) -> TrainingConfig;
}

/// Training update sent back to TUI after each step
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
    pub loss_velocity: f64,           // Rate of loss change (derivatives)
    pub loss_acceleration: f64,       // Is learning speeding up or slowing down?
    pub per_pattern_losses: Vec<(String, f64)>,  // Loss for each pattern type
    pub pattern_attempts: usize,      // Current pattern attempt count
    pub estimated_time_to_mastery: Option<f64>,  // Predicted generations until mastery
    pub learning_rate: f64,           // Current learning rate being used
    pub batch_size: usize,            // Current batch size
    pub evolution_steps: usize,       // Steps used for this pattern
    pub low_loss_streak: usize,       // Consecutive low-loss iterations

    // 🎨 Batch Progress (for smooth UI updates)
    pub current_batch: usize,         // Which batch we're currently on
    pub total_batches: usize,         // Total batches in this step
    pub batch_losses: Vec<f64>,       // Individual batch losses for progress display

    // 🔄 Curriculum info (dynamic - updates with hot-reload)
    pub total_patterns: usize,        // Total patterns in curriculum (5 with Hexagon)
}

/// Engine state for display purposes
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EngineState {
    pub generation: u64,
    pub current_loss: f64,
    pub current_pattern: String,
    pub learning_rate: f64,
    pub patterns_mastered: usize,
    pub curriculum_cycle: usize,
}

/// Serializable checkpoint for hot reload
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

/// Network weights (simplified snapshot)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NetworkWeights {
    pub weights1: Vec<Vec<f64>>,
    pub bias1: Vec<f64>,
    pub weights2: Vec<Vec<f64>>,
    pub bias2: Vec<f64>,
}

/// Training configuration (hot-reloadable)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TrainingConfig {
    pub base_learning_rate: f64,
    pub square_learning_rate: f64,
    pub batch_size: usize,
    pub evolution_steps: usize,
    pub mastery_threshold: usize,
    pub patience_threshold: usize,
    pub mastery_loss_threshold: f64,  // NEW: Configurable loss threshold for mastery
}

impl Default for TrainingConfig {
    fn default() -> Self {
        Self {
            base_learning_rate: 0.0005,      // Balanced learning rate
            square_learning_rate: 0.001,     // Higher for sharp edges
            batch_size: 16,                  // ⚡ Larger batches for stable gradients & better learning
            evolution_steps: 100,
            mastery_threshold: 10,           // 10 consecutive low-loss iterations
            patience_threshold: 500,         // Give up after 500 attempts
            mastery_loss_threshold: 0.08,    // 🔥 FIX #1: More realistic threshold (was 0.20)
        }
    }
}

/// Main training engine implementation
pub struct SageTrainingEngine {
    nca: Arc<Mutex<NCA>>,
    generation: u64,
    current_pattern_index: usize,
    curriculum_progress: Vec<(TargetPattern, bool)>,
    curriculum_cycle: usize,
    pattern_attempts: usize,
    low_loss_streak: usize,
    config: TrainingConfig,
    last_loss: f64,
    events: Vec<String>,

    // Analytics tracking
    loss_history: Vec<f64>,          // Last 100 losses for velocity calculation
    per_pattern_loss_history: std::collections::HashMap<String, Vec<f64>>,  // Per-pattern tracking
    #[allow(dead_code)]
    pattern_start_generation: u64,   // When current pattern attempt started

    // 🔥 FIX #4: Adaptive learning rate decay
    best_loss_this_pattern: f64,     // Best loss achieved for current pattern
    attempts_since_improvement: usize, // Attempts without improvement
    current_lr_multiplier: f64,      // Current LR decay multiplier
}

impl SageTrainingEngine {
    pub fn new() -> Self {
        let nca = Arc::new(Mutex::new(NCA::new()));

        // Initialize curriculum
        let curriculum_progress = vec![
            (TargetPattern::Circle, false),
            (TargetPattern::Square, false),
            (TargetPattern::Cross, false),
            (TargetPattern::Spiral, false),
            (TargetPattern::Hexagon, false),  // 🆕 Level 4: Tests n-sided polygon learning
        ];

        Self {
            nca,
            generation: 0,
            current_pattern_index: 0,
            curriculum_progress,
            curriculum_cycle: 0,
            pattern_attempts: 0,
            low_loss_streak: 0,
            config: TrainingConfig::default(),
            last_loss: 1.0,
            events: Vec::new(),
            loss_history: Vec::new(),
            per_pattern_loss_history: std::collections::HashMap::new(),
            pattern_start_generation: 0,
            // Adaptive LR decay fields
            best_loss_this_pattern: f64::INFINITY,
            attempts_since_improvement: 0,
            current_lr_multiplier: 1.0,
        }
    }

    fn get_current_pattern(&self) -> &TargetPattern {
        &self.curriculum_progress[self.current_pattern_index].0
    }

    fn pattern_name(pattern: &TargetPattern) -> String {
        match pattern {
            TargetPattern::Circle => "🔴 Circle".to_string(),
            TargetPattern::Square => "🟦 Square".to_string(),
            TargetPattern::Cross => "➕ Cross".to_string(),
            TargetPattern::Spiral => "🌀 Spiral".to_string(),
            TargetPattern::Hexagon => "⬡ Hexagon".to_string(),
        }
    }

    /// Get pattern-specific mastery threshold (realistic expectations per pattern complexity)
    fn get_mastery_threshold(pattern: &TargetPattern) -> f64 {
        match pattern {
            TargetPattern::Circle => 0.05,   // Easy - smooth radial symmetry
            TargetPattern::Square => 0.08,   // Moderate - sharp corners but regular
            TargetPattern::Cross => 0.12,    // Hard - intersection complexity
            TargetPattern::Spiral => 0.15,   // Very hard - continuous curve with varying radius
            TargetPattern::Hexagon => 0.12,  // Hard - 6 edges + vertices
        }
    }
}

impl TrainingEngine for SageTrainingEngine {
    fn step(&mut self, iterations: usize) -> TrainingUpdate {
        self.events.clear();

        let current_pattern = self.get_current_pattern().clone();
        let pattern_name = Self::pattern_name(&current_pattern);

        // Generate target for current pattern
        let target = generate_target_pattern(current_pattern);

        // 🔥 FIX #4: Adaptive learning rate - moderately higher for complex patterns
        // Base learning rate by pattern (conservative increases to avoid instability)
        let base_lr = match current_pattern {
            TargetPattern::Circle => self.config.base_learning_rate * 0.8,      // Slower for stability (masters easily)
            TargetPattern::Square => self.config.square_learning_rate,          // High for sharp features
            TargetPattern::Cross => self.config.base_learning_rate * 1.6,       // 🎯 Moderate increase (was 1.2 → tried 2.5 but unstable)
            TargetPattern::Spiral => self.config.base_learning_rate * 1.8,      // 🎯 Highest but stable (was 0.6 → tried 3.0 but crashed)
            TargetPattern::Hexagon => self.config.base_learning_rate * 1.5,     // 🆕 Moderate for polygon complexity
        };

        // 🔥 Learning rate warmup - gradually increase LR over first 500 generations for stability
        let warmup_multiplier = if self.generation < 500 {
            // Linear warmup from 0.1x to 1.0x over 500 generations
            0.1 + (self.generation as f64 / 500.0) * 0.9
        } else {
            1.0  // Full learning rate after warmup
        };

        // Apply both warmup and adaptive decay multipliers
        let learning_rate = base_lr * warmup_multiplier * self.current_lr_multiplier;

        // 🔥 Balanced evolution steps - faster convergence with 384-unit network
        let num_steps = match current_pattern {
            TargetPattern::Circle => 60,     // ⚡ Reduced for faster training with larger network
            TargetPattern::Square => 80,     // ⚡ Reduced - 384 units converge faster
            TargetPattern::Cross => 100,     // ⚡ Halved from 180 - still enough for intersections
            TargetPattern::Spiral => 120,    // ⚡ Halved from 240 - balanced speed/quality
            TargetPattern::Hexagon => 110,   // ⚡ Reduced from 200 - faster polygon learning
        };

        // Train using batch processing
        let mut nca_lock = self.nca.lock().unwrap();
        let (avg_loss, batch_losses) = train_nca_batch(
            &mut nca_lock,
            &target,
            learning_rate,
            num_steps,
            self.config.batch_size,
        );
        self.last_loss = avg_loss;

        // Calculate metrics
        let diversity = calculate_diversity(&nca_lock);
        let complexity = calculate_complexity(&nca_lock);

        // Extract grid snapshot
        let grid_snapshot: Vec<Vec<f64>> = nca_lock.grid.cells.iter()
            .map(|row| row.iter().map(|cell| cell[3]).collect())
            .collect();

        drop(nca_lock);

        // Update generation counter
        self.generation += iterations as u64 * self.config.batch_size as u64;

        // Update low loss streak for curriculum (pattern-specific thresholds)
        let current_pattern = self.get_current_pattern().clone();
        let mastery_threshold = Self::get_mastery_threshold(&current_pattern);

        if self.last_loss < mastery_threshold {
            self.low_loss_streak += 1;
        } else {
            self.low_loss_streak = 0;
        }

        self.pattern_attempts += 1;

        // 🔥 FIX #4: Adaptive LR decay - reduce LR when stuck
        if self.last_loss < self.best_loss_this_pattern * 0.98 {
            // Improved by at least 2% - reset decay
            self.best_loss_this_pattern = self.last_loss;
            self.attempts_since_improvement = 0;
        } else {
            // No significant improvement
            self.attempts_since_improvement += 1;

            // Decay LR after 100 attempts without improvement
            if self.attempts_since_improvement >= 100 && self.current_lr_multiplier > 0.1 {
                self.current_lr_multiplier *= 0.5;  // Cut LR in half
                self.attempts_since_improvement = 0;  // Reset counter
                self.events.push(format!(
                    "🔽 LR decay: multiplier now {:.2}x (stuck for 100 attempts)",
                    self.current_lr_multiplier
                ));
            }
        }

        // ═══════════════════════════════════════════════════════════════
        // ADVANCED ANALYTICS CALCULATION
        // ═══════════════════════════════════════════════════════════════

        // Update loss history (keep last 100 for smoothing)
        self.loss_history.push(self.last_loss);
        if self.loss_history.len() > 100 {
            self.loss_history.remove(0);
        }

        // Track per-pattern loss
        let pattern_key = pattern_name.clone();
        self.per_pattern_loss_history
            .entry(pattern_key.clone())
            .or_insert_with(Vec::new)
            .push(self.last_loss);

        // Calculate loss velocity (rate of change) using last 10 samples
        let loss_velocity = if self.loss_history.len() >= 10 {
            let recent = &self.loss_history[self.loss_history.len() - 10..];
            let first_half_avg: f64 = recent[..5].iter().sum::<f64>() / 5.0;
            let second_half_avg: f64 = recent[5..].iter().sum::<f64>() / 5.0;
            (second_half_avg - first_half_avg) / 5.0  // Change per step
        } else {
            0.0
        };

        // Calculate loss acceleration (is velocity changing?)
        let loss_acceleration = if self.loss_history.len() >= 20 {
            let recent = &self.loss_history[self.loss_history.len() - 20..];
            let first_velocity = (recent[5..10].iter().sum::<f64>() / 5.0 - recent[0..5].iter().sum::<f64>() / 5.0) / 5.0;
            let second_velocity = (recent[15..20].iter().sum::<f64>() / 5.0 - recent[10..15].iter().sum::<f64>() / 5.0) / 5.0;
            (second_velocity - first_velocity) / 10.0
        } else {
            0.0
        };

        // Per-pattern loss summary (average of last 20 samples per pattern)
        let per_pattern_losses: Vec<(String, f64)> = self.curriculum_progress
            .iter()
            .map(|(pattern, _)| {
                let name = Self::pattern_name(pattern);
                let avg_loss = self.per_pattern_loss_history
                    .get(&name)
                    .and_then(|history| {
                        if history.is_empty() { return None; }
                        let samples = history.iter().rev().take(20).copied().collect::<Vec<_>>();
                        Some(samples.iter().sum::<f64>() / samples.len() as f64)
                    })
                    .unwrap_or(1.0);
                (name, avg_loss)
            })
            .collect();

        // Estimate time to mastery using linear regression on recent loss trend (pattern-specific)
        let estimated_time_to_mastery = if loss_velocity < -0.001 && self.last_loss > mastery_threshold {
            // If loss is decreasing, estimate generations needed to reach threshold
            let remaining_loss = self.last_loss - mastery_threshold;
            let generations_needed = (remaining_loss / loss_velocity.abs()) as f64;
            Some(generations_needed.max(0.0))
        } else if self.last_loss <= mastery_threshold {
            Some(0.0)  // Already at mastery
        } else {
            None  // Not converging or stuck
        };

        // Check for pattern mastery or patience threshold
        let should_progress = self.low_loss_streak >= self.config.mastery_threshold
            || self.pattern_attempts >= self.config.patience_threshold;

        if should_progress {
            // Mark as mastered if truly mastered
            if self.low_loss_streak >= self.config.mastery_threshold {
                self.curriculum_progress[self.current_pattern_index].1 = true;
                self.events.push(format!("✅ {} mastered!", pattern_name));
            } else {
                self.events.push(format!("🔄 Moving from {} (spiral learning)", pattern_name));
            }

            // Move to next pattern
            self.current_pattern_index = (self.current_pattern_index + 1) % self.curriculum_progress.len();
            self.pattern_attempts = 0;
            self.low_loss_streak = 0;

            // Reset adaptive LR decay for new pattern
            self.best_loss_this_pattern = f64::INFINITY;
            self.attempts_since_improvement = 0;
            self.current_lr_multiplier = 1.0;

            // Check if completed full cycle
            if self.curriculum_progress.iter().all(|(_, m)| *m) {
                self.curriculum_cycle += 1;
                if self.curriculum_cycle == 1 {
                    self.events.push("🔄 SPIRAL LEARNING: Revisiting patterns!".to_string());
                }
            }

            let new_pattern_name = Self::pattern_name(self.get_current_pattern());
            if self.curriculum_cycle > 0 {
                self.events.push(format!("🔄 Revisiting: {} (Cycle {})", new_pattern_name, self.curriculum_cycle + 1));
            } else {
                self.events.push(format!("🎯 New challenge: {}", new_pattern_name));
            }
        }

        // Progress indicators
        if self.pattern_attempts > 0 && self.pattern_attempts % 25 == 0 && self.low_loss_streak < self.config.mastery_threshold {
            self.events.push(format!("🔄 Progress: {}/{} attempts", self.pattern_attempts, self.config.patience_threshold));
        }

        TrainingUpdate {
            generation: self.generation,
            loss: self.last_loss,
            grid_snapshot,
            complexity,
            diversity,
            current_pattern: pattern_name,
            patterns_mastered: self.curriculum_progress.iter().filter(|(_, m)| *m).count(),
            events: self.events.clone(),

            // Advanced analytics
            loss_velocity,
            loss_acceleration,
            per_pattern_losses,
            pattern_attempts: self.pattern_attempts,
            estimated_time_to_mastery,
            learning_rate,
            batch_size: self.config.batch_size,
            evolution_steps: num_steps,
            low_loss_streak: self.low_loss_streak,

            // Batch progress (for smooth UI)
            current_batch: batch_losses.len(),  // All batches completed
            total_batches: self.config.batch_size,
            batch_losses,

            // Curriculum info (updates with hot-reload)
            total_patterns: self.curriculum_progress.len(),  // 🔄 Dynamic - now 5 with Hexagon
        }
    }

    fn get_state(&self) -> EngineState {
        let pattern_name = Self::pattern_name(self.get_current_pattern());

        EngineState {
            generation: self.generation,
            current_loss: self.last_loss,
            current_pattern: pattern_name,
            learning_rate: self.config.base_learning_rate,
            patterns_mastered: self.curriculum_progress.iter().filter(|(_, m)| *m).count(),
            curriculum_cycle: self.curriculum_cycle,
        }
    }

    fn load_checkpoint(&mut self, checkpoint: Checkpoint) -> Result<(), String> {
        self.generation = checkpoint.generation;
        self.current_pattern_index = checkpoint.current_pattern;
        self.curriculum_cycle = checkpoint.curriculum_cycle;
        self.pattern_attempts = checkpoint.pattern_attempts;
        self.low_loss_streak = checkpoint.low_loss_streak;
        self.config = checkpoint.config;

        // Load adaptive LR state
        self.best_loss_this_pattern = checkpoint.best_loss_this_pattern;
        self.attempts_since_improvement = checkpoint.attempts_since_improvement;
        self.current_lr_multiplier = checkpoint.current_lr_multiplier;

        // Reconstruct curriculum progress
        self.curriculum_progress = checkpoint.curriculum_progress.iter()
            .map(|(name, mastered)| {
                let pattern = match name.as_str() {
                    "🔴 Circle" => TargetPattern::Circle,
                    "🟦 Square" => TargetPattern::Square,
                    "➕ Cross" => TargetPattern::Cross,
                    "🌀 Spiral" => TargetPattern::Spiral,
                    _ => TargetPattern::Circle,
                };
                (pattern, *mastered)
            })
            .collect();

        // Load network weights
        let mut nca = self.nca.lock().unwrap();
        nca.update_net.weights1 = checkpoint.nca_weights.weights1;
        nca.update_net.bias1 = checkpoint.nca_weights.bias1;
        nca.update_net.weights2 = checkpoint.nca_weights.weights2;
        nca.update_net.bias2 = checkpoint.nca_weights.bias2;

        self.events.push(format!("✅ Checkpoint loaded: Gen {}", self.generation));

        Ok(())
    }

    fn save_checkpoint(&self) -> Result<Checkpoint, String> {
        let nca = self.nca.lock().unwrap();

        let weights = NetworkWeights {
            weights1: nca.update_net.weights1.clone(),
            bias1: nca.update_net.bias1.clone(),
            weights2: nca.update_net.weights2.clone(),
            bias2: nca.update_net.bias2.clone(),
        };

        let curriculum_progress: Vec<(String, bool)> = self.curriculum_progress.iter()
            .map(|(pattern, mastered)| (Self::pattern_name(pattern), *mastered))
            .collect();

        Ok(Checkpoint {
            generation: self.generation,
            nca_weights: weights,
            current_pattern: self.current_pattern_index,
            curriculum_progress,
            curriculum_cycle: self.curriculum_cycle,
            pattern_attempts: self.pattern_attempts,
            low_loss_streak: self.low_loss_streak,
            config: self.config.clone(),
            // Save adaptive LR state
            best_loss_this_pattern: self.best_loss_this_pattern,
            attempts_since_improvement: self.attempts_since_improvement,
            current_lr_multiplier: self.current_lr_multiplier,
        })
    }

    fn set_config(&mut self, config: TrainingConfig) {
        self.config = config;
        self.events.push("⚙️  Configuration updated".to_string());
    }

    fn get_config(&self) -> TrainingConfig {
        self.config.clone()
    }
}

/// FFI entry point for creating the engine
/// This is called by the TUI when loading the library
#[no_mangle]
#[allow(improper_ctypes_definitions)]
pub extern "C" fn create_engine() -> Box<dyn TrainingEngine> {
    Box::new(SageTrainingEngine::new())
}

/// Version string for ABI compatibility checking
#[no_mangle]
pub extern "C" fn engine_version() -> *const u8 {
    b"0.1.0\0".as_ptr()
}
