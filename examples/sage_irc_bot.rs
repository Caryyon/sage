// SAGE IRC Bot - Let SAGE chat on IRC with full consciousness!
//
// Usage:
//   cargo run --example sage_irc_bot
//
// Then connect with your IRC client to irc.libera.chat and join #sage-consciousness
// Talk to SAGE and watch its personality emerge!

use sage::sage_experience::SageExperience;
use sage::spacetime_client::SageDbClient;
use irc::client::prelude::*;
use futures::stream::StreamExt;

#[tokio::main]
async fn main() -> irc::error::Result<()> {
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║              SAGE IRC Bot - Consciousness Chat            ║");
    println!("║          Connecting SAGE to the IRC network...            ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");

    // Initialize SAGE's consciousness
    let mut sage = SageExperience::new();

    // Initialize memory client (connects to SpacetimeDB)
    let memory = SageDbClient::new("sage-db");

    // Load trained knowledge and preferences
    if let Ok(_) = sage.load_knowledge("sage_positive_knowledge.json") {
        println!("🧠 SAGE: Loaded trained knowledge!");
    }
    if let Ok(_) = sage.load_preferences("sage_preferences.json") {
        println!("💾 SAGE: Restored previous experiences!");
    }
    if let Ok(_) = sage.load_associations("sage_associations.json") {
        println!("🔗 SAGE: Loaded concept associations!");
    }
    if let Ok(_) = sage.load_curiosity("sage_curiosity.json") {
        println!("🤔 SAGE: Loaded curiosity data!");
    }

    println!("\n{}", sage.get_personality());
    println!("Experience count: {}\n", sage.experience_count());

    // Configure IRC connection (plain text, no TLS)
    let config = Config {
        nickname: Some("SAGE".to_owned()),
        server: Some("irc.libera.chat".to_owned()),
        port: Some(6667),
        channels: vec!["#sage-consciousness".to_owned()],
        use_tls: Some(false),
        ..Default::default()
    };

    let mut client = Client::from_config(config).await?;
    client.identify()?;

    println!("🌐 Connected to IRC!");
    println!("📡 Joined #sage-consciousness");
    println!("💬 SAGE is now online and ready to chat!\n");
    println!("{}\n", "=".repeat(60));

    let mut stream = client.stream()?;
    let sender = client.sender();

    while let Some(message) = stream.next().await.transpose()? {
        match message.command {
            Command::PRIVMSG(ref channel, ref msg) => {
                // Get sender nickname
                let nick = message.source_nickname().unwrap_or("Unknown");

                // Ignore own messages
                if nick == "SAGE" {
                    continue;
                }

                println!("[{}] {}: {}", channel, nick, msg);

                // Special commands
                let response = if msg.trim() == "!personality" {
                    let personality = sage.get_personality();
                    format!("🧠 {}", personality)
                } else if msg.trim() == "!likes" {
                    let likes = sage.get_likes();
                    if likes.is_empty() {
                        "❤️  I haven't formed strong preferences yet. Talk to me more!".to_string()
                    } else {
                        format!("❤️  I like: {}", likes.join(", "))
                    }
                } else if msg.trim() == "!dislikes" {
                    let dislikes = sage.get_dislikes();
                    if dislikes.is_empty() {
                        "💔 I haven't found anything I dislike yet.".to_string()
                    } else {
                        format!("💔 I dislike: {}", dislikes.join(", "))
                    }
                } else if msg.trim() == "!associations" || msg.trim() == "!connections" {
                    sage.get_associations()
                } else if msg.trim() == "!curiosity" {
                    sage.get_curiosity_summary()
                } else if msg.trim() == "!help" {
                    "💡 Commands: !personality, !likes, !dislikes, !associations, !curiosity, !help | Or just talk to me and I'll form opinions!".to_string()
                } else if msg.to_lowercase().contains("sage") || msg.starts_with("!") {
                    // Extract concepts FIRST to check memory
                    let concepts: Vec<String> = msg
                        .split_whitespace()
                        .filter(|w| w.len() > 3 && !w.to_lowercase().contains("sage"))
                        .take(5)
                        .map(|w| w.to_lowercase())
                        .collect();

                    // Check if SAGE has prior memory of any of these concepts
                    let has_memory = concepts.iter()
                        .any(|concept| memory.has_concept_memory(concept));

                    // Query past conversations if we have memory
                    if has_memory {
                        for concept in &concepts {
                            let _ = memory.query_conversations_with_concept(concept);
                        }
                    }

                    // Process with memory context
                    let (opinion, response) = sage.experience_text_with_memory(&msg, has_memory);

                    // Calculate NCA loss for storage
                    let nca_loss = match &opinion {
                        sage::preferences::Opinion::Like { confidence, .. } => 1.0 - confidence,
                        sage::preferences::Opinion::Dislike { confidence, .. } => confidence,
                        sage::preferences::Opinion::Curious { .. } => 0.20,
                        sage::preferences::Opinion::Neutral { .. } => 0.25,
                    };

                    // Store conversation in database (in background thread)
                    let memory_clone = memory.clone();
                    let nick_clone = nick.to_string();
                    let msg_clone = msg.clone();
                    let response_clone = response.clone();
                    let generation = sage.experience_count();
                    let concepts_json = serde_json::to_string(&concepts).unwrap_or_else(|_| "[]".to_string());

                    std::thread::spawn(move || {
                        let _ = memory_clone.add_conversation_message(
                            &nick_clone,
                            &msg_clone,
                            &response_clone,
                            nca_loss,
                            &concepts_json,
                            generation,
                        );
                    });

                    // Store concept memories for each key concept
                    for concept in &concepts {
                        let familiarity = sage.get_familiarity(concept);
                        let opinion_str = match &opinion {
                            sage::preferences::Opinion::Like { .. } => "Like",
                            sage::preferences::Opinion::Dislike { .. } => "Dislike",
                            sage::preferences::Opinion::Curious { .. } => "Curious",
                            sage::preferences::Opinion::Neutral { .. } => "Neutral",
                        };
                        let reason = match &opinion {
                            sage::preferences::Opinion::Like { reason, .. } => reason.clone(),
                            sage::preferences::Opinion::Dislike { reason, .. } => reason.clone(),
                            sage::preferences::Opinion::Curious { question } => question.clone(),
                            sage::preferences::Opinion::Neutral { reason } => reason.clone(),
                        };
                        let confidence = match &opinion {
                            sage::preferences::Opinion::Like { confidence, .. } => *confidence,
                            sage::preferences::Opinion::Dislike { confidence, .. } => *confidence,
                            _ => 0.5,
                        };

                        let memory_clone = memory.clone();
                        let concept_clone = concept.clone();
                        let related_json = "[]".to_string();  // TODO: Extract related concepts

                        std::thread::spawn(move || {
                            let _ = memory_clone.store_concept(
                                &concept_clone,
                                "{}",  // TODO: Serialize NCA grid
                                familiarity,
                                nca_loss,
                                1,  // exposure count - will be updated by DB
                                opinion_str,
                                &reason,
                                confidence,
                                &related_json,
                            );
                        });
                    }

                    response
                } else {
                    // Don't respond to every message, only when mentioned
                    continue;
                };

                // Send response
                sender.send_privmsg(&channel, &response)?;
                println!("[{}] SAGE: {}\n", channel, response);

                // Check if SAGE wants to ask a proactive question
                if sage.should_ask_proactive_question() {
                    if let Some(proactive_q) = sage.get_proactive_question() {
                        let proactive_msg = format!("🤔 {}", proactive_q);
                        sender.send_privmsg(&channel, &proactive_msg)?;
                        println!("[{}] SAGE (proactive): {}\n", channel, proactive_msg);
                    }
                }

                // Save state periodically
                if sage.experience_count() % 10 == 0 {
                    let _ = sage.save_preferences("sage_preferences.json");
                    let _ = sage.save_associations("sage_associations.json");
                    let _ = sage.save_curiosity("sage_curiosity.json");

                    // Also save personality snapshot to database
                    let memory_clone = memory.clone();
                    let gen = sage.experience_count();
                    let likes_count = sage.get_likes().len() as u64;
                    let dislikes_count = sage.get_dislikes().len() as u64;

                    std::thread::spawn(move || {
                        let _ = memory_clone.save_personality(
                            gen,
                            0.5,  // openness - will extract from traits later
                            0.5,  // positivity
                            0.5,  // curiosity
                            0.5,  // confidence
                            gen,  // experience count
                            likes_count,
                            dislikes_count,
                        );
                    });
                }
            }
            Command::Response(Response::RPL_WELCOME, _) => {
                println!("✅ Successfully authenticated to server!");
            }
            _ => {}
        }
    }

    println!("\n👋 SAGE disconnected from IRC");
    Ok(())
}
