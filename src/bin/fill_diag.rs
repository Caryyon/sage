use sage::distributed_knowledge::{NCAKnowledge, KnowledgeStore};
use sage::distributed_knowledge::encoder::{encode_text, feature_to_position, EncoderConfig};
use sage::grid::GRID_SIZE;

fn main() {
    let store = NCAKnowledge::new();
    let default_config = EncoderConfig::default();
    
    println!("Store config: {:?}", store.config);
    println!("\nDefault config: {:?}", default_config);
    println!("\nEqual: {}", store.config.num_features == default_config.num_features 
        && store.config.ngram_sizes == default_config.ngram_sizes
        && store.config.num_hash_positions == default_config.num_hash_positions
        && store.config.spread_radius == default_config.spread_radius);
    
    let text = "fill test item number 9";
    let f1 = encode_text(text, &store.config);
    let f2 = encode_text(text, &default_config);
    let (x1, y1) = feature_to_position(&f1, GRID_SIZE, GRID_SIZE);
    let (x2, y2) = feature_to_position(&f2, GRID_SIZE, GRID_SIZE);
    println!("\nStore config features: is_semantic={}, len={}, pos=({},{})", f1.is_semantic, f1.values.len(), x1, y1);
    println!("Default config features: is_semantic={}, len={}, pos=({},{})", f2.is_semantic, f2.values.len(), x2, y2);
    println!("Feature values match: {}", f1.values == f2.values);
}
