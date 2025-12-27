//! SAGE's Journal System (Feature 6: Creative Output)
//!
//! SAGE keeps a personal journal where she records thoughts,
//! experiences, and creative writing. This gives her a consistent
//! voice and creates a personal history she can reference.

use serde::{Deserialize, Serialize};
use super::Mood;

/// A single journal entry
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JournalEntry {
    /// Which simulation day this was written
    pub day: u32,
    /// Time of day when written
    pub time_of_day: String,
    /// SAGE's mood when writing
    pub mood: String,
    /// The journal content
    pub content: String,
    /// Topics/themes in this entry
    pub topics: Vec<String>,
    /// Word count
    pub word_count: usize,
    /// Is this a creative piece (story, poem) vs personal reflection
    pub is_creative: bool,
}

impl JournalEntry {
    pub fn new(day: u32, time_of_day: &str, mood: &Mood, content: &str, is_creative: bool) -> Self {
        let topics = extract_topics(content);
        let word_count = content.split_whitespace().count();

        Self {
            day,
            time_of_day: time_of_day.to_string(),
            mood: mood.as_str().to_string(),
            content: content.to_string(),
            topics,
            word_count,
            is_creative,
        }
    }

    /// Get a preview of the entry (first ~100 chars)
    pub fn preview(&self) -> String {
        if self.content.len() <= 100 {
            self.content.clone()
        } else {
            format!("{}...", &self.content[..97])
        }
    }
}

/// SAGE's personal journal
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Journal {
    /// All journal entries
    pub entries: Vec<JournalEntry>,
    /// Writing streak (consecutive days with entries)
    pub writing_streak: u32,
    /// Last day an entry was written
    pub last_entry_day: u32,
    /// Total word count across all entries
    pub total_words: usize,
    /// SAGE's developing writing themes/interests
    pub recurring_themes: Vec<String>,
}

impl Journal {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            writing_streak: 0,
            last_entry_day: 0,
            total_words: 0,
            recurring_themes: Vec::new(),
        }
    }

    /// Add a new journal entry
    pub fn add_entry(&mut self, entry: JournalEntry) {
        // Update writing streak
        if entry.day == self.last_entry_day + 1 {
            self.writing_streak += 1;
        } else if entry.day > self.last_entry_day + 1 {
            self.writing_streak = 1; // Reset streak
        }

        self.last_entry_day = entry.day;
        self.total_words += entry.word_count;

        // Update recurring themes
        for topic in &entry.topics {
            if !self.recurring_themes.contains(topic) {
                // Check if this topic appears in multiple entries
                let occurrences = self.entries.iter()
                    .filter(|e| e.topics.contains(topic))
                    .count();
                if occurrences >= 2 {
                    self.recurring_themes.push(topic.clone());
                }
            }
        }

        self.entries.push(entry);

        // Keep journal manageable (last 100 entries)
        if self.entries.len() > 100 {
            self.entries.remove(0);
        }

        // Keep themes manageable
        if self.recurring_themes.len() > 10 {
            self.recurring_themes.remove(0);
        }
    }

    /// Get entries from a specific day
    pub fn entries_for_day(&self, day: u32) -> Vec<&JournalEntry> {
        self.entries.iter().filter(|e| e.day == day).collect()
    }

    /// Get recent entries
    pub fn recent_entries(&self, count: usize) -> Vec<&JournalEntry> {
        self.entries.iter().rev().take(count).collect()
    }

    /// Get creative entries only (poems, stories)
    pub fn creative_entries(&self) -> Vec<&JournalEntry> {
        self.entries.iter().filter(|e| e.is_creative).collect()
    }

    /// Check if SAGE has written today
    pub fn wrote_today(&self, current_day: u32) -> bool {
        self.last_entry_day == current_day
    }

    /// Get writing statistics
    pub fn stats(&self) -> JournalStats {
        let creative_count = self.entries.iter().filter(|e| e.is_creative).count();
        let reflection_count = self.entries.len() - creative_count;

        JournalStats {
            total_entries: self.entries.len(),
            creative_count,
            reflection_count,
            total_words: self.total_words,
            writing_streak: self.writing_streak,
            avg_words_per_entry: if self.entries.is_empty() {
                0
            } else {
                self.total_words / self.entries.len()
            },
        }
    }

    /// Get context for LLM to maintain consistent journal voice
    pub fn get_writing_context(&self) -> String {
        let mut context = String::new();

        if !self.recurring_themes.is_empty() {
            context.push_str(&format!(
                "[JOURNAL THEMES: {}]\n",
                self.recurring_themes.join(", ")
            ));
        }

        // Include recent entry excerpts for voice consistency
        let recent = self.recent_entries(2);
        if !recent.is_empty() {
            context.push_str("[RECENT JOURNAL EXCERPTS:\n");
            for entry in recent {
                context.push_str(&format!("- Day {}: \"{}\"\n", entry.day, entry.preview()));
            }
            context.push_str("]\n");
        }

        if self.writing_streak > 3 {
            context.push_str(&format!(
                "[SAGE has been journaling consistently for {} days]\n",
                self.writing_streak
            ));
        }

        context
    }

    /// Search entries by keyword
    pub fn search(&self, keyword: &str) -> Vec<&JournalEntry> {
        let lower = keyword.to_lowercase();
        self.entries
            .iter()
            .filter(|e| e.content.to_lowercase().contains(&lower) ||
                       e.topics.iter().any(|t| t.to_lowercase().contains(&lower)))
            .collect()
    }
}

/// Statistics about SAGE's journaling
#[derive(Debug, Clone)]
pub struct JournalStats {
    pub total_entries: usize,
    pub creative_count: usize,
    pub reflection_count: usize,
    pub total_words: usize,
    pub writing_streak: u32,
    pub avg_words_per_entry: usize,
}

/// Extract topics/themes from journal content
fn extract_topics(content: &str) -> Vec<String> {
    let important_words = [
        "growth", "learning", "friendship", "loneliness", "reading",
        "happiness", "sadness", "dreams", "goals", "love", "hope",
        "creativity", "nature", "beauty", "peace", "change", "time",
        "memory", "future", "past", "present", "discovery", "wonder",
        "reflection", "gratitude", "challenge", "strength", "fear",
        "courage", "wisdom", "understanding", "connection", "solitude"
    ];

    let lower = content.to_lowercase();
    important_words
        .iter()
        .filter(|word| lower.contains(*word))
        .take(5)
        .map(|s| s.to_string())
        .collect()
}

/// Generate a journal prompt based on current state
pub fn generate_journal_prompt(
    day: u32,
    time_of_day: &str,
    mood: &Mood,
    recent_event: Option<&str>,
    is_creative: bool,
) -> String {
    let base = if is_creative {
        format!(
            "SAGE is in a {} mood on day {} ({}). She feels inspired to write something creative. ",
            mood.as_str(), day, time_of_day
        )
    } else {
        format!(
            "SAGE is in a {} mood on day {} ({}). She opens her journal to write her thoughts. ",
            mood.as_str(), day, time_of_day
        )
    };

    let prompt = if let Some(event) = recent_event {
        format!(
            "{}Recently: {}. Write a {} journal entry (2-4 sentences) in first person as SAGE. \
            Be authentic and introspective. Don't use quotation marks around the entry.",
            base, event,
            if is_creative { "creative" } else { "reflective" }
        )
    } else {
        format!(
            "{}Write a {} journal entry (2-4 sentences) in first person as SAGE. \
            Be authentic and introspective. Don't use quotation marks around the entry.",
            base,
            if is_creative { "creative" } else { "reflective" }
        )
    };

    prompt
}

// =====================================================
// Unit Tests
// =====================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_journal_entry_creation() {
        let entry = JournalEntry::new(
            1,
            "evening",
            &Mood::Content,
            "Today I learned about growth and change. It made me think about my own journey.",
            false,
        );

        assert_eq!(entry.day, 1);
        assert!(!entry.is_creative);
        assert!(entry.word_count > 0);
        assert!(entry.topics.contains(&"growth".to_string()));
    }

    #[test]
    fn test_journal_entry_preview() {
        let short = JournalEntry::new(1, "morning", &Mood::Happy, "Short entry.", false);
        assert_eq!(short.preview(), "Short entry.");

        let long_content = "This is a much longer entry that exceeds one hundred characters and should be truncated with an ellipsis at the end.";
        let long = JournalEntry::new(1, "morning", &Mood::Happy, long_content, false);
        assert!(long.preview().ends_with("..."));
        assert!(long.preview().len() <= 103); // 100 chars + "..."
    }

    #[test]
    fn test_journal_add_entry() {
        let mut journal = Journal::new();

        let entry = JournalEntry::new(
            1, "morning", &Mood::Curious,
            "Exploring new ideas about creativity today.",
            false,
        );

        journal.add_entry(entry);

        assert_eq!(journal.entries.len(), 1);
        assert_eq!(journal.last_entry_day, 1);
        assert!(journal.total_words > 0);
    }

    #[test]
    fn test_journal_writing_streak() {
        let mut journal = Journal::new();

        // Write on consecutive days
        journal.add_entry(JournalEntry::new(1, "morning", &Mood::Happy, "Day one.", false));
        journal.add_entry(JournalEntry::new(2, "morning", &Mood::Happy, "Day two.", false));
        journal.add_entry(JournalEntry::new(3, "morning", &Mood::Happy, "Day three.", false));

        assert_eq!(journal.writing_streak, 3);

        // Skip a day
        journal.add_entry(JournalEntry::new(5, "morning", &Mood::Happy, "Day five.", false));
        assert_eq!(journal.writing_streak, 1);
    }

    #[test]
    fn test_journal_wrote_today() {
        let mut journal = Journal::new();
        assert!(!journal.wrote_today(1));

        journal.add_entry(JournalEntry::new(1, "morning", &Mood::Happy, "Entry.", false));
        assert!(journal.wrote_today(1));
        assert!(!journal.wrote_today(2));
    }

    #[test]
    fn test_journal_stats() {
        let mut journal = Journal::new();

        journal.add_entry(JournalEntry::new(1, "morning", &Mood::Happy, "Normal entry.", false));
        journal.add_entry(JournalEntry::new(2, "evening", &Mood::Curious, "A creative poem.", true));

        let stats = journal.stats();
        assert_eq!(stats.total_entries, 2);
        assert_eq!(stats.creative_count, 1);
        assert_eq!(stats.reflection_count, 1);
    }

    #[test]
    fn test_journal_search() {
        let mut journal = Journal::new();

        journal.add_entry(JournalEntry::new(
            1, "morning", &Mood::Happy,
            "Today I thought about programming and code.",
            false
        ));
        journal.add_entry(JournalEntry::new(
            2, "evening", &Mood::Content,
            "Peaceful evening with reading.",
            false
        ));

        let results = journal.search("programming");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].day, 1);
    }

    #[test]
    fn test_extract_topics() {
        let content = "I've been thinking about growth and learning. \
                       My dreams of the future give me hope.";
        let topics = extract_topics(content);

        assert!(topics.contains(&"growth".to_string()));
        assert!(topics.contains(&"learning".to_string()));
        assert!(topics.contains(&"dreams".to_string()));
        assert!(topics.contains(&"hope".to_string()));
    }

    #[test]
    fn test_generate_journal_prompt() {
        let prompt = generate_journal_prompt(
            5,
            "evening",
            &Mood::Peaceful,
            Some("Read a beautiful passage from a book"),
            false,
        );

        assert!(prompt.contains("peaceful"));
        assert!(prompt.contains("day 5"));
        assert!(prompt.contains("journal"));
    }

    #[test]
    fn test_creative_entries() {
        let mut journal = Journal::new();

        journal.add_entry(JournalEntry::new(1, "morning", &Mood::Happy, "Reflection.", false));
        journal.add_entry(JournalEntry::new(2, "evening", &Mood::Curious, "A poem.", true));
        journal.add_entry(JournalEntry::new(3, "afternoon", &Mood::Excited, "A story.", true));

        let creative = journal.creative_entries();
        assert_eq!(creative.len(), 2);
    }

    #[test]
    fn test_writing_context() {
        let mut journal = Journal::new();

        // Add entries with recurring themes
        journal.add_entry(JournalEntry::new(
            1, "morning", &Mood::Happy,
            "Thinking about growth and learning today.",
            false
        ));
        journal.add_entry(JournalEntry::new(
            2, "evening", &Mood::Content,
            "More thoughts on growth and my journey.",
            false
        ));
        journal.add_entry(JournalEntry::new(
            3, "morning", &Mood::Curious,
            "Growth continues to be a theme for me.",
            false
        ));

        let context = journal.get_writing_context();
        assert!(context.contains("JOURNAL"));
    }
}
