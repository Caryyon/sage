//! Integration tests for KnowledgeLoop query routing
//!
//! These tests verify that the query router correctly selects
//! the inference backend based on query complexity.

use crate::knowledge_loop::KnowledgeLoop;
use crate::inference::{ChatMessage, ChatRole, InferenceEngine};
use std::sync::Arc;

/// Mock engine that tracks which backend was used
struct TrackingMockEngine {
    calls: std::sync::Mutex<Vec<String>>,
}

impl TrackingMockEngine {
    fn new() -> Self {
        Self {
            calls: std::sync::Mutex::new(Vec::new()),
        }
    }
    fn get_calls(&self) -> Vec<String> {
        self.calls.lock().unwrap().clone()
    }
}

impl InferenceEngine for TrackingMockEngine {
    fn generate(&self, _prompt: &str, _max_tokens: usize) -> Result<String, Box<dyn std::error::Error>> {
        Ok("LLM response".to_string())
    }

    fn chat(
        &self,
        messages: &[ChatMessage],
        _max_tokens: usize,
    ) -> Result<String, Box<dyn std::error::Error>> {
        // Record that LLM was called
        if let Some(last) = messages.last() {
            self.calls.lock().unwrap().push(last.content.clone());
        }
        Ok("LLM response".to_string())
    }

    fn generate_streaming(
        &self,
        _prompt: &str,
        _max_tokens: usize,
        _callback: Box<dyn FnMut(&str) + Send>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    fn chat_streaming(
        &self,
        _messages: &[ChatMessage],
        _max_tokens: usize,
        _callback: Box<dyn FnMut(&str) + Send>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    fn name(&self) -> &str {
        "tracking-mock"
    }

    fn is_available(&self) -> bool {
        true
    }
}

#[test]
fn test_simple_query_routes_correctly() {
    let engine = Arc::new(TrackingMockEngine::new());
    let mut kl = KnowledgeLoop::new(engine.clone());
    
    // Simple query
    let result = kl.chat("What is SAGE?").unwrap();
    
    // Should get some response (either from NCA or LLM fallback)
    assert!(!result.is_empty());
}

#[test]
fn test_complex_query_uses_llm() {
    let engine = Arc::new(TrackingMockEngine::new());
    let mut kl = KnowledgeLoop::new(engine.clone());
    
    // Complex query
    let result = kl.chat("Why does the NCA grid converge to stable patterns?").unwrap();
    
    // Should use LLM for complex queries
    assert_eq!(result, "LLM response");
    
    // Verify LLM was called
    let calls = engine.get_calls();
    assert!(calls.iter().any(|c| c.contains("converge")));
}

#[test]
fn test_moderate_query_uses_llm() {
    let engine = Arc::new(TrackingMockEngine::new());
    let mut kl = KnowledgeLoop::new(engine.clone());
    
    // Moderate query
    let result = kl.chat("Compare SAGE to other AI systems").unwrap();
    
    // Should use LLM for moderate queries
    assert_eq!(result, "LLM response");
}

#[test]
fn test_empty_predictor_falls_back_to_llm() {
    let engine = Arc::new(TrackingMockEngine::new());
    let mut kl = KnowledgeLoop::new(engine.clone());
    
    // Simple query without predictor weights
    let result = kl.chat("Hello").unwrap();
    
    // Should fallback to LLM
    assert_eq!(result, "LLM response");
}
