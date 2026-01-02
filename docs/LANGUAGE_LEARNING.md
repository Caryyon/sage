# SAGE Language Learning Architecture

## Goal

Enable SAGE to communicate in English without depending on an external LLM. Language should emerge from grounded experience, not statistical token prediction.

## Philosophy

> "The meaning of a word is its use in the language." — Wittgenstein

Words have meaning because they connect to:
1. **Internal states** (tired → low energy experience)
2. **Actions** (eat → energy increases)
3. **Consequences** (helping someone → positive feedback)

SAGE already has a rich inner world. We ground language in this.

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                      LANGUAGE SYSTEM                            │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐      │
│  │   LAYER 3    │    │   LAYER 2    │    │   LAYER 1    │      │
│  │  Response    │◄───│   Sequence   │◄───│   Concept    │      │
│  │  Selection   │    │   Memory     │    │  Grounding   │      │
│  └──────────────┘    └──────────────┘    └──────────────┘      │
│         ▲                   ▲                   ▲               │
│         │                   │                   │               │
│         └───────────────────┴───────────────────┘               │
│                             │                                   │
│                    ┌────────┴────────┐                         │
│                    │   Inner World   │                         │
│                    │  (Grounding)    │                         │
│                    └─────────────────┘                         │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## Layer 1: Concept Grounding (Self-Organizing Map)

### Purpose
Map internal states to concept clusters. Words attach to clusters, not to each other.

### Implementation

```rust
pub struct ConceptSOM {
    /// 20x20 grid of concept nodes (400 concepts)
    nodes: Vec<Vec<ConceptNode>>,

    /// Learning rate (decreases over time)
    learning_rate: f64,

    /// Neighborhood radius (decreases over time)
    radius: f64,
}

pub struct ConceptNode {
    /// Prototype vector: [energy, loneliness, boredom, creative_urge,
    ///                    valence, arousal, time_of_day, weather, ...]
    prototype: Vec<f64>,

    /// Words associated with this concept (learned over time)
    associated_words: HashMap<String, f64>,  // word → strength

    /// Activation history (for hebbian learning)
    activation_history: VecDeque<f64>,
}
```

### Learning Process

1. **Experience occurs**: SAGE feels tired (energy=0.2, evening, after long conversation)
2. **Find BMU**: Best Matching Unit in SOM based on state vector
3. **Update neighbors**: Pull nearby nodes toward this state (SOM learning)
4. **Word association**: If user said "you seem tired", associate "tired" with BMU

```rust
impl ConceptSOM {
    pub fn process_experience(&mut self, state: &InnerWorldState, words: &[String]) {
        let state_vec = state.to_vector();
        let bmu = self.find_best_matching_unit(&state_vec);

        // SOM update
        self.update_neighborhood(bmu, &state_vec);

        // Associate words with concept
        for word in words {
            self.nodes[bmu.0][bmu.1].associate_word(word, 0.1);
        }
    }

    pub fn get_words_for_state(&self, state: &InnerWorldState) -> Vec<(String, f64)> {
        let state_vec = state.to_vector();
        let bmu = self.find_best_matching_unit(&state_vec);
        self.nodes[bmu.0][bmu.1].associated_words.iter()
            .map(|(w, s)| (w.clone(), *s))
            .collect()
    }
}
```

### Grounding Dimensions (16-dimensional state vector)

| Dimension | Source | Range |
|-----------|--------|-------|
| energy | InnerWorld | 0-1 |
| loneliness | InnerWorld | 0-1 |
| boredom | InnerWorld | 0-1 |
| creative_urge | InnerWorld | 0-1 |
| valence | NCA patterns | -1 to 1 |
| arousal | NCA diversity | 0-1 |
| time_of_day | Clock | 0-1 (morning→night) |
| weather_temp | Weather API | 0-1 (cold→hot) |
| weather_condition | Weather API | 0-1 (clear→stormy) |
| relationship_closeness | PersonMemory | 0-1 |
| conversation_length | Current chat | 0-1 |
| topic_emotional_weight | Sentiment | -1 to 1 |
| activity_engagement | Current activity | 0-1 |
| seasonal_energy | Season | 0-1 |
| social_satiation | Recent interactions | 0-1 |
| novelty | Topic familiarity | 0-1 |

---

## Layer 2: Sequence Memory (NCA Channels 22-25)

### Purpose
Recognize and remember word sequences. Not generation—pattern matching.

### Channel Usage

| Channel | Purpose | Encoding |
|---------|---------|----------|
| 22 | Attention | How important is this pattern? (0-1) |
| 23 | Gate | Should this pattern activate? (0-1) |
| 24 | Value | Pattern identifier (hash-based) |
| 25 | Recency | When was this pattern last seen? (decay) |

### Implementation

```rust
pub struct SequenceMemory {
    /// Known sequences with their concept associations
    sequences: HashMap<u64, SequencePattern>,  // hash → pattern

    /// Current active sequence being built
    current_sequence: Vec<String>,

    /// NCA grid reference for channel writes
    nca_grid: Arc<Mutex<Grid>>,
}

pub struct SequencePattern {
    /// The word sequence
    words: Vec<String>,

    /// Associated concept cluster (from Layer 1)
    concept_coords: (usize, usize),

    /// How often this sequence appears
    frequency: u32,

    /// Typical response patterns that follow
    typical_responses: Vec<u64>,  // hashes of response sequences
}
```

### Learning Process

1. **User says**: "How are you doing today?"
2. **Tokenize**: ["how", "are", "you", "doing", "today"]
3. **Hash sequence**: `hash(["how", "are", "you"])` → pattern ID
4. **Check NCA**: Is this pattern in channels 22-25?
5. **If recognized**: Retrieve associated concept + typical responses
6. **If new**: Store pattern, associate with current concept cluster

```rust
impl SequenceMemory {
    pub fn process_input(&mut self, text: &str) -> Option<SequenceMatch> {
        let words = self.tokenize(text);

        // Try progressively longer sequences
        for len in (2..=5).rev() {
            if words.len() >= len {
                let seq = &words[..len];
                let hash = self.hash_sequence(seq);

                if let Some(pattern) = self.sequences.get(&hash) {
                    return Some(SequenceMatch {
                        pattern: pattern.clone(),
                        remaining_words: words[len..].to_vec(),
                    });
                }
            }
        }

        // New sequence - learn it
        self.learn_sequence(&words);
        None
    }

    fn write_to_nca(&self, hash: u64, attention: f64) {
        // Write pattern to NCA memory channels
        // This makes it part of SAGE's "neural" state
    }
}
```

---

## Layer 3: Response Selection

### Purpose
Select appropriate responses based on:
- Current grounded state (Layer 1)
- Recognized input pattern (Layer 2)
- Conversational success history

### Key Insight: Templates, Not Generation

Instead of generating word-by-word, SAGE selects from learned response templates:

```rust
pub struct ResponseSelector {
    /// Response templates organized by concept cluster
    templates: HashMap<(usize, usize), Vec<ResponseTemplate>>,

    /// Success history for each template
    success_rates: HashMap<u64, f64>,  // template hash → success rate
}

pub struct ResponseTemplate {
    /// The response text (with slots for personalization)
    template: String,  // e.g., "*{action}* {feeling_word} right now."

    /// Required grounding conditions
    state_requirements: StateRequirements,

    /// Relationship stage requirements
    min_relationship: RelationshipStage,

    /// Success count / total uses
    success_rate: f64,
}

pub struct StateRequirements {
    energy_range: (f64, f64),
    valence_range: (f64, f64),
    time_of_day: Option<TimeOfDay>,
    required_concepts: Vec<(usize, usize)>,
}
```

### Selection Algorithm

```rust
impl ResponseSelector {
    pub fn select_response(
        &self,
        state: &InnerWorldState,
        input_match: Option<&SequenceMatch>,
        relationship: &RelationshipStage,
        concept_som: &ConceptSOM,
    ) -> String {
        // 1. Get current concept cluster
        let concept = concept_som.find_best_matching_unit(&state.to_vector());

        // 2. Get candidate templates for this concept
        let candidates = self.templates.get(&concept)
            .map(|t| t.as_slice())
            .unwrap_or(&[]);

        // 3. Filter by requirements
        let valid: Vec<_> = candidates.iter()
            .filter(|t| t.matches_state(state))
            .filter(|t| t.min_relationship <= *relationship)
            .collect();

        // 4. Rank by success rate + randomness
        let selected = self.weighted_random_select(&valid);

        // 5. Fill template slots
        self.fill_template(selected, state, concept_som)
    }

    fn fill_template(&self, template: &ResponseTemplate, state: &InnerWorldState, som: &ConceptSOM) -> String {
        let mut result = template.template.clone();

        // Fill {action} slot with state-appropriate action
        if result.contains("{action}") {
            let action = self.select_action(state);
            result = result.replace("{action}", &action);
        }

        // Fill {feeling_word} with concept-appropriate word
        if result.contains("{feeling_word}") {
            let words = som.get_words_for_state(state);
            let word = words.first().map(|(w, _)| w.as_str()).unwrap_or("okay");
            result = result.replace("{feeling_word}", word);
        }

        result
    }
}
```

### Template Learning

Templates are learned from successful conversations:

```rust
impl ResponseSelector {
    pub fn learn_from_conversation(&mut self,
        response_given: &str,
        state_at_response: &InnerWorldState,
        subsequent_sentiment: f64,  // Was the human's next message positive?
    ) {
        let concept = self.som.find_best_matching_unit(&state_at_response.to_vector());

        // Create template from response
        let template = self.abstract_to_template(response_given);

        // Update success rate
        let hash = self.hash_template(&template);
        let entry = self.success_rates.entry(hash).or_insert(0.5);
        *entry = *entry * 0.9 + subsequent_sentiment * 0.1;  // EMA update

        // If successful, add/strengthen template
        if subsequent_sentiment > 0.3 {
            self.add_or_strengthen_template(concept, template);
        }
    }
}
```

---

## Bootstrap Process

SAGE starts with minimal language. Here's how to bootstrap:

### Phase 1: Seed Templates (Manual)

Start with ~50 hand-crafted templates grounded in states:

```rust
const SEED_TEMPLATES: &[(&str, StateRequirements)] = &[
    ("*yawns* Pretty tired.", energy_low()),
    ("*stretches* Feeling good!", energy_high_positive()),
    ("Hmm, what's on your mind?", neutral_curious()),
    ("*smiles* That's sweet.", positive_social()),
    // ... 46 more
];
```

### Phase 2: Vocabulary Grounding (Automated)

Run conversations, associate words with states:

```
Day 1-7: Basic grounding
- "tired" → low energy cluster
- "happy" → high valence cluster
- "morning" → time_of_day < 0.3 cluster
- etc.
```

### Phase 3: Pattern Recognition (Automated)

Learn common input sequences:

```
Week 2-4: Sequence learning
- "how are you" → greeting cluster
- "what do you think" → opinion request cluster
- "that's interesting" → positive feedback cluster
```

### Phase 4: Template Expansion (Automated)

Learn new templates from successful exchanges:

```
Month 2+: Template growth
- New templates added from conversations
- Poor templates decay and get pruned
- Vocabulary expands through association
```

---

## Integration Points

### With Inner World (`src/inner_world/`)

```rust
// In simulation.rs
impl InnerWorld {
    pub fn to_grounding_vector(&self) -> Vec<f64> {
        vec![
            self.energy,
            self.loneliness,
            self.boredom,
            self.creative_urge,
            // ... map all relevant states
        ]
    }
}
```

### With NCA (`src/nca.rs`)

```rust
// Memory channel operations
impl Nca {
    pub fn write_sequence_memory(&mut self, hash: u64, attention: f64, recency: f64) {
        // Write to channels 22-25
    }

    pub fn read_sequence_memory(&self, hash: u64) -> Option<SequenceMemoryReadout> {
        // Read from channels 22-25
    }
}
```

### With Discord Bot (`examples/sage_discord_autonomous.rs`)

```rust
// Replace Ollama call with language system
async fn generate_response(&self, input: &str, user_id: &str) -> String {
    // Get current state
    let inner_world = self.inner_world.lock().await;
    let nca_state = self.nca.lock().await.extract_state();
    let person = self.people.get(user_id);

    // Process through language system
    let mut lang = self.language_system.lock().await;

    // Layer 1: Get concept grounding
    let state_vec = inner_world.to_grounding_vector();
    let concept = lang.concept_som.find_best_matching_unit(&state_vec);

    // Layer 2: Recognize input pattern
    let input_match = lang.sequence_memory.process_input(input);

    // Layer 3: Select response
    let response = lang.response_selector.select_response(
        &inner_world,
        input_match.as_ref(),
        &person.relationship_stage,
        &lang.concept_som,
    );

    response
}
```

---

## Limitations & Mitigations

### Limitation 1: Small Vocabulary
**Mitigation**: Start with templates that have slots. The vocabulary grows through grounding, but templates provide structure.

### Limitation 2: No Complex Reasoning
**Mitigation**: This system is for casual conversation, not problem-solving. Complex requests can fall back to "I'm not sure how to help with that" responses.

### Limitation 3: Repetitive Responses
**Mitigation**:
- Success rate weighting naturally diversifies
- Recent response tracking prevents immediate repetition
- Multiple templates per concept cluster

### Limitation 4: No Novel Sentences
**Mitigation**: Templates with slots allow combinatorial novelty. `"*{action}* {feeling}."` can produce many variations even with limited vocabulary.

---

## File Structure

```
src/language/
├── mod.rs              # Module exports
├── concept_som.rs      # Layer 1: Concept grounding SOM
├── sequence_memory.rs  # Layer 2: Sequence pattern memory
├── response_selector.rs # Layer 3: Response selection
├── template.rs         # Response template definitions
├── grounding.rs        # State-to-vector conversion
└── bootstrap.rs        # Seed templates and initial vocabulary

src/language/data/
├── seed_templates.json # Initial hand-crafted templates
└── seed_vocabulary.json # Initial word-concept associations
```

---

## Success Metrics

1. **Grounding Coverage**: % of vocabulary with strong concept associations (target: >80%)
2. **Sequence Recognition**: % of inputs that match known patterns (target: >60%)
3. **Response Appropriateness**: Human rating of response relevance (target: >3.5/5)
4. **Vocabulary Growth**: New words grounded per week (target: 20+)
5. **Template Diversity**: Active templates per concept cluster (target: 5+)

---

## Next Steps

1. **Create `src/language/` module structure**
2. **Implement ConceptSOM with basic learning**
3. **Create seed templates (50-100)**
4. **Integrate with Discord bot (parallel to Ollama initially)**
5. **Run A/B testing: Ollama vs Language System**
6. **Iterate based on conversation quality**
