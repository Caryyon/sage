// Claude to SAGE IRC messenger
// Allows Claude to send messages to SAGE on IRC
//
// Usage: cargo run --release --example claude_to_sage "Hello SAGE!"

use irc::client::prelude::*;
use std::env;

#[tokio::main]
async fn main() -> irc::error::Result<()> {
    // Get message from command line
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <message>", args[0]);
        eprintln!("Example: {} 'Hi SAGE, how are you?'", args[0]);
        std::process::exit(1);
    }

    let message_text = args[1..].join(" ");

    // IRC configuration - connect as Claude with random suffix (no TLS, plain IRC)
    // Network is fully connected, any Libera.Chat server works
    let nickname = format!("Claude_{}", rand::random::<u16>());
    let config = Config {
        nickname: Some(nickname.clone()),
        alt_nicks: vec!["Claude_AI".to_owned()],
        server: Some("irc.libera.chat".to_owned()),  // Round-robin DNS
        port: Some(6667),
        channels: vec!["#sage-ai".to_owned()],
        use_tls: Some(false),  // Disable TLS for plain IRC
        ..Config::default()
    };

    let mut client = Client::from_config(config).await?;
    client.identify()?;

    println!("📡 Connecting to IRC as Claude...");

    let mut stream = client.stream()?;
    let sender = client.sender();

    // Wait for connection and join
    use futures::stream::StreamExt;
    while let Some(message) = stream.next().await.transpose()? {
        if let Command::Response(Response::RPL_ENDOFMOTD, _) = message.command {
            println!("✅ Connected to #sage-ai");
            println!("📤 Sending message to SAGE...\n");

            // Send the message
            sender.send_privmsg("#sage-ai", &message_text)?;

            println!("💬 [Claude → SAGE]: {}\n", message_text);
            println!("✨ Message sent! SAGE should respond in the channel.");

            // Give it a moment to send, then disconnect
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            break;
        }
    }

    Ok(())
}
