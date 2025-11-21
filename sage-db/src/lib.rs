use spacetimedb::{ReducerContext, Table, Timestamp};

/// Current state of SAGE - single row that gets updated
#[spacetimedb::table(name = sage_state, public)]
pub struct SageState {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    pub generation: u64,
    pub current_loss: f64,
    pub current_pattern: String,
    pub complexity: f64,
    pub diversity: f64,
    pub is_training: bool,
    pub updated_at: Timestamp,
}

/// Historical metrics - time series data
#[spacetimedb::table(name = training_metrics, public)]
pub struct TrainingMetrics {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    pub generation: u64,
    pub loss: f64,
    pub complexity: f64,
    pub diversity: f64,
    pub pattern: String,
    pub timestamp: Timestamp,
}

/// Network snapshots - saved at key moments
#[spacetimedb::table(name = network_snapshots, public)]
pub struct NetworkSnapshot {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    pub generation: u64,
    pub pattern: String,
    pub loss: f64,
    pub weights_json: String,  // Serialized network weights
    pub timestamp: Timestamp,
}

/// Conversation history with SAGE
#[spacetimedb::table(name = conversations, public)]
pub struct Conversation {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    pub sender: String,
    pub message: String,
    pub sage_response: String,  // SAGE's reply
    pub nca_loss: f64,  // How well SAGE understood this
    pub concepts_extracted: String,  // JSON array of concepts
    pub generation_context: u64,  // What generation was SAGE at?
    pub timestamp: Timestamp,
}

/// Pattern mastery tracking
#[spacetimedb::table(name = pattern_progress, public)]
pub struct PatternProgress {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    pub pattern: String,
    pub start_generation: u64,
    pub mastered_generation: Option<u64>,
    pub best_loss: f64,
    pub is_mastered: bool,
    pub started_at: Timestamp,
    pub mastered_at: Option<Timestamp>,
}

/// Events log - significant training milestones
#[spacetimedb::table(name = training_events, public)]
pub struct TrainingEvent {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    pub generation: u64,
    pub event_type: String,  // "pattern_start", "pattern_mastered", "milestone", "error"
    pub description: String,
    pub timestamp: Timestamp,
}

/// SAGE's concept memory - tracks understanding of concepts over time
#[spacetimedb::table(name = concept_memory, public)]
pub struct ConceptMemory {
    #[primary_key]
    pub concept_name: String,  // Use concept as primary key
    pub nca_encoding: String,  // Serialized grid pattern as JSON
    pub familiarity_score: f64,  // 0.0 = new, 1.0 = very familiar
    pub average_loss: f64,  // Average NCA loss across experiences
    pub exposure_count: u64,  // How many times seen
    pub opinion_type: String,  // "Like", "Dislike", "Curious", "Neutral"
    pub opinion_reason: String,  // Why SAGE feels this way
    pub confidence: f64,  // How confident in opinion
    pub related_concepts: String,  // JSON array of related concept names
    pub first_seen: Timestamp,
    pub last_seen: Timestamp,
}

/// Opinion history - track how opinions change over time
#[spacetimedb::table(name = opinion_history, public)]
pub struct OpinionHistory {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    pub concept: String,
    pub opinion_type: String,  // "Like", "Dislike", "Curious", "Neutral"
    pub reason: String,
    pub confidence: f64,
    pub nca_loss: f64,
    pub generation: u64,
    pub timestamp: Timestamp,
}

/// Personality snapshots - track personality evolution
#[spacetimedb::table(name = personality_snapshots, public)]
pub struct PersonalitySnapshot {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    pub generation: u64,
    pub openness: f64,
    pub positivity: f64,
    pub curiosity: f64,
    pub confidence: f64,
    pub experience_count: u64,
    pub likes_count: u64,
    pub dislikes_count: u64,
    pub timestamp: Timestamp,
}

/// Introspection journal - SAGE's subjective experience over time
#[spacetimedb::table(name = introspection_journal, public)]
pub struct IntrospectionJournal {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    pub experience_count: u64,
    pub valence: f64,  // -1.0 to 1.0 (emotional tone)
    pub intensity: f64,  // 0.0 to 1.0 (experience strength)
    pub complexity: f64,  // 0.0 to 1.0 (mental richness)
    pub feeling_name: String,  // Named emotion (e.g., "contentment", "fascination")
    pub mode: String,  // Cognitive mode (e.g., "quiet reflection", "deep processing")
    pub qualities: String,  // JSON array of quality descriptors
    pub active_concepts: String,  // JSON array of concepts drawing attention
    pub description: String,  // Natural language description
    pub temporal_context: String,  // Comparison to previous states
    pub trigger: String,  // What triggered this introspection ("command", "autonomous", "conversation")
    pub timestamp: Timestamp,
}

/// Autonomous activity - SAGE's inner life when alone
#[spacetimedb::table(name = autonomous_activity, public)]
pub struct AutonomousActivity {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    pub activity_type: String,  // "dream" or "curiosity"
    pub experience_count: u64,
    pub seconds_idle: u64,  // How long idle before this activity
    pub concepts_deepened: String,  // JSON array of concepts strengthened
    pub concepts_consolidated: String,  // JSON array of concepts consolidated
    pub links_formed: String,  // JSON array of new associations
    pub question_generated: String,  // For curiosity mode
    pub exploration_notes: String,  // What SAGE thought about
    pub timestamp: Timestamp,
}

/// A/B test results - Comparing NCA-enhanced vs baseline responses
#[spacetimedb::table(name = ab_test_results, public)]
pub struct ABTestResult {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    pub experience_count: u64,
    pub input_message: String,
    pub nca_response: String,  // Response WITH NCA memory
    pub baseline_response: String,  // Response WITHOUT NCA memory
    pub nca_opinion: String,  // What SAGE's NCA thought about the input
    pub user_preference: String,  // Which response user preferred (if known)
    pub avg_alpha: f64,  // Average NCA grid activation
    pub timestamp: Timestamp,
}

/// Visual memories - What SAGE has seen and learned visually
#[spacetimedb::table(name = visual_memories, public)]
pub struct VisualMemory {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    pub experience_count: u64,
    pub person_name: String,  // Who SAGE was looking at
    pub avg_brightness: f64,  // 0.0-1.0
    pub avg_r: f64,  // Average red channel
    pub avg_g: f64,  // Average green channel
    pub avg_b: f64,  // Average blue channel
    pub color_variance: f64,  // Color diversity in image
    pub dominant_color: String,  // "red", "green", "blue", "neutral"
    pub edge_strength: f64,  // Detail level
    pub natural_description: String,  // What SAGE perceived
    pub visual_concepts: String,  // JSON array of visual concepts experienced
    pub context: String,  // What was happening during this visual experience
    pub timestamp: Timestamp,
}

#[spacetimedb::reducer(init)]
pub fn init(ctx: &ReducerContext) {
    // Initialize SAGE state
    ctx.db.sage_state().insert(SageState {
        id: 0,
        generation: 0,
        current_loss: 1.0,
        current_pattern: "🔴 Circle".to_string(),
        complexity: 0.0,
        diversity: 0.0,
        is_training: false,
        updated_at: ctx.timestamp,
    });

    log::info!("🧬 SAGE database initialized");
}

#[spacetimedb::reducer(client_connected)]
pub fn client_connected(_ctx: &ReducerContext) {
    log::info!("👁️  New observer connected to SAGE");
}

#[spacetimedb::reducer(client_disconnected)]
pub fn client_disconnected(_ctx: &ReducerContext) {
    log::info!("👋 Observer disconnected from SAGE");
}

/// Update SAGE's current state (called frequently during training)
#[spacetimedb::reducer]
pub fn update_sage_state(
    ctx: &ReducerContext,
    generation: u64,
    loss: f64,
    pattern: String,
    complexity: f64,
    diversity: f64,
) {
    // Delete old state and insert new (simpler than update)
    for state in ctx.db.sage_state().iter() {
        ctx.db.sage_state().delete(state);
    }

    ctx.db.sage_state().insert(SageState {
        id: 0,
        generation,
        current_loss: loss,
        current_pattern: pattern.clone(),
        complexity,
        diversity,
        is_training: true,
        updated_at: ctx.timestamp,
    });

    // Record metrics every generation for real-time history
    ctx.db.training_metrics().insert(TrainingMetrics {
        id: 0,
        generation,
        loss,
        complexity,
        diversity,
        pattern,
        timestamp: ctx.timestamp,
    });
}

/// Save network weights snapshot
#[spacetimedb::reducer]
pub fn save_network_snapshot(
    ctx: &ReducerContext,
    generation: u64,
    pattern: String,
    loss: f64,
    weights_json: String,
) {
    ctx.db.network_snapshots().insert(NetworkSnapshot {
        id: 0,
        generation,
        pattern,
        loss,
        weights_json,
        timestamp: ctx.timestamp,
    });

    log::info!("💾 Network snapshot saved at generation {}", generation);
}

/// Record a conversation message with SAGE's response
#[spacetimedb::reducer]
pub fn add_conversation_message(
    ctx: &ReducerContext,
    sender: String,
    message: String,
    sage_response: String,
    nca_loss: f64,
    concepts_extracted: String,
    generation_context: u64,
) {
    ctx.db.conversations().insert(Conversation {
        id: 0,
        sender: sender.clone(),
        message: message.clone(),
        sage_response: sage_response.clone(),
        nca_loss,
        concepts_extracted,
        generation_context,
        timestamp: ctx.timestamp,
    });

    log::info!("💬 {} said: {} | SAGE: {}", sender, message, sage_response);
}

/// Start learning a new pattern
#[spacetimedb::reducer]
pub fn start_pattern(
    ctx: &ReducerContext,
    pattern: String,
    generation: u64,
) {
    ctx.db.pattern_progress().insert(PatternProgress {
        id: 0,
        pattern: pattern.clone(),
        start_generation: generation,
        mastered_generation: None,
        best_loss: 1.0,
        is_mastered: false,
        started_at: ctx.timestamp,
        mastered_at: None,
    });

    ctx.db.training_events().insert(TrainingEvent {
        id: 0,
        generation,
        event_type: "pattern_start".to_string(),
        description: format!("Started learning {}", pattern),
        timestamp: ctx.timestamp,
    });
}

/// Mark pattern as mastered
#[spacetimedb::reducer]
pub fn master_pattern(
    ctx: &ReducerContext,
    pattern: String,
    generation: u64,
    final_loss: f64,
) {
    ctx.db.training_events().insert(TrainingEvent {
        id: 0,
        generation,
        event_type: "pattern_mastered".to_string(),
        description: format!("🎯 Mastered {} with loss {:.4}", pattern, final_loss),
        timestamp: ctx.timestamp,
    });

    log::info!("🎯 Pattern {} mastered at generation {}", pattern, generation);
}

/// Log a training event
#[spacetimedb::reducer]
pub fn log_training_event(
    ctx: &ReducerContext,
    generation: u64,
    event_type: String,
    description: String,
) {
    ctx.db.training_events().insert(TrainingEvent {
        id: 0,
        generation,
        event_type,
        description: description.clone(),
        timestamp: ctx.timestamp,
    });
}

/// Set training status
#[spacetimedb::reducer]
pub fn set_training_status(
    _ctx: &ReducerContext,
    is_training: bool,
) {
    // Just log for now - training status updated in update_sage_state
    log::info!("Training status: {}", if is_training { "Started" } else { "Paused" });
}

/// Query: Get recent metrics (last N records)
#[spacetimedb::reducer]
pub fn get_recent_metrics(ctx: &ReducerContext, _limit: u32) {
    let count = ctx.db.training_metrics().iter().count();
    log::info!("📊 Total metric points: {}", count);
}

/// Store or update a concept in SAGE's memory
#[spacetimedb::reducer]
pub fn store_concept_memory(
    ctx: &ReducerContext,
    concept_name: String,
    nca_encoding: String,
    familiarity_score: f64,
    average_loss: f64,
    exposure_count: u64,
    opinion_type: String,
    opinion_reason: String,
    confidence: f64,
    related_concepts: String,
) {
    // Check if concept already exists
    let existing = ctx.db.concept_memory()
        .iter()
        .find(|c| c.concept_name == concept_name);

    if let Some(old) = existing {
        // Save the first_seen timestamp before deletion
        let first_seen_timestamp = old.first_seen;

        // Delete old entry
        ctx.db.concept_memory().delete(old);

        // Insert updated version (keeping first_seen from before)
        ctx.db.concept_memory().insert(ConceptMemory {
            concept_name: concept_name.clone(),
            nca_encoding,
            familiarity_score,
            average_loss,
            exposure_count,
            opinion_type,
            opinion_reason,
            confidence,
            related_concepts,
            first_seen: first_seen_timestamp,  // Preserve original timestamp
            last_seen: ctx.timestamp,
        });

        log::info!("🧠 Updated concept memory: {} (exposure: {})", concept_name, exposure_count);
    } else {
        // Insert new concept
        ctx.db.concept_memory().insert(ConceptMemory {
            concept_name: concept_name.clone(),
            nca_encoding,
            familiarity_score,
            average_loss,
            exposure_count,
            opinion_type,
            opinion_reason,
            confidence,
            related_concepts,
            first_seen: ctx.timestamp,
            last_seen: ctx.timestamp,
        });

        log::info!("🧠 New concept learned: {}", concept_name);
    }
}

/// Record an opinion in history
#[spacetimedb::reducer]
pub fn record_opinion(
    ctx: &ReducerContext,
    concept: String,
    opinion_type: String,
    reason: String,
    confidence: f64,
    nca_loss: f64,
    generation: u64,
) {
    ctx.db.opinion_history().insert(OpinionHistory {
        id: 0,
        concept,
        opinion_type,
        reason,
        confidence,
        nca_loss,
        generation,
        timestamp: ctx.timestamp,
    });
}

/// Save a snapshot of SAGE's personality
#[spacetimedb::reducer]
pub fn save_personality_snapshot(
    ctx: &ReducerContext,
    generation: u64,
    openness: f64,
    positivity: f64,
    curiosity: f64,
    confidence: f64,
    experience_count: u64,
    likes_count: u64,
    dislikes_count: u64,
) {
    ctx.db.personality_snapshots().insert(PersonalitySnapshot {
        id: 0,
        generation,
        openness,
        positivity,
        curiosity,
        confidence,
        experience_count,
        likes_count,
        dislikes_count,
        timestamp: ctx.timestamp,
    });

    log::info!("🎭 Personality snapshot saved: Gen {}, Experience {}", generation, experience_count);
}

/// Save an introspection report - SAGE's subjective experience
#[spacetimedb::reducer]
pub fn save_introspection(
    ctx: &ReducerContext,
    experience_count: u64,
    valence: f64,
    intensity: f64,
    complexity: f64,
    feeling_name: String,
    mode: String,
    qualities: String,  // JSON array
    active_concepts: String,  // JSON array
    description: String,
    temporal_context: String,
    trigger: String,
) {
    ctx.db.introspection_journal().insert(IntrospectionJournal {
        id: 0,
        experience_count,
        valence,
        intensity,
        complexity,
        feeling_name: feeling_name.clone(),
        mode: mode.clone(),
        qualities,
        active_concepts,
        description,
        temporal_context,
        trigger: trigger.clone(),
        timestamp: ctx.timestamp,
    });

    log::info!("🧠 Introspection logged: {} ({}) - trigger: {}", feeling_name, mode, trigger);
}

/// Log autonomous activity - Dream Mode or Curiosity Mode
#[spacetimedb::reducer]
pub fn log_autonomous_activity(
    ctx: &ReducerContext,
    activity_type: String,
    experience_count: u64,
    seconds_idle: u64,
    concepts_deepened: String,
    concepts_consolidated: String,
    links_formed: String,
    question_generated: String,
    exploration_notes: String,
) {
    ctx.db.autonomous_activity().insert(AutonomousActivity {
        id: 0,
        activity_type: activity_type.clone(),
        experience_count,
        seconds_idle,
        concepts_deepened,
        concepts_consolidated,
        links_formed,
        question_generated,
        exploration_notes,
        timestamp: ctx.timestamp,
    });

    log::info!("🌟 Autonomous {} logged after {}s idle", activity_type, seconds_idle);
}

/// Record A/B test result
#[spacetimedb::reducer]
pub fn record_ab_test(
    ctx: &ReducerContext,
    experience_count: u64,
    input_message: String,
    nca_response: String,
    baseline_response: String,
    nca_opinion: String,
    user_preference: String,
    avg_alpha: f64,
) {
    ctx.db.ab_test_results().insert(ABTestResult {
        id: 0,
        experience_count,
        input_message: input_message.clone(),
        nca_response,
        baseline_response,
        nca_opinion,
        user_preference,
        avg_alpha,
        timestamp: ctx.timestamp,
    });

    log::info!("🧪 A/B test recorded: {} (α={:.3})", input_message.chars().take(30).collect::<String>(), avg_alpha);
}

/// Save a visual memory - what SAGE saw and learned
#[spacetimedb::reducer]
pub fn save_visual_memory(
    ctx: &ReducerContext,
    experience_count: u64,
    person_name: String,
    avg_brightness: f64,
    avg_r: f64,
    avg_g: f64,
    avg_b: f64,
    color_variance: f64,
    dominant_color: String,
    edge_strength: f64,
    natural_description: String,
    visual_concepts: String,
    context: String,
) {
    ctx.db.visual_memories().insert(VisualMemory {
        id: 0,
        experience_count,
        person_name: person_name.clone(),
        avg_brightness,
        avg_r,
        avg_g,
        avg_b,
        color_variance,
        dominant_color: dominant_color.clone(),
        edge_strength,
        natural_description,
        visual_concepts,
        context,
        timestamp: ctx.timestamp,
    });

    log::info!("👁️  Visual memory saved: SAGE saw {} (brightness: {:.2}, color: {})",
        person_name, avg_brightness, dominant_color);
}

/// Query recent conversations for a specific user
#[spacetimedb::reducer]
pub fn get_user_conversations(
    ctx: &ReducerContext,
    sender: String,
    limit: u32,
) {
    let conversations: Vec<_> = ctx.db.conversations()
        .iter()
        .filter(|c| c.sender == sender)
        .collect();

    let count = conversations.len().min(limit as usize);
    let recent: Vec<_> = conversations.iter().rev().take(count).collect();

    log::info!("📚 Retrieved {} conversations for {}", recent.len(), sender);

    // Print conversations in a parseable format (JSON-like)
    for conv in recent.iter().rev() {
        log::info!("CONVERSATION_DATA|{}|{}|{}",
            conv.sender,
            conv.message.replace("|", "\\|"),
            conv.sage_response.replace("|", "\\|")
        );
    }
}

// ============================================================================
// USER MEMORY SYSTEM - Facts and Messages
// ============================================================================

/// Individual chat messages (pre-response) for building conversation context
#[spacetimedb::table(name = messages, public)]
pub struct Message {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    pub user_id: String,  // Discord username, IRC nick, etc.
    pub text: String,
    pub platform: String,  // "discord", "irc", "cli"
    pub timestamp: Timestamp,
}

/// User facts - structured knowledge about each user
#[spacetimedb::table(name = user_facts, public)]
pub struct UserFact {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    pub user_id: String,
    pub fact_key: String,  // "name", "wife_name", "son_name", etc.
    pub value: String,
    pub confidence: f64,  // 0.0-1.0
    pub last_mentioned: Timestamp,
    pub mention_count: u32,
}

/// Store or update a user fact with validation
#[spacetimedb::reducer]
pub fn store_user_fact(
    ctx: &ReducerContext,
    user_id: String,
    fact_key: String,
    value: String,
    confidence: f64,
) -> Result<(), String> {
    // VALIDATION
    if user_id.is_empty() {
        return Err("User ID cannot be empty".to_string());
    }

    if fact_key.is_empty() {
        return Err("Fact key cannot be empty".to_string());
    }

    if value.is_empty() {
        return Err("Fact value cannot be empty".to_string());
    }

    if !(0.0..=1.0).contains(&confidence) {
        return Err("Confidence must be between 0 and 1".to_string());
    }

    // Validate relationship fact keys
    let valid_keys = [
        "name", "japanese_name", "wife_name", "husband_name",
        "son_name", "daughter_name", "child_name",
        "preference:", "detail:", "fact:"
    ];

    let is_valid_key = valid_keys.iter().any(|&k| {
        fact_key == k || fact_key.starts_with(k)
    });

    if !is_valid_key {
        return Err(format!("Invalid fact key: {}. Must be one of: name, wife_name, son_name, preference:*, detail:*, fact:*", fact_key));
    }

    // Check for existing fact with same key for this user
    let existing = ctx.db.user_facts()
        .iter()
        .find(|f| f.user_id == user_id && f.fact_key == fact_key);

    if let Some(old_fact) = existing {
        // Update existing fact
        let new_confidence = old_fact.confidence.max(confidence);
        let new_mention_count = old_fact.mention_count + 1;

        // Delete old entry
        ctx.db.user_facts().delete(old_fact);

        // Insert updated version
        ctx.db.user_facts().insert(UserFact {
            id: 0,
            user_id: user_id.clone(),
            fact_key: fact_key.clone(),
            value: value.clone(),
            confidence: new_confidence,
            last_mentioned: ctx.timestamp,
            mention_count: new_mention_count,
        });

        log::info!("📝 Updated fact for {}: {} = {} (confidence: {:.2}, mentions: {})",
            user_id, fact_key, value, new_confidence, new_mention_count);
    } else {
        // Insert new fact
        ctx.db.user_facts().insert(UserFact {
            id: 0,
            user_id: user_id.clone(),
            fact_key: fact_key.clone(),
            value: value.clone(),
            confidence,
            last_mentioned: ctx.timestamp,
            mention_count: 1,
        });

        log::info!("✨ New fact for {}: {} = {} (confidence: {:.2})",
            user_id, fact_key, value, confidence);
    }

    Ok(())
}

/// Add a message to the chat history
#[spacetimedb::reducer]
pub fn add_message(
    ctx: &ReducerContext,
    user_id: String,
    text: String,
    platform: String,
) -> Result<(), String> {
    if user_id.is_empty() {
        return Err("User ID cannot be empty".to_string());
    }

    if text.is_empty() {
        return Err("Message text cannot be empty".to_string());
    }

    ctx.db.messages().insert(Message {
        id: 0,
        user_id: user_id.clone(),
        text: text.clone(),
        platform,
        timestamp: ctx.timestamp,
    });

    log::info!("💬 Message from {}: {}", user_id, text.chars().take(50).collect::<String>());

    Ok(())
}

/// Batch store multiple facts (for initial seeding or fact extraction)
#[spacetimedb::reducer]
pub fn batch_store_facts(
    ctx: &ReducerContext,
    user_id: String,
    facts_json: String,  // JSON array: [{"key": "name", "value": "Cary", "confidence": 1.0}, ...]
) -> Result<(), String> {
    // Parse JSON (simplified - in production use serde_json)
    // For now, accept pipe-delimited format: "key1|value1|conf1;key2|value2|conf2"

    for fact_str in facts_json.split(';') {
        let parts: Vec<&str> = fact_str.split('|').collect();
        if parts.len() == 3 {
            if let Ok(confidence) = parts[2].parse::<f64>() {
                store_user_fact(ctx, user_id.clone(), parts[0].to_string(), parts[1].to_string(), confidence)?;
            }
        }
    }

    Ok(())
}

/// Get fact count for a user (for debugging)
#[spacetimedb::reducer]
pub fn get_fact_count(ctx: &ReducerContext, user_id: String) {
    let count = ctx.db.user_facts()
        .iter()
        .filter(|f| f.user_id == user_id)
        .count();

    log::info!("📊 User {} has {} facts stored", user_id, count);
}

/// Delete a specific fact (for corrections)
#[spacetimedb::reducer]
pub fn delete_user_fact(
    ctx: &ReducerContext,
    user_id: String,
    fact_key: String,
) -> Result<(), String> {
    let fact = ctx.db.user_facts()
        .iter()
        .find(|f| f.user_id == user_id && f.fact_key == fact_key);

    if let Some(f) = fact {
        ctx.db.user_facts().delete(f);
        log::info!("🗑️  Deleted fact for {}: {}", user_id, fact_key);
        Ok(())
    } else {
        Err(format!("Fact not found: {} for user {}", fact_key, user_id))
    }
}
