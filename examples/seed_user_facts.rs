//! Manual fact seeding utility
//!
//! Use this to manually add facts about users that SAGE should remember

use sage::fact_memory::{FactMemory, Fact};

fn main() {
    let mut fact_memory = FactMemory::new();

    // Seed facts for caryyon (カライーオン)
    println!("🌱 Seeding facts for caryyon...");

    fact_memory.add_or_update_fact(
        "caryyon",
        "name".to_string(),
        "Cary".to_string(),
        1.0, // High confidence
    );

    fact_memory.add_or_update_fact(
        "caryyon",
        "japanese_name".to_string(),
        "カライーオン".to_string(),
        1.0,
    );

    fact_memory.add_or_update_fact(
        "caryyon",
        "wife_name".to_string(),
        "Resse".to_string(),
        1.0,
    );

    fact_memory.add_or_update_fact(
        "caryyon",
        "son_name".to_string(),
        "Loki".to_string(),
        1.0,
    );

    // Display what SAGE would see
    println!("\n✅ Facts seeded! Here's what SAGE will know:\n");
    println!("{}", fact_memory.format_facts_for_context("caryyon"));

    // Save to JSON for now (until database integration is ready)
    let json = serde_json::to_string_pretty(&fact_memory).expect("Failed to serialize");
    std::fs::write("/tmp/sage_facts.json", json).expect("Failed to write file");

    println!("💾 Facts saved to /tmp/sage_facts.json");
    println!("\n📝 To use these facts, modify the Discord bot to load from this file on startup.");
}
