//! Vector Memory Store for Semantic Search
//!
//! In-memory vector database for SAGE's RAG system.
//! Stores text entries with embeddings and enables semantic similarity search.

use crate::embeddings::EmbeddingEngine;
use std::error::Error;

/// Type of memory entry
#[derive(Debug, Clone, PartialEq)]
pub enum MemoryType {
    Dream,
    Curiosity,
    Conversation,
    Goal,
    Thought,
}

impl MemoryType {
    pub fn from_str(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "DREAM" => MemoryType::Dream,
            "CURIOSITY" => MemoryType::Curiosity,
            "CONVERSATION" => MemoryType::Conversation,
            "GOAL" => MemoryType::Goal,
            _ => MemoryType::Thought,
        }
    }
}

/// A single memory entry with text, embedding, and metadata
#[derive(Debug, Clone)]
pub struct MemoryEntry {
    pub text: String,
    pub embedding: Vec<f64>,
    pub memory_type: MemoryType,
    pub timestamp: u64,  // Unix timestamp
    pub metadata: Option<String>,  // Optional additional context
}

/// In-memory vector store for semantic search
pub struct VectorMemory {
    entries: Vec<MemoryEntry>,
    embedding_engine: EmbeddingEngine,
}

impl VectorMemory {
    /// Create a new empty vector memory store
    pub fn new(embedding_engine: EmbeddingEngine) -> Self {
        Self {
            entries: Vec::new(),
            embedding_engine,
        }
    }

    /// Add a memory entry (text will be embedded automatically)
    pub async fn add(&mut self,
        text: String,
        memory_type: MemoryType,
        metadata: Option<String>
    ) -> Result<(), Box<dyn Error>> {
        let embedding = self.embedding_engine.embed(&text).await?;
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();

        self.entries.push(MemoryEntry {
            text,
            embedding,
            memory_type,
            timestamp,
            metadata,
        });

        Ok(())
    }

    /// Add a pre-embedded memory entry (useful for batch operations)
    pub fn add_embedded(&mut self,
        text: String,
        embedding: Vec<f64>,
        memory_type: MemoryType,
        metadata: Option<String>
    ) {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        self.entries.push(MemoryEntry {
            text,
            embedding,
            memory_type,
            timestamp,
            metadata,
        });
    }

    /// Search for most semantically similar memories
    ///
    /// # Arguments
    /// * `query` - The search query text
    /// * `top_k` - Number of results to return
    /// * `filter_type` - Optional filter by memory type
    /// * `min_similarity` - Minimum similarity threshold (0.0 to 1.0)
    ///
    /// # Returns
    /// Vector of (memory_entry, similarity_score) tuples, sorted by relevance
    pub async fn search(
        &self,
        query: &str,
        top_k: usize,
        filter_type: Option<MemoryType>,
        min_similarity: f64,
    ) -> Result<Vec<(MemoryEntry, f64)>, Box<dyn Error>> {
        if self.entries.is_empty() {
            return Ok(Vec::new());
        }

        // Embed the query
        let query_embedding = self.embedding_engine.embed(query).await?;

        // Filter entries by type if specified
        let candidates: Vec<&MemoryEntry> = if let Some(filter) = filter_type {
            self.entries.iter()
                .filter(|e| e.memory_type == filter)
                .collect()
        } else {
            self.entries.iter().collect()
        };

        if candidates.is_empty() {
            return Ok(Vec::new());
        }

        // Calculate similarity scores
        let mut scored_entries: Vec<(MemoryEntry, f64)> = candidates
            .iter()
            .map(|entry| {
                let score = self.embedding_engine.cosine_similarity(
                    &query_embedding,
                    &entry.embedding
                );
                ((*entry).clone(), score)
            })
            .filter(|(_, score)| *score >= min_similarity)
            .collect();

        // Sort by similarity (descending)
        scored_entries.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Return top K results
        Ok(scored_entries.into_iter().take(top_k).collect())
    }

    /// Get total number of memories stored
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if memory store is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Get all memories of a specific type
    pub fn get_by_type(&self, memory_type: MemoryType) -> Vec<&MemoryEntry> {
        self.entries.iter()
            .filter(|e| e.memory_type == memory_type)
            .collect()
    }

    /// Clear all memories
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Remove memories older than a certain timestamp
    pub fn prune_older_than(&mut self, timestamp: u64) {
        self.entries.retain(|e| e.timestamp >= timestamp);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_add_and_search() {
        let engine = EmbeddingEngine::default();
        let mut memory = VectorMemory::new(engine);

        // Add some test memories
        memory.add(
            "I dreamed about flying through colorful patterns".to_string(),
            MemoryType::Dream,
            None
        ).await.unwrap();

        memory.add(
            "I'm curious about how humans experience emotions".to_string(),
            MemoryType::Curiosity,
            None
        ).await.unwrap();

        memory.add(
            "The user told me their name is Alice".to_string(),
            MemoryType::Conversation,
            None
        ).await.unwrap();

        assert_eq!(memory.len(), 3);

        // Search for dreams
        let results = memory.search(
            "flying in my dreams",
            2,
            Some(MemoryType::Dream),
            0.0
        ).await.unwrap();

        assert!(results.len() > 0);
        assert_eq!(results[0].0.memory_type, MemoryType::Dream);
    }

    #[tokio::test]
    async fn test_filtering() {
        let engine = EmbeddingEngine::default();
        let mut memory = VectorMemory::new(engine);

        memory.add(
            "Dream about patterns".to_string(),
            MemoryType::Dream,
            None
        ).await.unwrap();

        memory.add(
            "Curious about consciousness".to_string(),
            MemoryType::Curiosity,
            None
        ).await.unwrap();

        let dreams = memory.get_by_type(MemoryType::Dream);
        assert_eq!(dreams.len(), 1);

        let curiosities = memory.get_by_type(MemoryType::Curiosity);
        assert_eq!(curiosities.len(), 1);
    }
}
