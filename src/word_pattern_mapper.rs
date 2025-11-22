//! Word-Pattern Mapper - Grounds language in NCA patterns
//!
//! This module creates bidirectional mappings between words and NCA patterns:
//! - Word → Pattern: "circle" activates the circle pattern in NCA
//! - Pattern → Word: Recognizing a circle pattern can generate "circle"
//!
//! This is the foundation of language grounding - connecting symbols to substrates.

use crate::grid::{Grid, GRID_SIZE};
use std::collections::HashMap;

/// Maps words to NCA pattern types and generators
pub struct WordPatternMapper {
    /// Word → pattern type mapping
    word_to_pattern: HashMap<String, PatternType>,
    /// Pattern type → associated words (for generation)
    pattern_to_words: HashMap<PatternType, Vec<String>>,
    /// Emotional words → gradient parameters
    emotion_mappings: HashMap<String, EmotionGradient>,
    /// Learning history: tracks word-pattern associations seen
    association_strength: HashMap<(String, PatternType), f64>,
}

/// Types of patterns SAGE can produce
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PatternType {
    // Tier 1: Basic shapes
    Circle,
    Square,
    Cross,
    Spiral,
    // Tier 2: Advanced shapes
    Triangle,
    Star,
    Ring,
    Hexagon,
    Checkerboard,
    // Special patterns
    Gradient,
    Noise,
    Custom,
}

/// Emotional gradient parameters for mood-based patterns
#[derive(Debug, Clone)]
pub struct EmotionGradient {
    pub valence: f64,      // -1.0 (negative) to 1.0 (positive)
    pub arousal: f64,      // 0.0 (calm) to 1.0 (excited)
    pub color_bias: [f64; 3], // RGB color shift
}

impl WordPatternMapper {
    pub fn new() -> Self {
        let mut mapper = Self {
            word_to_pattern: HashMap::new(),
            pattern_to_words: HashMap::new(),
            emotion_mappings: HashMap::new(),
            association_strength: HashMap::new(),
        };
        mapper.initialize_default_mappings();
        mapper
    }

    /// Initialize default word-pattern associations
    fn initialize_default_mappings(&mut self) {
        // Shape words → patterns
        let shape_mappings = [
            // Circle variants
            ("circle", PatternType::Circle),
            ("round", PatternType::Circle),
            ("sphere", PatternType::Circle),
            ("ball", PatternType::Circle),
            ("dot", PatternType::Circle),
            ("sun", PatternType::Circle),
            ("moon", PatternType::Circle),
            // Square variants
            ("square", PatternType::Square),
            ("box", PatternType::Square),
            ("cube", PatternType::Square),
            ("rectangle", PatternType::Square),
            ("block", PatternType::Square),
            // Cross variants
            ("cross", PatternType::Cross),
            ("plus", PatternType::Cross),
            ("x", PatternType::Cross),
            ("intersection", PatternType::Cross),
            // Spiral variants
            ("spiral", PatternType::Spiral),
            ("swirl", PatternType::Spiral),
            ("vortex", PatternType::Spiral),
            ("helix", PatternType::Spiral),
            ("tornado", PatternType::Spiral),
            // Triangle variants
            ("triangle", PatternType::Triangle),
            ("pyramid", PatternType::Triangle),
            ("arrow", PatternType::Triangle),
            ("point", PatternType::Triangle),
            // Star variants
            ("star", PatternType::Star),
            ("sparkle", PatternType::Star),
            ("twinkle", PatternType::Star),
            // Ring variants
            ("ring", PatternType::Ring),
            ("donut", PatternType::Ring),
            ("torus", PatternType::Ring),
            ("halo", PatternType::Ring),
            // Hexagon variants
            ("hexagon", PatternType::Hexagon),
            ("honeycomb", PatternType::Hexagon),
            ("bee", PatternType::Hexagon),
            // Checkerboard variants
            ("checkerboard", PatternType::Checkerboard),
            ("chess", PatternType::Checkerboard),
            ("grid", PatternType::Checkerboard),
            ("tiles", PatternType::Checkerboard),
        ];

        for (word, pattern) in shape_mappings {
            self.word_to_pattern.insert(word.to_string(), pattern);
            self.pattern_to_words
                .entry(pattern)
                .or_insert_with(Vec::new)
                .push(word.to_string());
        }

        // Emotion words → gradients
        let emotion_mappings = [
            // Positive emotions (warm colors, high activity)
            ("happy", EmotionGradient { valence: 1.0, arousal: 0.7, color_bias: [1.0, 0.8, 0.0] }),
            ("joy", EmotionGradient { valence: 1.0, arousal: 0.8, color_bias: [1.0, 0.9, 0.2] }),
            ("excited", EmotionGradient { valence: 0.8, arousal: 1.0, color_bias: [1.0, 0.5, 0.0] }),
            ("love", EmotionGradient { valence: 1.0, arousal: 0.6, color_bias: [1.0, 0.3, 0.5] }),
            ("calm", EmotionGradient { valence: 0.6, arousal: 0.1, color_bias: [0.5, 0.7, 1.0] }),
            ("peaceful", EmotionGradient { valence: 0.7, arousal: 0.0, color_bias: [0.6, 0.9, 0.6] }),
            // Negative emotions (cool/dark colors, varied activity)
            ("sad", EmotionGradient { valence: -0.8, arousal: 0.2, color_bias: [0.3, 0.3, 0.6] }),
            ("angry", EmotionGradient { valence: -0.7, arousal: 1.0, color_bias: [0.9, 0.1, 0.1] }),
            ("fear", EmotionGradient { valence: -0.9, arousal: 0.8, color_bias: [0.2, 0.1, 0.3] }),
            ("anxious", EmotionGradient { valence: -0.5, arousal: 0.7, color_bias: [0.5, 0.4, 0.3] }),
            // Neutral/cognitive states
            ("curious", EmotionGradient { valence: 0.3, arousal: 0.5, color_bias: [0.7, 0.5, 1.0] }),
            ("focused", EmotionGradient { valence: 0.2, arousal: 0.4, color_bias: [0.4, 0.6, 0.8] }),
            ("confused", EmotionGradient { valence: -0.2, arousal: 0.5, color_bias: [0.6, 0.5, 0.5] }),
        ];

        for (word, gradient) in emotion_mappings {
            self.emotion_mappings.insert(word.to_string(), gradient);
        }
    }

    /// Get pattern type for a word (if mapped)
    pub fn word_to_pattern(&self, word: &str) -> Option<PatternType> {
        self.word_to_pattern.get(&word.to_lowercase()).copied()
    }

    /// Get pattern index for conditioning (0-8 for the 9 patterns)
    pub fn pattern_to_index(&self, pattern: PatternType) -> usize {
        match pattern {
            PatternType::Circle => 0,
            PatternType::Square => 1,
            PatternType::Cross => 2,
            PatternType::Spiral => 3,
            PatternType::Triangle => 4,
            PatternType::Star => 5,
            PatternType::Ring => 6,
            PatternType::Hexagon => 7,
            PatternType::Checkerboard => 8,
            PatternType::Gradient | PatternType::Noise | PatternType::Custom => 0,
        }
    }

    /// Get emotion gradient for a word
    pub fn word_to_emotion(&self, word: &str) -> Option<&EmotionGradient> {
        self.emotion_mappings.get(&word.to_lowercase())
    }

    /// Generate a target grid for a pattern type
    pub fn generate_pattern_grid(&self, pattern: PatternType) -> Grid {
        let mut grid = Grid::new(GRID_SIZE, GRID_SIZE);
        let center = GRID_SIZE as f64 / 2.0;

        match pattern {
            PatternType::Circle => {
                let radius = 8.0;
                for y in 0..GRID_SIZE {
                    for x in 0..GRID_SIZE {
                        let dx = x as f64 - center;
                        let dy = y as f64 - center;
                        let dist = (dx * dx + dy * dy).sqrt();
                        if dist < radius {
                            grid.cells[y][x][3] = 1.0;
                            grid.cells[y][x][0] = 1.0;
                        }
                    }
                }
            }
            PatternType::Square => {
                let half_size = 8;
                let start = GRID_SIZE / 2 - half_size;
                let end = GRID_SIZE / 2 + half_size;
                for y in start..end {
                    for x in start..end {
                        grid.cells[y][x][3] = 1.0;
                        grid.cells[y][x][0] = 1.0;
                    }
                }
            }
            PatternType::Cross => {
                let arm_width = 4;
                let half_width = arm_width / 2;
                for y in 0..GRID_SIZE {
                    for x in 0..GRID_SIZE {
                        let in_vertical = x >= GRID_SIZE/2 - half_width && x < GRID_SIZE/2 + half_width;
                        let in_horizontal = y >= GRID_SIZE/2 - half_width && y < GRID_SIZE/2 + half_width;
                        if in_vertical || in_horizontal {
                            grid.cells[y][x][3] = 1.0;
                            grid.cells[y][x][0] = 1.0;
                        }
                    }
                }
            }
            PatternType::Spiral => {
                for y in 0..GRID_SIZE {
                    for x in 0..GRID_SIZE {
                        let dx = x as f64 - center;
                        let dy = y as f64 - center;
                        let r = (dx * dx + dy * dy).sqrt();
                        let theta = dy.atan2(dx);
                        let spiral_r = 2.0 + theta.abs() * 2.0;
                        if (r - spiral_r).abs() < 2.0 && r < 14.0 {
                            grid.cells[y][x][3] = 1.0;
                            grid.cells[y][x][0] = 1.0;
                        }
                    }
                }
            }
            PatternType::Triangle => {
                let size = 12.0;
                for y in 0..GRID_SIZE {
                    for x in 0..GRID_SIZE {
                        let px = x as f64 - center;
                        let py = y as f64 - center + size/2.0;
                        let in_triangle = py >= 0.0 && py <= size && px.abs() <= (size - py) / 2.0;
                        if in_triangle {
                            grid.cells[y][x][3] = 1.0;
                            grid.cells[y][x][0] = 1.0;
                        }
                    }
                }
            }
            PatternType::Star => {
                let outer_r = 12.0;
                let inner_r = 5.0;
                for y in 0..GRID_SIZE {
                    for x in 0..GRID_SIZE {
                        let dx = x as f64 - center;
                        let dy = y as f64 - center;
                        let r = (dx * dx + dy * dy).sqrt();
                        let theta = dy.atan2(dx);
                        let point_r = inner_r + (outer_r - inner_r) * ((theta * 2.5).cos().abs());
                        if r < point_r {
                            grid.cells[y][x][3] = 1.0;
                            grid.cells[y][x][0] = 1.0;
                        }
                    }
                }
            }
            PatternType::Ring => {
                let outer_r = 10.0;
                let inner_r = 5.0;
                for y in 0..GRID_SIZE {
                    for x in 0..GRID_SIZE {
                        let dx = x as f64 - center;
                        let dy = y as f64 - center;
                        let dist = (dx * dx + dy * dy).sqrt();
                        if dist >= inner_r && dist < outer_r {
                            grid.cells[y][x][3] = 1.0;
                            grid.cells[y][x][0] = 1.0;
                        }
                    }
                }
            }
            PatternType::Hexagon => {
                let size = 10.0;
                for y in 0..GRID_SIZE {
                    for x in 0..GRID_SIZE {
                        let dx = (x as f64 - center).abs();
                        let dy = (y as f64 - center).abs();
                        if dx <= size && dy <= size * 0.866 && dx + dy * 0.577 <= size {
                            grid.cells[y][x][3] = 1.0;
                            grid.cells[y][x][0] = 1.0;
                        }
                    }
                }
            }
            PatternType::Checkerboard => {
                let cell_size = 4;
                for y in 0..GRID_SIZE {
                    for x in 0..GRID_SIZE {
                        let is_white = ((x / cell_size) + (y / cell_size)) % 2 == 0;
                        if is_white {
                            grid.cells[y][x][3] = 1.0;
                            grid.cells[y][x][0] = 1.0;
                        }
                    }
                }
            }
            PatternType::Gradient => {
                for y in 0..GRID_SIZE {
                    for x in 0..GRID_SIZE {
                        let val = x as f64 / GRID_SIZE as f64;
                        grid.cells[y][x][3] = val;
                        grid.cells[y][x][0] = val;
                    }
                }
            }
            PatternType::Noise => {
                use rand::Rng;
                let mut rng = rand::thread_rng();
                for y in 0..GRID_SIZE {
                    for x in 0..GRID_SIZE {
                        let val = rng.gen::<f64>();
                        grid.cells[y][x][3] = val;
                        grid.cells[y][x][0] = val;
                    }
                }
            }
            PatternType::Custom => {
                // Empty grid for custom patterns
            }
        }

        grid
    }

    /// Apply emotion gradient to a grid (modifies colors based on mood)
    pub fn apply_emotion_to_grid(&self, grid: &mut Grid, emotion: &EmotionGradient) {
        for y in 0..grid.height {
            for x in 0..grid.width {
                if grid.cells[y][x][3] > 0.1 {  // Only affect alive cells
                    // Blend existing color with emotion color bias
                    let blend = 0.5;
                    grid.cells[y][x][0] = grid.cells[y][x][0] * (1.0 - blend) + emotion.color_bias[0] * blend;
                    grid.cells[y][x][1] = grid.cells[y][x][1] * (1.0 - blend) + emotion.color_bias[1] * blend;
                    grid.cells[y][x][2] = grid.cells[y][x][2] * (1.0 - blend) + emotion.color_bias[2] * blend;
                }
            }
        }
    }

    /// Parse text and extract pattern/emotion activations
    pub fn parse_text(&self, text: &str) -> TextActivation {
        let words: Vec<&str> = text.split_whitespace().collect();
        let mut patterns = Vec::new();
        let mut emotions = Vec::new();

        for word in words {
            let clean_word = word.trim_matches(|c: char| !c.is_alphanumeric()).to_lowercase();

            if let Some(pattern) = self.word_to_pattern(&clean_word) {
                patterns.push((clean_word.clone(), pattern));
            }

            if let Some(emotion) = self.word_to_emotion(&clean_word) {
                emotions.push((clean_word.clone(), emotion.clone()));
            }
        }

        TextActivation { patterns, emotions }
    }

    /// Strengthen association between word and pattern (learning)
    pub fn learn_association(&mut self, word: &str, pattern: PatternType, strength: f64) {
        let key = (word.to_lowercase(), pattern);
        let current = self.association_strength.get(&key).unwrap_or(&0.0);
        self.association_strength.insert(key, current + strength);
    }

    /// Get words associated with a pattern type
    pub fn pattern_to_words(&self, pattern: PatternType) -> Option<&Vec<String>> {
        self.pattern_to_words.get(&pattern)
    }
}

/// Result of parsing text for activations
#[derive(Debug, Clone)]
pub struct TextActivation {
    pub patterns: Vec<(String, PatternType)>,
    pub emotions: Vec<(String, EmotionGradient)>,
}

impl TextActivation {
    pub fn is_empty(&self) -> bool {
        self.patterns.is_empty() && self.emotions.is_empty()
    }

    pub fn primary_pattern(&self) -> Option<PatternType> {
        self.patterns.first().map(|(_, p)| *p)
    }

    pub fn primary_emotion(&self) -> Option<&EmotionGradient> {
        self.emotions.first().map(|(_, e)| e)
    }
}

impl Default for WordPatternMapper {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_word_to_pattern() {
        let mapper = WordPatternMapper::new();
        assert_eq!(mapper.word_to_pattern("circle"), Some(PatternType::Circle));
        assert_eq!(mapper.word_to_pattern("SQUARE"), Some(PatternType::Square));
        assert_eq!(mapper.word_to_pattern("unknown"), None);
    }

    #[test]
    fn test_parse_text() {
        let mapper = WordPatternMapper::new();
        let activation = mapper.parse_text("I see a happy circle in the sky");
        assert!(!activation.patterns.is_empty());
        assert!(!activation.emotions.is_empty());
    }
}
