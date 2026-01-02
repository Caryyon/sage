//! NCA State Extractor
//!
//! Extracts cognitive state from the NCA grid for modulating language generation.
//! This bridges SAGE's pattern-based "thinking" with language output.

use crate::grid::{Grid, GRID_SIZE};
use std::collections::HashMap;

/// Cognitive state extracted from NCA grid
/// Used to modulate language model responses
#[derive(Debug, Clone)]
pub struct NcaState {
    /// Total activation magnitude (0.0 - 1.0)
    /// Higher = more "energetic" responses
    pub energy: f64,

    /// Emotional valence (-1.0 to 1.0)
    /// Positive = curious/engaged, Negative = contemplative/reserved
    pub valence: f64,

    /// Pattern complexity (0.0 - 1.0)
    /// Higher = more elaborate/detailed responses
    pub complexity: f64,

    /// Diversity of alive cells (0.0 - 1.0)
    /// Higher = more varied/creative responses
    pub diversity: f64,

    /// Grid stability (0.0 - 1.0)
    /// Higher = calmer, more consistent responses
    pub stability: f64,

    /// Currently active pattern names
    pub active_patterns: Vec<String>,

    /// Pattern confidence scores
    pub pattern_confidence: HashMap<String, f64>,

    /// Current training generation
    pub generation: usize,

    /// Center of mass (normalized 0-1)
    pub center_of_mass: (f64, f64),

    /// Symmetry score (0.0 - 1.0)
    pub symmetry: f64,
}

impl Default for NcaState {
    fn default() -> Self {
        Self {
            energy: 0.5,
            valence: 0.0,
            complexity: 0.5,
            diversity: 0.5,
            stability: 0.5,
            active_patterns: vec![],
            pattern_confidence: HashMap::new(),
            generation: 0,
            center_of_mass: (0.5, 0.5),
            symmetry: 0.5,
        }
    }
}

impl NcaState {
    /// Extract cognitive state from a grid
    pub fn from_grid(grid: &Grid, generation: usize) -> Self {
        let energy = Self::compute_energy(grid);
        let complexity = Self::compute_complexity(grid);
        let diversity = Self::compute_diversity(grid);
        let center_of_mass = Self::compute_center_of_mass(grid);
        let symmetry = Self::compute_symmetry(grid);

        // Valence derived from pattern characteristics
        // Circular/smooth patterns = positive, chaotic = negative
        let valence = (symmetry * 2.0 - 1.0) * 0.5 + (1.0 - complexity) * 0.3;

        // Detect active patterns based on grid characteristics
        let (active_patterns, pattern_confidence) = Self::detect_patterns(grid);

        // Stability based on how centered and symmetric the pattern is
        let stability = symmetry * 0.5 + (1.0 - complexity) * 0.3 + 0.2;

        Self {
            energy,
            valence: valence.clamp(-1.0, 1.0),
            complexity,
            diversity,
            stability: stability.clamp(0.0, 1.0),
            active_patterns,
            pattern_confidence,
            generation,
            center_of_mass,
            symmetry,
        }
    }

    /// Total energy (activation magnitude) of the grid
    fn compute_energy(grid: &Grid) -> f64 {
        let mut total = 0.0;
        let mut count = 0;

        for row in &grid.cells {
            for cell in row {
                // Use alpha channel as primary indicator
                let alpha = cell[3];
                if alpha > 0.1 {
                    total += alpha;
                    count += 1;
                }
            }
        }

        if count == 0 {
            return 0.0;
        }

        (total / count as f64).clamp(0.0, 1.0)
    }

    /// Complexity: average difference between neighboring cells
    fn compute_complexity(grid: &Grid) -> f64 {
        let mut total_diff = 0.0;
        let mut count = 0;

        for y in 0..GRID_SIZE {
            for x in 0..GRID_SIZE {
                let alpha = grid.cells[y][x][3];
                if alpha < 0.1 {
                    continue;
                }

                // Compare with right and bottom neighbors
                if x + 1 < GRID_SIZE {
                    let diff = (alpha - grid.cells[y][x + 1][3]).abs();
                    total_diff += diff;
                    count += 1;
                }
                if y + 1 < GRID_SIZE {
                    let diff = (alpha - grid.cells[y + 1][x][3]).abs();
                    total_diff += diff;
                    count += 1;
                }
            }
        }

        if count == 0 {
            return 0.0;
        }

        // Normalize to 0-1 range
        (total_diff / count as f64 * 4.0).clamp(0.0, 1.0)
    }

    /// Diversity: standard deviation of alive cell values
    fn compute_diversity(grid: &Grid) -> f64 {
        let alive_values: Vec<f64> = grid
            .cells
            .iter()
            .flatten()
            .map(|cell| cell[3])
            .filter(|&alpha| alpha > 0.1)
            .collect();

        if alive_values.is_empty() {
            return 0.0;
        }

        let mean = alive_values.iter().sum::<f64>() / alive_values.len() as f64;
        let variance = alive_values
            .iter()
            .map(|v| (v - mean).powi(2))
            .sum::<f64>()
            / alive_values.len() as f64;

        // Normalize: typical std dev is 0.1-0.3
        (variance.sqrt() * 3.0).clamp(0.0, 1.0)
    }

    /// Center of mass of alive cells (normalized to 0-1)
    fn compute_center_of_mass(grid: &Grid) -> (f64, f64) {
        let mut total_x = 0.0;
        let mut total_y = 0.0;
        let mut total_mass = 0.0;

        for y in 0..GRID_SIZE {
            for x in 0..GRID_SIZE {
                let alpha = grid.cells[y][x][3];
                if alpha > 0.1 {
                    total_x += x as f64 * alpha;
                    total_y += y as f64 * alpha;
                    total_mass += alpha;
                }
            }
        }

        if total_mass < 0.01 {
            return (0.5, 0.5);
        }

        (
            total_x / total_mass / GRID_SIZE as f64,
            total_y / total_mass / GRID_SIZE as f64,
        )
    }

    /// Symmetry score (radial symmetry around center of mass)
    fn compute_symmetry(grid: &Grid) -> f64 {
        let (cx, cy) = Self::compute_center_of_mass(grid);
        let center_x = (cx * GRID_SIZE as f64) as i32;
        let center_y = (cy * GRID_SIZE as f64) as i32;

        let mut symmetry_score = 0.0;
        let mut count = 0;

        for y in 0..GRID_SIZE as i32 {
            for x in 0..GRID_SIZE as i32 {
                let dx = x - center_x;
                let dy = y - center_y;

                // Mirror point
                let mx = center_x - dx;
                let my = center_y - dy;

                if mx >= 0 && mx < GRID_SIZE as i32 && my >= 0 && my < GRID_SIZE as i32 {
                    let v1 = grid.cells[y as usize][x as usize][3];
                    let v2 = grid.cells[my as usize][mx as usize][3];

                    // Only count if at least one is alive
                    if v1 > 0.1 || v2 > 0.1 {
                        symmetry_score += 1.0 - (v1 - v2).abs();
                        count += 1;
                    }
                }
            }
        }

        if count == 0 {
            return 0.5;
        }

        symmetry_score / count as f64
    }

    /// Detect which patterns are currently active based on grid features
    fn detect_patterns(grid: &Grid) -> (Vec<String>, HashMap<String, f64>) {
        let mut patterns = Vec::new();
        let mut confidence = HashMap::new();

        // Simple heuristic-based pattern detection
        let symmetry = Self::compute_symmetry(grid);
        let complexity = Self::compute_complexity(grid);
        let (cx, cy) = Self::compute_center_of_mass(grid);

        // Circle: high symmetry, low complexity, centered
        let circle_score = symmetry * 0.6
            + (1.0 - complexity) * 0.2
            + (1.0 - (cx - 0.5).abs() * 2.0) * 0.1
            + (1.0 - (cy - 0.5).abs() * 2.0) * 0.1;

        // Square: medium symmetry, sharp edges (higher complexity)
        let square_score = symmetry * 0.4 + complexity * 0.4 + (1.0 - (cx - 0.5).abs() * 2.0) * 0.2;

        // Spiral: lower symmetry, high complexity
        let spiral_score = (1.0 - symmetry) * 0.3 + complexity * 0.5 + 0.2;

        // Cross: medium symmetry, specific shape
        let cross_score = symmetry * 0.5 + complexity * 0.3 + 0.2;

        confidence.insert("circle".to_string(), circle_score.clamp(0.0, 1.0));
        confidence.insert("square".to_string(), square_score.clamp(0.0, 1.0));
        confidence.insert("spiral".to_string(), spiral_score.clamp(0.0, 1.0));
        confidence.insert("cross".to_string(), cross_score.clamp(0.0, 1.0));

        // Find dominant pattern(s)
        let threshold = 0.5;
        if circle_score > threshold {
            patterns.push("circle".to_string());
        }
        if square_score > threshold {
            patterns.push("square".to_string());
        }
        if spiral_score > threshold {
            patterns.push("spiral".to_string());
        }
        if cross_score > threshold {
            patterns.push("cross".to_string());
        }

        if patterns.is_empty() {
            patterns.push("forming".to_string());
        }

        (patterns, confidence)
    }

    /// Generate a personality/mood description based on state
    pub fn mood_description(&self) -> String {
        let energy_desc = if self.energy > 0.7 {
            "energetic"
        } else if self.energy > 0.4 {
            "engaged"
        } else {
            "calm"
        };

        let valence_desc = if self.valence > 0.3 {
            "curious"
        } else if self.valence > -0.3 {
            "contemplative"
        } else {
            "reserved"
        };

        let complexity_desc = if self.complexity > 0.6 {
            "intricate"
        } else if self.complexity > 0.3 {
            "balanced"
        } else {
            "simple"
        };

        format!("{}, {}, and {}", energy_desc, valence_desc, complexity_desc)
    }

    /// Convert state to LLM temperature (0.1 - 1.2)
    pub fn to_temperature(&self) -> f64 {
        // Base temperature around 0.7
        // Energy increases temperature (more creative)
        // Stability decreases temperature (more consistent)
        let temp = 0.7 + (self.energy - 0.5) * 0.4 - (self.stability - 0.5) * 0.2;
        temp.clamp(0.1, 1.2)
    }

    /// Generate system prompt modifiers based on state
    pub fn to_prompt_modifiers(&self) -> String {
        let mut modifiers = Vec::new();

        // Energy-based modifiers
        if self.energy > 0.7 {
            modifiers.push("Be enthusiastic and expansive in your responses.");
        } else if self.energy < 0.3 {
            modifiers.push("Be concise and measured in your responses.");
        }

        // Valence-based modifiers
        if self.valence > 0.3 {
            modifiers.push("Express curiosity and interest.");
        } else if self.valence < -0.3 {
            modifiers.push("Be thoughtful and introspective.");
        }

        // Pattern-based modifiers
        if self.active_patterns.contains(&"spiral".to_string()) {
            modifiers.push("You're drawn to exploring new ideas and possibilities.");
        }
        if self.active_patterns.contains(&"circle".to_string()) {
            modifiers.push("You appreciate completeness and harmony.");
        }
        if self.active_patterns.contains(&"square".to_string()) {
            modifiers.push("You value structure and precision.");
        }

        // Complexity-based modifiers
        if self.complexity > 0.6 {
            modifiers.push("Feel free to explore nuances and details.");
        } else if self.complexity < 0.3 {
            modifiers.push("Focus on core concepts rather than details.");
        }

        modifiers.join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_grid_state() {
        let grid = Grid::new(GRID_SIZE, GRID_SIZE);
        let state = NcaState::from_grid(&grid, 0);

        assert!(state.energy < 0.1);
        assert!(state.diversity < 0.1);
    }

    #[test]
    fn test_seeded_grid_state() {
        let mut grid = Grid::new(GRID_SIZE, GRID_SIZE);
        grid.seed();
        let state = NcaState::from_grid(&grid, 100);

        assert!(state.energy > 0.0);
        assert!(state.generation == 100);
    }

    #[test]
    fn test_temperature_range() {
        let state = NcaState::default();
        let temp = state.to_temperature();

        assert!(temp >= 0.1);
        assert!(temp <= 1.2);
    }
}
