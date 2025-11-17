// Visual Memory System - Integrates vision, learning, and dreams
//
// This module implements SAGE's cross-modal learning and dream-vision integration.
// Visual experiences are converted to NCA training targets, stored in memory,
// and replayed/remixed during dream mode for memory consolidation.

use crate::grid::Grid;
use crate::vision::VisualFeatures;
use std::collections::{VecDeque, HashMap};
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Serialize, Deserialize};

/// Maximum number of visual memories to retain
const MAX_VISUAL_MEMORIES: usize = 100;

/// Visual memory system that bridges vision, learning, and dreams
pub struct VisualMemory {
    /// Recent visual experiences with timestamps and emotional tags
    recent_experiences: VecDeque<VisualExperience>,

    /// Co-occurrence tracking for concept relationships
    concept_pairs: HashMap<(String, String), u32>,

    /// Queue of visual concepts to replay in dreams
    dream_queue: VecDeque<Vec<String>>,

    /// Statistics for curiosity-driven learning
    novelty_scores: HashMap<String, f64>,
}

/// A single visual experience captured by SAGE
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualExperience {
    pub timestamp: u64,
    pub features: StoredVisualFeatures,
    pub concepts: Vec<String>,
    pub emotional_tag: EmotionalTag,
    pub novelty: f64,
}

/// Simplified visual features for storage (VisualFeatures can't be serialized directly)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredVisualFeatures {
    pub avg_brightness: f64,
    pub avg_r: f64,
    pub avg_g: f64,
    pub avg_b: f64,
    pub color_variance: f64,
    pub dominant_color: String,
    pub edge_strength: f64,
}

impl From<&VisualFeatures> for StoredVisualFeatures {
    fn from(features: &VisualFeatures) -> Self {
        Self {
            avg_brightness: features.avg_brightness,
            avg_r: features.avg_r,
            avg_g: features.avg_g,
            avg_b: features.avg_b,
            color_variance: features.color_variance,
            dominant_color: features.dominant_color.clone(),
            edge_strength: features.edge_strength,
        }
    }
}

/// Emotional response to visual experience
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum EmotionalTag {
    Curious,      // High novelty, want to learn more
    Beautiful,    // Aesthetically pleasing patterns
    Confusing,    // Unexpected or hard to process
    Familiar,     // Low novelty, already understood
    Exciting,     // High variance/movement
    Peaceful,     // Low variance/stable
}

impl VisualMemory {
    pub fn new() -> Self {
        Self {
            recent_experiences: VecDeque::with_capacity(MAX_VISUAL_MEMORIES),
            concept_pairs: HashMap::new(),
            dream_queue: VecDeque::new(),
            novelty_scores: HashMap::new(),
        }
    }

    /// Record a new visual experience
    pub fn record_experience(&mut self, features: &VisualFeatures, concepts: Vec<String>) {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Calculate novelty based on how recently we've seen similar concepts
        let novelty = self.calculate_novelty(&concepts);

        // Determine emotional response based on features and novelty
        let emotional_tag = self.classify_emotion(features, novelty);

        let experience = VisualExperience {
            timestamp,
            features: features.into(),
            concepts: concepts.clone(),
            emotional_tag,
            novelty,
        };

        // Track concept co-occurrences
        self.update_concept_pairs(&concepts);

        // Add to recent experiences
        self.recent_experiences.push_back(experience);

        // Maintain size limit
        if self.recent_experiences.len() > MAX_VISUAL_MEMORIES {
            self.recent_experiences.pop_front();
        }

        // Add novel experiences to dream queue
        if novelty > 0.5 {
            self.dream_queue.push_back(concepts.clone());
            if self.dream_queue.len() > 20 {
                self.dream_queue.pop_front();
            }
        }

        // Update novelty scores
        for concept in &concepts {
            self.novelty_scores.insert(concept.clone(), novelty);
        }
    }

    /// Calculate how novel a set of concepts is
    fn calculate_novelty(&self, concepts: &[String]) -> f64 {
        if concepts.is_empty() {
            return 0.0;
        }

        let mut total_novelty = 0.0;
        for concept in concepts {
            // Novelty decreases the more we've seen this concept
            let seen_count = self.novelty_scores.get(concept).unwrap_or(&1.0);
            total_novelty += 1.0 / seen_count;
        }

        (total_novelty / concepts.len() as f64).min(1.0)
    }

    /// Classify emotional response to visual experience
    fn classify_emotion(&self, features: &VisualFeatures, novelty: f64) -> EmotionalTag {
        // High novelty = curiosity
        if novelty > 0.7 {
            return EmotionalTag::Curious;
        }

        // High color variance + moderate brightness = beautiful
        if features.color_variance > 0.2 && features.avg_brightness > 0.4 && features.avg_brightness < 0.8 {
            return EmotionalTag::Beautiful;
        }

        // High edge strength + high variance = exciting
        if features.edge_strength > 40.0 && features.color_variance > 0.15 {
            return EmotionalTag::Exciting;
        }

        // Low variance + moderate brightness = peaceful
        if features.color_variance < 0.1 && features.avg_brightness > 0.3 && features.avg_brightness < 0.7 {
            return EmotionalTag::Peaceful;
        }

        // Moderate novelty = confusing
        if novelty > 0.3 && novelty < 0.7 {
            return EmotionalTag::Confusing;
        }

        EmotionalTag::Familiar
    }

    /// Track which concepts appear together
    fn update_concept_pairs(&mut self, concepts: &[String]) {
        for i in 0..concepts.len() {
            for j in (i + 1)..concepts.len() {
                let pair = if concepts[i] < concepts[j] {
                    (concepts[i].clone(), concepts[j].clone())
                } else {
                    (concepts[j].clone(), concepts[i].clone())
                };
                *self.concept_pairs.entry(pair).or_insert(0) += 1;
            }
        }
    }

    /// Get a visual experience to replay in dreams (with remixing)
    pub fn get_dream_material(&mut self) -> Option<Vec<String>> {
        if self.dream_queue.is_empty() && !self.recent_experiences.is_empty() {
            // If dream queue is empty, sample from recent experiences
            let idx = (rand::random::<usize>() % self.recent_experiences.len()).min(self.recent_experiences.len() - 1);
            return Some(self.recent_experiences[idx].concepts.clone());
        }

        self.dream_queue.pop_front()
    }

    /// Mix two sets of visual concepts (for dream remixing)
    pub fn remix_concepts(&self, concepts1: &[String], concepts2: &[String]) -> Vec<String> {
        let mut mixed = Vec::new();

        // Take some from each
        let take1 = (concepts1.len() + 1) / 2;
        let take2 = (concepts2.len() + 1) / 2;

        mixed.extend_from_slice(&concepts1[..take1.min(concepts1.len())]);
        mixed.extend_from_slice(&concepts2[..take2.min(concepts2.len())]);

        mixed
    }

    /// Get recently seen concepts for curiosity-driven exploration
    pub fn get_recent_concepts(&self, count: usize) -> Vec<String> {
        let mut concepts = Vec::new();
        for exp in self.recent_experiences.iter().rev().take(count) {
            concepts.extend(exp.concepts.clone());
        }
        concepts.sort();
        concepts.dedup();
        concepts
    }

    /// Get experiences with specific emotional tag
    pub fn get_experiences_by_emotion(&self, tag: EmotionalTag) -> Vec<&VisualExperience> {
        self.recent_experiences
            .iter()
            .filter(|exp| exp.emotional_tag == tag)
            .collect()
    }

    /// Get total experience count
    pub fn experience_count(&self) -> usize {
        self.recent_experiences.len()
    }
}

/// Convert visual features to NCA pattern target
pub fn visual_features_to_pattern(features: &VisualFeatures, size: usize) -> Grid {
    let mut grid = Grid::new(size, size);
    let center = size as f64 / 2.0;

    // Strategy: Map visual properties to spatial patterns

    // 1. Brightness → Alpha (overall opacity)
    let base_alpha = features.avg_brightness;

    // 2. Dominant color → RGB channels
    let (r_mult, g_mult, b_mult) = match features.dominant_color.as_str() {
        "red" => (1.0, 0.5, 0.5),
        "green" => (0.5, 1.0, 0.5),
        "blue" => (0.5, 0.5, 1.0),
        _ => (0.7, 0.7, 0.7), // neutral
    };

    // 3. Edge strength → Pattern sharpness
    // High edge = sharp circular gradient
    // Low edge = soft diffuse gradient
    let gradient_power = if features.edge_strength > 30.0 { 3.0 } else { 1.5 };

    // 4. Color variance → Pattern complexity
    // High variance = multiple circles/layers
    // Low variance = simple solid
    let use_layers = features.color_variance > 0.15;

    for y in 0..size {
        for x in 0..size {
            let dx = x as f64 - center;
            let dy = y as f64 - center;
            let distance = (dx * dx + dy * dy).sqrt();
            let normalized_dist = distance / (size as f64 / 2.0);

            // Calculate intensity based on distance (radial gradient)
            let intensity = (1.0 - normalized_dist.min(1.0)).powf(gradient_power);

            // Apply color multipliers
            let r = (features.avg_r * r_mult * intensity).min(1.0);
            let g = (features.avg_g * g_mult * intensity).min(1.0);
            let b = (features.avg_b * b_mult * intensity).min(1.0);
            let alpha = (base_alpha * intensity).min(1.0);

            // Add layering for high color variance
            let final_alpha = if use_layers && normalized_dist > 0.5 && normalized_dist < 0.75 {
                alpha * 1.2 // Brighten middle ring
            } else {
                alpha
            };

            grid.cells[y][x][0] = r;
            grid.cells[y][x][1] = g;
            grid.cells[y][x][2] = b;
            grid.cells[y][x][3] = final_alpha;
        }
    }

    grid
}

/// Convert visual concepts (strings) to NCA pattern target
pub fn concepts_to_pattern(concepts: &[String], size: usize) -> Grid {
    let mut grid = Grid::new(size, size);

    // Simple mapping: use first concept to determine pattern type
    if concepts.is_empty() {
        return grid;
    }

    let primary_concept = &concepts[0].to_lowercase();

    // Map concepts to colors and patterns
    if primary_concept.contains("red") || primary_concept.contains("warm") {
        create_colored_circle(&mut grid, size, 1.0, 0.3, 0.3);
    } else if primary_concept.contains("green") || primary_concept.contains("natural") {
        create_colored_circle(&mut grid, size, 0.3, 1.0, 0.3);
    } else if primary_concept.contains("blue") || primary_concept.contains("cool") {
        create_colored_circle(&mut grid, size, 0.3, 0.3, 1.0);
    } else if primary_concept.contains("bright") || primary_concept.contains("light") {
        create_colored_circle(&mut grid, size, 0.9, 0.9, 0.9);
    } else if primary_concept.contains("dark") || primary_concept.contains("dim") {
        create_colored_circle(&mut grid, size, 0.2, 0.2, 0.2);
    } else if primary_concept.contains("edge") || primary_concept.contains("sharp") {
        create_cross_pattern(&mut grid, size);
    } else {
        // Default: moderate gray circle
        create_colored_circle(&mut grid, size, 0.5, 0.5, 0.5);
    }

    grid
}

/// Helper: Create a colored circle pattern
fn create_colored_circle(grid: &mut Grid, size: usize, r: f64, g: f64, b: f64) {
    let center = size as f64 / 2.0;
    let radius = size as f64 / 3.0;

    for y in 0..size {
        for x in 0..size {
            let dx = x as f64 - center;
            let dy = y as f64 - center;
            let dist = (dx * dx + dy * dy).sqrt();

            if dist <= radius {
                let intensity = 1.0 - (dist / radius).powf(1.5);
                grid.cells[y][x][0] = r * intensity;
                grid.cells[y][x][1] = g * intensity;
                grid.cells[y][x][2] = b * intensity;
                grid.cells[y][x][3] = intensity;
            }
        }
    }
}

/// Helper: Create a cross/edge pattern
fn create_cross_pattern(grid: &mut Grid, size: usize) {
    let center = size as f64 / 2.0;
    let thickness = 4.0;
    let half_thickness = thickness / 2.0;

    for y in 0..size {
        for x in 0..size {
            let x_dist = (x as f64 - center).abs();
            let y_dist = (y as f64 - center).abs();

            // Vertical or horizontal bar
            if x_dist < half_thickness || y_dist < half_thickness {
                grid.cells[y][x][0] = 0.8;
                grid.cells[y][x][1] = 0.8;
                grid.cells[y][x][2] = 0.8;
                grid.cells[y][x][3] = 1.0;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_visual_memory_basic() {
        let mut memory = VisualMemory::new();

        let features = VisualFeatures {
            avg_brightness: 0.8,
            avg_r: 0.9,
            avg_g: 0.5,
            avg_b: 0.4,
            color_variance: 0.25,
            dominant_color: "red".to_string(),
            edge_strength: 45.0,
        };

        memory.record_experience(&features, vec!["red_dominant".to_string(), "bright".to_string()]);

        assert_eq!(memory.experience_count(), 1);
        assert!(memory.get_recent_concepts(10).contains(&"red_dominant".to_string()));
    }

    #[test]
    fn test_concept_remixing() {
        let memory = VisualMemory::new();
        let concepts1 = vec!["red".to_string(), "circle".to_string()];
        let concepts2 = vec!["blue".to_string(), "square".to_string()];

        let mixed = memory.remix_concepts(&concepts1, &concepts2);

        // Should contain elements from both
        assert!(mixed.len() >= 2);
    }
}
