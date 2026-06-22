use sage::distributed_knowledge::{KnowledgeStore, NCAKnowledge, default_brain_path};
use sage::grid::{Grid, KNOWLEDGE_ACTIVATION, NUM_CHANNELS};

fn main() {
    println!("=== Test 1: Fresh brain, encode, check alive ===");
    let mut k = NCAKnowledge::new();
    println!("Before encode: alive={}, cells.len()={}", k.grid.alive_count(), k.grid.cells.len());
    
    k.encode("The quick brown fox jumps over the lazy dog", 0.85);
    k.encode("Neural networks learn through gradient descent", 0.85);
    k.encode("Rust provides memory safety without garbage collection", 0.85);
    
    let alive = k.grid.alive_count();
    println!("After 3 encodes: alive={}, cells.len()={}", alive, k.grid.cells.len());
    
    // Check specific cells
    let mut nonzero = 0;
    for y in 0..k.grid.height {
        for x in 0..k.grid.width {
            if k.grid.cells[y][x][KNOWLEDGE_ACTIVATION] > 0.1 {
                nonzero += 1;
            }
        }
    }
    println!("Manual scan: {} cells with KNOWLEDGE_ACTIVATION > 0.1", nonzero);
    
    println!("\n=== Test 2: Save and reload ===");
    let test_path = "/tmp/sage_brain_test.bin";
    let _ = std::fs::remove_file(test_path);
    let _ = std::fs::remove_file(format!("{}.tmp", test_path));
    
    match k.save(test_path) {
        Ok(()) => println!("Save: OK"),
        Err(e) => { println!("Save ERR: {}", e); return; }
    }
    
    let mut k2 = NCAKnowledge::new();
    match k2.load(test_path) {
        Ok(()) => {
            let alive2 = k2.grid.alive_count();
            let cells_len = k2.grid.cells.len();
            println!("Reload: alive={}, cells.len()={}", alive2, cells_len);
            if cells_len > 0 {
                let ch_len = k2.grid.cells[0][0].len();
                println!("channels: {}", ch_len);
            }
        }
        Err(e) => println!("Reload ERR: {}", e),
    }
    
    // Check the file directly
    use std::convert::TryInto;
    let data = std::fs::read(test_path).unwrap();
    println!("\nFile size: {} bytes", data.len());
    
    // Header size
    let header_size = sage::distributed_knowledge::BrainHeader::serialized_size();
    println!("BrainHeader::serialized_size() = {}", header_size);
    
    let gd = &data[header_size..];
    let cells_len = u64::from_le_bytes(gd[0..8].try_into().unwrap());
    println!("File cells_len = {}", cells_len);
    
    if cells_len == 256 {
        println!("File has correct cells length!");
        // Check if data is non-zero
        let offset = 8 + 8; // skip cells_len, skip cells[0] len
        let inner_len = u64::from_le_bytes(gd[offset..offset+8].try_into().unwrap());
        println!("File cells[0] len = {}", inner_len);
    }
}