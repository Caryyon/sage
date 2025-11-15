// A/B Testing Framework for NCA Memory Validation
// Compares LLM responses with vs without NCA memory to prove effectiveness

use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::Write;
use chrono::Local;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ABTestResult {
    pub timestamp: String,
    pub input: String,
    pub response_with_nca: String,
    pub response_without_nca: String,
    pub nca_opinion: String,
    pub baseline_opinion: String,
    pub nca_grid_activity: f64,  // Average grid alpha
    pub similarity_score: f64,   // How similar the responses are (0-1)
    pub nca_referenced_memory: bool,  // Did NCA response reference past?
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ABTestMetrics {
    pub total_tests: usize,
    pub avg_similarity: f64,
    pub memory_reference_rate: f64,  // % of times NCA referenced past
    pub opinion_divergence_rate: f64,  // % of times opinions differed
    pub significant_differences: usize,  // Responses with <50% similarity
}

pub struct ABTester {
    results: Vec<ABTestResult>,
    log_file: String,
}

impl ABTester {
    pub fn new(log_file: &str) -> Self {
        Self {
            results: Vec::new(),
            log_file: log_file.to_string(),
        }
    }

    /// Record a single A/B test comparison
    pub fn record_test(
        &mut self,
        input: String,
        response_with_nca: String,
        response_without_nca: String,
        nca_opinion: String,
        baseline_opinion: String,
        nca_grid_activity: f64,
    ) {
        let similarity = self.calculate_similarity(&response_with_nca, &response_without_nca);
        let nca_referenced_memory = self.check_memory_reference(&response_with_nca);

        let result = ABTestResult {
            timestamp: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            input: input.clone(),
            response_with_nca: response_with_nca.clone(),
            response_without_nca: response_without_nca.clone(),
            nca_opinion: nca_opinion.clone(),
            baseline_opinion: baseline_opinion.clone(),
            nca_grid_activity,
            similarity_score: similarity,
            nca_referenced_memory,
        };

        self.results.push(result.clone());
        self.log_to_file(&result);
    }

    /// Calculate similarity between two responses (simple word overlap)
    fn calculate_similarity(&self, resp_a: &str, resp_b: &str) -> f64 {
        let lower_a = resp_a.to_lowercase();
        let lower_b = resp_b.to_lowercase();
        let words_a: Vec<&str> = lower_a.split_whitespace().collect();
        let words_b: Vec<&str> = lower_b.split_whitespace().collect();

        if words_a.is_empty() || words_b.is_empty() {
            return 0.0;
        }

        let common_words: usize = words_a.iter()
            .filter(|w| words_b.contains(w))
            .count();

        let total_unique = words_a.len().max(words_b.len());
        common_words as f64 / total_unique as f64
    }

    /// Check if response references past conversations
    fn check_memory_reference(&self, response: &str) -> bool {
        let memory_phrases = [
            "before",
            "earlier",
            "remember",
            "recall",
            "previously",
            "last time",
            "we discussed",
            "you mentioned",
            "as we talked",
        ];

        let response_lower = response.to_lowercase();
        memory_phrases.iter().any(|phrase| response_lower.contains(phrase))
    }

    /// Calculate overall metrics
    pub fn calculate_metrics(&self) -> ABTestMetrics {
        if self.results.is_empty() {
            return ABTestMetrics {
                total_tests: 0,
                avg_similarity: 0.0,
                memory_reference_rate: 0.0,
                opinion_divergence_rate: 0.0,
                significant_differences: 0,
            };
        }

        let total = self.results.len();
        let avg_similarity = self.results.iter()
            .map(|r| r.similarity_score)
            .sum::<f64>() / total as f64;

        let memory_references = self.results.iter()
            .filter(|r| r.nca_referenced_memory)
            .count();

        let opinion_divergences = self.results.iter()
            .filter(|r| r.nca_opinion != r.baseline_opinion)
            .count();

        let significant_diffs = self.results.iter()
            .filter(|r| r.similarity_score < 0.5)
            .count();

        ABTestMetrics {
            total_tests: total,
            avg_similarity,
            memory_reference_rate: memory_references as f64 / total as f64,
            opinion_divergence_rate: opinion_divergences as f64 / total as f64,
            significant_differences: significant_diffs,
        }
    }

    /// Log result to file
    fn log_to_file(&self, result: &ABTestResult) {
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_file)
        {
            let json = serde_json::to_string_pretty(result).unwrap_or_default();
            let _ = writeln!(file, "{}", json);
            let _ = writeln!(file, "---");
        }
    }

    /// Export metrics report
    pub fn export_metrics_report(&self, filename: &str) -> Result<(), String> {
        let metrics = self.calculate_metrics();

        let mut file = File::create(filename)
            .map_err(|e| format!("Failed to create report: {}", e))?;

        writeln!(file, "# SAGE NCA Memory A/B Testing Report")
            .map_err(|e| e.to_string())?;
        writeln!(file, "Generated: {}\n", Local::now().format("%Y-%m-%d %H:%M:%S"))
            .map_err(|e| e.to_string())?;

        writeln!(file, "## Summary Statistics")
            .map_err(|e| e.to_string())?;
        writeln!(file, "- Total Tests: {}", metrics.total_tests)
            .map_err(|e| e.to_string())?;
        writeln!(file, "- Average Response Similarity: {:.2}%", metrics.avg_similarity * 100.0)
            .map_err(|e| e.to_string())?;
        writeln!(file, "- Memory Reference Rate: {:.2}%", metrics.memory_reference_rate * 100.0)
            .map_err(|e| e.to_string())?;
        writeln!(file, "- Opinion Divergence Rate: {:.2}%", metrics.opinion_divergence_rate * 100.0)
            .map_err(|e| e.to_string())?;
        writeln!(file, "- Significant Differences (<50% similar): {}", metrics.significant_differences)
            .map_err(|e| e.to_string())?;

        writeln!(file, "\n## Interpretation")
            .map_err(|e| e.to_string())?;

        if metrics.avg_similarity < 0.5 {
            writeln!(file, "✅ **NCA memory produces significantly different responses** (avg similarity: {:.1}%)", metrics.avg_similarity * 100.0)
                .map_err(|e| e.to_string())?;
        } else {
            writeln!(file, "⚠️  Responses are similar (avg: {:.1}%), NCA impact may be subtle", metrics.avg_similarity * 100.0)
                .map_err(|e| e.to_string())?;
        }

        if metrics.memory_reference_rate > 0.2 {
            writeln!(file, "✅ **NCA actively references past conversations** ({:.1}% of responses)", metrics.memory_reference_rate * 100.0)
                .map_err(|e| e.to_string())?;
        }

        if metrics.opinion_divergence_rate > 0.3 {
            writeln!(file, "✅ **NCA develops different opinions than baseline** ({:.1}% divergence)", metrics.opinion_divergence_rate * 100.0)
                .map_err(|e| e.to_string())?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_similarity_calculation() {
        let tester = ABTester::new("test.log");

        // Identical responses
        let sim1 = tester.calculate_similarity("hello world", "hello world");
        assert!(sim1 > 0.9);

        // Completely different
        let sim2 = tester.calculate_similarity("hello world", "foo bar");
        assert!(sim2 < 0.3);

        // Partial overlap
        let sim3 = tester.calculate_similarity("hello beautiful world", "hello world");
        assert!(sim3 > 0.5 && sim3 < 0.9);
    }

    #[test]
    fn test_memory_reference_detection() {
        let tester = ABTester::new("test.log");

        assert!(tester.check_memory_reference("As we discussed before, this is interesting"));
        assert!(tester.check_memory_reference("You mentioned earlier that..."));
        assert!(!tester.check_memory_reference("This is a completely new thought"));
    }
}
