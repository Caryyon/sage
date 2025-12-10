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
// Commands: /state, /evolve, /ask, /save, /load, /snapshots, /give, /library

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

/// Strip roleplay actions like *smiles* or *leans back in chair* from responses
fn strip_roleplay_actions(text: &str) -> String {
    // Remove *action* patterns (asterisk-wrapped text)
    let re = Regex::new(r"\*[^*]+\*").unwrap();
    let result = re.replace_all(text, "");

    // Clean up extra whitespace and leading/trailing spaces
    let result = result.trim();
    let re_spaces = Regex::new(r"\s+").unwrap();
    re_spaces.replace_all(result, " ").to_string()
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
        let is_mentioned = msg.mentions_me(&ctx.http).await.unwrap_or(false);

        // Debug: show all incoming messages
        println!("📩 [{}] {}: \"{}\" (DM: {}, mentioned: {})",
            msg.channel_id, msg.author.name, content, is_dm, is_mentioned);

        if !is_dm && !is_mentioned {
            return;
        }

        // Clean up the message (remove @mention)
        let clean_content = content
            .split_whitespace()
            .filter(|word| !word.starts_with("<@"))
            .collect::<Vec<_>>()
            .join(" ");

        if clean_content.trim().is_empty() {
            println!("⚠️  Empty message after removing mention, sending greeting...");
            // Respond to empty @mention with a greeting
            let _ = msg.channel_id.say(&ctx.http, format!("<@{}> Hey! What's on your mind?", msg.author.id)).await;
            return;
        }

        // Record user ID for potential outreach later
        {
            let mut world = self.inner_world.lock().await;
            let tick = world.sage.time_alive;
            world.outreach.record_person_with_id(
                &msg.author.name,
                msg.author.id.get(),
                tick,
                None, // Topic recorded after response
            );
        }

        // Show typing indicator
        let _ = msg.channel_id.broadcast_typing(&ctx.http).await;

        // Generate response
        let response = self.generate_response(&clean_content, &msg.author.name).await;

        // Strip roleplay actions like *smiles* that the model sometimes adds
        let response = strip_roleplay_actions(&response);

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
                eprintln!("Error sending message: {:?}", e);
            }
        }
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
            println!("👤 {} came online", username);
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("\n╔════════════════════════════════════════════════════════════╗");
        println!("║         SAGE Discord Bot - Ollama + NCA Edition            ║");
        println!("╚════════════════════════════════════════════════════════════╝");
        println!("✅ Connected as: {}", ready.user.name);

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
        ];

        match serenity::model::application::Command::set_global_commands(&ctx.http, commands).await {
            Ok(cmds) => println!("✅ Registered {} slash commands", cmds.len()),
            Err(e) => eprintln!("❌ Failed to register commands: {:?}", e),
        }

        println!("🤖 Ready! @mention me or DM to chat.\n");

        // Spawn proactive outreach loop
        let http = ctx.http.clone();
        let inner_world_outreach = Arc::clone(&self.inner_world);
        let llm_outreach = Arc::clone(&self.llm);

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

                        println!("💭 SAGE wants to reach out to {} ({:?})", target_name, desire.trigger);

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
                                                println!("📤 SAGE messaged {}: \"{}\"", target_name, clean_msg);
                                                // Mark as fulfilled
                                                world.outreach.record_outreach(&target_name, tick);
                                            }
                                            Err(e) => {
                                                eprintln!("❌ Failed to send DM to {}: {}", target_name, e);
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        eprintln!("❌ Failed to create DM channel for {}: {}", target_name, e);
                                    }
                                }
                            } else {
                                println!("💭 SAGE wants to message {} but doesn't know their user ID yet", target_name);
                            }
                        }
                    }
                }
            }
        });
        println!("💬 Proactive outreach system started\n");
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
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
                "ask" => {
                    let question = command.data.options.iter()
                        .find(|opt| opt.name == "question")
                        .and_then(|opt| opt.value.as_str())
                        .unwrap_or("Hello");

                    self.generate_response(question, &command.user.name).await
                }
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
                "give" => {
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
                        None => {
                            "❌ No file attached! Please upload a PDF or text file.".to_string();
                            return;
                        }
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
                        "❌ Please upload a PDF (.pdf) or text (.txt) file.".to_string()
                    } else {
                        // Download the file
                        let file_size = attachment.size as u64;
                        match attachment.download().await {
                            Ok(file_bytes) => {
                                // Extract text content
                                let content_result = if is_pdf {
                                    extract_pdf_text(&file_bytes)
                                } else {
                                    String::from_utf8(file_bytes)
                                        .map_err(|e| format!("Invalid text encoding: {}", e))
                                };

                                match content_result {
                                    Ok(content) if content.trim().is_empty() => {
                                        "❌ The file appears to be empty or couldn't extract any text. If this is a scanned PDF, it needs OCR which isn't supported yet.".to_string()
                                    }
                                    Ok(content) => {
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
                                        match std::fs::write(&filename, &book_content) {
                                            Ok(_) => {
                                                // Reload the book into SAGE's library
                                                let mut world = self.inner_world.lock().await;
                                                match world.library.load_books("books") {
                                                    Ok(count) => {
                                                        let page_count = content.len() / 2000 + 1;
                                                        println!("📚 {} gave SAGE a book: \"{}\" by {} (~{} pages, {} chars)",
                                                            command.user.name, title, author, page_count, content.len());
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
                                            Err(e) => format!("❌ Couldn't save book: {}", e)
                                        }
                                    }
                                    Err(e) => format!("❌ Couldn't extract text: {}", e)
                                }
                            }
                            Err(e) => format!("❌ Couldn't download file: {}", e)
                        }
                    }
                }
                "library" => {
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
                }
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
        // Get NCA state for personality modulation
        let (nca_state, generation) = {
            let nca = self.nca.lock().unwrap();
            let gen = *self.nca_generation.lock().unwrap();
            (NcaState::from_grid(&nca.grid, gen), gen)
        };

        println!("💬 {} says: {}", username, content);
        println!("🧠 NCA: {} (energy: {:.0}%)", nca_state.mood_description(), nca_state.energy * 100.0);

        // Load conversation history from DB if not already loaded
        let conversation_context = {
            let mut convos = self.conversations.lock().await;

            // If no messages for this user, try to load from database
            if convos.get_message_count(username) == 0 {
                if let Err(e) = convos.load_from_database(username, &self.memory) {
                    eprintln!("⚠️  Could not load conversation history: {}", e);
                }
                let count = convos.get_message_count(username);
                if count > 0 {
                    println!("📚 Loaded {} previous messages for {}", count, username);
                }
            }

            // Get formatted conversation history
            convos.format_context(username)
        };

        // Get relevant past conversations using semantic search (RAG)
        let rag_context = {
            let semantic_mem = self.semantic_memory.lock().await;
            match semantic_mem.get_context(content, Some(username), 3).await {
                Ok(ctx) if !ctx.is_empty() => {
                    println!("🔍 RAG: Found {} relevant memories", ctx.lines().filter(|l| l.starts_with("SAGE replied:")).count());
                    ctx
                }
                Ok(_) => String::new(),
                Err(e) => {
                    eprintln!("⚠️  RAG search failed: {}", e);
                    String::new()
                }
            }
        };

        // Get inner world context (what SAGE has been experiencing)
        let inner_world_context = {
            let world = self.inner_world.lock().await;
            simulation::format_inner_experience_for_chat(&world)
        };

        // Build context for Ollama with conversation history + RAG memories + inner world
        let mood = nca_state.mood_description();
        let mut context_parts = Vec::new();

        // Inner world state (SAGE's current situation)
        if !inner_world_context.is_empty() {
            context_parts.push(inner_world_context);
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

        // Call Ollama
        let response = match self.llm.generate(content, &context).await {
            Ok(resp) => {
                println!("✅ Ollama response: {} chars", resp.len());
                // Clean up any leaked prompt instructions from the response
                clean_response(&resp)
            }
            Err(e) => {
                eprintln!("❌ Ollama error: {}", e);
                format!("Hey! I'm feeling {} right now. What would you like to talk about?", mood)
            }
        };

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
            eprintln!("⚠️  Could not save conversation: {}", e);
        } else {
            println!("💾 Saved conversation to database");
        }

        // Store conversation with embedding for future RAG retrieval
        {
            let mut semantic_mem = self.semantic_memory.lock().await;
            if let Err(e) = semantic_mem.add(username, content, &response).await {
                eprintln!("⚠️  Could not store embedding: {}", e);
            } else {
                // Auto-save every 5 memories to persist RAG data
                if semantic_mem.len() % 5 == 0 {
                    if let Err(e) = semantic_mem.save("sage_semantic_memory.json") {
                        eprintln!("⚠️  Could not save semantic memory: {}", e);
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

    println!("\n╔════════════════════════════════════════════════════════════╗");
    println!("║      SAGE Discord Bot - Clean Ollama + NCA Architecture    ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");

    // Check Ollama is running
    println!("🔌 Checking Ollama connection...");
    let llm = LlmClient::with_model("sage");
    match llm.test_connection().await {
        Ok(()) => println!("✅ Ollama ready (custom sage model)\n"),
        Err(e) => {
            eprintln!("❌ Ollama not available: {}", e);
            eprintln!("   Please run: ollama serve");
            eprintln!("   Then: ollama create sage -f Modelfile.sage\n");
        }
    }

    // Initialize SAGE's consciousness
    let mut sage = SageExperience::new();

    // Load trained knowledge
    if sage.load_knowledge("sage_positive_knowledge.json").is_ok() {
        println!("🧠 Loaded trained knowledge");
    }
    if sage.load_preferences("sage_preferences.json").is_ok() {
        println!("💾 Restored preferences");
    }
    if sage.load_associations("sage_associations.json").is_ok() {
        println!("🔗 Loaded associations");
    }
    if sage.load_curiosity("sage_curiosity.json").is_ok() {
        println!("🤔 Loaded curiosity data");
    }
    println!();

    // Initialize memory client
    let memory = SageDbClient::new("sage-db");

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
    println!("🎛️  Registered with Control Center (PID: {})", std::process::id());

    // Initialize NCA Neural Grid
    println!("🧠 Initializing NCA neural grid...");
    let mut nca = NCA::new();
    for _ in 0..100 {
        nca.step();
    }
    let nca_shared = Arc::new(StdMutex::new(nca));
    let nca_generation = Arc::new(StdMutex::new(100_usize));
    println!("✅ NCA grid ready (Gen 100)");

    // Initialize auto-snapshot manager
    let auto_snapshot = Arc::new(StdMutex::new(AutoSnapshotManager::new()));

    // Spawn background NCA evolution thread
    let nca_bg = Arc::clone(&nca_shared);
    let nca_gen_bg = Arc::clone(&nca_generation);
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_secs(10));
            let mut nca = nca_bg.lock().unwrap();
            let mut gen = nca_gen_bg.lock().unwrap();
            for _ in 0..10 {
                nca.step();
            }
            *gen += 10;
            if *gen % 100 == 0 {
                let state = NcaState::from_grid(&nca.grid, *gen);
                println!("🔄 NCA Gen {} - {}", *gen, state.mood_description());
            }
        }
    });
    println!("🔄 Background NCA evolution started\n");

    // Get Discord token
    let token = env::var("DISCORD_TOKEN").expect("Expected DISCORD_TOKEN in environment");

    // Initialize conversation manager
    let conversations = ConversationContextManager::new();

    // Initialize semantic memory for RAG (load existing or create new)
    let semantic_memory = SemanticMemory::load_or_new("sage_semantic_memory.json");
    if !semantic_memory.is_empty() {
        println!("🧠 Loaded {} semantic memories for RAG", semantic_memory.len());
    } else {
        println!("🧠 Starting fresh semantic memory (RAG)");
    }
    let semantic_memory = Arc::new(TokioMutex::new(semantic_memory));

    // Initialize inner world (SAGE's house simulation)
    let inner_world = InnerWorld::load_or_new("sage_inner_world.json");
    println!("🏠 Inner world: Day {}, {} in the {}",
        inner_world.sage.day,
        inner_world.sage.time_of_day.as_str(),
        inner_world.current_room().map(|r| r.name.as_str()).unwrap_or("unknown")
    );
    if !inner_world.resolved_events.is_empty() {
        println!("📖 {} life events experienced, {} lessons learned",
            inner_world.resolved_events.len(),
            inner_world.learned_facts.len()
        );
    }
    let inner_world = Arc::new(TokioMutex::new(inner_world));

    // Clone for background simulation
    let inner_world_bg = Arc::clone(&inner_world);
    let semantic_memory_bg = Arc::clone(&semantic_memory);
    let llm_bg = LlmClient::with_model("sage");

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
                        simulation::run_simulation_step(
                            &mut world,
                            &llm_bg,
                            Some(&mut memory),
                        )
                    ).await;

                    match step_result {
                        Ok(step) => {
                            // Log significant events
                            if let Some(event) = &step.event_occurred {
                                println!("🌟 Inner world event: {}", event.lines().next().unwrap_or(""));
                            }

                            // Log day changes
                            if world.sage.time_of_day == sage::inner_world::TimeOfDay::Dawn {
                                println!("🌅 Inner world: Day {} begins", world.sage.day);
                            }

                            // Save periodically (every 10 ticks = ~5 minutes)
                            if world.sage.time_alive % 10 == 0 {
                                if let Err(e) = world.save("sage_inner_world.json") {
                                    eprintln!("⚠️  Could not save inner world: {}", e);
                                }
                            }
                        }
                        Err(_) => {
                            println!("⏱️  Inner world simulation timed out, skipping tick");
                        }
                    }
                }
                _ => {
                    // Locks are held (someone is chatting), skip this tick
                    println!("💬 Inner world simulation skipped (busy with chat)");
                }
            }
        }
    });
    println!("🌍 Inner world simulation started (30s intervals)\n");

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
        println!("Client error: {:?}", why);
    }
}
