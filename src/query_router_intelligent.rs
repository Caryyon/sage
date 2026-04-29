//! Intelligent Query Router — Self-Improving Query Classification and Routing
//!
//! Replaces the static word-count heuristic with a learning system that:
//! 1. Tracks actual accuracy per query pattern
//! 2. Learns optimal complexity thresholds from outcomes
//! 3. Persists knowledge across sessions
//! 4. Provides detailed routing analytics
//!
//! The router observes:
//! - Query text and classification
//! - Which backend was used (NCA vs LLM)
//! - Response quality (user feedback, self-evaluation, or downstream task success)
//!
//! It learns:
//! - Which query patterns are best handled by NCA (fast, offline)
//! - Which patterns need full LLM reasoning
//! - Optimal confidence thresholds for each category

use crate::inference::nca_predictor::NcaPredictor;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------
// Query Pattern Types
// ---------------------------------------------------------------------------

/// Pattern categories for query classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub enum QueryPattern {
    /// Factual lookup: "What is X?", "Who created Y?"
    FactualLookup,
    /// Temporal: "When did X happen?", "What time..."
    Temporal,
    /// Spatial/Location: "Where is X?", "How do I get to..."
    Spatial,
    /// Quantitative: "How many...", "How much..."
    Quantitative,
    /// Definitional: "What does X mean?", "Define..."
    Definitional,
    /// Comparative: "Compare X and Y", "What's the difference..."
    Comparative,
    /// Causal: "Why does X...", "What causes..."
    Causal,
    /// Procedural: "How do I...", "Steps to..."
    Procedural,
    /// Analytical: "Analyze...", "Explain why..."
    Analytical,
    /// Creative/Generative: "Write a...", "Generate..."
    Creative,
    /// Conversational: "Hello", "Thanks", small talk
    Conversational,
    /// Ambiguous/Unknown: Doesn't match other patterns
    Ambiguous,
}

impl QueryPattern {
    /// Human-readable name for logging
    pub fn name(&self) -> &'static str {
        match self {
            QueryPattern::FactualLookup => "factual_lookup",
            QueryPattern::Temporal => "temporal",
            QueryPattern::Spatial => "spatial",
            QueryPattern::Quantitative => "quantitative",
            QueryPattern::Definitional => "definitional",
            QueryPattern::Comparative => "comparative",
            QueryPattern::Causal => "causal",
            QueryPattern::Procedural => "procedural",
            QueryPattern::Analytical => "analytical",
            QueryPattern::Creative => "creative",
            QueryPattern::Conversational => "conversational",
            QueryPattern::Ambiguous => "ambiguous",
        }
    }

    /// Default complexity assignment (starting point, will be refined)
    pub fn default_complexity(&self) -> QueryComplexity {
        match self {
            QueryPattern::FactualLookup
            | QueryPattern::Temporal
            | QueryPattern::Spatial
            | QueryPattern::Quantitative
            | QueryPattern::Definitional
            | QueryPattern::Conversational => QueryComplexity::Simple,

            QueryPattern::Comparative | QueryPattern::Causal | QueryPattern::Procedural => {
                QueryComplexity::Moderate
            }

            QueryPattern::Analytical | QueryPattern::Creative | QueryPattern::Ambiguous => {
                QueryComplexity::Complex
            }

            // Fallback for new patterns
            _ => QueryComplexity::Moderate
        }
    }
}

/// Query complexity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QueryComplexity {
    Simple,
    Moderate,
    Complex,
}

impl QueryComplexity {
    pub fn as_u8(&self) -> u8 {
        match self {
            QueryComplexity::Simple => 0,
            QueryComplexity::Moderate => 1,
            QueryComplexity::Complex => 2,
        }
    }
}

/// Backend selection for query execution
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Backend {
    /// NCA predictor (fast, offline, approximate)
    Nca,
    /// Full LLM (slower, online, high quality)
    Llm,
}

// ---------------------------------------------------------------------------
// Pattern Detection
// ---------------------------------------------------------------------------

/// Detect the query pattern using multiple signals
pub fn detect_pattern(query: &str) -> QueryPattern {
    let query_lower = query.trim().to_lowercase();
    let words: Vec<&str> = query_lower.split_whitespace().collect();

    if words.is_empty() {
        return QueryPattern::Ambiguous;
    }

    let first_word = words[0];
    let word_count = words.len();

    // Conversational patterns
    if is_conversational(&query_lower, &words) {
        return QueryPattern::Conversational;
    }

    // Pattern matching based on question words and structure
    match first_word {
        "what" => {
            if query_lower.contains("mean") || query_lower.contains("definition") {
                QueryPattern::Definitional
            } else if query_lower.contains("difference")
                || query_lower.contains("compare")
                || query_lower.contains("vs")
                || query_lower.contains("versus")
            {
                QueryPattern::Comparative
            } else if query_lower.contains("time")
                || query_lower.contains("date")
                || query_lower.contains("when")
            {
                QueryPattern::Temporal
            } else if word_count <= 5 {
                QueryPattern::FactualLookup
            } else {
                QueryPattern::Analytical
            }
        }
        "who" | "whom" => QueryPattern::FactualLookup,
        "when" => QueryPattern::Temporal,
        "where" => QueryPattern::Spatial,
        "how" => {
            if query_lower.starts_with("how many")
                || query_lower.starts_with("how much")
                || query_lower.starts_with("how long")
                || query_lower.starts_with("how far")
            {
                QueryPattern::Quantitative
            } else if query_lower.contains("step")
                || query_lower.contains("do i")
                || query_lower.contains("can i")
            {
                QueryPattern::Procedural
            } else if word_count <= 6 {
                QueryPattern::FactualLookup
            } else {
                QueryPattern::Analytical
            }
        }
        "why" => QueryPattern::Causal,
        "which" => {
            if query_lower.contains("better")
                || query_lower.contains("difference")
                || query_lower.contains("vs")
            {
                QueryPattern::Comparative
            } else {
                QueryPattern::FactualLookup
            }
        }
        "is" | "are" | "was" | "were" | "did" | "does" | "do" | "can" | "could"
        | "would" | "should" | "will" => {
            if word_count <= 5 {
                QueryPattern::FactualLookup
            } else {
                QueryPattern::Ambiguous
            }
        }
        "explain" | "analyze" | "describe" | "discuss" => QueryPattern::Analytical,
        "compare" | "contrast" | "differentiate" => QueryPattern::Comparative,
        "write" | "generate" | "create" | "make" | "draft" => QueryPattern::Creative,
        "tell" | "show" | "give" => {
            if query_lower.contains("me about") || query_lower.contains("steps") {
                QueryPattern::Procedural
            } else {
                QueryPattern::FactualLookup
            }
        }
        _ => {
            // No clear question word - analyze by content
            if query_lower.contains("vs")
                || query_lower.contains("versus")
                || query_lower.contains("compare")
                || query_lower.contains("difference between")
            {
                QueryPattern::Comparative
            } else if query_lower.contains("because")
                || query_lower.contains("reason")
                || query_lower.contains("cause")
            {
                QueryPattern::Causal
            } else if word_count <= 4 {
                QueryPattern::FactualLookup
            } else {
                QueryPattern::Ambiguous
            }
        }
    }
}

fn is_conversational(query: &str, words: &[&str]) -> bool {
    let greetings = ["hello", "hi", "hey", "greetings", "good morning", "good afternoon", "good evening"];
    let closings = ["thanks", "thank you", "bye", "goodbye", "see you", "talk later"];
    
    if words.len() <= 3 {
        for greeting in &greetings {
            if query.starts_with(greeting) {
                return true;
            }
        }
        for closing in &closings {
            if query.starts_with(closing) {
                return true;
            }
        }
    }
    
    // Check for standalone acknowledgments
    if words.len() <= 2 && (query == "ok" || query == "okay" || query == "sure" || query == "great" || query == "nice") {
        return true;
    }
    
    false
}

// ---------------------------------------------------------------------------
// Routing Decision and Statistics
// ---------------------------------------------------------------------------

/// Statistics for a single pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternStats {
    /// How many times this pattern was seen
    pub total_queries: u64,
    /// How many times NCA was used
    pub nca_attempts: u64,
    /// How many times LLM was used
    pub llm_attempts: u64,
    /// NCA success count (when NCA was used and succeeded)
    pub nca_successes: u64,
    /// LLM success count (when LLM was used and succeeded)
    pub llm_successes: u64,
    /// Average response time for NCA (ms)
    pub nca_avg_time_ms: f64,
    /// Average response time for LLM (ms)
    pub llm_avg_time_ms: f64,
}

impl PatternStats {
    pub fn new() -> Self {
        Self {
            total_queries: 0,
            nca_attempts: 0,
            llm_attempts: 0,
            nca_successes: 0,
            llm_successes: 0,
            nca_avg_time_ms: 0.0,
            llm_avg_time_ms: 0.0,
        }
    }

    /// NCA accuracy (0.0 if no attempts)
    pub fn nca_accuracy(&self) -> f64 {
        if self.nca_attempts == 0 {
            0.0
        } else {
            self.nca_successes as f64 / self.nca_attempts as f64
        }
    }

    /// LLM accuracy (0.0 if no attempts)
    pub fn llm_accuracy(&self) -> f64 {
        if self.llm_attempts == 0 {
            0.0
        } else {
            self.llm_successes as f64 / self.llm_attempts as f64
        }
    }

    /// Whether NCA is preferred based on accuracy
    pub fn nca_preferred(&self, min_attempts: u64) -> bool {
        if self.nca_attempts < min_attempts {
            return false; // Not enough data
        }
        let nca_acc = self.nca_accuracy();
        let llm_acc = if self.llm_attempts > 0 {
            self.llm_accuracy()
        } else {
            0.8 // Assume LLM is decent
        };
        // Prefer NCA if within 10% of LLM accuracy (faster)
        nca_acc >= llm_acc - 0.1
    }
}

impl Default for PatternStats {
    fn default() -> Self {
        Self::new()
    }
}

/// Outcome of a routing decision
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct RoutingOutcome {
    /// Which backend was used
    pub backend: Backend,
    /// Whether the response was successful/acceptable
    pub success: bool,
    /// Response time in milliseconds
    pub response_time_ms: u64,
    /// User satisfaction if available (0-1, optional)
    pub user_satisfaction: Option<f64>,
}

// ---------------------------------------------------------------------------
// Intelligent Router
// ---------------------------------------------------------------------------

/// Self-improving query router
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntelligentRouter {
    /// Stats per pattern
    pub pattern_stats: HashMap<QueryPattern, PatternStats>,
    /// Minimum attempts before trusting pattern-specific routing
    pub min_attempts_for_learning: u64,
    /// Whether to use learned preferences (vs static rules)
    pub use_learning: bool,
    /// Default fallback complexity
    pub default_complexity: QueryComplexity,
    /// NCA availability
    pub nca_available: bool,
    /// Exploration rate: probability of trying NCA even when LLM is expected better
    pub exploration_rate: f64,
}

impl IntelligentRouter {
    /// Create a new router with default settings
    pub fn new() -> Self {
        let mut pattern_stats = HashMap::new();
        for pattern in [
            QueryPattern::FactualLookup,
            QueryPattern::Temporal,
            QueryPattern::Spatial,
            QueryPattern::Quantitative,
            QueryPattern::Definitional,
            QueryPattern::Comparative,
            QueryPattern::Causal,
            QueryPattern::Procedural,
            QueryPattern::Analytical,
            QueryPattern::Creative,
            QueryPattern::Conversational,
            QueryPattern::Ambiguous,
        ] {
            pattern_stats.insert(pattern, PatternStats::new());
        }

        Self {
            pattern_stats,
            min_attempts_for_learning: 10,
            use_learning: true,
            default_complexity: QueryComplexity::Moderate,
            nca_available: false,
            exploration_rate: 0.1,
        }
    }

    /// Configure NCA availability
    pub fn with_nca_available(mut self, available: bool) -> Self {
        self.nca_available = available;
        self
    }

    /// Set exploration rate (0.0 = pure exploitation, 1.0 = always explore)
    pub fn with_exploration_rate(mut self, rate: f64) -> Self {
        self.exploration_rate = rate.clamp(0.0, 1.0);
        self
    }

    /// Route a query to the appropriate backend
    ///
    /// Returns (backend, pattern, confidence)
    pub fn route(&self, query: &str, nca_available: bool) -> (Backend, QueryPattern, f64) {
        let pattern = detect_pattern(query);
        let stats = self.pattern_stats.get(&pattern).cloned().unwrap_or_default();

        // If learning is disabled or we don't have enough data, use static rules
        if !self.use_learning || stats.total_queries < self.min_attempts_for_learning {
            let complexity = pattern.default_complexity();
            let backend = match complexity {
                QueryComplexity::Simple if nca_available => Backend::Nca,
                _ => Backend::Llm,
            };
            return (backend, pattern, 0.5); // Low confidence when not learned
        }

        // Use learned preferences
        let should_explore = rand::random::<f64>() < self.exploration_rate;
        
        let backend = if should_explore {
            // Exploration: try the less-used backend
            if stats.nca_attempts <= stats.llm_attempts && nca_available {
                Backend::Nca
            } else {
                Backend::Llm
            }
        } else {
            // Exploitation: use the better-performing backend
            if stats.nca_preferred(self.min_attempts_for_learning) && nca_available {
                Backend::Nca
            } else {
                Backend::Llm
            }
        };

        // Confidence based on number of samples
        let confidence = (stats.total_queries as f64 / 100.0).min(1.0);

        (backend, pattern, confidence)
    }

    /// Record the outcome of a routing decision
    pub fn record_outcome(&mut self, pattern: QueryPattern, outcome: RoutingOutcome) {
        let stats = self.pattern_stats.entry(pattern).or_default();
        
        stats.total_queries += 1;
        
        match outcome.backend {
            Backend::Nca => {
                stats.nca_attempts += 1;
                if outcome.success {
                    stats.nca_successes += 1;
                }
                // Update running average
                stats.nca_avg_time_ms = 
                    (stats.nca_avg_time_ms * (stats.nca_attempts - 1) as f64 
                     + outcome.response_time_ms as f64) 
                    / stats.nca_attempts as f64;
            }
            Backend::Llm => {
                stats.llm_attempts += 1;
                if outcome.success {
                    stats.llm_successes += 1;
                }
                stats.llm_avg_time_ms = 
                    (stats.llm_avg_time_ms * (stats.llm_attempts - 1) as f64 
                     + outcome.response_time_ms as f64) 
                    / stats.llm_attempts as f64;
            }
        }
    }

    /// Get a summary of routing statistics
    pub fn summary(&self) -> String {
        let mut lines = vec![
            "Intelligent Router Statistics".to_string(),
            "==============================".to_string(),
        ];

        let mut patterns: Vec<_> = self.pattern_stats.iter().collect();
        patterns.sort_by_key(|(p, _)| *p);

        for (pattern, stats) in patterns {
            if stats.total_queries == 0 {
                continue;
            }
            lines.push(format!(
                "{}: {} queries, NCA={:.1}% acc ({}), LLM={:.1}% acc ({})",
                pattern.name(),
                stats.total_queries,
                stats.nca_accuracy() * 100.0,
                stats.nca_attempts,
                stats.llm_accuracy() * 100.0,
                stats.llm_attempts
            ));
        }

        lines.join("\n")
    }

    /// Save router state to disk
    pub fn save(&self, path: &Path) -> Result<(), Box<dyn Error>> {
        let json = serde_json::to_string_pretty(self)?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, json)?;
        Ok(())
    }

    /// Load router state from disk
    pub fn load(path: &Path) -> Result<Self, Box<dyn Error>> {
        let json = fs::read_to_string(path)?;
        let router: Self = serde_json::from_str(&json)?;
        Ok(router)
    }

    /// Get default save path
    pub fn default_path() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".sage")
            .join("intelligent_router.json")
    }
}

impl Default for IntelligentRouter {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Integration with existing query_router
// ---------------------------------------------------------------------------

/// Bridge function: route using intelligent router if available, fallback to static
///
/// This allows gradual adoption - the intelligent router is used if it has
/// learned data, otherwise we fall back to the static classification.
pub fn intelligent_route(
    query: &str,
    router: Option<&IntelligentRouter>,
    nca_available: bool,
) -> Backend {
    if let Some(r) = router {
        let (backend, _, _) = r.route(query, nca_available);
        backend
    } else {
        // Fallback to original static routing
        let pattern = detect_pattern(query);
        let complexity = pattern.default_complexity();
        match complexity {
            QueryComplexity::Simple if nca_available => Backend::Nca,
            _ => Backend::Llm,
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_pattern_factual() {
        assert_eq!(detect_pattern("What is SAGE?"), QueryPattern::FactualLookup);
        assert_eq!(detect_pattern("Who created this?"), QueryPattern::FactualLookup);
        assert_eq!(detect_pattern("Where is the config file?"), QueryPattern::Spatial);
        assert_eq!(detect_pattern("When was it released?"), QueryPattern::Temporal);
    }

    #[test]
    fn test_detect_pattern_complex() {
        assert_eq!(detect_pattern("Why does the NCA grid converge?"), QueryPattern::Causal);
        assert_eq!(detect_pattern("Explain how gossip protocol works"), QueryPattern::Analytical);
        assert_eq!(detect_pattern("Write a poem about neural networks"), QueryPattern::Creative);
    }

    #[test]
    fn test_detect_pattern_comparative() {
        assert_eq!(detect_pattern("Compare NCA and RNN"), QueryPattern::Comparative);
        assert_eq!(detect_pattern("What is the difference between X and Y?"), QueryPattern::Comparative);
        assert_eq!(detect_pattern("X vs Y"), QueryPattern::Comparative);
    }

    #[test]
    fn test_pattern_stats_accuracy() {
        let mut stats = PatternStats::new();
        assert_eq!(stats.nca_accuracy(), 0.0);
        assert_eq!(stats.llm_accuracy(), 0.0);

        stats.nca_attempts = 10;
        stats.nca_successes = 7;
        assert!((stats.nca_accuracy() - 0.7).abs() < 0.01);
    }

    #[test]
    fn test_router_learning_threshold() {
        let mut router = IntelligentRouter::new()
            .with_nca_available(true)
            .with_exploration_rate(0.0);
        
        router.min_attempts_for_learning = 5;

        // Before learning threshold, uses static rules
        let (backend, pattern, confidence) = router.route("What is X?", true);
        assert_eq!(pattern, QueryPattern::FactualLookup);
        assert_eq!(backend, Backend::Nca); // Simple query + NCA available
        assert_eq!(confidence, 0.5); // Low confidence before learning

        // Add some outcomes
        for _ in 0..5 {
            router.record_outcome(
                QueryPattern::FactualLookup,
                RoutingOutcome {
                    backend: Backend::Nca,
                    success: true,
                    response_time_ms: 50,
                    user_satisfaction: None,
                },
            );
        }

        // Now should have learned - confidence increases with more samples
        let (backend2, _, confidence2) = router.route("What is X?", true);
        assert_eq!(backend2, Backend::Nca);
        // With 5 samples, confidence = 5/100 = 0.05, but learning is active
        assert!(confidence2 <= 0.5, "Low sample confidence should be <= 0.5");
    }

    #[test]
    fn test_router_exploration() {
        let mut router = IntelligentRouter::new()
            .with_nca_available(true)
            .with_exploration_rate(1.0); // Always explore
        
        // Seed with NCA successes
        for _ in 0..10 {
            router.record_outcome(
                QueryPattern::FactualLookup,
                RoutingOutcome {
                    backend: Backend::Nca,
                    success: true,
                    response_time_ms: 50,
                    user_satisfaction: None,
                },
            );
        }

        // With exploration=1.0, should try LLM sometimes
        let mut _nca_count = 0;
        let mut llm_count = 0;
        for _ in 0..100 {
            let (backend, _, _) = router.route("What is X?", true);
            match backend {
                Backend::Nca => _nca_count += 1,
                Backend::Llm => llm_count += 1,
            }
        }
        
        // With exploration=1.0, we should see both backends
        assert!(llm_count > 0, "Should explore LLM with exploration_rate=1.0");
    }

    #[test]
    fn test_save_load() {
        let mut router = IntelligentRouter::new();
        router.record_outcome(
            QueryPattern::FactualLookup,
            RoutingOutcome {
                backend: Backend::Nca,
                success: true,
                response_time_ms: 50,
                user_satisfaction: None,
            },
        );

        let temp_path = std::env::temp_dir().join("test_router.json");
        router.save(&temp_path).unwrap();

        let loaded = IntelligentRouter::load(&temp_path).unwrap();
        let stats = loaded.pattern_stats.get(&QueryPattern::FactualLookup).unwrap();
        assert_eq!(stats.total_queries, 1);
        assert_eq!(stats.nca_attempts, 1);
        assert_eq!(stats.nca_successes, 1);

        let _ = fs::remove_file(&temp_path);
    }

    #[test]
    fn test_intelligent_route_fallback() {
        // Without router, should use static rules
        let backend = intelligent_route("What is X?", None, true);
        assert_eq!(backend, Backend::Nca);

        let backend2 = intelligent_route("Why does X happen?", None, true);
        assert_eq!(backend2, Backend::Llm); // Causal = complex
    }

    #[test]
    fn test_conversational_detection() {
        assert_eq!(detect_pattern("Hello"), QueryPattern::Conversational);
        assert_eq!(detect_pattern("Hi there"), QueryPattern::Conversational);
        assert_eq!(detect_pattern("Thanks!"), QueryPattern::Conversational);
        assert_eq!(detect_pattern("Goodbye"), QueryPattern::Conversational);
    }
}
