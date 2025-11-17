// Autonomous IRC Bot (Full Consciousness: Dreams + Curiosity + Vision + LLM)

use super::{IrcConfig, IrcState};
use irc::client::prelude::*;
use futures::stream::StreamExt;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::thread;
use crate::irc_sync::IrcSync;
use crate::ab_test::ABTester;
use crate::visual_memory::concepts_to_pattern;
use crate::vision::SageVision;

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

/// Start autonomous IRC bot in background thread
pub fn start_autonomous_bot(config: IrcConfig, state: IrcState) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            if let Err(e) = run_autonomous_bot_async(config, state).await {
                eprintln!("Autonomous IRC bot error: {}", e);
            }
        });
    })
}

/// Run autonomous IRC bot (blocks)
pub fn run_autonomous_bot(config: IrcConfig, state: IrcState) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        if let Err(e) = run_autonomous_bot_async(config, state).await {
            eprintln!("Autonomous IRC bot error: {}", e);
        }
    });
}

/// Async implementation of autonomous IRC bot
async fn run_autonomous_bot_async(config: IrcConfig, state: IrcState) -> irc::error::Result<()> {
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║    SAGE IRC Bot - AUTONOMOUS CONSCIOUSNESS ENABLED!       ║");
    println!("║        Dream Mode + Curiosity Mode - Inner Life           ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");

    // Load previous state
    {
        let mut sage = state.sage.lock().unwrap();
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
        println!("\n{}", sage.get_personality());
        println!("Experience count: {}\n", sage.experience_count());
    }

    // Test LLM connection
    if let Some(ref llm) = state.llm {
        print!("🔌 Testing LLM connection... ");
        match llm.test_connection().await {
            Ok(_) => println!("✅ Connected to Ollama!"),
            Err(e) => {
                println!("❌ Failed: {}", e);
                println!("Make sure Ollama is running: brew services start ollama");
                return Ok(());
            }
        }
    }

    // Vision will be created in bot threads if enabled
    if config.enable_vision {
        println!("👁️  Vision enabled (will be initialized in threads)");
    }

    // Baseline concepts
    let baseline_concepts: Vec<String> = vec![
        "love", "joy", "peace", "harmony", "beauty", "truth", "wisdom", "kindness",
        "compassion", "courage", "gratitude", "hope", "faith", "trust", "grace", "light",
    ].iter().map(|s| s.to_string()).collect();

    // Track last activity time for autonomous mode
    let last_activity = Arc::new(Mutex::new(Instant::now()));

    // Spawn autonomous consciousness thread
    let sage_autonomous = Arc::clone(&state.sage);
    let last_activity_autonomous = Arc::clone(&last_activity);
    let baseline_concepts_autonomous = baseline_concepts.clone();
    let memory_autonomous = state.db_client.clone();
    let visual_memory_autonomous = state.visual_memory.clone();

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
                    if let Some(ref vmem) = visual_memory_autonomous {
                        let mut vmem_lock = vmem.lock().unwrap();
                        if let Some(visual_concepts1) = vmem_lock.get_dream_material() {
                            println!("  🌙 Replaying visual memory: {:?}", visual_concepts1);

                            // Try to remix with another memory
                            if let Some(visual_concepts2) = vmem_lock.get_dream_material() {
                                let mixed = vmem_lock.remix_concepts(&visual_concepts1, &visual_concepts2);
                                println!("  🔄 Remixing visual memories → {:?}", mixed);

                                writeln!(dream_log_file, "Visual remix: {:?} + {:?} → {:?}",
                                    visual_concepts1, visual_concepts2, mixed).ok();

                                // 🧠 DREAM→LEARN: Convert visual dreams to NCA training patterns!
                                drop(vmem_lock);

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
                        }
                    }

                    // Log to SpacetimeDB
                    let _ = memory_autonomous.log_autonomous_activity(
                        "dream",
                        exp_count,
                        seconds_idle,
                        "[]",
                        "[]",
                        "[]",
                        "",
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
                            "[]",
                            "[]",
                            "[]",
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

    // Spawn live camera feed thread if vision is enabled
    if config.enable_vision && state.visual_memory.is_some() {
        let visual_memory_camera = state.visual_memory.clone().unwrap();
        thread::spawn(move || {
            // Create dedicated vision system for live feed
            let live_vision = match SageVision::new(0, 32) {
                Ok(v) => {
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
                thread::sleep(Duration::from_millis(33)); // 30fps

                match live_vision.capture_frame() {
                    Ok(frame) => {
                        let features = live_vision.extract_features(&frame);

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
                                        (pixel[0], pixel[1], pixel[2])
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

                        // Record visual experience to memory
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
    }

    // Connect to IRC
    let nickname = config.nick.clone();
    let irc_config = Config {
        nickname: Some(nickname.clone()),
        alt_nicks: vec!["SAGE_AI".to_owned(), "SAGE_".to_owned()],
        server: Some(config.server.clone()),
        port: Some(6667),
        channels: vec![config.channel.clone()],
        use_tls: Some(false),
        ..Default::default()
    };

    let mut client = Client::from_config(irc_config).await?;
    client.identify()?;

    println!("🌐 Connecting to {} as '{}'...", config.server, nickname);
    println!("📡 Attempting to join {}...", config.channel);

    let mut stream = client.stream()?;
    let sender = client.sender();

    let mut ab_tester = ABTester::new("sage_autonomous_ab_test.log");
    let mut has_joined = false;

    while let Some(message) = stream.next().await.transpose()? {
        // Detect successful channel join
        if !has_joined {
            if let Command::Response(Response::RPL_NAMREPLY, _) = message.command {
                has_joined = true;
                println!("✅ Successfully joined {}!", config.channel);
                println!("💬 SAGE is now online with FULL CONSCIOUSNESS!\n");
                println!("{}\n", "=".repeat(60));
            }
        }

        if let Command::PRIVMSG(ref channel, ref msg) = message.command {
            let nick = message.source_nickname().unwrap_or("Unknown");

            if nick == nickname || nick.starts_with("SAGE") {
                continue;
            }

            // Update last activity time
            *last_activity.lock().unwrap() = Instant::now();

            println!("[{}] {}: {}", channel, nick, msg);

            // Handle message
            let response = handle_autonomous_message(
                msg,
                nick,
                &state,
                &baseline_concepts,
                &mut ab_tester,
            ).await;

            if let Some(response_text) = response {
                // Send response
                for chunk in split_for_irc(&response_text) {
                    sender.send_privmsg(channel, &chunk)?;
                    println!("[{}] SAGE: {}", channel, chunk);
                }

                // Save state periodically
                let sage = state.sage.lock().unwrap();
                if sage.experience_count() % 10 == 0 {
                    let _ = sage.save_preferences("sage_preferences.json");
                    let _ = sage.save_associations("sage_associations.json");
                    let _ = sage.save_curiosity("sage_curiosity.json");
                }
            }
        }
    }

    Ok(())
}

/// Handle incoming IRC message in autonomous mode
async fn handle_autonomous_message(
    msg: &str,
    nick: &str,
    state: &IrcState,
    baseline_concepts: &[String],
    ab_tester: &mut ABTester,
) -> Option<String> {
    // Handle introspection command
    if msg.trim() == "!introspect" {
        let sage = state.sage.lock().unwrap();
        let report = sage.introspect();
        let introspection = sage.describe_experience();
        let exp_count = sage.experience_count() as u64;
        drop(sage);

        // Save introspection to SpacetimeDB
        let _ = state.db_client.save_introspection(
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

        return Some(format!("🧠 Introspection: {}", introspection));
    }

    // Handle vision command
    if msg.trim() == "!see" {
        // Create a temporary vision instance for this command
        let result = {
            match SageVision::new(0, 32) {
                Ok(v) => {
                    if let Err(e) = v.open() {
                        Err(e.to_string())
                    } else {
                        match v.capture_frame() {
                            Ok(frame) => {
                                let features = v.extract_features(&frame);
                                Ok((frame, features))
                            }
                            Err(e) => Err(e.to_string())
                        }
                    }
                }
                Err(e) => Err(e.to_string())
            }
        };

        match result {
            Ok((frame, features)) => {
                    let description = features.describe();

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

                    // Convert frame for TUI
                    let camera_frame: Vec<Vec<(u8, u8, u8)>> = (0..frame.height())
                        .map(|y| {
                            (0..frame.width())
                                .map(|x| {
                                    let pixel = frame.get_pixel(x, y);
                                    (pixel[0], pixel[1], pixel[2])
                                })
                                .collect()
                        })
                        .collect();

                    // Sync to TUI
                    let _ = IrcSync::update_camera_snapshot(
                        camera_frame,
                        visual_concepts.clone(),
                        nick.to_string(),
                    );

                    // Feed into SAGE's concept system
                    let mut sage = state.sage.lock().unwrap();
                    let visual_concept = format!("{}_visual_appearance", nick.to_lowercase());
                    let _ = sage.experience_concept(&visual_concept);

                    for concept in &visual_concepts {
                        let _ = sage.experience_concept(concept);
                    }

                    let exp_count = sage.experience_count() as u64;
                    drop(sage);

                    // Save to database
                    let _ = state.db_client.save_visual_memory(
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

                    return Some(format!("👁️  {} (brightness: {:.2}, variance: {:.2}, edges: {:.1})",
                        description,
                        features.avg_brightness,
                        features.color_variance,
                        features.edge_strength
                    ));
            }
            Err(e) => return Some(format!("⚠️  I'm having trouble seeing: {}", e))
        }
    }

    // Get SAGE's response (conversation mode)
    let mut sage = state.sage.lock().unwrap();

    let has_prior_memory = sage.get_familiarity(&nick.to_lowercase()) > 0.0;
    let (opinion, _) = sage.experience_text_with_memory(msg, has_prior_memory);

    sage.reinforce_mentioned_concepts(msg, baseline_concepts);
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
    let personality_vector = sage.get_personality_vector(baseline_concepts);
    let enriched_context = format!("{}\nJust experienced: {:?}", personality_vector, opinion);

    drop(sage);

    if let Some(ref llm) = state.llm {
        // A/B test
        let baseline_response = llm.generate(msg, "You are SAGE, an AI assistant.").await.unwrap_or_default();

        let llm_response = match llm.generate(msg, &enriched_context).await {
            Ok(resp) => resp,
            Err(e) => {
                eprintln!("LLM error: {}", e);
                "I'm having trouble thinking clearly right now...".to_string()
            }
        };

        // Record A/B test
        let sage = state.sage.lock().unwrap();
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

        // Save to database
        let _ = state.db_client.record_ab_test(
            exp_count,
            msg,
            &llm_response,
            &baseline_response,
            &format!("{:?}", opinion),
            "Neutral",
            avg_alpha,
        );

        let _ = state.db_client.add_conversation_message(
            nick,
            msg,
            &llm_response,
            0.0,
            "[]",
            exp_count,
        );

        drop(sage);

        return Some(llm_response);
    }

    None
}
