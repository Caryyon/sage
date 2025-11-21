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
