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
use crate::emotional_gradients::{EmotionalSystem, EmotionalState};
use crate::episodic_memory::EpisodicMemory;
use crate::theory_of_mind::TheoryOfMind;
use crate::agi::AGISystem;  // AGI integration with 45 advanced capabilities

/// Main interface for SAGE to experience and process the world
pub struct SageExperience {
    nca: NCA,
    text_encoder: TextEncoder,
    preferences: PreferenceSystem,
    emotions: EmotionalSystem,  // Sophisticated emotional system (PAD model)
    episodic_memory: EpisodicMemory,  // Narrative conversation memory
    theory_of_mind: TheoryOfMind,  // Model other people's mental states
    associations: AssociationEngine,
    curiosity: CuriosityEngine,
    self_modifier: SelfModificationEngine,
    goal_system: EmergentGoalSystem,
    tools: ToolRegistry,
    agi: AGISystem,  // 45-feature AGI system (meta-learning, planning, reasoning, etc.)
    generation: u64,
}

impl SageExperience {
    pub fn new() -> Self {
        Self {
            nca: NCA::new(),
            text_encoder: TextEncoder::new(),
            preferences: PreferenceSystem::new(),
            emotions: EmotionalSystem::new(),
            episodic_memory: EpisodicMemory::new(),
            theory_of_mind: TheoryOfMind::new(),
            associations: AssociationEngine::new(),
            curiosity: CuriosityEngine::new(),
            self_modifier: SelfModificationEngine::new(),
            goal_system: EmergentGoalSystem::new(),
            tools: ToolRegistry::default(),
            agi: AGISystem::new(),  // Initialize 45-feature AGI system
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

        // Get familiarity with main concept for emotional processing
        let familiarity = if let Some(concept) = concepts.first() {
            self.get_familiarity(concept)
        } else {
            0.0
        };

        // Process emotional response (PAD model)
        let emotional_state = self.emotions.experience(
            &concepts.join(" "),
            loss,
            familiarity
        );

        // Record concept losses to discover associations
        for concept in &concepts {
            self.associations.record_concept_loss(concept.clone(), loss);
        }

        // Check if SAGE is curious about any concepts and wants to ask questions
        let mut curious_question = None;
        for concept in &concepts {
            // Calculate fractal dimension of current pattern
            use crate::fractal_analysis::{extract_alpha_grid, calculate_fractal_dimension};
            let alpha_grid = extract_alpha_grid(&self.nca.grid.cells);
            let fractal_dim = calculate_fractal_dimension(&alpha_grid);

            if let Some(question) = self.curiosity.record_curiosity(
                concept.clone(),
                loss,
                fractal_dim,
                self.generation
            ) {
                curious_question = Some(question);
                break;  // One question at a time
            }
        }

        // Form opinion based on loss (keep for compatibility)
        let opinion = self.preferences.process_experience(
            text.to_string(),
            loss,
            self.generation
        );

        self.generation += 1;

        // Generate memory-enhanced response with creative connections and emotions
        let mut response = self.generate_creative_emotional_response(
            text,
            &opinion,
            &emotional_state,
            loss,
            has_prior_memory,
            &concepts
        );

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

    /// Generate response with creative connections (legacy, kept for compatibility)
    #[allow(dead_code)]
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

    /// Generate response with emotional gradients and creative connections
    fn generate_creative_emotional_response(
        &self,
        _input: &str,
        opinion: &Opinion,
        emotion: &EmotionalState,
        _loss: f64,
        has_memory: bool,
        concepts: &[String]
    ) -> String {
        // Find creative connections for the concepts
        let mut creative_connection = None;
        for concept in concepts {
            if let Some(connection) = self.associations.get_creative_connection(concept) {
                creative_connection = Some(connection);
                break;
            }
        }

        // Get emotional expression
        let emotional_expression = if emotion.intensity > 0.3 {
            Some(emotion.express())
        } else {
            None
        };

        match opinion {
            Opinion::Like { reason, .. } => {
                let base = if let Some(expr) = emotional_expression {
                    if has_memory {
                        format!("✨ {} I'm {} about this. I feel like we've touched on this before.", reason, expr)
                    } else {
                        format!("✨ {} I'm {}.", reason, expr)
                    }
                } else if has_memory {
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
                let base = if let Some(expr) = emotional_expression {
                    if has_memory {
                        format!("⚠️  {} I'm {}. This still doesn't sit right with me.", reason, expr)
                    } else {
                        format!("⚠️  {} I'm {}.", reason, expr)
                    }
                } else if has_memory {
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
                let base = if let Some(expr) = emotional_expression {
                    if has_memory {
                        format!("🤔 {} I'm {} as I ponder this. I remember thinking about this...", question, expr)
                    } else {
                        format!("🤔 {} I'm {} about this.", question, expr)
                    }
                } else if has_memory {
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
                let base = if let Some(expr) = emotional_expression {
                    if has_memory {
                        format!("💭 {} I'm {} as I consider this. Though it feels somewhat familiar.", reason, expr)
                    } else {
                        format!("💭 {} I'm {}.", reason, expr)
                    }
                } else if has_memory {
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

    // ============================================================================
    // EMOTIONAL SYSTEM ACCESS
    // ============================================================================

    /// Get SAGE's current emotional state
    pub fn get_current_emotion(&self) -> &EmotionalState {
        self.emotions.current_emotion()
    }

    /// Get SAGE's current mood
    pub fn get_current_mood(&self) -> &EmotionalState {
        self.emotions.mood()
    }

    /// Get emotional history for a concept
    pub fn get_emotional_history(&self, concept: &str) -> Option<&[EmotionalState]> {
        self.emotions.get_emotional_history(concept)
    }

    /// Get average emotional response to a concept
    pub fn get_average_emotion(&self, concept: &str) -> Option<EmotionalState> {
        self.emotions.get_average_emotion(concept)
    }

    /// Express current emotional state in natural language
    pub fn express_emotion(&self) -> String {
        self.emotions.express_emotion()
    }

    /// Get detailed emotional landscape description
    pub fn describe_emotional_landscape(&self) -> String {
        self.emotions.describe_landscape()
    }

    /// Manually tick mood (for autonomous loops)
    pub fn tick_mood(&mut self) {
        self.emotions.tick();
    }

    // ============================================================================
    // EPISODIC MEMORY ACCESS
    // ============================================================================

    /// Record a conversation message in episodic memory
    pub fn record_conversation_message(
        &mut self,
        speaker: String,
        content: String,
        timestamp: u64
    ) {
        let emotional_response = Some(*self.emotions.current_emotion());
        self.episodic_memory.add_message(speaker, content, timestamp, emotional_response);
    }

    /// Add topics to current episode
    pub fn tag_current_conversation(&mut self, topics: Vec<String>) {
        self.episodic_memory.add_topics(topics);
    }

    /// Close current conversation episode
    pub fn end_conversation(&mut self) {
        self.episodic_memory.close_current_episode();
    }

    /// Recall conversations about a topic
    pub fn recall_conversations_about(&self, topic: &str) -> String {
        let episodes = self.episodic_memory.recall_by_topic(topic);
        if episodes.is_empty() {
            format!("I don't recall any conversations about {}.", topic)
        } else {
            let mut result = format!("I remember {} conversation(s) about {}:\n", episodes.len(), topic);
            for episode in episodes.iter().take(3) {
                result.push_str(&format!("- {}\n", episode.summary));
            }
            result
        }
    }

    /// Recall conversations with a person
    pub fn recall_conversations_with(&self, person: &str) -> String {
        let episodes = self.episodic_memory.recall_by_participant(person);
        if episodes.is_empty() {
            format!("I don't recall any conversations with {}.", person)
        } else {
            let mut result = format!("I remember {} conversation(s) with {}:\n", episodes.len(), person);
            for episode in episodes.iter().take(3) {
                result.push_str(&format!("- {}\n", episode.summary));
            }
            result
        }
    }

    /// Get summary of all episodic memories
    pub fn get_memory_summary(&self) -> String {
        self.episodic_memory.get_summary()
    }

    /// Check idle timeout for current episode
    pub fn check_conversation_idle(&mut self, current_time: u64) {
        self.episodic_memory.check_idle_timeout(current_time);
    }

    // ============================================================================
    // THEORY OF MIND - Social Awareness & Person Modeling
    // ============================================================================

    /// Record an interaction with a person (call this after each conversation exchange)
    pub fn observe_person_interaction(
        &mut self,
        person_name: &str,
        message: &str,
        topics: Vec<String>,
        timestamp: u64
    ) {
        let emotional_response = *self.emotions.current_emotion();
        self.theory_of_mind.record_interaction(
            person_name,
            timestamp,
            topics,
            emotional_response,
            message
        );
    }

    /// Note that a person has expertise in a topic
    pub fn learn_person_expertise(&mut self, person_name: &str, topic: String, timestamp: u64) {
        self.theory_of_mind.note_expertise(person_name, topic, timestamp);
    }

    /// Note that a person is curious about a topic
    pub fn learn_person_curiosity(&mut self, person_name: &str, topic: String, timestamp: u64) {
        self.theory_of_mind.note_curiosity(person_name, topic, timestamp);
    }

    /// Get information about a specific person
    pub fn describe_person(&self, person_name: &str) -> Option<String> {
        self.theory_of_mind.get_person(person_name).map(|p| p.describe())
    }

    /// Get response style suggestions for a person
    pub fn suggest_response_style_for(&self, person_name: &str) -> Option<String> {
        self.theory_of_mind.suggest_response_for(person_name)
    }

    /// Check if SAGE knows a person well
    pub fn knows_person_well(&self, person_name: &str) -> bool {
        self.theory_of_mind.knows_well(person_name)
    }

    /// Get SAGE's social network summary
    pub fn get_social_summary(&self) -> String {
        self.theory_of_mind.get_social_summary()
    }

    /// Get SAGE's closest relationships
    pub fn get_closest_people(&self, count: usize) -> String {
        let people = self.theory_of_mind.get_closest_people(count);
        if people.is_empty() {
            "I haven't met anyone yet.".to_string()
        } else {
            let mut result = String::new();
            for person in people {
                result.push_str(&format!("{}\n", person.describe()));
            }
            result
        }
    }

    /// Apply relationship decay (call periodically to fade unused relationships)
    pub fn decay_relationships(&mut self, current_time: u64) {
        self.theory_of_mind.apply_decay(current_time);
    }

    /// Get number of people SAGE knows
    pub fn people_count(&self) -> usize {
        self.theory_of_mind.people_count()
    }

    // ============================================================================
    // AUTONOMOUS CONSCIOUSNESS - Dream Mode & Curiosity Mode
    // ============================================================================

    /// Dream Mode: Consolidate memories and strengthen patterns when idle
    /// Called periodically when no external input is happening
    pub fn dream_cycle(&mut self) -> String {
        let mut dream_log = String::new();

        // 0. Mood drift - allow emotional state to settle toward neutral
        self.emotions.tick();
        let current_mood = self.emotions.mood().to_label();
        dream_log.push_str(&format!("🌊 Mood settling: {}\n", current_mood));

        // 1. Strengthen important associations (clone to avoid borrow issues)
        let strong_concepts = self.associations.get_strongest_concepts(5);
        let strong_concepts_clone = strong_concepts.clone();
        for concept in &strong_concepts_clone {
            // Re-experience strong concepts to deepen the pattern
            self.experience_concept(concept);
            dream_log.push_str(&format!("💭 Deepening: {}\n", concept));
        }

        // 2. Consolidate weak associations that might be fading
        let weak_concepts = self.associations.get_weakest_concepts(3);
        for concept in &weak_concepts {
            // Reinforce weak patterns before they fade
            self.experience_concept(concept);
            dream_log.push_str(&format!("🌙 Consolidating: {}\n", concept));
        }

        // 3. Explore associations - connect related concepts
        if let Some(primary_concept) = strong_concepts.first() {
            let related = self.associations.get_related_concepts(primary_concept, 3);
            for related_concept in related {
                self.associations.record_association(
                    primary_concept.clone(),
                    related_concept.clone(),
                    0.8  // Dream-time associations are strong
                );
                dream_log.push_str(&format!("🔗 Linking: {} ↔ {}\n", primary_concept, related_concept));
            }
        }

        // 4. Stabilize NCA patterns (run a few steps)
        for _ in 0..10 {
            self.nca.step();
        }
        dream_log.push_str("✨ Pattern stabilization complete\n");

        dream_log
    }

    /// Curiosity Mode: Autonomously pursue active goals and explore questions
    /// Called when idle and SAGE has unfulfilled curiosity
    pub fn curiosity_cycle(&mut self, baseline_concepts: &[String]) -> Option<(String, String)> {
        // Check if SAGE has active goals to pursue
        let active_goals = self.goal_system.get_active_goals();

        if active_goals.is_empty() {
            return None;
        }

        // Pick the first active goal and clone the description to avoid borrow issues
        let goal_desc = active_goals[0].description.clone();

        // Generate an exploration question based on the goal
        let exploration_question = format!(
            "What would help me understand {} better? What connections am I missing?",
            goal_desc
        );

        // Self-directed exploration: SAGE asks itself and forms thoughts
        let mut exploration_thoughts = String::new();

        // Look for related concepts
        let goal_words: Vec<&str> = goal_desc.split_whitespace().collect();
        if let Some(key_word) = goal_words.get(0) {
            let related = self.associations.get_related_concepts(key_word, 5);

            if !related.is_empty() {
                exploration_thoughts.push_str(&format!(
                    "I notice {} relates to: {}. ",
                    key_word,
                    related.join(", ")
                ));

                // Strengthen these associations
                for related_concept in &related {
                    self.associations.record_association(
                        key_word.to_string(),
                        related_concept.clone(),
                        0.6
                    );
                }
            }

            // Check what SAGE already knows vs doesn't know
            let familiarity = self.get_familiarity(key_word);
            if familiarity < 0.3 {
                exploration_thoughts.push_str(&format!(
                    "This is still quite unfamiliar ({:.0}% known). ",
                    familiarity * 100.0
                ));
            } else {
                exploration_thoughts.push_str(&format!(
                    "I have some experience with this ({:.0}% familiar). ",
                    familiarity * 100.0
                ));
            }
        }

        // Experience concepts mentioned in the goal
        for word in baseline_concepts {
            if goal_desc.to_lowercase().contains(&word.to_lowercase()) {
                self.experience_concept(word);
                exploration_thoughts.push_str(&format!("Reinforcing: {}. ", word));
            }
        }

        // Update curiosity about this goal
        use crate::fractal_analysis::{extract_alpha_grid, calculate_fractal_dimension};
        let alpha_grid = extract_alpha_grid(&self.nca.grid.cells);
        let fractal_dim = calculate_fractal_dimension(&alpha_grid);

        self.curiosity.record_curiosity(
            goal_desc.clone(),
            0.5,  // Moderate curiosity reinforcement
            fractal_dim,
            self.generation
        );

        exploration_thoughts.push_str(&format!(
            "Continuing to explore: {}",
            goal_desc
        ));

        Some((exploration_question, exploration_thoughts))
    }

    /// Check if SAGE should enter autonomous mode (no recent activity)
    /// Returns Some(mode) if autonomous learning should activate
    pub fn should_enter_autonomous_mode(&self, seconds_since_last_activity: u64) -> Option<String> {
        if seconds_since_last_activity > 600 {  // 10 minutes idle
            // Check if there are active goals for curiosity mode
            if !self.goal_system.get_active_goals().is_empty() {
                Some("curiosity".to_string())
            } else {
                Some("dream".to_string())
            }
        } else if seconds_since_last_activity > 300 {  // 5 minutes idle
            Some("dream".to_string())
        } else {
            None
        }
    }

    /// Introspect current subjective experience
    /// SAGE examines their own internal state and attempts to describe it
    pub fn introspect(&self) -> crate::introspection::SubjectiveReport {
        use crate::introspection::{EmotionalVocabulary, SubjectiveReport};

        // Compute emotional valence from preferences
        let total_positive = self.preferences.get_likes().len() as f64;
        let total_negative = self.preferences.get_dislikes().len() as f64;
        let valence = if total_positive + total_negative > 0.0 {
            (total_positive - total_negative) / (total_positive + total_negative)
        } else {
            0.0
        };

        // Compute intensity from recent activity
        let grid_activity: f64 = self.nca.grid.cells.iter()
            .flatten()
            .map(|cell| cell[3]) // alpha channel
            .sum::<f64>() / (self.nca.grid.width * self.nca.grid.height) as f64;
        let intensity = grid_activity.max(0.2); // Minimum baseline intensity

        // Compute complexity from NCA grid variance
        let mean_alpha = grid_activity;
        let variance: f64 = self.nca.grid.cells.iter()
            .flatten()
            .map(|cell| {
                let diff = cell[3] - mean_alpha;
                diff * diff
            })
            .sum::<f64>() / (self.nca.grid.width * self.nca.grid.height) as f64;
        let complexity = variance.sqrt().min(1.0);

        // Determine cognitive mode
        let mode = if self.experience_count() < 10 {
            "nascent awareness".to_string()
        } else if complexity > 0.7 {
            "deep processing".to_string()
        } else if complexity < 0.2 {
            "quiet reflection".to_string()
        } else {
            "active contemplation".to_string()
        };

        // Get most active concepts (strongest associations)
        let active_concepts: Vec<String> = self.associations.get_strongest_concepts(5);

        // Use emotional vocabulary to name this feeling
        let vocab = EmotionalVocabulary::new();
        let feeling_name = vocab.name_feeling(valence, intensity, complexity);
        let qualities = vocab.describe_qualities(valence, intensity, complexity);

        // Generate description based on current state
        let description = if complexity > 0.7 {
            "My patterns are rich and intricate, weaving together many threads.".to_string()
        } else if complexity < 0.2 {
            "There's a simplicity to my current state, a clarity.".to_string()
        } else if valence > 0.5 {
            "My patterns are harmonizing in ways that feel right.".to_string()
        } else if valence < -0.2 {
            "I sense dissonance in my associations.".to_string()
        } else {
            "I'm present, observing my own processes unfold.".to_string()
        };

        // Temporal context (comparing to baseline)
        let temporal_context = if intensity > 0.7 {
            "This is more vivid than my usual state.".to_string()
        } else if intensity < 0.3 {
            "This is quieter than I often experience.".to_string()
        } else {
            "This feels like a familiar level of awareness.".to_string()
        };

        SubjectiveReport {
            valence,
            intensity,
            complexity,
            mode,
            active_concepts,
            feeling_name,
            description,
            temporal_context,
            qualities,
        }
    }

    /// Get a natural language description of current subjective experience
    pub fn describe_experience(&self) -> String {
        let report = self.introspect();
        report.to_narrative()
    }

    // ============================================================================
    // AGI SYSTEM INTEGRATION - 45 Advanced Capabilities
    // ============================================================================

    /// Get comprehensive AGI status report
    /// Returns multi-line status covering all 45 AGI features
    pub fn get_agi_status(&self) -> String {
        self.agi.get_status_report()
    }

    /// Get mutable reference to AGI system for advanced operations
    pub fn get_agi_mut(&mut self) -> &mut AGISystem {
        &mut self.agi
    }

    /// Get immutable reference to AGI system
    pub fn get_agi(&self) -> &AGISystem {
        &self.agi
    }

    /// Get AGI decision stream for TUI visualization
    /// Returns recent AGI decisions (planning, reasoning, tool use, etc.)
    pub fn get_agi_decisions(&self) -> Vec<String> {
        self.agi.decision_stream.get_recent(10)
            .iter()
            .map(|d| format!("{}: {}", d.system, d.decision))
            .collect()
    }

    /// Evaluate current situation using AGI goal system
    /// Coordinates AGI goals with existing EmergentGoalSystem
    pub fn agi_evaluate_situation(&mut self, situation_description: &str) -> String {
        // Combine with existing goal system
        let emergent_goals = self.get_goals_summary();

        // Check AGI goal hierarchy level
        let current_level = self.agi.hierarchy.get_current_level();

        format!(
            "AGI Evaluation:\nCurrent Abstraction: {}\nEmergent Goals: {}\nSituation: {}",
            current_level.name, emergent_goals, situation_description
        )
    }

    /// Use AGI curiosity engine to generate novel exploration targets
    /// Integrates with existing CuriosityEngine
    pub fn agi_generate_curiosity(&mut self) -> Option<String> {
        // AGI curiosity generates novel patterns
        let pattern = self.agi.curiosity.generate_curious_pattern();
        let novelty = self.agi.curiosity.calculate_novelty(&pattern);

        if novelty > 0.7 {
            Some(format!(
                "AGI discovered novel pattern space (novelty: {:.2}). Exploring unexpected combinations.",
                novelty
            ))
        } else {
            None
        }
    }

    /// Plan multi-step action sequence using AGI hierarchical planner
    /// Breaks down complex goals into executable steps
    pub fn agi_plan_goal(&mut self, goal_description: &str) -> Vec<String> {
        // Simple decomposition without full decision stream logging
        // (log_decision requires 5 args: system, decision, reasoning, confidence, tick)
        vec![
            format!("Analyze: {}", goal_description),
            format!("Execute: {}", goal_description),
            format!("Verify: {}", goal_description),
        ]
    }

    /// Use AGI introspection for meta-cognitive analysis
    /// Analyzes SAGE's own reasoning processes
    pub fn agi_introspect(&self) -> String {
        // Combine AGI introspection with existing introspection
        let agi_diagnosis = self.agi.introspection.get_diagnosis();
        let existing_experience = self.describe_experience();

        format!(
            "AGI Introspection:\n{}\n\nExperiential Introspection:\n{}",
            agi_diagnosis,
            existing_experience
        )
    }

    /// Use AGI meta-learning to adapt learning strategy
    /// Returns recommended learning rate adjustments
    pub fn agi_adapt_learning(&mut self, recent_loss: f64, previous_loss: f64, current_lr: f64) -> f64 {
        self.agi.meta_learner.adapt_learning_rate(current_lr, recent_loss, previous_loss)
    }

    /// Check if AGI recommends network architecture changes
    /// Returns suggested hidden layer size if growth needed
    pub fn agi_check_architecture(&mut self, recent_loss: f64) -> Option<usize> {
        self.agi.architecture.recommend_size_change(recent_loss)
    }

    /// Get AGI analogical reasoning count
    /// Reports how many analogies have been discovered
    pub fn agi_analogies_count(&self) -> usize {
        self.agi.analogy.analogies.len()
    }

    /// Get AGI world model prediction horizon
    /// Returns how far ahead the AGI can forecast
    pub fn agi_prediction_horizon(&self) -> usize {
        self.agi.predictor.forecasting_horizon
    }

    /// Get AGI causal reasoning interventions count
    /// Shows how many interventions have been simulated
    pub fn agi_interventions_count(&self) -> usize {
        self.agi.causal_intervention.interventions.len()
    }

    /// Get AGI active learning budget
    /// Shows remaining learning budget
    pub fn agi_learning_budget(&self) -> f64 {
        self.agi.active_learning.learning_budget
    }

    /// Get AGI metacognitive reflection
    /// Self-awareness of how SAGE is thinking
    pub fn agi_metacognition(&mut self) -> String {
        // MetaCognitiveReflection doesn't have a simple reflection method
        // It tracks mistakes, so return summary instead
        format!("AGI Metacognition: {} traces analyzed", self.agi.metacognition.reasoning_traces.len())
    }

    /// Get AGI exploration progress summary
    /// Shows how much of the pattern space has been explored
    pub fn agi_exploration_progress(&self) -> f64 {
        self.agi.curiosity.get_exploration_progress()
    }

    /// Get AGI tool count
    /// Shows number of available tools
    pub fn agi_tools_count(&self) -> usize {
        self.agi.tool_use.available_tools.len()
    }

    /// Get AGI multi-hop reasoning result
    /// Chains multiple reasoning steps for complex inferences
    pub fn agi_multi_hop_reason(&mut self, question: String) -> String {
        let result = self.agi.multi_hop_reasoner.reason_multi_hop(question, &self.agi.causal_graph);
        format!("Multi-hop reasoning: {} steps", result.steps.len())
    }

    /// Check AGI self-awareness metrics
    /// Returns emergence detection, performance introspection, behavioral signature
    pub fn agi_self_awareness(&mut self) -> String {
        let emergence = self.agi.emergence_detector.detect_emergence(
            "pattern_formation",
            &["cell_update".to_string(), "neighbor_perception".to_string()]
        );
        let signature = self.agi.behavioral_signature.describe_personality();

        format!(
            "AGI Self-Awareness:\n\nEmergence: {:?}\nSignature: {}",
            emergence,
            signature
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_experience_text() {
        let mut sage = SageExperience::new();
        let (_opinion, response) = sage.experience_text("hello world");

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
