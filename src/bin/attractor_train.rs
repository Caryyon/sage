//! attractor-train — Train and test the Attractor Network
//!
//! Usage:
//!   attractor-train --store "text to memorize"
//!   attractor-train --recall "partial query"
//!   attractor-train --demo  (runs a full store/recall demo)

use sage::inference::attractor_network::{
    self, AttractorMlp, decode_pattern, downsample_grid, encode_pattern, recall, store_memory,
    write_attractor_state,
};
use sage::inference::nca_predictor::SimpleTokenizer;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let device = candle_core::Device::Cpu;

    let mut store_texts: Vec<String> = Vec::new();
    let mut recall_text: Option<String> = None;
    let mut demo = false;
    let mut grid_size = 64usize;
    let mut steps = 60usize;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--store" | "-s" => {
                i += 1;
                store_texts.push(args[i].clone());
            }
            "--recall" | "-r" => {
                i += 1;
                recall_text = Some(args[i].clone());
            }
            "--demo" => demo = true,
            "--grid" => {
                i += 1;
                grid_size = args[i].parse().unwrap_or(64);
            }
            "--steps" => {
                i += 1;
                steps = args[i].parse().unwrap_or(60);
            }
            _ => {}
        }
        i += 1;
    }

    let tokenizer = SimpleTokenizer::new(4096);

    if demo || (!store_texts.is_empty() && recall_text.is_some()) {
        run_demo(&tokenizer, &device, &store_texts, &recall_text, grid_size, steps, demo)?;
    } else if !store_texts.is_empty() {
        // Just store, no recall
        let mut mlp = AttractorMlp::new(&device)?;
        for text in &store_texts {
            println!("📝 Storing: \"{}\"", text);
            let pattern = encode_pattern(text, &tokenizer, grid_size, &device)?;
            let trace = store_memory(&pattern, &mut mlp, grid_size, steps, &device)?;
            println!("   Stabilized in {} steps", trace.len());

            // Write frames for TUI
            let frames: Vec<Vec<Vec<f64>>> = trace
                .iter()
                .map(|t| downsample_grid(t, grid_size))
                .collect();
            write_attractor_state("storing", text, &frames, store_texts.len(), grid_size);
        }
        let save_path = dirs::home_dir()
            .unwrap_or_default()
            .join(".sage")
            .join("attractor_weights.safetensors");
        mlp.save(&save_path)?;
        println!("💾 Saved weights to {}", save_path.display());
    } else if let Some(ref query) = recall_text {
        // Load weights and recall
        let load_path = dirs::home_dir()
            .unwrap_or_default()
            .join(".sage")
            .join("attractor_weights.safetensors");
        if !load_path.exists() {
            eprintln!("❌ No saved weights found. Store some memories first.");
            return Ok(());
        }
        let mlp = AttractorMlp::load(&load_path, &device)?;
        println!("🔍 Recalling with seed: \"{}\"", query);
        let seed = encode_pattern(query, &tokenizer, grid_size, &device)?;
        let trace = recall(&seed, &mlp, grid_size, steps)?;
        println!("   Converged in {} steps", trace.len());

        let stabilized = trace.last().unwrap();
        let decoded = decode_pattern(stabilized, &tokenizer, grid_size, 30)?;
        println!("🧠 Recalled: \"{}\"", decoded);

        // Write frames for TUI
        let frames: Vec<Vec<Vec<f64>>> = trace
            .iter()
            .map(|t| downsample_grid(t, grid_size))
            .collect();
        write_attractor_state("recalling", query, &frames, 1, grid_size);
    } else {
        eprintln!("Usage: attractor-train --store <text> [--store <text2>...] [--recall <query>]");
        eprintln!("       attractor-train --demo");
    }

    Ok(())
}

fn run_demo(
    tokenizer: &SimpleTokenizer,
    device: &candle_core::Device,
    store_texts: &[String],
    recall_text: &Option<String>,
    grid_size: usize,
    steps: usize,
    demo_mode: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let memories: Vec<String> = if demo_mode {
        vec![
            "the cat sat on the mat".to_string(),
            "the dog ran in the park".to_string(),
            "birds fly south for winter".to_string(),
            "fish swim in the ocean deep".to_string(),
        ]
    } else {
        store_texts.to_vec()
    };

    let query = if demo_mode {
        "the cat on the".to_string()
    } else {
        recall_text.clone().unwrap_or_default()
    };

    println!("═══ Attractor Network Demo ═══");
    println!("Grid: {}×{} ({} cells)", grid_size, grid_size, grid_size * grid_size);
    println!("NCA steps: {}", steps);
    println!();

    let mut mlp = AttractorMlp::new(device)?;

    // Phase 1: Store memories
    println!("📥 PHASE 1: Storing Memories");
    for (i, text) in memories.iter().enumerate() {
        println!("  [{}/{}] \"{}\"", i + 1, memories.len(), text);
        let pattern = encode_pattern(text, tokenizer, grid_size, device)?;
        let trace = store_memory(&pattern, &mut mlp, grid_size, steps, device)?;
        println!("       Stabilized in {} NCA steps", trace.len());

        // Write frames for TUI
        let frames: Vec<Vec<Vec<f64>>> = trace
            .iter()
            .map(|t| downsample_grid(t, grid_size))
            .collect();
        write_attractor_state("storing", text, &frames, i + 1, grid_size);
    }

    // Save weights
    let save_path = dirs::home_dir()
        .unwrap_or_default()
        .join(".sage")
        .join("attractor_weights.safetensors");
    fs::create_dir_all(save_path.parent().unwrap())?;
    mlp.save(&save_path)?;
    println!("💾 Saved weights to {}", save_path.display());
    println!();

    // Phase 2: Recall
    println!("🔍 PHASE 2: Recall");
    println!("  Seed: \"{}\"", query);
    let seed = encode_pattern(&query, tokenizer, grid_size, device)?;
    let trace = recall(&seed, &mlp, grid_size, steps)?;
    println!("  Converged in {} NCA steps", trace.len());

    let stabilized = trace.last().unwrap();
    let decoded = decode_pattern(stabilized, tokenizer, grid_size, 30)?;
    println!("🧠 Recalled: \"{}\"", decoded);

    // Write frames for TUI
    let frames: Vec<Vec<Vec<f64>>> = trace
        .iter()
        .map(|t| downsample_grid(t, grid_size))
        .collect();
    write_attractor_state("recalling", &query, &frames, memories.len(), grid_size);

    // Phase 3: Show convergence
    println!();
    println!("📊 PHASE 3: Convergence Analysis");
    for (i, grid) in trace.iter().enumerate().step_by(trace.len() / 10) {
        let mean_act: f64 = grid
            .mean_all()?
            .to_scalar()?;
        println!(
            "  Step {:3}: mean activation = {:.4}",
            i, mean_act
        );
    }

    println!();
    println!("✅ Demo complete. Run sage-tui to see the grid animation.");

    Ok(())
}
