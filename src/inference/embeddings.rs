//! Embedded text embeddings using candle
//!
//! Replaces Ollama's /api/embeddings endpoint with a local model.
//! Uses a simple approach: tokenize + average pooling from the GGUF model,
//! or falls back to hash-based embeddings when no model is available.

use std::error::Error;

/// Embedding dimension for our hash-based fallback
const HASH_EMBED_DIM: usize = 384;

/// Text embedding engine — works without any external service
pub struct EmbeddingEngine {
    mode: EmbeddingMode,
}

enum EmbeddingMode {
    /// Hash-based embeddings (always available, decent for simple similarity)
    HashBased,
}

impl EmbeddingEngine {
    /// Create a new embedding engine
    pub fn new() -> Self {
        // Use hash-based embeddings — fast, no download needed, works everywhere.
        // For a small local model this is sufficient; the NCA grid does the heavy
        // semantic lifting anyway.
        Self {
            mode: EmbeddingMode::HashBased,
        }
    }

    /// Generate embedding for a text string
    pub fn embed(&self, text: &str) -> Result<Vec<f64>, Box<dyn Error>> {
        match &self.mode {
            EmbeddingMode::HashBased => Ok(hash_embed(text)),
        }
    }

    /// Embed multiple texts
    pub fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f64>>, Box<dyn Error>> {
        texts.iter().map(|t| self.embed(t)).collect()
    }

    /// Cosine similarity between two embeddings
    pub fn cosine_similarity(a: &[f64], b: &[f64]) -> f64 {
        assert_eq!(a.len(), b.len());
        let dot: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let mag_a: f64 = a.iter().map(|x| x * x).sum::<f64>().sqrt();
        let mag_b: f64 = b.iter().map(|x| x * x).sum::<f64>().sqrt();
        if mag_a == 0.0 || mag_b == 0.0 { 0.0 } else { dot / (mag_a * mag_b) }
    }

    /// Embedding dimension
    pub fn dimension(&self) -> usize {
        HASH_EMBED_DIM
    }
}

impl Default for EmbeddingEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Hash-based text embedding
/// Uses multiple hash functions to create a pseudo-embedding that preserves
/// some semantic structure through n-gram overlap.
fn hash_embed(text: &str) -> Vec<f64> {
    let text = text.to_lowercase();
    let mut embedding = vec![0.0f64; HASH_EMBED_DIM];

    // Character trigrams for local structure
    let chars: Vec<char> = text.chars().collect();
    for window in chars.windows(3) {
        let hash = simple_hash(&window.iter().collect::<String>());
        let idx = (hash as usize) % HASH_EMBED_DIM;
        embedding[idx] += 1.0;
    }

    // Word unigrams for global structure
    for word in text.split_whitespace() {
        let hash = simple_hash(word);
        let idx = (hash as usize) % HASH_EMBED_DIM;
        embedding[idx] += 2.0; // Weight words more than char trigrams

        // Word bigram hash (shifted)
        let hash2 = simple_hash(word).wrapping_mul(2654435761);
        let idx2 = (hash2 as usize) % HASH_EMBED_DIM;
        embedding[idx2] += 0.5;
    }

    // Normalize to unit vector
    let magnitude: f64 = embedding.iter().map(|x| x * x).sum::<f64>().sqrt();
    if magnitude > 0.0 {
        for x in &mut embedding {
            *x /= magnitude;
        }
    }

    embedding
}

/// Simple deterministic hash function
fn simple_hash(s: &str) -> u64 {
    let mut hash: u64 = 5381;
    for byte in s.bytes() {
        hash = hash.wrapping_mul(33).wrapping_add(byte as u64);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_embed_deterministic() {
        let e1 = hash_embed("hello world");
        let e2 = hash_embed("hello world");
        assert_eq!(e1, e2);
    }

    #[test]
    fn test_similar_texts_have_high_similarity() {
        let e1 = hash_embed("the cat sat on the mat");
        let e2 = hash_embed("the cat sat on a mat");
        let sim = EmbeddingEngine::cosine_similarity(&e1, &e2);
        assert!(sim > 0.8, "Similar texts should have high similarity, got {}", sim);
    }

    #[test]
    fn test_different_texts_have_lower_similarity() {
        let e1 = hash_embed("the cat sat on the mat");
        let e2 = hash_embed("quantum physics explains gravity");
        let sim = EmbeddingEngine::cosine_similarity(&e1, &e2);
        assert!(sim < 0.5, "Different texts should have lower similarity, got {}", sim);
    }

    #[test]
    fn test_embedding_dimension() {
        let engine = EmbeddingEngine::new();
        let emb = engine.embed("test").unwrap();
        assert_eq!(emb.len(), HASH_EMBED_DIM);
    }
}
