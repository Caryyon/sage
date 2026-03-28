//! Inference engine abstraction
//!
//! Provides a unified trait for LLM inference, with multiple backends:
//! - `LocalLLM`: Runs GGUF models via llama.cpp (optional, requires local-llm feature)
//! - `EmbeddedLLM`: Runs a quantized GGUF model in-process via candle (default)
//! - `OllamaEngine`: HTTP client to external Ollama process (optional fallback)
//! - `OfflineEngine`: Graceful degradation when no LLM is available (Phase 3 groundwork)

pub mod backprop_trainer;
pub mod criticality;
pub mod distributed;
pub mod embedded;
pub mod embeddings;
pub mod kan;
pub mod local_llm;
pub mod nca_predictor;
pub mod offline;
pub mod ollama;
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

/// Generation backend status for display
#[derive(Clone, Debug)]
pub enum GenerationBackend {
    /// Local llama.cpp model
    Local { model_name: String, model_size: String },
    /// Ollama with model name
    Ollama { model: String },
    /// Embedded SmolLM2 via candle
    Embedded { model_name: String },
    /// Offline mode (retrieval only)
    Offline,
}

impl std::fmt::Display for GenerationBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GenerationBackend::Local { model_name, model_size } => {
                write!(f, "local ({}, {})", model_name, model_size)
            }
            GenerationBackend::Ollama { model } => {
                write!(f, "Ollama ({})", model)
            }
            GenerationBackend::Embedded { model_name } => {
                write!(f, "embedded ({})", model_name)
            }
            GenerationBackend::Offline => {
                write!(f, "offline (retrieval only)")
            }
        }
    }
}

/// Detect current generation backend status
pub fn detect_generation_backend() -> GenerationBackend {
    // Check local llama.cpp model first
    if local_llm::LocalLLM::model_exists() {
        #[cfg(feature = "local-llm")]
        {
            if let Ok(llm) = local_llm::LocalLLM::new(None) {
                return GenerationBackend::Local {
                    model_name: llm.model_name().to_string(),
                    model_size: llm.model_size_formatted(),
                };
            }
        }
        #[cfg(not(feature = "local-llm"))]
        {
            // Model exists but feature not enabled
            if let Ok(metadata) = std::fs::metadata(local_llm::LocalLLM::default_path()) {
                let size = metadata.len();
                let gb = size as f64 / 1_073_741_824.0;
                let size_str = if gb >= 1.0 {
                    format!("{:.1}GB", gb)
                } else {
                    format!("{:.0}MB", size as f64 / 1_048_576.0)
                };
                return GenerationBackend::Local {
                    model_name: "model".to_string(),
                    model_size: size_str,
                };
            }
        }
    }

    // Check Ollama
    let ollama = ollama::OllamaEngine::new(None, None);
    if ollama.is_available() {
        return GenerationBackend::Ollama {
            model: "qwen2.5:14b".to_string(),
        };
    }

    // Check embedded
    if let Ok(engine) = embedded::EmbeddedLLM::new(None) {
        return GenerationBackend::Embedded {
            model_name: engine.name().to_string(),
        };
    }

    GenerationBackend::Offline
}

/// Create the default inference engine.
/// Priority: LocalLLM > Embedded > Ollama > Offline
pub fn default_engine() -> Box<dyn InferenceEngine> {
    // Try local llama.cpp first (if feature enabled and model exists)
    #[cfg(feature = "local-llm")]
    {
        if local_llm::LocalLLM::model_exists() {
            match local_llm::LocalLLM::new(None) {
                Ok(engine) => {
                    eprintln!(
                        "⚡ Using local {} ({}) via llama.cpp",
                        engine.model_name(),
                        engine.model_size_formatted()
                    );
                    return Box::new(engine);
                }
                Err(e) => {
                    eprintln!("⚠️  Local LLM unavailable: {}", e);
                }
            }
        }
    }

    // Try embedded next
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

    // Final fallback: offline mode (knowledge retrieval only)
    eprintln!("⚠️  No LLM available — using offline mode (knowledge retrieval only)");
    eprintln!("   Start Ollama or install an embedded model for full responses.");
    Box::new(offline::OfflineEngine::new())
}

/// Check if Ollama is available at the given URL (500ms timeout)
pub fn is_ollama_available(url: &str) -> bool {
    offline::is_ollama_available(url)
}

/// Create an engine with a specific preference
pub fn engine_with_preference(
    prefer_ollama: bool,
    model: Option<&str>,
    ollama_url: Option<&str>,
) -> Box<dyn InferenceEngine> {
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
            eprintln!("⚠️  Embedded LLM unavailable: {}", e);
            // Final fallback: offline mode
            eprintln!("⚠️  No LLM available — using offline mode (knowledge retrieval only)");
            Box::new(offline::OfflineEngine::new())
        }
    }
}
