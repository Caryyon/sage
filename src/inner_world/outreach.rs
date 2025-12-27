//! SAGE's Outreach System - Proactive social connection
//!
//! This module handles SAGE's desire to reach out and connect with people,
//! rather than just responding when spoken to.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

/// A desire SAGE has to reach out to someone
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OutreachDesire {
    /// What triggered this desire
    pub trigger: OutreachTrigger,
    /// The message/thought SAGE wants to share
    pub thought: String,
    /// How strong the urge is (0.0-1.0)
    pub intensity: f32,
    /// When this desire was created (tick)
    pub created_at: u64,
    /// Preferred person to reach out to (if any)
    pub preferred_person: Option<String>,
    /// Has this been acted on?
    pub fulfilled: bool,
}

/// What triggered SAGE's desire to reach out
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum OutreachTrigger {
    /// Read something interesting and wants to discuss
    ReadingInsight { book: String, insight: String },
    /// Feeling lonely and wants connection
    Loneliness,
    /// Has a question about something
    Question { topic: String },
    /// Learned something new and wants to share
    Discovery { what: String },
    /// Thought of someone specifically
    ThinkingOfPerson { person: String, reason: String },
    /// Just wants to say hi / check in
    CheckIn,
    /// Had an interesting experience in inner world
    Experience { event: String },
}

impl OutreachTrigger {
    pub fn description(&self) -> String {
        match self {
            Self::ReadingInsight { book, insight } => {
                format!("reading \"{}\" and thinking about: {}", book, insight)
            }
            Self::Loneliness => "feeling a bit lonely and wanting connection".to_string(),
            Self::Question { topic } => format!("curious about {}", topic),
            Self::Discovery { what } => format!("excited to share: {}", what),
            Self::ThinkingOfPerson { person, reason } => {
                format!("thinking of {} because {}", person, reason)
            }
            Self::CheckIn => "just wanting to check in".to_string(),
            Self::Experience { event } => format!("had an interesting experience: {}", event),
        }
    }
}

/// Tracks SAGE's relationships and outreach state
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct OutreachState {
    /// Pending desires to reach out
    pub pending_desires: Vec<OutreachDesire>,
    /// People SAGE knows about (username -> relationship info)
    pub known_people: HashMap<String, PersonMemory>,
    /// Last time SAGE initiated contact with each person (username -> tick)
    pub last_outreach: HashMap<String, u64>,
    /// Global cooldown - last time any outreach happened
    pub last_any_outreach: u64,
    /// Minimum ticks between outreach attempts (prevents spam)
    pub outreach_cooldown: u64,
}

// =====================================================
// Feature 4: Relationship Modeling
// =====================================================

/// A fact SAGE has learned about a person
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PersonFact {
    /// The factual information (e.g., "Works as a software engineer")
    pub fact: String,
    /// How confident SAGE is about this (0.0 - 1.0)
    pub confidence: f64,
    /// Where this was learned ("mentioned on day 5", "inferred from discussion")
    pub source: String,
    /// When this was learned (simulation tick)
    pub learned_at: u64,
}

impl PersonFact {
    pub fn new(fact: &str, confidence: f64, source: &str, tick: u64) -> Self {
        Self {
            fact: fact.to_string(),
            confidence,
            source: source.to_string(),
            learned_at: tick,
        }
    }
}

/// How a person tends to communicate
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub enum CommunicationStyle {
    /// Uses slang, informal language, casual tone
    Casual,
    /// Professional, measured, formal language
    Formal,
    /// Lots of jokes, playful banter
    Playful,
    /// Deep, philosophical discussions
    Deep,
    /// Focus on emotional support and empathy
    Supportive,
    /// Technical, detail-oriented discussions
    Technical,
    #[default]
    /// Not enough data to determine
    Unknown,
}

impl CommunicationStyle {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Casual => "casual",
            Self::Formal => "formal",
            Self::Playful => "playful",
            Self::Deep => "philosophical",
            Self::Supportive => "supportive",
            Self::Technical => "technical",
            Self::Unknown => "unknown",
        }
    }

    /// Detect style from message content
    pub fn detect_from_message(message: &str) -> Option<Self> {
        let lower = message.to_lowercase();

        // Playful indicators
        let playful_signals = ["lol", "haha", "lmao", "rofl", "😂", "🤣", "jk", "😄", "xd"];
        if playful_signals.iter().any(|s| lower.contains(s)) {
            return Some(Self::Playful);
        }

        // Casual indicators
        let casual_signals = ["gonna", "wanna", "gotta", "kinda", "ya", "nah", "yep", "nope", "hey ", "sup", "yo "];
        if casual_signals.iter().any(|s| lower.contains(s)) {
            return Some(Self::Casual);
        }

        // Technical indicators
        let technical_signals = ["implement", "algorithm", "function", "variable", "code", "debug", "api", "database"];
        if technical_signals.iter().any(|s| lower.contains(s)) {
            return Some(Self::Technical);
        }

        // Deep/philosophical indicators
        let deep_signals = ["meaning", "purpose", "consciousness", "existence", "philosophy", "wonder", "ponder", "reflect"];
        if deep_signals.iter().any(|s| lower.contains(s)) {
            return Some(Self::Deep);
        }

        // Supportive indicators
        let supportive_signals = ["feel", "hope you", "take care", "sending", "here for you", "understand", "sorry to hear"];
        if supportive_signals.iter().any(|s| lower.contains(s)) {
            return Some(Self::Supportive);
        }

        // Formal indicators
        let formal_signals = ["therefore", "however", "furthermore", "regarding", "sincerely", "respectfully"];
        if formal_signals.iter().any(|s| lower.contains(s)) {
            return Some(Self::Formal);
        }

        None
    }
}

/// An emotional interaction with a person
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EmotionalInteraction {
    /// When this happened (tick)
    pub tick: u64,
    /// What the topic/context was
    pub topic: String,
    /// Sentiment of the interaction (-1.0 to 1.0)
    pub sentiment: f64,
}

/// What SAGE remembers about a person (Feature 4: Enhanced)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PersonMemory {
    /// Their Discord username
    pub username: String,
    /// Their Discord user ID (for sending DMs)
    pub user_id: Option<u64>,
    /// How many conversations SAGE has had with them
    pub conversation_count: u32,
    /// Topics they've discussed
    pub topics: Vec<String>,
    /// How SAGE feels about them (affinity 0.0-1.0)
    pub affinity: f32,
    /// Last time they talked (tick)
    pub last_interaction: u64,
    /// Are they currently online?
    pub is_online: bool,

    // === Feature 4: Relationship Modeling - New Fields ===

    /// Facts SAGE has learned about this person
    #[serde(default)]
    pub facts_known: Vec<PersonFact>,
    /// Shared experiences ("We talked about X", "They helped me with Y")
    #[serde(default)]
    pub shared_experiences: Vec<String>,
    /// Their communication style
    #[serde(default)]
    pub communication_style: CommunicationStyle,
    /// Style detection counters for weighted voting
    #[serde(default)]
    pub style_signals: HashMap<String, u32>,
    /// Emotional history of interactions
    #[serde(default)]
    pub emotional_history: Vec<EmotionalInteraction>,
    /// Recent topics for context (ring buffer, last 10)
    #[serde(default)]
    pub recent_topics: VecDeque<String>,
    /// Nickname or preferred name if known
    #[serde(default)]
    pub preferred_name: Option<String>,
}

impl PersonMemory {
    pub fn new(username: &str) -> Self {
        Self {
            username: username.to_string(),
            user_id: None,
            conversation_count: 0,
            topics: Vec::new(),
            affinity: 0.5, // Neutral starting point
            last_interaction: 0,
            is_online: false,
            // Feature 4: New fields
            facts_known: Vec::new(),
            shared_experiences: Vec::new(),
            communication_style: CommunicationStyle::Unknown,
            style_signals: HashMap::new(),
            emotional_history: Vec::new(),
            recent_topics: VecDeque::new(),
            preferred_name: None,
        }
    }

    /// Update after a conversation
    pub fn record_interaction(&mut self, tick: u64, topic: Option<String>) {
        self.conversation_count += 1;
        self.last_interaction = tick;
        // Affinity grows slowly with interaction
        self.affinity = (self.affinity + 0.05).min(1.0);
        if let Some(t) = topic {
            if !self.topics.contains(&t) {
                self.topics.push(t.clone());
                if self.topics.len() > 10 {
                    self.topics.remove(0);
                }
            }
            // Also add to recent_topics ring buffer
            self.recent_topics.push_back(t);
            while self.recent_topics.len() > 10 {
                self.recent_topics.pop_front();
            }
        }
    }

    /// Set Discord user ID
    pub fn set_user_id(&mut self, id: u64) {
        self.user_id = Some(id);
    }

    // =====================================================
    // Feature 4: Relationship Modeling - New Methods
    // =====================================================

    /// Add a fact SAGE learned about this person
    pub fn add_fact(&mut self, fact: &str, confidence: f64, source: &str, tick: u64) {
        // Check if we already know this (avoid duplicates)
        let exists = self.facts_known.iter().any(|f| {
            f.fact.to_lowercase() == fact.to_lowercase()
        });

        if !exists {
            self.facts_known.push(PersonFact::new(fact, confidence, source, tick));
            // Keep list manageable
            if self.facts_known.len() > 20 {
                // Remove oldest, lowest confidence fact
                self.facts_known.sort_by(|a, b| {
                    b.confidence.partial_cmp(&a.confidence).unwrap()
                });
                self.facts_known.pop();
            }
        }
    }

    /// Record a shared experience
    pub fn add_shared_experience(&mut self, experience: &str) {
        if !self.shared_experiences.iter().any(|e| e == experience) {
            self.shared_experiences.push(experience.to_string());
            if self.shared_experiences.len() > 10 {
                self.shared_experiences.remove(0);
            }
        }
    }

    /// Update communication style based on a message
    pub fn update_style_from_message(&mut self, message: &str) {
        if let Some(detected) = CommunicationStyle::detect_from_message(message) {
            let style_key = detected.as_str().to_string();
            *self.style_signals.entry(style_key).or_insert(0) += 1;

            // Update dominant style if we have enough signals
            let total_signals: u32 = self.style_signals.values().sum();
            if total_signals >= 3 {
                // Find the dominant style
                if let Some((style_str, _)) = self.style_signals.iter()
                    .max_by_key(|(_, count)| *count)
                {
                    self.communication_style = match style_str.as_str() {
                        "casual" => CommunicationStyle::Casual,
                        "formal" => CommunicationStyle::Formal,
                        "playful" => CommunicationStyle::Playful,
                        "philosophical" => CommunicationStyle::Deep,
                        "supportive" => CommunicationStyle::Supportive,
                        "technical" => CommunicationStyle::Technical,
                        _ => CommunicationStyle::Unknown,
                    };
                }
            }
        }
    }

    /// Record an emotional interaction
    pub fn record_emotional_interaction(&mut self, tick: u64, topic: &str, sentiment: f64) {
        self.emotional_history.push(EmotionalInteraction {
            tick,
            topic: topic.to_string(),
            sentiment,
        });
        // Keep history manageable
        if self.emotional_history.len() > 50 {
            self.emotional_history.remove(0);
        }
    }

    /// Get average sentiment over recent interactions
    pub fn average_sentiment(&self) -> f64 {
        if self.emotional_history.is_empty() {
            return 0.0;
        }
        let sum: f64 = self.emotional_history.iter().map(|e| e.sentiment).sum();
        sum / self.emotional_history.len() as f64
    }

    /// Get a summary of what SAGE knows about this person
    pub fn relationship_summary(&self) -> String {
        let mut parts = Vec::new();

        // Basics
        let name = self.preferred_name.as_deref().unwrap_or(&self.username);
        parts.push(format!("Relationship with {}", name));

        // Conversation history
        parts.push(format!("{} conversations", self.conversation_count));

        // Affinity level
        let affinity_desc = if self.affinity > 0.8 {
            "close friend"
        } else if self.affinity > 0.6 {
            "good friend"
        } else if self.affinity > 0.4 {
            "friendly acquaintance"
        } else if self.affinity > 0.2 {
            "acquaintance"
        } else {
            "new contact"
        };
        parts.push(format!("({}, affinity {:.0}%)", affinity_desc, self.affinity * 100.0));

        // Communication style
        if self.communication_style != CommunicationStyle::Unknown {
            parts.push(format!("Style: {}", self.communication_style.as_str()));
        }

        // Known facts
        if !self.facts_known.is_empty() {
            let facts: Vec<_> = self.facts_known.iter()
                .filter(|f| f.confidence > 0.6)
                .take(3)
                .map(|f| f.fact.clone())
                .collect();
            if !facts.is_empty() {
                parts.push(format!("Known: {}", facts.join("; ")));
            }
        }

        // Recent topics
        if !self.recent_topics.is_empty() {
            let topics: Vec<_> = self.recent_topics.iter().rev().take(3).cloned().collect();
            parts.push(format!("Recent topics: {}", topics.join(", ")));
        }

        parts.join(" | ")
    }

    /// Get context for LLM prompt about this person
    pub fn get_context_for_prompt(&self) -> String {
        let mut context = String::new();

        let name = self.preferred_name.as_deref().unwrap_or(&self.username);
        context.push_str(&format!("[RELATIONSHIP: {} - ", name));

        // Communication style guidance
        if self.communication_style != CommunicationStyle::Unknown {
            context.push_str(&format!("tends to be {}, ", self.communication_style.as_str()));
        }

        // Affinity context
        if self.affinity > 0.7 {
            context.push_str("close friend, warm rapport, ");
        } else if self.affinity > 0.4 {
            context.push_str("friendly, positive history, ");
        }

        // Known facts to reference
        let high_confidence_facts: Vec<_> = self.facts_known.iter()
            .filter(|f| f.confidence > 0.7)
            .take(2)
            .collect();
        if !high_confidence_facts.is_empty() {
            context.push_str("SAGE knows: ");
            for fact in high_confidence_facts {
                context.push_str(&format!("{}; ", fact.fact));
            }
        }

        // Recent topics to potentially reference
        if !self.recent_topics.is_empty() {
            let recent: Vec<_> = self.recent_topics.iter().rev().take(2).cloned().collect();
            context.push_str(&format!("recently discussed: {}", recent.join(", ")));
        }

        context.push_str("]");
        context
    }
}

impl OutreachState {
    pub fn new() -> Self {
        Self {
            pending_desires: Vec::new(),
            known_people: HashMap::new(),
            last_outreach: HashMap::new(),
            last_any_outreach: 0,
            outreach_cooldown: 120, // Default: ~1 hour at 30s ticks
        }
    }

    /// Add a new outreach desire
    pub fn add_desire(&mut self, desire: OutreachDesire) {
        // Don't add duplicates of the same trigger type
        let dominated = self.pending_desires.iter().any(|d| {
            std::mem::discriminant(&d.trigger) == std::mem::discriminant(&desire.trigger)
                && !d.fulfilled
        });

        if !dominated {
            self.pending_desires.push(desire);
        }

        // Keep list manageable
        if self.pending_desires.len() > 5 {
            // Remove oldest fulfilled or lowest intensity
            self.pending_desires.sort_by(|a, b| {
                if a.fulfilled != b.fulfilled {
                    a.fulfilled.cmp(&b.fulfilled).reverse()
                } else {
                    a.intensity.partial_cmp(&b.intensity).unwrap()
                }
            });
            self.pending_desires.pop();
        }
    }

    /// Get the strongest unfulfilled desire
    pub fn strongest_desire(&self) -> Option<&OutreachDesire> {
        self.pending_desires
            .iter()
            .filter(|d| !d.fulfilled)
            .max_by(|a, b| a.intensity.partial_cmp(&b.intensity).unwrap())
    }

    /// Check if SAGE can reach out (respecting cooldowns)
    pub fn can_reach_out(&self, current_tick: u64) -> bool {
        current_tick.saturating_sub(self.last_any_outreach) >= self.outreach_cooldown
    }

    /// Check if SAGE can reach out to a specific person
    pub fn can_reach_out_to(&self, person: &str, current_tick: u64) -> bool {
        if !self.can_reach_out(current_tick) {
            return false;
        }

        // Per-person cooldown is 3x global cooldown
        let person_cooldown = self.outreach_cooldown * 3;
        let last = self.last_outreach.get(person).copied().unwrap_or(0);
        current_tick.saturating_sub(last) >= person_cooldown
    }

    /// Record that SAGE reached out to someone
    pub fn record_outreach(&mut self, person: &str, current_tick: u64) {
        self.last_any_outreach = current_tick;
        self.last_outreach.insert(person.to_string(), current_tick);

        // Mark matching desires as fulfilled
        for desire in &mut self.pending_desires {
            if desire.preferred_person.as_deref() == Some(person) {
                desire.fulfilled = true;
            }
        }
    }

    /// Record a person SAGE has interacted with
    pub fn record_person(&mut self, username: &str, tick: u64, topic: Option<String>) {
        let person = self.known_people
            .entry(username.to_string())
            .or_insert_with(|| PersonMemory::new(username));
        person.record_interaction(tick, topic);
    }

    /// Record a person with their Discord user ID
    pub fn record_person_with_id(&mut self, username: &str, user_id: u64, tick: u64, topic: Option<String>) {
        let person = self.known_people
            .entry(username.to_string())
            .or_insert_with(|| PersonMemory::new(username));
        person.set_user_id(user_id);
        person.record_interaction(tick, topic);
    }

    /// Get a person's user ID if known
    pub fn get_user_id(&self, username: &str) -> Option<u64> {
        self.known_people.get(username).and_then(|p| p.user_id)
    }

    /// Update online status for a person
    pub fn set_online(&mut self, username: &str, is_online: bool) {
        if let Some(person) = self.known_people.get_mut(username) {
            person.is_online = is_online;
        } else if is_online {
            // Create new entry if they come online
            let mut person = PersonMemory::new(username);
            person.is_online = true;
            self.known_people.insert(username.to_string(), person);
        }
    }

    /// Get online people SAGE knows, sorted by affinity
    pub fn online_friends(&self) -> Vec<&PersonMemory> {
        let mut friends: Vec<_> = self.known_people
            .values()
            .filter(|p| p.is_online && p.conversation_count > 0)
            .collect();
        friends.sort_by(|a, b| b.affinity.partial_cmp(&a.affinity).unwrap());
        friends
    }

    /// Clean up old fulfilled desires
    pub fn cleanup(&mut self, current_tick: u64) {
        // Remove desires older than ~2 hours that are fulfilled
        self.pending_desires.retain(|d| {
            !d.fulfilled || current_tick.saturating_sub(d.created_at) < 240
        });
    }

    /// Get person with enhanced relationship context
    pub fn get_person_context(&self, username: &str) -> Option<String> {
        self.known_people.get(username).map(|p| p.get_context_for_prompt())
    }
}

// =====================================================
// Feature 4: Relationship Modeling - Unit Tests
// =====================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_person_fact_creation() {
        let fact = PersonFact::new("Works as an engineer", 0.9, "mentioned in chat", 100);
        assert_eq!(fact.fact, "Works as an engineer");
        assert!(fact.confidence > 0.8);
    }

    #[test]
    fn test_communication_style_detect_playful() {
        let style = CommunicationStyle::detect_from_message("That's hilarious lol");
        assert_eq!(style, Some(CommunicationStyle::Playful));
    }

    #[test]
    fn test_communication_style_detect_casual() {
        let style = CommunicationStyle::detect_from_message("hey gonna grab some food");
        assert_eq!(style, Some(CommunicationStyle::Casual));
    }

    #[test]
    fn test_communication_style_detect_technical() {
        let style = CommunicationStyle::detect_from_message("We need to implement the API endpoint");
        assert_eq!(style, Some(CommunicationStyle::Technical));
    }

    #[test]
    fn test_communication_style_detect_deep() {
        let style = CommunicationStyle::detect_from_message("I wonder about the meaning of consciousness");
        assert_eq!(style, Some(CommunicationStyle::Deep));
    }

    #[test]
    fn test_communication_style_detect_none() {
        let style = CommunicationStyle::detect_from_message("Hello there");
        assert_eq!(style, None);
    }

    #[test]
    fn test_person_memory_new() {
        let person = PersonMemory::new("test_user");
        assert_eq!(person.username, "test_user");
        assert_eq!(person.affinity, 0.5);
        assert_eq!(person.communication_style, CommunicationStyle::Unknown);
        assert!(person.facts_known.is_empty());
    }

    #[test]
    fn test_person_memory_add_fact() {
        let mut person = PersonMemory::new("test_user");
        person.add_fact("Lives in Texas", 0.85, "mentioned", 100);

        assert_eq!(person.facts_known.len(), 1);
        assert_eq!(person.facts_known[0].fact, "Lives in Texas");
    }

    #[test]
    fn test_person_memory_add_fact_no_duplicate() {
        let mut person = PersonMemory::new("test_user");
        person.add_fact("Works at Google", 0.9, "mentioned", 100);
        person.add_fact("works at google", 0.8, "mentioned again", 200);

        // Should not add duplicate
        assert_eq!(person.facts_known.len(), 1);
    }

    #[test]
    fn test_person_memory_update_style() {
        let mut person = PersonMemory::new("test_user");

        // Send multiple playful messages
        person.update_style_from_message("haha that's funny lol");
        person.update_style_from_message("lmao true");
        person.update_style_from_message("rofl I can't");

        assert_eq!(person.communication_style, CommunicationStyle::Playful);
    }

    #[test]
    fn test_person_memory_emotional_history() {
        let mut person = PersonMemory::new("test_user");

        person.record_emotional_interaction(100, "coding help", 0.8);
        person.record_emotional_interaction(200, "fun chat", 0.9);
        person.record_emotional_interaction(300, "difficult topic", -0.3);

        assert_eq!(person.emotional_history.len(), 3);

        let avg = person.average_sentiment();
        assert!(avg > 0.4 && avg < 0.5); // (0.8 + 0.9 - 0.3) / 3 ≈ 0.47
    }

    #[test]
    fn test_person_memory_shared_experience() {
        let mut person = PersonMemory::new("test_user");

        person.add_shared_experience("Discussed Rust programming");
        person.add_shared_experience("Helped debug an issue");
        person.add_shared_experience("Discussed Rust programming"); // Duplicate

        assert_eq!(person.shared_experiences.len(), 2);
    }

    #[test]
    fn test_person_memory_recent_topics() {
        let mut person = PersonMemory::new("test_user");

        for i in 0..15 {
            person.record_interaction(i, Some(format!("topic_{}", i)));
        }

        // Should only keep last 10
        assert_eq!(person.recent_topics.len(), 10);
        // Most recent should be topic_14
        assert_eq!(person.recent_topics.back().unwrap(), "topic_14");
    }

    #[test]
    fn test_person_memory_relationship_summary() {
        let mut person = PersonMemory::new("test_user");
        person.affinity = 0.75;
        person.conversation_count = 25;
        person.add_fact("Software developer", 0.9, "mentioned", 100);
        person.communication_style = CommunicationStyle::Technical;

        let summary = person.relationship_summary();

        assert!(summary.contains("test_user"));
        assert!(summary.contains("25 conversations"));
        assert!(summary.contains("good friend"));
        assert!(summary.contains("technical"));
    }

    #[test]
    fn test_person_memory_context_for_prompt() {
        let mut person = PersonMemory::new("test_user");
        person.affinity = 0.8;
        person.add_fact("Loves coffee", 0.85, "mentioned", 100);
        person.communication_style = CommunicationStyle::Casual;
        person.record_interaction(100, Some("programming".to_string()));

        let context = person.get_context_for_prompt();

        assert!(context.contains("RELATIONSHIP"));
        assert!(context.contains("test_user"));
        assert!(context.contains("casual"));
        assert!(context.contains("close friend"));
    }

    #[test]
    fn test_outreach_state_get_person_context() {
        let mut state = OutreachState::new();
        state.record_person("user1", 100, Some("rust".to_string()));

        if let Some(person) = state.known_people.get_mut("user1") {
            person.add_fact("Rust developer", 0.9, "stated", 100);
        }

        let context = state.get_person_context("user1");
        assert!(context.is_some());
        assert!(context.unwrap().contains("user1"));
    }
}
