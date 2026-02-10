//! Embedded text embeddings using candle
//!
//! Supports two modes:
//! - **Semantic**: all-MiniLM-L6-v2 via candle (BERT transformer, 384-dim)
//! - **HashBased**: deterministic hash-based fallback (always available)

use candle_core::{DType, Device, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::models::bert::{BertModel, Config as BertConfig};
use std::error::Error;
use tokenizers::Tokenizer;

/// Embedding dimension (both modes produce 384-dim vectors)
const EMBED_DIM: usize = 384;

/// HuggingFace repo for the embedding model
const EMBED_REPO: &str = "sentence-transformers/all-MiniLM-L6-v2";

/// Text embedding engine — works without any external service
pub struct EmbeddingEngine {
    mode: EmbeddingMode,
}

enum EmbeddingMode {
    /// Semantic embeddings via all-MiniLM-L6-v2
    Semantic {
        model: BertModel,
        tokenizer: Tokenizer,
        device: Device,
    },
    /// Hash-based embeddings (always available, decent for simple similarity)
    HashBased,
}

impl EmbeddingEngine {
    /// Create a new embedding engine.
    /// Tries to load the semantic model; falls back to hash-based on failure.
    pub fn new() -> Self {
        match Self::try_load_semantic() {
            Ok(mode) => {
                eprintln!("🧠 Embedding mode: Semantic (all-MiniLM-L6-v2)");
                Self { mode }
            }
            Err(e) => {
                eprintln!("⚠️  Semantic embeddings unavailable ({}), using hash-based fallback", e);
                Self {
                    mode: EmbeddingMode::HashBased,
                }
            }
        }
    }

    fn try_load_semantic() -> Result<EmbeddingMode, Box<dyn Error>> {
        let api = hf_hub::api::sync::Api::new()?;
        let repo = api.model(EMBED_REPO.to_string());

        let config_path = repo.get("config.json")?;
        let tokenizer_path = repo.get("tokenizer.json")?;
        let weights_path = repo.get("model.safetensors")?;

        let config: BertConfig = serde_json::from_str(&std::fs::read_to_string(&config_path)?)?;
        let tokenizer = Tokenizer::from_file(&tokenizer_path)
            .map_err(|e| format!("tokenizer load error: {}", e))?;

        let device = Device::Cpu;
        let vb = unsafe {
            VarBuilder::from_mmaped_safetensors(&[weights_path], DType::F32, &device)?
        };
        let model = BertModel::load(vb, &config)?;

        Ok(EmbeddingMode::Semantic {
            model,
            tokenizer,
            device,
        })
    }

    /// Generate embedding for a text string
    pub fn embed(&self, text: &str) -> Result<Vec<f64>, Box<dyn Error>> {
        match &self.mode {
            EmbeddingMode::Semantic {
                model,
                tokenizer,
                device,
            } => semantic_embed(model, tokenizer, device, text),
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
        if mag_a == 0.0 || mag_b == 0.0 {
            0.0
        } else {
            dot / (mag_a * mag_b)
        }
    }

    /// Embedding dimension
    pub fn dimension(&self) -> usize {
        EMBED_DIM
    }
}

impl Default for EmbeddingEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Semantic embedding via BERT forward pass + mean pooling + L2 normalize
fn semantic_embed(
    model: &BertModel,
    tokenizer: &Tokenizer,
    device: &Device,
    text: &str,
) -> Result<Vec<f64>, Box<dyn Error>> {
    let encoding = tokenizer
        .encode(text, true)
        .map_err(|e| format!("tokenize error: {}", e))?;

    let input_ids = encoding.get_ids();
    let token_type_ids = encoding.get_type_ids();
    let attention_mask = encoding.get_attention_mask();

    let _len = input_ids.len();
    let input_ids = Tensor::new(input_ids, device)?.unsqueeze(0)?;
    let token_type_ids = Tensor::new(token_type_ids, device)?.unsqueeze(0)?;
    let attention_mask_tensor = Tensor::new(attention_mask, device)?
        .unsqueeze(0)?
        .to_dtype(DType::F32)?;

    // Forward pass: [1, seq_len, hidden_size]
    let output = model.forward(&input_ids, &token_type_ids, Some(&attention_mask_tensor))?;

    // Mean pooling over non-padding tokens
    let mask = attention_mask_tensor.unsqueeze(2)?; // [1, seq_len, 1]
    let masked = output.broadcast_mul(&mask)?;
    let summed = masked.sum(1)?; // [1, hidden_size]
    let count = mask.sum(1)?; // [1, 1]
    let pooled = summed.broadcast_div(&count)?; // [1, hidden_size]

    // Extract as f32, convert to f64
    let pooled = pooled.squeeze(0)?; // [hidden_size]
    let values: Vec<f32> = pooled.to_vec1()?;

    // L2 normalize
    let mag: f64 = values.iter().map(|x| (*x as f64) * (*x as f64)).sum::<f64>().sqrt();
    let embedding: Vec<f64> = if mag > 0.0 {
        values.iter().map(|x| (*x as f64) / mag).collect()
    } else {
        values.iter().map(|x| *x as f64).collect()
    };

    debug_assert_eq!(embedding.len(), EMBED_DIM);
    Ok(embedding)
}

/// Hash-based text embedding
/// Uses multiple hash functions to create a pseudo-embedding that preserves
/// some semantic structure through n-gram overlap.
fn hash_embed(text: &str) -> Vec<f64> {
    let text = text.to_lowercase();
    let mut embedding = vec![0.0f64; EMBED_DIM];

    // Character trigrams for local structure
    let chars: Vec<char> = text.chars().collect();
    for window in chars.windows(3) {
        let hash = simple_hash(&window.iter().collect::<String>());
        let idx = (hash as usize) % EMBED_DIM;
        embedding[idx] += 1.0;
    }

    // Word unigrams for global structure
    for word in text.split_whitespace() {
        let hash = simple_hash(word);
        let idx = (hash as usize) % EMBED_DIM;
        embedding[idx] += 2.0;

        // Word bigram hash (shifted)
        let hash2 = simple_hash(word).wrapping_mul(2654435761);
        let idx2 = (hash2 as usize) % EMBED_DIM;
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
        assert!(
            sim > 0.8,
            "Similar texts should have high similarity, got {}",
            sim
        );
    }

    #[test]
    fn test_different_texts_have_lower_similarity() {
        let e1 = hash_embed("the cat sat on the mat");
        let e2 = hash_embed("quantum physics explains gravity");
        let sim = EmbeddingEngine::cosine_similarity(&e1, &e2);
        assert!(
            sim < 0.5,
            "Different texts should have lower similarity, got {}",
            sim
        );
    }

    #[test]
    fn test_embedding_dimension() {
        let engine = EmbeddingEngine::new();
        let emb = engine.embed("test").unwrap();
        assert_eq!(emb.len(), EMBED_DIM);
    }

    #[test]
    fn test_semantic_beats_hash_for_synonyms() {
        let engine = EmbeddingEngine::new();

        let e_dog = engine.embed("dog").unwrap();
        let e_canine = engine.embed("canine").unwrap();
        let semantic_sim = EmbeddingEngine::cosine_similarity(&e_dog, &e_canine);

        let h_dog = hash_embed("dog");
        let h_canine = hash_embed("canine");
        let hash_sim = EmbeddingEngine::cosine_similarity(&h_dog, &h_canine);

        eprintln!(
            "dog/canine — semantic: {:.4}, hash: {:.4}",
            semantic_sim, hash_sim
        );

        // Semantic model should recognize synonyms better than hash
        // If semantic model isn't available, both use hash and this is a no-op check
        match engine.mode {
            EmbeddingMode::Semantic { .. } => {
                assert!(
                    semantic_sim > hash_sim,
                    "Semantic similarity ({:.4}) should beat hash ({:.4}) for dog/canine",
                    semantic_sim,
                    hash_sim
                );
            }
            EmbeddingMode::HashBased => {
                eprintln!("⚠️  Semantic model not available, skipping comparison");
            }
        }
    }
}
