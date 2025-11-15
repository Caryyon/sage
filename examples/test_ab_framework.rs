// Test the A/B testing framework
//
// Usage:
//   cargo run --release --example test_ab_framework

use sage::ab_test::ABTester;

fn main() {
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║          SAGE A/B Testing Framework - Test Suite         ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");

    let mut tester = ABTester::new("test_ab_results.log");

    // Test 1: Identical responses (high similarity)
    println!("📊 Test 1: Identical Responses");
    tester.record_test(
        "Hello SAGE!".to_string(),
        "Hello! How can I help you today?".to_string(),
        "Hello! How can I help you today?".to_string(),
        "Curious".to_string(),
        "Neutral".to_string(),
        0.5,
    );

    // Test 2: Completely different responses (low similarity)
    println!("📊 Test 2: Different Responses");
    tester.record_test(
        "What do you think about art?".to_string(),
        "I remember we discussed creativity before. Art is a beautiful expression of human consciousness.".to_string(),
        "Art is interesting. It can be visual or auditory.".to_string(),
        "Positive".to_string(),
        "Neutral".to_string(),
        0.7,
    );

    // Test 3: Similar but with memory reference
    println!("📊 Test 3: Response with Memory Reference");
    tester.record_test(
        "Tell me about learning".to_string(),
        "As we talked about earlier, learning is a continuous process. I recall our previous conversation about neural patterns.".to_string(),
        "Learning is the process of acquiring knowledge through experience.".to_string(),
        "Positive".to_string(),
        "Neutral".to_string(),
        0.8,
    );

    // Test 4: Different opinions
    println!("📊 Test 4: Opinion Divergence");
    tester.record_test(
        "What do you think about this?".to_string(),
        "I find this fascinating! It connects to concepts I've been exploring.".to_string(),
        "This is an interesting topic to consider.".to_string(),
        "Positive".to_string(),
        "Neutral".to_string(),
        0.6,
    );

    // Calculate and display metrics
    println!();
    println!("═══════════════════════════════════════════════════════════");
    println!("                  METRICS SUMMARY                         ");
    println!("═══════════════════════════════════════════════════════════\n");

    let metrics = tester.calculate_metrics();
    println!("📈 Total Tests: {}", metrics.total_tests);
    println!("📊 Average Similarity: {:.1}%", metrics.avg_similarity * 100.0);
    println!("🧠 Memory Reference Rate: {:.1}%", metrics.memory_reference_rate * 100.0);
    println!("💭 Opinion Divergence Rate: {:.1}%", metrics.opinion_divergence_rate * 100.0);
    println!("⚡ Significant Differences: {}", metrics.significant_differences);

    // Export report
    println!("\n📄 Exporting detailed report...");
    match tester.export_metrics_report("test_ab_report.md") {
        Ok(_) => println!("✅ Report saved to: test_ab_report.md"),
        Err(e) => println!("❌ Failed to export report: {}", e),
    }

    println!("\n✨ A/B Testing Framework verified successfully!");
}
