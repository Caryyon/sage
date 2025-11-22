//! Test damage resistance in the NCA
//!
//! This test verifies that:
//! 1. The NCA can form a pattern
//! 2. Damage visibly destroys part of the pattern
//! 3. The NCA can recover from damage
//!
//! Run with: cargo run --release --example test_damage_resistance

use sage::nca::NCA;
use sage::grid::Grid;

fn main() {
    println!("=== NCA Damage Resistance Test ===\n");

    // Create NCA (uses default 32x32 grid)
    let mut nca = NCA::new();

    // Try to load trained weights
    if nca.load_weights_from_file("pattern_training_weights.json").is_ok() {
        println!("[OK] Loaded trained weights from pattern_training_weights.json");
    } else {
        println!("[WARN] No trained weights found, using random initialization");
        println!("       (Run the main TUI first to train some patterns)\n");
    }

    // Create a circle target for reference
    let target = create_circle_target(32);

    println!("\n--- Phase 1: Initial Pattern Formation ---");
    nca.reset_with_seed();

    // Evolve to form pattern
    for step in 0..80 {
        nca.step();
        if step % 20 == 19 {
            let loss = calculate_loss(&nca.grid, &target);
            println!("Step {}: loss = {:.4}", step + 1, loss);
        }
    }

    let pre_damage_loss = calculate_loss(&nca.grid, &target);
    println!("\nPattern formed! Loss: {:.4}", pre_damage_loss);
    print_grid_mini(&nca.grid, "Formed Pattern");

    println!("\n--- Phase 2: Applying Damage ---");

    // Store pre-damage alive count
    let pre_damage_alive = count_alive_cells(&nca.grid);

    // Apply damage
    nca.apply_damage();

    let post_damage_alive = count_alive_cells(&nca.grid);
    let post_damage_loss = calculate_loss(&nca.grid, &target);

    println!("Damage applied!");
    println!("  Alive cells: {} -> {} (lost {})",
        pre_damage_alive, post_damage_alive, pre_damage_alive - post_damage_alive);
    println!("  Loss: {:.4} -> {:.4} (increased by {:.4})",
        pre_damage_loss, post_damage_loss, post_damage_loss - pre_damage_loss);
    print_grid_mini(&nca.grid, "After Damage");

    println!("\n--- Phase 3: Recovery ---");

    // Try to recover
    let mut recovery_losses = vec![];
    for step in 0..48 {
        nca.step();
        if step % 12 == 11 {
            let loss = calculate_loss(&nca.grid, &target);
            recovery_losses.push(loss);
            println!("Recovery step {}: loss = {:.4}", step + 1, loss);
        }
    }

    let final_loss = calculate_loss(&nca.grid, &target);
    let final_alive = count_alive_cells(&nca.grid);

    println!("\nFinal state:");
    println!("  Alive cells: {} (recovered {} cells)", final_alive, final_alive as i32 - post_damage_alive as i32);
    println!("  Loss: {:.4}", final_loss);
    print_grid_mini(&nca.grid, "After Recovery");

    println!("\n=== Results ===");
    let recovery_amount = post_damage_loss - final_loss;
    let recovery_percent = (recovery_amount / (post_damage_loss - pre_damage_loss)) * 100.0;

    println!("Pre-damage loss:  {:.4}", pre_damage_loss);
    println!("Post-damage loss: {:.4}", post_damage_loss);
    println!("Final loss:       {:.4}", final_loss);
    println!("Recovery:         {:.4} ({:.1}% of damage recovered)", recovery_amount, recovery_percent.max(0.0));

    if final_loss < post_damage_loss * 0.8 {
        println!("\n[OK] DAMAGE RESISTANCE WORKING - Pattern recovered significantly!");
    } else if final_loss < post_damage_loss {
        println!("\n[PARTIAL] Some recovery detected, but may need more training");
    } else {
        println!("\n[FAIL] No significant recovery - damage resistance needs more training");
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
                grid.cells[y][x][3] = 1.0;  // Alpha
                grid.cells[y][x][0] = 1.0;  // R
            }
        }
    }
    grid
}

fn calculate_loss(grid: &Grid, target: &Grid) -> f64 {
    let mut loss = 0.0;
    let mut count = 0;
    for y in 0..grid.height {
        for x in 0..grid.width {
            let diff = grid.cells[y][x][3] - target.cells[y][x][3];
            loss += diff * diff;
            count += 1;
        }
    }
    loss / count as f64
}

fn count_alive_cells(grid: &Grid) -> usize {
    grid.cells.iter()
        .flatten()
        .filter(|cell| cell[3] > 0.1)
        .count()
}

fn print_grid_mini(grid: &Grid, title: &str) {
    println!("\n{} (16x8 preview):", title);
    // Print a downsampled view
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
