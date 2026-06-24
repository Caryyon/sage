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

        if let Some(context) = context {
            if context.is_empty() {
                response.push_str("No relevant knowledge found in NCA brain.\n");
            } else {
                response.push_str("Retrieved knowledge from NCA brain:\n\n");
                response.push_str(context);
                response.push('\n');
            }
        } else {
            response.push_str("No relevant knowledge found in NCA brain.\n");
        }

        response.push_str("\n(Start Ollama or install an embedded model for full responses)");
        response
    }

    /// Extract ALL knowledge sections from the system message.
    ///
    /// The KnowledgeLoop augments the system prompt with multiple sections:
    /// - ## Recalled Knowledge (semantic retrieval)
    /// - ## Associatively Recalled Concepts (delta retrieval)
    /// - ## NCA Activation Summary (COCONUT thought layer)
    /// - ## NCA Intuition (keyword associations from Hebbian propagation)
    ///
    /// In offline mode, we extract all of these and present them to the user
    /// as raw knowledge — no LLM synthesis, but the information is there.
    fn extract_knowledge_context(messages: &[ChatMessage]) -> Option<String> {
        let mut sections: Vec<String> = Vec::new();

        for msg in messages {
            if msg.role != ChatRole::System {
                continue;
            }

            // Find all ## sections that contain knowledge
            let content = &msg.content;
            let mut search_from = 0;
            while let Some(start) = content[search_from..].find("## ") {
                let abs_start = search_from + start;
                // Get the section header
                let rest = &content[abs_start..];
                let header_end = rest.find('\n').unwrap_or(rest.len());
                let header = &rest[..header_end];

                // Find the end of this section (next ## at same level or end)
                let after_header = &rest[header_end..];
                let section_end = after_header
                    .find("\n\n## ")
                    .map(|i| header_end + i)
                    .unwrap_or(rest.len());
                let section_content = &rest[..section_end];

                // Only extract knowledge-related sections
                if header.contains("Recalled Knowledge")
                    || header.contains("Associatively Recalled")
                    || header.contains("NCA Activation")
                    || header.contains("NCA Intuition")
                {
                    sections.push(section_content.to_string());
                }

                search_from = abs_start + section_end;
                if search_from >= content.len() {
                    break;
                }
            }
        }

        if sections.is_empty() {
            None
        } else {
            Some(sections.join("\n\n"))
        }
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
        assert!(response.contains("capital of France is Paris"));
    }

    #[test]
    fn test_extract_nca_intuition_section() {
        let engine = OfflineEngine::new();
        let messages = vec![
            ChatMessage {
                role: ChatRole::System,
                content: "You are SAGE.\n\n## NCA Intuition\n**Associated concepts:** philosophy (3), power (2)\n\n**Activated knowledge clusters:**\n1. Republic by Plato...\n\n## Other stuff".to_string(),
            },
            ChatMessage {
                role: ChatRole::User,
                content: "Tell me about philosophy".to_string(),
            },
        ];

        let response = engine.chat(&messages, 100).unwrap();
        assert!(response.contains("NCA Intuition"));
        assert!(response.contains("philosophy"));
    }

    #[test]
    fn test_extract_multiple_sections() {
        let engine = OfflineEngine::new();
        let messages = vec![
            ChatMessage {
                role: ChatRole::System,
                content: "You are SAGE.\n\n## Recalled Knowledge\n- Paris is the capital of France\n\n## Associatively Recalled Concepts\n- French Revolution\n\n## NCA Intuition\n**Associated concepts:** france (2)\n\n## NCA Activation Summary\n- Cell (100,200): 0.8 activation\n".to_string(),
            },
            ChatMessage {
                role: ChatRole::User,
                content: "What is the capital of France?".to_string(),
            },
        ];

        let response = engine.chat(&messages, 100).unwrap();
        assert!(response.contains("Recalled Knowledge"));
        assert!(response.contains("Associatively Recalled"));
        assert!(response.contains("NCA Intuition"));
        assert!(response.contains("NCA Activation"));
    }

    #[test]
    fn test_is_ollama_available_timeout() {
        // Test with a non-existent URL - should return false quickly
        let result = is_ollama_available("http://127.0.0.1:59999");
        assert!(!result);
    }
}