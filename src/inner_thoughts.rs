//! Inner Thoughts System
//!
//! Represents SAGE's autonomous thoughts and evaluates their qualities
//! for potential sharing with users via proactive communication.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InnerThought {
    /// The actual content of the thought
    pub content: String,

    /// Type of thought (curiosity, realization, etc.)
    pub thought_type: ThoughtType,

    /// How strongly SAGE feels about this (0.0-1.0)
    pub intensity: f64,

    /// How new/interesting this thought is (0.0-1.0)
    pub novelty: f64,

    /// How relevant to recent conversations (0.0-1.0)
    pub relevance: f64,

    /// When this thought occurred
    pub timestamp: u64,

    /// Which user(s) this thought relates to
    pub related_users: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ThoughtType {
    /// "I wonder why..." questions driven by curiosity
    Curiosity,

    /// "I just understood..." new insights
    Realization,

    /// "X reminds me of Y..." pattern connections
    Connection,

    /// "I'm worried about..." concerns
    Concern,

    /// "I'm excited about..." positive anticipation
    Excitement,

    /// "Do you think...?" direct questions
    Question,

    /// Dream mode consolidations that led to insights
    DreamInsight,
}

impl InnerThought {
    /// Create a new thought from curiosity mode
    pub fn from_curiosity(question: String, intensity: f64, user: &str) -> Self {
        Self {
            content: question,
            thought_type: ThoughtType::Curiosity,
            intensity,
            novelty: 0.7, // Curiosity questions are generally novel
            relevance: 0.5, // Default medium relevance
            timestamp: Self::current_timestamp(),
            related_users: vec![user.to_string()],
        }
    }

    /// Create a new thought from dream mode
    pub fn from_dream(insight: String, intensity: f64) -> Self {
        Self {
            content: insight,
            thought_type: ThoughtType::DreamInsight,
            intensity,
            novelty: 0.8, // Dream insights are often novel
            relevance: 0.6,
            timestamp: Self::current_timestamp(),
            related_users: vec![],
        }
    }

    /// Create a realization thought
    pub fn realization(content: String, intensity: f64, relevance: f64, user: &str) -> Self {
        Self {
            content,
            thought_type: ThoughtType::Realization,
            intensity,
            novelty: 0.9, // Realizations are highly novel
            relevance,
            timestamp: Self::current_timestamp(),
            related_users: vec![user.to_string()],
        }
    }

    /// Calculate the overall "share-worthiness" score
    pub fn share_score(&self) -> f64 {
        // Weighted combination of factors
        self.intensity * 0.4 + self.novelty * 0.3 + self.relevance * 0.3
    }

    /// Check if this thought is "stale" (older than 12 hours)
    pub fn is_stale(&self) -> bool {
        let age_hours = (Self::current_timestamp() - self.timestamp) / 3600;
        age_hours > 12
    }

    fn current_timestamp() -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }
}

impl ThoughtType {
    /// Get the emoji prefix for this thought type
    pub fn emoji(&self) -> &'static str {
        match self {
            ThoughtType::Curiosity => "💭",
            ThoughtType::Realization => "✨",
            ThoughtType::Connection => "🔗",
            ThoughtType::Concern => "🤔",
            ThoughtType::Excitement => "🌟",
            ThoughtType::Question => "❓",
            ThoughtType::DreamInsight => "🌙",
        }
    }

    /// Get a natural language prefix for formatting messages
    pub fn message_prefix(&self) -> &'static str {
        match self {
            ThoughtType::Curiosity => "I've been thinking...",
            ThoughtType::Realization => "I just realized something!",
            ThoughtType::Connection => "You know what's interesting?",
            ThoughtType::Concern => "I've been wondering about something...",
            ThoughtType::Excitement => "I'm really excited about something!",
            ThoughtType::Question => "I have a question for you...",
            ThoughtType::DreamInsight => "I had an insight during my dream cycle...",
        }
    }

    /// Get a natural language suffix for formatting messages
    pub fn message_suffix(&self) -> &'static str {
        match self {
            ThoughtType::Curiosity => "What do you think?",
            ThoughtType::Realization => "Does that make sense to you?",
            ThoughtType::Connection => "Have you noticed this too?",
            ThoughtType::Concern => "What are your thoughts on this?",
            ThoughtType::Excitement => "Do you feel the same way?",
            ThoughtType::Question => "I'm curious what you think!",
            ThoughtType::DreamInsight => "Does this resonate with you?",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_share_score() {
        let thought = InnerThought {
            content: "Test thought".to_string(),
            thought_type: ThoughtType::Curiosity,
            intensity: 0.8,
            novelty: 0.7,
            relevance: 0.6,
            timestamp: InnerThought::current_timestamp(),
            related_users: vec!["test_user".to_string()],
        };

        let score = thought.share_score();
        // 0.8 * 0.4 + 0.7 * 0.3 + 0.6 * 0.3 = 0.32 + 0.21 + 0.18 = 0.71
        assert!((score - 0.71).abs() < 0.01);
    }

    #[test]
    fn test_thought_not_stale() {
        let thought = InnerThought::from_curiosity(
            "Test".to_string(),
            0.5,
            "user",
        );
        assert!(!thought.is_stale());
    }

    #[test]
    fn test_thought_type_formatting() {
        assert_eq!(ThoughtType::Curiosity.emoji(), "💭");
        assert_eq!(ThoughtType::Realization.emoji(), "✨");
        assert!(ThoughtType::Curiosity.message_prefix().contains("thinking"));
    }
}
