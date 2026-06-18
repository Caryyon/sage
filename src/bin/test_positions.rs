use sage::distributed_knowledge::encoder::{encode_text, feature_to_position, EncoderConfig};

fn main() {
    let config = EncoderConfig::default();
    let texts = [
        "Neural Cellular Automata",
        "Rust ownership memory safety",
        "gradient descent",
        "libp2p gossip protocol",
        "Linux process management",
        "Docker containers",
        "SSH key authentication",
        "KOAP HOA management",
        "Python is fast for NumPy",
        "Attention mechanisms",
    ];
    
    println!("Feature dim: {}", config.num_features);
    println!("Grid: 256x256");
    println!();
    for t in &texts {
        let f = encode_text(t, &config);
        let (x, y) = feature_to_position(&f, 256, 256);
        println!("({:3},{:3})  is_semantic={}  \"{}\"", x, y, f.is_semantic, t);
    }
}
