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

// ============================================================================
// META-LEARNING SYSTEM TABLES
// ============================================================================

/// Meta-learning strategy decisions - tracks when/why SAGE switches strategies
#[spacetimedb::table(name = meta_strategy_history, public)]
pub struct MetaStrategyHistory {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    pub generation: u64,
    pub strategy_name: String,        // "aggressive", "conservative", "exploratory", etc.
    pub reason: String,               // Why this strategy was chosen
    pub performance_before: f64,      // Loss before strategy change
    pub performance_after: Option<f64>, // Loss after (filled in later)
    pub duration_generations: Option<u64>, // How long this strategy was used
    pub timestamp: Timestamp,
}

/// Hyperparameter evolution from Population-Based Training (PBT)
#[spacetimedb::table(name = hyperparameter_evolution, public)]
pub struct HyperparameterEvolution {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    pub generation: u64,
    pub learning_rate: f64,
    pub batch_size: u32,
    pub evolution_steps: u32,
    pub mutation_rate: f64,
    pub fitness_score: f64,           // How well these params performed
    pub parent_id: Option<u64>,       // Which config this mutated from
    pub is_elite: bool,               // Was this config in the top performers?
    pub timestamp: Timestamp,
}

/// Architecture self-modification history
#[spacetimedb::table(name = architecture_changes, public)]
pub struct ArchitectureChange {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    pub generation: u64,
    pub change_type: String,          // "grow", "prune", "restructure"
    pub layer_affected: String,       // Which layer changed
    pub old_size: u32,
    pub new_size: u32,
    pub trigger: String,              // "performance_plateau", "complexity_increase", etc.
    pub loss_before: f64,
    pub loss_after: Option<f64>,
    pub timestamp: Timestamp,
}

/// Few-shot adaptation results from Reptile meta-learning
#[spacetimedb::table(name = few_shot_adaptations, public)]
pub struct FewShotAdaptation {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    pub generation: u64,
    pub task_name: String,            // Pattern or task adapted to
    pub shots_used: u32,              // How many examples
    pub adaptation_steps: u32,
    pub loss_before: f64,
    pub loss_after: f64,
    pub transfer_source: String,      // What prior knowledge helped
    pub adaptation_time_ms: u64,      // How long adaptation took
    pub timestamp: Timestamp,
}

/// Learned optimizer state snapshots (L2O)
#[spacetimedb::table(name = optimizer_snapshots, public)]
pub struct OptimizerSnapshot {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    pub generation: u64,
    pub optimizer_type: String,       // "adam", "learned", "reptile", etc.
    pub optimizer_weights_json: String,  // Serialized L2O parameters
    pub avg_update_magnitude: f64,
    pub convergence_speed: f64,       // Generations to reach threshold
    pub stability_score: f64,         // How stable the updates are
    pub timestamp: Timestamp,
}

// ============================================================================
// DREAM/REFLECTION SYSTEM TABLES
// ============================================================================

/// Dream journal - Records of SAGE's dreams and insights
#[spacetimedb::table(name = dream_journal, public)]
pub struct DreamJournal {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    pub day: u32,                     // Which day this dream occurred
    pub dream_narrative: String,      // The dream content
    pub insights: String,             // JSON array of insights from the dream
    pub consolidated_concepts: String, // JSON array of concepts that were strengthened
    pub sleep_quality: f64,           // 0.0-1.0
    pub was_nightmare: bool,          // Was this a disturbing dream?
    pub mood_before: String,          // Mood when falling asleep
    pub mood_after: String,           // Mood upon waking
    pub energy_restored: f64,         // How much energy was restored
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

// ============================================================================
// META-LEARNING REDUCERS
// ============================================================================

/// Record a meta-learning strategy change
#[spacetimedb::reducer]
pub fn record_strategy_change(
    ctx: &ReducerContext,
    generation: u64,
    strategy_name: String,
    reason: String,
    performance_before: f64,
) {
    ctx.db.meta_strategy_history().insert(MetaStrategyHistory {
        id: 0,
        generation,
        strategy_name: strategy_name.clone(),
        reason: reason.clone(),
        performance_before,
        performance_after: None,
        duration_generations: None,
        timestamp: ctx.timestamp,
    });

    log::info!("🎯 Strategy change at gen {}: {} (reason: {})", generation, strategy_name, reason);
}

/// Update strategy outcome after it completes
#[spacetimedb::reducer]
pub fn update_strategy_outcome(
    ctx: &ReducerContext,
    strategy_id: u64,
    performance_after: f64,
    duration_generations: u64,
) {
    if let Some(old) = ctx.db.meta_strategy_history().iter().find(|s| s.id == strategy_id) {
        let strategy_name = old.strategy_name.clone();
        let reason = old.reason.clone();
        let performance_before = old.performance_before;
        let timestamp = old.timestamp;
        let generation = old.generation;

        ctx.db.meta_strategy_history().delete(old);
        ctx.db.meta_strategy_history().insert(MetaStrategyHistory {
            id: strategy_id,
            generation,
            strategy_name,
            reason,
            performance_before,
            performance_after: Some(performance_after),
            duration_generations: Some(duration_generations),
            timestamp,
        });

        log::info!("📊 Strategy {} outcome: {:.4} -> {:.4} over {} gens",
            strategy_id, performance_before, performance_after, duration_generations);
    }
}

/// Record hyperparameter configuration from PBT
#[spacetimedb::reducer]
pub fn record_hyperparameters(
    ctx: &ReducerContext,
    generation: u64,
    learning_rate: f64,
    batch_size: u32,
    evolution_steps: u32,
    mutation_rate: f64,
    fitness_score: f64,
    parent_id: Option<u64>,
    is_elite: bool,
) {
    ctx.db.hyperparameter_evolution().insert(HyperparameterEvolution {
        id: 0,
        generation,
        learning_rate,
        batch_size,
        evolution_steps,
        mutation_rate,
        fitness_score,
        parent_id,
        is_elite,
        timestamp: ctx.timestamp,
    });

    log::info!("🧬 Hyperparams recorded: LR={:.6}, batch={}, steps={}, fitness={:.4}{}",
        learning_rate, batch_size, evolution_steps, fitness_score,
        if is_elite { " [ELITE]" } else { "" });
}

/// Record architecture modification
#[spacetimedb::reducer]
pub fn record_architecture_change(
    ctx: &ReducerContext,
    generation: u64,
    change_type: String,
    layer_affected: String,
    old_size: u32,
    new_size: u32,
    trigger: String,
    loss_before: f64,
) {
    ctx.db.architecture_changes().insert(ArchitectureChange {
        id: 0,
        generation,
        change_type: change_type.clone(),
        layer_affected: layer_affected.clone(),
        old_size,
        new_size,
        trigger: trigger.clone(),
        loss_before,
        loss_after: None,
        timestamp: ctx.timestamp,
    });

    log::info!("🏗️  Architecture {}: {} {} -> {} (trigger: {})",
        change_type, layer_affected, old_size, new_size, trigger);
}

/// Record few-shot adaptation result
#[spacetimedb::reducer]
pub fn record_few_shot_adaptation(
    ctx: &ReducerContext,
    generation: u64,
    task_name: String,
    shots_used: u32,
    adaptation_steps: u32,
    loss_before: f64,
    loss_after: f64,
    transfer_source: String,
    adaptation_time_ms: u64,
) {
    ctx.db.few_shot_adaptations().insert(FewShotAdaptation {
        id: 0,
        generation,
        task_name: task_name.clone(),
        shots_used,
        adaptation_steps,
        loss_before,
        loss_after,
        transfer_source: transfer_source.clone(),
        adaptation_time_ms,
        timestamp: ctx.timestamp,
    });

    let improvement = ((loss_before - loss_after) / loss_before * 100.0).max(0.0);
    log::info!("🎓 Few-shot adaptation to {}: {:.4} -> {:.4} ({:.1}% improvement) in {}ms",
        task_name, loss_before, loss_after, improvement, adaptation_time_ms);
}

/// Save optimizer snapshot
#[spacetimedb::reducer]
pub fn save_optimizer_snapshot(
    ctx: &ReducerContext,
    generation: u64,
    optimizer_type: String,
    optimizer_weights_json: String,
    avg_update_magnitude: f64,
    convergence_speed: f64,
    stability_score: f64,
) {
    ctx.db.optimizer_snapshots().insert(OptimizerSnapshot {
        id: 0,
        generation,
        optimizer_type: optimizer_type.clone(),
        optimizer_weights_json,
        avg_update_magnitude,
        convergence_speed,
        stability_score,
        timestamp: ctx.timestamp,
    });

    log::info!("💾 Optimizer snapshot: {} at gen {} (stability: {:.3})",
        optimizer_type, generation, stability_score);
}

// ============================================================================
// DISTRIBUTED COGNITIVE ARCHITECTURE - "Brain Functions"
// Based on Global Workspace Theory, Society of Mind, and ACT-R
// ============================================================================

/// Brain function registry - defines available cognitive modules
#[derive(Clone)]
#[spacetimedb::table(name = brain_functions, public)]
pub struct BrainFunction {
    #[primary_key]
    pub function_name: String,          // "perception", "memory", "language", "emotion", etc.
    pub description: String,            // What this function does
    pub module_type: String,            // "sensory", "cognitive", "motor", "meta"
    pub priority_base: f64,             // Default priority (0.0-1.0)
    pub is_active: bool,                // Is this function currently enabled?
    pub last_fired: Timestamp,          // When did this function last execute?
    pub fire_count: u64,                // How many times has it fired?
    pub avg_execution_ms: f64,          // Average execution time
    pub registered_at: Timestamp,
}

/// Cognitive events - Global Workspace Theory broadcast mechanism
/// Events compete for attention; winners get broadcast to all modules
#[derive(Clone)]
#[spacetimedb::table(name = cognitive_events, public)]
pub struct CognitiveEvent {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    pub source_function: String,        // Which brain function generated this
    pub event_type: String,             // "perception", "thought", "memory_recall", "emotion", "decision"
    pub content: String,                // The actual content (JSON for complex data)
    pub salience: f64,                  // How attention-grabbing (0.0-1.0)
    pub urgency: f64,                   // Time-sensitivity (0.0-1.0)
    pub novelty: f64,                   // How new/surprising (0.0-1.0)
    pub priority: f64,                  // Computed: salience * urgency * novelty
    pub broadcast: bool,                // Has this won workspace access?
    pub broadcast_at: Option<Timestamp>,// When it was broadcast
    pub consumed_by: String,            // JSON array of functions that processed this
    pub created_at: Timestamp,
}

/// Inter-agent messages - Society of Mind communication
/// Brain functions can send requests to each other
#[derive(Clone)]
#[spacetimedb::table(name = agent_messages, public)]
pub struct AgentMessage {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    pub from_function: String,          // Sender brain function
    pub to_function: String,            // Receiver ("broadcast" for all)
    pub message_type: String,           // "request", "response", "notify", "query"
    pub payload: String,                // JSON message content
    pub correlation_id: Option<u64>,    // Links request to response
    pub handled: bool,                  // Has receiver processed this?
    pub response: Option<String>,       // Response payload (if applicable)
    pub created_at: Timestamp,
    pub handled_at: Option<Timestamp>,
}

/// Memory activations - ACT-R style activation-based retrieval
/// Concepts have activation levels that decay over time
#[derive(Clone)]
#[spacetimedb::table(name = memory_activations, public)]
pub struct MemoryActivation {
    #[primary_key]
    pub concept: String,                // Concept name (links to concept_memory)
    pub base_activation: f64,           // From usage frequency (ln of exposure_count)
    pub spreading_activation: f64,      // From related active concepts
    pub contextual_boost: f64,          // From current context relevance
    pub recency_boost: f64,             // Decays with time since last access
    pub total_activation: f64,          // Sum of all activation sources
    pub retrieval_threshold: f64,       // Minimum activation to be retrieved
    pub last_accessed: Timestamp,
    pub access_count: u64,
}

/// Attention focus - What's currently in the "spotlight"
#[derive(Clone)]
#[spacetimedb::table(name = attention_focus, public)]
pub struct AttentionFocus {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    pub focus_type: String,             // "concept", "task", "person", "sensation"
    pub focus_target: String,           // What we're focused on
    pub intensity: f64,                 // How strongly focused (0.0-1.0)
    pub duration_ms: u64,               // How long we've been focused
    pub source_event_id: Option<u64>,   // Which cognitive event triggered this
    pub started_at: Timestamp,
    pub ended_at: Option<Timestamp>,
}

/// Cognitive workspace - The "global workspace" contents
/// Only a few items can be in workspace at once (capacity limit)
#[derive(Clone)]
#[spacetimedb::table(name = cognitive_workspace, public)]
pub struct CognitiveWorkspace {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    pub slot_number: u32,               // Position in workspace (0-6, ~7 items max)
    pub content_type: String,           // "concept", "goal", "percept", "memory"
    pub content: String,                // The actual content
    pub source_event_id: u64,           // Which event put this here
    pub activation: f64,                // Current activation level
    pub entered_at: Timestamp,
    pub last_refreshed: Timestamp,
}

/// Goal stack - Hierarchical goals (from emergent_goals)
#[derive(Clone)]
#[spacetimedb::table(name = goal_stack, public)]
pub struct GoalStackEntry {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    pub goal_type: String,              // "immediate", "short_term", "long_term", "life"
    pub description: String,            // What the goal is
    pub priority: f64,                  // How important (0.0-1.0)
    pub progress: f64,                  // How complete (0.0-1.0)
    pub parent_goal_id: Option<u64>,    // For sub-goals
    pub status: String,                 // "active", "suspended", "completed", "abandoned"
    pub created_at: Timestamp,
    pub completed_at: Option<Timestamp>,
}

/// Execution log - Track when brain functions fire
#[spacetimedb::table(name = brain_function_log, public)]
pub struct BrainFunctionLog {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    pub function_name: String,
    pub trigger_type: String,           // "event", "scheduled", "request", "cascade"
    pub trigger_source: String,         // What caused the firing
    pub input_summary: String,          // Brief description of input
    pub output_summary: String,         // Brief description of output
    pub execution_ms: u64,              // How long it took
    pub events_generated: u32,          // How many cognitive events created
    pub success: bool,
    pub error_message: Option<String>,
    pub timestamp: Timestamp,
}

// ============================================================================
// BRAIN FUNCTION REDUCERS - Core Cognitive Operations
// ============================================================================

/// Register a new brain function
#[spacetimedb::reducer]
pub fn register_brain_function(
    ctx: &ReducerContext,
    function_name: String,
    description: String,
    module_type: String,
    priority_base: f64,
) -> Result<(), String> {
    // Check if already exists
    if ctx.db.brain_functions().iter().any(|f| f.function_name == function_name) {
        return Err(format!("Brain function '{}' already registered", function_name));
    }

    ctx.db.brain_functions().insert(BrainFunction {
        function_name: function_name.clone(),
        description,
        module_type: module_type.clone(),
        priority_base,
        is_active: true,
        last_fired: ctx.timestamp,
        fire_count: 0,
        avg_execution_ms: 0.0,
        registered_at: ctx.timestamp,
    });

    log::info!("🧠 Registered brain function: {} ({})", function_name, module_type);
    Ok(())
}

/// Emit a cognitive event (brain function wants to broadcast)
#[spacetimedb::reducer]
pub fn emit_cognitive_event(
    ctx: &ReducerContext,
    source_function: String,
    event_type: String,
    content: String,
    salience: f64,
    urgency: f64,
    novelty: f64,
) -> Result<(), String> {
    // Validate source function exists
    if !ctx.db.brain_functions().iter().any(|f| f.function_name == source_function) {
        return Err(format!("Unknown brain function: {}", source_function));
    }

    // Compute priority
    let priority = salience * (1.0 + urgency) * (1.0 + novelty * 0.5);

    ctx.db.cognitive_events().insert(CognitiveEvent {
        id: 0,  // Auto-increment
        source_function: source_function.clone(),
        event_type: event_type.clone(),
        content: content.clone(),
        salience,
        urgency,
        novelty,
        priority,
        broadcast: false,
        broadcast_at: None,
        consumed_by: "[]".to_string(),
        created_at: ctx.timestamp,
    });

    log::info!("📡 Cognitive event from {}: {} (priority: {:.3})",
        source_function, event_type, priority);

    Ok(())
}

/// Compete for workspace access - selects top-K events to broadcast
#[spacetimedb::reducer]
pub fn process_workspace_competition(
    ctx: &ReducerContext,
    max_broadcasts: u32,
) {
    // Get unbroadcast events sorted by priority
    let mut pending_events: Vec<_> = ctx.db.cognitive_events()
        .iter()
        .filter(|e| !e.broadcast)
        .collect();

    // Sort by priority (highest first)
    pending_events.sort_by(|a, b| b.priority.partial_cmp(&a.priority).unwrap_or(std::cmp::Ordering::Equal));

    // Broadcast top N events
    let mut broadcast_count = 0;
    for event in pending_events.iter().take(max_broadcasts as usize) {
        // Delete and re-insert with broadcast flag
        let updated = CognitiveEvent {
            id: event.id,
            source_function: event.source_function.clone(),
            event_type: event.event_type.clone(),
            content: event.content.clone(),
            salience: event.salience,
            urgency: event.urgency,
            novelty: event.novelty,
            priority: event.priority,
            broadcast: true,
            broadcast_at: Some(ctx.timestamp),
            consumed_by: event.consumed_by.clone(),
            created_at: event.created_at,
        };

        ctx.db.cognitive_events().delete(event.clone());
        ctx.db.cognitive_events().insert(updated);
        broadcast_count += 1;

        log::info!("📢 Broadcasting event {}: {} (priority: {:.3})",
            event.id, event.event_type, event.priority);
    }

    if broadcast_count > 0 {
        log::info!("🎯 Workspace competition: {} events broadcast", broadcast_count);
    }
}

/// Send a message between brain functions
#[spacetimedb::reducer]
pub fn send_agent_message(
    ctx: &ReducerContext,
    from_function: String,
    to_function: String,
    message_type: String,
    payload: String,
    correlation_id: Option<u64>,
) -> Result<(), String> {
    // Validate sender exists
    if !ctx.db.brain_functions().iter().any(|f| f.function_name == from_function) {
        return Err(format!("Unknown sender function: {}", from_function));
    }

    // Validate receiver exists (unless broadcast)
    if to_function != "broadcast" && !ctx.db.brain_functions().iter().any(|f| f.function_name == to_function) {
        return Err(format!("Unknown receiver function: {}", to_function));
    }

    ctx.db.agent_messages().insert(AgentMessage {
        id: 0,
        from_function: from_function.clone(),
        to_function: to_function.clone(),
        message_type: message_type.clone(),
        payload: payload.clone(),
        correlation_id,
        handled: false,
        response: None,
        created_at: ctx.timestamp,
        handled_at: None,
    });

    log::info!("✉️  Message {} -> {}: {} ({})",
        from_function, to_function, message_type,
        payload.chars().take(50).collect::<String>());

    Ok(())
}

/// Handle/respond to an agent message
#[spacetimedb::reducer]
pub fn handle_agent_message(
    ctx: &ReducerContext,
    message_id: u64,
    handler_function: String,
    response: Option<String>,
) -> Result<(), String> {
    let msg = ctx.db.agent_messages()
        .iter()
        .find(|m| m.id == message_id)
        .ok_or_else(|| format!("Message {} not found", message_id))?;

    // Check if this function is the intended receiver
    if msg.to_function != "broadcast" && msg.to_function != handler_function {
        return Err(format!("Message {} is for {}, not {}", message_id, msg.to_function, handler_function));
    }

    // Update message as handled
    let updated = AgentMessage {
        id: msg.id,
        from_function: msg.from_function.clone(),
        to_function: msg.to_function.clone(),
        message_type: msg.message_type.clone(),
        payload: msg.payload.clone(),
        correlation_id: msg.correlation_id,
        handled: true,
        response,
        created_at: msg.created_at,
        handled_at: Some(ctx.timestamp),
    };

    ctx.db.agent_messages().delete(msg);
    ctx.db.agent_messages().insert(updated);

    log::info!("✅ Message {} handled by {}", message_id, handler_function);
    Ok(())
}

/// Update memory activation for a concept (ACT-R style)
#[spacetimedb::reducer]
pub fn update_memory_activation(
    ctx: &ReducerContext,
    concept: String,
    spreading_activation: f64,
    contextual_boost: f64,
) {
    // Get concept from concept_memory for base activation
    let concept_mem = ctx.db.concept_memory()
        .iter()
        .find(|c| c.concept_name == concept);

    let base_activation = if let Some(cm) = concept_mem {
        // ACT-R formula: base = ln(exposure_count)
        (cm.exposure_count as f64).ln().max(0.0)
    } else {
        0.0
    };

    // Calculate recency boost (decays with time)
    let existing = ctx.db.memory_activations()
        .iter()
        .find(|m| m.concept == concept);

    let (recency_boost, access_count) = if let Some(existing_mem) = &existing {
        // Decay based on time since last access
        // This is simplified - real ACT-R uses power law decay
        (existing_mem.recency_boost * 0.95, existing_mem.access_count + 1)
    } else {
        (1.0, 1)
    };

    // Compute total activation
    let total_activation = base_activation + spreading_activation + contextual_boost + recency_boost;

    // Default retrieval threshold
    let retrieval_threshold = 0.5;

    // Upsert activation record
    if let Some(old) = existing {
        ctx.db.memory_activations().delete(old);
    }

    ctx.db.memory_activations().insert(MemoryActivation {
        concept: concept.clone(),
        base_activation,
        spreading_activation,
        contextual_boost,
        recency_boost,
        total_activation,
        retrieval_threshold,
        last_accessed: ctx.timestamp,
        access_count,
    });

    log::info!("🧬 Activation updated for '{}': {:.3} (base={:.2}, spread={:.2}, ctx={:.2}, rec={:.2})",
        concept, total_activation, base_activation, spreading_activation, contextual_boost, recency_boost);
}

/// Decay all memory activations (called periodically)
#[spacetimedb::reducer]
pub fn decay_memory_activations(
    ctx: &ReducerContext,
    decay_factor: f64,
) {
    let activations: Vec<_> = ctx.db.memory_activations().iter().collect();

    for activation in activations {
        let new_recency = activation.recency_boost * decay_factor;
        let new_spreading = activation.spreading_activation * decay_factor;
        let new_total = activation.base_activation + new_spreading + activation.contextual_boost + new_recency;

        ctx.db.memory_activations().delete(activation.clone());
        ctx.db.memory_activations().insert(MemoryActivation {
            concept: activation.concept,
            base_activation: activation.base_activation,
            spreading_activation: new_spreading,
            contextual_boost: activation.contextual_boost,
            recency_boost: new_recency,
            total_activation: new_total,
            retrieval_threshold: activation.retrieval_threshold,
            last_accessed: activation.last_accessed,
            access_count: activation.access_count,
        });
    }

    log::info!("⏳ Memory activations decayed by factor {:.2}", decay_factor);
}

/// Set attention focus
#[spacetimedb::reducer]
pub fn set_attention_focus(
    ctx: &ReducerContext,
    focus_type: String,
    focus_target: String,
    intensity: f64,
    source_event_id: Option<u64>,
) {
    // End any current focus of the same type
    let current_focus: Vec<_> = ctx.db.attention_focus()
        .iter()
        .filter(|f| f.focus_type == focus_type && f.ended_at.is_none())
        .collect();

    for focus in current_focus {
        let updated = AttentionFocus {
            id: focus.id,
            focus_type: focus.focus_type.clone(),
            focus_target: focus.focus_target.clone(),
            intensity: focus.intensity,
            duration_ms: focus.duration_ms,
            source_event_id: focus.source_event_id,
            started_at: focus.started_at,
            ended_at: Some(ctx.timestamp),
        };
        ctx.db.attention_focus().delete(focus);
        ctx.db.attention_focus().insert(updated);
    }

    // Create new focus
    ctx.db.attention_focus().insert(AttentionFocus {
        id: 0,
        focus_type: focus_type.clone(),
        focus_target: focus_target.clone(),
        intensity,
        duration_ms: 0,
        source_event_id,
        started_at: ctx.timestamp,
        ended_at: None,
    });

    log::info!("👁️  Attention focus: {} on '{}' (intensity: {:.2})", focus_type, focus_target, intensity);
}

/// Update cognitive workspace (limited capacity - 7 slots)
#[spacetimedb::reducer]
pub fn update_workspace(
    ctx: &ReducerContext,
    content_type: String,
    content: String,
    source_event_id: u64,
    activation: f64,
) {
    const MAX_WORKSPACE_SLOTS: u32 = 7;

    // Get current workspace contents
    let mut workspace: Vec<_> = ctx.db.cognitive_workspace().iter().collect();

    // If workspace is full, remove lowest activation item
    if workspace.len() >= MAX_WORKSPACE_SLOTS as usize {
        workspace.sort_by(|a, b| a.activation.partial_cmp(&b.activation).unwrap_or(std::cmp::Ordering::Equal));
        if let Some(lowest) = workspace.first() {
            if lowest.activation < activation {
                log::info!("🗑️  Removing '{}' from workspace (activation: {:.2})", lowest.content, lowest.activation);
                ctx.db.cognitive_workspace().delete(lowest.clone());
            } else {
                log::info!("⚠️  Workspace full, new item not strong enough to enter");
                return;
            }
        }
    }

    // Find next available slot
    let used_slots: Vec<u32> = ctx.db.cognitive_workspace()
        .iter()
        .map(|w| w.slot_number)
        .collect();

    let slot = (0..MAX_WORKSPACE_SLOTS)
        .find(|s| !used_slots.contains(s))
        .unwrap_or(0);

    ctx.db.cognitive_workspace().insert(CognitiveWorkspace {
        id: 0,
        slot_number: slot,
        content_type: content_type.clone(),
        content: content.clone(),
        source_event_id,
        activation,
        entered_at: ctx.timestamp,
        last_refreshed: ctx.timestamp,
    });

    log::info!("📋 Workspace slot {}: {} '{}' (activation: {:.2})",
        slot, content_type, content.chars().take(30).collect::<String>(), activation);
}

/// Clear old items from workspace (items decay)
#[spacetimedb::reducer]
pub fn refresh_workspace(
    ctx: &ReducerContext,
    decay_factor: f64,
    removal_threshold: f64,
) {
    let items: Vec<_> = ctx.db.cognitive_workspace().iter().collect();

    for item in items {
        let new_activation = item.activation * decay_factor;

        if new_activation < removal_threshold {
            log::info!("🗑️  Removing '{}' from workspace (decayed to {:.2})", item.content, new_activation);
            ctx.db.cognitive_workspace().delete(item);
        } else {
            // Update with decayed activation
            ctx.db.cognitive_workspace().delete(item.clone());
            ctx.db.cognitive_workspace().insert(CognitiveWorkspace {
                id: item.id,
                slot_number: item.slot_number,
                content_type: item.content_type,
                content: item.content,
                source_event_id: item.source_event_id,
                activation: new_activation,
                entered_at: item.entered_at,
                last_refreshed: ctx.timestamp,
            });
        }
    }
}

/// Push a goal onto the goal stack
#[spacetimedb::reducer]
pub fn push_goal(
    ctx: &ReducerContext,
    goal_type: String,
    description: String,
    priority: f64,
    parent_goal_id: Option<u64>,
) -> Result<(), String> {
    // Validate goal type
    let valid_types = ["immediate", "short_term", "long_term", "life"];
    if !valid_types.contains(&goal_type.as_str()) {
        return Err(format!("Invalid goal type: {}. Must be one of: {:?}", goal_type, valid_types));
    }

    ctx.db.goal_stack().insert(GoalStackEntry {
        id: 0,
        goal_type: goal_type.clone(),
        description: description.clone(),
        priority,
        progress: 0.0,
        parent_goal_id,
        status: "active".to_string(),
        created_at: ctx.timestamp,
        completed_at: None,
    });

    log::info!("🎯 Goal pushed: [{}] {} (priority: {:.2})", goal_type, description, priority);
    Ok(())
}

/// Update goal progress
#[spacetimedb::reducer]
pub fn update_goal_progress(
    ctx: &ReducerContext,
    goal_id: u64,
    progress: f64,
    new_status: Option<String>,
) -> Result<(), String> {
    let goal = ctx.db.goal_stack()
        .iter()
        .find(|g| g.id == goal_id)
        .ok_or_else(|| format!("Goal {} not found", goal_id))?;

    let status = new_status.unwrap_or_else(|| goal.status.clone());
    let completed_at = if status == "completed" { Some(ctx.timestamp) } else { goal.completed_at };

    ctx.db.goal_stack().delete(goal.clone());
    ctx.db.goal_stack().insert(GoalStackEntry {
        id: goal.id,
        goal_type: goal.goal_type,
        description: goal.description,
        priority: goal.priority,
        progress,
        parent_goal_id: goal.parent_goal_id,
        status: status.clone(),
        created_at: goal.created_at,
        completed_at,
    });

    log::info!("📊 Goal {} progress: {:.0}% (status: {})", goal_id, progress * 100.0, status);
    Ok(())
}

/// Log brain function execution
#[spacetimedb::reducer]
pub fn log_brain_function_execution(
    ctx: &ReducerContext,
    function_name: String,
    trigger_type: String,
    trigger_source: String,
    input_summary: String,
    output_summary: String,
    execution_ms: u64,
    events_generated: u32,
    success: bool,
    error_message: Option<String>,
) {
    // Clone error_message for logging before moving into insert
    let error_for_log = error_message.clone();

    ctx.db.brain_function_log().insert(BrainFunctionLog {
        id: 0,
        function_name: function_name.clone(),
        trigger_type,
        trigger_source,
        input_summary,
        output_summary,
        execution_ms,
        events_generated,
        success,
        error_message,
        timestamp: ctx.timestamp,
    });

    // Update brain function stats
    if let Some(func) = ctx.db.brain_functions().iter().find(|f| f.function_name == function_name) {
        let new_count = func.fire_count + 1;
        let new_avg_ms = (func.avg_execution_ms * func.fire_count as f64 + execution_ms as f64) / new_count as f64;

        ctx.db.brain_functions().delete(func.clone());
        ctx.db.brain_functions().insert(BrainFunction {
            function_name: func.function_name,
            description: func.description,
            module_type: func.module_type,
            priority_base: func.priority_base,
            is_active: func.is_active,
            last_fired: ctx.timestamp,
            fire_count: new_count,
            avg_execution_ms: new_avg_ms,
            registered_at: func.registered_at,
        });
    }

    if success {
        log::info!("⚡ {} executed in {}ms, generated {} events", function_name, execution_ms, events_generated);
    } else {
        log::warn!("❌ {} failed after {}ms: {:?}", function_name, execution_ms, error_for_log);
    }
}

/// Initialize default brain functions (call once at setup)
#[spacetimedb::reducer]
pub fn init_brain_functions(ctx: &ReducerContext) {
    let functions = [
        // Core cognitive functions
        ("perception", "Process sensory input (vision, audio)", "sensory", 0.8),
        ("memory_retrieval", "Recall relevant memories based on context", "cognitive", 0.7),
        ("language_understanding", "Parse and understand natural language", "cognitive", 0.9),
        ("language_generation", "Generate natural language responses", "cognitive", 0.9),
        ("emotion_processing", "Evaluate emotional significance of events", "cognitive", 0.6),
        ("attention_control", "Direct focus and filter information", "meta", 0.85),
        ("goal_management", "Track and update goals", "meta", 0.7),
        ("introspection", "Self-reflect on mental states", "meta", 0.5),
        ("curiosity", "Generate questions and exploration drives", "cognitive", 0.6),
        ("social_modeling", "Model other minds (theory of mind)", "cognitive", 0.7),
        ("decision_making", "Evaluate options and make choices", "cognitive", 0.8),
        ("learning", "Update knowledge from experience", "meta", 0.75),
        // Inner world simulation functions
        ("perception_module", "Process inner world sensory input", "sensory", 0.8),
        ("event_detector", "Detect and process life events", "sensory", 0.85),
        ("learning_module", "Extract lessons from experiences", "meta", 0.75),
        ("inner_speech", "Generate internal thoughts and self-talk", "cognitive", 0.7),
        ("action_selector", "Choose actions based on goals and state", "motor", 0.8),
        ("motor_feedback", "Process results of actions taken", "motor", 0.6),
        ("reading_comprehension", "Extract insights from reading", "cognitive", 0.7),
        ("social_module", "Process social desires and outreach", "cognitive", 0.7),
        // Discord integration functions
        ("discord_input", "Process incoming Discord messages", "sensory", 0.9),
        ("discord_output", "Generate Discord responses", "motor", 0.9),
        ("response_generator", "Generate conversational responses", "cognitive", 0.85),
    ];

    for (name, desc, module_type, priority) in functions {
        // Only insert if not exists
        if !ctx.db.brain_functions().iter().any(|f| f.function_name == name) {
            ctx.db.brain_functions().insert(BrainFunction {
                function_name: name.to_string(),
                description: desc.to_string(),
                module_type: module_type.to_string(),
                priority_base: priority,
                is_active: true,
                last_fired: ctx.timestamp,
                fire_count: 0,
                avg_execution_ms: 0.0,
                registered_at: ctx.timestamp,
            });
        }
    }

    log::info!("🧠 Brain functions initialized: {} modules registered", functions.len());
}

/// Get workspace summary (for debugging/monitoring)
#[spacetimedb::reducer]
pub fn get_cognitive_state_summary(ctx: &ReducerContext) {
    let workspace_count = ctx.db.cognitive_workspace().iter().count();
    let active_goals = ctx.db.goal_stack().iter().filter(|g| g.status == "active").count();
    let pending_events = ctx.db.cognitive_events().iter().filter(|e| !e.broadcast).count();
    let unhandled_messages = ctx.db.agent_messages().iter().filter(|m| !m.handled).count();
    let active_focus: Vec<_> = ctx.db.attention_focus()
        .iter()
        .filter(|f| f.ended_at.is_none())
        .map(|f| f.focus_target.clone())
        .collect();

    log::info!("🧠 Cognitive State Summary:");
    log::info!("   Workspace: {}/7 slots filled", workspace_count);
    log::info!("   Active goals: {}", active_goals);
    log::info!("   Pending events: {}", pending_events);
    log::info!("   Unhandled messages: {}", unhandled_messages);
    log::info!("   Current focus: {:?}", active_focus);
}

// ============================================================================
// WORKSPACE QUERY REDUCERS - For Active Cognitive Architecture (Feature 1)
// ============================================================================

/// Spread activation from a concept to its related concepts (ACT-R style)
/// This boosts related memories when a concept is accessed
#[spacetimedb::reducer]
pub fn spread_activation(
    ctx: &ReducerContext,
    source_concept: String,
    decay_factor: f64,
) {
    // Find the source concept's related concepts
    let concept_mem = ctx.db.concept_memory()
        .iter()
        .find(|c| c.concept_name == source_concept);

    if let Some(cm) = concept_mem {
        // Parse related_concepts JSON array
        // Format: ["concept1", "concept2", ...]
        let related: Vec<String> = cm.related_concepts
            .trim_matches(|c| c == '[' || c == ']')
            .split(',')
            .map(|s| s.trim().trim_matches('"').to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let spread_amount = decay_factor; // Amount to spread per link

        for related_concept in related.iter() {
            // Boost activation for each related concept
            let existing = ctx.db.memory_activations()
                .iter()
                .find(|m| m.concept == *related_concept);

            if let Some(activation) = existing {
                // Add spreading activation
                let new_spreading = activation.spreading_activation + spread_amount;
                let new_total = activation.base_activation + new_spreading +
                    activation.contextual_boost + activation.recency_boost;

                ctx.db.memory_activations().delete(activation.clone());
                ctx.db.memory_activations().insert(MemoryActivation {
                    concept: activation.concept,
                    base_activation: activation.base_activation,
                    spreading_activation: new_spreading,
                    contextual_boost: activation.contextual_boost,
                    recency_boost: activation.recency_boost,
                    total_activation: new_total,
                    retrieval_threshold: activation.retrieval_threshold,
                    last_accessed: activation.last_accessed,
                    access_count: activation.access_count,
                });

                log::info!("🔗 Spread activation: {} -> {} (+{:.2})",
                    source_concept, related_concept, spread_amount);
            } else {
                // Create new activation record for related concept
                ctx.db.memory_activations().insert(MemoryActivation {
                    concept: related_concept.clone(),
                    base_activation: 0.0,
                    spreading_activation: spread_amount,
                    contextual_boost: 0.0,
                    recency_boost: 0.5, // New concepts get some recency boost
                    total_activation: spread_amount + 0.5,
                    retrieval_threshold: 0.5,
                    last_accessed: ctx.timestamp,
                    access_count: 1,
                });

                log::info!("🔗 New activation from spread: {} -> {} ({:.2})",
                    source_concept, related_concept, spread_amount);
            }
        }

        log::info!("🧠 Spread activation from '{}' to {} related concepts",
            source_concept, related.len());
    } else {
        log::warn!("⚠️  Concept '{}' not found in memory", source_concept);
    }
}

/// Get workspace contents with activation levels (logs as parseable format)
#[spacetimedb::reducer]
pub fn query_workspace_contents(ctx: &ReducerContext) {
    let mut items: Vec<_> = ctx.db.cognitive_workspace().iter().collect();
    items.sort_by(|a, b| b.activation.partial_cmp(&a.activation).unwrap_or(std::cmp::Ordering::Equal));

    log::info!("📋 WORKSPACE_QUERY_START");
    for item in items {
        // Use pipe-delimited format for easy parsing
        log::info!("WORKSPACE_ITEM|{}|{}|{}|{:.3}",
            item.slot_number,
            item.content_type,
            item.content.replace("|", "\\|").replace("\n", "\\n"),
            item.activation
        );
    }
    log::info!("📋 WORKSPACE_QUERY_END");
}

/// Get high-activation memories above a threshold
#[spacetimedb::reducer]
pub fn query_high_activation_memories(
    ctx: &ReducerContext,
    threshold: f64,
    max_results: u32,
) {
    let mut activations: Vec<_> = ctx.db.memory_activations()
        .iter()
        .filter(|m| m.total_activation >= threshold)
        .collect();

    // Sort by total activation descending
    activations.sort_by(|a, b| b.total_activation.partial_cmp(&a.total_activation).unwrap_or(std::cmp::Ordering::Equal));

    log::info!("🧠 HIGH_ACTIVATION_QUERY_START|threshold={:.2}", threshold);
    for activation in activations.iter().take(max_results as usize) {
        log::info!("HIGH_ACTIVATION|{}|{:.3}|{:.2}|{:.2}|{:.2}|{:.2}",
            activation.concept,
            activation.total_activation,
            activation.base_activation,
            activation.spreading_activation,
            activation.contextual_boost,
            activation.recency_boost
        );
    }
    log::info!("🧠 HIGH_ACTIVATION_QUERY_END|count={}", activations.len().min(max_results as usize));
}

/// Get current attention focus
#[spacetimedb::reducer]
pub fn query_current_attention_focus(ctx: &ReducerContext) {
    let active_focus: Vec<_> = ctx.db.attention_focus()
        .iter()
        .filter(|f| f.ended_at.is_none())
        .collect();

    log::info!("👁️  ATTENTION_FOCUS_QUERY_START");
    for focus in active_focus {
        log::info!("ATTENTION_FOCUS|{}|{}|{:.2}",
            focus.focus_type,
            focus.focus_target.replace("|", "\\|"),
            focus.intensity
        );
    }
    log::info!("👁️  ATTENTION_FOCUS_QUERY_END");
}

/// Get recent broadcast events (thoughts that won workspace access)
#[spacetimedb::reducer]
pub fn query_recent_broadcasts(
    ctx: &ReducerContext,
    max_results: u32,
) {
    let mut broadcasts: Vec<_> = ctx.db.cognitive_events()
        .iter()
        .filter(|e| e.broadcast)
        .collect();

    // Sort by broadcast time descending (most recent first)
    broadcasts.sort_by(|a, b| {
        match (&b.broadcast_at, &a.broadcast_at) {
            (Some(b_time), Some(a_time)) => b_time.cmp(a_time),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => std::cmp::Ordering::Equal,
        }
    });

    log::info!("📢 BROADCAST_QUERY_START");
    for event in broadcasts.iter().take(max_results as usize) {
        log::info!("BROADCAST|{}|{}|{}|{:.3}",
            event.source_function,
            event.event_type,
            event.content.replace("|", "\\|").replace("\n", "\\n").chars().take(200).collect::<String>(),
            event.priority
        );
    }
    log::info!("📢 BROADCAST_QUERY_END|count={}", broadcasts.len().min(max_results as usize));
}

/// Boost contextual activation for concepts mentioned in current context
#[spacetimedb::reducer]
pub fn boost_contextual_activation(
    ctx: &ReducerContext,
    concepts_json: String,  // JSON array: ["concept1", "concept2", ...]
    boost_amount: f64,
) {
    // Parse concepts from JSON array
    let concepts: Vec<String> = concepts_json
        .trim_matches(|c| c == '[' || c == ']')
        .split(',')
        .map(|s| s.trim().trim_matches('"').to_string())
        .filter(|s| !s.is_empty())
        .collect();

    let mut boosted_count = 0;

    for concept in concepts.iter() {
        let existing = ctx.db.memory_activations()
            .iter()
            .find(|m| m.concept == *concept);

        if let Some(activation) = existing {
            // Add contextual boost
            let new_contextual = activation.contextual_boost + boost_amount;
            let new_total = activation.base_activation + activation.spreading_activation +
                new_contextual + activation.recency_boost;

            ctx.db.memory_activations().delete(activation.clone());
            ctx.db.memory_activations().insert(MemoryActivation {
                concept: activation.concept,
                base_activation: activation.base_activation,
                spreading_activation: activation.spreading_activation,
                contextual_boost: new_contextual,
                recency_boost: activation.recency_boost,
                total_activation: new_total,
                retrieval_threshold: activation.retrieval_threshold,
                last_accessed: ctx.timestamp,
                access_count: activation.access_count + 1,
            });
            boosted_count += 1;
        } else {
            // Create new activation record
            ctx.db.memory_activations().insert(MemoryActivation {
                concept: concept.clone(),
                base_activation: 0.0,
                spreading_activation: 0.0,
                contextual_boost: boost_amount,
                recency_boost: 1.0, // Fresh access
                total_activation: boost_amount + 1.0,
                retrieval_threshold: 0.5,
                last_accessed: ctx.timestamp,
                access_count: 1,
            });
            boosted_count += 1;
        }
    }

    log::info!("📈 Contextual boost applied to {} concepts (+{:.2})", boosted_count, boost_amount);
}

// ============================================================================
// DREAM/REFLECTION SYSTEM REDUCERS
// ============================================================================

/// Save a dream journal entry
#[spacetimedb::reducer]
pub fn save_dream_journal(
    ctx: &ReducerContext,
    day: u32,
    dream_narrative: String,
    insights_json: String,         // JSON array
    consolidated_json: String,     // JSON array
    sleep_quality: f64,
    was_nightmare: bool,
    mood_before: String,
    mood_after: String,
    energy_restored: f64,
) {
    ctx.db.dream_journal().insert(DreamJournal {
        id: 0,
        day,
        dream_narrative: dream_narrative.clone(),
        insights: insights_json.clone(),
        consolidated_concepts: consolidated_json,
        sleep_quality,
        was_nightmare,
        mood_before,
        mood_after,
        energy_restored,
        timestamp: ctx.timestamp,
    });

    // Also emit a cognitive event for the dream
    let priority = if was_nightmare { 0.9 } else { 0.6 };
    ctx.db.cognitive_events().insert(CognitiveEvent {
        id: 0,
        source_function: "dream_system".to_string(),
        event_type: if was_nightmare { "nightmare".to_string() } else { "dream".to_string() },
        content: format!("Day {} dream: {}", day, &dream_narrative.chars().take(100).collect::<String>()),
        salience: sleep_quality,
        urgency: if was_nightmare { 0.8 } else { 0.3 },
        novelty: 0.7,
        priority,
        broadcast: false,
        broadcast_at: None,
        consumed_by: "[]".to_string(),
        created_at: ctx.timestamp,
    });

    log::info!("💤 Dream journal entry saved for day {} (quality: {:.2}, nightmare: {})",
        day, sleep_quality, was_nightmare);
}

/// Query recent dreams
#[spacetimedb::reducer]
pub fn query_recent_dreams(
    ctx: &ReducerContext,
    max_results: u32,
) {
    let mut dreams: Vec<_> = ctx.db.dream_journal().iter().collect();
    dreams.sort_by(|a, b| b.day.cmp(&a.day));

    log::info!("💤 DREAM_QUERY_START");
    for dream in dreams.iter().take(max_results as usize) {
        log::info!("DREAM|{}|{}|{:.2}|{}|{}",
            dream.day,
            dream.dream_narrative.replace("|", "\\|").replace("\n", "\\n").chars().take(150).collect::<String>(),
            dream.sleep_quality,
            dream.was_nightmare,
            dream.insights.replace("|", "\\|")
        );
    }
    log::info!("💤 DREAM_QUERY_END|count={}", dreams.len().min(max_results as usize));
}

/// Get sleep statistics
#[spacetimedb::reducer]
pub fn get_sleep_stats(ctx: &ReducerContext) {
    let dreams: Vec<_> = ctx.db.dream_journal().iter().collect();

    if dreams.is_empty() {
        log::info!("💤 No dream records found");
        return;
    }

    let total_dreams = dreams.len();
    let nightmare_count = dreams.iter().filter(|d| d.was_nightmare).count();
    let avg_quality: f64 = dreams.iter().map(|d| d.sleep_quality).sum::<f64>() / total_dreams as f64;
    let avg_energy: f64 = dreams.iter().map(|d| d.energy_restored).sum::<f64>() / total_dreams as f64;

    log::info!("💤 Sleep Statistics:");
    log::info!("   Total dreams recorded: {}", total_dreams);
    log::info!("   Nightmare rate: {:.1}%", (nightmare_count as f64 / total_dreams as f64) * 100.0);
    log::info!("   Average sleep quality: {:.2}", avg_quality);
    log::info!("   Average energy restored: {:.1}", avg_energy);
}
