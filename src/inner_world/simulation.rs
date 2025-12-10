//! Inner World Simulation Loop
//!
//! Runs SAGE's inner life in the background, making decisions via LLM,
//! experiencing events, and building a rich inner history.

use super::{InnerWorld, events::LifeEvent, OutreachDesire, OutreachTrigger};
use crate::llm_client::LlmClient;
use crate::embeddings::SemanticMemory;

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
}

/// Run one step of the inner world simulation
pub async fn run_simulation_step(
    world: &mut InnerWorld,
    llm: &LlmClient,
    mut semantic_memory: Option<&mut SemanticMemory>,
) -> SimulationStep {
    // Advance time
    world.tick();

    // Handle automatic outfit changes based on time of day
    maybe_change_outfit(world);

    // 1. Perception - what SAGE currently experiences
    let perception = world.describe_current_state();

    // 2. Check for random life events
    let mut event_narrative = None;
    let mut lesson = None;

    if let Some(event) = world.maybe_trigger_event() {
        // Let the LLM choose how to respond to the event
        let choice_idx = choose_event_response(llm, world, &event).await;
        let narrative = world.resolve_event(&event, choice_idx);

        // Extract lesson if one was learned
        if let Some(last_event) = world.resolved_events.last() {
            lesson = last_event.lesson_learned.clone();
        }

        event_narrative = Some(narrative);
    }

    // 3. Generate SAGE's inner thought
    let thought = generate_inner_thought(llm, world, &perception, event_narrative.as_deref()).await;

    // 4. Decide on an action
    let available_actions = world.available_actions();
    let chosen_action = choose_action(llm, world, &perception, &thought, &available_actions).await;

    // 5. Execute the action
    let result = world.execute_action(&chosen_action);

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
                    println!("📖 Reading insight: {}", insight_text);
                }

                // Maybe SAGE wants to share this insight with someone!
                maybe_create_reading_outreach(world, &insight_text);
            }
        }
    }

    // 5c. Check for other outreach triggers (loneliness, etc.)
    maybe_create_outreach_desires(world);

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

    SimulationStep {
        tick: world.sage.time_alive,
        perception,
        thought,
        action_taken: chosen_action,
        action_result: result.message,
        event_occurred: event_narrative,
        lesson_learned: lesson,
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
                println!("💧 SAGE is thirsty ({:.0}%), getting water...", world.sage.thirst);
                return action.clone();
            }
        } else if world.sage.location == "bathroom" {
            if let Some(action) = available_actions.iter().find(|a| a.contains("drink")) {
                println!("💧 SAGE is thirsty ({:.0}%), drinking from sink...", world.sage.thirst);
                return action.clone();
            }
        } else {
            // Go to kitchen for water
            if let Some(action) = available_actions.iter().find(|a| a.contains("go") && a.contains("kitchen")) {
                println!("💧 SAGE is thirsty ({:.0}%), heading to kitchen...", world.sage.thirst);
                return action.clone();
            }
            if let Some(action) = available_actions.iter().find(|a| a.contains("go") && a.contains("east")) {
                return action.clone();
            }
        }
    }

    // 2. HUNGER - needs food
    if world.sage.hunger > 60.0 {
        // Very hungry - go to kitchen or eat
        if world.sage.location == "kitchen" {
            // In kitchen - eat!
            if let Some(action) = available_actions.iter().find(|a| a.contains("food") || a.contains("eat") || a.contains("cook") || a.contains("snack")) {
                println!("🍽️  SAGE is hungry ({:.0}%), getting food...", world.sage.hunger);
                return action.clone();
            }
        } else {
            if let Some(action) = available_actions.iter().find(|a| a.contains("go") && a.contains("kitchen")) {
                println!("🍽️  SAGE is hungry ({:.0}%), heading toward food...", world.sage.hunger);
                return action.clone();
            }
            if let Some(action) = available_actions.iter().find(|a| a.contains("go") && a.contains("east")) {
                return action.clone();
            }
        }
    }

    // 3. ENERGY - needs rest
    if world.sage.energy < 30.0 {
        // Very tired - rest or sleep
        if let Some(action) = available_actions.iter().find(|a| a.contains("sleep") || a.contains("rest") || a.contains("nap")) {
            println!("😴 SAGE is tired ({:.0}% energy), resting...", world.sage.energy);
            return action.clone();
        }
        // Go to bedroom if not there
        if world.sage.location != "bedroom" {
            if let Some(action) = available_actions.iter().find(|a| a.contains("go") && a.contains("bedroom")) {
                println!("😴 SAGE is tired ({:.0}% energy), heading to bedroom...", world.sage.energy);
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
                println!("🚿 SAGE needs to freshen up ({:.0}% hygiene)...", world.sage.hygiene);
                return action.clone();
            }
        } else {
            // Go to bathroom
            if let Some(action) = available_actions.iter().find(|a| a.contains("go") && a.contains("bathroom")) {
                println!("🚿 SAGE needs to freshen up ({:.0}% hygiene), heading to bathroom...", world.sage.hygiene);
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
            println!("🏃 SAGE is feeling restless ({:.0}%), needs to move...", world.sage.restlessness);
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
            println!("🎨 SAGE is feeling creative ({:.0}%)...", world.sage.creative_urge);
            return action.clone();
        }
    }

    // 7. BOREDOM - needs variety
    if world.sage.boredom > 70.0 {
        if let Some(action) = available_actions.iter().find(|a|
            a.contains("read") || a.contains("browse") || a.contains("look")
        ) {
            println!("😐 SAGE is bored ({:.0}%), looking for something to do...", world.sage.boredom);
            return action.clone();
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
                    return action.clone();
                }
            }

            // Try to parse as a number
            if let Ok(num) = response.parse::<usize>() {
                if num > 0 && num <= available_actions.len() {
                    return available_actions[num - 1].clone();
                }
            }

            // Default to first action or wait
            available_actions.iter()
                .find(|a| a.contains("wait"))
                .cloned()
                .unwrap_or_else(|| available_actions.first().cloned().unwrap_or_else(|| "wait".to_string()))
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

/// Format an inner world experience for inclusion in Discord responses
pub fn format_inner_experience_for_chat(world: &InnerWorld) -> String {
    let mut context = String::new();

    // Physical presence and appearance
    context.push_str(&format!(
        "[Inner world: Day {}, {} ({}). SAGE is in the {}, wearing {}. Feeling {}.]\n",
        world.sage.day,
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
                        // Include insights from this book
                        if !progress.insights.is_empty() {
                            let insights: Vec<_> = progress.insights.iter().take(3).collect();
                            context.push_str(&format!("[Insights from this book: {}]\n", insights.iter().map(|s| s.as_str()).collect::<Vec<_>>().join("; ")));
                        }
                    } else {
                        context.push_str(&format!(
                            "[Currently reading: \"{}\" by {} (page {}/{})]\n",
                            book.title, book.author, progress.current_page + 1, book.total_pages()
                        ));
                    }
                }
            }
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
        r#"SAGE wants to reach out to {} because: {}

{}

SAGE's current mood: {}
Time of day: {}

Write a SHORT, casual message (1-2 sentences) that SAGE would send to start a conversation.
It should feel natural, like texting a friend - not forced or formal.
Don't start with "Hey" every time - vary the opener.
Don't explain why you're reaching out too directly - just start the conversation naturally.

Message:"#,
        target_person,
        trigger_desc,
        person_context,
        world.sage.mood.as_str(),
        world.sage.time_of_day.as_str()
    );

    match llm.generate_raw(&prompt).await {
        Ok(msg) => {
            let msg = msg.trim();
            let msg = msg.trim_matches('"');
            let msg = msg.trim();

            // Validate message quality
            if msg.len() > 10 && msg.len() < 300 && !msg.contains("SAGE") {
                Some(msg.to_string())
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
        println!("💭 SAGE wants to share what they're reading...");
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
            println!("💭 SAGE is feeling lonely and wants to reach out...");
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
            println!("💭 SAGE is thinking about {}...", username);
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
                println!("🌙 {}", msg);
            }
        }
        TimeOfDay::Dawn | TimeOfDay::Morning => {
            // Change out of sleepwear
            if current_style == super::ClothingStyle::Sleep {
                let msg = world.change_for_day();
                println!("☀️ {}", msg);
            }
        }
        _ => {}
    }
}
