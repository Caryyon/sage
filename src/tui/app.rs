// Application state and event loop for SAGE Mission Control

use crossterm::event::{self, Event, KeyCode, KeyEvent};
use ratatui::{
    backend::Backend,
    Terminal,
};
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
        if let Ok(_) = sage.get_nca_mut().load_weights_from_file("pattern_training_weights.json") {
            // Successfully restored pattern training progress
            eprintln!("🔄 Restored pattern training weights from previous session");
        }

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
            current_screen: ScreenType::UnifiedDashboard,  // Start with pattern training visualization
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
            // Pattern training initialization
            pattern_sequence: vec!["Circle".to_string(), "Square".to_string(), "Cross".to_string(), "Spiral".to_string()],
            pattern_current_index: 0,
            pattern_current_iteration: 0,
            pattern_total_iterations: 100,
            pattern_target_grid: None,
            pattern_mastery_status: vec![
                ("Circle".to_string(), false, 1.0),
                ("Square".to_string(), false, 1.0),
                ("Cross".to_string(), false, 1.0),
                ("Spiral".to_string(), false, 1.0),
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
        // INSTANT LAUNCH - No bootstrap, jump straight to live NCA with HOT-RELOAD!
        let app = Self {
            state: AppState::new(),
            should_quit: false,
            start_time: Instant::now(),
            training_runner: TrainingRunner::empty(),
            init_state: InitState::Ready,  // Start ready immediately!
            init_progress: "".to_string(),
            bootstrap_state: None,  // No bootstrap needed
            demo_grid: Self::generate_demo_grid(),
            use_hot_reload: true,  // Enable hot-reload by default!
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

        // DISABLED: Old automatic training loops that endlessly process shapes
        // SAGE now trains on-demand when IRC messages arrive (event-driven)
        // Living Neural Field updates in real-time with each message
        // if app.use_hot_reload {
        //     app.training_runner.start_training_with_hot_reload();
        // } else {
        //     app.training_runner.start_training();
        // }

        app
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
                self.state.training_state.add_event("🔥 Hot-reload requested via 'L' key".to_string());
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
            Action::Quit => {
                // Save pattern training weights before exit
                let _ = self.state.sage.save_knowledge("pattern_training_weights.json");
                self.state.training_state.add_event("💾 Saved training progress on exit".to_string());
                self.should_quit = true;
            }
        }
    }

    pub fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> std::io::Result<()> {
        // 60 FPS render loop - decoupled from training rate
        let target_fps = 60;
        let frame_duration = Duration::from_millis(1000 / target_fps);

        loop {
            let frame_start = Instant::now();

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

            // Update state from training thread (async, non-blocking)
            self.update();

            // Brain Monitor animation tick (60fps)
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

            // Frame pacing - sleep to maintain 60fps
            let frame_elapsed = frame_start.elapsed();
            if frame_elapsed < frame_duration {
                std::thread::sleep(frame_duration - frame_elapsed);
            }
        }

        Ok(())
    }

    fn update(&mut self) {
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

        // Update training state from training runner
        self.state.training_state = self.training_runner.get_state();
        self.state.current_phase = self.state.training_state.current_phase;

        // Update health status based on AGI state
        // TODO: Implement actual health checks
        self.state.health_status = HealthStatus::Healthy;

        // PATTERN TRAINING MODE: Train on geometric patterns (Circle, Square, Cross, Spiral)
        if self.state.training_mode == TrainingMode::PatternTraining {
            if self.state.pattern_current_index < self.state.pattern_sequence.len() {
                let pattern_name = &self.state.pattern_sequence[self.state.pattern_current_index].clone();

                // Generate target pattern if not already set for this pattern
                if self.state.pattern_target_grid.is_none() {
                    use crate::grid::{Grid, GRID_SIZE};
                    let target = match pattern_name.as_str() {
                        "Circle" => {
                            let mut grid = Grid::new(GRID_SIZE, GRID_SIZE);
                            let center_x = GRID_SIZE / 2;
                            let center_y = GRID_SIZE / 2;
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
                            grid
                        },
                        "Square" => {
                            let mut grid = Grid::new(GRID_SIZE, GRID_SIZE);
                            let center_x = GRID_SIZE / 2;
                            let center_y = GRID_SIZE / 2;
                            let size = 10;
                            for y in (center_y.saturating_sub(size))..(center_y + size).min(GRID_SIZE) {
                                for x in (center_x.saturating_sub(size))..(center_x + size).min(GRID_SIZE) {
                                    grid.cells[y][x][3] = 1.0;
                                    grid.cells[y][x][1] = 1.0;
                                }
                            }
                            grid
                        },
                        "Cross" => {
                            let mut grid = Grid::new(GRID_SIZE, GRID_SIZE);
                            let center_x = GRID_SIZE / 2;
                            let center_y = GRID_SIZE / 2;
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
                            grid
                        },
                        "Spiral" => {
                            let mut grid = Grid::new(GRID_SIZE, GRID_SIZE);
                            let center_x = GRID_SIZE / 2;
                            let center_y = GRID_SIZE / 2;
                            for i in 0..100 {
                                let angle = i as f64 * 0.3;
                                let radius = i as f64 * 0.1;
                                let x = (center_x as f64 + radius * angle.cos()) as usize;
                                let y = (center_y as f64 + radius * angle.sin()) as usize;
                                if x < GRID_SIZE && y < GRID_SIZE {
                                    grid.cells[y][x][3] = 1.0;
                                    grid.cells[y][x][0] = 1.0;
                                    grid.cells[y][x][1] = 1.0;
                                }
                            }
                            grid
                        },
                        _ => Grid::new(GRID_SIZE, GRID_SIZE),
                    };
                    self.state.pattern_target_grid = Some(target);
                }

                let target = self.state.pattern_target_grid.as_ref().unwrap();
                let learning_rate = if pattern_name == "Square" { 0.0002 } else { 0.0001 };

                // PAPER'S APPROACH: Batch training with variable evolution steps
                // + DAMAGE RESISTANCE: After iteration 50, train with damage/recovery
                use rand::Rng;
                let mut rng = rand::thread_rng();
                const BATCH_SIZE: usize = 8;

                // Phase 1 (iterations 0-49): Normal pattern formation
                // Phase 2 (iterations 50+): Damage resistance training
                let damage_resistance_mode = self.state.pattern_current_iteration >= 50;

                let mut batch_losses = Vec::new();
                let mut sample_grids = Vec::new();

                // Train batch of samples
                for sample_idx in 0..BATCH_SIZE {
                    // Each sample: fresh seed + random evolution steps
                    self.state.sage.get_nca_mut().reset_with_seed();
                    let evolution_steps = rng.gen_range(64..=96);

                    // Evolve from seed to form pattern
                    for _ in 0..evolution_steps {
                        self.state.sage.get_nca_mut().step();
                    }

                    // DAMAGE RESISTANCE TRAINING (Phase 2)
                    if damage_resistance_mode {
                        // Apply damage to the formed pattern
                        self.state.sage.get_nca_mut().apply_damage();

                        // Let the network try to recover (32-48 more steps)
                        let recovery_steps = rng.gen_range(32..=48);
                        for _ in 0..recovery_steps {
                            self.state.sage.get_nca_mut().step();
                        }
                    }

                    // Calculate loss for this sample (measures recovery quality in Phase 2)
                    let nca_grid = self.state.sage.get_current_nca_grid();
                    let mut sample_loss = 0.0;
                    let mut count = 0;

                    for y in 0..nca_grid.height {
                        for x in 0..nca_grid.width {
                            // Only RGBA channels (0-3) contribute to loss
                            for channel in 0..4 {
                                let diff = nca_grid.cells[y][x][channel] - target.cells[y][x][channel];
                                sample_loss += diff * diff;
                                count += 1;
                            }
                        }
                    }
                    sample_loss /= count as f64;
                    batch_losses.push(sample_loss);

                    // Save last sample's grid for visualization
                    if sample_idx == BATCH_SIZE - 1 {
                        sample_grids.push(nca_grid.clone());
                    }

                    // Train on this sample
                    self.state.sage.get_nca_mut().train_step(target, learning_rate);
                }

                // Average batch loss (paper's approach)
                let loss: f64 = batch_losses.iter().sum::<f64>() / batch_losses.len() as f64;

                // Update display with last sample's output
                if let Some(display_grid) = sample_grids.last() {
                    for y in 0..32 {
                        for x in 0..32 {
                            let alpha = display_grid.cells[y][x][3];
                            self.state.training_state.grid_snapshot[y][x] = alpha;
                        }
                    }
                }

                // Update metadata with phase indicator
                let phase_indicator = if damage_resistance_mode { "🛡️ REGEN" } else { "📐 FORM" };
                let progress_text = format!("({}/{}) {} {} - Iter {}/{}",
                    self.state.pattern_current_index + 1,
                    self.state.pattern_sequence.len(),
                    pattern_name,
                    phase_indicator,
                    self.state.pattern_current_iteration + 1,
                    self.state.pattern_total_iterations
                );
                self.state.training_state.nca_current_pattern = progress_text;
                self.state.training_state.current_loss = loss;

                // Log phase transition
                if self.state.pattern_current_iteration == 50 {
                    self.state.training_state.add_event(format!("🛡️ {} entering DAMAGE RESISTANCE phase", pattern_name));
                }

                // Update metrics
                let total_progress = self.state.pattern_current_index * self.state.pattern_total_iterations
                    + self.state.pattern_current_iteration;
                self.state.training_state.nca_generation = total_progress as u64;
                self.state.training_state.batch_size = BATCH_SIZE;
                self.state.training_state.current_batch = BATCH_SIZE;  // All samples completed
                self.state.training_state.total_batches = BATCH_SIZE;  // Total in this iteration
                self.state.training_state.nca_diversity = 0.6;
                self.state.training_state.nca_complexity = loss;

                // Add to metrics history for Database Monitor
                self.state.training_state.add_metric_snapshot(
                    total_progress as u64,
                    loss,
                    loss,  // complexity
                    0.6,   // diversity
                    pattern_name.clone()
                );

                // Update best loss for current pattern
                if let Some(status) = self.state.pattern_mastery_status.get_mut(self.state.pattern_current_index) {
                    if loss < status.2 {
                        status.2 = loss;  // Update best loss
                    }
                }

                // Advance iteration
                self.state.pattern_current_iteration += 1;

                // Periodic checkpoint saves (every 25 iterations to avoid too much I/O)
                if self.state.pattern_current_iteration % 25 == 0 {
                    let _ = self.state.sage.save_knowledge("pattern_training_weights.json");
                }

                // Check if pattern is mastered (loss < 0.05) or hit max iterations
                if loss < 0.05 || self.state.pattern_current_iteration >= self.state.pattern_total_iterations {
                    let mastered = loss < 0.05;

                    // Mark as mastered if loss is good enough
                    if mastered {
                        if let Some(status) = self.state.pattern_mastery_status.get_mut(self.state.pattern_current_index) {
                            status.1 = true;  // Mark as mastered
                        }
                    }

                    // 💾 SAVE WEIGHTS: Dual-layer persistence
                    // Layer 1: Local JSON file (fast restore)
                    if let Ok(_) = self.state.sage.save_knowledge("pattern_training_weights.json") {
                        let status_msg = if mastered {
                            format!("💾 Saved weights: {} MASTERED (loss: {:.4})", pattern_name, loss)
                        } else {
                            format!("💾 Saved weights: {} (loss: {:.4})", pattern_name, loss)
                        };
                        self.state.training_state.add_event(status_msg);
                    }

                    // Layer 2: SpacetimeDB snapshot (historical tracking)
                    if mastered || self.state.pattern_current_index % 2 == 0 {  // Save mastered patterns + every other pattern
                        let weights = self.state.sage.get_nca().get_weights();
                        if let Ok(weights_json) = serde_json::to_string(&weights) {
                            let generation = total_progress as u64;
                            let _ = self.state.memory.save_network_snapshot(
                                generation,
                                &pattern_name,
                                loss,
                                &weights_json,
                            );
                        }
                    }

                    self.state.pattern_current_index += 1;
                    self.state.pattern_current_iteration = 0;
                    self.state.pattern_target_grid = None;  // Reset for next pattern
                }
            } else {
                // All patterns complete! Save final state and loop back
                let _ = self.state.sage.save_knowledge("pattern_training_weights.json");
                self.state.training_state.add_event("🔄 Pattern cycle complete! Continuing with refined learning...".to_string());

                self.state.pattern_current_index = 0;
                self.state.pattern_current_iteration = 0;
                self.state.pattern_target_grid = None;
            }
        }

        // BASELINE TRAINING MODE: Train on positive concepts and visualize
        if self.state.training_mode == TrainingMode::BaselineTraining {
            use crate::text_encoder::TextEncoder;
            let mut text_encoder = TextEncoder::new();

            if self.state.baseline_current_concept < self.state.baseline_concepts.len() {
                let concept = &self.state.baseline_concepts[self.state.baseline_current_concept].clone();
                let learning_rate = 0.001;

                // Encode current concept to target grid
                let target = text_encoder.encode_concept(concept);

                // One training iteration per frame (smooth animation)
                self.state.sage.get_nca_mut().reset_with_seed();
                for _ in 0..80 {
                    self.state.sage.get_nca_mut().step();
                }

                // Calculate loss
                let loss = {
                    let nca_grid = self.state.sage.get_current_nca_grid();
                    let mut total_loss = 0.0;
                    let mut count = 0;

                    for y in 0..nca_grid.height {
                        for x in 0..nca_grid.width {
                            for channel in 0..4 {
                                let diff = nca_grid.cells[y][x][channel] - target.cells[y][x][channel];
                                total_loss += diff * diff;
                                count += 1;
                            }
                        }
                    }

                    total_loss / count as f64
                };

                // THE GOLDEN RULE: Sync to Living Neural Field
                // Show the TARGET pattern (what we're learning) for better visualization
                let progress_text = format!("[{}/{}] {} - Iter {}/{}",
                    self.state.baseline_current_concept + 1,
                    self.state.baseline_concepts.len(),
                    concept,
                    self.state.baseline_current_iteration + 1,
                    self.state.baseline_total_iterations
                );

                // DIRECT WRITE: Bypass sync and write directly to grid_snapshot
                // Target grid is already 32x32, so direct 1:1 copy
                for y in 0..32 {
                    for x in 0..32 {
                        let alpha = target.cells[y][x][3];
                        let boosted = alpha.abs().powf(0.7).min(1.0).max(0.0);
                        self.state.training_state.grid_snapshot[y][x] = boosted;
                    }
                }

                // Update metadata
                self.state.training_state.nca_current_pattern = progress_text.clone();
                self.state.training_state.current_loss = loss;

                // Update metrics for dashboard display
                let total_progress = self.state.baseline_current_concept * self.state.baseline_total_iterations
                    + self.state.baseline_current_iteration;

                self.state.training_state.nca_generation = total_progress as u64;
                self.state.training_state.batch_size = 1;  // One concept at a time
                self.state.training_state.nca_diversity = self.state.baseline_current_concept as f64 / self.state.baseline_concepts.len() as f64;
                self.state.training_state.nca_complexity = loss;  // Complexity correlates with loss

                // Train one step
                self.state.sage.get_nca_mut().train_step(&target, learning_rate);

                // Advance iteration
                self.state.baseline_current_iteration += 1;

                // Move to next concept when done with current
                if self.state.baseline_current_iteration >= self.state.baseline_total_iterations {
                    self.state.baseline_current_concept += 1;
                    self.state.baseline_current_iteration = 0;
                }
            } else {
                // Training complete! Save and switch to IRC learning mode
                let _ = self.state.sage.save_knowledge("sage_positive_knowledge.json");
                self.state.training_mode = TrainingMode::IrcLearning;
            }
        }

        // DISABLED: Proactive AGI communication
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
                    ("Like".to_string(), format!("❤️ I understand! {}", msg.message))
                } else if loss < 0.28 {
                    ("Curious".to_string(), format!("🤔 Tell me more about: {}", msg.message))
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
