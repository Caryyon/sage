//! Test visual learning pipeline
//!
//! This test verifies that SAGE can learn to reproduce visual patterns
//! without requiring camera access (uses synthetic images).
//!
//! Run with: cargo run --release --example test_visual_learning

use sage::nca::NCA;
use sage::grid::Grid;
use image::{RgbaImage, Rgba};

fn main() {
    println!("=== Visual Learning Pipeline Test ===\n");

    // Test 1: Create synthetic "visual" targets
    println!("--- Test 1: Synthetic Visual Targets ---");

    let gradient_target = create_gradient_image(32);
    let checkerboard_target = create_checkerboard_image(32, 4);
    let edge_target = create_edge_image(32);

    println!("[OK] Created 3 synthetic visual targets");
    print_image_preview(&gradient_target, "Gradient");
    print_image_preview(&checkerboard_target, "Checkerboard");
    print_image_preview(&edge_target, "Edge Pattern");

    // Test 2: Convert images to Grid targets
    println!("\n--- Test 2: Image to Grid Conversion ---");

    let gradient_grid = image_to_grid(&gradient_target);
    let checkerboard_grid = image_to_grid(&checkerboard_target);
    let edge_grid = image_to_grid(&edge_target);

    println!("[OK] Converted all images to Grid format");
    println!("Grid size: {}x{}", gradient_grid.width, gradient_grid.height);
    println!("Gradient avg brightness: {:.3}", calculate_avg_brightness(&gradient_grid));
    println!("Checkerboard avg brightness: {:.3}", calculate_avg_brightness(&checkerboard_grid));
    println!("Edge avg brightness: {:.3}", calculate_avg_brightness(&edge_grid));

    // Test 3: Train NCA on visual target
    println!("\n--- Test 3: Visual Learning (Gradient) ---");

    let mut nca = NCA::new();

    // Load existing weights as starting point
    if nca.load_weights_from_file("pattern_training_weights.json").is_ok() {
        println!("[OK] Loaded pre-trained weights");
    } else {
        println!("[INFO] Starting with random weights");
    }

    // Train on gradient for 100 iterations
    let mut losses = vec![];
    for i in 0..100 {
        nca.reset_with_seed();

        // Evolve
        for _ in 0..100 {
            nca.step();
        }

        // Calculate loss against visual target
        let loss = calculate_visual_loss(&nca.grid, &gradient_grid);
        losses.push(loss);

        // Train
        nca.train_step(&gradient_grid, 0.0002);

        if i % 25 == 0 || i == 99 {
            println!("Iteration {}: loss = {:.4}", i, loss);
        }
    }

    let initial_loss = losses[0];
    let final_loss = losses[losses.len() - 1];
    let improvement = ((initial_loss - final_loss) / initial_loss) * 100.0;

    println!("\nVisual learning results:");
    println!("  Initial loss: {:.4}", initial_loss);
    println!("  Final loss:   {:.4}", final_loss);
    println!("  Improvement:  {:.1}%", improvement);

    // Test 4: Reproduce learned visual pattern
    println!("\n--- Test 4: Reproduce Learned Pattern ---");

    nca.reset_with_seed();
    for _ in 0..150 {
        nca.step();
    }

    let final_visual_loss = calculate_visual_loss(&nca.grid, &gradient_grid);
    println!("Reproduction loss after 150 steps: {:.4}", final_visual_loss);

    print_grid_preview(&nca.grid, "NCA Output");
    print_grid_preview(&gradient_grid, "Target (Gradient)");

    // Test 5: Color channel learning
    println!("\n--- Test 5: Color Channel Test ---");

    let color_target = create_color_test_image(32);
    let color_grid = image_to_grid(&color_target);

    let mut color_nca = NCA::new();
    println!("Training on color pattern...");

    for i in 0..50 {
        color_nca.reset_with_seed();
        for _ in 0..100 {
            color_nca.step();
        }
        color_nca.train_step(&color_grid, 0.0003);

        if i == 49 {
            let loss = calculate_visual_loss(&color_nca.grid, &color_grid);
            println!("Final color loss: {:.4}", loss);
        }
    }

    // Summary
    println!("\n=== Summary ===");
    if improvement > 10.0 {
        println!("[OK] Visual learning is working - loss improved by {:.1}%", improvement);
    } else if improvement > 0.0 {
        println!("[PARTIAL] Some visual learning - consider more iterations");
    } else {
        println!("[NEEDS WORK] Visual learning not converging - may need architecture changes");
    }
}

fn create_gradient_image(size: u32) -> RgbaImage {
    let mut img = RgbaImage::new(size, size);
    for y in 0..size {
        for x in 0..size {
            let brightness = ((x as f64 / size as f64) * 255.0) as u8;
            img.put_pixel(x, y, Rgba([brightness, brightness, brightness, 255]));
        }
    }
    img
}

fn create_checkerboard_image(size: u32, cell_size: u32) -> RgbaImage {
    let mut img = RgbaImage::new(size, size);
    for y in 0..size {
        for x in 0..size {
            let is_white = ((x / cell_size) + (y / cell_size)) % 2 == 0;
            let val = if is_white { 255 } else { 0 };
            img.put_pixel(x, y, Rgba([val, val, val, 255]));
        }
    }
    img
}

fn create_edge_image(size: u32) -> RgbaImage {
    let mut img = RgbaImage::new(size, size);
    let half = size / 2;
    for y in 0..size {
        for x in 0..size {
            let val = if x < half { 0 } else { 255 };
            img.put_pixel(x, y, Rgba([val, val, val, 255]));
        }
    }
    img
}

fn create_color_test_image(size: u32) -> RgbaImage {
    let mut img = RgbaImage::new(size, size);
    let third = size / 3;
    for y in 0..size {
        for x in 0..size {
            let (r, g, b) = if x < third {
                (255, 0, 0)  // Red
            } else if x < third * 2 {
                (0, 255, 0)  // Green
            } else {
                (0, 0, 255)  // Blue
            };
            img.put_pixel(x, y, Rgba([r, g, b, 255]));
        }
    }
    img
}

fn image_to_grid(img: &RgbaImage) -> Grid {
    let size = img.width() as usize;
    let mut grid = Grid::new(size, size);

    for y in 0..size {
        for x in 0..size {
            let pixel = img.get_pixel(x as u32, y as u32);
            grid.cells[y][x][0] = pixel[0] as f64 / 255.0;  // R
            grid.cells[y][x][1] = pixel[1] as f64 / 255.0;  // G
            grid.cells[y][x][2] = pixel[2] as f64 / 255.0;  // B
            grid.cells[y][x][3] = pixel[3] as f64 / 255.0;  // A
        }
    }

    grid
}

fn calculate_visual_loss(nca_grid: &Grid, target: &Grid) -> f64 {
    let mut loss = 0.0;
    // Calculate loss on RGBA channels (visual channels)
    for y in 0..nca_grid.height {
        for x in 0..nca_grid.width {
            for c in 0..4 {  // RGBA channels
                let diff = nca_grid.cells[y][x][c] - target.cells[y][x][c];
                loss += diff * diff;
            }
        }
    }
    loss / (nca_grid.width * nca_grid.height * 4) as f64
}

fn calculate_avg_brightness(grid: &Grid) -> f64 {
    let mut total = 0.0;
    for y in 0..grid.height {
        for x in 0..grid.width {
            total += grid.cells[y][x][3];  // Alpha as brightness
        }
    }
    total / (grid.width * grid.height) as f64
}

fn print_image_preview(img: &RgbaImage, title: &str) {
    println!("\n{} (8x4 preview):", title);
    for y in (0..32).step_by(8) {
        let mut line = String::from("  ");
        for x in (0..32).step_by(4) {
            let pixel = img.get_pixel(x, y);
            let brightness = (pixel[0] as f64 + pixel[1] as f64 + pixel[2] as f64) / (3.0 * 255.0);
            let ch = if brightness > 0.75 { '#' }
                else if brightness > 0.5 { '*' }
                else if brightness > 0.25 { '.' }
                else { ' ' };
            line.push(ch);
        }
        println!("{}", line);
    }
}

fn print_grid_preview(grid: &Grid, title: &str) {
    println!("\n{} (8x4 preview):", title);
    for y in (0..32).step_by(8) {
        let mut line = String::from("  ");
        for x in (0..32).step_by(4) {
            let val = grid.cells[y][x][3];  // Alpha channel
            let ch = if val > 0.75 { '#' }
                else if val > 0.5 { '*' }
                else if val > 0.25 { '.' }
                else { ' ' };
            line.push(ch);
        }
        println!("{}", line);
    }
}
