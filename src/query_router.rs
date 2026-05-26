//! Query complexity router — decides which inference backend to use
//!
//! Simple queries (who/what/when/where) → NCA predictor (fast, offline)
//! Complex queries (why/how/analysis) → Ollama (full reasoning)

/// Query classification
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum QueryComplexity {
    /// Factual lookup — single answer, no reasoning
    Simple,
    /// Moderate — some reasoning, short context
    Moderate,
    /// Complex — analysis, synthesis, long context
    Complex,
}

/// Simple heuristic: short factual questions are simple
pub fn classify_query(query: &str) -> QueryComplexity {
    let query = query.trim().to_lowercase();
    let words: Vec<&str> = query.split_whitespace().collect();
    let word_count = words.len();

    // Question words
    let is_factual = words
        .first()
        .map(|w| {
            matches!(
                *w,
                "who"
                    | "what"
                    | "when"
                    | "where"
                    | "how many"
                    | "how much"
                    | "is"
                    | "are"
                    | "did"
                    | "does"
            )
        })
        .unwrap_or(false);

    let is_analytical = words
        .first()
        .map(|w| {
            matches!(
                *w,
                "why" | "how" | "explain" | "analyze" | "compare" | "what if"
            )
        })
        .unwrap_or(false);

    if word_count <= 8 && is_factual {
        QueryComplexity::Simple
    } else if word_count <= 15 && !is_analytical {
        QueryComplexity::Moderate
    } else {
        QueryComplexity::Complex
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_simple() {
        assert_eq!(classify_query("What is SAGE?"), QueryComplexity::Simple);
        assert_eq!(classify_query("Who created this?"), QueryComplexity::Simple);
        assert_eq!(
            classify_query("When was it released?"),
            QueryComplexity::Simple
        );
    }

    #[test]
    fn test_classify_complex() {
        assert_eq!(
            classify_query("Why does the NCA grid converge to stable patterns?"),
            QueryComplexity::Complex
        );
        assert_eq!(
            classify_query("Explain how gossip protocol prevents poisoning attacks in detail"),
            QueryComplexity::Complex
        );
    }
}
