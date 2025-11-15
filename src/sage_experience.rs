// SAGE Experience System - Let SAGE process text, images, and form opinions

use crate::nca::NCA;
use crate::text_encoder::TextEncoder;
use crate::preferences::{PreferenceSystem, Opinion};
use crate::concept_associations::AssociationEngine;
use crate::curiosity::CuriosityEngine;
use crate::self_modification::SelfModificationEngine;
use crate::emergent_goals::EmergentGoalSystem;
use crate::tool_system::ToolRegistry;
use crate::grid::Grid;

/// Main interface for SAGE to experience and process the world
pub struct SageExperience {
    nca: NCA,
    text_encoder: TextEncoder,
    preferences: PreferenceSystem,
    associations: AssociationEngine,
    curiosity: CuriosityEngine,
    self_modifier: SelfModificationEngine,
    goal_system: EmergentGoalSystem,
    tools: ToolRegistry,
    generation: u64,
}

impl SageExperience {
    pub fn new() -> Self {
        Self {
            nca: NCA::new(),
            text_encoder: TextEncoder::new(),
            preferences: PreferenceSystem::new(),
            associations: AssociationEngine::new(),
            curiosity: CuriosityEngine::new(),
            self_modifier: SelfModificationEngine::new(),
            goal_system: EmergentGoalSystem::new(),
            tools: ToolRegistry::default(),
            generation: 0,
        }
    }

    /// Let SAGE experience text and form an opinion
    pub fn experience_text(&mut self, text: &str) -> (Opinion, String) {
        // Encode text into NCA grid
        let grid = self.text_encoder.encode_text(text);

        // Process through NCA to measure "resonance"
        let loss = self.process_with_nca(&grid);

        // Form opinion based on loss
        let opinion = self.preferences.process_experience(
            text.to_string(),
            loss,
            self.generation
        );

        self.generation += 1;

        // Generate response
        let response = self.generate_response(text, &opinion, loss);

        (opinion, response)
    }

    /// Experience text with memory context (for memory-enhanced responses)
    pub fn experience_text_with_memory(&mut self, text: &str, has_prior_memory: bool) -> (Opinion, String) {
        // Encode text into NCA grid
        let grid = self.text_encoder.encode_text(text);

        // Process through NCA to measure "resonance"
        let loss = self.process_with_nca(&grid);

        // Extract key concepts for association discovery
        let concepts: Vec<String> = text
            .split_whitespace()
            .filter(|w| w.len() > 4)  // Meaningful words
            .take(3)
            .map(|w| w.to_lowercase())
            .collect();

        // Record concept losses to discover associations
        for concept in &concepts {
            self.associations.record_concept_loss(concept.clone(), loss);
        }

        // Check if SAGE is curious about any concepts and wants to ask questions
        let mut curious_question = None;
        for concept in &concepts {
            if let Some(question) = self.curiosity.record_curiosity(
                concept.clone(),
                loss,
                self.generation
            ) {
                curious_question = Some(question);
                break;  // One question at a time
            }
        }

        // Form opinion based on loss
        let opinion = self.preferences.process_experience(
            text.to_string(),
            loss,
            self.generation
        );

        self.generation += 1;

        // Generate memory-enhanced response with creative connections
        let mut response = self.generate_creative_response(text, &opinion, loss, has_prior_memory, &concepts);

        // If SAGE is curious, append the question
        if let Some(question) = curious_question {
            response.push_str(&format!(" {}", question));
        }

        (opinion, response)
    }

    /// Generate response that incorporates memory
    #[allow(dead_code)]
    fn generate_memory_enhanced_response(&self, _input: &str, opinion: &Opinion, _loss: f64, has_memory: bool) -> String {
        match opinion {
            Opinion::Like { reason, .. } => {
                if has_memory {
                    format!("✨ {} I feel like we've touched on this before.", reason)
                } else {
                    format!("✨ {}", reason)
                }
            }
            Opinion::Dislike { reason, .. } => {
                if has_memory {
                    format!("⚠️  {} This still doesn't sit right with me.", reason)
                } else {
                    format!("⚠️  {}", reason)
                }
            }
            Opinion::Curious { question } => {
                if has_memory {
                    format!("🤔 {} I remember thinking about this...", question)
                } else {
                    format!("🤔 {}", question)
                }
            }
            Opinion::Neutral { reason } => {
                if has_memory {
                    format!("💭 {} Though it feels somewhat familiar.", reason)
                } else {
                    format!("💭 {}", reason)
                }
            }
        }
    }

    /// Generate response with creative connections
    fn generate_creative_response(&self, _input: &str, opinion: &Opinion, _loss: f64, has_memory: bool, concepts: &[String]) -> String {
        // Find creative connections for the concepts
        let mut creative_connection = None;
        for concept in concepts {
            if let Some(connection) = self.associations.get_creative_connection(concept) {
                creative_connection = Some(connection);
                break;
            }
        }

        match opinion {
            Opinion::Like { reason, .. } => {
                let base = if has_memory {
                    format!("✨ {} I feel like we've touched on this before.", reason)
                } else {
                    format!("✨ {}", reason)
                };

                if let Some(connection) = creative_connection {
                    format!("{} {}", base, connection)
                } else {
                    base
                }
            }
            Opinion::Dislike { reason, .. } => {
                let base = if has_memory {
                    format!("⚠️  {} This still doesn't sit right with me.", reason)
                } else {
                    format!("⚠️  {}", reason)
                };

                if let Some(connection) = creative_connection {
                    format!("{} {}", base, connection)
                } else {
                    base
                }
            }
            Opinion::Curious { question } => {
                let base = if has_memory {
                    format!("🤔 {} I remember thinking about this...", question)
                } else {
                    format!("🤔 {}", question)
                };

                if let Some(connection) = creative_connection {
                    format!("{} {}", base, connection)
                } else {
                    base
                }
            }
            Opinion::Neutral { reason } => {
                let base = if has_memory {
                    format!("💭 {} Though it feels somewhat familiar.", reason)
                } else {
                    format!("💭 {}", reason)
                };

                if let Some(connection) = creative_connection {
                    format!("{} {}", base, connection)
                } else {
                    base
                }
            }
        }
    }

    /// Let SAGE experience a concept and form an opinion
    pub fn experience_concept(&mut self, concept: &str) -> (Opinion, String) {
        let grid = self.text_encoder.encode_concept(concept);
        let loss = self.process_with_nca(&grid);

        let opinion = self.preferences.process_experience(
            concept.to_string(),
            loss,
            self.generation
        );

        self.generation += 1;

        let response = self.generate_response(concept, &opinion, loss);

        (opinion, response)
    }

    /// Process a grid through NCA and measure how well SAGE understands it
    fn process_with_nca(&mut self, target: &Grid) -> f64 {
        // Reset NCA with seed
        self.nca.reset_with_seed();

        // Evolve for a moderate number of steps
        for _ in 0..80 {
            self.nca.step();
        }

        // Measure how close NCA got to the target pattern
        // Lower loss = SAGE "understands" this better
        let loss = self.calculate_grid_loss(&self.nca.grid, target);

        loss
    }

    /// Calculate MSE loss between two grids
    fn calculate_grid_loss(&self, current: &Grid, target: &Grid) -> f64 {
        let mut total_loss = 0.0;
        let mut count = 0;

        for y in 0..current.height {
            for x in 0..current.width {
                for channel in 0..4 {  // RGBA channels
                    let diff = current.cells[y][x][channel] - target.cells[y][x][channel];
                    total_loss += diff * diff;
                    count += 1;
                }
            }
        }

        total_loss / count as f64
    }

    /// Generate SAGE's response based on opinion
    fn generate_response(&self, _input: &str, opinion: &Opinion, _loss: f64) -> String {
        match opinion {
            Opinion::Like { reason, .. } => {
                format!("✨ {}", reason)
            }
            Opinion::Dislike { reason, .. } => {
                format!("⚠️  {}", reason)
            }
            Opinion::Curious { question } => {
                format!("🤔 {}", question)
            }
            Opinion::Neutral { reason } => {
                format!("💭 {}", reason)
            }
        }
    }

    /// Ask SAGE about its personality
    pub fn get_personality(&self) -> String {
        self.preferences.get_personality_summary()
    }

    /// Ask SAGE what it likes
    pub fn get_likes(&self) -> Vec<String> {
        self.preferences.get_likes().to_vec()
    }

    /// Ask SAGE what it dislikes
    pub fn get_dislikes(&self) -> Vec<String> {
        self.preferences.get_dislikes().to_vec()
    }

    /// Get SAGE's familiarity with a concept (0.0 = new, 1.0 = very familiar)
    pub fn get_familiarity(&self, concept: &str) -> f64 {
        self.preferences.get_familiarity(concept)
    }

    /// Get experience count
    pub fn experience_count(&self) -> usize {
        self.preferences.experience_count()
    }

    /// Save SAGE's preferences to file
    pub fn save_preferences(&self, path: &str) -> Result<(), String> {
        use std::fs::File;
        use std::io::Write;

        let json = serde_json::to_string_pretty(&self.preferences)
            .map_err(|e| format!("Serialization error: {}", e))?;

        let mut file = File::create(path)
            .map_err(|e| format!("File create error: {}", e))?;

        file.write_all(json.as_bytes())
            .map_err(|e| format!("Write error: {}", e))?;

        Ok(())
    }

    /// Load SAGE's preferences from file
    pub fn load_preferences(&mut self, path: &str) -> Result<(), String> {
        use std::fs::File;
        use std::io::Read;

        let mut file = File::open(path)
            .map_err(|e| format!("File open error: {}", e))?;

        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .map_err(|e| format!("Read error: {}", e))?;

        self.preferences = serde_json::from_str(&contents)
            .map_err(|e| format!("Deserialization error: {}", e))?;

        Ok(())
    }

    /// Load trained NCA weights to give SAGE knowledge
    pub fn load_knowledge(&mut self, weights_path: &str) -> Result<(), String> {
        self.nca.load_weights_from_file(weights_path)
    }

    /// Save SAGE's current NCA knowledge
    pub fn save_knowledge(&self, weights_path: &str) -> Result<(), String> {
        self.nca.save_weights_to_file(weights_path)
    }

    /// Get discovered associations
    pub fn get_associations(&self) -> String {
        use crate::concept_associations::ConnectionType;

        let associations = self.associations.get_associations();

        if associations.is_empty() {
            return "I haven't discovered any connections yet.".to_string();
        }

        let mut result = format!("I've discovered {} connections:\n", associations.len());

        for (i, assoc) in associations.iter().take(10).enumerate() {
            let connection = match assoc.connection_type {
                ConnectionType::Similar => "≈",
                ConnectionType::Related => "↔",
                ConnectionType::Analogous => "~",
                ConnectionType::Opposite => "⚡",
            };

            result.push_str(&format!(
                "{}. {} {} {} (similarity: {:.0}%)\n",
                i + 1,
                assoc.concept_a,
                connection,
                assoc.concept_b,
                assoc.similarity_score * 100.0
            ));
        }

        result
    }

    /// Get concept clusters
    pub fn get_concept_clusters(&self) -> Vec<Vec<String>> {
        self.associations.get_concept_clusters(2)
    }

    /// Save associations to file
    pub fn save_associations(&self, path: &str) -> Result<(), String> {
        use std::fs::File;
        use std::io::Write;

        let json = self.associations.export_associations();

        let mut file = File::create(path)
            .map_err(|e| format!("File create error: {}", e))?;

        file.write_all(json.as_bytes())
            .map_err(|e| format!("Write error: {}", e))?;

        Ok(())
    }

    /// Load associations from file
    pub fn load_associations(&mut self, path: &str) -> Result<(), String> {
        use std::fs::File;
        use std::io::Read;

        let mut file = File::open(path)
            .map_err(|e| format!("File open error: {}", e))?;

        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .map_err(|e| format!("Read error: {}", e))?;

        self.associations.import_associations(&contents)
    }

    /// Get curiosity summary
    pub fn get_curiosity_summary(&self) -> String {
        self.curiosity.get_curiosity_summary()
    }

    /// Check if SAGE should ask a proactive question now
    pub fn should_ask_proactive_question(&self) -> bool {
        self.curiosity.should_ask_proactive_question(self.generation)
    }

    /// Get a proactive question if SAGE has one
    pub fn get_proactive_question(&self) -> Option<String> {
        self.curiosity.get_proactive_question()
    }

    /// Save curiosity data to file
    pub fn save_curiosity(&self, path: &str) -> Result<(), String> {
        use std::fs::File;
        use std::io::Write;

        let json = self.curiosity.export_curiosity();

        let mut file = File::create(path)
            .map_err(|e| format!("File create error: {}", e))?;

        file.write_all(json.as_bytes())
            .map_err(|e| format!("Write error: {}", e))?;

        Ok(())
    }

    /// Load curiosity data from file
    pub fn load_curiosity(&mut self, path: &str) -> Result<(), String> {
        use std::fs::File;
        use std::io::Read;

        let mut file = File::open(path)
            .map_err(|e| format!("File open error: {}", e))?;

        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .map_err(|e| format!("Read error: {}", e))?;

        self.curiosity.import_curiosity(&contents)
    }

    /// Get current NCA grid (for visualization)
    pub fn get_current_nca_grid(&self) -> &Grid {
        &self.nca.grid
    }

    /// Get NCA for direct access (training, etc.)
    pub fn get_nca_mut(&mut self) -> &mut NCA {
        &mut self.nca
    }

    /// Get NCA for read-only access
    pub fn get_nca(&self) -> &NCA {
        &self.nca
    }

    /// Export NCA grid alpha values for visualization
    pub fn export_grid_alpha_values(&self) -> Vec<f64> {
        self.nca.grid.cells
            .iter()
            .flat_map(|row| row.iter().map(|cell| cell[3])) // Alpha channel is index 3
            .collect()
    }

    /// Check if SAGE should ask a proactive question
    pub fn should_ask_question(&self) -> bool {
        self.curiosity.should_ask_proactive_question(self.generation)
    }

    /// Get a proactive question to ask
    pub fn ask_proactive_question(&self) -> Option<String> {
        self.curiosity.get_proactive_question()
    }

    /// Get current curiosity state
    pub fn get_curiosity_state(&self) -> String {
        self.curiosity.get_curiosity_state()
    }

    /// Check if SAGE should explore autonomously
    pub fn should_explore(&self, idle_count: u64) -> bool {
        self.curiosity.should_explore_autonomously(idle_count)
    }

    /// Generate an exploration prompt
    pub fn generate_exploration(&self) -> Option<String> {
        self.curiosity.generate_exploration_prompt()
    }

    /// Get self-modification summary
    pub fn get_self_modification_state(&self) -> String {
        self.self_modifier.get_modification_summary()
    }

    /// Get SAGE's weaknesses (for introspection)
    pub fn get_weaknesses(&self) -> Vec<String> {
        self.self_modifier.get_weaknesses()
    }

    /// Get SAGE's strengths (for introspection)
    pub fn get_strengths(&self) -> Vec<String> {
        self.self_modifier.get_strengths()
    }

    /// Analyze performance and get self-diagnosis
    pub fn self_diagnose(&self) -> String {
        let diagnosis = self.self_modifier.introspector.diagnose();
        format!("{:?}", diagnosis)
    }

    /// Get current goals summary
    pub fn get_goals_summary(&self) -> String {
        self.goal_system.get_goals_summary()
    }

    /// Get values summary
    pub fn get_values_summary(&self) -> String {
        self.goal_system.get_values_summary()
    }

    /// Form a new goal autonomously
    pub fn form_goal(&mut self, goal_type: crate::emergent_goals::GoalType, description: String, priority: crate::emergent_goals::GoalPriority, motivation: String) -> Option<String> {
        let goal = self.goal_system.form_goal(
            goal_type,
            description.clone(),
            priority,
            motivation,
            self.generation
        );

        goal.map(|g| format!("Formed new goal: {}", g.description))
    }

    /// Check if SAGE should form goals based on curiosity
    pub fn should_form_goals(&self) -> bool {
        // Form goals when sufficiently experienced and curious
        self.experience_count() > 20 && self.curiosity.get_curiosity_summary().len() > 50
    }

    /// Autonomously form goals from curiosity
    pub fn autonomous_goal_formation(&mut self) -> Vec<String> {
        let knowledge_gaps = self.curiosity.get_knowledge_gaps();
        let mut formed_goals = Vec::new();

        // Only form 1 goal at a time
        if let Some(gap) = knowledge_gaps.first() {
            if let Some(msg) = self.form_goal(
                crate::emergent_goals::GoalType::Learning { subject: gap.clone() },
                format!("Deeply understand {}", gap),
                crate::emergent_goals::GoalPriority::Medium,
                format!("I'm very curious about {} and want to learn more", gap)
            ) {
                formed_goals.push(msg);
            }
        }

        formed_goals
    }

    /// Reinforce a value from experience
    pub fn reinforce_value(&mut self, value: &str, strength: f64) {
        self.goal_system.experience_value(value, strength, self.generation);
    }

    /// Use a tool
    pub fn use_tool(&mut self, tool_name: &str, input: &str) -> Result<crate::tool_system::ToolResult, String> {
        self.tools.execute(tool_name, input)
    }

    /// List available tools
    pub fn list_tools(&self) -> Vec<(&str, &str)> {
        self.tools.list_tools()
    }

    /// Get tool usage statistics
    pub fn get_tool_stats(&self) -> Vec<(String, usize)> {
        self.tools.get_usage_stats()
    }

    /// Check if SAGE should use tools for current goal
    /// Returns (tool_name, query) if SAGE should use a tool
    pub fn should_use_tools_for_goal(&self) -> Option<(String, String)> {
        // Get current goal
        if let Some(goal_summary) = self.goal_system.get_current_goal() {
            match goal_summary.goal_type {
                // Learning goals: use knowledge tools
                crate::emergent_goals::GoalType::Learning { .. } => {
                    let knowledge_gaps = self.curiosity.get_knowledge_gaps();
                    if let Some(gap) = knowledge_gaps.first() {
                        // Prefer Wikipedia for factual topics
                        let factual_keywords = ["history", "science", "biology", "physics",
                                              "mathematics", "chemistry", "geography", "definition"];
                        let is_factual = factual_keywords.iter()
                            .any(|kw| gap.to_lowercase().contains(kw));

                        let tool = if is_factual { "wikipedia" } else { "web_search" };
                        return Some((tool.to_string(), gap.clone()));
                    }
                }

                // Exploratory goals: use news or general search
                crate::emergent_goals::GoalType::Exploratory { .. } => {
                    let knowledge_gaps = self.curiosity.get_knowledge_gaps();
                    if let Some(gap) = knowledge_gaps.first() {
                        // Check if it's about current events
                        let news_keywords = ["news", "latest", "current", "today", "recent"];
                        let is_news = news_keywords.iter()
                            .any(|kw| gap.to_lowercase().contains(kw));

                        let tool = if is_news { "news" } else { "web_search" };
                        let query = if is_news { "tech".to_string() } else { gap.clone() };
                        return Some((tool.to_string(), query));
                    }
                }

                // Creative goals: might use time for temporal awareness
                crate::emergent_goals::GoalType::Creative { .. } => {
                    // Sometimes get current time to ground creativity
                    if rand::random::<f64>() < 0.1 {
                        return Some(("time".to_string(), "now".to_string()));
                    }
                }

                _ => {}
            }
        }
        None
    }

    /// Get SAGE's emotional context for LLM conversation
    /// Returns a string describing SAGE's current memories, emotional state, and recent activity
    pub fn get_emotional_context(&self, baseline_concepts: &[String]) -> String {
        let mut context = String::new();

        // Find strongest memories (concepts SAGE knows well)
        let mut memories: Vec<(String, f64)> = baseline_concepts
            .iter()
            .map(|concept| {
                let familiarity = self.get_familiarity(concept);
                (concept.clone(), familiarity)
            })
            .filter(|(_, fam)| *fam > 0.3) // Only meaningful memories
            .collect();

        memories.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        if !memories.is_empty() {
            context.push_str("Strongest memories: ");
            for (concept, familiarity) in memories.iter().take(5) {
                context.push_str(&format!("{} ({:.0}%), ", concept, familiarity * 100.0));
            }
            context = context.trim_end_matches(", ").to_string();
            context.push_str(". ");
        }

        // Emotional state based on average familiarity
        let avg_familiarity: f64 = if !memories.is_empty() {
            memories.iter().take(10).map(|(_, f)| f).sum::<f64>() / memories.len().min(10) as f64
        } else {
            0.0
        };

        let mood = if avg_familiarity > 0.7 {
            "confident and clear-minded"
        } else if avg_familiarity > 0.4 {
            "thoughtful and actively learning"
        } else {
            "curious and exploring new ideas"
        };

        context.push_str(&format!("Current emotional state: {}. ", mood));

        // Preferences
        let likes = self.get_likes();
        let dislikes = self.get_dislikes();

        if !likes.is_empty() {
            context.push_str(&format!("I tend to resonate with: {}. ", likes.join(", ")));
        }

        if !dislikes.is_empty() {
            context.push_str(&format!("I tend to struggle with: {}. ", dislikes.join(", ")));
        }

        // Recent associations
        let associations = self.associations.get_associations();
        if !associations.is_empty() {
            let recent = &associations[0];
            context.push_str(&format!(
                "Recently discovered: {} reminds me of {}. ",
                recent.concept_a, recent.concept_b
            ));
        }

        context
    }

    /// Reinforce patterns for concepts mentioned in a message
    /// This makes SAGE's memory stronger when concepts are discussed
    pub fn reinforce_mentioned_concepts(&mut self, message: &str, baseline_concepts: &[String]) {
        let message_lower = message.to_lowercase();

        for concept in baseline_concepts {
            if message_lower.contains(&concept.to_lowercase()) {
                // Reinforce by processing this concept again
                let _ = self.experience_concept(concept);
            }
        }
    }

    /// Extract NCA activation map - which spatial regions are most active
    /// Returns average activation strength (alpha channel) for different quadrants
    pub fn get_nca_activation_map(&self) -> std::collections::HashMap<String, f64> {
        let grid = &self.nca.grid;
        let mut map = std::collections::HashMap::new();

        // Divide grid into 4 quadrants
        let mid_h = grid.height / 2;
        let mid_w = grid.width / 2;

        let mut quadrants = vec![
            ("top_left", 0.0, 0),
            ("top_right", 0.0, 0),
            ("bottom_left", 0.0, 0),
            ("bottom_right", 0.0, 0),
        ];

        for y in 0..grid.height {
            for x in 0..grid.width {
                let alpha = grid.cells[y][x][3]; // Alpha channel

                if alpha > 0.1 { // Only count living cells
                    let quad_idx = match (y < mid_h, x < mid_w) {
                        (true, true) => 0,   // top_left
                        (true, false) => 1,  // top_right
                        (false, true) => 2,  // bottom_left
                        (false, false) => 3, // bottom_right
                    };

                    quadrants[quad_idx].1 += alpha;
                    quadrants[quad_idx].2 += 1;
                }
            }
        }

        // Calculate averages
        for (name, sum, count) in quadrants {
            let avg = if count > 0 { sum / count as f64 } else { 0.0 };
            map.insert(name.to_string(), avg);
        }

        // Add overall coherence (how organized the pattern is)
        let mut total_alpha = 0.0;
        let mut living_cells = 0;
        for y in 0..grid.height {
            for x in 0..grid.width {
                let alpha = grid.cells[y][x][3];
                if alpha > 0.1 {
                    total_alpha += alpha;
                    living_cells += 1;
                }
            }
        }

        let coherence = if living_cells > 0 {
            total_alpha / living_cells as f64
        } else {
            0.0
        };
        map.insert("coherence".to_string(), coherence);

        map
    }

    /// Get concept strengths - which concepts SAGE knows best
    /// Returns map of concept -> strength (inverse of average loss)
    pub fn get_concept_strengths(&self, baseline_concepts: &[String]) -> std::collections::HashMap<String, f64> {
        use std::collections::HashMap;

        let mut strengths = HashMap::new();

        for concept in baseline_concepts {
            let familiarity = self.get_familiarity(concept);

            // Convert familiarity to strength
            // Familiarity is based on repeated exposure, strength is competence
            if familiarity > 0.0 {
                strengths.insert(concept.clone(), familiarity);
            }
        }

        strengths
    }

    /// Generate personality vector from NCA state + concept strengths
    /// This describes SAGE's current cognitive/emotional state for LLM context
    pub fn get_personality_vector(&self, baseline_concepts: &[String]) -> String {
        let mut vector = String::new();

        // 1. NCA activation patterns
        let activation_map = self.get_nca_activation_map();
        let coherence = activation_map.get("coherence").unwrap_or(&0.0);

        let cognitive_state = if *coherence > 0.7 {
            "highly organized and focused"
        } else if *coherence > 0.4 {
            "actively processing and learning"
        } else {
            "exploring and forming new patterns"
        };

        vector.push_str(&format!("Neural state: {}. ", cognitive_state));

        // 2. Strongest concept memories
        let strengths = self.get_concept_strengths(baseline_concepts);
        let mut sorted: Vec<_> = strengths.iter().collect();
        sorted.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap());

        if !sorted.is_empty() {
            vector.push_str("Strongest patterns: ");
            for (concept, strength) in sorted.iter().take(3) {
                vector.push_str(&format!("{} ({:.0}%), ", concept, **strength * 100.0));
            }
            vector = vector.trim_end_matches(", ").to_string();
            vector.push_str(". ");
        }

        // 3. Spatial activation (which quadrants are active)
        let top_left = activation_map.get("top_left").unwrap_or(&0.0);
        let top_right = activation_map.get("top_right").unwrap_or(&0.0);
        let bottom_left = activation_map.get("bottom_left").unwrap_or(&0.0);
        let bottom_right = activation_map.get("bottom_right").unwrap_or(&0.0);

        let max_activation = top_left.max(*top_right).max(*bottom_left).max(*bottom_right);

        if max_activation > 0.5 {
            let dominant_region = if *top_left == max_activation {
                "analytical (left-brain)"
            } else if *top_right == max_activation {
                "creative (right-brain)"
            } else if *bottom_left == max_activation {
                "grounded and practical"
            } else {
                "intuitive and holistic"
            };

            vector.push_str(&format!("Current mode: {}. ", dominant_region));
        }

        // 4. Experience count and maturity
        let exp_count = self.experience_count();
        let maturity = if exp_count > 100 {
            "experienced and mature"
        } else if exp_count > 30 {
            "developing understanding"
        } else {
            "young and impressionable"
        };

        vector.push_str(&format!("Cognitive maturity: {} ({} experiences). ", maturity, exp_count));

        // 5. Curiosity state
        let curiosity_state = self.curiosity.get_curiosity_state();
        if !curiosity_state.is_empty() && curiosity_state != "Not actively curious about anything right now." {
            vector.push_str(&curiosity_state);
            vector.push(' ');
        }

        // 6. Uncertainty level
        let uncertainty = self.curiosity.get_uncertainty_level();
        if uncertainty > 0.0 {
            let uncertainty_desc = if uncertainty > 0.7 {
                "highly uncertain, seeking clarity"
            } else if uncertainty > 0.4 {
                "moderately curious, exploring"
            } else {
                "mildly curious, refining understanding"
            };
            vector.push_str(&format!("Exploration mode: {}. ", uncertainty_desc));
        }

        // 7. Self-modification state
        let self_mod_summary = self.self_modifier.get_modification_summary();
        if !self_mod_summary.is_empty() {
            vector.push_str(&self_mod_summary);
            vector.push(' ');
        }

        // 8. Introspection - strengths and weaknesses
        let strengths = self.self_modifier.get_strengths();
        if !strengths.is_empty() {
            vector.push_str(&format!("Strengths: {}. ", strengths.join(", ")));
        }

        let weaknesses = self.self_modifier.get_weaknesses();
        if !weaknesses.is_empty() {
            vector.push_str(&format!("Working on: {}. ", weaknesses.join(", ")));
        }

        // 9. Current goals
        let goals_summary = self.goal_system.get_goals_summary();
        if !goals_summary.is_empty() && !goals_summary.starts_with("No active goals") {
            vector.push_str(&goals_summary);
            vector.push(' ');
        }

        // 10. Core values
        let values_summary = self.goal_system.get_values_summary();
        if !values_summary.is_empty() && !values_summary.starts_with("Still discovering") {
            vector.push_str(&values_summary);
            vector.push(' ');
        }

        vector
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_experience_text() {
        let mut sage = SageExperience::new();
        let (opinion, response) = sage.experience_text("hello world");

        // Should have formed some opinion
        assert!(!response.is_empty());
        println!("SAGE's opinion: {}", response);
    }

    #[test]
    fn test_personality_evolution() {
        let mut sage = SageExperience::new();

        // Feed SAGE various concepts
        sage.experience_concept("love");
        sage.experience_concept("joy");
        sage.experience_concept("peace");
        sage.experience_concept("harmony");
        sage.experience_concept("beauty");
        sage.experience_concept("truth");
        sage.experience_concept("wisdom");
        sage.experience_concept("kindness");
        sage.experience_concept("compassion");
        sage.experience_concept("courage");

        // Check personality formation
        let personality = sage.get_personality();
        println!("SAGE's personality after 10 concepts: {}", personality);

        assert!(sage.experience_count() == 10);
    }
}
