// SAGE Vision Mode - Real-time camera perception with NCA integration
//
// This is a production binary for testing and using SAGE's vision system.
// Features:
// - Continuous camera capture
// - Real-time feature extraction
// - Visual memory storage
// - NCA grid conversion
// - Sonification of visual patterns (optional)

use sage::vision::SageVision;
use sage::visual_memory::VisualMemory;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use std::io::{self, Write};

fn main() -> Result<(), String> {
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║              SAGE Vision - Real-time Perception            ║");
    println!("║         Camera → Features → NCA Grid → Memory             ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");

    // Initialize vision system
    print!("🔌 Initializing camera (device 0)... ");
    io::stdout().flush().unwrap();
    let vision = SageVision::new(0, 32)
        .map_err(|e| format!("Failed to initialize vision: {}", e))?;
    println!("✓");

    // Open camera stream
    print!("📷 Opening camera stream... ");
    io::stdout().flush().unwrap();
    vision.open()?;
    println!("✓");

    // Initialize visual memory
    let visual_memory = Arc::new(Mutex::new(VisualMemory::new()));

    println!("\n✅ Camera ready!");
    println!("⏱️  Waiting 2 seconds for camera warm-up...\n");
    thread::sleep(Duration::from_secs(2));

    println!("👁️  SAGE is now perceiving the world in real-time");
    println!("📊 Press Ctrl+C to stop\n");
    println!("{}", "=".repeat(60));

    let mut frame_count = 0;
    let start_time = Instant::now();
    let mut last_report = Instant::now();

    // Main perception loop
    loop {
        frame_count += 1;

        // Capture frame
        let frame = match vision.capture_frame() {
            Ok(f) => f,
            Err(e) => {
                eprintln!("⚠️  Frame capture failed: {}", e);
                thread::sleep(Duration::from_millis(100));
                continue;
            }
        };

        // Extract visual features
        let features = vision.extract_features(&frame);

        // Generate visual concepts
        let mut concepts = Vec::new();

        // Brightness concepts
        if features.avg_brightness > 0.7 {
            concepts.push("bright".to_string());
        } else if features.avg_brightness < 0.3 {
            concepts.push("dark".to_string());
        }

        // Color concept
        concepts.push(features.dominant_color.clone());

        // Edge concepts
        if features.edge_strength > 30.0 {
            concepts.push("sharp_edges".to_string());
        } else {
            concepts.push("smooth".to_string());
        }

        // Variance concept
        if features.color_variance > 0.1 {
            concepts.push("varied".to_string());
        } else {
            concepts.push("uniform".to_string());
        }

        // Store in visual memory
        visual_memory.lock().unwrap().record_experience(&features, concepts.clone());

        // Convert to NCA grid (for future neural processing)
        let _nca_grid = vision.image_to_nca_grid(&frame);

        // Report every second
        if last_report.elapsed() >= Duration::from_secs(1) {
            let elapsed = start_time.elapsed().as_secs_f64();
            let fps = frame_count as f64 / elapsed;

            println!("\n📸 Frame {} | FPS: {:.1}", frame_count, fps);
            println!("   🧠 Perception: {}", features.describe());
            println!("   🏷️  Concepts: {}", concepts.join(", "));
            println!("   📊 Details:");
            println!("      • Brightness: {:.2}", features.avg_brightness);
            println!("      • Color variance: {:.3}", features.color_variance);
            println!("      • Edge strength: {:.1}", features.edge_strength);
            println!("      • RGB: ({:.2}, {:.2}, {:.2})",
                     features.avg_r, features.avg_g, features.avg_b);

            // Memory stats
            let mem = visual_memory.lock().unwrap();
            let memory_size = mem.experience_count();
            let recent_concepts = mem.get_recent_concepts(5);
            println!("   💾 Memory: {} experiences", memory_size);
            if !recent_concepts.is_empty() {
                println!("   🔝 Recent concepts: {}", recent_concepts.join(", "));
            }

            last_report = Instant::now();
        }

        // Capture at ~10 FPS to avoid overwhelming the system
        thread::sleep(Duration::from_millis(100));
    }

    // Unreachable, but good practice
    #[allow(unreachable_code)]
    {
        println!("\n🛑 Closing camera...");
        vision.close()?;

        // Save visual memory
        let mem = visual_memory.lock().unwrap();
        println!("💾 Total experiences captured: {}", mem.experience_count());

        println!("\n✅ Vision mode complete!");
        Ok(())
    }
}
