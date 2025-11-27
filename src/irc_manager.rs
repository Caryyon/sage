//! IRC Manager - Stub module (IRC functionality removed)
//!
//! This is a placeholder to maintain compatibility while IRC is disabled.

/// IRC response message (stub)
#[derive(Debug, Clone, Default)]
pub struct IrcResponse {
    pub channel: String,
    pub message: String,
    pub opinion_type: String,
    pub loss: f64,
}

/// IRC message (stub)
#[derive(Debug, Clone, Default)]
pub struct IrcMessage {
    pub sender: String,
    pub content: String,
    pub message: String,
    pub channel: String,
    pub sage_response: String,
    pub concepts_mentioned: Vec<String>,
    pub emotional_tone: f64,
}

/// IRC Manager - manages IRC bot connection (stub - does nothing)
pub struct IrcManager;

impl IrcManager {
    pub fn start() -> Self {
        Self
    }

    pub fn poll_messages(&self) -> Vec<IrcMessage> {
        Vec::new()
    }

    pub fn send_response(&self, _response: IrcResponse) -> Result<(), String> {
        Ok(())
    }

    pub fn is_connected(&self) -> bool {
        false
    }
}
