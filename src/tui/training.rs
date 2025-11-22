// Training runner - executes training phases in background

use std::sync::{Arc, Mutex};
use std::thread;
use super::app::Phase;
use crate::spacetime_client::SageDbClient;
use crate::learning::meta_learning::MetaLearner;
use crate::learning::meta_learning_manager::{MetaLearningManager, MetaLearningConfig, MetaLearningEvent};

/// Target patterns for NCA to learn
#[derive(Clone, Copy)]
enum TargetPattern {
    Circle,
    Square,
    Cross,
    Spiral,
}

/// Generate a target pattern for the NCA to learn
fn generate_target_pattern(pattern: TargetPattern) -> crate::grid::Grid {
    use crate::grid::{Grid, GRID_SIZE};

    let mut grid = Grid::new(GRID_SIZE, GRID_SIZE);
    let center_x = GRID_SIZE / 2;
    let center_y = GRID_SIZE / 2;

    match pattern {
        TargetPattern::Circle => {
            // Create a circle in the center
            let radius = 8.0;
            for y in 0..GRID_SIZE {
                for x in 0..GRID_SIZE {
                    let dx = x as f64 - center_x as f64;
                    let dy = y as f64 - center_y as f64;
                    let dist = (dx * dx + dy * dy).sqrt();

                    if dist < radius {
                        // Inside circle - full alpha
                        grid.cells[y][x][3] = 1.0;
                        // Also set RGB to make it visible
                        grid.cells[y][x][0] = 1.0;
                        grid.cells[y][x][1] = 0.0;
                        grid.cells[y][x][2] = 0.0;
                    }
                }
            }
        }
        TargetPattern::Square => {
            // Create a square
            let size = 12;
            for y in (center_y - size)..(center_y + size) {
                for x in (center_x - size)..(center_x + size) {
                    if y < GRID_SIZE && x < GRID_SIZE {
                        grid.cells[y][x][3] = 1.0;
                        grid.cells[y][x][0] = 0.0;
                        grid.cells[y][x][1] = 1.0;
                        grid.cells[y][x][2] = 0.0;
                    }
                }
            }
        }
        TargetPattern::Cross => {
            // Create a cross/plus shape
            let arm_width = 4;
            let arm_length = 10;

            // Horizontal arm
            for x in (center_x - arm_length)..(center_x + arm_length) {
                for dy in 0..arm_width {
                    let y = center_y - arm_width/2 + dy;
                    if y < GRID_SIZE && x < GRID_SIZE {
                        grid.cells[y][x][3] = 1.0;
                        grid.cells[y][x][0] = 0.0;
                        grid.cells[y][x][1] = 0.0;
                        grid.cells[y][x][2] = 1.0;
                    }
                }
            }

            // Vertical arm
            for y in (center_y - arm_length)..(center_y + arm_length) {
                for dx in 0..arm_width {
                    let x = center_x - arm_width/2 + dx;
                    if y < GRID_SIZE && x < GRID_SIZE {
                        grid.cells[y][x][3] = 1.0;
                        grid.cells[y][x][0] = 0.0;
                        grid.cells[y][x][1] = 0.0;
                        grid.cells[y][x][2] = 1.0;
                    }
                }
            }
        }
        TargetPattern::Spiral => {
            // Create a spiral
            for i in 0..100 {
                let angle = i as f64 * 0.3;
                let radius = i as f64 * 0.1;
                let x = (center_x as f64 + radius * angle.cos()) as usize;
                let y = (center_y as f64 + radius * angle.sin()) as usize;

                if x < GRID_SIZE && y < GRID_SIZE {
                    grid.cells[y][x][3] = 1.0;
                    grid.cells[y][x][0] = 1.0;
                    grid.cells[y][x][1] = 1.0;
                    grid.cells[y][x][2] = 0.0;
                }
            }
        }
    }

    grid
}

/// Initialize grid with a noisy version of the target pattern
/// This makes learning easier - NCA refines existing pattern rather than growing from scratch
fn initialize_grid_from_pattern(grid: &mut crate::grid::Grid, target: &crate::grid::Grid) {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    for y in 0..grid.height {
        for x in 0..grid.width {
            // Copy target with some noise
            for channel in 0..4 {  // RGBA channels
                let target_val = target.cells[y][x][channel];
                // Add noise: ±20% of target value
                let noise = rng.gen_range(-0.2..0.2);
                grid.cells[y][x][channel] = (target_val + noise).clamp(0.0, 1.0);
            }

            // Initialize hidden channels with small random values
            for channel in 4..crate::grid::NUM_CHANNELS {
                grid.cells[y][x][channel] = rng.gen_range(-0.1..0.1);
            }
        }
    }
}

// OLD: Generate realistic NCA-like patterns based on training progress
pub struct TrainingState {
    pub current_phase: Phase,
    pub phase1_progress: f64,
    pub phase2_progress: f64,
    pub phase3_progress: f64,
    pub phase4_progress: f64,
    pub phase5_progress: f64,
    pub is_running: bool,
    pub recent_events: Vec<String>,
    pub debug_logs: Vec<String>,  // Real-time debug logs for visualization debugging
    pub grid_snapshot: Vec<Vec<f64>>,  // 32x32 grid showing current NCA state
    pub previous_grid_snapshot: Vec<Vec<f64>>,  // Previous grid for interpolation
    pub grid_update_time: std::time::Instant,  // When grid was last updated
    pub current_loss: f64,
    pub learning_rate: f64,
    pub active_thought_count: usize,
    pub active_decision_count: usize,
    pub performance: f64,  // 0.0-100.0 based on accumulated knowledge
    pub concept_count: usize,  // Number of concepts in memory

    // NEW: Learning system metrics
    pub neural_classifier_accuracy: f64,
    pub curiosity_total_explorations: usize,
    pub curiosity_avg_novelty: f64,
    pub curiosity_total_reward: f64,
    pub hierarchical_total_concepts: usize,
    pub hierarchical_level_counts: Vec<usize>,
    pub meta_learning_rate: f64,
    pub meta_curriculum_difficulty: f64,
    pub meta_learning_efficiency: f64,

    // NCA-specific metrics
    pub nca_generation: u64,
    pub nca_diversity: f64,
    pub nca_complexity: f64,
    pub nca_current_pattern: String,  // What pattern is being learned

    // Chat history
    pub chat_history: Vec<(String, String)>,  // (sender, message) pairs

    // NEW: Advanced Analytics (AI Engineer Dashboard)
    pub loss_velocity: f64,            // Rate of loss change
    pub loss_acceleration: f64,        // Is learning speeding up or slowing down?
    pub per_pattern_losses: Vec<(String, f64)>,  // Loss for each pattern
    pub pattern_attempts: usize,       // Current pattern attempt count
    pub estimated_time_to_mastery: Option<f64>,  // ETA in generations
    pub batch_size: usize,             // Current batch size
    pub evolution_steps: usize,        // Steps used for current pattern
    pub low_loss_streak: usize,        // Consecutive low-loss iterations
    pub patterns_mastered: usize,      // Total patterns mastered

    // Batch Progress (for smooth UI)
    pub current_batch: usize,          // Current batch being processed
    pub total_batches: usize,          // Total batches per step
    pub batch_losses: Vec<f64>,        // Individual batch losses

    // [SYNC] Curriculum info (dynamic - updates with hot-reload)
    pub total_patterns: usize,         // Total patterns in curriculum

    // Training history buffer (for Database Monitor charts)
    pub metrics_history: Vec<MetricSnapshot>,  // Last 200 training iterations

    // Meta-Learning Dashboard fields (Phase A-F integration)
    pub meta_learning_strategy: String,      // Current strategy name
    pub strategy_confidence: f64,            // Confidence in strategy selection
    pub strategy_reasoning: String,          // Why this strategy was chosen
    pub warmup_complete: bool,               // Phase A: warmup done?
    pub lr_cycle_progress: f64,              // Phase A: cosine cycle progress 0-1
    pub reptile_meta_steps: usize,           // Phase B: meta-steps completed
    pub reptile_avg_loss: f64,               // Phase B: average loss across tasks
    pub pbt_population_size: usize,          // Phase C: population size
    pub pbt_evolutions: usize,               // Phase C: evolution count
    pub pbt_best_score: f64,                 // Phase C: best score found
    pub pbt_best_lr: f64,                    // Phase C: best learning rate
    pub arch_neurons: usize,                 // Phase E: hidden neuron count
    pub arch_modifications: usize,           // Phase E: modification count
    pub arch_rollbacks: usize,               // Phase E: rollback count
    pub meta_learning_events: Vec<String>,   // Recent meta-learning events
}

#[derive(Debug, Clone)]
pub struct MetricSnapshot {
    pub generation: u64,
    pub loss: f64,
    pub complexity: f64,
    pub diversity: f64,
    pub pattern: String,
}

impl TrainingState {
    pub fn new() -> Self {
        Self {
            current_phase: Phase::Idle,
            phase1_progress: 0.0,
            phase2_progress: 0.0,
            phase3_progress: 0.0,
            phase4_progress: 0.0,
            phase5_progress: 0.0,
            is_running: false,
            recent_events: Vec::new(),
            debug_logs: Vec::new(),
            grid_snapshot: vec![vec![0.0; 32]; 32],
            previous_grid_snapshot: vec![vec![0.0; 32]; 32],
            grid_update_time: std::time::Instant::now(),
            current_loss: 1.0,
            learning_rate: 0.01,
            active_thought_count: 0,
            active_decision_count: 0,
            performance: 0.0,
            concept_count: 0,

            // Initialize new learning metrics
            neural_classifier_accuracy: 0.0,
            curiosity_total_explorations: 0,
            curiosity_avg_novelty: 0.0,
            curiosity_total_reward: 0.0,
            hierarchical_total_concepts: 0,
            hierarchical_level_counts: vec![0, 0, 0],
            meta_learning_rate: 0.01,
            meta_curriculum_difficulty: 0.3,
            meta_learning_efficiency: 0.0,
            nca_generation: 0,
            nca_diversity: 0.0,
            nca_complexity: 0.0,
            nca_current_pattern: "Circle".to_string(),
            chat_history: Vec::new(),

            // Initialize advanced analytics
            loss_velocity: 0.0,
            loss_acceleration: 0.0,
            per_pattern_losses: Vec::new(),
            pattern_attempts: 0,
            estimated_time_to_mastery: None,
            batch_size: 5,
            evolution_steps: 100,
            low_loss_streak: 0,
            patterns_mastered: 0,

            // Initialize batch progress
            current_batch: 0,
            total_batches: 0,
            batch_losses: Vec::new(),

            // Initialize curriculum info
            total_patterns: 4,  // Will be updated by hot-reload to 5

            // Initialize training history
            metrics_history: Vec::new(),

            // Initialize meta-learning fields
            meta_learning_strategy: "Standard".to_string(),
            strategy_confidence: 0.5,
            strategy_reasoning: "Initial strategy".to_string(),
            warmup_complete: false,
            lr_cycle_progress: 0.0,
            reptile_meta_steps: 0,
            reptile_avg_loss: 1.0,
            pbt_population_size: 6,
            pbt_evolutions: 0,
            pbt_best_score: 0.0,
            pbt_best_lr: 0.0001,
            arch_neurons: 16,
            arch_modifications: 0,
            arch_rollbacks: 0,
            meta_learning_events: Vec::new(),
        }
    }

    pub fn add_metric_snapshot(&mut self, generation: u64, loss: f64, complexity: f64, diversity: f64, pattern: String) {
        self.metrics_history.push(MetricSnapshot {
            generation,
            loss,
            complexity,
            diversity,
            pattern,
        });
        // Keep only last 200 metrics for performance
        if self.metrics_history.len() > 200 {
            self.metrics_history.remove(0);
        }
    }

    pub fn add_event(&mut self, event: String) {
        self.recent_events.push(event);
        // Keep only last 10 events
        if self.recent_events.len() > 10 {
            self.recent_events.remove(0);
        }
    }

    /// THE GOLDEN RULE: Sync NCA grid from SAGE for visualization
    /// Every learning event MUST call this to update the Living Neural Field
    pub fn sync_nca_from_sage(&mut self, nca_grid: &crate::grid::Grid, concept: &str, loss: f64) {
        use std::time::Instant;

        // Store previous grid for smooth interpolation
        self.previous_grid_snapshot = self.grid_snapshot.clone();
        self.grid_update_time = Instant::now();

        // Convert SAGE's NCA grid (48x48 RGBA) to visualization grid (32x32 grayscale)
        let grid_size = nca_grid.width.min(nca_grid.height);
        let scale = grid_size / 32;

        // DEBUG: Track maximum values to diagnose visualization
        let mut max_alpha = 0.0f64;
        let mut max_boosted = 0.0f64;

        for y in 0..32 {
            for x in 0..32 {
                let src_y = (y * scale).min(grid_size - 1);
                let src_x = (x * scale).min(grid_size - 1);

                // Use alpha channel (channel 3) for visualization - the "alive" channel
                // This is where most NCA activity happens
                let cell = &nca_grid.cells[src_y][src_x];
                let alpha = cell[3];
                max_alpha = max_alpha.max(alpha.abs());

                // Boost contrast: clamp to [0, 1] and apply power curve for visibility
                let boosted = alpha.abs().powf(0.7).min(1.0).max(0.0);
                max_boosted = max_boosted.max(boosted);

                self.grid_snapshot[y][x] = boosted;
            }
        }

        // Update metadata
        self.nca_current_pattern = concept.to_string();
        self.current_loss = loss;
        self.nca_generation += 1;
    }
}

pub struct TrainingRunner {
    pub state: Arc<Mutex<TrainingState>>,
    pub agi: Arc<Mutex<crate::agi::AGISystem>>,
    pub language_learner: Arc<Mutex<crate::language_learning::LanguageLearner>>,
    pub nca: Arc<Mutex<crate::nca::NCA>>,  // The core NCA system
}

impl TrainingRunner {
    /// Create empty runner (fast, no neural network initialization)
    pub fn empty() -> Self {
        let lang_learner = crate::language_learning::LanguageLearner::new();
        let mut nca = crate::nca::NCA::new();
        nca.reset_with_seed();  // Initialize with random seed

        Self {
            state: Arc::new(Mutex::new(TrainingState::new())),
            agi: Arc::new(Mutex::new(crate::agi::AGISystem::new())),
            language_learner: Arc::new(Mutex::new(lang_learner)),
            nca: Arc::new(Mutex::new(nca)),
        }
    }

    /// Initialize the learning systems (trains neural network)
    pub fn initialize_systems(&self) {
        let mut lang_learner = self.language_learner.lock().unwrap();

        // Enable neural classifier (will bootstrap from rules automatically in silent mode)
        // This trains on 1000 samples for 500 epochs - visible during TUI startup
        lang_learner.set_use_neural(true, true);

        // Enable self-supervised learning
        lang_learner.enable_self_supervised();

        // Enable curiosity-driven exploration
        lang_learner.enable_curiosity();
    }

    /// Fast initialization - skip heavy training since visualization already showed it
    pub fn initialize_systems_fast(&self) {
        let mut lang_learner = self.language_learner.lock().unwrap();

        // Just enable the systems without heavy bootstrap training
        // The visualization already showed the conceptual training process
        lang_learner.enable_self_supervised();
        lang_learner.enable_curiosity();

        // Note: Neural classifier stays disabled or uses rule-based fallback
        // This makes startup instant after the visualization
    }

    pub fn new() -> Self {
        let runner = Self::empty();
        runner.initialize_systems();
        runner
    }

    pub fn with_agi(agi: Arc<Mutex<crate::agi::AGISystem>>) -> Self {
        // Create language learner with ALL advanced learning systems enabled
        let mut lang_learner = crate::language_learning::LanguageLearner::new();

        // Enable neural classifier (will bootstrap from rules automatically in silent mode)
        lang_learner.set_use_neural(true, true);

        // Enable self-supervised learning
        lang_learner.enable_self_supervised();

        // Enable curiosity-driven exploration
        lang_learner.enable_curiosity();

        // Initialize NCA
        let mut nca = crate::nca::NCA::new();
        nca.reset_with_seed();

        Self {
            state: Arc::new(Mutex::new(TrainingState::new())),
            agi,
            language_learner: Arc::new(Mutex::new(lang_learner)),
            nca: Arc::new(Mutex::new(nca)),
        }
    }

    pub fn start_training(&self) {
        let state = Arc::clone(&self.state);
        let _agi = Arc::clone(&self.agi);
        let _language_learner = Arc::clone(&self.language_learner);
        let nca = Arc::clone(&self.nca);

        thread::spawn(move || {
            // Set running state at the start
            {
                let mut s = state.lock().unwrap();
                s.is_running = true;
                s.add_event("🧬 NCA evolution started".to_string());
            }

            // Initialize communication handler
            let mut comm_handler = crate::communication::CommunicationHandler::new();

            // Initialize SpacetimeDB client
            let mut db_client = SageDbClient::new("sage-db");
            let _ = db_client.connect();

            // LOAD BEST WEIGHTS FROM DATABASE - Resume training from previous best!
            let load_result = std::process::Command::new("spacetime")
                .args(&["sql", "sage-db", "SELECT * FROM network_snapshots"])
                .stderr(std::process::Stdio::null())
                .output();

            #[allow(unused_assignments)]
            let mut resumed_from_gen = 0u64;
            #[allow(unused_assignments)]
            let mut resumed_pattern = String::from("Circle");

            if let Ok(output) = load_result {
                if output.status.success() {
                    let text = String::from_utf8_lossy(&output.stdout);
                    // Parse and find best snapshot
                    let lines: Vec<&str> = text.lines().collect();
                    if lines.len() > 2 {
                        // Get last snapshot (most recent save)
                        if let Some(last_line) = lines.iter().rev().find(|l| !l.starts_with('+') && !l.is_empty() && !l.contains("generation")) {
                            let parts: Vec<&str> = last_line.split('|').map(|s| s.trim()).collect();
                            if parts.len() >= 7 {
                                resumed_from_gen = parts[2].parse().unwrap_or(0);
                                resumed_pattern = parts[3].trim_matches('"').to_string();

                                // Try to load the weights (currently just "{}" placeholder)
                                if let Ok(_weights_json) = serde_json::from_str::<serde_json::Value>(parts[5]) {
                                    // TODO: Actually restore NCA weights when we serialize them
                                    let mut s = state.lock().unwrap();
                                    s.add_event(format!("[SYNC] Resuming from generation {} ({})", resumed_from_gen, resumed_pattern));
                                }
                            }
                        }
                    }
                }
            }

            let _ = db_client.log_training_event(resumed_from_gen, "pattern_start", &format!("🧬 NCA evolution started (resumed from gen {})", resumed_from_gen));

            // NCA TRAINING LOOP - Following Growing NCA paper methodology
            let mut step_count = resumed_from_gen;
            let mut current_pattern_type = TargetPattern::Circle;
            let mut current_target = generate_target_pattern(current_pattern_type);

            // Start from seed like the paper (or resume from loaded state)
            if resumed_from_gen == 0 {
                let mut nca_lock = nca.lock().unwrap();
                nca_lock.reset_with_seed();
            }

            let mut steps_on_target = 0;
            let mut low_loss_streak = 0;
            let mut last_damage_step = 0u64;
            #[allow(unused_assignments)]
            let mut learning_rate = 0.00005;  // Start with ultra-low learning rate for stability with Adam
            let mut pattern_attempts = 0u64;  // Track attempts on current pattern

            // META-LEARNING: Adaptive learning rate controller
            // Initializes with pattern-specific base rates, then adjusts based on progress
            let base_lr = 0.00005;  // Base learning rate for meta-learner
            let mut meta_learner = MetaLearner::new(base_lr);

            // META-LEARNING MANAGER: Coordinates all 6 meta-learning phases
            let mut meta_manager = MetaLearningManager::new(MetaLearningConfig {
                base_lr,
                enable_reptile: true,
                enable_pbt: true,
                enable_architecture_mod: true,
                pbt_population_size: 6,
                pbt_evolution_interval: 100,
                architecture_check_interval: 200,
            });

            // Initialize meta-learning manager with NCA
            {
                let nca_lock = nca.lock().unwrap();
                meta_manager.initialize(&nca_lock);
            }

            // Start first pattern with meta-manager
            let _ = meta_manager.start_pattern("Circle", &[]);

            // Spiral curriculum: track patterns to revisit
            let mut curriculum_progress = vec![
                (TargetPattern::Circle, false),     // Pattern, is_mastered
                (TargetPattern::Square, false),
                (TargetPattern::Cross, false),
                (TargetPattern::Spiral, false),
            ];
            let mut curriculum_cycle = 0;  // Which cycle through curriculum we're on
            let mut current_pattern_index = 0;

            loop {
                // Check if we should continue
                {
                    let s = state.lock().unwrap();
                    if !s.is_running {
                        break;
                    }
                }

                // ADAPTIVE TRAINING STRATEGY based on current pattern
                // Different patterns need different approaches
                pattern_attempts += 1;

                // META-LEARNING: Get adaptive learning rate and apply pattern-specific multipliers
                // Meta-learner adjusts based on loss trends, then we scale for pattern difficulty
                let base_adaptive_rate = meta_learner.get_learning_rate();
                learning_rate = match current_pattern_type {
                    TargetPattern::Circle => {
                        // Circle is easiest - use standard adaptive rate
                        base_adaptive_rate
                    },
                    TargetPattern::Square => {
                        // Square needs sharper edges - boost learning rate 2×
                        let boosted_rate = base_adaptive_rate * 2.0;

                        // If struggling (many attempts), provide extra help
                        if pattern_attempts > 500 && pattern_attempts % 100 == 0 {
                            let mut s = state.lock().unwrap();
                            s.add_event("[TIP] Tip: Square requires sharp corners - increasing precision training".to_string());
                        }

                        boosted_rate
                    },
                    TargetPattern::Cross => {
                        // Cross needs understanding of intersections - moderate boost
                        base_adaptive_rate * 1.6
                    },
                    TargetPattern::Spiral => {
                        // Spiral is most complex - slightly boosted rate
                        base_adaptive_rate * 1.2
                    },
                };

                // BALANCED TRAINING - Pattern-specific step counts
                let num_steps = match current_pattern_type {
                    TargetPattern::Circle => rand::random::<usize>() % 9 + 24,  // 24-32 steps
                    TargetPattern::Square => rand::random::<usize>() % 13 + 32, // 32-44 steps - needs more time for edges
                    TargetPattern::Cross => rand::random::<usize>() % 11 + 28,  // 28-38 steps
                    TargetPattern::Spiral => rand::random::<usize>() % 17 + 36, // 36-52 steps - most complex
                };

                // MULTI-THREADED OPTIMIZATION: Process 5 generations in parallel batches
                // This amortizes the lock overhead and allows better CPU utilization
                let batch_size = 5;
                let mut batch_losses = Vec::with_capacity(batch_size);

                for _ in 0..batch_size {
                    let loss = {
                        let mut nca_lock = nca.lock().unwrap();

                        // PATTERN-SPECIFIC INITIALIZATION
                        // For harder patterns, sometimes initialize closer to target
                        let use_guided_init = match current_pattern_type {
                            TargetPattern::Square | TargetPattern::Cross if pattern_attempts > 300 => {
                                rand::random::<f64>() < 0.3  // 30% chance of guided init
                            },
                            _ => false,
                        };

                        if use_guided_init {
                            // Initialize with hint of target pattern
                            initialize_grid_from_pattern(&mut nca_lock.grid, &current_target);
                            // Then add some noise so it still has to learn
                            for y in 0..nca_lock.grid.height {
                                for x in 0..nca_lock.grid.width {
                                    if rand::random::<f64>() < 0.3 {
                                        nca_lock.grid.cells[y][x][3] *= rand::random::<f64>() * 0.5;
                                    }
                                }
                            }
                        } else {
                            nca_lock.reset_with_seed();
                        }

                        // Evolve for num_steps (keeping lock - much faster!)
                        // Note: step() is already parallelized with rayon
                        for _ in 0..(num_steps - 1) {
                            nca_lock.step();
                        }

                        // Train on final state
                        let loss = nca_lock.train_step(&current_target, learning_rate);
                        nca_lock.step();  // Apply trained update

                        loss
                    };

                    batch_losses.push(loss);
                    step_count += 1;
                }

                // Use average loss from batch for stability
                let loss = batch_losses.iter().sum::<f64>() / batch_size as f64;

                // META-LEARNING: Record training step for adaptive learning rate adjustment
                // Use inverse loss as "accuracy" (higher is better) for meta-learner
                let pseudo_accuracy = 1.0 / (1.0 + loss);  // 0.5 at loss=1.0, approaches 1.0 as loss→0
                meta_learner.record_step(step_count as usize, loss, pseudo_accuracy);

                // META-LEARNING MANAGER: Update with current loss and record result
                let gradient_magnitude = loss.abs() * 0.1;  // Approximate gradient from loss
                let _manager_lr = meta_manager.get_learning_rate(loss, gradient_magnitude);
                {
                    let nca_lock = nca.lock().unwrap();
                    meta_manager.record_result(loss, &nca_lock);
                }

                // OPTIMIZATION: Only calculate expensive metrics every 10 steps for speed
                let should_update_metrics = step_count % 10 == 0;
                let (grid_data, diversity, complexity) = if should_update_metrics {
                    let nca_lock = nca.lock().unwrap();

                    // Convert NCA grid to 2D array for visualization (use alpha channel)
                    let grid: Vec<Vec<f64>> = nca_lock.grid.cells.iter()
                        .map(|row| row.iter().map(|cell| cell[3]).collect())  // Use channel 3 (alpha)
                        .collect();

                    // Calculate actual diversity (standard deviation of ALIVE cell values only)
                    // This measures pattern variety within the living organism, not the empty space
                    let alive_threshold = 0.1;
                    let mut alive_values = Vec::new();

                    for row in &nca_lock.grid.cells {
                        for cell in row {
                            let alpha = cell[3];
                            if alpha > alive_threshold {
                                alive_values.push(alpha);
                            }
                        }
                    }

                    let div = if alive_values.len() > 1 {
                        let mean = alive_values.iter().sum::<f64>() / alive_values.len() as f64;
                        let variance = alive_values.iter()
                            .map(|&v| (v - mean).powi(2))
                            .sum::<f64>() / alive_values.len() as f64;
                        variance.sqrt()
                    } else {
                        0.0  // No alive cells or only one, no diversity
                    };

                    // Calculate complexity (spatial variation between neighbors)
                    let mut total_diff = 0.0;
                    let mut diff_count = 0;
                    for y in 0..nca_lock.grid.height {
                        for x in 0..nca_lock.grid.width {
                            let val = nca_lock.grid.cells[y][x][3];
                            // Check right neighbor
                            if x + 1 < nca_lock.grid.width {
                                total_diff += (val - nca_lock.grid.cells[y][x + 1][3]).abs();
                                diff_count += 1;
                            }
                            // Check bottom neighbor
                            if y + 1 < nca_lock.grid.height {
                                total_diff += (val - nca_lock.grid.cells[y + 1][x][3]).abs();
                                diff_count += 1;
                            }
                        }
                    }
                    let comp = if diff_count > 0 { total_diff / diff_count as f64 } else { 0.0 };

                    (grid, div, comp)
                } else {
                    // Reuse previous metrics for speed
                    let s = state.lock().unwrap();
                    (s.grid_snapshot.clone(), s.nca_diversity, s.nca_complexity)
                };

                // Update state with new grid and metrics
                {
                    let mut s = state.lock().unwrap();
                    s.grid_snapshot = grid_data;
                    s.nca_generation = step_count;
                    s.nca_diversity = diversity;
                    s.nca_complexity = complexity;
                    s.current_loss = loss;

                    // Update current pattern being learned
                    s.nca_current_pattern = match current_pattern_type {
                        TargetPattern::Circle => "🔴 Circle",
                        TargetPattern::Square => "🟦 Square",
                        TargetPattern::Cross => "➕ Cross",
                        TargetPattern::Spiral => "Spiral",
                    }.to_string();

                    // Update meta-learning stats from manager
                    let meta_stats = meta_manager.get_stats();
                    s.meta_learning_strategy = meta_stats.current_strategy;
                    s.strategy_confidence = meta_stats.strategy_confidence;
                    s.strategy_reasoning = meta_stats.strategy_reasoning;
                    s.learning_rate = meta_stats.current_lr;
                    s.warmup_complete = meta_stats.warmup_complete;
                    s.lr_cycle_progress = meta_stats.lr_cycle_progress;
                    s.reptile_meta_steps = meta_stats.reptile_meta_steps;
                    s.reptile_avg_loss = meta_stats.avg_loss;
                    s.pbt_evolutions = meta_stats.pbt_generations;
                    s.pbt_best_score = meta_stats.pbt_best_score;
                    s.pbt_best_lr = meta_stats.pbt_best_lr;
                    s.arch_neurons = meta_stats.architecture_neurons;
                    s.arch_modifications = meta_stats.architecture_modifications;
                    s.arch_rollbacks = meta_stats.architecture_rollbacks;

                    // Drain and add meta-learning events
                    for event in meta_manager.drain_events() {
                        let event_str = match event {
                            MetaLearningEvent::StrategySelected { strategy, reason } =>
                                format!("[Strategy] {} - {}", strategy, reason),
                            MetaLearningEvent::LearningRateAdjusted { old_lr, new_lr, reason } =>
                                format!("[LR] {:.6} → {:.6}: {}", old_lr, new_lr, reason),
                            MetaLearningEvent::ReptileMetaStep { avg_loss, tasks } =>
                                format!("[Reptile] Meta-step: {} tasks, avg loss {:.4}", tasks, avg_loss),
                            MetaLearningEvent::PBTEvolution { generation, best_score } =>
                                format!("[PBT] Gen {}: best score {:.4}", generation, best_score),
                            MetaLearningEvent::ArchitectureChange { description, applied } =>
                                format!("[Arch] {}{}", if applied { "✓ " } else { "? " }, description),
                            MetaLearningEvent::CurriculumAdvanced { pattern, difficulty } =>
                                format!("[Curriculum] {} (difficulty: {:.2})", pattern, difficulty),
                            MetaLearningEvent::Milestone { message } =>
                                format!("[Milestone] {}", message),
                        };
                        s.meta_learning_events.push(event_str.clone());
                        s.add_event(event_str);
                    }
                    // Keep meta_learning_events bounded
                    while s.meta_learning_events.len() > 20 {
                        s.meta_learning_events.remove(0);
                    }

                    // Update SpacetimeDB every generation (real-time sync)
                    let _ = db_client.update_sage_state(
                        step_count,
                        loss,
                        &s.nca_current_pattern,
                        complexity,
                        diversity,
                    );

                    // Add detailed events for different milestones
                    if step_count == 1 {
                        s.add_event("🌱 NCA initialized - cells beginning to learn".to_string());
                        let _ = db_client.log_training_event(step_count, "milestone", "🌱 NCA initialized");
                    } else if step_count == 10 {
                        s.add_event(format!("First training cycle complete - loss: {:.4}", loss));
                        let _ = db_client.log_training_event(step_count, "milestone", &format!("First cycle - loss: {:.4}", loss));
                    } else if step_count % 100 == 0 {
                        s.add_event(format!("Generation {} - loss: {:.4}, diversity: {:.3}", step_count, loss, diversity));
                        let _ = db_client.log_training_event(step_count, "milestone", &format!("Gen {} - loss: {:.4}", step_count, loss));
                    } else if step_count % 50 == 0 {
                        let pattern_name = match current_pattern_type {
                            TargetPattern::Circle => "Circle",
                            TargetPattern::Square => "Square",
                            TargetPattern::Cross => "Cross",
                            TargetPattern::Spiral => "Spiral",
                        };
                        s.add_event(format!("[TARGET] {} learning - {} steps, loss: {:.4}", pattern_name, steps_on_target, loss));
                    }

                    // Learning quality events
                    if loss < 0.03 && low_loss_streak == 1 {
                        s.add_event("✨ High quality learning achieved (loss < 0.03)".to_string());
                    } else if low_loss_streak == 25 {
                        s.add_event("🎓 Pattern stabilizing - 25 steps of consistent learning".to_string());
                    } else if low_loss_streak == 50 {
                        s.add_event("🏆 Pattern forming well - 50 step streak".to_string());
                    } else if low_loss_streak == 75 {
                        s.add_event("Approaching mastery - 75 consecutive good steps".to_string());
                    } else if low_loss_streak == 100 {
                        s.add_event("[TARGET] MASTERY ACHIEVED! Moving to next pattern...".to_string());
                    }

                    // Save checkpoint every 100 generations for quick resume
                    if step_count % 100 == 0 && step_count > 0 {
                        let _nca_lock = nca.lock().unwrap();
                        let weights_json = "{}"; // TODO: Serialize actual weights
                        let _ = db_client.save_network_snapshot(
                            step_count,
                            &s.nca_current_pattern,
                            loss,
                            weights_json
                        );
                        s.add_event(format!("Checkpoint saved at generation {}", step_count));
                    }
                }

                // Check for successful repair after damage
                if last_damage_step > 0 && step_count == last_damage_step + 50 && loss < 0.02 {
                    let mut s = state.lock().unwrap();
                    s.add_event("[OK] Self-repair successful! Pattern restored.".to_string());
                    last_damage_step = 0;  // Reset
                }

                // Track learning progress and advance patterns
                // Relaxed threshold: 0.1 is good for general pattern recognition
                // With batch processing, we need a more forgiving threshold
                if loss < 0.1 {
                    low_loss_streak += batch_size as u64;
                } else {
                    low_loss_streak = 0;
                }

                // Show progress updates every 25 generations
                if low_loss_streak > 0 && low_loss_streak % 25 == 0 {
                    let mut s = state.lock().unwrap();
                    let progress_pct = (low_loss_streak as f64 / 50.0 * 100.0).min(100.0);
                    s.add_event(format!("Pattern mastery: {:.0}% ({}/50 streak)", progress_pct, low_loss_streak));
                }

                // Show spiral curriculum progress every 25 attempts
                if pattern_attempts > 0 && pattern_attempts % 25 == 0 && low_loss_streak < 50 {
                    let mut s = state.lock().unwrap();
                    s.add_event(format!("[SYNC] Spiral curriculum: {}/100 attempts (will move on if not mastered)", pattern_attempts));
                }

                // SPIRAL CURRICULUM: Move on after either mastery OR enough attempts
                // This allows SAGE to gain experience elsewhere and return later
                // 100 attempts = 500 generations (with batch size 5)
                let should_progress = low_loss_streak >= 50 || pattern_attempts >= 100;

                if should_progress {
                    // Mark current pattern as mastered if we actually mastered it
                    if low_loss_streak >= 50 {
                        curriculum_progress[current_pattern_index].1 = true;
                    }

                    // Find next pattern to learn (circular with revisiting)
                    let mut next_pattern_found = false;
                    for _attempt in 0..curriculum_progress.len() {
                        current_pattern_index = (current_pattern_index + 1) % curriculum_progress.len();

                        // Skip already mastered patterns unless we've cycled through all
                        if !curriculum_progress[current_pattern_index].1 || curriculum_cycle > 0 {
                            next_pattern_found = true;
                            break;
                        }
                    }

                    // If all patterns mastered in first cycle, start second cycle (revisit with new knowledge)
                    if !next_pattern_found || curriculum_progress.iter().all(|(_, mastered)| *mastered) {
                        curriculum_cycle += 1;
                        current_pattern_index = 0;

                        if curriculum_cycle == 1 {
                            let mut s = state.lock().unwrap();
                            s.add_event("[SYNC] SPIRAL LEARNING: Revisiting all patterns with new knowledge!".to_string());
                        }
                    }

                    let next_pattern = Some(curriculum_progress[current_pattern_index].0);

                    if let Some(next) = next_pattern {
                        let old_pattern_name = match current_pattern_type {
                            TargetPattern::Circle => "🔴 Circle",
                            TargetPattern::Square => "🟦 Square",
                            TargetPattern::Cross => "➕ Cross",
                            TargetPattern::Spiral => "Spiral",
                        };

                        // Save knowledge before transitioning (transfer learning)
                        {
                            let mut s = state.lock().unwrap();
                            if low_loss_streak >= 50 {
                                s.add_event(format!("[OK] {} mastered! Saving knowledge", old_pattern_name));
                            } else {
                                s.add_event(format!("[SYNC] Moving from {} (will revisit with new skills)", old_pattern_name));
                            }
                        }

                        if low_loss_streak >= 50 {
                            let _ = db_client.master_pattern(old_pattern_name, step_count, loss);
                            let _ = db_client.log_training_event(step_count, "pattern_mastered", &format!("{} mastered!", old_pattern_name));
                        } else {
                            let _ = db_client.log_training_event(step_count, "pattern_deferred", &format!("{} deferred for spiral learning", old_pattern_name));
                        }

                        {
                            let _s = state.lock().unwrap();

                            let mut nca_lock = nca.lock().unwrap();
                            nca_lock.save_knowledge();
                        }

                        current_pattern_type = next;
                        current_target = generate_target_pattern(current_pattern_type);
                        low_loss_streak = 0;
                        steps_on_target = 0;
                        pattern_attempts = 0;  // Reset for new pattern

                        let new_pattern_name = match current_pattern_type {
                            TargetPattern::Circle => "🔴 Circle",
                            TargetPattern::Square => "🟦 Square",
                            TargetPattern::Cross => "➕ Cross",
                            TargetPattern::Spiral => "Spiral",
                        };

                        // META-LEARNING MANAGER: Start new pattern
                        let loss_history: Vec<f64> = batch_losses.iter().copied().collect();
                        let _ = meta_manager.start_pattern(new_pattern_name, &loss_history);

                        let mut s = state.lock().unwrap();
                        if curriculum_cycle > 0 {
                            s.add_event(format!("[SYNC] Revisiting: {} (Cycle {})", new_pattern_name, curriculum_cycle + 1));
                            s.add_event("[TIP] Applying knowledge from other patterns".to_string());
                        } else {
                            s.add_event(format!("[TARGET] New challenge: {}", new_pattern_name));
                            s.add_event("[SYNC] Initializing grid with new pattern template".to_string());
                        }

                        // Initialize grid with new pattern
                        let mut nca_lock = nca.lock().unwrap();
                        initialize_grid_from_pattern(&mut nca_lock.grid, &current_target);
                    } else {
                        // Mastered all patterns!
                        let mut s = state.lock().unwrap();
                        s.add_event("Spiral mastered - completing full curriculum!".to_string());
                        s.add_event("🎊 CURRICULUM COMPLETE! All patterns learned.".to_string());
                        s.add_event("🌌 Entering free-form evolution mode...".to_string());

                        // Continue evolving without a target
                        low_loss_streak = 0;  // Reset to prevent constant triggering
                    }
                }

                // Check for incoming messages from Claude and respond
                if step_count % 20 == 0 {  // Check every ~1 second (20 steps * 50ms)
                    let messages = comm_handler.check_inbox();
                    for message in messages {
                        let response = {
                            let s = state.lock().unwrap();
                            comm_handler.process_message(&message, &s)
                        };

                        comm_handler.send_message(response.clone());

                        let mut s = state.lock().unwrap();
                        // Add to chat history
                        s.chat_history.push((message.sender.clone(), message.content.clone()));
                        s.chat_history.push(("SAGE".to_string(), response.clone()));

                        // Also add summary to events
                        s.add_event(format!(" {} asked a question", message.sender));
                        s.add_event("🤖 SAGE responded".to_string());
                    }
                }

                steps_on_target += batch_size as u64;

                // Sleep to control evolution speed - BALANCED for stability
                // 20ms provides good visual feedback and reduces CPU load
                thread::sleep(std::time::Duration::from_millis(20));
            }  // End of NCA training loop
        });
    }

    /// Start training with hot-reload support (NEW!)
    pub fn start_training_with_hot_reload(&self) {
        let state = Arc::clone(&self.state);

        thread::spawn(move || {
            // Initialize
            {
                let mut s = state.lock().unwrap();
                s.is_running = true;  // Set running state!
                s.add_event("Hot-reload training engine starting...".to_string());
            }

            // Load engine
            let mut loader = match super::engine_loader::EngineLoader::new() {
                Ok(l) => l,
                Err(e) => {
                    let mut s = state.lock().unwrap();
                    s.add_event(format!("❌ Failed to create loader: {}", e));
                    return;
                }
            };

            let mut engine = match loader.load() {
                Ok(e) => e,
                Err(e) => {
                    let mut s = state.lock().unwrap();
                    s.add_event(format!("❌ Failed to load engine: {}", e));
                    return;
                }
            };

            {
                let mut s = state.lock().unwrap();
                s.add_event(format!("[OK] Engine loaded! Version: {}", loader.version()));
            }

            // Initialize SpacetimeDB
            let mut db_client = SageDbClient::new("sage-db");
            let _ = db_client.connect();

            // Training loop
            let mut iteration = 0;
            let batch_size = 10;  // Process 10 steps per iteration

            loop {
                // Check if should stop
                {
                    let s = state.lock().unwrap();
                    if !s.is_running {
                        break;
                    }
                }

                // Check for hot-reload
                if loader.check_for_reload() {
                    let mut s = state.lock().unwrap();
                    s.add_event("[SYNC] LIBRARY CHANGED! Hot-reloading...".to_string());

                    // Save checkpoint
                    match engine.save_checkpoint() {
                        Ok(checkpoint) => {
                            s.add_event(format!("   Checkpoint saved: Gen {}", checkpoint.generation));
                            drop(s);  // Release lock before reload

                            // Reload engine
                            match loader.load() {
                                Ok(new_engine) => {
                                    engine = new_engine;
                                    let mut s = state.lock().unwrap();
                                    s.add_event(format!("   📚 New engine loaded! Version: {}", loader.version()));

                                    // Restore checkpoint
                                    match engine.load_checkpoint(checkpoint) {
                                        Ok(_) => {
                                            s.add_event("   [OK] HOT RELOAD COMPLETE!".to_string());
                                        }
                                        Err(e) => {
                                            s.add_event(format!("   [WARN]  Checkpoint restore failed: {}", e));
                                        }
                                    }
                                }
                                Err(e) => {
                                    let mut s = state.lock().unwrap();
                                    s.add_event(format!("   ❌ Failed to reload engine: {}", e));
                                }
                            }
                        }
                        Err(e) => {
                            s.add_event(format!("   ❌ Failed to save checkpoint: {}", e));
                        }
                    }
                }

                // Run training iteration
                let update = engine.step(batch_size);

                // Update state
                {
                    let mut s = state.lock().unwrap();

                    // Update core metrics
                    s.nca_generation = update.generation;
                    s.current_loss = update.loss;
                    s.nca_complexity = update.complexity;
                    s.nca_diversity = update.diversity;
                    s.nca_current_pattern = update.current_pattern.clone();
                    s.grid_snapshot = update.grid_snapshot.clone();

                    // Update advanced analytics
                    s.loss_velocity = update.loss_velocity;
                    s.loss_acceleration = update.loss_acceleration;
                    s.per_pattern_losses = update.per_pattern_losses.clone();
                    s.pattern_attempts = update.pattern_attempts;
                    s.estimated_time_to_mastery = update.estimated_time_to_mastery;
                    s.learning_rate = update.learning_rate;
                    s.batch_size = update.batch_size;
                    s.evolution_steps = update.evolution_steps;
                    s.low_loss_streak = update.low_loss_streak;
                    s.patterns_mastered = update.patterns_mastered;

                    // Update batch progress
                    s.current_batch = update.current_batch;
                    s.total_batches = update.total_batches;
                    s.batch_losses = update.batch_losses.clone();

                    // Add events from engine
                    for event in update.events {
                        s.add_event(event);
                    }
                }

                // Update SpacetimeDB every 10 iterations
                if iteration % 10 == 0 {
                    let s = state.lock().unwrap();
                    let _ = db_client.update_sage_state(
                        s.nca_generation,
                        s.current_loss,
                        &s.nca_current_pattern,
                        s.nca_complexity,
                        s.nca_diversity,
                    );
                }

                iteration += 1;

                // Small sleep to avoid hammering the lock
                thread::sleep(std::time::Duration::from_millis(50));
            }

            let mut s = state.lock().unwrap();
            s.add_event("🛑 Training stopped".to_string());
        });
    }

    pub fn pause(&self) {
        let mut s = self.state.lock().unwrap();
        s.is_running = false;
        s.add_event("Evolution paused".to_string());
    }

    pub fn get_state(&self) -> TrainingState {
        self.state.lock().unwrap().clone()
    }
}

impl Clone for TrainingState {
    fn clone(&self) -> Self {
        Self {
            current_phase: self.current_phase,
            phase1_progress: self.phase1_progress,
            phase2_progress: self.phase2_progress,
            phase3_progress: self.phase3_progress,
            phase4_progress: self.phase4_progress,
            phase5_progress: self.phase5_progress,
            is_running: self.is_running,
            recent_events: self.recent_events.clone(),
            debug_logs: self.debug_logs.clone(),
            grid_snapshot: self.grid_snapshot.clone(),
            previous_grid_snapshot: self.previous_grid_snapshot.clone(),
            grid_update_time: self.grid_update_time,
            current_loss: self.current_loss,
            learning_rate: self.learning_rate,
            active_thought_count: self.active_thought_count,
            active_decision_count: self.active_decision_count,
            performance: self.performance,
            concept_count: self.concept_count,
            neural_classifier_accuracy: self.neural_classifier_accuracy,
            curiosity_total_explorations: self.curiosity_total_explorations,
            curiosity_avg_novelty: self.curiosity_avg_novelty,
            curiosity_total_reward: self.curiosity_total_reward,
            hierarchical_total_concepts: self.hierarchical_total_concepts,
            hierarchical_level_counts: self.hierarchical_level_counts.clone(),
            meta_learning_rate: self.meta_learning_rate,
            meta_curriculum_difficulty: self.meta_curriculum_difficulty,
            meta_learning_efficiency: self.meta_learning_efficiency,
            nca_generation: self.nca_generation,
            nca_diversity: self.nca_diversity,
            nca_complexity: self.nca_complexity,
            nca_current_pattern: self.nca_current_pattern.clone(),
            chat_history: self.chat_history.clone(),

            // Clone advanced analytics
            loss_velocity: self.loss_velocity,
            loss_acceleration: self.loss_acceleration,
            per_pattern_losses: self.per_pattern_losses.clone(),
            pattern_attempts: self.pattern_attempts,
            estimated_time_to_mastery: self.estimated_time_to_mastery,
            batch_size: self.batch_size,
            evolution_steps: self.evolution_steps,
            low_loss_streak: self.low_loss_streak,
            patterns_mastered: self.patterns_mastered,

            // Clone batch progress
            current_batch: self.current_batch,
            total_batches: self.total_batches,
            batch_losses: self.batch_losses.clone(),

            // Clone curriculum info
            total_patterns: self.total_patterns,

            // Clone training history
            metrics_history: self.metrics_history.clone(),

            // Clone meta-learning fields
            meta_learning_strategy: self.meta_learning_strategy.clone(),
            strategy_confidence: self.strategy_confidence,
            strategy_reasoning: self.strategy_reasoning.clone(),
            warmup_complete: self.warmup_complete,
            lr_cycle_progress: self.lr_cycle_progress,
            reptile_meta_steps: self.reptile_meta_steps,
            reptile_avg_loss: self.reptile_avg_loss,
            pbt_population_size: self.pbt_population_size,
            pbt_evolutions: self.pbt_evolutions,
            pbt_best_score: self.pbt_best_score,
            pbt_best_lr: self.pbt_best_lr,
            arch_neurons: self.arch_neurons,
            arch_modifications: self.arch_modifications,
            arch_rollbacks: self.arch_rollbacks,
            meta_learning_events: self.meta_learning_events.clone(),
        }
    }
}
// END OF FILE - Old phase code removed
