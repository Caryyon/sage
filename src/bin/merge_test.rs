//! Quick merge quality test — verifies collision-aware merge fix.

use sage::distributed_knowledge::{NCAKnowledge, KnowledgeStore};

fn main() {
    println!("=== Merge Quality Test ===\n");

    let items_per_node = 50;
    let mut store_a = NCAKnowledge::new().with_node_id(1.0);
    store_a.config.ollama_url = None;
    let mut store_b = NCAKnowledge::new().with_node_id(2.0);
    store_b.config.ollama_url = None;

    let mut a_facts = Vec::new();
    let mut b_facts = Vec::new();

    for i in 0..items_per_node {
        let fact_a = format!("alpha knowledge item {}", i);
        let fact_b = format!("beta knowledge item {}", i);
        store_a.encode(&fact_a, 0.9);
        store_b.encode(&fact_b, 0.9);
        a_facts.push(fact_a);
        b_facts.push(fact_b);
    }

    // Check pre-merge retrievability
    let a_pre = a_facts.iter().filter(|f| {
        let r = store_a.query(f, 3);
        r.iter().any(|res| res.text.as_deref() == Some(f.as_str()))
    }).count();
    let b_pre = b_facts.iter().filter(|f| {
        let r = store_b.query(f, 3);
        r.iter().any(|res| res.text.as_deref() == Some(f.as_str()))
    }).count();

    println!("Pre-merge: A={}/{} B={}/{}", a_pre, items_per_node, b_pre, items_per_node);

    // Merge B into A (with text store!)
    store_a.merge_with_text(&store_b, 0.8);

    // Check post-merge retrievability
    let a_post = a_facts.iter().filter(|f| {
        let r = store_a.query(f, 3);
        r.iter().any(|res| res.text.as_deref() == Some(f.as_str()))
    }).count();
    let b_post = b_facts.iter().filter(|f| {
        let r = store_a.query(f, 3);
        r.iter().any(|res| res.text.as_deref() == Some(f.as_str()))
    }).count();

    println!("Post-merge: A={}/{} B={}/{}", a_post, items_per_node, b_post, items_per_node);

    let total = a_post + b_post;
    let ideal = items_per_node * 2;
    let degradation = ((ideal - total) as f64 / ideal as f64) * 100.0;

    println!("\nTotal retrievable: {}/{} ({:.1}%)", total, ideal, (total as f64 / ideal as f64) * 100.0);
    println!("Degradation: {:.1}% (was 78.0%)", degradation);

    if degradation < 50.0 {
        println!("✅ Improvement confirmed!");
    } else {
        println!("⚠️  Still high degradation");
    }

    // Also test fill levels
    println!("\n=== Fill Level Test ===\n");
    for &fill in &[50, 100, 200, 500] {
        let mut s = NCAKnowledge::new();
        s.config.ollama_url = None;
        let mut encoded_facts = Vec::new();
        for i in 0..fill {
            let f = format!("fill test item number {}", i);
            s.encode(&f, 0.8);
            encoded_facts.push(f);
        }
        let sample_size = fill.min(50);
        let retrievable = encoded_facts.iter().take(sample_size).filter(|f| {
            let r = s.query(f, 3);
            r.iter().any(|res| res.text.as_deref() == Some(f.as_str()))
        }).count();
        println!("  Fill {}: {}/{} retrievable ({:.1}%)",
            fill, retrievable, sample_size,
            (retrievable as f64 / sample_size as f64) * 100.0);
    }
}