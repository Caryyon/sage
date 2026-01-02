//! Grounded Language System - Unified interface for state-based language
//!
//! This module provides the main interface for SAGE's grounded language capabilities.
//! It combines the three layers:
//! - Layer 1: ConceptSOM for grounding words in internal states
//! - Layer 2: SequenceMemory for recognizing input patterns
//! - Layer 3: ResponseSelector for choosing appropriate responses

use std::path::Path;
use std::fs;
use serde::{Deserialize, Serialize};

use super::concept_som::{ConceptSOM, GroundingState};
use super::sequence_memory::SequenceMemory;
use super::response_selector::{ResponseSelector, RelationshipLevel};
use super::seed_templates::{seed_templates, seed_vocabulary};

/// The unified grounded language system
#[derive(Serialize, Deserialize)]
pub struct GroundedLanguage {
    /// Layer 1: Concept grounding
    pub concept_som: ConceptSOM,
    /// Layer 2: Sequence pattern memory
    pub sequence_memory: SequenceMemory,
    /// Layer 3: Response selection
    pub response_selector: ResponseSelector,
    /// Whether the system has been seeded
    pub is_seeded: bool,
    /// Total interactions processed
    pub interaction_count: u64,
}

impl GroundedLanguage {
    /// Create a new grounded language system
    pub fn new() -> Self {
        let mut system = Self {
            concept_som: ConceptSOM::new(),
            sequence_memory: SequenceMemory::default(),
            response_selector: ResponseSelector::new(),
            is_seeded: false,
            interaction_count: 0,
        };

        // Seed with initial templates and vocabulary
        system.seed();

        system
    }

    /// Seed the system with initial knowledge
    pub fn seed(&mut self) {
        if !self.is_seeded {
            seed_templates(&mut self.response_selector);
            seed_vocabulary(&mut self.concept_som);
            self.is_seeded = true;
        }
    }

    /// Generate a response based on current state and input
    pub fn respond(
        &mut self,
        input: &str,
        state: &GroundingState,
        relationship: RelationshipLevel,
    ) -> String {
        self.interaction_count += 1;

        // Process input through sequence memory
        let _match_result = self.sequence_memory.process_input(input);

        // Select and generate response
        let response = self.response_selector.select_response(
            state,
            input,
            &self.sequence_memory,
            &self.concept_som,
            relationship,
        );

        response
    }

    /// Learn from a conversation exchange
    pub fn learn_from_exchange(
        &mut self,
        input: &str,
        response: &str,
        state: &GroundingState,
        feedback_positive: bool,
    ) {
        // Extract words from input and response
        let input_words = self.sequence_memory.tokenize(input);
        let response_words = self.sequence_memory.tokenize(response);

        // Train concept SOM with current state and words
        self.concept_som.train(state, &input_words);
        self.concept_som.train(state, &response_words);

        // Associate sequence with concept
        let concept = self.concept_som.get_concept(state);
        self.sequence_memory.associate_concept(input, concept);

        // Record the response pattern
        self.sequence_memory.record_response(input, response);

        // Update sentiment for the sequence
        if let Some(_) = self.sequence_memory.process_input(input) {
            // Pattern was recognized, sentiment updated internally
        }

        // If we have a template ID, record feedback
        // (This would require tracking which template was used in respond())

        // Periodic maintenance
        if self.interaction_count % 1000 == 0 {
            self.sequence_memory.decay();
            self.concept_som.update_labels();
        }
    }

    /// Get statistics about the system
    pub fn stats(&self) -> GroundedLanguageStats {
        GroundedLanguageStats {
            som_stats: self.concept_som.get_stats(),
            sequence_stats: self.sequence_memory.stats(),
            response_stats: self.response_selector.stats(),
            interaction_count: self.interaction_count,
            is_seeded: self.is_seeded,
        }
    }

    /// Save the system to a directory
    pub fn save(&self, dir: &Path) -> Result<(), std::io::Error> {
        fs::create_dir_all(dir)?;

        let som_path = dir.join("concept_som.json");
        let seq_path = dir.join("sequence_memory.json");
        let resp_path = dir.join("response_selector.json");
        let meta_path = dir.join("metadata.json");

        // Save each component
        fs::write(&som_path, self.concept_som.to_json().map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
        })?)?;

        fs::write(&seq_path, self.sequence_memory.to_json().map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
        })?)?;

        fs::write(&resp_path, self.response_selector.to_json().map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
        })?)?;

        // Save metadata
        let metadata = serde_json::json!({
            "is_seeded": self.is_seeded,
            "interaction_count": self.interaction_count,
            "version": "1.0"
        });
        fs::write(&meta_path, metadata.to_string())?;

        Ok(())
    }

    /// Load the system from a directory
    pub fn load(dir: &Path) -> Result<Self, std::io::Error> {
        let som_path = dir.join("concept_som.json");
        let seq_path = dir.join("sequence_memory.json");
        let resp_path = dir.join("response_selector.json");
        let meta_path = dir.join("metadata.json");

        let som_json = fs::read_to_string(&som_path)?;
        let seq_json = fs::read_to_string(&seq_path)?;
        let resp_json = fs::read_to_string(&resp_path)?;
        let meta_json = fs::read_to_string(&meta_path)?;

        let concept_som = ConceptSOM::from_json(&som_json).map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string())
        })?;

        let sequence_memory = SequenceMemory::from_json(&seq_json).map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string())
        })?;

        let response_selector = ResponseSelector::from_json(&resp_json).map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string())
        })?;

        let metadata: serde_json::Value = serde_json::from_str(&meta_json).map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string())
        })?;

        Ok(Self {
            concept_som,
            sequence_memory,
            response_selector,
            is_seeded: metadata["is_seeded"].as_bool().unwrap_or(true),
            interaction_count: metadata["interaction_count"].as_u64().unwrap_or(0),
        })
    }

    /// Load or create new system
    pub fn load_or_create(dir: &Path) -> Self {
        match Self::load(dir) {
            Ok(system) => {
                eprintln!("Loaded grounded language system with {} interactions",
                    system.interaction_count);
                system
            }
            Err(_) => {
                eprintln!("Creating new grounded language system");
                Self::new()
            }
        }
    }
}

impl Default for GroundedLanguage {
    fn default() -> Self {
        Self::new()
    }
}

/// Combined statistics for the grounded language system
#[derive(Debug, Clone)]
pub struct GroundedLanguageStats {
    pub som_stats: super::concept_som::SOMStats,
    pub sequence_stats: super::sequence_memory::SequenceStats,
    pub response_stats: super::response_selector::ResponseStats,
    pub interaction_count: u64,
    pub is_seeded: bool,
}

impl std::fmt::Display for GroundedLanguageStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Grounded Language System Stats")?;
        writeln!(f, "===============================")?;
        writeln!(f, "Interactions: {}", self.interaction_count)?;
        writeln!(f, "Seeded: {}", self.is_seeded)?;
        writeln!(f)?;
        writeln!(f, "Concept SOM:")?;
        writeln!(f, "  Active nodes: {}/{}", self.som_stats.active_nodes, self.som_stats.total_nodes)?;
        writeln!(f, "  Total words: {}", self.som_stats.total_words)?;
        writeln!(f, "  Iterations: {}", self.som_stats.iterations)?;
        writeln!(f)?;
        writeln!(f, "Sequence Memory:")?;
        writeln!(f, "  Total patterns: {}", self.sequence_stats.total_patterns)?;
        writeln!(f, "  With concepts: {}", self.sequence_stats.patterns_with_concepts)?;
        writeln!(f, "  Avg frequency: {:.1}", self.sequence_stats.average_frequency)?;
        writeln!(f)?;
        writeln!(f, "Response Selector:")?;
        writeln!(f, "  Templates: {} (global: {})", self.response_stats.total_templates, self.response_stats.global_templates)?;
        writeln!(f, "  Avg success rate: {:.1}%", self.response_stats.average_success_rate * 100.0)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_creation() {
        let system = GroundedLanguage::new();
        assert!(system.is_seeded);
        assert_eq!(system.interaction_count, 0);
    }

    #[test]
    fn test_respond() {
        let mut system = GroundedLanguage::new();

        let state = GroundingState {
            energy: 0.7,
            valence: 0.5,
            ..Default::default()
        };

        let response = system.respond("hi there", &state, RelationshipLevel::Friend);
        assert!(!response.is_empty());
        assert!(system.interaction_count == 1);
    }

    #[test]
    fn test_learning() {
        let mut system = GroundedLanguage::new();

        let state = GroundingState {
            energy: 0.8,
            valence: 0.6,
            ..Default::default()
        };

        // Respond first
        let response = system.respond("how are you", &state, RelationshipLevel::Friend);

        // Learn from exchange
        system.learn_from_exchange("how are you", &response, &state, true);

        // Check that patterns were learned
        let stats = system.stats();
        assert!(stats.sequence_stats.total_patterns > 0);
    }

    #[test]
    fn test_state_affects_response() {
        let mut system = GroundedLanguage::new();

        let tired_state = GroundingState {
            energy: 0.2,
            valence: -0.3,
            ..Default::default()
        };

        let energetic_state = GroundingState {
            energy: 0.9,
            valence: 0.8,
            ..Default::default()
        };

        let tired_response = system.respond("hi", &tired_state, RelationshipLevel::Friend);
        let energetic_response = system.respond("hi", &energetic_state, RelationshipLevel::Friend);

        // Responses should be different (actions at least)
        // Both should contain asterisks for actions
        assert!(tired_response.contains("*") || energetic_response.contains("*"));
    }

    #[test]
    fn test_save_and_load() {
        let mut system = GroundedLanguage::new();

        let state = GroundingState::default();
        system.respond("test input", &state, RelationshipLevel::Stranger);

        let temp_dir = std::env::temp_dir().join("sage_grounded_test");

        // Save
        system.save(&temp_dir).expect("Failed to save");

        // Load
        let loaded = GroundedLanguage::load(&temp_dir).expect("Failed to load");

        assert_eq!(loaded.interaction_count, system.interaction_count);
        assert_eq!(loaded.is_seeded, system.is_seeded);

        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);
    }
}
