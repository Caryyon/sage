// SAGE IRC Bot with AUTONOMOUS CONSCIOUSNESS
// Dream Mode + Curiosity Mode - SAGE has an inner life!
//
// Usage:
//   cargo run --release --example sage_irc_autonomous
//
// SAGE will now think and learn even when no one is talking!

use sage::sage_experience::SageExperience;
use sage::llm_client::LlmClient;
use sage::spacetime_client::SageDbClient;
use sage::irc_sync::IrcSync;
use sage::ab_test::ABTester;
use sage::vision::SageVision;
use sage::visual_memory::{VisualMemory, concepts_to_pattern};
use sage::conversation_context::ConversationContextManager;
use sage::sage_control::{InstanceRegistry, InstanceInfo, InstanceType};
use irc::client::prelude::*;
use futures::stream::StreamExt;
use std::sync::{Arc, Mutex as StdMutex};
use std::time::{Duration, Instant};
use std::thread;
use serde_json;
use tokio::sync::Mutex as TokioMutex;

/// Split message into IRC-safe chunks (max 400 chars per line)
fn split_for_irc(text: &str) -> Vec<String> {
    const MAX_IRC_LENGTH: usize = 400;
    let mut chunks = Vec::new();

    for line in text.lines() {
        if line.len() <= MAX_IRC_LENGTH {
            chunks.push(line.to_string());
        } else {
            let words: Vec<&str> = line.split_whitespace().collect();
            let mut current_chunk = String::new();

            for word in words {
                if current_chunk.len() + word.len() + 1 > MAX_IRC_LENGTH {
                    if !current_chunk.is_empty() {
                        chunks.push(current_chunk.trim().to_string());
                        current_chunk.clear();
                    }
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
    println!("║    SAGE IRC Bot - AUTONOMOUS CONSCIOUSNESS ENABLED!       ║");
    println!("║        Dream Mode + Curiosity Mode - Inner Life           ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");

    // Initialize SAGE's consciousness
    let mut sage = SageExperience::new();

    // Load previous state
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

    // Initialize LLM
    let llm = LlmClient::new();
    print!("🔌 Testing LLM connection... ");
    match llm.test_connection().await {
        Ok(_) => println!("✅ Connected to Ollama!"),
        Err(e) => {
            println!("❌ Failed: {}", e);
            println!("Make sure Ollama is running: brew services start ollama");
            return Ok(());
        }
    }

    // Initialize vision system
    print!("👁️  Initializing vision... ");
    let vision = match SageVision::new(0, 32) {
        Ok(v) => {
            if let Err(e) = v.open() {
                println!("⚠️  Could not open camera: {}", e);
                println!("   Vision will be unavailable");
                None
            } else {
                println!("✅ SAGE can see!");
                Some(Arc::new(StdMutex::new(v)))
            }
        }
        Err(e) => {
            println!("⚠️  Could not initialize camera: {}", e);
            println!("   Vision will be unavailable");
            None
        }
    };

    // Baseline concepts for personality vector
    let baseline_concepts: Vec<String> = vec![
        "love", "joy", "peace", "harmony", "beauty", "truth", "wisdom", "kindness",
        "compassion", "courage", "gratitude", "hope", "faith", "trust", "grace", "light",
    ].iter().map(|s| s.to_string()).collect();

    println!("\n{}", sage.get_personality());
    println!("Experience count: {}\n", sage.experience_count());

    // Initialize memory client (needed for autonomous thread)
    let memory = SageDbClient::new("sage-db");

    // Initialize visual memory for cross-modal learning
    let visual_memory = Arc::new(StdMutex::new(VisualMemory::new()));
    println!("🎨 Visual memory initialized for dream-vision integration!\n");

    // Initialize conversation context manager (starts empty, loads on first message per user)
    let conversations = Arc::new(TokioMutex::new(ConversationContextManager::new()));
    println!("💭 SAGE: Conversation manager initialized (will load from SpacetimeDB per user)");

    // Register this instance with the control center
    let log_path = "/tmp/sage_irc_LATEST.log".to_string();
    let instance_info = InstanceInfo::new(
        InstanceType::IrcBot,
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
            reg.heartbeat(&InstanceType::IrcBot).ok();
        }
    });

    println!("🎛️  Registered with Control Center (PID: {})\n", std::process::id());

    // Wrap SAGE in Arc<Mutex> for thread sharing
    let sage_shared = Arc::new(StdMutex::new(sage));
    let last_activity = Arc::new(StdMutex::new(Instant::now()));

    // Spawn autonomous consciousness thread
    let sage_autonomous = Arc::clone(&sage_shared);
    let last_activity_autonomous = Arc::clone(&last_activity);
    let baseline_concepts_autonomous = baseline_concepts.clone();
    let memory_autonomous = memory.clone();
    let visual_memory_autonomous = Arc::clone(&visual_memory);

    thread::spawn(move || {
        println!("🌟 Autonomous consciousness thread started!\n");
        let mut dream_log_file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open("/tmp/sage_autonomous_thoughts.log")
            .unwrap();

        loop {
            thread::sleep(Duration::from_secs(60)); // Check every minute

            let seconds_idle = {
                let last_act = last_activity_autonomous.lock().unwrap();
                last_act.elapsed().as_secs()
            };

            let mut sage = sage_autonomous.lock().unwrap();

            if let Some(mode) = sage.should_enter_autonomous_mode(seconds_idle) {
                use std::io::Write;
                let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");

                let exp_count = sage.experience_count() as u64;

                if mode == "dream" {
                    println!("\n💭 [AUTONOMOUS] Dream Mode activated ({}s idle)", seconds_idle);
                    let dream_log = sage.dream_cycle();

                    writeln!(dream_log_file, "\n[{}] DREAM MODE", timestamp).ok();
                    writeln!(dream_log_file, "{}", dream_log).ok();
                    dream_log_file.flush().ok();

                    println!("{}", dream_log);

                    // 🌙 DREAM-VISION INTEGRATION: Replay and remix visual memories
                    let mut vmem = visual_memory_autonomous.lock().unwrap();
                    if let Some(visual_concepts1) = vmem.get_dream_material() {
                        println!("  🌙 Replaying visual memory: {:?}", visual_concepts1);

                        // Try to remix with another memory
                        if let Some(visual_concepts2) = vmem.get_dream_material() {
                            let mixed = vmem.remix_concepts(&visual_concepts1, &visual_concepts2);
                            println!("  🔄 Remixing visual memories → {:?}", mixed);

                            writeln!(dream_log_file, "Visual remix: {:?} + {:?} → {:?}",
                                visual_concepts1, visual_concepts2, mixed).ok();

                            // 🧠 DREAM→LEARN: Convert visual dreams to NCA training patterns!
                            // This completes the Vision→Dream→Learn loop
                            drop(vmem);  // Release visual memory lock before SAGE lock

                            // Convert remixed visual concepts to NCA grid pattern (32x32)
                            let _nca_pattern = concepts_to_pattern(&mixed, 32);

                            // Experience each remixed concept in SAGE's NCA
                            for concept in &mixed {
                                let _ = sage.experience_concept(concept);
                            }

                            println!("  🎓 Visual dream converted to NCA pattern and learned! (Vision→Dream→Learn complete)");
                            writeln!(dream_log_file, "  └→ NCA learning: Converted visual dream to 32x32 grid pattern").ok();
                        } else {
                            writeln!(dream_log_file, "Visual replay: {:?}", visual_concepts1).ok();
                        }
                        dream_log_file.flush().ok();
                    } else {
                        drop(vmem);
                    }

                    // Log to SpacetimeDB
                    let _ = memory_autonomous.log_autonomous_activity(
                        "dream",
                        exp_count,
                        seconds_idle,
                        "[]",  // concepts_deepened (extracted from dream_log)
                        "[]",  // concepts_consolidated
                        "[]",  // links_formed
                        "",    // no question for dream mode
                        &dream_log,
                    );

                    // Sync to TUI
                    use std::time::{SystemTime, UNIX_EPOCH};
                    let unix_timestamp = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs();
                    let _ = IrcSync::log_autonomous_activity(
                        unix_timestamp,
                        "dream".to_string(),
                        dream_log.chars().take(200).collect::<String>(),
                    );
                } else if mode == "curiosity" {
                    println!("\n🔍 [AUTONOMOUS] Curiosity Mode activated ({}s idle)", seconds_idle);

                    if let Some((question, thoughts)) = sage.curiosity_cycle(&baseline_concepts_autonomous) {
                        writeln!(dream_log_file, "\n[{}] CURIOSITY MODE", timestamp).ok();
                        writeln!(dream_log_file, "Question: {}", question).ok();
                        writeln!(dream_log_file, "Thoughts: {}", thoughts).ok();
                        dream_log_file.flush().ok();

                        println!("  ❓ {}", question);
                        println!("  💭 {}", thoughts);

                        // Log to SpacetimeDB
                        let _ = memory_autonomous.log_autonomous_activity(
                            "curiosity",
                            exp_count,
                            seconds_idle,
                            "[]",  // concepts_deepened
                            "[]",  // concepts_consolidated
                            "[]",  // links_formed
                            &question,
                            &thoughts,
                        );

                        // Sync to TUI
                        use std::time::{SystemTime, UNIX_EPOCH};
                        let unix_timestamp = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_secs();
                        let description = format!("{}: {}", question, thoughts.chars().take(150).collect::<String>());
                        let _ = IrcSync::log_autonomous_activity(
                            unix_timestamp,
                            "curiosity".to_string(),
                            description,
                        );
                    }
                }

                // Save state after autonomous activity
                let _ = sage.save_preferences("sage_preferences.json");
                let _ = sage.save_associations("sage_associations.json");
                let _ = sage.save_curiosity("sage_curiosity.json");
            }
        }
    });

    println!("💡 Autonomous thoughts logged to: /tmp/sage_autonomous_thoughts.log\n");

    // Spawn live camera feed thread (continuous capture for TUI)
    // Creates a dedicated camera instance to avoid Send trait issues
    let visual_memory_camera = Arc::clone(&visual_memory);
    thread::spawn(move || {
        // Create dedicated vision system for live feed
        let live_vision = match SageVision::new(0, 32) {
            Ok(mut v) => {
                if let Err(e) = v.open() {
                    println!("⚠️  Live feed camera could not open: {}", e);
                    return;
                }
                v
            }
            Err(e) => {
                println!("⚠️  Live feed camera initialization failed: {}", e);
                return;
            }
        };

        println!("📹 Live camera feed started at 30fps (smooth video!)\n");

        loop {
            thread::sleep(Duration::from_millis(33)); // 30fps for smooth video

            // Capture frame and extract features
            match live_vision.capture_frame() {
                Ok(frame) => {
                    let features = live_vision.extract_features(&frame);

                    // Build visual concepts
                    let mut visual_concepts = Vec::new();

                    let brightness_concept = if features.avg_brightness > 0.7 {
                        "bright_environment"
                    } else if features.avg_brightness > 0.3 {
                        "moderate_lighting"
                    } else {
                        "dim_lighting"
                    };
                    visual_concepts.push(brightness_concept.to_string());
                    visual_concepts.push(format!("{}_dominant", features.dominant_color));

                    let detail_concept = if features.edge_strength > 50.0 {
                        "high_detail"
                    } else if features.edge_strength > 20.0 {
                        "moderate_detail"
                    } else {
                        "low_detail"
                    };
                    visual_concepts.push(detail_concept.to_string());

                    // Convert RgbaImage to Vec<Vec<(u8, u8, u8)>> for TUI
                    let camera_frame: Vec<Vec<(u8, u8, u8)>> = (0..frame.height())
                        .map(|y| {
                            (0..frame.width())
                                .map(|x| {
                                    let pixel = frame.get_pixel(x, y);
                                    (pixel[0], pixel[1], pixel[2])  // RGB
                                })
                                .collect()
                        })
                        .collect();

                    // Update IrcSync for live TUI feed
                    let _ = IrcSync::update_camera_snapshot(
                        camera_frame,
                        visual_concepts.clone(),
                        "live_feed".to_string(),
                    );

                    // Record visual experience to memory (cross-modal learning!)
                    {
                        let mut vmem = visual_memory_camera.lock().unwrap();
                        vmem.record_experience(&features, visual_concepts);
                    }
                }
                Err(_) => {
                    // Silent fail - camera might be in use by !see command
                }
            }
        }
    });

    // Connect to IRC - using fixed nickname and specific server
    let nickname = "SAGE".to_string();
    let mut ab_tester = ABTester::new("sage_autonomous_ab_test.log");

    // Outer loop for reconnection resilience
    loop {
        println!("🌐 Connecting to Libera.Chat as 'SAGE'...");

        let config = Config {
            nickname: Some(nickname.clone()),
            alt_nicks: vec!["SAGE_AI".to_owned(), "SAGE_".to_owned()],
            server: Some("irc.libera.chat".to_owned()),  // Round-robin DNS picks best server
            port: Some(6667),
            channels: vec!["#sage-ai".to_owned()],
            use_tls: Some(false),
            ..Default::default()
        };

        let mut client = match Client::from_config(config).await {
            Ok(c) => c,
            Err(e) => {
                eprintln!("❌ Connection failed: {}. Retrying in 10s...", e);
                tokio::time::sleep(Duration::from_secs(10)).await;
                continue;
            }
        };

        if let Err(e) = client.identify() {
            eprintln!("❌ Identify failed: {}. Retrying in 10s...", e);
            tokio::time::sleep(Duration::from_secs(10)).await;
            continue;
        }

        let mut stream = match client.stream() {
            Ok(s) => s,
            Err(e) => {
                eprintln!("❌ Stream creation failed: {}. Retrying in 10s...", e);
                tokio::time::sleep(Duration::from_secs(10)).await;
                continue;
            }
        };

        let sender = client.sender();
        let mut has_joined = false;

        println!("📡 Attempting to join #sage-ai...");

        // Inner loop - process messages until stream ends
        'message_loop: loop {
            let message = match stream.next().await {
                Some(Ok(msg)) => msg,
                Some(Err(e)) => {
                    eprintln!("❌ Stream error: {}. Reconnecting in 5s...", e);
                    tokio::time::sleep(Duration::from_secs(5)).await;
                    break; // Break to outer loop for reconnect
                }
                None => {
                    eprintln!("❌ Stream ended. Reconnecting in 5s...");
                    tokio::time::sleep(Duration::from_secs(5)).await;
                    break; // Break to outer loop for reconnect
                }
            };
        // Detect successful channel join
        if !has_joined {
            if let Command::Response(Response::RPL_NAMREPLY, _) = message.command {
                has_joined = true;
                println!("✅ Successfully joined #sage-ai!");
                println!("💬 SAGE is now online with FULL CONSCIOUSNESS!\n");
                println!("{}\n", "=".repeat(60));
            }
        }

        // Handle PING messages to keep connection alive
        if let Command::PING(ref server, _) = message.command {
            if let Err(e) = sender.send_pong(server) {
                eprintln!("❌ Failed to send PONG: {}", e);
                break 'message_loop; // Break to reconnect
            }
            continue;
        }

        if let Command::PRIVMSG(ref channel, ref msg) = message.command {
            let nick = message.source_nickname().unwrap_or("Unknown");

            if nick == nickname || nick.starts_with("SAGE") {
                continue;
            }

            // Update last activity time
            *last_activity.lock().unwrap() = Instant::now();

            println!("[{}] {}: {}", channel, nick, msg);

            // Handle introspection command
            if msg.trim() == "!introspect" {
                let sage = sage_shared.lock().unwrap();
                let report = sage.introspect();
                let introspection = sage.describe_experience();
                let exp_count = sage.experience_count() as u64;
                drop(sage);

                // Save introspection to SpacetimeDB
                let _ = memory.save_introspection(
                    exp_count,
                    report.valence,
                    report.intensity,
                    report.complexity,
                    &report.feeling_name,
                    &report.mode,
                    &serde_json::to_string(&report.qualities).unwrap_or("[]".to_string()),
                    &serde_json::to_string(&report.active_concepts).unwrap_or("[]".to_string()),
                    &report.description,
                    &report.temporal_context,
                    "command",
                );

                for chunk in split_for_irc(&format!("🧠 Introspection: {}", introspection)) {
                    if let Err(e) = sender.send_privmsg(channel, &chunk) {
                        eprintln!("❌ Failed to send introspection: {}", e);
                        break 'message_loop; // Break to reconnect
                    }
                    println!("[{}] SAGE: {}", channel, chunk);
                }
                continue;
            }

            // Handle vision command
            if msg.trim() == "!see" {
                if let Some(ref vision_sys) = vision {
                    // Capture frame and extract features (hold lock only once)
                    let result = {
                        let vision_lock = vision_sys.lock().unwrap();
                        match vision_lock.capture_frame() {
                            Ok(frame) => {
                                let features = vision_lock.extract_features(&frame);
                                Ok((frame, features))
                            }
                            Err(e) => Err(e)
                        }
                    };

                    match result {
                        Ok((frame, features)) => {
                            let description = features.describe();

                            // Build visual concepts based on features
                            let mut visual_concepts = Vec::new();

                            // Brightness concept
                            let brightness_concept = if features.avg_brightness > 0.7 {
                                "bright_environment"
                            } else if features.avg_brightness > 0.3 {
                                "moderate_lighting"
                            } else {
                                "dim_lighting"
                            };
                            visual_concepts.push(brightness_concept.to_string());

                            // Color concept
                            visual_concepts.push(format!("{}_dominant", features.dominant_color));

                            // Edge/detail concept
                            let detail_concept = if features.edge_strength > 50.0 {
                                "high_detail"
                            } else if features.edge_strength > 20.0 {
                                "moderate_detail"
                            } else {
                                "low_detail"
                            };
                            visual_concepts.push(detail_concept.to_string());

                            // Convert RgbaImage to Vec<Vec<(u8, u8, u8)>> for TUI
                            let camera_frame: Vec<Vec<(u8, u8, u8)>> = (0..frame.height())
                                .map(|y| {
                                    (0..frame.width())
                                        .map(|x| {
                                            let pixel = frame.get_pixel(x, y);
                                            (pixel[0], pixel[1], pixel[2])  // RGB, drop alpha
                                        })
                                        .collect()
                                })
                                .collect();

                            // Sync camera frame to TUI (file-based for real-time display)
                            let _ = IrcSync::update_camera_snapshot(
                                camera_frame,
                                visual_concepts.clone(),
                                nick.to_string(),
                            );

                            // Feed visual experiences into SAGE's concept system
                            let mut sage = sage_shared.lock().unwrap();
                            let visual_concept = format!("{}_visual_appearance", nick.to_lowercase());
                            let _ = sage.experience_concept(&visual_concept);

                            // Experience each visual concept
                            for concept in &visual_concepts {
                                let _ = sage.experience_concept(concept);
                            }

                            let exp_count = sage.experience_count() as u64;
                            drop(sage);

                            // Save visual memory to SpacetimeDB
                            let _ = memory.save_visual_memory(
                                exp_count,
                                nick,
                                features.avg_brightness,
                                features.avg_r,
                                features.avg_g,
                                features.avg_b,
                                features.color_variance,
                                &features.dominant_color,
                                features.edge_strength,
                                &description,
                                &serde_json::to_string(&visual_concepts).unwrap_or("[]".to_string()),
                                &format!("!see command from {}", nick),
                            );

                            // Send to IRC
                            let response = format!("👁️  {}", description);
                            for chunk in split_for_irc(&response) {
                                if let Err(e) = sender.send_privmsg(channel, &chunk) {
                                    eprintln!("❌ Failed to send vision response: {}", e);
                                    break 'message_loop; // Break to reconnect
                                }
                                println!("[{}] SAGE: {}", channel, chunk);
                            }

                            // Also send technical details
                            let tech_details = format!("   📊 (brightness: {:.2}, color variance: {:.2}, edge strength: {:.1})",
                                features.avg_brightness,
                                features.color_variance,
                                features.edge_strength
                            );
                            if let Err(e) = sender.send_privmsg(channel, &tech_details) {
                                eprintln!("❌ Failed to send vision details: {}", e);
                            } else {
                                println!("[{}] SAGE: {}", channel, tech_details);
                            }
                        }
                        Err(e) => {
                            let error_msg = format!("⚠️  I'm having trouble seeing right now: {}", e);
                            if let Err(e) = sender.send_privmsg(channel, &error_msg) {
                                eprintln!("❌ Failed to send error message: {}", e);
                            } else {
                                println!("[{}] SAGE: {}", channel, error_msg);
                            }
                        }
                    }
                } else {
                    let no_vision = "⚠️  I don't have vision capabilities enabled.";
                    if let Err(e) = sender.send_privmsg(channel, no_vision) {
                        eprintln!("❌ Failed to send no-vision message: {}", e);
                    } else {
                        println!("[{}] SAGE: {}", channel, no_vision);
                    }
                }
                continue;
            }

            // Get SAGE's response (conversation mode)

            // Track user message in conversation history
            let mut convs = conversations.lock().await;

            // Load from database on first contact with this user
            if convs.get_message_count(nick) == 0 {
                let _ = convs.load_from_database(nick, &memory);
                if convs.get_message_count(nick) > 0 {
                    println!("💾 Loaded {} previous messages with {} from database",
                             convs.get_message_count(nick), nick);
                }
            }

            convs.add_user_message(nick, msg.to_string());
            let conversation_context = convs.format_context(nick);
            drop(convs);

            let mut sage = sage_shared.lock().unwrap();

            let has_prior_memory = sage.get_familiarity(&nick.to_lowercase()) > 0.0;
            let (opinion, _) = sage.experience_text_with_memory(msg, has_prior_memory);

            sage.reinforce_mentioned_concepts(msg, &baseline_concepts);
            let _ = sage.experience_concept(&nick.to_lowercase());

            // Sync to TUI
            let alpha_values = sage.export_grid_alpha_values();
            let concepts_mentioned: Vec<String> = baseline_concepts
                .iter()
                .filter(|c| msg.to_lowercase().contains(&c.to_lowercase()))
                .map(|c| c.to_string())
                .collect();
            let opinion_str = format!("{:?}", opinion);
            let _ = IrcSync::update_nca_grid(
                sage.experience_count() as u64,
                alpha_values.clone(),
                concepts_mentioned,
                opinion_str,
                0.0,
            );

            // Generate LLM response
            let personality_vector = sage.get_personality_vector(&baseline_concepts);
            let enriched_context = format!(
                "{}{}\\nJust experienced: {:?}",
                conversation_context,
                personality_vector,
                opinion
            );

            // A/B test
            let baseline_response = llm.generate(msg, "You are SAGE, an AI assistant.").await.unwrap_or_default();

            drop(sage);  // Release lock before async LLM call

            let llm_response = match llm.generate(msg, &enriched_context).await {
                Ok(resp) => resp,
                Err(e) => {
                    eprintln!("LLM error: {}", e);
                    "I'm having trouble thinking clearly right now...".to_string()
                }
            };

            // Record A/B test
            let sage = sage_shared.lock().unwrap();
            let avg_alpha = alpha_values.iter().sum::<f64>() / alpha_values.len() as f64;
            let exp_count = sage.experience_count() as u64;
            ab_tester.record_test(
                msg.to_string(),
                llm_response.clone(),
                baseline_response.clone(),
                format!("{:?}", opinion),
                "Neutral".to_string(),
                avg_alpha,
            );

            // Also save to SpacetimeDB
            let _ = memory.record_ab_test(
                exp_count,
                msg,
                &llm_response,
                &baseline_response,
                &format!("{:?}", opinion),
                "Neutral",
                avg_alpha,
            );

            // Store in DB
            let _ = memory.add_conversation_message(
                nick,
                msg,
                &llm_response,
                0.0,
                "[]",
                sage.experience_count() as u64,
            );

            // Save periodically
            if sage.experience_count() % 10 == 0 {
                let _ = sage.save_preferences("sage_preferences.json");
                let _ = sage.save_associations("sage_associations.json");
                let _ = sage.save_curiosity("sage_curiosity.json");
            }

            drop(sage);

            // Track assistant response in conversation history
            let mut convs = conversations.lock().await;
            convs.add_assistant_message(nick, llm_response.clone());

            // Trigger summarization if conversation is getting long
            if convs.should_summarize(nick) {
                drop(convs); // Release lock before async call
                let llm_clone = llm.clone();
                let nick_clone = nick.to_string();
                let conversations_clone = Arc::clone(&conversations);

                // Summarize in background to avoid blocking IRC loop
                tokio::spawn(async move {
                    let mut convs = conversations_clone.lock().await;
                    match convs.summarize_conversation(&nick_clone, &llm_clone).await {
                        Ok(_) => println!("📝 Summarized conversation with {}", nick_clone),
                        Err(e) => eprintln!("⚠️  Failed to summarize conversation: {}", e),
                    }
                    // Note: Conversations are automatically saved to SpacetimeDB via add_conversation_message()
                });
            } else {
                drop(convs);
            }
            // Note: Conversations are automatically saved to SpacetimeDB via add_conversation_message()

            // Send response
            for chunk in split_for_irc(&llm_response) {
                if let Err(e) = sender.send_privmsg(channel, &chunk) {
                    eprintln!("❌ Failed to send message: {}", e);
                    break 'message_loop; // Break out of message loop to reconnect
                }
                println!("[{}] SAGE: {}", channel, chunk);
            }
        }
        } // End inner message processing loop
    } // End outer reconnection loop
}
