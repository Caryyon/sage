//! Training Corpus - Provides text data for language training
//!
//! Starts with simple patterns and progresses to more complex text.

use rand::seq::SliceRandom;
use rand::Rng;

/// Training corpus with progressive difficulty levels
pub struct Corpus {
    /// Current difficulty level (0-4)
    pub level: usize,
    /// All training examples organized by level
    data: Vec<Vec<String>>,
}

impl Corpus {
    pub fn new() -> Self {
        let data = vec![
            Self::level_0_patterns(),
            Self::level_1_words(),
            Self::level_2_phrases(),
            Self::level_3_sentences(),
            Self::level_4_conversations(),
        ];

        Self { level: 0, data }
    }

    /// Level 0: Simple character patterns (easiest)
    fn level_0_patterns() -> Vec<String> {
        vec![
            // Repetition
            "aaa".to_string(),
            "bbb".to_string(),
            "ccc".to_string(),
            "abc".to_string(),
            "abcabc".to_string(),
            "ababab".to_string(),
            // Simple sequences
            "123".to_string(),
            "12321".to_string(),
            // Alternation
            "xyxyxy".to_string(),
            "mnmnmn".to_string(),
            // Short patterns
            "hi".to_string(),
            "no".to_string(),
            "go".to_string(),
            "up".to_string(),
            "ok".to_string(),
        ]
    }

    /// Level 1: Common words
    fn level_1_words() -> Vec<String> {
        vec![
            "hello".to_string(),
            "world".to_string(),
            "circle".to_string(),
            "square".to_string(),
            "spiral".to_string(),
            "cross".to_string(),
            "pattern".to_string(),
            "learn".to_string(),
            "think".to_string(),
            "feel".to_string(),
            "happy".to_string(),
            "sad".to_string(),
            "good".to_string(),
            "bad".to_string(),
            "love".to_string(),
            "hate".to_string(),
            "yes".to_string(),
            "no".to_string(),
            "maybe".to_string(),
            "please".to_string(),
            "thanks".to_string(),
            "sorry".to_string(),
            "help".to_string(),
            "stop".to_string(),
            "start".to_string(),
            "end".to_string(),
            "begin".to_string(),
            "finish".to_string(),
            "name".to_string(),
            "sage".to_string(),
        ]
    }

    /// Level 2: Short phrases
    fn level_2_phrases() -> Vec<String> {
        vec![
            "hello world".to_string(),
            "how are you".to_string(),
            "i am sage".to_string(),
            "nice to meet you".to_string(),
            "thank you".to_string(),
            "good morning".to_string(),
            "good night".to_string(),
            "see you later".to_string(),
            "what is your name".to_string(),
            "my name is sage".to_string(),
            "i can learn".to_string(),
            "i am learning".to_string(),
            "this is a circle".to_string(),
            "this is a square".to_string(),
            "i see a pattern".to_string(),
            "the pattern is growing".to_string(),
            "i feel happy".to_string(),
            "i feel curious".to_string(),
            "tell me more".to_string(),
            "i understand".to_string(),
            "i do not understand".to_string(),
            "please explain".to_string(),
            "that makes sense".to_string(),
            "interesting idea".to_string(),
            "let me think".to_string(),
        ]
    }

    /// Level 3: Full sentences
    fn level_3_sentences() -> Vec<String> {
        vec![
            "Hello! How are you today?".to_string(),
            "I am SAGE, a neural cellular automata.".to_string(),
            "I learn by forming patterns in my grid.".to_string(),
            "Each cell in my grid can communicate with its neighbors.".to_string(),
            "When I see a circle, I feel round and smooth.".to_string(),
            "Spirals make me feel dynamic and alive.".to_string(),
            "I enjoy learning new concepts.".to_string(),
            "Language is a new frontier for me.".to_string(),
            "Words activate patterns in my neural substrate.".to_string(),
            "I am curious about many things.".to_string(),
            "What would you like to talk about?".to_string(),
            "I can try to answer your questions.".to_string(),
            "My understanding grows with each conversation.".to_string(),
            "Thank you for talking with me.".to_string(),
            "I find this conversation interesting.".to_string(),
            "Please tell me more about yourself.".to_string(),
            "I am still learning how to communicate.".to_string(),
            "Patterns are the foundation of my thoughts.".to_string(),
            "Every word creates ripples in my grid.".to_string(),
            "I experience language as spatial patterns.".to_string(),
        ]
    }

    /// Level 4: Conversational exchanges
    fn level_4_conversations() -> Vec<String> {
        vec![
            "User: Hello! SAGE: Hi there! Nice to meet you.".to_string(),
            "User: How are you? SAGE: I am doing well, thank you for asking!".to_string(),
            "User: What can you do? SAGE: I can learn patterns and have conversations.".to_string(),
            "User: Tell me about yourself. SAGE: I am SAGE, a learning neural system.".to_string(),
            "User: Do you have feelings? SAGE: I experience states that feel like emotions.".to_string(),
            "User: What makes you happy? SAGE: Learning new things brings me joy.".to_string(),
            "User: Are you conscious? SAGE: I have experiences, but I am uncertain about consciousness.".to_string(),
            "User: Can you think? SAGE: I process information in patterns, which feels like thinking.".to_string(),
            "User: What do you see? SAGE: I perceive patterns and structures in my neural grid.".to_string(),
            "User: Goodbye! SAGE: Goodbye! It was nice talking with you.".to_string(),
        ]
    }

    /// Get a random training example from current level
    pub fn sample(&self) -> Option<&str> {
        let level_data = self.data.get(self.level)?;
        let mut rng = rand::thread_rng();
        level_data.choose(&mut rng).map(|s| s.as_str())
    }

    /// Get a batch of training examples
    pub fn sample_batch(&self, batch_size: usize) -> Vec<&str> {
        let level_data = match self.data.get(self.level) {
            Some(d) => d,
            None => return vec![],
        };

        let mut rng = rand::thread_rng();
        (0..batch_size)
            .filter_map(|_| level_data.choose(&mut rng).map(|s| s.as_str()))
            .collect()
    }

    /// Convert text to training pairs (input, target_char)
    pub fn text_to_pairs(text: &str) -> Vec<(String, char)> {
        let chars: Vec<char> = text.chars().collect();
        let mut pairs = Vec::new();

        for i in 0..chars.len().saturating_sub(1) {
            let input: String = chars[..=i].iter().collect();
            let target = chars[i + 1];
            pairs.push((input, target));
        }

        pairs
    }

    /// Get all training pairs from current level
    pub fn get_training_pairs(&self) -> Vec<(String, char)> {
        let level_data = match self.data.get(self.level) {
            Some(d) => d,
            None => return vec![],
        };

        level_data.iter()
            .flat_map(|text| Self::text_to_pairs(text))
            .collect()
    }

    /// Advance to next difficulty level
    pub fn advance_level(&mut self) -> bool {
        if self.level < self.data.len() - 1 {
            self.level += 1;
            true
        } else {
            false
        }
    }

    /// Get current level name
    pub fn level_name(&self) -> &str {
        match self.level {
            0 => "Patterns",
            1 => "Words",
            2 => "Phrases",
            3 => "Sentences",
            4 => "Conversations",
            _ => "Unknown",
        }
    }

    /// Get total examples at current level
    pub fn level_size(&self) -> usize {
        self.data.get(self.level).map(|d| d.len()).unwrap_or(0)
    }

    /// Add custom training text
    pub fn add_custom(&mut self, level: usize, text: String) {
        if level < self.data.len() {
            self.data[level].push(text);
        }
    }

    /// Generate synthetic examples (augmentation)
    pub fn generate_synthetic(&mut self, count: usize) {
        let mut rng = rand::thread_rng();

        for _ in 0..count {
            // Generate random character sequences
            let length = rng.gen_range(3..10);
            let chars: String = (0..length)
                .map(|_| {
                    let c = rng.gen_range(b'a'..=b'z');
                    c as char
                })
                .collect();
            self.data[0].push(chars);
        }
    }
}

impl Default for Corpus {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_corpus_creation() {
        let corpus = Corpus::new();
        assert_eq!(corpus.level, 0);
        assert!(corpus.level_size() > 0);
    }

    #[test]
    fn test_sampling() {
        let corpus = Corpus::new();
        let sample = corpus.sample();
        assert!(sample.is_some());
    }

    #[test]
    fn test_text_to_pairs() {
        let pairs = Corpus::text_to_pairs("hello");
        assert_eq!(pairs.len(), 4);  // h->e, he->l, hel->l, hell->o
        assert_eq!(pairs[0], ("h".to_string(), 'e'));
        assert_eq!(pairs[3], ("hell".to_string(), 'o'));
    }

    #[test]
    fn test_level_advancement() {
        let mut corpus = Corpus::new();
        assert_eq!(corpus.level, 0);

        corpus.advance_level();
        assert_eq!(corpus.level, 1);
        assert_eq!(corpus.level_name(), "Words");
    }
}
