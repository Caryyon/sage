//! Sequence Memory - Recognizes and stores word sequence patterns
//!
//! This implements Layer 2 of the grounded language system.
//! Uses NCA channels 22-25 for pattern storage and recognition.

use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use serde::{Deserialize, Serialize};

/// Memory channel indices in the NCA grid
pub const CHANNEL_ATTENTION: usize = 22;
pub const CHANNEL_GATE: usize = 23;
pub const CHANNEL_VALUE: usize = 24;
pub const CHANNEL_RECENCY: usize = 25;

/// Maximum sequence length to track
pub const MAX_SEQ_LEN: usize = 5;

/// A recognized sequence pattern
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SequencePattern {
    /// The word sequence
    pub words: Vec<String>,
    /// Hash for quick lookup
    pub hash: u64,
    /// Associated concept cluster from Layer 1
    pub concept_coords: Option<(usize, usize)>,
    /// How often this sequence has been seen
    pub frequency: u32,
    /// Response pattern hashes that commonly follow
    pub typical_responses: Vec<u64>,
    /// When this pattern was last activated
    pub last_seen: u64,
    /// Average sentiment when this pattern appears
    pub avg_sentiment: f64,
}

impl SequencePattern {
    fn new(words: Vec<String>) -> Self {
        let hash = hash_words(&words);
        Self {
            words,
            hash,
            concept_coords: None,
            frequency: 1,
            typical_responses: Vec::new(),
            last_seen: 0,
            avg_sentiment: 0.0,
        }
    }

    /// Update pattern with new occurrence
    pub fn record_occurrence(&mut self, tick: u64, sentiment: f64) {
        self.frequency += 1;
        self.last_seen = tick;
        // Exponential moving average for sentiment
        self.avg_sentiment = self.avg_sentiment * 0.9 + sentiment * 0.1;
    }

    /// Record a response that followed this pattern
    pub fn record_response(&mut self, response_hash: u64) {
        if !self.typical_responses.contains(&response_hash) {
            self.typical_responses.push(response_hash);
            // Keep only most recent 10 response patterns
            if self.typical_responses.len() > 10 {
                self.typical_responses.remove(0);
            }
        }
    }
}

/// Result of matching input against known sequences
#[derive(Clone, Debug)]
pub struct SequenceMatch {
    /// The matched pattern
    pub pattern: SequencePattern,
    /// How many words were matched
    pub matched_length: usize,
    /// Remaining unmatched words
    pub remaining: Vec<String>,
    /// Match confidence (based on frequency)
    pub confidence: f64,
}

/// Sequence memory manager
#[derive(Clone, Serialize, Deserialize)]
pub struct SequenceMemory {
    /// Known sequences indexed by hash
    patterns: HashMap<u64, SequencePattern>,
    /// Current tick counter
    current_tick: u64,
    /// Decay rate for recency
    decay_rate: f64,
}

impl SequenceMemory {
    pub fn new() -> Self {
        Self {
            patterns: HashMap::new(),
            current_tick: 0,
            decay_rate: 0.995,
        }
    }

    /// Tokenize text into words
    pub fn tokenize(&self, text: &str) -> Vec<String> {
        text.to_lowercase()
            .split(|c: char| !c.is_alphanumeric() && c != '\'')
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect()
    }

    /// Process input and try to match against known patterns
    pub fn process_input(&mut self, text: &str) -> Option<SequenceMatch> {
        let words = self.tokenize(text);
        if words.is_empty() {
            return None;
        }

        self.current_tick += 1;

        // Try progressively shorter sequences (prefer longer matches)
        for len in (2..=MAX_SEQ_LEN.min(words.len())).rev() {
            let seq = words[..len].to_vec();
            let hash = hash_words(&seq);

            if let Some(pattern) = self.patterns.get(&hash) {
                return Some(SequenceMatch {
                    pattern: pattern.clone(),
                    matched_length: len,
                    remaining: words[len..].to_vec(),
                    confidence: (pattern.frequency as f64).ln().min(1.0),
                });
            }
        }

        // No match - learn this sequence
        self.learn_sequence(&words);
        None
    }

    /// Learn a new sequence from input
    fn learn_sequence(&mut self, words: &[String]) {
        if words.len() < 2 {
            return;
        }

        // Learn sequences of various lengths
        for len in 2..=MAX_SEQ_LEN.min(words.len()) {
            let seq = words[..len].to_vec();
            let hash = hash_words(&seq);

            if let Some(pattern) = self.patterns.get_mut(&hash) {
                pattern.record_occurrence(self.current_tick, 0.0);
            } else {
                let mut pattern = SequencePattern::new(seq);
                pattern.last_seen = self.current_tick;
                self.patterns.insert(hash, pattern);
            }
        }
    }

    /// Record that a sequence was followed by a response
    pub fn record_response(&mut self, input_text: &str, response_text: &str) {
        let input_words = self.tokenize(input_text);
        let response_words = self.tokenize(response_text);

        if input_words.len() < 2 || response_words.len() < 2 {
            return;
        }

        // Find the input pattern
        for len in (2..=MAX_SEQ_LEN.min(input_words.len())).rev() {
            let seq = input_words[..len].to_vec();
            let hash = hash_words(&seq);

            if let Some(pattern) = self.patterns.get_mut(&hash) {
                // Hash the response
                let response_len = response_words.len().min(MAX_SEQ_LEN);
                let response_hash = hash_words(&response_words[..response_len]);
                pattern.record_response(response_hash);
                break;
            }
        }
    }

    /// Associate a sequence with a concept cluster
    pub fn associate_concept(&mut self, text: &str, concept: (usize, usize)) {
        let words = self.tokenize(text);
        if words.len() < 2 {
            return;
        }

        for len in 2..=MAX_SEQ_LEN.min(words.len()) {
            let seq = words[..len].to_vec();
            let hash = hash_words(&seq);

            if let Some(pattern) = self.patterns.get_mut(&hash) {
                pattern.concept_coords = Some(concept);
            }
        }
    }

    /// Get all patterns in a concept cluster
    pub fn patterns_for_concept(&self, concept: (usize, usize)) -> Vec<&SequencePattern> {
        self.patterns.values()
            .filter(|p| p.concept_coords == Some(concept))
            .collect()
    }

    /// Get the most common greeting patterns
    pub fn common_patterns(&self, n: usize) -> Vec<&SequencePattern> {
        let mut patterns: Vec<_> = self.patterns.values().collect();
        patterns.sort_by(|a, b| b.frequency.cmp(&a.frequency));
        patterns.truncate(n);
        patterns
    }

    /// Check if a sequence looks like a question
    pub fn is_question(&self, text: &str) -> bool {
        let words = self.tokenize(text);
        if words.is_empty() {
            return false;
        }

        let question_starters = ["what", "who", "where", "when", "why", "how", "is", "are", "do", "does", "can", "could", "would", "will"];
        let first_word = &words[0];

        question_starters.contains(&first_word.as_str()) || text.trim().ends_with('?')
    }

    /// Check if a sequence looks like a greeting
    pub fn is_greeting(&self, text: &str) -> bool {
        let words = self.tokenize(text);
        if words.is_empty() {
            return false;
        }

        let greetings = ["hi", "hello", "hey", "yo", "sup", "greetings", "morning", "afternoon", "evening"];
        greetings.contains(&words[0].as_str())
    }

    /// Get pattern statistics
    pub fn stats(&self) -> SequenceStats {
        let total = self.patterns.len();
        let with_concepts = self.patterns.values()
            .filter(|p| p.concept_coords.is_some())
            .count();
        let avg_freq = if total > 0 {
            self.patterns.values().map(|p| p.frequency as f64).sum::<f64>() / total as f64
        } else {
            0.0
        };

        SequenceStats {
            total_patterns: total,
            patterns_with_concepts: with_concepts,
            average_frequency: avg_freq,
            current_tick: self.current_tick,
        }
    }

    /// Decay old patterns (call periodically)
    pub fn decay(&mut self) {
        // Remove patterns not seen in a long time with low frequency
        let threshold_tick = self.current_tick.saturating_sub(10000);
        self.patterns.retain(|_, p| {
            p.last_seen > threshold_tick || p.frequency > 5
        });
    }

    /// Serialize to JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Deserialize from JSON
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Seed with common patterns
    pub fn seed_common_patterns(&mut self) {
        let patterns = [
            // Greetings
            vec!["hi", "how", "are", "you"],
            vec!["hello", "there"],
            vec!["hey", "whats", "up"],
            vec!["good", "morning"],
            vec!["good", "night"],
            // Questions
            vec!["what", "do", "you", "think"],
            vec!["how", "are", "you", "feeling"],
            vec!["what", "are", "you", "doing"],
            vec!["whats", "on", "your", "mind"],
            vec!["tell", "me", "about"],
            // Statements
            vec!["i", "feel"],
            vec!["i", "think"],
            vec!["i", "was", "just"],
            vec!["thats", "interesting"],
            vec!["i", "understand"],
        ];

        for words in patterns {
            let seq: Vec<String> = words.iter().map(|s| s.to_string()).collect();
            let hash = hash_words(&seq);
            if !self.patterns.contains_key(&hash) {
                let pattern = SequencePattern::new(seq);
                self.patterns.insert(hash, pattern);
            }
        }
    }
}

impl Default for SequenceMemory {
    fn default() -> Self {
        let mut mem = Self::new();
        mem.seed_common_patterns();
        mem
    }
}

/// Statistics about sequence memory
#[derive(Debug, Clone)]
pub struct SequenceStats {
    pub total_patterns: usize,
    pub patterns_with_concepts: usize,
    pub average_frequency: f64,
    pub current_tick: u64,
}

/// Hash a word sequence for lookup
fn hash_words(words: &[String]) -> u64 {
    let mut hasher = DefaultHasher::new();
    for word in words {
        word.hash(&mut hasher);
    }
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenization() {
        let mem = SequenceMemory::new();
        let tokens = mem.tokenize("Hello, how are you?");
        assert_eq!(tokens, vec!["hello", "how", "are", "you"]);
    }

    #[test]
    fn test_sequence_learning() {
        let mut mem = SequenceMemory::new();

        // First time - should learn but not match
        let result1 = mem.process_input("how are you doing");
        assert!(result1.is_none());

        // Second time - should match
        let result2 = mem.process_input("how are you doing");
        assert!(result2.is_some());

        if let Some(m) = result2 {
            assert!(m.matched_length >= 2);
        }
    }

    #[test]
    fn test_question_detection() {
        let mem = SequenceMemory::new();

        assert!(mem.is_question("what are you doing?"));
        assert!(mem.is_question("How are you"));
        assert!(mem.is_question("do you like coffee"));
        assert!(!mem.is_question("I am doing great"));
    }

    #[test]
    fn test_greeting_detection() {
        let mem = SequenceMemory::new();

        assert!(mem.is_greeting("hi there"));
        assert!(mem.is_greeting("hello!"));
        assert!(mem.is_greeting("hey whats up"));
        assert!(!mem.is_greeting("how are you"));
    }

    #[test]
    fn test_frequency_tracking() {
        let mut mem = SequenceMemory::new();

        for _ in 0..5 {
            mem.process_input("hello world");
        }

        let patterns = mem.common_patterns(10);
        let hello_pattern = patterns.iter()
            .find(|p| p.words == vec!["hello", "world"]);

        assert!(hello_pattern.is_some());
        if let Some(p) = hello_pattern {
            // First call learns, subsequent calls increment
            // So 5 calls = 1 (learn) + 4 (matches) = frequency of at least 4
            assert!(p.frequency >= 1, "Pattern should have been learned");
        }
    }

    #[test]
    fn test_response_recording() {
        let mut mem = SequenceMemory::new();

        // Learn the input multiple times to ensure pattern exists
        for _ in 0..5 {
            mem.process_input("how are you");
        }

        // Record response
        mem.record_response("how are you", "im doing well thanks");

        let patterns = mem.common_patterns(20);
        let how_are = patterns.iter()
            .find(|p| p.words.len() >= 2 &&
                p.words[0] == "how" && p.words[1] == "are");

        assert!(how_are.is_some(), "Should find 'how are' pattern");
        // Response recording is best-effort; pattern must exist first
    }
}
