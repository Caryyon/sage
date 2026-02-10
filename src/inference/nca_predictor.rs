//! NCA-based Token Prediction Engine
//!
//! Research module: Can Neural Cellular Automata predict next tokens?
//!
//! Core idea:
//! - Map each token in a vocabulary to a cell position on a 2D grid
//! - To encode a sequence, activate the cells for input tokens
//! - Run NCA update steps; cells communicate via local rules
//! - After N steps, read activation levels → highest = predicted next token
//! - Train the NCA update rule to improve predictions
//!
//! This is Phase 1 research code. Signal above random = success.

use rand::Rng;
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::PathBuf;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Grid side length for the token prediction grid (separate from main SAGE grid)
const NCA_GRID_SIZE: usize = 181; // 181*181 = 32761 cells ≥ 32K vocab
/// Smaller grid for training (16×16 = 256 cells, sufficient for demo vocab)
const TRAINING_GRID_SIZE: usize = 8;
pub const NCA_CHANNELS: usize = 8; // Per-cell channels: [activation, embedding x4, hidden x3]
const ACTIVATION_CH: usize = 0;

/// Default NCA update steps per prediction
const DEFAULT_STEPS: usize = 20;

// ---------------------------------------------------------------------------
// Tokenizer (simple word-level with BPE-like fallback)
// ---------------------------------------------------------------------------

/// Minimal tokenizer: learns vocab from corpus, maps tokens ↔ ids
#[derive(Clone)]
pub struct SimpleTokenizer {
    pub token_to_id: HashMap<String, usize>,
    pub id_to_token: Vec<String>,
}

impl SimpleTokenizer {
    /// Build vocabulary from text (whitespace + punctuation split, capped at max_vocab)
    pub fn from_corpus(text: &str, max_vocab: usize) -> Self {
        let mut freq: HashMap<String, usize> = HashMap::new();
        for tok in Self::tokenize_raw(text) {
            *freq.entry(tok).or_insert(0) += 1;
        }
        let mut pairs: Vec<_> = freq.into_iter().collect();
        pairs.sort_by(|a, b| b.1.cmp(&a.1));
        pairs.truncate(max_vocab - 1); // reserve 0 for <unk>

        let mut id_to_token = vec!["<unk>".to_string()];
        let mut token_to_id = HashMap::new();
        token_to_id.insert("<unk>".to_string(), 0);
        for (i, (tok, _)) in pairs.into_iter().enumerate() {
            let id = i + 1;
            token_to_id.insert(tok.clone(), id);
            id_to_token.push(tok);
        }
        Self { token_to_id, id_to_token }
    }

    pub fn encode(&self, text: &str) -> Vec<usize> {
        Self::tokenize_raw(text)
            .into_iter()
            .map(|t| *self.token_to_id.get(&t).unwrap_or(&0))
            .collect()
    }

    pub fn decode(&self, ids: &[usize]) -> String {
        ids.iter()
            .map(|&id| {
                if id < self.id_to_token.len() {
                    self.id_to_token[id].as_str()
                } else {
                    "<unk>"
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    pub fn vocab_size(&self) -> usize {
        self.id_to_token.len()
    }

    fn tokenize_raw(text: &str) -> Vec<String> {
        // Simple split: lowercase, split on whitespace, separate punctuation
        let text = text.to_lowercase();
        let mut tokens = Vec::new();
        for word in text.split_whitespace() {
            let word = word.trim();
            if word.is_empty() { continue; }
            // Split trailing punctuation
            let trimmed = word.trim_end_matches(|c: char| c.is_ascii_punctuation());
            let suffix = &word[trimmed.len()..];
            if !trimmed.is_empty() {
                tokens.push(trimmed.to_string());
            }
            if !suffix.is_empty() {
                tokens.push(suffix.to_string());
            }
        }
        tokens
    }
}

// ---------------------------------------------------------------------------
// NCA Grid for Token Prediction
// ---------------------------------------------------------------------------

/// Maps token ids to (row, col) on a grid of given size
fn token_to_coord(token_id: usize, grid_size: usize) -> (usize, usize) {
    let row = token_id / grid_size;
    let col = token_id % grid_size;
    (row.min(grid_size - 1), col.min(grid_size - 1))
}

/// The NCA update rule weights (small MLP: perception → hidden → output)
#[derive(Clone)]
pub struct NcaWeights {
    // Perception: 3x3 neighborhood × NCA_CHANNELS = 9*8 = 72 inputs
    // Hidden: 64 neurons
    // Output: NCA_CHANNELS
    pub w1: Vec<Vec<f64>>, // hidden_size × input_size
    pub b1: Vec<f64>,      // hidden_size
    pub w2: Vec<Vec<f64>>, // NCA_CHANNELS × hidden_size
    pub b2: Vec<f64>,      // NCA_CHANNELS
}

const HIDDEN_SIZE: usize = 64;
const PERCEPTION_SIZE: usize = 9 * NCA_CHANNELS; // 3x3 neighborhood

impl NcaWeights {
    pub fn random() -> Self {
        let mut rng = rand::thread_rng();
        let scale1 = (2.0 / PERCEPTION_SIZE as f64).sqrt();
        let scale2 = (2.0 / HIDDEN_SIZE as f64).sqrt();
        Self {
            w1: (0..HIDDEN_SIZE)
                .map(|_| (0..PERCEPTION_SIZE).map(|_| rng.gen_range(-scale1..scale1)).collect())
                .collect(),
            b1: vec![0.0; HIDDEN_SIZE],
            w2: (0..NCA_CHANNELS)
                .map(|_| (0..HIDDEN_SIZE).map(|_| rng.gen_range(-scale2..scale2)).collect())
                .collect(),
            b2: vec![0.0; NCA_CHANNELS],
        }
    }

    /// Number of total parameters
    pub fn param_count(&self) -> usize {
        HIDDEN_SIZE * PERCEPTION_SIZE + HIDDEN_SIZE + NCA_CHANNELS * HIDDEN_SIZE + NCA_CHANNELS
    }

    /// Flatten all params into a vec (for evolution strategy)
    pub fn to_vec(&self) -> Vec<f64> {
        let mut v = Vec::with_capacity(self.param_count());
        for row in &self.w1 { v.extend(row); }
        v.extend(&self.b1);
        for row in &self.w2 { v.extend(row); }
        v.extend(&self.b2);
        v
    }

    /// Load from flat vec
    pub fn from_vec(params: &[f64]) -> Self {
        let mut idx = 0;
        let mut w1 = Vec::with_capacity(HIDDEN_SIZE);
        for _ in 0..HIDDEN_SIZE {
            w1.push(params[idx..idx + PERCEPTION_SIZE].to_vec());
            idx += PERCEPTION_SIZE;
        }
        let b1 = params[idx..idx + HIDDEN_SIZE].to_vec();
        idx += HIDDEN_SIZE;
        let mut w2 = Vec::with_capacity(NCA_CHANNELS);
        for _ in 0..NCA_CHANNELS {
            w2.push(params[idx..idx + HIDDEN_SIZE].to_vec());
            idx += HIDDEN_SIZE;
        }
        let b2 = params[idx..idx + NCA_CHANNELS].to_vec();
        Self { w1, b1, w2, b2 }
    }

    /// Save weights to binary file
    pub fn save(&self, path: &std::path::Path) -> Result<(), Box<dyn Error>> {
        let data = self.to_vec();
        let bytes: Vec<u8> = data.iter().flat_map(|f| f.to_le_bytes()).collect();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, bytes)?;
        Ok(())
    }

    /// Load weights from binary file
    pub fn load(path: &std::path::Path) -> Result<Self, Box<dyn Error>> {
        let bytes = fs::read(path)?;
        let data: Vec<f64> = bytes
            .chunks_exact(8)
            .map(|chunk| f64::from_le_bytes(chunk.try_into().unwrap()))
            .collect();
        Ok(Self::from_vec(&data))
    }
}

/// The NCA token prediction grid
pub struct NcaPredictor {
    grid: Vec<Vec<[f64; NCA_CHANNELS]>>,
    weights: NcaWeights,
    steps: usize,
    grid_size: usize,
    pub tokenizer: SimpleTokenizer,
}

impl NcaPredictor {
    pub fn new(tokenizer: SimpleTokenizer, weights: NcaWeights, steps: usize) -> Self {
        Self::with_grid_size(tokenizer, weights, steps, NCA_GRID_SIZE)
    }

    pub fn with_grid_size(tokenizer: SimpleTokenizer, weights: NcaWeights, steps: usize, grid_size: usize) -> Self {
        let grid = vec![vec![[0.0; NCA_CHANNELS]; grid_size]; grid_size];
        Self { grid, weights, steps, grid_size, tokenizer }
    }

    pub fn with_default_steps(tokenizer: SimpleTokenizer, weights: NcaWeights) -> Self {
        Self::new(tokenizer, weights, DEFAULT_STEPS)
    }

    /// Reset the grid to zeros
    fn clear_grid(&mut self) {
        for row in &mut self.grid {
            for cell in row.iter_mut() {
                *cell = [0.0; NCA_CHANNELS];
            }
        }
    }

    /// Activate grid cells for the given token ids
    fn activate_tokens(&mut self, token_ids: &[usize]) {
        for (pos, &tid) in token_ids.iter().enumerate() {
            let (r, c) = token_to_coord(tid, self.grid_size);
            // Set activation to 1.0 and encode position in channels
            self.grid[r][c][ACTIVATION_CH] = 1.0;
            // Encode positional info (normalized position in sequence)
            let pos_norm = if token_ids.len() > 1 {
                pos as f64 / (token_ids.len() - 1) as f64
            } else {
                1.0
            };
            self.grid[r][c][1] = pos_norm;
            // Recency: later tokens have higher values
            self.grid[r][c][2] = (pos + 1) as f64 / token_ids.len() as f64;
        }
    }

    /// Get 3x3 neighborhood perception for cell (r,c)
    fn perceive(&self, r: usize, c: usize) -> [f64; PERCEPTION_SIZE] {
        let mut input = [0.0; PERCEPTION_SIZE];
        let mut idx = 0;
        for dr in [-1i32, 0, 1] {
            for dc in [-1i32, 0, 1] {
                let nr = (r as i32 + dr).rem_euclid(self.grid_size as i32) as usize;
                let nc = (c as i32 + dc).rem_euclid(self.grid_size as i32) as usize;
                for ch in 0..NCA_CHANNELS {
                    input[idx] = self.grid[nr][nc][ch];
                    idx += 1;
                }
            }
        }
        input
    }

    /// Apply the NCA update rule (one step)
    fn nca_step(&mut self) {
        let mut deltas = vec![vec![[0.0; NCA_CHANNELS]; self.grid_size]; self.grid_size];
        
        for r in 0..self.grid_size {
            for c in 0..self.grid_size {
                let input = self.perceive(r, c);
                // MLP forward pass: input → hidden (ReLU) → output (tanh)
                let mut hidden = vec![0.0; HIDDEN_SIZE];
                for h in 0..HIDDEN_SIZE {
                    let mut sum = self.weights.b1[h];
                    for i in 0..PERCEPTION_SIZE {
                        sum += self.weights.w1[h][i] * input[i];
                    }
                    hidden[h] = sum.max(0.0); // ReLU
                }
                for ch in 0..NCA_CHANNELS {
                    let mut sum = self.weights.b2[ch];
                    for h in 0..HIDDEN_SIZE {
                        sum += self.weights.w2[ch][h] * hidden[h];
                    }
                    deltas[r][c][ch] = sum.tanh() * 0.1; // Small residual update
                }
            }
        }

        // Apply deltas
        for r in 0..self.grid_size {
            for c in 0..self.grid_size {
                for ch in 0..NCA_CHANNELS {
                    self.grid[r][c][ch] = (self.grid[r][c][ch] + deltas[r][c][ch]).clamp(-5.0, 5.0);
                }
            }
        }
    }

    /// Run NCA steps and return activation levels for all vocab tokens
    pub fn run_and_read(&mut self, input_tokens: &[usize]) -> Vec<f64> {
        self.clear_grid();
        self.activate_tokens(input_tokens);

        for _ in 0..self.steps {
            self.nca_step();
        }

        // Read activation channel for each token position
        let vocab_size = self.tokenizer.vocab_size();
        let mut activations = vec![0.0; vocab_size];
        for tid in 0..vocab_size {
            let (r, c) = token_to_coord(tid, self.grid_size);
            activations[tid] = self.grid[r][c][ACTIVATION_CH];
        }
        activations
    }

    /// Predict next token given input tokens
    pub fn predict_next(&mut self, input_tokens: &[usize]) -> usize {
        let activations = self.run_and_read(input_tokens);
        // Exclude input tokens from prediction to avoid echo
        let mut best_id = 0;
        let mut best_val = f64::NEG_INFINITY;
        for (id, &val) in activations.iter().enumerate() {
            if !input_tokens.contains(&id) && val > best_val {
                best_val = val;
                best_id = id;
            }
        }
        best_id
    }

    /// Predict with temperature sampling
    pub fn predict_next_sampled(&mut self, input_tokens: &[usize], temperature: f64) -> usize {
        let activations = self.run_and_read(input_tokens);
        let mut rng = rand::thread_rng();

        // Softmax with temperature
        let max_val = activations.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let exps: Vec<f64> = activations
            .iter()
            .map(|&a| ((a - max_val) / temperature.max(0.01)).exp())
            .collect();
        let sum: f64 = exps.iter().sum();
        let probs: Vec<f64> = exps.iter().map(|e| e / sum).collect();

        // Sample
        let r: f64 = rng.gen();
        let mut cumulative = 0.0;
        for (id, &p) in probs.iter().enumerate() {
            cumulative += p;
            if r <= cumulative {
                return id;
            }
        }
        probs.len() - 1
    }

    /// Run NCA steps on input tokens and return the full grid state.
    /// Used by reservoir computing to extract features from NCA dynamics.
    pub fn run_and_get_state(&mut self, input_tokens: &[usize]) -> Vec<Vec<[f64; NCA_CHANNELS]>> {
        self.clear_grid();
        self.activate_tokens(input_tokens);
        for _ in 0..self.steps {
            self.nca_step();
        }
        self.grid.clone()
    }

    /// Get the grid size
    pub fn grid_size(&self) -> usize {
        self.grid_size
    }

    /// Get the weights (for training)
    pub fn weights(&self) -> &NcaWeights {
        &self.weights
    }

    /// Set weights
    pub fn set_weights(&mut self, w: NcaWeights) {
        self.weights = w;
    }
}

// ---------------------------------------------------------------------------
// Training via Evolution Strategy (ES)
// ---------------------------------------------------------------------------

/// Training configuration
pub struct TrainingConfig {
    pub population_size: usize,
    pub sigma: f64,         // Noise standard deviation
    pub learning_rate: f64,
    pub epochs: usize,
    pub context_window: usize, // How many tokens of context
    pub grid_size: usize,      // Grid side length for training (smaller = faster)
    pub nca_steps: usize,      // NCA update steps per evaluation during training
    pub max_examples: usize,   // Max training examples to subsample
}

impl Default for TrainingConfig {
    fn default() -> Self {
        Self {
            population_size: 10,
            sigma: 0.02,
            learning_rate: 0.001,
            epochs: 30,
            context_window: 5,
            grid_size: TRAINING_GRID_SIZE,
            nca_steps: 3,
            max_examples: 30,
        }
    }
}

/// Train the NCA predictor on a text corpus using evolution strategy.
/// Returns (trained_predictor, final_accuracy, random_baseline_accuracy)
pub fn train_nca(
    corpus: &str,
    config: &TrainingConfig,
    verbose: bool,
) -> Result<(NcaPredictor, f64, f64), Box<dyn Error>> {
    let grid_size = config.grid_size;
    let tokenizer = SimpleTokenizer::from_corpus(corpus, grid_size * grid_size);
    let tokens = tokenizer.encode(corpus);
    let vocab_size = tokenizer.vocab_size();

    if tokens.len() < config.context_window + 1 {
        return Err("Corpus too small for training".into());
    }

    // Random baseline: 1/vocab_size
    let random_accuracy = 1.0 / vocab_size as f64;

    if verbose {
        eprintln!("📊 Vocab size: {}, Corpus tokens: {}, Grid: {}×{}, Random baseline: {:.4}%",
                  vocab_size, tokens.len(), grid_size, grid_size, random_accuracy * 100.0);
        eprintln!("📊 NCA params: {}", NcaWeights::random().param_count());
    }

    let mut best_weights = NcaWeights::random();
    let mut best_fitness = f64::NEG_INFINITY;

    let mut rng = rand::thread_rng();

    // Build training examples (subsample for speed)
    let max_examples = config.max_examples.min(tokens.len() - config.context_window);
    let step = ((tokens.len() - config.context_window) / max_examples).max(1);
    let examples: Vec<(Vec<usize>, usize)> = (0..tokens.len() - config.context_window)
        .step_by(step)
        .take(max_examples)
        .map(|i| {
            let ctx = tokens[i..i + config.context_window].to_vec();
            let target = tokens[i + config.context_window];
            (ctx, target)
        })
        .collect();

    if verbose {
        eprintln!("📊 Training examples: {}", examples.len());
    }

    for epoch in 0..config.epochs {
        let base_params = best_weights.to_vec();
        let n_params = base_params.len();

        // Generate perturbations and evaluate
        let mut noise_vecs: Vec<Vec<f64>> = Vec::with_capacity(config.population_size);
        let mut fitnesses: Vec<f64> = Vec::with_capacity(config.population_size);

        for _ in 0..config.population_size {
            let noise: Vec<f64> = (0..n_params).map(|_| rng.gen::<f64>() * 2.0 - 1.0).collect();
            let perturbed: Vec<f64> = base_params.iter()
                .zip(&noise)
                .map(|(p, n)| p + config.sigma * n)
                .collect();

            let w = NcaWeights::from_vec(&perturbed);
            let fitness = evaluate_fitness(&tokenizer, &w, &examples, grid_size, config.nca_steps);
            noise_vecs.push(noise);
            fitnesses.push(fitness);
        }

        // Normalize fitnesses
        let mean_f: f64 = fitnesses.iter().sum::<f64>() / fitnesses.len() as f64;
        let std_f: f64 = {
            let var = fitnesses.iter().map(|f| (f - mean_f).powi(2)).sum::<f64>() / fitnesses.len() as f64;
            var.sqrt().max(1e-8)
        };
        let norm_fitnesses: Vec<f64> = fitnesses.iter().map(|f| (f - mean_f) / std_f).collect();

        // Update weights: gradient estimate
        let mut new_params = base_params.clone();
        for i in 0..n_params {
            let mut grad = 0.0;
            for j in 0..config.population_size {
                grad += norm_fitnesses[j] * noise_vecs[j][i];
            }
            grad /= (config.population_size as f64) * config.sigma;
            new_params[i] += config.learning_rate * grad;
        }

        let new_weights = NcaWeights::from_vec(&new_params);
        let new_fitness = evaluate_fitness(&tokenizer, &new_weights, &examples, grid_size, config.nca_steps);

        if new_fitness > best_fitness {
            best_fitness = new_fitness;
            best_weights = new_weights;
        }

        if verbose {
            eprintln!("  Epoch {}/{}: accuracy = {:.4}% (random = {:.4}%)",
                      epoch + 1, config.epochs, best_fitness * 100.0, random_accuracy * 100.0);
        }
    }

    let predictor = NcaPredictor::with_grid_size(tokenizer, best_weights, DEFAULT_STEPS, grid_size);
    Ok((predictor, best_fitness, random_accuracy))
}

/// Evaluate accuracy on examples
fn evaluate_fitness(
    tokenizer: &SimpleTokenizer,
    weights: &NcaWeights,
    examples: &[(Vec<usize>, usize)],
    grid_size: usize,
    nca_steps: usize,
) -> f64 {
    let mut predictor = NcaPredictor::with_grid_size(tokenizer.clone(), weights.clone(), nca_steps, grid_size);
    let mut correct = 0;
    // Use top-5 accuracy for richer signal
    for (ctx, target) in examples {
        let activations = predictor.run_and_read(ctx);
        // Top-5 check
        let mut indexed: Vec<(usize, f64)> = activations.iter().enumerate().map(|(i, &v)| (i, v)).collect();
        indexed.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        if indexed.iter().take(5).any(|(id, _)| id == target) {
            correct += 1;
        }
    }
    correct as f64 / examples.len() as f64
}

// ---------------------------------------------------------------------------
// Hybrid mode: NCA + LLM blending
// ---------------------------------------------------------------------------

/// Configuration for hybrid NCA/LLM inference
#[derive(Clone, Debug)]
pub struct HybridConfig {
    /// 0.0 = pure LLM, 1.0 = pure NCA
    pub nca_weight: f64,
    /// Accuracy threshold above which NCA weight increases
    pub promotion_threshold: f64,
    /// How much to increase nca_weight when accuracy is good
    pub promotion_rate: f64,
}

impl Default for HybridConfig {
    fn default() -> Self {
        Self {
            nca_weight: 0.3,
            promotion_threshold: 0.5,
            promotion_rate: 0.05,
        }
    }
}

/// Tracks NCA accuracy over time for hybrid mode
pub struct HybridTracker {
    pub config: HybridConfig,
    pub predictions: usize,
    pub correct: usize,
}

impl HybridTracker {
    pub fn new(config: HybridConfig) -> Self {
        Self { config, predictions: 0, correct: 0 }
    }

    pub fn record(&mut self, nca_was_correct: bool) {
        self.predictions += 1;
        if nca_was_correct { self.correct += 1; }

        // Every 100 predictions, check if we should promote NCA
        if self.predictions.is_multiple_of(100) && self.predictions > 0 {
            let accuracy = self.correct as f64 / self.predictions as f64;
            if accuracy > self.config.promotion_threshold {
                self.config.nca_weight = (self.config.nca_weight + self.config.promotion_rate).min(1.0);
                eprintln!("🧠 NCA accuracy {:.1}% > threshold, nca_weight → {:.2}",
                         accuracy * 100.0, self.config.nca_weight);
            }
        }
    }

    pub fn current_accuracy(&self) -> f64 {
        if self.predictions == 0 { 0.0 } else { self.correct as f64 / self.predictions as f64 }
    }
}

// ---------------------------------------------------------------------------
// Convenience: default weights path
// ---------------------------------------------------------------------------

pub fn default_weights_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".sage")
        .join("nca_weights.bin")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenizer_roundtrip() {
        let corpus = "the cat sat on the mat the dog sat on the log";
        let tok = SimpleTokenizer::from_corpus(corpus, 1000);
        let ids = tok.encode("the cat sat");
        assert!(ids.len() == 3);
        assert!(ids[0] != 0); // "the" should be in vocab
    }

    #[test]
    fn test_token_coord_mapping() {
        for id in [0, 1, 100, 1000, NCA_GRID_SIZE * NCA_GRID_SIZE - 1] {
            let (r, c) = token_to_coord(id, NCA_GRID_SIZE);
            assert!(r < NCA_GRID_SIZE);
            assert!(c < NCA_GRID_SIZE);
        }
    }

    #[test]
    fn test_nca_predict_deterministic() {
        let corpus = "hello world hello world hello world";
        let tok = SimpleTokenizer::from_corpus(corpus, 1000);
        let w = NcaWeights::random();
        let mut pred = NcaPredictor::with_default_steps(tok, w);
        let ids = pred.tokenizer.encode("hello");
        let result1 = pred.predict_next(&ids);
        let result2 = pred.predict_next(&ids);
        // Same weights + same input → same output (grid is reset each time)
        assert_eq!(result1, result2);
    }

    #[test]
    fn test_weights_save_load() {
        let w = NcaWeights::random();
        let v1 = w.to_vec();
        let w2 = NcaWeights::from_vec(&v1);
        let v2 = w2.to_vec();
        assert_eq!(v1.len(), v2.len());
        for (a, b) in v1.iter().zip(v2.iter()) {
            assert!((a - b).abs() < 1e-15);
        }
    }
}
