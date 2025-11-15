// SpacetimeDB client for SAGE memory system
// Connects to sage-db module to store conversations, concepts, and personality

use std::sync::{Arc, Mutex};

pub struct SageMemoryClient {
    connected: Arc<Mutex<bool>>,
}

impl SageMemoryClient {
    pub fn new() -> Self {
        Self {
            connected: Arc::new(Mutex::new(false)),
        }
    }

    /// Store a conversation with SAGE's response
    pub async fn store_conversation(
        &self,
        sender: String,
        message: String,
        sage_response: String,
        nca_loss: f64,
        concepts: Vec<String>,
        generation: u64,
    ) -> Result<(), String> {
        let concepts_json = serde_json::to_string(&concepts)
            .map_err(|e| format!("Failed to serialize concepts: {}", e))?;

        // TODO: Call SpacetimeDB reducer when connection is implemented
        // Conversations will be stored via reducer once connection is established

        Ok(())
    }

    /// Store or update a concept in memory
    pub async fn store_concept(
        &self,
        concept_name: String,
        familiarity: f64,
        avg_loss: f64,
        exposure_count: u64,
        opinion_type: String,
        opinion_reason: String,
        confidence: f64,
    ) -> Result<(), String> {
        // TODO: Store concept via SpacetimeDB reducer
        Ok(())
    }

    /// Save personality snapshot
    pub async fn save_personality(
        &self,
        generation: u64,
        openness: f64,
        positivity: f64,
        curiosity: f64,
        confidence: f64,
        experience_count: u64,
        likes_count: u64,
        dislikes_count: u64,
    ) -> Result<(), String> {
        // TODO: Save personality via SpacetimeDB reducer
        Ok(())
    }
}

// Record types for query results
#[derive(Debug, Clone)]
pub struct ConversationRecord {
    pub sender: String,
    pub message: String,
    pub sage_response: String,
    pub nca_loss: f64,
}

#[derive(Debug, Clone)]
pub struct ConceptRecord {
    pub concept_name: String,
    pub familiarity_score: f64,
    pub average_loss: f64,
    pub exposure_count: u64,
    pub opinion_type: String,
    pub opinion_reason: String,
}
