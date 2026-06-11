//! Embedded LLM inference using candle + GGUF quantized models
//!
//! Runs SmolLM2-1.7B-Instruct (Q4_K_M) directly in-process.
//! No external dependencies — fully self-contained.

use candle_core::{Device, Tensor};
use candle_transformers::generation::LogitsProcessor;
use candle_transformers::models::quantized_llama::ModelWeights;
use std::error::Error;
use std::path::PathBuf;
use tokenizers::Tokenizer;

use super::{ChatMessage, ChatRole, InferenceEngine};

type TokenCallback = Option<Box<dyn FnMut(&str) + Send>>;

/// HuggingFace repo for the default model
const DEFAULT_REPO: &str = "HuggingFaceTB/SmolLM2-1.7B-Instruct-GGUF";
const DEFAULT_MODEL_FILE: &str = "smollm2-1.7b-instruct-q4_k_m.gguf";
const DEFAULT_TOKENIZER_REPO: &str = "HuggingFaceTB/SmolLM2-1.7B-Instruct";
const MODEL_DISPLAY_NAME: &str = "SmolLM2 1.7B";

/// Embedded LLM engine using candle for GGUF inference
pub struct EmbeddedLLM {
    model: ModelWeights,
    tokenizer: Tokenizer,
    device: Device,
    temperature: f64,
    top_p: f64,
    repeat_penalty: f32,
    repeat_last_n: usize,
}

impl EmbeddedLLM {
    /// Create a new embedded LLM, downloading the model if needed.
    /// `model_path`: Optional path to a GGUF file. If None, downloads default.
    pub fn new(model_path: Option<PathBuf>) -> Result<Self, Box<dyn Error>> {
        let device = Device::Cpu;

        let (model_file, tokenizer_file) = if let Some(path) = model_path {
            // User-provided model path — look for tokenizer.json next to it
            let tokenizer_path = path
                .parent()
                .unwrap_or(std::path::Path::new("."))
                .join("tokenizer.json");
            if !path.exists() {
                return Err(format!("Model file not found: {}", path.display()).into());
            }
            if !tokenizer_path.exists() {
                return Err(format!("Tokenizer not found at: {}", tokenizer_path.display()).into());
            }
            (path, tokenizer_path)
        } else {
            // Download from HuggingFace Hub
            let model_file = Self::ensure_model()?;
            let tokenizer_file = Self::ensure_tokenizer()?;
            (model_file, tokenizer_file)
        };

        eprintln!("📦 Loading model from {}...", model_file.display());

        // Load GGUF model
        let mut file = std::fs::File::open(&model_file)?;
        let model_content = candle_core::quantized::gguf_file::Content::read(&mut file)
            .map_err(|e| format!("Failed to read GGUF: {}", e))?;
        let model = ModelWeights::from_gguf(model_content, &mut file, &device)
            .map_err(|e| format!("Failed to load model weights: {}", e))?;

        // Load tokenizer
        let tokenizer = Tokenizer::from_file(&tokenizer_file)
            .map_err(|e| format!("Failed to load tokenizer: {}", e))?;

        eprintln!("✅ Model loaded successfully");

        Ok(Self {
            model,
            tokenizer,
            device,
            temperature: 0.7,
            top_p: 0.9,
            repeat_penalty: 1.3,
            repeat_last_n: 64,
        })
    }

    /// Ensure the GGUF model file exists, downloading if needed
    fn ensure_model() -> Result<PathBuf, Box<dyn Error>> {
        // Check ~/.sage/models/ first
        let sage_models = dirs::home_dir()
            .unwrap_or_default()
            .join(".sage")
            .join("models");

        let local_path = sage_models.join(DEFAULT_MODEL_FILE);
        if local_path.exists() {
            return Ok(local_path);
        }

        eprintln!("📥 Downloading {} from HuggingFace...", DEFAULT_MODEL_FILE);
        eprintln!("   Repo: {}", DEFAULT_REPO);
        eprintln!("   This is a one-time download (~1GB)");

        let api = hf_hub::api::sync::Api::new()?;
        let repo = api.model(DEFAULT_REPO.to_string());
        let path = repo.get(DEFAULT_MODEL_FILE)?;

        // Copy to ~/.sage/models/ for future use
        std::fs::create_dir_all(&sage_models)?;
        if !local_path.exists() {
            // The hf-hub cache already has it, just use that path
            eprintln!("✅ Model downloaded to HF cache: {}", path.display());
        }

        Ok(path)
    }

    /// Ensure the tokenizer file exists, downloading if needed
    fn ensure_tokenizer() -> Result<PathBuf, Box<dyn Error>> {
        let api = hf_hub::api::sync::Api::new()?;
        let repo = api.model(DEFAULT_TOKENIZER_REPO.to_string());
        let path = repo.get("tokenizer.json")?;
        Ok(path)
    }

    /// Format chat messages into a prompt string
    fn format_chat_prompt(messages: &[ChatMessage]) -> String {
        // SmolLM2-Instruct uses a simple chat format
        let mut prompt = String::new();
        for msg in messages {
            match msg.role {
                ChatRole::System => {
                    prompt.push_str(&format!("<|im_start|>system\n{}<|im_end|>\n", msg.content));
                }
                ChatRole::User => {
                    prompt.push_str(&format!("<|im_start|>user\n{}<|im_end|>\n", msg.content));
                }
                ChatRole::Assistant => {
                    prompt.push_str(&format!(
                        "<|im_start|>assistant\n{}<|im_end|>\n",
                        msg.content
                    ));
                }
            }
        }
        prompt.push_str("<|im_start|>assistant\n");
        prompt
    }

    /// Core generation logic
    fn generate_inner(
        &self,
        prompt: &str,
        max_tokens: usize,
        mut callback: TokenCallback,
    ) -> Result<String, Box<dyn Error>> {
        let encoding = self
            .tokenizer
            .encode(prompt, true)
            .map_err(|e| format!("Tokenizer error: {}", e))?;
        let prompt_tokens = encoding.get_ids().to_vec();

        let mut all_tokens = prompt_tokens.clone();
        let mut logits_processor = LogitsProcessor::from_sampling(
            299792458, // seed
            candle_transformers::generation::Sampling::TopP {
                p: self.top_p,
                temperature: self.temperature,
            },
        );

        // Clone model for mutable forward pass
        let mut model = self.model.clone();
        let mut generated = String::new();
        let eos_token = self
            .tokenizer
            .token_to_id("<|im_end|>")
            .or_else(|| self.tokenizer.token_to_id("</s>"))
            .unwrap_or(2);

        // Process prompt
        let prompt_tensor = Tensor::new(prompt_tokens.as_slice(), &self.device)?.unsqueeze(0)?;
        let logits = model.forward(&prompt_tensor, 0)?;
        let logits = logits.squeeze(0)?;
        let logits = if logits.dims().len() == 1 {
            logits
        } else {
            logits.get(logits.dim(0)? - 1)?
        };

        // Apply repeat penalty
        let logits = Self::apply_repeat_penalty(
            &logits,
            self.repeat_penalty,
            &all_tokens[all_tokens.len().saturating_sub(self.repeat_last_n)..],
        )?;

        let mut next_token = logits_processor.sample(&logits)?;
        all_tokens.push(next_token);

        if next_token == eos_token {
            return Ok(generated);
        }

        if let Ok(text) = self.tokenizer.decode(&[next_token], false) {
            generated.push_str(&text);
            if let Some(ref mut cb) = callback {
                cb(&text);
            }
        }

        // Generate remaining tokens
        for i in 1..max_tokens {
            let input = Tensor::new(&[next_token], &self.device)?.unsqueeze(0)?;
            let logits = model.forward(&input, prompt_tokens.len() + i)?;
            let logits = logits.squeeze(0)?;
            let logits = if logits.dims().len() == 1 {
                logits
            } else {
                logits.get(logits.dim(0)? - 1)?
            };

            let logits = Self::apply_repeat_penalty(
                &logits,
                self.repeat_penalty,
                &all_tokens[all_tokens.len().saturating_sub(self.repeat_last_n)..],
            )?;

            next_token = logits_processor.sample(&logits)?;
            all_tokens.push(next_token);

            if next_token == eos_token {
                break;
            }

            if let Ok(text) = self.tokenizer.decode(&[next_token], false) {
                generated.push_str(&text);
                if let Some(ref mut cb) = callback {
                    cb(&text);
                }
            }
        }

        // Decode properly using the tokenizer for clean output
        let gen_token_ids: Vec<u32> = all_tokens[prompt_tokens.len()..].to_vec();
        if let Ok(decoded) = self.tokenizer.decode(&gen_token_ids, true) {
            return Ok(decoded);
        }

        Ok(generated)
    }

    /// Apply repetition penalty to logits
    fn apply_repeat_penalty(
        logits: &Tensor,
        penalty: f32,
        recent_tokens: &[u32],
    ) -> Result<Tensor, candle_core::Error> {
        if penalty == 1.0 || recent_tokens.is_empty() {
            return Ok(logits.clone());
        }
        let device = logits.device();
        let mut logits_vec: Vec<f32> = logits.to_vec1()?;
        for &token in recent_tokens {
            let token = token as usize;
            if token < logits_vec.len() {
                if logits_vec[token] > 0.0 {
                    logits_vec[token] /= penalty;
                } else {
                    logits_vec[token] *= penalty;
                }
            }
        }
        Tensor::from_vec(logits_vec, logits.shape(), device)
    }
}

impl InferenceEngine for EmbeddedLLM {
    fn generate(&self, prompt: &str, max_tokens: usize) -> Result<String, Box<dyn Error>> {
        self.generate_inner(prompt, max_tokens, None)
    }

    fn chat(&self, messages: &[ChatMessage], max_tokens: usize) -> Result<String, Box<dyn Error>> {
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
        MODEL_DISPLAY_NAME
    }

    fn is_available(&self) -> bool {
        true // If we constructed successfully, we're available
    }
}
