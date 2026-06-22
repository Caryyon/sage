use sage::distributed_knowledge::{KnowledgeStore, NCAKnowledge, default_brain_path};
use sage::distributed_knowledge::encoder::encode_text;

fn main() {
    let brain_path = default_brain_path();
    let mut knowledge = NCAKnowledge::new();
    knowledge.load(&brain_path).unwrap();
    
    println!("Brain: {} alive, {} entries", knowledge.grid.alive_count(), knowledge.text_store.len());
    
    for q in &["Neural Cellular Automata", "Rust ownership memory safety", "gradient descent", "KOAP HOA", "libp2p gossip protocol"] {
        let results = knowledge.query(q, 5);
        println!("\nQuery '{}': {} results", q, results.len());
        for (i, r) in results.iter().enumerate() {
            let text = knowledge.text_store.get(r.position.0, r.position.1).unwrap_or("(no text)");
            println!("  {}. [rel={:.3}, act={:.3}] {}", i+1, r.relevance, r.activation, 
                &text[..text.len().min(80)]);
        }
    }
}