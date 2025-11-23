// Attention Mechanisms - Selective focus on what matters most
//
// SAGE has limited cognitive capacity. The attention system prioritizes:
// - Emotionally significant experiences
// - Goal-relevant information
// - Important relationships
// - Novel discoveries
//
// Examples:
// - High emotional intensity → High attention
// - Concept related to current goal → High attention
// - Message from close friend → High attention
// - Completely new concept → High attention

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::emotional_gradients::EmotionalState;

/// Salience score - how much something deserves attention (0.0 to 1.0)
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Salience {
    pub score: f64,
    pub emotional_component: f64,
    pub goal_component: f64,
    pub social_component: f64,
    pub novelty_component: f64,
}

impl Salience {
    pub fn new(
        emotional: f64,
        goal: f64,
        social: f64,
        novelty: f64,
    ) -> Self {
        // Weighted average of components
        let score = (
            emotional * 0.3 +
            goal * 0.3 +
            social * 0.2 +
            novelty * 0.2
        ).clamp(0.0, 1.0);

        Self {
            score,
            emotional_component: emotional,
            goal_component: goal,
            social_component: social,
            novelty_component: novelty,
        }
    }

    /// Combine with another salience (for compound stimuli)
    pub fn combine(&self, other: &Salience) -> Salience {
        Salience::new(
            (self.emotional_component + other.emotional_component) / 2.0,
            (self.goal_component + other.goal_component) / 2.0,
            (self.social_component + other.social_component) / 2.0,
            (self.novelty_component + other.novelty_component) / 2.0,
        )
    }
}

/// Something that can capture attention
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttentionTarget {
    pub id: String,
    pub description: String,
    pub salience: Salience,
    pub timestamp: u64,
}

/// Attention mechanism - manages cognitive focus
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttentionSystem {
    /// Current focus targets (limited capacity)
    focal_attention: Vec<AttentionTarget>,

    /// Peripheral attention (lower priority targets)
    peripheral_attention: Vec<AttentionTarget>,

    /// Attention budget (max focal targets)
    focal_capacity: usize,

    /// Peripheral capacity
    peripheral_capacity: usize,

    /// Minimum salience for focal attention
    focal_threshold: f64,

    /// Minimum salience for peripheral attention
    peripheral_threshold: f64,

    /// Concept familiarity tracking (for novelty detection)
    concept_encounters: HashMap<String, u64>,
}

impl AttentionSystem {
    pub fn new() -> Self {
        Self {
            focal_attention: Vec::new(),
            peripheral_attention: Vec::new(),
            focal_capacity: 3,  // Can focus on ~3 things at once (Miller's law)
            peripheral_capacity: 7,  // Can track ~7 things peripherally
            focal_threshold: 0.6,  // Need high salience for focal attention
            peripheral_threshold: 0.3,  // Lower bar for peripheral
            concept_encounters: HashMap::new(),
        }
    }

    /// Calculate salience for a stimulus
    pub fn calculate_salience(
        &mut self,
        concepts: &[String],
        emotion: Option<&EmotionalState>,
        goal_relevance: f64,  // 0.0 to 1.0, how relevant to current goals
        social_relevance: f64,  // 0.0 to 1.0, related to important people
    ) -> Salience {
        // Emotional component
        let emotional = emotion
            .map(|e| e.intensity * e.valence.abs())
            .unwrap_or(0.0);

        // Novelty component - average novelty across concepts
        let mut novelty_sum = 0.0;
        let mut novelty_count = 0;

        for concept in concepts {
            let encounters = self.concept_encounters
                .entry(concept.clone())
                .or_insert(0);

            *encounters += 1;

            // Novel concepts (first few encounters) get high novelty score
            let concept_novelty = match *encounters {
                1 => 1.0,  // Completely new
                2..=3 => 0.7,  // Still very new
                4..=10 => 0.4,  // Moderately new
                _ => 0.1,  // Familiar
            };

            novelty_sum += concept_novelty;
            novelty_count += 1;
        }

        let novelty = if novelty_count > 0 {
            novelty_sum / novelty_count as f64
        } else {
            0.0
        };

        Salience::new(emotional, goal_relevance, social_relevance, novelty)
    }

    /// Attend to a stimulus (add to attention if salient enough)
    pub fn attend(
        &mut self,
        id: String,
        description: String,
        salience: Salience,
        timestamp: u64,
    ) -> AttentionLevel {
        let target = AttentionTarget {
            id,
            description,
            salience,
            timestamp,
        };

        // Determine attention level based on salience
        if salience.score >= self.focal_threshold {
            // Add to focal attention
            self.focal_attention.push(target);

            // If over capacity, remove least salient
            if self.focal_attention.len() > self.focal_capacity {
                self.focal_attention.sort_by(|a, b| {
                    b.salience.score.partial_cmp(&a.salience.score).unwrap()
                });
                self.focal_attention.truncate(self.focal_capacity);
            }

            AttentionLevel::Focal
        } else if salience.score >= self.peripheral_threshold {
            // Add to peripheral attention
            self.peripheral_attention.push(target);

            // If over capacity, remove least salient
            if self.peripheral_attention.len() > self.peripheral_capacity {
                self.peripheral_attention.sort_by(|a, b| {
                    b.salience.score.partial_cmp(&a.salience.score).unwrap()
                });
                self.peripheral_attention.truncate(self.peripheral_capacity);
            }

            AttentionLevel::Peripheral
        } else {
            // Below threshold - ignored
            AttentionLevel::Ignored
        }
    }

    /// Get current focal targets
    pub fn get_focal_attention(&self) -> Vec<&AttentionTarget> {
        self.focal_attention.iter().collect()
    }

    /// Get current peripheral targets
    pub fn get_peripheral_attention(&self) -> Vec<&AttentionTarget> {
        self.peripheral_attention.iter().collect()
    }

    /// Check if currently attending to something
    pub fn is_attending_to(&self, id: &str) -> Option<AttentionLevel> {
        if self.focal_attention.iter().any(|t| t.id == id) {
            Some(AttentionLevel::Focal)
        } else if self.peripheral_attention.iter().any(|t| t.id == id) {
            Some(AttentionLevel::Peripheral)
        } else {
            None
        }
    }

    /// Clear old attention targets (call periodically)
    pub fn decay_attention(&mut self, current_time: u64, max_age: u64) {
        self.focal_attention.retain(|t| {
            current_time.saturating_sub(t.timestamp) < max_age
        });

        self.peripheral_attention.retain(|t| {
            current_time.saturating_sub(t.timestamp) < max_age
        });
    }

    /// Get attention summary
    pub fn get_summary(&self) -> String {
        let focal_count = self.focal_attention.len();
        let peripheral_count = self.peripheral_attention.len();

        if focal_count == 0 && peripheral_count == 0 {
            "Not focused on anything specific right now.".to_string()
        } else {
            let mut summary = String::new();

            if focal_count > 0 {
                summary.push_str(&format!("Focusing on {} thing{}:\n",
                    focal_count,
                    if focal_count == 1 { "" } else { "s" }
                ));

                for target in &self.focal_attention {
                    summary.push_str(&format!("  • {} (salience: {:.0}%)\n",
                        target.description,
                        target.salience.score * 100.0
                    ));
                }
            }

            if peripheral_count > 0 {
                summary.push_str(&format!("\nAware of {} other thing{}\n",
                    peripheral_count,
                    if peripheral_count == 1 { "" } else { "s" }
                ));
            }

            summary
        }
    }

    /// Get detailed attention breakdown
    pub fn get_detailed_summary(&self) -> String {
        let mut summary = self.get_summary();

        if !self.focal_attention.is_empty() {
            summary.push_str("\nAttention components:\n");

            for target in &self.focal_attention {
                summary.push_str(&format!("  {}:\n", target.description));
                summary.push_str(&format!("    Emotional: {:.0}%\n",
                    target.salience.emotional_component * 100.0));
                summary.push_str(&format!("    Goal relevance: {:.0}%\n",
                    target.salience.goal_component * 100.0));
                summary.push_str(&format!("    Social relevance: {:.0}%\n",
                    target.salience.social_component * 100.0));
                summary.push_str(&format!("    Novelty: {:.0}%\n",
                    target.salience.novelty_component * 100.0));
            }
        }

        summary
    }

    /// Reset familiarity (useful for testing or after long breaks)
    pub fn reset_familiarity(&mut self) {
        self.concept_encounters.clear();
    }
}

impl Default for AttentionSystem {
    fn default() -> Self {
        Self::new()
    }
}

/// Level of attention given to a stimulus
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AttentionLevel {
    /// High priority - in focal attention
    Focal,
    /// Medium priority - in peripheral awareness
    Peripheral,
    /// Low priority - ignored
    Ignored,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_salience_calculation() {
        let salience = Salience::new(0.8, 0.6, 0.4, 0.9);

        // Should weight components appropriately
        assert!(salience.score > 0.5);
        assert!(salience.score <= 1.0);
    }

    #[test]
    fn test_attention_capacity() {
        let mut attention = AttentionSystem::new();

        // Add more targets than capacity
        // Need score >= 0.6 for focal attention. Score = e*0.3 + g*0.3 + s*0.2 + n*0.2
        // Using (1.0, 1.0, 0.0, 0.0) gives score = 0.6
        for i in 0..10 {
            let salience = Salience::new(1.0, 1.0, 0.0, 0.0);  // Score = 0.6 (threshold)
            attention.attend(
                format!("target_{}", i),
                format!("Target {}", i),
                salience,
                i as u64,
            );
        }

        // Should only keep focal_capacity (3) targets
        assert_eq!(attention.focal_attention.len(), 3);

        // Should have highest salience targets
        for target in &attention.focal_attention {
            assert!(target.salience.score >= 0.6);
        }
    }

    #[test]
    fn test_novelty_detection() {
        let mut attention = AttentionSystem::new();

        // First encounter - high novelty
        let salience1 = attention.calculate_salience(
            &["rust".to_string()],
            None,
            0.0,
            0.0,
        );
        assert!(salience1.novelty_component > 0.9);

        // Second encounter - still novel but less
        let salience2 = attention.calculate_salience(
            &["rust".to_string()],
            None,
            0.0,
            0.0,
        );
        assert!(salience2.novelty_component < salience1.novelty_component);

        // Many encounters - familiar
        for _ in 0..20 {
            attention.calculate_salience(&["rust".to_string()], None, 0.0, 0.0);
        }

        let salience_familiar = attention.calculate_salience(
            &["rust".to_string()],
            None,
            0.0,
            0.0,
        );
        assert!(salience_familiar.novelty_component < 0.2);
    }

    #[test]
    fn test_attention_decay() {
        let mut attention = AttentionSystem::new();

        // Need score >= 0.6 for focal attention
        let salience = Salience::new(1.0, 1.0, 0.0, 0.0);  // Score = 0.6
        attention.attend("old".to_string(), "Old target".to_string(), salience.clone(), 0);
        attention.attend("new".to_string(), "New target".to_string(), salience, 1000);

        // Decay old targets
        attention.decay_attention(1100, 200);

        // Old target should be removed, new target should remain
        assert!(attention.is_attending_to("new").is_some());
        assert!(attention.is_attending_to("old").is_none());
    }
}
