//! Startup E2E Tests
//!
//! Tests for KnowledgeLoop initialization, brain file handling,
//! and serialization roundtrips.

use sage::distributed_knowledge::{
    BrainHeader, KnowledgeStore, NCAKnowledge, BRAIN_MAGIC, BRAIN_VERSION,
};
use sage::grid::NUM_CHANNELS;
use sage::inference::{ChatMessage, ChatRole, InferenceEngine};
use std::error::Error;
use std::sync::Arc;

/// Mock inference engine for testing (echoes input)
struct MockEngine;

impl InferenceEngine for MockEngine {
    fn generate(&self, prompt: &str, _max_tokens: usize) -> Result<String, Box<dyn Error>> {
        Ok(format!("Echo: {}", prompt))
    }

    fn chat(&self, messages: &[ChatMessage], _max_tokens: usize) -> Result<String, Box<dyn Error>> {
        let last = messages
            .iter()
            .rev()
            .find(|m| m.role == ChatRole::User)
            .map(|m| m.content.clone())
            .unwrap_or_default();
        Ok(format!("Echo: {}", last))
    }

    fn generate_streaming(
        &self,
        prompt: &str,
        max_tokens: usize,
        mut callback: Box<dyn FnMut(&str) + Send>,
    ) -> Result<(), Box<dyn Error>> {
        let response = self.generate(prompt, max_tokens)?;
        callback(&response);
        Ok(())
    }

    fn chat_streaming(
        &self,
        messages: &[ChatMessage],
        max_tokens: usize,
        mut callback: Box<dyn FnMut(&str) + Send>,
    ) -> Result<(), Box<dyn Error>> {
        let response = self.chat(messages, max_tokens)?;
        callback(&response);
        Ok(())
    }

    fn name(&self) -> &str {
        "mock"
    }

    fn is_available(&self) -> bool {
        true
    }
}

/// Test: KnowledgeLoop starts cleanly with no brain.bin file
#[test]
fn test_fresh_start_no_brain_file() {
    use sage::knowledge_loop::KnowledgeLoop;

    let temp_path = format!("/tmp/sage_e2e_fresh_{}.bin", std::process::id());
    let _ = std::fs::remove_file(&temp_path); // Ensure it doesn't exist

    let engine = Arc::new(MockEngine);
    let mut kl = KnowledgeLoop::new(engine).with_brain_path(&temp_path);

    // Load should succeed (file doesn't exist, fresh grid)
    let result = kl.load_brain();
    assert!(
        result.is_ok(),
        "Fresh start with no brain file should not panic: {:?}",
        result
    );

    // Should have zero active cells
    assert_eq!(
        kl.active_cells(),
        0,
        "Fresh KnowledgeLoop should have no active cells"
    );

    // Should be able to encode without issues
    let pos = kl.encode("test fresh start encoding", 0.8);
    assert!(
        pos.0 < 256 && pos.1 < 256,
        "Encode should return valid position"
    );
    assert!(
        kl.active_cells() > 0,
        "Should have active cells after encoding"
    );

    // Cleanup
    let _ = std::fs::remove_file(&temp_path);
}

/// Test: Stale or corrupted brain.bin is handled gracefully
/// - Should NOT panic
/// - Should return an error or fall back to fresh grid
/// - Application should be able to continue
#[test]
fn test_stale_brain_file_is_handled_gracefully() {
    let temp_path = format!("/tmp/sage_e2e_stale_{}.bin", std::process::id());
    let backup_path = format!("{}.bak", temp_path);
    let _ = std::fs::remove_file(&temp_path);
    let _ = std::fs::remove_file(&backup_path);

    // Create a corrupted brain file (wrong structure / truncated)
    // This simulates a file from an older version or corrupted data
    let corrupt_data = b"SAGE\x00\x00\x00\x03\x40\x00\x00\x00TRUNCATED_DATA_HERE";
    std::fs::write(&temp_path, corrupt_data).expect("Write corrupt brain");

    // Now try to load — this should detect the corruption
    let mut store = NCAKnowledge::new();
    let result = store.load(&temp_path);

    // Should return an error (not panic)
    assert!(
        result.is_err(),
        "Loading corrupted brain should return Err, not panic"
    );

    let err_msg = result.err().unwrap();
    // The error should be some kind of deserialization or format error
    assert!(!err_msg.is_empty(), "Error message should not be empty");
    eprintln!("Got expected error for corrupted brain: {}", err_msg);

    // Application should be able to continue with fresh grid
    let mut fresh_store = NCAKnowledge::new();
    fresh_store.encode("recovery after stale brain", 0.9);
    assert!(
        !fresh_store.active_knowledge(0.01).is_empty(),
        "Fresh grid should work after stale brain error"
    );

    // Cleanup
    let _ = std::fs::remove_file(&temp_path);
    let _ = std::fs::remove_file(&backup_path);
}

/// Test: Brain serialization roundtrip preserves grid state
#[test]
fn test_brain_serialization_roundtrip_preserves_knowledge() {
    let temp_path = format!("/tmp/sage_e2e_roundtrip_{}_brain.bin", std::process::id());
    let text_store_path = temp_path.replace("brain.bin", "text_store.bin");
    let _ = std::fs::remove_file(&temp_path);
    let _ = std::fs::remove_file(&text_store_path);

    // Encode 3 distinct facts
    let facts = [
        "Paris is the capital of France",
        "Rust is a systems programming language",
        "The sky is blue on clear days",
    ];

    let active_before: usize;
    {
        let mut store = NCAKnowledge::new();

        for fact in &facts {
            store.encode(fact, 0.9);
        }

        // Verify encoding worked
        active_before = store.active_knowledge(0.01).len();
        assert!(active_before > 0, "Should have active cells after encoding");

        // Save to disk
        store.save(&temp_path).expect("Save should succeed");
    }

    // Load into a new store
    let mut loaded_store = NCAKnowledge::new();
    loaded_store.load(&temp_path).expect("Load should succeed");

    // Verify active cells survived (grid state preserved)
    let active_after = loaded_store.active_knowledge(0.01).len();
    assert!(active_after > 0, "Loaded brain should have active cells");
    assert_eq!(
        active_before, active_after,
        "Active cell count should match: before={} after={}",
        active_before, active_after
    );

    // Query each fact and verify retrieval produces results (grid state works)
    let mut queries_with_results = 0;
    for fact in &facts {
        let query = fact
            .split_whitespace()
            .take(2)
            .collect::<Vec<_>>()
            .join(" ");
        let results = loaded_store.query(&query, 10);
        if !results.is_empty() {
            queries_with_results += 1;
        }
    }

    // At least some queries should return results (proves grid state preserved)
    assert!(
        queries_with_results >= 1,
        "At least 1 query should return results after roundtrip. \
         Got {}/3 queries with results.",
        queries_with_results
    );

    // Cleanup
    let _ = std::fs::remove_file(&temp_path);
    let _ = std::fs::remove_file(&text_store_path);
}

/// Test: Brain header has correct magic bytes and version
#[test]
fn test_brain_header_validity() {
    let temp_path = format!("/tmp/sage_e2e_header_{}.bin", std::process::id());
    let _ = std::fs::remove_file(&temp_path);

    let mut store = NCAKnowledge::new();
    store.encode("header test", 0.8);
    store.save(&temp_path).expect("Save should succeed");

    // Read raw bytes and verify header
    let data = std::fs::read(&temp_path).expect("Read should succeed");
    let header_size = BrainHeader::serialized_size();

    assert!(
        data.len() > header_size,
        "Brain file should be larger than header"
    );

    let header: BrainHeader =
        bincode::deserialize(&data[..header_size]).expect("Header should deserialize");

    assert_eq!(header.magic, BRAIN_MAGIC, "Magic bytes should be SAGE");
    assert_eq!(
        header.version, BRAIN_VERSION,
        "Version should match BRAIN_VERSION"
    );
    assert_eq!(
        header.channels as usize, NUM_CHANNELS,
        "Channels should be 38"
    );

    // Cleanup
    let _ = std::fs::remove_file(&temp_path);
}

/// Test: KnowledgeLoop can save and load brain via API
#[test]
fn test_knowledge_loop_brain_persistence() {
    use sage::knowledge_loop::KnowledgeLoop;

    let temp_path = format!("/tmp/sage_e2e_kl_persist_{}.bin", std::process::id());
    let _ = std::fs::remove_file(&temp_path);

    let engine = Arc::new(MockEngine);

    // Create, encode, and save
    {
        let mut kl = KnowledgeLoop::new(engine.clone()).with_brain_path(&temp_path);
        kl.encode("knowledge loop persistence test", 0.9);
        kl.save_brain().expect("Save should succeed");
    }

    // Load and verify
    {
        let mut kl = KnowledgeLoop::new(engine).with_brain_path(&temp_path);
        kl.load_brain().expect("Load should succeed");
        assert!(
            kl.active_cells() > 0,
            "Loaded KnowledgeLoop should have active cells"
        );
    }

    // Cleanup
    let _ = std::fs::remove_file(&temp_path);
}
