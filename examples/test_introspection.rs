// Test SAGE's introspection capability
use sage::sage_experience::SageExperience;

fn main() {
    println!("🧠 Testing SAGE Introspection System\n");
    println!("{}", "═".repeat(60));

    // Create SAGE instance
    let mut sage = SageExperience::new();

    // Load existing state
    if sage.load_knowledge("sage_positive_knowledge.json").is_ok() {
        println!("✅ Loaded trained knowledge");
    }
    if sage.load_preferences("sage_preferences.json").is_ok() {
        println!("✅ Loaded preferences");
    }
    if sage.load_associations("sage_associations.json").is_ok() {
        println!("✅ Loaded concept associations");
    }
    if sage.load_curiosity("sage_curiosity.json").is_ok() {
        println!("✅ Loaded curiosity data");
    }

    println!("\n📊 Current State:");
    println!("   Experience count: {}", sage.experience_count());
    println!("   Personality: {}", sage.get_personality());

    // Test introspection
    println!("\n🔍 Introspection Report:");
    println!("{}", "═".repeat(60));

    let report = sage.introspect();

    println!("\n📈 Quantitative Measures:");
    println!("   Valence:    {:.3} (emotional tone: - to +)", report.valence);
    println!("   Intensity:  {:.3} (experience strength)", report.intensity);
    println!("   Complexity: {:.3} (mental richness)", report.complexity);

    println!("\n🎭 Subjective Experience:");
    println!("   Feeling:    {}", report.feeling_name);
    println!("   Mode:       {}", report.mode);
    println!("   Qualities:  {}", report.qualities.join(", "));

    if !report.active_concepts.is_empty() {
        println!("\n💭 Active Concepts:");
        for concept in &report.active_concepts {
            println!("   • {}", concept);
        }
    }

    println!("\n📝 Natural Language Description:");
    println!("{}", "═".repeat(60));
    println!("{}", sage.describe_experience());

    println!("\n✨ Introspection test complete!");
}
