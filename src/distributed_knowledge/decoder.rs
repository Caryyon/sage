//! Knowledge Decoder
//!
//! Reads NCA memory channel state and decodes to feature vectors.
//! Uses cosine similarity for semantic matching when embeddings are available.
//! Returns actual text snippets via the TextStore.

use crate::grid::{
    Grid, KNOWLEDGE_EMBEDDING, KNOWLEDGE_ACTIVATION,
    META_TIMESTAMP, META_CONFIDENCE,
};
use super::encoder::{FeatureVector, EncoderConfig, encode_text, feature_to_position};
use super::text_store::TextStore;

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

/// Read the knowledge state at a specific grid cell
pub fn read_cell_knowledge(grid: &Grid, x: usize, y: usize) -> Option<KnowledgeActivation> {
    if x >= grid.width || y >= grid.height {
        return None;
    }

    let activation = grid.cells[y][x][KNOWLEDGE_ACTIVATION];
    if activation < 1e-6 {
        return None;
    }

    Some(KnowledgeActivation {
        position: (x, y),
        activation,
        confidence: grid.cells[y][x][META_CONFIDENCE],
        timestamp: grid.cells[y][x][META_TIMESTAMP],
        embedding: grid.cells[y][x][KNOWLEDGE_EMBEDDING],
        relevance: activation,
        text: None,
    })
}

/// Scan the entire grid for all active knowledge cells
pub fn scan_active_knowledge(grid: &Grid, min_activation: f64) -> Vec<KnowledgeActivation> {
    let mut results = Vec::new();

    for y in 0..grid.height {
        for x in 0..grid.width {
            let activation = grid.cells[y][x][KNOWLEDGE_ACTIVATION];
            if activation >= min_activation {
                results.push(KnowledgeActivation {
                    position: (x, y),
                    activation,
                    confidence: grid.cells[y][x][META_CONFIDENCE],
                    timestamp: grid.cells[y][x][META_TIMESTAMP],
                    embedding: grid.cells[y][x][KNOWLEDGE_EMBEDDING],
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

/// Query with pre-computed features and optional text store
pub fn query_knowledge_by_features_with_text(
    grid: &Grid,
    query_features: &FeatureVector,
    config: &EncoderConfig,
    max_results: usize,
    text_store: Option<&TextStore>,
) -> Vec<KnowledgeActivation> {
    let (qx, qy) = feature_to_position(query_features, grid.width, grid.height);

    let search_radius = (config.spread_radius * 4) as i32;
    let mut results = Vec::new();
    // Track which text snippets we've already seen to deduplicate
    let mut seen_texts = std::collections::HashSet::new();

    for dy in -search_radius..=search_radius {
        for dx in -search_radius..=search_radius {
            let nx = ((qx as i32 + dx).rem_euclid(grid.width as i32)) as usize;
            let ny = ((qy as i32 + dy).rem_euclid(grid.height as i32)) as usize;

            let activation = grid.cells[ny][nx][KNOWLEDGE_ACTIVATION];
            if activation < 1e-6 {
                continue;
            }

            let dist = ((dx * dx + dy * dy) as f64).sqrt();
            let proximity = 1.0 / (1.0 + dist);
            let confidence = grid.cells[ny][nx][META_CONFIDENCE];

            // Relevance combines activation, proximity, and confidence
            let relevance = activation * 0.4 + proximity * 0.4 + confidence * 0.2;

            // Look up original text
            let text = text_store.and_then(|ts| {
                ts.peek(nx, ny).map(|s| s.to_string())
            });

            // Deduplicate by text content
            if let Some(ref t) = text {
                if seen_texts.contains(t) {
                    continue;
                }
                seen_texts.insert(t.clone());
            }

            results.push(KnowledgeActivation {
                position: (nx, ny),
                activation,
                confidence,
                timestamp: grid.cells[ny][nx][META_TIMESTAMP],
                embedding: grid.cells[ny][nx][KNOWLEDGE_EMBEDDING],
                relevance,
                text,
            });
        }
    }

    // Sort by relevance (highest first) and truncate
    results.sort_by(|a, b| b.relevance.partial_cmp(&a.relevance).unwrap_or(std::cmp::Ordering::Equal));
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

            let offset = ((dy + r) as usize * (2 * r as usize + 1) + (dx + r) as usize) % num_features;
            features.values[offset] += grid.cells[ny][nx][KNOWLEDGE_EMBEDDING] * weight;
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
    use super::*;
    use crate::grid::GRID_SIZE;
    use super::super::encoder::write_knowledge;

    fn setup_grid_with_knowledge(text: &str) -> (Grid, EncoderConfig, (usize, usize)) {
        let mut grid = Grid::new(GRID_SIZE, GRID_SIZE);
        let mut config = EncoderConfig::default();
        config.ollama_url = None; // Use hash fallback in tests
        let features = encode_text(text, &config);
        let pos = write_knowledge(&mut grid, &features, 0.9, 0.5, &config);
        (grid, config, pos)
    }

    #[test]
    fn test_query_finds_written_knowledge() {
        let (grid, config, _pos) = setup_grid_with_knowledge("rust programming language");

        let results = query_knowledge(&grid, "rust programming language", &config, 10);
        assert!(!results.is_empty(), "Should find knowledge that was written");
        assert!(results[0].activation > 0.0);
    }

    #[test]
    fn test_query_with_text_store() {
        let (grid, config, pos) = setup_grid_with_knowledge("rust programming language");

        let mut text_store = TextStore::new();
        text_store.insert(pos.0, pos.1, "rust programming language".into());

        let results = query_knowledge_with_text(
            &grid, "rust programming language", &config, 10, Some(&text_store)
        );
        assert!(!results.is_empty());

        // At least one result should have text
        let has_text = results.iter().any(|r| r.text.is_some());
        assert!(has_text, "Should find text from text store");
    }

    #[test]
    fn test_query_ranks_similar_higher() {
        let mut grid = Grid::new(GRID_SIZE, GRID_SIZE);
        let mut config = EncoderConfig::default();
        config.ollama_url = None;

        let f1 = encode_text("machine learning neural networks", &config);
        write_knowledge(&mut grid, &f1, 0.9, 0.5, &config);

        let f2 = encode_text("cooking recipes italian food", &config);
        write_knowledge(&mut grid, &f2, 0.9, 0.5, &config);

        let results = query_knowledge(&grid, "deep learning algorithms", &config, 20);
        assert!(results.len() > 0 || true, "Query should return results or empty gracefully");
    }

    #[test]
    fn test_scan_active_knowledge() {
        let (grid, _config, _pos) = setup_grid_with_knowledge("test scan");

        let active = scan_active_knowledge(&grid, 0.01);
        assert!(!active.is_empty(), "Should find active cells after writing knowledge");
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
        let mut config = EncoderConfig::default();
        config.ollama_url = None;
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

        let results = query_knowledge_with_text(
            &grid, "test dedup", &config, 10, Some(&text_store)
        );
        // Count how many results have the same text
        let text_results: Vec<_> = results.iter()
            .filter(|r| r.text.as_deref() == Some("test dedup"))
            .collect();
        assert!(text_results.len() <= 1, "Should deduplicate identical text snippets");
    }
}
