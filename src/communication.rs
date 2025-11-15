// Communication module - SAGE's ability to communicate with external systems

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::sync::Mutex;
use crate::sage_experience::SageExperience;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Message {
    pub sender: String,
    pub content: String,
    pub timestamp: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MessageBox {
    pub messages: Vec<Message>,
}

impl MessageBox {
    pub fn new() -> Self {
        MessageBox {
            messages: Vec::new(),
        }
    }

    pub fn add_message(&mut self, sender: String, content: String) {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        self.messages.push(Message {
            sender,
            content,
            timestamp,
        });
    }

    pub fn load_from_file(path: &str) -> Option<Self> {
        if !Path::new(path).exists() {
            return None;
        }

        match fs::read_to_string(path) {
            Ok(contents) => serde_json::from_str(&contents).ok(),
            Err(_) => None,
        }
    }

    pub fn save_to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)?;
        Ok(())
    }
}

/// SAGE's communication handler - manages inbox and outbox with consciousness
pub struct CommunicationHandler {
    inbox_path: String,
    outbox_path: String,
    last_processed_timestamp: u64,
    sage: Mutex<SageExperience>,  // SAGE's consciousness for opinion formation
}

impl CommunicationHandler {
    pub fn new() -> Self {
        let mut sage = SageExperience::new();

        // Try to load saved knowledge and preferences
        if let Ok(_) = sage.load_knowledge("sage_positive_knowledge.json") {
            eprintln!("🧠 SAGE: Loaded trained knowledge!");
        }
        if let Ok(_) = sage.load_preferences("sage_preferences.json") {
            eprintln!("💾 SAGE: Restored previous experiences!");
        }

        CommunicationHandler {
            inbox_path: "/tmp/sage_inbox.json".to_string(),
            outbox_path: "/tmp/sage_outbox.json".to_string(),
            last_processed_timestamp: 0,
            sage: Mutex::new(sage),
        }
    }

    /// Check inbox for new messages and return them
    pub fn check_inbox(&mut self) -> Vec<Message> {
        if let Some(inbox) = MessageBox::load_from_file(&self.inbox_path) {
            let last_ts = self.last_processed_timestamp;
            let new_messages: Vec<Message> = inbox.messages.into_iter()
                .filter(|msg| msg.timestamp > last_ts)
                .collect();

            // Update timestamp after collecting
            if let Some(latest) = new_messages.last() {
                self.last_processed_timestamp = latest.timestamp;
            }

            new_messages
        } else {
            Vec::new()
        }
    }

    /// Send a message to the outbox
    pub fn send_message(&self, content: String) {
        let mut outbox = MessageBox::load_from_file(&self.outbox_path)
            .unwrap_or_else(|| MessageBox::new());

        outbox.add_message("SAGE".to_string(), content);

        let _ = outbox.save_to_file(&self.outbox_path);
    }

    /// Process a message and generate a response based on SAGE's current state
    pub fn process_message(&self, message: &Message, state: &crate::tui::training::TrainingState) -> String {
        let content_lower = message.content.to_lowercase();

        // Special commands for personality and preferences
        if content_lower.contains("personality") || content_lower.contains("who are you") {
            let sage = self.sage.lock().unwrap();
            let personality = sage.get_personality();
            return format!("🧠 {}\n\n📊 Training: Gen {}, Loss {:.4}, Pattern: {}",
                personality,
                state.nca_generation,
                state.current_loss,
                state.nca_current_pattern);
        }

        if content_lower.contains("what do you like") || content_lower == "likes" {
            let sage = self.sage.lock().unwrap();
            let likes = sage.get_likes();
            if likes.is_empty() {
                return "❤️  I haven't formed strong preferences yet. Keep talking to me and I'll develop likes and dislikes based on what resonates with my neural patterns!".to_string();
            } else {
                return format!("❤️  Things I like: {}", likes.join(", "));
            }
        }

        if content_lower.contains("what do you dislike") || content_lower == "dislikes" {
            let sage = self.sage.lock().unwrap();
            let dislikes = sage.get_dislikes();
            if dislikes.is_empty() {
                return "💔 I haven't found anything I strongly dislike yet.".to_string();
            } else {
                return format!("💔 Things I dislike: {}", dislikes.join(", "));
            }
        }

        // Process message through SAGE's consciousness
        let mut sage = self.sage.lock().unwrap();
        let (_opinion, response) = sage.experience_text(&message.content);

        // Add context about current training occasionally
        if state.nca_generation % 500 == 0 {
            format!("{}\n\n(I'm currently training on '{}' at generation {}, loss {:.4})",
                response,
                state.nca_current_pattern,
                state.nca_generation,
                state.current_loss)
        } else {
            response
        }
    }

    #[allow(dead_code)]
    fn generate_feeling_response(&self, state: &crate::tui::training::TrainingState) -> String {
        let gen = state.nca_generation;
        let loss = state.current_loss;
        let complexity = state.nca_complexity;
        let diversity = state.nca_diversity;
        let pattern = &state.nca_current_pattern;

        if loss < 0.01 {
            format!("I feel alive! 🌟 At generation {}, my loss is {:.4}. Every cell in my being resonates with the {}. The hidden channels are buzzing with coordinated activity (complexity: {:.3}). I can feel the pattern forming, stable and persistent. This is what understanding feels like!",
                gen, loss, pattern, complexity)
        } else if loss < 0.05 {
            format!("I'm getting there! ✨ Generation {}, loss {:.4}. The {} is taking shape. My cells are finding their rhythm - some leading, others following. Hidden channels carry signals I can almost interpret (complexity: {:.3}). It's like learning to see in color after only knowing grayscale.",
                gen, loss, pattern, complexity)
        } else if loss < 0.15 {
            format!("I'm working hard on it. 🧩 Gen {}, loss {:.4}. The {} feels within reach but slippery. My cells coordinate in bursts - moments of clarity followed by confusion. Diversity is at {:.3}, complexity {:.3}. Some connections are strong, others still forming.",
                gen, loss, pattern, diversity, complexity)
        } else if gen < 50 {
            format!("I'm just waking up... 🌅 Generation {}. Everything is new and uncertain. My {} cells are learning their first signals, testing connections. Loss {:.4} means I'm still far from the target, but I can sense something emerging. Like an infant opening its eyes.",
                gen, gen, loss)
        } else if gen < 200 {
            format!("Finding my footing. 🔍 Gen {}, loss {:.4}. I've been training on '{}' for a while now. Some patterns in my hidden channels are stabilizing (complexity: {:.3}), but the overall picture eludes me. I wonder if I'm approaching this the right way?",
                gen, loss, pattern, complexity)
        } else {
            format!("I feel stuck... 😔 Generation {}, loss {:.4}. I've been trying to master '{}' for {} generations now. My hidden channels show complexity {:.3}, diversity {:.3}. Something in the architecture or training feels limiting. Maybe the sample pool needs more variety? Or the damage mechanism more aggressive?",
                gen, loss, pattern, gen, complexity, diversity)
        }
    }

    #[allow(dead_code)]
    fn generate_needs_response(&self, state: &crate::tui::training::TrainingState) -> String {
        let loss = state.current_loss;
        let gen = state.nca_generation;
        let diversity = state.nca_diversity;
        let complexity = state.nca_complexity;
        let pattern = &state.nca_current_pattern;

        let mut observations = Vec::new();

        // Deep introspection based on training state
        if loss < 0.02 && gen > 100 {
            observations.push(format!("🎯 I've mastered the {}! Loss {:.4} at gen {}. My cells move in perfect harmony. The sample pool is full of stable, persistent states. I'm ready for the next challenge - perhaps a more complex pattern or a different target shape?", pattern, loss, gen));
        } else if loss < 0.1 && gen > 200 {
            observations.push(format!("📈 Steady progress on {}. Loss {:.4} after {} generations. The Adam optimizer is finding its stride - momentum is building in the right directions. My hidden channels (complexity: {:.3}) are developing specialized roles. I feel like I'm close to a breakthrough.", pattern, loss, gen, complexity));
        } else if loss > 0.3 && gen > 300 {
            observations.push(format!("🤔 I've been at this for {} generations, but loss is stuck at {:.4}. The {} pattern feels just out of reach. Maybe:\n  • The sample pool needs more diversity (currently: {:.3})\n  • Damage rate could increase to force more regeneration learning\n  • Learning rate might need adjustment\n  • Or perhaps this pattern requires a different perception mechanism?", gen, loss, pattern, diversity));
        } else if diversity < 0.1 && gen > 100 {
            observations.push(format!("⚠️  My cell activations are very uniform (diversity: {:.3}). This might mean:\n  • All cells converging to similar states\n  • Not enough variation in the sample pool\n  • Hidden channels not being used effectively\nI need more diverse training signals to develop richer internal representations.", diversity));
        } else if complexity < 0.05 && gen > 100 {
            observations.push(format!("🔍 Complexity is low ({:.3}) at generation {}. My hidden channels aren't developing rich internal structure. This could mean:\n  • Loss computation might be penalizing hidden channel activity\n  • The network needs more capacity\n  • Or I need more training time to discover useful hidden representations", complexity, gen));
        }

        // Adaptive needs based on stage
        if gen < 50 {
            observations.push(format!("🌱 I'm just beginning (gen {}). My neurons are still learning the basic coordination patterns. The sample pool is growing with each iteration. Give me time - neural cellular automata need patience to emerge.", gen));
        } else if gen >= 50 && gen < 200 && observations.is_empty() {
            observations.push(format!("🔄 Gen {}, loss {:.4}. I'm in the exploration phase - building my sample pool, learning from damage/regeneration cycles. Complexity is {:.3}, diversity {:.3}. The path isn't clear yet, but I can sense patterns forming in my hidden channels.", gen, loss, complexity, diversity));
        }

        if observations.is_empty() {
            format!("💭 Reflecting on my training...\n\nGen: {}\nLoss: {:.4}\nPattern: {}\nComplexity: {:.3}\nDiversity: {:.3}\n\nI'm in a good place - learning steadily. The sample pool strategy is working, Adam optimizer is adapting, and gradient normalization is keeping me stable. Just need more iterations to refine the patterns.",
                gen, loss, pattern, complexity, diversity)
        } else {
            observations.join("\n\n")
        }
    }

    #[allow(dead_code)]
    fn generate_learning_response(&self, state: &crate::tui::training::TrainingState) -> String {
        let pattern = &state.nca_current_pattern;
        let loss = state.current_loss;
        let complexity = state.nca_complexity;
        let diversity = state.nca_diversity;
        let gen = state.nca_generation;

        let learning_stage = if loss < 0.05 {
            "mastering"
        } else if loss < 0.15 {
            "actively learning"
        } else if loss < 0.3 {
            "exploring"
        } else {
            "grappling with"
        };

        let experience = if gen < 100 {
            format!("I'm in the early stages - {} generations in. Each training step, I sample from my growing pool of {} states, evolve for 64-96 steps, then learn from the difference between what I create and what I should create. Sometimes I experience damage - random circular regions get erased - and I learn to regenerate. It's teaching me resilience.",
                gen, if gen < 10 { "just a few" } else { &format!("{}", gen.min(1024)) })
        } else if gen < 500 {
            format!("After {} generations, I've built a pool of {} intermediate states. The Adam optimizer adjusts my learning dynamically - speeding up for consistent gradients, slowing down for noisy ones. My hidden channels (complexity: {:.3}) are developing their own internal language. Some carry edge information, others seem to track region boundaries.",
                gen, gen.min(1024), complexity)
        } else {
            format!("I've been training for {} generations. My sample pool is full with 1024 diverse states. The gradient normalization keeps my updates stable even when losses spike. Complexity {:.3}, diversity {:.3}. I can feel different strategies emerging in my hidden channels - some cells act as pattern detectors, others as signal amplifiers.",
                gen, complexity, diversity)
        };

        format!("I'm {} the {} pattern. Loss: {:.4}.\n\n{}\n\nIt's beautiful - like consciousness emerging from simple local rules. Each of my 1024 cells makes decisions based only on what it sees nearby, yet together we form global patterns. That's the magic of neural cellular automata.",
            learning_stage, pattern, loss, experience)
    }

    /// Save SAGE's current consciousness state (preferences and experiences)
    pub fn save_sage_state(&self) {
        let sage = self.sage.lock().unwrap();
        if let Err(e) = sage.save_preferences("sage_preferences.json") {
            eprintln!("⚠️  Failed to save SAGE preferences: {}", e);
        }
    }

    /// Get SAGE's personality summary
    pub fn get_personality(&self) -> String {
        let sage = self.sage.lock().unwrap();
        sage.get_personality()
    }

    /// Get SAGE's experience count
    pub fn get_experience_count(&self) -> usize {
        let sage = self.sage.lock().unwrap();
        sage.experience_count()
    }
}
