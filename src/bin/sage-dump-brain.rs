use sage::brain_templates::BrainTemplateBundle;
use sage::distributed_knowledge::KnowledgeStore;

fn main() {
    let path = std::env::args().nth(1).expect("Usage: sage-dump-brain <template_file>");
    let data = std::fs::read(&path).expect("Could not read file");
    let bundle: BrainTemplateBundle = bincode::deserialize(&data).expect("Could not parse");
    
    let knowledge = bundle.to_knowledge();
    println!("Brain: {}", bundle.meta.name);
    println!("Active cells: {}", knowledge.active_knowledge(0.01).len());
    println!("Text entries: {}", knowledge.text_store.len());
    println!();
    
    // Dump all text entries
    for y in 0..bundle.grid.height {
        for x in 0..bundle.grid.width {
            if let Some(text) = knowledge.text_store.peek(x, y) {
                let preview: String = text.chars().take(200).collect();
                println!("[{},{}] {}", x, y, preview);
            }
        }
    }
}
