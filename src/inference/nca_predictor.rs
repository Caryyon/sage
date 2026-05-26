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
// ---------------------------------------------------------------------------
// Architecture constants — tune these to hit your target param count
// ---------------------------------------------------------------------------
// Pi 4 budget (f64, 4 cores): ~5 GFLOPS → 500ms = 2.5 GFLOPS
// At 64×64 grid, 3 steps: need ≤ ~200K FLOPS/cell/step
// Current: 3-layer MLP (144→384→128→16) = 107,024 params, ~213K FLOPS/cell
// → 64×64, 3 steps ≈ 2.6 GFLOPS ≈ 520ms on Pi 4 (close enough)
// ---------------------------------------------------------------------------

pub const NCA_CHANNELS: usize = 16; // Per-cell channels (was 8)
const ACTIVATION_CH: usize = 0;

/// Hidden layer sizes for the 3-layer MLP update rule
const HIDDEN1_SIZE: usize = 384; // First hidden layer
const HIDDEN2_SIZE: usize = 128; // Second hidden layer

/// Perception neighborhood: 3×3 = 9 cells × NCA_CHANNELS
const PERCEPTION_SIZE: usize = 9 * NCA_CHANNELS; // 144

/// Default NCA update steps per prediction
const DEFAULT_STEPS: usize = 10; // Reduced from 20 for Pi-friendly inference

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
        Self {
            token_to_id,
            id_to_token,
        }
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
            if word.is_empty() {
                continue;
            }
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

/// The NCA update rule weights (3-layer MLP: perception → hidden1 → hidden2 → output)
///
/// Architecture: 144 → 384 → 128 → 16 = 107,024 parameters
/// Optimized for Raspberry Pi 4: fits in <1MB, runs in ~500ms on 64×64 grid
#[derive(Clone)]
pub struct NcaWeights {
    pub w1: Vec<Vec<f64>>, // HIDDEN1_SIZE × PERCEPTION_SIZE
    pub b1: Vec<f64>,      // HIDDEN1_SIZE
    pub w2: Vec<Vec<f64>>, // HIDDEN2_SIZE × HIDDEN1_SIZE
    pub b2: Vec<f64>,      // HIDDEN2_SIZE
    pub w3: Vec<Vec<f64>>, // NCA_CHANNELS × HIDDEN2_SIZE
    pub b3: Vec<f64>,      // NCA_CHANNELS
}

impl NcaWeights {
    pub fn random() -> Self {
        let mut rng = rand::thread_rng();
        let scale1 = (2.0 / PERCEPTION_SIZE as f64).sqrt();
        let scale2 = (2.0 / HIDDEN1_SIZE as f64).sqrt();
        let scale3 = (2.0 / HIDDEN2_SIZE as f64).sqrt();
        Self {
            w1: (0..HIDDEN1_SIZE)
                .map(|_| {
                    (0..PERCEPTION_SIZE)
                        .map(|_| rng.gen_range(-scale1..scale1))
                        .collect()
                })
                .collect(),
            b1: vec![0.0; HIDDEN1_SIZE],
            w2: (0..HIDDEN2_SIZE)
                .map(|_| {
                    (0..HIDDEN1_SIZE)
                        .map(|_| rng.gen_range(-scale2..scale2))
                        .collect()
                })
                .collect(),
            b2: vec![0.0; HIDDEN2_SIZE],
            w3: (0..NCA_CHANNELS)
                .map(|_| {
                    (0..HIDDEN2_SIZE)
                        .map(|_| rng.gen_range(-scale3..scale3))
                        .collect()
                })
                .collect(),
            b3: vec![0.0; NCA_CHANNELS],
        }
    }

    /// Number of total parameters
    pub fn param_count(&self) -> usize {
        // Layer 1: PERCEPTION_SIZE × HIDDEN1_SIZE + HIDDEN1_SIZE
        // Layer 2: HIDDEN1_SIZE × HIDDEN2_SIZE + HIDDEN2_SIZE
        // Layer 3: HIDDEN2_SIZE × NCA_CHANNELS + NCA_CHANNELS
        HIDDEN1_SIZE * PERCEPTION_SIZE
            + HIDDEN1_SIZE
            + HIDDEN2_SIZE * HIDDEN1_SIZE
            + HIDDEN2_SIZE
            + NCA_CHANNELS * HIDDEN2_SIZE
            + NCA_CHANNELS
    }

    /// Flatten all params into a vec (for evolution strategy)
    pub fn to_vec(&self) -> Vec<f64> {
        let mut v = Vec::with_capacity(self.param_count());
        for row in &self.w1 {
            v.extend(row);
        }
        v.extend(&self.b1);
        for row in &self.w2 {
            v.extend(row);
        }
        v.extend(&self.b2);
        for row in &self.w3 {
            v.extend(row);
        }
        v.extend(&self.b3);
        v
    }

    /// Load from flat vec
    pub fn from_vec(params: &[f64]) -> Self {
        let mut idx = 0;

        let mut w1 = Vec::with_capacity(HIDDEN1_SIZE);
        for _ in 0..HIDDEN1_SIZE {
            w1.push(params[idx..idx + PERCEPTION_SIZE].to_vec());
            idx += PERCEPTION_SIZE;
        }
        let b1 = params[idx..idx + HIDDEN1_SIZE].to_vec();
        idx += HIDDEN1_SIZE;

        let mut w2 = Vec::with_capacity(HIDDEN2_SIZE);
        for _ in 0..HIDDEN2_SIZE {
            w2.push(params[idx..idx + HIDDEN1_SIZE].to_vec());
            idx += HIDDEN1_SIZE;
        }
        let b2 = params[idx..idx + HIDDEN2_SIZE].to_vec();
        idx += HIDDEN2_SIZE;

        let mut w3 = Vec::with_capacity(NCA_CHANNELS);
        for _ in 0..NCA_CHANNELS {
            w3.push(params[idx..idx + HIDDEN2_SIZE].to_vec());
            idx += HIDDEN2_SIZE;
        }
        let b3 = params[idx..idx + NCA_CHANNELS].to_vec();

        Self {
            w1,
            b1,
            w2,
            b2,
            w3,
            b3,
        }
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

    pub fn with_grid_size(
        tokenizer: SimpleTokenizer,
        weights: NcaWeights,
        steps: usize,
        grid_size: usize,
    ) -> Self {
        let grid = vec![vec![[0.0; NCA_CHANNELS]; grid_size]; grid_size];
        Self {
            grid,
            weights,
            steps,
            grid_size,
            tokenizer,
        }
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

    /// Apply the NCA update rule (one step) — 3-layer MLP
    fn nca_step(&mut self) {
        let mut deltas = vec![vec![[0.0; NCA_CHANNELS]; self.grid_size]; self.grid_size];

        for r in 0..self.grid_size {
            for c in 0..self.grid_size {
                let input = self.perceive(r, c);

                // Layer 1: perception → hidden1 (ReLU)
                let mut h1 = vec![0.0; HIDDEN1_SIZE];
                for h in 0..HIDDEN1_SIZE {
                    let mut sum = self.weights.b1[h];
                    for i in 0..PERCEPTION_SIZE {
                        sum += self.weights.w1[h][i] * input[i];
                    }
                    h1[h] = sum.max(0.0); // ReLU
                }

                // Layer 2: hidden1 → hidden2 (ReLU)
                let mut h2 = vec![0.0; HIDDEN2_SIZE];
                for h in 0..HIDDEN2_SIZE {
                    let mut sum = self.weights.b2[h];
                    for i in 0..HIDDEN1_SIZE {
                        sum += self.weights.w2[h][i] * h1[i];
                    }
                    h2[h] = sum.max(0.0); // ReLU
                }

                // Layer 3: hidden2 → output (tanh, residual)
                for ch in 0..NCA_CHANNELS {
                    let mut sum = self.weights.b3[ch];
                    for h in 0..HIDDEN2_SIZE {
                        sum += self.weights.w3[ch][h] * h2[h];
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
        let max_val = activations
            .iter()
            .cloned()
            .fold(f64::NEG_INFINITY, f64::max);
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

    /// Generate an answer string given a query and context.
    ///
    /// Encodes the query + context, then auto-regressively predicts
    /// next tokens using the NCA grid dynamics.
    ///
    /// Returns the decoded text answer, or an error if the corpus
    /// is too small / no context provided.
    pub fn answer(
        &mut self,
        query: &str,
        context: Option<&str>,
        max_tokens: usize,
    ) -> Result<String, Box<dyn std::error::Error>> {
        // Build prompt from query + context
        let prompt = if let Some(ctx) = context {
            format!("{} context: {}", query, ctx)
        } else {
            query.to_string()
        };

        // Encode prompt
        let tokens = self.tokenizer.encode(&prompt);
        if tokens.is_empty() {
            return Err("Empty prompt after tokenization".into());
        }

        // Auto-regressive generation
        let mut generated = tokens.clone();
        let mut last_token = *tokens.last().unwrap();

        for _ in 0..max_tokens {
            // Predict next token
            let next_id = self.predict_next_sampled(&generated, 0.7);

            // Stop if we repeat the same token (stuck) or hit <unk>
            if next_id == last_token || next_id == 0 {
                break;
            }

            generated.push(next_id);
            last_token = next_id;

            // Stop if predicted token is a sentence-ending punctuation
            if let Some(token_str) = self.tokenizer.id_to_token.get(next_id) {
                if token_str.ends_with('.') || token_str.ends_with('?') || token_str.ends_with('!')
                {
                    break;
                }
            }
        }

        // Decode only the newly generated tokens (not the prompt)
        let answer_ids = &generated[tokens.len().saturating_sub(0)..];
        let answer = self.tokenizer.decode(answer_ids);

        // Clean up: remove trailing context marker if present
        let answer = answer.replace(" context: ", "").trim().to_string();

        if answer.is_empty() {
            return Err("NCA predictor produced empty answer".into());
        }

        Ok(answer)
    }

    /// Set weights
    pub fn set_weights(&mut self, w: NcaWeights) {
        self.weights = w;
    }
}

// ---------------------------------------------------------------------------
// Training via Evolution Strategy (ES)
// ---------------------------------------------------------------------------

/// Cell brain type: MLP (traditional) or KAN (Kolmogorov-Arnold)
#[derive(Clone, Debug)]
pub enum CellType {
    Mlp,
    Kan,
}

impl std::fmt::Display for CellType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CellType::Mlp => write!(f, "mlp"),
            CellType::Kan => write!(f, "kan"),
        }
    }
}

/// Which optimizer to use for training
#[derive(Clone, Debug, PartialEq)]
pub enum Optimizer {
    /// Original naive evolution strategy (gradient estimate from perturbations)
    Es,
    /// Separable CMA-ES (diagonal covariance — practical for high-dim params)
    CmaEs,
    /// Backpropagation through unrolled NCA steps (Adam optimizer)
    Backprop,
}

impl std::fmt::Display for Optimizer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Optimizer::Es => write!(f, "es"),
            Optimizer::CmaEs => write!(f, "cma-es"),
            Optimizer::Backprop => write!(f, "backprop"),
        }
    }
}

/// Training configuration
pub struct TrainingConfig {
    pub population_size: usize,
    pub sigma: f64, // Noise standard deviation / initial step size
    pub learning_rate: f64,
    pub epochs: usize,
    pub context_window: usize, // How many tokens of context
    pub grid_size: usize,      // Grid side length for training (smaller = faster)
    pub nca_steps: usize,      // NCA update steps per evaluation during training
    pub max_examples: usize,   // Max training examples to subsample
    pub optimizer: Optimizer,  // Which optimizer to use
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
            optimizer: Optimizer::Es,
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

    let random_accuracy = 1.0 / vocab_size as f64;

    if verbose {
        eprintln!("📊 Optimizer: {}", config.optimizer);
        eprintln!(
            "📊 Vocab size: {}, Corpus tokens: {}, Grid: {}×{}, Random baseline: {:.4}%",
            vocab_size,
            tokens.len(),
            grid_size,
            grid_size,
            random_accuracy * 100.0
        );
        let pc = NcaWeights::random().param_count();
        eprintln!(
            "📊 NCA params: {} ({:.1} KB as f64)",
            pc,
            pc as f64 * 8.0 / 1024.0
        );
        eprintln!(
            "📊 Architecture: {} → {} (ReLU) → {} (ReLU) → {} (tanh)",
            PERCEPTION_SIZE, HIDDEN1_SIZE, HIDDEN2_SIZE, NCA_CHANNELS
        );
    }

    // Build training examples (subsample for speed)
    let max_examples = config
        .max_examples
        .min(tokens.len() - config.context_window);
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

    let (best_weights, best_fitness) = match config.optimizer {
        Optimizer::Es => train_es(
            &tokenizer,
            &examples,
            config,
            grid_size,
            verbose,
            random_accuracy,
        ),
        Optimizer::CmaEs => train_cma_es(
            &tokenizer,
            &examples,
            config,
            grid_size,
            verbose,
            random_accuracy,
        ),
        Optimizer::Backprop => {
            // Backprop is handled separately via backprop_trainer module
            return Err("Use backprop_trainer::train_nca_backprop() for backprop optimizer".into());
        }
    };

    let predictor = NcaPredictor::with_grid_size(tokenizer, best_weights, DEFAULT_STEPS, grid_size);
    Ok((predictor, best_fitness, random_accuracy))
}

/// Train the NCA predictor using KAN cell brains instead of MLP.
/// Returns (trained_predictor_with_mlp_weights_placeholder, final_accuracy, random_baseline)
pub fn train_nca_kan(
    corpus: &str,
    config: &TrainingConfig,
    verbose: bool,
) -> Result<(NcaPredictor, f64, f64), Box<dyn Error>> {
    use super::kan::KanNcaWeights;

    let grid_size = config.grid_size;
    let tokenizer = SimpleTokenizer::from_corpus(corpus, grid_size * grid_size);
    let tokens = tokenizer.encode(corpus);
    let vocab_size = tokenizer.vocab_size();

    if tokens.len() < config.context_window + 1 {
        return Err("Corpus too small for training".into());
    }

    let random_accuracy = 1.0 / vocab_size as f64;
    let kan_weights = KanNcaWeights::random();
    let mlp_weights = NcaWeights::random();

    if verbose {
        eprintln!("📊 Cell type: KAN");
        eprintln!("📊 Optimizer: CMA-ES (KAN training)");
        eprintln!(
            "📊 Vocab: {}, Tokens: {}, Grid: {}×{}, Random: {:.4}%",
            vocab_size,
            tokens.len(),
            grid_size,
            grid_size,
            random_accuracy * 100.0
        );
        eprintln!(
            "📊 KAN params: {} ({:.1} KB)",
            kan_weights.param_count(),
            kan_weights.param_count() as f64 * 8.0 / 1024.0
        );
        eprintln!("📊 MLP params: {} (comparison)", mlp_weights.param_count());
        let kw = &kan_weights;
        eprintln!(
            "📊 KAN arch: {:?} grid={} order={}",
            kw.widths(),
            kw.grid_size(),
            kw.order()
        );
    }

    // Build training examples
    let max_examples = config
        .max_examples
        .min(tokens.len() - config.context_window);
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

    // Train with CMA-ES on KAN params
    let (best_kan_weights, best_fitness) = train_cma_es_kan(
        &tokenizer,
        &examples,
        config,
        grid_size,
        verbose,
        random_accuracy,
    );

    // Return an NcaPredictor with default MLP weights (KAN weights saved separately)
    // The accuracy/fitness reflects KAN performance
    let predictor =
        NcaPredictor::with_grid_size(tokenizer, NcaWeights::random(), DEFAULT_STEPS, grid_size);
    // Store KAN weights reference for saving
    KAN_LAST_WEIGHTS.with(|w| *w.borrow_mut() = Some(best_kan_weights));
    Ok((predictor, best_fitness, random_accuracy))
}

thread_local! {
    static KAN_LAST_WEIGHTS: std::cell::RefCell<Option<super::kan::KanNcaWeights>> = const { std::cell::RefCell::new(None) };
}

/// CMA-ES optimizer for KAN weights
fn train_cma_es_kan(
    tokenizer: &SimpleTokenizer,
    examples: &[(Vec<usize>, usize)],
    config: &TrainingConfig,
    grid_size: usize,
    verbose: bool,
    random_accuracy: f64,
) -> (super::kan::KanNcaWeights, f64) {
    use super::kan::KanNcaWeights;

    let initial = KanNcaWeights::random();
    let n = initial.param_count();
    let mean: Vec<f64> = initial.to_vec();

    // CMA-ES hyperparams
    let lambda = config
        .population_size
        .max(4 + (3.0 * (n as f64).ln()).floor() as usize);
    let mu = lambda / 2;
    let weights_raw: Vec<f64> = (0..mu)
        .map(|i| ((lambda as f64 + 1.0) / 2.0).ln() - ((i + 1) as f64).ln())
        .collect();
    let w_sum: f64 = weights_raw.iter().sum();
    let weights: Vec<f64> = weights_raw.iter().map(|w| w / w_sum).collect();
    let mu_eff: f64 = 1.0 / weights.iter().map(|w| w * w).sum::<f64>();

    let sigma_init = if config.sigma > 0.1 {
        config.sigma
    } else {
        0.3
    };
    let mut sigma = sigma_init;
    let mut m = mean;

    // Diagonal covariance (separable CMA-ES)
    let mut diag_c: Vec<f64> = vec![1.0; n];
    let mut p_sigma: Vec<f64> = vec![0.0; n];
    let c_sigma = 4.0 / (n as f64 + 4.0);
    let d_sigma =
        1.0 + 2.0 * (0.0_f64).max(((mu_eff - 1.0) / (n as f64 + 1.0)).sqrt() - 1.0) + c_sigma;
    let chi_n =
        (n as f64).sqrt() * (1.0 - 1.0 / (4.0 * n as f64) + 1.0 / (21.0 * n as f64 * n as f64));
    let c1 = 2.0 / ((n as f64 + 1.3).powi(2) + mu_eff);
    let c_mu =
        (2.0 * (mu_eff - 2.0 + 1.0 / mu_eff) / ((n as f64 + 2.0).powi(2) + mu_eff)).min(1.0 - c1);

    if verbose {
        eprintln!(
            "📊 CMA-ES: λ={}, μ={}, μ_eff={:.1}, σ₀={:.3}, n={}",
            lambda, mu, mu_eff, sigma_init, n
        );
    }

    let mut rng = rand::thread_rng();
    let mut best_fitness = f64::NEG_INFINITY;
    let mut best_params = m.clone();

    for epoch in 0..config.epochs {
        // Sample candidates
        let mut candidates: Vec<(Vec<f64>, f64)> = Vec::with_capacity(lambda);
        for _ in 0..lambda {
            let z: Vec<f64> = (0..n)
                .map(|i| {
                    let norm = rand_normal(&mut rng);
                    m[i] + sigma * diag_c[i].sqrt() * norm
                })
                .collect();
            let w = KanNcaWeights::from_vec(&z);
            let fitness =
                evaluate_fitness_kan(tokenizer, &w, examples, grid_size, config.nca_steps);
            candidates.push((z, fitness));
        }

        // Sort by fitness (descending)
        candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        if candidates[0].1 > best_fitness {
            best_fitness = candidates[0].1;
            best_params = candidates[0].0.clone();
        }

        // Update mean
        let old_m = m.clone();
        m = vec![0.0; n];
        for i in 0..mu {
            for j in 0..n {
                m[j] += weights[i] * candidates[i].0[j];
            }
        }

        // Update evolution path
        for j in 0..n {
            let dj = (m[j] - old_m[j]) / (sigma * diag_c[j].sqrt());
            p_sigma[j] =
                (1.0 - c_sigma) * p_sigma[j] + (c_sigma * (2.0 - c_sigma) * mu_eff).sqrt() * dj;
        }

        // Update sigma
        let ps_norm: f64 = p_sigma.iter().map(|x| x * x).sum::<f64>().sqrt();
        sigma *= ((c_sigma / d_sigma) * (ps_norm / chi_n - 1.0)).exp();

        // Update diagonal covariance
        for j in 0..n {
            let mut rank_mu_update = 0.0;
            for i in 0..mu {
                let dj = (candidates[i].0[j] - old_m[j]) / (sigma * diag_c[j].sqrt());
                rank_mu_update += weights[i] * (dj * dj - 1.0);
            }
            diag_c[j] *= (1.0 + c1 * (p_sigma[j] * p_sigma[j] - 1.0) + c_mu * rank_mu_update)
                .max(0.1)
                .exp()
                .min(10.0);
        }

        if verbose {
            eprintln!(
                "  Epoch {}/{}: accuracy = {:.4}% (best = {:.4}%, σ = {:.6}, random = {:.4}%)",
                epoch + 1,
                config.epochs,
                candidates[0].1 * 100.0,
                best_fitness * 100.0,
                sigma,
                random_accuracy * 100.0
            );
        }
    }

    (KanNcaWeights::from_vec(&best_params), best_fitness)
}

/// Evaluate fitness using KAN weights
fn evaluate_fitness_kan(
    tokenizer: &SimpleTokenizer,
    weights: &super::kan::KanNcaWeights,
    examples: &[(Vec<usize>, usize)],
    grid_size: usize,
    nca_steps: usize,
) -> f64 {
    // Build a temporary grid and run KAN-based NCA
    let vocab_size = tokenizer.vocab_size();
    let mut correct = 0;

    for (ctx, target) in examples {
        // Initialize grid
        let mut grid = vec![vec![[0.0f64; NCA_CHANNELS]; grid_size]; grid_size];

        // Encode context tokens
        for &tok_id in ctx {
            if tok_id < vocab_size {
                let r = tok_id / grid_size;
                let c = tok_id % grid_size;
                if r < grid_size && c < grid_size {
                    grid[r][c][ACTIVATION_CH] = 1.0;
                }
            }
        }

        // Run NCA steps with KAN update rule
        for _ in 0..nca_steps {
            let mut new_grid = grid.clone();
            for r in 0..grid_size {
                for c in 0..grid_size {
                    // Perceive 3×3 neighborhood
                    let mut input = vec![0.0f64; 9 * NCA_CHANNELS];
                    let mut idx = 0;
                    for dr in -1i32..=1 {
                        for dc in -1i32..=1 {
                            let nr = ((r as i32 + dr).rem_euclid(grid_size as i32)) as usize;
                            let nc = ((c as i32 + dc).rem_euclid(grid_size as i32)) as usize;
                            for ch in 0..NCA_CHANNELS {
                                input[idx] = grid[nr][nc][ch];
                                idx += 1;
                            }
                        }
                    }

                    // KAN forward pass
                    let output = weights.forward(&input);
                    for ch in 0..NCA_CHANNELS.min(output.len()) {
                        new_grid[r][c][ch] = (grid[r][c][ch] + output[ch]).tanh();
                    }
                }
            }
            grid = new_grid;
        }

        // Read activations — top-5
        let mut indexed: Vec<(usize, f64)> = (0..vocab_size.min(grid_size * grid_size))
            .map(|id| {
                let r = id / grid_size;
                let c = id % grid_size;
                (id, grid[r][c][ACTIVATION_CH])
            })
            .collect();
        indexed.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        if indexed.iter().take(5).any(|(id, _)| id == target) {
            correct += 1;
        }
    }

    correct as f64 / examples.len() as f64
}

/// Original naive ES optimizer
fn train_es(
    tokenizer: &SimpleTokenizer,
    examples: &[(Vec<usize>, usize)],
    config: &TrainingConfig,
    grid_size: usize,
    verbose: bool,
    random_accuracy: f64,
) -> (NcaWeights, f64) {
    let mut best_weights = NcaWeights::random();
    let mut best_fitness = f64::NEG_INFINITY;
    let mut rng = rand::thread_rng();

    for epoch in 0..config.epochs {
        let base_params = best_weights.to_vec();
        let n_params = base_params.len();

        let mut noise_vecs: Vec<Vec<f64>> = Vec::with_capacity(config.population_size);
        let mut fitnesses: Vec<f64> = Vec::with_capacity(config.population_size);

        for _ in 0..config.population_size {
            let noise: Vec<f64> = (0..n_params)
                .map(|_| rng.gen::<f64>() * 2.0 - 1.0)
                .collect();
            let perturbed: Vec<f64> = base_params
                .iter()
                .zip(&noise)
                .map(|(p, n)| p + config.sigma * n)
                .collect();

            let w = NcaWeights::from_vec(&perturbed);
            let fitness = evaluate_fitness(tokenizer, &w, examples, grid_size, config.nca_steps);
            noise_vecs.push(noise);
            fitnesses.push(fitness);
        }

        let mean_f: f64 = fitnesses.iter().sum::<f64>() / fitnesses.len() as f64;
        let std_f: f64 = {
            let var = fitnesses.iter().map(|f| (f - mean_f).powi(2)).sum::<f64>()
                / fitnesses.len() as f64;
            var.sqrt().max(1e-8)
        };
        let norm_fitnesses: Vec<f64> = fitnesses.iter().map(|f| (f - mean_f) / std_f).collect();

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
        let new_fitness = evaluate_fitness(
            tokenizer,
            &new_weights,
            examples,
            grid_size,
            config.nca_steps,
        );

        if new_fitness > best_fitness {
            best_fitness = new_fitness;
            best_weights = new_weights;
        }

        if verbose {
            eprintln!(
                "  Epoch {}/{}: accuracy = {:.4}% (random = {:.4}%)",
                epoch + 1,
                config.epochs,
                best_fitness * 100.0,
                random_accuracy * 100.0
            );
        }
    }

    (best_weights, best_fitness)
}

/// Separable CMA-ES optimizer (diagonal covariance — O(n) memory for high-dim)
///
/// Implementation follows Hansen's sep-CMA-ES:
/// - Diagonal covariance matrix C = diag(c_1, ..., c_n) instead of full n×n
/// - Same step-size adaptation (CSA) and rank-μ/rank-1 update logic
/// - Population size λ = 4 + floor(3 * ln(n)), μ = λ/2
fn train_cma_es(
    tokenizer: &SimpleTokenizer,
    examples: &[(Vec<usize>, usize)],
    config: &TrainingConfig,
    grid_size: usize,
    verbose: bool,
    random_accuracy: f64,
) -> (NcaWeights, f64) {
    let mut rng = rand::thread_rng();
    let init_weights = NcaWeights::random();
    let n = init_weights.param_count();

    // --- CMA-ES parameters ---
    let lambda = (4.0 + (3.0 * (n as f64).ln()).floor()) as usize;
    let lambda = lambda.max(config.population_size); // at least user-specified
    let mu = lambda / 2;

    // Recombination weights (log-linear)
    let raw_weights: Vec<f64> = (0..mu)
        .map(|i| ((mu as f64 + 0.5).ln() - ((i + 1) as f64).ln()).max(0.0))
        .collect();
    let w_sum: f64 = raw_weights.iter().sum();
    let weights: Vec<f64> = raw_weights.iter().map(|w| w / w_sum).collect();
    let mu_eff = 1.0 / weights.iter().map(|w| w * w).sum::<f64>();

    // Step-size adaptation
    let c_sigma = (mu_eff + 2.0) / (n as f64 + mu_eff + 5.0);
    let d_sigma = 1.0 + 2.0 * (((mu_eff - 1.0) / (n as f64 + 1.0)).sqrt() - 1.0).max(0.0) + c_sigma;
    let chi_n =
        (n as f64).sqrt() * (1.0 - 1.0 / (4.0 * n as f64) + 1.0 / (21.0 * n as f64 * n as f64));

    // Covariance adaptation (sep-CMA: only diagonal)
    let c_c = (4.0 + mu_eff / n as f64) / (n as f64 + 4.0 + 2.0 * mu_eff / n as f64);
    let c_1 = 2.0 / ((n as f64 + 1.3).powi(2) + mu_eff);
    let c_mu_val =
        (1.0 - c_1).min(2.0 * (mu_eff - 2.0 + 1.0 / mu_eff) / ((n as f64 + 2.0).powi(2) + mu_eff));

    // State vectors
    let mut mean = init_weights.to_vec();
    let mut sigma = config.sigma.max(0.3); // CMA-ES needs larger initial sigma
    let mut p_sigma = vec![0.0; n]; // evolution path for sigma
    let mut p_c = vec![0.0; n]; // evolution path for covariance
    let mut diag_c = vec![1.0; n]; // diagonal of covariance matrix

    let mut best_weights = init_weights;
    let mut best_fitness = evaluate_fitness(
        tokenizer,
        &best_weights,
        examples,
        grid_size,
        config.nca_steps,
    );

    if verbose {
        eprintln!(
            "📊 CMA-ES: λ={}, μ={}, μ_eff={:.1}, σ₀={:.3}, n={}",
            lambda, mu, mu_eff, sigma, n
        );
    }

    for epoch in 0..config.epochs {
        // Sample λ candidates: x_k = mean + σ * D * z_k where D = sqrt(diag_c)
        let sqrt_c: Vec<f64> = diag_c.iter().map(|c| f64::sqrt(*c)).collect();

        let mut candidates: Vec<(Vec<f64>, Vec<f64>, f64)> = Vec::with_capacity(lambda);
        // (params, z_vector, fitness)

        for _ in 0..lambda {
            let z: Vec<f64> = (0..n).map(|_| rand_normal(&mut rng)).collect();
            let params: Vec<f64> = (0..n).map(|i| mean[i] + sigma * sqrt_c[i] * z[i]).collect();
            let w = NcaWeights::from_vec(&params);
            let fitness = evaluate_fitness(tokenizer, &w, examples, grid_size, config.nca_steps);
            candidates.push((params, z, fitness));
        }

        // Sort by fitness (descending — we maximize)
        candidates.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));

        // Track best ever
        if candidates[0].2 > best_fitness {
            best_fitness = candidates[0].2;
            best_weights = NcaWeights::from_vec(&candidates[0].0);
        }

        // Weighted recombination: new mean
        let old_mean = mean.clone();
        for i in 0..n {
            mean[i] = 0.0;
            for j in 0..mu {
                mean[i] += weights[j] * candidates[j].0[i];
            }
        }

        // Weighted z-step
        let mut z_w = vec![0.0; n];
        for i in 0..n {
            for j in 0..mu {
                z_w[i] += weights[j] * candidates[j].1[i];
            }
        }

        // Update evolution path for sigma (CSA)
        let _sqrt_mu_eff = mu_eff.sqrt();
        for i in 0..n {
            p_sigma[i] =
                (1.0 - c_sigma) * p_sigma[i] + (c_sigma * (2.0 - c_sigma) * mu_eff).sqrt() * z_w[i];
        }

        // |p_sigma|
        let ps_norm: f64 = p_sigma.iter().map(|x| x * x).sum::<f64>().sqrt();

        // h_sigma: stall indicator
        let h_sigma = if ps_norm / (1.0 - (1.0 - c_sigma).powi(2 * (epoch as i32 + 1))).sqrt()
            < (1.4 + 2.0 / (n as f64 + 1.0)) * chi_n
        {
            1.0
        } else {
            0.0
        };

        // Update evolution path for covariance
        for i in 0..n {
            let step = (mean[i] - old_mean[i]) / sigma;
            p_c[i] = (1.0 - c_c) * p_c[i]
                + h_sigma * (c_c * (2.0 - c_c) * mu_eff).sqrt() * step / sqrt_c[i].max(1e-30);
        }

        // Update diagonal covariance
        let delta_h = (1.0 - h_sigma) * c_c * (2.0 - c_c);
        for i in 0..n {
            let mut rank_mu_sum = 0.0;
            for j in 0..mu {
                let d = candidates[j].1[i]; // z_i for candidate j
                rank_mu_sum += weights[j] * d * d;
            }
            diag_c[i] = (1.0 - c_1 - c_mu_val + c_1 * delta_h) * diag_c[i]
                + c_1 * p_c[i] * p_c[i]
                + c_mu_val * rank_mu_sum;
            // Clamp to prevent degeneration
            diag_c[i] = diag_c[i].clamp(1e-20, 1e6);
        }

        // Step-size adaptation
        sigma *= ((c_sigma / d_sigma) * (ps_norm / chi_n - 1.0)).exp();
        sigma = sigma.clamp(1e-20, 1e3);

        if verbose {
            eprintln!(
                "  Epoch {}/{}: accuracy = {:.4}% (best = {:.4}%, σ = {:.6}, random = {:.4}%)",
                epoch + 1,
                config.epochs,
                candidates[0].2 * 100.0,
                best_fitness * 100.0,
                sigma,
                random_accuracy * 100.0
            );
        }
    }

    (best_weights, best_fitness)
}

/// Sample from standard normal using Box-Muller transform
fn rand_normal(rng: &mut impl Rng) -> f64 {
    let u1: f64 = rng.gen::<f64>().max(1e-30);
    let u2: f64 = rng.gen::<f64>();
    (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos()
}

/// Evaluate accuracy on examples
fn evaluate_fitness(
    tokenizer: &SimpleTokenizer,
    weights: &NcaWeights,
    examples: &[(Vec<usize>, usize)],
    grid_size: usize,
    nca_steps: usize,
) -> f64 {
    let mut predictor =
        NcaPredictor::with_grid_size(tokenizer.clone(), weights.clone(), nca_steps, grid_size);
    let mut correct = 0;
    // Use top-5 accuracy for richer signal
    for (ctx, target) in examples {
        let activations = predictor.run_and_read(ctx);
        // Top-5 check
        let mut indexed: Vec<(usize, f64)> = activations
            .iter()
            .enumerate()
            .map(|(i, &v)| (i, v))
            .collect();
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
        Self {
            config,
            predictions: 0,
            correct: 0,
        }
    }

    pub fn record(&mut self, nca_was_correct: bool) {
        self.predictions += 1;
        if nca_was_correct {
            self.correct += 1;
        }

        // Every 100 predictions, check if we should promote NCA
        if self.predictions.is_multiple_of(100) && self.predictions > 0 {
            let accuracy = self.correct as f64 / self.predictions as f64;
            if accuracy > self.config.promotion_threshold {
                self.config.nca_weight =
                    (self.config.nca_weight + self.config.promotion_rate).min(1.0);
                eprintln!(
                    "🧠 NCA accuracy {:.1}% > threshold, nca_weight → {:.2}",
                    accuracy * 100.0,
                    self.config.nca_weight
                );
            }
        }
    }

    pub fn current_accuracy(&self) -> f64 {
        if self.predictions == 0 {
            0.0
        } else {
            self.correct as f64 / self.predictions as f64
        }
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
        // Use small grid + few steps for fast determinism check
        let mut pred = NcaPredictor::with_grid_size(tok, w, 2, 8);
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
