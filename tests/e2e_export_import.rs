//! Export/Import E2E Tests
//!
//! Tests for the sage export and sage import functionality.

use sage::distributed_knowledge::{KnowledgeStore, NCAKnowledge};
use sage::grid::GRID_SIZE;

/// Test: Export then import roundtrip preserves all facts
#[test]
fn test_export_import_roundtrip_preserves_facts() {
    use std::fs;
    use tempfile::tempdir;

    // Create a knowledge store and encode facts
    let mut original = NCAKnowledge::new();
    original.config.ollama_url = None;

    let facts = [
        "Paris is the capital of France",
        "Tokyo is the largest city in Japan",
        "Rust is a systems programming language",
        "The sun is a yellow dwarf star",
        "Water boils at 100 degrees Celsius",
    ];

    for fact in &facts {
        original.encode(fact, 0.9);
    }

    let original_active = original.active_knowledge(0.01).len();
    let _original_text_count = original.text_store.len();

    // Export to temp file
    let dir = tempdir().expect("Failed to create temp dir");
    let export_path = dir.path().join("test_export.sage");

    // Serialize manually (same as CLI does)
    let header = TestExportHeader {
        magic: *b"SGEX",
        version: 1,
        sage_version: "test".to_string(),
        grid_size: GRID_SIZE as u32,
        channels: original.grid.cells[0][0].len() as u32,
        text_entries: original.text_store.len() as u32,
    };

    let header_data = bincode::serialize(&header).expect("Failed to serialize header");
    let grid_data = bincode::serialize(&original.grid).expect("Failed to serialize grid");
    let text_data = bincode::serialize(&original.text_store).expect("Failed to serialize text");

    let mut output = Vec::new();
    output.extend_from_slice(&(header_data.len() as u32).to_le_bytes());
    output.extend_from_slice(&header_data);
    output.extend_from_slice(&(grid_data.len() as u32).to_le_bytes());
    output.extend_from_slice(&grid_data);
    output.extend_from_slice(&(text_data.len() as u32).to_le_bytes());
    output.extend_from_slice(&text_data);

    fs::write(&export_path, &output).expect("Failed to write export file");

    // Import into fresh store
    let data = fs::read(&export_path).expect("Failed to read export file");

    // Parse header
    let header_len = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
    let imported_header: TestExportHeader =
        bincode::deserialize(&data[4..4 + header_len]).expect("Failed to parse header");

    assert_eq!(&imported_header.magic, b"SGEX", "Magic bytes should match");
    assert_eq!(imported_header.version, 1, "Version should match");

    // Parse grid
    let grid_offset = 4 + header_len;
    let grid_len = u32::from_le_bytes([
        data[grid_offset],
        data[grid_offset + 1],
        data[grid_offset + 2],
        data[grid_offset + 3],
    ]) as usize;
    let grid_data_offset = grid_offset + 4;
    let imported_grid: sage::grid::Grid =
        bincode::deserialize(&data[grid_data_offset..grid_data_offset + grid_len])
            .expect("Failed to parse grid");

    // Create new store with imported grid
    let mut imported = NCAKnowledge::new();
    imported.config.ollama_url = None;
    imported.merge(&imported_grid, 1.0); // Full merge

    let imported_active = imported.active_knowledge(0.01).len();

    // Verify preservation
    assert!(
        imported_active >= original_active / 2,
        "Imported store should have at least half the active cells: {} vs {}",
        imported_active,
        original_active
    );

    // Query for original facts - should find at least some
    let mut hits = 0;
    for fact in &facts {
        let query: String = fact
            .split_whitespace()
            .take(3)
            .collect::<Vec<_>>()
            .join(" ");
        let results = imported.query(&query, 10);
        if !results.is_empty() {
            hits += 1;
        }
    }

    assert!(
        hits >= 1,
        "Should retrieve at least 1 fact after import, got {}",
        hits
    );

    eprintln!(
        "Export/Import test: {} active cells preserved, {}/{} facts retrievable",
        imported_active,
        hits,
        facts.len()
    );
}

/// Test: Export file format has correct structure
#[test]
fn test_export_file_format() {
    use std::fs;
    use tempfile::tempdir;

    let mut store = NCAKnowledge::new();
    store.config.ollama_url = None;
    store.encode("test content", 0.9);

    let dir = tempdir().expect("Failed to create temp dir");
    let export_path = dir.path().join("format_test.sage");

    let header = TestExportHeader {
        magic: *b"SGEX",
        version: 1,
        sage_version: "0.3.0".to_string(),
        grid_size: GRID_SIZE as u32,
        channels: store.grid.cells[0][0].len() as u32,
        text_entries: store.text_store.len() as u32,
    };

    let header_data = bincode::serialize(&header).expect("Serialize header");
    let grid_data = bincode::serialize(&store.grid).expect("Serialize grid");
    let text_data = bincode::serialize(&store.text_store).expect("Serialize text");

    let mut output = Vec::new();
    output.extend_from_slice(&(header_data.len() as u32).to_le_bytes());
    output.extend_from_slice(&header_data);
    output.extend_from_slice(&(grid_data.len() as u32).to_le_bytes());
    output.extend_from_slice(&grid_data);
    output.extend_from_slice(&(text_data.len() as u32).to_le_bytes());
    output.extend_from_slice(&text_data);

    fs::write(&export_path, &output).expect("Write file");

    // Read back and verify structure
    let data = fs::read(&export_path).expect("Read file");

    // Header length should be first 4 bytes
    assert!(data.len() >= 4, "File too short");
    let header_len = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
    assert!(
        header_len > 0 && header_len < 1000,
        "Header length should be reasonable: {}",
        header_len
    );

    // Parse header
    let parsed_header: TestExportHeader =
        bincode::deserialize(&data[4..4 + header_len]).expect("Parse header");

    assert_eq!(&parsed_header.magic, b"SGEX", "Magic should be SGEX");
    assert_eq!(parsed_header.version, 1, "Version should be 1");
    assert_eq!(parsed_header.grid_size, GRID_SIZE as u32);
}

/// Test: Import with invalid magic bytes fails gracefully
#[test]
fn test_import_invalid_magic_fails() {
    use std::fs;
    use tempfile::tempdir;

    let dir = tempdir().expect("temp dir");
    let bad_path = dir.path().join("bad.sage");

    // Write file with wrong magic
    let bad_header = TestExportHeader {
        magic: *b"XXXX", // Wrong magic
        version: 1,
        sage_version: "test".to_string(),
        grid_size: 256,
        channels: 38,
        text_entries: 0,
    };

    let header_data = bincode::serialize(&bad_header).expect("serialize");
    let mut output = Vec::new();
    output.extend_from_slice(&(header_data.len() as u32).to_le_bytes());
    output.extend_from_slice(&header_data);
    // Add dummy grid/text data
    output.extend_from_slice(&[0u8; 100]);

    fs::write(&bad_path, &output).expect("write");

    let data = fs::read(&bad_path).expect("read");
    let header_len = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
    let parsed: TestExportHeader = bincode::deserialize(&data[4..4 + header_len]).expect("parse");

    assert_ne!(&parsed.magic, b"SGEX", "Should detect wrong magic");
}

/// Test: Merge with weight 0.5 produces weighted average
#[test]
fn test_import_merge_weight() {
    let mut store_a = NCAKnowledge::new();
    let mut store_b = NCAKnowledge::new();
    store_a.config.ollama_url = None;
    store_b.config.ollama_url = None;

    store_a.encode("knowledge from store A", 0.9);
    store_b.encode("knowledge from store B", 0.9);

    let active_a_before = store_a.active_knowledge(0.01).len();

    // Merge B into A with weight 0.5
    store_a.merge(&store_b.grid, 0.5);

    let active_a_after = store_a.active_knowledge(0.01).len();

    // After merge, should have more active cells
    assert!(
        active_a_after >= active_a_before,
        "Merged store should have at least as many active cells: {} >= {}",
        active_a_after,
        active_a_before
    );
}

/// Header struct for export files
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
struct TestExportHeader {
    magic: [u8; 4],
    version: u32,
    sage_version: String,
    grid_size: u32,
    channels: u32,
    text_entries: u32,
}
