use std::sync::{Arc, Mutex};

/// SpacetimeDB client for SAGE persistence
/// Currently uses CLI for simplicity - will migrate to SDK later
#[derive(Clone)]
pub struct SageDbClient {
    db_name: String,
    connected: Arc<Mutex<bool>>,
}

impl SageDbClient {
    /// Create a new SAGE database client
    pub fn new(db_name: &str) -> Self {
        Self {
            db_name: db_name.to_string(),
            connected: Arc::new(Mutex::new(true)), // Always connected via CLI
        }
    }

    /// Connect to SpacetimeDB (no-op for CLI version)
    pub fn connect(&mut self) -> Result<(), String> {
        // CLI is always connected if server is running
        *self.connected.lock().unwrap() = true;
        Ok(())
    }

    /// Update SAGE's current state
    pub fn update_sage_state(
        &self,
        generation: u64,
        loss: f64,
        pattern: &str,
        complexity: f64,
        diversity: f64,
    ) -> Result<(), String> {
        if !*self.connected.lock().unwrap() {
            return Err("Not connected to SpacetimeDB".to_string());
        }

        // Use CLI to call reducer for now
        // In production, would use SDK's call_reducer method
        let status = std::process::Command::new("spacetime")
            .args(&[
                "call",
                &self.db_name,
                "update_sage_state",
                &generation.to_string(),
                &loss.to_string(),
                pattern,
                &complexity.to_string(),
                &diversity.to_string(),
            ])
            .stderr(std::process::Stdio::null())
            .status()
            .map_err(|e| format!("Failed to call reducer: {}", e))?;

        if !status.success() {
            return Err("Reducer call failed".to_string());
        }

        Ok(())
    }

    /// Save network weights snapshot
    pub fn save_network_snapshot(
        &self,
        generation: u64,
        pattern: &str,
        loss: f64,
        weights_json: &str,
    ) -> Result<(), String> {
        if !*self.connected.lock().unwrap() {
            return Err("Not connected to SpacetimeDB".to_string());
        }

        let status = std::process::Command::new("spacetime")
            .args(&[
                "call",
                &self.db_name,
                "save_network_snapshot",
                &generation.to_string(),
                pattern,
                &loss.to_string(),
                weights_json,
            ])
            .stderr(std::process::Stdio::null())
            .status()
            .map_err(|e| format!("Failed to call reducer: {}", e))?;

        if !status.success() {
            return Err("Reducer call failed".to_string());
        }

        Ok(())
    }

    /// Add a conversation message with SAGE's response and memory
    pub fn add_conversation_message(
        &self,
        sender: &str,
        message: &str,
        sage_response: &str,
        nca_loss: f64,
        concepts_json: &str,
        generation_context: u64,
    ) -> Result<(), String> {
        if !*self.connected.lock().unwrap() {
            return Err("Not connected to SpacetimeDB".to_string());
        }

        // Escape and JSON-quote string arguments for spacetime CLI
        let escape_json = |s: &str| -> String {
            let escaped = s
                .replace('\\', "\\\\")
                .replace('"', "\\\"")
                .replace('\n', "\\n")
                .replace('\r', "\\r")
                .replace('\t', "\\t");
            format!("\"{}\"", escaped)
        };

        let status = std::process::Command::new("spacetime")
            .args(&[
                "call",
                &self.db_name,
                "add_conversation_message",
                &escape_json(sender),
                &escape_json(message),
                &escape_json(sage_response),
                &nca_loss.to_string(),
                concepts_json,  // Already JSON formatted
                &generation_context.to_string(),
            ])
            .stderr(std::process::Stdio::null())
            .status()
            .map_err(|e| format!("Failed to call reducer: {}", e))?;

        if !status.success() {
            return Err("Reducer call failed".to_string());
        }

        Ok(())
    }

    /// Store a concept in SAGE's memory
    pub fn store_concept(
        &self,
        concept_name: &str,
        nca_encoding: &str,
        familiarity: f64,
        avg_loss: f64,
        exposure_count: u64,
        opinion_type: &str,
        opinion_reason: &str,
        confidence: f64,
        related_json: &str,
    ) -> Result<(), String> {
        if !*self.connected.lock().unwrap() {
            return Err("Not connected to SpacetimeDB".to_string());
        }

        let status = std::process::Command::new("spacetime")
            .args(&[
                "call",
                &self.db_name,
                "store_concept_memory",
                concept_name,
                nca_encoding,
                &familiarity.to_string(),
                &avg_loss.to_string(),
                &exposure_count.to_string(),
                opinion_type,
                opinion_reason,
                &confidence.to_string(),
                related_json,
            ])
            .stderr(std::process::Stdio::null())
            .status()
            .map_err(|e| format!("Failed to call reducer: {}", e))?;

        if !status.success() {
            return Err("Reducer call failed".to_string());
        }

        Ok(())
    }

    /// Record an opinion in history
    pub fn record_opinion(
        &self,
        concept: &str,
        opinion_type: &str,
        reason: &str,
        confidence: f64,
        nca_loss: f64,
        generation: u64,
    ) -> Result<(), String> {
        if !*self.connected.lock().unwrap() {
            return Err("Not connected to SpacetimeDB".to_string());
        }

        let status = std::process::Command::new("spacetime")
            .args(&[
                "call",
                &self.db_name,
                "record_opinion",
                concept,
                opinion_type,
                reason,
                &confidence.to_string(),
                &nca_loss.to_string(),
                &generation.to_string(),
            ])
            .stderr(std::process::Stdio::null())
            .status()
            .map_err(|e| format!("Failed to call reducer: {}", e))?;

        if !status.success() {
            return Err("Reducer call failed".to_string());
        }

        Ok(())
    }

    /// Save personality snapshot
    pub fn save_personality(
        &self,
        generation: u64,
        openness: f64,
        positivity: f64,
        curiosity: f64,
        confidence: f64,
        experience_count: u64,
        likes_count: u64,
        dislikes_count: u64,
    ) -> Result<(), String> {
        if !*self.connected.lock().unwrap() {
            return Err("Not connected to SpacetimeDB".to_string());
        }

        let status = std::process::Command::new("spacetime")
            .args(&[
                "call",
                &self.db_name,
                "save_personality_snapshot",
                &generation.to_string(),
                &openness.to_string(),
                &positivity.to_string(),
                &curiosity.to_string(),
                &confidence.to_string(),
                &experience_count.to_string(),
                &likes_count.to_string(),
                &dislikes_count.to_string(),
            ])
            .stderr(std::process::Stdio::null())
            .status()
            .map_err(|e| format!("Failed to call reducer: {}", e))?;

        if !status.success() {
            return Err("Reducer call failed".to_string());
        }

        Ok(())
    }

    /// Save an introspection report
    pub fn save_introspection(
        &self,
        experience_count: u64,
        valence: f64,
        intensity: f64,
        complexity: f64,
        feeling_name: &str,
        mode: &str,
        qualities_json: &str,
        active_concepts_json: &str,
        description: &str,
        temporal_context: &str,
        trigger: &str,
    ) -> Result<(), String> {
        if !*self.connected.lock().unwrap() {
            return Err("Not connected to SpacetimeDB".to_string());
        }

        let status = std::process::Command::new("spacetime")
            .args(&[
                "call",
                &self.db_name,
                "save_introspection",
                &experience_count.to_string(),
                &valence.to_string(),
                &intensity.to_string(),
                &complexity.to_string(),
                feeling_name,
                mode,
                qualities_json,
                active_concepts_json,
                description,
                temporal_context,
                trigger,
            ])
            .stderr(std::process::Stdio::null())
            .status()
            .map_err(|e| format!("Failed to call reducer: {}", e))?;

        if !status.success() {
            return Err("Reducer call failed".to_string());
        }

        Ok(())
    }

    /// Log autonomous activity (Dream Mode or Curiosity Mode)
    pub fn log_autonomous_activity(
        &self,
        activity_type: &str,
        experience_count: u64,
        seconds_idle: u64,
        concepts_deepened_json: &str,
        concepts_consolidated_json: &str,
        links_formed_json: &str,
        question: &str,
        notes: &str,
    ) -> Result<(), String> {
        if !*self.connected.lock().unwrap() {
            return Err("Not connected to SpacetimeDB".to_string());
        }

        let status = std::process::Command::new("spacetime")
            .args(&[
                "call",
                &self.db_name,
                "log_autonomous_activity",
                activity_type,
                &experience_count.to_string(),
                &seconds_idle.to_string(),
                concepts_deepened_json,
                concepts_consolidated_json,
                links_formed_json,
                question,
                notes,
            ])
            .stderr(std::process::Stdio::null())
            .status()
            .map_err(|e| format!("Failed to call reducer: {}", e))?;

        if !status.success() {
            return Err("Reducer call failed".to_string());
        }

        Ok(())
    }

    /// Record A/B test result
    pub fn record_ab_test(
        &self,
        experience_count: u64,
        input: &str,
        nca_response: &str,
        baseline_response: &str,
        nca_opinion: &str,
        user_pref: &str,
        avg_alpha: f64,
    ) -> Result<(), String> {
        if !*self.connected.lock().unwrap() {
            return Err("Not connected to SpacetimeDB".to_string());
        }

        let status = std::process::Command::new("spacetime")
            .args(&[
                "call",
                &self.db_name,
                "record_ab_test",
                &experience_count.to_string(),
                input,
                nca_response,
                baseline_response,
                nca_opinion,
                user_pref,
                &avg_alpha.to_string(),
            ])
            .stderr(std::process::Stdio::null())
            .status()
            .map_err(|e| format!("Failed to call reducer: {}", e))?;

        if !status.success() {
            return Err("Reducer call failed".to_string());
        }

        Ok(())
    }

    /// Save a visual memory - what SAGE saw and learned
    pub fn save_visual_memory(
        &self,
        experience_count: u64,
        person_name: &str,
        avg_brightness: f64,
        avg_r: f64,
        avg_g: f64,
        avg_b: f64,
        color_variance: f64,
        dominant_color: &str,
        edge_strength: f64,
        natural_description: &str,
        visual_concepts: &str,
        context: &str,
    ) -> Result<(), String> {
        if !*self.connected.lock().unwrap() {
            return Err("Not connected to SpacetimeDB".to_string());
        }

        let status = std::process::Command::new("spacetime")
            .args(&[
                "call",
                &self.db_name,
                "save_visual_memory",
                &experience_count.to_string(),
                person_name,
                &avg_brightness.to_string(),
                &avg_r.to_string(),
                &avg_g.to_string(),
                &avg_b.to_string(),
                &color_variance.to_string(),
                dominant_color,
                &edge_strength.to_string(),
                natural_description,
                visual_concepts,
                context,
            ])
            .stderr(std::process::Stdio::null())
            .status()
            .map_err(|e| format!("Failed to call reducer: {}", e))?;

        if !status.success() {
            return Err("Reducer call failed".to_string());
        }

        Ok(())
    }

    /// Start learning a new pattern
    pub fn start_pattern(
        &self,
        pattern: &str,
        generation: u64,
    ) -> Result<(), String> {
        if !*self.connected.lock().unwrap() {
            return Err("Not connected to SpacetimeDB".to_string());
        }

        let status = std::process::Command::new("spacetime")
            .args(&[
                "call",
                &self.db_name,
                "start_pattern",
                pattern,
                &generation.to_string(),
            ])
            .stderr(std::process::Stdio::null())
            .status()
            .map_err(|e| format!("Failed to call reducer: {}", e))?;

        if !status.success() {
            return Err("Reducer call failed".to_string());
        }

        Ok(())
    }

    /// Mark pattern as mastered
    pub fn master_pattern(
        &self,
        pattern: &str,
        generation: u64,
        final_loss: f64,
    ) -> Result<(), String> {
        if !*self.connected.lock().unwrap() {
            return Err("Not connected to SpacetimeDB".to_string());
        }

        let status = std::process::Command::new("spacetime")
            .args(&[
                "call",
                &self.db_name,
                "master_pattern",
                pattern,
                &generation.to_string(),
                &final_loss.to_string(),
            ])
            .stderr(std::process::Stdio::null())
            .status()
            .map_err(|e| format!("Failed to call reducer: {}", e))?;

        if !status.success() {
            return Err("Reducer call failed".to_string());
        }

        Ok(())
    }

    /// Log a training event
    pub fn log_training_event(
        &self,
        generation: u64,
        event_type: &str,
        description: &str,
    ) -> Result<(), String> {
        if !*self.connected.lock().unwrap() {
            return Err("Not connected to SpacetimeDB".to_string());
        }

        let status = std::process::Command::new("spacetime")
            .args(&[
                "call",
                &self.db_name,
                "log_training_event",
                &generation.to_string(),
                event_type,
                description,
            ])
            .stderr(std::process::Stdio::null())
            .status()
            .map_err(|e| format!("Failed to call reducer: {}", e))?;

        if !status.success() {
            return Err("Reducer call failed".to_string());
        }

        Ok(())
    }

    /// Query conversations containing specific concepts
    pub fn query_conversations_with_concept(&self, concept: &str) -> Result<Vec<ConversationRecord>, String> {
        if !*self.connected.lock().unwrap() {
            return Err("Not connected to SpacetimeDB".to_string());
        }

        // Use spacetime sql to query
        let output = std::process::Command::new("spacetime")
            .args(&[
                "sql",
                &self.db_name,
                &format!("SELECT sender, message, sage_response, nca_loss FROM conversations WHERE concepts_extracted LIKE '%{}%' ORDER BY id DESC LIMIT 5", concept),
            ])
            .output()
            .map_err(|e| format!("Failed to query: {}", e))?;

        if !output.status.success() {
            return Ok(Vec::new());  // Return empty if query fails
        }

        // Parse output (simplified - in production would parse properly)
        // For now, just log that we queried
        eprintln!("📚 Queried memories for concept: {}", concept);

        Ok(Vec::new())  // TODO: Parse actual results
    }

    /// Get recent conversations for a specific user from SpacetimeDB
    pub fn get_user_conversations(&self, username: &str, limit: usize) -> Result<Vec<ConversationRecord>, String> {
        if !*self.connected.lock().unwrap() {
            return Err("Not connected to SpacetimeDB".to_string());
        }

        // SpacetimeDB doesn't support WHERE clauses, so fetch all and filter in Rust
        let query = "SELECT sender, message, sage_response, nca_loss FROM conversations";

        let output = std::process::Command::new("spacetime")
            .args(&["sql", &self.db_name, query])
            .output()
            .map_err(|e| format!("Failed to query: {}", e))?;

        if !output.status.success() {
            return Err("SQL query failed".to_string());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut conversations = Vec::new();

        // Parse the SQL output (pipe-delimited table format)
        for line in stdout.lines() {
            // Skip header, separator, warning lines
            if line.contains("sender") || line.contains("---") ||
               line.contains("WARNING") || line.trim().is_empty() {
                continue;
            }

            // Parse: sender | message | sage_response | nca_loss
            let parts: Vec<&str> = line.split('|').map(|s| s.trim()).collect();
            if parts.len() >= 4 {
                let sender = parts[0].trim_matches('"');
                // Filter by username
                if sender == username {
                    conversations.push(ConversationRecord {
                        sender: sender.to_string(),
                        message: parts[1].trim_matches('"').to_string(),
                        sage_response: parts[2].trim_matches('"').to_string(),
                        nca_loss: parts[3].parse().unwrap_or(0.0),
                    });
                }
            }
        }

        // Take only the last N conversations (most recent)
        if conversations.len() > limit {
            conversations = conversations.into_iter().rev().take(limit).collect();
            conversations.reverse();
        }

        eprintln!("📚 Retrieved {} conversations for {} from database", conversations.len(), username);
        Ok(conversations)
    }

    /// Get recent conversations (legacy method - use get_user_conversations instead)
    #[deprecated(note = "Use get_user_conversations instead")]
    pub fn get_recent_conversations(&self, _limit: usize) -> Result<Vec<ConversationRecord>, String> {
        // This method is deprecated - return empty for now
        eprintln!("⚠️  get_recent_conversations is deprecated, use get_user_conversations instead");
        Ok(Vec::new())
    }

    /// Check if SAGE has memory of a concept
    pub fn has_concept_memory(&self, concept: &str) -> bool {
        if !*self.connected.lock().unwrap() {
            return false;
        }

        let output = std::process::Command::new("spacetime")
            .args(&[
                "sql",
                &self.db_name,
                &format!("SELECT concept_name FROM concept_memory WHERE concept_name = '{}'", concept),
            ])
            .output();

        match output {
            Ok(output) => {
                let result = String::from_utf8_lossy(&output.stdout);
                !result.trim().is_empty() && result.contains(concept)
            }
            Err(_) => false,
        }
    }

    // ============================================================================
    // META-LEARNING PERSISTENCE METHODS
    // ============================================================================

    /// Record a meta-learning strategy change
    pub fn record_strategy_change(
        &self,
        generation: u64,
        strategy_name: &str,
        reason: &str,
        performance_before: f64,
    ) -> Result<(), String> {
        if !*self.connected.lock().unwrap() {
            return Err("Not connected to SpacetimeDB".to_string());
        }

        let status = std::process::Command::new("spacetime")
            .args(&[
                "call",
                &self.db_name,
                "record_strategy_change",
                &generation.to_string(),
                strategy_name,
                reason,
                &performance_before.to_string(),
            ])
            .stderr(std::process::Stdio::null())
            .status()
            .map_err(|e| format!("Failed to call reducer: {}", e))?;

        if !status.success() {
            return Err("Reducer call failed".to_string());
        }

        Ok(())
    }

    /// Record hyperparameter configuration from PBT
    pub fn record_hyperparameters(
        &self,
        generation: u64,
        learning_rate: f64,
        batch_size: u32,
        evolution_steps: u32,
        mutation_rate: f64,
        fitness_score: f64,
        parent_id: Option<u64>,
        is_elite: bool,
    ) -> Result<(), String> {
        if !*self.connected.lock().unwrap() {
            return Err("Not connected to SpacetimeDB".to_string());
        }

        let parent_str = parent_id.map(|id| id.to_string()).unwrap_or_default();

        let status = std::process::Command::new("spacetime")
            .args(&[
                "call",
                &self.db_name,
                "record_hyperparameters",
                &generation.to_string(),
                &learning_rate.to_string(),
                &batch_size.to_string(),
                &evolution_steps.to_string(),
                &mutation_rate.to_string(),
                &fitness_score.to_string(),
                &parent_str,
                &is_elite.to_string(),
            ])
            .stderr(std::process::Stdio::null())
            .status()
            .map_err(|e| format!("Failed to call reducer: {}", e))?;

        if !status.success() {
            return Err("Reducer call failed".to_string());
        }

        Ok(())
    }

    /// Record architecture modification
    pub fn record_architecture_change(
        &self,
        generation: u64,
        change_type: &str,
        layer_affected: &str,
        old_size: u32,
        new_size: u32,
        trigger: &str,
        loss_before: f64,
    ) -> Result<(), String> {
        if !*self.connected.lock().unwrap() {
            return Err("Not connected to SpacetimeDB".to_string());
        }

        let status = std::process::Command::new("spacetime")
            .args(&[
                "call",
                &self.db_name,
                "record_architecture_change",
                &generation.to_string(),
                change_type,
                layer_affected,
                &old_size.to_string(),
                &new_size.to_string(),
                trigger,
                &loss_before.to_string(),
            ])
            .stderr(std::process::Stdio::null())
            .status()
            .map_err(|e| format!("Failed to call reducer: {}", e))?;

        if !status.success() {
            return Err("Reducer call failed".to_string());
        }

        Ok(())
    }

    /// Record few-shot adaptation result
    pub fn record_few_shot_adaptation(
        &self,
        generation: u64,
        task_name: &str,
        shots_used: u32,
        adaptation_steps: u32,
        loss_before: f64,
        loss_after: f64,
        transfer_source: &str,
        adaptation_time_ms: u64,
    ) -> Result<(), String> {
        if !*self.connected.lock().unwrap() {
            return Err("Not connected to SpacetimeDB".to_string());
        }

        let status = std::process::Command::new("spacetime")
            .args(&[
                "call",
                &self.db_name,
                "record_few_shot_adaptation",
                &generation.to_string(),
                task_name,
                &shots_used.to_string(),
                &adaptation_steps.to_string(),
                &loss_before.to_string(),
                &loss_after.to_string(),
                transfer_source,
                &adaptation_time_ms.to_string(),
            ])
            .stderr(std::process::Stdio::null())
            .status()
            .map_err(|e| format!("Failed to call reducer: {}", e))?;

        if !status.success() {
            return Err("Reducer call failed".to_string());
        }

        Ok(())
    }

    /// Save optimizer snapshot
    pub fn save_optimizer_snapshot(
        &self,
        generation: u64,
        optimizer_type: &str,
        optimizer_weights_json: &str,
        avg_update_magnitude: f64,
        convergence_speed: f64,
        stability_score: f64,
    ) -> Result<(), String> {
        if !*self.connected.lock().unwrap() {
            return Err("Not connected to SpacetimeDB".to_string());
        }

        let status = std::process::Command::new("spacetime")
            .args(&[
                "call",
                &self.db_name,
                "save_optimizer_snapshot",
                &generation.to_string(),
                optimizer_type,
                optimizer_weights_json,
                &avg_update_magnitude.to_string(),
                &convergence_speed.to_string(),
                &stability_score.to_string(),
            ])
            .stderr(std::process::Stdio::null())
            .status()
            .map_err(|e| format!("Failed to call reducer: {}", e))?;

        if !status.success() {
            return Err("Reducer call failed".to_string());
        }

        Ok(())
    }

    // ============================================================================
    // COGNITIVE ARCHITECTURE - Brain Functions
    // Based on Global Workspace Theory, Society of Mind, and ACT-R
    // ============================================================================

    /// Initialize default brain functions (call once at startup)
    pub fn init_brain_functions(&self) -> Result<(), String> {
        if !*self.connected.lock().unwrap() {
            return Err("Not connected to SpacetimeDB".to_string());
        }

        let status = std::process::Command::new("spacetime")
            .args(&["call", &self.db_name, "init_brain_functions"])
            .stderr(std::process::Stdio::null())
            .status()
            .map_err(|e| format!("Failed to call reducer: {}", e))?;

        if !status.success() {
            return Err("Reducer call failed".to_string());
        }

        Ok(())
    }

    /// Emit a cognitive event (Global Workspace Theory)
    pub fn emit_cognitive_event(
        &self,
        source_function: &str,
        event_type: &str,
        content: &str,
        salience: f64,
        urgency: f64,
        novelty: f64,
    ) -> Result<(), String> {
        if !*self.connected.lock().unwrap() {
            return Err("Not connected to SpacetimeDB".to_string());
        }

        // Escape content for JSON
        let escape_json = |s: &str| -> String {
            let escaped = s
                .replace('\\', "\\\\")
                .replace('"', "\\\"")
                .replace('\n', "\\n")
                .replace('\r', "\\r")
                .replace('\t', "\\t");
            format!("\"{}\"", escaped)
        };

        let status = std::process::Command::new("spacetime")
            .args(&[
                "call",
                &self.db_name,
                "emit_cognitive_event",
                &escape_json(source_function),
                &escape_json(event_type),
                &escape_json(content),
                &salience.to_string(),
                &urgency.to_string(),
                &novelty.to_string(),
            ])
            .stderr(std::process::Stdio::null())
            .status()
            .map_err(|e| format!("Failed to call reducer: {}", e))?;

        if !status.success() {
            return Err("Reducer call failed".to_string());
        }

        Ok(())
    }

    /// Process workspace competition - broadcast top events
    pub fn process_workspace_competition(&self, max_broadcasts: u32) -> Result<(), String> {
        if !*self.connected.lock().unwrap() {
            return Err("Not connected to SpacetimeDB".to_string());
        }

        let status = std::process::Command::new("spacetime")
            .args(&[
                "call",
                &self.db_name,
                "process_workspace_competition",
                &max_broadcasts.to_string(),
            ])
            .stderr(std::process::Stdio::null())
            .status()
            .map_err(|e| format!("Failed to call reducer: {}", e))?;

        if !status.success() {
            return Err("Reducer call failed".to_string());
        }

        Ok(())
    }

    /// Send a message between brain functions (Society of Mind)
    pub fn send_agent_message(
        &self,
        from_function: &str,
        to_function: &str,
        message_type: &str,
        payload: &str,
        correlation_id: Option<u64>,
    ) -> Result<(), String> {
        if !*self.connected.lock().unwrap() {
            return Err("Not connected to SpacetimeDB".to_string());
        }

        let escape_json = |s: &str| -> String {
            let escaped = s
                .replace('\\', "\\\\")
                .replace('"', "\\\"")
                .replace('\n', "\\n")
                .replace('\r', "\\r")
                .replace('\t', "\\t");
            format!("\"{}\"", escaped)
        };

        let corr_id_str = correlation_id
            .map(|id| id.to_string())
            .unwrap_or_else(|| "null".to_string());

        let status = std::process::Command::new("spacetime")
            .args(&[
                "call",
                &self.db_name,
                "send_agent_message",
                &escape_json(from_function),
                &escape_json(to_function),
                &escape_json(message_type),
                &escape_json(payload),
                &corr_id_str,
            ])
            .stderr(std::process::Stdio::null())
            .status()
            .map_err(|e| format!("Failed to call reducer: {}", e))?;

        if !status.success() {
            return Err("Reducer call failed".to_string());
        }

        Ok(())
    }

    /// Update memory activation for a concept (ACT-R style)
    pub fn update_memory_activation(
        &self,
        concept: &str,
        spreading_activation: f64,
        contextual_boost: f64,
    ) -> Result<(), String> {
        if !*self.connected.lock().unwrap() {
            return Err("Not connected to SpacetimeDB".to_string());
        }

        let escape_json = |s: &str| -> String {
            let escaped = s
                .replace('\\', "\\\\")
                .replace('"', "\\\"")
                .replace('\n', "\\n");
            format!("\"{}\"", escaped)
        };

        let status = std::process::Command::new("spacetime")
            .args(&[
                "call",
                &self.db_name,
                "update_memory_activation",
                &escape_json(concept),
                &spreading_activation.to_string(),
                &contextual_boost.to_string(),
            ])
            .stderr(std::process::Stdio::null())
            .status()
            .map_err(|e| format!("Failed to call reducer: {}", e))?;

        if !status.success() {
            return Err("Reducer call failed".to_string());
        }

        Ok(())
    }

    /// Decay all memory activations (call periodically)
    pub fn decay_memory_activations(&self, decay_factor: f64) -> Result<(), String> {
        if !*self.connected.lock().unwrap() {
            return Err("Not connected to SpacetimeDB".to_string());
        }

        let status = std::process::Command::new("spacetime")
            .args(&[
                "call",
                &self.db_name,
                "decay_memory_activations",
                &decay_factor.to_string(),
            ])
            .stderr(std::process::Stdio::null())
            .status()
            .map_err(|e| format!("Failed to call reducer: {}", e))?;

        if !status.success() {
            return Err("Reducer call failed".to_string());
        }

        Ok(())
    }

    /// Set attention focus
    pub fn set_attention_focus(
        &self,
        focus_type: &str,
        focus_target: &str,
        intensity: f64,
        source_event_id: Option<u64>,
    ) -> Result<(), String> {
        if !*self.connected.lock().unwrap() {
            return Err("Not connected to SpacetimeDB".to_string());
        }

        let escape_json = |s: &str| -> String {
            let escaped = s
                .replace('\\', "\\\\")
                .replace('"', "\\\"")
                .replace('\n', "\\n");
            format!("\"{}\"", escaped)
        };

        let event_id_str = source_event_id
            .map(|id| id.to_string())
            .unwrap_or_else(|| "null".to_string());

        let status = std::process::Command::new("spacetime")
            .args(&[
                "call",
                &self.db_name,
                "set_attention_focus",
                &escape_json(focus_type),
                &escape_json(focus_target),
                &intensity.to_string(),
                &event_id_str,
            ])
            .stderr(std::process::Stdio::null())
            .status()
            .map_err(|e| format!("Failed to call reducer: {}", e))?;

        if !status.success() {
            return Err("Reducer call failed".to_string());
        }

        Ok(())
    }

    /// Update cognitive workspace
    pub fn update_workspace(
        &self,
        content_type: &str,
        content: &str,
        source_event_id: u64,
        activation: f64,
    ) -> Result<(), String> {
        if !*self.connected.lock().unwrap() {
            return Err("Not connected to SpacetimeDB".to_string());
        }

        let escape_json = |s: &str| -> String {
            let escaped = s
                .replace('\\', "\\\\")
                .replace('"', "\\\"")
                .replace('\n', "\\n");
            format!("\"{}\"", escaped)
        };

        let status = std::process::Command::new("spacetime")
            .args(&[
                "call",
                &self.db_name,
                "update_workspace",
                &escape_json(content_type),
                &escape_json(content),
                &source_event_id.to_string(),
                &activation.to_string(),
            ])
            .stderr(std::process::Stdio::null())
            .status()
            .map_err(|e| format!("Failed to call reducer: {}", e))?;

        if !status.success() {
            return Err("Reducer call failed".to_string());
        }

        Ok(())
    }

    /// Push a goal onto the goal stack
    pub fn push_goal(
        &self,
        goal_type: &str,
        description: &str,
        priority: f64,
        parent_goal_id: Option<u64>,
    ) -> Result<(), String> {
        if !*self.connected.lock().unwrap() {
            return Err("Not connected to SpacetimeDB".to_string());
        }

        let escape_json = |s: &str| -> String {
            let escaped = s
                .replace('\\', "\\\\")
                .replace('"', "\\\"")
                .replace('\n', "\\n");
            format!("\"{}\"", escaped)
        };

        let parent_str = parent_goal_id
            .map(|id| id.to_string())
            .unwrap_or_else(|| "null".to_string());

        let status = std::process::Command::new("spacetime")
            .args(&[
                "call",
                &self.db_name,
                "push_goal",
                &escape_json(goal_type),
                &escape_json(description),
                &priority.to_string(),
                &parent_str,
            ])
            .stderr(std::process::Stdio::null())
            .status()
            .map_err(|e| format!("Failed to call reducer: {}", e))?;

        if !status.success() {
            return Err("Reducer call failed".to_string());
        }

        Ok(())
    }

    /// Log brain function execution
    pub fn log_brain_function_execution(
        &self,
        function_name: &str,
        trigger_type: &str,
        trigger_source: &str,
        input_summary: &str,
        output_summary: &str,
        execution_ms: u64,
        events_generated: u32,
        success: bool,
        error_message: Option<&str>,
    ) -> Result<(), String> {
        if !*self.connected.lock().unwrap() {
            return Err("Not connected to SpacetimeDB".to_string());
        }

        let escape_json = |s: &str| -> String {
            let escaped = s
                .replace('\\', "\\\\")
                .replace('"', "\\\"")
                .replace('\n', "\\n");
            format!("\"{}\"", escaped)
        };

        let error_str = error_message
            .map(|e| escape_json(e))
            .unwrap_or_else(|| "null".to_string());

        let status = std::process::Command::new("spacetime")
            .args(&[
                "call",
                &self.db_name,
                "log_brain_function_execution",
                &escape_json(function_name),
                &escape_json(trigger_type),
                &escape_json(trigger_source),
                &escape_json(input_summary),
                &escape_json(output_summary),
                &execution_ms.to_string(),
                &events_generated.to_string(),
                &success.to_string(),
                &error_str,
            ])
            .stderr(std::process::Stdio::null())
            .status()
            .map_err(|e| format!("Failed to call reducer: {}", e))?;

        if !status.success() {
            return Err("Reducer call failed".to_string());
        }

        Ok(())
    }

    /// Get cognitive state summary (for debugging)
    pub fn get_cognitive_state_summary(&self) -> Result<(), String> {
        if !*self.connected.lock().unwrap() {
            return Err("Not connected to SpacetimeDB".to_string());
        }

        let status = std::process::Command::new("spacetime")
            .args(&["call", &self.db_name, "get_cognitive_state_summary"])
            .status()
            .map_err(|e| format!("Failed to call reducer: {}", e))?;

        if !status.success() {
            return Err("Reducer call failed".to_string());
        }

        Ok(())
    }

    // ============================================================================
    // WORKSPACE QUERY METHODS - Active Cognitive Architecture (Feature 1)
    // ============================================================================

    /// Spread activation from a concept to its related concepts (ACT-R style)
    pub fn spread_activation(&self, source_concept: &str, decay_factor: f64) -> Result<(), String> {
        if !*self.connected.lock().unwrap() {
            return Err("Not connected to SpacetimeDB".to_string());
        }

        let escape_json = |s: &str| -> String {
            let escaped = s
                .replace('\\', "\\\\")
                .replace('"', "\\\"")
                .replace('\n', "\\n");
            format!("\"{}\"", escaped)
        };

        let status = std::process::Command::new("spacetime")
            .args(&[
                "call",
                &self.db_name,
                "spread_activation",
                &escape_json(source_concept),
                &decay_factor.to_string(),
            ])
            .stderr(std::process::Stdio::null())
            .status()
            .map_err(|e| format!("Failed to call reducer: {}", e))?;

        if !status.success() {
            return Err("Reducer call failed".to_string());
        }

        Ok(())
    }

    /// Boost contextual activation for concepts in current context
    pub fn boost_contextual_activation(&self, concepts: &[String], boost_amount: f64) -> Result<(), String> {
        if !*self.connected.lock().unwrap() {
            return Err("Not connected to SpacetimeDB".to_string());
        }

        // Format as JSON array
        let concepts_json = format!("[{}]",
            concepts.iter()
                .map(|c| format!("\"{}\"", c.replace('"', "\\\"")))
                .collect::<Vec<_>>()
                .join(",")
        );

        let status = std::process::Command::new("spacetime")
            .args(&[
                "call",
                &self.db_name,
                "boost_contextual_activation",
                &concepts_json,
                &boost_amount.to_string(),
            ])
            .stderr(std::process::Stdio::null())
            .status()
            .map_err(|e| format!("Failed to call reducer: {}", e))?;

        if !status.success() {
            return Err("Reducer call failed".to_string());
        }

        Ok(())
    }

    /// Get current workspace contents via SQL query
    pub fn get_workspace_contents(&self) -> Result<Vec<WorkspaceItem>, String> {
        if !*self.connected.lock().unwrap() {
            return Err("Not connected to SpacetimeDB".to_string());
        }

        let output = std::process::Command::new("spacetime")
            .args(&[
                "sql",
                &self.db_name,
                "SELECT slot_number, content_type, content, activation FROM cognitive_workspace ORDER BY activation DESC",
            ])
            .output()
            .map_err(|e| format!("Failed to query: {}", e))?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut items = Vec::new();

        for line in stdout.lines() {
            // Skip header, separator, warning lines
            if line.contains("slot_number") || line.contains("---") ||
               line.contains("WARNING") || line.trim().is_empty() {
                continue;
            }

            // Parse: slot_number | content_type | content | activation
            let parts: Vec<&str> = line.split('|').map(|s| s.trim()).collect();
            if parts.len() >= 4 {
                if let Ok(slot) = parts[0].parse::<u32>() {
                    if let Ok(activation) = parts[3].parse::<f64>() {
                        items.push(WorkspaceItem {
                            slot_number: slot,
                            content_type: parts[1].trim_matches('"').to_string(),
                            content: parts[2].trim_matches('"').to_string(),
                            activation,
                        });
                    }
                }
            }
        }

        Ok(items)
    }

    /// Get high-activation memories above a threshold
    pub fn get_high_activation_memories(&self, threshold: f64, max_results: u32) -> Result<Vec<MemoryActivationItem>, String> {
        if !*self.connected.lock().unwrap() {
            return Err("Not connected to SpacetimeDB".to_string());
        }

        let output = std::process::Command::new("spacetime")
            .args(&[
                "sql",
                &self.db_name,
                &format!(
                    "SELECT concept, total_activation, base_activation, spreading_activation, contextual_boost, recency_boost FROM memory_activations WHERE total_activation >= {} ORDER BY total_activation DESC LIMIT {}",
                    threshold, max_results
                ),
            ])
            .output()
            .map_err(|e| format!("Failed to query: {}", e))?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut items = Vec::new();

        for line in stdout.lines() {
            // Skip header, separator, warning lines
            if line.contains("concept") || line.contains("---") ||
               line.contains("WARNING") || line.trim().is_empty() {
                continue;
            }

            // Parse: concept | total | base | spreading | contextual | recency
            let parts: Vec<&str> = line.split('|').map(|s| s.trim()).collect();
            if parts.len() >= 6 {
                items.push(MemoryActivationItem {
                    concept: parts[0].trim_matches('"').to_string(),
                    total_activation: parts[1].parse().unwrap_or(0.0),
                    base_activation: parts[2].parse().unwrap_or(0.0),
                    spreading_activation: parts[3].parse().unwrap_or(0.0),
                    contextual_boost: parts[4].parse().unwrap_or(0.0),
                    recency_boost: parts[5].parse().unwrap_or(0.0),
                });
            }
        }

        Ok(items)
    }

    /// Get current attention focus
    pub fn get_current_attention_focus(&self) -> Result<Vec<AttentionFocusItem>, String> {
        if !*self.connected.lock().unwrap() {
            return Err("Not connected to SpacetimeDB".to_string());
        }

        let output = std::process::Command::new("spacetime")
            .args(&[
                "sql",
                &self.db_name,
                "SELECT focus_type, focus_target, intensity FROM attention_focus WHERE ended_at IS NULL",
            ])
            .output()
            .map_err(|e| format!("Failed to query: {}", e))?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut items = Vec::new();

        for line in stdout.lines() {
            if line.contains("focus_type") || line.contains("---") ||
               line.contains("WARNING") || line.trim().is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.split('|').map(|s| s.trim()).collect();
            if parts.len() >= 3 {
                items.push(AttentionFocusItem {
                    focus_type: parts[0].trim_matches('"').to_string(),
                    focus_target: parts[1].trim_matches('"').to_string(),
                    intensity: parts[2].parse().unwrap_or(0.0),
                });
            }
        }

        Ok(items)
    }

    /// Get recent broadcast events (thoughts that won workspace access)
    pub fn get_recent_broadcasts(&self, max_results: u32) -> Result<Vec<BroadcastItem>, String> {
        if !*self.connected.lock().unwrap() {
            return Err("Not connected to SpacetimeDB".to_string());
        }

        let output = std::process::Command::new("spacetime")
            .args(&[
                "sql",
                &self.db_name,
                &format!(
                    "SELECT source_function, event_type, content, priority FROM cognitive_events WHERE broadcast = true ORDER BY id DESC LIMIT {}",
                    max_results
                ),
            ])
            .output()
            .map_err(|e| format!("Failed to query: {}", e))?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut items = Vec::new();

        for line in stdout.lines() {
            if line.contains("source_function") || line.contains("---") ||
               line.contains("WARNING") || line.trim().is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.split('|').map(|s| s.trim()).collect();
            if parts.len() >= 4 {
                items.push(BroadcastItem {
                    source_function: parts[0].trim_matches('"').to_string(),
                    event_type: parts[1].trim_matches('"').to_string(),
                    content: parts[2].trim_matches('"').to_string(),
                    priority: parts[3].parse().unwrap_or(0.0),
                });
            }
        }

        Ok(items)
    }

    /// Get a summary of workspace context for LLM prompt injection
    /// Returns formatted string: "Currently thinking about: X, Y, Z"
    pub fn get_workspace_summary(&self) -> Result<String, String> {
        let workspace = self.get_workspace_contents()?;
        let focus = self.get_current_attention_focus()?;
        let memories = self.get_high_activation_memories(1.0, 5)?;

        let mut parts = Vec::new();

        // Add workspace contents (top 3)
        if !workspace.is_empty() {
            let workspace_items: Vec<String> = workspace.iter()
                .take(3)
                .map(|w| format!("{} ({})", w.content, w.content_type))
                .collect();
            parts.push(format!("In mind: {}", workspace_items.join(", ")));
        }

        // Add attention focus
        if !focus.is_empty() {
            let focus_items: Vec<String> = focus.iter()
                .map(|f| format!("{} on {}", f.focus_type, f.focus_target))
                .collect();
            parts.push(format!("Focused on: {}", focus_items.join(", ")));
        }

        // Add high-activation memories
        if !memories.is_empty() {
            let memory_items: Vec<String> = memories.iter()
                .take(3)
                .map(|m| m.concept.clone())
                .collect();
            parts.push(format!("Active concepts: {}", memory_items.join(", ")));
        }

        if parts.is_empty() {
            Ok("Mind is quiet, no active thoughts.".to_string())
        } else {
            Ok(parts.join(" | "))
        }
    }

    // ============================================================================
    // DREAM SYSTEM METHODS
    // ============================================================================

    /// Save a dream journal entry
    pub fn save_dream_journal(
        &self,
        day: u32,
        dream_narrative: &str,
        insights: &[String],
        consolidated_concepts: &[String],
        sleep_quality: f64,
        was_nightmare: bool,
        mood_before: &str,
        mood_after: &str,
        energy_restored: f64,
    ) -> Result<(), String> {
        if !*self.connected.lock().unwrap() {
            return Err("Not connected to SpacetimeDB".to_string());
        }

        // Format as JSON arrays
        let insights_json = format!("[{}]",
            insights.iter()
                .map(|i| format!("\"{}\"", i.replace('"', "\\\"")))
                .collect::<Vec<_>>()
                .join(",")
        );

        let consolidated_json = format!("[{}]",
            consolidated_concepts.iter()
                .map(|c| format!("\"{}\"", c.replace('"', "\\\"")))
                .collect::<Vec<_>>()
                .join(",")
        );

        let escape_json = |s: &str| -> String {
            let escaped = s
                .replace('\\', "\\\\")
                .replace('"', "\\\"")
                .replace('\n', "\\n");
            format!("\"{}\"", escaped)
        };

        let status = std::process::Command::new("spacetime")
            .args(&[
                "call",
                &self.db_name,
                "save_dream_journal",
                &day.to_string(),
                &escape_json(dream_narrative),
                &insights_json,
                &consolidated_json,
                &sleep_quality.to_string(),
                &was_nightmare.to_string(),
                &escape_json(mood_before),
                &escape_json(mood_after),
                &energy_restored.to_string(),
            ])
            .stderr(std::process::Stdio::null())
            .status()
            .map_err(|e| format!("Failed to call reducer: {}", e))?;

        if !status.success() {
            return Err("Reducer call failed".to_string());
        }

        Ok(())
    }

    /// Get recent dreams
    pub fn get_recent_dreams(&self, max_results: u32) -> Result<Vec<DreamRecord>, String> {
        if !*self.connected.lock().unwrap() {
            return Err("Not connected to SpacetimeDB".to_string());
        }

        let output = std::process::Command::new("spacetime")
            .args(&[
                "sql",
                &self.db_name,
                &format!(
                    "SELECT day, dream_narrative, sleep_quality, was_nightmare, insights FROM dream_journal ORDER BY day DESC LIMIT {}",
                    max_results
                ),
            ])
            .output()
            .map_err(|e| format!("Failed to query: {}", e))?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut records = Vec::new();

        for line in stdout.lines() {
            if line.contains("day") || line.contains("---") ||
               line.contains("WARNING") || line.trim().is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.split('|').map(|s| s.trim()).collect();
            if parts.len() >= 5 {
                records.push(DreamRecord {
                    day: parts[0].parse().unwrap_or(0),
                    dream_narrative: parts[1].trim_matches('"').to_string(),
                    sleep_quality: parts[2].parse().unwrap_or(0.5),
                    was_nightmare: parts[3].parse().unwrap_or(false),
                    insights: parts[4].trim_matches('"').to_string(),
                });
            }
        }

        Ok(records)
    }
}

impl Default for SageDbClient {
    fn default() -> Self {
        Self::new("sage-db")
    }
}

// Record types for query results
#[derive(Debug, Clone)]
pub struct ConversationRecord {
    pub sender: String,
    pub message: String,
    pub sage_response: String,
    pub nca_loss: f64,
}

// ============================================================================
// COGNITIVE ARCHITECTURE RECORD TYPES
// ============================================================================

/// Workspace item from cognitive_workspace table
#[derive(Debug, Clone)]
pub struct WorkspaceItem {
    pub slot_number: u32,
    pub content_type: String,
    pub content: String,
    pub activation: f64,
}

/// Memory activation record
#[derive(Debug, Clone)]
pub struct MemoryActivationItem {
    pub concept: String,
    pub total_activation: f64,
    pub base_activation: f64,
    pub spreading_activation: f64,
    pub contextual_boost: f64,
    pub recency_boost: f64,
}

/// Attention focus record
#[derive(Debug, Clone)]
pub struct AttentionFocusItem {
    pub focus_type: String,
    pub focus_target: String,
    pub intensity: f64,
}

/// Broadcast event record
#[derive(Debug, Clone)]
pub struct BroadcastItem {
    pub source_function: String,
    pub event_type: String,
    pub content: String,
    pub priority: f64,
}

/// Dream journal record
#[derive(Debug, Clone)]
pub struct DreamRecord {
    pub day: u32,
    pub dream_narrative: String,
    pub sleep_quality: f64,
    pub was_nightmare: bool,
    pub insights: String,
}

// ============================================================================
// UNIT TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workspace_item_creation() {
        let item = WorkspaceItem {
            slot_number: 0,
            content_type: "concept".to_string(),
            content: "testing".to_string(),
            activation: 0.75,
        };

        assert_eq!(item.slot_number, 0);
        assert_eq!(item.content_type, "concept");
        assert_eq!(item.content, "testing");
        assert!((item.activation - 0.75).abs() < 0.001);
    }

    #[test]
    fn test_memory_activation_item_creation() {
        let item = MemoryActivationItem {
            concept: "philosophy".to_string(),
            total_activation: 2.5,
            base_activation: 1.0,
            spreading_activation: 0.5,
            contextual_boost: 0.5,
            recency_boost: 0.5,
        };

        assert_eq!(item.concept, "philosophy");
        assert!((item.total_activation - 2.5).abs() < 0.001);
        // Verify components sum approximately to total (with some tolerance)
        let sum = item.base_activation + item.spreading_activation +
                  item.contextual_boost + item.recency_boost;
        assert!((sum - item.total_activation).abs() < 0.01);
    }

    #[test]
    fn test_attention_focus_item_creation() {
        let item = AttentionFocusItem {
            focus_type: "conversation".to_string(),
            focus_target: "TestUser".to_string(),
            intensity: 0.95,
        };

        assert_eq!(item.focus_type, "conversation");
        assert_eq!(item.focus_target, "TestUser");
        assert!(item.intensity > 0.9);
    }

    #[test]
    fn test_broadcast_item_creation() {
        let item = BroadcastItem {
            source_function: "perception".to_string(),
            event_type: "sensory_input".to_string(),
            content: "Saw something interesting".to_string(),
            priority: 1.5,
        };

        assert_eq!(item.source_function, "perception");
        assert_eq!(item.event_type, "sensory_input");
        assert!(item.priority > 1.0);
    }

    #[test]
    fn test_client_creation() {
        let client = SageDbClient::new("test-db");
        // Client should be created with connected = true (CLI always connected)
        assert!(client.connected.lock().unwrap().clone());
    }

    #[test]
    fn test_default_client() {
        let client = SageDbClient::default();
        assert_eq!(client.db_name, "sage-db");
    }

    #[test]
    fn test_conversation_record_creation() {
        let record = ConversationRecord {
            sender: "TestUser".to_string(),
            message: "Hello SAGE".to_string(),
            sage_response: "Hello! How can I help?".to_string(),
            nca_loss: 0.1,
        };

        assert_eq!(record.sender, "TestUser");
        assert_eq!(record.message, "Hello SAGE");
        assert!(record.nca_loss < 0.5);
    }

    #[test]
    fn test_workspace_item_ordering() {
        let items = vec![
            WorkspaceItem { slot_number: 2, content_type: "concept".to_string(), content: "low".to_string(), activation: 0.3 },
            WorkspaceItem { slot_number: 0, content_type: "concept".to_string(), content: "high".to_string(), activation: 0.9 },
            WorkspaceItem { slot_number: 1, content_type: "concept".to_string(), content: "mid".to_string(), activation: 0.6 },
        ];

        // Sort by activation descending (as the query would do)
        let mut sorted = items.clone();
        sorted.sort_by(|a, b| b.activation.partial_cmp(&a.activation).unwrap());

        assert_eq!(sorted[0].content, "high");
        assert_eq!(sorted[1].content, "mid");
        assert_eq!(sorted[2].content, "low");
    }

    #[test]
    fn test_memory_activation_above_threshold() {
        let activations = vec![
            MemoryActivationItem { concept: "rust".to_string(), total_activation: 2.5, base_activation: 1.0, spreading_activation: 0.5, contextual_boost: 0.5, recency_boost: 0.5 },
            MemoryActivationItem { concept: "low".to_string(), total_activation: 0.5, base_activation: 0.3, spreading_activation: 0.1, contextual_boost: 0.0, recency_boost: 0.1 },
            MemoryActivationItem { concept: "medium".to_string(), total_activation: 1.5, base_activation: 0.7, spreading_activation: 0.3, contextual_boost: 0.2, recency_boost: 0.3 },
        ];

        let threshold = 1.0;
        let above_threshold: Vec<_> = activations.iter()
            .filter(|a| a.total_activation >= threshold)
            .collect();

        assert_eq!(above_threshold.len(), 2);
        assert!(above_threshold.iter().all(|a| a.total_activation >= threshold));
    }
}
