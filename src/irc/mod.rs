//! IRC Module - Stub (IRC functionality removed)
//!
//! This is a placeholder to maintain compatibility while IRC is disabled.

use std::sync::{Arc, Mutex};
use crate::sage_experience::SageExperience;
use crate::llm_client::LlmClient;
use crate::visual_memory::VisualMemory;
use crate::spacetime_client::SageDbClient;

/// IRC configuration (stub)
#[derive(Debug, Clone, Default)]
pub struct IrcConfig {
    pub server: String,
    pub port: u16,
    pub nickname: String,
    pub channel: String,
    pub nick: String,
    pub enable_vision: bool,
    pub enable_autonomous: bool,
    pub enable_llm: bool,
}

/// IRC state (stub)
pub struct IrcState {
    pub connected: bool,
    pub messages: Vec<String>,
    pub sage: Arc<Mutex<SageExperience>>,
    pub llm: Option<LlmClient>,
    pub visual_memory: Arc<Mutex<VisualMemory>>,
    pub db_client: SageDbClient,
}

impl IrcConfig {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for IrcState {
    fn default() -> Self {
        Self {
            connected: false,
            messages: Vec::new(),
            sage: Arc::new(Mutex::new(SageExperience::new())),
            llm: None,
            visual_memory: Arc::new(Mutex::new(VisualMemory::new())),
            db_client: SageDbClient::new("sage-db"),
        }
    }
}

/// Spawn IRC bot (stub - does nothing)
pub fn spawn_irc_bot(_config: IrcConfig, _state: IrcState) -> std::thread::JoinHandle<()> {
    std::thread::spawn(|| {
        // IRC disabled - do nothing
    })
}
