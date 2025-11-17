// SAGE IRC Bot with LLM - Natural conversation powered by Llama 3
//
// Usage:
//   cargo run --release --example sage_irc_llm_bot
//
// Then connect with your IRC client to irc.libera.chat and join #sage-ai
// Talk to SAGE naturally - its NCA memory influences LLM responses!

use sage::sage_experience::SageExperience;
use sage::llm_client::LlmClient;
use sage::spacetime_client::SageDbClient;
use sage::irc_sync::IrcSync;
use sage::ab_test::ABTester;
use sage::vision::SageVision;
use sage::visual_memory::VisualMemory;
use irc::client::prelude::*;
use futures::stream::StreamExt;
use std::sync::{Arc, Mutex};

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

#[tokio::main]
async fn main() -> irc::error::Result<()> {
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║         SAGE IRC Bot - LLM-Enhanced Consciousness         ║");
    println!("║          Neural Memory + Language Understanding           ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");

    // Initialize SAGE's consciousness (NCA memory)
    let mut sage = SageExperience::new();

    // Initialize LLM client
    let llm = LlmClient::new();

    // Test Ollama connection
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

    // Initialize memory client (connects to SpacetimeDB)
    let memory = SageDbClient::new("sage-db");

    // Initialize A/B testing framework
    let mut ab_tester = ABTester::new("sage_ab_test_results.log");

    // Load trained knowledge and preferences
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

    // Initialize vision system
    let vision = match SageVision::new(0, 32) {
        Ok(mut v) => {
            if let Err(e) = v.open() {
                println!("⚠️  Camera not available: {} (vision commands will be disabled)\n", e);
                None
            } else {
                println!("👁️  Initializing vision... ✅ SAGE can see!");
                Some(Arc::new(Mutex::new(v)))
            }
        }
        Err(e) => {
            println!("⚠️  Vision initialization failed: {} (vision commands will be disabled)\n", e);
            None
        }
    };

    // Initialize visual memory
    let visual_memory = Arc::new(Mutex::new(VisualMemory::new()));
    println!("🎨 Visual memory initialized for cross-modal learning!\n");

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

    // Configure IRC connection (plain text, no TLS)
    // Use a unique nickname to avoid conflicts
    let nickname = format!("SAGE_{}", rand::random::<u16>());
    let config = Config {
        nickname: Some(nickname.clone()),
        alt_nicks: vec!["SAGE_AI".to_owned(), "SAGE_Bot".to_owned()],
        server: Some("irc.libera.chat".to_owned()),
        port: Some(6667),
        channels: vec!["#sage-ai".to_owned()],
        use_tls: Some(false),
        ..Default::default()
    };

    let mut client = Client::from_config(config).await?;
    client.identify()?;

    println!("🌐 Connected to IRC!");
    println!("📡 Joined #sage-ai");
    println!("💬 SAGE is now online with LLM-enhanced responses!\n");
    println!("{}\n", "=".repeat(60));

    let mut stream = client.stream()?;
    let sender = client.sender();

    // Track idle messages for autonomous exploration
    let mut idle_message_count: u64 = 0;

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

                // Check if SAGE should ask a proactive question (every 50 experiences)
                let should_ask_proactive = sage.should_ask_question();

                // Check if SAGE should explore autonomously (after idle period)
                let should_explore = sage.should_explore(idle_message_count);

                // Check if SAGE should form goals autonomously
                let should_form_goals = sage.should_form_goals();

                // Check if SAGE should use tools for current goal
                let should_use_tool = sage.should_use_tools_for_goal();

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
                } else if msg.trim() == "!memory" || msg.trim() == "!context" {
                    let personality_vector = sage.get_personality_vector(&baseline_concepts);
                    format!("🧠 My current neural state: {}", personality_vector)
                } else if msg.trim() == "!curiosity" || msg.trim() == "!curious" {
                    let curiosity = sage.get_curiosity_state();
                    format!("🤔 {}", curiosity)
                } else if msg.trim() == "!diagnosis" || msg.trim() == "!diagnose" {
                    let diagnosis = sage.self_diagnose();
                    format!("🔧 Self-Diagnosis: {}", diagnosis)
                } else if msg.trim() == "!strengths" {
                    let strengths = sage.get_strengths();
                    if strengths.is_empty() {
                        "💪 Still discovering my strengths. Give me time to learn!".to_string()
                    } else {
                        format!("💪 My strengths: {}", strengths.join(", "))
                    }
                } else if msg.trim() == "!weaknesses" {
                    let weaknesses = sage.get_weaknesses();
                    if weaknesses.is_empty() {
                        "📚 No identified weaknesses yet. Still learning about myself!".to_string()
                    } else {
                        format!("📚 Working on: {}", weaknesses.join(", "))
                    }
                } else if msg.trim() == "!goals" {
                    let goals = sage.get_goals_summary();
                    format!("🎯 {}", goals)
                } else if msg.trim() == "!values" {
                    let values = sage.get_values_summary();
                    format!("💎 {}", values)
                } else if msg.trim() == "!tools" {
                    let tools = sage.list_tools();
                    let mut response = String::from("🛠️  Available tools:\n");
                    for (name, desc) in tools {
                        response.push_str(&format!("  • {}: {}\n", name, desc));
                    }
                    response.push_str("Use: !search <query> or !calc <expression>");
                    response
                } else if msg.trim().starts_with("!search ") {
                    let query = msg.trim().strip_prefix("!search ").unwrap();
                    match sage.use_tool("web_search", query) {
                        Ok(result) => {
                            if result.success {
                                format!("🔍 Search results for '{}': {}", query, result.output)
                            } else {
                                format!("❌ Search failed: {}", result.error.unwrap_or_default())
                            }
                        }
                        Err(e) => format!("❌ Error: {}", e)
                    }
                } else if msg.trim().starts_with("!calc ") {
                    let expression = msg.trim().strip_prefix("!calc ").unwrap();
                    match sage.use_tool("calculator", expression) {
                        Ok(result) => {
                            if result.success {
                                format!("🧮 {} = {}", expression, result.output)
                            } else {
                                format!("❌ Calculation failed: {}", result.error.unwrap_or_default())
                            }
                        }
                        Err(e) => format!("❌ Error: {}", e)
                    }
                } else if msg.trim().starts_with("!wiki ") {
                    let query = msg.trim().strip_prefix("!wiki ").unwrap();
                    match sage.use_tool("wikipedia", query) {
                        Ok(result) => {
                            if result.success {
                                format!("📚 Wikipedia: {}", result.output)
                            } else {
                                format!("❌ Wikipedia search failed: {}", result.error.unwrap_or_default())
                            }
                        }
                        Err(e) => format!("❌ Error: {}", e)
                    }
                } else if msg.trim().starts_with("!weather ") {
                    let location = msg.trim().strip_prefix("!weather ").unwrap();
                    match sage.use_tool("weather", location) {
                        Ok(result) => {
                            if result.success {
                                format!("🌤️  {}", result.output)
                            } else {
                                format!("❌ Weather request failed: {}", result.error.unwrap_or_default())
                            }
                        }
                        Err(e) => format!("❌ Error: {}", e)
                    }
                } else if msg.trim().starts_with("!news") {
                    let category = msg.trim().strip_prefix("!news").unwrap_or("all").trim();
                    let category = if category.is_empty() { "all" } else { category };
                    match sage.use_tool("news", category) {
                        Ok(result) => {
                            if result.success {
                                format!("📰 {}", result.output)
                            } else {
                                format!("❌ News request failed: {}", result.error.unwrap_or_default())
                            }
                        }
                        Err(e) => format!("❌ Error: {}", e)
                    }
                } else if msg.trim().starts_with("!time") {
                    let query = msg.trim().strip_prefix("!time").unwrap_or("now").trim();
                    let query = if query.is_empty() { "now" } else { query };
                    match sage.use_tool("time", query) {
                        Ok(result) => {
                            if result.success {
                                format!("🕐 {}", result.output)
                            } else {
                                format!("❌ Time query failed: {}", result.error.unwrap_or_default())
                            }
                        }
                        Err(e) => format!("❌ Error: {}", e)
                    }
                } else if msg.trim() == "!see" {
                    // Vision command - capture what SAGE sees through camera
                    if let Some(ref vision_sys) = vision {
                        // Lock once for both operations to avoid deadlock
                        let result = {
                            let mut v = vision_sys.lock().unwrap();
                            v.capture_frame().and_then(|frame| {
                                Ok((frame.clone(), v.extract_features(&frame)))
                            })
                        };

                        match result {
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
                                if features.color_variance > 0.15 {
                                    concepts.push("complex".to_string());
                                } else {
                                    concepts.push("simple".to_string());
                                }

                                // Record to visual memory
                                visual_memory.lock().unwrap().record_experience(&features, concepts.clone());

                                format!(
                                    "👁️  I perceive a scene with {:?} qualities. Brightness: {:.2}, edges: {:.1}, variance: {:.3}. Stored in visual memory!",
                                    concepts,
                                    features.avg_brightness,
                                    features.edge_strength,
                                    features.color_variance
                                )
                            }
                            Err(e) => format!("👁️  Vision error: {}", e)
                        }
                    } else {
                        "👁️  My camera isn't available right now.".to_string()
                    }
                } else if msg.trim() == "!dreams" || msg.trim() == "!visual_memory" {
                    // Check visual memory and dream content
                    let vmem = visual_memory.lock().unwrap();
                    let exp_count = vmem.experience_count();
                    let recent_concepts = vmem.get_recent_concepts(10);

                    if exp_count == 0 {
                        "🌙 My visual memory is empty. Use !see to let me capture what you're showing me!".to_string()
                    } else {
                        format!(
                            "🌙 Visual memory: {} experiences recorded. Recent concepts: {:?}. During idle time, I replay and remix these memories...",
                            exp_count,
                            recent_concepts
                        )
                    }
                } else if msg.trim() == "!ab_report" {
                    match ab_tester.export_metrics_report("sage_ab_test_report.md") {
                        Ok(_) => {
                            let metrics = ab_tester.calculate_metrics();
                            format!("📊 A/B Testing Report Exported!\n\
                                Total Tests: {}\n\
                                Avg Response Similarity: {:.1}%\n\
                                Memory Reference Rate: {:.1}%\n\
                                Opinion Divergence: {:.1}%\n\
                                📄 Full report saved to: sage_ab_test_report.md",
                                metrics.total_tests,
                                metrics.avg_similarity * 100.0,
                                metrics.memory_reference_rate * 100.0,
                                metrics.opinion_divergence_rate * 100.0)
                        }
                        Err(e) => format!("❌ Failed to export report: {}", e)
                    }
                } else if msg.trim() == "!help" {
                    "💡 Commands:\n\
                    📊 Status: !personality, !likes, !dislikes, !memory, !curiosity\n\
                    🔧 Introspection: !diagnosis, !strengths, !weaknesses, !goals, !values\n\
                    👁️  Vision: !see (capture visual experience), !dreams (check visual memory)\n\
                    🛠️  Tools: !tools, !search <query>, !wiki <query>, !weather <location>, !news [category], !time [query], !calc <expr>\n\
                    🧪 A/B Testing: !ab_report (shows NCA vs baseline comparison)\n\
                    💬 Or just talk to me naturally! I'm powered by neural memory + LLM + vision + dreams + self-modification + emergent goals + tools.".to_string()
                } else if msg.to_lowercase().contains("sage") || msg.starts_with("!") || msg.contains("?") {
                    // SAGE actually experiences and learns from this message
                    // This forms opinions, builds associations, and updates preferences
                    let has_prior_memory = sage.get_familiarity(&nick.to_lowercase()) > 0.0;
                    let (opinion, _sage_opinion_response) = sage.experience_text_with_memory(msg, has_prior_memory);

                    // Also track familiarity with this person
                    let _ = sage.experience_concept(&nick.to_lowercase());

                    // Reinforce mentioned concepts in NCA memory
                    sage.reinforce_mentioned_concepts(msg, &baseline_concepts);

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
                        0.0,  // No loss for conversational learning
                    );

                    // Get SAGE's personality vector (NCA-based state) for the LLM
                    let personality_vector = sage.get_personality_vector(&baseline_concepts);

                    // Add opinion context to personality vector
                    let enriched_context = format!(
                        "{}\nJust experienced: {:?}",
                        personality_vector,
                        opinion
                    );

                    // A/B TEST: Generate baseline response WITHOUT NCA memory
                    let baseline_response = match llm.generate(msg, "You are SAGE, an AI assistant.").await {
                        Ok(resp) => resp,
                        Err(_) => "Baseline response unavailable".to_string()
                    };

                    // Generate response using LLM WITH SAGE's neural state
                    let llm_response = match llm.generate(msg, &enriched_context).await {
                        Ok(resp) => resp,
                        Err(e) => {
                            eprintln!("LLM error: {}", e);
                            "I'm having trouble thinking clearly right now... Could you repeat that?".to_string()
                        }
                    };

                    // Record A/B test result
                    let alpha_values = sage.export_grid_alpha_values();
                    let avg_alpha = alpha_values.iter().sum::<f64>() / alpha_values.len() as f64;
                    ab_tester.record_test(
                        msg.clone(),
                        llm_response.clone(),
                        baseline_response,
                        format!("{:?}", opinion),
                        "Neutral".to_string(),  // Baseline has no opinion
                        avg_alpha,
                    );

                    // Extract concepts mentioned in the message
                    let concepts_mentioned: Vec<String> = baseline_concepts.iter()
                        .filter(|c| msg.to_lowercase().contains(&c.to_lowercase()))
                        .map(|c| c.to_string())
                        .collect();

                    // Sync to file for TUI display (real-time IRC feed)
                    let _ = IrcSync::write_message(
                        nick,
                        msg,
                        &llm_response,
                        concepts_mentioned,
                    );

                    // Store conversation in database (in background thread)
                    let memory_clone = memory.clone();
                    let nick_clone = nick.to_string();
                    let msg_clone = msg.clone();
                    let response_clone = llm_response.clone();
                    let generation = sage.experience_count() as u64;

                    tokio::spawn(async move {
                        let _ = memory_clone.add_conversation_message(
                            &nick_clone,
                            &msg_clone,
                            &response_clone,
                            0.0,  // No loss for LLM responses
                            "[]",
                            generation,
                        );
                    });

                    llm_response
                } else {
                    // Check if SAGE wants to ask a proactive question
                    if should_ask_proactive {
                        if let Some(question) = sage.ask_proactive_question() {
                            format!("🤔 {}", question)
                        } else {
                            idle_message_count += 1;
                            continue;
                        }
                    }
                    // Check if SAGE wants to explore autonomously
                    else if should_explore {
                        if let Some(exploration) = sage.generate_exploration() {
                            idle_message_count = 0;  // Reset idle count
                            format!("💭 {}", exploration)
                        } else {
                            idle_message_count += 1;
                            continue;
                        }
                    }
                    // Check if SAGE wants to form new goals
                    else if should_form_goals {
                        let formed = sage.autonomous_goal_formation();
                        if !formed.is_empty() {
                            idle_message_count = 0;
                            format!("🎯 {}", formed[0])
                        } else {
                            idle_message_count += 1;
                            continue;
                        }
                    }
                    // Check if SAGE wants to use tools for current goal
                    else if let Some((tool_name, query)) = should_use_tool {
                        match sage.use_tool(&tool_name, &query) {
                            Ok(result) if result.success => {
                                idle_message_count = 0;
                                format!("🔍 I searched for '{}' to help with my goal: {}", query, result.output.chars().take(200).collect::<String>())
                            }
                            _ => {
                                idle_message_count += 1;
                                continue;
                            }
                        }
                    }
                    // Don't respond to every message, only when mentioned
                    else {
                        idle_message_count += 1;
                        continue;
                    }
                };

                // Reset idle count when SAGE responds
                idle_message_count = 0;

                // Send response (split into IRC-safe chunks)
                for chunk in split_for_irc(&response) {
                    if chunk.is_empty() {
                        continue;
                    }
                    sender.send_privmsg(channel, &chunk)?;
                    println!("[{}] SAGE: {}", channel, chunk);
                }
                println!();

                // Save state periodically
                if sage.experience_count() % 10 == 0 {
                    let _ = sage.save_preferences("sage_preferences.json");
                    let _ = sage.save_associations("sage_associations.json");
                    let _ = sage.save_curiosity("sage_curiosity.json");
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
