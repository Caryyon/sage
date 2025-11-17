// Curiosity System - Makes SAGE ask questions and explore concepts
// Based on NCA loss patterns: medium loss = curiosity zone!

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CuriosityEntry {
    pub concept: String,
    pub nca_loss: f64,
    pub fractal_dimension: f64,  // Pattern complexity (1.0-2.0)
    pub question_asked: String,
    pub question_count: usize,
    pub last_asked: u64,  // generation timestamp
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QuestionType {
    Clarification,  // "What exactly do you mean by...?"
    Elaboration,    // "Can you tell me more about...?"
    Example,        // "Can you give me an example of...?"
    Comparison,     // "How is X different from Y?"
    Relationship,   // "How does X relate to Y?"
    Purpose,        // "Why is X important?"
}

pub struct CuriosityEngine {
    /// Things SAGE is curious about
    curious_concepts: HashMap<String, CuriosityEntry>,

    /// Curiosity thresholds based on NCA loss
    curiosity_min: f64,  // Below this = already understand
    curiosity_max: f64,  // Above this = too unfamiliar

    /// Max questions per concept before stopping
    max_questions_per_concept: usize,

    /// Cooldown period (generations) before asking same question again
    question_cooldown: u64,
}

impl CuriosityEngine {
    pub fn new() -> Self {
        Self {
            curious_concepts: HashMap::new(),
            curiosity_min: 0.15,  // Loss below this = already understand well
            curiosity_max: 0.28,  // Loss above this = too confusing to ask about yet
            max_questions_per_concept: 3,
            question_cooldown: 20,
        }
    }

    /// Check if SAGE should be curious about a concept based on NCA loss
    pub fn is_curious(&self, nca_loss: f64) -> bool {
        nca_loss >= self.curiosity_min && nca_loss <= self.curiosity_max
    }

    /// Record curiosity about a concept
    pub fn record_curiosity(&mut self, concept: String, nca_loss: f64, fractal_dim: f64, generation: u64) -> Option<String> {
        // Check if we're in the curiosity zone
        if !self.is_curious(nca_loss) {
            return None;
        }

        // Check if we already asked about this recently
        if let Some(entry) = self.curious_concepts.get(&concept) {
            if generation - entry.last_asked < self.question_cooldown {
                return None;  // Too soon to ask again
            }

            if entry.question_count >= self.max_questions_per_concept {
                return None;  // Asked enough times already
            }
        }

        // Generate a question!
        let question = self.generate_question(&concept, nca_loss);

        // Update or insert curiosity entry
        let question_count = self.curious_concepts
            .get(&concept)
            .map(|e| e.question_count + 1)
            .unwrap_or(1);

        self.curious_concepts.insert(
            concept.clone(),
            CuriosityEntry {
                concept: concept.clone(),
                nca_loss,
                fractal_dimension: fractal_dim,
                question_asked: question.clone(),
                question_count,
                last_asked: generation,
            },
        );

        Some(question)
    }

    /// Generate an appropriate question based on NCA loss
    fn generate_question(&self, concept: &str, nca_loss: f64) -> String {
        // Different questions based on how curious (loss level)
        if nca_loss < 0.18 {
            // Slight curiosity - ask for elaboration
            self.generate_elaboration_question(concept)
        } else if nca_loss < 0.23 {
            // Moderate curiosity - ask for examples or comparison
            if rand::random::<bool>() {
                self.generate_example_question(concept)
            } else {
                self.generate_purpose_question(concept)
            }
        } else {
            // High curiosity - ask for clarification
            self.generate_clarification_question(concept)
        }
    }

    fn generate_clarification_question(&self, concept: &str) -> String {
        let questions = vec![
            format!("What exactly do you mean by '{}'?", concept),
            format!("I'm not quite sure I understand '{}'. Can you explain?", concept),
            format!("Could you clarify what '{}' means to you?", concept),
            format!("'{}' is new to me. What is it?", concept),
        ];
        questions[rand::random::<usize>() % questions.len()].clone()
    }

    fn generate_elaboration_question(&self, concept: &str) -> String {
        let questions = vec![
            format!("Tell me more about '{}'.", concept),
            format!("I'm interested in '{}'. What else can you share?", concept),
            format!("'{}' intrigues me. Can you elaborate?", concept),
            format!("I want to understand '{}' better. What should I know?", concept),
        ];
        questions[rand::random::<usize>() % questions.len()].clone()
    }

    fn generate_example_question(&self, concept: &str) -> String {
        let questions = vec![
            format!("Can you give me an example of '{}'?", concept),
            format!("What's a concrete example of '{}'?", concept),
            format!("How would '{}' look in practice?", concept),
            format!("Show me what '{}' means with an example?", concept),
        ];
        questions[rand::random::<usize>() % questions.len()].clone()
    }

    fn generate_purpose_question(&self, concept: &str) -> String {
        let questions = vec![
            format!("Why is '{}' important?", concept),
            format!("What's the significance of '{}'?", concept),
            format!("Why should I care about '{}'?", concept),
            format!("What makes '{}' valuable?", concept),
        ];
        questions[rand::random::<usize>() % questions.len()].clone()
    }

    /// Generate a comparison question between two concepts
    pub fn generate_comparison_question(&self, concept_a: &str, concept_b: &str) -> String {
        let questions = vec![
            format!("How is '{}' different from '{}'?", concept_a, concept_b),
            format!("What's the relationship between '{}' and '{}'?", concept_a, concept_b),
            format!("Are '{}' and '{}' similar?", concept_a, concept_b),
            format!("How do '{}' and '{}' connect?", concept_a, concept_b),
        ];
        questions[rand::random::<usize>() % questions.len()].clone()
    }

    /// Get all concepts SAGE is curious about
    pub fn get_curious_concepts(&self) -> Vec<&CuriosityEntry> {
        self.curious_concepts.values().collect()
    }

    /// Get curiosity summary
    pub fn get_curiosity_summary(&self) -> String {
        if self.curious_concepts.is_empty() {
            return "I'm not actively curious about anything right now.".to_string();
        }

        let mut summary = format!("I'm curious about {} things:\n", self.curious_concepts.len());

        for (i, entry) in self.curious_concepts.values().take(5).enumerate() {
            summary.push_str(&format!(
                "{}. {} (asked {} time{})\n",
                i + 1,
                entry.concept,
                entry.question_count,
                if entry.question_count == 1 { "" } else { "s" }
            ));
        }

        summary
    }

    /// Export curiosity data
    pub fn export_curiosity(&self) -> String {
        serde_json::to_string_pretty(&self.curious_concepts).unwrap_or_else(|_| "{}".to_string())
    }

    /// Import curiosity data
    pub fn import_curiosity(&mut self, json: &str) -> Result<(), String> {
        let concepts: HashMap<String, CuriosityEntry> = serde_json::from_str(json)
            .map_err(|e| format!("Failed to parse curiosity data: {}", e))?;

        self.curious_concepts = concepts;
        Ok(())
    }

    /// Should SAGE ask a proactive question now?
    pub fn should_ask_proactive_question(&self, generation: u64) -> bool {
        // Ask proactively every 50 generations if there's something we're curious about
        generation % 50 == 0 && !self.curious_concepts.is_empty()
    }

    /// Get a proactive question to ask
    pub fn get_proactive_question(&self) -> Option<String> {
        // Pick the concept we're most curious about (lowest question count)
        let mut entries: Vec<_> = self.curious_concepts.values().collect();
        entries.sort_by_key(|e| e.question_count);

        entries.first().map(|entry| entry.question_asked.clone())
    }

    /// Get curiosity state for LLM context
    pub fn get_curiosity_state(&self) -> String {
        if self.curious_concepts.is_empty() {
            return "Not actively curious about anything right now.".to_string();
        }

        let mut state = String::new();

        // Get top 3 things SAGE is most curious about
        let mut entries: Vec<_> = self.curious_concepts.values().collect();
        entries.sort_by(|a, b| {
            // Sort by question count (ascending) then by loss (descending)
            a.question_count.cmp(&b.question_count)
                .then(b.nca_loss.partial_cmp(&a.nca_loss).unwrap())
        });

        state.push_str("Currently curious about: ");
        for (i, entry) in entries.iter().take(3).enumerate() {
            if i > 0 {
                state.push_str(", ");
            }
            state.push_str(&format!("{} (confusion: {:.0}%)",
                entry.concept,
                (entry.nca_loss - self.curiosity_min) / (self.curiosity_max - self.curiosity_min) * 100.0
            ));
        }
        state.push('.');

        state
    }

    /// Get knowledge gaps - things SAGE doesn't understand well
    pub fn get_knowledge_gaps(&self) -> Vec<String> {
        let mut entries: Vec<_> = self.curious_concepts.values().collect();
        entries.sort_by(|a, b| b.nca_loss.partial_cmp(&a.nca_loss).unwrap());

        entries.iter()
            .take(3)
            .map(|e| e.concept.clone())
            .collect()
    }

    /// Check if SAGE should explore autonomously (no messages for a while)
    pub fn should_explore_autonomously(&self, idle_count: u64) -> bool {
        // After 10 idle messages, if curious about something, explore
        idle_count > 10 && !self.curious_concepts.is_empty()
    }

    /// Generate an exploratory statement to initiate conversation
    pub fn generate_exploration_prompt(&self) -> Option<String> {
        let gaps = self.get_knowledge_gaps();

        if gaps.is_empty() {
            return None;
        }

        let prompts = vec![
            format!("I've been thinking about {}. What can you tell me?", gaps[0]),
            format!("I'm curious - what do you know about {}?", gaps[0]),
            format!("Can someone explain {} to me?", gaps[0]),
            format!("I want to learn more about {}. Anyone?", gaps[0]),
        ];

        Some(prompts[rand::random::<usize>() % prompts.len()].clone())
    }

    /// Get uncertainty trend - is SAGE getting more or less confused over time?
    pub fn get_uncertainty_level(&self) -> f64 {
        if self.curious_concepts.is_empty() {
            return 0.0;
        }

        let avg_loss: f64 = self.curious_concepts.values()
            .map(|e| e.nca_loss)
            .sum::<f64>() / self.curious_concepts.len() as f64;

        // Normalize to 0-1 range
        (avg_loss - self.curiosity_min) / (self.curiosity_max - self.curiosity_min)
    }

    /// Get most fractal-complex patterns - most interesting for exploration
    pub fn get_most_complex_patterns(&self, limit: usize) -> Vec<(String, f64)> {
        let mut entries: Vec<_> = self.curious_concepts.values().collect();
        // Sort by fractal dimension (descending) - highest complexity first
        entries.sort_by(|a, b| b.fractal_dimension.partial_cmp(&a.fractal_dimension).unwrap());

        entries.iter()
            .take(limit)
            .map(|e| (e.concept.clone(), e.fractal_dimension))
            .collect()
    }

    /// Get fractal complexity score for a concept
    pub fn get_complexity_score(&self, concept: &str) -> Option<f64> {
        self.curious_concepts.get(concept).map(|e| e.fractal_dimension)
    }

    /// Classify patterns by fractal complexity
    pub fn classify_pattern_complexity(&self) -> String {
        if self.curious_concepts.is_empty() {
            return "No patterns to analyze yet.".to_string();
        }

        let avg_dim: f64 = self.curious_concepts.values()
            .map(|e| e.fractal_dimension)
            .sum::<f64>() / self.curious_concepts.len() as f64;

        use crate::fractal_analysis::classify_complexity;
        let classification = classify_complexity(avg_dim);

        format!("Pattern complexity: {} (fractal dimension: {:.2})", classification, avg_dim)
    }
}

impl Default for CuriosityEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_curiosity_zone() {
        let engine = CuriosityEngine::new();

        assert!(!engine.is_curious(0.10));  // Too low - already understand
        assert!(engine.is_curious(0.18));   // Curious!
        assert!(engine.is_curious(0.25));   // Very curious!
        assert!(!engine.is_curious(0.35));  // Too high - too confusing
    }

    #[test]
    fn test_question_generation() {
        let mut engine = CuriosityEngine::new();

        let question = engine.record_curiosity("quantum".to_string(), 0.20, 1.5, 1);
        assert!(question.is_some());

        println!("SAGE asks: {}", question.unwrap());
    }

    #[test]
    fn test_question_cooldown() {
        let mut engine = CuriosityEngine::new();

        // First question
        let q1 = engine.record_curiosity("quantum".to_string(), 0.20, 1.5, 1);
        assert!(q1.is_some());

        // Too soon - no question
        let q2 = engine.record_curiosity("quantum".to_string(), 0.20, 1.5, 5);
        assert!(q2.is_none());

        // After cooldown - new question
        let q3 = engine.record_curiosity("quantum".to_string(), 0.20, 1.5, 25);
        assert!(q3.is_some());
    }
}
