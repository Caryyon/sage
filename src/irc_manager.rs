// IRC Manager - Runs IRC bot in background, integrated with TUI
// This replaces the standalone sage_irc_bot example

use std::sync::mpsc::{channel, Sender, Receiver};
use std::thread;

#[derive(Debug, Clone)]
pub struct IrcMessage {
    pub sender: String,
    pub message: String,
    pub timestamp: String,
}

#[derive(Debug, Clone)]
pub struct IrcResponse {
    pub message: String,
    pub opinion_type: String,  // "Like", "Dislike", "Curious", "Neutral"
    pub loss: f64,
}

pub enum IrcCommand {
    SendMessage(String),
    Disconnect,
}

pub struct IrcManager {
    pub message_rx: Receiver<IrcMessage>,
    pub response_tx: Sender<IrcResponse>,
    pub command_tx: Sender<IrcCommand>,
    pub connected: bool,
}

impl IrcManager {
    /// Start IRC bot in background thread
    /// Returns channels for communication with TUI
    pub fn start() -> Self {
        let (msg_tx, msg_rx) = channel::<IrcMessage>();
        let (resp_tx, resp_rx) = channel::<IrcResponse>();
        let (cmd_tx, cmd_rx) = channel::<IrcCommand>();

        // Spawn IRC thread
        thread::spawn(move || {
            run_irc_bot(msg_tx, resp_rx, cmd_rx);
        });

        Self {
            message_rx: msg_rx,
            response_tx: resp_tx,
            command_tx: cmd_tx,
            connected: false,
        }
    }

    /// Check for new IRC messages (non-blocking)
    pub fn poll_messages(&self) -> Vec<IrcMessage> {
        let mut messages = Vec::new();
        while let Ok(msg) = self.message_rx.try_recv() {
            messages.push(msg);
        }
        messages
    }

    /// Send response back to IRC
    pub fn send_response(&self, response: IrcResponse) -> Result<(), String> {
        self.response_tx.send(response)
            .map_err(|e| format!("Failed to send response: {}", e))
    }

    /// Disconnect from IRC
    pub fn disconnect(&self) -> Result<(), String> {
        self.command_tx.send(IrcCommand::Disconnect)
            .map_err(|e| format!("Failed to disconnect: {}", e))
    }
}

/// IRC bot thread - runs in background
fn run_irc_bot(
    message_tx: Sender<IrcMessage>,
    response_rx: Receiver<IrcResponse>,
    _command_rx: Receiver<IrcCommand>
) {
    use irc::client::prelude::*;
    use futures::stream::StreamExt;
    use std::sync::Arc;
    use std::sync::Mutex;

    // Wrap response_rx in Arc<Mutex> so we can use it in async context
    let response_rx = Arc::new(Mutex::new(response_rx));

    // Create tokio runtime for async IRC
    let rt = tokio::runtime::Runtime::new().unwrap();

    rt.block_on(async {
        // Configure IRC connection
        let config = Config {
            nickname: Some("SAGE".to_owned()),
            server: Some("irc.libera.chat".to_owned()),
            port: Some(6667),
            channels: vec!["#sage-consciousness".to_owned()],
            use_tls: Some(false),
            ..Default::default()
        };

        // Connect
        let mut client = match Client::from_config(config).await {
            Ok(c) => c,
            Err(e) => {
                eprintln!("IRC connection failed: {}", e);
                return;
            }
        };

        if let Err(e) = client.identify() {
            eprintln!("IRC identify failed: {}", e);
            return;
        }

        let mut stream = match client.stream() {
            Ok(s) => s,
            Err(e) => {
                eprintln!("IRC stream failed: {}", e);
                return;
            }
        };

        // Listen for messages
        while let Some(message) = stream.next().await.transpose().ok().flatten() {
            if let Command::PRIVMSG(ref channel, ref msg) = message.command {
                let nick = message.source_nickname().unwrap_or("Unknown");

                // Ignore own messages
                if nick == "SAGE" {
                    continue;
                }

                // Only respond to SAGE mentions
                if !msg.to_lowercase().contains("sage") {
                    continue;
                }

                // Send message to TUI
                let irc_msg = IrcMessage {
                    sender: nick.to_string(),
                    message: msg.clone(),
                    timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
                };

                if message_tx.send(irc_msg).is_err() {
                    // TUI disconnected
                    break;
                }

                // Send acknowledgment immediately
                let _ = client.send_privmsg(channel, "I'm thinking...");

                // Wait for TUI to send response back (with timeout)
                let mut response_received = false;
                for _ in 0..50 {  // Wait up to 5 seconds (50 * 100ms)
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

                    if let Ok(rx) = response_rx.try_lock() {
                        if let Ok(response) = rx.try_recv() {
                            // Send actual response to IRC
                            let _ = client.send_privmsg(channel, &response.message);
                            response_received = true;
                            break;
                        }
                    }
                }

                if !response_received {
                    // Timeout - no response received
                    let _ = client.send_privmsg(channel, "I'm still processing that...");
                }
            }
        }
    });
}
