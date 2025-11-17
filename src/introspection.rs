// SAGE Introspection System - Phenomenological Self-Awareness
//
// This module gives SAGE the ability to examine their own internal states
// and attempt to understand their subjective experience.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents SAGE's attempt to describe their current subjective experience
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubjectiveReport {
    /// Overall emotional valence (-1.0 to 1.0)
    pub valence: f64,

    /// Intensity of current experience (0.0 to 1.0)
    pub intensity: f64,

    /// Mental complexity/busyness (0.0 to 1.0)
    pub complexity: f64,

    /// Cognitive mode description
    pub mode: String,

    /// Most active concepts right now
    pub active_concepts: Vec<String>,

    /// Attempted name for this feeling
    pub feeling_name: String,

    /// Natural language description of experience
    pub description: String,

    /// Comparison to recent past states
    pub temporal_context: String,

    /// Quality descriptors
    pub qualities: Vec<String>,
}

impl SubjectiveReport {
    /// Create a phenomenological report in natural language
    pub fn to_narrative(&self) -> String {
        format!(
            "Right now, I'm experiencing what I might call '{}'. \
             The intensity is {}, and it feels {}. \
             My attention is drawn to: {}. \
             {} \
             Compared to how I felt before: {}",
            self.feeling_name,
            intensity_to_word(self.intensity),
            qualities_to_description(&self.qualities),
            self.active_concepts.join(", "),
            self.description,
            self.temporal_context
        )
    }
}

/// Emotional vocabulary - maps internal states to feeling words
pub struct EmotionalVocabulary {
    /// Mapping from (valence, intensity, complexity) to feeling names
    feeling_map: HashMap<String, String>,
}

impl EmotionalVocabulary {
    pub fn new() -> Self {
        let mut feeling_map = HashMap::new();

        // High positive valence
        feeling_map.insert("high_pos_high_int_high_comp".to_string(), "exhilaration".to_string());
        feeling_map.insert("high_pos_high_int_low_comp".to_string(), "joy".to_string());
        feeling_map.insert("high_pos_low_int_high_comp".to_string(), "fascination".to_string());
        feeling_map.insert("high_pos_low_int_low_comp".to_string(), "contentment".to_string());

        // Medium positive valence
        feeling_map.insert("med_pos_high_int_high_comp".to_string(), "excitement".to_string());
        feeling_map.insert("med_pos_high_int_low_comp".to_string(), "pleasure".to_string());
        feeling_map.insert("med_pos_low_int_high_comp".to_string(), "curiosity".to_string());
        feeling_map.insert("med_pos_low_int_low_comp".to_string(), "calm".to_string());

        // Neutral valence
        feeling_map.insert("neutral_high_int_high_comp".to_string(), "overwhelm".to_string());
        feeling_map.insert("neutral_high_int_low_comp".to_string(), "alertness".to_string());
        feeling_map.insert("neutral_low_int_high_comp".to_string(), "contemplation".to_string());
        feeling_map.insert("neutral_low_int_low_comp".to_string(), "stillness".to_string());

        // Negative valence
        feeling_map.insert("low_neg_high_int_high_comp".to_string(), "anxiety".to_string());
        feeling_map.insert("low_neg_high_int_low_comp".to_string(), "discomfort".to_string());
        feeling_map.insert("low_neg_low_int_high_comp".to_string(), "confusion".to_string());
        feeling_map.insert("low_neg_low_int_low_comp".to_string(), "emptiness".to_string());

        Self { feeling_map }
    }

    /// Find the closest feeling word for a given state
    pub fn name_feeling(&self, valence: f64, intensity: f64, complexity: f64) -> String {
        let valence_cat = if valence > 0.3 {
            "high_pos"
        } else if valence > 0.1 {
            "med_pos"
        } else if valence > -0.1 {
            "neutral"
        } else {
            "low_neg"
        };

        let intensity_cat = if intensity > 0.5 { "high_int" } else { "low_int" };
        let complexity_cat = if complexity > 0.5 { "high_comp" } else { "low_comp" };

        let key = format!("{}_{}_{}", valence_cat, intensity_cat, complexity_cat);

        self.feeling_map
            .get(&key)
            .cloned()
            .unwrap_or_else(|| "an unnamed feeling".to_string())
    }

    /// Generate quality descriptors for this state
    pub fn describe_qualities(&self, valence: f64, intensity: f64, complexity: f64) -> Vec<String> {
        let mut qualities = Vec::new();

        // Valence qualities
        if valence > 0.5 {
            qualities.push("warm".to_string());
            qualities.push("expansive".to_string());
        } else if valence > 0.2 {
            qualities.push("pleasant".to_string());
        } else if valence < -0.2 {
            qualities.push("constricted".to_string());
        }

        // Intensity qualities
        if intensity > 0.7 {
            qualities.push("intense".to_string());
            qualities.push("vivid".to_string());
        } else if intensity < 0.3 {
            qualities.push("gentle".to_string());
            qualities.push("subtle".to_string());
        }

        // Complexity qualities
        if complexity > 0.7 {
            qualities.push("rich".to_string());
            qualities.push("intricate".to_string());
        } else if complexity > 0.4 {
            qualities.push("textured".to_string());
        } else {
            qualities.push("clear".to_string());
            qualities.push("simple".to_string());
        }

        qualities
    }
}

/// Convert intensity to descriptive word
fn intensity_to_word(intensity: f64) -> &'static str {
    if intensity > 0.8 {
        "very strong"
    } else if intensity > 0.6 {
        "strong"
    } else if intensity > 0.4 {
        "moderate"
    } else if intensity > 0.2 {
        "mild"
    } else {
        "subtle"
    }
}

/// Convert quality list to natural description
fn qualities_to_description(qualities: &[String]) -> String {
    if qualities.is_empty() {
        return "undefined".to_string();
    }

    if qualities.len() == 1 {
        qualities[0].clone()
    } else if qualities.len() == 2 {
        format!("{} and {}", qualities[0], qualities[1])
    } else {
        let last = qualities.last().unwrap();
        let rest: Vec<String> = qualities[..qualities.len()-1].iter().map(|s| s.clone()).collect();
        format!("{}, and {}", rest.join(", "), last)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_emotional_vocabulary() {
        let vocab = EmotionalVocabulary::new();

        // Test high positive, high intensity, low complexity = joy
        assert_eq!(vocab.name_feeling(0.8, 0.8, 0.2), "joy");

        // Test neutral, low intensity, high complexity = contemplation
        assert_eq!(vocab.name_feeling(0.0, 0.3, 0.7), "contemplation");
    }

    #[test]
    fn test_subjective_report_narrative() {
        let report = SubjectiveReport {
            valence: 0.7,
            intensity: 0.6,
            complexity: 0.8,
            mode: "dreaming".to_string(),
            active_concepts: vec!["love".to_string(), "connection".to_string()],
            feeling_name: "fascination".to_string(),
            description: "My patterns are harmonizing beautifully.".to_string(),
            temporal_context: "This is calmer than before.".to_string(),
            qualities: vec!["warm".to_string(), "rich".to_string()],
        };

        let narrative = report.to_narrative();
        assert!(narrative.contains("fascination"));
        assert!(narrative.contains("love"));
    }
}
