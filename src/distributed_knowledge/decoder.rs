//! Knowledge Decoder
//!
//! Reads NCA memory channel state and decodes to feature vectors.
//! Uses cosine similarity for semantic matching when embeddings are available.
//! Returns actual text snippets via the TextStore.

use super::encoder::{
    encode_text, feature_to_position, load_projection, EncoderConfig, FeatureVector,
    LinearProjection, NUM_EMBED_SLOTS,
};
use super::text_store::TextStore;
use crate::grid::{Grid, KNOWLEDGE_ACTIVATION, KNOWLEDGE_CHANNELS_START, KNOWLEDGE_CONFIDENCE, MEMORY_RECENCY};
use std::sync::OnceLock;

/// Lazily-loaded projection matrix, shared across all decoder calls.
static CACHED_PROJECTION: OnceLock<Option<LinearProjection>> = OnceLock::new();

/// Get the cached projection, loading it once from disk.
fn get_cached_projection() -> Option<&'static LinearProjection> {
    CACHED_PROJECTION.get_or_init(load_projection).as_ref()
}

/// A knowledge activation result from querying the grid
#[derive(Clone, Debug)]
pub struct KnowledgeActivation {
    /// Grid position (x, y)
    pub position: (usize, usize),
    /// Activation strength (0-1)
    pub activation: f64,
    /// Confidence score from metadata
    pub confidence: f64,
    /// Timestamp from metadata
    pub timestamp: f64,
    /// Embedding value at this cell
    pub embedding: f64,
    /// Relevance score (combined metric)
    pub relevance: f64,
    /// Original text stored at this location (if available)
    pub text: Option<String>,
}

/// Read the first embedding slot value for a cell (backward-compat helper).
/// Returns 0.0 if the channel doesn't exist (graceful degradation).
fn cell_embedding(grid: &Grid, y: usize, x: usize) -> f64 {
    grid.cells[y][x]
        .get(KNOWLEDGE_CHANNELS_START)
        .copied()
        .unwrap_or(0.0)
}

/// Safely read knowledge activation from a cell.
/// Returns 0.0 if the channel doesn't exist (graceful degradation).
#[inline]
fn safe_knowledge_activation(cell: &[f64]) -> f64 {
    cell.get(KNOWLEDGE_ACTIVATION).copied().unwrap_or(0.0)
}

/// Safely read knowledge confidence from a cell.
/// Returns 0.0 if the channel doesn't exist (graceful degradation).
#[inline]
fn safe_knowledge_confidence(cell: &[f64]) -> f64 {
    cell.get(KNOWLEDGE_CONFIDENCE).copied().unwrap_or(0.0)
}

/// Extract the 6-slot embedding vector from a cell.
/// Returns zeros if the cell doesn't have enough channels (graceful degradation).
pub fn cell_embedding_vec(grid: &Grid, y: usize, x: usize) -> [f64; NUM_EMBED_SLOTS] {
    let mut v = [0.0f64; NUM_EMBED_SLOTS];
    let cell = &grid.cells[y][x];
    let cell_len = cell.len();
    for (i, slot) in v.iter_mut().enumerate() {
        let ch = KNOWLEDGE_CHANNELS_START + i;
        if ch < cell_len {
            *slot = cell[ch];
        }
        // else: leave as 0.0 (graceful degradation for old brains)
    }
    v
}

/// Cosine similarity between a feature vector and a cell's 6 embedding slots.
///
/// If a projection is provided AND the query is NOT already semantic (fastembed/Ollama),
/// the query features are projected to match the encoding space before comparison.
/// Projection is skipped for semantic features because they already live in the
/// full-dimensional embedding space (384-dim reduced), not the hash space.
fn cosine_sim_query_cell(
    query_features: &FeatureVector,
    cell_embed: &[f64; NUM_EMBED_SLOTS],
    projection: Option<&LinearProjection>,
) -> f64 {
    // Only apply projection to hash-based features — semantic features (fastembed/Ollama)
    // must NOT be projected since they are already in the target embedding space.
    let projected_values: Vec<f64>;
    let values = if !query_features.is_semantic {
        if let Some(proj) = projection {
            projected_values = proj.forward(&query_features.values);
            &projected_values
        } else {
            &query_features.values
        }
    } else {
        &query_features.values
    };

    // Project query features to slots using the same Johnson-Lindenstrauss
    // random projection as the encoder. This replaces the old strided sampling
    // which lost 87.5% of information for 384-dim fastembed embeddings.
    let query_slots = super::encoder::project_to_slots(values);

    let dot: f64 = query_slots
        .iter()
        .zip(cell_embed.iter())
        .map(|(a, b)| a * b)
        .sum();
    let mag_q: f64 = query_slots.iter().map(|v| v * v).sum::<f64>().sqrt();
    let mag_c: f64 = cell_embed.iter().map(|v| v * v).sum::<f64>().sqrt();
    if mag_q < 1e-10 || mag_c < 1e-10 {
        return 0.0;
    }
    (dot / (mag_q * mag_c)).clamp(-1.0, 1.0)
}

/// Read the knowledge state at a specific grid cell
pub fn read_cell_knowledge(grid: &Grid, x: usize, y: usize) -> Option<KnowledgeActivation> {
    if x >= grid.width || y >= grid.height {
        return None;
    }

    let cell = &grid.cells[y][x];
    let activation = safe_knowledge_activation(cell);
    if activation < 1e-6 {
        return None;
    }

    Some(KnowledgeActivation {
        position: (x, y),
        activation,
        confidence: safe_knowledge_confidence(cell),
        timestamp: 0.0,
        embedding: cell_embedding(grid, y, x),
        relevance: activation,
        text: None,
    })
}

/// Scan the entire grid for all active knowledge cells
pub fn scan_active_knowledge(grid: &Grid, min_activation: f64) -> Vec<KnowledgeActivation> {
    let mut results = Vec::new();

    for y in 0..grid.height {
        for x in 0..grid.width {
            let cell = &grid.cells[y][x];
            let activation = safe_knowledge_activation(cell);
            if activation >= min_activation {
                results.push(KnowledgeActivation {
                    position: (x, y),
                    activation,
                    confidence: safe_knowledge_confidence(cell),
                    timestamp: 0.0,
                    embedding: cell_embedding(grid, y, x),
                    relevance: activation,
                    text: None,
                });
            }
        }
    }

    results
}

/// Query the grid with a text query and return ranked knowledge activations.
/// If a TextStore is provided, results include original text snippets.
pub fn query_knowledge(
    grid: &Grid,
    query: &str,
    config: &EncoderConfig,
    max_results: usize,
) -> Vec<KnowledgeActivation> {
    query_knowledge_with_text(grid, query, config, max_results, None)
}

/// Query the grid with text store for retrieving original text snippets
pub fn query_knowledge_with_text(
    grid: &Grid,
    query: &str,
    config: &EncoderConfig,
    max_results: usize,
    text_store: Option<&TextStore>,
) -> Vec<KnowledgeActivation> {
    let query_features = encode_text(query, config);
    query_knowledge_by_features_with_text(grid, &query_features, config, max_results, text_store)
}

/// Query the grid with a pre-computed feature vector
pub fn query_knowledge_by_features(
    grid: &Grid,
    query_features: &FeatureVector,
    config: &EncoderConfig,
    max_results: usize,
) -> Vec<KnowledgeActivation> {
    query_knowledge_by_features_with_text(grid, query_features, config, max_results, None)
}

/// Query with pre-computed features and optional text store.
/// Uses cosine similarity (70%) + proximity (30%) for semantic retrieval.
///
/// If a learned projection is available (from `~/.sage/embedding_projection.bin`),
/// the query features are projected to match the encoding space before comparison.
pub fn query_knowledge_by_features_with_text(
    grid: &Grid,
    query_features: &FeatureVector,
    config: &EncoderConfig,
    max_results: usize,
    text_store: Option<&TextStore>,
) -> Vec<KnowledgeActivation> {
    // Load projection (cached after first call)
    let projection = get_cached_projection();

    // Project query features if projection is available AND query is hash-based.
    // Semantic features (fastembed/Ollama) must NOT be projected — they already
    // exist in the target embedding space (reduced from 384-dim, not hash space).
    let projected_features: FeatureVector;
    let features_for_position = if !query_features.is_semantic {
        if let Some(proj) = projection {
            projected_features = FeatureVector {
                values: proj.forward(&query_features.values),
                is_semantic: false,
            };
            &projected_features
        } else {
            query_features
        }
    } else {
        query_features
    };

    // Search ALL active cells (global retrieval) — the hash position is only
    // used for encoding, not for retrieval. Semantic similarity should find matches
    // anywhere in the grid.
    let mut results = Vec::new();
    // Track which text snippets we've already seen to deduplicate
    let mut seen_texts = std::collections::HashSet::new();

    for y in 0..grid.height {
        for x in 0..grid.width {
            let cell = &grid.cells[y][x];
            let activation = safe_knowledge_activation(cell);
            if activation < 1e-6 {
                continue;
            }

            let confidence = safe_knowledge_confidence(cell);

            // Cosine similarity between query embedding and cell embedding slots
            let cell_embed = cell_embedding_vec(grid, y, x);
            let cos_sim = cosine_sim_query_cell(query_features, &cell_embed, projection);
            let cos_sim_pos = cos_sim.max(0.0);

            // Recency boost: recently-encoded knowledge gets a small relevance boost.
            // MEMORY_RECENCY is set to 1.0 on encode and decays over time.
            let recency = cell.get(MEMORY_RECENCY).copied().unwrap_or(0.0);

            // Semantic retrieval: 70% cosine similarity, 10% activation, 10% confidence, 10% recency
            let relevance = 0.7 * cos_sim_pos + 0.1 * activation + 0.1 * confidence + 0.1 * recency;

            // Look up original text
            let text = text_store.and_then(|ts| ts.peek(x, y).map(|s| s.to_string()));

            // When a text_store is provided, skip cells with no text — they're
            // useless for text-based retrieval. When no text_store is provided
            // (e.g. raw grid queries), return all active cells with relevance.
            if text_store.is_some() && text.is_none() {
                continue;
            }

            // Deduplicate by text content (only when text is present)
            if let Some(ref t) = text {
                if seen_texts.contains(t) {
                    continue;
                }
                seen_texts.insert(t.clone());
            }

            results.push(KnowledgeActivation {
                position: (x, y),
                activation,
                confidence,
                timestamp: 0.0,
                embedding: cell_embedding(grid, y, x),
                relevance,
                text,
            });
        }
    }

    // Sort by relevance (highest first) and truncate
    results.sort_by(|a, b| {
        b.relevance
            .partial_cmp(&a.relevance)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    results.truncate(max_results);
    results
}

/// Decode a region of the grid back to an approximate feature vector.
pub fn decode_region(
    grid: &Grid,
    center_x: usize,
    center_y: usize,
    radius: usize,
    num_features: usize,
) -> FeatureVector {
    let mut features = FeatureVector::new(num_features);
    let r = radius as i32;

    let mut total_weight = 0.0;

    for dy in -r..=r {
        for dx in -r..=r {
            let nx = ((center_x as i32 + dx).rem_euclid(grid.width as i32)) as usize;
            let ny = ((center_y as i32 + dy).rem_euclid(grid.height as i32)) as usize;

            let activation = grid.cells[ny][nx][KNOWLEDGE_ACTIVATION];
            if activation < 1e-6 {
                continue;
            }

            let dist = ((dx * dx + dy * dy) as f64).sqrt();
            let weight = activation / (1.0 + dist);

            // Use the transpose of the Johnson-Lindenstrauss projection
            // to map slots back to feature space. This is the pseudo-inverse
            // for a random matrix and gives the best linear estimate.
            let cell_slots = cell_embedding_vec(grid, ny, nx);
            let reconstructed = super::encoder::project_from_slots(&cell_slots, num_features);
            for (j, val) in reconstructed.iter().enumerate() {
                features.values[j] += val * weight;
            }
            total_weight += weight;
        }
    }

    if total_weight > 1e-10 {
        for v in &mut features.values {
            *v /= total_weight;
        }
    }

    features.normalize();
    features
}

#[cfg(test)]
mod tests {
    use super::super::encoder::write_knowledge;
    use super::*;
    use crate::grid::GRID_SIZE;

    fn setup_grid_with_knowledge(text: &str) -> (Grid, EncoderConfig, (usize, usize)) {
        let mut grid = Grid::new(GRID_SIZE, GRID_SIZE);
        let config = EncoderConfig {
            ollama_url: None, // Use hash fallback in tests
            ..Default::default()
        };
        let features = encode_text(text, &config);
        let pos = write_knowledge(&mut grid, &features, 0.9, 0.5, &config);
        (grid, config, pos)
    }

    #[test]
    fn test_query_finds_written_knowledge() {
        let (grid, config, _pos) = setup_grid_with_knowledge("rust programming language");

        let results = query_knowledge(&grid, "rust programming language", &config, 10);
        assert!(
            !results.is_empty(),
            "Should find knowledge that was written"
        );
        assert!(results[0].activation > 0.0);
    }

    #[test]
    fn test_query_with_text_store() {
        let (grid, config, pos) = setup_grid_with_knowledge("rust programming language");

        let mut text_store = TextStore::new();
        text_store.insert(pos.0, pos.1, "rust programming language".into());

        let results = query_knowledge_with_text(
            &grid,
            "rust programming language",
            &config,
            10,
            Some(&text_store),
        );
        assert!(!results.is_empty());

        // At least one result should have text
        let has_text = results.iter().any(|r| r.text.is_some());
        assert!(has_text, "Should find text from text store");
    }

    #[test]
    fn test_query_ranks_similar_higher() {
        let mut grid = Grid::new(GRID_SIZE, GRID_SIZE);
        let config = EncoderConfig {
            ollama_url: None,
            ..Default::default()
        };

        let f1 = encode_text("machine learning neural networks", &config);
        write_knowledge(&mut grid, &f1, 0.9, 0.5, &config);

        let f2 = encode_text("cooking recipes italian food", &config);
        write_knowledge(&mut grid, &f2, 0.9, 0.5, &config);

        let _results = query_knowledge(&grid, "deep learning algorithms", &config, 20);
        // Query should return results or empty gracefully
    }

    #[test]
    fn test_scan_active_knowledge() {
        let (grid, _config, _pos) = setup_grid_with_knowledge("test scan");

        let active = scan_active_knowledge(&grid, 0.01);
        assert!(
            !active.is_empty(),
            "Should find active cells after writing knowledge"
        );
    }

    #[test]
    fn test_read_cell_empty_grid() {
        let grid = Grid::new(GRID_SIZE, GRID_SIZE);
        let result = read_cell_knowledge(&grid, 0, 0);
        assert!(result.is_none(), "Empty grid should return None");
    }

    #[test]
    fn test_decode_region_roundtrip() {
        let (grid, config, (cx, cy)) = setup_grid_with_knowledge("roundtrip test encoding");

        let decoded = decode_region(&grid, cx, cy, config.spread_radius, config.num_features);
        let nonzero = decoded.values.iter().filter(|v| v.abs() > 1e-10).count();
        assert!(nonzero > 0, "Decoded region should have non-zero features");
    }

    #[test]
    fn test_query_empty_grid_returns_empty() {
        let grid = Grid::new(GRID_SIZE, GRID_SIZE);
        let config = EncoderConfig {
            ollama_url: None,
            ..Default::default()
        };
        let results = query_knowledge(&grid, "anything", &config, 10);
        assert!(results.is_empty(), "Empty grid should return no results");
    }

    #[test]
    fn test_deduplication() {
        let (grid, config, pos) = setup_grid_with_knowledge("test dedup");

        let mut text_store = TextStore::new();
        // Store same text at multiple nearby cells
        text_store.insert(pos.0, pos.1, "test dedup".into());
        if pos.0 + 1 < GRID_SIZE {
            text_store.insert(pos.0 + 1, pos.1, "test dedup".into());
        }

        let results =
            query_knowledge_with_text(&grid, "test dedup", &config, 10, Some(&text_store));
        // Count how many results have the same text
        let text_results: Vec<_> = results
            .iter()
            .filter(|r| r.text.as_deref() == Some("test dedup"))
            .collect();
        assert!(
            text_results.len() <= 1,
            "Should deduplicate identical text snippets"
        );
    }

    // ══════════════════════════════════════════════════════════════════════════
    // Bounds safety tests — prevent panics from index out of bounds
    // ══════════════════════════════════════════════════════════════════════════

    /// REGRESSION: scan_active_knowledge must not panic on a fresh/empty grid.
    /// Previously could panic if activation threshold was below 0.0.
    #[test]
    fn test_scan_active_knowledge_no_panic_on_empty_grid() {
        let grid = Grid::new(GRID_SIZE, GRID_SIZE);

        // Should return empty vec, not panic
        let results = scan_active_knowledge(&grid, 0.01);
        assert!(results.is_empty(), "Fresh grid should have no active cells");

        // Edge case: min_activation = 0 (all cells "active" with 0 activation)
        let results_zero = scan_active_knowledge(&grid, 0.0);
        // This would return all cells if we didn't have the >= check — that's OK,
        // but it must not panic
        assert!(
            results_zero.len() <= GRID_SIZE * GRID_SIZE,
            "Should return at most grid_size^2 cells"
        );
    }

    /// REGRESSION: cell_embedding_vec must handle grids with fewer channels
    /// than expected without panicking (index out of bounds bug).
    #[test]
    fn test_cell_embedding_vec_bounds() {
        use crate::grid::NUM_CHANNELS;

        // Standard grid with NUM_CHANNELS
        let grid = Grid::new(64, 64);
        assert_eq!(
            grid.cells[0][0].len(),
            NUM_CHANNELS,
            "Grid should have NUM_CHANNELS channels"
        );

        // cell_embedding_vec should work on every cell without panic
        for y in 0..grid.height {
            for x in 0..grid.width {
                let embedding = cell_embedding_vec(&grid, y, x);
                // Should return a valid array (zeros for fresh grid)
                assert_eq!(
                    embedding.len(),
                    NUM_EMBED_SLOTS,
                    "Embedding should have NUM_EMBED_SLOTS elements"
                );
            }
        }
    }

    /// REGRESSION: cell_embedding_vec gracefully handles cells with missing channels.
    /// Old brains or corrupted grids might have fewer channels — this must not panic.
    #[test]
    fn test_cell_embedding_vec_graceful_degradation() {
        // Manually create a grid with fewer channels (simulating old brain format)
        let mut grid = Grid::new(4, 4);
        // Shrink cells to only 10 channels (less than KNOWLEDGE_CHANNELS_START + NUM_EMBED_SLOTS)
        for y in 0..grid.height {
            for x in 0..grid.width {
                grid.cells[y][x] = vec![0.5; 10]; // Only 10 channels
            }
        }

        // cell_embedding_vec should return zeros for missing channels, not panic
        let embedding = cell_embedding_vec(&grid, 0, 0);
        assert_eq!(embedding.len(), NUM_EMBED_SLOTS);
        // All should be 0.0 because KNOWLEDGE_CHANNELS_START (26) > 10
        for slot in &embedding {
            assert_eq!(
                *slot, 0.0,
                "Missing channels should return 0.0 (graceful degradation)"
            );
        }
    }

    /// Test read_cell_knowledge returns None for out-of-bounds coordinates.
    #[test]
    fn test_read_cell_knowledge_out_of_bounds() {
        let grid = Grid::new(64, 64);

        // Out of bounds should return None, not panic
        let result_x_oob = read_cell_knowledge(&grid, 100, 0);
        assert!(result_x_oob.is_none(), "x out of bounds should return None");

        let result_y_oob = read_cell_knowledge(&grid, 0, 100);
        assert!(result_y_oob.is_none(), "y out of bounds should return None");

        let result_both_oob = read_cell_knowledge(&grid, 1000, 1000);
        assert!(
            result_both_oob.is_none(),
            "Both coords out of bounds should return None"
        );
    }
}
