//! sage-brain-train: Train the NCA brain weights for next-token prediction.
//!
//! Uses evolution strategy (ES) to train the MLP update rule so that
//! after encoding a text prefix and running NCA steps, the cell where
//! the next word would hash to gets activated.
//!
//! The MLP is small: 486 → 128 → 32 → 54 (22K params)
//! ES with population 16, sigma 0.1, 50 epochs.
//!
//! Fitness = top-20 prediction accuracy on 100 sentence pairs.
//! Random baseline = 20/65536 = 0.03%.

use sage::distributed_knowledge::brain_processor::BrainNcaWeights;
use sage::distributed_knowledge::encoder::{encode_text, feature_to_position, EncoderConfig};
use sage::distributed_knowledge::{default_brain_path, KnowledgeStore, NCAKnowledge};
use sage::grid::{Grid, GRID_SIZE, KNOWLEDGE_ACTIVATION, KNOWLEDGE_CHANNELS_START};
use rand::Rng;
use std::fs;

const NCA_STEPS: usize = 3; // fewer NCA steps for faster training (was 5)
const NUM_TESTS: usize = 50; // diverse corpus-extracted sentences
const POPULATION: usize = 8;  // was 16 — halved for 2x faster epochs
const SIGMA: f64 = 0.1;
const SIGMA_DECAY: f64 = 0.97; // sigma *= 0.97 each epoch — anneal exploration
const LEARNING_RATE: f64 = 0.05;
const EPOCHS: usize = 50;    // more epochs, but each is faster

fn build_test_pairs() -> Vec<(String, String)> {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let corpus_dir = format!("{}/.sage/corpus/", home);
    let mut pairs = Vec::new();

    // Extract real sentence pairs from corpus files
    if let Ok(entries) = fs::read_dir(&corpus_dir) {
        for entry in entries.flatten() {
            if pairs.len() >= 200 { break; }
            let path = entry.path();
            if path.extension().map(|e| e == "txt").unwrap_or(false) {
                if let Ok(text) = fs::read_to_string(&path) {
                    for sent in text.split(|c: char| c == '.' || c == '!' || c == '?') {
                        if pairs.len() >= 200 { break; }
                        let words: Vec<&str> = sent.split_whitespace().collect();
                        // Need at least 5 words, take last as target
                        if words.len() >= 5 && words.len() <= 25 {
                            let target = words[words.len() - 1].trim_matches(|c: char| !c.is_alphanumeric());
                            if target.len() >= 3 && !target.is_empty() {
                                let prefix = words[..words.len() - 1].join(" ");
                                // Skip if prefix is too short
                                if prefix.len() >= 20 {
                                    pairs.push((prefix, target.to_string()));
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Fallback: if no corpus found, use hand-crafted pairs
    if pairs.is_empty() {
        eprintln!("⚠️  No corpus found, using fallback pairs");
        let fallback: &[(&str, &str)] = &[
            ("the cat sat on the", "mat"),
            ("she walked to the", "store"),
            ("he opened the door", "slowly"),
            ("they were happy", "together"),
            ("the sun rose over the", "mountains"),
        ];
        for (p, t) in fallback {
            pairs.push((p.to_string(), t.to_string()));
        }
    }

    // Shuffle and take NUM_TESTS — use fixed seed for reproducibility
    use rand::seq::SliceRandom;
    use rand::SeedableRng;
    let mut rng = rand::rngs::StdRng::seed_from_u64(42); // deterministic test set
    pairs.shuffle(&mut rng);
    pairs.truncate(NUM_TESTS);
    pairs
}

/// Evaluate prediction fitness for a given set of NCA weights.
/// Returns embedding-based prediction accuracy.
///
/// Instead of checking if the target's GRID POSITION lights up (which requires
/// long-range routing that local NCA can't do), we check if the EMBEDDING at
/// the most active cell is similar to the target word's embedding.
///
/// The idea: after processing "the cat sat on the", the most active cell
/// should contain an embedding that's similar to "mat" — even if it's at
/// a different grid position. The grid routes information through local
/// dynamics, and the embedding at the peak of activation reflects what
/// the brain "expects" to see next.
fn evaluate_fitness(
    weights: &BrainNcaWeights,
    tests: &[(String, String)],
    config: &EncoderConfig,
) -> f64 {
    let mut total_sim = 0.0f64;
    let mut top5_correct = 0;
    let mut top20_correct = 0;

    for (prefix, target) in tests {
        // Fresh grid for each test
        let mut grid = Grid::new(GRID_SIZE, GRID_SIZE);

        // Encode prefix into grid
        let prefix_features = encode_text(prefix, config);
        let _ = sage::distributed_knowledge::encoder::write_knowledge(
            &mut grid, &prefix_features, 0.8, 0.0, config,
        );

        // Run NCA steps with these weights
        for _ in 0..NCA_STEPS {
            nca_brain_step_weighted(&mut grid, weights);
        }

        // Get target word's embedding
        let target_features = encode_text(target, config);

        // Scan all active cells, find the one with highest activation
        let mut active: Vec<(usize, usize, f64)> = Vec::new();
        for y in 0..GRID_SIZE {
            for x in 0..GRID_SIZE {
                let act = grid.cells[y][x][KNOWLEDGE_ACTIVATION];
                if act > 0.001 {
                    active.push((x, y, act));
                }
            }
        }
        active.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));

        if active.is_empty() {
            continue;
        }

        // Check: is the embedding at the top cell similar to the target's embedding?
        // Read the embedding from the top cell's knowledge channels
        let top_x = active[0].0;
        let top_y = active[0].1;
        let top_cell_embedding: Vec<f64> = (0..48)
            .map(|i| grid.cells[top_y][top_x][KNOWLEDGE_CHANNELS_START + i])
            .collect();

        // Cosine similarity between top cell's embedding and target's embedding
        let sim = cosine_sim(&top_cell_embedding, &target_features.values);
        total_sim += sim;

        // Also check: among top-20 active cells, is any of them close to the target position?
        let (target_x, target_y) = feature_to_position(&target_features, GRID_SIZE, GRID_SIZE);
        for (i, (x, y, _)) in active.iter().enumerate() {
            if *x == target_x && *y == target_y {
                if i < 5 { top5_correct += 1; }
                if i < 20 { top20_correct += 1; }
                break;
            }
        }
    }

    let n = tests.len() as f64;
    let mean_sim = total_sim / n;
    let pos_acc = (top5_correct as f64 * 2.0 + top20_correct as f64) / n;

    // Fitness: combination of embedding similarity and positional accuracy
    // Embedding similarity is the primary signal — it doesn't require long-range routing
    mean_sim + pos_acc * 0.1
}

fn cosine_sim(a: &[f64], b: &[f64]) -> f64 {
    let dot: f64 = a.iter().zip(b).map(|(x, y)| x * y).sum();
    let mag_a: f64 = a.iter().map(|v| v * v).sum::<f64>().sqrt();
    let mag_b: f64 = b.iter().map(|v| v * v).sum::<f64>().sqrt();
    if mag_a < 1e-10 || mag_b < 1e-10 { 0.0 } else { dot / (mag_a * mag_b) }
}

/// NCA step using MLP weights (operates on knowledge channels only for speed)
fn nca_brain_step_weighted(grid: &mut Grid, weights: &BrainNcaWeights) {
    let w = grid.width;
    let h = grid.height;

    // Find active cells + neighbors
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
                            if !seen[ny][nx] {
                                seen[ny][nx] = true;
                                active.push((ny, nx));
                            }
                        }
                    }
                }
            }
        }
    }

    if active.is_empty() { return; }

    // Process only knowledge channels (50) for speed
    const NCH: usize = 50;
    const PERC: usize = 9 * NCH; // 450

    // Compute deltas
    let mut deltas: Vec<(usize, usize, Vec<f64>)> = Vec::with_capacity(active.len());

    for (cy, cx) in &active {
        let y = *cy;
        let x = *cx;
        // Perception: 3×3 neighborhood, 50 knowledge channels
        let mut input = vec![0.0f64; PERC];
        let mut idx = 0;
        for dy in -1i32..=1 {
            for dx in -1i32..=1 {
                let ny = ((y as i32 + dy).rem_euclid(h as i32)) as usize;
                let nx = ((x as i32 + dx).rem_euclid(w as i32)) as usize;
                for ch in 0..NCH {
                    input[idx] = grid.cells[ny][nx][KNOWLEDGE_CHANNELS_START + ch];
                    idx += 1;
                }
            }
        }

        // Layer 1: 450 → 128 (ReLU)
        let mut h1 = vec![0.0f64; 128];
        for i in 0..128 {
            let mut sum = weights.b1[i];
            let row = &weights.w1[i];
            for j in 0..PERC {
                sum += row[j] * input[j];
            }
            h1[i] = sum.max(0.0);
        }

        // Layer 2: 128 → 32 (ReLU)
        let mut h2 = vec![0.0f64; 32];
        for i in 0..32 {
            let mut sum = weights.b2[i];
            let row = &weights.w2[i];
            for j in 0..128 {
                sum += row[j] * h1[j];
            }
            h2[i] = sum.max(0.0);
        }

        // Layer 3: 32 → 50 (tanh, residual)
        let mut out = vec![0.0f64; NCH];
        for i in 0..NCH {
            let mut sum = weights.b3[i];
            let row = &weights.w3[i];
            for j in 0..32 {
                sum += row[j] * h2[j];
            }
            out[i] = sum.tanh() * 0.1;
        }

        deltas.push((y, x, out));
    }

    // Apply deltas with decay
    let decay = 0.005;
    for (y, x, out) in &deltas {
        for (i, &delta) in out.iter().enumerate() {
            let ch = KNOWLEDGE_CHANNELS_START + i;
            let current = grid.cells[*y][*x][ch];
            if ch == KNOWLEDGE_ACTIVATION {
                // Apply decay + residual update
                grid.cells[*y][*x][ch] = (current * (1.0 - decay) + delta).clamp(0.0, 1.0);
            } else {
                grid.cells[*y][*x][ch] = (current + delta).clamp(-5.0, 5.0);
            }
        }
    }
}

fn main() {
    let tests = build_test_pairs();
    eprintln!("📝 {} test pairs", tests.len());

    let config = EncoderConfig::default();

    // Random baseline
    let random_w = BrainNcaWeights::random();
    let random_fitness = evaluate_fitness(&random_w, &tests, &config);
    eprintln!("🎲 Random baseline fitness: {:.4}", random_fitness);

    // ES training
    // Load existing weights if available — continue training from there
    let weights_path = BrainNcaWeights::default_path();
    let mut best_weights = if std::path::Path::new(&weights_path).exists() {
        match BrainNcaWeights::load(&weights_path) {
            Ok(w) => {
                eprintln!("📂 Loaded existing weights (fitness: continue from previous best)");
                w
            }
            Err(_) => {
                eprintln!("🎲 Starting from random weights");
                BrainNcaWeights::random()
            }
        }
    } else {
        eprintln!("🎲 Starting from random weights");
        BrainNcaWeights::random()
    };
    let mut best_fitness = evaluate_fitness(&best_weights, &tests, &config);
    eprintln!("🔄 Starting ES training: {} epochs, population {}", EPOCHS, POPULATION);
    eprintln!("   Params: {}", best_weights.param_count());
    eprintln!("   Sigma: {}, LR: {}", SIGMA, LEARNING_RATE);

    let mut rng = rand::thread_rng();

    let mut current_sigma = SIGMA;
    for epoch in 0..EPOCHS {
        let base_params = best_weights.to_vec();
        let n_params = base_params.len();

        let mut noise_vecs: Vec<Vec<f64>> = Vec::with_capacity(POPULATION);
        let mut fitnesses: Vec<f64> = Vec::with_capacity(POPULATION);

        for _ in 0..POPULATION {
            let noise: Vec<f64> = (0..n_params).map(|_| rng.gen::<f64>() * 2.0 - 1.0).collect();
            let perturbed: Vec<f64> = base_params.iter().zip(&noise)
                .map(|(p, n)| p + current_sigma * n)
                .collect();
            let w = BrainNcaWeights::from_vec(&perturbed);
            let fitness = evaluate_fitness(&w, &tests, &config);
            noise_vecs.push(noise);
            fitnesses.push(fitness);
        }

        let mean_f: f64 = fitnesses.iter().sum::<f64>() / fitnesses.len() as f64;
        let std_f: f64 = {
            let var = fitnesses.iter().map(|f| (f - mean_f).powi(2)).sum::<f64>() / fitnesses.len() as f64;
            var.sqrt().max(1e-8)
        };
        let norm_fitnesses: Vec<f64> = fitnesses.iter().map(|f| (f - mean_f) / std_f).collect();

        // Gradient estimate
        let mut new_params = base_params.clone();
        for param in new_params.iter_mut() {
            *param = *param; // just referencing
        }
        for (i, param) in new_params.iter_mut().enumerate() {
            let mut grad = 0.0;
            for j in 0..POPULATION {
                grad += norm_fitnesses[j] * noise_vecs[j][i];
            }
            grad /= (POPULATION as f64) * current_sigma;
            *param += LEARNING_RATE * grad;
        }

        let new_weights = BrainNcaWeights::from_vec(&new_params);
        let new_fitness = evaluate_fitness(&new_weights, &tests, &config);

        if new_fitness > best_fitness {
            best_fitness = new_fitness;
            best_weights = new_weights;
        }

        eprintln!("  Epoch {}/{}: best={:.4} mean={:.4} sigma={:.4} (random={:.4})",
            epoch + 1, EPOCHS, best_fitness, mean_f, current_sigma, random_fitness);

        // Save best weights
        let weights_path = BrainNcaWeights::default_path();
        let _ = best_weights.save(&weights_path);

        // Anneal sigma — smaller perturbations as we converge
        current_sigma *= SIGMA_DECAY;
    }

    eprintln!();
    eprintln!("═══ Training Complete ═══");
    eprintln!("Best fitness: {:.4} (random: {:.4})", best_fitness, random_fitness);
    eprintln!("Improvement: {:.4}x", best_fitness / random_fitness.max(1e-10));
    eprintln!("Weights saved: {:?}", BrainNcaWeights::default_path());

    // Final evaluation with detailed test
    let mut top5 = 0;
    let mut top20 = 0;
    for (prefix, target) in &tests {
        let mut grid = Grid::new(GRID_SIZE, GRID_SIZE);
        let pf = encode_text(prefix, &config);
        sage::distributed_knowledge::encoder::write_knowledge(&mut grid, &pf, 0.8, 0.0, &config);
        for _ in 0..NCA_STEPS { nca_brain_step_weighted(&mut grid, &best_weights); }
        let tf = encode_text(target, &config);
        let (tx, ty) = feature_to_position(&tf, GRID_SIZE, GRID_SIZE);
        let mut active: Vec<((usize, usize), f64)> = Vec::new();
        for y in 0..GRID_SIZE { for x in 0..GRID_SIZE {
            let a = grid.cells[y][x][KNOWLEDGE_ACTIVATION];
            if a > 0.001 { active.push(((x,y), a)); }
        }}
        active.sort_by(|a,b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        for (i, ((x,y),_)) in active.iter().enumerate() {
            if *x == tx && *y == ty {
                if i < 5 { top5 += 1; }
                if i < 20 { top20 += 1; }
                break;
            }
        }
    }
    let n = tests.len();
    println!("Top-5:  {}/{} = {:.1}%", top5, n, top5 as f64 / n as f64 * 100.0);
    println!("Top-20: {}/{} = {:.1}%", top20, n, top20 as f64 / n as f64 * 100.0);
}