//! Inference engine abstraction
//!
//! Provides a unified trait for LLM inference, with multiple backends:
//! - `LocalLLM`: Runs GGUF models via llama.cpp (optional, requires local-llm feature)
//! - `EmbeddedLLM`: Runs a quantized GGUF model in-process via candle (default)
//! - `OllamaEngine`: HTTP client to external Ollama process (optional fallback)
//! - `OfflineEngine`: Graceful degradation when no LLM is available (Phase 3 groundwork)

pub mod attractor_network;
pub mod backprop_trainer;
pub mod binary_nca;
pub mod bpe_tokenizer;
pub mod consolidation_trainer;
pub mod criticality;
pub mod distributed;
pub mod embedded;
pub mod embeddings;
pub mod kan;
pub mod local_llm;
pub mod nca_language_head;
pub mod nca_lm;
pub mod nca_lm_gpu;
pub mod nca_lm_trainer;
pub mod nca_predictor;
pub mod offline;
pub mod ollama;
pub mod reservoir;

// Re-export key types for public API
pub use nca_language_head::NcaLanguageHead;
pub use offline::OfflineEngine;
pub use ollama::OllamaEngine;

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
    Local {
        model_name: String,
        model_size: String,
    },
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
            GenerationBackend::Local {
                model_name,
                model_size,
            } => {
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

    // Check embedded first — fully self-contained
    if let Ok(engine) = embedded::EmbeddedLLM::new(None) {
        return GenerationBackend::Embedded {
            model_name: engine.name().to_string(),
        };
    }

    // Check Ollama
    let ollama = ollama::OllamaEngine::new(None, None);
    if ollama.is_available() {
        return GenerationBackend::Ollama {
            model: "qwen2.5:14b".to_string(),
        };
    }

    GenerationBackend::Offline
}

/// Create the default inference engine.
/// Priority: Embedded (candle/SmolLM2) > Ollama > NCA-native (if properly trained) > Offline
/// The NCA-native head is experimental — only used if it has a real vocabulary.
pub fn default_engine() -> Box<dyn InferenceEngine> {
    // 1. Try NCA Language Model first — fully self-contained, no external dependencies
    let lm_weights = nca_lm::default_lm_weights_path();
    let lm_vocab = nca_lm::default_lm_vocab_path();
    let lm_config = nca_lm::default_lm_config_path();
    if lm_weights.exists() && lm_vocab.exists() && lm_config.exists() {
        match nca_lm::NcaLanguageModel::load(Some(&lm_weights), Some(&lm_vocab), Some(&lm_config)) {
            Ok(model) => {
                if model.is_trained() && model.vocab_size() >= 100 {
                    eprintln!("🧬 Using NCA Language Model — {}×{} grid, {} tokens, {}K params",
                        model.config().grid_size, model.config().grid_size,
                        model.vocab_size(), model.weights().param_count() / 1000);
                    return Box::new(nca_lm::NcaLmEngine::new(model));
                }
            }
            Err(e) => {
                eprintln!("⚠️  NCA LM found but failed to load: {}", e);
            }
        }
    }

    // 2. Try embedded SmolLM2 — fully self-contained, real language model
    match embedded::EmbeddedLLM::new(None) {
        Ok(engine) => {
            eprintln!("🧠 Using embedded {} (candle) — fully self-contained", engine.name());
            return Box::new(engine);
        }
        Err(e) => {
            eprintln!("⚠️  Embedded LLM unavailable: {}", e);
        }
    }

    // 3. Try Ollama — local but external process
    let ollama = ollama::OllamaEngine::new(None, None);
    if ollama.is_available() {
        eprintln!("🔗 Using {} — local Ollama backend", ollama.name());
        return Box::new(ollama);
    }
    eprintln!("⚠️  Ollama not available at http://localhost:11434");

    // 4. NCA-native language head — ONLY if it has a real vocabulary (≥100 tokens)
    let head_path = nca_language_head::default_language_head_path();
    let vocab_path = nca_language_head::default_vocab_path();
    if head_path.exists() && vocab_path.exists() {
        match nca_language_head::NcaLanguageHead::load_trained(Some(&head_path), Some(&vocab_path)) {
            Ok(head) => {
                if head.vocab_size() >= 100 {
                    eprintln!("🧠 Using NCA-native language head ({} tokens)", head.vocab_size());
                    return Box::new(head);
                } else {
                    eprintln!("⚠️  NCA language head has only {} tokens — skipping (need ≥100)", head.vocab_size());
                }
            }
            Err(e) => {
                eprintln!("⚠️  Trained NCA head found but failed to load: {}", e);
            }
        }
    }

    // Final fallback: offline mode
    eprintln!("💭 No LLM available — using offline mode (knowledge retrieval only)");
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
