//! sage-diag — Brain file diagnostic tool
//! Dumps exact serialization structure to find the corruption bug.

use sage::distributed_knowledge::{BrainHeader, BRAIN_MAGIC, NCAKnowledge, KnowledgeStore, default_brain_path};
use sage::grid::Grid;
use std::path::Path;

fn main() {
    let brain_path = default_brain_path();
    let path = Path::new(&brain_path);
    
    println!("=== Brain File Diagnostic ===\n");
    println!("Path: {}", brain_path);
    
    if !path.exists() {
        println!("File does not exist!");
        return;
    }
    
    let file_size = std::fs::metadata(path).unwrap().len();
    println!("File size: {} bytes ({:.1} MB)", file_size, file_size as f64 / 1_048_576.0);
    
    // Read raw bytes
    let data = std::fs::read(path).unwrap();
    
    // Parse header
    let header_size = BrainHeader::serialized_size();
    println!("Header serialized size: {} bytes", header_size);
    
    let header: BrainHeader = bincode::deserialize(&data[..header_size]).unwrap();
    println!("Header: magic={:?} version={} grid_size={} channels={} created_at={}",
        std::str::from_utf8(&header.magic).unwrap_or("???"),
        header.version, header.grid_size, header.channels, header.created_at);
    
    // Now try to deserialize the grid
    let grid_offset = header_size;
    println!("\nGrid data starts at offset: {}", grid_offset);
    println!("Grid data size: {} bytes", data.len() - grid_offset);
    
    // Peek at first few bytes of grid data
    println!("\nFirst 40 bytes of grid data:");
    for i in 0..40 {
        if i % 8 == 0 { print!("  +{}: ", i); }
        print!("{:02x} ", data[grid_offset + i]);
        if i % 8 == 7 { println!(); }
    }
    println!();
    
    // Try to deserialize
    match bincode::deserialize::<Grid>(&data[grid_offset..]) {
        Ok(grid) => {
            println!("\nGrid deserialized OK:");
            println!("  width={} height={}", grid.width, grid.height);
            println!("  cells.len()={}", grid.cells.len());
            if !grid.cells.is_empty() {
                println!("  cells[0].len()={}", grid.cells[0].len());
                if !grid.cells[0].is_empty() {
                    println!("  cells[0][0].len()={}", grid.cells[0][0].len());
                    println!("  cells[0][0][0..4]={:?}", &grid.cells[0][0][..4.min(grid.cells[0][0].len())]);
                }
            }
            println!("  alive_count={}", grid.alive_count());
            println!("  death_counters.len()={}", grid.death_counters.len());
            println!("  dead_cells.len()={}", grid.dead_cells.len());
            println!("  species.len()={}", grid.species.len());
        }
        Err(e) => {
            println!("\nGrid deserialization FAILED: {}", e);
        }
    }
    
    // Now try full NCAKnowledge load
    println!("\n=== Full NCAKnowledge Load ===");
    let mut knowledge = NCAKnowledge::new();
    match knowledge.load(&brain_path) {
        Ok(()) => {
            println!("Load OK:");
            println!("  grid: {}×{}, {} alive cells", 
                knowledge.grid.width, knowledge.grid.height, knowledge.grid.alive_count());
            println!("  text_store: {} entries", knowledge.text_store.len());
        }
        Err(e) => {
            println!("Load FAILED: {}", e);
        }
    }
    
    // Check text_store file
    let ts_path = brain_path.replace(".bin", "_text_store.bin");
    let ts_path2 = brain_path.replace("brain.bin", "text_store.bin");
    println!("\nText store paths:");
    for p in &[&ts_path, &ts_path2] {
        let pp = Path::new(p);
        println!("  {}: exists={} size={}", p, pp.exists(), 
            if pp.exists() { std::fs::metadata(pp).unwrap().len() } else { 0 });
    }
}
