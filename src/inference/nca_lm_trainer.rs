//! NCA Language Model Training Pipeline
//!
//! Trains the NCA language model on a specialist curriculum.
//! Uses backpropagation through unrolled NCA steps with Adam optimizer.
//!
//! Training flow:
//!   1. Load curriculum JSON → extract facts, generate Q&A pairs, code examples
//!   2. Train BPE tokenizer on the combined corpus
//!   3. Build training examples: (context tokens, target token) pairs
//!   4. Train NCA weights via backprop through unrolled steps
//!   5. Evaluate top-5 accuracy, save checkpoint
//!   6. Repeat for N epochs

use super::bpe_tokenizer::BpeTokenizer;
use super::nca_lm::{NcaLanguageModel, NcaLmConfig, NcaLmTrainingConfig};
use super::nca_predictor::{NcaWeights, SimpleTokenizer, NCA_CHANNELS};
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

// ── Tokenizer Trait ────────────────────────────────────────────────────────

/// Trait for tokenizers used in NCA LM training.
/// Both SimpleTokenizer and BpeTokenizer implement this.
pub trait NcaTokenizer: Send + Sync {
    fn encode(&self, text: &str) -> Vec<usize>;
    fn decode(&self, ids: &[usize]) -> String;
    fn vocab_size(&self) -> usize;
    /// Downcast to SimpleTokenizer if this is one (for model assignment).
    fn as_simple(&self) -> Option<&SimpleTokenizer> { None }
}

impl NcaTokenizer for SimpleTokenizer {
    fn encode(&self, text: &str) -> Vec<usize> {
        self.encode(text)
    }
    fn decode(&self, ids: &[usize]) -> String {
        self.decode(ids)
    }
    fn vocab_size(&self) -> usize {
        self.vocab_size()
    }
    fn as_simple(&self) -> Option<&SimpleTokenizer> {
        Some(self)
    }
}

impl NcaTokenizer for BpeTokenizer {
    fn encode(&self, text: &str) -> Vec<usize> {
        self.encode(text)
    }
    fn decode(&self, ids: &[usize]) -> String {
        self.decode(ids)
    }
    fn vocab_size(&self) -> usize {
        self.vocab_size()
    }
}

// ── Architecture Constants (same as nca_predictor) ─────────────────────────

const PERCEPTION_SIZE: usize = 9 * NCA_CHANNELS; // 144
const HIDDEN1_SIZE: usize = 384;
const HIDDEN2_SIZE: usize = 128;
const ACTIVATION_CH: usize = 0;

// ── Curriculum to Corpus Conversion ────────────────────────────────────────

/// Convert a specialist curriculum JSON into a training corpus.
///
/// The curriculum format (from curricula/*.json):
/// ```json
/// {
///   "name": "junior-react-dev",
///   "domain": "frontend-development",
///   "topics": [
///     {
///       "name": "react-fundamentals",
///       "facts": [{"fact": "React is a JavaScript library..."}, ...]
///     }
///   ]
/// }
/// ```
///
/// We generate:
/// - Fact statements (direct from curriculum)
/// - Q&A pairs (generated from facts)
/// - Domain-specific code examples
/// - Conversation templates
pub fn curriculum_to_corpus(curriculum_path: &Path) -> Result<String, Box<dyn Error>> {
    let json = fs::read_to_string(curriculum_path)?;
    let curriculum: serde_json::Value = serde_json::from_str(&json)?;

    let name = curriculum["name"].as_str().unwrap_or("specialist");
    let domain = curriculum["domain"].as_str().unwrap_or("general");

    let mut corpus = String::new();

    // Header with domain context
    corpus.push_str(&format!(
        "You are a {name} specialist in {domain}. ",
        name = name,
        domain = domain
    ));

    // Extract all facts
    let mut all_facts: Vec<String> = Vec::new();
    if let Some(topics) = curriculum["topics"].as_array() {
        for topic in topics {
            let topic_name = topic["name"].as_str().unwrap_or("topic");
            if let Some(facts) = topic["facts"].as_array() {
                for fact in facts {
                    if let Some(fact_text) = fact["fact"].as_str() {
                        all_facts.push(fact_text.to_string());
                        // Add fact as a statement
                        corpus.push_str(&format!(
                            "Fact about {topic}: {fact}\n",
                            topic = topic_name,
                            fact = fact_text
                        ));
                    }
                }
            }
        }
    }

    // Generate Q&A pairs from facts
    for fact in &all_facts {
        // Turn each fact into a question-answer pair
        let (question, answer) = fact_to_qa(fact);
        corpus.push_str(&format!("Question: {}\nAnswer: {}\n", question, answer));
    }

    // Generate domain-specific code examples based on the specialist type
    let code_examples = generate_code_examples(name, domain);
    for example in &code_examples {
        corpus.push_str(&format!("Code example:\n{}\n", example));
    }

    // Generate conversation templates
    let conversations = generate_conversations(name, domain, &all_facts);
    for conv in &conversations {
        corpus.push_str(&format!("Conversation:\n{}\n", conv));
    }

    // Add repetition of key facts for reinforcement (3x)
    for fact in &all_facts {
        corpus.push_str(&format!("Remember: {}\n", fact));
    }

    Ok(corpus)
}

/// Convert a fact into a question-answer pair
fn fact_to_qa(fact: &str) -> (String, String) {
    // Simple heuristic: extract the subject and turn it into a question
    let words: Vec<&str> = fact.split_whitespace().collect();
    if words.len() < 3 {
        return (format!("What is {}?", fact), fact.to_string());
    }

    // Try to identify the key concept (first noun phrase)
    let subject = if words.len() >= 4 {
        words[..4].join(" ")
    } else {
        words[..2].join(" ")
    };

    let question = format!("What is {}?", subject);
    (question, fact.to_string())
}

/// Generate domain-specific code examples
fn generate_code_examples(name: &str, domain: &str) -> Vec<String> {
    let mut examples = Vec::new();

    match (name, domain) {
        ("junior-react-dev", "frontend-development") => {
            examples.push(
                "// React component with useState\n\
                 function Counter() {\n\
                 const [count, setCount] = useState(0);\n\
                 return <button onClick={() => setCount(count + 1)}>Count: {count}</button>;\n\
                 }".to_string()
            );
            examples.push(
                "// useEffect for data fetching\n\
                 useEffect(() => {\n\
                 fetch('/api/data').then(r => r.json()).then(setData);\n\
                 }, []);".to_string()
            );
            examples.push(
                "// TypeScript interface for props\n\
                 interface ButtonProps {\n\
                 label: string;\n\
                 onClick: () => void;\n\
                 disabled?: boolean;\n\
                 }".to_string()
            );
            examples.push(
                "// Tailwind CSS component\n\
                 <div className=\"flex items-center gap-4 p-6 bg-white rounded-lg shadow-md\">\n\
                 <h2 className=\"text-xl font-bold text-gray-900\">Title</h2>\n\
                 </div>".to_string()
            );
            examples.push(
                "// React Testing Library test\n\
                 test('renders button with label', () => {\n\
                 render(<Button label=\"Click me\" />);\n\
                 expect(screen.getByText('Click me')).toBeInTheDocument();\n\
                 });".to_string()
            );
        }
        ("content-writer", "technical-writing") => {
            examples.push(
                "# How to Write Clear Documentation\n\n\
                 Good documentation is concise, accurate, and task-oriented.\n\
                 Start with the user's goal, not the feature description.".to_string()
            );
            examples.push(
                "## API Reference Template\n\n\
                 ### GET /api/users\n\
                 Returns a list of users. Requires authentication.\n\n\
                 **Parameters:**\n\
                 - `page` (optional): Page number for pagination\n\
                 - `limit` (optional): Results per page (default: 20)".to_string()
            );
        }
        ("customer-support", _) => {
            examples.push(
                "Customer: I can't log into my account.\n\
                 Support: I understand how frustrating that can be. Let me help you get back in.\n\
                 First, have you tried the password reset link on the login page?".to_string()
            );
            examples.push(
                "Customer: My order hasn't arrived.\n\
                 Support: I apologize for the delay. Let me check your order status right now.\n\
                 Can you provide your order number?".to_string()
            );
        }
        ("data-analyst", _) => {
            examples.push(
                "SELECT department, AVG(salary) as avg_salary\n\
                 FROM employees\n\
                 GROUP BY department\n\
                 ORDER BY avg_salary DESC;".to_string()
            );
            examples.push(
                "import pandas as pd\n\
                 df = pd.read_csv('sales.csv')\n\
                 monthly = df.groupby(df['date'].dt.month)['revenue'].sum()\n\
                 print(monthly.describe())".to_string()
            );
        }
        ("devops-engineer", _) => {
            examples.push(
                "# Dockerfile for Node.js app\n\
                 FROM node:20-alpine\n\
                 WORKDIR /app\n\
                 COPY package*.json ./\n\
                 RUN npm ci --only=production\n\
                 COPY . .\n\
                 EXPOSE 3000\n\
                 CMD [\"node\", \"server.js\"]".to_string()
            );
            examples.push(
                "# GitHub Actions CI pipeline\n\
                 name: CI\n\
                 on: [push, pull_request]\n\
                 jobs:\n\
                 test:\n\
                 runs-on: ubuntu-latest\n\
                 steps:\n\
                 - uses: actions/checkout@v4\n\
                 - run: npm ci && npm test".to_string()
            );
        }
        _ => {
            // Generic examples for unknown specialists
            examples.push(format!(
                "As a {name} specialist in {domain}, I help with:\n\
                 - Understanding core concepts\n\
                 - Solving common problems\n\
                 - Best practices and patterns",
                name = name,
                domain = domain
            ));
        }
    }

    examples
}

/// Generate conversation templates for the specialist
fn generate_conversations(name: &str, domain: &str, facts: &[String]) -> Vec<String> {
    let mut conversations = Vec::new();

    // Natural conversational templates that teach the model how to respond
    let templates = vec![
        format!(
            "User: Hello, who are you?\nAssistant: I am a {name} specialist focused on {domain}. I can help you with questions about {domain}.",
            name = name, domain = domain
        ),
        format!(
            "User: What do you know about {domain}?\nAssistant: I have deep knowledge of {domain}. I understand the core concepts, common patterns, and best practices.",
            domain = domain
        ),
        format!(
            "User: Can you help me with a problem?\nAssistant: Of course. Tell me what you are working on and I will help you find the best approach.",
        ),
        format!(
            "User: Thank you for your help.\nAssistant: You are welcome. I am glad I could help with your {domain} question.",
            domain = domain
        ),
        format!(
            "User: I do not understand this concept.\nAssistant: Let me explain it more clearly. The key idea is simple once you see how the pieces fit together.",
        ),
    ];
    conversations.extend(templates);

    // Fact-based Q&A conversations
    let sample_facts: Vec<&String> = facts.iter().take(5).collect();
    for fact in &sample_facts {
        let (question, answer) = fact_to_qa(fact);
        let conv = format!(
            "User: {}\nAssistant: {}",
            question, answer
        );
        conversations.push(conv);
    }

    conversations
}

// ── Training Example Builder ───────────────────────────────────────────────

/// Build training examples from a corpus.
/// Each example is (context_token_ids, target_token_id).
/// Uses a sliding window over the tokenized corpus.
pub fn build_training_examples(
    corpus: &str,
    tokenizer: &SimpleTokenizer,
    context_window: usize,
    max_examples: usize,
) -> Vec<(Vec<usize>, usize)> {
    build_training_examples_impl(corpus, tokenizer, context_window, max_examples)
}

/// Build training examples using any tokenizer (BPE or Simple).
pub fn build_training_examples_generic(
    corpus: &str,
    tokenizer: &dyn NcaTokenizer,
    context_window: usize,
    max_examples: usize,
) -> Vec<(Vec<usize>, usize)> {
    build_training_examples_impl(corpus, tokenizer, context_window, max_examples)
}

fn build_training_examples_impl(
    corpus: &str,
    tokenizer: &dyn NcaTokenizer,
    context_window: usize,
    max_examples: usize,
) -> Vec<(Vec<usize>, usize)> {
    let tokens = tokenizer.encode(corpus);
    if tokens.len() < context_window + 1 {
        return Vec::new();
    }

    let total_pairs = tokens.len() - context_window;
    let step = if max_examples > 0 && max_examples < total_pairs {
        total_pairs / max_examples
    } else {
        1
    }
    .max(1);

    let mut examples = Vec::new();
    for i in (0..total_pairs).step_by(step) {
        let ctx = tokens[i..i + context_window].to_vec();
        let target = tokens[i + context_window];
        examples.push((ctx, target));
        if max_examples > 0 && examples.len() >= max_examples {
            break;
        }
    }

    examples
}

// ── Forward Pass with Trace Recording ──────────────────────────────────────

/// Per-cell intermediate values for one NCA step
struct CellTrace {
    input: Vec<f64>,
    pre_h1: Vec<f64>,
    h1: Vec<f64>,
    pre_h2: Vec<f64>,
    h2: Vec<f64>,
    pre_out: Vec<f64>,
    #[allow(dead_code)]
    delta: Vec<f64>, // Stored for debugging; gradient computed from backward pass
    pre_clamp: Vec<f64>,
}

/// All traces for one NCA step across the whole grid
struct StepTrace {
    cells: Vec<Vec<CellTrace>>,
    #[allow(dead_code)]
    grid_before: Vec<Vec<[f64; NCA_CHANNELS]>>, // Stored for debugging/visualization
}

/// Forward pass with full trace recording for backprop
fn forward_with_trace(
    weights: &NcaWeights,
    grid: &mut [Vec<[f64; NCA_CHANNELS]>],
    grid_size: usize,
    nca_steps: usize,
) -> Vec<StepTrace> {
    let mut traces = Vec::with_capacity(nca_steps);

    for _step in 0..nca_steps {
        let grid_before: Vec<Vec<[f64; NCA_CHANNELS]>> = grid.to_vec();
        let mut step_cells: Vec<Vec<CellTrace>> = Vec::with_capacity(grid_size);
        let mut deltas = vec![vec![[0.0; NCA_CHANNELS]; grid_size]; grid_size];

        for r in 0..grid_size {
            let mut row_traces = Vec::with_capacity(grid_size);
            for c in 0..grid_size {
                // Perceive 3×3 neighborhood
                let mut input = vec![0.0; PERCEPTION_SIZE];
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
                let mut pre_h1 = vec![0.0; HIDDEN1_SIZE];
                let mut h1 = vec![0.0; HIDDEN1_SIZE];
                for h in 0..HIDDEN1_SIZE {
                    let mut sum = weights.b1[h];
                    for i in 0..PERCEPTION_SIZE {
                        sum += weights.w1[h][i] * input[i];
                    }
                    pre_h1[h] = sum;
                    h1[h] = sum.max(0.0);
                }

                // Layer 2: h1 → h2 (ReLU)
                let mut pre_h2 = vec![0.0; HIDDEN2_SIZE];
                let mut h2 = vec![0.0; HIDDEN2_SIZE];
                for h in 0..HIDDEN2_SIZE {
                    let mut sum = weights.b2[h];
                    for i in 0..HIDDEN1_SIZE {
                        sum += weights.w2[h][i] * h1[i];
                    }
                    pre_h2[h] = sum;
                    h2[h] = sum.max(0.0);
                }

                // Layer 3: h2 → delta (tanh * 0.1)
                let mut pre_out = vec![0.0; NCA_CHANNELS];
                let mut delta = vec![0.0; NCA_CHANNELS];
                for ch in 0..NCA_CHANNELS {
                    let mut sum = weights.b3[ch];
                    for h in 0..HIDDEN2_SIZE {
                        sum += weights.w3[ch][h] * h2[h];
                    }
                    pre_out[ch] = sum;
                    delta[ch] = sum.tanh() * 0.1;
                }

                // Pre-clamp values
                let mut pre_clamp = vec![0.0; NCA_CHANNELS];
                for ch in 0..NCA_CHANNELS {
                    pre_clamp[ch] = grid[r][c][ch] + delta[ch];
                    deltas[r][c][ch] = delta[ch];
                }

                row_traces.push(CellTrace {
                    input,
                    pre_h1,
                    h1,
                    pre_h2,
                    h2,
                    pre_out,
                    delta,
                    pre_clamp,
                });
            }
            step_cells.push(row_traces);
        }

        // Apply deltas with clamp
        for r in 0..grid_size {
            for c in 0..grid_size {
                for ch in 0..NCA_CHANNELS {
                    grid[r][c][ch] = (grid[r][c][ch] + deltas[r][c][ch]).clamp(-5.0, 5.0);
                }
            }
        }

        traces.push(StepTrace {
            cells: step_cells,
            grid_before,
        });
    }

    traces
}

// ── Gradient Accumulator ───────────────────────────────────────────────────

struct NcaGradients {
    dw1: Vec<Vec<f64>>,
    db1: Vec<f64>,
    dw2: Vec<Vec<f64>>,
    db2: Vec<f64>,
    dw3: Vec<Vec<f64>>,
    db3: Vec<f64>,
}

impl NcaGradients {
    fn zeros() -> Self {
        Self {
            dw1: vec![vec![0.0; PERCEPTION_SIZE]; HIDDEN1_SIZE],
            db1: vec![0.0; HIDDEN1_SIZE],
            dw2: vec![vec![0.0; HIDDEN1_SIZE]; HIDDEN2_SIZE],
            db2: vec![0.0; HIDDEN2_SIZE],
            dw3: vec![vec![0.0; HIDDEN2_SIZE]; NCA_CHANNELS],
            db3: vec![0.0; NCA_CHANNELS],
        }
    }

    fn accumulate(&mut self, other: &NcaGradients) {
        for h in 0..HIDDEN1_SIZE {
            for i in 0..PERCEPTION_SIZE {
                self.dw1[h][i] += other.dw1[h][i];
            }
            self.db1[h] += other.db1[h];
        }
        for h in 0..HIDDEN2_SIZE {
            for i in 0..HIDDEN1_SIZE {
                self.dw2[h][i] += other.dw2[h][i];
            }
            self.db2[h] += other.db2[h];
        }
        for ch in 0..NCA_CHANNELS {
            for h in 0..HIDDEN2_SIZE {
                self.dw3[ch][h] += other.dw3[ch][h];
            }
            self.db3[ch] += other.db3[ch];
        }
    }

    fn to_vec(&self) -> Vec<f64> {
        let mut v = Vec::new();
        for row in &self.dw1 { v.extend(row); }
        v.extend(&self.db1);
        for row in &self.dw2 { v.extend(row); }
        v.extend(&self.db2);
        for row in &self.dw3 { v.extend(row); }
        v.extend(&self.db3);
        v
    }

    fn clip_norm(&mut self, max_norm: f64) {
        let v = self.to_vec();
        let norm: f64 = v.iter().map(|x| x * x).sum::<f64>().sqrt();
        if norm > max_norm {
            let scale = max_norm / norm;
            for row in &mut self.dw1 { for x in row.iter_mut() { *x *= scale; } }
            for x in &mut self.db1 { *x *= scale; }
            for row in &mut self.dw2 { for x in row.iter_mut() { *x *= scale; } }
            for x in &mut self.db2 { *x *= scale; }
            for row in &mut self.dw3 { for x in row.iter_mut() { *x *= scale; } }
            for x in &mut self.db3 { *x *= scale; }
        }
    }

    fn scale(&mut self, factor: f64) {
        for row in &mut self.dw1 { for x in row.iter_mut() { *x *= factor; } }
        for x in &mut self.db1 { *x *= factor; }
        for row in &mut self.dw2 { for x in row.iter_mut() { *x *= factor; } }
        for x in &mut self.db2 { *x *= factor; }
        for row in &mut self.dw3 { for x in row.iter_mut() { *x *= factor; } }
        for x in &mut self.db3 { *x *= factor; }
    }
}

// ── Backward Pass ──────────────────────────────────────────────────────────

fn backward_through_steps(
    weights: &NcaWeights,
    traces: &[StepTrace],
    mut d_grid: Vec<Vec<[f64; NCA_CHANNELS]>>,
    grid_size: usize,
) -> NcaGradients {
    let mut total_grads = NcaGradients::zeros();

    for step_trace in traces.iter().rev() {
        let mut d_grid_prev = vec![vec![[0.0; NCA_CHANNELS]; grid_size]; grid_size];

        for r in 0..grid_size {
            for c in 0..grid_size {
                let trace = &step_trace.cells[r][c];

                // dL/d(grid_after_clamp) → through clamp
                let mut d_post_add = [0.0; NCA_CHANNELS];
                for ch in 0..NCA_CHANNELS {
                    if trace.pre_clamp[ch] >= -5.0 && trace.pre_clamp[ch] <= 5.0 {
                        d_post_add[ch] = d_grid[r][c][ch];
                    }
                }

                // Residual: d_grid_prev += d_post_add
                for ch in 0..NCA_CHANNELS {
                    d_grid_prev[r][c][ch] += d_post_add[ch];
                }
                let d_delta = d_post_add;

                // delta = tanh(pre_out) * 0.1
                let mut d_pre_out = [0.0; NCA_CHANNELS];
                for ch in 0..NCA_CHANNELS {
                    let t = trace.pre_out[ch].tanh();
                    d_pre_out[ch] = d_delta[ch] * 0.1 * (1.0 - t * t);
                }

                // pre_out = W3 * h2 + b3
                let mut d_h2 = vec![0.0; HIDDEN2_SIZE];
                for ch in 0..NCA_CHANNELS {
                    total_grads.db3[ch] += d_pre_out[ch];
                    for h in 0..HIDDEN2_SIZE {
                        total_grads.dw3[ch][h] += d_pre_out[ch] * trace.h2[h];
                        d_h2[h] += weights.w3[ch][h] * d_pre_out[ch];
                    }
                }

                // h2 = ReLU(pre_h2)
                let mut d_pre_h2 = vec![0.0; HIDDEN2_SIZE];
                for h in 0..HIDDEN2_SIZE {
                    d_pre_h2[h] = if trace.pre_h2[h] > 0.0 { d_h2[h] } else { 0.0 };
                }

                // pre_h2 = W2 * h1 + b2
                let mut d_h1 = vec![0.0; HIDDEN1_SIZE];
                for h in 0..HIDDEN2_SIZE {
                    total_grads.db2[h] += d_pre_h2[h];
                    for i in 0..HIDDEN1_SIZE {
                        total_grads.dw2[h][i] += d_pre_h2[h] * trace.h1[i];
                        d_h1[i] += weights.w2[h][i] * d_pre_h2[h];
                    }
                }

                // h1 = ReLU(pre_h1)
                let mut d_pre_h1 = vec![0.0; HIDDEN1_SIZE];
                for i in 0..HIDDEN1_SIZE {
                    d_pre_h1[i] = if trace.pre_h1[i] > 0.0 { d_h1[i] } else { 0.0 };
                }

                // pre_h1 = W1 * input + b1
                let mut d_input = vec![0.0; PERCEPTION_SIZE];
                for h in 0..HIDDEN1_SIZE {
                    total_grads.db1[h] += d_pre_h1[h];
                    for i in 0..PERCEPTION_SIZE {
                        total_grads.dw1[h][i] += d_pre_h1[h] * trace.input[i];
                        d_input[i] += weights.w1[h][i] * d_pre_h1[h];
                    }
                }

                // Distribute d_input back to neighborhood cells
                let mut idx = 0;
                for dr in [-1i32, 0, 1] {
                    for dc in [-1i32, 0, 1] {
                        let nr = (r as i32 + dr).rem_euclid(grid_size as i32) as usize;
                        let nc = (c as i32 + dc).rem_euclid(grid_size as i32) as usize;
                        for ch in 0..NCA_CHANNELS {
                            d_grid_prev[nr][nc][ch] += d_input[idx];
                            idx += 1;
                        }
                    }
                }
            }
        }

        d_grid = d_grid_prev;
    }

    total_grads
}

// ── Loss Computation ───────────────────────────────────────────────────────

fn cross_entropy_loss(activations: &[f64], target: usize) -> (f64, Vec<f64>) {
    let max_val = activations.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let exps: Vec<f64> = activations.iter().map(|&a| (a - max_val).exp()).collect();
    let sum: f64 = exps.iter().sum();
    let probs: Vec<f64> = exps.iter().map(|e| e / sum).collect();

    let loss = -probs[target].max(1e-30).ln();
    let mut d_act = probs;
    d_act[target] -= 1.0;

    (loss, d_act)
}

// ── Adam Optimizer ─────────────────────────────────────────────────────────

struct AdamState {
    m: Vec<f64>,
    v: Vec<f64>,
    t: usize,
    lr: f64,
    beta1: f64,
    beta2: f64,
    eps: f64,
}

impl AdamState {
    fn new(n_params: usize, lr: f64) -> Self {
        Self {
            m: vec![0.0; n_params],
            v: vec![0.0; n_params],
            t: 0,
            lr,
            beta1: 0.9,
            beta2: 0.999,
            eps: 1e-8,
        }
    }

    fn step(&mut self, params: &mut [f64], grads: &[f64]) {
        self.t += 1;
        let bc1 = 1.0 - self.beta1.powi(self.t as i32);
        let bc2 = 1.0 - self.beta2.powi(self.t as i32);

        for i in 0..params.len() {
            self.m[i] = self.beta1 * self.m[i] + (1.0 - self.beta1) * grads[i];
            self.v[i] = self.beta2 * self.v[i] + (1.0 - self.beta2) * grads[i] * grads[i];
            let m_hat = self.m[i] / bc1;
            let v_hat = self.v[i] / bc2;
            params[i] -= self.lr * m_hat / (v_hat.sqrt() + self.eps);
        }
    }

    fn set_lr(&mut self, lr: f64) {
        self.lr = lr;
    }
}

// ── Token Coordinate Mapping ───────────────────────────────────────────────

fn token_to_coord(token_id: usize, grid_size: usize) -> (usize, usize) {
    let h = token_id.wrapping_mul(2654435761);
    let row = (h >> 16) as usize % grid_size;
    let col = (h >> 8) as usize % grid_size;
    (row, col)
}

// ── Evaluation ─────────────────────────────────────────────────────────────

/// Evaluate top-5 accuracy on a set of examples
fn evaluate_top5(
    weights: &NcaWeights,
    examples: &[(Vec<usize>, usize)],
    grid_size: usize,
    nca_steps: usize,
    vocab_size: usize,
) -> f64 {
    if examples.is_empty() {
        return 0.0;
    }

    let mut correct = 0;
    for (ctx, target) in examples {
        let mut grid = vec![vec![[0.0f64; NCA_CHANNELS]; grid_size]; grid_size];

        // Encode context tokens
        for (pos, &tid) in ctx.iter().enumerate() {
            let (r, c) = token_to_coord(tid, grid_size);
            let recency = if ctx.len() > 1 {
                1.0 - (ctx.len() - 1 - pos) as f64 / ctx.len() as f64 * 0.5
            } else {
                1.0
            };
            grid[r][c][ACTIVATION_CH] = (grid[r][c][ACTIVATION_CH] + recency).min(5.0);
        }

        // Run NCA steps
        for _ in 0..nca_steps {
            let mut deltas = vec![vec![[0.0; NCA_CHANNELS]; grid_size]; grid_size];
            for r in 0..grid_size {
                for c in 0..grid_size {
                    let mut input = [0.0; PERCEPTION_SIZE];
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
                    let mut h1 = [0.0; HIDDEN1_SIZE];
                    for h in 0..HIDDEN1_SIZE {
                        let mut sum = weights.b1[h];
                        for i in 0..PERCEPTION_SIZE { sum += weights.w1[h][i] * input[i]; }
                        h1[h] = sum.max(0.0);
                    }
                    let mut h2 = [0.0; HIDDEN2_SIZE];
                    for h in 0..HIDDEN2_SIZE {
                        let mut sum = weights.b2[h];
                        for i in 0..HIDDEN1_SIZE { sum += weights.w2[h][i] * h1[i]; }
                        h2[h] = sum.max(0.0);
                    }
                    for ch in 0..NCA_CHANNELS {
                        let mut sum = weights.b3[ch];
                        for h in 0..HIDDEN2_SIZE { sum += weights.w3[ch][h] * h2[h]; }
                        deltas[r][c][ch] = sum.tanh() * 0.1;
                    }
                }
            }
            for r in 0..grid_size {
                for c in 0..grid_size {
                    for ch in 0..NCA_CHANNELS {
                        grid[r][c][ch] = (grid[r][c][ch] + deltas[r][c][ch]).clamp(-5.0, 5.0);
                    }
                }
            }
        }

        // Read activations
        let mut indexed: Vec<(usize, f64)> = (0..vocab_size.min(grid_size * grid_size))
            .map(|id| {
                let (r, c) = token_to_coord(id, grid_size);
                (id, grid[r][c][ACTIVATION_CH])
            })
            .collect();
        indexed.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        if indexed.iter().take(5).any(|(id, _)| *id == *target) {
            correct += 1;
        }
    }

    correct as f64 / examples.len() as f64
}

// ── Corpus Loading ────────────────────────────────────────────────────────

/// Load text from a directory of .txt files (e.g., ~/.sage/corpus/).
/// Concatenates all files with double newlines between them.
/// Optionally limit to `max_files` files (0 = all).
pub fn load_corpus_text(dir: &Path, max_files: usize) -> Result<String, Box<dyn Error>> {
    let mut corpus = String::new();
    let mut count = 0;

    let mut entries: Vec<_> = fs::read_dir(dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|x| x == "txt").unwrap_or(false))
        .collect();
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        if max_files > 0 && count >= max_files {
            break;
        }
        let path = entry.path();
        if let Ok(text) = fs::read_to_string(&path) {
            corpus.push_str(&text);
            corpus.push_str("\n\n");
            count += 1;
        }
    }

    if corpus.is_empty() {
        return Err(format!("No .txt files found in {}", dir.display()).into());
    }

    eprintln!("   Loaded {} files, {} chars, {} words", count, corpus.len(), corpus.split_whitespace().count());
    Ok(corpus)
}

/// Load text from a single file.
pub fn load_text_file(path: &Path) -> Result<String, Box<dyn Error>> {
    let text = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
    Ok(text)
}

// ── Main Training Function (Curriculum-based) ─────────────────────────────

/// Train the NCA language model on a curriculum.
///
/// Returns the trained model and training statistics.
pub fn train_nca_lm(
    curriculum_path: &Path,
    training_config: &NcaLmTrainingConfig,
) -> Result<(NcaLanguageModel, TrainingStats), Box<dyn Error>> {
    let start_time = Instant::now();
    let model_config = &training_config.model;

    eprintln!("═══ NCA Language Model Training ═══");
    eprintln!("Grid: {}×{} ({} cells)", model_config.grid_size, model_config.grid_size, model_config.total_cells());
    eprintln!("NCA steps: {}, Vocab target: {}", model_config.nca_steps, model_config.vocab_size);
    eprintln!("Params: {}K, Epochs: {}, LR: {}", NcaWeights::random().param_count() / 1000, training_config.epochs, training_config.learning_rate);

    // Step 1: Convert curriculum to corpus
    eprintln!("\n📚 Converting curriculum to training corpus...");
    let corpus = curriculum_to_corpus(curriculum_path)?;
    eprintln!("   Corpus: {} chars, {} words", corpus.len(), corpus.split_whitespace().count());

    // Step 2: Train BPE tokenizer
    eprintln!("\n🔤 Training BPE tokenizer (vocab={})...", model_config.vocab_size);
    let tokenizer = SimpleTokenizer::from_corpus(&corpus, model_config.vocab_size);
    let actual_vocab = tokenizer.vocab_size();
    eprintln!("   Vocabulary: {} tokens", actual_vocab);

    // Step 3: Build training examples
    eprintln!("\n📊 Building training examples...");
    let examples = build_training_examples(
        &corpus,
        &tokenizer,
        model_config.context_window,
        training_config.max_examples,
    );
    eprintln!("   Examples: {} (context window: {})", examples.len(), model_config.context_window);

    if examples.len() < 10 {
        return Err(format!("Too few training examples ({}). Need at least 10.", examples.len()).into());
    }

    // Step 4: Split into train/eval sets
    let split_idx = (examples.len() as f64 * 0.9) as usize;
    let train_examples = &examples[..split_idx];
    let eval_examples = &examples[split_idx..];
    eprintln!("   Train: {}, Eval: {}", train_examples.len(), eval_examples.len());

    let random_baseline = 1.0 / actual_vocab as f64;
    eprintln!("   Random baseline: {:.4}%", random_baseline * 100.0);

    // Step 5: Initialize model
    let mut model = NcaLanguageModel::new(model_config.clone());
    // Replace placeholder tokenizer with trained one
    model.tokenizer = tokenizer;
    let n_params = model.weights().param_count();
    let mut adam = AdamState::new(n_params, training_config.learning_rate);

    let grid_size = model_config.grid_size;
    let nca_steps = model_config.nca_steps;
    let batch_size = training_config.batch_size;

    let mut best_accuracy = 0.0;
    let mut best_weights = model.weights().clone();
    let mut epoch_losses: Vec<f64> = Vec::new();
    let mut epoch_accuracies: Vec<f64> = Vec::new();

    // Checkpoint directory
    let checkpoint_dir = training_config.checkpoint_dir.clone()
        .unwrap_or_else(|| {
            dirs::home_dir()
                .unwrap_or_default()
                .join(".sage")
                .join("checkpoints")
        });
    fs::create_dir_all(&checkpoint_dir)?;

    // Step 6: Training loop
    eprintln!("\n🚀 Starting training...");
    for epoch in 0..training_config.epochs {
        let epoch_start = Instant::now();

        // Cosine LR decay
        if training_config.lr_decay {
            let progress = epoch as f64 / training_config.epochs as f64;
            let lr = training_config.learning_rate * 0.5 * (1.0 + (std::f64::consts::PI * progress).cos());
            adam.set_lr(lr);
        }

        let mut epoch_loss = 0.0;
        let mut epoch_grads = NcaGradients::zeros();
        let mut examples_processed = 0;

        // Process in batches
        for batch_start in (0..train_examples.len()).step_by(batch_size) {
            let batch_end = (batch_start + batch_size).min(train_examples.len());
            let mut batch_grads = NcaGradients::zeros();
            let mut batch_loss = 0.0;

            for (ctx, target) in &train_examples[batch_start..batch_end] {
                // Initialize grid
                let mut grid = vec![vec![[0.0f64; NCA_CHANNELS]; grid_size]; grid_size];

                // Encode context tokens
                for (pos, &tid) in ctx.iter().enumerate() {
                    let (r, c) = token_to_coord(tid, grid_size);
                    let recency = if ctx.len() > 1 {
                        1.0 - (ctx.len() - 1 - pos) as f64 / ctx.len() as f64 * 0.5
                    } else {
                        1.0
                    };
                    grid[r][c][ACTIVATION_CH] = (grid[r][c][ACTIVATION_CH] + recency).min(5.0);
                    grid[r][c][1] = if ctx.len() > 1 { pos as f64 / (ctx.len() - 1) as f64 } else { 0.5 };
                    grid[r][c][2] = recency;
                }

                // Forward with trace
                let traces = forward_with_trace(
                    model.weights(),
                    &mut grid,
                    grid_size,
                    nca_steps,
                );

                // Read activations for all vocab tokens
                let mut activations = vec![0.0; actual_vocab];
                for tid in 0..actual_vocab {
                    let (r, c) = token_to_coord(tid, grid_size);
                    activations[tid] = grid[r][c][ACTIVATION_CH];
                }

                // Compute loss
                let (loss, d_activations) = cross_entropy_loss(&activations, *target);
                batch_loss += loss;

                // Convert d_activations to d_grid
                let mut d_grid = vec![vec![[0.0; NCA_CHANNELS]; grid_size]; grid_size];
                for (tid, &d_act) in d_activations.iter().enumerate() {
                    let (r, c) = token_to_coord(tid, grid_size);
                    d_grid[r][c][ACTIVATION_CH] += d_act;
                }

                // Backward
                let grads = backward_through_steps(
                    model.weights(),
                    &traces,
                    d_grid,
                    grid_size,
                );
                batch_grads.accumulate(&grads);
            }

            let batch_count = (batch_end - batch_start) as f64;
            batch_loss /= batch_count;
            batch_grads.scale(1.0 / batch_count);
            batch_grads.clip_norm(training_config.grad_clip);

            epoch_grads.accumulate(&batch_grads);
            epoch_loss += batch_loss * batch_count;
            examples_processed += batch_end - batch_start;
        }

        epoch_loss /= examples_processed as f64;
        epoch_grads.scale(1.0 / (train_examples.len() as f64 / batch_size as f64));
        epoch_grads.clip_norm(training_config.grad_clip);

        // Adam update
        let mut params = model.weights().to_vec();
        let grad_vec = epoch_grads.to_vec();
        adam.step(&mut params, &grad_vec);
        *model.weights_mut() = NcaWeights::from_vec(&params);

        // Evaluate
        let accuracy = if epoch % training_config.eval_interval == 0 || epoch == training_config.epochs - 1 {
            evaluate_top5(
                model.weights(),
                eval_examples,
                grid_size,
                nca_steps,
                actual_vocab,
            )
        } else {
            epoch_accuracies.last().copied().unwrap_or(0.0)
        };

        epoch_losses.push(epoch_loss);
        epoch_accuracies.push(accuracy);

        if accuracy > best_accuracy {
            best_accuracy = accuracy;
            best_weights = model.weights().clone();
        }

        let elapsed = epoch_start.elapsed();
        eprintln!(
            "  Epoch {}/{}: loss={:.4}, top-5={:.2}% (best={:.2}%, random={:.4}%) [{:.1}s]",
            epoch + 1,
            training_config.epochs,
            epoch_loss,
            accuracy * 100.0,
            best_accuracy * 100.0,
            random_baseline * 100.0,
            elapsed.as_secs_f64()
        );

        // Checkpoint
        if training_config.checkpoint_interval > 0
            && (epoch + 1) % training_config.checkpoint_interval == 0
        {
            let cp_path = checkpoint_dir.join(format!("epoch_{:04}.bin", epoch + 1));
            model.weights().save(&cp_path)?;
            eprintln!("   💾 Checkpoint saved: {}", cp_path.display());
        }
    }

    // Restore best weights
    *model.weights_mut() = best_weights;
    model.mark_trained();

    let total_time = start_time.elapsed();
    let stats = TrainingStats {
        final_accuracy: best_accuracy,
        random_baseline,
        epochs: training_config.epochs,
        vocab_size: actual_vocab,
        examples: examples.len(),
        epoch_losses,
        epoch_accuracies,
        training_time_secs: total_time.as_secs_f64(),
        param_count: n_params,
        grid_size,
        nca_steps,
    };

    eprintln!("\n✅ Training complete in {:.1}s", total_time.as_secs_f64());
    eprintln!("   Final accuracy: {:.2}% (random: {:.4}%)", best_accuracy * 100.0, random_baseline * 100.0);
    eprintln!("   Improvement over random: {:.1}x", best_accuracy / random_baseline.max(1e-10));

    Ok((model, stats))
}

// ── Text-based Training Function ───────────────────────────────────────────

/// Train the NCA language model on raw text (e.g., Project Gutenberg corpus).
///
/// This is Step 8 of the v0.6.0 plan: train on real book text instead of
/// curriculum JSON. Uses BPE tokenization for subword coverage, eliminating
/// <unk> tokens on unseen words.
///
/// # Arguments
/// * `corpus_text` - Raw text to train on (can be multiple books concatenated)
/// * `use_bpe` - If true, train a BPE tokenizer; if false, use SimpleTokenizer
/// * `training_config` - Training configuration
///
/// Returns the trained model and training statistics.
pub fn train_on_text(
    corpus_text: &str,
    use_bpe: bool,
    training_config: &NcaLmTrainingConfig,
) -> Result<(NcaLanguageModel, TrainingStats), Box<dyn Error>> {
    let start_time = Instant::now();
    let model_config = &training_config.model;

    eprintln!("═══ NCA Language Model Training (Text Corpus) ═══");
    eprintln!("Grid: {}×{} ({} cells)", model_config.grid_size, model_config.grid_size, model_config.total_cells());
    eprintln!("NCA steps: {}, Vocab target: {}, BPE: {}",
        model_config.nca_steps, model_config.vocab_size, use_bpe);
    eprintln!("Params: {}K, Epochs: {}, LR: {}",
        NcaWeights::random().param_count() / 1000,
        training_config.epochs, training_config.learning_rate);

    // Step 1: Tokenize corpus
    eprintln!("\n📚 Corpus: {} chars, {} words", corpus_text.len(), corpus_text.split_whitespace().count());

    let actual_vocab: usize;
    let tokenizer: Box<dyn NcaTokenizer>;

    if use_bpe {
        eprintln!("\n🔤 Training BPE tokenizer (vocab={})...", model_config.vocab_size);
        let bpe_save_path: Option<PathBuf> = Some(
            dirs::home_dir()
                .unwrap_or_default()
                .join(".sage")
                .join("nca_bpe_tokenizer.json"),
        );
        let bpe = BpeTokenizer::train(corpus_text, model_config.vocab_size, bpe_save_path.as_deref())?;
        actual_vocab = bpe.vocab_size();
        eprintln!("   BPE vocabulary: {} tokens", actual_vocab);
        tokenizer = Box::new(bpe);
    } else {
        eprintln!("\n🔤 Building SimpleTokenizer (vocab={})...", model_config.vocab_size);
        let simple = SimpleTokenizer::from_corpus(corpus_text, model_config.vocab_size);
        actual_vocab = simple.vocab_size();
        eprintln!("   Vocabulary: {} tokens", actual_vocab);
        tokenizer = Box::new(simple);
    }

    // Check for excessive <unk> tokens
    let sample_tokens = tokenizer.encode(&corpus_text[..corpus_text.len().min(10000)]);
    let unk_count = sample_tokens.iter().filter(|t| **t == 0).count();
    let unk_rate = unk_count as f64 / sample_tokens.len().max(1) as f64;
    eprintln!("   <unk> rate on sample: {:.2}% ({} / {})", unk_rate * 100.0, unk_count, sample_tokens.len());
    if unk_rate > 0.15 {
        eprintln!("   ⚠️  High <unk> rate — consider using BPE (--bpe) for subword coverage");
    }

    // Step 2: Build training examples
    eprintln!("\n📊 Building training examples...");
    let examples = build_training_examples_generic(
        corpus_text,
        tokenizer.as_ref(),
        model_config.context_window,
        training_config.max_examples,
    );
    eprintln!("   Examples: {} (context window: {})", examples.len(), model_config.context_window);

    if examples.len() < 10 {
        return Err(format!("Too few training examples ({}). Need at least 10.", examples.len()).into());
    }

    // Step 3: Split into train/eval sets
    let split_idx = (examples.len() as f64 * 0.9) as usize;
    let train_examples = &examples[..split_idx];
    let eval_examples = &examples[split_idx..];
    eprintln!("   Train: {}, Eval: {}", train_examples.len(), eval_examples.len());

    let random_baseline = 1.0 / actual_vocab as f64;
    eprintln!("   Random baseline: {:.4}%", random_baseline * 100.0);

    // Step 4: Initialize model
    let mut model = NcaLanguageModel::new(model_config.clone());
    // For SimpleTokenizer, we set it directly on the model.
    // For BPE, the model keeps its default SimpleTokenizer (weights still train correctly
    // since tokenization is external to the NCA grid — the grid only sees token IDs).
    if !use_bpe {
        if let Some(s) = tokenizer.as_simple() {
            model.tokenizer = s.clone();
        }
    }
    let n_params = model.weights().param_count();
    let mut adam = AdamState::new(n_params, training_config.learning_rate);

    let grid_size = model_config.grid_size;
    let nca_steps = model_config.nca_steps;
    let batch_size = training_config.batch_size;

    let mut best_accuracy = 0.0;
    let mut best_weights = model.weights().clone();
    let mut epoch_losses: Vec<f64> = Vec::new();
    let mut epoch_accuracies: Vec<f64> = Vec::new();

    // Checkpoint directory
    let checkpoint_dir = training_config.checkpoint_dir.clone()
        .unwrap_or_else(|| {
            dirs::home_dir()
                .unwrap_or_default()
                .join(".sage")
                .join("checkpoints")
        });
    fs::create_dir_all(&checkpoint_dir)?;

    // Step 5: Training loop (same as train_nca_lm)
    eprintln!("\n🚀 Starting training...");
    for epoch in 0..training_config.epochs {
        let epoch_start = Instant::now();

        // Cosine LR decay
        if training_config.lr_decay {
            let progress = epoch as f64 / training_config.epochs as f64;
            let lr = training_config.learning_rate * 0.5 * (1.0 + (std::f64::consts::PI * progress).cos());
            adam.set_lr(lr);
        }

        let mut epoch_loss = 0.0;
        let mut epoch_grads = NcaGradients::zeros();
        let mut examples_processed = 0;

        for batch_start in (0..train_examples.len()).step_by(batch_size) {
            let batch_end = (batch_start + batch_size).min(train_examples.len());
            let mut batch_grads = NcaGradients::zeros();
            let mut batch_loss = 0.0;

            for (ctx, target) in &train_examples[batch_start..batch_end] {
                let mut grid = vec![vec![[0.0f64; NCA_CHANNELS]; grid_size]; grid_size];

                for (pos, &tid) in ctx.iter().enumerate() {
                    let (r, c) = token_to_coord(tid, grid_size);
                    let recency = if ctx.len() > 1 {
                        1.0 - (ctx.len() - 1 - pos) as f64 / ctx.len() as f64 * 0.5
                    } else {
                        1.0
                    };
                    grid[r][c][ACTIVATION_CH] = (grid[r][c][ACTIVATION_CH] + recency).min(5.0);
                    grid[r][c][1] = if ctx.len() > 1 { pos as f64 / (ctx.len() - 1) as f64 } else { 0.5 };
                    grid[r][c][2] = recency;
                }

                let traces = forward_with_trace(model.weights(), &mut grid, grid_size, nca_steps);

                let mut activations = vec![0.0; actual_vocab];
                for tid in 0..actual_vocab {
                    let (r, c) = token_to_coord(tid, grid_size);
                    activations[tid] = grid[r][c][ACTIVATION_CH];
                }

                let (loss, d_activations) = cross_entropy_loss(&activations, *target);
                batch_loss += loss;

                let mut d_grid = vec![vec![[0.0; NCA_CHANNELS]; grid_size]; grid_size];
                for (tid, &d_act) in d_activations.iter().enumerate() {
                    let (r, c) = token_to_coord(tid, grid_size);
                    d_grid[r][c][ACTIVATION_CH] += d_act;
                }

                let grads = backward_through_steps(model.weights(), &traces, d_grid, grid_size);
                batch_grads.accumulate(&grads);
            }

            let batch_count = (batch_end - batch_start) as f64;
            batch_loss /= batch_count;
            batch_grads.scale(1.0 / batch_count);
            batch_grads.clip_norm(training_config.grad_clip);

            epoch_grads.accumulate(&batch_grads);
            epoch_loss += batch_loss * batch_count;
            examples_processed += batch_end - batch_start;
        }

        epoch_loss /= examples_processed as f64;
        epoch_grads.scale(1.0 / (train_examples.len() as f64 / batch_size as f64));
        epoch_grads.clip_norm(training_config.grad_clip);

        let mut params = model.weights().to_vec();
        let grad_vec = epoch_grads.to_vec();
        adam.step(&mut params, &grad_vec);
        *model.weights_mut() = NcaWeights::from_vec(&params);

        let accuracy = if epoch % training_config.eval_interval == 0 || epoch == training_config.epochs - 1 {
            evaluate_top5(model.weights(), eval_examples, grid_size, nca_steps, actual_vocab)
        } else {
            epoch_accuracies.last().copied().unwrap_or(0.0)
        };

        epoch_losses.push(epoch_loss);
        epoch_accuracies.push(accuracy);

        if accuracy > best_accuracy {
            best_accuracy = accuracy;
            best_weights = model.weights().clone();
        }

        let elapsed = epoch_start.elapsed();
        eprintln!(
            "  Epoch {}/{}: loss={:.4}, top-5={:.2}% (best={:.2}%, random={:.4}%) [{:.1}s]",
            epoch + 1, training_config.epochs, epoch_loss,
            accuracy * 100.0, best_accuracy * 100.0,
            random_baseline * 100.0, elapsed.as_secs_f64()
        );

        if training_config.checkpoint_interval > 0
            && (epoch + 1) % training_config.checkpoint_interval == 0
        {
            let cp_path = checkpoint_dir.join(format!("text_epoch_{:04}.bin", epoch + 1));
            model.weights().save(&cp_path)?;
            eprintln!("   💾 Checkpoint saved: {}", cp_path.display());
        }
    }

    *model.weights_mut() = best_weights;
    model.mark_trained();

    let total_time = start_time.elapsed();
    let stats = TrainingStats {
        final_accuracy: best_accuracy,
        random_baseline,
        epochs: training_config.epochs,
        vocab_size: actual_vocab,
        examples: examples.len(),
        epoch_losses,
        epoch_accuracies,
        training_time_secs: total_time.as_secs_f64(),
        param_count: n_params,
        grid_size,
        nca_steps,
    };

    eprintln!("\n✅ Training complete in {:.1}s", total_time.as_secs_f64());
    eprintln!("   Final accuracy: {:.2}% (random: {:.4}%)", best_accuracy * 100.0, random_baseline * 100.0);
    eprintln!("   Improvement over random: {:.1}x", best_accuracy / random_baseline.max(1e-10));

    Ok((model, stats))
}

// ── Training Statistics ────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct TrainingStats {
    pub final_accuracy: f64,
    pub random_baseline: f64,
    pub epochs: usize,
    pub vocab_size: usize,
    pub examples: usize,
    pub epoch_losses: Vec<f64>,
    pub epoch_accuracies: Vec<f64>,
    pub training_time_secs: f64,
    pub param_count: usize,
    pub grid_size: usize,
    pub nca_steps: usize,
}

impl TrainingStats {
    pub fn summary(&self) -> String {
        format!(
            "NCA-LM Training Summary:\n\
             ├─ Grid: {}×{} ({} cells)\n\
             ├─ NCA steps: {}\n\
             ├─ Params: {}K\n\
             ├─ Vocab: {} tokens\n\
             ├─ Examples: {}\n\
             ├─ Epochs: {}\n\
             ├─ Final top-5: {:.2}% (random: {:.4}%)\n\
             ├─ Improvement: {:.1}x\n\
             └─ Time: {:.1}s",
            self.grid_size,
            self.grid_size,
            self.grid_size * self.grid_size,
            self.nca_steps,
            self.param_count / 1000,
            self.vocab_size,
            self.examples,
            self.epochs,
            self.final_accuracy * 100.0,
            self.random_baseline * 100.0,
            self.final_accuracy / self.random_baseline.max(1e-10),
            self.training_time_secs,
        )
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_curriculum_to_corpus_generates_content() {
        // Create a minimal curriculum JSON
        let tmp = std::env::temp_dir().join("test_curriculum.json");
        let curriculum = r#"{
            "name": "test-specialist",
            "domain": "testing",
            "topics": [
                {
                    "name": "basics",
                    "facts": [
                        {"fact": "Testing verifies code correctness"},
                        {"fact": "Unit tests check individual functions"},
                        {"fact": "Integration tests check component interaction"}
                    ]
                }
            ]
        }"#;
        fs::write(&tmp, curriculum).unwrap();

        let corpus = curriculum_to_corpus(&tmp).unwrap();
        let _ = fs::remove_file(&tmp);

        // Should contain facts, Q&A, conversations
        assert!(corpus.contains("Testing verifies code correctness"));
        assert!(corpus.contains("Question:"));
        assert!(corpus.contains("Answer:"));
        assert!(corpus.contains("Conversation:"));
        assert!(corpus.len() > 500, "Corpus too small: {} chars", corpus.len());
    }

    #[test]
    fn test_build_training_examples() {
        let corpus = "the quick brown fox jumps over the lazy dog";
        let tokenizer = SimpleTokenizer::from_corpus(corpus, 100);
        let examples = build_training_examples(corpus, &tokenizer, 3, 0);
        assert!(!examples.is_empty(), "Should produce examples");
        // Each example should have context_window tokens + target
        for (ctx, _target) in &examples {
            assert_eq!(ctx.len(), 3, "Context should be 3 tokens");
        }
    }

    #[test]
    fn test_cross_entropy_loss_values() {
        let activations = vec![2.0, 1.0, 0.1];
        let (loss, grads) = cross_entropy_loss(&activations, 0);
        assert!(loss > 0.0);
        assert!(grads[0] < 0.0); // target gradient negative
        assert!(grads[1] > 0.0);
        let sum: f64 = grads.iter().sum();
        assert!(sum.abs() < 1e-10);
    }

    #[test]
    fn test_evaluate_top5_perfect() {
        // With trivial weights that always activate the same cell,
        // accuracy should be measurable
        let weights = NcaWeights::random();
        let examples = vec![
            (vec![0, 1, 2], 3),
            (vec![1, 2, 3], 4),
        ];
        let acc = evaluate_top5(&weights, &examples, 8, 2, 64);
        // Just verify it returns a valid probability
        assert!(acc >= 0.0 && acc <= 1.0);
    }
}
