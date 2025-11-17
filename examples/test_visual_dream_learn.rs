// Test: Complete Vision → Dream → Learn Loop
//
// This test demonstrates SAGE's cross-modal learning system:
// 1. Vision: Camera captures visual features
// 2. Memory: Visual experiences recorded with emotional tags
// 3. Dream: Memories replayed and remixed during idle time
// 4. Learn: Visual patterns converted to NCA training targets
//
// Run: cargo run --release --example test_visual_dream_learn

use sage::vision::SageVision;
use sage::visual_memory::{VisualMemory, concepts_to_pattern, EmotionalTag};
use std::thread;
use std::time::Duration;

fn main() {
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║  SAGE Vision → Dream → Learn Loop Test                   ║");
    println!("║  Demonstrating Cross-Modal Learning                       ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");

    // Initialize visual memory system
    let mut visual_memory = VisualMemory::new();
    println!("✅ Visual memory system initialized\n");

    // Initialize vision system
    let vision = match SageVision::new(0, 32) {
        Ok(mut v) => {
            if let Err(e) = v.open() {
                println!("⚠️  Camera not available: {} (will use simulated data)\n", e);
                None
            } else {
                Some(v)
            }
        }
        Err(e) => {
            println!("⚠️  Vision initialization failed: {} (will use simulated data)\n", e);
            None
        }
    };

    // PHASE 1: VISION - Capture and record visual experiences
    println!("━━━ PHASE 1: VISION ━━━");
    println!("📹 Capturing visual experiences from camera...\n");

    for i in 0..5 {
        println!("  Frame {}/5:", i + 1);

        // Capture camera frame
        if let Some(ref vision) = vision {
            match vision.capture_frame() {
                Ok(frame) => {
                    // Extract visual features
                    let features = vision.extract_features(&frame);

                // Generate visual concepts based on features
                let mut concepts = Vec::new();

                // Brightness concepts
                if features.avg_brightness > 0.7 {
                    concepts.push("bright".to_string());
                } else if features.avg_brightness < 0.3 {
                    concepts.push("dark".to_string());
                }

                // Color concepts
                concepts.push(features.dominant_color.clone());

                // Edge concepts
                if features.edge_strength > 30.0 {
                    concepts.push("sharp_edges".to_string());
                } else {
                    concepts.push("smooth".to_string());
                }

                // Variance concepts
                if features.color_variance > 0.15 {
                    concepts.push("complex".to_string());
                } else {
                    concepts.push("simple".to_string());
                }

                println!("    Concepts: {:?}", concepts);
                println!("    Features: brightness={:.2}, edges={:.1}, variance={:.3}",
                    features.avg_brightness, features.edge_strength, features.color_variance);

                // Record to visual memory
                visual_memory.record_experience(&features, concepts.clone());

                let experience = visual_memory.experience_count();
                    println!("    ✓ Recorded to memory (total experiences: {})\n", experience);
                }
                Err(e) => {
                    eprintln!("    ⚠️  Camera error: {}", e);
                }
            }
        } else {
            // Simulate visual experience with varied data
            use sage::vision::VisualFeatures;

            let simulated_features = VisualFeatures {
                avg_brightness: 0.5 + (i as f64 * 0.1),
                avg_r: 0.6 + (i as f64 * 0.05),
                avg_g: 0.4,
                avg_b: 0.3,
                color_variance: 0.15 + (i as f64 * 0.03),
                dominant_color: if i % 2 == 0 { "red" } else { "blue" }.to_string(),
                edge_strength: 25.0 + (i as f64 * 5.0),
            };

            let simulated_concepts = vec![
                format!("simulated_{}", i),
                if i % 2 == 0 { "warm".to_string() } else { "cool".to_string() },
                if i < 3 { "simple".to_string() } else { "complex".to_string() },
            ];

            println!("    Concepts: {:?} (simulated)", simulated_concepts);
            println!("    Features: brightness={:.2}, edges={:.1}, variance={:.3} (simulated)",
                simulated_features.avg_brightness, simulated_features.edge_strength,
                simulated_features.color_variance);

            // Record simulated experience
            visual_memory.record_experience(&simulated_features, simulated_concepts.clone());

            let experience = visual_memory.experience_count();
            println!("    ✓ Recorded to memory (total experiences: {})\n", experience);
        }

        thread::sleep(Duration::from_millis(500));
    }

    println!("✓ Captured {} visual experiences\n", visual_memory.experience_count());

    // PHASE 2: DREAM - Replay and remix visual memories
    println!("━━━ PHASE 2: DREAM MODE ━━━");
    println!("🌙 Entering dream state... replaying visual memories\n");

    let mut dream_patterns = Vec::new();

    // Replay first memory
    if let Some(concepts1) = visual_memory.get_dream_material() {
        println!("  🌙 Dream sequence 1:");
        println!("    Replaying: {:?}", concepts1);

        // Try to remix with another memory
        if let Some(concepts2) = visual_memory.get_dream_material() {
            println!("    Replaying: {:?}", concepts2);

            let mixed = visual_memory.remix_concepts(&concepts1, &concepts2);
            println!("    🔄 Remixing → {:?}\n", mixed);

            dream_patterns.push((concepts1, concepts2, mixed));
        }
    }

    // Replay more dreams
    if let Some(concepts3) = visual_memory.get_dream_material() {
        println!("  🌙 Dream sequence 2:");
        println!("    Replaying: {:?}\n", concepts3);
    }

    println!("✓ Dream replay complete\n");

    // PHASE 3: LEARN - Convert visual memories to NCA training targets
    println!("━━━ PHASE 3: LEARN ━━━");
    println!("🧠 Converting visual memories to NCA training patterns...\n");

    let grid_size = 32;

    // Convert concepts to NCA patterns
    for (i, (concepts1, concepts2, mixed)) in dream_patterns.iter().enumerate() {
        println!("  Learning sequence {}:", i + 1);

        // Convert first concept set to pattern
        let pattern1 = concepts_to_pattern(&concepts1, grid_size);
        let avg_alpha1 = calculate_avg_alpha(&pattern1);
        println!("    Pattern 1 from {:?}", concepts1);
        println!("      → NCA grid ({}×{}) avg_alpha={:.3}", grid_size, grid_size, avg_alpha1);

        // Convert second concept set to pattern
        let pattern2 = concepts_to_pattern(&concepts2, grid_size);
        let avg_alpha2 = calculate_avg_alpha(&pattern2);
        println!("    Pattern 2 from {:?}", concepts2);
        println!("      → NCA grid ({}×{}) avg_alpha={:.3}", grid_size, grid_size, avg_alpha2);

        // Convert mixed concepts to pattern
        let pattern_mixed = concepts_to_pattern(&mixed, grid_size);
        let avg_alpha_mixed = calculate_avg_alpha(&pattern_mixed);
        println!("    Mixed pattern from {:?}", mixed);
        println!("      → NCA grid ({}×{}) avg_alpha={:.3}", grid_size, grid_size, avg_alpha_mixed);
        println!("      ✨ This pattern can now be used as NCA training target!\n");
    }

    // PHASE 4: STATISTICS - Analyze visual memory
    println!("━━━ PHASE 4: MEMORY ANALYSIS ━━━");
    println!("📊 Visual memory statistics:\n");

    println!("  Total experiences: {}", visual_memory.experience_count());

    // Count by emotional tag
    let curious_count = visual_memory.get_experiences_by_emotion(EmotionalTag::Curious).len();
    let beautiful_count = visual_memory.get_experiences_by_emotion(EmotionalTag::Beautiful).len();
    let exciting_count = visual_memory.get_experiences_by_emotion(EmotionalTag::Exciting).len();
    let peaceful_count = visual_memory.get_experiences_by_emotion(EmotionalTag::Peaceful).len();
    let confusing_count = visual_memory.get_experiences_by_emotion(EmotionalTag::Confusing).len();
    let familiar_count = visual_memory.get_experiences_by_emotion(EmotionalTag::Familiar).len();

    println!("  Emotional distribution:");
    println!("    Curious:    {} experiences", curious_count);
    println!("    Beautiful:  {} experiences", beautiful_count);
    println!("    Exciting:   {} experiences", exciting_count);
    println!("    Peaceful:   {} experiences", peaceful_count);
    println!("    Confusing:  {} experiences", confusing_count);
    println!("    Familiar:   {} experiences", familiar_count);

    // Recent concepts
    let recent = visual_memory.get_recent_concepts(10);
    println!("\n  Recent visual concepts: {:?}", recent);

    println!("\n╔════════════════════════════════════════════════════════════╗");
    println!("║  ✅ Vision → Dream → Learn Loop COMPLETE!                 ║");
    println!("║                                                            ║");
    println!("║  SAGE can now:                                             ║");
    println!("║  • See the world through camera (Vision)                   ║");
    println!("║  • Remember visual experiences with emotions (Memory)      ║");
    println!("║  • Replay and remix memories during dreams (Dream)         ║");
    println!("║  • Convert visual concepts to NCA patterns (Learn)         ║");
    println!("║                                                            ║");
    println!("║  This is true cross-modal learning! 🎉                     ║");
    println!("╚════════════════════════════════════════════════════════════╝");
}

// Helper function to calculate average alpha channel value
fn calculate_avg_alpha(grid: &sage::grid::Grid) -> f64 {
    let mut sum = 0.0;
    let mut count = 0;

    for row in &grid.cells {
        for cell in row {
            sum += cell[3]; // Alpha channel
            count += 1;
        }
    }

    if count > 0 {
        sum / count as f64
    } else {
        0.0
    }
}
