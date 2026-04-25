//! Offline demo — no internet required

use sage::knowledge_loop::KnowledgeLoop;
use sage::inference::OfflineEngine;
use std::sync::Arc;

fn main() {
    // Create offline engine (no LLM needed)
    let engine = Arc::new(OfflineEngine::new());
    
    // Create knowledge loop with trained NCA weights
    let mut sage = KnowledgeLoop::new(engine);
    
    println!("SAGE Offline Demo — No internet required!");
    
    // Simple questions work offline
    let response = sage.chat("What is SAGE?").unwrap();
    println!("Q: What is SAGE?");
    println!("A: {}", response);
    
    // Complex questions need LLM (will show fallback message)
    let response = sage.chat("Why does NCA converge?").unwrap();
    println!("Q: Why does NCA converge?");
    println!("A: {}", response);
}
