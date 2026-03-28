//! Local LLM inference using llama.cpp via llama-cpp-rs
//!
//! Runs GGUF models directly in-process using llama.cpp bindings.
//! No external dependencies — fully self-contained.
//!
//! This module is only compiled when the `local-llm` feature is enabled.

#[cfg(feature = "local-llm")]
mod implementation {
    use super::super::{ChatMessage, ChatRole, InferenceEngine};
    use llama_cpp::{LlamaModel, LlamaParams, SessionParams};
    use std::error::Error;
    use std::path::PathBuf;
    use std::sync::Mutex;

    /// Default model path
    fn default_model_path() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_default()
            .join(".sage")
            .join("model.gguf")
    }

    /// Local LLM engine using llama.cpp for GGUF inference
    pub struct LocalLLM {
        model: Mutex<LlamaModel>,
        model_name: String,
        model_size: u64,
        context_size: u32,
        temperature: f32,
    }

    impl LocalLLM {
        /// Create a new local LLM from a GGUF model file.
        /// `model_path`: Optional path to a GGUF file. If None, uses ~/.sage/model.gguf
        pub fn new(model_path: Option<PathBuf>) -> Result<Self, Box<dyn Error>> {
            let path = model_path.unwrap_or_else(default_model_path);

            if !path.exists() {
                return Err(format!(
                    "Model file not found: {}\n\n\
                     To enable local generation without Ollama:\n\
                       Download a model: sage model download phi3-mini\n\
                       Or place any GGUF file at: {}",
                    path.display(),
                    default_model_path().display()
                )
                .into());
            }

            let model_size = std::fs::metadata(&path)?.len();
            let model_name = path
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "local".to_string());

            eprintln!(
                "Loading local model from {}...",
                path.display()
            );

            let params = LlamaParams::default();
            let model = LlamaModel::load_from_file(&path, params)
                .map_err(|e| format!("Failed to load GGUF model: {}", e))?;

            eprintln!(
                "Local model loaded: {} ({:.1} GB)",
                model_name,
                model_size as f64 / 1_073_741_824.0
            );

            Ok(Self {
                model: Mutex::new(model),
                model_name,
                model_size,
                context_size: 2048,
                temperature: 0.7,
            })
        }

        /// Check if a model file exists at the default path
        pub fn model_exists() -> bool {
            default_model_path().exists()
        }

        /// Get the default model path
        pub fn default_path() -> PathBuf {
            default_model_path()
        }

        /// Get model name
        pub fn model_name(&self) -> &str {
            &self.model_name
        }

        /// Get model size in bytes
        pub fn model_size(&self) -> u64 {
            self.model_size
        }

        /// Get formatted model size (e.g., "2.3GB")
        pub fn model_size_formatted(&self) -> String {
            let gb = self.model_size as f64 / 1_073_741_824.0;
            if gb >= 1.0 {
                format!("{:.1}GB", gb)
            } else {
                let mb = self.model_size as f64 / 1_048_576.0;
                format!("{:.0}MB", mb)
            }
        }

        /// Format chat messages into a prompt string (ChatML format)
        fn format_chat_prompt(messages: &[ChatMessage]) -> String {
            let mut prompt = String::new();
            for msg in messages {
                match msg.role {
                    ChatRole::System => {
                        prompt.push_str(&format!(
                            "<|system|>\n{}<|end|>\n",
                            msg.content
                        ));
                    }
                    ChatRole::User => {
                        prompt.push_str(&format!(
                            "<|user|>\n{}<|end|>\n",
                            msg.content
                        ));
                    }
                    ChatRole::Assistant => {
                        prompt.push_str(&format!(
                            "<|assistant|>\n{}<|end|>\n",
                            msg.content
                        ));
                    }
                }
            }
            prompt.push_str("<|assistant|>\n");
            prompt
        }

        /// Core generation logic
        fn generate_inner(
            &self,
            prompt: &str,
            max_tokens: usize,
            mut callback: Option<Box<dyn FnMut(&str) + Send>>,
        ) -> Result<String, Box<dyn Error>> {
            let model = self.model.lock().map_err(|e| format!("Model lock error: {}", e))?;

            let mut session_params = SessionParams::default();
            session_params.n_ctx = self.context_size;

            let mut session = model
                .create_session(session_params)
                .map_err(|e| format!("Failed to create session: {}", e))?;

            session
                .advance_context(prompt)
                .map_err(|e| format!("Failed to process prompt: {}", e))?;

            let mut generated = String::new();
            let mut completions = session.start_completing_with(
                llama_cpp::standard_sampler::StandardSampler::default(),
                max_tokens,
            ).map_err(|e| format!("Failed to start completion: {}", e))?;

            for token in completions.into_iter().take(max_tokens) {
                let text = token.text;

                // Check for end tokens
                if text.contains("<|end|>") || text.contains("<|endoftext|>") {
                    break;
                }

                generated.push_str(&text);
                if let Some(ref mut cb) = callback {
                    cb(&text);
                }
            }

            Ok(generated.trim().to_string())
        }
    }

    impl InferenceEngine for LocalLLM {
        fn generate(&self, prompt: &str, max_tokens: usize) -> Result<String, Box<dyn Error>> {
            self.generate_inner(prompt, max_tokens, None)
        }

        fn chat(
            &self,
            messages: &[ChatMessage],
            max_tokens: usize,
        ) -> Result<String, Box<dyn Error>> {
            let prompt = Self::format_chat_prompt(messages);
            self.generate_inner(&prompt, max_tokens, None)
        }

        fn generate_streaming(
            &self,
            prompt: &str,
            max_tokens: usize,
            callback: Box<dyn FnMut(&str) + Send>,
        ) -> Result<(), Box<dyn Error>> {
            self.generate_inner(prompt, max_tokens, Some(callback))?;
            Ok(())
        }

        fn chat_streaming(
            &self,
            messages: &[ChatMessage],
            max_tokens: usize,
            callback: Box<dyn FnMut(&str) + Send>,
        ) -> Result<(), Box<dyn Error>> {
            let prompt = Self::format_chat_prompt(messages);
            self.generate_inner(&prompt, max_tokens, Some(callback))?;
            Ok(())
        }

        fn name(&self) -> &str {
            &self.model_name
        }

        fn is_available(&self) -> bool {
            true // If we constructed successfully, we're available
        }
    }
}

#[cfg(feature = "local-llm")]
pub use implementation::LocalLLM;

// Stub implementation when feature is disabled
#[cfg(not(feature = "local-llm"))]
mod stub {
    use super::super::{ChatMessage, InferenceEngine};
    use std::error::Error;
    use std::path::PathBuf;

    /// Stub LocalLLM when the local-llm feature is disabled
    pub struct LocalLLM;

    impl LocalLLM {
        pub fn new(_model_path: Option<PathBuf>) -> Result<Self, Box<dyn Error>> {
            Err("Local LLM support not compiled. Rebuild with: cargo build --features local-llm".into())
        }

        pub fn model_exists() -> bool {
            // Check if a model file exists, even if feature is disabled
            Self::default_path().exists()
        }

        pub fn default_path() -> PathBuf {
            dirs::home_dir()
                .unwrap_or_default()
                .join(".sage")
                .join("model.gguf")
        }

        pub fn model_name(&self) -> &str {
            "disabled"
        }

        pub fn model_size(&self) -> u64 {
            0
        }

        pub fn model_size_formatted(&self) -> String {
            "N/A".to_string()
        }
    }

    impl InferenceEngine for LocalLLM {
        fn generate(&self, _prompt: &str, _max_tokens: usize) -> Result<String, Box<dyn Error>> {
            Err("Local LLM not available".into())
        }

        fn chat(
            &self,
            _messages: &[ChatMessage],
            _max_tokens: usize,
        ) -> Result<String, Box<dyn Error>> {
            Err("Local LLM not available".into())
        }

        fn generate_streaming(
            &self,
            _prompt: &str,
            _max_tokens: usize,
            _callback: Box<dyn FnMut(&str) + Send>,
        ) -> Result<(), Box<dyn Error>> {
            Err("Local LLM not available".into())
        }

        fn chat_streaming(
            &self,
            _messages: &[ChatMessage],
            _max_tokens: usize,
            _callback: Box<dyn FnMut(&str) + Send>,
        ) -> Result<(), Box<dyn Error>> {
            Err("Local LLM not available".into())
        }

        fn name(&self) -> &str {
            "local-llm (disabled)"
        }

        fn is_available(&self) -> bool {
            false
        }
    }
}

#[cfg(not(feature = "local-llm"))]
pub use stub::LocalLLM;

/// Print helpful message when local model is not found
pub fn print_model_help() {
    let path = LocalLLM::default_path();
    eprintln!();
    eprintln!("To enable local generation without Ollama:");
    eprintln!("   Download a model: sage model download phi3-mini");
    eprintln!("   Or place any GGUF file at: {}", path.display());
    eprintln!();
}

/// Model presets available for download
#[derive(Clone, Debug)]
pub struct ModelPreset {
    pub name: &'static str,
    pub size: &'static str,
    pub description: &'static str,
    pub url: &'static str,
    pub filename: &'static str,
}

/// Available model presets
pub const MODEL_PRESETS: &[ModelPreset] = &[
    ModelPreset {
        name: "phi3-mini",
        size: "2.3GB",
        description: "Microsoft Phi-3 Mini -- fast, good for Q&A",
        url: "https://huggingface.co/microsoft/Phi-3-mini-4k-instruct-gguf/resolve/main/Phi-3-mini-4k-instruct-q4.gguf",
        filename: "Phi-3-mini-4k-instruct-q4.gguf",
    },
    ModelPreset {
        name: "tinyllama",
        size: "600MB",
        description: "TinyLlama 1.1B -- fastest, lowest quality",
        url: "https://huggingface.co/TheBloke/TinyLlama-1.1B-Chat-v1.0-GGUF/resolve/main/tinyllama-1.1b-chat-v1.0.Q4_K_M.gguf",
        filename: "tinyllama-1.1b-chat-v1.0.Q4_K_M.gguf",
    },
    ModelPreset {
        name: "mistral-7b",
        size: "4.1GB",
        description: "Mistral 7B -- best quality, needs 8GB RAM",
        url: "https://huggingface.co/TheBloke/Mistral-7B-Instruct-v0.2-GGUF/resolve/main/mistral-7b-instruct-v0.2.Q4_K_M.gguf",
        filename: "mistral-7b-instruct-v0.2.Q4_K_M.gguf",
    },
];

/// Get a model preset by name
pub fn get_preset(name: &str) -> Option<&'static ModelPreset> {
    MODEL_PRESETS.iter().find(|p| p.name == name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_presets_exist() {
        assert!(!MODEL_PRESETS.is_empty());
        assert!(get_preset("phi3-mini").is_some());
        assert!(get_preset("tinyllama").is_some());
        assert!(get_preset("mistral-7b").is_some());
        assert!(get_preset("nonexistent").is_none());
    }

    #[test]
    fn test_default_path() {
        let path = LocalLLM::default_path();
        assert!(path.to_string_lossy().contains(".sage"));
        assert!(path.to_string_lossy().contains("model.gguf"));
    }
}
