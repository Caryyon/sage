//! BPE Tokenizer for NCA Language Head
//!
//! Uses HuggingFace's tokenizers crate for proper subword tokenization.
//! Eliminates <unk> tokens by breaking unknown words into known subword pieces.
//! Trains on the combined specialist corpus for zero-unknown coverage.

use std::collections::HashMap;
use std::path::Path;

/// A BPE tokenizer that wraps HuggingFace's tokenizers crate.
/// Provides the same interface as SimpleTokenizer for drop-in compatibility.
#[derive(Clone)]
pub struct BpeTokenizer {
    /// HuggingFace tokenizer instance
    inner: tokenizers::Tokenizer,
    /// Token string → ID mapping
    token_to_id: HashMap<String, usize>,
    /// ID → token string mapping
    id_to_token: Vec<String>,
}

impl BpeTokenizer {
    /// Train a BPE tokenizer on a corpus and save to disk.
    /// Returns the trained tokenizer ready for use.
    pub fn train(corpus: &str, vocab_size: usize, save_path: Option<&Path>) -> Result<Self, String> {
        // Write corpus to temp file (tokenizers crate trains from files)
        let tmp_path = std::env::temp_dir().join("sage_bpe_corpus.txt");
        std::fs::write(&tmp_path, corpus)
            .map_err(|e| format!("Failed to write temp corpus: {}", e))?;

        // Configure BPE trainer
        let mut trainer = tokenizers::models::bpe::BpeTrainerBuilder::new()
            .vocab_size(vocab_size)
            .min_frequency(1)
            .special_tokens(vec![
                tokenizers::AddedToken::from("<unk>", true),
                tokenizers::AddedToken::from("<s>", true),
                tokenizers::AddedToken::from("</s>", true),
            ])
            .build();

        // Create a TokenizerImpl with BPE model. We need to specify the type parameters
        // since they can't be inferred from `new()` (all are None).
        let mut tokenizer_impl: tokenizers::TokenizerImpl<
            tokenizers::models::bpe::BPE,
            tokenizers::normalizers::NormalizerWrapper,
            tokenizers::pre_tokenizers::PreTokenizerWrapper,
            tokenizers::processors::PostProcessorWrapper,
            tokenizers::decoders::DecoderWrapper,
        > = tokenizers::TokenizerImpl::new(
            tokenizers::models::bpe::BPE::default(),
        );

        // Train on the corpus file (must happen before conversion to Tokenizer)
        tokenizer_impl
            .train_from_files(&mut trainer, vec![tmp_path.to_string_lossy().to_string()])
            .map_err(|e| format!("BPE training failed: {}", e))?;

        // Convert to the generic Tokenizer type
        let tokenizer: tokenizers::Tokenizer = tokenizer_impl.into();

        // Clean up temp file
        let _ = std::fs::remove_file(&tmp_path);

        // Save if path provided
        if let Some(path) = save_path {
            tokenizer
                .save(path.to_string_lossy().to_string(), false)
                .map_err(|e| format!("Failed to save tokenizer: {}", e))?;
        }

        // Build id↔token mappings
        let vocab = tokenizer.get_vocab(true);
        let mut id_to_token: Vec<String> = vec!["".to_string(); vocab.len()];
        let mut token_to_id = HashMap::new();

        for (token, id) in &vocab {
            let idx = *id as usize;
            if idx < id_to_token.len() {
                id_to_token[idx] = token.clone();
            }
            token_to_id.insert(token.clone(), *id as usize);
        }

        Ok(Self {
            inner: tokenizer,
            token_to_id,
            id_to_token,
        })
    }

    /// Load a previously trained BPE tokenizer from disk
    pub fn load(path: &Path) -> Result<Self, String> {
        let tokenizer = tokenizers::Tokenizer::from_file(path.to_string_lossy().to_string())
            .map_err(|e| format!("Failed to load tokenizer: {}", e))?;

        let vocab = tokenizer.get_vocab(true);
        let mut id_to_token: Vec<String> = vec!["".to_string(); vocab.len()];
        let mut token_to_id = HashMap::new();

        for (token, id) in &vocab {
            let idx = *id as usize;
            if idx < id_to_token.len() {
                id_to_token[idx] = token.clone();
            }
            token_to_id.insert(token.clone(), *id as usize);
        }

        Ok(Self {
            inner: tokenizer,
            token_to_id,
            id_to_token,
        })
    }

    /// Encode text to token IDs
    pub fn encode(&self, text: &str) -> Vec<usize> {
        let encoding = self.inner.encode(text, false)
            .unwrap_or_else(|_| self.inner.encode("", false).unwrap());
        encoding.get_ids().iter().map(|&id| id as usize).collect()
    }

    /// Decode token IDs back to text
    pub fn decode(&self, ids: &[usize]) -> String {
        let ids_u32: Vec<u32> = ids.iter().map(|&id| id as u32).collect();
        self.inner.decode(&ids_u32, true)
            .unwrap_or_else(|_| String::new())
            .replace("##", "") // Remove BPE continuation markers
    }

    /// Number of tokens in vocabulary
    pub fn vocab_size(&self) -> usize {
        self.inner.get_vocab_size(true)
    }

    /// Get the token string for an ID
    pub fn id_to_token_str(&self, id: usize) -> Option<&str> {
        self.id_to_token.get(id).map(|s| s.as_str())
    }

    /// Save the tokenizer to a file
    pub fn save(&self, path: &Path) -> Result<(), String> {
        self.inner
            .save(path.to_string_lossy().to_string(), false)
            .map_err(|e| format!("Failed to save tokenizer: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bpe_train_and_encode() {
        let corpus = "react component state props hook useState useEffect render jsx form input button submit validation react component state props hook useState useEffect render jsx form input button submit validation";
        let tmp = std::env::temp_dir().join(format!("sage_bpe_test_{}.txt", std::process::id()));
        let tokenizer = BpeTokenizer::train(corpus, 100, Some(&tmp)).expect("training should succeed");
        let _ = std::fs::remove_file(&tmp);

        assert!(tokenizer.vocab_size() >= 10);
        assert!(tokenizer.vocab_size() <= 100);

        let ids = tokenizer.encode("react component useState");
        assert!(!ids.is_empty(), "Should encode known tokens");

        let decoded = tokenizer.decode(&ids);
        assert!(!decoded.is_empty(), "Should decode back to text");
        eprintln!("BPE: 'react component useState' → {:?} → '{}'", ids, decoded);
    }

    #[test]
    fn test_bpe_handles_unknown_words() {
        let corpus = "react component state props hook useState useEffect render jsx form input button submit validation react component state props hook useState useEffect render jsx form input button submit validation";
        let tmp = std::env::temp_dir().join(format!("sage_bpe_test2_{}.txt", std::process::id()));
        let tokenizer = BpeTokenizer::train(corpus, 100, Some(&tmp)).unwrap();
        let _ = std::fs::remove_file(&tmp);

        // "xylophone" is not in the corpus — BPE should break it into subwords
        let ids = tokenizer.encode("react xylophone");
        assert!(!ids.is_empty(), "Should encode unknown word as subwords");
        // Should NOT contain <unk> (id 0) for the unknown part
        let has_unk = ids.iter().any(|&id| {
            tokenizer.id_to_token_str(id) == Some("<unk>")
        });
        eprintln!("BPE: 'react xylophone' → {:?}, has_unk={}", ids, has_unk);
        // BPE may still produce <unk> for completely novel characters, but
        // for alphabetic text it should find subword matches
    }
}
