//! Text Embedding Engine using Ollama
//!
//! Generates dense vector embeddings for text using local Ollama models.
//! Used for semantic search in SAGE's RAG (Retrieval-Augmented Generation) system.

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::error::Error;

/// Request format for Ollama embeddings API
#[derive(Debug, Serialize)]
struct EmbeddingRequest {
    model: String,
    prompt: String,
}

/// Response format from Ollama embeddings API
#[derive(Debug, Deserialize)]
struct EmbeddingResponse {
    embedding: Vec<f64>,
}

/// Text embedding engine
#[derive(Clone)]
pub struct EmbeddingEngine {
    client: Client,
    model: String,
    ollama_url: String,
}

impl EmbeddingEngine {
    /// Create a new embedding engine
    ///
    /// # Arguments
    /// * `model` - Model name (default: "nomic-embed-text")
    /// * `ollama_url` - Ollama API URL (default: "http://localhost:11434")
    pub fn new(model: Option<String>, ollama_url: Option<String>) -> Self {
        Self {
            client: Client::new(),
            model: model.unwrap_or_else(|| "nomic-embed-text".to_string()),
            ollama_url: ollama_url.unwrap_or_else(|| "http://localhost:11434".to_string()),
        }
    }

    /// Generate embedding for a text string
    ///
    /// Returns a 768-dimensional vector (for nomic-embed-text)
    pub async fn embed(&self, text: &str) -> Result<Vec<f64>, Box<dyn Error>> {
        let url = format!("{}/api/embeddings", self.ollama_url);

        let request = EmbeddingRequest {
            model: self.model.clone(),
            prompt: text.to_string(),
        };

        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(format!("Ollama API error: {}", response.status()).into());
        }

        let embedding_response: EmbeddingResponse = response.json().await?;
        Ok(embedding_response.embedding)
    }

    /// Embed multiple texts in batch (sequential for now, can be parallelized)
    pub async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f64>>, Box<dyn Error>> {
        let mut embeddings = Vec::new();
        for text in texts {
            embeddings.push(self.embed(text).await?);
        }
        Ok(embeddings)
    }

    /// Calculate cosine similarity between two embeddings
    ///
    /// Returns a value between -1.0 (opposite) and 1.0 (identical)
    pub fn cosine_similarity(&self, a: &[f64], b: &[f64]) -> f64 {
        assert_eq!(a.len(), b.len(), "Embeddings must have same dimension");

        let dot: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let mag_a: f64 = a.iter().map(|x| x * x).sum::<f64>().sqrt();
        let mag_b: f64 = b.iter().map(|x| x * x).sum::<f64>().sqrt();

        if mag_a == 0.0 || mag_b == 0.0 {
            return 0.0;
        }

        dot / (mag_a * mag_b)
    }

    /// Find the most similar embedding from a list
    ///
    /// Returns (index, similarity_score)
    pub fn find_most_similar(&self, query: &[f64], candidates: &[Vec<f64>]) -> Option<(usize, f64)> {
        if candidates.is_empty() {
            return None;
        }

        let mut best_idx = 0;
        let mut best_score = self.cosine_similarity(query, &candidates[0]);

        for (idx, candidate) in candidates.iter().enumerate().skip(1) {
            let score = self.cosine_similarity(query, candidate);
            if score > best_score {
                best_score = score;
                best_idx = idx;
            }
        }

        Some((best_idx, best_score))
    }
}

impl Default for EmbeddingEngine {
    fn default() -> Self {
        Self::new(None, None)
    }
}

/// A conversation stored with its embedding for semantic search
#[derive(Clone, Serialize, Deserialize)]
pub struct EmbeddedConversation {
    pub username: String,
    pub user_message: String,
    pub sage_response: String,
    pub embedding: Vec<f64>,
    pub timestamp: u64,
}

/// Semantic memory - stores conversations with embeddings for RAG
pub struct SemanticMemory {
    conversations: Vec<EmbeddedConversation>,
    engine: EmbeddingEngine,
}

impl SemanticMemory {
    pub fn new() -> Self {
        Self {
            conversations: Vec::new(),
            engine: EmbeddingEngine::default(),
        }
    }

    /// Add a conversation to memory (generates embedding automatically)
    pub async fn add(
        &mut self,
        username: &str,
        user_message: &str,
        sage_response: &str,
    ) -> Result<(), Box<dyn Error>> {
        // Embed the combined message for better semantic matching
        let text_to_embed = format!("{}: {}", username, user_message);
        let embedding = self.engine.embed(&text_to_embed).await?;

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        self.conversations.push(EmbeddedConversation {
            username: username.to_string(),
            user_message: user_message.to_string(),
            sage_response: sage_response.to_string(),
            embedding,
            timestamp,
        });

        println!("\x1b[35m[MEM]\x1b[0m Stored memory \x1b[1m#{}\x1b[0m for \x1b[36m{}\x1b[0m", self.conversations.len(), username);
        Ok(())
    }

    /// Find relevant past conversations for a query
    /// Returns conversations sorted by relevance (most relevant first)
    pub async fn find_relevant(
        &self,
        query: &str,
        username: Option<&str>,
        limit: usize,
    ) -> Result<Vec<&EmbeddedConversation>, Box<dyn Error>> {
        if self.conversations.is_empty() {
            return Ok(Vec::new());
        }

        // Embed the query
        let query_embedding = self.engine.embed(query).await?;

        // Score all conversations
        let mut scored: Vec<(f64, &EmbeddedConversation)> = self.conversations
            .iter()
            .filter(|conv| username.map_or(true, |u| conv.username == u))
            .map(|conv| {
                let score = self.engine.cosine_similarity(&query_embedding, &conv.embedding);
                (score, conv)
            })
            .collect();

        // Sort by score (highest first)
        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        // Return top results
        Ok(scored.into_iter().take(limit).map(|(_, conv)| conv).collect())
    }

    /// Format relevant memories as context for the LLM
    pub async fn get_context(
        &self,
        query: &str,
        username: Option<&str>,
        limit: usize,
    ) -> Result<String, Box<dyn Error>> {
        let relevant = self.find_relevant(query, username, limit).await?;

        if relevant.is_empty() {
            return Ok(String::new());
        }

        let mut context = String::from("=== RELEVANT MEMORIES ===\n");
        for conv in relevant {
            context.push_str(&format!(
                "{} said: \"{}\"\nSAGE replied: \"{}\"\n\n",
                conv.username, conv.user_message, conv.sage_response
            ));
        }

        Ok(context)
    }

    pub fn len(&self) -> usize {
        self.conversations.len()
    }

    pub fn is_empty(&self) -> bool {
        self.conversations.is_empty()
    }

    /// Save memories to file
    pub fn save(&self, path: &str) -> Result<(), Box<dyn Error>> {
        let json = serde_json::to_string_pretty(&self.conversations)?;
        std::fs::write(path, json)?;
        println!("💾 Saved {} memories to {}", self.conversations.len(), path);
        Ok(())
    }

    /// Load memories from file
    pub fn load(path: &str) -> Result<Self, Box<dyn Error>> {
        let json = std::fs::read_to_string(path)?;
        let conversations: Vec<EmbeddedConversation> = serde_json::from_str(&json)?;
        println!("📚 Loaded {} memories from {}", conversations.len(), path);
        Ok(Self {
            conversations,
            engine: EmbeddingEngine::default(),
        })
    }

    /// Load or create new
    pub fn load_or_new(path: &str) -> Self {
        Self::load(path).unwrap_or_else(|_| {
            println!("📝 Starting fresh semantic memory");
            Self::new()
        })
    }
}

impl Default for SemanticMemory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_embedding_generation() {
        let engine = EmbeddingEngine::default();

        let embedding = engine.embed("Hello, world!").await;
        assert!(embedding.is_ok());

        let emb = embedding.unwrap();
        assert_eq!(emb.len(), 768); // nomic-embed-text returns 768-dim vectors
    }

    #[test]
    fn test_cosine_similarity() {
        let engine = EmbeddingEngine::default();

        // Same vectors = similarity 1.0
        let v1 = vec![1.0, 2.0, 3.0];
        let v2 = vec![1.0, 2.0, 3.0];
        let sim = engine.cosine_similarity(&v1, &v2);
        assert!((sim - 1.0).abs() < 0.001);

        // Opposite vectors = similarity -1.0
        let v3 = vec![-1.0, -2.0, -3.0];
        let sim2 = engine.cosine_similarity(&v1, &v3);
        assert!((sim2 + 1.0).abs() < 0.001);

        // Orthogonal vectors = similarity 0.0
        let v4 = vec![1.0, 0.0, 0.0];
        let v5 = vec![0.0, 1.0, 0.0];
        let sim3 = engine.cosine_similarity(&v4, &v5);
        assert!(sim3.abs() < 0.001);
    }
}
