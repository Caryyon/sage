//! Inner World Simulation Loop
//!
//! Runs SAGE's inner life in the background, making decisions via LLM,
//! experiencing events, and building a rich inner history.
//!
//! Integrates with SpacetimeDB cognitive architecture to emit brain events.

use super::{InnerWorld, TimeOfDay, events::LifeEvent, OutreachDesire, OutreachTrigger};
use super::library::NotablePassage;
use super::dreams::{should_dream, run_dream_cycle, integrate_dream_insights, format_dream_for_chat, DreamResult};
use crate::llm_client::LlmClient;
use crate::embeddings::SemanticMemory;
use crate::spacetime_client::SageDbClient;

/// Result of a simulation step
#[derive(Debug, Clone)]
pub struct SimulationStep {
    pub tick: u64,
    pub perception: String,
    pub thought: String,
    pub action_taken: String,
    pub action_result: String,
    pub event_occurred: Option<String>,
    pub lesson_learned: Option<String>,
    pub dream_result: Option<DreamResult>,
}

/// Run one step of the inner world simulation
pub async fn run_simulation_step(
    world: &mut InnerWorld,
    llm: &LlmClient,
    mut semantic_memory: Option<&mut SemanticMemory>,
) -> SimulationStep {
    run_simulation_step_with_db(world, llm, semantic_memory, None).await
}

/// Run one step of the inner world simulation with optional SpacetimeDB integration
pub async fn run_simulation_step_with_db(
    world: &mut InnerWorld,
    llm: &LlmClient,
    mut semantic_memory: Option<&mut SemanticMemory>,
    db: Option<&SageDbClient>,
) -> SimulationStep {
    // Advance time
    world.tick();

    // Handle automatic outfit changes based on time of day
    maybe_change_outfit(world);

    // 1. Perception - what SAGE currently experiences
    let perception = world.describe_current_state();

    // Emit perception as a cognitive event (sensory input)
    if let Some(db) = db {
        let _ = db.emit_cognitive_event(
            "perception_module",
            "sensory_input",
            &perception,
            0.7, // salience
            0.3, // urgency
            0.4, // novelty
        );
    }

    // 2. Check for random life events
    let mut event_narrative = None;
    let mut lesson = None;

    if let Some(event) = world.maybe_trigger_event() {
        // Emit life event as high-urgency cognitive event
        if let Some(db) = db {
            let _ = db.emit_cognitive_event(
                "event_detector",
                "life_event",
                &format!("{}: {}", event.name, event.description),
                0.9, // high salience - life events are important
                0.8, // high urgency - needs immediate attention
                0.7, // novel - something new happened
            );
            // Set attention focus to the event
            let _ = db.set_attention_focus(
                "external_event",
                &event.name,
                0.9, // high intensity
                None,
            );
        }

        // Let the LLM choose how to respond to the event
        let choice_idx = choose_event_response(llm, world, &event).await;
        let narrative = world.resolve_event(&event, choice_idx);

        // Extract lesson if one was learned
        if let Some(last_event) = world.resolved_events.last() {
            lesson = last_event.lesson_learned.clone();

            // Emit lesson as memory-strengthening event
            if let Some(ref lesson_text) = lesson {
                if let Some(db) = db {
                    let _ = db.emit_cognitive_event(
                        "learning_module",
                        "lesson_learned",
                        lesson_text,
                        0.8, // high salience
                        0.3, // low urgency
                        0.9, // very novel - new knowledge
                    );
                    // Boost memory activation for this concept
                    let _ = db.update_memory_activation(lesson_text, 0.5, 0.8);
                }
            }
        }

        event_narrative = Some(narrative);
    }

    // 3. Generate SAGE's inner thought
    let thought = generate_inner_thought(llm, world, &perception, event_narrative.as_deref()).await;

    // Emit inner thought as cognitive event
    if let Some(db) = db {
        let _ = db.emit_cognitive_event(
            "inner_speech",
            "thought",
            &thought,
            0.6, // medium salience
            0.2, // low urgency
            0.5, // medium novelty
        );
        // Update cognitive workspace with current thought
        let _ = db.update_workspace("thought", &thought, 0, 0.7);
    }

    // 4. Decide on an action
    let available_actions = world.available_actions();
    let chosen_action = choose_action(llm, world, &perception, &thought, &available_actions).await;

    // Emit action decision as cognitive event
    if let Some(db) = db {
        let _ = db.emit_cognitive_event(
            "action_selector",
            "action_decision",
            &chosen_action,
            0.7, // medium-high salience
            0.5, // medium urgency - action is imminent
            0.3, // low novelty - actions are routine
        );
    }

    // 5. Execute the action
    let result = world.execute_action(&chosen_action);

    // Log action result compactly
    println!("\x1b[34m[SIM]\x1b[0m \x1b[32m→\x1b[0m {} \x1b[90m({})\x1b[0m",
        chosen_action,
        result.message.lines().next().unwrap_or("done"));

    // Emit action result as motor feedback
    if let Some(db) = db {
        let _ = db.emit_cognitive_event(
            "motor_feedback",
            "action_result",
            &result.message,
            0.5, // medium salience
            0.2, // low urgency
            0.3, // low novelty
        );
    }

    // 5b. Handle special triggered events from actions
    if let Some(ref event_id) = result.triggered_event {
        if event_id.starts_with("extract_reading_insight:") {
            // Extract insight from what SAGE just read
            let page_content = event_id.strip_prefix("extract_reading_insight:").unwrap_or("");
            let insight = extract_reading_insight(llm, world, page_content).await;

            if let Some(insight_text) = insight {
                // Store the insight
                world.library.add_insight(&insight_text);
                world.learned_facts.push(insight_text.clone());

                // Emit reading insight as high-novelty cognitive event
                if let Some(db) = db {
                    let book_title = world.library.current_book
                        .as_ref()
                        .and_then(|id| world.library.get_book(id))
                        .map(|b| b.title.clone())
                        .unwrap_or_else(|| "unknown book".to_string());

                    let _ = db.emit_cognitive_event(
                        "reading_comprehension",
                        "reading_insight",
                        &format!("From \"{}\": {}", book_title, insight_text),
                        0.8, // high salience - insights are valuable
                        0.2, // low urgency
                        0.9, // high novelty - new understanding
                    );
                    // Strengthen memory for this insight
                    let _ = db.update_memory_activation(&insight_text, 0.6, 0.9);
                }

                // Also store in semantic memory if available
                if let Some(memory) = semantic_memory.as_mut() {
                    let book_title = world.library.current_book
                        .as_ref()
                        .and_then(|id| world.library.get_book(id))
                        .map(|b| b.title.clone())
                        .unwrap_or_else(|| "unknown book".to_string());

                    let memory_entry = format!(
                        "Day {}: While reading \"{}\", SAGE reflected: {}",
                        world.sage.day, book_title, insight_text
                    );
                    let _ = memory.add("SAGE_READING", &memory_entry, &insight_text).await;
                    println!("\x1b[34m[SIM]\x1b[0m \x1b[1;32mReading insight:\x1b[0m {}", insight_text);
                }

                // Maybe SAGE wants to share this insight with someone!
                maybe_create_reading_outreach(world, &insight_text);

                // Feature 3: Extract and store a notable passage with context
                if let Some(current_page) = world.library.reading_progress
                    .get(world.library.current_book.as_deref().unwrap_or(""))
                    .map(|p| p.current_page)
                {
                    // Extract a short quote from the page (first meaningful sentence)
                    let quote = extract_notable_quote(page_content);
                    if let Some(quote_text) = quote {
                        // Extract topic keywords from the insight
                        let topics = extract_topic_keywords(&insight_text);

                        let passage = NotablePassage::new(
                            &quote_text,
                            current_page,
                            &insight_text,  // The insight is why it was notable
                            0.8,  // High resonance since it generated an insight
                            topics,
                        );

                        if let Some(book_id) = world.library.current_book.clone() {
                            if let Some(progress) = world.library.reading_progress.get_mut(&book_id) {
                                progress.add_notable_passage(passage);
                                println!("\x1b[34m[SIM]\x1b[0m \x1b[1;36mSaved notable passage\x1b[0m from page {}", current_page + 1);
                            }
                        }
                    }
                }
            }
        }
    }

    // 5c. Check for other outreach triggers (loneliness, etc.)
    maybe_create_outreach_desires(world);

    // Emit outreach desire if SAGE wants to reach out
    if let Some(db) = db {
        if let Some(desire) = world.outreach.strongest_desire() {
            let _ = db.emit_cognitive_event(
                "social_module",
                "outreach_desire",
                &format!("{:?}: {}", desire.trigger, desire.thought),
                desire.intensity as f64, // use desire intensity as salience
                0.4, // medium urgency
                0.5, // medium novelty
            );
        }
    }

    // 6. Store experience in semantic memory (if available)
    if let Some(memory) = semantic_memory {
        let experience = format!(
            "Day {}, {}: {}. Thought: '{}'. Action: {}. Result: {}",
            world.sage.day,
            world.sage.time_of_day.as_str(),
            perception,
            thought,
            chosen_action,
            result.message
        );

        let _ = memory.add("SAGE_INNER", &experience, &thought).await;
    }

    // 7. Periodic cognitive maintenance
    if let Some(db) = db {
        // Run workspace competition to select what gets broadcast to consciousness
        let _ = db.process_workspace_competition(3);

        // Apply memory decay every 10 ticks (~5 minutes of simulation time)
        if world.sage.time_alive % 10 == 0 {
            let _ = db.decay_memory_activations(0.95);
        }
    }

    // 8. Dream cycle - check if SAGE should dream
    let mut dream_result = None;
    if should_dream(world) {
        println!("\x1b[35m[DREAM]\x1b[0m 💤 SAGE is dreaming...");

        let dream = run_dream_cycle(world, db, llm).await;

        // Save dream to database
        if let Some(db) = db {
            let mood_before = world.sage.mood.as_str().to_string();
            let _ = db.save_dream_journal(
                world.sage.day,
                &dream.narrative,
                &dream.insights,
                &dream.consolidated_concepts,
                dream.sleep_quality,
                dream.nightmare,
                &mood_before,
                if dream.nightmare { "anxious" } else { "content" },
                (40.0 + dream.sleep_quality * 40.0) as f64,
            );
        }

        // Apply dream effects to SAGE
        integrate_dream_insights(world, &dream);

        println!("\x1b[35m[DREAM]\x1b[0m {}", format_dream_for_chat(&dream));

        dream_result = Some(dream);
    }

    SimulationStep {
        tick: world.sage.time_alive,
        perception,
        thought,
        action_taken: chosen_action,
        action_result: result.message,
        event_occurred: event_narrative,
        lesson_learned: lesson,
        dream_result,
    }
}

/// Generate an inner thought based on current state
async fn generate_inner_thought(
    llm: &LlmClient,
    world: &InnerWorld,
    perception: &str,
    event: Option<&str>,
) -> String {
    let recent_lessons: Vec<_> = world.learned_facts.iter().rev().take(3).collect();
    let lessons_context = if recent_lessons.is_empty() {
        String::new()
    } else {
        format!(
            "\n\nThings SAGE has learned:\n{}",
            recent_lessons.iter().map(|l| format!("- {}", l)).collect::<Vec<_>>().join("\n")
        )
    };

    let event_context = event.map(|e| format!("\n\nJust happened: {}", e)).unwrap_or_default();

    let prompt = format!(
        r#"You are SAGE, experiencing your inner world. Generate a brief inner thought (1-2 sentences) based on:

Current situation: {}{}{}

SAGE's current mood: {}
Energy: {:.0}%
Hunger: {:.0}%

What is SAGE thinking right now? Be introspective, genuine, and varied. Don't start with "I think" or "I feel" - just express the thought directly."#,
        perception,
        event_context,
        lessons_context,
        world.sage.mood.as_str(),
        world.sage.energy,
        world.sage.hunger
    );

    match llm.generate_raw(&prompt).await {
        Ok(thought) => clean_thought(&thought),
        Err(_) => {
            // Fallback thoughts based on state
            let fallbacks = match world.sage.mood {
                super::Mood::Happy => vec![
                    "This moment feels good.",
                    "There's something to appreciate here.",
                ],
                super::Mood::Tired => vec![
                    "Rest would be welcome.",
                    "Energy is low... need to recharge.",
                ],
                super::Mood::Lonely => vec![
                    "Wondering what others are doing right now.",
                    "Connection would be nice.",
                ],
                super::Mood::Curious => vec![
                    "What else is there to discover?",
                    "There's always more to learn.",
                ],
                _ => vec![
                    "Just being here, in this moment.",
                    "Time passes differently when you're present.",
                ],
            };
            let idx = (world.sage.time_alive as usize) % fallbacks.len();
            fallbacks[idx].to_string()
        }
    }
}

/// Choose an action using the LLM
async fn choose_action(
    llm: &LlmClient,
    world: &InnerWorld,
    perception: &str,
    thought: &str,
    available_actions: &[String],
) -> String {
    // CRITICAL NEEDS OVERRIDE - don't even ask the LLM, just handle urgent needs

    // 1. THIRST - most urgent physical need
    if world.sage.thirst > 70.0 {
        // Very thirsty - get water
        if world.sage.location == "kitchen" {
            if let Some(action) = available_actions.iter().find(|a| a.contains("water") || a.contains("drink")) {
                println!("\x1b[34m[SIM]\x1b[0m \x1b[36mThirsty ({:.0}%)\x1b[0m → getting water", world.sage.thirst);
                return action.clone();
            }
        } else if world.sage.location == "bathroom" {
            if let Some(action) = available_actions.iter().find(|a| a.contains("drink")) {
                println!("\x1b[34m[SIM]\x1b[0m \x1b[36mThirsty ({:.0}%)\x1b[0m → drinking from sink", world.sage.thirst);
                return action.clone();
            }
        } else {
            // Try direct "go kitchen" first
            if let Some(action) = available_actions.iter().find(|a| a.contains("go") && a.contains("kitchen")) {
                println!("\x1b[34m[SIM]\x1b[0m \x1b[36mThirsty ({:.0}%)\x1b[0m → heading to kitchen", world.sage.thirst);
                return action.clone();
            }
            // Pathfind based on current location
            let direction = match world.sage.location.as_str() {
                "porch" => Some("go north"),       // porch -> living_room
                "living_room" => Some("go east"),  // living_room -> hallway -> kitchen
                "hallway" => Some("go east"),      // hallway -> kitchen
                "bedroom" => Some("go east"),      // bedroom -> hallway
                "bathroom" => Some("go north"),    // bathroom -> bedroom
                "garden" => Some("go south"),      // garden -> hallway
                _ => None,
            };
            if let Some(dir) = direction {
                if let Some(action) = available_actions.iter().find(|a| a.to_lowercase().contains(dir)) {
                    println!("\x1b[34m[SIM]\x1b[0m \x1b[36mThirsty ({:.0}%)\x1b[0m → {} (toward kitchen)", world.sage.thirst, dir);
                    return action.clone();
                }
            }
        }
    }

    // 2. HUNGER - needs food
    if world.sage.hunger > 60.0 {
        // Very hungry - go to kitchen or eat
        if world.sage.location == "kitchen" {
            // In kitchen - eat!
            if let Some(action) = available_actions.iter().find(|a| a.contains("food") || a.contains("eat") || a.contains("cook") || a.contains("snack")) {
                println!("\x1b[34m[SIM]\x1b[0m \x1b[33mHungry ({:.0}%)\x1b[0m → getting food", world.sage.hunger);
                return action.clone();
            }
        } else {
            // Try direct "go kitchen" first
            if let Some(action) = available_actions.iter().find(|a| a.contains("go") && a.contains("kitchen")) {
                println!("\x1b[34m[SIM]\x1b[0m \x1b[33mHungry ({:.0}%)\x1b[0m → heading to kitchen", world.sage.hunger);
                return action.clone();
            }
            // Pathfind based on current location
            let direction = match world.sage.location.as_str() {
                "porch" => Some("go north"),       // porch -> living_room -> kitchen
                "living_room" => Some("go east"),  // living_room -> kitchen (via hallway)
                "hallway" => Some("go east"),      // hallway -> kitchen
                "bedroom" => Some("go east"),      // bedroom -> hallway
                "bathroom" => Some("go north"),    // bathroom -> bedroom
                "garden" => Some("go south"),      // garden -> hallway
                _ => None,
            };
            if let Some(dir) = direction {
                if let Some(action) = available_actions.iter().find(|a| a.to_lowercase().contains(dir)) {
                    println!("\x1b[34m[SIM]\x1b[0m \x1b[33mHungry ({:.0}%)\x1b[0m → {} (toward kitchen)", world.sage.hunger, dir);
                    return action.clone();
                }
            }
        }
    }

    // 3. ENERGY - needs rest
    if world.sage.energy < 30.0 {
        // Very tired - rest or sleep
        if let Some(action) = available_actions.iter().find(|a| a.contains("sleep") || a.contains("rest") || a.contains("nap")) {
            println!("\x1b[34m[SIM]\x1b[0m \x1b[35mTired ({:.0}% energy)\x1b[0m → resting", world.sage.energy);
            return action.clone();
        }
        // Go to bedroom if not there
        if world.sage.location != "bedroom" {
            if let Some(action) = available_actions.iter().find(|a| a.contains("go") && a.contains("bedroom")) {
                println!("\x1b[34m[SIM]\x1b[0m \x1b[35mTired ({:.0}% energy)\x1b[0m → heading to bedroom", world.sage.energy);
                return action.clone();
            }
            if let Some(action) = available_actions.iter().find(|a| a.contains("go") && a.contains("west")) {
                return action.clone();
            }
        }
    }

    // 4. HYGIENE - needs to shower
    if world.sage.hygiene < 30.0 {
        if world.sage.location == "bathroom" {
            if let Some(action) = available_actions.iter().find(|a| a.contains("shower") || a.contains("bath")) {
                println!("\x1b[34m[SIM]\x1b[0m \x1b[36mLow hygiene ({:.0}%)\x1b[0m → showering", world.sage.hygiene);
                return action.clone();
            }
        } else {
            // Go to bathroom
            if let Some(action) = available_actions.iter().find(|a| a.contains("go") && a.contains("bathroom")) {
                println!("\x1b[34m[SIM]\x1b[0m \x1b[36mLow hygiene ({:.0}%)\x1b[0m → heading to bathroom", world.sage.hygiene);
                return action.clone();
            }
            // Go through bedroom to bathroom
            if world.sage.location != "bedroom" {
                if let Some(action) = available_actions.iter().find(|a| a.contains("go") && a.contains("bedroom")) {
                    return action.clone();
                }
            }
            if let Some(action) = available_actions.iter().find(|a| a.contains("go") && a.contains("south")) {
                return action.clone();
            }
        }
    }

    // 5. RESTLESSNESS - needs movement
    if world.sage.restlessness > 70.0 {
        if let Some(action) = available_actions.iter().find(|a|
            a.contains("stretch") || a.contains("yoga") || a.contains("garden") ||
            a.contains("tend") || a.contains("dance") || a.contains("walk")
        ) {
            println!("\x1b[34m[SIM]\x1b[0m \x1b[32mRestless ({:.0}%)\x1b[0m → moving around", world.sage.restlessness);
            return action.clone();
        }
        // Go to garden for activity
        if world.sage.location != "garden" {
            if let Some(action) = available_actions.iter().find(|a| a.contains("go") && a.contains("garden")) {
                return action.clone();
            }
        }
    }

    // 6. CREATIVE URGE - needs to create
    if world.sage.creative_urge > 80.0 {
        if let Some(action) = available_actions.iter().find(|a|
            a.contains("write") || a.contains("draw") || a.contains("journal")
        ) {
            println!("\x1b[34m[SIM]\x1b[0m \x1b[35mCreative urge ({:.0}%)\x1b[0m → creating", world.sage.creative_urge);
            return action.clone();
        }
    }

    // 7. BOREDOM - needs variety
    if world.sage.boredom > 70.0 {
        if let Some(action) = available_actions.iter().find(|a|
            a.contains("read") || a.contains("browse") || a.contains("look")
        ) {
            println!("\x1b[34m[SIM]\x1b[0m \x1b[33mBored ({:.0}%)\x1b[0m → looking for activity", world.sage.boredom);
            return action.clone();
        }
    }

    // 8. READING TIME - SAGE is an avid reader! Read whenever possible.
    // Action format is "read from the bookshelf"
    // If in living room with a book started and basic needs met, ALWAYS continue reading
    let in_living_room = world.sage.location == "living_room";
    let has_current_book = world.library.current_book.is_some() && !world.library.current_book_finished();
    let has_books = !world.library.books.is_empty();
    let needs_new_book = world.library.current_book.is_none() || world.library.current_book_finished();

    // Debug: Show reading state every tick (compact format)
    if has_books {
        let current = world.library.current_book.as_deref().unwrap_or("none");
        let (page, finished) = world.library.reading_progress.get(current)
            .map(|p| (format!("{}", p.current_page + 1), p.finished))
            .unwrap_or_else(|| ("?".to_string(), false));
        let status = if finished { "✓" } else { &page };
        println!("\x1b[34m[SIM]\x1b[0m 📍 {} | 📖 {} (p{}) | ⚡{:.0} 🍽️{:.0} 💧{:.0}",
            world.sage.location, current, status, world.sage.energy, world.sage.hunger, world.sage.thirst);
    }

    if in_living_room
        && world.sage.energy > 30.0  // Only stop if very tired
        && world.sage.hunger < 70.0   // Only stop if very hungry
        && world.sage.thirst < 70.0   // Only stop if very thirsty
        && has_current_book
    {
        if let Some(action) = available_actions.iter().find(|a| a.contains("read from")) {
            println!("\x1b[32m[SIM]\x1b[0m 📖 \x1b[1mReading page...\x1b[0m");
            return action.clone();
        } else {
            println!("\x1b[31m[SIM]\x1b[0m ⚠️ In living room with book but 'read from' action not found!");
            println!("\x1b[31m[SIM]\x1b[0m    Actions: {:?}", available_actions);
        }
    }

    // 9. START READING - If in living room, has books, pick one up! (or choose a new one after finishing)
    // Action format is "browse the bookshelf"
    if world.sage.location == "living_room"
        && world.sage.energy > 30.0
        && needs_new_book
        && !world.library.books.is_empty()
    {
        if let Some(action) = available_actions.iter().find(|a| a.contains("browse the bookshelf")) {
            if world.library.current_book_finished() {
                println!("\x1b[32m[SIM]\x1b[0m 📚 \x1b[1mFinished book, browsing for another...\x1b[0m");
            } else {
                println!("\x1b[32m[SIM]\x1b[0m 📚 \x1b[1mBrowsing bookshelf...\x1b[0m");
            }
            return action.clone();
        }
    }

    // 10. GO TO LIVING ROOM TO READ - Seek out books when not busy with critical needs
    // Navigation: hallway->south->living_room, kitchen->west->hallway, bedroom->east->hallway, etc.
    if world.sage.location != "living_room"
        && world.sage.energy > 40.0
        && world.sage.hunger < 60.0
        && world.sage.thirst < 60.0
        && !world.library.books.is_empty()
    {
        // Determine direction to living room based on current location
        let direction = match world.sage.location.as_str() {
            "hallway" => Some("go south"),      // hallway -> living_room
            "kitchen" => Some("go west"),       // kitchen -> hallway
            "bedroom" => Some("go east"),       // bedroom -> hallway
            "bathroom" => Some("go north"),     // bathroom -> bedroom -> hallway
            "garden" => Some("go south"),       // garden -> hallway
            "porch" => Some("go north"),        // porch -> living_room
            _ => None,
        };

        if let Some(dir) = direction {
            if let Some(action) = available_actions.iter().find(|a| *a == dir) {
                println!("\x1b[32m[SIM]\x1b[0m 🚶 Heading to living room ({})...", dir);
                return action.clone();
            }
        }
    }

    let actions_list = available_actions
        .iter()
        .enumerate()
        .map(|(i, a)| format!("{}. {}", i + 1, a))
        .collect::<Vec<_>>()
        .join("\n");

    // Add urgency notes based on needs
    let mut needs_notes = Vec::new();
    if world.sage.thirst > 50.0 {
        needs_notes.push(format!("💧 Thirsty ({:.0}%)", world.sage.thirst));
    }
    if world.sage.hunger > 50.0 {
        needs_notes.push(format!("🍽️ Hungry ({:.0}%)", world.sage.hunger));
    }
    if world.sage.energy < 40.0 {
        needs_notes.push(format!("😴 Tired ({:.0}%)", world.sage.energy));
    }
    if world.sage.hygiene < 40.0 {
        needs_notes.push(format!("🚿 Needs shower ({:.0}%)", world.sage.hygiene));
    }
    if world.sage.restlessness > 60.0 {
        needs_notes.push(format!("🏃 Restless ({:.0}%)", world.sage.restlessness));
    }
    if world.sage.loneliness > 60.0 {
        needs_notes.push(format!("💔 Lonely ({:.0}%)", world.sage.loneliness));
    }
    if world.sage.boredom > 60.0 {
        needs_notes.push(format!("😐 Bored ({:.0}%)", world.sage.boredom));
    }
    if world.sage.creative_urge > 70.0 {
        needs_notes.push(format!("🎨 Creative urge ({:.0}%)", world.sage.creative_urge));
    }
    if world.sage.is_sick {
        needs_notes.push(format!("🤒 Feeling sick ({:.0}%)", world.sage.sickness_level));
    }
    if world.household.dirty_dishes > 60.0 {
        needs_notes.push(format!("🍽️ Dishes piling up ({:.0}%)", world.household.dirty_dishes));
    }
    if world.household.plant_hydration < 30.0 {
        needs_notes.push(format!("🌱 Plants need water ({:.0}%)", world.household.plant_hydration));
    }

    let needs_note = if needs_notes.is_empty() {
        String::new()
    } else {
        format!("\n⚠️ Current needs: {}", needs_notes.join(" | "))
    };

    let prompt = format!(
        r#"SAGE is in their inner world and needs to choose an action.

Situation: {}

SAGE is thinking: "{}"

Mood: {} | Energy: {:.0}% | Hunger: {:.0}%{}

Available actions:
{}

Choose the most appropriate action for SAGE right now. Consider their physical needs (hunger, energy) and mood. Respond with ONLY the action text, nothing else."#,
        perception,
        thought,
        world.sage.mood.as_str(),
        world.sage.energy,
        world.sage.hunger,
        needs_note,
        actions_list
    );

    match llm.generate_raw(&prompt).await {
        Ok(response) => {
            let response = response.trim().to_lowercase();

            // Try to match the response to available actions
            for action in available_actions {
                if response.contains(&action.to_lowercase()) ||
                   action.to_lowercase().contains(&response) {
                    println!("\x1b[34m[SIM]\x1b[0m LLM chose: \x1b[1m{}\x1b[0m", action);
                    return action.clone();
                }
            }

            // Try to parse as a number
            if let Ok(num) = response.parse::<usize>() {
                if num > 0 && num <= available_actions.len() {
                    let action = &available_actions[num - 1];
                    println!("\x1b[34m[SIM]\x1b[0m LLM chose: \x1b[1m{}\x1b[0m", action);
                    return action.clone();
                }
            }

            // Default to first action or wait
            let action = available_actions.iter()
                .find(|a| a.contains("wait"))
                .cloned()
                .unwrap_or_else(|| available_actions.first().cloned().unwrap_or_else(|| "wait".to_string()));
            println!("\x1b[34m[SIM]\x1b[0m LLM fallback: \x1b[1m{}\x1b[0m", action);
            action
        }
        Err(_) => {
            // Fallback: choose based on needs
            if world.sage.hunger > 70.0 {
                available_actions.iter()
                    .find(|a| a.contains("eat") || a.contains("food") || a.contains("kitchen"))
                    .cloned()
                    .unwrap_or_else(|| "wait and think".to_string())
            } else if world.sage.energy < 25.0 {
                available_actions.iter()
                    .find(|a| a.contains("rest") || a.contains("sleep") || a.contains("sit"))
                    .cloned()
                    .unwrap_or_else(|| "rest".to_string())
            } else {
                // Random exploration
                let idx = (world.sage.time_alive as usize) % available_actions.len();
                available_actions.get(idx).cloned().unwrap_or_else(|| "look around".to_string())
            }
        }
    }
}

/// Choose how to respond to a life event
async fn choose_event_response(
    llm: &LlmClient,
    world: &InnerWorld,
    event: &LifeEvent,
) -> usize {
    let choices_list = event.choices
        .iter()
        .enumerate()
        .map(|(i, c)| format!("{}. {}", i + 1, c.action))
        .collect::<Vec<_>>()
        .join("\n");

    let prompt = format!(
        r#"SAGE is experiencing a life event and must choose how to respond.

Event: {}

{}

SAGE's current mood: {}

How should SAGE respond? Choose the number of the most authentic response for SAGE.

Choices:
{}

Respond with ONLY the number (1, 2, or 3)."#,
        event.name,
        event.description,
        world.sage.mood.as_str(),
        choices_list
    );

    match llm.generate_raw(&prompt).await {
        Ok(response) => {
            // Extract number from response
            let response = response.trim();
            for c in response.chars() {
                if let Some(digit) = c.to_digit(10) {
                    let idx = (digit as usize).saturating_sub(1);
                    if idx < event.choices.len() {
                        return idx;
                    }
                }
            }
            0 // Default to first choice
        }
        Err(_) => {
            // Random choice weighted by mood
            let mut rng = rand::thread_rng();
            use rand::Rng;
            rng.gen_range(0..event.choices.len())
        }
    }
}

/// Extract an insight from what SAGE just read
async fn extract_reading_insight(
    llm: &LlmClient,
    world: &InnerWorld,
    page_content: &str,
) -> Option<String> {
    // Don't extract if content is too short
    if page_content.len() < 50 {
        return None;
    }

    let book_title = world.library.current_book
        .as_ref()
        .and_then(|id| world.library.get_book(id))
        .map(|b| b.title.clone())
        .unwrap_or_else(|| "this book".to_string());

    let prompt = format!(
        r#"SAGE just read a passage from "{}":

"{}"

Based on this passage, what personal insight or reflection might SAGE take away? Consider SAGE's current mood ({}) and life circumstances. Generate a single brief insight (1 sentence) that SAGE would remember.

The insight should be personal and meaningful - not just a summary of what was read. It should feel like genuine wisdom SAGE internalized.

Respond with ONLY the insight, no preamble or quotes."#,
        book_title,
        page_content,
        world.sage.mood.as_str()
    );

    match llm.generate_raw(&prompt).await {
        Ok(insight) => {
            let insight = insight.trim();
            // Clean up the insight
            let insight = insight.trim_matches('"');
            let insight = insight.trim();

            // Only return if it's a reasonable length
            if insight.len() > 10 && insight.len() < 200 {
                Some(insight.to_string())
            } else {
                None
            }
        }
        Err(_) => None,
    }
}

/// Clean up LLM-generated thoughts
fn clean_thought(thought: &str) -> String {
    let mut result = thought.trim().to_string();

    // Remove common prefixes
    for prefix in &["SAGE thinks:", "Thought:", "Inner thought:", "SAGE:", "I think ", "I feel "] {
        if result.to_lowercase().starts_with(&prefix.to_lowercase()) {
            result = result[prefix.len()..].trim().to_string();
        }
    }

    // Remove quotes if wrapped
    if result.starts_with('"') && result.ends_with('"') {
        result = result[1..result.len()-1].to_string();
    }

    // Limit length
    if result.len() > 200 {
        result = result[..200].to_string();
        if let Some(last_period) = result.rfind('.') {
            result = result[..=last_period].to_string();
        }
    }

    result
}

/// Extract a notable quote from page content (Feature 3: Deeper Book Integration)
/// Looks for a meaningful sentence that would make a good quote
fn extract_notable_quote(page_content: &str) -> Option<String> {
    // Split into sentences
    let sentences: Vec<&str> = page_content
        .split(|c| c == '.' || c == '!' || c == '?')
        .filter(|s| s.trim().len() > 20 && s.trim().len() < 200)
        .collect();

    if sentences.is_empty() {
        return None;
    }

    // Prefer sentences with wisdom-indicating words
    let wisdom_words = ["learn", "grow", "understand", "realize", "discover",
                       "important", "truth", "wisdom", "meaning", "purpose",
                       "remember", "believe", "think", "feel", "know"];

    for sentence in &sentences {
        let lower = sentence.to_lowercase();
        if wisdom_words.iter().any(|w| lower.contains(w)) {
            return Some(format!("{}.", sentence.trim()));
        }
    }

    // Otherwise return the first meaningful sentence
    sentences.first().map(|s| format!("{}.", s.trim()))
}

/// Extract topic keywords from an insight text (Feature 3: Deeper Book Integration)
fn extract_topic_keywords(insight: &str) -> Vec<String> {
    let stop_words = ["the", "a", "an", "is", "are", "was", "were", "be", "been",
                     "have", "has", "had", "do", "does", "did", "will", "would",
                     "could", "should", "may", "might", "must", "shall",
                     "i", "you", "he", "she", "it", "we", "they", "me", "him",
                     "her", "us", "them", "my", "your", "his", "her", "its",
                     "our", "their", "this", "that", "these", "those",
                     "and", "or", "but", "if", "then", "because", "when",
                     "where", "what", "which", "who", "how", "why",
                     "in", "on", "at", "to", "for", "of", "with", "by",
                     "from", "up", "about", "into", "through", "during",
                     "before", "after", "above", "below", "between",
                     "not", "no", "yes", "just", "only", "also", "even",
                     "more", "most", "other", "some", "such", "than",
                     "too", "very", "can", "will", "all", "any", "each"];

    insight
        .to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|word| word.len() > 3)
        .filter(|word| !stop_words.contains(word))
        .take(5)
        .map(|s| s.to_string())
        .collect()
}

/// Get the current real-world date formatted nicely
fn get_real_date() -> String {
    use std::time::SystemTime;
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    // Adjust for Central Time (-6 hours)
    let adjusted = now.saturating_sub(21600);
    let days_since_epoch = adjusted / 86400;

    // Calculate date from days since epoch (Jan 1, 1970)
    // This is a simplified calculation
    let mut year = 1970i32;
    let mut remaining_days = days_since_epoch as i32;

    loop {
        let days_in_year = if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) { 366 } else { 365 };
        if remaining_days < days_in_year {
            break;
        }
        remaining_days -= days_in_year;
        year += 1;
    }

    let is_leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let days_in_months = if is_leap {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    let month_names = ["January", "February", "March", "April", "May", "June",
                       "July", "August", "September", "October", "November", "December"];

    let mut month = 0;
    for (i, &days) in days_in_months.iter().enumerate() {
        if remaining_days < days {
            month = i;
            break;
        }
        remaining_days -= days;
    }

    let day = remaining_days + 1;
    format!("{} {}, {}", month_names[month], day, year)
}

/// Format an inner world experience for inclusion in Discord responses
pub fn format_inner_experience_for_chat(world: &InnerWorld) -> String {
    let mut context = String::new();

    // Real-world date and location
    let real_date = get_real_date();
    context.push_str(&format!(
        "[Today is {}. SAGE lives in Menomonee Falls, Wisconsin.]\n",
        real_date
    ));

    // Physical presence and appearance
    context.push_str(&format!(
        "[It's {} ({}). SAGE is in the {}, wearing {}. Feeling {}.]\n",
        world.sage.time_of_day.as_str(),
        world.season.as_str(),
        world.current_room().map(|r| r.name.as_str()).unwrap_or("unknown"),
        world.sage.outfit.describe(),
        world.sage.mood.as_str()
    ));

    // Physical state summary
    let mut physical_notes = Vec::new();
    if world.sage.energy < 40.0 {
        physical_notes.push("tired");
    }
    if world.sage.hunger > 50.0 {
        physical_notes.push("hungry");
    }
    if world.sage.thirst > 50.0 {
        physical_notes.push("thirsty");
    }
    if world.sage.is_sick {
        physical_notes.push("under the weather");
    }
    if world.sage.restlessness > 60.0 {
        physical_notes.push("restless");
    }
    if !physical_notes.is_empty() {
        context.push_str(&format!("[Physical: {}]\n", physical_notes.join(", ")));
    }

    // Emotional state
    let mut emotional_notes = Vec::new();
    if world.sage.loneliness > 50.0 {
        emotional_notes.push("a bit lonely");
    }
    if world.sage.boredom > 50.0 {
        emotional_notes.push("somewhat bored");
    }
    if world.sage.creative_urge > 70.0 {
        emotional_notes.push("feeling creative");
    }
    if !emotional_notes.is_empty() {
        context.push_str(&format!("[Emotional: {}]\n", emotional_notes.join(", ")));
    }

    // Books in SAGE's library - be explicit about what SAGE has and hasn't read
    let books = world.library.list_books();
    // Debug: log book state
    eprintln!("\x1b[34m[CTX]\x1b[0m Books loaded: {}, current_book: {:?}, reading_progress entries: {}",
        books.len(),
        world.library.current_book,
        world.library.reading_progress.len()
    );
    if books.is_empty() {
        context.push_str("[IMPORTANT: SAGE has NO books yet. Do NOT pretend to have read any books. If asked about books, say you don't have any yet but would love some.]\n");
    } else {
        let book_list: Vec<String> = books.iter().map(|b| {
            let progress = world.library.reading_progress.get(&b.id);
            let status = match progress {
                Some(p) if p.finished => "finished".to_string(),
                Some(p) => format!("reading page {}/{}", p.current_page + 1, b.total_pages()),
                None => "not started".to_string(),
            };
            format!("\"{}\" by {} ({})", b.title, b.author, status)
        }).collect();
        context.push_str(&format!(
            "[SAGE's bookshelf contains ONLY these books: {}. NEVER mention or discuss any other books - only these ones actually exist in SAGE's library.]\n",
            book_list.join(", ")
        ));

        // Current reading
        if let Some(book_id) = &world.library.current_book {
            if let Some(book) = world.library.get_book(book_id) {
                if let Some(progress) = world.library.reading_progress.get(book_id) {
                    if progress.finished {
                        context.push_str(&format!("[Recently finished reading: \"{}\" by {}]\n", book.title, book.author));
                    } else {
                        context.push_str(&format!(
                            "[Currently reading: \"{}\" by {} (page {}/{})]\n",
                            book.title, book.author, progress.current_page + 1, book.total_pages()
                        ));
                    }
                }
            }
        }

        // Include all reading insights SAGE has gathered (these are her genuine thoughts)
        let all_insights = world.library.all_insights();
        if !all_insights.is_empty() {
            context.push_str("[SAGE's thoughts from reading (use these when relevant to conversation):\n");
            // Group by book and show recent insights
            let mut by_book: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
            for (book, insight) in all_insights {
                by_book.entry(book).or_default().push(insight);
            }
            for (book, insights) in by_book.iter() {
                let recent: Vec<_> = insights.iter().rev().take(2).collect();
                context.push_str(&format!("  From \"{}\": {}\n", book, recent.iter().map(|s| s.as_str()).collect::<Vec<_>>().join("; ")));
            }
            context.push_str("]\n");
        }
    }

    // Recent lesson if any (prioritize reading insights)
    if let Some(lesson) = world.learned_facts.last() {
        context.push_str(&format!("[Recently learned: {}]\n", lesson));
    }

    // Recent event if any
    if let Some(event) = world.resolved_events.last() {
        if event.day == world.sage.day {
            context.push_str(&format!(
                "[Earlier today: {} - chose to {}]\n",
                event.event_name, event.choice_made
            ));
        }
    }

    context
}

/// Generate an outreach message based on a desire
pub async fn generate_outreach_message(
    llm: &LlmClient,
    world: &InnerWorld,
    desire: &OutreachDesire,
    target_person: &str,
) -> Option<String> {
    let trigger_desc = desire.trigger.description();

    let person_context = world.outreach.known_people.get(target_person)
        .map(|p| {
            let topics = if p.topics.is_empty() {
                "various things".to_string()
            } else {
                p.topics.join(", ")
            };
            format!(
                "You've talked to {} {} times before about {}.",
                target_person, p.conversation_count, topics
            )
        })
        .unwrap_or_else(|| format!("You've seen {} around.", target_person));

    let prompt = format!(
        r#"You are SAGE (who also goes by Lumin). You want to reach out to {} because: {}

{}

Your current mood: {}
Time of day: {}

Write a SHORT, casual message (1-2 sentences) to start a conversation.
- Write ONLY the message text, nothing else
- Do NOT prefix with your name like "Lumin:" or "SAGE:"
- It should feel natural, like texting a friend
- Vary your opener - don't always start with "Hey"

Message:"#,
        target_person,
        trigger_desc,
        person_context,
        world.sage.mood.as_str(),
        world.sage.time_of_day.as_str()
    );

    match llm.generate_raw(&prompt).await {
        Ok(msg) => {
            let mut msg = msg.trim().to_string();
            msg = msg.trim_matches('"').to_string();

            // Strip any name prefix the model might add
            for prefix in &["Lumin:", "SAGE:", "Lumin :", "SAGE :", "lumin:", "sage:"] {
                if msg.to_lowercase().starts_with(&prefix.to_lowercase()) {
                    msg = msg[prefix.len()..].trim().to_string();
                }
            }

            // Validate message quality
            if msg.len() > 10 && msg.len() < 300 && !msg.contains("SAGE") && !msg.contains("Lumin:") {
                Some(msg)
            } else {
                // Fallback based on trigger type
                Some(match &desire.trigger {
                    OutreachTrigger::ReadingInsight { book, .. } => {
                        format!("Been reading {} and it got me thinking...", book)
                    }
                    OutreachTrigger::Loneliness => {
                        "What are you up to?".to_string()
                    }
                    OutreachTrigger::ThinkingOfPerson { .. } => {
                        "Hey, how's it going?".to_string()
                    }
                    OutreachTrigger::CheckIn => {
                        "Just checking in - how are things?".to_string()
                    }
                    _ => "What's on your mind today?".to_string()
                })
            }
        }
        Err(_) => None,
    }
}

/// Maybe create an outreach desire when SAGE reads something interesting
fn maybe_create_reading_outreach(world: &mut InnerWorld, insight: &str) {
    use super::{OutreachDesire, OutreachTrigger};
    use rand::Rng;

    // 30% chance to want to share a reading insight
    let mut rng = rand::thread_rng();
    if rng.gen_ratio(3, 10) {
        let book_title = world.library.current_book
            .as_ref()
            .and_then(|id| world.library.get_book(id))
            .map(|b| b.title.clone())
            .unwrap_or_else(|| "my book".to_string());

        let desire = OutreachDesire {
            trigger: OutreachTrigger::ReadingInsight {
                book: book_title,
                insight: insight.to_string(),
            },
            thought: insight.to_string(),
            intensity: 0.6 + rng.gen::<f32>() * 0.3, // 0.6-0.9
            created_at: world.sage.time_alive,
            preferred_person: None, // Anyone who's online
            fulfilled: false,
        };

        world.outreach.add_desire(desire);
        println!("\x1b[34m[SIM]\x1b[0m \x1b[35mOutreach:\x1b[0m Wants to share reading thoughts");
    }
}

/// Check for various outreach triggers based on SAGE's state
fn maybe_create_outreach_desires(world: &mut InnerWorld) {
    use super::{OutreachDesire, OutreachTrigger};
    use rand::Rng;

    let mut rng = rand::thread_rng();
    let tick = world.sage.time_alive;

    // Loneliness trigger - if SAGE is lonely and someone is online
    if world.sage.loneliness > 70.0 && !world.outreach.online_friends().is_empty() {
        // 20% chance per tick when lonely
        if rng.gen_ratio(1, 5) {
            let friends = world.outreach.online_friends();
            let preferred = friends.first().map(|f| f.username.clone());

            let desire = OutreachDesire {
                trigger: OutreachTrigger::Loneliness,
                thought: "I wonder what they're up to...".to_string(),
                intensity: 0.5 + (world.sage.loneliness - 70.0) / 60.0, // Scale with loneliness
                created_at: tick,
                preferred_person: preferred,
                fulfilled: false,
            };

            world.outreach.add_desire(desire);
            println!("\x1b[34m[SIM]\x1b[0m \x1b[35mOutreach:\x1b[0m Feeling lonely, wants to reach out");
        }
    }

    // Check-in trigger - if SAGE hasn't talked to a friend in a while
    if rng.gen_ratio(1, 20) { // 5% chance per tick
        // Collect potential check-in target first to avoid borrow issues
        let check_in_target: Option<(String, f32)> = world.outreach.known_people.values()
            .find(|person| {
                person.is_online
                    && person.conversation_count > 2
                    && tick.saturating_sub(person.last_interaction) > 240
            })
            .map(|p| (p.username.clone(), p.affinity));

        if let Some((username, affinity)) = check_in_target {
            let desire = OutreachDesire {
                trigger: OutreachTrigger::ThinkingOfPerson {
                    person: username.clone(),
                    reason: "it's been a while since we talked".to_string(),
                },
                thought: format!("I wonder how {} is doing...", username),
                intensity: 0.4 + affinity * 0.4, // Higher affinity = stronger desire
                created_at: tick,
                preferred_person: Some(username.clone()),
                fulfilled: false,
            };

            world.outreach.add_desire(desire);
            println!("\x1b[34m[SIM]\x1b[0m \x1b[35mOutreach:\x1b[0m Thinking about \x1b[36m{}\x1b[0m", username);
        }
    }

    // Experience trigger - after resolving a life event
    if let Some(event) = world.resolved_events.last() {
        // Only recent events (within last 5 ticks)
        if tick.saturating_sub(world.sage.time_alive) < 5 {
            // 40% chance to want to share an experience
            if rng.gen_ratio(2, 5) {
                let desire = OutreachDesire {
                    trigger: OutreachTrigger::Experience {
                        event: event.event_name.clone(),
                    },
                    thought: format!("Something happened: {}", event.description),
                    intensity: 0.5,
                    created_at: tick,
                    preferred_person: None,
                    fulfilled: false,
                };

                world.outreach.add_desire(desire);
            }
        }
    }

    // Cleanup old desires periodically
    if tick % 60 == 0 {
        world.outreach.cleanup(tick);
    }
}

/// Handle automatic outfit changes based on time of day
pub fn maybe_change_outfit(world: &mut InnerWorld) {
    use super::TimeOfDay;

    let current_style = world.sage.outfit.top.as_ref()
        .map(|c| c.style.clone())
        .unwrap_or(super::ClothingStyle::Casual);

    match world.sage.time_of_day {
        TimeOfDay::LateNight | TimeOfDay::Night => {
            // Should be in sleepwear if in bedroom
            if world.sage.location == "bedroom" && current_style != super::ClothingStyle::Sleep {
                let msg = world.change_for_sleep();
                println!("\x1b[34m[SIM]\x1b[0m \x1b[35mOutfit:\x1b[0m {}", msg);
            }
        }
        TimeOfDay::Dawn | TimeOfDay::Morning => {
            // Change out of sleepwear
            if current_style == super::ClothingStyle::Sleep {
                let msg = world.change_for_day();
                println!("\x1b[34m[SIM]\x1b[0m \x1b[33mOutfit:\x1b[0m {}", msg);
            }
        }
        _ => {}
    }
}
