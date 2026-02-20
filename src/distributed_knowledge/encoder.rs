//! Knowledge Encoder
//!
//! Encodes text/semantic input into NCA memory channel patterns.
//! Uses Ollama embeddings for semantic encoding when available,
//! falling back to hashed n-grams.
//! Spatial locality: related knowledge clusters in nearby cells.

use crate::grid::{
    Grid, KNOWLEDGE_ACTIVATION, KNOWLEDGE_EMBEDDING, META_CONFIDENCE, META_TIMESTAMP,
};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Configuration for the knowledge encoder
#[derive(Clone, Debug)]
pub struct EncoderConfig {
    /// Number of hash features (dimensionality of embedding)
    pub num_features: usize,
    /// N-gram sizes to use (e.g., [1, 2, 3] for unigrams, bigrams, trigrams)
    pub ngram_sizes: Vec<usize>,
    /// Radius of spatial spread around the primary cell
    pub spread_radius: usize,
    /// Activation decay per cell distance from center
    pub spatial_decay: f64,
    /// Number of secondary hash positions for better distribution
    pub num_hash_positions: usize,
    /// Ollama base URL for embeddings
    pub ollama_url: Option<String>,
    /// Ollama model for embeddings
    pub embedding_model: String,
}

impl Default for EncoderConfig {
    fn default() -> Self {
        Self {
            num_features: 64,
            ngram_sizes: vec![1, 2, 3],
            spread_radius: 6,
            spatial_decay: 0.4,
            num_hash_positions: 1,
            ollama_url: Some("http://localhost:11434".into()),
            embedding_model: "nomic-embed-text".into(),
        }
    }
}

/// A feature vector produced by encoding text
#[derive(Clone, Debug)]
pub struct FeatureVector {
    pub values: Vec<f64>,
    /// Whether this was produced by semantic (Ollama) embedding
    pub is_semantic: bool,
}

impl FeatureVector {
    pub fn new(size: usize) -> Self {
        Self {
            values: vec![0.0; size],
            is_semantic: false,
        }
    }

    /// Cosine similarity with another feature vector
    pub fn cosine_similarity(&self, other: &FeatureVector) -> f64 {
        if self.values.len() != other.values.len() {
            return 0.0;
        }
        let dot: f64 = self
            .values
            .iter()
            .zip(&other.values)
            .map(|(a, b)| a * b)
            .sum();
        let mag_a: f64 = self.values.iter().map(|v| v * v).sum::<f64>().sqrt();
        let mag_b: f64 = other.values.iter().map(|v| v * v).sum::<f64>().sqrt();
        if mag_a < 1e-10 || mag_b < 1e-10 {
            return 0.0;
        }
        dot / (mag_a * mag_b)
    }

    /// L2 normalize in-place
    pub fn normalize(&mut self) {
        let mag: f64 = self.values.iter().map(|v| v * v).sum::<f64>().sqrt();
        if mag > 1e-10 {
            for v in &mut self.values {
                *v /= mag;
            }
        }
    }
}

/// Hash a string to a u64
fn hash_str(s: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    s.hash(&mut hasher);
    hasher.finish()
}

/// Extract character n-grams from text
fn extract_ngrams(text: &str, n: usize) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    if chars.len() < n {
        return vec![text.to_string()];
    }
    chars.windows(n).map(|w| w.iter().collect()).collect()
}

/// Get semantic embedding from Ollama's /api/embeddings endpoint.
/// Returns None if Ollama is unavailable or errors.
pub fn get_ollama_embedding(text: &str, config: &EncoderConfig) -> Option<Vec<f64>> {
    let url = config.ollama_url.as_ref()?;
    let endpoint = format!("{}/api/embeddings", url);

    let body = serde_json::json!({
        "model": config.embedding_model,
        "prompt": text,
    });

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .ok()?;

    let resp = client.post(&endpoint).json(&body).send().ok()?;

    if !resp.status().is_success() {
        return None;
    }

    let json: serde_json::Value = resp.json().ok()?;
    let embedding = json
        .get("embedding")?
        .as_array()?
        .iter()
        .filter_map(|v| v.as_f64())
        .collect::<Vec<f64>>();

    if embedding.is_empty() {
        None
    } else {
        Some(embedding)
    }
}

/// Reduce a high-dimensional embedding to the target size by averaging chunks
pub fn reduce_embedding(full: &[f64], target_size: usize) -> Vec<f64> {
    if full.len() <= target_size {
        let mut result = full.to_vec();
        result.resize(target_size, 0.0);
        return result;
    }
    let chunk_size = full.len() / target_size;
    let mut reduced = Vec::with_capacity(target_size);
    for i in 0..target_size {
        let start = i * chunk_size;
        let end = if i == target_size - 1 {
            full.len()
        } else {
            start + chunk_size
        };
        let avg: f64 = full[start..end].iter().sum::<f64>() / (end - start) as f64;
        reduced.push(avg);
    }
    reduced
}

/// Encode text into a feature vector.
/// Tries Ollama semantic embedding first, falls back to hash-based encoding.
pub fn encode_text(text: &str, config: &EncoderConfig) -> FeatureVector {
    // Try semantic embedding first
    if let Some(embedding) = get_ollama_embedding(text, config) {
        let reduced = reduce_embedding(&embedding, config.num_features);
        let mut features = FeatureVector {
            values: reduced,
            is_semantic: true,
        };
        features.normalize();
        return features;
    }

    // Fallback: hash-based encoding
    encode_text_hash(text, config)
}

/// Hash-based text encoding (fallback when Ollama is unavailable)
pub fn encode_text_hash(text: &str, config: &EncoderConfig) -> FeatureVector {
    let mut features = FeatureVector::new(config.num_features);
    let normalized = text.to_lowercase();

    let words: Vec<&str> = normalized.split_whitespace().collect();

    for &n in &config.ngram_sizes {
        let char_ngrams = extract_ngrams(&normalized, n);
        for ngram in &char_ngrams {
            let h = hash_str(ngram);
            let idx = (h % config.num_features as u64) as usize;
            let sign = if (h >> 32) & 1 == 0 { 1.0 } else { -1.0 };
            features.values[idx] += sign;
        }

        if words.len() >= n {
            for window in words.windows(n) {
                let ngram = window.join(" ");
                let h = hash_str(&ngram);
                let idx = (h % config.num_features as u64) as usize;
                let sign = if (h >> 32) & 1 == 0 { 1.0 } else { -1.0 };
                features.values[idx] += sign * 2.0;
            }
        }
    }

    features.normalize();
    features
}

/// Map a feature vector to a primary grid position using spatial hashing.
/// Uses multiple feature dimensions for better distribution across large grids.
pub fn feature_to_position(
    features: &FeatureVector,
    grid_width: usize,
    grid_height: usize,
) -> (usize, usize) {
    if features.values.is_empty() {
        return (grid_width / 2, grid_height / 2);
    }

    // Use more feature dimensions for position hashing to get better spread
    // Combine multiple dimensions to reduce clustering
    let n = features.values.len();
    let fx = if n >= 4 {
        // Mix multiple dimensions for x: use dims 0, 2, 4...
        let mix = features.values[0] * 0.5
            + features.values[2.min(n - 1)] * 0.3
            + features.values[4.min(n - 1)] * 0.2;
        (mix + 1.0) / 2.0
    } else {
        (features.values[0] + 1.0) / 2.0
    };
    let fy = if n >= 4 {
        let mix = features.values[1.min(n - 1)] * 0.5
            + features.values[3.min(n - 1)] * 0.3
            + features.values[5.min(n - 1)] * 0.2;
        (mix + 1.0) / 2.0
    } else if n > 1 {
        (features.values[1] + 1.0) / 2.0
    } else {
        0.5
    };

    let x = ((fx.clamp(0.0, 1.0) * grid_width as f64) as usize).min(grid_width - 1);
    let y = ((fy.clamp(0.0, 1.0) * grid_height as f64) as usize).min(grid_height - 1);
    (x, y)
}

/// Write encoded knowledge into the NCA grid's knowledge channels
pub fn write_knowledge(
    grid: &mut Grid,
    features: &FeatureVector,
    confidence: f64,
    timestamp: f64,
    config: &EncoderConfig,
) -> (usize, usize) {
    let (cx, cy) = feature_to_position(features, grid.width, grid.height);
    let radius = config.spread_radius as i32;

    for dy in -radius..=radius {
        for dx in -radius..=radius {
            let nx = ((cx as i32 + dx).rem_euclid(grid.width as i32)) as usize;
            let ny = ((cy as i32 + dy).rem_euclid(grid.height as i32)) as usize;

            let dist = ((dx * dx + dy * dy) as f64).sqrt();
            if dist > config.spread_radius as f64 {
                continue;
            }

            let decay = config.spatial_decay.powf(dist);

            let offset =
                (dy + radius) as usize * (2 * radius as usize + 1) + (dx + radius) as usize;
            let feat_idx = offset % features.values.len().max(1);
            let embedding_val = features.values[feat_idx];

            let existing_embed = grid.cells[ny][nx][KNOWLEDGE_EMBEDDING];
            let existing_act = grid.cells[ny][nx][KNOWLEDGE_ACTIVATION];

            grid.cells[ny][nx][KNOWLEDGE_EMBEDDING] = (existing_embed * (1.0 - decay * 0.5)
                + embedding_val * decay * 0.5)
                .clamp(-1.0, 1.0);
            grid.cells[ny][nx][KNOWLEDGE_ACTIVATION] =
                (existing_act + decay * confidence).clamp(0.0, 1.0);

            if decay > 0.3 {
                grid.cells[ny][nx][META_TIMESTAMP] = timestamp;
                grid.cells[ny][nx][META_CONFIDENCE] =
                    grid.cells[ny][nx][META_CONFIDENCE].max(confidence * decay);
            }
        }
    }

    (cx, cy)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::distributed_knowledge::GRID_SIZE;

    #[test]
    fn test_encode_text_produces_nonzero() {
        let mut config = EncoderConfig::default();
        config.ollama_url = None; // Force hash fallback for tests
        let features = encode_text("hello world", &config);
        let nonzero = features.values.iter().filter(|v| v.abs() > 1e-10).count();
        assert!(nonzero > 0, "Encoding should produce non-zero features");
        assert!(!features.is_semantic);
    }

    #[test]
    fn test_similar_texts_similar_embeddings() {
        let mut config = EncoderConfig::default();
        config.ollama_url = None;
        let f1 = encode_text("the cat sat on the mat", &config);
        let f2 = encode_text("the cat sat on a mat", &config);
        let f3 = encode_text("quantum physics equations", &config);

        let sim_close = f1.cosine_similarity(&f2);
        let sim_far = f1.cosine_similarity(&f3);

        assert!(
            sim_close > sim_far,
            "Similar texts should have higher similarity: close={} far={}",
            sim_close,
            sim_far
        );
    }

    #[test]
    fn test_encoding_is_normalized() {
        let mut config = EncoderConfig::default();
        config.ollama_url = None;
        let features = encode_text("test normalization", &config);
        let mag: f64 = features.values.iter().map(|v| v * v).sum::<f64>().sqrt();
        assert!(
            (mag - 1.0).abs() < 0.01,
            "Feature vector should be L2-normalized, got mag={}",
            mag
        );
    }

    #[test]
    fn test_write_knowledge_to_grid() {
        let mut grid = Grid::new(GRID_SIZE, GRID_SIZE);
        let mut config = EncoderConfig::default();
        config.ollama_url = None;
        let features = encode_text("test knowledge", &config);

        let (cx, cy) = write_knowledge(&mut grid, &features, 0.9, 0.5, &config);

        assert!(cx < GRID_SIZE);
        assert!(cy < GRID_SIZE);
        assert!(
            grid.cells[cy][cx][KNOWLEDGE_ACTIVATION] > 0.0,
            "Center cell should have activation"
        );
    }

    #[test]
    fn test_spatial_locality() {
        let mut grid = Grid::new(GRID_SIZE, GRID_SIZE);
        let mut config = EncoderConfig::default();
        config.ollama_url = None;
        let features = encode_text("spatial test", &config);

        let (cx, cy) = write_knowledge(&mut grid, &features, 0.9, 0.5, &config);

        let center_act = grid.cells[cy][cx][KNOWLEDGE_ACTIVATION];
        let edge_x = (cx + config.spread_radius).min(GRID_SIZE - 1);
        let edge_act = grid.cells[cy][edge_x][KNOWLEDGE_ACTIVATION];

        assert!(
            center_act >= edge_act,
            "Center activation ({}) should be >= edge activation ({})",
            center_act,
            edge_act
        );
    }

    #[test]
    fn test_feature_to_position_in_bounds() {
        let mut config = EncoderConfig::default();
        config.ollama_url = None;
        for text in &["hello", "world", "test", "another one", ""] {
            let features = encode_text(text, &config);
            let (x, y) = feature_to_position(&features, GRID_SIZE, GRID_SIZE);
            assert!(x < GRID_SIZE, "x={} out of bounds", x);
            assert!(y < GRID_SIZE, "y={} out of bounds", y);
        }
    }

    #[test]
    fn test_reduce_embedding() {
        let full = vec![1.0; 768];
        let reduced = reduce_embedding(&full, 64);
        assert_eq!(reduced.len(), 64);
        assert!((reduced[0] - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_reduce_embedding_small() {
        let small = vec![0.5, 0.3];
        let reduced = reduce_embedding(&small, 64);
        assert_eq!(reduced.len(), 64);
        assert_eq!(reduced[0], 0.5);
        assert_eq!(reduced[1], 0.3);
        assert_eq!(reduced[2], 0.0);
    }

    #[test]
    fn test_hash_fallback_when_no_ollama() {
        let mut config = EncoderConfig::default();
        config.ollama_url = None;
        let features = encode_text("test fallback", &config);
        assert!(!features.is_semantic);
        assert_eq!(features.values.len(), config.num_features);
    }
}
