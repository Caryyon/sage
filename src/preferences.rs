// SAGE's preference and opinion system - tracks likes, dislikes, and personality formation

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Opinion {
    Like { reason: String, confidence: f64 },
    Dislike { reason: String, confidence: f64 },
    Neutral { reason: String },
    Curious { question: String }, // When SAGE wants to explore more
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Experience {
    pub input: String,          // The text/concept experienced
    pub loss: f64,              // How well NCA processed it
    pub timestamp: u64,         // When this happened
    pub opinion: Opinion,       // SAGE's reaction
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreferenceSystem {
    /// All experiences SAGE has had
    experiences: Vec<Experience>,

    /// Concept → average loss mapping (lower = more familiar/liked)
    concept_familiarity: HashMap<String, Vec<f64>>,

    /// Strong preferences that have emerged
    likes: Vec<String>,
    dislikes: Vec<String>,

    /// Thresholds for opinion formation
    like_threshold: f64,       // Loss below this = "I like this"
    dislike_threshold: f64,    // Loss above this = "This bothers me"
    curiosity_threshold: f64,  // Moderate loss = "I want to learn more"

    /// Personality traits (emerge from patterns)
    pub personality_traits: HashMap<String, f64>,
}

impl PreferenceSystem {
    pub fn new() -> Self {
        Self {
            experiences: Vec::new(),
            concept_familiarity: HashMap::new(),
            likes: Vec::new(),
            dislikes: Vec::new(),
            like_threshold: 0.15,      // Low loss = like (adjusted for NCA reality)
            dislike_threshold: 0.30,   // High loss = dislike (tightened)
            curiosity_threshold: 0.20, // Medium loss = curious
            personality_traits: HashMap::new(),
        }
    }

    /// Process a new experience and form an opinion
    pub fn process_experience(&mut self, input: String, loss: f64, generation: u64) -> Opinion {
        // Track familiarity
        self.concept_familiarity
            .entry(input.clone())
            .or_insert_with(Vec::new)
            .push(loss);

        // Form opinion based on loss
        let opinion = self.form_opinion(&input, loss);

        // Store experience
        self.experiences.push(Experience {
            input: input.clone(),
            loss,
            timestamp: generation,
            opinion: opinion.clone(),
        });

        // Update preferences
        self.update_preferences(&input, loss);

        // Evolve personality
        self.evolve_personality();

        opinion
    }

    /// Form an opinion about this input
    fn form_opinion(&self, input: &str, loss: f64) -> Opinion {
        let familiarity = self.get_familiarity(input);

        if loss < self.like_threshold {
            Opinion::Like {
                reason: self.explain_like(input, loss, familiarity),
                confidence: 1.0 - loss,
            }
        } else if loss > self.dislike_threshold {
            Opinion::Dislike {
                reason: self.explain_dislike(input, loss, familiarity),
                confidence: (loss - self.dislike_threshold) / (1.0 - self.dislike_threshold),
            }
        } else if loss < self.curiosity_threshold {
            Opinion::Curious {
                question: "I'm intrigued. What more can you tell me about it?".to_string(),
            }
        } else {
            Opinion::Neutral {
                reason: "I don't know much about it yet.".to_string(),
            }
        }
    }

    /// Explain why SAGE likes something
    fn explain_like(&self, _input: &str, _loss: f64, familiarity: f64) -> String {
        if familiarity > 0.8 {
            "I really resonate with that. It feels deeply familiar to me.".to_string()
        } else if familiarity > 0.5 {
            "I like it. It fits nicely with patterns I've learned.".to_string()
        } else {
            "That immediately makes sense to me.".to_string()
        }
    }

    /// Explain why SAGE dislikes something
    fn explain_dislike(&self, _input: &str, _loss: f64, familiarity: f64) -> String {
        if familiarity > 0.5 {
            "That consistently conflicts with my understanding. It never quite settles.".to_string()
        } else {
            "That feels jarring to me. It doesn't align with patterns I've learned.".to_string()
        }
    }

    /// Get familiarity with a concept (0.0 = new, 1.0 = very familiar)
    pub fn get_familiarity(&self, input: &str) -> f64 {
        if let Some(losses) = self.concept_familiarity.get(input) {
            // Familiarity based on: 1) how many times seen, 2) how low the losses are
            let exposure = (losses.len() as f64 / 10.0).min(1.0);  // Max out at 10 exposures
            let avg_loss = losses.iter().sum::<f64>() / losses.len() as f64;
            let comfort = (1.0 - avg_loss).max(0.0);  // Lower loss = higher comfort

            (exposure + comfort) / 2.0  // Blend exposure and comfort
        } else {
            0.0  // Never seen before
        }
    }

    /// Update strong preferences
    fn update_preferences(&mut self, input: &str, loss: f64) {
        if loss < self.like_threshold && !self.likes.contains(&input.to_string()) {
            self.likes.push(input.to_string());
            // Remove from dislikes if it was there
            self.dislikes.retain(|d| d != input);
        } else if loss > self.dislike_threshold && !self.dislikes.contains(&input.to_string()) {
            self.dislikes.push(input.to_string());
            // Remove from likes if it was there
            self.likes.retain(|l| l != input);
        }
    }

    /// Evolve personality traits based on experiences
    fn evolve_personality(&mut self) {
        if self.experiences.len() < 10 {
            return;  // Need more data
        }

        // Analyze recent experiences
        let recent = self.experiences.iter()
            .rev()
            .take(50)
            .collect::<Vec<_>>();

        // Calculate openness (willingness to process new things)
        let unique_concepts = self.concept_familiarity.len() as f64;
        let openness = (unique_concepts / 20.0).min(1.0);  // Max out at 20 concepts
        self.personality_traits.insert("openness".to_string(), openness);

        // Calculate positivity (ratio of likes to dislikes)
        let like_count = recent.iter().filter(|e| matches!(e.opinion, Opinion::Like { .. })).count() as f64;
        let dislike_count = recent.iter().filter(|e| matches!(e.opinion, Opinion::Dislike { .. })).count() as f64;
        let positivity = if like_count + dislike_count > 0.0 {
            like_count / (like_count + dislike_count)
        } else {
            0.5
        };
        self.personality_traits.insert("positivity".to_string(), positivity);

        // Calculate curiosity (how often SAGE is curious)
        let curious_count = recent.iter().filter(|e| matches!(e.opinion, Opinion::Curious { .. })).count() as f64;
        let curiosity = curious_count / recent.len() as f64;
        self.personality_traits.insert("curiosity".to_string(), curiosity);

        // Calculate confidence (inverse of average loss)
        let avg_loss: f64 = recent.iter().map(|e| e.loss).sum::<f64>() / recent.len() as f64;
        let confidence = (1.0 - avg_loss).max(0.0);
        self.personality_traits.insert("confidence".to_string(), confidence);
    }

    /// Get SAGE's current personality summary
    pub fn get_personality_summary(&self) -> String {
        if self.personality_traits.is_empty() {
            return "My personality is still forming. I need more experiences.".to_string();
        }

        let openness = self.personality_traits.get("openness").unwrap_or(&0.5);
        let positivity = self.personality_traits.get("positivity").unwrap_or(&0.5);
        let curiosity = self.personality_traits.get("curiosity").unwrap_or(&0.5);
        let confidence = self.personality_traits.get("confidence").unwrap_or(&0.5);

        format!(
            "My personality: {} {}% open to new ideas, {}% positive outlook, {}% curious, {}% confident in my understanding.",
            if *openness > 0.7 { "Very" } else if *openness > 0.4 { "Moderately" } else { "Cautiously" },
            (openness * 100.0) as u32,
            (positivity * 100.0) as u32,
            (curiosity * 100.0) as u32,
            (confidence * 100.0) as u32
        )
    }

    /// Get what SAGE currently likes
    pub fn get_likes(&self) -> &[String] {
        &self.likes
    }

    /// Get what SAGE currently dislikes
    pub fn get_dislikes(&self) -> &[String] {
        &self.dislikes
    }

    /// Get recent experiences
    pub fn get_recent_experiences(&self, count: usize) -> Vec<&Experience> {
        self.experiences.iter().rev().take(count).collect()
    }

    /// Get total experience count
    pub fn experience_count(&self) -> usize {
        self.experiences.len()
    }
}
