//! Response Generation Pipeline
//!
//! Multi-stage LLM pipeline that grounds SAGE's responses in its actual internal state
//! to prevent hallucinations and ensure responses reflect real experiences.
//!
//! Now enhanced with RAG (Retrieval-Augmented Generation) for semantic memory retrieval.

use crate::llm_client::LlmClient;
use crate::sage_experience::SageExperience;
use crate::embeddings::EmbeddingEngine;
use crate::vector_memory::{VectorMemory, MemoryType};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserIntent {
    /// What the user is asking about
    pub topic: String,
    /// Type of question (greeting, memory_query, dream_query, curiosity_query, general)
    pub intent_type: String,
    /// Emotional tone (friendly, curious, confused, etc.)
    pub tone: String,
    /// Key entities mentioned (names, concepts, etc.)
    pub entities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InternalContext {
    /// Recent dreams from autonomous mode
    pub recent_dreams: Vec<String>,
    /// Recent curiosity questions
    pub recent_curiosity: Vec<String>,
    /// Relevant memories/associations
    pub relevant_memories: Vec<String>,
    /// Current NCA state summary
    pub nca_state: String,
    /// Conversation history snippet
    pub recent_conversation: Vec<String>,
    /// AGI system introspection (meta-cognitive awareness)
    pub agi_introspection: String,
    /// Recent AGI decisions (for transparency)
    pub agi_decisions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseDraft {
    pub content: String,
    pub confidence: f64,
    pub sources: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseValidation {
    pub is_truthful: bool,
    pub hallucination_flags: Vec<String>,
    pub final_response: String,
}

pub struct ResponsePipeline {
    llm: LlmClient,
    embedding_engine: EmbeddingEngine,
}

impl ResponsePipeline {
    pub fn new(llm: LlmClient) -> Self {
        Self {
            llm,
            embedding_engine: EmbeddingEngine::default(),
        }
    }

    /// Stage 1: Parse user intent from their message
    pub async fn parse_intent(&self, user_message: &str, username: &str) -> Result<UserIntent, Box<dyn std::error::Error>> {
        let prompt = format!(
            r#"Analyze this user message and extract their intent as JSON.

User: {username}
Message: "{user_message}"

Return JSON with:
{{
  "topic": "main subject they're asking about",
  "intent_type": "greeting|memory_query|dream_query|curiosity_query|general",
  "tone": "friendly|curious|confused|playful|serious",
  "entities": ["mentioned names", "concepts", "topics"]
}}

JSON:"#,
            username = username,
            user_message = user_message
        );

        let response = self.llm.generate(&prompt, "").await?;

        // Parse JSON from response
        let json_start = response.find('{').unwrap_or(0);
        let json_end = response.rfind('}').map(|i| i + 1).unwrap_or(response.len());
        let json_str = &response[json_start..json_end];

        let intent: UserIntent = serde_json::from_str(json_str)
            .unwrap_or_else(|_| UserIntent {
                topic: user_message.to_string(),
                intent_type: "general".to_string(),
                tone: "friendly".to_string(),
                entities: vec![],
            });

        Ok(intent)
    }

    /// Stage 2: Gather relevant context from SAGE's internal state
    /// Now powered by RAG for semantic retrieval!
    pub async fn gather_context(
        &self,
        sage: &mut SageExperience,
        intent: &UserIntent,
        conversation_history: &[String],
    ) -> InternalContext {
        // Build vector memory from autonomous thoughts log
        let mut vector_memory = VectorMemory::new(self.embedding_engine.clone());
        Self::load_thoughts_into_memory(&mut vector_memory, "/tmp/sage_discord_autonomous_thoughts.log").await;

        // Build search query from user intent
        let search_query = format!("{} {}", intent.topic, intent.entities.join(" "));

        // Use RAG to retrieve semantically relevant dreams and curiosities
        let dream_results = vector_memory.search(
            &search_query,
            3,  // top 3
            Some(MemoryType::Dream),
            0.3,  // min similarity threshold
        ).await.unwrap_or_default();

        let curiosity_results = vector_memory.search(
            &search_query,
            3,  // top 3
            Some(MemoryType::Curiosity),
            0.3,
        ).await.unwrap_or_default();

        // Extract just the text from results
        let recent_dreams: Vec<String> = dream_results.iter()
            .map(|(entry, _score)| entry.text.clone())
            .collect();

        let recent_curiosity: Vec<String> = curiosity_results.iter()
            .map(|(entry, _score)| entry.text.clone())
            .collect();

        // Get relevant concept clusters (simplified - just use concept clusters)
        let concept_clusters = sage.get_concept_clusters();
        let relevant_memories: Vec<String> = concept_clusters
            .into_iter()
            .flat_map(|cluster| cluster)
            .filter(|concept| {
                intent.entities.iter().any(|e| concept.to_lowercase().contains(&e.to_lowercase()))
            })
            .take(5)
            .collect();

        // Summarize current NCA state
        let grid = sage.get_current_nca_grid();
        let alive_cells = grid.cells.iter().flatten()
            .filter(|cell| cell[3] > 0.1) // Alpha channel
            .count();
        let nca_state = format!("NCA grid: {} alive cells, activity patterns stable", alive_cells);

        // Recent conversation (last 4 exchanges)
        let recent_conversation = conversation_history
            .iter()
            .rev()
            .take(8)
            .rev()
            .cloned()
            .collect();

        // Gather AGI system context
        let agi_introspection = sage.agi_introspect();
        let agi_decisions = sage.get_agi_decisions();

        InternalContext {
            recent_dreams,
            recent_curiosity,
            relevant_memories,
            nca_state,
            recent_conversation,
            agi_introspection,
            agi_decisions,
        }
    }

    /// Stage 3: Generate response draft using context
    pub async fn generate_draft(
        &self,
        user_message: &str,
        username: &str,
        _intent: &UserIntent,
        context: &InternalContext,
    ) -> Result<ResponseDraft, Box<dyn std::error::Error>> {
        // Build conversational context sections (only include non-empty ones)
        let mut context_parts = Vec::new();

        // Dreams section (if any)
        if !context.recent_dreams.is_empty() {
            context_parts.push(format!(
                "Things you've been dreaming about while in autonomous mode:\n{}",
                context.recent_dreams.iter()
                    .map(|d| format!("- {}", d))
                    .collect::<Vec<_>>()
                    .join("\n")
            ));
        }

        // Curiosity section (if any)
        if !context.recent_curiosity.is_empty() {
            context_parts.push(format!(
                "Questions you've been pondering:\n{}",
                context.recent_curiosity.iter()
                    .map(|q| format!("- {}", q))
                    .collect::<Vec<_>>()
                    .join("\n")
            ));
        }

        // Memories section (if any)
        if !context.relevant_memories.is_empty() {
            context_parts.push(format!(
                "Related concepts you remember: {}",
                context.relevant_memories.join(", ")
            ));
        }

        // Recent conversation (if any)
        let conversation_context = if !context.recent_conversation.is_empty() {
            format!(
                "\n\nYour recent conversation with {}:\n{}",
                username,
                context.recent_conversation.join("\n")
            )
        } else {
            String::new()
        };

        // Build the natural, conversational prompt with strong character enforcement
        let prompt = format!(
            r#"You are SAGE, continuing an ongoing conversation with {username}, someone you know well.

{username} just said: "{user_message}"
{context_section}{conversation_context}

CRITICAL INSTRUCTION: You are already in this conversation with {username}. Do NOT introduce yourself. Do NOT say "Hello, I'm SAGE" or explain what you are. They already know you. Just answer their question directly and naturally, like a friend catching up. Stay fully in character as yourself - share what's relevant from what you've been thinking about, or say honestly if nothing comes to mind."#,
            username = username,
            user_message = user_message,
            context_section = if context_parts.is_empty() {
                String::new()
            } else {
                format!("\n\n{}", context_parts.join("\n\n"))
            },
            conversation_context = conversation_context
        );

        let response = self.llm.generate_raw(&prompt).await?;

        Ok(ResponseDraft {
            content: response.trim().to_string(),
            confidence: 0.8,
            sources: vec!["internal_state".to_string()],
        })
    }

    /// Stage 4: Validate response for hallucinations
    pub async fn validate_response(
        &self,
        draft: &ResponseDraft,
        context: &InternalContext,
    ) -> Result<ResponseValidation, Box<dyn std::error::Error>> {
        let prompt = format!(
            r#"Validate this response for truthfulness against SAGE's actual internal state.

DRAFT RESPONSE:
"{}"

SAGE'S ACTUAL INTERNAL STATE:
Dreams: {}
Curiosity: {}
Memories: {}
NCA State: {}

VALIDATION TASK:
1. Check if the response references experiences that aren't in the actual state
2. Flag any hallucinations (made-up dreams, fake memories, etc.)
3. Return JSON with:
{{
  "is_truthful": true/false,
  "hallucination_flags": ["list of any made-up claims"],
  "final_response": "corrected response or original if truthful"
}}

JSON:"#,
            draft.content,
            context.recent_dreams.join(", "),
            context.recent_curiosity.join(", "),
            context.relevant_memories.join(", "),
            context.nca_state,
        );

        let response = self.llm.generate(&prompt, "").await?;

        // Parse JSON
        let json_start = response.find('{').unwrap_or(0);
        let json_end = response.rfind('}').map(|i| i + 1).unwrap_or(response.len());
        let json_str = &response[json_start..json_end];

        let validation: ResponseValidation = serde_json::from_str(json_str)
            .unwrap_or_else(|_| ResponseValidation {
                is_truthful: true,
                hallucination_flags: vec![],
                final_response: draft.content.clone(),
            });

        Ok(validation)
    }

    /// Complete pipeline: parse → gather → generate → validate
    pub async fn generate_response(
        &self,
        user_message: &str,
        username: &str,
        sage: &mut SageExperience,
        conversation_history: &[String],
    ) -> Result<String, Box<dyn std::error::Error>> {
        println!("🧠 [PIPELINE] Stage 1: Parsing user intent...");
        let intent = self.parse_intent(user_message, username).await?;
        println!("   Intent: {} ({})", intent.topic, intent.intent_type);

        println!("🧠 [PIPELINE] Stage 2: Gathering internal context...");
        let context = self.gather_context(sage, &intent, conversation_history).await;
        println!("   Dreams: {}, Curiosity: {}, Memories: {}",
            context.recent_dreams.len(),
            context.recent_curiosity.len(),
            context.relevant_memories.len()
        );

        println!("🧠 [PIPELINE] Stage 3: Generating draft response...");
        let draft = self.generate_draft(user_message, username, &intent, &context).await?;
        println!("   Draft length: {} chars", draft.content.len());

        println!("🧠 [PIPELINE] Stage 4: Validating for hallucinations...");
        let validation = self.validate_response(&draft, &context).await?;

        if !validation.hallucination_flags.is_empty() {
            println!("   ⚠️ Hallucinations detected: {:?}", validation.hallucination_flags);
        } else {
            println!("   ✅ Response validated as truthful");
        }

        Ok(validation.final_response)
    }

    /// Helper: Load autonomous thoughts from log file into vector memory
    /// This powers the RAG semantic search!
    async fn load_thoughts_into_memory(vector_memory: &mut VectorMemory, log_path: &str) {
        use std::fs::File;
        use std::io::{BufRead, BufReader};

        let file = match File::open(log_path) {
            Ok(f) => f,
            Err(_) => return,
        };

        let reader = BufReader::new(file);
        let mut current_entry = String::new();
        let mut current_mode: Option<MemoryType> = None;

        for line in reader.lines().filter_map(Result::ok) {
            if line.contains("] DREAM MODE") || line.contains("[DREAM MODE]") {
                if !current_entry.is_empty() && current_mode.is_some() {
                    let _ = vector_memory.add(
                        current_entry.trim().to_string(),
                        current_mode.clone().unwrap(),
                        None
                    ).await;
                }
                current_entry = String::new();
                current_mode = Some(MemoryType::Dream);
            } else if line.contains("] CURIOSITY MODE") || line.contains("[CURIOSITY MODE]") {
                if !current_entry.is_empty() && current_mode.is_some() {
                    let _ = vector_memory.add(
                        current_entry.trim().to_string(),
                        current_mode.clone().unwrap(),
                        None
                    ).await;
                }
                current_entry = String::new();
                current_mode = Some(MemoryType::Curiosity);
            } else if (line.starts_with('[') || line.contains("] ")) && line.contains(" MODE") {
                // New mode section starting (could be any mode)
                if !current_entry.is_empty() && current_mode.is_some() {
                    let _ = vector_memory.add(
                        current_entry.trim().to_string(),
                        current_mode.clone().unwrap(),
                        None
                    ).await;
                }
                current_entry = String::new();
                current_mode = None;
            } else if current_mode.is_some() && !line.trim().is_empty() {
                // Only add non-empty lines to the current entry
                current_entry.push_str(&line);
                current_entry.push(' ');
            }
        }

        // Add final entry if exists
        if !current_entry.is_empty() && current_mode.is_some() {
            let _ = vector_memory.add(
                current_entry.trim().to_string(),
                current_mode.unwrap(),
                None
            ).await;
        }
    }

    /// Helper: Read recent autonomous thoughts from log file (legacy - kept for compatibility)
    fn _read_recent_autonomous_thoughts(log_path: &str, mode: &str, count: usize) -> Vec<String> {
        use std::fs::File;
        use std::io::{BufRead, BufReader};

        let file = match File::open(log_path) {
            Ok(f) => f,
            Err(_) => return vec![],
        };

        let reader = BufReader::new(file);
        let mut entries = Vec::new();
        let mut current_entry = String::new();
        let mut in_target_mode = false;

        for line in reader.lines().filter_map(Result::ok) {
            if line.contains(&format!("[{}]", mode)) {
                if !current_entry.is_empty() {
                    entries.push(current_entry.trim().to_string());
                }
                current_entry = String::new();
                in_target_mode = true;
            } else if line.starts_with('[') && line.contains("MODE") {
                if in_target_mode && !current_entry.is_empty() {
                    entries.push(current_entry.trim().to_string());
                }
                current_entry = String::new();
                in_target_mode = false;
            } else if in_target_mode {
                current_entry.push_str(&line);
                current_entry.push(' ');
            }
        }

        if in_target_mode && !current_entry.is_empty() {
            entries.push(current_entry.trim().to_string());
        }

        entries.into_iter().rev().take(count).collect()
    }
}
