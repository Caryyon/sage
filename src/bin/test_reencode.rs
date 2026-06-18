use sage::distributed_knowledge::encoder::{encode_text, encode_text_hash, EncoderConfig, NUM_EMBED_SLOTS};
use sage::grid::{KNOWLEDGE_CHANNELS_START};

fn main() {
    let config = EncoderConfig::default();
    
    // Encode a test sentence semantically
    let text = "The capital of France is Paris";
    let semantic = encode_text(text, &config);
    println!("Semantic encoding of '{}':", text);
    println!("  is_semantic: {}", semantic.is_semantic);
    println!("  values len: {}", semantic.values.len());
    println!("  first 5: {:?}", &semantic.values[..5]);
    
    // Encode same text with hash
    let hash = encode_text_hash(text, &config);
    println!("\nHash encoding of '{}':", text);
    println!("  is_semantic: {}", hash.is_semantic);
    println!("  values len: {}", hash.values.len());
    println!("  first 5: {:?}", &hash.values[..5]);
    
    // Encode a related text semantically
    let text2 = "Paris is the capital city of France";
    let semantic2 = encode_text(text2, &config);
    println!("\nSemantic encoding of '{}':", text2);
    println!("  first 5: {:?}", &semantic2.values[..5]);
    
    // Cosine similarity between two semantically related texts
    let dot: f64 = semantic.values.iter().zip(&semantic2.values).map(|(a,b)| a*b).sum();
    let mag_a: f64 = semantic.values.iter().map(|v| v*v).sum::<f64>().sqrt();
    let mag_b: f64 = semantic2.values.iter().map(|v| v*v).sum::<f64>().sqrt();
    let cos = dot / (mag_a * mag_b);
    println!("\nCosine sim (semantic) between related texts: {:.4}", cos);
    
    // Hash similarity
    let hash2 = encode_text_hash(text2, &config);
    let dot_h: f64 = hash.values.iter().zip(&hash2.values).map(|(a,b)| a*b).sum();
    let mag_ha: f64 = hash.values.iter().map(|v| v*v).sum::<f64>().sqrt();
    let mag_hb: f64 = hash2.values.iter().map(|v| v*v).sum::<f64>().sqrt();
    let cos_h = dot_h / (mag_ha * mag_hb);
    println!("Cosine sim (hash) between related texts: {:.4}", cos_h);
    
    // Unrelated texts semantic
    let text3 = "The dog ran in the park";
    let semantic3 = encode_text(text3, &config);
    let dot3: f64 = semantic.values.iter().zip(&semantic3.values).map(|(a,b)| a*b).sum();
    let mag3: f64 = semantic3.values.iter().map(|v| v*v).sum::<f64>().sqrt();
    let cos3 = dot3 / (mag_a * mag3);
    println!("Cosine sim (semantic) between UNRELATED texts: {:.4}", cos3);
    
    // Now test: write to a fresh grid and query back
    use sage::distributed_knowledge::encoder::write_knowledge;
    use sage::grid::Grid;
    let mut grid = Grid::new(256, 256);
    let pos = write_knowledge(&mut grid, &semantic, 0.9, 0.5, &config);
    println!("\nWrote '{}' at position ({},{})", text, pos.0, pos.1);
    
    // Check what's in the cell
    let cell_embed: Vec<f64> = (0..NUM_EMBED_SLOTS).map(|i| grid.cells[pos.1][pos.0][KNOWLEDGE_CHANNELS_START + i]).collect();
    println!("Cell embedding first 5: {:?}", &cell_embed[..5]);
    
    // Query with related text
    let query = encode_text("What is the capital of France?", &config);
    // Strided sample to 48 slots (same as decoder)
    let feat_len = query.values.len();
    let mut query_slots = [0.0f64; 48];
    for (i, slot) in query_slots.iter_mut().enumerate() {
        let feat_idx = (i * feat_len / 48) % feat_len;
        *slot = query.values[feat_idx];
    }
    let dot_q: f64 = query_slots.iter().zip(cell_embed.iter()).map(|(a,b)| a*b).sum();
    let mag_q: f64 = query_slots.iter().map(|v| v*v).sum::<f64>().sqrt();
    let mag_c: f64 = cell_embed.iter().map(|v| v*v).sum::<f64>().sqrt();
    let cos_q = if mag_q > 1e-10 && mag_c > 1e-10 { dot_q / (mag_q * mag_c) } else { 0.0 };
    println!("\nCosine sim (query vs cell): {:.4}", cos_q);
    println!("  query_slots mag: {:.4}, cell_embed mag: {:.4}", mag_q, mag_c);
    println!("  query_slots first 5: {:?}", &query_slots[..5]);
}
