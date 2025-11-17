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
}

impl AppState {
    pub fn new() -> Self {
        let mut sage = SageExperience::new();

        // Try to load existing SAGE knowledge and state
        let _ = sage.load_knowledge("sage_positive_knowledge.json");
        let _ = sage.load_preferences("sage_preferences.json");
        let _ = sage.load_associations("sage_associations.json");
        let _ = sage.load_curiosity("sage_curiosity.json");

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
            current_screen: ScreenType::BrainMonitor,  // Start with Brain Monitor - live NCA visualization
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
            training_mode: TrainingMode::Idle,
            baseline_concepts,
            baseline_current_concept: 0,
            baseline_current_iteration: 0,
            baseline_total_iterations: 100,
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
                // Start baseline training on positive concepts
                if self.state.training_mode == TrainingMode::Idle {
                    self.state.training_mode = TrainingMode::BaselineTraining;
                    self.state.baseline_current_concept = 0;
                    self.state.baseline_current_iteration = 0;
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
            Action::Quit => {
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

        // Update camera snapshot and visual concepts from IRC sync
        use crate::irc_sync::IrcSync;
        if let Some(camera_snapshot) = IrcSync::get_camera_snapshot() {
            self.state.camera_frame = Some(camera_snapshot.frame);
            self.state.visual_concepts = camera_snapshot.visual_concepts;
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
    }
}
