//! Simple chat example using SAGE

use sage::knowledge_loop::KnowledgeLoop;
use sage::inference::OllamaEngine;
use std::sync::Arc;

fn main() {
    // Create Ollama engine for LLM fallback
    let engine = Arc::new(OllamaEngine::default());
    
    // Create knowledge loop
    let mut sage = KnowledgeLoop::new(engine);
    
    // Chat
    println!("SAGE: Hello! Ask me anything.");
    
    let response = sage.chat("What is SAGE?").unwrap();
    println!("SAGE: {}", response);
}
