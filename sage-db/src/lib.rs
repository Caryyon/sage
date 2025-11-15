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
