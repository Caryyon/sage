//! Query router integration for KnowledgeLoop
//!
//! Decides which inference backend to use based on query complexity.

use crate::query_router::{classify_query, QueryComplexity};
use crate::inference::nca_predictor::{NcaPredictor, SimpleTokenizer, NcaWeights};
use std::sync::Arc;

/// Router result: which backend to use
pub enum Backend {
    /// NCA predictor for simple factual queries
    Nca,
    /// Ollama/LLM for complex reasoning
    Llm,
}

/// Decide backend based on query + available systems
pub fn route_query(query: &str, nca_available: bool) -> Backend {
    let complexity = classify_query(query);
    match complexity {
        QueryComplexity::Simple if nca_available => Backend::Nca,
        QueryComplexity::Moderate if nca_available => Backend::Nca, // Can use NCA for moderate too
        _ => Backend::Llm,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_route_simple_with_nca() {
        assert!(matches!(
            route_query("What is SAGE?", true),
            Backend::Nca
        ));
    }

    #[test]
    fn test_route_simple_without_nca() {
        assert!(matches!(
            route_query("What is SAGE?", false),
            Backend::Llm
        ));
    }

    #[test]
    fn test_route_complex() {
        assert!(matches!(
            route_query("Why does the NCA grid converge?", true),
            Backend::Llm
        ));
    }
}
