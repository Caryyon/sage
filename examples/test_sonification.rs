use sage::grid::Grid;
use sage::nca::NCA;
use sage::sonification::SonificationEngine;
use std::thread;
use std::time::Duration;

fn main() {
    println!("🎵 SAGE Sonification Demo");
    println!("========================\n");

    // Initialize sonification engine
    let engine = match SonificationEngine::new() {
        Ok(engine) => engine,
        Err(e) => {
            eprintln!("❌ Failed to initialize audio: {}", e);
            return;
        }
    };

    println!("✅ Audio system initialized");
    println!("\nGenerating neural patterns and converting to sound...\n");

    // Create a simple circle pattern to sonify
    let grid = create_circle_pattern();

    println!("🔵 Pattern: Circle");
    println!("   Sonifying with spatial audio...");
    println!("   - Higher cells = higher pitch");
    println!("   - Left/right position = stereo panning");
    println!("   - Alpha value = volume\n");

    // Sonify the circle pattern (500ms duration)
    engine.sonify_grid(&grid, 500);
    thread::sleep(Duration::from_millis(600));

    // Create an evolving pattern sequence
    println!("🌀 Generating evolving spiral pattern...");
    let spiral_sequence = create_spiral_sequence(20);

    println!("   Playing evolution over {} frames", spiral_sequence.len());
    println!("   (You should hear the pattern form and grow)\n");

    engine.sonify_evolution(&spiral_sequence, 150);

    println!("\n✨ Demo complete!");
    println!("\nHow sonification works:");
    println!("  • Each living cell (alpha > 0.1) generates a tone");
    println!("  • Y position maps to frequency (200-2000 Hz)");
    println!("  • X position maps to stereo panning");
    println!("  • Alpha channel maps to volume");
    println!("  • Multiple cells create harmonic patterns");

    // Wait for audio to finish
    while engine.is_playing() {
        thread::sleep(Duration::from_millis(100));
    }
}

/// Create a simple circle pattern in the 32x32 grid
fn create_circle_pattern() -> [[f64; 22]; 1024] {
    let mut grid = [[0.0; 22]; 1024];
    let center_x = 16.0;
    let center_y = 16.0;
    let radius = 8.0;

    for y in 0..32 {
        for x in 0..32 {
            let dx = x as f64 - center_x;
            let dy = y as f64 - center_y;
            let dist = (dx * dx + dy * dy).sqrt();

            let i = y * 32 + x;

            // Create a smooth circle with falloff
            if dist < radius {
                let alpha = (1.0 - (dist / radius)).max(0.0);
                grid[i][3] = alpha; // Alpha channel
            }
        }
    }

    grid
}

/// Create an evolving spiral pattern sequence
fn create_spiral_sequence(frames: usize) -> Vec<[[f64; 22]; 1024]> {
    let mut sequence = Vec::new();

    for frame in 0..frames {
        let mut grid = [[0.0; 22]; 1024];
        let t = frame as f64 / frames as f64;
        let center_x = 16.0;
        let center_y = 16.0;

        for y in 0..32 {
            for x in 0..32 {
                let dx = x as f64 - center_x;
                let dy = y as f64 - center_y;
                let dist = (dx * dx + dy * dy).sqrt();
                let angle = dy.atan2(dx);

                let i = y * 32 + x;

                // Spiral equation: theta = a * r
                let spiral_angle = angle + dist * 0.3;
                let spiral_phase = (spiral_angle - t * 6.28).cos();

                // Gradually expanding spiral
                let max_radius = 4.0 + t * 8.0;

                if dist < max_radius && dist > 2.0 {
                    let alpha = ((spiral_phase + 1.0) / 2.0).max(0.0);
                    grid[i][3] = alpha * (1.0 - (dist / max_radius)).max(0.0);
                }
            }
        }

        sequence.push(grid);
    }

    sequence
}
