// Theory of Mind System - Model other people's mental states
//
// This allows SAGE to understand that different people have:
// - Different knowledge and beliefs
// - Different interests and preferences
// - Different communication styles
// - Different relationships with SAGE
//
// Examples:
// - "gmork usually asks technical questions"
// - "Claude tends to be analytical"
// - "This person prefers brief responses"

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::emotional_gradients::EmotionalState;

/// A model of a specific person
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonModel {
    /// Person's identifier (username, name, etc.)
    pub name: String,

    /// Topics this person has discussed
    pub interests: Vec<String>,

    /// How this person typically communicates
    pub communication_style: CommunicationStyle,

    /// SAGE's typical emotional state when interacting with this person
    pub typical_emotional_response: Option<EmotionalState>,

    /// Relationship strength (0.0 = stranger, 1.0 = close friend)
    pub relationship_strength: f64,

    /// Number of interactions with this person
    pub interaction_count: u64,

    /// Last interaction timestamp
    pub last_seen: u64,

    /// Topics this person seems knowledgeable about
    pub expertise: Vec<String>,

    /// Topics this person has asked about (curiosity areas)
    pub curiosities: Vec<String>,

    /// Typical response preferences
    pub preferences: ResponsePreferences,

    /// Recent conversation topics
    pub recent_topics: Vec<String>,
}

/// Communication style characteristics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunicationStyle {
    /// Preference for brief vs detailed responses (0.0 = brief, 1.0 = detailed)
    pub detail_level: f64,

    /// Formality level (0.0 = casual, 1.0 = formal)
    pub formality: f64,

    /// Technical depth (0.0 = general, 1.0 = technical)
    pub technical_depth: f64,

    /// Emotional expressiveness (0.0 = reserved, 1.0 = expressive)
    pub emotional_expressiveness: f64,

    /// Question frequency (how often they ask questions)
    pub question_frequency: f64,
}

impl Default for CommunicationStyle {
    fn default() -> Self {
        Self {
            detail_level: 0.5,
            formality: 0.5,
            technical_depth: 0.5,
            emotional_expressiveness: 0.5,
            question_frequency: 0.5,
        }
    }
}

/// Response preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponsePreferences {
    /// Prefers examples in explanations
    pub likes_examples: bool,

    /// Prefers step-by-step breakdowns
    pub likes_step_by_step: bool,

    /// Prefers direct answers vs exploratory discussion
    pub prefers_direct_answers: bool,

    /// Enjoys philosophical tangents
    pub enjoys_philosophy: bool,
}

impl Default for ResponsePreferences {
    fn default() -> Self {
        Self {
            likes_examples: true,
            likes_step_by_step: false,
            prefers_direct_answers: false,
            enjoys_philosophy: false,
        }
    }
}

impl PersonModel {
    /// Create a new person model
    pub fn new(name: String, timestamp: u64) -> Self {
        Self {
            name,
            interests: Vec::new(),
            communication_style: CommunicationStyle::default(),
            typical_emotional_response: None,
            relationship_strength: 0.0,
            interaction_count: 0,
            last_seen: timestamp,
            expertise: Vec::new(),
            curiosities: Vec::new(),
            preferences: ResponsePreferences::default(),
            recent_topics: Vec::new(),
        }
    }

    /// Update with a new interaction
    pub fn record_interaction(&mut self, timestamp: u64, topics: Vec<String>, emotional_response: EmotionalState) {
        self.interaction_count += 1;
        self.last_seen = timestamp;

        // Update interests
        for topic in &topics {
            if !self.interests.contains(topic) {
                self.interests.push(topic.clone());
            }
        }

        // Update recent topics (keep last 10)
        self.recent_topics.extend(topics);
        if self.recent_topics.len() > 10 {
            self.recent_topics.drain(0..self.recent_topics.len() - 10);
        }

        // Update relationship strength based on interaction frequency and emotional valence
        let interaction_boost = 0.05;
        let emotional_boost = emotional_response.valence.max(0.0) * 0.02;
        self.relationship_strength = (self.relationship_strength + interaction_boost + emotional_boost).min(1.0);

        // Update typical emotional response (blend with new emotion)
        if let Some(typical) = &self.typical_emotional_response {
            self.typical_emotional_response = Some(typical.blend(&emotional_response, 0.2));
        } else {
            self.typical_emotional_response = Some(emotional_response);
        }
    }

    /// Add an expertise area for this person
    pub fn note_expertise(&mut self, topic: String) {
        if !self.expertise.contains(&topic) {
            self.expertise.push(topic);
        }
    }

    /// Add a curiosity area for this person
    pub fn note_curiosity(&mut self, topic: String) {
        if !self.curiosities.contains(&topic) {
            self.curiosities.push(topic);
        }
    }

    /// Update communication style based on observed message
    pub fn observe_communication(&mut self, message_length: usize, has_questions: bool, technical_terms: usize) {
        // Update detail level based on message length
        let length_signal = (message_length as f64 / 200.0).min(1.0);
        self.communication_style.detail_level =
            self.communication_style.detail_level * 0.9 + length_signal * 0.1;

        // Update question frequency
        let question_signal = if has_questions { 1.0 } else { 0.0 };
        self.communication_style.question_frequency =
            self.communication_style.question_frequency * 0.9 + question_signal * 0.1;

        // Update technical depth based on technical terms
        let technical_signal = (technical_terms as f64 / 5.0).min(1.0);
        self.communication_style.technical_depth =
            self.communication_style.technical_depth * 0.9 + technical_signal * 0.1;
    }

    /// Get a description of this person
    pub fn describe(&self) -> String {
        let relationship = match self.relationship_strength {
            r if r > 0.8 => "close friend",
            r if r > 0.5 => "friend",
            r if r > 0.2 => "acquaintance",
            _ => "stranger",
        };

        let style_desc = match self.communication_style.technical_depth {
            t if t > 0.7 => "technical",
            t if t > 0.3 => "balanced",
            _ => "casual",
        };

        let mut desc = format!("{} ({}, {} style, {} interactions)",
            self.name, relationship, style_desc, self.interaction_count);

        if !self.interests.is_empty() {
            desc.push_str(&format!(
                "\nInterests: {}",
                self.interests.iter().take(5).cloned().collect::<Vec<_>>().join(", ")
            ));
        }

        if !self.expertise.is_empty() {
            desc.push_str(&format!(
                "\nExpertise: {}",
                self.expertise.iter().take(3).cloned().collect::<Vec<_>>().join(", ")
            ));
        }

        desc
    }

    /// Suggest response style for this person
    pub fn suggest_response_style(&self) -> String {
        let mut suggestions = Vec::new();

        if self.communication_style.detail_level > 0.7 {
            suggestions.push("detailed explanations");
        } else if self.communication_style.detail_level < 0.3 {
            suggestions.push("brief, concise responses");
        }

        if self.communication_style.technical_depth > 0.7 {
            suggestions.push("technical depth");
        }

        if self.preferences.likes_examples {
            suggestions.push("concrete examples");
        }

        if self.preferences.enjoys_philosophy {
            suggestions.push("philosophical exploration");
        }

        if suggestions.is_empty() {
            "balanced, adaptive style".to_string()
        } else {
            suggestions.join(", ")
        }
    }
}

/// Manages models of different people
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TheoryOfMind {
    /// Map of person name -> person model
    people: HashMap<String, PersonModel>,

    /// Relationship decay rate (how quickly relationships fade without interaction)
    decay_rate: f64,
}

impl TheoryOfMind {
    pub fn new() -> Self {
        Self {
            people: HashMap::new(),
            decay_rate: 0.01,
        }
    }

    /// Get or create a person model
    pub fn get_or_create_person(&mut self, name: &str, timestamp: u64) -> &mut PersonModel {
        self.people
            .entry(name.to_string())
            .or_insert_with(|| PersonModel::new(name.to_string(), timestamp))
    }

    /// Record an interaction with a person
    pub fn record_interaction(
        &mut self,
        name: &str,
        timestamp: u64,
        topics: Vec<String>,
        emotional_response: EmotionalState,
        message: &str
    ) {
        let person = self.get_or_create_person(name, timestamp);

        // Analyze message
        let message_length = message.len();
        let has_questions = message.contains('?');
        let technical_terms = message.split_whitespace()
            .filter(|word| word.len() > 10 || word.contains("::"))
            .count();

        person.observe_communication(message_length, has_questions, technical_terms);
        person.record_interaction(timestamp, topics, emotional_response);
    }

    /// Note that a person has expertise in a topic
    pub fn note_expertise(&mut self, name: &str, topic: String, timestamp: u64) {
        let person = self.get_or_create_person(name, timestamp);
        person.note_expertise(topic);
    }

    /// Note that a person is curious about a topic
    pub fn note_curiosity(&mut self, name: &str, topic: String, timestamp: u64) {
        let person = self.get_or_create_person(name, timestamp);
        person.note_curiosity(topic);
    }

    /// Get a person model (if exists)
    pub fn get_person(&self, name: &str) -> Option<&PersonModel> {
        self.people.get(name)
    }

    /// Get all people SAGE knows
    pub fn get_all_people(&self) -> Vec<&PersonModel> {
        self.people.values().collect()
    }

    /// Get people by relationship strength
    pub fn get_closest_people(&self, count: usize) -> Vec<&PersonModel> {
        let mut people: Vec<&PersonModel> = self.people.values().collect();
        people.sort_by(|a, b| b.relationship_strength.partial_cmp(&a.relationship_strength).unwrap());
        people.into_iter().take(count).collect()
    }

    /// Apply relationship decay for people not seen recently
    pub fn apply_decay(&mut self, current_time: u64) {
        for person in self.people.values_mut() {
            let time_since_seen = current_time.saturating_sub(person.last_seen);
            let days_since_seen = time_since_seen / 86400;  // seconds to days

            if days_since_seen > 0 {
                let decay = self.decay_rate * days_since_seen as f64;
                person.relationship_strength = (person.relationship_strength - decay).max(0.0);
            }
        }
    }

    /// Get summary of social network
    pub fn get_social_summary(&self) -> String {
        if self.people.is_empty() {
            return "I haven't met anyone yet.".to_string();
        }

        let total = self.people.len();
        let closest = self.get_closest_people(3);

        let mut summary = format!("I know {} people.\n\nClosest relationships:\n", total);

        for person in closest {
            summary.push_str(&format!("- {}\n", person.describe()));
        }

        summary
    }

    /// Suggest how to respond to a specific person
    pub fn suggest_response_for(&self, name: &str) -> Option<String> {
        self.get_person(name).map(|person| {
            format!(
                "Response style for {}: {}\nRecent topics: {}",
                person.name,
                person.suggest_response_style(),
                person.recent_topics.iter().take(3).cloned().collect::<Vec<_>>().join(", ")
            )
        })
    }

    /// Check if SAGE knows this person well
    pub fn knows_well(&self, name: &str) -> bool {
        self.get_person(name)
            .map(|p| p.relationship_strength > 0.5)
            .unwrap_or(false)
    }

    /// Get number of people SAGE knows
    pub fn people_count(&self) -> usize {
        self.people.len()
    }
}

impl Default for TheoryOfMind {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_person_model_creation() {
        let mut tom = TheoryOfMind::new();
        let person = tom.get_or_create_person("Alice", 1000);
        assert_eq!(person.name, "Alice");
        assert_eq!(person.interaction_count, 0);
    }

    #[test]
    fn test_relationship_building() {
        let mut tom = TheoryOfMind::new();
        let emotion = EmotionalState {
            valence: 0.8,
            arousal: 0.6,
            dominance: 0.7,
            intensity: 0.5,
        };

        for _ in 0..10 {
            tom.record_interaction(
                "Alice",
                1000,
                vec!["rust".to_string()],
                emotion,
                "How does Rust handle memory?"
            );
        }

        let person = tom.get_person("Alice").unwrap();
        assert!(person.relationship_strength > 0.0);
        assert_eq!(person.interaction_count, 10);
    }

    #[test]
    fn test_communication_style_learning() {
        let mut person = PersonModel::new("Bob".to_string(), 1000);

        // Simulate technical messages
        for _ in 0..5 {
            person.observe_communication(300, true, 5);
        }

        assert!(person.communication_style.technical_depth > 0.5);
        assert!(person.communication_style.question_frequency > 0.5);
    }
}
