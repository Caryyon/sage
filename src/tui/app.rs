// Application state and event loop for SAGE Mission Control

use crossterm::event::{self, Event, KeyCode, KeyEvent};
use ratatui::{
    backend::Backend,
    Terminal,
};
use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};
use crate::agi::AGISystem;
use crate::autonomous::AutonomousTrainer;
use crate::sage_experience::SageExperience;
use crate::spacetime_client::SageDbClient;
use crate::irc_manager::IrcManager;
use crate::sonification::SonificationEngine;
use crate::audio_input::AudioInputEngine;
use super::screens::{Screen, ScreenType};
use super::training::{TrainingRunner, TrainingState};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Phase {
    Idle,
    Phase1Primitives,
    Phase2Geometric,
    Phase3Civilization,
    Phase4Curiosity,
    Phase5SelfImprovement,
    Phase6NCAEmergence,  // NEW: Neural Cellular Automata
    Complete,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HealthStatus {
    Healthy,
    Warning,
    Critical,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PanelType {
    ActivityFeed,
    DetailPanel,
    MindState,
    HealthDiagnostics,
}

pub struct PanelState {
    pub activity_feed_visible: bool,
    pub detail_panel_visible: bool,
    pub mind_state_visible: bool,
    pub health_diagnostics_visible: bool,
}

impl PanelState {
    pub fn new() -> Self {
        Self {
            activity_feed_visible: false,
            detail_panel_visible: false,
            mind_state_visible: false,
            health_diagnostics_visible: false,
        }
    }

    pub fn toggle(&mut self, panel: PanelType) {
        match panel {
            PanelType::ActivityFeed => self.activity_feed_visible = !self.activity_feed_visible,
            PanelType::DetailPanel => self.detail_panel_visible = !self.detail_panel_visible,
            PanelType::MindState => self.mind_state_visible = !self.mind_state_visible,
            PanelType::HealthDiagnostics => self.health_diagnostics_visible = !self.health_diagnostics_visible,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TrainingMode {
    Idle,              // No training happening
    PatternTraining,   // Training on geometric patterns (Circle, Square, Cross, Spiral)
    BaselineTraining,  // Training on positive concepts at startup
    IrcLearning,       // Learning from IRC messages
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConsciousnessState {
    Active,      // Actively learning from interactions
    Dreaming,    // Dream Mode - memory consolidation
    Curious,     // Curiosity Mode - autonomous goal pursuit
}

/// Performance mode controls CPU usage vs training speed
/// Visuals always render at 60fps - training is throttled via frame skipping
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PerformanceMode {
    Eco,      // ~20% CPU: batch=2, train every 6th frame (10 train/sec)
    Low,      // ~40% CPU: batch=4, train every 3rd frame (20 train/sec)
    Medium,   // ~60% CPU: batch=6, train every 2nd frame (30 train/sec)
    High,     // ~80% CPU: batch=8, train every frame (60 train/sec)
    Max,      // 100% CPU: batch=8, train every frame (60 train/sec)
}

impl PerformanceMode {
    pub fn batch_size(&self) -> usize {
        match self {
            PerformanceMode::Eco => 2,
            PerformanceMode::Low => 4,
            PerformanceMode::Medium => 6,
            PerformanceMode::High | PerformanceMode::Max => 8,
        }
    }

    /// How many frames to skip between training iterations
    /// Visuals always render at 60fps, but training is throttled
    pub fn train_every_n_frames(&self) -> u64 {
        match self {
            PerformanceMode::Eco => 6,    // Train every 6th frame = 10 training/sec
            PerformanceMode::Low => 3,    // Train every 3rd frame = 20 training/sec
            PerformanceMode::Medium => 2, // Train every 2nd frame = 30 training/sec
            PerformanceMode::High => 1,   // Train every frame = 60 training/sec
            PerformanceMode::Max => 1,    // Train every frame = 60 training/sec
        }
    }

    /// Visuals always at 60fps for smooth experience
    pub fn target_fps(&self) -> u64 {
        60  // Always 60fps for smooth visuals
    }

    pub fn next(&self) -> Self {
        match self {
            PerformanceMode::Eco => PerformanceMode::Low,
            PerformanceMode::Low => PerformanceMode::Medium,
            PerformanceMode::Medium => PerformanceMode::High,
            PerformanceMode::High => PerformanceMode::Max,
            PerformanceMode::Max => PerformanceMode::Eco,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            PerformanceMode::Eco => "ECO (~20%)",
            PerformanceMode::Low => "LOW (~40%)",
            PerformanceMode::Medium => "MED (~60%)",
            PerformanceMode::High => "HIGH (~80%)",
            PerformanceMode::Max => "MAX (100%)",
        }
    }
}

/// Shared state between training thread and render thread
/// Training thread writes, render thread reads
#[derive(Clone)]
pub struct TrainingSnapshot {
    pub grid: Vec<Vec<f64>>,           // 32x32 NCA grid (alpha channel)
    pub target_grid: Vec<Vec<f64>>,    // Target pattern grid
    pub loss: f64,
    pub generation: u64,
    pub pattern_name: String,
    pub pattern_index: usize,
    pub pattern_total: usize,
    pub iteration: usize,
    pub total_iterations: usize,
    pub is_damage_phase: bool,
    pub mastery_status: Vec<(String, bool, f64)>,  // (name, mastered, best_loss)
    pub events: Vec<String>,           // Recent training events
    pub metrics_history: Vec<MetricPoint>,  // Last 250 data points for charts
    pub low_loss_streak: usize,        // Consecutive iterations with loss below threshold
    pub pattern_attempts: usize,       // Total attempts on current pattern
}

/// Single data point for training history charts
#[derive(Clone, Debug)]
pub struct MetricPoint {
    pub generation: u64,
    pub loss: f64,
    pub diversity: f64,
    pub complexity: f64,
    pub stability: f64,  // Drift during stability phase (lower = more stable)
    pub pattern: String,
}

impl Default for TrainingSnapshot {
    fn default() -> Self {
        Self {
            grid: vec![vec![0.0; 32]; 32],
            target_grid: vec![vec![0.0; 32]; 32],
            loss: 1.0,
            generation: 0,
            pattern_name: "Circle".to_string(),
            pattern_index: 0,
            pattern_total: 9,
            iteration: 0,
            total_iterations: 100,
            is_damage_phase: false,
            mastery_status: vec![],
            events: vec![],
            metrics_history: Vec::with_capacity(250),
            low_loss_streak: 0,
            pattern_attempts: 0,
        }
    }
}

pub struct AppState {
    pub current_screen: ScreenType,
    pub current_phase: Phase,
    pub health_status: HealthStatus,
    pub panels: PanelState,
    pub is_paused: bool,
    pub uptime_seconds: u64,
    pub agi: AGISystem,
    pub autonomous_trainer: AutonomousTrainer,
    pub training_state: TrainingState,
    pub sage: SageExperience,  // SAGE consciousness system
    pub memory: SageDbClient,  // Persistent memory
    pub irc_manager: Option<IrcManager>,  // IRC bot (runs in background)
    pub irc_messages: Vec<(String, String, String)>,  // (sender, message, sage_response)
    pub chat_history: Vec<(String, String)>,  // (sender, message) pairs
    pub current_input: String,
    pub last_agi_thought_time: u64,  // Time since last proactive thought
    pub last_claude_observation_time: u64,  // Time since last Claude observation
    pub last_save_time: u64,  // Time since last state save
    pub evolution_mode: bool,  // True when Claude is actively evolving SAGE
    pub request_hot_reload: bool,  // Signal from UI to trigger manual hot-reload
    pub training_mode: TrainingMode,  // What kind of training is happening
    pub baseline_concepts: Vec<String>,  // Concepts for baseline training
    pub baseline_current_concept: usize,  // Current concept index
    pub baseline_current_iteration: usize,  // Current iteration within concept
    pub baseline_total_iterations: usize,  // Iterations per concept
    // Pattern training state
    pub pattern_sequence: Vec<String>,  // Sequence of patterns to train (Circle, Square, Cross, Spiral)
    pub pattern_current_index: usize,  // Current pattern index
    pub pattern_current_iteration: usize,  // Current iteration within pattern
    pub pattern_total_iterations: usize,  // Iterations per pattern
    pub pattern_target_grid: Option<crate::grid::Grid>,  // Current target pattern
    pub pattern_mastery_status: Vec<(String, bool, f64)>,  // (pattern_name, is_mastered, best_loss)
    // Brain Monitor animation state
    pub brain_pulse_phase: f64,  // Global animation phase
    pub brain_cell_offsets: Vec<f64>,  // Per-cell pulse offsets (1024 for 32×32)
    pub brain_activity_map: Vec<f64>,  // Recent activity intensity per cell
    pub brain_frame_count: u64,  // Frame counter for timing
    // Autonomous consciousness tracking
    pub consciousness_state: ConsciousnessState,  // Current consciousness state
    pub last_activity_time: Instant,  // Last time SAGE had external interaction
    pub idle_seconds: u64,  // How long SAGE has been idle
    // Vision system tracking
    pub camera_frame: Option<Vec<Vec<(u8, u8, u8)>>>,  // Last captured camera frame (RGB)
    pub visual_concepts: Vec<String>,  // Active visual concepts SAGE is experiencing
    pub autonomous_activities: Vec<(u64, String, String)>,  // (timestamp, type, description) - dream/curiosity log
    // Audio status (for display only, engine is in App)
    pub audio_available: bool,  // Is audio engine available?
    pub audio_enabled: bool,  // Is audio playback enabled?
    pub audio_volume: f32,  // Volume level (0.0 - 1.0)
    pub audio_input_available: bool,  // Is audio input engine available?
    pub audio_listening: bool,  // Is audio input listening enabled?
    // Vision status
    pub vision_available: bool,  // Is camera available?
    pub vision_enabled: bool,  // Is vision capture enabled?
    pub vision_fps: f64,  // Current vision FPS
    // Performance settings
    pub performance_mode: PerformanceMode,  // CPU usage vs training speed tradeoff
}

impl AppState {
    pub fn new() -> Self {
        let mut sage = SageExperience::new();

        // Try to load existing SAGE knowledge and state
        let _ = sage.load_knowledge("sage_positive_knowledge.json");
        let _ = sage.load_preferences("sage_preferences.json");
        let _ = sage.load_associations("sage_associations.json");
        let _ = sage.load_curiosity("sage_curiosity.json");

        // Try to load pattern training weights (restored from previous session)
        // Silently load - TUI will show status
        let _ = sage.get_nca_mut().load_weights_from_file("pattern_training_weights.json");

        // Start IRC bot automatically in background thread
        // DISABLED: We're using sage_irc_autonomous as a separate process instead
        let irc_manager = None; // Some(IrcManager::start());

        // Baseline training concepts
        let baseline_concepts = vec![
            "love", "joy", "peace", "harmony", "beauty",
            "truth", "wisdom", "kindness", "compassion", "courage",
            "creativity", "innovation", "learning", "growth", "understanding",
            "hope", "faith", "trust", "friendship", "connection",
            "music", "art", "poetry", "dance", "song",
            "light", "warmth", "comfort", "safety", "home",
            "hello", "hi", "greetings", "welcome", "thanks",
        ].iter().map(|s| s.to_string()).collect();

        Self {
            current_screen: ScreenType::BrainMonitor,  // Start with primary training visualization
            current_phase: Phase::Idle,
            health_status: HealthStatus::Healthy,
            panels: PanelState::new(),
            is_paused: false,
            uptime_seconds: 0,
            agi: AGISystem::new(),
            autonomous_trainer: AutonomousTrainer::new(),
            training_state: TrainingState::new(),
            sage,
            memory: SageDbClient::new("sage-db"),
            irc_manager,  // IRC bot runs in background thread
            irc_messages: Vec::new(),
            chat_history: Vec::new(),
            current_input: String::new(),
            last_agi_thought_time: 0,
            last_claude_observation_time: 0,
            last_save_time: 0,
            evolution_mode: false,
            request_hot_reload: false,
            training_mode: TrainingMode::PatternTraining,  // Auto-start pattern training
            baseline_concepts,
            baseline_current_concept: 0,
            baseline_current_iteration: 0,
            baseline_total_iterations: 100,
            // Pattern training initialization - Tier 1 (basic) + Tier 2 (advanced)
            pattern_sequence: vec![
                // Tier 1: Basic shapes
                "Circle".to_string(),
                "Square".to_string(),
                "Cross".to_string(),
                "Spiral".to_string(),
                // Tier 2: Advanced shapes
                "Triangle".to_string(),
                "Star".to_string(),
                "Ring".to_string(),
                "Hexagon".to_string(),
                "Checkerboard".to_string(),
            ],
            pattern_current_index: 0,
            pattern_current_iteration: 0,
            pattern_total_iterations: 100,
            pattern_target_grid: None,
            pattern_mastery_status: vec![
                // Tier 1
                ("Circle".to_string(), false, 1.0),
                ("Square".to_string(), false, 1.0),
                ("Cross".to_string(), false, 1.0),
                ("Spiral".to_string(), false, 1.0),
                // Tier 2
                ("Triangle".to_string(), false, 1.0),
                ("Star".to_string(), false, 1.0),
                ("Ring".to_string(), false, 1.0),
                ("Hexagon".to_string(), false, 1.0),
                ("Checkerboard".to_string(), false, 1.0),
            ],
            // Brain Monitor animation state initialization
            brain_pulse_phase: 0.0,
            brain_cell_offsets: {
                use rand::Rng;
                let mut rng = rand::thread_rng();
                (0..1024).map(|_| rng.gen::<f64>() * 2.0 * std::f64::consts::PI).collect()
            },
            brain_activity_map: vec![0.0; 1024],
            brain_frame_count: 0,
            // Autonomous consciousness initialization
            consciousness_state: ConsciousnessState::Active,
            last_activity_time: Instant::now(),
            idle_seconds: 0,
            // Vision system initialization
            camera_frame: None,
            visual_concepts: Vec::new(),
            autonomous_activities: Vec::new(),
            // Audio status initialization
            audio_available: false,
            audio_enabled: true,
            audio_volume: 0.3,
            audio_input_available: false,
            audio_listening: false,
            vision_available: false,
            vision_enabled: true,
            vision_fps: 0.0,
            // Performance settings - default to Medium for balanced CPU usage
            performance_mode: PerformanceMode::Medium,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Action {
    None,
    SwitchScreen(ScreenType),
    TogglePanel(PanelType),
    PauseResume,
    StartTraining,
    ExportData,
    HotReload,  // NEW: Trigger hot-reload manually
    AudioToggle,  // NEW: Toggle audio output on/off
    AudioVolumeUp,  // NEW: Increase volume
    AudioVolumeDown,  // NEW: Decrease volume
    AudioListenToggle,  // NEW: Toggle audio input (listening) on/off
    VisionToggle,  // NEW: Toggle vision capture on/off
    CyclePerformanceMode,  // NEW: Cycle through performance modes (Eco/Low/Med/High/Max)
    Quit,
}

pub struct BootstrapState {
    pub samples_generated: usize,
    pub total_samples: usize,
    pub current_epoch: usize,
    pub total_epochs: usize,
    pub current_loss: f64,
    pub loss_history: Vec<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InitState {
    ShowingDemo,      // Showing demo grid while initializing
    Initializing,     // Background thread is working
    Ready,            // Fully initialized
}

pub struct App {
    pub state: AppState,
    pub should_quit: bool,
    pub start_time: Instant,
    pub training_runner: TrainingRunner,
    pub init_state: InitState,
    pub init_progress: String,
    pub bootstrap_state: Option<BootstrapState>,
    pub demo_grid: Vec<Vec<f64>>,  // Demo data for instant launch
    pub use_hot_reload: bool,  // Flag to use hot-reload system
    // Background training thread
    pub training_snapshot: Arc<Mutex<TrainingSnapshot>>,  // Shared state for real-time grid updates
    pub training_running: Arc<AtomicBool>,                // Control flag for training thread
    pub training_paused: Arc<AtomicBool>,                 // Pause flag
    pub training_thread: Option<JoinHandle<()>>,          // Background training thread handle
    // Audio sonification (output)
    pub audio_engine: Option<SonificationEngine>,  // Audio output engine (may fail to initialize)
    pub audio_enabled: bool,  // Is audio playback enabled?
    pub audio_volume: f32,  // Volume level (0.0 - 1.0)
    pub last_sonification_time: Instant,  // Throttle audio updates
    // Audio input (listening)
    pub audio_input_engine: Option<AudioInputEngine>,  // Audio input engine (may fail to initialize)
    pub audio_listening: bool,  // Is audio listening enabled?
    pub last_audio_input_time: Instant,  // Throttle audio input processing
    // Vision system (camera)
    pub vision_engine: Option<crate::vision::SageVision>,  // Camera capture system (may fail to initialize)
    pub vision_enabled: bool,  // Is vision capture enabled?
    pub last_vision_capture_time: Instant,  // Throttle to 30 FPS
    pub vision_frame_counter: u64,  // Frame counter for debugging
}

impl App {
    /// Generate a synthetic demo grid for instant visualization
    fn generate_demo_grid() -> Vec<Vec<f64>> {
        let size = 32;
        let mut grid = vec![vec![0.0; size]; size];
        let center_x = size as f64 / 2.0;
        let center_y = size as f64 / 2.0;

        // Create an organic, flowing pattern with multiple waves
        for y in 0..size {
            for x in 0..size {
                let dx = x as f64 - center_x;
                let dy = y as f64 - center_y;
                let dist = (dx * dx + dy * dy).sqrt();
                let angle = dy.atan2(dx);

                // Combine multiple wave patterns for organic feel
                let wave1 = (dist * 0.3 + angle * 2.0).sin();
                let wave2 = (dist * 0.15 - angle * 1.5).cos();
                let radial = (-dist * dist / 200.0).exp();

                grid[y][x] = (wave1 * 0.4 + wave2 * 0.3 + radial * 0.5)
                    .max(0.0)
                    .min(1.0);
            }
        }

        grid
    }

    pub fn new() -> Self {
        // Initialize shared state for background training
        let training_snapshot = Arc::new(Mutex::new(TrainingSnapshot::default()));
        let training_running = Arc::new(AtomicBool::new(true));
        let training_paused = Arc::new(AtomicBool::new(false));

        // INSTANT LAUNCH - No bootstrap, jump straight to live NCA!
        let mut app = Self {
            state: AppState::new(),
            should_quit: false,
            start_time: Instant::now(),
            training_runner: TrainingRunner::empty(),
            init_state: InitState::Ready,  // Start ready immediately!
            init_progress: "".to_string(),
            bootstrap_state: None,  // No bootstrap needed
            demo_grid: Self::generate_demo_grid(),
            use_hot_reload: true,  // Enable hot-reload by default!
            // Background training thread
            training_snapshot,
            training_running,
            training_paused,
            training_thread: None,  // Will be started below
            // Audio sonification initialization (output) - DISABLED for pattern training focus
            audio_engine: None,  // Disabled
            audio_enabled: false,  // Disabled for pattern training focus
            audio_volume: 0.3,  // Start at 30% volume
            last_sonification_time: Instant::now(),
            // Audio input initialization (listening) - DISABLED for pattern training focus
            audio_input_engine: None,  // Disabled
            audio_listening: false,  // Disabled
            last_audio_input_time: Instant::now(),
            // Vision system initialization (camera) - DISABLED for pattern training focus
            vision_engine: None,  // Disabled
            vision_enabled: false,  // Disabled for pattern training focus
            last_vision_capture_time: Instant::now(),
            vision_frame_counter: 0,
        };

        // Quick initialization - just enable systems, no heavy training
        app.training_runner.initialize_systems_fast();

        // Start the background training thread for real-time NCA visualization
        app.start_background_training();

        app
    }

    /// Start background training thread for real-time NCA updates
    fn start_background_training(&mut self) {
        let snapshot = Arc::clone(&self.training_snapshot);
        let running = Arc::clone(&self.training_running);
        let paused = Arc::clone(&self.training_paused);

        let handle = thread::spawn(move || {
            // Create a fresh SageExperience for the background thread
            let mut sage = SageExperience::new();

            // Try to load saved weights (silently - TUI will show status)
            let _ = sage.get_nca_mut().load_weights_from_file("pattern_training_weights.json");

            Self::training_thread_loop(snapshot, running, paused, sage);
        });

        self.training_thread = Some(handle);
    }

    /// The actual training loop that runs in the background thread
    fn training_thread_loop(
        snapshot: Arc<Mutex<TrainingSnapshot>>,
        running: Arc<AtomicBool>,
        paused: Arc<AtomicBool>,
        mut sage: SageExperience,
    ) {
        use crate::grid::GRID_SIZE;
        use rand::Rng;

        // Pattern sequence
        let patterns = vec![
            "Circle", "Square", "Cross", "Spiral",
            "Triangle", "Star", "Ring", "Hexagon", "Checkerboard"
        ];

        let mut pattern_index = 0;
        let mut iteration = 0;
        let total_iterations = 100;
        let mut generation: u64 = 0;
        let mut mastery_status: Vec<(String, bool, f64)> = patterns.iter()
            .map(|p| (p.to_string(), false, 1.0))
            .collect();
        let mut rng = rand::thread_rng();

        // Progress tracking for gauges
        let mut low_loss_streak: usize = 0;
        let mut pattern_attempts: usize = 0;

        // Generate initial target pattern
        let mut target_grid = Self::generate_pattern(&patterns[pattern_index]);

        while running.load(Ordering::Relaxed) {
            // Check if paused
            if paused.load(Ordering::Relaxed) {
                thread::sleep(Duration::from_millis(50));
                continue;
            }

            let pattern_name = patterns[pattern_index];
            // Start damage resistance earlier for better regeneration training
            // Phase 1 (0-29): Pure formation
            // Phase 2 (30-99): Damage + regeneration (70 iterations of damage training)
            let damage_phase = iteration >= 30;

            // Learning rate based on pattern difficulty
            let learning_rate = match pattern_name {
                "Square" | "Star" | "Checkerboard" => 0.0002,
                "Ring" | "Hexagon" => 0.00015,
                _ => 0.0001,
            };

            // STABILITY TRAINING: Extended evolution with sample pool
            // Growing CA paper approach: train from both seeds AND intermediate states
            // This teaches the network to maintain patterns as stable attractors
            //
            // Phase 1 (steps 0-100): Pattern formation
            // Phase 2 (steps 100-250): Stability phase - pattern should persist

            let batch_size = 4;
            let mut batch_loss = 0.0;
            let mut batch_stability = 0.0;
            let mut last_snapshot_time = std::time::Instant::now();
            let snapshot_interval = Duration::from_millis(16); // 60fps updates

            for batch_idx in 0..batch_size {
                // SAMPLE POOL: 30% of batches start from pool (intermediate states)
                // This is key to stability training - learn to maintain pattern from any state
                let use_pool = iteration > 10 && rng.gen::<f64>() < 0.3 && sage.get_nca().pool_size() > 0;
                if use_pool {
                    let pool_grid = sage.get_nca_mut().sample_from_pool();
                    sage.get_nca_mut().grid = pool_grid;
                } else {
                    sage.get_nca_mut().reset_with_seed();
                }

                // Set pattern conditioning (one-hot encoding for first 4 patterns)
                // This teaches the network to produce different patterns based on input
                if pattern_index < 4 {
                    sage.get_nca_mut().grid.set_pattern_condition(pattern_index);
                }

                // EXTENDED EVOLUTION: 150-250 steps (was 64-96)
                // Longer evolution ensures patterns become stable attractors
                let evolution_steps = rng.gen_range(150..=250);

                // STABILITY TRACKING: Capture state at step 100 for drift measurement
                let mut checkpoint_alpha: Option<Vec<Vec<f64>>> = None;

                // Evolve - update snapshot periodically for smooth visualization
                for step in 0..evolution_steps {
                    sage.get_nca_mut().step();

                    // Capture checkpoint at step 100 (start of stability phase)
                    if step == 100 {
                        let grid = sage.get_current_nca_grid();
                        checkpoint_alpha = Some(
                            grid.cells.iter()
                                .map(|row| row.iter().map(|cell| cell[3]).collect())
                                .collect()
                        );
                    }

                    // Update snapshot at 60fps intervals (non-blocking)
                    if last_snapshot_time.elapsed() >= snapshot_interval {
                        let current_grid = sage.get_current_nca_grid();
                        let phase_name = if step < 100 { "FORM" } else { "STABLE" };
                        if let Ok(mut snap) = snapshot.try_lock() {
                            for y in 0..32 {
                                for x in 0..32 {
                                    snap.grid[y][x] = current_grid.cells[y][x][3];
                                    snap.target_grid[y][x] = target_grid.cells[y][x][3];
                                }
                            }
                            snap.generation = generation;
                            snap.pattern_name = format!("{} [{}] {} {}", pattern_name, batch_idx + 1, phase_name, step + 1);
                            snap.pattern_index = pattern_index;
                            snap.pattern_total = patterns.len();
                            snap.iteration = iteration;
                            snap.total_iterations = total_iterations;
                            snap.is_damage_phase = damage_phase;
                            snap.mastery_status = mastery_status.clone();
                        }
                        last_snapshot_time = std::time::Instant::now();
                    }
                }

                // Calculate stability: how much did pattern change from step 100 to final?
                // Lower is better - pattern should be a stable attractor
                let stability = if let Some(ref checkpoint) = checkpoint_alpha {
                    let final_grid = sage.get_current_nca_grid();
                    let mut drift = 0.0;
                    for y in 0..GRID_SIZE {
                        for x in 0..GRID_SIZE {
                            let diff = final_grid.cells[y][x][3] - checkpoint[y][x];
                            drift += diff * diff;
                        }
                    }
                    drift / (GRID_SIZE * GRID_SIZE) as f64
                } else {
                    0.0
                };
                batch_stability += stability;

                // Damage phase
                if damage_phase {
                    sage.get_nca_mut().apply_damage();
                    // More recovery steps to give network time to regenerate
                    let recovery_steps = rng.gen_range(48..=80);

                    for step in 0..recovery_steps {
                        sage.get_nca_mut().step();

                        // Update snapshot at 60fps intervals
                        if last_snapshot_time.elapsed() >= snapshot_interval {
                            let current_grid = sage.get_current_nca_grid();
                            if let Ok(mut snap) = snapshot.try_lock() {
                                for y in 0..32 {
                                    for x in 0..32 {
                                        snap.grid[y][x] = current_grid.cells[y][x][3];
                                    }
                                }
                                snap.pattern_name = format!("{} REGEN step {}", pattern_name, step + 1);
                            }
                            last_snapshot_time = std::time::Instant::now();
                        }
                    }
                }

                // Calculate loss for this sample
                let nca_grid = sage.get_current_nca_grid();
                let mut pattern_loss = 0.0;
                for y in 0..GRID_SIZE {
                    for x in 0..GRID_SIZE {
                        let diff = nca_grid.cells[y][x][3] - target_grid.cells[y][x][3];
                        pattern_loss += diff * diff;
                    }
                }
                batch_loss += pattern_loss / (GRID_SIZE * GRID_SIZE) as f64;

                // Train with combined loss (pattern matching)
                // Stability is implicitly trained via pool sampling and extended evolution
                sage.get_nca_mut().train_step(&target_grid, learning_rate);

                // ADD TO SAMPLE POOL: Store intermediate states for future training
                // This is essential for stability - network learns to maintain pattern from any state
                let grid_for_pool = sage.get_current_nca_grid().clone();
                sage.get_nca_mut().add_to_pool(grid_for_pool, pattern_loss);
            }

            let current_loss = batch_loss / batch_size as f64;
            let avg_stability = batch_stability / batch_size as f64;

            // Calculate diversity (std dev of alive cells)
            let nca_grid = sage.get_current_nca_grid();
            let alive_values: Vec<f64> = nca_grid.cells.iter()
                .flatten()
                .map(|cell| cell[3])
                .filter(|&alpha| alpha > 0.1)
                .collect();
            let diversity = if alive_values.len() > 1 {
                let mean = alive_values.iter().sum::<f64>() / alive_values.len() as f64;
                let variance = alive_values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / alive_values.len() as f64;
                variance.sqrt()
            } else {
                0.0
            };

            // Calculate complexity (neighbor differences)
            let mut complexity = 0.0;
            let mut count = 0;
            for y in 1..31 {
                for x in 1..31 {
                    let center = nca_grid.cells[y][x][3];
                    let neighbors = [
                        nca_grid.cells[y-1][x][3], nca_grid.cells[y+1][x][3],
                        nca_grid.cells[y][x-1][3], nca_grid.cells[y][x+1][3],
                    ];
                    for n in neighbors {
                        complexity += (center - n).abs();
                        count += 1;
                    }
                }
            }
            complexity = if count > 0 { complexity / count as f64 } else { 0.0 };

            // Final snapshot update with loss and metrics history
            {
                let mut snap = snapshot.lock().unwrap();
                snap.loss = current_loss;
                snap.pattern_name = pattern_name.to_string();

                if current_loss < mastery_status[pattern_index].2 {
                    mastery_status[pattern_index].2 = current_loss;
                }
                snap.mastery_status = mastery_status.clone();
                snap.low_loss_streak = low_loss_streak;
                snap.pattern_attempts = pattern_attempts;

                // Add to metrics history (keep last 250 points)
                snap.metrics_history.push(MetricPoint {
                    generation,
                    loss: current_loss,
                    diversity,
                    complexity,
                    stability: avg_stability,  // Track pattern drift during stability phase
                    pattern: pattern_name.to_string(),
                });
                if snap.metrics_history.len() > 250 {
                    snap.metrics_history.remove(0);
                }
            }

            let loss = current_loss;
            iteration += 1;
            generation += 1;

            // Update progress tracking for gauges
            pattern_attempts += 1;
            if loss < 0.1 {
                low_loss_streak += 1;
            } else {
                low_loss_streak = 0;
            }

            // Check for pattern completion
            if loss < 0.05 || iteration >= total_iterations {
                let mastered = loss < 0.05;
                mastery_status[pattern_index].1 = mastered;

                // Save weights periodically
                let _ = sage.save_knowledge("pattern_training_weights.json");

                // Add event
                {
                    let mut snap = snapshot.lock().unwrap();
                    let msg = if mastered {
                        format!("[OK] {} MASTERED (loss: {:.4})", pattern_name, loss)
                    } else {
                        format!("{} completed (loss: {:.4})", pattern_name, loss)
                    };
                    snap.events.push(msg);
                    if snap.events.len() > 10 {
                        snap.events.remove(0);
                    }
                }

                // Move to next pattern
                pattern_index = (pattern_index + 1) % patterns.len();
                iteration = 0;
                target_grid = Self::generate_pattern(&patterns[pattern_index]);

                // Reset progress tracking for new pattern
                low_loss_streak = 0;
                pattern_attempts = 0;
            }

            // No sleep - run as fast as possible for real-time feel
            // The mutex lock provides natural throttling
        }
    }

    /// Generate a target pattern grid
    fn generate_pattern(name: &str) -> crate::grid::Grid {
        use crate::grid::{Grid, GRID_SIZE};
        #[allow(unused_imports)]
        use Grid as GridType;

        let mut grid = Grid::new(GRID_SIZE, GRID_SIZE);
        let center_x = GRID_SIZE / 2;
        let center_y = GRID_SIZE / 2;

        match name {
            "Circle" => {
                let radius = 8.0;
                for y in 0..GRID_SIZE {
                    for x in 0..GRID_SIZE {
                        let dx = x as f64 - center_x as f64;
                        let dy = y as f64 - center_y as f64;
                        let dist = (dx * dx + dy * dy).sqrt();
                        if dist < radius {
                            grid.cells[y][x][3] = 1.0;
                            grid.cells[y][x][0] = 1.0;
                        }
                    }
                }
            },
            "Square" => {
                let size = 10;
                for y in (center_y.saturating_sub(size))..(center_y + size).min(GRID_SIZE) {
                    for x in (center_x.saturating_sub(size))..(center_x + size).min(GRID_SIZE) {
                        grid.cells[y][x][3] = 1.0;
                        grid.cells[y][x][1] = 1.0;
                    }
                }
            },
            "Cross" => {
                let arm_width = 4;
                let arm_length = 10;
                for x in (center_x.saturating_sub(arm_length))..(center_x + arm_length).min(GRID_SIZE) {
                    for dy in 0..arm_width {
                        let y = center_y.saturating_sub(arm_width/2) + dy;
                        if y < GRID_SIZE {
                            grid.cells[y][x][3] = 1.0;
                            grid.cells[y][x][2] = 1.0;
                        }
                    }
                }
                for y in (center_y.saturating_sub(arm_length))..(center_y + arm_length).min(GRID_SIZE) {
                    for dx in 0..arm_width {
                        let x = center_x.saturating_sub(arm_width/2) + dx;
                        if x < GRID_SIZE {
                            grid.cells[y][x][3] = 1.0;
                            grid.cells[y][x][2] = 1.0;
                        }
                    }
                }
            },
            "Spiral" => {
                for i in 0..400 {
                    let t = i as f64 * 0.1;
                    let r = 1.0 + t * 0.5;
                    let x = center_x as f64 + r * t.cos();
                    let y = center_y as f64 + r * t.sin();
                    let xi = x as usize;
                    let yi = y as usize;
                    if xi < GRID_SIZE && yi < GRID_SIZE {
                        grid.cells[yi][xi][3] = 1.0;
                        grid.cells[yi][xi][0] = 1.0;
                        grid.cells[yi][xi][1] = 0.5;
                    }
                }
            },
            "Triangle" => {
                let size = 12.0;
                for y in 0..GRID_SIZE {
                    for x in 0..GRID_SIZE {
                        let px = x as f64 - center_x as f64;
                        let py = y as f64 - center_y as f64;
                        let v0 = (0.0, -size);
                        let v1 = (-size * 0.866, size * 0.5);
                        let v2 = (size * 0.866, size * 0.5);
                        let area = 0.5 * (-v1.1 * v2.0 + v0.1 * (-v1.0 + v2.0) + v0.0 * (v1.1 - v2.1) + v1.0 * v2.1);
                        let s = (v0.1 * v2.0 - v0.0 * v2.1 + (v2.1 - v0.1) * px + (v0.0 - v2.0) * py) / (2.0 * area);
                        let t = (v0.0 * v1.1 - v0.1 * v1.0 + (v0.1 - v1.1) * px + (v1.0 - v0.0) * py) / (2.0 * area);
                        if s >= 0.0 && t >= 0.0 && (s + t) <= 1.0 {
                            grid.cells[y][x][3] = 1.0;
                            grid.cells[y][x][0] = 0.8;
                            grid.cells[y][x][1] = 0.4;
                        }
                    }
                }
            },
            "Star" => {
                let outer_r = 12.0;
                let inner_r = 5.0;
                for y in 0..GRID_SIZE {
                    for x in 0..GRID_SIZE {
                        let dx = x as f64 - center_x as f64;
                        let dy = y as f64 - center_y as f64;
                        let dist = (dx * dx + dy * dy).sqrt();
                        let angle = dy.atan2(dx);
                        let point_angle = (angle * 5.0 / std::f64::consts::PI).abs() % 2.0;
                        let r = if point_angle < 1.0 {
                            inner_r + (outer_r - inner_r) * (1.0 - point_angle)
                        } else {
                            inner_r + (outer_r - inner_r) * (point_angle - 1.0)
                        };
                        if dist < r {
                            grid.cells[y][x][3] = 1.0;
                            grid.cells[y][x][0] = 1.0;
                            grid.cells[y][x][1] = 0.8;
                        }
                    }
                }
            },
            "Ring" => {
                let outer_r = 10.0;
                let inner_r = 5.0;
                for y in 0..GRID_SIZE {
                    for x in 0..GRID_SIZE {
                        let dx = x as f64 - center_x as f64;
                        let dy = y as f64 - center_y as f64;
                        let dist = (dx * dx + dy * dy).sqrt();
                        if dist >= inner_r && dist <= outer_r {
                            grid.cells[y][x][3] = 1.0;
                            grid.cells[y][x][1] = 0.7;
                            grid.cells[y][x][2] = 1.0;
                        }
                    }
                }
            },
            "Hexagon" => {
                let size = 10.0;
                for y in 0..GRID_SIZE {
                    for x in 0..GRID_SIZE {
                        let dx = (x as f64 - center_x as f64).abs();
                        let dy = (y as f64 - center_y as f64).abs();
                        if dx <= size && dy <= size * 0.866 && dx + dy * 0.577 <= size {
                            grid.cells[y][x][3] = 1.0;
                            grid.cells[y][x][0] = 0.6;
                            grid.cells[y][x][1] = 0.9;
                            grid.cells[y][x][2] = 0.3;
                        }
                    }
                }
            },
            "Checkerboard" => {
                let cell_size = 4;
                for y in 0..GRID_SIZE {
                    for x in 0..GRID_SIZE {
                        let cx = x / cell_size;
                        let cy = y / cell_size;
                        if (cx + cy) % 2 == 0 {
                            grid.cells[y][x][3] = 1.0;
                            grid.cells[y][x][0] = 0.9;
                            grid.cells[y][x][1] = 0.9;
                            grid.cells[y][x][2] = 0.9;
                        }
                    }
                }
            },
            _ => {}
        }

        grid
    }


    pub fn handle_input(&mut self, key: KeyEvent) -> Action {
        // Simple screen navigation and commands
        match key.code {
            // Tab to cycle through screens: Social Mind -> Neural Observatory -> Evolution Timeline
            KeyCode::Tab => {
                let next_screen = self.state.current_screen.next();
                Action::SwitchScreen(next_screen)
            }

            // Panel toggles
            KeyCode::Char('r') | KeyCode::Char('R') => Action::TogglePanel(PanelType::ActivityFeed),
            KeyCode::Char('d') | KeyCode::Char('D') => Action::TogglePanel(PanelType::DetailPanel),
            KeyCode::Char('m') | KeyCode::Char('M') => Action::TogglePanel(PanelType::MindState),
            KeyCode::Char('h') | KeyCode::Char('H') => Action::TogglePanel(PanelType::HealthDiagnostics),

            // Actions
            KeyCode::Char(' ') => Action::PauseResume,
            KeyCode::Char('n') | KeyCode::Char('N') => Action::StartTraining,
            KeyCode::Char('e') | KeyCode::Char('E') => Action::ExportData,
            KeyCode::Char('l') | KeyCode::Char('L') => Action::HotReload,  // NEW: Hot-reload

            // Audio controls
            KeyCode::Char('a') | KeyCode::Char('A') => Action::AudioToggle,
            KeyCode::Char('+') | KeyCode::Char('=') => Action::AudioVolumeUp,
            KeyCode::Char('-') | KeyCode::Char('_') => Action::AudioVolumeDown,
            KeyCode::Char('i') | KeyCode::Char('I') => Action::AudioListenToggle,  // NEW: Toggle audio input

            // Vision controls
            KeyCode::Char('v') | KeyCode::Char('V') => Action::VisionToggle,  // NEW: Toggle vision capture

            // Performance controls
            KeyCode::Char('p') | KeyCode::Char('P') => Action::CyclePerformanceMode,  // Cycle: Eco -> Low -> Med -> High -> Max

            // Quit
            KeyCode::Char('q') | KeyCode::Char('Q') => Action::Quit,

            _ => Action::None,
        }
    }

    pub fn handle_action(&mut self, action: Action) {
        match action {
            Action::None => {}
            Action::SwitchScreen(screen) => {
                self.state.current_screen = screen;
            }
            Action::TogglePanel(panel) => {
                self.state.panels.toggle(panel);
            }
            Action::PauseResume => {
                self.state.is_paused = !self.state.is_paused;
                // Update the atomic flag for the background training thread
                self.training_paused.store(self.state.is_paused, Ordering::Relaxed);
                if self.state.is_paused {
                    self.training_runner.pause();
                }
            }
            Action::StartTraining => {
                // Start pattern training on geometric shapes
                if self.state.training_mode == TrainingMode::Idle {
                    self.state.training_mode = TrainingMode::PatternTraining;
                    self.state.pattern_current_index = 0;
                    self.state.pattern_current_iteration = 0;
                    self.state.pattern_target_grid = None;
                }
            }
            Action::ExportData => {
                // TODO: Implement export
            }
            Action::HotReload => {
                // Signal hot-reload request to training thread
                self.state.training_state.add_event("Hot-reload requested via 'L' key".to_string());
                self.state.request_hot_reload = true;
            }
            Action::AudioToggle => {
                // Toggle audio on/off
                self.audio_enabled = !self.audio_enabled;
                if let Some(ref engine) = self.audio_engine {
                    if !self.audio_enabled {
                        engine.stop();
                    }
                }
            }
            Action::AudioVolumeUp => {
                // Increase volume by 10%, max 1.0
                self.audio_volume = (self.audio_volume + 0.1).min(1.0);
                if let Some(ref engine) = self.audio_engine {
                    engine.set_volume(self.audio_volume);
                }
            }
            Action::AudioVolumeDown => {
                // Decrease volume by 10%, min 0.0
                self.audio_volume = (self.audio_volume - 0.1).max(0.0);
                if let Some(ref engine) = self.audio_engine {
                    engine.set_volume(self.audio_volume);
                }
            }
            Action::AudioListenToggle => {
                // Toggle audio input (listening) on/off
                self.audio_listening = !self.audio_listening;
                if let Some(ref engine) = self.audio_input_engine {
                    engine.set_listening(self.audio_listening);
                }
            }
            Action::VisionToggle => {
                // Toggle vision capture on/off
                self.vision_enabled = !self.vision_enabled;
            }
            Action::CyclePerformanceMode => {
                // Cycle through performance modes: Eco -> Low -> Med -> High -> Max -> Eco
                let old_mode = self.state.performance_mode;
                self.state.performance_mode = self.state.performance_mode.next();
                self.state.training_state.add_event(format!(
                    "Performance: {} -> {}",
                    old_mode.label(),
                    self.state.performance_mode.label()
                ));
            }
            Action::Quit => {
                // Stop the background training thread
                self.training_running.store(false, Ordering::Relaxed);

                // Save pattern training weights before exit
                let _ = self.state.sage.save_knowledge("pattern_training_weights.json");
                self.state.training_state.add_event("Saved training progress on exit".to_string());
                self.should_quit = true;
            }
        }
    }

    pub fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> std::io::Result<()> {
        // Always render at 60fps for smooth visuals
        let target_fps: u64 = 60;
        let frame_duration = Duration::from_millis(1000 / target_fps);
        let mut frame_counter: u64 = 0;

        loop {
            let frame_start = Instant::now();
            frame_counter = frame_counter.wrapping_add(1);

            // Render current screen at 60fps
            terminal.draw(|f| {
                use ratatui::layout::{Constraint, Direction, Layout};
                use crate::tui::components::{ActivityFeedPanel, HealthDiagnosticsPanel};

                let screen = Screen::get_screen(self.state.current_screen);
                let main_area = f.size();

                // Determine the main content area based on what panels are visible
                let content_area = if self.state.panels.activity_feed_visible {
                    // If activity feed is visible, shrink main content to make room
                    let chunks = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints([
                            Constraint::Percentage(70),
                            Constraint::Percentage(30),
                        ])
                        .split(main_area);
                    chunks[0]
                } else {
                    main_area
                };

                // Render main screen
                screen.render(f, content_area, &self.state);

                // Render overlay panels if visible
                if self.state.panels.activity_feed_visible {
                    // Activity feed on the right side
                    let chunks = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints([
                            Constraint::Percentage(70),
                            Constraint::Percentage(30),
                        ])
                        .split(main_area);

                    ActivityFeedPanel::render(f, chunks[1], &self.state);
                }

                if self.state.panels.health_diagnostics_visible {
                    // Health diagnostics centered overlay
                    let vert_chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([
                            Constraint::Percentage(15),
                            Constraint::Percentage(70),
                            Constraint::Percentage(15),
                        ])
                        .split(main_area);

                    let horz_chunks = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints([
                            Constraint::Percentage(15),
                            Constraint::Percentage(70),
                            Constraint::Percentage(15),
                        ])
                        .split(vert_chunks[1]);

                    HealthDiagnosticsPanel::render(f, horz_chunks[1], &self.state);
                }
            })?;

            // Non-blocking input check - process immediately if available
            if event::poll(Duration::from_millis(0))? {
                if let Event::Key(key) = event::read()? {
                    let action = self.handle_input(key);
                    self.handle_action(action);

                    if self.should_quit {
                        break;
                    }
                }
            }

            // Read from background training thread and update display
            // Training runs independently, we just read the latest snapshot
            self.sync_from_training_thread();

            // Update other state (vision, audio, etc.)
            self.update_non_training_state();

            // Brain Monitor animation tick (always 60fps for smooth visuals)
            self.state.brain_pulse_phase = (self.state.brain_pulse_phase + 0.15) % (2.0 * std::f64::consts::PI);
            self.state.brain_frame_count += 1;

            // Decay activity map (previous changes fade over time)
            for activity in &mut self.state.brain_activity_map {
                *activity *= 0.92;
            }

            // Random neural firing (sparks) - about 2-5 cells per frame
            use rand::Rng;
            let mut rng = rand::thread_rng();
            let num_fires = rng.gen_range(2..6);
            for _ in 0..num_fires {
                let idx = rng.gen_range(0..1024);
                self.state.brain_activity_map[idx] = (self.state.brain_activity_map[idx] + 0.6).min(1.0);
            }

            // Frame pacing - always 60fps for smooth visuals
            let frame_elapsed = frame_start.elapsed();
            if frame_elapsed < frame_duration {
                std::thread::sleep(frame_duration - frame_elapsed);
            }
        }

        Ok(())
    }

    /// Read latest training state from background thread and update display
    fn sync_from_training_thread(&mut self) {
        use crate::grid::Grid;

        // Read from shared training snapshot (non-blocking)
        if let Ok(snap) = self.training_snapshot.try_lock() {
            // Update grid display (current NCA state)
            for y in 0..32 {
                for x in 0..32 {
                    self.state.training_state.grid_snapshot[y][x] = snap.grid[y][x];
                }
            }

            // Update target grid for TARGET panel display
            let mut target_grid = Grid::new(32, 32);
            for y in 0..32 {
                for x in 0..32 {
                    target_grid.cells[y][x][3] = snap.target_grid[y][x];  // Alpha channel
                    target_grid.cells[y][x][0] = snap.target_grid[y][x];  // R channel for color
                }
            }
            self.state.pattern_target_grid = Some(target_grid);

            // Update training metrics
            self.state.training_state.current_loss = snap.loss;
            self.state.training_state.nca_generation = snap.generation;
            self.state.training_state.nca_current_pattern = format!(
                "({}/{}) {} {} - Iter {}/{}",
                snap.pattern_index + 1,
                snap.pattern_total,
                snap.pattern_name,
                if snap.is_damage_phase { "REGEN" } else { "FORM" },
                snap.iteration + 1,
                snap.total_iterations
            );

            // Update pattern mastery status
            self.state.pattern_mastery_status = snap.mastery_status.clone();
            self.state.pattern_current_index = snap.pattern_index;
            self.state.pattern_current_iteration = snap.iteration;

            // Copy events to training state (avoid duplicates)
            for event in &snap.events {
                if !self.state.training_state.recent_events.contains(event) {
                    self.state.training_state.add_event(event.clone());
                }
            }

            // Sync progress tracking for gauges
            self.state.training_state.low_loss_streak = snap.low_loss_streak;
            self.state.training_state.pattern_attempts = snap.pattern_attempts;

            // Sync metrics history for charts
            self.state.training_state.metrics_history = snap.metrics_history.iter()
                .map(|mp| crate::tui::training::MetricSnapshot {
                    generation: mp.generation,
                    loss: mp.loss,
                    complexity: mp.complexity,
                    diversity: mp.diversity,
                    pattern: mp.pattern.clone(),
                })
                .collect();
        }
    }

    fn update_non_training_state(&mut self) {
        // Sync audio status to AppState for TUI display
        self.state.audio_available = self.audio_engine.is_some();
        self.state.audio_enabled = self.audio_enabled;
        self.state.audio_volume = self.audio_volume;
        self.state.audio_input_available = self.audio_input_engine.is_some();
        self.state.audio_listening = self.audio_listening;

        // Sync vision status to AppState for TUI display
        self.state.vision_available = self.vision_engine.is_some();
        self.state.vision_enabled = self.vision_enabled;
        // Calculate actual FPS from frame counter
        let elapsed_secs = self.start_time.elapsed().as_secs_f64();
        self.state.vision_fps = if elapsed_secs > 0.0 {
            self.vision_frame_counter as f64 / elapsed_secs
        } else {
            0.0
        };

        // Update uptime from elapsed time
        let elapsed = self.start_time.elapsed();
        self.state.uptime_seconds = elapsed.as_secs();

        // Update consciousness state based on idle time
        self.state.idle_seconds = self.state.last_activity_time.elapsed().as_secs();

        if let Some(mode) = self.state.sage.should_enter_autonomous_mode(self.state.idle_seconds) {
            if mode == "dream" && self.state.consciousness_state != ConsciousnessState::Dreaming {
                self.state.consciousness_state = ConsciousnessState::Dreaming;
                // Execute dream cycle
                let _dream_log = self.state.sage.dream_cycle();
            } else if mode == "curiosity" && self.state.consciousness_state != ConsciousnessState::Curious {
                self.state.consciousness_state = ConsciousnessState::Curious;
                // Execute curiosity cycle
                let _curiosity_result = self.state.sage.curiosity_cycle(&self.state.baseline_concepts);
            }
        } else if self.state.idle_seconds < 300 {
            // Reset to active if we're back to normal activity
            self.state.consciousness_state = ConsciousnessState::Active;
        }

        // Live camera capture at 30 FPS (if vision engine available)
        const VISION_FPS: u64 = 30;
        const VISION_FRAME_DURATION_MS: u64 = 1000 / VISION_FPS;  // ~33ms

        if self.vision_enabled {
            if let Some(ref vision) = self.vision_engine {
                // Throttle to 30 FPS
                if self.last_vision_capture_time.elapsed().as_millis() >= VISION_FRAME_DURATION_MS as u128 {
                    // Capture frame
                    if let Ok(frame) = vision.capture_frame() {
                        // Extract visual features
                        let features = vision.extract_features(&frame);

                        // Generate visual concepts
                        let mut concepts = Vec::new();

                        // Brightness concepts
                        if features.avg_brightness > 0.7 {
                            concepts.push("bright".to_string());
                        } else if features.avg_brightness < 0.3 {
                            concepts.push("dark".to_string());
                        }

                        // Color concept
                        concepts.push(features.dominant_color.clone());

                        // Edge concepts
                        if features.edge_strength > 30.0 {
                            concepts.push("sharp_edges".to_string());
                        } else {
                            concepts.push("smooth".to_string());
                        }

                        // Variance concept
                        if features.color_variance > 0.1 {
                            concepts.push("varied".to_string());
                        } else {
                            concepts.push("uniform".to_string());
                        }

                        // Convert frame to RGB format for TUI display
                        let height = frame.height() as usize;
                        let width = frame.width() as usize;
                        let mut camera_frame = vec![vec![(0u8, 0u8, 0u8); width]; height];

                        for y in 0..height {
                            for x in 0..width {
                                let pixel = frame.get_pixel(x as u32, y as u32);
                                camera_frame[y][x] = (pixel[0], pixel[1], pixel[2]);
                            }
                        }

                        // Update app state
                        self.state.camera_frame = Some(camera_frame);
                        self.state.visual_concepts = concepts.clone();

                        // Update timing and counter
                        self.last_vision_capture_time = Instant::now();
                        self.vision_frame_counter += 1;
                    }
                }
            }
        }

        // Only use IRC sync camera if we don't have live vision
        use crate::irc_sync::IrcSync;
        if !self.vision_enabled || self.vision_engine.is_none() {
            if let Some(camera_snapshot) = IrcSync::get_camera_snapshot() {
                self.state.camera_frame = Some(camera_snapshot.frame);
                self.state.visual_concepts = camera_snapshot.visual_concepts;
            }
        }

        // Update autonomous activities from IRC sync
        let activities = IrcSync::get_autonomous_activities();
        self.state.autonomous_activities = activities
            .into_iter()
            .map(|a| (a.timestamp, a.activity_type, a.description))
            .collect();

        // NOTE: Training state is now updated by sync_from_training_thread()
        // Don't overwrite it here - background thread handles NCA training
        // self.state.training_state = self.training_runner.get_state();
        // self.state.current_phase = self.state.training_state.current_phase;

        // Update health status based on AGI state
        // TODO: Implement actual health checks
        self.state.health_status = HealthStatus::Healthy;

        // NOTE: Pattern training now runs in background thread
        // See sync_from_training_thread() for real-time updates

        // REMOVED: All inline pattern training code
        // Training now happens in training_thread_loop() which runs in a separate thread
        // The main loop just reads from the shared training_snapshot at 60fps

        // SAGE now reports what it learns during training instead of auto-responding
        // The language learner handles this in the training loop
        /*
        if self.state.uptime_seconds - self.state.last_agi_thought_time >= 30 {
            use crate::consciousness::{Consciousness, ClaudePersona};

            if let Some(thought) = Consciousness::generate_thought(
                &self.state.agi,
                &self.state.training_state,
                &self.state.autonomous_trainer,
                self.state.current_phase,
                self.state.health_status,
            ) {
                // Add proactive thought to chat history
                self.state.chat_history.push((
                    "SAGE".to_string(),
                    thought.clone(),
                ));

                // Write SAGE's proactive thought to chat file
                use crate::message_queue;
                let _ = message_queue::write_message("SAGE", &thought);

                self.state.last_agi_thought_time = self.state.uptime_seconds;

                // Claude may respond to SAGE's thought
                if let Some(claude_observation) = ClaudePersona::generate_observation(
                    &self.state.agi,
                    &self.state.training_state,
                    &self.state.autonomous_trainer,
                    self.state.current_phase,
                    Some(&thought),
                ) {
                    self.state.chat_history.push((
                        "Claude".to_string(),
                        claude_observation,
                    ));
                    self.state.last_claude_observation_time = self.state.uptime_seconds;
                }

                // Check if Claude should propose evolution
                if let Some(evolution_proposal) = ClaudePersona::propose_evolution(
                    &thought,
                    &self.state.training_state,
                    self.state.current_phase,
                ) {
                    self.state.chat_history.push((
                        "Claude".to_string(),
                        evolution_proposal,
                    ));
                    self.state.evolution_mode = true;
                }
            }
        }
        */

        // DISABLED: Claude's independent observations
        // Claude will only respond when asked questions
        /*
        // Claude's independent observations - every 45 seconds
        if self.state.uptime_seconds - self.state.last_claude_observation_time >= 45 {
            use crate::consciousness::ClaudePersona;

            // Get last SAGE message if any
            let last_sage_msg = self.state.chat_history.iter()
                .rev()
                .find(|(sender, _)| sender == "SAGE")
                .map(|(_, msg)| msg.as_str());

            if let Some(observation) = ClaudePersona::generate_observation(
                &self.state.agi,
                &self.state.training_state,
                &self.state.autonomous_trainer,
                self.state.current_phase,
                last_sage_msg,
            ) {
                self.state.chat_history.push((
                    "Claude".to_string(),
                    observation,
                ));
                self.state.last_claude_observation_time = self.state.uptime_seconds;
            }
        }
        */

        // Auto-save state every 10 seconds
        use crate::persistence::StateManager;
        if let Ok(saved) = StateManager::auto_save(
            self.state.uptime_seconds,
            self.state.last_save_time,
            &self.state.training_state,
            self.state.current_phase,
            &self.state.chat_history,
            self.state.agi.mind_stream.active_thoughts.len(),
            self.state.agi.decision_stream.decisions.len(),
        ) {
            if saved {
                self.state.last_save_time = self.state.uptime_seconds;
            }
        }

        // Check chat file for new messages
        use crate::message_queue;
        if let Ok(new_messages) = message_queue::read_new_messages() {
            for (sender, content) in new_messages {
                // Skip messages SAGE already sent
                if sender == "SAGE" {
                    continue;
                }

                // SAGE should respond to both "You" and "Claude" messages
                if sender == "Claude" || sender == "You" {
                    // Use language learner to respond based on what SAGE has learned
                    let lang_learner = self.training_runner.language_learner.lock().unwrap();
                    let sage_response = lang_learner.respond_to_question(&content);
                    drop(lang_learner);

                    // Write SAGE's response to chat file
                    let _ = message_queue::write_message("SAGE", &sage_response);
                }
            }
        }

        // Poll IRC messages (non-blocking) and process through SAGE
        // Only process IRC when not doing baseline training
        if self.state.training_mode == TrainingMode::IrcLearning {
            if let Some(ref irc_manager) = self.state.irc_manager {
                let messages = irc_manager.poll_messages();

            for msg in messages {
                // Process message through SAGE with NCA
                use crate::text_encoder::TextEncoder;
                let mut text_encoder = TextEncoder::new();

                // Encode message to NCA grid
                let target_grid = text_encoder.encode_text(&msg.message);

                // Process through SAGE's NCA (THE GOLDEN RULE: All thinking goes through NCA)
                self.state.sage.get_nca_mut().reset_with_seed();
                for _ in 0..80 {
                    self.state.sage.get_nca_mut().step();
                }

                // Calculate loss to determine SAGE's understanding
                let loss = {
                    let nca_grid = self.state.sage.get_current_nca_grid();
                    let mut total_loss = 0.0;
                    let mut count = 0;

                    for y in 0..nca_grid.height {
                        for x in 0..nca_grid.width {
                            for channel in 0..4 {
                                let diff = nca_grid.cells[y][x][channel] - target_grid.cells[y][x][channel];
                                total_loss += diff * diff;
                                count += 1;
                            }
                        }
                    }

                    total_loss / count as f64
                };

                // THE GOLDEN RULE: Sync NCA to Living Neural Field visualization
                self.state.training_state.sync_nca_from_sage(
                    self.state.sage.get_current_nca_grid(),
                    &msg.message,
                    loss
                );

                // REAL-TIME LEARNING: If SAGE doesn't understand (high loss), learn from it!
                let learning_rate = 0.001;
                let training_iterations = if loss > 0.28 {
                    // High loss = completely unknown, train more
                    10
                } else if loss > 0.15 {
                    // Medium loss = partially understood, light training
                    5
                } else {
                    // Low loss = already understood, no training needed
                    0
                };

                if training_iterations > 0 {
                    // Train SAGE on this text pattern
                    for _ in 0..training_iterations {
                        self.state.sage.get_nca_mut().reset_with_seed();
                        for _ in 0..80 {
                            self.state.sage.get_nca_mut().step();
                        }
                        self.state.sage.get_nca_mut().train_step(&target_grid, learning_rate);
                    }

                    // Recalculate loss after training
                    self.state.sage.get_nca_mut().reset_with_seed();
                    for _ in 0..80 {
                        self.state.sage.get_nca_mut().step();
                    }

                    // Auto-save learned knowledge every 10 IRC messages
                    static mut MESSAGE_COUNT: usize = 0;
                    unsafe {
                        MESSAGE_COUNT += 1;
                        if MESSAGE_COUNT % 10 == 0 {
                            let _ = self.state.sage.save_knowledge("sage_positive_knowledge.json");
                        }
                    }
                }

                // Generate response based on loss
                let (opinion_type, response) = if loss < 0.15 {
                    ("Like".to_string(), format!("I understand! {}", msg.message))
                } else if loss < 0.28 {
                    ("Curious".to_string(), format!("Tell me more about: {}", msg.message))
                } else {
                    ("Dislike".to_string(), "I don't know much about that yet, but I'm learning!".to_string())
                };

                // Store in IRC history
                self.state.irc_messages.push((
                    msg.sender.clone(),
                    msg.message.clone(),
                    response.clone()
                ));

                // Send response back to IRC
                use crate::irc_manager::IrcResponse;
                let _ = irc_manager.send_response(IrcResponse {
                    message: response,
                    opinion_type,
                    loss,
                });
            }
            }
        }

        // AUDIO SONIFICATION: Convert neural patterns to sound (OUTPUT)
        if self.audio_enabled {
            if let Some(ref engine) = self.audio_engine {
                // Throttle sonification to ~6-7 FPS (150ms) to avoid overwhelming the audio engine
                let elapsed = self.last_sonification_time.elapsed();
                if elapsed.as_millis() >= 150 {
                    // Convert grid_snapshot (Vec<Vec<f64>>) to format expected by sonify_grid
                    // Expected format: [[f64; 22]; 1024] where each cell has 22 channels
                    let mut grid: [[f64; 22]; 1024] = [[0.0; 22]; 1024];

                    for y in 0..32 {
                        for x in 0..32 {
                            let i = y * 32 + x;
                            if y < self.state.training_state.grid_snapshot.len()
                                && x < self.state.training_state.grid_snapshot[y].len() {
                                let alpha = self.state.training_state.grid_snapshot[y][x];
                                grid[i][3] = alpha; // Alpha channel (used for sonification)
                            }
                        }
                    }

                    // Sonify the grid (150ms duration matches throttle interval)
                    engine.sonify_grid(&grid, 150);
                    self.last_sonification_time = Instant::now();
                }
            }
        }

        // AUDIO INPUT: Convert sound to neural patterns (INPUT) - BIDIRECTIONAL AUDIO!
        if self.audio_listening {
            if let Some(ref engine) = self.audio_input_engine {
                // Throttle audio input processing to ~10 FPS (100ms)
                let elapsed = self.last_audio_input_time.elapsed();
                if elapsed.as_millis() >= 100 {
                    // Get audio-derived grid from input engine
                    let audio_grid = engine.get_grid();

                    // Convert audio grid [[f64; 22]; 1024] to 32x32 grid for visualization
                    // Extract alpha channel and display it
                    let mut audio_snapshot = vec![vec![0.0; 32]; 32];
                    for y in 0..32 {
                        for x in 0..32 {
                            let i = y * 32 + x;
                            audio_snapshot[y][x] = audio_grid[i][3]; // Alpha channel
                        }
                    }

                    // TODO: Feed audio patterns into SAGE's NCA for learning
                    // For now, this demonstrates that audio input is working
                    // In the future, we could:
                    // 1. Blend audio grid with current NCA state
                    // 2. Use audio as training signal for pattern recognition
                    // 3. Create audio-visual cross-modal learning

                    self.last_audio_input_time = Instant::now();
                }
            }
        }
    }
}
