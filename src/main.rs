// SAGE: Self-Adaptive General Explorer
// Mission Control - Unified system for AGI training, IRC, vision, and autonomous consciousness

use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use std::io;
use std::sync::{Arc, Mutex};
use clap::Parser;

use sage::tui::App;
use sage::cli::Cli;
// IRC disabled - use sage::irc::{IrcConfig, IrcState};
use sage::sage_experience::SageExperience;
use sage::llm_client::LlmClient;
use sage::spacetime_client::SageDbClient;
use sage::visual_memory::VisualMemory;
use sage::sage_control::{InstanceRegistry, InstanceInfo, InstanceType};

fn main() -> Result<(), io::Error> {
    // Parse CLI arguments
    let args = Cli::parse();

    // println!("╔════════════════════════════════════════════════════════════╗");
    // println!("║                SAGE - Unified System Launch               ║");
    // println!("╚════════════════════════════════════════════════════════════╝\n");

    // Initialize SAGE consciousness
    let sage = Arc::new(Mutex::new(SageExperience::new()));
    // println!("🧠 SAGE consciousness initialized");

    // Initialize SpacetimeDB client
    let db_client = SageDbClient::new("sage-db");
    // println!("💾 SpacetimeDB client connected");

    // Initialize LLM client if IRC is enabled
    let llm_client = if args.should_enable_irc() {
        // println!("🤖 Initializing LLM client for IRC...");
        Some(Arc::new(LlmClient::new()))
    } else {
        None
    };

    // Initialize visual memory if vision is enabled
    let visual_memory = if args.should_enable_vision() {
        // println!("🎨 Visual memory initialized (vision will be created in bot threads)");
        Some(Arc::new(Mutex::new(VisualMemory::new())))
    } else {
        None
    };

    // IRC is disabled - stub out the handle
    let _irc_handle: Option<std::thread::JoinHandle<()>> = None;
    // Suppress unused variable warnings
    let _ = (&args, &sage, &llm_client, &visual_memory, &db_client);

    // Display subsystem status (suppressed for clean TUI)
    // println!("\n📊 Active subsystems:");
    // println!("  ✓ Neural Cellular Automata (NCA)");
    // println!("  ✓ SpacetimeDB persistence");
    // if args.should_enable_irc() {
    //     println!("  ✓ IRC bot ({})", if args.should_enable_autonomous() { "autonomous" } else { "basic" });
    // }
    // if args.should_enable_vision() {
    //     println!("  ✓ Vision system");
    // }
    // if args.should_enable_autonomous() {
    //     println!("  ✓ Autonomous consciousness (dreams + curiosity)");
    // }
    // println!();

    // Register main TUI with Control Center
    let log_path = "/tmp/sage_main.log".to_string();
    let instance_info = InstanceInfo::new(
        InstanceType::MainTui,
        std::process::id(),
        log_path,
    );
    {
        let mut registry = InstanceRegistry::load();
        registry.register(instance_info).ok();
    }

    // Spawn heartbeat thread for main TUI
    std::thread::spawn(move || {
        use std::time::Duration;
        loop {
            std::thread::sleep(Duration::from_secs(3));
            let mut reg = InstanceRegistry::load();
            reg.heartbeat(&InstanceType::MainTui).ok();
        }
    });

    // println!("🎛️  Registered with Control Center (PID: {})\n", std::process::id());

    // Run TUI if in TUI mode, otherwise block on IRC bot
    if args.should_run_tui() {
        // println!("🖥️  Starting TUI Mission Control...\n");

        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        terminal.hide_cursor()?;

        // Create and run app
        let mut app = App::new();
        let result = app.run(&mut terminal);

        // Cleanup terminal
        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        terminal.show_cursor()?;

        result
    } else {
        // IRC-only mode (no TUI)
        // println!("💬 Running in IRC-only mode (press Ctrl+C to exit)...\n");

        // Block until interrupted
        if let Some(handle) = _irc_handle {
            let _ = handle.join();
        }

        Ok(())
    }
}
