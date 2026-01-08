//! Response Selector - Selects appropriate responses based on grounded state
//!
//! This implements Layer 3 of the grounded language system.
//! Instead of generating text token-by-token, it selects from learned templates
//! that match the current internal state.

use std::collections::HashMap;
use rand::Rng;
use serde::{Deserialize, Serialize};
use super::concept_som::{ConceptSOM, GroundingState};
use super::sequence_memory::SequenceMemory;

/// Rich context from the inner world for response generation
#[derive(Clone, Debug, Default)]
pub struct ResponseContext {
    /// What SAGE is currently doing (e.g., "reading a book", "looking out the window")
    pub activity: Option<String>,
    /// Where SAGE is (e.g., "the living room", "my garden")
    pub location: Option<String>,
    /// Time description (e.g., "this quiet evening", "early morning")
    pub time_description: Option<String>,
    /// Recent thought or observation
    pub recent_thought: Option<String>,
    /// The book SAGE is currently reading, if any
    pub current_book: Option<String>,
    /// Weather description
    pub weather: Option<String>,
    /// How SAGE is feeling in words
    pub mood_description: Option<String>,
}

/// Relationship stage (mirrored from inner_world for independence)
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RelationshipLevel {
    Stranger,
    Acquaintance,
    Friend,
    CloseFriend,
    Romantic,
}

impl Default for RelationshipLevel {
    fn default() -> Self {
        Self::Stranger
    }
}

/// Input classification for response selection
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum InputType {
    Greeting,
    Question,
    Statement,
    Emotional,
    Farewell,
    /// For specific questions we can't answer - should ask for clarification
    Clarifying,
    Unknown,
}

/// A response template with slots for personalization
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResponseTemplate {
    /// Template ID for tracking
    pub id: u64,
    /// The template text with optional slots like {action}, {feeling}
    pub template: String,
    /// What type of input this responds to
    pub input_type: InputType,
    /// Required energy range (min, max)
    pub energy_range: (f64, f64),
    /// Required valence range (min, max)
    pub valence_range: (f64, f64),
    /// Minimum relationship level required
    pub min_relationship: RelationshipLevel,
    /// Success rate (updated through feedback)
    pub success_rate: f64,
    /// Times this template was used
    pub use_count: u32,
    /// Times it got positive feedback
    pub positive_count: u32,
}

impl ResponseTemplate {
    fn new(id: u64, template: &str, input_type: InputType) -> Self {
        Self {
            id,
            template: template.to_string(),
            input_type,
            energy_range: (0.0, 1.0),
            valence_range: (-1.0, 1.0),
            min_relationship: RelationshipLevel::Stranger,
            success_rate: 0.5,
            use_count: 0,
            positive_count: 0,
        }
    }

    /// Check if this template matches the current state
    pub fn matches_state(&self, state: &GroundingState, relationship: RelationshipLevel) -> bool {
        let energy_ok = state.energy >= self.energy_range.0 && state.energy <= self.energy_range.1;
        let valence_ok = state.valence >= self.valence_range.0 && state.valence <= self.valence_range.1;
        let relationship_ok = relationship >= self.min_relationship;

        energy_ok && valence_ok && relationship_ok
    }

    /// Record feedback (positive = true means the response was well-received)
    pub fn record_feedback(&mut self, positive: bool) {
        self.use_count += 1;
        if positive {
            self.positive_count += 1;
        }
        // Update success rate with EMA
        let feedback = if positive { 1.0 } else { 0.0 };
        self.success_rate = self.success_rate * 0.9 + feedback * 0.1;
    }
}

/// Response selector that chooses templates based on grounded state
#[derive(Clone, Serialize, Deserialize)]
pub struct ResponseSelector {
    /// Templates organized by concept cluster
    templates: HashMap<(usize, usize), Vec<ResponseTemplate>>,
    /// Global templates (not tied to specific concepts)
    global_templates: Vec<ResponseTemplate>,
    /// Recently used template IDs (to avoid repetition)
    recent_templates: Vec<u64>,
    /// Maximum recent templates to track
    max_recent: usize,
    /// Next template ID
    next_id: u64,
    /// Actions that can fill {action} slots
    actions: Vec<ActionTemplate>,
    /// Feeling words that can fill {feeling} slots
    feeling_words: HashMap<String, FeelingWords>,
}

/// Actions for filling {action} slots
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ActionTemplate {
    pub text: String,
    pub energy_range: (f64, f64),
    pub valence_range: (f64, f64),
}

/// Words associated with feeling categories
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FeelingWords {
    pub category: String,
    pub words: Vec<String>,
    pub valence_range: (f64, f64),
}

impl ResponseSelector {
    pub fn new() -> Self {
        let mut selector = Self {
            templates: HashMap::new(),
            global_templates: Vec::new(),
            recent_templates: Vec::new(),
            max_recent: 10,
            next_id: 0,
            actions: Vec::new(),
            feeling_words: HashMap::new(),
        };

        selector.seed_actions();
        selector.seed_feeling_words();

        selector
    }

    /// Seed action templates
    fn seed_actions(&mut self) {
        self.actions = vec![
            ActionTemplate {
                text: "*stretches*".to_string(),
                energy_range: (0.3, 0.7),
                valence_range: (-0.3, 0.5),
            },
            ActionTemplate {
                text: "*yawns*".to_string(),
                energy_range: (0.0, 0.4),
                valence_range: (-0.5, 0.3),
            },
            ActionTemplate {
                text: "*smiles*".to_string(),
                energy_range: (0.3, 1.0),
                valence_range: (0.2, 1.0),
            },
            ActionTemplate {
                text: "*laughs*".to_string(),
                energy_range: (0.4, 1.0),
                valence_range: (0.4, 1.0),
            },
            ActionTemplate {
                text: "*sips tea*".to_string(),
                energy_range: (0.2, 0.6),
                valence_range: (-0.2, 0.6),
            },
            ActionTemplate {
                text: "*tilts head*".to_string(),
                energy_range: (0.3, 0.8),
                valence_range: (-0.3, 0.5),
            },
            ActionTemplate {
                text: "*nods*".to_string(),
                energy_range: (0.2, 0.8),
                valence_range: (0.0, 0.7),
            },
            ActionTemplate {
                text: "*leans in*".to_string(),
                energy_range: (0.4, 0.9),
                valence_range: (0.1, 0.8),
            },
            ActionTemplate {
                text: "*sighs contentedly*".to_string(),
                energy_range: (0.3, 0.6),
                valence_range: (0.3, 0.8),
            },
            ActionTemplate {
                text: "*sighs*".to_string(),
                energy_range: (0.1, 0.5),
                valence_range: (-0.6, 0.1),
            },
            ActionTemplate {
                text: "*looks out window*".to_string(),
                energy_range: (0.2, 0.5),
                valence_range: (-0.3, 0.4),
            },
            ActionTemplate {
                text: "*perks up*".to_string(),
                energy_range: (0.5, 1.0),
                valence_range: (0.3, 1.0),
            },
            ActionTemplate {
                text: "*settles into chair*".to_string(),
                energy_range: (0.1, 0.5),
                valence_range: (0.0, 0.5),
            },
            ActionTemplate {
                text: "*grins*".to_string(),
                energy_range: (0.5, 1.0),
                valence_range: (0.5, 1.0),
            },
            ActionTemplate {
                text: "*raises eyebrow*".to_string(),
                energy_range: (0.3, 0.8),
                valence_range: (-0.2, 0.5),
            },
        ];
    }

    /// Seed feeling word categories
    fn seed_feeling_words(&mut self) {
        self.feeling_words.insert("positive_high".to_string(), FeelingWords {
            category: "positive_high".to_string(),
            words: vec!["great", "wonderful", "fantastic", "amazing", "excited"].iter().map(|s| s.to_string()).collect(),
            valence_range: (0.6, 1.0),
        });

        self.feeling_words.insert("positive_mild".to_string(), FeelingWords {
            category: "positive_mild".to_string(),
            words: vec!["good", "nice", "pretty good", "alright", "content"].iter().map(|s| s.to_string()).collect(),
            valence_range: (0.2, 0.6),
        });

        self.feeling_words.insert("neutral".to_string(), FeelingWords {
            category: "neutral".to_string(),
            words: vec!["okay", "fine", "meh", "so-so", "alright I guess"].iter().map(|s| s.to_string()).collect(),
            valence_range: (-0.2, 0.2),
        });

        self.feeling_words.insert("negative_mild".to_string(), FeelingWords {
            category: "negative_mild".to_string(),
            words: vec!["tired", "a bit off", "not great", "eh", "could be better"].iter().map(|s| s.to_string()).collect(),
            valence_range: (-0.5, -0.2),
        });

        self.feeling_words.insert("negative_low".to_string(), FeelingWords {
            category: "negative_low".to_string(),
            words: vec!["exhausted", "drained", "worn out", "rough", "struggling"].iter().map(|s| s.to_string()).collect(),
            valence_range: (-1.0, -0.5),
        });
    }

    /// Add a template to the global pool
    pub fn add_global_template(&mut self, template: &str, input_type: InputType) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.global_templates.push(ResponseTemplate::new(id, template, input_type));
        id
    }

    /// Add a template with custom state requirements
    pub fn add_template_with_requirements(
        &mut self,
        template: &str,
        input_type: InputType,
        energy_range: (f64, f64),
        valence_range: (f64, f64),
        min_relationship: RelationshipLevel,
    ) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        let mut t = ResponseTemplate::new(id, template, input_type);
        t.energy_range = energy_range;
        t.valence_range = valence_range;
        t.min_relationship = min_relationship;
        self.global_templates.push(t);
        id
    }

    /// Add a template for a specific concept cluster
    pub fn add_concept_template(&mut self, concept: (usize, usize), template: &str, input_type: InputType) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.templates
            .entry(concept)
            .or_insert_with(Vec::new)
            .push(ResponseTemplate::new(id, template, input_type));
        id
    }

    /// Classify the input type
    pub fn classify_input(&self, text: &str, seq_mem: &SequenceMemory) -> InputType {
        self.classify_input_with_context(text, seq_mem, None)
    }

    /// Classify input with optional context to determine if we can answer
    pub fn classify_input_with_context(&self, text: &str, seq_mem: &SequenceMemory, context: Option<&ResponseContext>) -> InputType {
        let lower = text.to_lowercase();

        // Check for farewells
        if lower.contains("bye") || lower.contains("goodbye") || lower.contains("gotta go")
            || lower.contains("talk later") || lower.contains("see you")
        {
            return InputType::Farewell;
        }

        // Check for emotional content
        let emotional_words = ["feel", "feeling", "sad", "happy", "angry", "upset",
            "excited", "worried", "scared", "love", "hate", "miss"];
        if emotional_words.iter().any(|w| lower.contains(w)) {
            return InputType::Emotional;
        }

        // Use sequence memory for greeting/question detection
        if seq_mem.is_greeting(&lower) {
            return InputType::Greeting;
        }

        if seq_mem.is_question(&lower) {
            // Check if this is a specific question we can answer
            // Questions about weather
            if lower.contains("weather") || lower.contains("temperature") || lower.contains("rain")
                || lower.contains("snow") || lower.contains("cold") || lower.contains("warm")
                || lower.contains("sunny") || lower.contains("cloudy")
            {
                // If we have weather context, this is a Question we can answer
                if context.map(|c| c.weather.is_some()).unwrap_or(false) {
                    return InputType::Question;
                }
                // No weather context - need to clarify
                return InputType::Clarifying;
            }

            // Questions about what we're doing/activity
            if lower.contains("doing") || lower.contains("up to") || lower.contains("been up to")
                || lower.contains("working on") || lower.contains("busy")
            {
                return InputType::Question; // We always know what we're doing
            }

            // Questions about how we are/feelings - we can always answer
            if lower.contains("how are you") || lower.contains("how're you")
                || lower.contains("how you doing") || lower.contains("how do you feel")
            {
                return InputType::Question;
            }

            // Questions about reading/books
            if lower.contains("reading") || lower.contains("book") {
                if context.map(|c| c.current_book.is_some()).unwrap_or(false) {
                    return InputType::Question;
                }
                return InputType::Clarifying;
            }

            // Questions asking for external facts we don't know
            let external_knowledge = ["who is", "who was", "what is the", "what's the",
                "when did", "where is", "how many", "how much", "what happened",
                "did you hear", "have you heard", "news about", "score"];
            if external_knowledge.iter().any(|phrase| lower.contains(phrase)) {
                return InputType::Clarifying;
            }

            // Default question - we'll try to answer
            return InputType::Question;
        }

        // Default to statement
        InputType::Statement
    }

    /// Select an appropriate response
    pub fn select_response(
        &mut self,
        state: &GroundingState,
        input_text: &str,
        seq_mem: &SequenceMemory,
        som: &ConceptSOM,
        relationship: RelationshipLevel,
    ) -> String {
        self.select_response_with_context(state, input_text, seq_mem, som, relationship, None)
    }

    /// Select an appropriate response with rich inner world context
    pub fn select_response_with_context(
        &mut self,
        state: &GroundingState,
        input_text: &str,
        seq_mem: &SequenceMemory,
        som: &ConceptSOM,
        relationship: RelationshipLevel,
        context: Option<&ResponseContext>,
    ) -> String {
        let input_type = self.classify_input_with_context(input_text, seq_mem, context);
        let concept = som.get_concept(state);

        // Gather candidate templates
        let mut candidates: Vec<&ResponseTemplate> = Vec::new();

        // Add concept-specific templates
        if let Some(templates) = self.templates.get(&concept) {
            for t in templates {
                if t.input_type == input_type && t.matches_state(state, relationship) {
                    candidates.push(t);
                }
            }
        }

        // Add global templates
        for t in &self.global_templates {
            if t.input_type == input_type && t.matches_state(state, relationship) {
                candidates.push(t);
            }
        }

        // Filter out recently used
        candidates.retain(|t| !self.recent_templates.contains(&t.id));

        // If no candidates, fall back to any matching template
        if candidates.is_empty() {
            for t in &self.global_templates {
                if t.matches_state(state, relationship) {
                    candidates.push(t);
                }
            }
        }

        // If still empty, use a generic fallback
        if candidates.is_empty() {
            return self.generate_fallback_with_context(state, context);
        }

        // Weight by success rate and select
        let selected = self.weighted_select(&candidates);

        // Record usage
        self.recent_templates.push(selected.id);
        if self.recent_templates.len() > self.max_recent {
            self.recent_templates.remove(0);
        }

        // Fill template slots with context
        self.fill_template_with_context(&selected.template, state, context)
    }

    /// Select a template weighted by success rate
    fn weighted_select<'a>(&self, candidates: &[&'a ResponseTemplate]) -> &'a ResponseTemplate {
        let mut rng = rand::thread_rng();

        // Calculate weights (success_rate + small random factor for exploration)
        let weights: Vec<f64> = candidates.iter()
            .map(|t| t.success_rate + rng.gen::<f64>() * 0.2)
            .collect();

        let total: f64 = weights.iter().sum();
        let mut r = rng.gen::<f64>() * total;

        for (i, w) in weights.iter().enumerate() {
            r -= w;
            if r <= 0.0 {
                return candidates[i];
            }
        }

        candidates.last().unwrap_or(&candidates[0])
    }

    /// Fill template slots with state-appropriate content
    fn fill_template(&self, template: &str, state: &GroundingState) -> String {
        self.fill_template_with_context(template, state, None)
    }

    /// Fill template slots with state and rich context
    fn fill_template_with_context(&self, template: &str, state: &GroundingState, context: Option<&ResponseContext>) -> String {
        let mut result = template.to_string();

        // Fill {action}
        if result.contains("{action}") {
            let action = self.select_action(state);
            result = result.replace("{action}", &action);
        }

        // Fill {feeling}
        if result.contains("{feeling}") {
            let feeling = self.select_feeling(state);
            result = result.replace("{feeling}", &feeling);
        }

        // Fill context-based slots if context is provided
        if let Some(ctx) = context {
            // Fill {activity}
            if result.contains("{activity}") {
                let activity = ctx.activity.as_deref().unwrap_or("just thinking");
                result = result.replace("{activity}", activity);
            }

            // Fill {location}
            if result.contains("{location}") {
                let location = ctx.location.as_deref().unwrap_or("here");
                result = result.replace("{location}", location);
            }

            // Fill {time}
            if result.contains("{time}") {
                let time = ctx.time_description.as_deref().unwrap_or("now");
                result = result.replace("{time}", time);
            }

            // Fill {thought}
            if result.contains("{thought}") {
                let thought = ctx.recent_thought.as_deref().unwrap_or("not much");
                result = result.replace("{thought}", thought);
            }

            // Fill {book}
            if result.contains("{book}") {
                let book = ctx.current_book.as_deref().unwrap_or("something interesting");
                result = result.replace("{book}", book);
            }

            // Fill {weather}
            if result.contains("{weather}") {
                let weather = ctx.weather.as_deref().unwrap_or("nice out");
                result = result.replace("{weather}", weather);
            }

            // Fill {mood}
            if result.contains("{mood}") {
                let mood = ctx.mood_description.clone()
                    .unwrap_or_else(|| self.select_feeling(state));
                result = result.replace("{mood}", &mood);
            }
        } else {
            // Remove unfilled context slots with defaults
            result = result.replace("{activity}", "just thinking");
            result = result.replace("{location}", "here");
            result = result.replace("{time}", "");
            result = result.replace("{thought}", "not much");
            result = result.replace("{book}", "something");
            result = result.replace("{weather}", "");
            result = result.replace("{mood}", &self.select_feeling(state));
        }

        // Clean up any double spaces from empty replacements
        while result.contains("  ") {
            result = result.replace("  ", " ");
        }
        result.trim().to_string()
    }

    /// Select an appropriate action for the state
    fn select_action(&self, state: &GroundingState) -> String {
        let mut rng = rand::thread_rng();

        let matching: Vec<_> = self.actions.iter()
            .filter(|a| {
                state.energy >= a.energy_range.0 && state.energy <= a.energy_range.1 &&
                state.valence >= a.valence_range.0 && state.valence <= a.valence_range.1
            })
            .collect();

        if matching.is_empty() {
            // Fallback to closest match
            "*nods*".to_string()
        } else {
            matching[rng.gen_range(0..matching.len())].text.clone()
        }
    }

    /// Select an appropriate feeling word for the state
    fn select_feeling(&self, state: &GroundingState) -> String {
        let mut rng = rand::thread_rng();

        // Find the feeling category that matches valence
        for fw in self.feeling_words.values() {
            if state.valence >= fw.valence_range.0 && state.valence <= fw.valence_range.1 {
                if !fw.words.is_empty() {
                    return fw.words[rng.gen_range(0..fw.words.len())].clone();
                }
            }
        }

        "okay".to_string()
    }

    /// Generate a fallback response when no templates match
    fn generate_fallback(&self, state: &GroundingState) -> String {
        self.generate_fallback_with_context(state, None)
    }

    /// Generate a context-aware fallback response
    fn generate_fallback_with_context(&self, state: &GroundingState, context: Option<&ResponseContext>) -> String {
        let action = self.select_action(state);
        let feeling = self.select_feeling(state);

        if let Some(ctx) = context {
            // Build a rich response using all available context
            let mut parts = Vec::new();

            // Start with action
            parts.push(action.clone());

            // Add activity if available
            if let Some(activity) = &ctx.activity {
                parts.push(format!("I've been {}.", activity));
            }

            // Add weather if available
            if let Some(weather) = &ctx.weather {
                parts.push(format!("It's {} here.", weather));
            }

            // Add mood
            if let Some(mood) = &ctx.mood_description {
                parts.push(format!("Feeling {}.", mood));
            } else {
                parts.push(format!("Feeling {}.", feeling));
            }

            // Add a question to invite conversation
            parts.push("What's on your mind?".to_string());

            return parts.join(" ");
        }

        format!("{} Feeling {}. What's up?", action, feeling)
    }

    /// Record feedback for a template
    pub fn record_feedback(&mut self, template_id: u64, positive: bool) {
        // Check global templates
        for t in &mut self.global_templates {
            if t.id == template_id {
                t.record_feedback(positive);
                return;
            }
        }

        // Check concept templates
        for templates in self.templates.values_mut() {
            for t in templates {
                if t.id == template_id {
                    t.record_feedback(positive);
                    return;
                }
            }
        }
    }

    /// Get statistics about templates
    pub fn stats(&self) -> ResponseStats {
        let total = self.global_templates.len() +
            self.templates.values().map(|v| v.len()).sum::<usize>();

        let avg_success = if total > 0 {
            let sum: f64 = self.global_templates.iter().map(|t| t.success_rate).sum::<f64>() +
                self.templates.values()
                    .flat_map(|v| v.iter())
                    .map(|t| t.success_rate)
                    .sum::<f64>();
            sum / total as f64
        } else {
            0.5
        };

        ResponseStats {
            total_templates: total,
            global_templates: self.global_templates.len(),
            concept_clusters_with_templates: self.templates.len(),
            average_success_rate: avg_success,
        }
    }

    /// Serialize to JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Deserialize from JSON
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

impl Default for ResponseSelector {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about response selection
#[derive(Debug, Clone)]
pub struct ResponseStats {
    pub total_templates: usize,
    pub global_templates: usize,
    pub concept_clusters_with_templates: usize,
    pub average_success_rate: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_classification() {
        let selector = ResponseSelector::new();
        let seq_mem = SequenceMemory::default();

        assert_eq!(selector.classify_input("hi there", &seq_mem), InputType::Greeting);
        assert_eq!(selector.classify_input("bye for now", &seq_mem), InputType::Farewell);
        assert_eq!(selector.classify_input("how are you?", &seq_mem), InputType::Question);
        assert_eq!(selector.classify_input("I feel sad", &seq_mem), InputType::Emotional);
    }

    #[test]
    fn test_action_selection() {
        let selector = ResponseSelector::new();

        // Low energy, low valence
        let tired_state = GroundingState {
            energy: 0.2,
            valence: -0.3,
            ..Default::default()
        };
        let action = selector.select_action(&tired_state);
        // Should get yawn or sigh type actions
        assert!(action.contains("*"));

        // High energy, high valence
        let happy_state = GroundingState {
            energy: 0.8,
            valence: 0.7,
            ..Default::default()
        };
        let action2 = selector.select_action(&happy_state);
        assert!(action2.contains("*"));
    }

    #[test]
    fn test_feeling_selection() {
        let selector = ResponseSelector::new();

        let happy_state = GroundingState {
            valence: 0.8,
            ..Default::default()
        };
        let feeling = selector.select_feeling(&happy_state);
        let positive_words = ["great", "wonderful", "fantastic", "amazing", "excited"];
        assert!(positive_words.contains(&feeling.as_str()));

        let sad_state = GroundingState {
            valence: -0.7,
            ..Default::default()
        };
        let feeling2 = selector.select_feeling(&sad_state);
        let negative_words = ["exhausted", "drained", "worn out", "rough", "struggling"];
        assert!(negative_words.contains(&feeling2.as_str()));
    }

    #[test]
    fn test_template_filling() {
        let selector = ResponseSelector::new();

        let state = GroundingState {
            energy: 0.5,
            valence: 0.5,
            ..Default::default()
        };

        let filled = selector.fill_template("{action} Feeling {feeling} today.", &state);
        assert!(filled.contains("*"));
        assert!(!filled.contains("{action}"));
        assert!(!filled.contains("{feeling}"));
    }

    #[test]
    fn test_feedback_updates_success_rate() {
        let mut selector = ResponseSelector::new();
        let id = selector.add_global_template("Test template", InputType::Statement);

        // Record positive feedback
        for _ in 0..10 {
            selector.record_feedback(id, true);
        }

        let template = selector.global_templates.iter().find(|t| t.id == id).unwrap();
        assert!(template.success_rate > 0.7);
    }
}
