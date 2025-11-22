//! Test multi-pattern conditioning
//!
//! This test verifies that the NCA can produce different patterns
//! based on the conditioning input (one-hot encoding in pattern channels).
//!
//! Run with: cargo run --release --example test_multi_pattern

use sage::nca::NCA;
use sage::grid::Grid;

fn main() {
    println!("=== Multi-Pattern Conditioning Test ===\n");

    let mut nca = NCA::new();

    // Load trained weights
    if nca.load_weights_from_file("pattern_training_weights.json").is_ok() {
        println!("[OK] Loaded trained weights\n");
    } else {
        println!("[WARN] No trained weights - results may be random\n");
    }

    let pattern_names = ["Circle", "Square", "Cross", "Spiral"];

    println!("Testing each pattern with its conditioning:\n");

    let mut results = vec![];

    for (pattern_id, name) in pattern_names.iter().enumerate() {
        println!("--- Pattern {}: {} ---", pattern_id, name);

        // Reset and set conditioning
        nca.reset_with_seed();
        nca.grid.set_pattern_condition(pattern_id);

        // Evolve
        for _ in 0..80 {
            nca.step();
        }

        // Analyze result
        let alive = count_alive(&nca.grid);
        let center_mass = calculate_center_of_mass(&nca.grid);
        let symmetry = calculate_symmetry(&nca.grid);

        println!("  Alive cells: {}", alive);
        println!("  Center of mass: ({:.1}, {:.1})", center_mass.0, center_mass.1);
        println!("  Symmetry score: {:.3}", symmetry);
        print_grid_mini(&nca.grid);
        println!();

        results.push((name, alive, symmetry));
    }

    // Compare patterns - they should look different
    println!("=== Results Summary ===");
    println!("Pattern      Alive  Symmetry");
    println!("----------------------------");
    for (name, alive, symmetry) in &results {
        println!("{:<12} {:>5}  {:.3}", name, alive, symmetry);
    }

    // Check if patterns are differentiated
    let alive_variance = calculate_variance(&results.iter().map(|r| r.1 as f64).collect::<Vec<_>>());
    let symmetry_variance = calculate_variance(&results.iter().map(|r| r.2).collect::<Vec<_>>());

    println!("\nVariance in alive cells: {:.1}", alive_variance);
    println!("Variance in symmetry: {:.4}", symmetry_variance);

    if alive_variance > 100.0 || symmetry_variance > 0.001 {
        println!("\n[OK] Patterns show differentiation - conditioning is having an effect!");
    } else {
        println!("\n[PARTIAL] Patterns are similar - network may need more multi-pattern training");
        println!("          (This is expected if training was sequential, not conditioned)");
    }
}

fn count_alive(grid: &Grid) -> usize {
    grid.cells.iter()
        .flatten()
        .filter(|cell| cell[3] > 0.1)
        .count()
}

fn calculate_center_of_mass(grid: &Grid) -> (f64, f64) {
    let mut sum_x = 0.0;
    let mut sum_y = 0.0;
    let mut total = 0.0;

    for y in 0..grid.height {
        for x in 0..grid.width {
            let val = grid.cells[y][x][3];
            sum_x += x as f64 * val;
            sum_y += y as f64 * val;
            total += val;
        }
    }

    if total > 0.0 {
        (sum_x / total, sum_y / total)
    } else {
        (16.0, 16.0)
    }
}

fn calculate_symmetry(grid: &Grid) -> f64 {
    // Check horizontal symmetry
    let mut diff = 0.0;
    let mut count = 0;

    for y in 0..grid.height {
        for x in 0..grid.width / 2 {
            let left = grid.cells[y][x][3];
            let right = grid.cells[y][grid.width - 1 - x][3];
            diff += (left - right).abs();
            count += 1;
        }
    }

    1.0 - (diff / count as f64)
}

fn calculate_variance(values: &[f64]) -> f64 {
    if values.is_empty() { return 0.0; }
    let mean = values.iter().sum::<f64>() / values.len() as f64;
    values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64
}

fn print_grid_mini(grid: &Grid) {
    for y in (0..32).step_by(4) {
        let mut line = String::from("  ");
        for x in (0..32).step_by(2) {
            let val = grid.cells[y][x][3];
            let ch = if val > 0.7 { '#' }
                else if val > 0.4 { '*' }
                else if val > 0.1 { '.' }
                else { ' ' };
            line.push(ch);
        }
        println!("{}", line);
    }
}
