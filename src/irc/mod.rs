// IRC Bot Module - Unified interface for SAGE IRC capabilities

pub mod bot;
pub mod autonomous;

use crate::sage_experience::SageExperience;
use crate::llm_client::LlmClient;
use crate::spacetime_client::SageDbClient;
use crate::visual_memory::VisualMemory;
use std::sync::{Arc, Mutex};

/// Configuration for IRC bot
#[derive(Clone)]
pub struct IrcConfig {
    pub server: String,
    pub channel: String,
    pub nick: String,
    pub enable_vision: bool,
    pub enable_autonomous: bool,
    pub enable_llm: bool,
}

impl Default for IrcConfig {
    fn default() -> Self {
        Self {
            server: "irc.libera.chat".to_string(),
            channel: "#sage-ai".to_string(),
            nick: "SAGE".to_string(),
            enable_vision: false,
            enable_autonomous: false,
            enable_llm: true,
        }
    }
}

/// Shared state for IRC bot
pub struct IrcState {
    pub sage: Arc<Mutex<SageExperience>>,
    pub llm: Option<Arc<LlmClient>>,
    pub visual_memory: Option<Arc<Mutex<VisualMemory>>>,
    pub db_client: SageDbClient,
}

/// Start IRC bot in a background thread
///
/// Returns a join handle that can be used to wait for the bot to finish
pub fn start_irc_bot(config: IrcConfig, state: IrcState) -> std::thread::JoinHandle<()> {
    if config.enable_autonomous {
        autonomous::start_autonomous_bot(config, state)
    } else {
        bot::start_basic_bot(config, state)
    }
}

/// Start IRC bot (blocks current thread)
pub fn run_irc_bot(config: IrcConfig, state: IrcState) {
    if config.enable_autonomous {
        autonomous::run_autonomous_bot(config, state)
    } else {
        bot::run_basic_bot(config, state)
    }
}
