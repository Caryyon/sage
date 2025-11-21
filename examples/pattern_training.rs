// Pattern Training - Simple visualization of SAGE learning geometric shapes
//
// This example shows SAGE training on the 4 patterns from the Growing CA paper:
// Circle, Square, Cross, Spiral
//
// Watch the loss decrease as SAGE learns each pattern!

use sage::nca::NCA;
use sage::grid::{Grid, GRID_SIZE};

fn main() {
    println!("╔════════════════════════════════════════════════════════╗");
    println!("║   SAGE Pattern Training - Growing CA Implementation   ║");
    println!("║          Circle → Square → Cross → Spiral              ║");
    println!("╚════════════════════════════════════════════════════════╝\n");

    let mut nca = NCA::new();

    let patterns = vec![
        ("Circle", generate_circle()),
        ("Square", generate_square()),
        ("Cross", generate_cross()),
        ("Spiral", generate_spiral()),
    ];

    for (name, target) in patterns {
        println!("\n═══ Training: {} ═══", name);
        train_pattern(&mut nca, &target, name);
    }

    println!("\n✅ All patterns completed!");
}

fn train_pattern(nca: &mut NCA, target: &Grid, name: &str) {
    let learning_rate = if name == "Square" { 0.0002 } else { 0.0001 };
    let max_iterations = 1000;
    let evolution_steps = 100;

    for iteration in 0..max_iterations {
        // Reset and evolve
        nca.reset_with_seed();
        for _ in 0..evolution_steps {
            nca.step();
        }

        // Calculate loss
        let loss = calculate_loss(nca, target);

        // Train
        nca.train_step(target, learning_rate);

        // Print progress
        if iteration % 100 == 0 {
            print_progress(iteration, max_iterations, loss, nca, target);
        }

        // Check if converged
        if loss < 0.05 {
            println!("\n✓ MASTERED at iteration {} - Loss: {:.4}", iteration, loss);
            return;
        }
    }

    let final_loss = calculate_loss(nca, target);
    println!("\n→ Moved on after {} iterations - Final loss: {:.4}", max_iterations, final_loss);
}

fn calculate_loss(nca: &NCA, target: &Grid) -> f64 {
    let mut total_loss = 0.0;
    let mut count = 0;

    for y in 0..GRID_SIZE {
        for x in 0..GRID_SIZE {
            for channel in 0..4 {  // Only RGBA channels
                let diff = nca.grid.cells[y][x][channel] - target.cells[y][x][channel];
                total_loss += diff * diff;
                count += 1;
            }
        }
    }

    total_loss / count as f64
}

fn print_progress(iteration: usize, max_iterations: usize, loss: f64, nca: &NCA, target: &Grid) {
    let progress = (iteration as f64 / max_iterations as f64 * 100.0) as usize;
    let stage = if loss < 0.05 {
        "MASTERED"
    } else if loss < 0.10 {
        "CONVERGING"
    } else if loss < 0.20 {
        "LEARNING"
    } else {
        "EXPLORING"
    };

    println!("  [{:3}%] Iter {:4}/{} | Loss: {:.4} | {}",
        progress, iteration, max_iterations, loss, stage);

    // Show visualization every 200 iterations
    if iteration % 200 == 0 {
        print_comparison(nca, target);
    }
}

fn print_comparison(nca: &NCA, target: &Grid) {
    println!("\n    TARGET          CURRENT         DIFFERENCE");
    println!("  ┌───────────┐  ┌───────────┐  ┌───────────┐");

    // Print a 16x16 subsection (center of the 32x32 grid)
    let start = 8;
    let end = 24;

    for y in start..end {
        print!("  │");
        // TARGET
        for x in start..end {
            let alpha = target.cells[y][x][3];
            print!("{}", value_to_char(alpha));
        }

        print!("│  │");
        // CURRENT
        for x in start..end {
            let alpha = nca.grid.cells[y][x][3];
            print!("{}", value_to_char(alpha));
        }

        print!("│  │");
        // DIFFERENCE
        for x in start..end {
            let target_alpha = target.cells[y][x][3];
            let current_alpha = nca.grid.cells[y][x][3];
            let diff = (target_alpha - current_alpha).abs();
            print!("{}", diff_to_char(diff));
        }

        println!("│");
    }

    println!("  └───────────┘  └───────────┘  └───────────┘\n");
}

fn value_to_char(value: f64) -> char {
    if value > 0.85 { '#' }
    else if value > 0.70 { '@' }
    else if value > 0.55 { '&' }
    else if value > 0.40 { '*' }
    else if value > 0.30 { '+' }
    else if value > 0.20 { '=' }
    else if value > 0.12 { '-' }
    else if value > 0.06 { '.' }
    else { ' ' }
}

fn diff_to_char(diff: f64) -> char {
    if diff > 0.5 { '#' }
    else if diff > 0.3 { '@' }
    else if diff > 0.15 { '%' }
    else if diff > 0.08 { 'o' }
    else if diff > 0.04 { '.' }
    else { ' ' }
}

// Pattern generators
fn generate_circle() -> Grid {
    let mut grid = Grid::new(GRID_SIZE, GRID_SIZE);
    let center_x = GRID_SIZE as f64 / 2.0;
    let center_y = GRID_SIZE as f64 / 2.0;
    let radius = 8.0;

    for y in 0..GRID_SIZE {
        for x in 0..GRID_SIZE {
            let dx = x as f64 - center_x;
            let dy = y as f64 - center_y;
            let dist = (dx * dx + dy * dy).sqrt();

            if dist < radius {
                grid.cells[y][x][3] = 1.0;  // Alpha
                grid.cells[y][x][0] = 1.0;  // Red
            }
        }
    }

    grid
}

fn generate_square() -> Grid {
    let mut grid = Grid::new(GRID_SIZE, GRID_SIZE);
    let center = GRID_SIZE / 2;
    let size = 12;

    for y in (center - size)..(center + size) {
        for x in (center - size)..(center + size) {
            if y < GRID_SIZE && x < GRID_SIZE {
                grid.cells[y][x][3] = 1.0;  // Alpha
                grid.cells[y][x][1] = 1.0;  // Green
            }
        }
    }

    grid
}

fn generate_cross() -> Grid {
    let mut grid = Grid::new(GRID_SIZE, GRID_SIZE);
    let center = GRID_SIZE / 2;
    let arm_width = 4;
    let arm_length = 10;

    // Horizontal arm
    for x in (center - arm_length)..(center + arm_length) {
        for dy in 0..arm_width {
            let y = center - arm_width/2 + dy;
            if y < GRID_SIZE && x < GRID_SIZE {
                grid.cells[y][x][3] = 1.0;  // Alpha
                grid.cells[y][x][2] = 1.0;  // Blue
            }
        }
    }

    // Vertical arm
    for y in (center - arm_length)..(center + arm_length) {
        for dx in 0..arm_width {
            let x = center - arm_width/2 + dx;
            if y < GRID_SIZE && x < GRID_SIZE {
                grid.cells[y][x][3] = 1.0;  // Alpha
                grid.cells[y][x][2] = 1.0;  // Blue
            }
        }
    }

    grid
}

fn generate_spiral() -> Grid {
    let mut grid = Grid::new(GRID_SIZE, GRID_SIZE);
    let center = GRID_SIZE / 2;

    for i in 0..100 {
        let angle = i as f64 * 0.3;
        let radius = i as f64 * 0.1;
        let x = (center as f64 + radius * angle.cos()) as usize;
        let y = (center as f64 + radius * angle.sin()) as usize;

        if x < GRID_SIZE && y < GRID_SIZE {
            grid.cells[y][x][3] = 1.0;  // Alpha
            grid.cells[y][x][0] = 1.0;  // Red
            grid.cells[y][x][1] = 0.5;  // Green (yellow-ish)

            // Make it thicker
            for dy in 0..2 {
                for dx in 0..2 {
                    let ny = y + dy;
                    let nx = x + dx;
                    if ny < GRID_SIZE && nx < GRID_SIZE {
                        grid.cells[ny][nx][3] = 1.0;
                        grid.cells[ny][nx][0] = 1.0;
                        grid.cells[ny][nx][1] = 0.5;
                    }
                }
            }
        }
    }

    grid
}
