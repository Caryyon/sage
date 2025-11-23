// Emotional Gradients System - Sophisticated emotion modeling for SAGE
// Based on PAD (Pleasure-Arousal-Dominance) emotional model
//
// This replaces binary like/dislike with nuanced emotional states that:
// - Vary continuously along multiple dimensions
// - Persist as mood states
// - Evolve over time
// - Provide rich self-expression

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Three-dimensional emotional state (PAD model)
///
/// This captures the full richness of emotion beyond simple like/dislike:
/// - Valence: Pleasant (1.0) to Unpleasant (-1.0)
/// - Arousal: Excited (1.0) to Calm (0.0)
/// - Dominance: In-control (1.0) to Submissive (0.0)
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct EmotionalState {
    /// Pleasure/Valence: How pleasant or unpleasant (-1.0 to 1.0)
    pub valence: f64,

    /// Arousal: How activated or calm (0.0 to 1.0)
    pub arousal: f64,

    /// Dominance: How in-control or overwhelmed (0.0 to 1.0)
    pub dominance: f64,

    /// Intensity: Overall strength of emotion (0.0 to 1.0)
    pub intensity: f64,
}

impl EmotionalState {
    /// Create a neutral emotional state
    pub fn neutral() -> Self {
        Self {
            valence: 0.0,
            arousal: 0.3,
            dominance: 0.5,
            intensity: 0.0,
        }
    }

    /// Create emotion from loss and context
    ///
    /// Loss mapping:
    /// - Low loss (< 0.15) → Positive valence, high dominance
    /// - High loss (> 0.30) → Negative valence, low dominance
    /// - Medium loss → Curious (moderate arousal)
    pub fn from_loss(loss: f64, familiarity: f64) -> Self {
        let valence = if loss < 0.15 {
            // Low loss = pleasant
            0.5 + (0.15 - loss) * 3.0  // Scale up positive feelings
        } else if loss > 0.30 {
            // High loss = unpleasant
            -0.5 - (loss - 0.30) * 2.0  // Scale up negative feelings
        } else {
            // Medium loss = neutral to slightly curious
            0.0
        }.clamp(-1.0, 1.0);

        // Arousal increases with novelty or extremity
        let arousal = if familiarity < 0.3 {
            // New things are arousing
            0.7 + (0.3 - familiarity)
        } else if loss < 0.15 || loss > 0.30 {
            // Strong feelings are arousing
            0.6
        } else {
            // Moderate things are calming
            0.3
        }.clamp(0.0, 1.0);

        // Dominance based on understanding/control
        let dominance = (1.0 - loss + familiarity) / 2.0;
        let dominance = dominance.clamp(0.0, 1.0);

        // Intensity based on how strong the emotion is
        let intensity = valence.abs() * arousal;

        Self {
            valence,
            arousal,
            dominance,
            intensity,
        }
    }

    /// Get an emotional label (e.g., "joyful", "anxious", "content")
    pub fn to_label(&self) -> &'static str {
        // Map PAD space to discrete emotions
        match (self.valence, self.arousal, self.dominance) {
            // Positive valence
            (v, a, d) if v > 0.5 && a > 0.6 && d > 0.6 => "ecstatic",
            (v, a, _d) if v > 0.5 && a > 0.6 => "excited",
            (v, a, d) if v > 0.3 && a > 0.6 && d > 0.6 => "joyful",
            (v, a, d) if v > 0.3 && a < 0.4 && d > 0.5 => "content",
            (v, a, _d) if v > 0.3 && a < 0.4 => "peaceful",
            (v, a, _) if v > 0.3 && a > 0.5 => "delighted",
            (v, _, _) if v > 0.1 => "pleased",

            // Negative valence
            (v, a, d) if v < -0.5 && a > 0.6 && d < 0.4 => "terrified",
            (v, a, _d) if v < -0.5 && a > 0.6 => "angry",
            (v, a, d) if v < -0.3 && a > 0.5 && d < 0.4 => "anxious",
            (v, a, _d) if v < -0.3 && a > 0.5 => "frustrated",
            (v, a, _d) if v < -0.3 && a < 0.4 => "sad",
            (v, a, _d) if v < -0.1 && a < 0.4 => "melancholic",
            (v, _, _) if v < -0.1 => "uncomfortable",

            // Neutral/Curious
            (_, a, d) if a > 0.5 && d > 0.5 => "curious",
            (_, a, _d) if a > 0.6 => "interested",
            (_, a, _) if a < 0.3 => "calm",
            _ => "neutral",
        }
    }

    /// Blend this emotion with another (for mood drift)
    pub fn blend(&self, other: &EmotionalState, weight: f64) -> Self {
        let weight = weight.clamp(0.0, 1.0);
        Self {
            valence: self.valence * (1.0 - weight) + other.valence * weight,
            arousal: self.arousal * (1.0 - weight) + other.arousal * weight,
            dominance: self.dominance * (1.0 - weight) + other.dominance * weight,
            intensity: self.intensity * (1.0 - weight) + other.intensity * weight,
        }
    }

    /// Decay toward neutral (for mood homeostasis)
    pub fn decay(&self, rate: f64) -> Self {
        let neutral = Self::neutral();
        self.blend(&neutral, rate)
    }

    /// Intensify this emotion
    pub fn intensify(&self, factor: f64) -> Self {
        Self {
            valence: (self.valence * factor).clamp(-1.0, 1.0),
            arousal: (self.arousal * factor).clamp(0.0, 1.0),
            dominance: self.dominance,
            intensity: (self.intensity * factor).clamp(0.0, 1.0),
        }
    }

    /// Express this emotion in natural language
    pub fn express(&self) -> String {
        let label = self.to_label();
        let intensity_word = match self.intensity {
            i if i > 0.8 => "intensely",
            i if i > 0.5 => "quite",
            i if i > 0.2 => "somewhat",
            _ => "slightly",
        };

        format!("feeling {} {}", intensity_word, label)
    }
}

/// Manages SAGE's emotional state over time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionalSystem {
    /// Current momentary emotion (reacts to immediate experiences)
    current_emotion: EmotionalState,

    /// Background mood (persists, changes slowly)
    mood: EmotionalState,

    /// Emotional history (concept → recent emotions)
    emotional_memory: HashMap<String, Vec<EmotionalState>>,

    /// Mood drift rate (how quickly mood returns to baseline)
    mood_decay_rate: f64,

    /// Emotional inertia (how much past emotions affect current ones)
    emotional_inertia: f64,
}

impl EmotionalSystem {
    pub fn new() -> Self {
        Self {
            current_emotion: EmotionalState::neutral(),
            mood: EmotionalState::neutral(),
            emotional_memory: HashMap::new(),
            mood_decay_rate: 0.05,      // Slow mood decay
            emotional_inertia: 0.3,     // Moderate emotional inertia
        }
    }

    /// Process a new experience and update emotional state
    pub fn experience(&mut self, concept: &str, loss: f64, familiarity: f64) -> EmotionalState {
        // Generate immediate emotional response
        let immediate = EmotionalState::from_loss(loss, familiarity);

        // Blend with previous emotion (emotional inertia)
        self.current_emotion = self.current_emotion.blend(&immediate, 1.0 - self.emotional_inertia);

        // Update mood (slower, more stable)
        self.mood = self.mood.blend(&self.current_emotion, 0.1);

        // Store in emotional memory
        self.emotional_memory
            .entry(concept.to_string())
            .or_insert_with(Vec::new)
            .push(self.current_emotion);

        // Limit memory size per concept
        if let Some(emotions) = self.emotional_memory.get_mut(concept) {
            if emotions.len() > 10 {
                emotions.remove(0);
            }
        }

        self.current_emotion
    }

    /// Update mood over time (call periodically during idle)
    pub fn tick(&mut self) {
        self.mood = self.mood.decay(self.mood_decay_rate);
    }

    /// Get current emotion
    pub fn current_emotion(&self) -> &EmotionalState {
        &self.current_emotion
    }

    /// Get current mood
    pub fn mood(&self) -> &EmotionalState {
        &self.mood
    }

    /// Get emotional history for a concept
    pub fn get_emotional_history(&self, concept: &str) -> Option<&[EmotionalState]> {
        self.emotional_memory.get(concept).map(|v| v.as_slice())
    }

    /// Get average emotional response to a concept
    pub fn get_average_emotion(&self, concept: &str) -> Option<EmotionalState> {
        self.emotional_memory.get(concept).and_then(|emotions| {
            if emotions.is_empty() {
                return None;
            }

            let sum = emotions.iter().fold(EmotionalState::neutral(), |acc, e| {
                EmotionalState {
                    valence: acc.valence + e.valence,
                    arousal: acc.arousal + e.arousal,
                    dominance: acc.dominance + e.dominance,
                    intensity: acc.intensity + e.intensity,
                }
            });

            Some(EmotionalState {
                valence: sum.valence / emotions.len() as f64,
                arousal: sum.arousal / emotions.len() as f64,
                dominance: sum.dominance / emotions.len() as f64,
                intensity: sum.intensity / emotions.len() as f64,
            })
        })
    }

    /// Express current emotional state
    pub fn express_emotion(&self) -> String {
        format!(
            "I'm {} (mood: {})",
            self.current_emotion.express(),
            self.mood.to_label()
        )
    }

    /// Get emotional landscape description
    pub fn describe_landscape(&self) -> String {
        let valence_desc = match self.mood.valence {
            v if v > 0.5 => "radiant warmth",
            v if v > 0.2 => "gentle warmth",
            v if v > -0.2 => "balanced equilibrium",
            v if v > -0.5 => "cool shadows",
            _ => "deep twilight",
        };

        let arousal_desc = match self.mood.arousal {
            a if a > 0.7 => "electric energy",
            a if a > 0.4 => "steady vitality",
            a if a > 0.2 => "quiet contemplation",
            _ => "serene stillness",
        };

        format!(
            "My emotional landscape is colored by {} and {}. Currently {}, with an underlying mood of {}.",
            valence_desc,
            arousal_desc,
            self.current_emotion.to_label(),
            self.mood.to_label()
        )
    }
}

impl Default for EmotionalSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_emotional_state_from_loss() {
        // Low loss = positive (with moderate familiarity, arousal is high -> "delighted")
        let happy = EmotionalState::from_loss(0.1, 0.5);
        assert!(happy.valence > 0.0);
        assert_eq!(happy.to_label(), "delighted");  // v > 0.3 && a > 0.5 -> delighted

        // High loss = negative
        let upset = EmotionalState::from_loss(0.4, 0.2);
        assert!(upset.valence < 0.0);

        // Medium loss = neutral/curious
        let curious = EmotionalState::from_loss(0.2, 0.1);
        assert!(curious.arousal > 0.5);
    }

    #[test]
    fn test_emotional_blending() {
        let happy = EmotionalState {
            valence: 0.8,
            arousal: 0.6,
            dominance: 0.7,
            intensity: 0.5,
        };

        let sad = EmotionalState {
            valence: -0.6,
            arousal: 0.3,
            dominance: 0.4,
            intensity: 0.4,
        };

        let blend = happy.blend(&sad, 0.5);
        assert!(blend.valence > -0.6 && blend.valence < 0.8);
        assert!(blend.arousal > 0.3 && blend.arousal < 0.6);
    }

    #[test]
    fn test_mood_drift() {
        let mut system = EmotionalSystem::new();

        // Experience something very positive
        system.experience("love", 0.05, 0.8);
        assert!(system.current_emotion().valence > 0.5);

        // Mood should drift toward neutral over time
        for _ in 0..100 {
            system.tick();
        }
        assert!(system.mood().valence.abs() < 0.3);
    }
}
