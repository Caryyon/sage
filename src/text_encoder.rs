// Text encoding for NCA processing - converts language into spatial patterns

use crate::grid::{Grid, GRID_SIZE};
use std::collections::HashMap;

/// Encodes text into NCA-compatible grid patterns
pub struct TextEncoder {
    /// Character → spatial pattern mapping
    char_embeddings: HashMap<char, Vec<f64>>,
    /// Word frequency for importance weighting
    word_frequencies: HashMap<String, usize>,
}

impl TextEncoder {
    pub fn new() -> Self {
        Self {
            char_embeddings: Self::initialize_char_embeddings(),
            word_frequencies: HashMap::new(),
        }
    }

    /// Initialize character embeddings using simple spatial encoding
    /// Each character gets a unique pattern that the NCA can learn
    fn initialize_char_embeddings() -> HashMap<char, Vec<f64>> {
        let mut embeddings = HashMap::new();

        // Create deterministic patterns for each character
        // Using prime number spacing to ensure uniqueness
        let chars = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789 .,!?-";

        for (i, ch) in chars.chars().enumerate() {
            let mut pattern = vec![0.0; 64]; // 8x8 mini-pattern for each char

            // Create unique spatial pattern using character index
            for j in 0..64 {
                let phase = (i * 7 + j * 3) as f64; // Prime multiples for uniqueness
                pattern[j] = ((phase.sin() + 1.0) / 2.0).clamp(0.0, 1.0);
            }

            embeddings.insert(ch, pattern);
        }

        embeddings
    }

    /// Encode text into a 32x32 NCA grid
    /// Strategy: Place character patterns in sequence, wrapping as needed
    pub fn encode_text(&mut self, text: &str) -> Grid {
        let mut grid = Grid::new(GRID_SIZE, GRID_SIZE);

        // Track word frequency for preference learning
        for word in text.split_whitespace() {
            *self.word_frequencies.entry(word.to_lowercase()).or_insert(0) += 1;
        }

        let chars: Vec<char> = text.chars().take(128).collect(); // Limit to first 128 chars

        // Place character patterns in 8x8 blocks across the grid
        // Grid is 32x32, so we can fit 4x4 = 16 character blocks
        for (idx, &ch) in chars.iter().enumerate() {
            if idx >= 16 { break; } // Max 16 characters per grid

            let block_x = (idx % 4) * 8;
            let block_y = (idx / 4) * 8;

            if let Some(pattern) = self.char_embeddings.get(&ch) {
                // Place 8x8 pattern into grid
                for dy in 0..8 {
                    for dx in 0..8 {
                        let grid_y = block_y + dy;
                        let grid_x = block_x + dx;
                        let pattern_idx = dy * 8 + dx;

                        if grid_y < GRID_SIZE && grid_x < GRID_SIZE && pattern_idx < pattern.len() {
                            // Set alpha channel (pattern visibility)
                            grid.cells[grid_y][grid_x][3] = pattern[pattern_idx];
                            // Set color based on character type
                            if ch.is_alphabetic() {
                                grid.cells[grid_y][grid_x][0] = 0.6; // R
                                grid.cells[grid_y][grid_x][1] = 0.8; // G
                                grid.cells[grid_y][grid_x][2] = 1.0; // B - Blue for letters
                            } else if ch.is_numeric() {
                                grid.cells[grid_y][grid_x][0] = 1.0; // R - Red for numbers
                                grid.cells[grid_y][grid_x][1] = 0.6; // G
                                grid.cells[grid_y][grid_x][2] = 0.6; // B
                            } else {
                                grid.cells[grid_y][grid_x][0] = 0.8; // R - Purple for symbols
                                grid.cells[grid_y][grid_x][1] = 0.6; // G
                                grid.cells[grid_y][grid_x][2] = 0.8; // B
                            }
                        }
                    }
                }
            } else {
                // Unknown character - use neutral pattern
                for dy in 0..8 {
                    for dx in 0..8 {
                        let grid_y = block_y + dy;
                        let grid_x = block_x + dx;
                        if grid_y < GRID_SIZE && grid_x < GRID_SIZE {
                            grid.cells[grid_y][grid_x][3] = 0.3; // Low alpha for unknown
                        }
                    }
                }
            }
        }

        grid
    }

    /// Encode a concept/word into a simpler pattern
    /// Useful for single-word opinion formation
    pub fn encode_concept(&mut self, concept: &str) -> Grid {
        let normalized = concept.to_lowercase();
        let grid = self.encode_text(&normalized);

        grid
    }

    /// Get word frequency (how often SAGE has seen this word)
    pub fn get_word_frequency(&self, word: &str) -> usize {
        *self.word_frequencies.get(&word.to_lowercase()).unwrap_or(&0)
    }

    /// Get most frequent words (what SAGE has been exposed to most)
    pub fn get_top_words(&self, limit: usize) -> Vec<(String, usize)> {
        let mut sorted: Vec<_> = self.word_frequencies.iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect();
        sorted.sort_by(|a, b| b.1.cmp(&a.1));
        sorted.into_iter().take(limit).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_simple_text() {
        let mut encoder = TextEncoder::new();
        let grid = encoder.encode_text("hello");

        // Should have non-zero cells
        let mut has_content = false;
        for row in &grid.cells {
            for cell in row {
                if cell[3] > 0.0 {
                    has_content = true;
                    break;
                }
            }
        }
        assert!(has_content);
    }

    #[test]
    fn test_word_frequency_tracking() {
        let mut encoder = TextEncoder::new();
        encoder.encode_text("hello world hello");

        assert_eq!(encoder.get_word_frequency("hello"), 2);
        assert_eq!(encoder.get_word_frequency("world"), 1);
        assert_eq!(encoder.get_word_frequency("unknown"), 0);
    }
}
