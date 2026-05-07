//! Ollama HTTP client as an optional inference backend
//!
//! Falls back to this when embedded LLM isn't available,
//! or when the user prefers a larger model via Ollama.

use serde::{Deserialize, Serialize};
use std::error::Error;
use std::io::BufRead;

use super::{ChatMessage, ChatRole, InferenceEngine};

#[derive(Serialize)]
struct OllamaChatRequest {
    model: String,
    messages: Vec<OllamaChatMessage>,
    stream: bool,
}

#[derive(Serialize)]
struct OllamaChatMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct OllamaChatResponse {
    message: Option<OllamaMessageContent>,
    done: Option<bool>,
}

#[derive(Deserialize)]
struct OllamaMessageContent {
    content: Option<String>,
}

#[derive(Serialize)]
struct OllamaGenerateRequest {
    model: String,
    prompt: String,
    stream: bool,
}

#[derive(Deserialize)]
struct OllamaGenerateResponse {
    response: Option<String>,
    done: Option<bool>,
}

/// Ollama HTTP inference engine
pub struct OllamaEngine {
    url: String,
    model: String,
    display_name: String,
}

impl Default for OllamaEngine {
    fn default() -> Self {
        Self::new(None, None)
    }
}

impl OllamaEngine {
    pub fn new(model: Option<String>, url: Option<String>) -> Self {
        let default_url =
            std::env::var("OLLAMA_HOST").unwrap_or_else(|_| "http://localhost:11434".to_string());
        let model = model.unwrap_or_else(|| "qwen2.5:14b".to_string());
        let display_name = format!("Ollama ({})", model);
        Self {
            url: url.unwrap_or(default_url),
            model,
            display_name,
        }
    }

    fn chat_messages_to_ollama(messages: &[ChatMessage]) -> Vec<OllamaChatMessage> {
        messages
            .iter()
            .map(|m| OllamaChatMessage {
                role: match m.role {
                    ChatRole::System => "system".to_string(),
                    ChatRole::User => "user".to_string(),
                    ChatRole::Assistant => "assistant".to_string(),
                },
                content: m.content.clone(),
            })
            .collect()
    }
}

impl InferenceEngine for OllamaEngine {
    fn generate(&self, prompt: &str, _max_tokens: usize) -> Result<String, Box<dyn Error>> {
        let client = reqwest::blocking::Client::new();
        let resp = client
            .post(format!("{}/api/generate", self.url))
            .json(&OllamaGenerateRequest {
                model: self.model.clone(),
                prompt: prompt.to_string(),
                stream: false,
            })
            .timeout(std::time::Duration::from_secs(120))
            .send()?;

        if !resp.status().is_success() {
            return Err(format!("Ollama error: {}", resp.status()).into());
        }

        let body: OllamaGenerateResponse = resp.json()?;
        Ok(body.response.unwrap_or_default().trim().to_string())
    }

    fn chat(&self, messages: &[ChatMessage], _max_tokens: usize) -> Result<String, Box<dyn Error>> {
        let client = reqwest::blocking::Client::new();
        let resp = client
            .post(format!("{}/api/chat", self.url))
            .json(&OllamaChatRequest {
                model: self.model.clone(),
                messages: Self::chat_messages_to_ollama(messages),
                stream: false,
            })
            .timeout(std::time::Duration::from_secs(120))
            .send()?;

        if !resp.status().is_success() {
            return Err(format!("Ollama error: {}", resp.status()).into());
        }

        let body: OllamaChatResponse = resp.json()?;
        Ok(body
            .message
            .and_then(|m| m.content)
            .unwrap_or_default()
            .trim()
            .to_string())
    }

    fn generate_streaming(
        &self,
        prompt: &str,
        _max_tokens: usize,
        mut callback: Box<dyn FnMut(&str) + Send>,
    ) -> Result<(), Box<dyn Error>> {
        let client = reqwest::blocking::Client::new();
        let resp = client
            .post(format!("{}/api/generate", self.url))
            .json(&OllamaGenerateRequest {
                model: self.model.clone(),
                prompt: prompt.to_string(),
                stream: true,
            })
            .timeout(std::time::Duration::from_secs(120))
            .send()?;

        let reader = std::io::BufReader::new(resp);
        for line in reader.lines() {
            let line = line?;
            if line.is_empty() {
                continue;
            }
            if let Ok(val) = serde_json::from_str::<OllamaGenerateResponse>(&line) {
                if let Some(text) = val.response {
                    callback(&text);
                }
                if val.done == Some(true) {
                    break;
                }
            }
        }
        Ok(())
    }

    fn chat_streaming(
        &self,
        messages: &[ChatMessage],
        _max_tokens: usize,
        mut callback: Box<dyn FnMut(&str) + Send>,
    ) -> Result<(), Box<dyn Error>> {
        let client = reqwest::blocking::Client::new();
        let resp = client
            .post(format!("{}/api/chat", self.url))
            .json(&OllamaChatRequest {
                model: self.model.clone(),
                messages: Self::chat_messages_to_ollama(messages),
                stream: true,
            })
            .timeout(std::time::Duration::from_secs(120))
            .send()?;

        let reader = std::io::BufReader::new(resp);
        for line in reader.lines() {
            let line = line?;
            if line.is_empty() {
                continue;
            }
            if let Ok(val) = serde_json::from_str::<OllamaChatResponse>(&line) {
                if let Some(msg) = val.message {
                    if let Some(content) = msg.content {
                        callback(&content);
                    }
                }
                if val.done == Some(true) {
                    break;
                }
            }
        }
        Ok(())
    }

    fn name(&self) -> &str {
        &self.display_name
    }

    fn is_available(&self) -> bool {
        let client = reqwest::blocking::Client::new();
        client
            .get(format!("{}/api/tags", self.url))
            .timeout(std::time::Duration::from_secs(2))
            .send()
            .map(|r| r.status().is_success())
            .unwrap_or(false)
    }
}
