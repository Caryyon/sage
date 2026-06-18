use sage::distributed_knowledge::{default_brain_path, KnowledgeStore, NCAKnowledge};
use sage::grid::{GRID_SIZE, KNOWLEDGE_ACTIVATION, KNOWLEDGE_CHANNELS_START};

fn main() {
    let mut knowledge = NCAKnowledge::new();
    knowledge.load(&default_brain_path()).unwrap();
    
    let grid = &knowledge.grid;
    let mut active_cells = 0;
    let mut sample_cells = Vec::new();
    
    for y in 0..GRID_SIZE {
        for x in 0..GRID_SIZE {
            let act = grid.cells[y][x][KNOWLEDGE_ACTIVATION];
            if act > 0.01 {
                active_cells += 1;
                if sample_cells.len() < 5 {
                    let embed: Vec<f64> = (0..48).map(|i| grid.cells[y][x][KNOWLEDGE_CHANNELS_START + i]).collect();
                    let text = knowledge.text_store.peek(x, y).map(|s| s.to_string());
                    sample_cells.push((x, y, act, embed, text));
                }
            }
        }
    }
    
    println!("Active cells: {}", active_cells);
    println!();
    
    for (x, y, act, embed, text) in &sample_cells {
        println!("Cell ({},{}): activation={:.4}", x, y, act);
        let non_zero: usize = embed.iter().filter(|v| v.abs() > 1e-6).count();
        let mag: f64 = embed.iter().map(|v| v*v).sum::<f64>().sqrt();
        println!("  Embedding: {} non-zero, magnitude={:.4}", non_zero, mag);
        println!("  First 5: {:?}", &embed[..5]);
        if let Some(t) = text {
            let preview = if t.len() > 80 { &t[..80] } else { t };
            println!("  Text: {}...", preview);
        } else {
            println!("  Text: (none)");
        }
        println!();
    }
    
    // Now test: encode a query and compare with cells
    use sage::distributed_knowledge::encoder::encode_text;
    let config = sage::distributed_knowledge::encoder::EncoderConfig::default();
    let q_features = encode_text("What is the capital of France?", &config);
    println!("Query: 'What is the capital of France?'");
    println!("  is_semantic: {}", q_features.is_semantic);
    println!("  values len: {}", q_features.values.len());
    println!("  first 5: {:?}", &q_features.values[..5]);
    let mag: f64 = q_features.values.iter().map(|v| v*v).sum::<f64>().sqrt();
    println!("  magnitude: {:.4}", mag);
    
    // Manually compute cosine sim with each sample cell
    println!("\nManual cosine similarities:");
    for (x, y, act, embed, text) in &sample_cells {
        let feat_len = q_features.values.len();
        let mut query_slots = [0.0f64; 48];
        for (i, slot) in query_slots.iter_mut().enumerate() {
            let feat_idx = (i * feat_len / 48) % feat_len;
            *slot = q_features.values[feat_idx];
        }
        let dot: f64 = query_slots.iter().zip(embed.iter()).map(|(a,b)| a*b).sum();
        let mag_q: f64 = query_slots.iter().map(|v| v*v).sum::<f64>().sqrt();
        let mag_c: f64 = embed.iter().map(|v| v*v).sum::<f64>().sqrt();
        let cos = if mag_q > 1e-10 && mag_c > 1e-10 { dot / (mag_q * mag_c) } else { 0.0 };
        println!("  Cell ({},{}): cos_sim={:.4}, act={:.4}, rel={:.4}", 
            x, y, cos, act, 0.8*cos.max(0.0) + 0.1*act);
        if let Some(t) = text {
            let preview = if t.len() > 60 { &t[..60] } else { t };
            println!("    text: {}...", preview);
        }
    }
}
