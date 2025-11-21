//! Fact Memory System
//!
//! Extracts and stores specific facts about users from conversations.
//! Facts are stored in SpacetimeDB for persistence across restarts.

use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// A single fact about a user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fact {
    pub key: String,          // e.g., "name", "wife_name", "son_name", "preference:food"
    pub value: String,        // The actual value
    pub confidence: f32,      // 0.0-1.0, how confident we are about this fact
    pub last_mentioned: u64,  // Timestamp when this was last mentioned
    pub mention_count: u32,   // How many times this has been mentioned
}

/// Manages facts for all users
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactMemory {
    /// Per-user facts: user_id -> (fact_key -> Fact)
    facts: HashMap<String, HashMap<String, Fact>>,
}

impl FactMemory {
    /// Create a new fact memory system
    pub fn new() -> Self {
        Self {
            facts: HashMap::new(),
        }
    }

    /// Extract facts from a conversation using LLM
    pub async fn extract_facts_from_conversation(
        &mut self,
        user_id: &str,
        conversation_text: &str,
        llm_client: &crate::llm_client::LlmClient,
    ) -> Result<Vec<Fact>, Box<dyn std::error::Error>> {
        let extraction_prompt = format!(
            "Extract specific facts about the user from this conversation. \
            Focus on:\n\
            - User's name (key: \"name\")\n\
            - Family members and their names (key: \"wife_name\", \"son_name\", \"daughter_name\", etc.)\n\
            - Preferences (key: \"preference:topic\")\n\
            - Important life details (key: \"detail:topic\")\n\n\
            Conversation:\n{}\n\n\
            Output ONLY facts in this exact format, one per line:\n\
            KEY|VALUE\n\n\
            Example:\n\
            name|Cary\n\
            wife_name|Resse\n\
            son_name|Loki\n\
            preference:language|Japanese\n\n\
            If no facts are found, output: NONE",
            conversation_text
        );

        let response = llm_client
            .generate(&extraction_prompt, "You are a fact extraction system. Extract only clear, explicit facts.")
            .await?;

        let mut extracted_facts = Vec::new();
        let timestamp = current_timestamp();

        for line in response.lines() {
            let line = line.trim();
            if line.is_empty() || line == "NONE" {
                continue;
            }

            if let Some((key, value)) = line.split_once('|') {
                let key = key.trim().to_lowercase();
                let value = value.trim().to_string();

                if !key.is_empty() && !value.is_empty() {
                    extracted_facts.push(Fact {
                        key: key.clone(),
                        value: value.clone(),
                        confidence: 0.8, // High confidence from explicit conversation
                        last_mentioned: timestamp,
                        mention_count: 1,
                    });

                    // Store in memory
                    self.add_or_update_fact(user_id, key, value, 0.8);
                }
            }
        }

        Ok(extracted_facts)
    }

    /// Add or update a fact for a user
    pub fn add_or_update_fact(&mut self, user_id: &str, key: String, value: String, confidence: f32) {
        let user_facts = self.facts
            .entry(user_id.to_string())
            .or_insert_with(HashMap::new);

        if let Some(existing_fact) = user_facts.get_mut(&key) {
            // Update existing fact
            existing_fact.value = value;
            existing_fact.confidence = existing_fact.confidence.max(confidence); // Keep higher confidence
            existing_fact.last_mentioned = current_timestamp();
            existing_fact.mention_count += 1;
        } else {
            // Add new fact
            user_facts.insert(key.clone(), Fact {
                key,
                value,
                confidence,
                last_mentioned: current_timestamp(),
                mention_count: 1,
            });
        }
    }

    /// Get a specific fact for a user
    pub fn get_fact(&self, user_id: &str, key: &str) -> Option<&Fact> {
        self.facts.get(user_id)?.get(key)
    }

    /// Get all facts for a user
    pub fn get_user_facts(&self, user_id: &str) -> Vec<&Fact> {
        if let Some(user_facts) = self.facts.get(user_id) {
            user_facts.values().collect()
        } else {
            Vec::new()
        }
    }

    /// Format facts for inclusion in LLM context
    pub fn format_facts_for_context(&self, user_id: &str) -> String {
        let facts = self.get_user_facts(user_id);

        if facts.is_empty() {
            return String::new();
        }

        let mut context = String::from("=== WHAT I KNOW ABOUT YOU ===\n");

        // Sort by mention count (most mentioned first)
        let mut sorted_facts = facts.clone();
        sorted_facts.sort_by(|a, b| b.mention_count.cmp(&a.mention_count));

        for fact in sorted_facts {
            if fact.confidence > 0.5 {  // Only include high-confidence facts
                let fact_desc = match fact.key.as_str() {
                    "name" => format!("Your name is {}", fact.value),
                    "wife_name" => format!("Your wife's name is {}", fact.value),
                    "husband_name" => format!("Your husband's name is {}", fact.value),
                    "son_name" => format!("Your son's name is {}", fact.value),
                    "daughter_name" => format!("Your daughter's name is {}", fact.value),
                    key if key.starts_with("preference:") => {
                        let topic = key.strip_prefix("preference:").unwrap();
                        format!("You prefer {} for {}", fact.value, topic)
                    },
                    key if key.starts_with("detail:") => {
                        let topic = key.strip_prefix("detail:").unwrap();
                        format!("About {}: {}", topic, fact.value)
                    },
                    _ => format!("{}: {}", fact.key, fact.value),
                };
                context.push_str(&format!("• {} (mentioned {} times)\n", fact_desc, fact.mention_count));
            }
        }

        context.push_str("\n");
        context
    }

    /// Save facts to SpacetimeDB (TODO: Implement database methods)
    #[allow(dead_code)]
    pub fn save_to_database(
        &self,
        _user_id: &str,
        _db_client: &crate::spacetime_client::SageDbClient,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // TODO: Implement when database schema is ready
        // if let Some(user_facts) = self.facts.get(user_id) {
        //     for fact in user_facts.values() {
        //         db_client.store_user_fact(
        //             user_id,
        //             &fact.key,
        //             &fact.value,
        //             fact.confidence,
        //             fact.mention_count,
        //         )?;
        //     }
        // }
        Ok(())
    }

    /// Load facts from SpacetimeDB (TODO: Implement database methods)
    #[allow(dead_code)]
    pub fn load_from_database(
        &mut self,
        _user_id: &str,
        _db_client: &crate::spacetime_client::SageDbClient,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // TODO: Implement when database schema is ready
        // let facts = db_client.get_user_facts(user_id)?;
        //
        // if !facts.is_empty() {
        //     eprintln!("📚 Loaded {} facts for {} from database", facts.len(), user_id);
        //
        //     for fact in facts {
        //         self.add_or_update_fact(
        //             user_id,
        //             fact.key,
        //             fact.value,
        //             fact.confidence,
        //         );
        //     }
        // }

        Ok(())
    }

    /// Clear all facts for a user
    pub fn clear_user_facts(&mut self, user_id: &str) {
        self.facts.remove(user_id);
    }
}

impl Default for FactMemory {
    fn default() -> Self {
        Self::new()
    }
}

fn current_timestamp() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_fact() {
        let mut memory = FactMemory::new();
        memory.add_or_update_fact("alice", "name".to_string(), "Alice".to_string(), 0.9);

        let fact = memory.get_fact("alice", "name").unwrap();
        assert_eq!(fact.value, "Alice");
        assert_eq!(fact.confidence, 0.9);
    }

    #[test]
    fn test_update_fact() {
        let mut memory = FactMemory::new();
        memory.add_or_update_fact("bob", "name".to_string(), "Bob".to_string(), 0.8);
        memory.add_or_update_fact("bob", "name".to_string(), "Robert".to_string(), 0.9);

        let fact = memory.get_fact("bob", "name").unwrap();
        assert_eq!(fact.value, "Robert");
        assert_eq!(fact.mention_count, 2);
    }

    #[test]
    fn test_format_context() {
        let mut memory = FactMemory::new();
        memory.add_or_update_fact("charlie", "name".to_string(), "Charlie".to_string(), 0.9);
        memory.add_or_update_fact("charlie", "wife_name".to_string(), "Diana".to_string(), 0.85);

        let context = memory.format_facts_for_context("charlie");
        assert!(context.contains("Your name is Charlie"));
        assert!(context.contains("Your wife's name is Diana"));
    }
}
