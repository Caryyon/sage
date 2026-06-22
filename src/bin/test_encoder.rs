use sage::distributed_knowledge::embedder;
use sage::distributed_knowledge::encoder::{encode_text, EncoderConfig};

fn main() {
    println!("fastembed available: {}", embedder::is_available());
    println!("fastembed dim: {}", embedder::dimension());
    
    let config = EncoderConfig::default();
    println!("Config ollama_url: {:?}", config.ollama_url);
    println!("Config embedding_model: {}", config.embedding_model);
    
    let features = encode_text("What is the capital of France?", &config);
    println!("Feature vector size: {}", features.values.len());
    println!("Is semantic: {}", features.is_semantic);
    println!("First 5 values: {:?}", &features.values[..5]);
    
    // Try direct fastembed
    if let Some(emb) = embedder::embed_text_f64("What is the capital of France?") {
        println!("\nDirect fastembed: {} dims", emb.len());
        println!("First 5: {:?}", &emb[..5]);
    } else {
        println!("\nDirect fastembed: FAILED");
    }
    
    // Try ollama
    let ollama_url = "http://localhost:11434";
    let model = "nomic-embed-text";
    let client = reqwest::blocking::Client::new();
    let res = client.post(&format!("{}/api/embeddings", ollama_url))
        .json(&serde_json::json!({"model": model, "prompt": "test"}))
        .send();
    match res {
        Ok(r) => println!("\nOllama embed API: {} status", r.status()),
        Err(e) => println!("\nOllama embed API: FAILED - {}", e),
    }
}
