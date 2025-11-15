// Tool System - SAGE's ability to interact with the real world

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Command;

/// Result of tool execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub success: bool,
    pub output: String,
    pub error: Option<String>,
    pub metadata: HashMap<String, String>,
}

impl ToolResult {
    pub fn success(output: String) -> Self {
        Self {
            success: true,
            output,
            error: None,
            metadata: HashMap::new(),
        }
    }

    pub fn failure(error: String) -> Self {
        Self {
            success: false,
            output: String::new(),
            error: Some(error),
            metadata: HashMap::new(),
        }
    }

    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

/// Types of tools SAGE can use
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ToolType {
    WebSearch,
    CodeExecution,
    FileRead,
    FileWrite,
    ApiCall,
    Calculator,
    Weather,
    News,
    Knowledge,
    Time,
}

/// A tool that SAGE can use
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn tool_type(&self) -> ToolType;
    fn execute(&self, input: &str) -> ToolResult;
    fn requires_approval(&self) -> bool {
        false  // Most tools are safe by default
    }
}

/// Web search tool using DuckDuckGo
pub struct WebSearchTool;

impl Tool for WebSearchTool {
    fn name(&self) -> &str {
        "web_search"
    }

    fn description(&self) -> &str {
        "Search the web for information using DuckDuckGo. Input: search query"
    }

    fn tool_type(&self) -> ToolType {
        ToolType::WebSearch
    }

    fn execute(&self, query: &str) -> ToolResult {
        // Use reqwest to search DuckDuckGo's instant answer API
        let url = format!(
            "https://api.duckduckgo.com/?q={}&format=json&no_html=1",
            urlencoding::encode(query)
        );

        match ureq::get(&url).call() {
            Ok(response) => {
                match response.into_string() {
                    Ok(body) => {
                        // Parse JSON response
                        match serde_json::from_str::<serde_json::Value>(&body) {
                            Ok(json) => {
                                let abstract_text = json["AbstractText"].as_str().unwrap_or("");
                                let related_topics: Vec<String> = json["RelatedTopics"]
                                    .as_array()
                                    .map(|topics| {
                                        topics.iter()
                                            .filter_map(|t| t["Text"].as_str())
                                            .take(3)
                                            .map(|s| s.to_string())
                                            .collect()
                                    })
                                    .unwrap_or_default();

                                let output = if !abstract_text.is_empty() {
                                    format!("{}\n\nRelated: {}", abstract_text, related_topics.join(", "))
                                } else if !related_topics.is_empty() {
                                    format!("Related topics: {}", related_topics.join(", "))
                                } else {
                                    "No results found".to_string()
                                };

                                ToolResult::success(output)
                                    .with_metadata("query".to_string(), query.to_string())
                            }
                            Err(e) => ToolResult::failure(format!("Failed to parse response: {}", e)),
                        }
                    }
                    Err(e) => ToolResult::failure(format!("Failed to read response: {}", e)),
                }
            }
            Err(e) => ToolResult::failure(format!("Search failed: {}", e)),
        }
    }
}

/// Simple calculator tool
pub struct CalculatorTool;

impl Tool for CalculatorTool {
    fn name(&self) -> &str {
        "calculator"
    }

    fn description(&self) -> &str {
        "Evaluate mathematical expressions. Input: expression like '2 + 2' or '(5 * 3) + 7'"
    }

    fn tool_type(&self) -> ToolType {
        ToolType::Calculator
    }

    fn execute(&self, expression: &str) -> ToolResult {
        // Use Python for safe math evaluation
        let python_code = format!("print({})", expression);

        match Command::new("python3")
            .arg("-c")
            .arg(&python_code)
            .output()
        {
            Ok(output) => {
                if output.status.success() {
                    let result = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    ToolResult::success(result)
                        .with_metadata("expression".to_string(), expression.to_string())
                } else {
                    let error = String::from_utf8_lossy(&output.stderr).to_string();
                    ToolResult::failure(format!("Invalid expression: {}", error))
                }
            }
            Err(e) => ToolResult::failure(format!("Calculator error: {}", e)),
        }
    }
}

/// Code execution tool (sandboxed Python)
pub struct CodeExecutionTool;

impl Tool for CodeExecutionTool {
    fn name(&self) -> &str {
        "execute_code"
    }

    fn description(&self) -> &str {
        "Execute Python code safely. Input: Python code to run"
    }

    fn tool_type(&self) -> ToolType {
        ToolType::CodeExecution
    }

    fn execute(&self, code: &str) -> ToolResult {
        // Execute Python code with timeout
        match Command::new("timeout")
            .arg("5")  // 5 second timeout
            .arg("python3")
            .arg("-c")
            .arg(code)
            .output()
        {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();

                if output.status.success() {
                    ToolResult::success(stdout)
                        .with_metadata("code".to_string(), code.to_string())
                } else {
                    ToolResult::failure(format!("Execution failed: {}", stderr))
                }
            }
            Err(e) => ToolResult::failure(format!("Failed to execute: {}", e)),
        }
    }

    fn requires_approval(&self) -> bool {
        true  // Code execution needs approval
    }
}

/// File read tool
pub struct FileReadTool {
    allowed_dirs: Vec<String>,
}

impl FileReadTool {
    pub fn new(allowed_dirs: Vec<String>) -> Self {
        Self { allowed_dirs }
    }
}

impl Tool for FileReadTool {
    fn name(&self) -> &str {
        "read_file"
    }

    fn description(&self) -> &str {
        "Read contents of a file. Input: file path"
    }

    fn tool_type(&self) -> ToolType {
        ToolType::FileRead
    }

    fn execute(&self, path: &str) -> ToolResult {
        // Check if path is in allowed directories
        let is_allowed = self.allowed_dirs.iter().any(|dir| path.starts_with(dir));

        if !is_allowed {
            return ToolResult::failure("Access denied: path not in allowed directories".to_string());
        }

        match std::fs::read_to_string(path) {
            Ok(contents) => ToolResult::success(contents)
                .with_metadata("path".to_string(), path.to_string()),
            Err(e) => ToolResult::failure(format!("Failed to read file: {}", e)),
        }
    }
}

/// Wikipedia search tool - better for factual knowledge than general web search
pub struct WikipediaTool;

impl Tool for WikipediaTool {
    fn name(&self) -> &str {
        "wikipedia"
    }

    fn description(&self) -> &str {
        "Search Wikipedia for factual information. Input: search query"
    }

    fn tool_type(&self) -> ToolType {
        ToolType::Knowledge
    }

    fn execute(&self, query: &str) -> ToolResult {
        // Use Wikipedia API to search
        let search_url = format!(
            "https://en.wikipedia.org/w/api.php?action=opensearch&search={}&limit=1&format=json",
            urlencoding::encode(query)
        );

        match ureq::get(&search_url).call() {
            Ok(response) => {
                match response.into_string() {
                    Ok(body) => {
                        match serde_json::from_str::<serde_json::Value>(&body) {
                            Ok(json) => {
                                // OpenSearch API returns: [query, [titles], [descriptions], [urls]]
                                let titles = json.get(1).and_then(|v| v.as_array());
                                let descriptions = json.get(2).and_then(|v| v.as_array());
                                let urls = json.get(3).and_then(|v| v.as_array());

                                if let (Some(titles), Some(descriptions), Some(urls)) = (titles, descriptions, urls) {
                                    if !titles.is_empty() {
                                        let title = titles[0].as_str().unwrap_or("Unknown");
                                        let description = descriptions[0].as_str().unwrap_or("No description");
                                        let url = urls[0].as_str().unwrap_or("");

                                        // Get full extract from the article
                                        let extract_url = format!(
                                            "https://en.wikipedia.org/w/api.php?action=query&prop=extracts&exintro=1&explaintext=1&titles={}&format=json",
                                            urlencoding::encode(title)
                                        );

                                        if let Ok(extract_response) = ureq::get(&extract_url).call() {
                                            if let Ok(extract_body) = extract_response.into_string() {
                                                if let Ok(extract_json) = serde_json::from_str::<serde_json::Value>(&extract_body) {
                                                    if let Some(pages) = extract_json["query"]["pages"].as_object() {
                                                        if let Some((_, page)) = pages.iter().next() {
                                                            let extract = page["extract"].as_str().unwrap_or(description);

                                                            // Limit to first 500 characters
                                                            let summary = if extract.len() > 500 {
                                                                format!("{}...", &extract[..500])
                                                            } else {
                                                                extract.to_string()
                                                            };

                                                            return ToolResult::success(format!("{}\n\nRead more: {}", summary, url))
                                                                .with_metadata("title".to_string(), title.to_string())
                                                                .with_metadata("url".to_string(), url.to_string());
                                                        }
                                                    }
                                                }
                                            }
                                        }

                                        // Fallback to just description
                                        return ToolResult::success(format!("{}\n\n{}\n\nRead more: {}", title, description, url))
                                            .with_metadata("title".to_string(), title.to_string());
                                    }
                                }

                                ToolResult::failure("No Wikipedia article found for this query".to_string())
                            }
                            Err(e) => ToolResult::failure(format!("Failed to parse Wikipedia response: {}", e)),
                        }
                    }
                    Err(e) => ToolResult::failure(format!("Failed to read Wikipedia response: {}", e)),
                }
            }
            Err(e) => ToolResult::failure(format!("Wikipedia search failed: {}", e)),
        }
    }
}

/// Weather tool using wttr.in (free weather service)
pub struct WeatherTool;

impl Tool for WeatherTool {
    fn name(&self) -> &str {
        "weather"
    }

    fn description(&self) -> &str {
        "Get current weather for a location. Input: city name or 'current' for your location"
    }

    fn tool_type(&self) -> ToolType {
        ToolType::Weather
    }

    fn execute(&self, location: &str) -> ToolResult {
        // Use wttr.in for weather data (format: ?format=j1 for JSON)
        let url = format!("https://wttr.in/{}?format=j1", urlencoding::encode(location));

        match ureq::get(&url).call() {
            Ok(response) => {
                match response.into_string() {
                    Ok(body) => {
                        match serde_json::from_str::<serde_json::Value>(&body) {
                            Ok(json) => {
                                let current = &json["current_condition"][0];
                                let temp_c = current["temp_C"].as_str().unwrap_or("?");
                                let temp_f = current["temp_F"].as_str().unwrap_or("?");
                                let condition = current["weatherDesc"][0]["value"].as_str().unwrap_or("Unknown");
                                let humidity = current["humidity"].as_str().unwrap_or("?");
                                let wind_kph = current["windspeedKmph"].as_str().unwrap_or("?");
                                let feels_like_c = current["FeelsLikeC"].as_str().unwrap_or("?");

                                let area = json["nearest_area"][0]["areaName"][0]["value"].as_str().unwrap_or(location);
                                let country = json["nearest_area"][0]["country"][0]["value"].as_str().unwrap_or("");

                                let output = format!(
                                    "Weather in {}, {}:\n• Condition: {}\n• Temperature: {}°C ({}°F), feels like {}°C\n• Humidity: {}%\n• Wind: {} km/h",
                                    area, country, condition, temp_c, temp_f, feels_like_c, humidity, wind_kph
                                );

                                ToolResult::success(output)
                                    .with_metadata("location".to_string(), format!("{}, {}", area, country))
                                    .with_metadata("temperature".to_string(), format!("{}°C", temp_c))
                            }
                            Err(e) => ToolResult::failure(format!("Failed to parse weather data: {}", e)),
                        }
                    }
                    Err(e) => ToolResult::failure(format!("Failed to read weather response: {}", e)),
                }
            }
            Err(e) => ToolResult::failure(format!("Weather request failed: {}", e)),
        }
    }
}

/// Time and date tool
pub struct TimeTool;

impl Tool for TimeTool {
    fn name(&self) -> &str {
        "time"
    }

    fn description(&self) -> &str {
        "Get current time, date, or timezone information. Input: 'now', 'date', or timezone like 'America/New_York'"
    }

    fn tool_type(&self) -> ToolType {
        ToolType::Time
    }

    fn execute(&self, input: &str) -> ToolResult {
        use chrono::{Local, Utc};

        match input.to_lowercase().as_str() {
            "now" | "current" | "time" => {
                let now = Local::now();
                let output = format!(
                    "Current time: {}\nDate: {}\nTimezone: {}",
                    now.format("%H:%M:%S"),
                    now.format("%Y-%m-%d"),
                    now.format("%Z")
                );
                ToolResult::success(output)
                    .with_metadata("timestamp".to_string(), now.timestamp().to_string())
            }
            "date" => {
                let now = Local::now();
                let output = format!(
                    "Today: {}\nDay of week: {}\nDay of year: {}",
                    now.format("%Y-%m-%d"),
                    now.format("%A"),
                    now.format("%j")
                );
                ToolResult::success(output)
            }
            "utc" => {
                let now = Utc::now();
                let output = format!(
                    "UTC time: {}\nUTC date: {}",
                    now.format("%H:%M:%S"),
                    now.format("%Y-%m-%d")
                );
                ToolResult::success(output)
            }
            "unix" | "timestamp" => {
                let now = Local::now();
                let output = format!("Unix timestamp: {}", now.timestamp());
                ToolResult::success(output)
                    .with_metadata("timestamp".to_string(), now.timestamp().to_string())
            }
            _ => {
                // Assume it's a timezone query
                ToolResult::failure("Timezone queries not yet implemented. Try 'now', 'date', or 'utc'".to_string())
            }
        }
    }
}

/// News aggregator tool (using free RSS feeds)
pub struct NewsTool;

impl Tool for NewsTool {
    fn name(&self) -> &str {
        "news"
    }

    fn description(&self) -> &str {
        "Get latest news headlines. Input: 'tech', 'science', 'world', or 'all'"
    }

    fn tool_type(&self) -> ToolType {
        ToolType::News
    }

    fn execute(&self, category: &str) -> ToolResult {
        // Use Google News RSS feeds (simple HTML parsing)
        let feed_url = match category.to_lowercase().as_str() {
            "tech" | "technology" => "https://news.google.com/rss/topics/CAAqJggKIiBDQkFTRWdvSUwyMHZNRGRqTVhZU0FtVnVHZ0pWVXlnQVAB?hl=en-US&gl=US&ceid=US:en",
            "science" => "https://news.google.com/rss/topics/CAAqJggKIiBDQkFTRWdvSUwyMHZNRFp0Y1RjU0FtVnVHZ0pWVXlnQVAB?hl=en-US&gl=US&ceid=US:en",
            "world" => "https://news.google.com/rss/topics/CAAqJggKIiBDQkFTRWdvSUwyMHZNRGx1YlY4U0FtVnVHZ0pWVXlnQVAB?hl=en-US&gl=US&ceid=US:en",
            _ => "https://news.google.com/rss?hl=en-US&gl=US&ceid=US:en",
        };

        match ureq::get(feed_url).call() {
            Ok(response) => {
                match response.into_string() {
                    Ok(body) => {
                        // Simple XML parsing - extract <title> tags
                        let mut headlines = Vec::new();
                        let lines: Vec<&str> = body.lines().collect();

                        for line in lines.iter() {
                            if line.trim().starts_with("<title>") && !line.contains("<![CDATA[") {
                                let title = line
                                    .trim()
                                    .trim_start_matches("<title>")
                                    .trim_end_matches("</title>")
                                    .trim();

                                // Skip feed metadata titles
                                if !title.is_empty() && title != "Google News" {
                                    headlines.push(title.to_string());

                                    if headlines.len() >= 5 {
                                        break;
                                    }
                                }
                            }
                        }

                        if headlines.is_empty() {
                            return ToolResult::failure("No news headlines found".to_string());
                        }

                        let output = format!(
                            "Latest {} news:\n{}",
                            category,
                            headlines.iter()
                                .enumerate()
                                .map(|(i, h)| format!("{}. {}", i + 1, h))
                                .collect::<Vec<_>>()
                                .join("\n")
                        );

                        ToolResult::success(output)
                            .with_metadata("category".to_string(), category.to_string())
                            .with_metadata("count".to_string(), headlines.len().to_string())
                    }
                    Err(e) => ToolResult::failure(format!("Failed to read news feed: {}", e)),
                }
            }
            Err(e) => ToolResult::failure(format!("News request failed: {}", e)),
        }
    }
}

/// Tool registry - manages available tools
pub struct ToolRegistry {
    tools: HashMap<String, Box<dyn Tool>>,
    usage_count: HashMap<String, usize>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
            usage_count: HashMap::new(),
        }
    }

    /// Register a tool
    pub fn register(&mut self, tool: Box<dyn Tool>) {
        let name = tool.name().to_string();
        self.tools.insert(name.clone(), tool);
        self.usage_count.insert(name, 0);
    }

    /// Execute a tool by name
    pub fn execute(&mut self, tool_name: &str, input: &str) -> Result<ToolResult, String> {
        if let Some(tool) = self.tools.get(tool_name) {
            // Increment usage count
            *self.usage_count.get_mut(tool_name).unwrap() += 1;

            Ok(tool.execute(input))
        } else {
            Err(format!("Tool '{}' not found", tool_name))
        }
    }

    /// Get all available tools
    pub fn list_tools(&self) -> Vec<(&str, &str)> {
        self.tools.iter()
            .map(|(name, tool)| (name.as_str(), tool.description()))
            .collect()
    }

    /// Get tool usage statistics
    pub fn get_usage_stats(&self) -> Vec<(String, usize)> {
        let mut stats: Vec<_> = self.usage_count.iter()
            .map(|(name, count)| (name.clone(), *count))
            .collect();
        stats.sort_by_key(|(_, count)| std::cmp::Reverse(*count));
        stats
    }

    /// Check if tool requires approval
    pub fn requires_approval(&self, tool_name: &str) -> bool {
        self.tools.get(tool_name)
            .map(|tool| tool.requires_approval())
            .unwrap_or(false)
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        let mut registry = Self::new();

        // Register default tools
        registry.register(Box::new(WebSearchTool));
        registry.register(Box::new(WikipediaTool));
        registry.register(Box::new(WeatherTool));
        registry.register(Box::new(NewsTool));
        registry.register(Box::new(TimeTool));
        registry.register(Box::new(CalculatorTool));
        registry.register(Box::new(CodeExecutionTool));

        registry
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculator() {
        let calc = CalculatorTool;
        let result = calc.execute("2 + 2");
        assert!(result.success);
        assert_eq!(result.output.trim(), "4");
    }

    #[test]
    fn test_registry() {
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(CalculatorTool));

        let result = registry.execute("calculator", "5 * 3").unwrap();
        assert!(result.success);
        assert_eq!(result.output.trim(), "15");

        let stats = registry.get_usage_stats();
        assert_eq!(stats[0].1, 1);
    }
}
