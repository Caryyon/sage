//! Offline inference engine — graceful degradation when no LLM is available
//!
//! Phase 3 groundwork: SAGE should be able to answer from its own NCA knowledge
//! without needing Ollama. This engine returns retrieved knowledge snippets
//! directly with a notice that LLM generation is unavailable.

use super::{ChatMessage, ChatRole, InferenceEngine};
use std::error::Error;

/// Offline inference engine — no LLM, knowledge retrieval only
pub struct OfflineEngine {
    display_name: String,
}

impl Default for OfflineEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl OfflineEngine {
    pub fn new() -> Self {
        Self {
            display_name: "Offline (knowledge retrieval only)".to_string(),
        }
    }

    /// Format a response from the offline engine
    fn offline_response(&self, context: Option<&str>) -> String {
        let mut response = String::from(
            "Running in offline mode. Knowledge retrieval active, LLM unavailable.\n\n",
        );

        if let Some(ctx) = context {
            // Extract knowledge snippets from the context
            let snippets: Vec<&str> = ctx
                .lines()
                .filter(|l| l.starts_with("- "))
                .map(|l| l.trim_start_matches("- "))
                .collect();

            if !snippets.is_empty() {
                response.push_str("Retrieved knowledge:\n");
                for snippet in snippets {
                    response.push_str(&format!("  - {}\n", snippet));
                }
            } else {
                response.push_str("No relevant knowledge found in NCA brain.\n");
            }
        } else {
            response.push_str("No relevant knowledge found in NCA brain.\n");
        }

        response.push_str("\n(Start Ollama or install an embedded model for full responses)");
        response
    }

    /// Extract recalled knowledge from the system message
    fn extract_knowledge_context(messages: &[ChatMessage]) -> Option<String> {
        for msg in messages {
            if msg.role == ChatRole::System {
                // Look for the "Recalled Knowledge" section
                if let Some(start) = msg.content.find("## Recalled Knowledge") {
                    let section = &msg.content[start..];
                    // Find the end of the section (next ## or end of string)
                    let end = section[3..]
                        .find("\n\n##")
                        .map(|i| i + 3)
                        .unwrap_or(section.len());
                    return Some(section[..end].to_string());
                }
                // Also check for NCA Activation Summary
                if let Some(start) = msg.content.find("## NCA Activation Summary") {
                    let section = &msg.content[start..];
                    let end = section[3..]
                        .find("\n\n##")
                        .map(|i| i + 3)
                        .unwrap_or(section.len());
                    return Some(section[..end].to_string());
                }
            }
        }
        None
    }
}

impl InferenceEngine for OfflineEngine {
    fn generate(&self, _prompt: &str, _max_tokens: usize) -> Result<String, Box<dyn Error>> {
        Ok(self.offline_response(None))
    }

    fn chat(&self, messages: &[ChatMessage], _max_tokens: usize) -> Result<String, Box<dyn Error>> {
        let context = Self::extract_knowledge_context(messages);
        Ok(self.offline_response(context.as_deref()))
    }

    fn generate_streaming(
        &self,
        _prompt: &str,
        _max_tokens: usize,
        mut callback: Box<dyn FnMut(&str) + Send>,
    ) -> Result<(), Box<dyn Error>> {
        let response = self.offline_response(None);
        callback(&response);
        Ok(())
    }

    fn chat_streaming(
        &self,
        messages: &[ChatMessage],
        _max_tokens: usize,
        mut callback: Box<dyn FnMut(&str) + Send>,
    ) -> Result<(), Box<dyn Error>> {
        let context = Self::extract_knowledge_context(messages);
        let response = self.offline_response(context.as_deref());
        callback(&response);
        Ok(())
    }

    fn name(&self) -> &str {
        &self.display_name
    }

    fn is_available(&self) -> bool {
        // Offline engine is always available as a fallback
        true
    }
}

/// Check if Ollama is reachable with a short timeout (500ms)
pub fn is_ollama_available(ollama_url: &str) -> bool {
    let client = match reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_millis(500))
        .build()
    {
        Ok(c) => c,
        Err(_) => return false,
    };

    client
        .get(format!("{}/api/tags", ollama_url))
        .send()
        .map(|r| r.status().is_success())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_offline_engine_always_available() {
        let engine = OfflineEngine::new();
        assert!(engine.is_available());
    }

    #[test]
    fn test_offline_response_format() {
        let engine = OfflineEngine::new();
        let response = engine.generate("test", 100).unwrap();
        assert!(response.contains("offline mode"));
        assert!(response.contains("LLM unavailable"));
    }

    #[test]
    fn test_extract_knowledge_from_chat() {
        let engine = OfflineEngine::new();
        let messages = vec![
            ChatMessage {
                role: ChatRole::System,
                content: "You are SAGE.\n\n## Recalled Knowledge\n- The capital of France is Paris\n- Rust is a systems language\n\n## Other".to_string(),
            },
            ChatMessage {
                role: ChatRole::User,
                content: "What is the capital of France?".to_string(),
            },
        ];

        let response = engine.chat(&messages, 100).unwrap();
        assert!(response.contains("Retrieved knowledge"));
    }

    #[test]
    fn test_is_ollama_available_timeout() {
        // Test with a non-existent URL - should return false quickly
        let result = is_ollama_available("http://127.0.0.1:59999");
        assert!(!result);
    }
}
