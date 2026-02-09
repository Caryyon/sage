//! Inference engine abstraction
//!
//! Provides a unified trait for LLM inference, with two backends:
//! - `EmbeddedLLM`: Runs a quantized GGUF model in-process via candle (default)
//! - `OllamaEngine`: HTTP client to external Ollama process (optional fallback)

pub mod embedded;
pub mod ollama;
pub mod embeddings;
pub mod distributed;
pub mod nca_predictor;
pub mod reservoir;

use std::error::Error;

/// Unified inference engine trait
pub trait InferenceEngine: Send + Sync {
    /// Generate a complete response for the given prompt
    fn generate(&self, prompt: &str, max_tokens: usize) -> Result<String, Box<dyn Error>>;

    /// Generate with chat messages (system + history)
    fn chat(&self, messages: &[ChatMessage], max_tokens: usize) -> Result<String, Box<dyn Error>>;

    /// Generate streaming tokens via callback
    fn generate_streaming(
        &self,
        prompt: &str,
        max_tokens: usize,
        callback: Box<dyn FnMut(&str) + Send>,
    ) -> Result<(), Box<dyn Error>>;

    /// Chat with streaming
    fn chat_streaming(
        &self,
        messages: &[ChatMessage],
        max_tokens: usize,
        callback: Box<dyn FnMut(&str) + Send>,
    ) -> Result<(), Box<dyn Error>>;

    /// Engine name for display
    fn name(&self) -> &str;

    /// Test if the engine is available
    fn is_available(&self) -> bool;
}

/// Chat message for conversation APIs
#[derive(Clone, Debug)]
pub struct ChatMessage {
    pub role: ChatRole,
    pub content: String,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ChatRole {
    System,
    User,
    Assistant,
}

/// Create the default inference engine.
/// Tries embedded LLM first, falls back to Ollama if available.
pub fn default_engine() -> Box<dyn InferenceEngine> {
    // Try embedded first
    match embedded::EmbeddedLLM::new(None) {
        Ok(engine) => {
            eprintln!("🧠 Using embedded {} (candle)", engine.name());
            return Box::new(engine);
        }
        Err(e) => {
            eprintln!("⚠️  Embedded LLM unavailable: {}", e);
        }
    }

    // Fall back to Ollama
    let ollama = ollama::OllamaEngine::new(None, None);
    if ollama.is_available() {
        eprintln!("🔗 Using {}", ollama.name());
        return Box::new(ollama);
    }

    eprintln!("❌ No inference engine available! Install a model or start Ollama.");
    // Return Ollama anyway (will error on use)
    Box::new(ollama)
}

/// Create an engine with a specific preference
pub fn engine_with_preference(prefer_ollama: bool, model: Option<&str>, ollama_url: Option<&str>) -> Box<dyn InferenceEngine> {
    if prefer_ollama {
        let ollama = ollama::OllamaEngine::new(
            model.map(|s| s.to_string()),
            ollama_url.map(|s| s.to_string()),
        );
        if ollama.is_available() {
            eprintln!("🔗 Using {}", ollama.name());
            return Box::new(ollama);
        }
        eprintln!("⚠️  Ollama not available, trying embedded...");
    }

    match embedded::EmbeddedLLM::new(None) {
        Ok(engine) => {
            eprintln!("🧠 Using embedded {} (candle)", engine.name());
            Box::new(engine)
        }
        Err(e) => {
            eprintln!("❌ No inference engine available: {}", e);
            Box::new(ollama::OllamaEngine::new(
                model.map(|s| s.to_string()),
                ollama_url.map(|s| s.to_string()),
            ))
        }
    }
}
