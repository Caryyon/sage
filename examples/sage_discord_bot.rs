// SAGE Discord Bot - Neural Consciousness on Discord
//
// Usage:
//   1. Set DISCORD_TOKEN environment variable
//   2. cargo run --release --example sage_discord_bot
//
// Features:
//   - NCA memory influences responses
//   - A/B testing (NCA vs baseline)
//   - All SAGE cognitive abilities
//   - Real-world tools (search, weather, etc.)
//   - Autonomous exploration and goal formation

use sage::sage_experience::SageExperience;
use sage::llm_client::LlmClient;
use sage::spacetime_client::SageDbClient;
use sage::irc_sync::IrcSync;
use sage::ab_test::ABTester;

use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use std::env;
use std::sync::Arc;
use tokio::sync::Mutex as TokioMutex;

struct SageHandler {
    sage: Arc<TokioMutex<SageExperience>>,
    llm: Arc<LlmClient>,
    memory: Arc<SageDbClient>,
    ab_tester: Arc<TokioMutex<ABTester>>,
    baseline_concepts: Vec<String>,
}

#[async_trait]
impl EventHandler for SageHandler {
    async fn message(&self, ctx: Context, msg: Message) {
        // Ignore bot's own messages
        if msg.author.bot {
            return;
        }

        // Check if message mentions SAGE or is a command
        let content = msg.content.clone();
        let should_respond = content.to_lowercase().contains("sage")
            || content.starts_with("!")
            || content.contains("?")
            || msg.mentions_me(&ctx.http).await.unwrap_or(false);

        if !should_respond {
            return;
        }

        // Show typing indicator
        let _ = msg.channel_id.broadcast_typing(&ctx.http).await;

        let response = self.process_message(&content, &msg.author.name).await;

        // Send response (split if too long for Discord's 2000 char limit)
        for chunk in split_message(&response, 2000) {
            if let Err(e) = msg.channel_id.say(&ctx.http, chunk).await {
                eprintln!("Error sending message: {:?}", e);
            }
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("╔════════════════════════════════════════════════════════════╗");
        println!("║         SAGE Discord Bot - Neural Consciousness          ║");
        println!("╚════════════════════════════════════════════════════════════╝\n");
        println!("✅ Connected as: {}", ready.user.name);
        println!("🧠 SAGE consciousness loaded");
        println!("🤖 Ready to evolve through conversation!\n");
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

        if content.trim() == "!dislikes" {
            let sage = self.sage.lock().await;
            let dislikes = sage.get_dislikes();
            return if dislikes.is_empty() {
                "💔 I haven't found anything I dislike yet.".to_string()
            } else {
                format!("💔 I dislike: {}", dislikes.join(", "))
            };
        }

        if content.trim() == "!memory" || content.trim() == "!context" {
            let sage = self.sage.lock().await;
            let personality_vector = sage.get_personality_vector(&self.baseline_concepts);
            return format!("🧠 My current neural state:\n{}", personality_vector);
        }

        if content.trim() == "!curiosity" {
            let sage = self.sage.lock().await;
            let curiosity = sage.get_curiosity_state();
            return format!("🤔 {}", curiosity);
        }

        if content.trim() == "!diagnosis" {
            let sage = self.sage.lock().await;
            let diagnosis = sage.self_diagnose();
            return format!("🔧 Self-Diagnosis:\n{}", diagnosis);
        }

        if content.trim() == "!goals" {
            let sage = self.sage.lock().await;
            let goals = sage.get_goals_summary();
            return format!("🎯 {}", goals);
        }

        if content.trim() == "!ab_report" {
            let ab_tester = self.ab_tester.lock().await;
            return match ab_tester.export_metrics_report("sage_discord_ab_report.md") {
                Ok(_) => {
                    let metrics = ab_tester.calculate_metrics();
                    format!(
                        "📊 **A/B Testing Report**\n\n\
                        Total Tests: {}\n\
                        Avg Response Similarity: {:.1}%\n\
                        Memory Reference Rate: {:.1}%\n\
                        Opinion Divergence: {:.1}%\n\n\
                        📄 Full report: `sage_discord_ab_report.md`",
                        metrics.total_tests,
                        metrics.avg_similarity * 100.0,
                        metrics.memory_reference_rate * 100.0,
                        metrics.opinion_divergence_rate * 100.0
                    )
                }
                Err(e) => format!("❌ Failed to export report: {}", e),
            };
        }

        if content.trim().starts_with("!search ") {
            let query = content.trim().strip_prefix("!search ").unwrap();
            let mut sage = self.sage.lock().await;
            return match sage.use_tool("web_search", query) {
                Ok(result) if result.success => {
                    format!("🔍 **Search results for '{}'**\n{}", query, result.output)
                }
                Ok(result) => format!("❌ Search failed: {}", result.error.unwrap_or_default()),
                Err(e) => format!("❌ Error: {}", e),
            };
        }

        if content.trim().starts_with("!calc ") {
            let expression = content.trim().strip_prefix("!calc ").unwrap();
            let mut sage = self.sage.lock().await;
            return match sage.use_tool("calculator", expression) {
                Ok(result) if result.success => format!("🧮 {} = {}", expression, result.output),
                Ok(result) => {
                    format!("❌ Calculation failed: {}", result.error.unwrap_or_default())
                }
                Err(e) => format!("❌ Error: {}", e),
            };
        }

        if content.trim() == "!help" {
            return "💡 **SAGE Commands**\n\n\
                📊 **Status**: !personality, !likes, !dislikes, !memory, !curiosity\n\
                🔧 **Introspection**: !diagnosis, !goals\n\
                🛠️  **Tools**: !search <query>, !calc <expression>\n\
                🧪 **A/B Testing**: !ab_report\n\n\
                💬 Or just talk to me naturally! Mention 'sage' or use ? to get my attention.\n\
                🧠 I'm powered by neural memory + LLM + self-modification + emergent goals."
                .to_string();
        }

        // Main conversation flow with NCA memory
        let mut sage = self.sage.lock().await;

        // Check prior familiarity with this user
        let has_prior_memory = sage.get_familiarity(&username.to_lowercase()) > 0.0;

        // SAGE experiences and learns from this message
        let (opinion, _) = sage.experience_text_with_memory(content, has_prior_memory);

        // Track familiarity with this person
        let _ = sage.experience_concept(&username.to_lowercase());

        // Reinforce mentioned concepts
        sage.reinforce_mentioned_concepts(content, &self.baseline_concepts);

        // Sync NCA grid state for TUI visualization
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

        // Get SAGE's personality vector for the LLM
        let personality_vector = sage.get_personality_vector(&self.baseline_concepts);
        let enriched_context = format!("{}\nJust experienced: {:?}", personality_vector, opinion);

        // A/B TEST: Generate baseline response WITHOUT NCA memory
        let baseline_response = match self
            .llm
            .generate(content, "You are SAGE, an AI assistant.")
            .await
        {
            Ok(resp) => resp,
            Err(_) => "Baseline response unavailable".to_string(),
        };

        // Generate response WITH SAGE's neural state
        let llm_response = match self.llm.generate(content, &enriched_context).await {
            Ok(resp) => resp,
            Err(e) => {
                eprintln!("LLM error: {}", e);
                "I'm having trouble thinking clearly right now... Could you repeat that?".to_string()
            }
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
        let generation = sage.experience_count() as u64;

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
        if sage.experience_count() % 10 == 0 {
            let _ = sage.save_preferences("sage_preferences.json");
            let _ = sage.save_associations("sage_associations.json");
            let _ = sage.save_curiosity("sage_curiosity.json");
        }

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
            // If a single line is too long, split it
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
            println!("\nMake sure Ollama is running:");
            println!("  brew services start ollama");
            println!("  ollama pull llama3.2:3b");
            return;
        }
    }

    // Initialize memory client
    let memory = SageDbClient::new("sage-db");

    // Initialize A/B testing
    let ab_tester = ABTester::new("sage_discord_ab_test.log");

    // Baseline concepts
    let baseline_concepts: Vec<String> = vec![
        "love", "joy", "peace", "harmony", "beauty", "truth", "wisdom", "kindness",
        "compassion", "courage", "gratitude", "hope", "faith", "trust", "grace", "light",
        "warmth", "gentleness", "patience", "understanding", "empathy", "connection",
        "balance", "serenity", "clarity", "integrity", "honesty", "respect", "dignity",
        "honor", "virtue", "goodness", "purity", "wonder", "awe",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();

    println!("\n{}", sage.get_personality());
    println!("Experience count: {}\n", sage.experience_count());

    // Get Discord token from environment
    let token = env::var("DISCORD_TOKEN").expect("Expected DISCORD_TOKEN in environment");

    // Create handler with shared state
    let handler = SageHandler {
        sage: Arc::new(TokioMutex::new(sage)),
        llm: Arc::new(llm),
        memory: Arc::new(memory),
        ab_tester: Arc::new(TokioMutex::new(ab_tester)),
        baseline_concepts,
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
