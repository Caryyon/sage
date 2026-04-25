//! Integration tests for KnowledgeLoop query routing
//!
//! These tests verify that the query router correctly selects
//! the inference backend based on query complexity.

#[cfg(test)]
mod tests {
    use super::super::*;
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
    }

    #[test]
    fn test_simple_query_routes_to_nca_when_available() {
        // This test verifies the routing logic is called
        // Actual NCA predictor availability depends on weights file
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
}
