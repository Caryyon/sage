//! Conversation Context Management
//!
//! Tracks per-user conversation history to enable context-aware responses.
//! Each user gets their own conversation buffer with recent messages.
//! When conversations get long, older messages are automatically summarized
//! to preserve context while managing token usage.

use std::collections::{HashMap, VecDeque};
use serde::{Serialize, Deserialize};

/// Maximum number of recent messages to keep in full (rest get summarized)
const MAX_RECENT_MESSAGES: usize = 50;

/// Trigger summarization when we exceed this many messages
const SUMMARIZATION_THRESHOLD: usize = 60;

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
    /// Per-user conversation history (recent messages only)
    conversations: HashMap<String, VecDeque<ConversationMessage>>,
    /// Per-user conversation summaries (older messages condensed)
    summaries: HashMap<String, String>,
    /// Maximum recent messages to keep per user
    max_recent_messages: usize,
    /// Threshold at which to trigger summarization
    summarization_threshold: usize,
}

impl ConversationContextManager {
    /// Create a new conversation context manager
    pub fn new() -> Self {
        Self {
            conversations: HashMap::new(),
            summaries: HashMap::new(),
            max_recent_messages: MAX_RECENT_MESSAGES,
            summarization_threshold: SUMMARIZATION_THRESHOLD,
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

        // No automatic trimming - let caller decide when to summarize
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

        // No automatic trimming - let caller decide when to summarize
    }

    /// Get conversation history for a user
    pub fn get_conversation(&self, user_id: &str) -> Vec<ConversationMessage> {
        self.conversations
            .get(user_id)
            .map(|conv| conv.iter().cloned().collect())
            .unwrap_or_default()
    }

    /// Format conversation history for LLM context (includes summary if present)
    pub fn format_context(&self, user_id: &str) -> String {
        let conversation = self.get_conversation(user_id);
        let summary = self.summaries.get(user_id);

        if conversation.is_empty() && summary.is_none() {
            return String::new();
        }

        let mut context = String::new();

        // Include summary of older messages if present
        if let Some(summary_text) = summary {
            context.push_str("=== Earlier Conversation Summary ===\n");
            context.push_str(summary_text);
            context.push_str("\n\n");
        }

        // Include recent messages in full
        if !conversation.is_empty() {
            context.push_str("=== CONVERSATION ===\n");
            for msg in conversation {
                let role_label = match msg.role {
                    MessageRole::User => format!("User ({})", user_id),
                    MessageRole::Assistant => "Assistant (SAGE)".to_string(),
                };
                context.push_str(&format!("{}: {}\n", role_label, msg.content));
            }
        }

        context
    }

    /// Clear conversation history for a user
    pub fn clear_conversation(&mut self, user_id: &str) {
        self.conversations.remove(user_id);
        self.summaries.remove(user_id);
    }

    /// Check if a user's conversation should be summarized
    pub fn should_summarize(&self, user_id: &str) -> bool {
        if let Some(conversation) = self.conversations.get(user_id) {
            conversation.len() > self.summarization_threshold
        } else {
            false
        }
    }

    /// Summarize a user's conversation using the LLM
    /// This is async and requires an LLM client
    pub async fn summarize_conversation(
        &mut self,
        user_id: &str,
        llm_client: &crate::llm_client::LlmClient,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let conversation = match self.conversations.get(user_id) {
            Some(conv) if conv.len() > self.summarization_threshold => conv,
            _ => return Ok(()), // Nothing to summarize
        };

        // Build a summary of older messages (all except the most recent N)
        let messages_to_summarize: Vec<_> = conversation
            .iter()
            .take(conversation.len() - self.max_recent_messages)
            .collect();

        if messages_to_summarize.is_empty() {
            return Ok(());
        }

        // Format messages for summarization
        let mut messages_text = String::new();
        for msg in &messages_to_summarize {
            let role = match msg.role {
                MessageRole::User => user_id,
                MessageRole::Assistant => "SAGE",
            };
            messages_text.push_str(&format!("{}: {}\n", role, msg.content));
        }

        // Get existing summary if present
        let existing_summary = self.summaries.get(user_id).cloned().unwrap_or_default();

        // Create summarization prompt
        let summarization_context = if existing_summary.is_empty() {
            format!(
                "Summarize this conversation in 2-3 concise sentences, capturing key topics, \
                questions, and SAGE's responses:\n\n{}",
                messages_text
            )
        } else {
            format!(
                "Previous summary: {}\n\n\
                New messages:\n{}\n\n\
                Create an updated summary (2-3 sentences) that combines the previous summary \
                with these new messages:",
                existing_summary, messages_text
            )
        };

        // Generate summary using LLM
        let new_summary = llm_client
            .generate(&summarization_context, "You are a conversation summarizer.")
            .await?;

        // Store the summary
        self.summaries.insert(user_id.to_string(), new_summary);

        // Keep only recent messages
        if let Some(conversation) = self.conversations.get_mut(user_id) {
            while conversation.len() > self.max_recent_messages {
                conversation.pop_front();
            }
        }

        Ok(())
    }

    /// Get the current summary for a user
    pub fn get_summary(&self, user_id: &str) -> Option<&String> {
        self.summaries.get(user_id)
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

    /// Load conversations from SpacetimeDB for a specific user
    pub fn load_from_database(
        &mut self,
        username: &str,
        db_client: &crate::spacetime_client::SageDbClient,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Query recent conversations from database
        let db_conversations = db_client.get_user_conversations(username, 20)?;

        // Convert database records to ConversationMessages and add to manager
        let mut messages = VecDeque::new();

        for record in db_conversations {
            // Add user message
            messages.push_back(ConversationMessage {
                role: MessageRole::User,
                content: record.message,
                timestamp: self.current_timestamp(),
            });

            // Add assistant message
            messages.push_back(ConversationMessage {
                role: MessageRole::Assistant,
                content: record.sage_response,
                timestamp: self.current_timestamp() + 1,
            });
        }

        // Keep only the most recent messages (to avoid overloading context)
        while messages.len() > self.max_recent_messages * 2 {
            messages.pop_front();
        }

        // Store in conversations map
        self.conversations.insert(username.to_string(), messages);

        Ok(())
    }

    /// Load all users' conversations from SpacetimeDB
    pub fn load_all_from_database(
        db_client: &crate::spacetime_client::SageDbClient,
        usernames: &[String],
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut manager = Self::new();

        for username in usernames {
            let _ = manager.load_from_database(username, db_client);
        }

        Ok(manager)
    }

    /// Save conversations to a JSON file (DEPRECATED - use SpacetimeDB)
    #[deprecated(note = "Use SpacetimeDB persistence instead")]
    pub fn save(&self, _path: &str) -> Result<(), Box<dyn std::error::Error>> {
        // No-op: conversations are automatically saved to SpacetimeDB
        Ok(())
    }

    /// Load conversations from a JSON file (DEPRECATED - use load_from_database)
    #[deprecated(note = "Use load_from_database instead")]
    pub fn load(_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        // Return empty manager - use load_from_database instead
        Ok(Self::new())
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
    fn test_messages_accumulate() {
        let mut manager = ConversationContextManager::new();

        // Add multiple messages (no auto-trimming)
        for i in 0..15 {
            manager.add_user_message("bob", format!("Message {}", i));
        }

        let conv = manager.get_conversation("bob");
        assert_eq!(conv.len(), 15); // All messages kept until manual summarization
    }

    #[test]
    fn test_format_context() {
        let mut manager = ConversationContextManager::new();

        manager.add_user_message("charlie", "What's your name?".to_string());
        manager.add_assistant_message("charlie", "I'm SAGE!".to_string());

        let context = manager.format_context("charlie");
        // Format is "User (username): message" and "Assistant (SAGE): message"
        assert!(context.contains("User (charlie): What's your name?"));
        assert!(context.contains("Assistant (SAGE): I'm SAGE!"));
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
