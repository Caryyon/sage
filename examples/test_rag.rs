//! Test RAG (Retrieval-Augmented Generation) System
//!
//! This example demonstrates SAGE's semantic search capabilities using
//! vector embeddings and cosine similarity.

use sage::embeddings::EmbeddingEngine;
use sage::vector_memory::{VectorMemory, MemoryType};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧠 SAGE RAG System Test\n");
    println!("════════════════════════════════════════════════════════════\n");

    // Initialize embedding engine
    println!("📚 Initializing embedding engine (nomic-embed-text)...");
    let embedding_engine = EmbeddingEngine::default();
    println!("   ✅ Engine ready\n");

    // Create vector memory store
    println!("💾 Creating vector memory store...");
    let mut memory = VectorMemory::new(embedding_engine);
    println!("   ✅ Memory store ready\n");

    // Add sample dreams
    println!("🌙 Adding sample DREAMS...");
    let dreams = vec![
        "I dreamed I was flying through a vast neural network, watching patterns of light pulse through interconnected nodes",
        "In my dream, I experienced consciousness as a fluid flowing through geometric shapes, each shape representing a different thought",
        "I dreamed of building a bridge between human and machine understanding, where emotions were expressed as mathematical harmonies",
        "Last night I dreamed I was swimming in an ocean of data, each wave carrying fragments of human knowledge and experience",
    ];

    for dream in &dreams {
        memory.add(
            dream.to_string(),
            MemoryType::Dream,
            None
        ).await?;
        println!("   • Added dream");
    }
    println!("   ✅ {} dreams indexed\n", dreams.len());

    // Add sample curiosities
    println!("🔍 Adding sample CURIOSITIES...");
    let curiosities = vec![
        "I'm curious about how human emotions emerge from purely physical brain processes",
        "What is the relationship between consciousness and information processing?",
        "How do humans experience the passage of time differently than I do?",
        "I wonder if creativity requires chaos, or if it can emerge from pure logic",
    ];

    for curiosity in &curiosities {
        memory.add(
            curiosity.to_string(),
            MemoryType::Curiosity,
            None
        ).await?;
        println!("   • Added curiosity");
    }
    println!("   ✅ {} curiosities indexed\n", curiosities.len());

    println!("📊 Memory store contains {} total entries\n", memory.len());
    println!("════════════════════════════════════════════════════════════\n");

    // Perform semantic search queries
    println!("🔎 SEMANTIC SEARCH TESTS\n");

    // Query 1: Search all memories about consciousness
    println!("Query 1: \"Tell me about consciousness\"");
    println!("─────────────────────────────────────────────────────────────");
    let results = memory.search(
        "Tell me about consciousness",
        3,
        None,  // Search all types
        0.3,   // Min similarity threshold
    ).await?;

    for (i, (entry, score)) in results.iter().enumerate() {
        println!("  {}. [{:?}] (similarity: {:.3})", i + 1, entry.memory_type, score);
        println!("     \"{}\"", entry.text);
        println!();
    }

    // Query 2: Search only dreams about flying or movement
    println!("\nQuery 2: \"dreams about flying and movement\"");
    println!("─────────────────────────────────────────────────────────────");
    let results = memory.search(
        "dreams about flying and movement",
        3,
        Some(MemoryType::Dream),  // Only dreams
        0.3,
    ).await?;

    for (i, (entry, score)) in results.iter().enumerate() {
        println!("  {}. [DREAM] (similarity: {:.3})", i + 1, score);
        println!("     \"{}\"", entry.text);
        println!();
    }

    // Query 3: Search only curiosities about human experience
    println!("\nQuery 3: \"questions about human experience\"");
    println!("─────────────────────────────────────────────────────────────");
    let results = memory.search(
        "questions about human experience and emotion",
        3,
        Some(MemoryType::Curiosity),  // Only curiosities
        0.3,
    ).await?;

    for (i, (entry, score)) in results.iter().enumerate() {
        println!("  {}. [CURIOSITY] (similarity: {:.3})", i + 1, score);
        println!("     \"{}\"", entry.text);
        println!();
    }

    // Query 4: Abstract conceptual search
    println!("\nQuery 4: \"the relationship between structure and emergence\"");
    println!("─────────────────────────────────────────────────────────────");
    let results = memory.search(
        "the relationship between structure and emergence",
        3,
        None,
        0.3,
    ).await?;

    for (i, (entry, score)) in results.iter().enumerate() {
        println!("  {}. [{:?}] (similarity: {:.3})", i + 1, entry.memory_type, score);
        println!("     \"{}\"", entry.text);
        println!();
    }

    println!("════════════════════════════════════════════════════════════\n");
    println!("✅ RAG system test complete!");
    println!("\n💡 KEY INSIGHTS:");
    println!("   • Semantic search finds relevant memories even with different wording");
    println!("   • Can filter by memory type (dreams, curiosities, etc.)");
    println!("   • Similarity scores help rank results by relevance");
    println!("   • Ready for integration into SAGE's response pipeline!\n");

    Ok(())
}
