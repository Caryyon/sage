//! NCA Language Model — Production-Scale Neural Cellular Automata for Language
//!
//! This is the real thing. No external LLM. The NCA grid IS the language model.
//!
//! Architecture:
//!   Text → SimpleTokenizer → token IDs → grid cell activation (positional encoding)
//!   → NCA update steps (3-layer MLP per cell, 3×3 neighborhood, recurrent across space+time)
//!   → activation readout → softmax → next token → auto-regressive generation
//!
//! Key insight: The same 107K-param MLP is applied recurrently across ALL cells
//! and ALL steps. This gives effective depth far beyond parameter count:
//!   - 32×32 grid × 5 steps = 5,120 MLP applications per forward pass
//!   - Each cell sees 3×3 neighborhood × 16 channels = 144 inputs
//!   - Information propagates across the grid through recurrent dynamics
//!
//! Training: Backpropagation through unrolled NCA steps (Adam optimizer).
//! The grid learns to route information — input tokens activate source cells,
//! NCA dynamics propagate and transform, output cells activate for predictions.
//!
//! Scale targets:
//!   - v0.1: 32×32 grid, 2K vocab, 5 steps → ~30 min training on CPU
//!   - v0.2: 64×64 grid, 8K vocab, 8 steps → ~2 hr training on CPU
//!   - v0.3: 128×128 grid, 16K vocab, 12 steps → GPU recommended

use crate::inference::nca_predictor::{NcaWeights, SimpleTokenizer, NCA_CHANNELS};
use crate::inference::{ChatMessage, ChatRole, InferenceEngine};
use rand::Rng;
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

// ── Architecture Constants ────────────────────────────────────────────────
// These are the same as nca_predictor.rs — the 3-layer MLP update rule.
// 144 → 384 → 128 → 16 = 107,024 parameters
// Applied recurrently across all cells and all steps.

const PERCEPTION_SIZE: usize = 9 * NCA_CHANNELS; // 144 (3×3 neighborhood × 16 channels)
const HIDDEN1_SIZE: usize = 384;
const HIDDEN2_SIZE: usize = 128;
const ACTIVATION_CH: usize = 0; // Channel 0 = token activation
const POSITION_CH: usize = 1; // Channel 1 = positional encoding
const CONFIDENCE_CH: usize = 2; // Channel 2 = confidence/order

// ── Default Paths ──────────────────────────────────────────────────────────

pub fn default_lm_weights_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_default()
        .join(".sage")
        .join("nca_lm_weights.bin")
}

pub fn default_lm_vocab_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_default()
        .join(".sage")
        .join("nca_lm_vocab.json")
}

pub fn default_lm_config_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_default()
        .join(".sage")
        .join("nca_lm_config.json")
}

// ── Configuration ──────────────────────────────────────────────────────────

/// Configuration for the NCA language model
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct NcaLmConfig {
    /// Grid side length (e.g., 32 → 32×32 = 1,024 cells)
    pub grid_size: usize,
    /// Number of NCA update steps per forward pass
    pub nca_steps: usize,
    /// BPE vocabulary size
    pub vocab_size: usize,
    /// Maximum tokens to generate
    pub max_tokens: usize,
    /// Temperature for sampling (0.0 = greedy, 1.0 = creative)
    pub temperature: f64,
    /// Top-k sampling (0 = disabled)
    pub top_k: usize,
    /// Top-p (nucleus) sampling (0.0 = disabled)
    pub top_p: f64,
    /// Repetition penalty (>1.0 = penalize repeats)
    pub repeat_penalty: f64,
    /// Context window size (how many previous tokens to encode)
    pub context_window: usize,
}

impl Default for NcaLmConfig {
    fn default() -> Self {
        Self {
            grid_size: 32,
            nca_steps: 5,
            vocab_size: 4096,
            max_tokens: 256,
            temperature: 0.8,
            top_k: 40,
            top_p: 0.9,
            repeat_penalty: 1.1,
            context_window: 64,
        }
    }
}

impl NcaLmConfig {
    /// Production config — 64×64 grid, 8K vocab, 8 steps
    pub fn production() -> Self {
        Self {
            grid_size: 64,
            nca_steps: 8,
            vocab_size: 8192,
            max_tokens: 512,
            temperature: 0.8,
            top_k: 50,
            top_p: 0.92,
            repeat_penalty: 1.15,
            context_window: 128,
        }
    }

    /// Fast training config — 16×16 grid, 1K vocab, 3 steps
    pub fn fast() -> Self {
        Self {
            grid_size: 16,
            nca_steps: 3,
            vocab_size: 1024,
            max_tokens: 128,
            temperature: 0.7,
            top_k: 20,
            top_p: 0.9,
            repeat_penalty: 1.05,
            context_window: 32,
        }
    }

    /// Total cells in the grid
    pub fn total_cells(&self) -> usize {
        self.grid_size * self.grid_size
    }

    /// Save config to disk
    pub fn save(&self, path: &Path) -> Result<(), Box<dyn Error>> {
        let json = serde_json::to_string_pretty(self)?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, json)?;
        Ok(())
    }

    /// Load config from disk
    pub fn load(path: &Path) -> Result<Self, Box<dyn Error>> {
        let json = fs::read_to_string(path)?;
        Ok(serde_json::from_str(&json)?)
    }
}

// ── Training Configuration ────────────────────────────────────────────────

/// Configuration for training the NCA language model
#[derive(Clone, Debug)]
pub struct NcaLmTrainingConfig {
    /// NCA model config
    pub model: NcaLmConfig,
    /// Learning rate for Adam
    pub learning_rate: f64,
    /// Number of training epochs
    pub epochs: usize,
    /// Gradient clipping norm
    pub grad_clip: f64,
    /// Maximum training examples (0 = use all)
    pub max_examples: usize,
    /// Whether to use cosine learning rate decay
    pub lr_decay: bool,
    /// Batch size for gradient accumulation
    pub batch_size: usize,
    /// Evaluation interval (epochs between evals)
    pub eval_interval: usize,
    /// Save checkpoint interval (epochs between saves)
    pub checkpoint_interval: usize,
    /// Checkpoint directory
    pub checkpoint_dir: Option<PathBuf>,
}

impl Default for NcaLmTrainingConfig {
    fn default() -> Self {
        Self {
            model: NcaLmConfig::default(),
            learning_rate: 0.001,
            epochs: 50,
            grad_clip: 1.0,
            max_examples: 0,
            lr_decay: true,
            batch_size: 16,
            eval_interval: 5,
            checkpoint_interval: 10,
            checkpoint_dir: None,
        }
    }
}

// ── Token Coordinate Mapping ───────────────────────────────────────────────

/// Map a token ID to a grid position using a hash-based scheme.
/// Unlike the simple row-major mapping in nca_predictor, this uses
/// a multiplicative hash to spread tokens across the grid, reducing
/// collisions when vocab > grid cells.
fn token_to_coord(token_id: usize, grid_size: usize) -> (usize, usize) {
    // Multiplicative hash for good distribution
    let h = token_id.wrapping_mul(2654435761); // Knuth's golden ratio hash
    let row = (h >> 16) as usize % grid_size;
    let col = (h >> 8) as usize % grid_size;
    (row, col)
}

// ── NCA Forward Pass ───────────────────────────────────────────────────────

/// Run one NCA update step on the entire grid.
/// Each cell computes its delta from its 3×3 neighborhood using the MLP.
fn nca_step(
    weights: &NcaWeights,
    grid: &mut [Vec<[f64; NCA_CHANNELS]>],
    grid_size: usize,
) {
    // Compute all deltas first, then apply (parallelizable in future)
    let mut deltas = vec![vec![[0.0f64; NCA_CHANNELS]; grid_size]; grid_size];

    for r in 0..grid_size {
        for c in 0..grid_size {
            // Perceive 3×3 neighborhood
            let mut input = [0.0f64; PERCEPTION_SIZE];
            let mut idx = 0;
            for dr in [-1i32, 0, 1] {
                for dc in [-1i32, 0, 1] {
                    let nr = (r as i32 + dr).rem_euclid(grid_size as i32) as usize;
                    let nc = (c as i32 + dc).rem_euclid(grid_size as i32) as usize;
                    for ch in 0..NCA_CHANNELS {
                        input[idx] = grid[nr][nc][ch];
                        idx += 1;
                    }
                }
            }

            // Layer 1: input → h1 (ReLU)
            let mut h1 = [0.0f64; HIDDEN1_SIZE];
            for h in 0..HIDDEN1_SIZE {
                let mut sum = weights.b1[h];
                for i in 0..PERCEPTION_SIZE {
                    sum += weights.w1[h][i] * input[i];
                }
                h1[h] = sum.max(0.0);
            }

            // Layer 2: h1 → h2 (ReLU)
            let mut h2 = [0.0f64; HIDDEN2_SIZE];
            for h in 0..HIDDEN2_SIZE {
                let mut sum = weights.b2[h];
                for i in 0..HIDDEN1_SIZE {
                    sum += weights.w2[h][i] * h1[i];
                }
                h2[h] = sum.max(0.0);
            }

            // Layer 3: h2 → delta (tanh * 0.1)
            for ch in 0..NCA_CHANNELS {
                let mut sum = weights.b3[ch];
                for h in 0..HIDDEN2_SIZE {
                    sum += weights.w3[ch][h] * h2[h];
                }
                deltas[r][c][ch] = sum.tanh() * 0.1;
            }
        }
    }

    // Apply deltas with clamp
    for r in 0..grid_size {
        for c in 0..grid_size {
            for ch in 0..NCA_CHANNELS {
                grid[r][c][ch] = (grid[r][c][ch] + deltas[r][c][ch]).clamp(-5.0, 5.0);
            }
        }
    }
}

/// Run multiple NCA steps
fn nca_forward(
    weights: &NcaWeights,
    grid: &mut [Vec<[f64; NCA_CHANNELS]>],
    grid_size: usize,
    steps: usize,
) {
    for _ in 0..steps {
        nca_step(weights, grid, grid_size);
    }
}

// ── Encoding Input Tokens into the Grid ────────────────────────────────────

/// Encode a sequence of token IDs into the NCA grid.
/// Each token activates its mapped cell with positional encoding.
fn encode_tokens(
    grid: &mut [Vec<[f64; NCA_CHANNELS]>],
    token_ids: &[usize],
    grid_size: usize,
) {
    let n = token_ids.len();
    for (pos, &tid) in token_ids.iter().enumerate() {
        let (r, c) = token_to_coord(tid, grid_size);
        // Activation strength decays with position (recent = stronger)
        let recency = if n > 1 {
            1.0 - (n - 1 - pos) as f64 / n as f64 * 0.5
        } else {
            1.0
        };
        grid[r][c][ACTIVATION_CH] = (grid[r][c][ACTIVATION_CH] + recency).min(5.0);
        // Positional encoding: normalized position in sequence
        grid[r][c][POSITION_CH] = if n > 1 {
            pos as f64 / (n - 1) as f64
        } else {
            0.5
        };
        // Confidence: higher for more recent tokens
        grid[r][c][CONFIDENCE_CH] = recency;
    }
}

// ── Reading Predictions from the Grid ──────────────────────────────────────

/// Read activation values for all vocabulary tokens from the grid.
/// Returns (token_id, activation) pairs sorted by activation descending.
fn read_activations(
    grid: &[Vec<[f64; NCA_CHANNELS]>],
    vocab_size: usize,
    grid_size: usize,
) -> Vec<(usize, f64)> {
    let mut activations: Vec<(usize, f64)> = (0..vocab_size)
        .map(|tid| {
            let (r, c) = token_to_coord(tid, grid_size);
            (tid, grid[r][c][ACTIVATION_CH])
        })
        .collect();
    activations.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    activations
}

/// Sample a token from activations using temperature + top-k + top-p
fn sample_token(
    activations: &[(usize, f64)],
    temperature: f64,
    top_k: usize,
    top_p: f64,
    rng: &mut impl Rng,
) -> usize {
    if activations.is_empty() {
        return 0; // <unk>
    }

    // Apply temperature scaling
    let logits: Vec<f64> = if temperature > 0.0 {
        activations
            .iter()
            .map(|(_, a)| a / temperature)
            .collect()
    } else {
        // Greedy: return highest activation
        return activations[0].0;
    };

    // Softmax
    let max_logit = logits.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let exps: Vec<f64> = logits.iter().map(|&l| (l - max_logit).exp()).collect();
    let sum: f64 = exps.iter().sum();
    let mut probs: Vec<f64> = exps.iter().map(|e| e / sum.max(1e-30)).collect();

    // Top-k: zero out probabilities beyond top-k
    if top_k > 0 && top_k < probs.len() {
        // Find the k-th highest probability
        let mut indexed: Vec<(usize, f64)> = probs.iter().cloned().enumerate().collect();
        indexed.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        let threshold = indexed[top_k.min(indexed.len() - 1)].1;
        for p in probs.iter_mut() {
            if *p < threshold {
                *p = 0.0;
            }
        }
        // Renormalize
        let new_sum: f64 = probs.iter().sum();
        if new_sum > 0.0 {
            for p in probs.iter_mut() {
                *p /= new_sum;
            }
        }
    }

    // Top-p (nucleus): keep tokens until cumulative probability > p
    if top_p > 0.0 && top_p < 1.0 {
        let mut indexed: Vec<(usize, f64)> = probs.iter().cloned().enumerate().collect();
        indexed.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        let mut cumsum = 0.0;
        let mut cutoff = 0;
        for (i, (_, p)) in indexed.iter().enumerate() {
            cumsum += p;
            if cumsum >= top_p {
                cutoff = i + 1;
                break;
            }
        }
        let keep_ids: Vec<usize> = indexed.iter().take(cutoff).map(|(id, _)| *id).collect();
        for (i, p) in probs.iter_mut().enumerate() {
            if !keep_ids.contains(&i) {
                *p = 0.0;
            }
        }
        // Renormalize
        let new_sum: f64 = probs.iter().sum();
        if new_sum > 0.0 {
            for p in probs.iter_mut() {
                *p /= new_sum;
            }
        }
    }

    // Sample from the filtered distribution
    let r: f64 = rng.gen();
    let mut cumsum = 0.0;
    for (i, &p) in probs.iter().enumerate() {
        cumsum += p;
        if r < cumsum {
            return activations[i].0;
        }
    }

    // Fallback: return highest probability token
    activations[0].0
}

// ── Auto-Regressive Generation ────────────────────────────────────────────

/// Generate tokens auto-regressively from a prompt.
/// Returns the generated token IDs.
fn generate_tokens(
    weights: &NcaWeights,
    tokenizer: &SimpleTokenizer,
    prompt_ids: &[usize],
    config: &NcaLmConfig,
) -> Vec<usize> {
    let mut rng = rand::thread_rng();
    let grid_size = config.grid_size;
    let vocab_size = tokenizer.vocab_size();
    let mut generated: Vec<usize> = Vec::with_capacity(config.max_tokens);
    let mut recent_tokens: Vec<usize> = prompt_ids.to_vec();

    // Track recently generated tokens for repetition penalty
    let mut token_counts: HashMap<usize, usize> = HashMap::new();

    for _ in 0..config.max_tokens {
        // Initialize fresh grid
        let mut grid = vec![vec![[0.0f64; NCA_CHANNELS]; grid_size]; grid_size];

        // Encode context window of recent tokens
        let ctx_start = if recent_tokens.len() > config.context_window {
            recent_tokens.len() - config.context_window
        } else {
            0
        };
        encode_tokens(&mut grid, &recent_tokens[ctx_start..], grid_size);

        // Run NCA dynamics
        nca_forward(weights, &mut grid, grid_size, config.nca_steps);

        // Read activations
        let mut activations = read_activations(&grid, vocab_size, grid_size);

        // Apply repetition penalty
        if config.repeat_penalty > 1.0 {
            for (tid, act) in activations.iter_mut() {
                if let Some(&count) = token_counts.get(tid) {
                    if count > 0 {
                        let penalty = config.repeat_penalty.powi(count as i32);
                        if *act > 0.0 {
                            *act /= penalty;
                        } else {
                            *act *= penalty;
                        }
                    }
                }
            }
            // Re-sort after penalty
            activations.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        }

        // Sample next token
        let next_token = sample_token(
            &activations,
            config.temperature,
            config.top_k,
            config.top_p,
            &mut rng,
        );

        // Check for EOS — SimpleTokenizer doesn't have a dedicated EOS token.
        // Instead, stop if we generate <unk> (ID 0) or hit max tokens.
        // We also stop if the same token repeats 3+ times consecutively (degeneration).
        if next_token == 0 {
            break;
        }
        // Stop if we're degenerating (same token 3+ times in a row)
        if generated.len() >= 2
            && generated[generated.len() - 1] == next_token
            && generated[generated.len() - 2] == next_token
        {
            break;
        }

        generated.push(next_token);
        recent_tokens.push(next_token);
        *token_counts.entry(next_token).or_insert(0) += 1;

        // Keep recent_tokens bounded
        if recent_tokens.len() > config.context_window * 2 {
            recent_tokens = recent_tokens[recent_tokens.len() - config.context_window..].to_vec();
        }
    }

    generated
}

// ── The NCA Language Model ─────────────────────────────────────────────────

/// The NCA Language Model — a fully self-contained language model
/// powered entirely by Neural Cellular Automata dynamics.
pub struct NcaLanguageModel {
    pub weights: NcaWeights,
    pub tokenizer: SimpleTokenizer,
    pub config: NcaLmConfig,
    pub trained: bool,
    name_str: String,
}

impl NcaLanguageModel {
    /// Create a new untrained model with random weights.
    /// You must call `train()` or `load()` before generating.
    pub fn new(config: NcaLmConfig) -> Self {
        let weights = NcaWeights::random();
        let grid_size = config.grid_size;
        let param_count = weights.param_count();
        // Create a minimal placeholder tokenizer — will be replaced during training
        let tokenizer = SimpleTokenizer::from_corpus("hello world", config.vocab_size);

        Self {
            weights,
            tokenizer,
            config,
            trained: false,
            name_str: format!(
                "NCA-LM ({}×{} grid, {} params, untrained)",
                grid_size, grid_size, param_count
            ),
        }
    }

    /// Load a trained model from disk.
    pub fn load(
        weights_path: Option<&Path>,
        vocab_path: Option<&Path>,
        config_path: Option<&Path>,
    ) -> Result<Self, Box<dyn Error>> {
        let wp = weights_path
            .map(|p| p.to_path_buf())
            .unwrap_or_else(default_lm_weights_path);
        let vp = vocab_path
            .map(|p| p.to_path_buf())
            .unwrap_or_else(default_lm_vocab_path);
        let cp = config_path
            .map(|p| p.to_path_buf())
            .unwrap_or_else(default_lm_config_path);

        let weights = NcaWeights::load(&wp)?;
        let config = NcaLmConfig::load(&cp)?;
        // Load vocabulary from corpus text file and rebuild tokenizer
        let vocab_text = fs::read_to_string(&vp)?;
        let tokenizer = SimpleTokenizer::from_corpus(&vocab_text, config.vocab_size);

        let param_count = weights.param_count();
        let grid_size = config.grid_size;
        let vocab_size = tokenizer.vocab_size();
        Ok(Self {
            weights,
            tokenizer,
            config,
            trained: true,
            name_str: format!(
                "NCA-LM ({}×{} grid, {}K tokens, {}K params, trained)",
                grid_size, grid_size,
                vocab_size / 1000,
                param_count / 1000
            ),
        })
    }

    /// Save the trained model to disk.
    pub fn save(
        &self,
        weights_path: Option<&Path>,
        vocab_path: Option<&Path>,
        config_path: Option<&Path>,
    ) -> Result<(), Box<dyn Error>> {
        let wp = weights_path
            .map(|p| p.to_path_buf())
            .unwrap_or_else(default_lm_weights_path);
        let vp = vocab_path
            .map(|p| p.to_path_buf())
            .unwrap_or_else(default_lm_vocab_path);
        let cp = config_path
            .map(|p| p.to_path_buf())
            .unwrap_or_else(default_lm_config_path);

        self.weights.save(&wp)?;
        // Save vocabulary as text (all tokens joined by spaces)
        let vocab_text: String = self.tokenizer.id_to_token.join(" ");
        fs::write(&vp, vocab_text)?;
        self.config.save(&cp)?;

        Ok(())
    }

    /// Check if the model has been trained
    pub fn is_trained(&self) -> bool {
        self.trained
    }

    /// Get vocabulary size
    pub fn vocab_size(&self) -> usize {
        self.tokenizer.vocab_size()
    }

    /// Get parameter count
    pub fn param_count(&self) -> usize {
        self.weights.param_count()
    }

    /// Get a reference to the config
    pub fn config(&self) -> &NcaLmConfig {
        &self.config
    }

    /// Generate text from a prompt string.
    pub fn generate_text(&self, prompt: &str) -> Result<String, Box<dyn Error>> {
        if !self.trained {
            return Err("NCA language model is not trained. Train it first.".into());
        }

        let prompt_ids = self.tokenizer.encode(prompt);
        let generated_ids = generate_tokens(
            &self.weights,
            &self.tokenizer,
            &prompt_ids,
            &self.config,
        );

        let decoded = self.tokenizer.decode(&generated_ids);
        Ok(decoded)
    }

    /// Generate text from chat messages.
    pub fn generate_chat(&self, messages: &[ChatMessage]) -> Result<String, Box<dyn Error>> {
        if !self.trained {
            return Err("NCA language model is not trained. Train it first.".into());
        }

        // Build a prompt from chat messages
        let mut prompt = String::new();

        // System message first
        if let Some(sys) = messages.iter().find(|m| m.role == ChatRole::System) {
            prompt.push_str(&format!("System: {}\n", sys.content));
        }

        // Conversation history
        for msg in messages.iter().filter(|m| m.role != ChatRole::System) {
            match msg.role {
                ChatRole::User => prompt.push_str(&format!("User: {}\n", msg.content)),
                ChatRole::Assistant => prompt.push_str(&format!("Assistant: {}\n", msg.content)),
                _ => {}
            }
        }

        // Add assistant prefix to trigger response
        prompt.push_str("Assistant: ");

        self.generate_text(&prompt)
    }

    /// Get a reference to the tokenizer (for training)
    pub fn tokenizer(&self) -> &SimpleTokenizer {
        &self.tokenizer
    }

    /// Get a mutable reference to the weights (for training)
    pub fn weights_mut(&mut self) -> &mut NcaWeights {
        &mut self.weights
    }

    /// Get a reference to the weights
    pub fn weights(&self) -> &NcaWeights {
        &self.weights
    }

    /// Mark as trained and update the display name
    pub fn mark_trained(&mut self) {
        self.trained = true;
        self.name_str = format!(
            "NCA-LM ({}×{} grid, {}K tokens, {}K params, trained)",
            self.config.grid_size,
            self.config.grid_size,
            self.tokenizer.vocab_size() / 1000,
            self.weights.param_count() / 1000
        );
    }
}

// ── InferenceEngine Implementation ─────────────────────────────────────────

/// Wrapper that makes NcaLanguageModel implement InferenceEngine.
/// Uses a Mutex because generation mutates internal state (the grid).
pub struct NcaLmEngine {
    model: Mutex<NcaLanguageModel>,
}

impl NcaLmEngine {
    pub fn new(model: NcaLanguageModel) -> Self {
        Self {
            model: Mutex::new(model),
        }
    }
}

impl InferenceEngine for NcaLmEngine {
    fn generate(&self, prompt: &str, max_tokens: usize) -> Result<String, Box<dyn Error>> {
        let mut model = self.model.lock().unwrap();
        model.config.max_tokens = max_tokens;
        model.generate_text(prompt)
    }

    fn chat(&self, messages: &[ChatMessage], max_tokens: usize) -> Result<String, Box<dyn Error>> {
        let mut model = self.model.lock().unwrap();
        model.config.max_tokens = max_tokens;
        model.generate_chat(messages)
    }

    fn generate_streaming(
        &self,
        prompt: &str,
        max_tokens: usize,
        mut callback: Box<dyn FnMut(&str) + Send>,
    ) -> Result<(), Box<dyn Error>> {
        let response = self.generate(prompt, max_tokens)?;
        // Simulate streaming by sending word by word
        for word in response.split_whitespace() {
            callback(word);
            callback(" ");
        }
        Ok(())
    }

    fn chat_streaming(
        &self,
        messages: &[ChatMessage],
        max_tokens: usize,
        mut callback: Box<dyn FnMut(&str) + Send>,
    ) -> Result<(), Box<dyn Error>> {
        let response = self.chat(messages, max_tokens)?;
        for word in response.split_whitespace() {
            callback(word);
            callback(" ");
        }
        Ok(())
    }

    fn name(&self) -> &str {
        // This is a bit hacky — we return a static string since name() returns &str
        // The actual name is in the model, but we can't return a reference to it
        "NCA-LM"
    }

    fn is_available(&self) -> bool {
        self.model.lock().unwrap().is_trained()
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_coord_distribution() {
        // Verify that token_to_coord spreads tokens across the grid
        let grid_size = 32;
        let mut coverage = vec![vec![false; grid_size]; grid_size];
        for tid in 0..1024 {
            let (r, c) = token_to_coord(tid, grid_size);
            coverage[r][c] = true;
        }
        let covered: usize = coverage.iter().flatten().filter(|&&x| x).count();
        // With 1024 tokens on 32×32=1024 cells, the multiplicative hash
        // distributes tokens across ~60% of cells (row/col derived from same hash)
        assert!(covered > 500, "Only {} of {} cells covered", covered, grid_size * grid_size);
    }

    #[test]
    fn test_nca_step_deterministic() {
        let weights = NcaWeights::random();
        let grid_size = 8;
        let mut grid1 = vec![vec![[0.0f64; NCA_CHANNELS]; grid_size]; grid_size];
        let mut grid2 = vec![vec![[0.0f64; NCA_CHANNELS]; grid_size]; grid_size];

        // Set same initial state
        grid1[0][0][ACTIVATION_CH] = 1.0;
        grid2[0][0][ACTIVATION_CH] = 1.0;

        nca_step(&weights, &mut grid1, grid_size);
        nca_step(&weights, &mut grid2, grid_size);

        // Same weights + same input → same output
        for r in 0..grid_size {
            for c in 0..grid_size {
                for ch in 0..NCA_CHANNELS {
                    assert!(
                        (grid1[r][c][ch] - grid2[r][c][ch]).abs() < 1e-15,
                        "Mismatch at ({}, {}, {}): {} vs {}",
                        r, c, ch, grid1[r][c][ch], grid2[r][c][ch]
                    );
                }
            }
        }
    }

    #[test]
    fn test_encode_tokens_activates_cells() {
        let grid_size = 8;
        let mut grid = vec![vec![[0.0f64; NCA_CHANNELS]; grid_size]; grid_size];
        let token_ids = vec![42, 100, 255];

        encode_tokens(&mut grid, &token_ids, grid_size);

        // At least some cells should be activated
        let active: usize = grid
            .iter()
            .flatten()
            .filter(|cell| cell[ACTIVATION_CH] > 0.0)
            .count();
        assert!(active > 0, "No cells activated after encoding");
    }

    #[test]
    fn test_sample_token_greedy() {
        let activations = vec![(5, 3.0), (3, 1.0), (7, 0.5)];
        let mut rng = rand::thread_rng();
        let token = sample_token(&activations, 0.0, 0, 0.0, &mut rng);
        // Greedy should always pick the highest activation
        assert_eq!(token, 5);
    }

    #[test]
    fn test_read_activations_sorted() {
        let grid_size = 8;
        let mut grid = vec![vec![[0.0f64; NCA_CHANNELS]; grid_size]; grid_size];
        // Set specific activations
        let (r0, c0) = token_to_coord(10, grid_size);
        let (r1, c1) = token_to_coord(20, grid_size);
        grid[r0][c0][ACTIVATION_CH] = 2.0;
        grid[r1][c1][ACTIVATION_CH] = 1.0;

        let activations = read_activations(&grid, 64, grid_size);
        // First entry should be the highest activation
        assert!(activations[0].1 >= activations[1].1);
    }

    #[test]
    fn test_config_save_load_roundtrip() {
        let config = NcaLmConfig::default();
        let tmp = std::env::temp_dir().join("test_nca_lm_config.json");
        config.save(&tmp).unwrap();
        let loaded = NcaLmConfig::load(&tmp).unwrap();
        assert_eq!(config.grid_size, loaded.grid_size);
        assert_eq!(config.nca_steps, loaded.nca_steps);
        assert_eq!(config.vocab_size, loaded.vocab_size);
        let _ = std::fs::remove_file(&tmp);
    }
}
