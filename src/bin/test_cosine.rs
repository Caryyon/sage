use sage::distributed_knowledge::embedder;
use sage::distributed_knowledge::encoder::{encode_text, EncoderConfig, NUM_EMBED_SLOTS};

fn main() {
    let config = EncoderConfig::default();
    
    let queries = ["Neural Cellular Automata", "Rust ownership memory safety", "gradient descent"];
    let stored = ["Neural Cellular Automata use local rules to produce global patterns",
                  "Rust ownership ensures memory safety without garbage collection",
                  "Gradient descent minimizes loss by adjusting weights iteratively",
                  "Linux process management uses signals like SIGKILL and SIGTERM",
                  "Docker containers provide isolated environments for applications"];
    
    // Encode all
    let q_feats: Vec<_> = queries.iter().map(|q| encode_text(q, &config)).collect();
    let s_feats: Vec<_> = stored.iter().map(|s| encode_text(s, &config)).collect();
    
    println!("NUM_EMBED_SLOTS = {}", NUM_EMBED_SLOTS);
    println!("Feature dim = {}", config.num_features);
    println!();
    
    // Extract slots (same as decoder does)
    let extract_slots = |f: &sage::distributed_knowledge::encoder::FeatureVector| -> [f64; NUM_EMBED_SLOTS] {
        let feat_len = f.values.len();
        let mut slots = [0.0f64; NUM_EMBED_SLOTS];
        for (i, slot) in slots.iter_mut().enumerate() {
            *slot = f.values[(i * feat_len / NUM_EMBED_SLOTS) % feat_len];
        }
        slots
    };
    
    for (qi, q) in queries.iter().enumerate() {
        let q_slots = extract_slots(&q_feats[qi]);
        let q_mag: f64 = q_slots.iter().map(|v| v*v).sum::<f64>().sqrt();
        println!("Query: \"{}\"", q);
        println!("  is_semantic: {}, slot magnitudes: first 5 = {:?}", q_feats[qi].is_semantic, &q_slots[..5.min(NUM_EMBED_SLOTS)]);
        
        for (si, s) in stored.iter().enumerate() {
            let s_slots = extract_slots(&s_feats[si]);
            let s_mag: f64 = s_slots.iter().map(|v| v*v).sum::<f64>().sqrt();
            let dot: f64 = q_slots.iter().zip(s_slots.iter()).map(|(a,b)| a*b).sum();
            let cos = if q_mag * s_mag > 0.0 { dot / (q_mag * s_mag) } else { 0.0 };
            println!("  [{:.4}] \"{}\"", cos, &s[..60.min(s.len())]);
        }
        println!();
    }
}
