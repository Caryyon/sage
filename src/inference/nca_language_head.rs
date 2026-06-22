//! NCA-Native Language Head
//!
//! A fully self-contained language model powered entirely by Neural Cellular
//! Automata dynamics. No external LLM, no API keys, no cloud, no downloads.
//!
//! Wraps NcaPredictor as an InferenceEngine so it can plug directly into
//! the specialist worker loop, chat TUI, and API server.
//!
//! Architecture:
//!   Text → SimpleTokenizer → token IDs → grid cell activation
//!   → NCA update steps (3-layer MLP per cell, local neighborhood)
//!   → activation readout → softmax → next token prediction
//!   → auto-regressive generation
//!
//! Training:
//!   - CMA-ES evolution strategy (population-based, no gradients needed)
//!   - Backpropagation through unrolled NCA steps (Adam optimizer)
//!   - Trained weights saved to ~/.sage/language_head.bin
//!
//! The predictor specializes to its domain. A React specialist only needs
//! to generate React code — vocabulary of ~3,000 tokens, not all of English.
//! This makes NCA-native generation practical for focused specialist roles.

use crate::inference::nca_predictor::{
    NcaPredictor, NcaWeights, SimpleTokenizer, DEFAULT_STEPS, NCA_GRID_SIZE,
};
use crate::inference::{ChatMessage, ChatRole, InferenceEngine};
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

/// Default path for trained language head weights
pub fn default_language_head_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_default()
        .join(".sage")
        .join("language_head.bin")
}

/// Default path for the tokenizer vocabulary
pub fn default_vocab_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_default()
        .join(".sage")
        .join("language_head.vocab")
}

/// The NCA-native language head — implements InferenceEngine
pub struct NcaLanguageHead {
    predictor: Mutex<NcaPredictor>,
    name_str: String,
    vocab_size: usize,
    trained: bool,
}

impl NcaLanguageHead {
    /// Create a new language head with random weights (untrained).
    /// Use `train_on_corpus()` or `load_trained()` before generating.
    pub fn new() -> Self {
        let tokenizer = SimpleTokenizer::from_corpus("", 1000);
        let weights = NcaWeights::random();
        let predictor = NcaPredictor::with_default_steps(tokenizer, weights);

        Self {
            predictor: Mutex::new(predictor),
            name_str: "NCA-native (untrained)".to_string(),
            vocab_size: 0,
            trained: false,
        }
    }

    /// Create from a trained weights file and vocabulary file
    pub fn load_trained(
        weights_path: Option<&Path>,
        vocab_path: Option<&Path>,
    ) -> Result<Self, Box<dyn Error>> {
        let wp = weights_path.map(|p| p.to_path_buf()).unwrap_or_else(default_language_head_path);
        let vp = vocab_path.map(|p| p.to_path_buf()).unwrap_or_else(default_vocab_path);

        let weights = NcaWeights::load(&wp)?;

        // Load vocabulary from file
        let vocab_text = fs::read_to_string(&vp)?;
        let tokenizer = SimpleTokenizer::from_corpus(&vocab_text, 5000);
        let vocab_size = tokenizer.vocab_size();

        // Load training config if available (grid_size, steps)
        let cp = wp.with_extension("json");
        let (grid_size, steps) = if let Ok(cfg_text) = fs::read_to_string(&cp) {
            if let Ok(cfg) = serde_json::from_str::<serde_json::Value>(&cfg_text) {
                let gs = cfg.get("grid_size").and_then(|v| v.as_u64()).unwrap_or(4) as usize;
                let st = cfg.get("steps").and_then(|v| v.as_u64()).unwrap_or(1) as usize;
                (gs, st)
            } else {
                (4, 1)
            }
        } else {
            (4, 1) // default to training config
        };

        let predictor = NcaPredictor::with_grid_size(tokenizer, weights, steps, grid_size);

        Ok(Self {
            predictor: Mutex::new(predictor),
            name_str: format!("NCA-native ({} tokens, trained)", vocab_size),
            vocab_size,
            trained: true,
        })
    }

    /// Train the language head on a corpus of text.
    /// Builds vocabulary from the corpus, then trains weights via CMA-ES.
    pub fn train_on_corpus(
        &mut self,
        corpus: &str,
        max_vocab: usize,
        epochs: usize,
        _population_size: usize,
    ) -> Result<(), Box<dyn Error>> {
        if corpus.trim().is_empty() {
            return Err("Empty corpus — cannot train".into());
        }

        // Build tokenizer from corpus
        let tokenizer = SimpleTokenizer::from_corpus(corpus, max_vocab);
        let vocab_size = tokenizer.vocab_size();

        if vocab_size < 10 {
            return Err(format!(
                "Corpus too small — only {} unique tokens. Need at least 10.",
                vocab_size
            )
            .into());
        }

        // Create training examples: consecutive token pairs
        let all_tokens = tokenizer.encode(corpus);
        let examples: Vec<(Vec<usize>, usize)> = all_tokens
            .windows(2)
            .map(|w| (vec![w[0]], w[1]))
            .collect();

        if examples.len() < 50 {
            return Err(format!(
                "Corpus too short — only {} token pairs. Need at least 50.",
                examples.len()
            )
            .into());
        }

        // Ultra-minimal config for fast training proof
        let grid_size = 4; // 4×4 = 16 cells — trains in seconds
        let config = crate::inference::backprop_trainer::BackpropConfig {
            learning_rate: 0.01,
            epochs: epochs.min(5),
            grad_clip: 1.0,
            nca_steps: 1,
            grid_size,
            context_window: 1,
            max_examples: examples.len().min(30),
            lr_decay: false,
        };

        let (trained_predictor, accuracy, random_accuracy) =
            crate::inference::backprop_trainer::train_nca_backprop(
                corpus,
                &config,
                true,
            )?;

        // Save trained weights and vocabulary
        let wp = default_language_head_path();
        let vp = default_vocab_path();
        trained_predictor.weights().save(&wp)?;
        fs::write(&vp, corpus)?;

        // Save training config for correct loading
        let cp = wp.with_extension("json");
        let cfg = serde_json::json!({
            "grid_size": grid_size,
            "steps": config.nca_steps,
        });
        fs::write(&cp, cfg.to_string())?;

        // Update predictor with trained weights
        let trained_weights = trained_predictor.weights().clone();
        let predictor = NcaPredictor::with_grid_size(
            tokenizer.clone(),
            trained_weights,
            DEFAULT_STEPS,
            NCA_GRID_SIZE,
        );

        self.predictor = Mutex::new(predictor);
        self.name_str = format!(
            "NCA-native ({} tokens, {:.1}% accuracy vs {:.1}% random)",
            vocab_size,
            accuracy * 100.0,
            random_accuracy * 100.0
        );
        self.vocab_size = vocab_size;
        self.trained = true;

        Ok(())
    }

    /// Check if the language head has been trained
    pub fn is_trained(&self) -> bool {
        self.trained
    }

    /// Get vocabulary size
    pub fn vocab_size(&self) -> usize {
        self.vocab_size
    }

    /// Get the predictor's tokenizer (for external training)
    pub fn tokenizer(&self) -> SimpleTokenizer {
        self.predictor.lock().unwrap().tokenizer.clone()
    }
}

impl InferenceEngine for NcaLanguageHead {
    fn generate(&self, prompt: &str, max_tokens: usize) -> Result<String, Box<dyn Error>> {
        if !self.trained {
            return Err("NCA language head is not trained. Run `sage train language-head` first.".into());
        }

        let mut predictor = self.predictor.lock().unwrap();
        predictor.answer(prompt, None, max_tokens)
    }

    fn chat(&self, messages: &[ChatMessage], max_tokens: usize) -> Result<String, Box<dyn Error>> {
        if !self.trained {
            return Err("NCA language head is not trained. Run `sage train language-head` first.".into());
        }

        // Extract the last user message as the query
        let user_msg = messages
            .iter()
            .filter(|m| m.role == ChatRole::User)
            .last()
            .map(|m| m.content.as_str())
            .unwrap_or("");

        // Extract system prompt as context
        let system_ctx = messages
            .iter()
            .filter(|m| m.role == ChatRole::System)
            .last()
            .map(|m| m.content.as_str());

        let mut predictor = self.predictor.lock().unwrap();
        predictor.answer(user_msg, system_ctx, max_tokens)
    }

    fn generate_streaming(
        &self,
        prompt: &str,
        max_tokens: usize,
        mut callback: Box<dyn FnMut(&str) + Send>,
    ) -> Result<(), Box<dyn Error>> {
        // NCA predictor generates all at once — stream the result word by word
        let response = self.generate(prompt, max_tokens)?;
        for word in response.split_whitespace() {
            callback(word);
            callback(" ");
        }
        Ok(())
    }

    fn chat_streaming(
        &self,
        messages: &[ChatMessage],
        max_tokens: usize,
        mut callback: Box<dyn FnMut(&str) + Send>,
    ) -> Result<(), Box<dyn Error>> {
        let response = self.chat(messages, max_tokens)?;
        for word in response.split_whitespace() {
            callback(word);
            callback(" ");
        }
        Ok(())
    }

    fn name(&self) -> &str {
        &self.name_str
    }

    fn is_available(&self) -> bool {
        self.trained
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_untrained_head_reports_unavailable() {
        let head = NcaLanguageHead::new();
        assert!(!head.is_available());
        assert!(!head.is_trained());
        assert_eq!(head.vocab_size(), 0);
    }

    #[test]
    #[ignore = "slow: NCA predictor uses 181×181 grid internally, takes 60s+"]
    fn test_train_on_small_corpus() {
        let corpus = "hello world sage test alpha beta gamma delta epsilon zeta hello world sage test alpha beta gamma delta epsilon zeta hello world sage test alpha beta gamma delta epsilon zeta hello world sage test alpha beta gamma delta epsilon zeta hello world sage test alpha beta gamma delta epsilon zeta hello world sage test alpha beta gamma delta epsilon zeta hello world sage test alpha beta gamma delta epsilon zeta hello world sage test alpha beta gamma delta epsilon zeta";
        let mut head = NcaLanguageHead::new();

        let result = head.train_on_corpus(corpus, 100, 2, 5);
        assert!(result.is_ok(), "Training should succeed: {:?}", result.err());

        assert!(head.is_trained());
        assert!(head.is_available());
        assert!(head.vocab_size() >= 10, "Should have at least 10 tokens");

        // Try generating
        let response = head.generate("hello", 10);
        assert!(response.is_ok(), "Generation should succeed: {:?}", response.err());
        let text = response.unwrap();
        assert!(!text.is_empty(), "Should produce non-empty output");
        eprintln!("NCA generated: '{}'", text);
    }

    #[test]
    #[ignore = "slow: NCA predictor uses 181×181 grid internally, takes 60s+"]
    fn test_chat_with_system_context() {
        let corpus = "react component state props hook useState useEffect render jsx form input button submit validation react component state props hook useState useEffect render jsx form input button submit validation react component state props hook useState useEffect render jsx form input button submit validation react component state props hook useState useEffect render jsx form input button submit validation react component state props hook useState useEffect render jsx form input button submit validation react component state props hook useState useEffect render jsx form input button submit validation";
        let mut head = NcaLanguageHead::new();
        head.train_on_corpus(corpus, 50, 2, 5).unwrap();

        let messages = vec![
            ChatMessage {
                role: ChatRole::System,
                content: "You are a React developer. Use React terminology.".to_string(),
            },
            ChatMessage {
                role: ChatRole::User,
                content: "form validation".to_string(),
            },
        ];

        let response = head.chat(&messages, 20);
        assert!(response.is_ok(), "Chat should succeed: {:?}", response.err());
        eprintln!("NCA chat response: '{}'", response.unwrap());
    }
}
