// Test SAGE's vision system - First glimpse of the world!
//
// Usage:
//   cargo run --release --example test_sage_vision

use sage::vision::SageVision;
use std::thread;
use std::time::Duration;

fn main() -> Result<(), String> {
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║           SAGE Vision Test - First Sight!                 ║");
    println!("║    Testing camera capture and visual perception           ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");

    // Initialize vision system
    println!("🔌 Initializing camera...");
    let vision = SageVision::new(0, 32)
        .map_err(|e| format!("Failed to initialize vision: {}", e))?;

    // Open camera stream
    println!("📷 Opening camera stream...");
    vision.open()?;

    println!("✅ Camera ready!\n");
    println!("⏱️  Waiting 2 seconds for camera to warm up...\n");
    thread::sleep(Duration::from_secs(2));

    // Capture and analyze 5 frames
    println!("👁️  SAGE is looking at the world...\n");
    println!("{}\n", "=".repeat(60));

    for i in 1..=5 {
        println!("📸 Frame {}/5:", i);

        // Capture frame
        let frame = vision.capture_frame()?;
        println!("   Resolution: {}×{}", frame.width(), frame.height());

        // Extract visual features
        let features = vision.extract_features(&frame);

        // Have SAGE describe what it sees
        println!("   🧠 SAGE's perception: {}", features.describe());
        println!("   📊 Technical details:");
        println!("      - Brightness: {:.2}", features.avg_brightness);
        println!("      - Color variance: {:.2}", features.color_variance);
        println!("      - Edge strength: {:.2}", features.edge_strength);
        println!("      - RGB: ({:.2}, {:.2}, {:.2})", features.avg_r, features.avg_g, features.avg_b);

        // Convert to NCA grid format
        let nca_grid = vision.image_to_nca_grid(&frame);
        println!("   🔢 NCA grid: {}×{}×4 channels", nca_grid.len(), nca_grid[0].len());

        println!();
        thread::sleep(Duration::from_millis(500));
    }

    println!("{}\n", "=".repeat(60));

    // Close camera
    println!("🛑 Closing camera...");
    vision.close()?;

    println!("\n✅ Vision test complete!");
    println!("\n💡 Next steps:");
    println!("   - Integrate with SAGE's NCA grid");
    println!("   - Create visual concept associations");
    println!("   - Add visual introspection");
    println!("   - Store visual memories in SpacetimeDB");

    Ok(())
}
