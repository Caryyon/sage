//! sage-predict: Next-token prediction test for the 256×256 NCA brain.
//!
//! Uses trained MLP weights (from sage-brain-train) to run NCA steps
//! and measures embedding similarity between the peak activation cell
//! and the target word's embedding.
//!
//! Two metrics:
//!   1. Embedding similarity: cosine sim between top cell's embedding and target
//!   2. Positional accuracy: is the target's hash position in top-5/20 active cells?
//!
//! Random baseline for embedding similarity: ~0.0 (random embeddings are orthogonal)
//! Random baseline for positional: 5/65536 = 0.008% (top-5)

use sage::distributed_knowledge::brain_processor::BrainNcaWeights;
use sage::distributed_knowledge::encoder::{encode_text, feature_to_position, EncoderConfig};
use sage::distributed_knowledge::{default_brain_path, KnowledgeStore, NCAKnowledge};
use sage::grid::{Grid, GRID_SIZE, KNOWLEDGE_ACTIVATION, KNOWLEDGE_CHANNELS_START};
use std::fs;

const NCA_STEPS: usize = 5;
const NUM_TEST_SENTENCES: usize = 200;

fn build_test_sentences(corpus_dir: &str) -> Vec<(String, String)> {
    let mut tests = Vec::new();
    let handcrafted: &[(&str, &str)] = &[
        ("the cat sat on the", "mat"),
        ("she walked to the", "store"),
        ("he opened the door", "slowly"),
        ("they were happy", "together"),
        ("the sun rose over the", "mountains"),
        ("she was a young", "woman"),
        ("he looked at her with", "surprise"),
        ("the old man walked", "slowly"),
        ("it was a dark and stormy", "night"),
        ("she closed her eyes and", "slept"),
        ("he took a deep", "breath"),
        ("the door opened", "slowly"),
        ("she smiled at", "him"),
        ("he shook his", "head"),
        ("the room was", "quiet"),
        ("she turned her", "head"),
        ("he stood up and", "walked"),
        ("the light was", "bright"),
        ("she put her hand on", "his"),
        ("he walked across the", "room"),
        ("it was a beautiful", "day"),
        ("the sound of the", "music"),
        ("she looked out the", "window"),
        ("he sat down in the", "chair"),
        ("the water was", "cold"),
        ("she held her", "breath"),
        ("he turned to", "her"),
        ("the night was", "dark"),
        ("she opened her", "eyes"),
        ("he walked through the", "door"),
        ("the wind was", "cold"),
        ("she walked down the", "street"),
        ("he picked up the", "book"),
        ("the sky was", "blue"),
        ("she heard a", "noise"),
        ("he looked at the", "sky"),
        ("the door was", "open"),
        ("she stood in the", "doorway"),
        ("the fire was", "warm"),
        ("she sat on the", "floor"),
        ("he turned his", "head"),
        ("the house was", "old"),
        ("she looked at the", "clock"),
        ("he walked toward the", "door"),
        ("the road was", "long"),
        ("she took his", "hand"),
        ("he closed the", "door"),
        ("the morning was", "cold"),
        ("the stars were", "bright"),
        ("he felt a sudden", "chill"),
    ];
    for (prefix, target) in handcrafted {
        tests.push((prefix.to_string(), target.to_string()));
    }
    if let Ok(entries) = fs::read_dir(corpus_dir) {
        for entry in entries.flatten() {
            if tests.len() >= NUM_TEST_SENTENCES { break; }
            let path = entry.path();
            if path.extension().map(|e| e == "txt").unwrap_or(false) {
                if let Ok(text) = fs::read_to_string(&path) {
                    for sent in text.split(|c: char| c == '.' || c == '!' || c == '?') {
                        if tests.len() >= NUM_TEST_SENTENCES { break; }
                        let words: Vec<&str> = sent.split_whitespace().collect();
                        if words.len() >= 6 && words.len() <= 20 {
                            let target = words[words.len() - 1];
                            let target_clean = target.trim_matches(|c: char| !c.is_alphanumeric());
                            if target_clean.len() >= 3 && !target_clean.is_empty() {
                                tests.push((words[..words.len()-1].join(" "), target_clean.to_string()));
                            }
                        }
                    }
                }
            }
        }
    }
    tests
}

fn cosine_sim(a: &[f64], b: &[f64]) -> f64 {
    let dot: f64 = a.iter().zip(b).map(|(x, y)| x * y).sum();
    let mag_a: f64 = a.iter().map(|v| v * v).sum::<f64>().sqrt();
    let mag_b: f64 = b.iter().map(|v| v * v).sum::<f64>().sqrt();
    if mag_a < 1e-10 || mag_b < 1e-10 { 0.0 } else { dot / (mag_a * mag_b) }
}

/// NCA step using trained MLP weights
fn nca_brain_step_weighted(grid: &mut Grid, weights: &BrainNcaWeights) {
    let w = grid.width;
    let h = grid.height;
    let mut active: Vec<(usize, usize)> = Vec::new();
    {
        let mut seen = vec![vec![false; w]; h];
        for y in 0..h {
            for x in 0..w {
                if grid.cells[y][x][KNOWLEDGE_ACTIVATION] > 0.01 {
                    for dy in -1i32..=1 {
                        for dx in -1i32..=1 {
                            let ny = ((y as i32 + dy).rem_euclid(h as i32)) as usize;
                            let nx = ((x as i32 + dx).rem_euclid(w as i32)) as usize;
                            if !seen[ny][nx] { seen[ny][nx] = true; active.push((ny, nx)); }
                        }
                    }
                }
            }
        }
    }
    if active.is_empty() { return; }
    const NCH: usize = 50;
    const PERC: usize = 9 * NCH;
    let mut deltas: Vec<(usize, usize, Vec<f64>)> = Vec::with_capacity(active.len());
    for (cy, cx) in &active {
        let y = *cy; let x = *cx;
        let mut input = vec![0.0f64; PERC];
        let mut idx = 0;
        for dy in -1i32..=1 {
            for dx in -1i32..=1 {
                let ny = ((y as i32 + dy).rem_euclid(h as i32)) as usize;
                let nx = ((x as i32 + dx).rem_euclid(w as i32)) as usize;
                for ch in 0..NCH { input[idx] = grid.cells[ny][nx][KNOWLEDGE_CHANNELS_START + ch]; idx += 1; }
            }
        }
        let mut h1 = vec![0.0f64; 128];
        for i in 0..128 { let mut sum = weights.b1[i]; let row = &weights.w1[i]; for j in 0..PERC { sum += row[j] * input[j]; } h1[i] = sum.max(0.0); }
        let mut h2 = vec![0.0f64; 32];
        for i in 0..32 { let mut sum = weights.b2[i]; let row = &weights.w2[i]; for j in 0..128 { sum += row[j] * h1[j]; } h2[i] = sum.max(0.0); }
        let mut out = vec![0.0f64; NCH];
        for i in 0..NCH { let mut sum = weights.b3[i]; let row = &weights.w3[i]; for j in 0..32 { sum += row[j] * h2[j]; } out[i] = sum.tanh() * 0.1; }
        deltas.push((y, x, out));
    }
    let decay = 0.005;
    for (y, x, out) in &deltas {
        for (i, &delta) in out.iter().enumerate() {
            let ch = KNOWLEDGE_CHANNELS_START + i;
            if ch == KNOWLEDGE_ACTIVATION {
                grid.cells[*y][*x][ch] = (grid.cells[*y][*x][ch] * (1.0 - decay) + delta).clamp(0.0, 1.0);
            } else {
                grid.cells[*y][*x][ch] = (grid.cells[*y][*x][ch] + delta).clamp(-5.0, 5.0);
            }
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let corpus_dir = args.get(1).cloned().unwrap_or_else(|| {
        format!("{}/.sage/corpus/", std::env::var("HOME").unwrap_or_else(|_| ".".to_string()))
    });

    // Load brain
    let brain_path = default_brain_path();
    let mut store = NCAKnowledge::new();
    if std::path::Path::new(&brain_path).exists() {
        match store.load(&brain_path) {
            Ok(()) => eprintln!("🧠 Loaded brain: {} alive cells, {} entries",
                store.grid.alive_count(), store.text_store.len()),
            Err(e) => { eprintln!("❌ Failed to load brain: {}", e); return; }
        }
    } else {
        eprintln!("❌ No brain found at {}", brain_path);
        eprintln!("   Run sage-learn first to build the brain.");
        return;
    }

    // Load trained weights (or use random if not trained)
    let weights_path = BrainNcaWeights::default_path();
    let weights = if std::path::Path::new(&weights_path).exists() {
        match BrainNcaWeights::load(&weights_path) {
            Ok(w) => { eprintln!("⚙️  Loaded trained weights ({} params)", w.param_count()); w }
            Err(e) => { eprintln!("⚠️  Failed to load weights: {} — using random", e); BrainNcaWeights::random() }
        }
    } else {
        eprintln!("⚠️  No trained weights — using random (run sage-brain-train first)");
        BrainNcaWeights::random()
    };

    let config = EncoderConfig::default();
    let tests = build_test_sentences(&corpus_dir);
    eprintln!("📝 {} test sentences", tests.len());

    // Run prediction test
    eprintln!("🔮 Running prediction test ({} NCA steps)...", NCA_STEPS);
    let mut total_sim = 0.0f64;
    let mut top5_pos = 0;
    let mut top20_pos = 0;
    let mut top1_sim = 0; // top cell's embedding is closest to target
    let mut top5_sim = 0; // target is in top-5 by embedding similarity

    for (prefix, target) in &tests {
        let mut grid = Grid::new(GRID_SIZE, GRID_SIZE);
        let pf = encode_text(prefix, &config);
        sage::distributed_knowledge::encoder::write_knowledge(&mut grid, &pf, 0.8, 0.0, &config);
        for _ in 0..NCA_STEPS { nca_brain_step_weighted(&mut grid, &weights); }

        let tf = encode_text(target, &config);
        let (tx, ty) = feature_to_position(&tf, GRID_SIZE, GRID_SIZE);

        // Collect active cells with their embeddings
        let mut active: Vec<(usize, usize, f64, Vec<f64>)> = Vec::new();
        for y in 0..GRID_SIZE {
            for x in 0..GRID_SIZE {
                let act = grid.cells[y][x][KNOWLEDGE_ACTIVATION];
                if act > 0.001 {
                    let emb: Vec<f64> = (0..48).map(|i| grid.cells[y][x][KNOWLEDGE_CHANNELS_START + i]).collect();
                    active.push((x, y, act, emb));
                }
            }
        }
        active.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));

        if active.is_empty() { continue; }

        // Embedding similarity: top cell's embedding vs target
        let sim = cosine_sim(&active[0].3, &tf.values);
        total_sim += sim;

        // Check if top cell's embedding is the most similar to target
        let mut best_sim = sim;
        let mut best_idx = 0;
        for (i, (_, _, _, emb)) in active.iter().enumerate() {
            let s = cosine_sim(emb, &tf.values);
            if s > best_sim { best_sim = s; best_idx = i; }
        }
        if best_idx == 0 { top1_sim += 1; }
        if best_idx < 5 { top5_sim += 1; }

        // Positional accuracy
        for (i, (x, y, _, _)) in active.iter().enumerate() {
            if *x == tx && *y == ty {
                if i < 5 { top5_pos += 1; }
                if i < 20 { top20_pos += 1; }
                break;
            }
        }
    }

    let n = tests.len() as f64;
    let mean_sim = total_sim / n;
    let random_sim = 0.0; // random embeddings are orthogonal
    let random_top5 = 5.0 / (GRID_SIZE * GRID_SIZE) as f64;
    let random_top20 = 20.0 / (GRID_SIZE * GRID_SIZE) as f64;

    println!();
    println!("═══ SAGE Prediction Test ═══");
    println!();
    println!("Brain: {} alive cells, {} entries", store.grid.alive_count(), store.text_store.len());
    println!("Weights: {} params, {}", weights.param_count(),
        if std::path::Path::new(&weights_path).exists() { "trained" } else { "random" });
    println!("Tests: {} sentences, {} NCA steps", tests.len(), NCA_STEPS);
    println!();
    println!("─── Embedding Similarity (primary metric) ───");
    println!("Mean cosine sim:  {:.4} — random: ~{:.4}", mean_sim, random_sim);
    println!("Top-1 best sim:   {:.1}% ({}/{})", top1_sim as f64 / n * 100.0, top1_sim, tests.len());
    println!("Top-5 best sim:   {:.1}% ({}/{})", top5_sim as f64 / n * 100.0, top5_sim, tests.len());
    println!();
    println!("─── Positional Accuracy (secondary) ───");
    println!("Top-5 pos:        {:.1}% ({}/{}) — random: {:.3}%",
        top5_pos as f64 / n * 100.0, top5_pos, tests.len(), random_top5 * 100.0);
    println!("Top-20 pos:       {:.1}% ({}/{}) — random: {:.3}%",
        top20_pos as f64 / n * 100.0, top20_pos, tests.len(), random_top20 * 100.0);
    println!();

    // Verdict
    if mean_sim > 0.05 {
        println!("🧠 SIGNAL DETECTED — the brain is predicting above random!");
        println!("   Mean embedding similarity {:.4} > random ~0.0", mean_sim);
    } else if mean_sim > 0.01 {
        println!("📈 Weak signal — above random but needs more training.");
    } else {
        println!("📭 No prediction signal yet.");
        println!("   Run sage-brain-train to train the NCA weights.");
    }
}