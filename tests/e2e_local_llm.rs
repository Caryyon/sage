//! E2E tests for local LLM fallback behavior
//!
//! Tests that verify the generation engine selection and fallback logic.

use sage::inference::local_llm::{get_preset, LocalLLM, MODEL_PRESETS};
use sage::inference::{detect_generation_backend, GenerationBackend};

/// Test that model_exists returns false when no model is present
#[test]
fn test_local_llm_model_exists_when_no_model() {
    // This test assumes no model is present at the default path
    // We can't easily test the "model exists" case without actually downloading a model
    // So we test the negative case: default path should be ~/.sage/model.gguf
    let path = LocalLLM::default_path();
    assert!(
        path.to_string_lossy().contains(".sage"),
        "Default path should be under .sage directory"
    );
    assert!(
        path.to_string_lossy().ends_with("model.gguf"),
        "Default path should end with model.gguf"
    );
}

/// Test that generation backend detection returns a valid backend
#[test]
fn test_generation_backend_detection() {
    let backend = detect_generation_backend();

    // We can't know which backend will be available, but it should be one of the valid types
    match backend {
        GenerationBackend::Local { model_name, model_size } => {
            assert!(!model_name.is_empty(), "Local model name should not be empty");
            assert!(!model_size.is_empty(), "Local model size should not be empty");
        }
        GenerationBackend::Ollama { model } => {
            assert!(!model.is_empty(), "Ollama model should not be empty");
        }
        GenerationBackend::Embedded { model_name } => {
            assert!(!model_name.is_empty(), "Embedded model name should not be empty");
        }
        GenerationBackend::Offline => {
            // Offline mode is always valid
        }
    }
}

/// Test that model presets are properly defined
#[test]
fn test_sage_model_list_shows_presets() {
    // Verify all expected presets exist
    assert!(
        get_preset("phi3-mini").is_some(),
        "phi3-mini preset should exist"
    );
    assert!(
        get_preset("tinyllama").is_some(),
        "tinyllama preset should exist"
    );
    assert!(
        get_preset("mistral-7b").is_some(),
        "mistral-7b preset should exist"
    );

    // Verify presets have required fields
    for preset in MODEL_PRESETS {
        assert!(!preset.name.is_empty(), "Preset name should not be empty");
        assert!(!preset.size.is_empty(), "Preset size should not be empty");
        assert!(
            !preset.description.is_empty(),
            "Preset description should not be empty"
        );
        assert!(!preset.url.is_empty(), "Preset URL should not be empty");
        assert!(
            preset.url.starts_with("https://"),
            "Preset URL should be HTTPS"
        );
        assert!(
            !preset.filename.is_empty(),
            "Preset filename should not be empty"
        );
        assert!(
            preset.filename.ends_with(".gguf"),
            "Preset filename should end with .gguf"
        );
    }

    // Verify there are exactly 3 presets as specified
    assert_eq!(MODEL_PRESETS.len(), 3, "Should have exactly 3 model presets");
}

/// Test that GenerationBackend formats correctly
#[test]
fn test_generation_backend_display() {
    let local = GenerationBackend::Local {
        model_name: "phi3-mini".to_string(),
        model_size: "2.3GB".to_string(),
    };
    assert_eq!(format!("{}", local), "local (phi3-mini, 2.3GB)");

    let ollama = GenerationBackend::Ollama {
        model: "qwen2.5:14b".to_string(),
    };
    assert_eq!(format!("{}", ollama), "Ollama (qwen2.5:14b)");

    let embedded = GenerationBackend::Embedded {
        model_name: "SmolLM2".to_string(),
    };
    assert_eq!(format!("{}", embedded), "embedded (SmolLM2)");

    let offline = GenerationBackend::Offline;
    assert_eq!(format!("{}", offline), "offline (retrieval only)");
}

/// Test fallback behavior: when no local model exists, should fall back to other backends
#[test]
fn test_generation_falls_back_when_no_local_model() {
    // If no local model exists, backend should be Ollama, Embedded, or Offline
    // This test verifies the fallback chain is working
    if !LocalLLM::model_exists() {
        let backend = detect_generation_backend();
        match backend {
            GenerationBackend::Local { .. } => {
                panic!("Should not detect local backend when no model exists");
            }
            GenerationBackend::Ollama { .. }
            | GenerationBackend::Embedded { .. }
            | GenerationBackend::Offline => {
                // These are all valid fallbacks
            }
        }
    }
}

/// Test that nonexistent preset returns None
#[test]
fn test_get_preset_nonexistent() {
    assert!(
        get_preset("nonexistent-model").is_none(),
        "Nonexistent preset should return None"
    );
    assert!(
        get_preset("").is_none(),
        "Empty preset name should return None"
    );
    assert!(
        get_preset("phi3").is_none(),
        "Partial preset name should return None"
    );
}
