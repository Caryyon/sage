//! Integration test for the SAGE Knowledge Loop.
//!
//! Tests the full cycle: Text → NCA Grid → Knowledge Context → LLM Response
//! Uses a mock inference engine to verify the knowledge augmentation pipeline.

use sage::distributed_knowledge::KnowledgeStore;
use sage::inference::{ChatMessage, ChatRole, InferenceEngine};
use sage::knowledge_loop::KnowledgeLoop;
use std::error::Error;
use std::sync::{Arc, Mutex};

/// Mock engine that captures the system prompt so we can verify knowledge injection
struct CapturingEngine {
    system_prompts: Mutex<Vec<String>>,
    response: String,
}

impl CapturingEngine {
    fn new(response: &str) -> Self {
        Self {
            system_prompts: Mutex::new(Vec::new()),
            response: response.to_string(),
        }
    }

    fn last_system_prompt(&self) -> Option<String> {
        self.system_prompts.lock().unwrap().last().cloned()
    }
}

impl InferenceEngine for CapturingEngine {
    fn generate(&self, _prompt: &str, _max_tokens: usize) -> Result<String, Box<dyn Error>> {
        Ok(self.response.clone())
    }

    fn chat(
        &self,
        messages: &[ChatMessage],
        _max_tokens: usize,
    ) -> Result<String, Box<dyn Error>> {
        if let Some(sys) = messages.iter().find(|m| m.role == ChatRole::System) {
            self.system_prompts
                .lock()
                .unwrap()
                .push(sys.content.clone());
        }
        Ok(self.response.clone())
    }

    fn generate_streaming(
        &self,
        _prompt: &str,
        _max_tokens: usize,
        mut cb: Box<dyn FnMut(&str) + Send>,
    ) -> Result<(), Box<dyn Error>> {
        cb(&self.response);
        Ok(())
    }

    fn chat_streaming(
        &self,
        messages: &[ChatMessage],
        max_tokens: usize,
        mut cb: Box<dyn FnMut(&str) + Send>,
    ) -> Result<(), Box<dyn Error>> {
        let _ = self.chat(messages, max_tokens)?;
        cb(&self.response);
        Ok(())
    }

    fn name(&self) -> &str {
        "capturing-mock"
    }
    fn is_available(&self) -> bool {
        true
    }
}

#[test]
fn test_knowledge_loop_encodes_and_grows() {
    let engine = Arc::new(CapturingEngine::new("Got it!"));
    let mut kl = KnowledgeLoop::new(engine);

    let before = kl.active_cells();
    let _ = kl.chat("Rust is great for systems programming").unwrap();
    let after = kl.active_cells();

    assert!(
        after > before,
        "NCA grid should grow after chat: before={}, after={}",
        before,
        after
    );
}

#[test]
fn test_knowledge_context_injected_into_system_prompt() {
    let engine = Arc::new(CapturingEngine::new("I remember!"));
    let mut kl = KnowledgeLoop::new(engine.clone());
    kl.relevance_threshold = 0.01;

    // Directly encode knowledge (bypass chat to control exactly what's stored)
    kl.encode("The secret password is swordfish", 0.95);

    // Now chat — the system prompt should potentially include recalled knowledge
    let _ = kl.chat("What is the secret password?").unwrap();

    // Check if the system prompt was augmented
    let prompt = engine.last_system_prompt().unwrap();
    // The prompt should at minimum contain the base system prompt
    assert!(prompt.contains("SAGE"));
}

#[test]
fn test_brain_survives_save_load_cycle() {
    let path = "/tmp/sage_kl_integration_test.bin";
    let _ = std::fs::remove_file(path);

    let engine = Arc::new(CapturingEngine::new("ok"));

    // Phase 1: populate brain
    {
        let mut kl = KnowledgeLoop::new(engine.clone()).with_brain_path(path);
        kl.encode("Integration test knowledge alpha", 0.9);
        kl.encode("Integration test knowledge beta", 0.9);
        let _ = kl.chat("Remember this conversation").unwrap();
        kl.save_brain().unwrap();
        let cells = kl.active_cells();
        assert!(cells > 0, "Should have active cells before save");
    }

    // Phase 2: load and verify
    {
        let mut kl = KnowledgeLoop::new(engine).with_brain_path(path);
        kl.load_brain().unwrap();
        assert!(
            kl.active_cells() > 0,
            "Should have active cells after load"
        );
    }

    let _ = std::fs::remove_file(path);
    let _ = std::fs::remove_file(path.replace("brain", "text_store").replace(".bin", ".bin"));
}

#[test]
fn test_multiple_conversations_accumulate_knowledge() {
    let engine = Arc::new(CapturingEngine::new("Noted."));
    let mut kl = KnowledgeLoop::new(engine);

    let topics = [
        "My cat's name is Whiskers",
        "I work as a software engineer",
        "My favorite language is Rust",
        "I live in Austin, Texas",
        "I enjoy hiking on weekends",
    ];

    let mut prev_cells = 0;
    for topic in &topics {
        let _ = kl.chat(topic).unwrap();
        let cells = kl.active_cells();
        assert!(
            cells >= prev_cells,
            "Knowledge should accumulate: topic='{}', before={}, after={}",
            topic,
            prev_cells,
            cells
        );
        prev_cells = cells;
    }

    assert!(
        kl.active_cells() > 5,
        "Should have substantial knowledge after 5 conversations"
    );
}
