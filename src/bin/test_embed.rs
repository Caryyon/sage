use sage::distributed_knowledge::embedder;

fn main() {
    println!("fastembed available: {}", embedder::is_available());
    println!("Model: {}", embedder::model_name());
    println!("Dim: {}", embedder::dimension());
    
    if let Some(embed) = embedder::embed_text_f64("hello world") {
        println!("Embedding works! First 5 values: {:?}", &embed[..5.min(embed.len())]);
        println!("Dim: {}", embed.len());
    } else {
        println!("Embedding FAILED - falling back to hash");
    }
}
