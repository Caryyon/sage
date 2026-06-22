use sage::distributed_knowledge::{default_brain_path, KnowledgeStore, NCAKnowledge};

fn main() {
    let brain_path = default_brain_path();
    let backup = format!("{}.saturated", brain_path);
    let _ = std::fs::copy(&brain_path, &backup);
    println!("Backed up saturated brain to {}", backup);
    
    let fresh = NCAKnowledge::new();
    match fresh.save(&brain_path) {
        Ok(()) => println!("Cleared brain at {}", brain_path),
        Err(e) => eprintln!("Failed: {}", e),
    }
}
