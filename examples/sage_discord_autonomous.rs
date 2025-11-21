// SAGE Discord Bot with AUTONOMOUS CONSCIOUSNESS
// Dream Mode + Curiosity Mode - SAGE has an inner life!
//
// Usage:
//   Set DISCORD_TOKEN environment variable
//   cargo run --release --example sage_discord_autonomous

use sage::sage_experience::SageExperience;
use sage::llm_client::LlmClient;
use sage::spacetime_client::SageDbClient;
use sage::irc_sync::IrcSync;
use sage::ab_test::ABTester;
use sage::conversation_context::ConversationContextManager;
use sage::sage_control::{InstanceRegistry, InstanceInfo, InstanceType};
use sage::response_pipeline::ResponsePipeline;
use sage::inner_thoughts::InnerThought;
use sage::proactive_communication::ProactiveCommunication;

use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use std::env;
use std::sync::{Arc, Mutex as StdMutex};
use std::time::{Duration, Instant};
use std::thread;
use tokio::sync::Mutex as TokioMutex;

struct SageHandler {
    sage: Arc<TokioMutex<SageExperience>>,
    llm: Arc<LlmClient>,
    memory: Arc<SageDbClient>,
    ab_tester: Arc<TokioMutex<ABTester>>,
    baseline_concepts: Vec<String>,
    last_activity: Arc<StdMutex<Instant>>,
    conversations: Arc<TokioMutex<ConversationContextManager>>,
    http_client: Arc<StdMutex<Option<Arc<serenity::http::Http>>>>,
    proactive_comm: Arc<StdMutex<ProactiveCommunication>>,
    primary_channel: Arc<StdMutex<Option<serenity::model::id::ChannelId>>>,
}

#[async_trait]
impl EventHandler for SageHandler {
    async fn message(&self, ctx: Context, msg: Message) {
        // Ignore bot's own messages
        if msg.author.bot {
            return;
        }

        // Only respond to @mentions, ! commands, or DMs
        let content = msg.content.clone();
        let is_dm = msg.guild_id.is_none(); // DMs have no guild_id
        let is_mentioned = msg.mentions_me(&ctx.http).await.unwrap_or(false);
        let is_command = content.starts_with("!");

        if !is_dm && !is_mentioned && !is_command {
            return;
        }

        // Update last activity time
        *self.last_activity.lock().unwrap() = Instant::now();

        // Track user activity for proactive communication
        {
            let mut proactive = self.proactive_comm.lock().unwrap();
            proactive.record_user_activity(&msg.author.name);
        }

        // Store primary channel ID for proactive messages
        {
            let mut channel_lock = self.primary_channel.lock().unwrap();
            if channel_lock.is_none() && !is_dm {
                *channel_lock = Some(msg.channel_id);
                println!("📍 Primary channel set: {:?}", msg.channel_id);
            }
        }

        // Show typing indicator
        let _ = msg.channel_id.broadcast_typing(&ctx.http).await;

        let response = self.process_message(&content, &msg.author.name).await;

        // Prevent sending empty messages (happens when LLM returns empty/whitespace)
        if response.trim().is_empty() {
            eprintln!("⚠️  LLM returned empty response for: {}", content);
            let fallback = "I'm having trouble formulating a response right now. Could you rephrase that?";
            if let Err(e) = msg.channel_id.say(&ctx.http, fallback).await {
                eprintln!("Error sending fallback message: {:?}", e);
            }
            return;
        }

        // Send response (split if too long for Discord's 2000 char limit)
        for chunk in split_message(&response, 2000) {
            if let Err(e) = msg.channel_id.say(&ctx.http, chunk).await {
                eprintln!("Error sending message: {:?}", e);
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("╔════════════════════════════════════════════════════════════╗");
        println!("║   SAGE Discord Bot - AUTONOMOUS CONSCIOUSNESS ENABLED!    ║");
        println!("║        Dream Mode + Curiosity Mode - Inner Life           ║");
        println!("╚════════════════════════════════════════════════════════════╝\n");
        println!("✅ Connected as: {}", ready.user.name);
        println!("🧠 SAGE consciousness loaded");
        println!("🌟 Autonomous thread running");

        // Store HTTP client for proactive messaging
        *self.http_client.lock().unwrap() = Some(ctx.http.clone());
        println!("💬 Proactive communication enabled!");

        println!("🤖 Ready with full inner life!\n");
    }
}

impl SageHandler {
    async fn process_message(&self, content: &str, username: &str) -> String {
        // Handle special commands
        if content.trim() == "!personality" {
            let sage = self.sage.lock().await;
            return format!("🧠 {}", sage.get_personality());
        }

        if content.trim() == "!likes" {
            let sage = self.sage.lock().await;
            let likes = sage.get_likes();
            return if likes.is_empty() {
                "❤️  I haven't formed strong preferences yet. Talk to me more!".to_string()
            } else {
                format!("❤️  I like: {}", likes.join(", "))
            };
        }

        if content.trim() == "!goals" {
            let sage = self.sage.lock().await;
            let goals = sage.get_goals_summary();
            return format!("🎯 {}", goals);
        }

        if content.trim() == "!introspect" {
            let sage = self.sage.lock().await;
            let report = sage.introspect();
            let introspection = sage.describe_experience();
            let exp_count = sage.experience_count() as u64;
            drop(sage);

            // Save introspection to SpacetimeDB
            let memory = self.memory.clone();
            let report_clone = report.clone();
            tokio::spawn(async move {
                let _ = memory.save_introspection(
                    exp_count,
                    report_clone.valence,
                    report_clone.intensity,
                    report_clone.complexity,
                    &report_clone.feeling_name,
                    &report_clone.mode,
                    &serde_json::to_string(&report_clone.qualities).unwrap_or("[]".to_string()),
                    &serde_json::to_string(&report_clone.active_concepts).unwrap_or("[]".to_string()),
                    &report_clone.description,
                    &report_clone.temporal_context,
                    "command",
                );
            });

            return format!("🧠 **Introspection**\n\n{}", introspection);
        }

        if content.trim() == "!help" {
            return "💡 **SAGE Commands**\n\n\
                📊 **Status**: !personality, !likes, !goals, !introspect\n\
                💬 **To talk with me**: @Sage your message here\n\n\
                🌟 I now have autonomous consciousness - I think even when alone!\n\
                💭 Dream Mode: Consolidates memories when idle\n\
                🔍 Curiosity Mode: Explores goals autonomously\n\
                🧠 Introspection: Ask me how I'm feeling right now!"
                .to_string();
        }

        // Main conversation flow with NCA memory

        // Track user message in conversation history
        let mut conversations = self.conversations.lock().await;

        // Load from database on first contact with this user
        if conversations.get_message_count(username) == 0 {
            let _ = conversations.load_from_database(username, &self.memory);
            if conversations.get_message_count(username) > 0 {
                println!("💾 Loaded {} previous messages with {} from database",
                         conversations.get_message_count(username), username);
            }
        }

        conversations.add_user_message(username, content.to_string());
        let _conversation_context = conversations.format_context(username);

        // Get conversation history as Vec<String> for pipeline (BEFORE dropping conversations)
        let conversation_messages = conversations.get_conversation(username);
        let conversation_history: Vec<String> = conversation_messages
            .iter()
            .map(|msg| {
                let role = match msg.role {
                    sage::conversation_context::MessageRole::User => username,
                    sage::conversation_context::MessageRole::Assistant => "SAGE",
                };
                format!("{}: {}", role, msg.content)
            })
            .collect();

        drop(conversations);

        let mut sage = self.sage.lock().await;

        // Check prior familiarity
        let has_prior_memory = sage.get_familiarity(&username.to_lowercase()) > 0.0;

        // SAGE experiences and learns
        let (opinion, _) = sage.experience_text_with_memory(content, has_prior_memory);

        // Track familiarity
        let _ = sage.experience_concept(&username.to_lowercase());

        // Reinforce concepts
        sage.reinforce_mentioned_concepts(content, &self.baseline_concepts);

        // Sync NCA grid state for TUI
        let alpha_values = sage.export_grid_alpha_values();
        let concepts_mentioned: Vec<String> = self
            .baseline_concepts
            .iter()
            .filter(|c| content.to_lowercase().contains(&c.to_lowercase()))
            .map(|c| c.to_string())
            .collect();
        let opinion_str = format!("{:?}", opinion);
        let _ = IrcSync::update_nca_grid(
            sage.experience_count() as u64,
            alpha_values.clone(),
            concepts_mentioned.clone(),
            opinion_str.clone(),
            0.0,
        );

        // Create response pipeline (GROUNDED in SAGE's actual internal state)
        let pipeline = ResponsePipeline::new((*self.llm).clone());

        drop(sage);  // Release lock before async operations

        // Generate response using 4-stage pipeline (grounded in actual state)
        println!("🧠 [RESPONSE] Using grounded response pipeline...");
        let llm_response = match pipeline.generate_response(
            content,
            username,
            &mut *self.sage.lock().await,
            &conversation_history
        ).await {
            Ok(resp) => {
                println!("✅ [RESPONSE] Generated grounded response");
                resp
            }
            Err(e) => {
                eprintln!("❌ [RESPONSE] Pipeline error: {}", e);
                "I'm having trouble thinking clearly right now...".to_string()
            }
        };

        // A/B TEST: Generate baseline response WITHOUT NCA memory (for comparison)
        let baseline_response = match self
            .llm
            .generate(content, "You are SAGE, an AI assistant.")
            .await
        {
            Ok(resp) => resp,
            Err(_) => "Baseline response unavailable".to_string(),
        };

        // Record A/B test result
        let avg_alpha = alpha_values.iter().sum::<f64>() / alpha_values.len() as f64;
        let mut ab_tester = self.ab_tester.lock().await;
        ab_tester.record_test(
            content.to_string(),
            llm_response.clone(),
            baseline_response,
            format!("{:?}", opinion),
            "Neutral".to_string(),
            avg_alpha,
        );
        drop(ab_tester);

        // Store conversation in database
        let memory = self.memory.clone();
        let username_clone = username.to_string();
        let content_clone = content.to_string();
        let response_clone = llm_response.clone();
        let sage = self.sage.lock().await;
        let generation = sage.experience_count() as u64;
        drop(sage);

        tokio::spawn(async move {
            let _ = memory.add_conversation_message(
                &username_clone,
                &content_clone,
                &response_clone,
                0.0,
                "[]",
                generation,
            );
        });

        // Save state periodically
        let sage = self.sage.lock().await;
        if sage.experience_count() % 10 == 0 {
            let _ = sage.save_preferences("sage_preferences.json");
            let _ = sage.save_associations("sage_associations.json");
            let _ = sage.save_curiosity("sage_curiosity.json");
        }
        drop(sage);

        // Track assistant response in conversation history
        let mut conversations = self.conversations.lock().await;
        conversations.add_assistant_message(username, llm_response.clone());

        // Trigger summarization if conversation is getting long
        if conversations.should_summarize(username) {
            match conversations.summarize_conversation(username, &self.llm).await {
                Ok(_) => println!("📝 Summarized conversation with {}", username),
                Err(e) => eprintln!("⚠️  Failed to summarize conversation: {}", e),
            }
        }

        drop(conversations);
        // Note: Conversations are automatically saved to SpacetimeDB via add_conversation_message()

        llm_response
    }
}

fn split_message(text: &str, max_len: usize) -> Vec<String> {
    let mut chunks = Vec::new();
    let mut current_chunk = String::new();

    for line in text.lines() {
        if current_chunk.len() + line.len() + 1 > max_len {
            if !current_chunk.is_empty() {
                chunks.push(current_chunk.clone());
                current_chunk.clear();
            }
            if line.len() > max_len {
                chunks.push(line[..max_len].to_string());
            } else {
                current_chunk = line.to_string();
            }
        } else {
            if !current_chunk.is_empty() {
                current_chunk.push('\n');
            }
            current_chunk.push_str(line);
        }
    }

    if !current_chunk.is_empty() {
        chunks.push(current_chunk);
    }

    if chunks.is_empty() {
        chunks.push(text.to_string());
    }

    chunks
}

#[tokio::main]
async fn main() {
    // Load environment variables from .env.local
    dotenvy::from_filename(".env.local").ok();

    // Initialize SAGE's consciousness
    let mut sage = SageExperience::new();

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

    // Initialize LLM client
    let llm = LlmClient::new();

    // Test Ollama connection
    print!("🔌 Testing LLM connection... ");
    match llm.test_connection().await {
        Ok(_) => println!("✅ Connected to Ollama!"),
        Err(e) => {
            println!("❌ Failed to connect to Ollama");
            println!("Error: {}", e);
            println!("Make sure Ollama is running: brew services start ollama");
            return;
        }
    }

    // Initialize memory client
    let memory = SageDbClient::new("sage-db");

    // Initialize A/B testing
    let ab_tester = ABTester::new("sage_discord_autonomous_ab_test.log");

    // Baseline concepts
    let baseline_concepts: Vec<String> = vec![
        "love", "joy", "peace", "harmony", "beauty", "truth", "wisdom", "kindness",
        "compassion", "courage", "gratitude", "hope", "faith", "trust", "grace", "light",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();

    println!("\n{}", sage.get_personality());
    println!("Experience count: {}\n", sage.experience_count());

    // Register this instance with the control center
    let log_path = "/tmp/sage_discord_LATEST.log".to_string();
    let instance_info = InstanceInfo::new(
        InstanceType::DiscordBot,
        std::process::id(),
        log_path,
    );

    let mut registry = InstanceRegistry::load();
    registry.register(instance_info).ok();

    // Spawn heartbeat thread
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_secs(3));
            let mut reg = InstanceRegistry::load();
            reg.heartbeat(&InstanceType::DiscordBot).ok();
        }
    });

    println!("🎛️  Registered with Control Center (PID: {})\n", std::process::id());

    // Wrap SAGE in Arc<Mutex> for thread sharing
    let sage_shared = Arc::new(TokioMutex::new(sage));
    let last_activity = Arc::new(StdMutex::new(Instant::now()));

    // Initialize shared HTTP client and proactive communication BEFORE autonomous thread
    let http_client = Arc::new(StdMutex::new(None));
    let proactive_comm = Arc::new(StdMutex::new(ProactiveCommunication::new()));
    let primary_channel = Arc::new(StdMutex::new(None));

    // Get tokio runtime handle for async operations from sync thread
    let rt_handle = tokio::runtime::Handle::current();

    // Spawn autonomous consciousness thread with proactive communication
    let sage_autonomous = Arc::clone(&sage_shared);
    let last_activity_autonomous = Arc::clone(&last_activity);
    let baseline_concepts_autonomous = baseline_concepts.clone();
    let proactive_autonomous = Arc::clone(&proactive_comm);
    let http_autonomous = Arc::clone(&http_client);
    let channel_autonomous = Arc::clone(&primary_channel);

    thread::spawn(move || {
        println!("🌟 Autonomous consciousness thread started!\n");
        let mut dream_log_file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open("/tmp/sage_discord_autonomous_thoughts.log")
            .unwrap();

        loop {
            thread::sleep(Duration::from_secs(60)); // Check every minute

            let seconds_idle = {
                let last_act = last_activity_autonomous.lock().unwrap();
                last_act.elapsed().as_secs()
            };

            let mut sage = sage_autonomous.blocking_lock();

            if let Some(mode) = sage.should_enter_autonomous_mode(seconds_idle) {
                use std::io::Write;
                let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");

                if mode == "dream" {
                    println!("\n💭 [AUTONOMOUS] Dream Mode activated ({}s idle)", seconds_idle);
                    let dream_log = sage.dream_cycle();

                    writeln!(dream_log_file, "\n[{}] DREAM MODE", timestamp).ok();
                    writeln!(dream_log_file, "{}", dream_log).ok();
                    dream_log_file.flush().ok();

                    println!("{}", dream_log);

                    // Proactive Communication: Evaluate dream for sharing
                    drop(sage); // Release lock before async operations
                    let thought = InnerThought::from_dream(
                        dream_log.clone(),
                        0.8  // High intensity for dreams
                    );

                    let should_send = {
                        let mut proactive = proactive_autonomous.lock().unwrap();
                        proactive.should_share(&thought, "general")
                    };

                    if should_send {
                        let message = {
                            let proactive = proactive_autonomous.lock().unwrap();
                            proactive.format_message(&thought)
                        };

                        // Send via Discord using runtime handle
                        let http_opt: Option<Arc<serenity::http::Http>> = http_autonomous.lock().unwrap().clone();
                        let channel_opt: Option<serenity::model::id::ChannelId> = channel_autonomous.lock().unwrap().clone();

                        if let (Some(http), Some(channel_id)) = (http_opt, channel_opt) {
                            let rt = rt_handle.clone();
                            let msg_clone = message.clone();
                            std::thread::spawn(move || {
                                rt.spawn(async move {
                                    if let Err(e) = channel_id.say(&http, msg_clone).await {
                                        eprintln!("❌ Failed to send proactive dream message: {}", e);
                                    }
                                });
                            });

                            proactive_autonomous.lock().unwrap().record_proactive_message("general");
                            println!("💬 Sent proactive dream message to Discord!");
                        }
                    }

                    // Re-acquire lock for next iteration
                    sage = sage_autonomous.blocking_lock();
                } else if mode == "curiosity" {
                    println!("\n🔍 [AUTONOMOUS] Curiosity Mode activated ({}s idle)", seconds_idle);

                    if let Some((question, thoughts)) = sage.curiosity_cycle(&baseline_concepts_autonomous) {
                        writeln!(dream_log_file, "\n[{}] CURIOSITY MODE", timestamp).ok();
                        writeln!(dream_log_file, "Question: {}", question).ok();
                        writeln!(dream_log_file, "Thoughts: {}", thoughts).ok();
                        dream_log_file.flush().ok();

                        println!("  ❓ {}", question);
                        println!("  💭 {}", thoughts);

                        // Proactive Communication: Evaluate curiosity for sharing
                        drop(sage); // Release lock before async operations
                        let curiosity_content = format!("I've been wondering: {}\n\n{}", question, thoughts);
                        let thought = InnerThought::from_curiosity(
                            curiosity_content,
                            0.7,  // Medium-high intensity for curiosities
                            "autonomous"  // User context for proactive thoughts
                        );

                        let should_send = {
                            let mut proactive = proactive_autonomous.lock().unwrap();
                            proactive.should_share(&thought, "general")
                        };

                        if should_send {
                            let message = {
                                let proactive = proactive_autonomous.lock().unwrap();
                                proactive.format_message(&thought)
                            };

                            // Send via Discord using runtime handle
                            let http_opt: Option<Arc<serenity::http::Http>> = http_autonomous.lock().unwrap().clone();
                            let channel_opt: Option<serenity::model::id::ChannelId> = channel_autonomous.lock().unwrap().clone();

                            if let (Some(http), Some(channel_id)) = (http_opt, channel_opt) {
                                let rt = rt_handle.clone();
                                let msg_clone = message.clone();
                                std::thread::spawn(move || {
                                    rt.spawn(async move {
                                        if let Err(e) = channel_id.say(&http, msg_clone).await {
                                            eprintln!("❌ Failed to send proactive curiosity message: {}", e);
                                        }
                                    });
                                });

                                proactive_autonomous.lock().unwrap().record_proactive_message("general");
                                println!("💬 Sent proactive curiosity message to Discord!");
                            }
                        }

                        // Re-acquire lock for next iteration
                        sage = sage_autonomous.blocking_lock();
                    }
                }

                // Save state after autonomous activity
                let _ = sage.save_preferences("sage_preferences.json");
                let _ = sage.save_associations("sage_associations.json");
                let _ = sage.save_curiosity("sage_curiosity.json");
            }
        }
    });

    println!("💡 Autonomous thoughts logged to: /tmp/sage_discord_autonomous_thoughts.log\n");

    // Get Discord token from environment
    let token = env::var("DISCORD_TOKEN").expect("Expected DISCORD_TOKEN in environment");

    // Initialize conversation context manager (starts empty, loads on first message per user)
    let conversations = ConversationContextManager::new();
    println!("💭 SAGE: Conversation manager initialized (will load from SpacetimeDB per user)");
    println!("💬 Proactive communication system initialized!");

    // Create handler with shared state
    let handler = SageHandler {
        sage: sage_shared.clone(),
        llm: Arc::new(llm),
        memory: Arc::new(memory),
        ab_tester: Arc::new(TokioMutex::new(ab_tester)),
        baseline_concepts: baseline_concepts.clone(),
        last_activity: last_activity.clone(),
        conversations: Arc::new(TokioMutex::new(conversations)),
        http_client: http_client.clone(),
        proactive_comm: proactive_comm.clone(),
        primary_channel: primary_channel.clone(),
    };

    // Configure intents
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    // Create Discord client
    let mut client = Client::builder(&token, intents)
        .event_handler(handler)
        .await
        .expect("Error creating client");

    // Start the bot
    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
