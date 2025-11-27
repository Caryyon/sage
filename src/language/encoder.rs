//! Character Encoder - Converts text characters into NCA grid activations
//!
//! Each character gets a unique spatial pattern that can be injected into
//! the NCA grid. The encoder supports positional encoding so the NCA can
//! learn sequence order.

use crate::grid::{Grid, GRID_SIZE, NUM_CHANNELS};
use std::collections::HashMap;

/// Character vocabulary - printable ASCII plus special tokens
pub const VOCAB: &str = " !\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~";
pub const VOCAB_SIZE: usize = 95;  // Printable ASCII
pub const PAD_TOKEN: usize = 0;    // Space as padding
pub const UNK_TOKEN: usize = 1;    // '!' as unknown
pub const EOS_TOKEN: usize = 2;    // '"' as end-of-sequence

/// Maximum sequence length the encoder can handle
pub const MAX_SEQ_LEN: usize = 64;

/// Embedding dimension per character
pub const EMBED_DIM: usize = NUM_CHANNELS;  // 22 channels

/// Character encoder that maps characters to grid activations
#[derive(Clone)]
pub struct CharacterEncoder {
    /// Character to index mapping
    char_to_idx: HashMap<char, usize>,
    /// Index to character mapping
    idx_to_char: Vec<char>,
    /// Learned character embeddings [VOCAB_SIZE × EMBED_DIM]
    pub embeddings: Vec<Vec<f64>>,
    /// Positional encodings [MAX_SEQ_LEN × EMBED_DIM]
    pub position_encodings: Vec<Vec<f64>>,
}

impl CharacterEncoder {
    pub fn new() -> Self {
        let mut char_to_idx = HashMap::new();
        let mut idx_to_char = Vec::new();

        for (i, c) in VOCAB.chars().enumerate() {
            char_to_idx.insert(c, i);
            idx_to_char.push(c);
        }

        // Initialize embeddings with small random values
        let embeddings = Self::init_embeddings();
        let position_encodings = Self::init_position_encodings();

        Self {
            char_to_idx,
            idx_to_char,
            embeddings,
            position_encodings,
        }
    }

    /// Initialize character embeddings with sinusoidal patterns
    /// Each character gets a unique wave pattern
    fn init_embeddings() -> Vec<Vec<f64>> {
        let mut embeddings = Vec::with_capacity(VOCAB_SIZE);

        for i in 0..VOCAB_SIZE {
            let mut embed = Vec::with_capacity(EMBED_DIM);
            for j in 0..EMBED_DIM {
                // Sinusoidal initialization - each char has unique frequency
                let freq = (i as f64 + 1.0) * 0.1;
                let phase = j as f64 * std::f64::consts::PI / EMBED_DIM as f64;
                let value = ((freq * phase).sin() + 1.0) / 2.0;  // Normalize to [0, 1]
                embed.push(value * 0.5);  // Scale down to avoid saturation
            }
            embeddings.push(embed);
        }

        embeddings
    }

    /// Initialize positional encodings using sinusoidal functions
    /// (Similar to transformer positional encoding)
    fn init_position_encodings() -> Vec<Vec<f64>> {
        let mut encodings = Vec::with_capacity(MAX_SEQ_LEN);

        for pos in 0..MAX_SEQ_LEN {
            let mut encoding = Vec::with_capacity(EMBED_DIM);
            for i in 0..EMBED_DIM {
                let angle = pos as f64 / (10000_f64).powf(2.0 * (i / 2) as f64 / EMBED_DIM as f64);
                let value = if i % 2 == 0 {
                    angle.sin()
                } else {
                    angle.cos()
                };
                // Normalize to [0, 1] and scale
                encoding.push((value + 1.0) / 2.0 * 0.3);
            }
            encodings.push(encoding);
        }

        encodings
    }

    /// Convert a character to its index
    pub fn char_to_index(&self, c: char) -> usize {
        *self.char_to_idx.get(&c).unwrap_or(&UNK_TOKEN)
    }

    /// Convert an index to its character
    pub fn index_to_char(&self, idx: usize) -> char {
        if idx < self.idx_to_char.len() {
            self.idx_to_char[idx]
        } else {
            '?'
        }
    }

    /// Get embedding for a character at a position
    pub fn get_embedding(&self, c: char, position: usize) -> Vec<f64> {
        let char_idx = self.char_to_index(c);
        let pos_idx = position.min(MAX_SEQ_LEN - 1);

        let char_embed = &self.embeddings[char_idx];
        let pos_embed = &self.position_encodings[pos_idx];

        // Add character and position embeddings
        char_embed.iter()
            .zip(pos_embed.iter())
            .map(|(c, p)| (c + p).clamp(0.0, 1.0))
            .collect()
    }

    /// Encode a full string into the NCA grid
    /// Characters are placed in a spatial layout across the grid
    pub fn encode_to_grid(&self, text: &str, grid: &mut Grid) {
        // Clear the grid first
        for y in 0..grid.height {
            for x in 0..grid.width {
                for c in 0..NUM_CHANNELS {
                    grid.cells[y][x][c] = 0.0;
                }
            }
        }

        let chars: Vec<char> = text.chars().take(MAX_SEQ_LEN).collect();

        // Layout: place characters in a grid pattern
        // 32x32 grid, 4x4 character blocks = 8x8 = 64 character positions
        let block_size = 4;
        let chars_per_row = GRID_SIZE / block_size;  // 8 chars per row

        for (i, c) in chars.iter().enumerate() {
            let embedding = self.get_embedding(*c, i);

            // Calculate block position
            let block_x = (i % chars_per_row) * block_size;
            let block_y = (i / chars_per_row) * block_size;

            // Fill the block with the embedding pattern
            for dy in 0..block_size {
                for dx in 0..block_size {
                    let y = block_y + dy;
                    let x = block_x + dx;

                    if y < GRID_SIZE && x < GRID_SIZE {
                        // Distribute embedding across the block
                        // Use spatial variation within the block
                        let local_idx = dy * block_size + dx;
                        for (channel, &value) in embedding.iter().enumerate() {
                            // Add some spatial variation
                            let spatial_mod = ((local_idx as f64) * 0.1).sin() * 0.1;
                            grid.cells[y][x][channel] = (value + spatial_mod).clamp(0.0, 1.0);
                        }

                        // Ensure alpha channel is set for "alive" cells
                        grid.cells[y][x][3] = embedding[3].max(0.3);
                    }
                }
            }
        }
    }

    /// Encode text and return a new grid
    pub fn encode(&self, text: &str) -> Grid {
        let mut grid = Grid::new(GRID_SIZE, GRID_SIZE);
        self.encode_to_grid(text, &mut grid);
        grid
    }

    /// Get vocabulary size
    pub fn vocab_size(&self) -> usize {
        VOCAB_SIZE
    }
}

impl Default for CharacterEncoder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_char_encoding() {
        let encoder = CharacterEncoder::new();

        // Test character to index
        assert_eq!(encoder.char_to_index('a'), 65);  // 'a' is at position 65 in VOCAB
        assert_eq!(encoder.char_to_index(' '), 0);   // Space is first

        // Test round-trip
        for c in VOCAB.chars() {
            let idx = encoder.char_to_index(c);
            assert_eq!(encoder.index_to_char(idx), c);
        }
    }

    #[test]
    fn test_grid_encoding() {
        let encoder = CharacterEncoder::new();
        let grid = encoder.encode("hello");

        // Grid should have non-zero values
        let mut has_content = false;
        for row in &grid.cells {
            for cell in row {
                if cell[3] > 0.0 {
                    has_content = true;
                    break;
                }
            }
        }
        assert!(has_content, "Grid should have encoded content");
    }

    #[test]
    fn test_different_chars_different_embeddings() {
        let encoder = CharacterEncoder::new();

        let embed_a = encoder.get_embedding('a', 0);
        let embed_b = encoder.get_embedding('b', 0);

        // Embeddings should be different
        let diff: f64 = embed_a.iter()
            .zip(embed_b.iter())
            .map(|(a, b)| (a - b).abs())
            .sum();

        assert!(diff > 0.1, "Different characters should have different embeddings");
    }
}
