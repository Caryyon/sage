// Episodic Memory System - Remember specific conversations as narrative experiences
//
// This allows SAGE to recall:
// - "That conversation with gmork about sophistication"
// - "When we talked about emotions on November 16th"
// - "The time you asked about my dreams"
//
// Instead of just concept-level associations, SAGE can now remember the narrative
// structure of conversations, including who said what, emotional arcs, and context.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::emotional_gradients::EmotionalState;

/// A single message in a conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub speaker: String,
    pub content: String,
    pub timestamp: u64,  // Unix timestamp
    pub emotional_response: Option<EmotionalState>,
}

/// An episode - a memorable conversation experience
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Episode {
    /// Unique identifier
    pub id: u64,

    /// Conversation messages in chronological order
    pub messages: Vec<Message>,

    /// People involved in this conversation
    pub participants: Vec<String>,

    /// Main topic(s) discussed
    pub topics: Vec<String>,

    /// SAGE's emotional journey through the conversation
    pub emotional_arc: Vec<EmotionalState>,

    /// When this episode started
    pub start_time: u64,

    /// When this episode ended
    pub end_time: u64,

    /// How significant/memorable this episode is (0.0 to 1.0)
    pub significance: f64,

    /// Brief summary of what happened
    pub summary: String,
}

impl Episode {
    /// Create a new episode
    pub fn new(id: u64, start_time: u64) -> Self {
        Self {
            id,
            messages: Vec::new(),
            participants: Vec::new(),
            topics: Vec::new(),
            emotional_arc: Vec::new(),
            start_time,
            end_time: start_time,
            significance: 0.0,
            summary: String::new(),
        }
    }

    /// Add a message to this episode
    pub fn add_message(&mut self, message: Message) {
        // Update participants
        if !self.participants.contains(&message.speaker) {
            self.participants.push(message.speaker.clone());
        }

        // Update emotional arc if message has emotional response
        if let Some(emotion) = &message.emotional_response {
            self.emotional_arc.push(*emotion);
        }

        // Update end time
        self.end_time = message.timestamp;

        self.messages.push(message);
    }

    /// Calculate significance based on emotional intensity and conversation length
    pub fn calculate_significance(&mut self) {
        let mut total_intensity = 0.0;
        let mut count = 0;

        for emotion in &self.emotional_arc {
            total_intensity += emotion.intensity;
            count += 1;
        }

        let avg_intensity = if count > 0 {
            total_intensity / count as f64
        } else {
            0.0
        };

        // Significance = emotional intensity + conversation length factor
        let length_factor = (self.messages.len() as f64 / 20.0).min(1.0);  // Cap at 20 messages
        self.significance = (avg_intensity * 0.7 + length_factor * 0.3).clamp(0.0, 1.0);
    }

    /// Generate a natural language summary
    pub fn generate_summary(&mut self) {
        if self.messages.is_empty() {
            self.summary = "Empty conversation".to_string();
            return;
        }

        let participants_str = if self.participants.len() == 1 {
            self.participants[0].clone()
        } else {
            self.participants.join(", ")
        };

        let dominant_emotion = if let Some(emotion) = self.emotional_arc.last() {
            emotion.to_label()
        } else {
            "neutral"
        };

        let topics_str = if !self.topics.is_empty() {
            format!(" about {}", self.topics.join(", "))
        } else {
            String::new()
        };

        self.summary = format!(
            "Conversation with {}{} ({} messages, feeling {})",
            participants_str,
            topics_str,
            self.messages.len(),
            dominant_emotion
        );
    }

    /// Get the duration of this episode in seconds
    pub fn duration(&self) -> u64 {
        self.end_time.saturating_sub(self.start_time)
    }
}

/// Manages SAGE's episodic memory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodicMemory {
    /// All remembered episodes
    episodes: Vec<Episode>,

    /// Current episode being constructed (if any)
    current_episode: Option<Episode>,

    /// Next episode ID
    next_id: u64,

    /// Index: topic -> episode IDs
    topic_index: HashMap<String, Vec<u64>>,

    /// Index: participant -> episode IDs
    participant_index: HashMap<String, Vec<u64>>,

    /// Maximum episodes to remember (older ones get consolidated/forgotten)
    max_episodes: usize,

    /// Idle timeout - seconds of inactivity before closing current episode
    idle_timeout: u64,
}

impl EpisodicMemory {
    pub fn new() -> Self {
        Self {
            episodes: Vec::new(),
            current_episode: None,
            next_id: 0,
            topic_index: HashMap::new(),
            participant_index: HashMap::new(),
            max_episodes: 100,
            idle_timeout: 300,  // 5 minutes
        }
    }

    /// Start a new episode
    pub fn start_episode(&mut self, timestamp: u64) {
        // Close current episode if one exists
        if let Some(mut episode) = self.current_episode.take() {
            episode.calculate_significance();
            episode.generate_summary();
            self.store_episode(episode);
        }

        // Create new episode
        let id = self.next_id;
        self.next_id += 1;
        self.current_episode = Some(Episode::new(id, timestamp));
    }

    /// Add a message to the current episode
    pub fn add_message(
        &mut self,
        speaker: String,
        content: String,
        timestamp: u64,
        emotional_response: Option<EmotionalState>
    ) {
        // Start new episode if none exists
        if self.current_episode.is_none() {
            self.start_episode(timestamp);
        }

        let message = Message {
            speaker,
            content,
            timestamp,
            emotional_response,
        };

        if let Some(episode) = &mut self.current_episode {
            episode.add_message(message);
        }
    }

    /// Check if current episode should be closed due to inactivity
    pub fn check_idle_timeout(&mut self, current_time: u64) {
        if let Some(episode) = &self.current_episode {
            let idle_duration = current_time.saturating_sub(episode.end_time);
            if idle_duration > self.idle_timeout {
                self.close_current_episode();
            }
        }
    }

    /// Manually close the current episode
    pub fn close_current_episode(&mut self) {
        if let Some(mut episode) = self.current_episode.take() {
            episode.calculate_significance();
            episode.generate_summary();
            self.store_episode(episode);
        }
    }

    /// Add topics to current episode
    pub fn add_topics(&mut self, topics: Vec<String>) {
        if let Some(episode) = &mut self.current_episode {
            for topic in topics {
                if !episode.topics.contains(&topic) {
                    episode.topics.push(topic);
                }
            }
        }
    }

    /// Store an episode and update indices
    fn store_episode(&mut self, episode: Episode) {
        let id = episode.id;

        // Update topic index
        for topic in &episode.topics {
            self.topic_index
                .entry(topic.clone())
                .or_insert_with(Vec::new)
                .push(id);
        }

        // Update participant index
        for participant in &episode.participants {
            self.participant_index
                .entry(participant.clone())
                .or_insert_with(Vec::new)
                .push(id);
        }

        // Store episode
        self.episodes.push(episode);

        // Consolidate if too many episodes
        if self.episodes.len() > self.max_episodes {
            self.consolidate_old_episodes();
        }
    }

    /// Consolidate old, less significant episodes
    fn consolidate_old_episodes(&mut self) {
        // Sort by significance
        self.episodes.sort_by(|a, b| {
            b.significance.partial_cmp(&a.significance).unwrap()
        });

        // Keep only the most significant episodes
        self.episodes.truncate(self.max_episodes);

        // Rebuild indices
        self.rebuild_indices();
    }

    /// Rebuild all indices after consolidation
    fn rebuild_indices(&mut self) {
        self.topic_index.clear();
        self.participant_index.clear();

        for episode in &self.episodes {
            let id = episode.id;

            for topic in &episode.topics {
                self.topic_index
                    .entry(topic.clone())
                    .or_insert_with(Vec::new)
                    .push(id);
            }

            for participant in &episode.participants {
                self.participant_index
                    .entry(participant.clone())
                    .or_insert_with(Vec::new)
                    .push(id);
            }
        }
    }

    /// Recall episodes about a specific topic
    pub fn recall_by_topic(&self, topic: &str) -> Vec<&Episode> {
        if let Some(ids) = self.topic_index.get(topic) {
            ids.iter()
                .filter_map(|id| self.episodes.iter().find(|e| e.id == *id))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Recall episodes with a specific participant
    pub fn recall_by_participant(&self, participant: &str) -> Vec<&Episode> {
        if let Some(ids) = self.participant_index.get(participant) {
            ids.iter()
                .filter_map(|id| self.episodes.iter().find(|e| e.id == *id))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get most recent episodes
    pub fn get_recent_episodes(&self, count: usize) -> Vec<&Episode> {
        self.episodes
            .iter()
            .rev()
            .take(count)
            .collect()
    }

    /// Get most significant episodes
    pub fn get_significant_episodes(&self, count: usize) -> Vec<Episode> {
        let mut sorted = self.episodes.clone();
        sorted.sort_by(|a, b| b.significance.partial_cmp(&a.significance).unwrap());
        sorted.into_iter().take(count).collect()
    }

    /// Search episodes by keywords in messages
    pub fn search(&self, keywords: &[String]) -> Vec<&Episode> {
        self.episodes
            .iter()
            .filter(|episode| {
                episode.messages.iter().any(|msg| {
                    let content_lower = msg.content.to_lowercase();
                    keywords.iter().any(|kw| content_lower.contains(&kw.to_lowercase()))
                })
            })
            .collect()
    }

    /// Get total number of episodes
    pub fn episode_count(&self) -> usize {
        self.episodes.len()
    }

    /// Get summary of episodic memory
    pub fn get_summary(&self) -> String {
        let total = self.episodes.len();
        let current = if self.current_episode.is_some() {
            " (1 ongoing)"
        } else {
            ""
        };

        if total == 0 {
            "No episodic memories yet.".to_string()
        } else {
            let recent = self.get_recent_episodes(3);
            let mut summary = format!("I remember {} conversation{}{}\n\nRecent episodes:\n",
                total,
                if total == 1 { "" } else { "s" },
                current
            );

            for episode in recent {
                summary.push_str(&format!("- {}\n", episode.summary));
            }

            summary
        }
    }
}

impl Default for EpisodicMemory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_episode_creation() {
        let mut memory = EpisodicMemory::new();

        memory.add_message(
            "Alice".to_string(),
            "Hello!".to_string(),
            1000,
            None
        );

        assert!(memory.current_episode.is_some());
        assert_eq!(memory.current_episode.as_ref().unwrap().messages.len(), 1);
    }

    #[test]
    fn test_episode_recall() {
        let mut memory = EpisodicMemory::new();

        memory.add_message("Alice".to_string(), "Let's talk about emotions".to_string(), 1000, None);
        memory.add_topics(vec!["emotions".to_string()]);
        memory.close_current_episode();

        let recalled = memory.recall_by_topic("emotions");
        assert_eq!(recalled.len(), 1);
    }

    #[test]
    fn test_significance_calculation() {
        let mut episode = Episode::new(0, 1000);

        // Add messages with varying emotional intensity
        for i in 0..5 {
            let emotion = EmotionalState {
                valence: 0.5,
                arousal: 0.6,
                dominance: 0.7,
                intensity: 0.8,
            };

            episode.add_message(Message {
                speaker: "Alice".to_string(),
                content: format!("Message {}", i),
                timestamp: 1000 + i,
                emotional_response: Some(emotion),
            });
        }

        episode.calculate_significance();
        assert!(episode.significance > 0.5);  // Should be significant
    }
}
