//! Conversation Context Management
//!
//! Tracks per-user conversation history to enable context-aware responses.
//! Each user gets their own conversation buffer with recent messages.

use std::collections::{HashMap, VecDeque};
use serde::{Serialize, Deserialize};

/// Maximum number of messages to keep per user
const MAX_CONTEXT_MESSAGES: usize = 10;

/// A single message in a conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationMessage {
    pub role: MessageRole,
    pub content: String,
    pub timestamp: u64,
}

/// Role of the message sender
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MessageRole {
    User,
    Assistant,
}

/// Manages conversation contexts for multiple users
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationContextManager {
    /// Per-user conversation history
    conversations: HashMap<String, VecDeque<ConversationMessage>>,
    /// Maximum messages to keep per user
    max_messages: usize,
}

impl ConversationContextManager {
    /// Create a new conversation context manager
    pub fn new() -> Self {
        Self {
            conversations: HashMap::new(),
            max_messages: MAX_CONTEXT_MESSAGES,
        }
    }

    /// Add a user message to the conversation history
    pub fn add_user_message(&mut self, user_id: &str, content: String) {
        let timestamp = self.current_timestamp();

        let conversation = self.conversations
            .entry(user_id.to_string())
            .or_insert_with(VecDeque::new);

        conversation.push_back(ConversationMessage {
            role: MessageRole::User,
            content,
            timestamp,
        });

        // Trim to max size
        while conversation.len() > self.max_messages {
            conversation.pop_front();
        }
    }

    /// Add an assistant (SAGE) message to the conversation history
    pub fn add_assistant_message(&mut self, user_id: &str, content: String) {
        let timestamp = self.current_timestamp();

        let conversation = self.conversations
            .entry(user_id.to_string())
            .or_insert_with(VecDeque::new);

        conversation.push_back(ConversationMessage {
            role: MessageRole::Assistant,
            content,
            timestamp,
        });

        // Trim to max size
        while conversation.len() > self.max_messages {
            conversation.pop_front();
        }
    }

    /// Get conversation history for a user
    pub fn get_conversation(&self, user_id: &str) -> Vec<ConversationMessage> {
        self.conversations
            .get(user_id)
            .map(|conv| conv.iter().cloned().collect())
            .unwrap_or_default()
    }

    /// Format conversation history for LLM context
    pub fn format_context(&self, user_id: &str) -> String {
        let conversation = self.get_conversation(user_id);

        if conversation.is_empty() {
            return String::new();
        }

        let mut context = String::from("Previous conversation:\n");

        for msg in conversation {
            let role = match msg.role {
                MessageRole::User => &user_id,
                MessageRole::Assistant => "SAGE",
            };
            context.push_str(&format!("{}: {}\n", role, msg.content));
        }

        context.push_str("\nContinuing conversation:\n");
        context
    }

    /// Clear conversation history for a user
    pub fn clear_conversation(&mut self, user_id: &str) {
        self.conversations.remove(user_id);
    }

    /// Get number of messages in a user's conversation
    pub fn get_message_count(&self, user_id: &str) -> usize {
        self.conversations
            .get(user_id)
            .map(|conv| conv.len())
            .unwrap_or(0)
    }

    /// Get total number of active conversations
    pub fn active_conversations(&self) -> usize {
        self.conversations.len()
    }

    /// Prune old conversations (older than 24 hours with no recent activity)
    pub fn prune_old_conversations(&mut self) {
        let current_time = self.current_timestamp();
        let day_in_seconds = 86400;

        self.conversations.retain(|_, conversation| {
            if let Some(last_msg) = conversation.back() {
                current_time - last_msg.timestamp < day_in_seconds
            } else {
                false
            }
        });
    }

    fn current_timestamp(&self) -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }
}

impl Default for ConversationContextManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_messages() {
        let mut manager = ConversationContextManager::new();

        manager.add_user_message("alice", "Hello!".to_string());
        manager.add_assistant_message("alice", "Hi Alice!".to_string());

        let conv = manager.get_conversation("alice");
        assert_eq!(conv.len(), 2);
        assert_eq!(conv[0].role, MessageRole::User);
        assert_eq!(conv[1].role, MessageRole::Assistant);
    }

    #[test]
    fn test_max_messages() {
        let mut manager = ConversationContextManager::new();

        // Add more than max messages
        for i in 0..15 {
            manager.add_user_message("bob", format!("Message {}", i));
        }

        let conv = manager.get_conversation("bob");
        assert_eq!(conv.len(), MAX_CONTEXT_MESSAGES);
    }

    #[test]
    fn test_format_context() {
        let mut manager = ConversationContextManager::new();

        manager.add_user_message("charlie", "What's your name?".to_string());
        manager.add_assistant_message("charlie", "I'm SAGE!".to_string());

        let context = manager.format_context("charlie");
        assert!(context.contains("charlie: What's your name?"));
        assert!(context.contains("SAGE: I'm SAGE!"));
    }

    #[test]
    fn test_separate_users() {
        let mut manager = ConversationContextManager::new();

        manager.add_user_message("user1", "Hello from user1".to_string());
        manager.add_user_message("user2", "Hello from user2".to_string());

        assert_eq!(manager.get_message_count("user1"), 1);
        assert_eq!(manager.get_message_count("user2"), 1);
        assert_eq!(manager.active_conversations(), 2);
    }
}
