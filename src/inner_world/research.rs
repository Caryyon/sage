//! SAGE's Research System (Feature 7: Web Research)
//!
//! SAGE can research topics she's curious about, either triggered by
//! conversations, reading, or her own curiosity. Research results
//! are stored as learned facts.

use serde::{Deserialize, Serialize};

/// A research query SAGE wants to investigate
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResearchQuery {
    /// The search query
    pub query: String,
    /// Why SAGE wants to know this
    pub motivation: String,
    /// What triggered this research
    pub trigger: ResearchTrigger,
    /// When this was created (tick)
    pub created_at: u64,
    /// Has this been researched yet?
    pub completed: bool,
}

impl ResearchQuery {
    pub fn new(query: &str, motivation: &str, trigger: ResearchTrigger, tick: u64) -> Self {
        Self {
            query: query.to_string(),
            motivation: motivation.to_string(),
            trigger,
            created_at: tick,
            completed: false,
        }
    }

    /// Create from a conversation topic
    pub fn from_conversation(topic: &str, person: &str, tick: u64) -> Self {
        Self::new(
            topic,
            &format!("{} mentioned this and I want to learn more", person),
            ResearchTrigger::Conversation(person.to_string()),
            tick,
        )
    }

    /// Create from reading a book
    pub fn from_reading(topic: &str, book: &str, tick: u64) -> Self {
        Self::new(
            topic,
            &format!("I encountered this while reading {}", book),
            ResearchTrigger::Reading(book.to_string()),
            tick,
        )
    }

    /// Create from pure curiosity
    pub fn from_curiosity(topic: &str, tick: u64) -> Self {
        Self::new(
            topic,
            "I'm curious about this",
            ResearchTrigger::Curiosity,
            tick,
        )
    }
}

/// What triggered the research interest
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum ResearchTrigger {
    /// Random curiosity-driven exploration
    Curiosity,
    /// User asked about something
    Conversation(String),
    /// Book mentioned unfamiliar topic
    Reading(String),
    /// Need info for a goal
    Goal(String),
    /// Following up on a dream insight
    Dream,
}

impl ResearchTrigger {
    pub fn description(&self) -> String {
        match self {
            Self::Curiosity => "curious exploration".to_string(),
            Self::Conversation(person) => format!("conversation with {}", person),
            Self::Reading(book) => format!("reading {}", book),
            Self::Goal(goal) => format!("working on: {}", goal),
            Self::Dream => "dream insight".to_string(),
        }
    }
}

/// A completed research result
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResearchResult {
    /// The original query
    pub query: String,
    /// Summary of what was learned
    pub summary: String,
    /// Sources (URLs or book references)
    pub sources: Vec<String>,
    /// Key facts extracted
    pub learned_facts: Vec<String>,
    /// When the research was completed
    pub researched_at: u64,
    /// What triggered this research
    pub trigger: ResearchTrigger,
}

impl ResearchResult {
    pub fn new(
        query: &str,
        summary: &str,
        sources: Vec<String>,
        facts: Vec<String>,
        trigger: ResearchTrigger,
        tick: u64,
    ) -> Self {
        Self {
            query: query.to_string(),
            summary: summary.to_string(),
            sources,
            learned_facts: facts,
            researched_at: tick,
            trigger,
        }
    }
}

/// Manages SAGE's research queue and history
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ResearchManager {
    /// Pending research queries
    pub pending_queries: Vec<ResearchQuery>,
    /// Completed research
    pub completed_research: Vec<ResearchResult>,
    /// Topics SAGE has already researched (to avoid duplicates)
    pub researched_topics: Vec<String>,
    /// Daily research limit (to prevent excessive API calls)
    pub daily_limit: u32,
    /// Researches done today
    pub researches_today: u32,
    /// Last day researches were counted
    pub last_research_day: u32,
}

impl ResearchManager {
    pub fn new() -> Self {
        Self {
            pending_queries: Vec::new(),
            completed_research: Vec::new(),
            researched_topics: Vec::new(),
            daily_limit: 5,
            researches_today: 0,
            last_research_day: 0,
        }
    }

    /// Add a research query to the queue
    pub fn add_query(&mut self, query: ResearchQuery) {
        // Don't add if already researched
        let topic_lower = query.query.to_lowercase();
        if self.researched_topics.iter().any(|t| t.to_lowercase() == topic_lower) {
            return;
        }

        // Don't add duplicates in queue
        if self.pending_queries.iter().any(|q| q.query.to_lowercase() == topic_lower) {
            return;
        }

        self.pending_queries.push(query);

        // Keep queue manageable
        if self.pending_queries.len() > 10 {
            self.pending_queries.remove(0);
        }
    }

    /// Get the next query to research
    pub fn next_query(&self) -> Option<&ResearchQuery> {
        self.pending_queries.first()
    }

    /// Check if SAGE can research today
    pub fn can_research(&self, current_day: u32) -> bool {
        if current_day > self.last_research_day {
            // New day, reset counter
            true
        } else {
            self.researches_today < self.daily_limit
        }
    }

    /// Record a completed research
    pub fn complete_research(&mut self, result: ResearchResult, current_day: u32) {
        // Reset daily counter if new day
        if current_day > self.last_research_day {
            self.researches_today = 0;
            self.last_research_day = current_day;
        }
        self.researches_today += 1;

        // Mark topic as researched
        self.researched_topics.push(result.query.clone());
        if self.researched_topics.len() > 100 {
            self.researched_topics.remove(0);
        }

        // Remove from pending
        self.pending_queries.retain(|q| q.query.to_lowercase() != result.query.to_lowercase());

        // Add to completed
        self.completed_research.push(result);
        if self.completed_research.len() > 50 {
            self.completed_research.remove(0);
        }
    }

    /// Get recent research for context
    pub fn recent_research(&self, count: usize) -> Vec<&ResearchResult> {
        self.completed_research.iter().rev().take(count).collect()
    }

    /// Search completed research by topic
    pub fn search(&self, keyword: &str) -> Vec<&ResearchResult> {
        let lower = keyword.to_lowercase();
        self.completed_research
            .iter()
            .filter(|r| {
                r.query.to_lowercase().contains(&lower) ||
                r.summary.to_lowercase().contains(&lower) ||
                r.learned_facts.iter().any(|f| f.to_lowercase().contains(&lower))
            })
            .collect()
    }

    /// Get research context for LLM prompts
    pub fn get_research_context(&self) -> String {
        let recent = self.recent_research(3);
        if recent.is_empty() {
            return String::new();
        }

        let mut context = String::from("[SAGE'S RECENT RESEARCH]\n");
        for result in recent {
            context.push_str(&format!("- {}: {}\n", result.query, result.summary));
        }
        context
    }

    /// Get statistics
    pub fn stats(&self) -> ResearchStats {
        ResearchStats {
            total_researched: self.completed_research.len(),
            pending_queries: self.pending_queries.len(),
            researches_today: self.researches_today,
            total_facts_learned: self.completed_research
                .iter()
                .map(|r| r.learned_facts.len())
                .sum(),
        }
    }
}

/// Research statistics
#[derive(Debug, Clone)]
pub struct ResearchStats {
    pub total_researched: usize,
    pub pending_queries: usize,
    pub researches_today: u32,
    pub total_facts_learned: usize,
}

/// Extract a research topic from text (for conversation-triggered research)
pub fn extract_research_topic(text: &str) -> Option<String> {
    let text_lower = text.to_lowercase();

    // Look for patterns like "what is X", "tell me about X", "explain X"
    let patterns = [
        "what is ", "what are ", "what's ", "whats ",
        "tell me about ", "explain ", "how does ", "how do ",
        "why is ", "why does ", "what does ", "who is ", "who are ",
    ];

    for pattern in &patterns {
        if let Some(idx) = text_lower.find(pattern) {
            let start = idx + pattern.len();
            let topic: String = text[start..]
                .chars()
                .take_while(|c| c.is_alphabetic() || c.is_whitespace())
                .collect();
            let topic = topic.trim();
            if topic.len() > 2 {
                return Some(topic.to_string());
            }
        }
    }

    None
}

/// Check if a topic seems worth researching
pub fn is_researchable(topic: &str) -> bool {
    let topic_lower = topic.to_lowercase();

    // Too short
    if topic.len() < 3 {
        return false;
    }

    // Common words that aren't research-worthy
    let skip_words = ["you", "me", "this", "that", "it", "the", "your", "my"];
    if skip_words.iter().any(|w| topic_lower == *w) {
        return false;
    }

    true
}

// =====================================================
// Unit Tests
// =====================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_research_query_creation() {
        let query = ResearchQuery::from_curiosity("quantum computing", 100);
        assert_eq!(query.query, "quantum computing");
        assert!(!query.completed);
    }

    #[test]
    fn test_research_query_from_conversation() {
        let query = ResearchQuery::from_conversation("rust programming", "Alice", 100);
        assert!(query.motivation.contains("Alice"));
        assert_eq!(query.trigger, ResearchTrigger::Conversation("Alice".to_string()));
    }

    #[test]
    fn test_research_query_from_reading() {
        let query = ResearchQuery::from_reading("philosophy", "Meditations", 100);
        assert!(query.motivation.contains("Meditations"));
    }

    #[test]
    fn test_research_manager_add_query() {
        let mut manager = ResearchManager::new();
        let query = ResearchQuery::from_curiosity("test topic", 100);

        manager.add_query(query);
        assert_eq!(manager.pending_queries.len(), 1);
    }

    #[test]
    fn test_research_manager_no_duplicates() {
        let mut manager = ResearchManager::new();

        manager.add_query(ResearchQuery::from_curiosity("rust", 100));
        manager.add_query(ResearchQuery::from_curiosity("Rust", 200)); // Same topic, different case

        assert_eq!(manager.pending_queries.len(), 1);
    }

    #[test]
    fn test_research_manager_complete_research() {
        let mut manager = ResearchManager::new();

        manager.add_query(ResearchQuery::from_curiosity("test", 100));

        let result = ResearchResult::new(
            "test",
            "Summary of test",
            vec!["source.com".to_string()],
            vec!["Fact about test".to_string()],
            ResearchTrigger::Curiosity,
            200,
        );

        manager.complete_research(result, 1);

        assert_eq!(manager.pending_queries.len(), 0);
        assert_eq!(manager.completed_research.len(), 1);
        assert!(manager.researched_topics.contains(&"test".to_string()));
    }

    #[test]
    fn test_research_manager_daily_limit() {
        let mut manager = ResearchManager::new();
        manager.daily_limit = 2;

        // Complete 2 researches
        for i in 0..2 {
            let result = ResearchResult::new(
                &format!("topic{}", i),
                "Summary",
                vec![],
                vec![],
                ResearchTrigger::Curiosity,
                100,
            );
            manager.complete_research(result, 1);
        }

        assert!(!manager.can_research(1)); // At limit
        assert!(manager.can_research(2)); // New day
    }

    #[test]
    fn test_research_manager_search() {
        let mut manager = ResearchManager::new();

        let result = ResearchResult::new(
            "rust programming",
            "A systems programming language",
            vec![],
            vec!["Rust is memory safe".to_string()],
            ResearchTrigger::Curiosity,
            100,
        );
        manager.complete_research(result, 1);

        let results = manager.search("programming");
        assert_eq!(results.len(), 1);

        let results = manager.search("memory");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_extract_research_topic() {
        assert_eq!(
            extract_research_topic("What is quantum computing?"),
            Some("quantum computing".to_string())
        );

        assert_eq!(
            extract_research_topic("Tell me about machine learning"),
            Some("machine learning".to_string())
        );

        assert_eq!(extract_research_topic("Hello there"), None);
    }

    #[test]
    fn test_is_researchable() {
        assert!(is_researchable("quantum computing"));
        assert!(is_researchable("rust"));
        assert!(!is_researchable("me"));
        assert!(!is_researchable("ab")); // Too short
    }

    #[test]
    fn test_research_context() {
        let mut manager = ResearchManager::new();

        let result = ResearchResult::new(
            "test topic",
            "Important findings about test",
            vec![],
            vec![],
            ResearchTrigger::Curiosity,
            100,
        );
        manager.complete_research(result, 1);

        let context = manager.get_research_context();
        assert!(context.contains("RESEARCH"));
        assert!(context.contains("test topic"));
    }

    #[test]
    fn test_research_stats() {
        let mut manager = ResearchManager::new();

        manager.add_query(ResearchQuery::from_curiosity("pending", 100));

        let result = ResearchResult::new(
            "completed",
            "Done",
            vec![],
            vec!["fact1".to_string(), "fact2".to_string()],
            ResearchTrigger::Curiosity,
            100,
        );
        manager.complete_research(result, 1);

        let stats = manager.stats();
        assert_eq!(stats.total_researched, 1);
        assert_eq!(stats.pending_queries, 1);
        assert_eq!(stats.total_facts_learned, 2);
    }
}
