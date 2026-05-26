//! Example: Ingest custom documents into SAGE's knowledge grid
//!
//! This example shows how to programmatically add documents, notes, or
//! any text content to SAGE's NCA knowledge store.
//!
//! Run with: cargo run --example custom-knowledge-source

use sage::inference::OllamaEngine;
use sage::knowledge_loop::KnowledgeLoop;
use std::sync::Arc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create an inference engine (Ollama fallback for queries)
    let engine = Arc::new(OllamaEngine::default());

    // Initialize the knowledge loop
    let mut sage = KnowledgeLoop::new(engine);

    // Ingest a custom knowledge base
    let documents = vec![
        "SAGE uses a 256×256 neural grid to store knowledge with zero collisions.",
        "The knowledge grid has 38 channels per cell including RGBA, hidden, pattern, and knowledge channels.",
        "NCA consolidation applies Hebbian reinforcement to frequently accessed memories.",
        "libp2p gossip protocol syncs knowledge diffs between peers.",
        "Ed25519 signatures prevent knowledge poisoning attacks.",
    ];

    println!("Ingesting {} documents into SAGE...\n", documents.len());

    for (i, doc) in documents.iter().enumerate() {
        sage.encode(doc, 0.7); // confidence = 0.7 for user content
        println!(
            "  [{}/{}] {}",
            i + 1,
            documents.len(),
            &doc[..60.min(doc.len())]
        );
    }

    println!(
        "\n✅ Knowledge ingested. Grid now contains {} cells worth of semantic embeddings.",
        documents.len() * 64 // rough estimate: each doc touches a neighborhood
    );

    // Save the brain to disk
    sage.save_brain()?;
    println!("💾 Brain saved to ~/.sage/brain.bin");

    // Now query it
    let query = "How does SAGE prevent attacks on the knowledge network?";
    println!("\n🧠 Query: {}", query);

    let response = sage.chat(query)?;
    println!("🤖 Response: {}", response);

    Ok(())
}
