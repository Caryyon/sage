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

/// SAGE's reading progress for a book
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReadingProgress {
    pub book_id: String,
    pub current_page: usize,
    pub started_day: u32,
    pub last_read_day: u32,
    pub finished: bool,
    /// Quotes or passages SAGE found meaningful
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
            favorite_passages: Vec::new(),
            insights: Vec::new(),
        }
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
}
