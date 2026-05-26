//! Example: Inspect and tune the intelligent query router
//!
//! SAGE v0.3.7+ ships with a self-improving query router that learns which
//! queries are best handled by the local NCA grid vs an LLM backend.
//!
//! This example shows how to:
//! - Classify queries by pattern
//! - Inspect router accuracy stats
//! - Tune complexity thresholds
//! - Simulate feedback to train the router
//!
//! Run with: cargo run --example router-tuning

use sage::query_router_intelligent::{
    Backend, IntelligentRouter, QueryComplexity, QueryPattern, RoutingOutcome,
};
use std::collections::HashMap;

fn main() {
    println!("🧭 SAGE Intelligent Query Router — Inspection & Tuning\n");

    // Create or load a router (persists to ~/.sage/intelligent_router.json)
    let mut router = IntelligentRouter::new()
        .with_nca_available(true)
        .with_exploration_rate(0.2);

    // Example queries across all pattern types
    let test_queries: Vec<(&str, QueryPattern, QueryComplexity)> = vec![
        (
            "What is SAGE?",
            QueryPattern::FactualLookup,
            QueryComplexity::Simple,
        ),
        (
            "When was SAGE released?",
            QueryPattern::Temporal,
            QueryComplexity::Simple,
        ),
        (
            "Where is my brain file stored?",
            QueryPattern::Spatial,
            QueryComplexity::Simple,
        ),
        (
            "How many channels per cell?",
            QueryPattern::Quantitative,
            QueryComplexity::Simple,
        ),
        (
            "What does NCA mean?",
            QueryPattern::Definitional,
            QueryComplexity::Simple,
        ),
        (
            "Compare SAGE and ChatGPT",
            QueryPattern::Comparative,
            QueryComplexity::Moderate,
        ),
        (
            "Why does knowledge decay?",
            QueryPattern::Causal,
            QueryComplexity::Moderate,
        ),
        (
            "How do I start a node?",
            QueryPattern::Procedural,
            QueryComplexity::Moderate,
        ),
        (
            "Analyze the architecture trade-offs",
            QueryPattern::Analytical,
            QueryComplexity::Complex,
        ),
        (
            "Write a poem about cellular automata",
            QueryPattern::Creative,
            QueryComplexity::Complex,
        ),
        (
            "Hello!",
            QueryPattern::Conversational,
            QueryComplexity::Simple,
        ),
        ("...", QueryPattern::Ambiguous, QueryComplexity::Complex),
    ];

    println!("📊 Classifying {} test queries:\n", test_queries.len());

    let mut route_counts: HashMap<String, usize> = HashMap::new();

    for (query, expected_pattern, _expected_complexity) in &test_queries {
        let (backend, pattern, confidence) = router.route(query, true);

        let pattern_match = if pattern == *expected_pattern {
            "✅"
        } else {
            "⚠️"
        };
        let route_key = format!("{:?}", pattern);
        *route_counts.entry(route_key).or_insert(0) += 1;

        let backend_name = match backend {
            Backend::Nca => "NCA",
            Backend::Llm => "LLM",
        };

        println!(
            "  {} {:30} → {:12} | confidence: {:.2}",
            pattern_match,
            format!("\"{}\"", query.chars().take(28).collect::<String>()),
            backend_name,
            confidence
        );
    }

    println!("\n📈 Pattern distribution:");
    let mut sorted: Vec<_> = route_counts.into_iter().collect();
    sorted.sort_by_key(|(_, count)| *count);
    sorted.reverse();
    for (pattern, count) in sorted {
        println!("  {:20}: {} queries", pattern, count);
    }

    println!("\n🔧 Router stats (before feedback):");
    println!("{}", router.summary());

    // Simulate user feedback to train the router
    println!("\n🎓 Training the router with simulated feedback...\n");

    // Simulate: NCA was correct for factual lookups
    router.record_outcome(
        QueryPattern::FactualLookup,
        RoutingOutcome {
            backend: Backend::Nca,
            success: true,
            response_time_ms: 45,
            user_satisfaction: Some(0.9),
        },
    );
    router.record_outcome(
        QueryPattern::Quantitative,
        RoutingOutcome {
            backend: Backend::Nca,
            success: true,
            response_time_ms: 52,
            user_satisfaction: Some(0.85),
        },
    );
    // Simulate: LLM was needed for creative queries
    router.record_outcome(
        QueryPattern::Creative,
        RoutingOutcome {
            backend: Backend::Llm,
            success: true,
            response_time_ms: 1200,
            user_satisfaction: Some(0.95),
        },
    );
    // Simulate: router made a mistake (LLM for simple factual)
    router.record_outcome(
        QueryPattern::Spatial,
        RoutingOutcome {
            backend: Backend::Llm,
            success: false,
            response_time_ms: 800,
            user_satisfaction: Some(0.3),
        },
    );

    println!("  Recorded 4 feedback events");
    println!("\n🔧 Router stats (after feedback):");
    println!("{}", router.summary());

    // Save learned thresholds
    let save_path = IntelligentRouter::default_path();
    router
        .save(&save_path)
        .expect("Failed to save router stats");
    println!("\n💾 Router stats saved to {}", save_path.display());

    println!("\n✅ Done! The router will remember these patterns for future sessions.");
}
