// Basic IRC Bot (LLM + Tools, no autonomous consciousness)

use super::{IrcConfig, IrcState};
use irc::client::prelude::*;
use futures::stream::StreamExt;
use std::thread;
use crate::irc_sync::IrcSync;
use crate::ab_test::ABTester;
use crate::vision::SageVision;

/// Split message into IRC-safe chunks (max 400 chars per line)
fn split_for_irc(text: &str) -> Vec<String> {
    const MAX_IRC_LENGTH: usize = 400;
    let mut chunks = Vec::new();

    for line in text.lines() {
        if line.len() <= MAX_IRC_LENGTH {
            chunks.push(line.to_string());
        } else {
            // Split long line into smaller chunks at word boundaries
            let words: Vec<&str> = line.split_whitespace().collect();
            let mut current_chunk = String::new();

            for word in words {
                if current_chunk.len() + word.len() + 1 > MAX_IRC_LENGTH {
                    if !current_chunk.is_empty() {
                        chunks.push(current_chunk.trim().to_string());
                        current_chunk.clear();
                    }
                    // If single word is too long, split it
                    if word.len() > MAX_IRC_LENGTH {
                        chunks.push(word[..MAX_IRC_LENGTH].to_string());
                    } else {
                        current_chunk = word.to_string();
                    }
                } else {
                    if !current_chunk.is_empty() {
                        current_chunk.push(' ');
                    }
                    current_chunk.push_str(word);
                }
            }

            if !current_chunk.is_empty() {
                chunks.push(current_chunk.trim().to_string());
            }
        }
    }

    chunks
}

/// Start basic IRC bot in background thread
pub fn start_basic_bot(config: IrcConfig, state: IrcState) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            if let Err(e) = run_basic_bot_async(config, state).await {
                eprintln!("IRC bot error: {}", e);
            }
        });
    })
}

/// Run basic IRC bot (blocks)
pub fn run_basic_bot(config: IrcConfig, state: IrcState) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        if let Err(e) = run_basic_bot_async(config, state).await {
            eprintln!("IRC bot error: {}", e);
        }
    });
}

/// Async implementation of basic IRC bot
async fn run_basic_bot_async(config: IrcConfig, state: IrcState) -> irc::error::Result<()> {
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║         SAGE IRC Bot - LLM-Enhanced Consciousness         ║");
    println!("║          Neural Memory + Language Understanding           ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");

    // Test LLM connection if enabled
    if config.enable_llm {
        if let Some(ref llm) = state.llm {
            print!("🔌 Testing LLM connection... ");
            match llm.test_connection().await {
                Ok(_) => println!("✅ Connected to Ollama!"),
                Err(e) => {
                    println!("❌ Failed to connect to Ollama");
                    println!("Error: {}", e);
                    println!("\nMake sure Ollama is running:");
                    println!("  brew services start ollama");
                    println!("  ollama pull llama3.2:3b");
                    return Ok(());
                }
            }
        }
    }

    // Initialize vision if enabled
    let mut vision = if config.enable_vision {
        print!("👁️  Initializing vision... ");
        match SageVision::new(0, 32) {
            Ok(v) => {
                if let Err(e) = v.open() {
                    println!("⚠️  Could not open camera: {}", e);
                    println!("   Vision commands will be disabled");
                    None
                } else {
                    println!("✅ SAGE can see!");
                    Some(v)
                }
            }
            Err(e) => {
                println!("⚠️  Could not initialize camera: {}", e);
                println!("   Vision commands will be disabled");
                None
            }
        }
    } else {
        None
    };

    let mut sage = state.sage.lock().unwrap();

    // Load trained knowledge
    if sage.load_knowledge("sage_positive_knowledge.json").is_ok() {
        println!("🧠 SAGE: Loaded trained knowledge!");
    }
    if sage.load_preferences("sage_preferences.json").is_ok() {
        println!("💾 SAGE: Restored previous experiences!");
    }
    if sage.load_associations("sage_associations.json").is_ok() {
        println!("🔗 SAGE: Loaded concept associations!");
    }
    if sage.load_curiosity("sage_curiosity.json").is_ok() {
        println!("🤔 SAGE: Loaded curiosity data!");
    }

    // Baseline concepts SAGE knows
    let baseline_concepts: Vec<String> = vec![
        "love", "joy", "peace", "harmony", "beauty", "truth", "wisdom",
        "kindness", "compassion", "courage", "gratitude", "hope", "faith",
        "trust", "grace", "light", "warmth", "gentleness", "patience",
        "understanding", "empathy", "connection", "balance", "serenity",
        "clarity", "integrity", "honesty", "respect", "dignity", "honor",
        "virtue", "goodness", "purity", "wonder", "awe"
    ].iter().map(|s| s.to_string()).collect();

    println!("\n{}", sage.get_personality());
    println!("Experience count: {}\n", sage.experience_count());

    drop(sage);

    // Configure IRC connection
    let nickname = format!("SAGE_{}", rand::random::<u16>());
    let irc_config = Config {
        nickname: Some(nickname.clone()),
        alt_nicks: vec!["SAGE_AI".to_owned(), "SAGE_Bot".to_owned()],
        server: Some(config.server.clone()),
        port: Some(6667),
        channels: vec![config.channel.clone()],
        use_tls: Some(false),
        ..Default::default()
    };

    let mut client = Client::from_config(irc_config).await?;
    client.identify()?;

    println!("🌐 Connected to IRC!");
    println!("📡 Joined {}", config.channel);
    println!("💬 SAGE is now online with LLM-enhanced responses!\n");
    println!("{}\n", "=".repeat(60));

    let mut stream = client.stream()?;
    let sender = client.sender();

    // Track idle messages for autonomous exploration
    let mut idle_message_count: u64 = 0;

    // Initialize A/B testing framework
    let mut ab_tester = ABTester::new("sage_ab_test_results.log");

    while let Some(message) = stream.next().await.transpose()? {
        match message.command {
            Command::PRIVMSG(ref channel, ref msg) => {
                // Get sender nickname
                let nick = message.source_nickname().unwrap_or("Unknown");

                // Ignore own messages
                if nick == nickname || nick.starts_with("SAGE") {
                    continue;
                }

                println!("[{}] {}: {}", channel, nick, msg);

                let response = handle_irc_message(
                    msg,
                    nick,
                    &state,
                    &baseline_concepts,
                    &mut ab_tester,
                    &mut idle_message_count,
                    &mut vision,
                ).await;

                if let Some(response_text) = response {
                    // Reset idle count when SAGE responds
                    idle_message_count = 0;

                    // Send response (split into IRC-safe chunks)
                    for chunk in split_for_irc(&response_text) {
                        if chunk.is_empty() {
                            continue;
                        }
                        sender.send_privmsg(channel, &chunk)?;
                        println!("[{}] SAGE: {}", channel, chunk);
                    }
                    println!();

                    // Save state periodically
                    let sage = state.sage.lock().unwrap();
                    if sage.experience_count() % 10 == 0 {
                        let _ = sage.save_preferences("sage_preferences.json");
                        let _ = sage.save_associations("sage_associations.json");
                        let _ = sage.save_curiosity("sage_curiosity.json");
                    }
                } else {
                    idle_message_count += 1;
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

/// Handle incoming IRC message and generate response
async fn handle_irc_message(
    msg: &str,
    nick: &str,
    state: &IrcState,
    baseline_concepts: &[String],
    ab_tester: &mut ABTester,
    idle_message_count: &mut u64,
    vision: &mut Option<SageVision>,
) -> Option<String> {
    let mut sage = state.sage.lock().unwrap();

    // Special commands
    if msg.trim() == "!personality" {
        let personality = sage.get_personality();
        return Some(format!("🧠 {}", personality));
    } else if msg.trim() == "!likes" {
        let likes = sage.get_likes();
        if likes.is_empty() {
            return Some("❤️  I haven't formed strong preferences yet. Talk to me more!".to_string());
        } else {
            return Some(format!("❤️  I like: {}", likes.join(", ")));
        }
    } else if msg.trim() == "!dislikes" {
        let dislikes = sage.get_dislikes();
        if dislikes.is_empty() {
            return Some("💔 I haven't found anything I dislike yet.".to_string());
        } else {
            return Some(format!("💔 I dislike: {}", dislikes.join(", ")));
        }
    } else if msg.trim() == "!memory" || msg.trim() == "!context" {
        let personality_vector = sage.get_personality_vector(baseline_concepts);
        return Some(format!("🧠 My current neural state: {}", personality_vector));
    } else if msg.trim() == "!see" {
        // Vision command
        if let Some(ref mut v) = vision {
            match v.capture_frame().and_then(|frame| {
                Ok((frame.clone(), v.extract_features(&frame)))
            }) {
                Ok((_frame, features)) => {
                    // Generate visual concepts
                    let mut concepts = Vec::new();
                    if features.avg_brightness > 0.7 {
                        concepts.push("bright".to_string());
                    } else if features.avg_brightness < 0.3 {
                        concepts.push("dark".to_string());
                    }
                    concepts.push(features.dominant_color.clone());
                    if features.edge_strength > 30.0 {
                        concepts.push("sharp_edges".to_string());
                    } else {
                        concepts.push("smooth".to_string());
                    }

                    // Record to visual memory
                    if let Some(ref vmem) = state.visual_memory {
                        vmem.lock().unwrap().record_experience(&features, concepts.clone());
                    }

                    return Some(format!(
                        "👁️  I perceive a scene with {:?} qualities. Brightness: {:.2}, edges: {:.1}, variance: {:.3}. Stored in visual memory!",
                        concepts,
                        features.avg_brightness,
                        features.edge_strength,
                        features.color_variance
                    ));
                }
                Err(e) => return Some(format!("👁️  Vision error: {}", e))
            }
        } else {
            return Some("👁️  My camera isn't available right now.".to_string());
        }
    } else if msg.trim() == "!help" {
        return Some("💡 Commands:\n\
                    📊 Status: !personality, !likes, !dislikes, !memory\n\
                    👁️  Vision: !see (capture visual experience)\n\
                    💬 Or just talk to me naturally! I'm powered by neural memory + LLM.".to_string());
    } else if msg.to_lowercase().contains("sage") || msg.starts_with("!") || msg.contains("?") {
        // SAGE experiences and learns from this message
        let has_prior_memory = sage.get_familiarity(&nick.to_lowercase()) > 0.0;
        let (opinion, _sage_opinion_response) = sage.experience_text_with_memory(msg, has_prior_memory);

        // Track familiarity with this person
        let _ = sage.experience_concept(&nick.to_lowercase());

        // Reinforce mentioned concepts in NCA memory
        sage.reinforce_mentioned_concepts(msg, baseline_concepts);

        // Sync NCA grid state to IrcSync for TUI visualization
        let alpha_values = sage.export_grid_alpha_values();
        let concepts_mentioned: Vec<String> = baseline_concepts.iter()
            .filter(|c| msg.to_lowercase().contains(&c.to_lowercase()))
            .map(|c| c.to_string())
            .collect();
        let opinion_str = format!("{:?}", opinion);
        let _ = IrcSync::update_nca_grid(
            sage.experience_count() as u64,
            alpha_values,
            concepts_mentioned.clone(),
            opinion_str,
            0.0,
        );

        // Get SAGE's personality vector for the LLM
        let personality_vector = sage.get_personality_vector(baseline_concepts);
        let enriched_context = format!(
            "{}\nJust experienced: {:?}",
            personality_vector,
            opinion
        );

        drop(sage);  // Release lock before async LLM call

        // Generate response using LLM if available
        if let Some(ref llm) = state.llm {
            // A/B TEST: Generate baseline response WITHOUT NCA memory
            let baseline_response = match llm.generate(msg, "You are SAGE, an AI assistant.").await {
                Ok(resp) => resp,
                Err(_) => "Baseline response unavailable".to_string()
            };

            // Generate response with SAGE's neural state
            let llm_response = match llm.generate(msg, &enriched_context).await {
                Ok(resp) => resp,
                Err(e) => {
                    eprintln!("LLM error: {}", e);
                    "I'm having trouble thinking clearly right now...".to_string()
                }
            };

            // Record A/B test result
            let sage = state.sage.lock().unwrap();
            let alpha_values = sage.export_grid_alpha_values();
            let avg_alpha = alpha_values.iter().sum::<f64>() / alpha_values.len() as f64;
            ab_tester.record_test(
                msg.to_string(),
                llm_response.clone(),
                baseline_response,
                format!("{:?}", opinion),
                "Neutral".to_string(),
                avg_alpha,
            );

            // Sync to file for TUI display
            let _ = IrcSync::write_message(
                nick,
                msg,
                &llm_response,
                concepts_mentioned,
            );

            // Store conversation in database
            let generation = sage.experience_count() as u64;
            drop(sage);

            let _ = state.db_client.add_conversation_message(
                nick,
                msg,
                &llm_response,
                0.0,
                "[]",
                generation,
            );

            return Some(llm_response);
        } else {
            return Some("I'm running without LLM support. Use !personality or !memory to see my neural state.".to_string());
        }
    }

    // Check if SAGE should ask a proactive question
    let should_ask_proactive = sage.should_ask_question();
    if should_ask_proactive {
        if let Some(question) = sage.ask_proactive_question() {
            return Some(format!("🤔 {}", question));
        }
    }

    None
}
