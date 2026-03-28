#![no_main]
use libfuzzer_sys::fuzz_target;
use sage::distributed_knowledge::encoder::{encode_text, EncoderConfig};

fuzz_target!(|data: &[u8]| {
    if let Ok(text) = std::str::from_utf8(data) {
        let mut config = EncoderConfig::default();
        config.ollama_url = None; // Use hash-only encoding for fuzzing

        // Must never panic, must return a valid FeatureVector
        let _ = encode_text(text, &config);
    }
});
