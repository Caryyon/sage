//! Concept Self-Organizing Map - Grounds language in internal states
//!
//! This implements Layer 1 of the grounded language system.
//! Words gain meaning by association with clusters of internal states.

use std::collections::HashMap;
use rand::Rng;
use serde::{Deserialize, Serialize};

/// Dimensions of the grounding state vector
pub const GROUNDING_DIM: usize = 16;

/// Size of the SOM grid
pub const SOM_SIZE: usize = 20;

/// State vector for grounding language
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct GroundingState {
    /// Energy level (0-1)
    pub energy: f64,
    /// Loneliness feeling (0-1)
    pub loneliness: f64,
    /// Boredom level (0-1)
    pub boredom: f64,
    /// Creative urge (0-1)
    pub creative_urge: f64,
    /// Emotional valence (-1 to 1, negative to positive)
    pub valence: f64,
    /// Arousal/activation level (0-1)
    pub arousal: f64,
    /// Time of day (0-1, morning to night)
    pub time_of_day: f64,
    /// Weather temperature (0-1, cold to hot)
    pub weather_temp: f64,
    /// Weather condition (0-1, clear to stormy)
    pub weather_condition: f64,
    /// Relationship closeness with current person (0-1)
    pub relationship_closeness: f64,
    /// Current conversation length (0-1, normalized)
    pub conversation_length: f64,
    /// Topic emotional weight (-1 to 1)
    pub topic_sentiment: f64,
    /// Current activity engagement (0-1)
    pub activity_engagement: f64,
    /// Seasonal energy (0-1)
    pub seasonal_energy: f64,
    /// Social satiation (0-1, how "peopled out")
    pub social_satiation: f64,
    /// Novelty of current topic (0-1)
    pub novelty: f64,
}

impl GroundingState {
    /// Convert to vector for SOM operations
    pub fn to_vector(&self) -> Vec<f64> {
        vec![
            self.energy,
            self.loneliness,
            self.boredom,
            self.creative_urge,
            (self.valence + 1.0) / 2.0, // Normalize to 0-1
            self.arousal,
            self.time_of_day,
            self.weather_temp,
            self.weather_condition,
            self.relationship_closeness,
            self.conversation_length,
            (self.topic_sentiment + 1.0) / 2.0, // Normalize to 0-1
            self.activity_engagement,
            self.seasonal_energy,
            self.social_satiation,
            self.novelty,
        ]
    }

    /// Create from vector
    pub fn from_vector(v: &[f64]) -> Self {
        Self {
            energy: v.get(0).copied().unwrap_or(0.5),
            loneliness: v.get(1).copied().unwrap_or(0.5),
            boredom: v.get(2).copied().unwrap_or(0.5),
            creative_urge: v.get(3).copied().unwrap_or(0.5),
            valence: v.get(4).copied().unwrap_or(0.5) * 2.0 - 1.0,
            arousal: v.get(5).copied().unwrap_or(0.5),
            time_of_day: v.get(6).copied().unwrap_or(0.5),
            weather_temp: v.get(7).copied().unwrap_or(0.5),
            weather_condition: v.get(8).copied().unwrap_or(0.5),
            relationship_closeness: v.get(9).copied().unwrap_or(0.5),
            conversation_length: v.get(10).copied().unwrap_or(0.5),
            topic_sentiment: v.get(11).copied().unwrap_or(0.5) * 2.0 - 1.0,
            activity_engagement: v.get(12).copied().unwrap_or(0.5),
            seasonal_energy: v.get(13).copied().unwrap_or(0.5),
            social_satiation: v.get(14).copied().unwrap_or(0.5),
            novelty: v.get(15).copied().unwrap_or(0.5),
        }
    }

    /// Euclidean distance to another state
    pub fn distance(&self, other: &Self) -> f64 {
        let v1 = self.to_vector();
        let v2 = other.to_vector();
        v1.iter()
            .zip(v2.iter())
            .map(|(a, b)| (a - b).powi(2))
            .sum::<f64>()
            .sqrt()
    }
}

/// A single node in the SOM
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConceptNode {
    /// Prototype state vector this node represents
    pub prototype: Vec<f64>,
    /// Words associated with this concept (word -> strength)
    pub associated_words: HashMap<String, f64>,
    /// Total activation count (for statistics)
    pub activation_count: u64,
    /// Human-readable label (derived from strongest associations)
    pub label: Option<String>,
}

impl ConceptNode {
    fn new(dim: usize) -> Self {
        let mut rng = rand::thread_rng();
        Self {
            prototype: (0..dim).map(|_| rng.gen()).collect(),
            associated_words: HashMap::new(),
            activation_count: 0,
            label: None,
        }
    }

    /// Associate a word with this concept
    pub fn associate_word(&mut self, word: &str, strength: f64) {
        let entry = self.associated_words.entry(word.to_lowercase()).or_insert(0.0);
        // Exponential moving average
        *entry = *entry * 0.9 + strength * 0.1;
    }

    /// Get the strongest word associations
    pub fn top_words(&self, n: usize) -> Vec<(String, f64)> {
        let mut words: Vec<_> = self.associated_words.iter()
            .map(|(w, s)| (w.clone(), *s))
            .collect();
        words.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        words.truncate(n);
        words
    }

    /// Update label based on associations
    pub fn update_label(&mut self) {
        if let Some((word, strength)) = self.top_words(1).first() {
            if *strength > 0.3 {
                self.label = Some(word.clone());
            }
        }
    }
}

/// Self-Organizing Map for concept grounding
#[derive(Clone, Serialize, Deserialize)]
pub struct ConceptSOM {
    /// 2D grid of concept nodes
    pub nodes: Vec<Vec<ConceptNode>>,
    /// Grid size
    pub size: usize,
    /// Current learning rate
    pub learning_rate: f64,
    /// Current neighborhood radius
    pub radius: f64,
    /// Total training iterations
    pub iterations: u64,
    /// Initial learning rate (for decay)
    initial_lr: f64,
    /// Initial radius (for decay)
    initial_radius: f64,
}

impl ConceptSOM {
    pub fn new() -> Self {
        let nodes: Vec<Vec<ConceptNode>> = (0..SOM_SIZE)
            .map(|_| (0..SOM_SIZE).map(|_| ConceptNode::new(GROUNDING_DIM)).collect())
            .collect();

        Self {
            nodes,
            size: SOM_SIZE,
            learning_rate: 0.5,
            radius: SOM_SIZE as f64 / 2.0,
            iterations: 0,
            initial_lr: 0.5,
            initial_radius: SOM_SIZE as f64 / 2.0,
        }
    }

    /// Find the Best Matching Unit (BMU) for a state vector
    pub fn find_bmu(&self, state: &[f64]) -> (usize, usize) {
        let mut best_dist = f64::MAX;
        let mut best_pos = (0, 0);

        for y in 0..self.size {
            for x in 0..self.size {
                let dist = euclidean_distance(state, &self.nodes[y][x].prototype);
                if dist < best_dist {
                    best_dist = dist;
                    best_pos = (y, x);
                }
            }
        }

        best_pos
    }

    /// Update the SOM with a new observation
    pub fn train(&mut self, state: &GroundingState, words: &[String]) {
        let state_vec = state.to_vector();
        let (bmu_y, bmu_x) = self.find_bmu(&state_vec);

        // Update neighborhood
        for y in 0..self.size {
            for x in 0..self.size {
                let dist = ((y as f64 - bmu_y as f64).powi(2) +
                           (x as f64 - bmu_x as f64).powi(2)).sqrt();

                if dist <= self.radius {
                    // Gaussian neighborhood function
                    let influence = (-dist.powi(2) / (2.0 * self.radius.powi(2))).exp();
                    let learning = self.learning_rate * influence;

                    // Update prototype
                    for (i, &s) in state_vec.iter().enumerate() {
                        self.nodes[y][x].prototype[i] += learning * (s - self.nodes[y][x].prototype[i]);
                    }
                }
            }
        }

        // Associate words with BMU
        self.nodes[bmu_y][bmu_x].activation_count += 1;
        for word in words {
            self.nodes[bmu_y][bmu_x].associate_word(word, 1.0);
        }

        // Decay learning parameters
        self.iterations += 1;
        self.decay_parameters();
    }

    /// Decay learning rate and radius over time
    fn decay_parameters(&mut self) {
        let time_constant = 1000.0;
        let decay = (-(self.iterations as f64) / time_constant).exp();
        self.learning_rate = self.initial_lr * decay;
        self.radius = self.initial_radius * decay;

        // Minimum bounds
        self.learning_rate = self.learning_rate.max(0.01);
        self.radius = self.radius.max(1.0);
    }

    /// Get words associated with a state
    pub fn get_words_for_state(&self, state: &GroundingState) -> Vec<(String, f64)> {
        let state_vec = state.to_vector();
        let (y, x) = self.find_bmu(&state_vec);
        self.nodes[y][x].top_words(10)
    }

    /// Get the concept cluster coordinates for a state
    pub fn get_concept(&self, state: &GroundingState) -> (usize, usize) {
        let state_vec = state.to_vector();
        self.find_bmu(&state_vec)
    }

    /// Get a description of the concept at a position
    pub fn describe_concept(&self, y: usize, x: usize) -> String {
        let node = &self.nodes[y][x];
        if let Some(label) = &node.label {
            label.clone()
        } else {
            let words = node.top_words(3);
            if words.is_empty() {
                format!("concept_{},{}", y, x)
            } else {
                words.iter().map(|(w, _)| w.as_str()).collect::<Vec<_>>().join("/")
            }
        }
    }

    /// Update all node labels
    pub fn update_labels(&mut self) {
        for row in &mut self.nodes {
            for node in row {
                node.update_label();
            }
        }
    }

    /// Get activation statistics
    pub fn get_stats(&self) -> SOMStats {
        let mut total_activations = 0u64;
        let mut total_words = 0usize;
        let mut active_nodes = 0usize;
        let mut max_activations = 0u64;

        for row in &self.nodes {
            for node in row {
                total_activations += node.activation_count;
                total_words += node.associated_words.len();
                if node.activation_count > 0 {
                    active_nodes += 1;
                }
                max_activations = max_activations.max(node.activation_count);
            }
        }

        SOMStats {
            total_nodes: self.size * self.size,
            active_nodes,
            total_activations,
            max_activations,
            total_words,
            iterations: self.iterations,
            current_lr: self.learning_rate,
            current_radius: self.radius,
        }
    }

    /// Serialize to JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Deserialize from JSON
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

impl Default for ConceptSOM {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about the SOM
#[derive(Debug, Clone)]
pub struct SOMStats {
    pub total_nodes: usize,
    pub active_nodes: usize,
    pub total_activations: u64,
    pub max_activations: u64,
    pub total_words: usize,
    pub iterations: u64,
    pub current_lr: f64,
    pub current_radius: f64,
}

/// Euclidean distance between two vectors
fn euclidean_distance(a: &[f64], b: &[f64]) -> f64 {
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| (x - y).powi(2))
        .sum::<f64>()
        .sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grounding_state_vector() {
        let state = GroundingState {
            energy: 0.8,
            loneliness: 0.2,
            valence: 0.5,
            ..Default::default()
        };

        let vec = state.to_vector();
        assert_eq!(vec.len(), GROUNDING_DIM);
        assert!((vec[0] - 0.8).abs() < 0.001);
    }

    #[test]
    fn test_som_creation() {
        let som = ConceptSOM::new();
        assert_eq!(som.nodes.len(), SOM_SIZE);
        assert_eq!(som.nodes[0].len(), SOM_SIZE);
    }

    #[test]
    fn test_bmu_finding() {
        let som = ConceptSOM::new();
        let state = GroundingState::default();
        let (y, x) = som.find_bmu(&state.to_vector());
        assert!(y < SOM_SIZE);
        assert!(x < SOM_SIZE);
    }

    #[test]
    fn test_training() {
        let mut som = ConceptSOM::new();

        let state = GroundingState {
            energy: 0.2,
            valence: -0.5,
            ..Default::default()
        };

        som.train(&state, &["tired".to_string(), "exhausted".to_string()]);

        let words = som.get_words_for_state(&state);
        assert!(words.iter().any(|(w, _)| w == "tired"));
    }

    #[test]
    fn test_word_association_strengthens() {
        let mut som = ConceptSOM::new();

        let state = GroundingState {
            energy: 0.9,
            valence: 0.8,
            ..Default::default()
        };

        // Train multiple times with same word
        for _ in 0..10 {
            som.train(&state, &["happy".to_string()]);
        }

        let words = som.get_words_for_state(&state);
        let happy_strength = words.iter()
            .find(|(w, _)| w == "happy")
            .map(|(_, s)| *s)
            .unwrap_or(0.0);

        assert!(happy_strength > 0.5, "Word association should strengthen with repetition");
    }

    #[test]
    fn test_similar_states_cluster() {
        let mut som = ConceptSOM::new();

        // Train with similar states
        let tired_state = GroundingState {
            energy: 0.1,
            valence: -0.2,
            ..Default::default()
        };

        let also_tired_state = GroundingState {
            energy: 0.15,
            valence: -0.15,
            ..Default::default()
        };

        som.train(&tired_state, &["tired".to_string()]);
        som.train(&also_tired_state, &["sleepy".to_string()]);

        // Both should map to nearby BMUs
        let (y1, x1) = som.get_concept(&tired_state);
        let (y2, x2) = som.get_concept(&also_tired_state);

        let dist = ((y1 as f64 - y2 as f64).powi(2) + (x1 as f64 - x2 as f64).powi(2)).sqrt();
        assert!(dist < 5.0, "Similar states should cluster together");
    }
}
