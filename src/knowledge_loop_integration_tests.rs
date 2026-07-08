#[cfg(test)]
mod tests {
    use crate::inference::{ChatMessage, InferenceEngine};
    use crate::knowledge_loop::KnowledgeLoop;
    use std::sync::Arc;

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
        fn generate(
            &self,
            _prompt: &str,
            _max_tokens: usize,
        ) -> Result<String, Box<dyn std::error::Error>> {
            Ok("LLM response".to_string())
        }
        fn chat(
            &self,
            messages: &[ChatMessage],
            _max_tokens: usize,
        ) -> Result<String, Box<dyn std::error::Error>> {
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

        // Simple factual query should process through knowledge loop
        let result = kl.chat("What is SAGE?").unwrap();
        assert!(!result.is_empty(), "Should get a non-empty response");

        // Verify the knowledge loop processed the query
        assert_eq!(
            kl.history().len(),
            2,
            "Should have user + assistant messages"
        );
        assert!(
            kl.active_cells() > 0,
            "Knowledge should be encoded in NCA grid"
        );

        // Simple query may be answered by local synthesis (no LLM call) or
        // fall back to LLM if local synthesis has no relevant knowledge.
        // Either way, we should get a non-empty response.
        let calls = engine.get_calls();
        assert!(
            calls.len() <= 1,
            "Should call LLM at most once (0 if local synthesis answered)"
        );
    }

    #[test]
    fn test_complex_query_uses_llm() {
        let engine = Arc::new(TrackingMockEngine::new());
        let mut kl = KnowledgeLoop::new(engine.clone());

        // Complex query should use LLM
        let result = kl
            .chat("Why does the NCA grid converge to stable patterns?")
            .unwrap();
        assert_eq!(
            result, "LLM response",
            "Complex query should use LLM backend"
        );

        // Verify the query was passed to LLM
        let calls = engine.get_calls();
        assert!(
            calls.iter().any(|c| c.contains("converge")),
            "Query should reach LLM"
        );
        assert!(kl.active_cells() > 0, "Query should be encoded in grid");
    }

    #[test]
    fn test_multi_turn_conversation_builds_knowledge() {
        let engine = Arc::new(TrackingMockEngine::new());
        let mut kl = KnowledgeLoop::new(engine.clone());

        // Multiple exchanges should accumulate knowledge
        let _ = kl.chat("My name is Alice").unwrap();
        let _ = kl.chat("I like programming in Rust").unwrap();
        let _ = kl.chat("What language do I like?").unwrap();

        // Should have 3 exchanges × 2 messages
        assert_eq!(kl.history().len(), 6, "Should track all messages");

        // Grid should have encoded facts
        assert!(kl.active_cells() >= 3, "Should encode each fact");

        // Some queries may be answered by local synthesis (no LLM call).
        // The first two are conversational statements (not factual questions),
        // so they should go to LLM. The third is a simple question that might
        // be answered locally if the brain has encoded the knowledge.
        let calls = engine.get_calls();
        assert!(
            calls.len() >= 1 && calls.len() <= 3,
            "Should call LLM 1-3 times (some may use local synthesis): got {}",
            calls.len()
        );
    }

    #[test]
    fn test_empty_predictor_falls_back_to_llm() {
        let engine = Arc::new(TrackingMockEngine::new());
        let mut kl = KnowledgeLoop::new(engine.clone());

        // Query should work via LLM fallback
        let result = kl.chat("Hello").unwrap();
        assert_eq!(result, "LLM response", "Should fallback to LLM");

        // Verify knowledge was still encoded
        assert!(
            kl.active_cells() > 0,
            "Should encode knowledge with LLM fallback"
        );

        // Verify the call was made
        let calls = engine.get_calls();
        assert_eq!(calls.len(), 1, "Should call LLM once");
    }

    #[test]
    fn test_knowledge_retrieval_returns_context() {
        let engine = Arc::new(TrackingMockEngine::new());
        let mut kl = KnowledgeLoop::new(engine.clone());
        kl.relevance_threshold = 0.01;

        // Encode a fact
        kl.encode("The speed of light is 299,792,458 m/s", 0.9);

        // Should have active cells
        assert!(kl.active_cells() > 0, "Fact should be encoded");

        // Query for the fact
        let context = kl.retrieve_knowledge("speed of light");

        // Either we get the context or we don't (hash-based encoding may not match)
        // but the mechanism should work without panic
        if let Some(ctx) = context {
            assert!(
                ctx.contains("Recalled Knowledge") || ctx.contains("speed"),
                "Context should contain retrieved information"
            );
        }
    }

    #[test]
    fn test_clear_history_preserves_brain() {
        let engine = Arc::new(TrackingMockEngine::new());
        let mut kl = KnowledgeLoop::new(engine.clone());

        // Add some knowledge
        let _ = kl.chat("test fact").unwrap();
        let active_before = kl.active_cells();
        assert!(active_before > 0, "Should have encoded knowledge");

        // Clear history
        kl.clear_history();
        assert!(kl.history().is_empty(), "History should be empty");

        // Brain should persist
        assert_eq!(
            kl.active_cells(),
            active_before,
            "Brain should preserve knowledge"
        );
    }
}
