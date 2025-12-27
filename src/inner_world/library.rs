//! SAGE's Library - Real books she can read and learn from
//!
//! Books are stored as text files in the `books/` directory.
//! SAGE tracks her reading progress and extracts insights to remember.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// A book SAGE can read
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Book {
    pub id: String,
    pub title: String,
    pub author: String,
    pub genre: String,
    /// The full text split into pages (roughly 2000 chars each)
    pub pages: Vec<String>,
    /// Brief description of the book
    pub description: String,
}

impl Book {
    /// Load a book from a text file
    /// Expected format:
    /// Line 1: Title
    /// Line 2: Author
    /// Line 3: Genre
    /// Line 4: Description
    /// Line 5: ---
    /// Rest: Content
    pub fn load_from_file(path: &Path) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read book file: {}", e))?;

        let lines: Vec<&str> = content.lines().collect();
        if lines.len() < 6 {
            return Err("Book file too short - needs title, author, genre, description, ---, content".to_string());
        }

        let title = lines[0].trim().to_string();
        let author = lines[1].trim().to_string();
        let genre = lines[2].trim().to_string();
        let description = lines[3].trim().to_string();

        // Find the separator
        let content_start = lines.iter().position(|l| l.trim() == "---")
            .ok_or("Book file missing --- separator")?;

        // Join the rest as content
        let book_content: String = lines[content_start + 1..].join("\n");

        // Split into pages (~2000 chars each, breaking at paragraph boundaries)
        let pages = split_into_pages(&book_content, 2000);

        let id = path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        Ok(Book {
            id,
            title,
            author,
            genre,
            pages,
            description,
        })
    }

    /// Get a specific page (0-indexed)
    pub fn get_page(&self, page_num: usize) -> Option<&str> {
        self.pages.get(page_num).map(|s| s.as_str())
    }

    /// Total number of pages
    pub fn total_pages(&self) -> usize {
        self.pages.len()
    }
}

/// Split text into pages, trying to break at paragraph boundaries
fn split_into_pages(text: &str, chars_per_page: usize) -> Vec<String> {
    let mut pages = Vec::new();
    let mut current_page = String::new();

    for paragraph in text.split("\n\n") {
        let paragraph = paragraph.trim();
        if paragraph.is_empty() {
            continue;
        }

        // If adding this paragraph would exceed the limit, start a new page
        if !current_page.is_empty() && current_page.len() + paragraph.len() > chars_per_page {
            pages.push(current_page.trim().to_string());
            current_page = String::new();
        }

        if !current_page.is_empty() {
            current_page.push_str("\n\n");
        }
        current_page.push_str(paragraph);
    }

    // Don't forget the last page
    if !current_page.is_empty() {
        pages.push(current_page.trim().to_string());
    }

    // If the book is empty, add a placeholder
    if pages.is_empty() {
        pages.push("(This book appears to be empty)".to_string());
    }

    pages
}

/// A notable passage from a book with context
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NotablePassage {
    /// The passage text
    pub text: String,
    /// Which page this was on (0-indexed)
    pub page: usize,
    /// Why SAGE found this meaningful
    pub reason: String,
    /// Emotional resonance (0.0 - 1.0)
    pub resonance: f64,
    /// Related topics/themes
    pub topics: Vec<String>,
}

impl NotablePassage {
    pub fn new(text: &str, page: usize, reason: &str, resonance: f64, topics: Vec<String>) -> Self {
        Self {
            text: text.to_string(),
            page,
            reason: reason.to_string(),
            resonance,
            topics,
        }
    }
}

/// SAGE's opinion about a book
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BookOpinion {
    pub book_id: String,
    /// Overall rating 0.0 - 1.0
    pub overall_rating: f64,
    /// Themes SAGE identified in the book
    pub themes: Vec<String>,
    /// How the book relates to SAGE personally
    pub personal_connections: Vec<String>,
    /// When the opinion was formed/updated
    pub formed_at: u32,
    /// Would SAGE recommend this book?
    pub would_recommend: bool,
    /// One-sentence summary in SAGE's words
    pub summary: String,
}

impl BookOpinion {
    pub fn new(book_id: &str, day: u32) -> Self {
        Self {
            book_id: book_id.to_string(),
            overall_rating: 0.5, // Neutral starting point
            themes: Vec::new(),
            personal_connections: Vec::new(),
            formed_at: day,
            would_recommend: false,
            summary: String::new(),
        }
    }

    /// Update opinion based on reading progress
    pub fn update_from_progress(&mut self, progress: &ReadingProgress, insights_count: usize, day: u32) {
        // More insights = higher rating
        let insight_bonus = (insights_count as f64 * 0.05).min(0.3);

        // More favorite passages = higher rating
        let passage_bonus = (progress.notable_passages.len() as f64 * 0.03).min(0.2);

        self.overall_rating = (0.5 + insight_bonus + passage_bonus).min(1.0);
        self.would_recommend = self.overall_rating > 0.7;
        self.formed_at = day;
    }
}

/// SAGE's reading progress for a book
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReadingProgress {
    pub book_id: String,
    pub current_page: usize,
    pub started_day: u32,
    pub last_read_day: u32,
    pub finished: bool,
    /// Notable passages with context (replaces simple favorite_passages)
    pub notable_passages: Vec<NotablePassage>,
    /// Legacy: simple favorite passages (for backwards compatibility)
    pub favorite_passages: Vec<String>,
    /// Insights SAGE extracted while reading
    pub insights: Vec<String>,
}

impl ReadingProgress {
    pub fn new(book_id: &str, day: u32) -> Self {
        Self {
            book_id: book_id.to_string(),
            current_page: 0,
            started_day: day,
            last_read_day: day,
            finished: false,
            notable_passages: Vec::new(),
            favorite_passages: Vec::new(),
            insights: Vec::new(),
        }
    }

    /// Add a notable passage with context
    pub fn add_notable_passage(&mut self, passage: NotablePassage) {
        // Also add to legacy favorite_passages for backwards compat
        if passage.resonance > 0.7 {
            self.favorite_passages.push(passage.text.clone());
        }
        self.notable_passages.push(passage);
    }
}

/// SAGE's personal library
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Library {
    /// All available books (loaded from files)
    #[serde(skip)]
    pub books: HashMap<String, Book>,
    /// Reading progress for each book SAGE has started
    pub reading_progress: HashMap<String, ReadingProgress>,
    /// SAGE's opinions about books she's read
    pub opinions: HashMap<String, BookOpinion>,
    /// Currently selected book (if any)
    pub current_book: Option<String>,
}

impl Default for Library {
    fn default() -> Self {
        Self::new()
    }
}

impl Library {
    pub fn new() -> Self {
        Self {
            books: HashMap::new(),
            reading_progress: HashMap::new(),
            opinions: HashMap::new(),
            current_book: None,
        }
    }

    /// Load all books from the books/ directory
    pub fn load_books(&mut self, books_dir: &str) -> Result<usize, String> {
        let path = Path::new(books_dir);
        if !path.exists() {
            fs::create_dir_all(path)
                .map_err(|e| format!("Failed to create books directory: {}", e))?;
            return Ok(0);
        }

        let mut count = 0;
        for entry in fs::read_dir(path).map_err(|e| format!("Failed to read books dir: {}", e))? {
            let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
            let file_path = entry.path();

            if file_path.extension().and_then(|s| s.to_str()) == Some("txt") {
                match Book::load_from_file(&file_path) {
                    Ok(book) => {
                        println!("📚 Loaded book: \"{}\" by {} ({} pages)",
                            book.title, book.author, book.total_pages());
                        self.books.insert(book.id.clone(), book);
                        count += 1;
                    }
                    Err(e) => {
                        eprintln!("⚠️  Failed to load {:?}: {}", file_path, e);
                    }
                }
            }
        }

        Ok(count)
    }

    /// List all available books
    pub fn list_books(&self) -> Vec<&Book> {
        self.books.values().collect()
    }

    /// Get a specific book
    pub fn get_book(&self, book_id: &str) -> Option<&Book> {
        self.books.get(book_id)
    }

    /// Start or continue reading a book
    pub fn select_book(&mut self, book_id: &str, current_day: u32) -> Option<&Book> {
        if self.books.contains_key(book_id) {
            self.current_book = Some(book_id.to_string());

            // Create reading progress if this is a new book
            if !self.reading_progress.contains_key(book_id) {
                self.reading_progress.insert(
                    book_id.to_string(),
                    ReadingProgress::new(book_id, current_day)
                );
            }

            self.books.get(book_id)
        } else {
            None
        }
    }

    /// Read the current page of the selected book
    pub fn read_current_page(&self) -> Option<(&Book, &str, usize)> {
        let book_id = self.current_book.as_ref()?;
        let book = self.books.get(book_id)?;
        let progress = self.reading_progress.get(book_id)?;
        let page_content = book.get_page(progress.current_page)?;

        Some((book, page_content, progress.current_page))
    }

    /// Advance to the next page, returns true if there are more pages
    pub fn turn_page(&mut self, current_day: u32) -> bool {
        if let Some(book_id) = &self.current_book.clone() {
            if let Some(book) = self.books.get(book_id) {
                if let Some(progress) = self.reading_progress.get_mut(book_id) {
                    progress.last_read_day = current_day;

                    if progress.current_page + 1 < book.total_pages() {
                        progress.current_page += 1;
                        return true;
                    } else {
                        progress.finished = true;
                        return false;
                    }
                }
            }
        }
        false
    }

    /// Add an insight from reading
    pub fn add_insight(&mut self, insight: &str) {
        if let Some(book_id) = &self.current_book.clone() {
            if let Some(progress) = self.reading_progress.get_mut(book_id) {
                progress.insights.push(insight.to_string());
            }
        }
    }

    /// Add a favorite passage
    pub fn add_favorite_passage(&mut self, passage: &str) {
        if let Some(book_id) = &self.current_book.clone() {
            if let Some(progress) = self.reading_progress.get_mut(book_id) {
                progress.favorite_passages.push(passage.to_string());
            }
        }
    }

    /// Get reading status summary
    pub fn reading_status(&self) -> String {
        if let Some(book_id) = &self.current_book {
            if let Some(book) = self.books.get(book_id) {
                if let Some(progress) = self.reading_progress.get(book_id) {
                    if progress.finished {
                        return format!("Finished reading \"{}\" by {}", book.title, book.author);
                    } else {
                        return format!(
                            "Reading \"{}\" by {} (page {}/{})",
                            book.title, book.author,
                            progress.current_page + 1, book.total_pages()
                        );
                    }
                }
            }
        }
        "Not currently reading anything".to_string()
    }

    /// Get all insights SAGE has learned from books
    pub fn all_insights(&self) -> Vec<(String, String)> {
        let mut insights = Vec::new();
        for (book_id, progress) in &self.reading_progress {
            if let Some(book) = self.books.get(book_id) {
                for insight in &progress.insights {
                    insights.push((book.title.clone(), insight.clone()));
                }
            }
        }
        insights
    }

    /// Get insights from a specific book
    pub fn insights_from_book(&self, book_id: &str) -> Vec<String> {
        self.reading_progress
            .get(book_id)
            .map(|p| p.insights.clone())
            .unwrap_or_default()
    }

    /// Get all finished books
    pub fn finished_books(&self) -> Vec<&Book> {
        self.reading_progress
            .iter()
            .filter(|(_, progress)| progress.finished)
            .filter_map(|(id, _)| self.books.get(id))
            .collect()
    }

    /// Get all unread books
    pub fn unread_books(&self) -> Vec<&Book> {
        self.books
            .values()
            .filter(|book| !self.reading_progress.contains_key(&book.id))
            .collect()
    }

    /// Get books in progress (started but not finished)
    pub fn in_progress_books(&self) -> Vec<&Book> {
        self.reading_progress
            .iter()
            .filter(|(_, progress)| !progress.finished)
            .filter_map(|(id, _)| self.books.get(id))
            .collect()
    }

    /// Choose a new book intelligently based on mood, variety, and what's unread
    pub fn choose_book_for_mood(&self, mood: &str) -> Option<&Book> {
        let unread = self.unread_books();
        let in_progress = self.in_progress_books();

        // First, continue an in-progress book if there is one
        if let Some(book) = in_progress.first() {
            return Some(book);
        }

        // If no unread books, re-read a favorite (one with insights)
        if unread.is_empty() {
            return self.books.values()
                .filter(|b| {
                    self.reading_progress.get(&b.id)
                        .map(|p| !p.insights.is_empty())
                        .unwrap_or(false)
                })
                .next();
        }

        // Match mood to genre preferences
        let preferred_genres: Vec<&str> = match mood.to_lowercase().as_str() {
            "sad" | "lonely" => vec!["philosophy", "self-help", "fiction"],
            "curious" | "excited" => vec!["science", "technology", "programming"],
            "peaceful" | "content" => vec!["philosophy", "poetry", "fiction"],
            "anxious" | "frustrated" => vec!["self-help", "philosophy", "stoicism"],
            "tired" => vec!["fiction", "poetry", "short stories"],
            _ => vec![], // No preference
        };

        // Try to find a book matching preferred genre
        for genre_pref in &preferred_genres {
            if let Some(book) = unread.iter()
                .find(|b| b.genre.to_lowercase().contains(&genre_pref.to_lowercase()))
            {
                return Some(book);
            }
        }

        // Otherwise just pick the first unread book
        unread.first().copied()
    }

    /// Check if current book is finished
    pub fn current_book_finished(&self) -> bool {
        self.current_book
            .as_ref()
            .and_then(|id| self.reading_progress.get(id))
            .map(|p| p.finished)
            .unwrap_or(false)
    }

    /// Clear current book selection (after finishing)
    pub fn finish_current_book(&mut self) {
        self.current_book = None;
    }

    /// Get a summary of what SAGE has read for conversation context
    pub fn reading_summary(&self) -> String {
        let finished = self.finished_books();
        let in_progress = self.in_progress_books();
        let all_insights = self.all_insights();

        let mut summary = String::new();

        if !finished.is_empty() {
            let titles: Vec<_> = finished.iter().map(|b| format!("\"{}\"", b.title)).collect();
            summary.push_str(&format!("Books I've finished: {}. ", titles.join(", ")));
        }

        if !in_progress.is_empty() {
            if let Some(book) = in_progress.first() {
                if let Some(progress) = self.reading_progress.get(&book.id) {
                    summary.push_str(&format!(
                        "Currently reading \"{}\" (page {}/{}). ",
                        book.title,
                        progress.current_page + 1,
                        book.total_pages()
                    ));
                }
            }
        }

        if !all_insights.is_empty() {
            // Include recent insights
            let recent: Vec<_> = all_insights.iter().rev().take(3).collect();
            summary.push_str("Some thoughts from my reading: ");
            for (book, insight) in recent {
                summary.push_str(&format!("From \"{}\": {}. ", book, insight));
            }
        }

        if summary.is_empty() {
            "I haven't read any books yet, but I'd love to!".to_string()
        } else {
            summary
        }
    }

    // =====================================================
    // Feature 3: Deeper Book Integration - New Methods
    // =====================================================

    /// Add a notable passage with full context to the current book
    pub fn add_notable_passage_with_context(
        &mut self,
        text: &str,
        page: usize,
        reason: &str,
        resonance: f64,
        topics: Vec<String>,
    ) {
        if let Some(book_id) = &self.current_book.clone() {
            if let Some(progress) = self.reading_progress.get_mut(book_id) {
                let passage = NotablePassage::new(text, page, reason, resonance, topics);
                progress.add_notable_passage(passage);
            }
        }
    }

    /// Get or create opinion for a book
    pub fn get_or_create_opinion(&mut self, book_id: &str, day: u32) -> &mut BookOpinion {
        if !self.opinions.contains_key(book_id) {
            self.opinions.insert(book_id.to_string(), BookOpinion::new(book_id, day));
        }
        self.opinions.get_mut(book_id).unwrap()
    }

    /// Get opinion for a book (immutable)
    pub fn get_opinion(&self, book_id: &str) -> Option<&BookOpinion> {
        self.opinions.get(book_id)
    }

    /// Update opinion based on current reading progress
    pub fn update_opinion(&mut self, book_id: &str, day: u32) {
        if let Some(progress) = self.reading_progress.get(book_id) {
            let insights_count = progress.insights.len();
            let progress_clone = progress.clone();

            let opinion = self.get_or_create_opinion(book_id, day);
            opinion.update_from_progress(&progress_clone, insights_count, day);
        }
    }

    /// Find a relevant quote for a given topic
    /// Returns (book_title, passage_text, page_number) if found
    pub fn get_relevant_quote(&self, topic: &str) -> Option<(String, String, usize)> {
        let topic_lower = topic.to_lowercase();
        let keywords: Vec<&str> = topic_lower.split_whitespace().collect();

        let mut best_match: Option<(String, &NotablePassage, f64)> = None;

        for (book_id, progress) in &self.reading_progress {
            for passage in &progress.notable_passages {
                // Score based on topic match and resonance
                let text_lower = passage.text.to_lowercase();
                let mut score = passage.resonance;

                // Check if passage text contains keywords
                for keyword in &keywords {
                    if text_lower.contains(keyword) {
                        score += 0.2;
                    }
                }

                // Check if passage topics overlap
                for ptopic in &passage.topics {
                    if ptopic.to_lowercase().contains(&topic_lower) {
                        score += 0.3;
                    }
                    for keyword in &keywords {
                        if ptopic.to_lowercase().contains(keyword) {
                            score += 0.15;
                        }
                    }
                }

                if best_match.is_none() || score > best_match.as_ref().unwrap().2 {
                    if let Some(book) = self.books.get(book_id) {
                        best_match = Some((book.title.clone(), passage, score));
                    }
                }
            }
        }

        // Only return if score is above threshold
        best_match
            .filter(|(_, _, score)| *score > 0.5)
            .map(|(title, passage, _)| (title, passage.text.clone(), passage.page))
    }

    /// Get all quotes related to a topic from all books
    pub fn get_all_relevant_quotes(&self, topic: &str, max_results: usize) -> Vec<(String, String, usize)> {
        let topic_lower = topic.to_lowercase();
        let keywords: Vec<&str> = topic_lower.split_whitespace().collect();

        let mut results: Vec<(String, String, usize, f64)> = Vec::new();

        for (book_id, progress) in &self.reading_progress {
            for passage in &progress.notable_passages {
                let text_lower = passage.text.to_lowercase();
                let mut score = passage.resonance * 0.5;

                // Score based on keyword matches
                for keyword in &keywords {
                    if text_lower.contains(keyword) {
                        score += 0.25;
                    }
                }

                for ptopic in &passage.topics {
                    for keyword in &keywords {
                        if ptopic.to_lowercase().contains(keyword) {
                            score += 0.2;
                        }
                    }
                }

                if score > 0.3 {
                    if let Some(book) = self.books.get(book_id) {
                        results.push((book.title.clone(), passage.text.clone(), passage.page, score));
                    }
                }
            }
        }

        // Sort by score descending and take top N
        results.sort_by(|a, b| b.3.partial_cmp(&a.3).unwrap_or(std::cmp::Ordering::Equal));
        results.into_iter()
            .take(max_results)
            .map(|(title, text, page, _)| (title, text, page))
            .collect()
    }

    /// Get book-related context for a conversation topic
    /// This can be injected into LLM prompts when the topic relates to books SAGE has read
    pub fn get_book_context_for_topic(&self, topic: &str) -> Option<String> {
        let quotes = self.get_all_relevant_quotes(topic, 2);

        if quotes.is_empty() {
            return None;
        }

        let mut context = String::from("RELEVANT BOOK KNOWLEDGE:\n");
        for (title, text, page) in quotes {
            // Truncate long quotes
            let quote_preview = if text.len() > 150 {
                format!("{}...", &text[..150])
            } else {
                text
            };
            context.push_str(&format!("- From \"{}\" (p.{}): \"{}\"\n", title, page + 1, quote_preview));
        }

        Some(context)
    }

    /// Get all books with high opinions (would recommend)
    pub fn recommended_books(&self) -> Vec<(&Book, &BookOpinion)> {
        self.opinions
            .iter()
            .filter(|(_, opinion)| opinion.would_recommend)
            .filter_map(|(book_id, opinion)| {
                self.books.get(book_id).map(|book| (book, opinion))
            })
            .collect()
    }

    /// Get insights from books that relate to a topic
    pub fn get_insights_for_topic(&self, topic: &str) -> Vec<(String, String)> {
        let topic_lower = topic.to_lowercase();
        let keywords: Vec<&str> = topic_lower.split_whitespace().collect();

        let mut results = Vec::new();

        for (book_id, progress) in &self.reading_progress {
            for insight in &progress.insights {
                let insight_lower = insight.to_lowercase();
                let mut matches = false;

                for keyword in &keywords {
                    if insight_lower.contains(keyword) {
                        matches = true;
                        break;
                    }
                }

                if matches {
                    if let Some(book) = self.books.get(book_id) {
                        results.push((book.title.clone(), insight.clone()));
                    }
                }
            }
        }

        results
    }
}

// =====================================================
// Unit Tests
// =====================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_book() -> Book {
        Book {
            id: "test_book".to_string(),
            title: "Test Book".to_string(),
            author: "Test Author".to_string(),
            genre: "philosophy".to_string(),
            pages: vec![
                "Page one content about growth and mindset.".to_string(),
                "Page two discusses perseverance and effort.".to_string(),
                "Page three covers learning and wisdom.".to_string(),
            ],
            description: "A test book about growth".to_string(),
        }
    }

    fn create_test_library() -> Library {
        let mut library = Library::new();
        let book = create_test_book();
        library.books.insert(book.id.clone(), book);
        library
    }

    #[test]
    fn test_notable_passage_creation() {
        let passage = NotablePassage::new(
            "Growth requires patience",
            0,
            "Resonated with me",
            0.8,
            vec!["growth".to_string(), "patience".to_string()],
        );
        assert_eq!(passage.page, 0);
        assert!(passage.resonance > 0.7);
        assert_eq!(passage.topics.len(), 2);
    }

    #[test]
    fn test_book_opinion_creation() {
        let opinion = BookOpinion::new("test_book", 1);
        assert_eq!(opinion.overall_rating, 0.5);
        assert!(!opinion.would_recommend);
    }

    #[test]
    fn test_reading_progress_add_notable_passage() {
        let mut progress = ReadingProgress::new("test_book", 1);
        let passage = NotablePassage::new(
            "Important quote",
            2,
            "Very meaningful",
            0.9,
            vec!["meaning".to_string()],
        );

        progress.add_notable_passage(passage);

        assert_eq!(progress.notable_passages.len(), 1);
        // High resonance should add to legacy favorite_passages too
        assert_eq!(progress.favorite_passages.len(), 1);
    }

    #[test]
    fn test_opinion_update_from_progress() {
        let mut progress = ReadingProgress::new("test_book", 1);

        // Add some insights
        progress.insights.push("Insight 1".to_string());
        progress.insights.push("Insight 2".to_string());

        // Add notable passages
        let passage = NotablePassage::new("Quote", 0, "Good", 0.8, vec![]);
        progress.add_notable_passage(passage);

        let mut opinion = BookOpinion::new("test_book", 1);
        opinion.update_from_progress(&progress, progress.insights.len(), 1);

        // Should have higher rating due to insights and passages
        assert!(opinion.overall_rating > 0.5);
    }

    #[test]
    fn test_library_add_notable_passage_with_context() {
        let mut library = create_test_library();
        library.select_book("test_book", 1);

        library.add_notable_passage_with_context(
            "Growth mindset is key",
            0,
            "Foundational concept",
            0.85,
            vec!["growth".to_string(), "mindset".to_string()],
        );

        let progress = library.reading_progress.get("test_book").unwrap();
        assert_eq!(progress.notable_passages.len(), 1);
    }

    #[test]
    fn test_get_relevant_quote() {
        let mut library = create_test_library();
        library.select_book("test_book", 1);

        library.add_notable_passage_with_context(
            "Growth requires consistent effort and patience",
            1,
            "Key lesson",
            0.9,
            vec!["growth".to_string(), "effort".to_string(), "patience".to_string()],
        );

        let quote = library.get_relevant_quote("growth");
        assert!(quote.is_some());

        let (title, text, _page) = quote.unwrap();
        assert_eq!(title, "Test Book");
        assert!(text.contains("Growth"));
    }

    #[test]
    fn test_get_relevant_quote_no_match() {
        let library = create_test_library();
        let quote = library.get_relevant_quote("quantum physics");
        assert!(quote.is_none());
    }

    #[test]
    fn test_get_book_context_for_topic() {
        let mut library = create_test_library();
        library.select_book("test_book", 1);

        library.add_notable_passage_with_context(
            "Learning is a journey, not a destination",
            2,
            "Beautiful metaphor",
            0.95,
            vec!["learning".to_string(), "journey".to_string()],
        );

        let context = library.get_book_context_for_topic("learning");
        assert!(context.is_some());

        let context_str = context.unwrap();
        assert!(context_str.contains("RELEVANT BOOK KNOWLEDGE"));
        assert!(context_str.contains("Test Book"));
    }

    #[test]
    fn test_recommended_books() {
        let mut library = create_test_library();
        library.select_book("test_book", 1);

        // Add enough insights and passages to make it recommendable
        for i in 0..5 {
            library.reading_progress.get_mut("test_book").unwrap()
                .insights.push(format!("Insight {}", i));
        }
        for i in 0..4 {
            library.add_notable_passage_with_context(
                &format!("Great passage {}", i),
                i,
                "Loved it",
                0.8,
                vec!["wisdom".to_string()],
            );
        }

        // Update opinion
        library.update_opinion("test_book", 1);

        let recommended = library.recommended_books();
        assert_eq!(recommended.len(), 1);
        assert!(recommended[0].1.would_recommend);
    }

    #[test]
    fn test_get_insights_for_topic() {
        let mut library = create_test_library();
        library.select_book("test_book", 1);

        library.reading_progress.get_mut("test_book").unwrap()
            .insights.push("Growth mindset helps overcome challenges".to_string());
        library.reading_progress.get_mut("test_book").unwrap()
            .insights.push("Patience is essential for mastery".to_string());

        let insights = library.get_insights_for_topic("growth");
        assert_eq!(insights.len(), 1);
        assert!(insights[0].1.contains("Growth"));
    }

    #[test]
    fn test_split_into_pages() {
        let text = "Paragraph one about learning.\n\nParagraph two about growth.\n\nParagraph three about wisdom.";
        let pages = split_into_pages(text, 50);
        assert!(pages.len() >= 1);
    }
}
