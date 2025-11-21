//! Proactive Communication System
//!
//! Enables SAGE to autonomously initiate conversations based on inner thoughts,
//! respecting social timing and user preferences.

use crate::inner_thoughts::{InnerThought, ThoughtType};
use std::collections::HashMap;
use std::time::Instant;
use chrono::{Local, Timelike};

/// Manages proactive communication decisions and message history
pub struct ProactiveCommunication {
    /// Track when we last sent a proactive message to each user
    last_proactive_messages: HashMap<String, Instant>,

    /// Track user activity timestamps
    last_user_activity: HashMap<String, Instant>,

    /// Minimum hours between proactive messages
    min_hours_between: f64,

    /// Base probability threshold for sending (0.0-1.0)
    share_threshold: f64,

    /// Personality modifier (how chatty SAGE is)
    personality_modifier: f64,
}

impl ProactiveCommunication {
    /// Create a new proactive communication manager
    pub fn new() -> Self {
        Self {
            last_proactive_messages: HashMap::new(),
            last_user_activity: HashMap::new(),
            min_hours_between: 2.0,
            share_threshold: 0.6,
            personality_modifier: 1.0,
        }
    }

    /// Configure how often SAGE reaches out (1.0 = normal, 2.0 = more chatty, 0.5 = quieter)
    pub fn set_chattiness(&mut self, modifier: f64) {
        self.personality_modifier = modifier.clamp(0.1, 3.0);
    }

    /// Record user activity (called when user sends a message)
    pub fn record_user_activity(&mut self, username: &str) {
        self.last_user_activity.insert(username.to_string(), Instant::now());
    }

    /// Evaluate if a thought should be shared with a user
    pub fn should_share(&mut self, thought: &InnerThought, username: &str) -> bool {
        // Don't share stale thoughts
        if thought.is_stale() {
            return false;
        }

        // Calculate base probability from thought quality
        let base_probability = thought.share_score();

        // Apply timing penalties
        let timing_penalty = self.calculate_timing_penalty(username);
        let adjusted_probability = base_probability * (1.0 - timing_penalty);

        // Apply personality modifier
        let final_probability = adjusted_probability * self.personality_modifier;

        // Check if we exceed threshold
        if final_probability < self.share_threshold {
            return false;
        }

        // Add randomness to feel natural
        let random_value: f64 = rand::random();
        random_value < final_probability
    }

    /// Calculate timing penalty based on social factors
    fn calculate_timing_penalty(&self, username: &str) -> f64 {
        let mut penalty = 0.0;

        // Penalty for messaging too soon after last proactive message
        if let Some(last_message_time) = self.last_proactive_messages.get(username) {
            let hours_since = last_message_time.elapsed().as_secs_f64() / 3600.0;

            if hours_since < self.min_hours_between {
                return 1.0; // 100% penalty - don't send at all
            }

            // Gradual penalty reduction up to 12 hours
            if hours_since < 12.0 {
                penalty += (12.0 - hours_since) / 12.0 * 0.3;
            }
        }

        // Time of day penalty (11pm-7am)
        let current_hour = Local::now().hour();
        if current_hour >= 23 || current_hour < 7 {
            penalty += 0.8; // Strong penalty for late night/early morning
        }

        // Penalty for inactive users
        if let Some(last_activity) = self.last_user_activity.get(username) {
            let days_since_activity = last_activity.elapsed().as_secs_f64() / 86400.0;

            if days_since_activity > 7.0 {
                penalty += 0.9; // Don't bother users who haven't been active in a week
            } else if days_since_activity > 3.0 {
                penalty += 0.5; // Moderate penalty for 3+ days inactive
            }
        } else {
            // No activity history - be cautious
            penalty += 0.4;
        }

        penalty.min(1.0)
    }

    /// Format a thought into a natural message
    pub fn format_message(&self, thought: &InnerThought) -> String {
        let emoji = thought.thought_type.emoji();
        let prefix = thought.thought_type.message_prefix();
        let suffix = thought.thought_type.message_suffix();

        format!(
            "{} {}\n\n{}\n\n{}",
            emoji,
            prefix,
            thought.content,
            suffix
        )
    }

    /// Record that we sent a proactive message
    pub fn record_proactive_message(&mut self, username: &str) {
        self.last_proactive_messages.insert(username.to_string(), Instant::now());
    }

    /// Get hours since last proactive message to a user
    pub fn hours_since_last_message(&self, username: &str) -> Option<f64> {
        self.last_proactive_messages
            .get(username)
            .map(|instant| instant.elapsed().as_secs_f64() / 3600.0)
    }

    /// Check if it's a good time to send any proactive messages (general timing check)
    pub fn is_good_time(&self) -> bool {
        let hour = Local::now().hour();

        // Good times: 8am-10pm
        hour >= 8 && hour < 22
    }
}

impl Default for ProactiveCommunication {
    fn default() -> Self {
        Self::new()
    }
}

/// A message ready to be sent proactively
pub struct ProactiveMessage {
    pub username: String,
    pub content: String,
    pub thought_type: ThoughtType,
}

impl ProactiveMessage {
    pub fn new(username: String, content: String, thought_type: ThoughtType) -> Self {
        Self {
            username,
            content,
            thought_type,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timing_penalty_too_soon() {
        let mut comm = ProactiveCommunication::new();
        comm.record_proactive_message("test_user");

        // Should have high penalty immediately after sending
        let penalty = comm.calculate_timing_penalty("test_user");
        assert!(penalty > 0.9);
    }

    #[test]
    fn test_good_time_check() {
        let comm = ProactiveCommunication::new();
        // This will depend on current time, but method should not panic
        let _ = comm.is_good_time();
    }

    #[test]
    fn test_message_formatting() {
        let comm = ProactiveCommunication::new();
        let thought = InnerThought::from_curiosity(
            "What makes consciousness emerge?".to_string(),
            0.8,
            "test_user",
        );

        let message = comm.format_message(&thought);
        assert!(message.contains("💭"));
        assert!(message.contains("thinking"));
        assert!(message.contains("What makes consciousness emerge?"));
        assert!(message.contains("What do you think?"));
    }

    #[test]
    fn test_chattiness_modifier() {
        let mut comm = ProactiveCommunication::new();
        assert_eq!(comm.personality_modifier, 1.0);

        comm.set_chattiness(2.0);
        assert_eq!(comm.personality_modifier, 2.0);

        // Test clamping
        comm.set_chattiness(10.0);
        assert_eq!(comm.personality_modifier, 3.0);
    }
}
