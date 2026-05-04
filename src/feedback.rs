//! User Feedback System — Learning from Actual Usage
//!
//! Tracks whether NCA responses satisfied users vs when they fell back to LLM.
//! This closes the learning loop: router predicts → NCA responds → user feedback → router improves
//!
//! Feedback types:
//! - Explicit: User runs `sage feedback good|bad` after a response
//! - Implicit: User repeats similar question (NCA failed), or moves on quickly (NCA succeeded)
//! - Fallback: User asks follow-up that triggers LLM routing (signal NCA wasn't enough)

use crate::query_router_intelligent::QueryPattern;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// A single feedback event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackEvent {
    /// Unix timestamp
    pub timestamp: u64,
    /// The query that was asked
    pub query: String,
    /// Pattern that was detected
    pub pattern: QueryPattern,
    /// Whether NCA was attempted
    pub nca_attempted: bool,
    /// Whether NCA satisfied the user (None = no feedback)
    pub nca_satisfied: Option<bool>,
    /// Whether LLM was used
    pub llm_used: bool,
    /// Response time in milliseconds
    pub response_time_ms: u64,
    /// User's explicit rating if provided (-1 to 1)
    pub explicit_rating: Option<f64>,
}

/// Aggregated statistics per pattern
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PatternStats {
    pub total_attempts: u64,
    pub nca_satisfactory: u64,
    pub nca_unsatisfactory: u64,
    pub llm_fallbacks: u64,
    /// Average explicit rating (-1.0 to 1.0)
    pub avg_rating: f64,
}

/// Complete feedback database
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FeedbackStore {
    pub events: Vec<FeedbackEvent>,
    pub pattern_stats: HashMap<String, PatternStats>,
    pub total_events: u64,
    /// Version for migration
    pub version: u32,
}

impl FeedbackStore {
    /// Load from disk or create new
    pub fn load_or_new() -> Self {
        let path = Self::default_path();
        if let Ok(data) = fs::read_to_string(&path) {
            if let Ok(store) = serde_json::from_str(&data) {
                return store;
            }
        }
        Self::default()
    }

    /// Save to disk
    pub fn save(&self) -> Result<(), Box<dyn Error>> {
        let path = Self::default_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        fs::write(&path, json)?;
        Ok(())
    }

    /// Record a new feedback event
    pub fn record(&mut self, event: FeedbackEvent) {
        let pattern_key = format!("{:?}", event.pattern);
        
        let stats = self.pattern_stats.entry(pattern_key).or_default();
        stats.total_attempts += 1;
        
        if let Some(satisfied) = event.nca_satisfied {
            if satisfied {
                stats.nca_satisfactory += 1;
            } else {
                stats.nca_unsatisfactory += 1;
            }
        }
        
        if event.llm_used {
            stats.llm_fallbacks += 1;
        }
        
        if let Some(rating) = event.explicit_rating {
            // Update running average
            let n = stats.total_attempts as f64;
            stats.avg_rating = (stats.avg_rating * (n - 1.0) + rating) / n;
        }
        
        self.events.push(event);
        self.total_events += 1;
        
        // Auto-save every 10 events
        if self.total_events % 10 == 0 {
            let _ = self.save();
        }
    }

    /// Get satisfaction rate for a pattern (0.0 to 1.0)
    pub fn satisfaction_rate(&self, pattern: &QueryPattern) -> Option<f64> {
        let key = format!("{:?}", pattern);
        self.pattern_stats.get(&key).map(|stats| {
            let total = stats.nca_satisfactory + stats.nca_unsatisfactory;
            if total == 0 {
                0.5 // Neutral default
            } else {
                stats.nca_satisfactory as f64 / total as f64
            }
        })
    }

    /// Overall statistics for display
    pub fn summary(&self) -> FeedbackSummary {
        let total_nca = self.events.iter().filter(|e| e.nca_attempted).count();
        let satisfied_nca = self.events.iter().filter(|e| e.nca_satisfied == Some(true)).count();
        let llm_fallbacks = self.events.iter().filter(|e| e.llm_used).count();
        
        FeedbackSummary {
            total_queries: self.events.len(),
            nca_attempts: total_nca,
            nca_satisfaction_rate: if total_nca > 0 {
                satisfied_nca as f64 / total_nca as f64
            } else {
                0.0
            },
            llm_fallback_rate: if !self.events.is_empty() {
                llm_fallbacks as f64 / self.events.len() as f64
            } else {
                0.0
            },
            pattern_breakdown: self.pattern_stats.clone(),
        }
    }

    fn default_path() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".sage")
            .join("feedback.json")
    }
}

/// Summary for CLI display
#[derive(Debug, Clone)]
pub struct FeedbackSummary {
    pub total_queries: usize,
    pub nca_attempts: usize,
    pub nca_satisfaction_rate: f64,
    pub llm_fallback_rate: f64,
    pub pattern_breakdown: HashMap<String, PatternStats>,
}

/// Feedback collection during a session
pub struct FeedbackCollector {
    store: FeedbackStore,
    pending_event: Option<FeedbackEvent>,
}

impl FeedbackCollector {
    pub fn new() -> Self {
        Self {
            store: FeedbackStore::load_or_new(),
            pending_event: None,
        }
    }

    /// Start tracking a query
    pub fn start_query(&mut self, query: String, pattern: QueryPattern, nca_attempted: bool) {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        self.pending_event = Some(FeedbackEvent {
            timestamp,
            query,
            pattern,
            nca_attempted,
            nca_satisfied: None,
            llm_used: false,
            response_time_ms: 0,
            explicit_rating: None,
        });
    }

    /// Mark that LLM was used (NCA wasn't enough)
    pub fn mark_llm_fallback(&mut self) {
        if let Some(ref mut event) = self.pending_event {
            event.llm_used = true;
            event.nca_satisfied = Some(false);
        }
    }

    /// Mark that NCA satisfied the user
    pub fn mark_nca_success(&mut self) {
        if let Some(ref mut event) = self.pending_event {
            event.nca_satisfied = Some(true);
        }
    }

    /// Record explicit user rating (-1.0 to 1.0)
    pub fn record_rating(&mut self, rating: f64) {
        if let Some(ref mut event) = self.pending_event {
            event.explicit_rating = Some(rating.clamp(-1.0, 1.0));
        }
    }

    /// Complete the event and save
    pub fn complete(&mut self, response_time_ms: u64) {
        if let Some(mut event) = self.pending_event.take() {
            event.response_time_ms = response_time_ms;
            self.store.record(event);
        }
    }

    /// Get current summary
    pub fn summary(&self) -> FeedbackSummary {
        self.store.summary()
    }

    /// Get satisfaction rate for a pattern
    pub fn pattern_satisfaction(&self, pattern: &QueryPattern) -> Option<f64> {
        self.store.satisfaction_rate(pattern)
    }

    /// Persist to disk
    pub fn save(&self) -> Result<(), Box<dyn Error>> {
        self.store.save()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feedback_event_creation() {
        let event = FeedbackEvent {
            timestamp: 1234567890,
            query: "What is Rust?".to_string(),
            pattern: QueryPattern::Definitional,
            nca_attempted: true,
            nca_satisfied: Some(true),
            llm_used: false,
            response_time_ms: 150,
            explicit_rating: Some(0.8),
        };
        
        assert_eq!(event.query, "What is Rust?");
        assert!(event.nca_satisfied.unwrap());
    }

    #[test]
    fn test_pattern_stats_update() {
        let mut stats = PatternStats::default();
        assert_eq!(stats.total_attempts, 0);
        
        stats.total_attempts += 1;
        stats.nca_satisfactory += 1;
        
        assert_eq!(stats.total_attempts, 1);
        assert_eq!(stats.nca_satisfactory, 1);
    }

    #[test]
    fn test_collector_flow() {
        let mut collector = FeedbackCollector::new();
        
        collector.start_query(
            "How does SAGE work?".to_string(),
            QueryPattern::Procedural,
            true,
        );
        
        collector.mark_nca_success();
        collector.complete(200);
        
        let summary = collector.summary();
        assert_eq!(summary.total_queries, 1);
        assert_eq!(summary.nca_attempts, 1);
    }
}
