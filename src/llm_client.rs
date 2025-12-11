use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Serialize, Debug)]
struct OllamaRequest {
    model: String,
    prompt: String,
    stream: bool,
}

#[derive(Deserialize, Debug)]
struct OllamaResponse {
    response: String,
}

#[derive(Clone)]
pub struct LlmClient {
    endpoint: String,
    model: String,
}

impl LlmClient {
    pub fn new() -> Self {
        Self {
            endpoint: "http://localhost:11434/api/generate".to_string(),
            model: "llama3.2:3b".to_string(),
        }
    }

    pub fn with_model(model: &str) -> Self {
        Self {
            endpoint: "http://localhost:11434/api/generate".to_string(),
            model: model.to_string(),
        }
    }

    /// Generate a response using the LLM with SAGE's emotional context
    /// username: The Discord username of who SAGE is responding to
    pub async fn generate(&self, user_message: &str, sage_context: &str) -> Result<String, Box<dyn Error>> {
        // Extract username from context (format: "User USERNAME says:")
        let username = sage_context
            .lines()
            .find(|line| line.starts_with("User ") && line.contains(" says:"))
            .and_then(|line| line.strip_prefix("User "))
            .and_then(|s| s.split(" says:").next())
            .unwrap_or("unknown");

        // Query user facts from database for this specific user
        let user_facts = get_user_facts_from_db(username).await.unwrap_or_default();

        // Build prompt - context goes first, then the actual message to respond to
        // No "SAGE:" at the end to avoid the model echoing it
        let full_prompt = format!(
            "[Talking with: {}]\n\n\
            {}\n\n\
            {}",
            username, user_facts, sage_context
        );

        // Condensed debug for chat
        eprintln!("\x1b[33m[LLM]\x1b[0m chat for \x1b[36m{}\x1b[0m ({} chars)", username, full_prompt.len());

        let client = reqwest::Client::new();
        let response = client
            .post(&self.endpoint)
            .json(&OllamaRequest {
                model: self.model.clone(),
                prompt: full_prompt,
                stream: false,
            })
            .timeout(std::time::Duration::from_secs(120))  // 2 min for dolphin-mixtral (first load can take 5+ min, but subsequent requests are faster)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(format!("LLM API error: {}", response.status()).into());
        }

        let ollama_response: OllamaResponse = response.json().await?;
        Ok(ollama_response.response.trim().to_string())
    }

    /// Test if Ollama is running and the model is available
    pub async fn test_connection(&self) -> Result<(), Box<dyn Error>> {
        let client = reqwest::Client::new();
        let response = client
            .post(&self.endpoint)
            .json(&OllamaRequest {
                model: self.model.clone(),
                prompt: "Test".to_string(),
                stream: false,
            })
            .timeout(std::time::Duration::from_secs(120))  // 2 min for dolphin-mixtral first load
            .send()
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(format!("Ollama not responding: {}", response.status()).into())
        }
    }

    /// Generate a response using a raw prompt (no system prompt wrapping)
    /// Used by ResponsePipeline which constructs its own complete prompts
    pub async fn generate_raw(&self, prompt: &str) -> Result<String, Box<dyn Error>> {
        // Condensed debug - just show prompt type and length
        let prompt_type = if prompt.contains("inner thought") {
            "inner thought"
        } else if prompt.contains("choose an action") {
            "action choice"
        } else if prompt.contains("event") {
            "event response"
        } else {
            "general"
        };
        eprintln!("\x1b[33m[LLM]\x1b[0m {} ({} chars)", prompt_type, prompt.len());

        let client = reqwest::Client::new();
        let response = client
            .post(&self.endpoint)
            .json(&OllamaRequest {
                model: self.model.clone(),
                prompt: prompt.to_string(),
                stream: false,
            })
            .timeout(std::time::Duration::from_secs(120))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(format!("LLM API error: {}", response.status()).into());
        }

        let ollama_response: OllamaResponse = response.json().await?;
        Ok(ollama_response.response.trim().to_string())
    }
}

impl Default for LlmClient {
    fn default() -> Self {
        Self::new()
    }
}

/// Query user facts from SpacetimeDB and format for LLM context
async fn get_user_facts_from_db(user_id: &str) -> Result<String, Box<dyn std::error::Error>> {
    // Query ALL facts from database (SpacetimeDB CLI doesn't support WHERE clauses)
    let output = std::process::Command::new("spacetime")
        .args(&[
            "sql",
            "sage-db",
            "SELECT user_id, fact_key, value, mention_count FROM user_facts"
        ])
        .output()?;

    if !output.status.success() {
        eprintln!("⚠️  Failed to query user facts from database");
        return Ok(String::new());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Parse the SQL output and filter for this user
    let mut user_facts: Vec<(String, String, u32)> = Vec::new();  // (fact_key, value, mention_count)

    for line in stdout.lines() {
        // Skip header, separator, and warning lines
        if line.contains("user_id") || line.contains("---") || line.contains("WARNING") || line.trim().is_empty() {
            continue;
        }

        // Parse: user_id | fact_key | value | mention_count
        let parts: Vec<&str> = line.split('|').map(|s| s.trim()).collect();
        if parts.len() >= 4 {
            let row_user_id = parts[0].trim_matches('"');

            // Filter for matching user_id
            if row_user_id == user_id {
                let fact_key = parts[1].trim_matches('"').to_string();
                let value = parts[2].trim_matches('"').to_string();
                let mentions = parts[3].parse::<u32>().unwrap_or(0);

                user_facts.push((fact_key, value, mentions));
            }
        }
    }

    if user_facts.is_empty() {
        return Ok(String::new());
    }

    // Sort by mention_count (descending)
    user_facts.sort_by(|a, b| b.2.cmp(&a.2));

    // Build facts context
    let mut context = String::from("\n🔵 CRITICAL USER FACTS (ALWAYS REMEMBER THESE):\n");
    let fact_count = user_facts.len();

    for (fact_key, value, mentions) in &user_facts {
        let fact_desc = match fact_key.as_str() {
            "name" => format!("• The user's REAL NAME is '{}' (NOT their username!) (mentioned {} times)", value, mentions),
            "japanese_name" => format!("• The user's Japanese name is {} (mentioned {} times)", value, mentions),
            "wife_name" => format!("• The user's wife's name is {} (mentioned {} times)", value, mentions),
            "husband_name" => format!("• The user's husband's name is {} (mentioned {} times)", value, mentions),
            "son_name" => format!("• The user's son's name is {} (mentioned {} times)", value, mentions),
            "daughter_name" => format!("• The user's daughter's name is {} (mentioned {} times)", value, mentions),
            key if key.starts_with("preference:") => {
                let topic = key.strip_prefix("preference:").unwrap_or(key);
                format!("• You prefer {} for {} (mentioned {} times)", value, topic, mentions)
            },
            key if key.starts_with("detail:") => {
                let topic = key.strip_prefix("detail:").unwrap_or(key);
                format!("• About {}: {} (mentioned {} times)", topic, value, mentions)
            },
            _ => format!("• {}: {} (mentioned {} times)", fact_key, value, mentions),
        };

        context.push_str(&fact_desc);
        context.push('\n');
    }

    context.push_str("• When asked about names or personal details, always refer to THESE specific facts above\n\n");

    eprintln!("📚 Loaded {} facts for {}", fact_count, user_id);
    Ok(context)
}
