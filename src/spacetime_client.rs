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

        let status = std::process::Command::new("spacetime")
            .args(&[
                "call",
                &self.db_name,
                "add_conversation_message",
                sender,
                message,
                sage_response,
                &nca_loss.to_string(),
                concepts_json,
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

    /// Get recent conversations
    pub fn get_recent_conversations(&self, limit: usize) -> Result<Vec<ConversationRecord>, String> {
        if !*self.connected.lock().unwrap() {
            return Err("Not connected to SpacetimeDB".to_string());
        }

        let output = std::process::Command::new("spacetime")
            .args(&[
                "sql",
                &self.db_name,
                &format!("SELECT sender, message, sage_response, nca_loss FROM conversations ORDER BY id DESC LIMIT {}", limit),
            ])
            .output()
            .map_err(|e| format!("Failed to query: {}", e))?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        eprintln!("📚 Retrieved {} recent conversations", limit);

        Ok(Vec::new())  // TODO: Parse actual results
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
