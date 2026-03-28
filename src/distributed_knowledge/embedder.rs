//! Bundled text embeddings via fastembed-rs
//!
//! Provides high-quality semantic embeddings using AllMiniLML6V2 (384-dim)
//! without requiring any external services like Ollama.
//!
//! The model is downloaded on first use to `~/.cache/fastembed/` and cached
//! for subsequent runs.

use fastembed::{EmbeddingModel, InitOptions, TextEmbedding as FastTextEmbedding};
use std::sync::OnceLock;

/// Global singleton for the embedding model (lazy-initialized on first use)
static EMBEDDER: OnceLock<Option<FastTextEmbedding>> = OnceLock::new();

/// Embedding dimension for AllMiniLML6V2
pub const EMBED_DIM: usize = 384;

/// Model name for display
pub const MODEL_NAME: &str = "AllMiniLML6V2";

/// Initialize the fastembed model (call once at startup or lazily)
fn init_embedder() -> Option<FastTextEmbedding> {
    let options = InitOptions::new(EmbeddingModel::AllMiniLML6V2).with_show_download_progress(true);
    match FastTextEmbedding::try_new(options) {
        Ok(model) => Some(model),
        Err(e) => {
            eprintln!("fastembed init error: {}", e);
            None
        }
    }
}

/// Get reference to the global embedder (initializes on first call)
fn get_embedder() -> Option<&'static FastTextEmbedding> {
    EMBEDDER.get_or_init(init_embedder).as_ref()
}

/// Check if fastembed is available (model loaded successfully)
pub fn is_available() -> bool {
    get_embedder().is_some()
}

/// Embed a single text string into a 384-dimensional vector.
/// Returns None if fastembed is unavailable.
pub fn embed_text(text: &str) -> Option<Vec<f32>> {
    let embedder = get_embedder()?;
    let texts = vec![text];
    match embedder.embed(texts, None) {
        Ok(embeddings) => embeddings.into_iter().next(),
        Err(e) => {
            eprintln!("fastembed embed error: {}", e);
            None
        }
    }
}

/// Embed multiple texts in a batch (more efficient than individual calls)
pub fn embed_batch(texts: &[&str]) -> Option<Vec<Vec<f32>>> {
    let embedder = get_embedder()?;
    let texts: Vec<&str> = texts.to_vec();
    match embedder.embed(texts, None) {
        Ok(embeddings) => Some(embeddings),
        Err(e) => {
            eprintln!("fastembed batch embed error: {}", e);
            None
        }
    }
}

/// Convert f32 embedding to f64 (for compatibility with existing code)
pub fn embed_text_f64(text: &str) -> Option<Vec<f64>> {
    embed_text(text).map(|v| v.into_iter().map(|x| x as f64).collect())
}

/// Get the embedding dimension
pub fn dimension() -> usize {
    EMBED_DIM
}

/// Get the model name for display
pub fn model_name() -> &'static str {
    MODEL_NAME
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dimension_is_384() {
        assert_eq!(dimension(), 384);
    }

    #[test]
    fn test_embed_text_returns_correct_dimension() {
        if !is_available() {
            eprintln!("Skipping test: fastembed not available");
            return;
        }
        let embedding = embed_text("hello world").expect("should embed");
        assert_eq!(
            embedding.len(),
            384,
            "Expected 384-dim embedding, got {}",
            embedding.len()
        );
    }

    #[test]
    fn test_embed_text_is_deterministic() {
        if !is_available() {
            eprintln!("Skipping test: fastembed not available");
            return;
        }
        let e1 = embed_text("test determinism").expect("should embed");
        let e2 = embed_text("test determinism").expect("should embed");
        assert_eq!(e1, e2, "Same input should produce same embedding");
    }

    #[test]
    fn test_semantic_similarity_cat_dog_vs_cat_car() {
        if !is_available() {
            eprintln!("Skipping test: fastembed not available");
            return;
        }

        let cat = embed_text("cat").expect("embed cat");
        let dog = embed_text("dog").expect("embed dog");
        let car = embed_text("car").expect("embed car");

        fn cosine_sim(a: &[f32], b: &[f32]) -> f32 {
            let dot: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
            let mag_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
            let mag_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
            if mag_a == 0.0 || mag_b == 0.0 {
                0.0
            } else {
                dot / (mag_a * mag_b)
            }
        }

        let sim_cat_dog = cosine_sim(&cat, &dog);
        let sim_cat_car = cosine_sim(&cat, &car);

        assert!(
            sim_cat_dog > sim_cat_car,
            "cat/dog similarity ({:.4}) should be higher than cat/car ({:.4})",
            sim_cat_dog,
            sim_cat_car
        );

        eprintln!(
            "Semantic similarity: cat/dog={:.4}, cat/car={:.4}",
            sim_cat_dog, sim_cat_car
        );
    }

    #[test]
    fn test_embed_batch() {
        if !is_available() {
            eprintln!("Skipping test: fastembed not available");
            return;
        }

        let texts = ["hello", "world", "test"];
        let embeddings = embed_batch(&texts).expect("batch embed should work");
        assert_eq!(embeddings.len(), 3, "Should have 3 embeddings");
        for (i, emb) in embeddings.iter().enumerate() {
            assert_eq!(
                emb.len(),
                384,
                "Embedding {} should be 384-dim, got {}",
                i,
                emb.len()
            );
        }
    }
}
