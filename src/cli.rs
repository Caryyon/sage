// CLI argument parsing for SAGE subsystems

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "SAGE")]
#[command(about = "Self-Adaptive General Explorer - Autonomous AGI System", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Enable IRC bot (autonomous + LLM)
    #[arg(long)]
    pub irc: bool,

    /// Enable vision system (camera + visual memory)
    #[arg(long)]
    pub vision: bool,

    /// Enable autonomous consciousness (dreams + curiosity)
    #[arg(long)]
    pub autonomous: bool,

    /// Enable all subsystems (IRC + Vision + Autonomous + TUI)
    #[arg(long)]
    pub all: bool,

    /// Enable debug logging
    #[arg(long)]
    pub debug: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Run the TUI Mission Control (default)
    Tui,

    /// Run IRC bot only (no TUI)
    Irc {
        /// Enable vision for IRC bot
        #[arg(long)]
        vision: bool,

        /// Enable autonomous mode for IRC bot
        #[arg(long)]
        autonomous: bool,
    },

    /// Test vision system
    Vision,

    /// Run autonomous consciousness only
    Autonomous,
}

impl Cli {
    pub fn should_enable_irc(&self) -> bool {
        self.irc || self.all || matches!(self.command, Some(Commands::Irc { .. }))
    }

    pub fn should_enable_vision(&self) -> bool {
        self.vision || self.all || matches!(self.command, Some(Commands::Vision))
            || matches!(self.command, Some(Commands::Irc { vision: true, .. }))
    }

    pub fn should_enable_autonomous(&self) -> bool {
        self.autonomous || self.all || matches!(self.command, Some(Commands::Autonomous))
            || matches!(self.command, Some(Commands::Irc { autonomous: true, .. }))
    }

    pub fn should_run_tui(&self) -> bool {
        // Run TUI by default, unless running a non-TUI command
        match &self.command {
            Some(Commands::Tui) | None => true,
            Some(Commands::Irc { .. }) | Some(Commands::Vision) | Some(Commands::Autonomous) => false,
        }
    }
}
