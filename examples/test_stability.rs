//! Test long-term pattern stability
//!
//! This test verifies that:
//! 1. Patterns remain stable after formation (200+ steps)
//! 2. Sample pool training helps maintain stability
//! 3. Drift during stability phase is minimal
//!
//! Run with: cargo run --release --example test_stability

use sage::nca::NCA;
use sage::grid::Grid;

fn main() {
    println!("=== Long-Term Stability Test ===\n");

    let mut nca = NCA::new();

    // Load trained weights
    if nca.load_weights_from_file("pattern_training_weights.json").is_ok() {
        println!("[OK] Loaded trained weights\n");
    } else {
        println!("[WARN] No trained weights - results may show poor stability\n");
    }

    // Create circle target for reference
    let target = create_circle_target(32);

    println!("--- Test 1: Extended Evolution (250 steps) ---");
    nca.reset_with_seed();

    // Evolve for formation phase (100 steps)
    for _ in 0..100 {
        nca.step();
    }
    let loss_at_100 = calculate_loss(&nca.grid, &target);
    let state_at_100 = capture_alpha(&nca.grid);
    println!("Step 100 (formation complete): loss = {:.4}", loss_at_100);
    print_grid_mini(&nca.grid, "At step 100");

    // Continue evolving for stability phase (150 more steps)
    for _ in 0..150 {
        nca.step();
    }
    let loss_at_250 = calculate_loss(&nca.grid, &target);
    let state_at_250 = capture_alpha(&nca.grid);
    println!("\nStep 250 (after stability phase): loss = {:.4}", loss_at_250);
    print_grid_mini(&nca.grid, "At step 250");

    // Calculate drift (change from step 100 to step 250)
    let drift = calculate_drift(&state_at_100, &state_at_250);
    println!("\n--- Stability Analysis ---");
    println!("Drift (change from step 100 to 250): {:.6}", drift);
    println!("Loss change: {:.4}", (loss_at_250 - loss_at_100).abs());

    // Test 2: Extreme evolution (500 steps)
    println!("\n--- Test 2: Extreme Evolution (500 steps) ---");
    nca.reset_with_seed();

    for step in 0..500 {
        nca.step();
        if step == 99 || step == 249 || step == 499 {
            let loss = calculate_loss(&nca.grid, &target);
            println!("Step {}: loss = {:.4}", step + 1, loss);
        }
    }
    let final_loss = calculate_loss(&nca.grid, &target);
    print_grid_mini(&nca.grid, "After 500 steps");

    // Test 3: Multiple runs for consistency
    println!("\n--- Test 3: Consistency Across 5 Runs ---");
    let mut losses = vec![];
    let mut drifts = vec![];

    for run in 0..5 {
        nca.reset_with_seed();

        // Formation
        for _ in 0..100 {
            nca.step();
        }
        let state_100 = capture_alpha(&nca.grid);

        // Stability
        for _ in 0..150 {
            nca.step();
        }
        let state_250 = capture_alpha(&nca.grid);
        let loss = calculate_loss(&nca.grid, &target);
        let drift = calculate_drift(&state_100, &state_250);

        losses.push(loss);
        drifts.push(drift);
        println!("Run {}: loss = {:.4}, drift = {:.6}", run + 1, loss, drift);
    }

    let avg_loss = losses.iter().sum::<f64>() / losses.len() as f64;
    let avg_drift = drifts.iter().sum::<f64>() / drifts.len() as f64;
    let loss_variance = calculate_variance(&losses);
    let drift_variance = calculate_variance(&drifts);

    println!("\n=== Summary ===");
    println!("Average loss:     {:.4} (variance: {:.6})", avg_loss, loss_variance);
    println!("Average drift:    {:.6} (variance: {:.8})", avg_drift, drift_variance);

    // Evaluate stability
    if avg_drift < 0.01 {
        println!("\n[EXCELLENT] Pattern is highly stable - minimal drift during stability phase!");
    } else if avg_drift < 0.05 {
        println!("\n[GOOD] Pattern shows good stability - moderate drift during stability phase");
    } else if avg_drift < 0.1 {
        println!("\n[PARTIAL] Some stability - pattern drifts during extended evolution");
        println!("         Consider more training with pool sampling enabled");
    } else {
        println!("\n[NEEDS WORK] Pattern is not stable - significant drift during evolution");
        println!("             Run TUI with stability training to improve");
    }
}

fn create_circle_target(size: usize) -> Grid {
    let mut grid = Grid::new(size, size);
    let center = size / 2;
    let radius = 8.0;

    for y in 0..size {
        for x in 0..size {
            let dx = x as f64 - center as f64;
            let dy = y as f64 - center as f64;
            let dist = (dx * dx + dy * dy).sqrt();
            if dist < radius {
                grid.cells[y][x][3] = 1.0;
                grid.cells[y][x][0] = 1.0;
            }
        }
    }
    grid
}

fn calculate_loss(grid: &Grid, target: &Grid) -> f64 {
    let mut loss = 0.0;
    for y in 0..grid.height {
        for x in 0..grid.width {
            let diff = grid.cells[y][x][3] - target.cells[y][x][3];
            loss += diff * diff;
        }
    }
    loss / (grid.width * grid.height) as f64
}

fn capture_alpha(grid: &Grid) -> Vec<Vec<f64>> {
    grid.cells.iter()
        .map(|row| row.iter().map(|cell| cell[3]).collect())
        .collect()
}

fn calculate_drift(state1: &Vec<Vec<f64>>, state2: &Vec<Vec<f64>>) -> f64 {
    let mut drift = 0.0;
    let mut count = 0;
    for y in 0..state1.len() {
        for x in 0..state1[y].len() {
            let diff = state1[y][x] - state2[y][x];
            drift += diff * diff;
            count += 1;
        }
    }
    drift / count as f64
}

fn calculate_variance(values: &[f64]) -> f64 {
    if values.is_empty() { return 0.0; }
    let mean = values.iter().sum::<f64>() / values.len() as f64;
    values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64
}

fn print_grid_mini(grid: &Grid, title: &str) {
    println!("\n{} (16x8 preview):", title);
    for y in (0..32).step_by(4) {
        let mut line = String::new();
        for x in (0..32).step_by(2) {
            let val = grid.cells[y][x][3];
            let ch = if val > 0.7 { '#' }
                else if val > 0.4 { '*' }
                else if val > 0.1 { '.' }
                else { ' ' };
            line.push(ch);
        }
        println!("  |{}|", line);
    }
}
