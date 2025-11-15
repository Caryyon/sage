// Test hot-reload functionality
// Run this, then recompile sage-training to see hot-reload in action

use sage::tui::engine_loader::EngineLoader;
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔥 SAGE Hot-Reload Test");
    println!("{}", "=".repeat(50));

    // Create engine loader
    let mut loader = EngineLoader::new()?;

    // Load initial engine
    println!("\n📚 Loading training engine...");
    let mut engine = loader.load()?;

    println!("✅ Engine loaded! Version: {}", loader.version());

    // Run a few iterations
    println!("\n🏃 Running 5 iterations...");
    for i in 1..=5 {
        let update = engine.step(10);
        println!(
            "  [{}] Gen: {}, Loss: {:.4}, Pattern: {}",
            i, update.generation, update.loss, update.current_pattern
        );
        thread::sleep(Duration::from_millis(500));
    }

    // Now watch for changes
    println!("\n👁️  Watching for library changes...");
    println!("   Try rebuilding: cargo build --release -p sage-training");
    println!("   Press Ctrl+C to exit\n");

    let mut iteration = 6;
    loop {
        // Check for hot-reload
        if loader.check_for_reload() {
            println!("\n🔄 LIBRARY CHANGED! Hot-reloading...");

            // Save checkpoint
            println!("   💾 Saving checkpoint...");
            let checkpoint = engine.save_checkpoint()?;
            println!("   ✓ Checkpoint saved: Gen {}", checkpoint.generation);

            // Reload engine
            println!("   📚 Loading new library...");
            engine = loader.load()?;
            println!("   ✓ New engine loaded! Version: {}", loader.version());

            // Restore checkpoint
            println!("   📥 Restoring checkpoint...");
            engine.load_checkpoint(checkpoint)?;
            println!("   ✅ HOT RELOAD COMPLETE!\n");
        }

        // Continue training
        let update = engine.step(10);
        println!(
            "  [{}] Gen: {}, Loss: {:.4}, Pattern: {}",
            iteration, update.generation, update.loss, update.current_pattern
        );

        iteration += 1;
        thread::sleep(Duration::from_secs(2));
    }
}
