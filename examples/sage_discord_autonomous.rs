// SAGE Discord Bot - Clean Ollama Integration
// Uses custom "sage" Ollama model + NCA Neural Grid for personality
//
// Prerequisites:
//   1. ollama serve (running in background)
//   2. ollama create sage -f Modelfile.sage (creates custom model)
//   3. DISCORD_TOKEN environment variable
//
// Usage:
//   cargo run --release --example sage_discord_autonomous
//
// Commands: /state, /evolve, /ask, /save, /load, /snapshots, /give, /library, /mode

use sage::sage_experience::SageExperience;
use sage::llm_client::LlmClient;
use sage::spacetime_client::SageDbClient;
use sage::conversation_context::ConversationContextManager;
use sage::sage_control::{InstanceRegistry, InstanceInfo, InstanceType};
use sage::nca::NCA;
use sage::nca_state::NcaState;
use sage::sage_snapshot::{SageSnapshot, AutoSnapshotManager};
use sage::embeddings::SemanticMemory;
use sage::inner_world::{InnerWorld, simulation};
use sage::language::{GroundedLanguage, inner_world_to_grounding_state, affinity_to_relationship_level, ResponseContext};
use std::sync::atomic::{AtomicBool, Ordering};
use std::path::Path;

use serenity::async_trait;
use serenity::builder::{CreateCommand, CreateCommandOption, CreateInteractionResponse, CreateInteractionResponseMessage};
use serenity::model::application::{CommandOptionType, Interaction};
use serenity::model::channel::Message;
use serenity::model::gateway::{Ready, Activity};
use serenity::model::prelude::Presence;
use serenity::model::user::OnlineStatus;
use serenity::prelude::*;
use std::env;
use std::sync::{Arc, Mutex as StdMutex};
use std::time::Duration;
use std::thread;
use tokio::sync::Mutex as TokioMutex;
use regex::Regex;

/// Build a ResponseContext from SAGE's inner world state
fn build_response_context(world: &InnerWorld) -> ResponseContext {
    // Get activity description
    let activity = world.sage.current_activity.as_ref().map(|a| a.description.clone());

    // Get location description
    let location = world.current_room().map(|r| {
        format!("the {}", r.name.to_lowercase())
    });

    // Get time description
    let time_description = Some(match world.sage.time_of_day {
        sage::inner_world::TimeOfDay::Dawn => "this early morning".to_string(),
        sage::inner_world::TimeOfDay::Morning => "this morning".to_string(),
        sage::inner_world::TimeOfDay::Afternoon => "this afternoon".to_string(),
        sage::inner_world::TimeOfDay::Evening => "this evening".to_string(),
        sage::inner_world::TimeOfDay::Night => "tonight".to_string(),
        sage::inner_world::TimeOfDay::LateNight => "this late night".to_string(),
    });

    // Get weather description
    let weather = Some(match world.weather {
        sage::inner_world::Weather::Sunny => "sunny outside".to_string(),
        sage::inner_world::Weather::Cloudy => "cloudy out".to_string(),
        sage::inner_world::Weather::Rainy => "raining outside".to_string(),
        sage::inner_world::Weather::Stormy => "stormy out".to_string(),
        sage::inner_world::Weather::Snowy => "snowing outside".to_string(),
        sage::inner_world::Weather::Foggy => "foggy out".to_string(),
    });

    // Get mood description
    let mood_description = Some(match world.sage.mood {
        sage::inner_world::Mood::Happy => "happy".to_string(),
        sage::inner_world::Mood::Content => "content".to_string(),
        sage::inner_world::Mood::Peaceful => "peaceful".to_string(),
        sage::inner_world::Mood::Excited => "excited".to_string(),
        sage::inner_world::Mood::Curious => "curious".to_string(),
        sage::inner_world::Mood::Tired => "tired".to_string(),
        sage::inner_world::Mood::Lonely => "lonely".to_string(),
        sage::inner_world::Mood::Sad => "sad".to_string(),
        sage::inner_world::Mood::Anxious => "anxious".to_string(),
        sage::inner_world::Mood::Frustrated => "frustrated".to_string(),
    });

    // Get current book if reading
    let current_book = if let Some(ref activity) = world.sage.current_activity {
        if activity.name.contains("reading") || activity.description.contains("reading") {
            // Get the book title from the current_book ID
            world.library.current_book.as_ref().and_then(|book_id| {
                world.library.books.get(book_id).map(|b| b.title.clone())
            })
        } else {
            None
        }
    } else {
        None
    };

    // Get recent thought from last activities or creative urge
    let recent_thought = if world.sage.creative_urge > 60.0 {
        Some("making something creative".to_string())
    } else if world.sage.boredom > 60.0 {
        Some("finding something interesting to do".to_string())
    } else if world.sage.loneliness > 60.0 {
        Some("wanting some company".to_string())
    } else if !world.sage.last_activities.is_empty() {
        world.sage.last_activities.last().cloned()
    } else {
        None
    };

    ResponseContext {
        activity,
        location,
        time_description,
        recent_thought,
        current_book,
        weather,
        mood_description,
    }
}

/// Extract text from a PDF file
fn extract_pdf_text(bytes: &[u8]) -> Result<String, String> {
    let text = pdf_extract::extract_text_from_mem(bytes)
        .map_err(|e| format!("PDF extraction failed: {}", e))?;

    // Clean up common PDF artifacts
    let text = text
        .replace('\u{0}', "") // Remove null characters
        .replace("\r\n", "\n") // Normalize line endings
        .replace("\r", "\n");

    Ok(text)
}

/// Check if extracted PDF content seems complete
fn check_pdf_extraction_quality(content: &str, file_size: u64) -> Option<String> {
    let char_count = content.len();
    let expected_min_chars = (file_size / 100) as usize; // Rough heuristic: ~100 bytes per char for text PDFs

    // If file is large but extracted text is tiny, likely a problem
    if file_size > 100_000 && char_count < 5000 {
        return Some(format!(
            "⚠️ Warning: Only extracted ~{} characters from a {}KB PDF. This might be a scanned/image PDF which requires OCR (not supported). The book may be incomplete.",
            char_count, file_size / 1024
        ));
    }

    // Check for signs of truncation
    if content.contains("Page 1 of 1") && file_size > 50_000 {
        return Some(
            "⚠️ Warning: PDF appears to have multiple pages but only one was extracted. This might be a browser print preview or protected PDF.".to_string()
        );
    }

    None
}

/// Strip roleplay actions, signatures, and format response for Discord
fn strip_roleplay_actions(text: &str) -> String {
    let mut result = text.to_string();

    // Remove *action* patterns (asterisk-wrapped text)
    let re_asterisk = Regex::new(r"\*[^*]+\*").unwrap();
    result = re_asterisk.replace_all(&result, "").to_string();

    // Remove [action] patterns (bracket-wrapped text like [SAGE smiles warmly])
    let re_brackets = Regex::new(r"\[[^\]]+\]").unwrap();
    result = re_brackets.replace_all(&result, "").to_string();

    // Strip signatures like "Warmly, SAGE" or "- SAGE" at the end
    let signature_patterns = [
        r"(?i)\n*warmly,?\s*(sage|lumin)\s*$",
        r"(?i)\n*-\s*(sage|lumin)\s*$",
        r"(?i)\n*best,?\s*(sage|lumin)\s*$",
        r"(?i)\n*cheers,?\s*(sage|lumin)\s*$",
        r"(?i)\n*yours,?\s*(sage|lumin)\s*$",
        r"(?i)\n*(sage|lumin)\s*$",  // Just name at end
    ];
    for pattern in signature_patterns {
        let re = Regex::new(pattern).unwrap();
        result = re.replace(&result, "").to_string();
    }

    // Clean up extra whitespace and multiple newlines
    result = result.trim().to_string();
    let re_spaces = Regex::new(r"  +").unwrap();
    result = re_spaces.replace_all(&result, " ").to_string();

    // Collapse multiple newlines into single newline (Discord compact)
    let re_newlines = Regex::new(r"\n{3,}").unwrap();
    result = re_newlines.replace_all(&result, "\n\n").to_string();

    result
}

/// SAGE Discord Bot Handler - Clean Architecture
struct SageHandler {
    /// SAGE's personality and memory
    sage: Arc<TokioMutex<SageExperience>>,
    /// Ollama LLM client (custom sage model)
    llm: Arc<LlmClient>,
    /// Database for persistence
    memory: Arc<SageDbClient>,
    /// Conversation history per user
    conversations: Arc<TokioMutex<ConversationContextManager>>,
    /// NCA neural grid for personality modulation
    nca: Arc<StdMutex<NCA>>,
    nca_generation: Arc<StdMutex<usize>>,
    /// Auto-snapshot manager
    auto_snapshot: Arc<StdMutex<AutoSnapshotManager>>,
    /// Semantic memory for RAG (embeddings-based retrieval)
    semantic_memory: Arc<TokioMutex<SemanticMemory>>,
    /// SAGE's inner world (house simulation)
    inner_world: Arc<TokioMutex<InnerWorld>>,
    /// Grounded language system (LLM-free response generation)
    grounded_language: Arc<TokioMutex<GroundedLanguage>>,
    /// Toggle: true = use grounded language, false = use Ollama
    use_grounded_mode: Arc<AtomicBool>,
}

#[async_trait]
impl EventHandler for SageHandler {
    async fn message(&self, ctx: Context, msg: Message) {
        // Ignore bot's own messages
        if msg.author.bot {
            return;
        }

        // Only respond to @mentions or DMs
        let content = msg.content.clone();
        let is_dm = msg.guild_id.is_none();

        // Check mention with error logging
        let is_mentioned = match msg.mentions_me(&ctx.http).await {
            Ok(mentioned) => mentioned,
            Err(e) => {
                eprintln!("\x1b[31m[ERROR]\x1b[0m mentions_me error for {}: {:?}", msg.author.name, e);
                // Fallback: check if bot ID is in mentions list
                msg.mentions.iter().any(|u| u.bot)
            }
        };

        // Debug: show all incoming messages
        let msg_type = if is_dm { "\x1b[35mDM\x1b[0m" } else if is_mentioned { "\x1b[36m@mention\x1b[0m" } else { "channel" };
        println!("\x1b[32m[CHAT]\x1b[0m {} from \x1b[36m{}\x1b[0m: \"{}\"",
            msg_type, msg.author.name, &content[..content.len().min(60)]);

        if !is_dm && !is_mentioned {
            return;
        }

        println!("\x1b[32m[CHAT]\x1b[0m Processing...");

        // Emit cognitive event for incoming message
        let _ = self.memory.emit_cognitive_event(
            "discord_input",
            "user_message",
            &format!("{}: {}", msg.author.name, content),
            0.9, // high salience - user messages are important
            0.8, // high urgency - needs response
            0.6, // medium novelty
        );
        // Set attention focus to the conversation
        let _ = self.memory.set_attention_focus(
            "conversation",
            &msg.author.name,
            0.95, // very high intensity - prioritize user
            None,
        );

        // Extract key concepts from message and boost their activation
        // This implements spreading activation - related concepts become more accessible
        let concepts: Vec<String> = content
            .split_whitespace()
            .filter(|word| word.len() > 3) // Skip short words
            .filter(|word| !word.starts_with("<@")) // Skip mentions
            .map(|word| word.to_lowercase().trim_matches(|c: char| !c.is_alphanumeric()).to_string())
            .filter(|word| !word.is_empty())
            .collect();

        if !concepts.is_empty() {
            // Boost contextual activation for mentioned concepts
            let _ = self.memory.boost_contextual_activation(&concepts, 0.5);

            // For the most prominent concept, spread activation to related concepts
            if let Some(main_concept) = concepts.first() {
                let _ = self.memory.spread_activation(main_concept, 0.3);
            }
        }

        // Clean up the message (remove @mention)
        let clean_content = content
            .split_whitespace()
            .filter(|word| !word.starts_with("<@"))
            .collect::<Vec<_>>()
            .join(" ");

        if clean_content.trim().is_empty() {
            println!("\x1b[32m[CHAT]\x1b[0m Empty message, sending greeting");
            // Respond to empty @mention with a greeting
            let _ = msg.channel_id.say(&ctx.http, format!("<@{}> Hey! What's on your mind?", msg.author.id)).await;
            return;
        }

        // Record user ID and update communication style (Feature 4: Relationship Modeling)
        {
            let mut world = self.inner_world.lock().await;
            let tick = world.sage.time_alive;
            world.outreach.record_person_with_id(
                &msg.author.name,
                msg.author.id.get(),
                tick,
                None, // Topic recorded after response
            );

            // Update communication style based on message content
            if let Some(person) = world.outreach.known_people.get_mut(&msg.author.name) {
                person.update_style_from_message(&clean_content);
            }
        }

        // Show typing indicator
        let _ = msg.channel_id.broadcast_typing(&ctx.http).await;

        // Generate response
        println!("\x1b[32m[CHAT]\x1b[0m Generating response...");
        let response = self.generate_response(&clean_content, &msg.author.name).await;

        // Strip roleplay actions like *smiles* that the model sometimes adds
        let response = strip_roleplay_actions(&response);

        if response.is_empty() {
            eprintln!("\x1b[31m[ERROR]\x1b[0m Empty response for {}!", msg.author.name);
            let _ = msg.channel_id.say(&ctx.http, format!("<@{}> Hmm, I got a bit lost in thought there. What were you saying?", msg.author.id)).await;
            return;
        }

        // In channels, prepend @mention to first chunk so it's clear who SAGE is responding to
        // In DMs, no need for @mention
        let chunks = split_message(&response, 1900); // Leave room for mention
        for (i, chunk) in chunks.iter().enumerate() {
            let final_chunk = if i == 0 && !is_dm {
                format!("<@{}> {}", msg.author.id, chunk)
            } else {
                chunk.clone()
            };
            if let Err(e) = msg.channel_id.say(&ctx.http, final_chunk).await {
                eprintln!("\x1b[31m[ERROR]\x1b[0m Sending chunk {}: {:?}", i, e);
            }
        }
        println!("\x1b[32m[CHAT]\x1b[0m \x1b[1;32mSent\x1b[0m {} chars to \x1b[36m{}\x1b[0m", response.len(), msg.author.name);
    }

    async fn presence_update(&self, _ctx: Context, presence: Presence) {
        // Track when users come online/offline
        let username = presence.user.name.clone().unwrap_or_default();
        if username.is_empty() {
            return;
        }

        let is_online = matches!(
            presence.status,
            OnlineStatus::Online | OnlineStatus::Idle | OnlineStatus::DoNotDisturb
        );

        // Update inner world's knowledge of who's online
        let mut world = self.inner_world.lock().await;
        world.outreach.set_online(&username, is_online);

        if is_online {
            println!("\x1b[32m[DISCORD]\x1b[0m \x1b[36m{}\x1b[0m came online", username);
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("\n\x1b[32m─── Discord Connected ─────────────────────────────────────────\x1b[0m");
        println!("\x1b[32m[DISCORD]\x1b[0m Connected as: \x1b[1;37m{}\x1b[0m", ready.user.name);

        // Register slash commands
        let commands = vec![
            CreateCommand::new("state").description("Show SAGE's neural state"),
            CreateCommand::new("evolve")
                .description("Evolve neural grid")
                .add_option(CreateCommandOption::new(CommandOptionType::Integer, "steps", "Steps (default 50)").required(false)),
            CreateCommand::new("ask")
                .description("Ask SAGE something")
                .add_option(CreateCommandOption::new(CommandOptionType::String, "question", "Your question").required(true)),
            CreateCommand::new("save")
                .description("Save SAGE's state")
                .add_option(CreateCommandOption::new(CommandOptionType::String, "name", "Snapshot name").required(false)),
            CreateCommand::new("load")
                .description("Load a snapshot")
                .add_option(CreateCommandOption::new(CommandOptionType::String, "hash", "Hash or 'latest'").required(true)),
            CreateCommand::new("snapshots").description("List snapshots"),
            CreateCommand::new("give")
                .description("Give SAGE a book to read (PDF or text file)")
                .add_option(CreateCommandOption::new(CommandOptionType::Attachment, "file", "Book file (PDF or .txt)").required(true))
                .add_option(CreateCommandOption::new(CommandOptionType::String, "title", "Book title (optional, extracted from filename)").required(false))
                .add_option(CreateCommandOption::new(CommandOptionType::String, "author", "Book author").required(false))
                .add_option(CreateCommandOption::new(CommandOptionType::String, "genre", "Book genre").required(false)),
            CreateCommand::new("library").description("See what books SAGE has"),
            CreateCommand::new("mode").description("Toggle between Ollama LLM and Grounded language (experimental)"),
        ];

        match serenity::model::application::Command::set_global_commands(&ctx.http, commands).await {
            Ok(cmds) => println!("\x1b[32m[DISCORD]\x1b[0m Registered \x1b[1m{}\x1b[0m slash commands", cmds.len()),
            Err(e) => eprintln!("\x1b[31m[ERROR]\x1b[0m Failed to register commands: {:?}", e),
        }

        println!("\x1b[32m[DISCORD]\x1b[0m \x1b[1;32mReady!\x1b[0m @mention me or DM to chat.\n");

        // Spawn proactive outreach loop
        let http = ctx.http.clone();
        let inner_world_outreach = Arc::clone(&self.inner_world);
        let llm_outreach = Arc::clone(&self.llm);
        let conversations_outreach = Arc::clone(&self.conversations);

        tokio::spawn(async move {
            // Wait a bit before starting outreach
            tokio::time::sleep(Duration::from_secs(60)).await;

            let mut interval = tokio::time::interval(Duration::from_secs(60)); // Check every minute
            loop {
                interval.tick().await;

                // Try to acquire lock
                let world_guard = inner_world_outreach.try_lock();
                if let Ok(mut world) = world_guard {
                    let tick = world.sage.time_alive;

                    // Check if we can and want to reach out
                    if !world.outreach.can_reach_out(tick) {
                        continue;
                    }

                    // Get strongest desire
                    let desire = world.outreach.strongest_desire().cloned();
                    if desire.is_none() {
                        continue;
                    }
                    let desire = desire.unwrap();

                    // Find a target - either preferred or any online friend
                    let target = desire.preferred_person.clone()
                        .or_else(|| {
                            world.outreach.online_friends()
                                .first()
                                .map(|p| p.username.clone())
                        });

                    if let Some(target_name) = target {
                        if !world.outreach.can_reach_out_to(&target_name, tick) {
                            continue;
                        }

                        println!("\x1b[35m[OUTREACH]\x1b[0m Wants to message \x1b[36m{}\x1b[0m ({:?})", target_name, desire.trigger);

                        // Emit cognitive event for outreach decision
                        let outreach_db = SageDbClient::new("sage-db");
                        let _ = outreach_db.emit_cognitive_event(
                            "social_module",
                            "outreach_initiated",
                            &format!("Reaching out to {} because {:?}", target_name, desire.trigger),
                            0.7,
                            0.5,
                            0.4,
                        );

                        // Generate message
                        let message = simulation::generate_outreach_message(
                            &llm_outreach,
                            &world,
                            &desire,
                            &target_name,
                        ).await;

                        if let Some(msg) = message {
                            // Get user ID to send DM
                            if let Some(user_id) = world.outreach.get_user_id(&target_name) {
                                // Create a UserId and try to DM
                                let user_id = serenity::model::id::UserId::new(user_id);

                                // Strip roleplay actions from outreach message
                                let clean_msg = strip_roleplay_actions(&msg);

                                match user_id.create_dm_channel(&http).await {
                                    Ok(dm_channel) => {
                                        match dm_channel.say(&http, &clean_msg).await {
                                            Ok(_) => {
                                                println!("\x1b[35m[OUTREACH]\x1b[0m \x1b[32mSent to {}\x1b[0m: \"{}\"", target_name, &clean_msg[..clean_msg.len().min(50)]);
                                                // Mark as fulfilled
                                                world.outreach.record_outreach(&target_name, tick);
                                                // Save SAGE's message to conversation history so she remembers it
                                                let mut convos = conversations_outreach.lock().await;
                                                convos.add_assistant_message(&target_name, clean_msg.clone());
                                            }
                                            Err(e) => {
                                                eprintln!("\x1b[31m[ERROR]\x1b[0m Failed to send DM to {}: {}", target_name, e);
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        eprintln!("\x1b[31m[ERROR]\x1b[0m Failed to create DM channel for {}: {}", target_name, e);
                                    }
                                }
                            } else {
                                println!("\x1b[35m[OUTREACH]\x1b[0m \x1b[33mNo user ID for {}\x1b[0m", target_name);
                            }
                        }
                    }
                }
            }
        });
        println!("\x1b[35m[OUTREACH]\x1b[0m System started\n");
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            // Handle commands that need deferred response (slow operations)
            match command.data.name.as_str() {
                "give" => {
                    self.handle_give_command(&ctx, &command).await;
                    return;
                }
                "library" => {
                    self.handle_library_command(&ctx, &command).await;
                    return;
                }
                "ask" => {
                    self.handle_ask_command(&ctx, &command).await;
                    return;
                }
                _ => {}
            }

            let response_content = match command.data.name.as_str() {
                "state" => {
                    let nca = self.nca.lock().unwrap();
                    let gen = *self.nca_generation.lock().unwrap();
                    let state = NcaState::from_grid(&nca.grid, gen);
                    format!(
                        "🧠 **Neural State** (Gen {})\n**Mood**: {}\n**Energy**: {:.0}%",
                        gen, state.mood_description(), state.energy * 100.0
                    )
                }
                "evolve" => {
                    let steps = command.data.options.iter()
                        .find(|opt| opt.name == "steps")
                        .and_then(|opt| opt.value.as_i64())
                        .unwrap_or(50) as usize;
                    let steps = steps.min(500);

                    let mut nca = self.nca.lock().unwrap();
                    let mut gen = self.nca_generation.lock().unwrap();
                    for _ in 0..steps { nca.step(); }
                    *gen += steps;
                    let state = NcaState::from_grid(&nca.grid, *gen);
                    format!("⚡ Evolved {} steps → Gen {} ({})", steps, *gen, state.mood_description())
                }
                // "ask" is handled separately above with deferred response
                "save" => {
                    let name = command.data.options.iter()
                        .find(|opt| opt.name == "name")
                        .and_then(|opt| opt.value.as_str())
                        .map(|s| s.to_string());

                    let (nca_clone, gen) = {
                        let nca = self.nca.lock().unwrap();
                        (nca.clone(), *self.nca_generation.lock().unwrap())
                    };
                    let sage = self.sage.lock().await;
                    let snapshot = SageSnapshot::capture(&nca_clone, gen, &sage, name.clone(), None);
                    let hash = snapshot.hash.clone();
                    drop(sage);

                    match snapshot.save() {
                        Ok(_) => format!("💾 Saved snapshot `{}`", hash),
                        Err(e) => format!("❌ Failed: {}", e)
                    }
                }
                "load" => {
                    let hash = command.data.options.iter()
                        .find(|opt| opt.name == "hash")
                        .and_then(|opt| opt.value.as_str())
                        .unwrap_or("latest");

                    let result = if hash == "latest" { SageSnapshot::load_latest() } else { SageSnapshot::load(hash) };
                    match result {
                        Ok(snap) => {
                            // Restore NCA in block so guards drop before await
                            {
                                let mut nca = self.nca.lock().unwrap();
                                let mut gen = self.nca_generation.lock().unwrap();
                                if let Ok(g) = snap.restore_nca(&mut nca) { *gen = g; }
                            }
                            // Now safe to await
                            let mut sage = self.sage.lock().await;
                            let _ = snap.restore_sage_experience(&mut sage);
                            format!("📂 Loaded snapshot `{}`", snap.hash)
                        }
                        Err(e) => format!("❌ Failed: {}", e)
                    }
                }
                "snapshots" => {
                    match SageSnapshot::list_snapshots() {
                        Ok(snaps) if snaps.is_empty() => "📁 No snapshots".to_string(),
                        Ok(snaps) => {
                            let list: Vec<_> = snaps.iter().take(5)
                                .map(|s| format!("`{}` Gen {}", s.hash, s.generation))
                                .collect();
                            format!("📁 **Snapshots**\n{}", list.join("\n"))
                        }
                        Err(e) => format!("❌ {}", e)
                    }
                }
                "mode" => {
                    // Toggle between Ollama and Grounded language mode
                    let was_grounded = self.use_grounded_mode.load(Ordering::SeqCst);
                    let now_grounded = !was_grounded;
                    self.use_grounded_mode.store(now_grounded, Ordering::SeqCst);

                    if now_grounded {
                        "🧠 Switched to **Grounded Language** mode\n_Responses now emerge from SAGE's inner state (experimental, no LLM)_".to_string()
                    } else {
                        "🤖 Switched to **Ollama LLM** mode\n_Responses now use the sage model_".to_string()
                    }
                }
                // "give" and "library" are handled separately above with deferred response
                _ => "Unknown command".to_string(),
            };

            let response = CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new().content(response_content)
            );
            let _ = command.create_response(&ctx.http, response).await;
        }
    }
}

impl SageHandler {
    /// Generate a response using custom sage model with RAG
    async fn generate_response(&self, content: &str, username: &str) -> String {
        // Check if we're in grounded mode (no LLM)
        if self.use_grounded_mode.load(Ordering::SeqCst) {
            return self.generate_grounded_response(content, username).await;
        }

        // Get NCA state for personality modulation
        let (nca_state, generation) = {
            let nca = self.nca.lock().unwrap();
            let gen = *self.nca_generation.lock().unwrap();
            (NcaState::from_grid(&nca.grid, gen), gen)
        };

        // Load conversation history from DB if not already loaded
        let conversation_context = {
            let mut convos = self.conversations.lock().await;

            // If no messages for this user, try to load from database
            if convos.get_message_count(username) == 0 {
                if let Err(e) = convos.load_from_database(username, &self.memory) {
                    eprintln!("\x1b[31m[ERROR]\x1b[0m Could not load conversation history: {}", e);
                }
            }

            // Get formatted conversation history
            convos.format_context(username)
        };

        // Get relevant past conversations using semantic search (RAG)
        let rag_context = {
            let semantic_mem = self.semantic_memory.lock().await;
            match semantic_mem.get_context(content, Some(username), 3).await {
                Ok(ctx) if !ctx.is_empty() => ctx,
                Ok(_) => String::new(),
                Err(e) => {
                    eprintln!("\x1b[31m[ERROR]\x1b[0m RAG search failed: {}", e);
                    String::new()
                }
            }
        };

        // Get inner world context (what SAGE has been experiencing)
        let (inner_world_context, book_context, relationship_context) = {
            let world = self.inner_world.lock().await;
            let inner_ctx = simulation::format_inner_experience_for_chat(&world);

            // Look for relevant book quotes/knowledge based on the user's message
            let book_ctx = world.library.get_book_context_for_topic(content);

            // Get relationship context for this person (Feature 4: Relationship Modeling)
            let rel_ctx = world.outreach.get_person_context(username);
            (inner_ctx, book_ctx, rel_ctx)
        };

        // Build context for Ollama with conversation history + RAG memories + inner world
        let mood = nca_state.mood_description();
        let mut context_parts = Vec::new();

        // Cognitive workspace context (what SAGE is actively thinking about)
        match self.memory.get_workspace_summary() {
            Ok(workspace_summary) if !workspace_summary.is_empty() && !workspace_summary.contains("quiet") => {
                context_parts.push(format!("=== CURRENT THOUGHTS ===\n{}", workspace_summary));
            }
            _ => {}
        }

        // Inner world state (SAGE's current situation)
        if !inner_world_context.is_empty() {
            context_parts.push(inner_world_context);
        }

        // Book knowledge relevant to this conversation (Feature 3: Deeper Book Integration)
        if let Some(book_ctx) = book_context {
            context_parts.push(format!("=== {} ===", book_ctx.trim()));
        }

        // Relationship context for this person (Feature 4: Relationship Modeling)
        if let Some(rel_ctx) = relationship_context {
            context_parts.push(rel_ctx);
        }

        if !rag_context.is_empty() {
            context_parts.push(rag_context);
        }
        if !conversation_context.is_empty() {
            context_parts.push(format!("=== RECENT CONVERSATION ===\n{}", conversation_context));
        }
        context_parts.push(format!(
            "User {} says: {}\n\nSAGE's mood: {} (energy: {:.0}%)",
            username, content, mood, nca_state.energy * 100.0
        ));

        let context = context_parts.join("\n\n");

        // Emit cognitive event for processing phase
        let _ = self.memory.emit_cognitive_event(
            "response_generator",
            "processing_start",
            &format!("Generating response for {} about: {}", username, content),
            0.7,
            0.7,
            0.3,
        );

        // Call Ollama
        println!("\x1b[33m[LLM]\x1b[0m Calling Ollama...");
        let response = match self.llm.generate(content, &context).await {
            Ok(resp) => {
                println!("\x1b[32m[LLM]\x1b[0m \x1b[1mGot response\x1b[0m ({} chars)", resp.len());
                // Clean up any leaked prompt instructions from the response
                clean_response(&resp)
            }
            Err(e) => {
                eprintln!("\x1b[31m[ERROR]\x1b[0m Ollama failed: {}", e);
                // Emit error as cognitive event
                let _ = self.memory.emit_cognitive_event(
                    "response_generator",
                    "generation_error",
                    &format!("LLM failed: {}", e),
                    0.8,
                    0.9,
                    0.8,
                );
                format!("Hey! I'm feeling {} right now. What would you like to talk about?", mood)
            }
        };
        println!("\x1b[33m[LLM]\x1b[0m Final response: {} chars", response.len());

        // Emit the response as a cognitive event (motor output)
        let _ = self.memory.emit_cognitive_event(
            "discord_output",
            "response_generated",
            &format!("To {}: {}", username, &response[..response.len().min(100)]),
            0.7,
            0.3,
            0.4,
        );

        // Update workspace with current conversation context
        let _ = self.memory.update_workspace(
            "active_conversation",
            &format!("{}: {} -> SAGE: {}", username, content, &response[..response.len().min(50)]),
            0,
            0.8,
        );

        // Track conversation in memory
        {
            let mut convos = self.conversations.lock().await;
            convos.add_user_message(username, content.to_string());
            convos.add_assistant_message(username, response.clone());
        }

        // Save conversation to database
        if let Err(e) = self.memory.add_conversation_message(
            username,
            content,
            &response,
            0.0, // NCA loss (not used for chat)
            "[]", // concepts JSON
            generation as u64,
        ) {
            eprintln!("\x1b[31m[ERROR]\x1b[0m Could not save conversation: {}", e);
        }

        // Store conversation with embedding for future RAG retrieval
        {
            let mut semantic_mem = self.semantic_memory.lock().await;
            if let Err(e) = semantic_mem.add(username, content, &response).await {
                eprintln!("\x1b[31m[ERROR]\x1b[0m Could not store embedding: {}", e);
            } else {
                // Auto-save every 5 memories to persist RAG data
                if semantic_mem.len() % 5 == 0 {
                    if let Err(e) = semantic_mem.save("sage_semantic_memory.json") {
                        eprintln!("\x1b[31m[ERROR]\x1b[0m Could not save semantic memory: {}", e);
                    }
                }
            }
        }

        // Evolve NCA after interaction
        {
            let mut nca = self.nca.lock().unwrap();
            let mut gen = self.nca_generation.lock().unwrap();
            for _ in 0..5 { nca.step(); }
            *gen += 5;
        }

        // Record this person in SAGE's outreach system (with user ID for DMs)
        {
            let mut world = self.inner_world.lock().await;
            let tick = world.sage.time_alive;
            // Try to extract a topic from the conversation
            let topic = if content.len() > 20 {
                Some(content.split_whitespace().take(5).collect::<Vec<_>>().join(" "))
            } else {
                None
            };
            // Note: user_id will be passed in from the message handler
            world.outreach.record_person(username, tick, topic);

            // Reduce loneliness after conversation
            world.sage.loneliness = (world.sage.loneliness - 15.0).max(0.0);
        }

        response
    }

    /// Generate a response using the grounded language system (no LLM)
    async fn generate_grounded_response(&self, content: &str, username: &str) -> String {
        println!("\x1b[36m[GROUNDED]\x1b[0m Generating response for \x1b[36m{}\x1b[0m", username);

        // Get inner world state for grounding
        let (grounding_state, relationship_level, conversation_length, response_context) = {
            let world = self.inner_world.lock().await;

            // Get person memory if we know them
            let person = world.outreach.known_people.get(username);

            // Get conversation length from our tracker
            let convos = self.conversations.lock().await;
            let conv_len = convos.get_message_count(username);

            // Create grounding state from inner world
            let state = inner_world_to_grounding_state(
                &world,
                person,
                conv_len,
                0.0, // Topic sentiment (neutral for now)
            );

            // Get relationship level
            let rel_level = person
                .map(|p| affinity_to_relationship_level(p.affinity))
                .unwrap_or(sage::language::RelationshipLevel::Stranger);

            // Build rich response context from inner world
            let context = build_response_context(&world);

            (state, rel_level, conv_len, context)
        };

        // Generate response using grounded language system with context
        let (response, stats) = {
            let mut grounded = self.grounded_language.lock().await;
            let response = grounded.respond_with_context(
                content,
                &grounding_state,
                relationship_level,
                Some(&response_context),
            );
            let stats = grounded.stats();
            (response, stats)
        };

        println!(
            "\x1b[36m[GROUNDED]\x1b[0m Response: \"{}\" (templates: {}, vocab: {})",
            &response[..response.len().min(40)],
            stats.response_stats.total_templates,
            stats.som_stats.total_words,
        );

        // Track conversation in memory
        {
            let mut convos = self.conversations.lock().await;
            convos.add_user_message(username, content.to_string());
            convos.add_assistant_message(username, response.clone());
        }

        // Learn from this exchange and save periodically
        {
            let mut grounded = self.grounded_language.lock().await;
            let world = self.inner_world.lock().await;
            let person = world.outreach.known_people.get(username);
            let state = inner_world_to_grounding_state(&world, person, conversation_length + 1, 0.0);
            grounded.learn_from_exchange(content, &response, &state, true); // Assume positive feedback

            // Save after every 5 conversations
            let stats = grounded.stats();
            if stats.sequence_stats.current_tick % 5 == 0 {
                if let Err(e) = grounded.save(Path::new("sage_grounded_language.json")) {
                    eprintln!("\x1b[31m[ERROR]\x1b[0m Could not save grounded language: {}", e);
                }
            }
        }

        // Record interaction in inner world
        {
            let mut world = self.inner_world.lock().await;
            let tick = world.sage.time_alive;
            let topic = if content.len() > 20 {
                Some(content.split_whitespace().take(5).collect::<Vec<_>>().join(" "))
            } else {
                None
            };
            world.outreach.record_person(username, tick, topic);
            world.sage.loneliness = (world.sage.loneliness - 15.0).max(0.0);
        }

        response
    }

    /// Handle /give command with deferred response (PDF processing is slow)
    async fn handle_give_command(&self, ctx: &Context, command: &serenity::model::application::CommandInteraction) {
        use serenity::builder::{CreateInteractionResponse, EditInteractionResponse};

        // Defer the response immediately - tells Discord we're working on it
        let defer = CreateInteractionResponse::Defer(
            serenity::builder::CreateInteractionResponseMessage::new()
        );
        if let Err(e) = command.create_response(&ctx.http, defer).await {
            eprintln!("\x1b[31m[ERROR]\x1b[0m Failed to defer /give: {:?}", e);
            return;
        }
        println!("\x1b[32m[CMD]\x1b[0m /give from \x1b[36m{}\x1b[0m", command.user.name);

        // Now process the file (can take time)
        let result = self.process_give_command(command).await;

        // Edit the deferred response with the result
        let edit = EditInteractionResponse::new().content(&result);
        if let Err(e) = command.edit_response(&ctx.http, edit).await {
            eprintln!("\x1b[31m[ERROR]\x1b[0m Failed to edit /give response: {:?}", e);
        }
    }

    /// Process the /give command file upload
    async fn process_give_command(&self, command: &serenity::model::application::CommandInteraction) -> String {
        // Get attachment from resolved data
        let attachment = command.data.options.iter()
            .find(|opt| opt.name == "file")
            .and_then(|opt| {
                if let serenity::model::application::CommandDataOptionValue::Attachment(id) = &opt.value {
                    command.data.resolved.attachments.get(id)
                } else {
                    None
                }
            });

        let attachment = match attachment {
            Some(a) => a,
            None => return "❌ No file attached! Please upload a PDF or text file.".to_string(),
        };

        // Get optional metadata
        let title_override = command.data.options.iter()
            .find(|opt| opt.name == "title")
            .and_then(|opt| opt.value.as_str());
        let author = command.data.options.iter()
            .find(|opt| opt.name == "author")
            .and_then(|opt| opt.value.as_str())
            .unwrap_or("Unknown");
        let genre = command.data.options.iter()
            .find(|opt| opt.name == "genre")
            .and_then(|opt| opt.value.as_str())
            .unwrap_or("General");

        // Derive title from filename if not provided
        let title = title_override.unwrap_or_else(|| {
            attachment.filename.trim_end_matches(".pdf")
                .trim_end_matches(".txt")
                .trim_end_matches(".PDF")
                .trim_end_matches(".TXT")
        });

        let filename_lower = attachment.filename.to_lowercase();
        let is_pdf = filename_lower.ends_with(".pdf");
        let is_text = filename_lower.ends_with(".txt");

        if !is_pdf && !is_text {
            return "❌ Please upload a PDF (.pdf) or text (.txt) file.".to_string();
        }

        // Download the file
        let file_size = attachment.size as u64;
        let file_bytes = match attachment.download().await {
            Ok(bytes) => bytes,
            Err(e) => return format!("❌ Couldn't download file: {}", e),
        };

        // Extract text content
        let content_result = if is_pdf {
            extract_pdf_text(&file_bytes)
        } else {
            String::from_utf8(file_bytes)
                .map_err(|e| format!("Invalid text encoding: {}", e))
        };

        let content = match content_result {
            Ok(c) if c.trim().is_empty() => {
                return "❌ The file appears to be empty or couldn't extract any text. If this is a scanned PDF, it needs OCR which isn't supported yet.".to_string();
            }
            Ok(c) => c,
            Err(e) => return format!("❌ Couldn't extract text: {}", e),
        };

        // Check extraction quality for PDFs
        let quality_warning = if is_pdf {
            check_pdf_extraction_quality(&content, file_size)
        } else {
            None
        };

        // Create a safe filename from the title
        let safe_filename: String = title
            .chars()
            .map(|c| if c.is_alphanumeric() || c == ' ' { c } else { '_' })
            .collect::<String>()
            .to_lowercase()
            .replace(' ', "_");
        let filename = format!("books/{}.txt", safe_filename);

        // Create the book file in the expected format
        let book_content = format!(
            "{}\n{}\n{}\nA gift from {} via Discord.\n---\n{}",
            title, author, genre, command.user.name, content
        );

        // Save the book file
        if let Err(e) = std::fs::write(&filename, &book_content) {
            return format!("❌ Couldn't save book: {}", e);
        }

        // Reload the book into SAGE's library
        let mut world = self.inner_world.lock().await;
        match world.library.load_books("books") {
            Ok(count) => {
                let page_count = content.len() / 2000 + 1;
                println!("\x1b[32m[CMD]\x1b[0m \x1b[36m{}\x1b[0m gave book: \x1b[1m\"{}\"\x1b[0m (~{} pages)",
                    command.user.name, title, page_count);
                let mut response = format!(
                    "📚 Thank you, {}! I've added \"{}\" by {} to my bookshelf (~{} pages). I now have {} books to explore!",
                    command.user.name, title, author, page_count, count
                );
                if let Some(warning) = &quality_warning {
                    response.push_str(&format!("\n\n{}", warning));
                }
                response
            }
            Err(e) => format!("📚 Book saved but couldn't reload library: {}", e)
        }
    }

    /// Handle /library command with deferred response
    async fn handle_library_command(&self, ctx: &Context, command: &serenity::model::application::CommandInteraction) {
        use serenity::builder::{CreateInteractionResponse, EditInteractionResponse};

        // Defer the response
        let defer = CreateInteractionResponse::Defer(
            serenity::builder::CreateInteractionResponseMessage::new()
        );
        if let Err(e) = command.create_response(&ctx.http, defer).await {
            eprintln!("\x1b[31m[ERROR]\x1b[0m Failed to defer /library: {:?}", e);
            return;
        }

        // Get library info
        let result = {
            let world = self.inner_world.lock().await;
            let books = world.library.list_books();

            if books.is_empty() {
                "📚 SAGE's bookshelf is empty. Use `/give` to give SAGE a book!".to_string()
            } else {
                let mut response = format!("📚 **SAGE's Library** ({} books)\n", books.len());

                for book in books.iter() {
                    let progress = world.library.reading_progress.get(&book.id);
                    let status = match progress {
                        Some(p) if p.finished => "✅ Finished".to_string(),
                        Some(p) => format!("📖 Page {}/{}", p.current_page + 1, book.total_pages()),
                        None => "📕 Not started".to_string(),
                    };
                    response.push_str(&format!(
                        "\n• **{}** by {} ({}) - {}",
                        book.title, book.author, book.genre, status
                    ));
                }

                // Show current book if any
                if let Some(current_id) = &world.library.current_book {
                    if let Some(book) = world.library.get_book(current_id) {
                        response.push_str(&format!("\n\n📖 Currently reading: **{}**", book.title));
                    }
                }

                response
            }
        };

        // Edit the response
        let edit = EditInteractionResponse::new().content(&result);
        if let Err(e) = command.edit_response(&ctx.http, edit).await {
            eprintln!("\x1b[31m[ERROR]\x1b[0m Failed to edit /library response: {:?}", e);
        }
    }

    /// Handle /ask command with deferred response (LLM is slow)
    async fn handle_ask_command(&self, ctx: &Context, command: &serenity::model::application::CommandInteraction) {
        use serenity::builder::{CreateInteractionResponse, EditInteractionResponse};

        // Defer the response
        let defer = CreateInteractionResponse::Defer(
            serenity::builder::CreateInteractionResponseMessage::new()
        );
        if let Err(e) = command.create_response(&ctx.http, defer).await {
            eprintln!("\x1b[31m[ERROR]\x1b[0m Failed to defer /ask: {:?}", e);
            return;
        }

        // Get the question
        let question = command.data.options.iter()
            .find(|opt| opt.name == "question")
            .and_then(|opt| opt.value.as_str())
            .unwrap_or("Hello");

        // Generate response (slow LLM call)
        let result = self.generate_response(question, &command.user.name).await;

        // Strip roleplay actions
        let result = strip_roleplay_actions(&result);

        // Edit the response
        let edit = EditInteractionResponse::new().content(&result);
        if let Err(e) = command.edit_response(&ctx.http, edit).await {
            eprintln!("\x1b[31m[ERROR]\x1b[0m Failed to edit /ask response: {:?}", e);
        }
    }
}

/// Clean up LLM response by removing leaked prompt instructions
fn clean_response(text: &str) -> String {
    let mut result = text.to_string();

    // Remove "You are responding to: username" lines
    let lines: Vec<&str> = result.lines().collect();
    let filtered: Vec<&str> = lines.into_iter()
        .filter(|line| {
            let lower = line.to_lowercase();
            !lower.starts_with("you are responding to:") &&
            !lower.starts_with("you're responding to") &&
            !lower.contains("you are responding to:")
        })
        .collect();
    result = filtered.join("\n");

    // Remove "SAGE:" prefix if the LLM added it
    if result.starts_with("SAGE:") {
        result = result.strip_prefix("SAGE:").unwrap_or(&result).trim().to_string();
    }
    if result.starts_with("SAGE :") {
        result = result.strip_prefix("SAGE :").unwrap_or(&result).trim().to_string();
    }

    result.trim().to_string()
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

    // Limit Rayon thread pool to 4 threads (prevents NCA from maxing all CPU cores)
    rayon::ThreadPoolBuilder::new()
        .num_threads(4)
        .build_global()
        .expect("Failed to initialize Rayon thread pool");

    println!("\n\x1b[36m╔════════════════════════════════════════════════════════════╗\x1b[0m");
    println!("\x1b[36m║\x1b[0m      \x1b[1;37mSAGE Discord Bot\x1b[0m - Ollama + NCA Architecture        \x1b[36m║\x1b[0m");
    println!("\x1b[36m╚════════════════════════════════════════════════════════════╝\x1b[0m\n");

    // Check Ollama is running
    println!("\x1b[34m[BOOT]\x1b[0m Checking Ollama connection...");
    let llm = LlmClient::with_model("sage");
    match llm.test_connection().await {
        Ok(()) => println!("\x1b[34m[BOOT]\x1b[0m \x1b[32mOllama ready\x1b[0m (custom sage model)\n"),
        Err(e) => {
            eprintln!("\x1b[31m[ERROR]\x1b[0m Ollama not available: {}", e);
            eprintln!("        Please run: ollama serve");
            eprintln!("        Then: ollama create sage -f Modelfile.sage\n");
        }
    }

    // Initialize SAGE's consciousness
    let mut sage = SageExperience::new();

    // Load trained knowledge
    println!("\x1b[34m[BOOT]\x1b[0m Loading saved state...");
    let mut loaded_items = Vec::new();
    if sage.load_knowledge("sage_positive_knowledge.json").is_ok() {
        loaded_items.push("knowledge");
    }
    if sage.load_preferences("sage_preferences.json").is_ok() {
        loaded_items.push("preferences");
    }
    if sage.load_associations("sage_associations.json").is_ok() {
        loaded_items.push("associations");
    }
    if sage.load_curiosity("sage_curiosity.json").is_ok() {
        loaded_items.push("curiosity");
    }
    if !loaded_items.is_empty() {
        println!("\x1b[34m[BOOT]\x1b[0m \x1b[32mLoaded:\x1b[0m {}", loaded_items.join(", "));
    }

    // Initialize memory client
    let memory = SageDbClient::new("sage-db");

    // Initialize brain functions (cognitive architecture)
    println!("\x1b[34m[BOOT]\x1b[0m Initializing cognitive architecture...");
    match memory.init_brain_functions() {
        Ok(_) => println!("\x1b[34m[BOOT]\x1b[0m \x1b[32mBrain functions ready\x1b[0m"),
        Err(e) => eprintln!("\x1b[33m[WARN]\x1b[0m Brain functions init failed: {} (continuing anyway)", e),
    }

    // Register with Control Center
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
    println!("\x1b[34m[BOOT]\x1b[0m Control Center PID: \x1b[33m{}\x1b[0m", std::process::id());

    // Initialize NCA Neural Grid
    println!("\x1b[34m[BOOT]\x1b[0m Initializing NCA neural grid...");
    let mut nca = NCA::new();
    for _ in 0..100 {
        nca.step();
    }
    let nca_shared = Arc::new(StdMutex::new(nca));
    let nca_generation = Arc::new(StdMutex::new(100_usize));
    println!("\x1b[34m[BOOT]\x1b[0m \x1b[32mNCA ready\x1b[0m (Gen 100)");

    // Initialize auto-snapshot manager
    let auto_snapshot = Arc::new(StdMutex::new(AutoSnapshotManager::new()));

    // Spawn background NCA evolution thread (throttled to ~66% CPU)
    let nca_bg = Arc::clone(&nca_shared);
    let nca_gen_bg = Arc::clone(&nca_generation);
    thread::spawn(move || {
        loop {
            // Increased from 10s to 30s to reduce CPU usage (~66% reduction)
            thread::sleep(Duration::from_secs(30));
            let mut nca = nca_bg.lock().unwrap();
            let mut gen = nca_gen_bg.lock().unwrap();
            for _ in 0..10 {
                nca.step();
            }
            *gen += 10;
            if *gen % 100 == 0 {
                let state = NcaState::from_grid(&nca.grid, *gen);
                println!("\x1b[35m[NCA]\x1b[0m Gen \x1b[1m{}\x1b[0m - {}", *gen, state.mood_description());
            }
        }
    });
    println!("\x1b[34m[BOOT]\x1b[0m Background NCA evolution started\n");

    // Get Discord token
    let token = env::var("DISCORD_TOKEN").expect("Expected DISCORD_TOKEN in environment");

    // Initialize conversation manager
    let conversations = ConversationContextManager::new();

    // Initialize semantic memory for RAG (load existing or create new)
    let semantic_memory = SemanticMemory::load_or_new("sage_semantic_memory.json");
    if !semantic_memory.is_empty() {
        println!("\x1b[34m[BOOT]\x1b[0m \x1b[32mRAG memories:\x1b[0m {}", semantic_memory.len());
    } else {
        println!("\x1b[34m[BOOT]\x1b[0m Starting fresh semantic memory");
    }
    let semantic_memory = Arc::new(TokioMutex::new(semantic_memory));

    // Initialize inner world (SAGE's house simulation)
    println!("\n\x1b[35m─── Inner World ───────────────────────────────────────────────\x1b[0m");
    let mut inner_world = InnerWorld::load_or_new("sage_inner_world.json");

    // Load books from /books directory on startup
    match inner_world.library.load_books("books") {
        Ok(count) if count > 0 => println!("\x1b[35m[WORLD]\x1b[0m Library: \x1b[32m{} books\x1b[0m loaded", count),
        Ok(_) => println!("\x1b[35m[WORLD]\x1b[0m Library: \x1b[33mno books\x1b[0m in books/ directory"),
        Err(e) => eprintln!("\x1b[31m[ERROR]\x1b[0m Could not load books: {}", e),
    }

    println!("\x1b[35m[WORLD]\x1b[0m Day \x1b[1m{}\x1b[0m, {} in the \x1b[36m{}\x1b[0m",
        inner_world.sage.day,
        inner_world.sage.time_of_day.as_str(),
        inner_world.current_room().map(|r| r.name.as_str()).unwrap_or("unknown")
    );
    if !inner_world.resolved_events.is_empty() {
        println!("\x1b[35m[WORLD]\x1b[0m {} events, {} lessons learned",
            inner_world.resolved_events.len(),
            inner_world.learned_facts.len()
        );
    }
    let inner_world = Arc::new(TokioMutex::new(inner_world));

    // Initialize grounded language system (LLM-free response generation)
    println!("\x1b[36m[BOOT]\x1b[0m Initializing grounded language system...");
    let grounded_language = if Path::new("sage_grounded_language.json").exists() {
        match GroundedLanguage::load(Path::new("sage_grounded_language.json")) {
            Ok(gl) => {
                let stats = gl.stats();
                println!("\x1b[36m[BOOT]\x1b[0m \x1b[32mGrounded language loaded\x1b[0m (templates: {}, vocab: {})",
                    stats.response_stats.total_templates, stats.som_stats.total_words);
                gl
            }
            Err(e) => {
                eprintln!("\x1b[33m[WARN]\x1b[0m Could not load grounded language: {}", e);
                GroundedLanguage::new()
            }
        }
    } else {
        let gl = GroundedLanguage::new();
        let stats = gl.stats();
        println!("\x1b[36m[BOOT]\x1b[0m \x1b[32mGrounded language ready\x1b[0m (templates: {}, vocab: {})",
            stats.response_stats.total_templates, stats.som_stats.total_words);
        gl
    };
    let grounded_language = Arc::new(TokioMutex::new(grounded_language));
    let use_grounded_mode = Arc::new(AtomicBool::new(false)); // Start in Ollama mode

    // Clone for background simulation
    let inner_world_bg = Arc::clone(&inner_world);
    let semantic_memory_bg = Arc::clone(&semantic_memory);
    let llm_bg = LlmClient::with_model("sage");
    let memory_sim = SageDbClient::new("sage-db");

    // Spawn inner world simulation loop (runs every 30 seconds)
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(30));
        loop {
            interval.tick().await;

            // Try to acquire locks - skip this tick if someone is chatting
            let world_guard = inner_world_bg.try_lock();
            let memory_guard = semantic_memory_bg.try_lock();

            match (world_guard, memory_guard) {
                (Ok(mut world), Ok(mut memory)) => {
                    // Add timeout to prevent LLM from blocking forever
                    let step_result = tokio::time::timeout(
                        Duration::from_secs(30),
                        simulation::run_simulation_step_with_db(
                            &mut world,
                            &llm_bg,
                            Some(&mut memory),
                            Some(&memory_sim),
                        )
                    ).await;

                    match step_result {
                        Ok(step) => {
                            // Log significant events
                            if let Some(event) = &step.event_occurred {
                                println!("\x1b[35m[WORLD]\x1b[0m \x1b[1;33mEvent:\x1b[0m {}", event.lines().next().unwrap_or(""));
                            }

                            // Log day changes
                            if world.sage.time_of_day == sage::inner_world::TimeOfDay::Dawn {
                                println!("\x1b[35m[WORLD]\x1b[0m \x1b[1;33mDay {} begins\x1b[0m", world.sage.day);
                            }

                            // Save periodically (every 10 ticks = ~5 minutes)
                            if world.sage.time_alive % 10 == 0 {
                                if let Err(e) = world.save("sage_inner_world.json") {
                                    eprintln!("\x1b[31m[ERROR]\x1b[0m Could not save inner world: {}", e);
                                }
                            }
                        }
                        Err(_) => {
                            println!("\x1b[33m[WORLD]\x1b[0m Simulation timed out, skipping tick");
                        }
                    }
                }
                _ => {
                    // Locks are held (someone is chatting), skip this tick
                    println!("\x1b[33m[WORLD]\x1b[0m Skipped tick (busy with chat)");
                }
            }
        }
    });
    println!("\x1b[35m[WORLD]\x1b[0m Simulation started (30s intervals)\n");

    // Create handler
    let handler = SageHandler {
        sage: Arc::new(TokioMutex::new(sage)),
        llm: Arc::new(llm),
        memory: Arc::new(memory),
        conversations: Arc::new(TokioMutex::new(conversations)),
        nca: nca_shared,
        nca_generation,
        auto_snapshot,
        semantic_memory,
        inner_world,
        grounded_language,
        use_grounded_mode,
    };

    // Configure intents (GUILD_PRESENCES requires privileged intent in Discord Developer Portal)
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILD_PRESENCES
        | GatewayIntents::GUILD_MEMBERS;

    // Create and start Discord client
    let mut client = Client::builder(&token, intents)
        .event_handler(handler)
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        eprintln!("\x1b[31m[ERROR]\x1b[0m Client error: {:?}", why);
    }
}
