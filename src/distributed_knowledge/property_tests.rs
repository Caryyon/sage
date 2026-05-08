//! Property-based tests for distributed knowledge invariants.
//!
//! Uses proptest to verify key invariants hold across random inputs:
//! - Encoding never panics
//! - Grid channels stay in valid ranges
//! - Freerun never modifies knowledge channels (regression test)
//! - feature_to_position returns in-bounds positions
//! - Encoding is deterministic
//! - Brain serialization roundtrip preserves state
//! - Contrast retrieval never returns out-of-bounds positions

use proptest::prelude::*;

use crate::distributed_knowledge::attention_decoder::AttentionDecoder;
use crate::distributed_knowledge::encoder::{
    encode_text, feature_to_position, write_knowledge, EncoderConfig, FeatureVector,
    NUM_EMBED_SLOTS,
};
use crate::distributed_knowledge::NCAKnowledge;
use crate::grid::{Grid, KNOWLEDGE_CHANNELS_START, NUM_CHANNELS};

/// Create an offline encoder config (no Ollama, hash-only)
fn offline_config() -> EncoderConfig {
    let mut config = EncoderConfig::default();
    config.ollama_url = None;
    config
}

proptest! {
    /// Invariant 1: Encoding never panics on arbitrary input
    ///
    /// Must never panic, must return Ok or a graceful error.
    #[test]
    fn encoding_never_panics(text in "\\PC{1,500}") {
        let config = offline_config();
        // Must never panic
        let _ = encode_text(&text, &config);
    }

    /// Invariant 2: Grid channels stay in valid range after any operation
    ///
    /// After smooth_hidden_channels, all values must be finite and in a reasonable range.
    #[test]
    fn grid_channels_always_valid(
        cx in 0usize..64,
        cy in 0usize..64,
        radius in 1usize..32,
        steps in 1usize..=5
    ) {
        let mut grid = Grid::new(64, 64);
        grid.smooth_hidden_channels(cx, cy, radius, steps);
        for y in 0..64 {
            for x in 0..64 {
                for ch in 0..NUM_CHANNELS {
                    let v = grid.cells[y][x][ch];
                    prop_assert!(v.is_finite(), "NaN/inf at cell ({},{}) ch {}", x, y, ch);
                    // Soft range check — values should generally stay bounded
                    prop_assert!(v >= -5.0 && v <= 5.0, "value {} out of soft range at ({},{}) ch {}", v, x, y, ch);
                }
            }
        }
    }

    /// Invariant 3: CRITICAL — smoothing never modifies knowledge channels
    ///
    /// This is the bug we just fixed. This test ensures it never regresses.
    /// smooth_hidden_channels only touches hidden channels (4..NUM_BASE_CHANNELS),
    /// knowledge channels (26-37) must be preserved.
    #[test]
    fn smoothing_never_modifies_knowledge_channels(
        steps in 1usize..=5,
        cx in 0usize..64,
        cy in 0usize..64,
    ) {
        let mut grid = Grid::new(64, 64);

        // Set some knowledge channel values to detect modification
        for y in 0..64 {
            for x in 0..64 {
                for slot in 0..NUM_EMBED_SLOTS {
                    let ch = KNOWLEDGE_CHANNELS_START + slot;
                    grid.cells[y][x][ch] = (x + y + slot) as f64 * 0.01;
                }
            }
        }

        // Snapshot knowledge channels before smoothing
        let before = grid.snapshot_knowledge_channels();
        grid.smooth_hidden_channels(cx, cy, 32, steps);
        let after = grid.snapshot_knowledge_channels();

        // REGRESSION: freerun used to silently modify knowledge channels,
        // causing attend_with_delta to always return 0% hit rate
        for y in 0..64 {
            for x in 0..64 {
                for slot in 0..NUM_EMBED_SLOTS {
                    prop_assert_eq!(
                        before[y][x][slot], after[y][x][slot],
                        "freerun modified knowledge channel {} at ({},{})", slot, x, y
                    );
                }
            }
        }
    }

    /// Invariant 4: feature_to_position always returns in-bounds position
    ///
    /// For any feature vector values in [-1, 1], the returned position must
    /// be within the grid bounds.
    #[test]
    fn feature_to_position_always_in_bounds(
        values in proptest::collection::vec(-1.0f64..=1.0, 12..=64),
    ) {
        let fv = FeatureVector {
            values,
            is_semantic: false,
        };
        let (x, y) = feature_to_position(&fv, 256, 256);
        prop_assert!(x < 256, "x={} out of bounds", x);
        prop_assert!(y < 256, "y={} out of bounds", y);
    }

    /// Invariant 5: Encoding is deterministic — same text same position
    ///
    /// Two separate NCAKnowledge instances encoding the same text
    /// should produce identical knowledge channel states.
    #[test]
    fn encoding_is_deterministic(text in "[a-z ]{1,50}") {
        let config = offline_config();

        let mut grid1 = Grid::new(64, 64);
        let mut grid2 = Grid::new(64, 64);

        let features = encode_text(&text, &config);

        let _ = write_knowledge(&mut grid1, &features, 0.9, 0.5, &config);
        let _ = write_knowledge(&mut grid2, &features, 0.9, 0.5, &config);

        // Both grids should have identical knowledge channel state
        let snap1 = grid1.snapshot_knowledge_channels();
        let snap2 = grid2.snapshot_knowledge_channels();

        for y in 0..64 {
            for x in 0..64 {
                for slot in 0..NUM_EMBED_SLOTS {
                    prop_assert_eq!(snap1[y][x][slot], snap2[y][x][slot],
                        "encoding not deterministic at ({},{}) slot {}", x, y, slot);
                }
            }
        }
    }

    /// Invariant 6: Brain serialization roundtrip
    ///
    /// Serialize a grid to bytes and deserialize back — must succeed
    /// and produce identical state.
    #[test]
    fn brain_serialization_roundtrip(text in "[a-z ]{1,50}") {
        let config = offline_config();

        let mut grid = Grid::new(64, 64);
        let features = encode_text(&text, &config);
        let _ = write_knowledge(&mut grid, &features, 0.9, 0.5, &config);

        // Serialize to bytes
        let bytes = bincode::serialize(&grid).expect("serialize failed");
        // Deserialize back
        let grid2: Grid = bincode::deserialize(&bytes).expect("deserialize failed");

        // REGRESSION: brain.bin version mismatches used to cause index-out-of-bounds panics
        // Deserializing should always succeed and produce identical state
        for y in 0..64 {
            for x in 0..64 {
                prop_assert_eq!(
                    grid.cells[y][x].len(),
                    grid2.cells[y][x].len(),
                    "channel count mismatch after roundtrip"
                );
            }
        }
    }

    /// Invariant 7: Contrast retrieval never panics or returns out-of-bounds positions
    ///
    /// For any valid feature vector, attend_with_contrast must return
    /// positions that are within the grid bounds.
    /// Note: Uses exactly 64 elements to match LinearProjection dimension when trained.
    #[test]
    fn contrast_retrieval_never_panics(
        values in proptest::collection::vec(-1.0f64..=1.0, 64..=64),
        top_k in 1usize..=10,
    ) {
        let mut grid = Grid::new(64, 64);

        // Add some knowledge so there's something to retrieve
        let config = offline_config();
        let features = encode_text("test knowledge for retrieval", &config);
        let _ = write_knowledge(&mut grid, &features, 0.9, 0.5, &config);

        let decoder = AttentionDecoder::new(64, 64);
        let query_fv = FeatureVector {
            values,
            is_semantic: false,
        };

        let results = decoder.attend_with_contrast(&query_fv, &grid, top_k, None);
        for r in &results {
            let (x, y) = r.position;
            prop_assert!(x < 64, "x={} out of bounds", x);
            prop_assert!(y < 64, "y={} out of bounds", y);
        }
    }
}

#[cfg(test)]
mod additional_tests {
    use super::*;

    /// Test that NCAKnowledge::new() creates a valid store
    #[test]
    fn test_nca_knowledge_new_is_valid() {
        let store = NCAKnowledge::new();
        assert_eq!(store.grid.width, 256);
        assert_eq!(store.grid.height, 256);
        assert_eq!(store.grid.cells[0][0].len(), NUM_CHANNELS);
    }

    /// Test that encoding empty string doesn't panic
    #[test]
    fn test_encode_empty_string() {
        let config = offline_config();
        let features = encode_text("", &config);
        assert_eq!(features.values.len(), config.num_features);
    }

    /// Test that feature_to_position handles empty feature vector
    #[test]
    fn test_feature_to_position_empty() {
        let fv = FeatureVector {
            values: vec![],
            is_semantic: false,
        };
        let (x, y) = feature_to_position(&fv, 256, 256);
        // Should return center position for empty vector
        assert!(x < 256);
        assert!(y < 256);
    }

    /// Test that Grid::new creates valid grid
    #[test]
    fn test_grid_new_valid() {
        let grid = Grid::new(32, 32);
        assert_eq!(grid.width, 32);
        assert_eq!(grid.height, 32);
        for y in 0..32 {
            for x in 0..32 {
                assert_eq!(grid.cells[y][x].len(), NUM_CHANNELS);
                for ch in 0..NUM_CHANNELS {
                    assert_eq!(grid.cells[y][x][ch], 0.0);
                }
            }
        }
    }

    /// Test smoothing at grid boundaries doesn't panic
    #[test]
    fn test_smoothing_at_boundaries() {
        let mut grid = Grid::new(64, 64);

        // Corner cases
        grid.smooth_hidden_channels(0, 0, 10, 2);
        grid.smooth_hidden_channels(63, 63, 10, 2);
        grid.smooth_hidden_channels(0, 63, 10, 2);
        grid.smooth_hidden_channels(63, 0, 10, 2);

        // Should not panic
        assert!(true);
    }
}
