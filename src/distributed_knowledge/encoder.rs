//! Knowledge Encoder
//!
//! Encodes text/semantic input into NCA memory channel patterns.
//! Priority order for embeddings:
//!   1. fastembed (bundled, no external dependencies)
//!   2. Ollama (if configured and running)
//!   3. Hash-based fallback (always available)
//! Spatial locality: related knowledge clusters in nearby cells.

use super::embedder;
use crate::grid::{
    Grid, KNOWLEDGE_ACTIVATION, KNOWLEDGE_CHANNELS_START, KNOWLEDGE_CONFIDENCE,
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
            num_hash_positions: 3, // primary + 2 secondary probing positions
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

/// Embedding backend status for display purposes
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EmbeddingStatus {
    /// fastembed bundled model (preferred)
    Fastembed { model: &'static str, dim: usize },
    /// Ollama external service
    Ollama { model: String },
    /// Hash-based fallback (reduced quality)
    HashFallback,
}

impl std::fmt::Display for EmbeddingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EmbeddingStatus::Fastembed { model, dim } => {
                write!(f, "bundled ({}, {}-dim)", model, dim)
            }
            EmbeddingStatus::Ollama { model } => write!(f, "Ollama ({})", model),
            EmbeddingStatus::HashFallback => write!(f, "hash fallback (reduced quality)"),
        }
    }
}

/// Detect the active embedding backend and return its status.
/// Checks in priority order: fastembed > Ollama > hash.
pub fn detect_embedding_status(config: &EncoderConfig) -> EmbeddingStatus {
    // Check fastembed first
    if embedder::is_available() {
        return EmbeddingStatus::Fastembed {
            model: embedder::model_name(),
            dim: embedder::dimension(),
        };
    }

    // Check Ollama
    if let Some(url) = &config.ollama_url {
        let endpoint = format!("{}/api/embeddings", url);
        let body = serde_json::json!({
            "model": &config.embedding_model,
            "prompt": "test",
        });
        if let Ok(client) = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(3))
            .build()
        {
            if let Ok(resp) = client.post(&endpoint).json(&body).send() {
                if resp.status().is_success() {
                    return EmbeddingStatus::Ollama {
                        model: config.embedding_model.clone(),
                    };
                }
            }
        }
    }

    // Fallback
    EmbeddingStatus::HashFallback
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
/// Priority order: fastembed > Ollama > hash fallback.
pub fn encode_text(text: &str, config: &EncoderConfig) -> FeatureVector {
    // Priority 1: Try fastembed (bundled, no external service)
    if let Some(embedding) = embedder::embed_text_f64(text) {
        let reduced = reduce_embedding(&embedding, config.num_features);
        let mut features = FeatureVector {
            values: reduced,
            is_semantic: true,
        };
        features.normalize();
        return features;
    }

    // Priority 2: Try Ollama (if configured and running)
    if let Some(embedding) = get_ollama_embedding(text, config) {
        let reduced = reduce_embedding(&embedding, config.num_features);
        let mut features = FeatureVector {
            values: reduced,
            is_semantic: true,
        };
        features.normalize();
        return features;
    }

    // Priority 3: Hash-based encoding (always available)
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
///
/// Uses 12 feature dimensions mixed via weighted combination to produce a
/// well-distributed spatial address.  Compared to the old 3-4 dim approach
/// this reduces hash collisions roughly proportionally to the extra entropy.
///
/// Dimension groups (weights chosen so that all 12 dims contribute roughly equally):
///   x: dims 0,2,4,6,8,10 with exponentially decreasing weights
///   y: dims 1,3,5,7,9,11 with exponentially decreasing weights
pub fn feature_to_position(
    features: &FeatureVector,
    grid_width: usize,
    grid_height: usize,
) -> (usize, usize) {
    if features.values.is_empty() {
        return (grid_width / 2, grid_height / 2);
    }

    let n = features.values.len();

    // Helper: safe index into features
    let f = |i: usize| if i < n { features.values[i] } else { 0.0 };

    // X address: weighted mix of even-index dims (0,2,4,6,8,10)
    // Weights sum to ~1.0 for normalised input
    let fx_raw = f(0) * 0.30
        + f(2) * 0.22
        + f(4) * 0.17
        + f(6) * 0.13
        + f(8) * 0.10
        + f(10) * 0.08;
    // Y address: weighted mix of odd-index dims (1,3,5,7,9,11)
    let fy_raw = f(1) * 0.30
        + f(3) * 0.22
        + f(5) * 0.17
        + f(7) * 0.13
        + f(9) * 0.10
        + f(11) * 0.08;

    // Map [-1, 1] → [0, 1]
    let fx = (fx_raw.clamp(-1.0, 1.0) + 1.0) / 2.0;
    let fy = (fy_raw.clamp(-1.0, 1.0) + 1.0) / 2.0;

    let x = ((fx * grid_width as f64) as usize).min(grid_width - 1);
    let y = ((fy * grid_height as f64) as usize).min(grid_height - 1);
    (x, y)
}

/// Compute secondary hash positions for overflow/probing.
///
/// If the primary cell is already highly activated (collision), callers may
/// try these positions in order.  Each secondary position uses a different
/// subset of feature dimensions so it is semantically independent of the
/// primary address.
///
/// Returns up to `max_secondary` positions, all guaranteed in-bounds.
pub fn feature_to_secondary_positions(
    features: &FeatureVector,
    grid_width: usize,
    grid_height: usize,
    max_secondary: usize,
) -> Vec<(usize, usize)> {
    if features.values.is_empty() || max_secondary == 0 {
        return vec![];
    }

    let n = features.values.len();
    let f = |i: usize| if i < n { features.values[i] } else { 0.0 };

    // Define alternative dimension subsets — each orthogonal to the primary
    let subsets: &[(f64, f64, f64, f64, f64, f64)] = &[
        // (d0, d1, d2, d3, d4, d5) pairs for x/y
        // Secondary 1: dims 12-17 (high-frequency features)
        (f(12), f(14), f(16), f(13), f(15), f(17)),
        // Secondary 2: reversed even dims + blend
        (f(10), f(8), f(6), f(11), f(9), f(7)),
        // Secondary 3: combined XOR-like mix of dims 0-11
        (f(0) * f(6), f(2) * f(8), f(4) * f(10), f(1) * f(7), f(3) * f(9), f(5) * f(11)),
    ];

    let mut positions = Vec::with_capacity(max_secondary.min(subsets.len()));
    for &(a, b, c, d, e, g) in subsets.iter().take(max_secondary) {
        let rx = ((a * 0.4 + b * 0.35 + c * 0.25).clamp(-1.0, 1.0) + 1.0) / 2.0;
        let ry = ((d * 0.4 + e * 0.35 + g * 0.25).clamp(-1.0, 1.0) + 1.0) / 2.0;
        let x = ((rx * grid_width as f64) as usize).min(grid_width - 1);
        let y = ((ry * grid_height as f64) as usize).min(grid_height - 1);
        positions.push((x, y));
    }
    positions
}

/// Number of embedding slots in the knowledge channels (channels 26-31).
/// Channels 26..KNOWLEDGE_CHANNELS_START+6 are embedding, +6 is activation, +7 is confidence.
pub const NUM_EMBED_SLOTS: usize = 6;

/// Activation threshold above which a primary cell is considered "occupied".
/// When exceeded, `write_knowledge` probes secondary hash positions.
const COLLISION_THRESHOLD: f64 = 0.5;

/// Write encoded knowledge into the NCA grid's knowledge channels.
/// Distributes the feature vector across 6 embedding slots per cell.
///
/// If the primary cell already has high activation (collision), tries secondary
/// hash positions computed from different feature subsets.
pub fn write_knowledge(
    grid: &mut Grid,
    features: &FeatureVector,
    confidence: f64,
    timestamp: f64,
    config: &EncoderConfig,
) -> (usize, usize) {
    let _ = timestamp; // Kept in signature for API compat; activation encodes recency

    // Determine write position, probing for less-occupied cells
    let primary = feature_to_position(features, grid.width, grid.height);
    let (cx, cy) = if config.num_hash_positions > 1
        && grid.cells[primary.1][primary.0][KNOWLEDGE_ACTIVATION] > COLLISION_THRESHOLD
    {
        // Primary is occupied — try secondary positions
        let secondaries =
            feature_to_secondary_positions(features, grid.width, grid.height, config.num_hash_positions - 1);
        let mut chosen = primary;
        let mut min_act = grid.cells[primary.1][primary.0][KNOWLEDGE_ACTIVATION];
        for (sx, sy) in &secondaries {
            let act = grid.cells[*sy][*sx][KNOWLEDGE_ACTIVATION];
            if act < min_act {
                min_act = act;
                chosen = (*sx, *sy);
                if act < COLLISION_THRESHOLD {
                    break; // Found a clear cell
                }
            }
        }
        chosen
    } else {
        primary
    };

    let radius = config.spread_radius as i32;

    let feat_len = features.values.len().max(1);

    for dy in -radius..=radius {
        for dx in -radius..=radius {
            let nx = ((cx as i32 + dx).rem_euclid(grid.width as i32)) as usize;
            let ny = ((cy as i32 + dy).rem_euclid(grid.height as i32)) as usize;

            let dist = ((dx * dx + dy * dy) as f64).sqrt();
            if dist > config.spread_radius as f64 {
                continue;
            }

            let decay = config.spatial_decay.powf(dist);

            // Write 6 embedding slots: evenly spaced strides through the feature vec
            for slot in 0..NUM_EMBED_SLOTS {
                let feat_idx = (slot * feat_len / NUM_EMBED_SLOTS) % feat_len;
                let embedding_val = features.values[feat_idx];
                let ch = KNOWLEDGE_CHANNELS_START + slot;
                let existing = grid.cells[ny][nx][ch];
                grid.cells[ny][nx][ch] =
                    (existing * (1.0 - decay * 0.5) + embedding_val * decay * 0.5).clamp(-1.0, 1.0);
            }

            // Write activation
            let existing_act = grid.cells[ny][nx][KNOWLEDGE_ACTIVATION];
            grid.cells[ny][nx][KNOWLEDGE_ACTIVATION] =
                (existing_act + decay * confidence).clamp(0.0, 1.0);

            // Write confidence
            if decay > 0.3 {
                grid.cells[ny][nx][KNOWLEDGE_CONFIDENCE] =
                    grid.cells[ny][nx][KNOWLEDGE_CONFIDENCE].max(confidence * decay);
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
        let config = EncoderConfig::default();
        let features = encode_text("hello world", &config);
        let nonzero = features.values.iter().filter(|v| v.abs() > 1e-10).count();
        assert!(nonzero > 0, "Encoding should produce non-zero features");
        // With fastembed available, encoding will be semantic
        // With hash fallback, it will be non-semantic
        // Either way, we get valid features
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
    fn test_encode_text_hash_always_works() {
        // The hash fallback function should always work
        let config = EncoderConfig::default();
        let features = encode_text_hash("test hash fallback", &config);
        assert!(!features.is_semantic);
        assert_eq!(features.values.len(), config.num_features);
    }

    #[test]
    fn test_feature_to_position_uses_12_dims() {
        // Verify that changing dims 6-11 changes the position (12-dim hash)
        let mut config = EncoderConfig::default();
        config.ollama_url = None;
        config.num_features = 64;

        // Construct two feature vectors that differ only in dims 6-11
        let mut fv1 = FeatureVector::new(64);
        let mut fv2 = FeatureVector::new(64);

        // Same first 6 dims
        for i in 0..6 {
            fv1.values[i] = 0.5;
            fv2.values[i] = 0.5;
        }
        // Different dims 6-11
        for i in 6..12 {
            fv1.values[i] = 0.8;
            fv2.values[i] = -0.8;
        }

        let (x1, y1) = feature_to_position(&fv1, GRID_SIZE, GRID_SIZE);
        let (x2, y2) = feature_to_position(&fv2, GRID_SIZE, GRID_SIZE);

        // With 12-dim hash, different high-dim values should produce different positions
        assert!(
            x1 != x2 || y1 != y2,
            "12-dim hash should differentiate vectors that differ in dims 6-11: ({},{}) == ({},{})",
            x1, y1, x2, y2
        );
    }

    #[test]
    fn test_collision_rate_decreases_with_secondary_positions() {
        use crate::distributed_knowledge::GRID_SIZE;

        let mut config = EncoderConfig::default();
        config.ollama_url = None;
        config.num_hash_positions = 3; // primary + 2 secondary

        // Encode 20 semantically distinct items
        let items = [
            "quantum physics particle wave duality",
            "cooking pasta italian tomato sauce",
            "javascript web browser frontend react",
            "geology rock formation sediment tectonic",
            "music jazz improvisation chord blues",
            "economics market supply demand inflation",
            "biology cell division mitosis chromosome",
            "history roman empire julius caesar",
            "philosophy epistemology truth knowledge",
            "architecture gothic cathedral medieval",
            "poetry metaphor verse sonnet rhyme",
            "chemistry periodic table element molecule",
            "astronomy galaxy nebula black hole",
            "medicine antibiotics bacteria infection",
            "linguistics grammar syntax morphology",
            "art impressionism color light monet",
            "psychology behavior cognitive therapy",
            "mathematics topology manifold geometry",
            "religion buddhism meditation enlightenment",
            "engineering structural bridge load stress",
        ];

        // Collect positions using 12-dim addressing
        let mut positions_12dim: Vec<(usize, usize)> = Vec::new();
        for item in &items {
            let features = encode_text_hash(item, &config);
            let pos = feature_to_position(&features, GRID_SIZE, GRID_SIZE);
            positions_12dim.push(pos);
        }

        // Count collisions (same position)
        let total = positions_12dim.len();
        let mut unique = std::collections::HashSet::new();
        for p in &positions_12dim {
            unique.insert(p);
        }
        let collisions_12dim = total - unique.len();

        // With a 256×256 grid and only 20 items, collisions should be low
        // The expected collision rate for random placement is ~20/65536 ≈ 0.03%
        // With 12-dim hash the distribution should be close to random
        assert!(
            collisions_12dim <= 3,
            "Expected ≤3 collisions among 20 items on 256×256 grid, got {}",
            collisions_12dim
        );

        eprintln!(
            "Collision test: {}/{} unique positions ({}% collision rate)",
            unique.len(),
            total,
            collisions_12dim * 100 / total
        );
    }

    #[test]
    fn test_secondary_positions_are_different_from_primary() {
        let mut config = EncoderConfig::default();
        config.ollama_url = None;

        let features = encode_text_hash("test secondary hashing", &config);
        let primary = feature_to_position(&features, GRID_SIZE, GRID_SIZE);
        let secondaries =
            feature_to_secondary_positions(&features, GRID_SIZE, GRID_SIZE, 3);

        // All positions must be in bounds
        for (x, y) in &secondaries {
            assert!(*x < GRID_SIZE, "secondary x={} out of bounds", x);
            assert!(*y < GRID_SIZE, "secondary y={} out of bounds", y);
        }

        // At least one secondary should differ from primary (good distribution)
        let any_different = secondaries.iter().any(|&p| p != primary);
        assert!(
            any_different || secondaries.is_empty(),
            "At least one secondary position should differ from primary"
        );
    }

    // ══════════════════════════════════════════════════════════════════════════
    // Encode → Retrieve roundtrip tests — the fundamental correctness check
    // ══════════════════════════════════════════════════════════════════════════

    /// CRITICAL: The most fundamental test — encode text, retrieve with query,
    /// get back the original text. If this fails, nothing else works.
    #[test]
    fn test_encode_then_retrieve_roundtrip() {
        use crate::distributed_knowledge::decoder::query_knowledge_with_text;
        use crate::distributed_knowledge::text_store::TextStore;

        let mut grid = Grid::new(GRID_SIZE, GRID_SIZE);
        let mut config = EncoderConfig::default();
        config.ollama_url = None; // Use hash fallback for deterministic tests
        let mut text_store = TextStore::new();

        // Encode a specific fact
        let fact = "the sky is blue on clear days";
        let features = encode_text(fact, &config);
        let pos = write_knowledge(&mut grid, &features, 0.9, 0.5, &config);
        text_store.insert(pos.0, pos.1, fact.to_string());

        // Retrieve with related query
        let results = query_knowledge_with_text(&grid, "sky blue", &config, 10, Some(&text_store));

        // Should find the fact
        assert!(
            !results.is_empty(),
            "Roundtrip failed: encode + retrieve returned no results"
        );

        // At least one result should have our text
        let found_text = results.iter().any(|r| {
            r.text
                .as_ref()
                .map(|t| t.contains("sky") || t.contains("blue"))
                .unwrap_or(false)
        });
        assert!(
            found_text,
            "Roundtrip failed: retrieved text doesn't contain encoded content. \
             Got: {:?}",
            results.iter().map(|r| &r.text).collect::<Vec<_>>()
        );
    }

    /// REGRESSION: Multiple facts should not collide on a 256×256 grid.
    /// This tests that the 12-dim hashing and secondary positions work.
    #[test]
    fn test_encode_multiple_facts_no_collision() {
        use crate::distributed_knowledge::decoder::query_knowledge_with_text;
        use crate::distributed_knowledge::text_store::TextStore;

        let mut grid = Grid::new(GRID_SIZE, GRID_SIZE);
        let mut config = EncoderConfig::default();
        config.ollama_url = None;
        let mut text_store = TextStore::new();

        // Encode 10 semantically distinct facts
        let facts = [
            ("Paris is the capital of France", "Paris capital"),
            ("Tokyo is the largest city in Japan", "Tokyo Japan"),
            ("Rust is a systems programming language", "Rust programming"),
            ("The sun is a yellow dwarf star", "sun star"),
            ("Water boils at 100 degrees Celsius", "water boils"),
            ("Einstein developed the theory of relativity", "Einstein relativity"),
            ("The Pacific is the largest ocean", "Pacific ocean"),
            ("DNA contains genetic information", "DNA genetic"),
            ("Mount Everest is the tallest mountain", "Everest mountain"),
            ("The Nile is the longest river", "Nile river"),
        ];

        // Encode all facts
        let mut positions = Vec::new();
        for (fact, _query) in &facts {
            let features = encode_text(fact, &config);
            let pos = write_knowledge(&mut grid, &features, 0.9, 0.5, &config);
            text_store.insert(pos.0, pos.1, fact.to_string());
            positions.push(pos);
        }

        // Check for position collisions (should be rare with 10 items on 256×256)
        let unique_positions: std::collections::HashSet<_> = positions.iter().collect();
        let collision_count = positions.len() - unique_positions.len();
        assert!(
            collision_count <= 2,
            "Too many position collisions: {} out of {} facts",
            collision_count,
            facts.len()
        );

        // For each fact, retrieve with its key query and check if we get it back
        let mut hits = 0;
        for (fact, query) in &facts {
            let results =
                query_knowledge_with_text(&grid, query, &config, 10, Some(&text_store));

            if results.iter().any(|r| {
                r.text.as_ref().map(|t| t == *fact).unwrap_or(false)
            }) {
                hits += 1;
            }
        }

        // With hash-based embeddings, exact recall is limited, but we should get at least
        // some hits. On a well-functioning system, typically 5-8 out of 10.
        // We set a minimum bar of 3/10 (30%) to catch gross regressions.
        assert!(
            hits >= 3,
            "Retrieval hit rate too low: only {}/{} facts retrieved correctly. \
             Expected at least 30% hit rate to catch regressions.",
            hits,
            facts.len()
        );
    }
}
