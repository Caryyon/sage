//! Check specific entries in the HDC store
use sage::hdc::{default_hdc_path, HdcStore};
use std::path::Path;

fn main() {
    let store = HdcStore::load(Path::new(&default_hdc_path())).unwrap();
    println!("Loaded {} entries\n", store.len());
    
    let client = reqwest::blocking::Client::new();
    
    // Test: Who wrote Alice in Wonderland?
    let q = "Who wrote Alice in Wonderland?";
    let res = client.post("http://localhost:11434/api/embeddings")
        .json(&serde_json::json!({"model":"nomic-embed-text","prompt":q}))
        .send().unwrap();
    let resp: serde_json::Value = res.json().unwrap();
    let emb: Vec<f32> = resp["embedding"].as_array().unwrap()
        .iter().map(|v| v.as_f64().unwrap() as f32).collect();
    
    let results = store.query(&emb, 5);
    println!("Q: {}", q);
    for (i, (rel, text)) in results.iter().enumerate() {
        let has_carroll = text.to_lowercase().contains("carroll");
        let has_alice = text.to_lowercase().contains("alice");
        println!("  #{} [rel={:.4}] carroll={} alice={}", i, rel, has_carroll, has_alice);
        println!("    {}", &text[..text.len().min(250)]);
        println!();
    }
    
    // Also test: how many chunks mention "Lewis Carroll"
    let carroll_count = store.entries.iter()
        .filter(|e| e.text.to_lowercase().contains("lewis carroll"))
        .count();
    println!("Entries with 'Lewis Carroll': {}", carroll_count);
    
    let alice_count = store.entries.iter()
        .filter(|e| e.text.to_lowercase().contains("alice"))
        .count();
    println!("Entries with 'alice': {}", alice_count);
    
    // Show one entry that mentions Carroll
    println!("\n--- Sample Carroll entry ---");
    if let Some(e) = store.entries.iter().find(|e| e.text.to_lowercase().contains("carroll")) {
        println!("len={}: {}", e.text.len(), &e.text[..e.text.len().min(300)]);
    }
    
    // Show one entry that mentions Watson
    println!("\n--- Sample Watson entry ---");
    if let Some(e) = store.entries.iter().find(|e| e.text.to_lowercase().contains("watson")) {
        println!("len={}: {}", e.text.len(), &e.text[..e.text.len().min(300)]);
    }
}