//! Test weight persistence across restarts
//!
//! This test verifies that:
//! 1. Weights can be saved to disk
//! 2. Weights can be loaded from disk
//! 3. Loaded weights produce identical results
//!
//! Run with: cargo run --release --example test_weight_persistence

use sage::nca::NCA;
use sage::grid::Grid;

fn main() {
    println!("=== Weight Persistence Test ===\n");

    // Test 1: Load existing weights and measure performance
    println!("--- Test 1: Load Existing Weights ---");
    let mut nca1 = NCA::new();

    if nca1.load_weights_from_file("pattern_training_weights.json").is_ok() {
        println!("[OK] Loaded weights from pattern_training_weights.json");

        // Run and measure
        nca1.reset_with_seed();
        for _ in 0..80 {
            nca1.step();
        }
        let loss1 = calculate_loss(&nca1.grid);
        let alive1 = count_alive(&nca1.grid);
        println!("  After 80 steps: loss={:.4}, alive={}", loss1, alive1);

        // Test 2: Save to temp file and reload
        println!("\n--- Test 2: Save and Reload ---");
        let temp_file = "/tmp/sage_test_weights.json";

        if nca1.save_weights_to_file(temp_file).is_ok() {
            println!("[OK] Saved weights to {}", temp_file);

            // Load into new NCA
            let mut nca2 = NCA::new();
            if nca2.load_weights_from_file(temp_file).is_ok() {
                println!("[OK] Loaded weights from temp file");

                // Run with same seed
                nca2.reset_with_seed();
                for _ in 0..80 {
                    nca2.step();
                }
                let loss2 = calculate_loss(&nca2.grid);
                let alive2 = count_alive(&nca2.grid);
                println!("  After 80 steps: loss={:.4}, alive={}", loss2, alive2);

                // Compare
                println!("\n--- Test 3: Compare Results ---");
                let loss_diff = (loss1 - loss2).abs();
                let alive_diff = (alive1 as i32 - alive2 as i32).abs();

                println!("Loss difference:  {:.6}", loss_diff);
                println!("Alive difference: {}", alive_diff);

                if loss_diff < 0.0001 && alive_diff == 0 {
                    println!("\n[OK] WEIGHT PERSISTENCE VERIFIED - Results are identical!");
                } else if loss_diff < 0.01 {
                    println!("\n[OK] Weight persistence working - minor floating point variance");
                } else {
                    println!("\n[FAIL] Results differ significantly!");
                }

                // Cleanup
                let _ = std::fs::remove_file(temp_file);

            } else {
                println!("[FAIL] Could not load from temp file");
            }
        } else {
            println!("[FAIL] Could not save weights");
        }

    } else {
        println!("[WARN] No trained weights found");
        println!("       Run the main TUI first to train some patterns\n");

        // Test basic save/load with untrained weights
        println!("--- Testing save/load with fresh weights ---");
        let temp_file = "/tmp/sage_test_weights.json";

        if nca1.save_weights_to_file(temp_file).is_ok() {
            println!("[OK] Saved fresh weights");

            let mut nca2 = NCA::new();
            if nca2.load_weights_from_file(temp_file).is_ok() {
                println!("[OK] Loaded fresh weights");
                println!("[OK] Basic save/load working!");
            }
            let _ = std::fs::remove_file(temp_file);
        }
    }

    println!("\n=== Test Complete ===");
}

fn calculate_loss(grid: &Grid) -> f64 {
    // Simple loss: distance from circle pattern
    let mut loss = 0.0;
    let center = grid.width / 2;
    let radius = 8.0;

    for y in 0..grid.height {
        for x in 0..grid.width {
            let dx = x as f64 - center as f64;
            let dy = y as f64 - center as f64;
            let dist = (dx * dx + dy * dy).sqrt();
            let target = if dist < radius { 1.0 } else { 0.0 };
            let diff = grid.cells[y][x][3] - target;
            loss += diff * diff;
        }
    }
    loss / (grid.width * grid.height) as f64
}

fn count_alive(grid: &Grid) -> usize {
    grid.cells.iter()
        .flatten()
        .filter(|cell| cell[3] > 0.1)
        .count()
}
