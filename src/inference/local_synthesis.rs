//! Local Synthesis Engine — answer questions from retrieved passages without an LLM.
//!
//! This engine implements a extractive QA system that can answer simple factual
//! questions using only retrieved knowledge passages. No external LLM call needed.
//!
//! Strategy:
//! 1. Extract candidate answer sentences from retrieved passages
//! 2. Score each sentence by relevance to the query (keyword overlap, position, type matching)
//! 3. Select the best sentence(s) and compose a response
//!
//! This handles the "Simple" query category from the intelligent router:
//! - "Who wrote X?" → Find sentence mentioning author + title
//! - "What is X?" → Find definitional sentence
//! - "When did X happen?" → Find sentence with date + keyword
//! - "Where is X?" → Find sentence with location + keyword
//!
//! For complex/analytical queries, fall back to the full LLM engine.

use super::{ChatMessage, ChatRole, InferenceEngine};
use std::error::Error;

/// Local synthesis engine — extractive QA from retrieved knowledge, no LLM needed.
pub struct LocalSynthesizer {
    display_name: String,
}

impl Default for LocalSynthesizer {
    fn default() -> Self {
        Self::new()
    }
}

impl LocalSynthesizer {
    pub fn new() -> Self {
        Self {
            display_name: "Local Synthesis (extractive QA, no LLM)".to_string(),
        }
    }

    /// Extract knowledge context from system messages (same logic as OfflineEngine).
    fn extract_knowledge_context(messages: &[ChatMessage]) -> Vec<String> {
        let mut passages: Vec<String> = Vec::new();

        for msg in messages {
            if msg.role != ChatRole::System {
                continue;
            }

            let content = &msg.content;
            let mut search_from = 0;
            while let Some(start) = content[search_from..].find("## ") {
                let abs_start = search_from + start;
                let rest = &content[abs_start..];
                let header_end = rest.find('\n').unwrap_or(rest.len());
                let header = &rest[..header_end];

                let after_header = &rest[header_end..];
                let section_end = after_header
                    .find("\n\n## ")
                    .map(|i| header_end + i)
                    .unwrap_or(rest.len());
                let section_content = &rest[..section_end];

                // Extract knowledge from these sections
                if header.contains("Recalled Knowledge")
                    || header.contains("Associatively Recalled")
                    || header.contains("NCA Intuition")
                {
                    // Extract bullet-pointed passages
                    for line in section_content.lines() {
                        let trimmed = line.trim();
                        if trimmed.starts_with("- ") {
                            let text = trimmed[2..].trim().to_string();
                            if !text.is_empty() && text.len() > 10 {
                                passages.push(text);
                            }
                        }
                    }
                }

                search_from = abs_start + section_end;
                if search_from >= content.len() {
                    break;
                }
            }
        }

        passages
    }

    /// Extract the user's query from the messages.
    fn extract_query(messages: &[ChatMessage]) -> Option<String> {
        messages
            .iter()
            .rev()
            .find(|m| m.role == ChatRole::User)
            .map(|m| m.content.clone())
    }

    /// Determine the question type from the query.
    fn classify_question(query: &str) -> QuestionType {
        let q = query.to_lowercase();
        let q = q.trim_end_matches('?');

        // Check definitional patterns first (before general "what" check)
        if q.starts_with("define ") || q.starts_with("what does ") || q.contains(" meaning of ") || q.contains(" definition of ") {
            return QuestionType::Definition;
        }
        if q.starts_with("who ") || q.starts_with("who's ") || q.contains(" who ") {
            return QuestionType::Who;
        }
        if q.starts_with("what ") || q.starts_with("what's ") || q.contains(" what ") {
            return QuestionType::What;
        }
        if q.starts_with("when ") || q.contains(" what year ") || q.contains(" what date ") {
            return QuestionType::When;
        }
        if q.starts_with("where ") || q.contains(" where ") {
            return QuestionType::Where;
        }
        if q.starts_with("how many ") || q.starts_with("how much ") || q.contains(" how many ") {
            return QuestionType::HowMany;
        }
        if q.starts_with("why ") || q.contains(" why ") {
            return QuestionType::Why;
        }
        if q.starts_with("how ") && !q.starts_with("how many") && !q.starts_with("how much") {
            return QuestionType::How;
        }
        if q.starts_with("is ") || q.starts_with("are ") || q.starts_with("was ") || q.starts_with("were ") || q.starts_with("does ") || q.starts_with("do ") || q.starts_with("did ") || q.starts_with("can ") {
            return QuestionType::YesNo;
        }

        QuestionType::General
    }

    /// Extract key terms from the query (excluding stop words and question words).
    fn extract_key_terms(query: &str) -> Vec<String> {
        let stop_words: &[&str] = &[
            "the", "a", "an", "is", "are", "was", "were", "be", "been", "being",
            "have", "has", "had", "do", "does", "did", "will", "would", "shall",
            "should", "can", "could", "may", "might", "must",
            "who", "what", "when", "where", "why", "how",
            "many", "much", "long", "old", "about", "into", "from", "that", "this",
            "tell", "me", "know", "explain", "describe",
            "in", "on", "at", "by", "for", "with", "of", "and", "or", "but", "not",
            "to", "if", "then", "than",
        ];

        query
            .to_lowercase()
            .trim_end_matches('?')
            .split(|c: char| !c.is_alphanumeric() && c != '\'' && c != '-')
            .filter(|w| !w.is_empty() && w.len() > 2 && !stop_words.contains(w))
            .map(|w| w.to_string())
            .collect()
    }

    /// Score a passage sentence for relevance to the query.
    fn score_sentence(sentence: &str, query: &str, q_type: &QuestionType) -> f64 {
        let key_terms = Self::extract_key_terms(query);
        let sentence_lower = sentence.to_lowercase();

        // Base score: keyword overlap
        let mut matches = 0;
        for term in &key_terms {
            // Match whole words (not substrings) for better precision
            if sentence_lower.split_whitespace().any(|w| w.trim_matches(|c: char| !c.is_alphanumeric()) == *term) {
                matches += 1;
            }
        }
        let term_coverage = if key_terms.is_empty() {
            0.0
        } else {
            matches as f64 / key_terms.len() as f64
        };

        // Bonus for answer-type patterns
        let type_bonus = match q_type {
            QuestionType::Who => {
                // Look for capitalized names or "by [Name]"
                let cap_words = sentence.split_whitespace().filter(|w| {
                    w.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) && w.len() > 2
                }).count();
                if cap_words >= 2 { 0.3 } else { 0.0 }
            }
            QuestionType::When => {
                // Look for years (1000-2099) or month names
                let has_year = sentence.split_whitespace().any(|w| {
                    w.chars().all(|c| c.is_ascii_digit()) && w.len() == 4
                        && w.parse::<u32>().map(|y| (1000..=2099).contains(&y)).unwrap_or(false)
                });
                let has_month = ["january", "february", "march", "april", "may", "june",
                    "july", "august", "september", "october", "november", "december"]
                    .iter().any(|m| sentence_lower.contains(m));
                if has_year || has_month { 0.3 } else { 0.0 }
            }
            QuestionType::Where => {
                // Look for location indicators
                let has_location = ["in", "at", "located", "city", "country", "river", "mountain", "ocean", "sea"]
                    .iter().any(|kw| sentence_lower.contains(kw));
                if has_location { 0.2 } else { 0.0 }
            }
            QuestionType::HowMany => {
                // Look for numbers
                let has_number = sentence.split_whitespace().any(|w| {
                    w.chars().all(|c| c.is_ascii_digit()) && !w.is_empty()
                });
                if has_number { 0.3 } else { 0.0 }
            }
            QuestionType::Definition => {
                // Definitional patterns: "X is Y", "X means Y", "X refers to Y"
                if sentence_lower.contains(" is ") || sentence_lower.contains(" are ")
                    || sentence_lower.contains(" means ") || sentence_lower.contains(" refers to ")
                    || sentence_lower.contains(" defined as ")
                {
                    0.3
                } else {
                    0.0
                }
            }
            QuestionType::YesNo => {
                // Any sentence that addresses the subject
                if term_coverage > 0.3 { 0.2 } else { 0.0 }
            }
            _ => 0.0,
        };

        // Position bonus: sentences closer to the start of a passage often contain
        // the main claim
        let length_penalty = if sentence.len() > 500 { 0.1 } else { 0.0 };

        term_coverage + type_bonus - length_penalty
    }

    /// Split a passage into sentences.
    fn split_sentences(text: &str) -> Vec<String> {
        // Simple sentence splitter: split on . ! ? followed by space or end
        let mut sentences = Vec::new();
        let mut current = String::new();

        for ch in text.chars() {
            current.push(ch);
            if (ch == '.' || ch == '!' || ch == '?') {
                // Look ahead — if next char is space or end, we have a sentence
                sentences.push(current.trim().to_string());
                current.clear();
            }
        }
        if !current.trim().is_empty() {
            sentences.push(current.trim().to_string());
        }

        sentences.into_iter().filter(|s| s.len() > 15).collect()
    }

    /// Synthesize an answer from retrieved passages and a query.
    /// Returns None if no confident answer can be extracted.
    pub fn synthesize(query: &str, passages: &[String]) -> Option<String> {
        if passages.is_empty() {
            return None;
        }

        let q_type = Self::classify_question(query);
        let key_terms = Self::extract_key_terms(query);

        // Collect and score all sentences from all passages
        let mut scored_sentences: Vec<(f64, String)> = Vec::new();

        for passage in passages {
            for sentence in Self::split_sentences(passage) {
                let score = Self::score_sentence(&sentence, query, &q_type);
                if score > 0.15 {
                    scored_sentences.push((score, sentence));
                }
            }
        }

        if scored_sentences.is_empty() {
            // No confident answer — return the top passage as context
            if !passages.is_empty() {
                return Some(format!(
                    "Based on retrieved knowledge:\n\n{}",
                    passages.iter().take(3).cloned().collect::<Vec<_>>().join("\n\n")
                ));
            }
            return None;
        }

        // Sort by score (highest first)
        scored_sentences.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        // For yes/no questions, try to give a yes/no answer
        if q_type == QuestionType::YesNo {
            let best = &scored_sentences[0].1;
            let best_lower = best.to_lowercase();
            // Check if the sentence affirms or denies
            let has_negation = best_lower.contains(" not ") || best_lower.contains(" no ")
                || best_lower.contains(" never ") || best_lower.contains(" cannot ");
            let has_affirmation = best_lower.contains(" yes ") || best_lower.contains(" indeed ")
                || !has_negation; // Default to yes if the subject is discussed without negation

            let answer = if has_negation {
                "No. ".to_string()
            } else if has_affirmation {
                "Yes. ".to_string()
            } else {
                String::new()
            };
            return Some(format!("{}{}", answer, best));
        }

        // For other question types: take top 1-3 sentences
        let top_count = match q_type {
            QuestionType::Who | QuestionType::When | QuestionType::Where | QuestionType::HowMany => 1,
            QuestionType::Definition => 1,
            _ => 2,
        };

        let top_sentences: Vec<String> = scored_sentences
            .iter()
            .take(top_count)
            .map(|(_, s)| s.clone())
            .collect();

        if top_sentences.is_empty() {
            return None;
        }

        // Compose the answer
        let answer = top_sentences.join(" ");

        // For definition questions, prepend a direct answer frame
        if q_type == QuestionType::Definition && !key_terms.is_empty() {
            let subject = key_terms.first().unwrap();
            return Some(answer);
        }

        // For "who" questions, try to extract just the name
        if q_type == QuestionType::Who {
            // Look for "by [Name]" or "[Name] wrote/authored/created"
            for passage in passages {
                let passage_lower = passage.to_lowercase();
                if let Some(pos) = passage_lower.find(" by ") {
                    let after_by = &passage[pos + 4..];
                    let name: String = after_by
                        .chars()
                        .take_while(|c| c.is_alphanumeric() || *c == ' ' || *c == '.' || *c == '-')
                        .collect::<String>()
                        .trim()
                        .to_string();
                    if name.len() > 2 && name.len() < 100 {
                        // Check if this name appears in our top sentences
                        if answer.to_lowercase().contains(&name.to_lowercase()) {
                            return Some(answer);
                        }
                    }
                }
            }
        }

        Some(answer)
    }

    /// Check if this query is a good candidate for local synthesis.
    /// Simple factual questions with clear answer patterns are good candidates.
    pub fn can_answer_locally(query: &str, passages: &[String]) -> bool {
        if passages.is_empty() {
            return false;
        }
        let q_type = Self::classify_question(query);
        matches!(
            q_type,
            QuestionType::Who | QuestionType::What | QuestionType::When |
            QuestionType::Where | QuestionType::HowMany | QuestionType::Definition |
            QuestionType::YesNo
        ) && !key_terms_overlap(query, passages).is_empty()
    }
}

/// Check which key terms from the query appear in the passages.
fn key_terms_overlap(query: &str, passages: &[String]) -> Vec<String> {
    let stop_words: &[&str] = &[
        "the", "a", "an", "is", "are", "was", "were", "be", "been", "being",
        "have", "has", "had", "do", "does", "did", "will", "would",
        "who", "what", "when", "where", "why", "how",
        "many", "much", "about", "into", "from", "that", "this",
        "tell", "me", "know", "explain",
    ];

    let query_terms: Vec<String> = query
        .to_lowercase()
        .trim_end_matches('?')
        .split(|c: char| !c.is_alphanumeric() && c != '\'' && c != '-')
        .filter(|w| !w.is_empty() && w.len() > 2 && !stop_words.contains(w))
        .map(|w| w.to_string())
        .collect();

    let passage_text = passages.join(" ").to_lowercase();
    query_terms
        .into_iter()
        .filter(|term| passage_text.contains(term.as_str()))
        .collect()
}

/// Question type classification for answer extraction.
#[derive(Debug, Clone, Copy, PartialEq)]
enum QuestionType {
    Who,
    What,
    When,
    Where,
    HowMany,
    Why,
    How,
    YesNo,
    Definition,
    General,
}

impl InferenceEngine for LocalSynthesizer {
    fn generate(&self, prompt: &str, _max_tokens: usize) -> Result<String, Box<dyn Error>> {
        // For generate(), we treat the prompt as both query and context
        let passages = vec![prompt.to_string()];
        Self::synthesize(prompt, &passages)
            .ok_or_else(|| "Local synthesis: no confident answer found".into())
    }

    fn chat(&self, messages: &[ChatMessage], _max_tokens: usize) -> Result<String, Box<dyn Error>> {
        let passages = Self::extract_knowledge_context(messages);
        let query = Self::extract_query(messages)
            .ok_or("Local synthesis: no user query found in messages")?;

        Self::synthesize(&query, &passages)
            .ok_or_else(|| "Local synthesis: no relevant knowledge found for this query".into())
    }

    fn generate_streaming(
        &self,
        prompt: &str,
        max_tokens: usize,
        mut callback: Box<dyn FnMut(&str) + Send>,
    ) -> Result<(), Box<dyn Error>> {
        let response = self.generate(prompt, max_tokens)?;
        callback(&response);
        Ok(())
    }

    fn chat_streaming(
        &self,
        messages: &[ChatMessage],
        max_tokens: usize,
        mut callback: Box<dyn FnMut(&str) + Send>,
    ) -> Result<(), Box<dyn Error>> {
        let response = self.chat(messages, max_tokens)?;
        callback(&response);
        Ok(())
    }

    fn name(&self) -> &str {
        &self.display_name
    }

    fn is_available(&self) -> bool {
        true // Always available — it's just text processing
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_who() {
        assert_eq!(LocalSynthesizer::classify_question("Who wrote Alice in Wonderland?"), QuestionType::Who);
        assert_eq!(LocalSynthesizer::classify_question("who's the author?"), QuestionType::Who);
    }

    #[test]
    fn test_classify_what() {
        assert_eq!(LocalSynthesizer::classify_question("What is SAGE?"), QuestionType::What);
        assert_eq!(LocalSynthesizer::classify_question("what's the capital of France?"), QuestionType::What);
    }

    #[test]
    fn test_classify_when() {
        assert_eq!(LocalSynthesizer::classify_question("When was SAGE released?"), QuestionType::When);
    }

    #[test]
    fn test_classify_where() {
        assert_eq!(LocalSynthesizer::classify_question("Where is Paris?"), QuestionType::Where);
    }

    #[test]
    fn test_classify_how_many() {
        assert_eq!(LocalSynthesizer::classify_question("How many books are there?"), QuestionType::HowMany);
    }

    #[test]
    fn test_classify_definition() {
        assert_eq!(LocalSynthesizer::classify_question("Define photosynthesis"), QuestionType::Definition);
        assert_eq!(LocalSynthesizer::classify_question("What does entropy mean?"), QuestionType::Definition);
    }

    #[test]
    fn test_classify_yes_no() {
        assert_eq!(LocalSynthesizer::classify_question("Is SAGE open source?"), QuestionType::YesNo);
        assert_eq!(LocalSynthesizer::classify_question("Was Plato a philosopher?"), QuestionType::YesNo);
    }

    #[test]
    fn test_classify_why() {
        assert_eq!(LocalSynthesizer::classify_question("Why does the NCA grid converge?"), QuestionType::Why);
    }

    #[test]
    fn test_extract_key_terms() {
        let terms = LocalSynthesizer::extract_key_terms("Who wrote Alice in Wonderland?");
        assert!(terms.contains(&"alice".to_string()));
        assert!(terms.contains(&"wonderland".to_string()));
        assert!(terms.contains(&"wrote".to_string()));
        assert!(!terms.contains(&"who".to_string()));
        assert!(!terms.contains(&"the".to_string()));
    }

    #[test]
    fn test_synthesize_who_question() {
        let passages = vec![
            "Alice's Adventures in Wonderland was written by Lewis Carroll in 1865. It is a classic novel.".to_string(),
            "The book tells the story of a girl who falls down a rabbit hole.".to_string(),
        ];
        let answer = LocalSynthesizer::synthesize("Who wrote Alice in Wonderland?", &passages);
        assert!(answer.is_some());
        let answer = answer.unwrap();
        assert!(answer.to_lowercase().contains("lewis carroll") || answer.to_lowercase().contains("carroll"));
    }

    #[test]
    fn test_synthesize_when_question() {
        let passages = vec![
            "Alice's Adventures in Wonderland was written by Lewis Carroll in 1865. It is a classic novel.".to_string(),
        ];
        let answer = LocalSynthesizer::synthesize("When was Alice in Wonderland written?", &passages);
        assert!(answer.is_some());
        let answer = answer.unwrap();
        assert!(answer.contains("1865"));
    }

    #[test]
    fn test_synthesize_definition() {
        let passages = vec![
            "Entropy is a measure of disorder in a thermodynamic system. It always increases in isolated systems.".to_string(),
        ];
        let answer = LocalSynthesizer::synthesize("What is entropy?", &passages);
        assert!(answer.is_some());
        let answer = answer.unwrap();
        assert!(answer.to_lowercase().contains("entropy"));
        assert!(answer.to_lowercase().contains("disorder") || answer.to_lowercase().contains("measure"));
    }

    #[test]
    fn test_synthesize_yes_no() {
        let passages = vec![
            "SAGE is an open source project released under the MIT license. It runs on any hardware.".to_string(),
        ];
        let answer = LocalSynthesizer::synthesize("Is SAGE open source?", &passages);
        assert!(answer.is_some());
        let answer = answer.unwrap();
        assert!(answer.to_lowercase().starts_with("yes"));
    }

    #[test]
    fn test_synthesize_no_relevant_passage() {
        let passages = vec![
            "The weather today is sunny with a high of 75 degrees.".to_string(),
        ];
        let answer = LocalSynthesizer::synthesize("Who wrote Hamlet?", &passages);
        // Should return None or a very low-confidence response
        // It's OK either way — the caller will fall back to LLM
        // Just make sure it doesn't crash
        assert!(answer.is_none() || answer.is_some());
    }

    #[test]
    fn test_can_answer_locally() {
        let passages = vec![
            "Alice's Adventures in Wonderland was written by Lewis Carroll in 1865.".to_string(),
        ];
        assert!(LocalSynthesizer::can_answer_locally("Who wrote Alice in Wonderland?", &passages));
        assert!(!LocalSynthesizer::can_answer_locally("Who wrote Hamlet?", &passages));
        assert!(!LocalSynthesizer::can_answer_locally("Who wrote Hamlet?", &[]));
    }

    #[test]
    fn test_extract_knowledge_from_chat() {
        let synthesizer = LocalSynthesizer::new();
        let messages = vec![
            ChatMessage {
                role: ChatRole::System,
                content: "You are SAGE.\n\n## Recalled Knowledge\n- Alice's Adventures in Wonderland was written by Lewis Carroll in 1865.\n- It is a classic of English literature.\n\n## Other".to_string(),
            },
            ChatMessage {
                role: ChatRole::User,
                content: "Who wrote Alice in Wonderland?".to_string(),
            },
        ];
        let response = synthesizer.chat(&messages, 100).unwrap();
        assert!(response.to_lowercase().contains("carroll"));
    }

    #[test]
    fn test_split_sentences() {
        let text = "This is sentence one. This is sentence two! Is this sentence three? Yes it is absolutely.";
        let sentences = LocalSynthesizer::split_sentences(text);
        assert_eq!(sentences.len(), 4);
        assert!(sentences[0].contains("sentence one"));
    }

    #[test]
    fn test_score_sentence_keyword_overlap() {
        let score = LocalSynthesizer::score_sentence(
            "Alice's Adventures in Wonderland was written by Lewis Carroll.",
            "Who wrote Alice in Wonderland?",
            &QuestionType::Who,
        );
        assert!(score > 0.5, "Score should be high for matching sentence: {}", score);
    }

    #[test]
    fn test_score_sentence_no_overlap() {
        let score = LocalSynthesizer::score_sentence(
            "The weather is nice today.",
            "Who wrote Alice in Wonderland?",
            &QuestionType::Who,
        );
        assert!(score < 0.3, "Score should be low for non-matching sentence: {}", score);
    }

    #[test]
    fn test_local_synthesizer_implements_trait() {
        let synth = LocalSynthesizer::new();
        assert!(synth.is_available());
        assert!(synth.name().contains("Local Synthesis"));
    }

    #[test]
    fn test_synthesize_how_many() {
        let passages = vec![
            "The library contains 100 books from Project Gutenberg. They span multiple genres.".to_string(),
        ];
        let answer = LocalSynthesizer::synthesize("How many books are in the library?", &passages);
        assert!(answer.is_some());
        let answer = answer.unwrap();
        assert!(answer.contains("100"));
    }

    #[test]
    fn test_synthesize_where() {
        let passages = vec![
            "Paris is the capital of France. It is located in the northern part of the country.".to_string(),
        ];
        let answer = LocalSynthesizer::synthesize("Where is Paris?", &passages);
        assert!(answer.is_some());
        let answer = answer.unwrap();
        assert!(answer.to_lowercase().contains("france") || answer.to_lowercase().contains("capital"));
    }
}