// Quick test to verify TextEncoder produces visible patterns

use sage::text_encoder::TextEncoder;

fn main() {
    let mut encoder = TextEncoder::new();

    // Test encoding "love" just like baseline training does
    let grid = encoder.encode_concept("love");

    // Find max/min alpha values
    let mut max_alpha = 0.0f64;
    let mut min_alpha = 1.0f64;
    let mut non_zero_count = 0;

    for y in 0..48 {
        for x in 0..48 {
            let alpha = grid.cells[y][x][3];
            max_alpha = max_alpha.max(alpha.abs());
            min_alpha = min_alpha.min(alpha.abs());
            if alpha.abs() > 0.001 {
                non_zero_count += 1;
            }
        }
    }

    println!("TextEncoder test for 'love':");
    println!("  Max alpha: {:.6}", max_alpha);
    println!("  Min alpha: {:.6}", min_alpha);
    println!("  Non-zero cells: {}/{}", non_zero_count, 48*48);

    // Test the contrast boost that sync_nca_from_sage applies
    let boosted = max_alpha.abs().powf(0.7);
    println!("  After contrast boost (^0.7): {:.6}", boosted);

    // Check if it would be visible in the TUI (threshold is 0.05)
    if boosted > 0.05 {
        println!("  ✓ Would be VISIBLE in Living Neural Field");
    } else {
        println!("  ✗ Would be INVISIBLE in Living Neural Field (< 0.05)");
    }
}
