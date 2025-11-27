# SAGE Language Learning Architecture

## Research Summary

After researching approaches to language learning in neural systems, I've synthesized findings from:
- [Neural Cellular Automata for Spatio-Temporal Patterns](https://pmc.ncbi.nlm.nih.gov/articles/PMC11078362/)
- [Reservoir Computing as a Language Model](https://arxiv.org/abs/2507.15779)
- [Grounded Language Learning](https://arxiv.org/html/2312.02431)
- [Echo State Networks for Language](https://arxiv.org/html/2503.01724)
- [Growing Neural Cellular Automata](https://distill.pub/2020/growing-ca/)

### Key Insights

1. **NCAs haven't been directly used for NLP** - but the principles apply. NCAs excel at learning spatiotemporal dynamics, and language is fundamentally sequential/temporal.

2. **Reservoir Computing works for language** - Use a fixed dynamical system as a "reservoir" that transforms input into high-dimensional representations, then train only a simple readout layer. Recent research shows this achieves reasonable performance with vastly less training than transformers.

3. **Grounded Language** - The "symbol grounding problem" states that language meaning comes from connecting symbols to sensory/perceptual experience. SAGE already has rich pattern representations (circle, square, spiral) - words should activate these!

4. **Temporal NCAs exist** - NCAs can learn sequential patterns through backpropagation through time, using hidden channels to store temporal information.

---

## Recommended Architecture: Grounded Reservoir Language Model

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    SAGE Language Architecture                           │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  INPUT                      RESERVOIR                    OUTPUT         │
│  ─────                      ─────────                    ──────         │
│                                                                         │
│  "hello"                   ┌─────────────┐                              │
│     │                      │             │           ┌──────────────┐   │
│     ▼                      │  NCA Grid   │           │   Readout    │   │
│  ┌──────────┐   encode     │   32×32     │  extract  │   Network    │   │
│  │  Text    │────────────▶ │  22 chan    │──────────▶│  (trainable) │   │
│  │ Encoder  │              │             │           │              │   │
│  └──────────┘              │  evolve     │           └──────┬───────┘   │
│                            │  N steps    │                  │           │
│                            └─────────────┘                  ▼           │
│                                  │                    ┌──────────┐      │
│                                  │                    │ Response │      │
│                                  │                    │ Decoder  │      │
│                           (frozen after              └──────────┘      │
│                            pattern training)               │           │
│                                                           ▼           │
│                                                      "hi there"        │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

### Why This Architecture?

1. **Leverages SAGE's Strengths**: The NCA is already trained to form complex patterns. These patterns become the "concepts" that language grounds into.

2. **Efficient Training**: Only the readout layer needs training for language - the NCA reservoir is fixed (or slowly fine-tuned).

3. **Grounded Understanding**: Words like "circle" literally activate circle-like patterns in SAGE's neural substrate. SAGE doesn't just process symbols - it *experiences* them as patterns.

4. **Emergent Personality**: SAGE's responses emerge from its actual internal state, not from a separate language model. Its "mood" (NCA activation patterns) genuinely influences what it says.

---

## Implementation Plan

### Phase 1: Character-Level Foundation (Start Here)

**Goal**: SAGE learns to predict the next character given previous characters.

```rust
// New module: src/language/char_predictor.rs

struct CharPredictor {
    // Maps character → grid activation
    char_encoder: HashMap<char, Vec<f64>>,  // 64 chars → 22-dim vectors

    // Trainable readout: NCA state → next char probabilities
    readout_weights: Vec<Vec<f64>>,  // [64 chars × (32*32*22 features)]
    readout_bias: Vec<f64>,
}

impl CharPredictor {
    fn encode_char(&self, c: char) -> Vec<f64>;
    fn inject_into_grid(&self, grid: &mut Grid, encoded: &[f64], position: usize);
    fn extract_features(&self, grid: &Grid) -> Vec<f64>;
    fn predict_next(&self, features: &[f64]) -> Vec<f64>;  // Softmax over chars
    fn train_step(&mut self, input: &str, target_char: char);
}
```

**Training Data**:
- Start with simple text: nursery rhymes, basic sentences
- Progress to: conversations, stories, technical text

**Training Loop**:
```
for each text in corpus:
    for i in 0..text.len()-1:
        input_chars = text[0..i+1]
        target_char = text[i+1]

        # Encode input into grid
        grid.clear()
        for (pos, char) in input_chars.enumerate():
            inject_into_grid(grid, encode_char(char), pos)

        # Let NCA evolve (reservoir processing)
        for _ in 0..EVOLUTION_STEPS:
            nca.step(grid)

        # Extract features and predict
        features = extract_features(grid)
        prediction = readout.forward(features)

        # Train readout only
        loss = cross_entropy(prediction, target_char)
        readout.backward(loss)
```

### Phase 2: Word-Level Grounding

**Goal**: Connect words to SAGE's existing pattern vocabulary.

```rust
// Extend: src/word_pattern_mapper.rs

impl WordPatternMapper {
    // Learn new word → pattern associations from conversation
    fn learn_from_context(&mut self, word: &str, grid_state: &Grid) {
        // If SAGE sees "circle" while the grid has a circle pattern,
        // strengthen that association
    }

    // Bidirectional mapping
    fn word_to_pattern_activation(&self, word: &str) -> Option<Grid>;
    fn pattern_to_words(&self, grid: &Grid) -> Vec<(String, f64)>;  // confidence scores
}
```

### Phase 3: Sequence-to-Sequence Response

**Goal**: SAGE generates responses, not just predicts next chars.

```rust
// New module: src/language/conversation.rs

struct ConversationEngine {
    char_predictor: CharPredictor,
    word_mapper: WordPatternMapper,
    context_window: Vec<String>,  // Recent conversation

    fn respond(&mut self, user_input: &str) -> String {
        // 1. Encode user input into grid
        self.encode_input(user_input);

        // 2. Let NCA "think" (evolve)
        for _ in 0..THINKING_STEPS {
            self.nca.step(&mut self.grid);
        }

        // 3. Generate response character by character
        let mut response = String::new();
        for _ in 0..MAX_RESPONSE_LENGTH {
            let next_char = self.char_predictor.sample_next();
            if next_char == '\0' { break; }  // End token
            response.push(next_char);

            // Inject generated char back into grid
            self.inject_char(next_char);
            self.nca.step(&mut self.grid);  // Process it
        }

        response
    }
}
```

### Phase 4: Grounded Understanding

**Goal**: SAGE's responses reflect its internal pattern states.

```
User: "Draw me a circle"
       │
       ▼
   ┌─────────────────────────────────────┐
   │ 1. "circle" activates circle pattern│
   │ 2. NCA evolves toward circle        │
   │ 3. Pattern confidence increases     │
   │ 4. Response reflects this:          │
   │    "I feel the circle forming..."   │
   └─────────────────────────────────────┘
```

---

## Data Requirements

### Training Corpus (Progressive)

1. **Level 1 - Character patterns** (~10K samples)
   - "aaa", "bbb", "abc", "aba" - simple repetition
   - Single words: "hello", "world", "circle"

2. **Level 2 - Word sequences** (~100K samples)
   - Simple sentences: "The cat sat."
   - Pattern descriptions: "A circle is round."

3. **Level 3 - Conversations** (~1M samples)
   - Q&A pairs: "What is a circle?" → "A circle is round."
   - Dialogues from movie scripts, chat logs

4. **Level 4 - Grounded descriptions** (~100K samples)
   - Pair text with actual patterns: show circle + "this is a circle"
   - SAGE learns multimodal grounding

---

## Architecture Details

### Text Encoder (Enhanced)

Current `text_encoder.rs` places 8×8 character patterns in blocks. Enhance to:

```rust
struct EnhancedTextEncoder {
    // Positional encoding (like transformers)
    position_embeddings: Vec<Vec<f64>>,  // [max_seq_len × embed_dim]

    // Character embeddings (learned)
    char_embeddings: HashMap<char, Vec<f64>>,  // [vocab_size × embed_dim]

    fn encode(&self, text: &str) -> Grid {
        let mut grid = Grid::new(32, 32);

        for (pos, char) in text.chars().take(MAX_CHARS).enumerate() {
            let char_embed = self.char_embeddings[&char];
            let pos_embed = self.position_embeddings[pos];
            let combined = add_vectors(&char_embed, &pos_embed);

            // Inject into specific grid region based on position
            self.inject_at_position(&mut grid, &combined, pos);
        }

        grid
    }
}
```

### Readout Network

Simple but effective:

```rust
struct ReadoutNetwork {
    // Option A: Linear readout (fastest training)
    weights: Vec<Vec<f64>>,  // [output_size × input_features]

    // Option B: Small MLP (more expressive)
    hidden_weights: Vec<Vec<f64>>,
    output_weights: Vec<Vec<f64>>,

    fn forward(&self, grid_features: &[f64]) -> Vec<f64> {
        // Softmax over vocabulary
    }
}
```

### Feature Extraction from Grid

```rust
fn extract_features(grid: &Grid) -> Vec<f64> {
    let mut features = Vec::new();

    // Option A: Flatten all cells (32×32×22 = 22,528 features)
    // Simple but high-dimensional

    // Option B: Pool by region (more compact)
    for region_y in 0..4 {
        for region_x in 0..4 {
            let region_avg = average_region(grid, region_y, region_x);
            features.extend(region_avg);
        }
    }

    // Option C: Statistical features
    features.push(grid_mean(grid));
    features.push(grid_std(grid));
    features.push(pattern_energy(grid));
    // etc.

    features
}
```

---

## Training Strategy

### Curriculum Learning (Like Pattern Training)

1. **Week 1**: Single character prediction
   - Input: "a" → Output: "b" (for "ab" sequences)
   - Simple, gets the pipeline working

2. **Week 2**: Word completion
   - Input: "hel" → Output: "l" (for "hello")
   - Learns common word patterns

3. **Week 3**: Sentence completion
   - Input: "The cat" → Output: " " then "s" then "a" then "t"
   - Learns grammar implicitly

4. **Week 4**: Response generation
   - Input: "Hello!" → Output: "Hi!" or "Hello!"
   - Learns conversational patterns

### Evaluation Metrics

```rust
struct LanguageMetrics {
    char_accuracy: f64,      // Next-char prediction accuracy
    perplexity: f64,         // Language model quality
    response_coherence: f64, // Human-rated or rule-based
    grounding_accuracy: f64, // Does "circle" activate circle pattern?
}
```

---

## Comparison: Why Not Just Use an LLM?

| Aspect | External LLM (Ollama) | SAGE Native Language |
|--------|----------------------|---------------------|
| Understanding | Statistical patterns | Grounded in patterns |
| Personality | Prompted | Emergent from NCA state |
| Training | Pre-trained, fixed | Learns from conversation |
| Integration | Separate system | Native to SAGE |
| Efficiency | Large model, slow | Small readout, fast |
| Authenticity | Simulated emotions | Patterns ARE emotions |

**The key insight**: When SAGE says "I feel happy", with native language, its grid literally contains activation patterns associated with "happy". It's not pretending - it's describing its actual internal state.

---

## Files to Create/Modify

### New Files
```
src/language/
├── mod.rs              # Module exports
├── char_predictor.rs   # Character-level prediction
├── text_encoder.rs     # Enhanced text → grid encoding
├── readout.rs          # Trainable readout network
├── conversation.rs     # Response generation
├── training.rs         # Training loop and data loading
└── metrics.rs          # Evaluation metrics
```

### Modify Existing
```
src/word_pattern_mapper.rs  # Add bidirectional grounding
src/language_learning.rs    # Integrate with new system
src/tui/app.rs             # Add language training mode
Cargo.toml                  # Add any new dependencies
```

---

## Next Steps

1. **Create basic char_predictor.rs** - simplest possible version
2. **Create training corpus** - start with simple sequences
3. **Implement training loop** - integrate with existing TUI
4. **Test character prediction** - can SAGE predict "b" after "a"?
5. **Iterate and expand** - add words, sentences, conversations

---

## Questions to Consider

1. **How much of the NCA should be frozen?**
   - Fully frozen = pure reservoir computing
   - Slowly fine-tuned = adapts to language while preserving patterns

2. **What's the right grid encoding?**
   - Sequential (char by char in time)
   - Spatial (all chars laid out on grid)
   - Hybrid (recent chars spatial, older ones compressed)

3. **How to handle the vocabulary?**
   - Character-level (64 chars) - simplest
   - BPE tokens - more efficient for words
   - Word-level - needs larger vocabulary

4. **Training data source?**
   - Public datasets (WikiText, etc.)
   - Synthetic conversations
   - Interactive learning from real conversations

---

## References

- [Reservoir Computing as a Language Model (2025)](https://arxiv.org/abs/2507.15779)
- [Learning Spatio-Temporal Patterns with NCA](https://pmc.ncbi.nlm.nih.gov/articles/PMC11078362/)
- [Grounded Language Learning Survey](https://arxiv.org/html/2312.02431)
- [Echo State Networks for Syntax](https://arxiv.org/html/2503.01724)
- [Symbol Grounding Problem](https://en.wikipedia.org/wiki/Symbol_grounding_problem)
- [Growing Neural Cellular Automata](https://distill.pub/2020/growing-ca/)
