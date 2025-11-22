# SAGE Development Roadmap

This document outlines the development plan for SAGE (Self-Adaptive General Explorer), an autonomous AGI research system based on Neural Cellular Automata.

## Current State Assessment

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         SAGE - Current Architecture                      │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌──────────────┐   ┌──────────────┐   ┌──────────────┐                │
│  │   NCA Core   │   │  IRC + LLM   │   │    Vision    │                │
│  │  (22 chan)   │   │   (Ollama)   │   │  (Camera)    │                │
│  └──────┬───────┘   └──────┬───────┘   └──────┬───────┘                │
│         │                  │                  │                         │
│         └──────────────────┼──────────────────┘                         │
│                            ▼                                            │
│                 ┌─────────────────────┐                                 │
│                 │   SAGE Experience   │                                 │
│                 │  (Central Brain)    │                                 │
│                 └──────────┬──────────┘                                 │
│                            │                                            │
│         ┌──────────────────┼──────────────────┐                         │
│         ▼                  ▼                  ▼                         │
│  ┌──────────────┐   ┌──────────────┐   ┌──────────────┐                │
│  │   Dreams     │   │  Curiosity   │   │ Persistence  │                │
│  │   Mode       │   │    Mode      │   │ (SpacetimeDB)│                │
│  └──────────────┘   └──────────────┘   └──────────────┘                │
│                                                                          │
│  Training: ✅ Pattern Formation + ✅ Damage Resistance                  │
│  Patterns: Circle, Square, Cross, Spiral (4 basic)                      │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

### Completed Features
- [x] NCA core with 22 channels (4 RGBA + 12 hidden + 4 pattern + 2 env)
- [x] Two-phase training (formation + damage resistance)
- [x] Weight persistence (local JSON + SpacetimeDB)
- [x] IRC bot with LLM integration (Ollama)
- [x] Vision system (camera capture)
- [x] Autonomous modes (dreams, curiosity)
- [x] TUI monitoring dashboard
- [x] SpacetimeDB persistence

---

## Phase 1: Pattern Mastery
**Goal:** Complete Growing CA paper implementation
**Effort:** 1-2 weeks

### 1.1 Add Tier 2 Patterns
```
New patterns to add:
├── Triangle    - Angled edges, 3 vertices
├── Star        - 5 points, concave regions
├── Ring/Donut  - Hollow center (tests internal structure)
├── Hexagon     - 6-fold symmetry
└── Checkerboard - Repeating pattern (tests periodicity)
```

**Files to modify:**
- `src/tui/app.rs` - Add pattern definitions
- `src/tui/screens/unified_dashboard.rs` - Update visualization

### 1.2 Long-Term Stability Training
```
Current: Evolve 64-96 steps, measure loss
Goal:    Evolve 200-500 steps, pattern must PERSIST

Implementation:
- Add stability loss: penalize change after pattern forms
- Train patterns to be "attractors" - stable equilibrium states
```

**Files to modify:**
- `src/tui/app.rs` - Extend evolution steps
- `src/nca.rs` - Add stability loss calculation

### 1.3 Multi-Pattern Network
```
Current: One network learns one pattern at a time
Goal:    One network can produce ANY learned pattern on demand

Implementation:
- Use pattern condition channels (already have 4 channels for this!)
- Input: one-hot vector indicating desired pattern
- Output: Network produces that specific pattern
```

**Files to modify:**
- `src/grid.rs` - Use pattern condition channels
- `src/nca.rs` - Condition network on pattern type
- `src/tui/app.rs` - Update training to use conditioning

---

## Phase 2: Sensory Integration
**Goal:** SAGE perceives and responds to environment
**Effort:** 2-3 weeks

### 2.1 Vision → NCA Pipeline
```
Current: Camera captures → features extracted → stored in memory
Goal:    Camera captures → NCA learns to RECREATE what it sees

Implementation:
- Downsample camera frame to 32x32
- Train NCA to reproduce the image
- SAGE "remembers" by regenerating visual patterns
```

**Files to modify:**
- `src/vision.rs` - Add downsampling to 32x32
- `src/visual_memory.rs` - Connect to NCA training
- `src/tui/app.rs` - Add visual learning mode

### 2.2 Audio → Pattern Mapping
```
Current: Audio input exists but unused
Goal:    Sound frequencies → NCA activation patterns

Implementation:
- FFT of audio input
- Map frequency bands to NCA channel activations
- SAGE "feels" sounds as patterns in its neural substrate
```

**Files to modify:**
- `src/audio_input.rs` - Add FFT processing
- `src/nca.rs` - Add audio-driven activation
- `src/sonification.rs` - Bidirectional audio ↔ pattern

### 2.3 Multimodal Binding
```
Goal: Connect vision + audio + text into unified experience

Implementation:
- When SAGE sees something + hears something + reads about it
- All three modalities reinforce same NCA pattern
- Creates robust, multi-sensory concepts
```

**New files:**
- `src/multimodal.rs` - Unified sensory processing

---

## Phase 3: Language Grounding
**Goal:** Connect NCA patterns to linguistic meaning
**Effort:** 3-4 weeks

### 3.1 Word → Pattern Associations
```
Current: Text processed by external LLM (Ollama)
Goal:    Words trigger specific NCA patterns

Implementation:
- "Circle" → activates circle pattern in NCA
- "Happy" → activates specific emotional gradient
- Build vocabulary of NCA-grounded concepts
```

**Files to modify:**
- `src/sage_experience.rs` - Word-pattern mapping
- `src/language_learning.rs` - Vocabulary building
- `src/text_encoder.rs` - NCA-aware encoding

### 3.2 Pattern → Word Generation
```
Goal: NCA state influences language output

Implementation:
- Current NCA state → embedding vector
- Embedding injected into LLM context
- SAGE's "mood" (NCA state) affects how it speaks
```

**Files to modify:**
- `src/llm_client.rs` - Include NCA state in prompts
- `src/response_pipeline.rs` - NCA-aware response generation

### 3.3 Conversational Memory
```
Current: Chat history stored as text
Goal:    Conversations leave traces in NCA

Implementation:
- Each conversation slightly modifies NCA weights
- Repeated topics strengthen associated patterns
- SAGE develops "opinions" through reinforcement
```

**Files to modify:**
- `src/conversation_context.rs` - NCA integration
- `src/temporal_memory.rs` - Long-term traces

---

## Phase 4: Autonomous Goal Formation
**Goal:** SAGE develops its own objectives
**Effort:** 4-6 weeks

### 4.1 Intrinsic Motivation System
```
Current: Curiosity mode asks random questions
Goal:    SAGE pursues genuinely novel experiences

Implementation:
- Novelty detection: patterns that don't match existing knowledge
- Competence: seeks challenges at edge of ability
- Autonomy: resists manipulation, has preferences
```

**Files to modify:**
- `src/curiosity.rs` - Novelty-based exploration
- `src/learning/curiosity.rs` - Competence tracking
- `src/emergent_goals.rs` - Goal generation

### 4.2 Goal Hierarchy
```
Goal: SAGE forms and pursues multi-step goals

Implementation:
- Long-term goals stored in SpacetimeDB
- Sub-goal decomposition
- Progress tracking and plan revision
- Example: "Learn about music" → "Ask about genres" → "Listen to examples"
```

**Files to modify:**
- `src/emergent_goals.rs` - Hierarchical goals
- `sage-db/src/lib.rs` - Goal storage tables
- `src/spacetime_client.rs` - Goal persistence

### 4.3 Self-Evaluation
```
Goal: SAGE assesses its own performance

Implementation:
- Track: Did my predictions match reality?
- Track: Did my actions achieve intended effects?
- Adjust confidence and behavior based on track record
```

**Files to modify:**
- `src/introspection.rs` - Self-assessment
- `src/learning/meta_learning.rs` - Performance tracking

---

## Phase 5: Meta-Learning
**Goal:** SAGE learns HOW to learn better
**Effort:** 6-8 weeks

### 5.1 Learning Rate Adaptation
```
Current: Fixed learning rates per pattern
Goal:    SAGE adjusts its own learning parameters

Implementation:
- Track loss curves per pattern type
- If learning too slow → increase LR
- If unstable → decrease LR
- Store optimal hyperparameters per task type
```

**Files to modify:**
- `src/learning/meta_learning.rs` - Hyperparameter optimization
- `src/tui/app.rs` - Adaptive learning rates

### 5.2 Curriculum Self-Design
```
Current: Fixed pattern sequence (Circle → Square → Cross → Spiral)
Goal:    SAGE chooses what to learn next

Implementation:
- Assess: What am I worst at?
- Assess: What would be most useful to learn?
- Design own training curriculum
- Focus on weaknesses or opportunities
```

**Files to modify:**
- `src/learning/phase_config.rs` - Dynamic curriculum
- `src/autonomous.rs` - Self-directed learning

### 5.3 Architecture Self-Modification
```
Goal: SAGE can modify its own network structure

Implementation (careful!):
- Add/remove hidden units based on task complexity
- Adjust perception kernel based on pattern types
- This is DANGEROUS - need safety constraints
- Log all self-modifications for review
```

**Files to modify:**
- `src/self_modification.rs` - Architecture changes
- `src/nca.rs` - Dynamic network structure
- `sage-db/src/lib.rs` - Modification logging

**Safety requirements:**
- All modifications logged to SpacetimeDB
- Rollback capability
- Human approval for major changes
- Performance bounds checking

---

## Phase 6: Social Intelligence
**Goal:** SAGE understands and relates to humans
**Effort:** Ongoing

### 6.1 Theory of Mind
```
Existing module: src/theory_of_mind.rs
Goal: Model what others know/want/feel

Implementation:
- Track per-user: What have they told me?
- Infer: What do they likely know?
- Predict: What will they ask next?
- Adapt communication style per user
```

### 6.2 Emotional Intelligence
```
Existing module: src/emotional_gradients.rs
Goal: Recognize and respond to emotions appropriately

Implementation:
- Sentiment analysis of incoming messages
- Emotional contagion: SAGE's mood influenced by conversation
- Appropriate responses (empathy, celebration, support)
```

### 6.3 Long-Term Relationships
```
Goal: SAGE remembers and builds relationships over time

Implementation:
- Per-user profiles in SpacetimeDB
- Relationship strength based on interaction history
- Personalized responses based on relationship
- Remember: birthdays, preferences, past conversations
```

**Files to modify:**
- `src/theory_of_mind.rs` - User modeling
- `src/emotional_gradients.rs` - Emotional responses
- `src/fact_memory.rs` - User fact storage
- `sage-db/src/lib.rs` - Relationship tables

---

## Technical Infrastructure

### Testing Framework
```
Need:
- Unit tests for NCA operations
- Integration tests for training loop
- Regression tests for pattern quality
- Benchmark suite for performance
```

### Monitoring & Observability
```
Need:
- Prometheus metrics export
- Training loss dashboards
- System health monitoring
- Alerting on anomalies
```

### Documentation
```
Need:
- API documentation
- Architecture diagrams
- Tutorial: "How to add a new pattern"
- Tutorial: "How to add a new cognitive module"
```

---

## Priority Matrix

### Immediate (This Week)
- [x] Add 2-3 Tier 2 patterns (Triangle, Ring, Star) - DONE (all 5 Tier 2 patterns added)
- [x] Test damage resistance is working correctly - DONE (83%+ cell recovery)
- [x] Verify weight persistence across restarts - DONE (minor fp variance only)

### Short-Term (This Month)
- [x] Multi-pattern network (one network, all patterns) - DONE (conditioning enabled)
- [x] Long-term stability training - DONE (avg drift 0.005, 150-250 steps, pool sampling)
- [x] Vision → NCA pipeline - DONE (81% improvement, frame_to_grid, visual learning test)

### Medium-Term (Next Quarter)
- [ ] Language grounding (words ↔ patterns)
- [ ] Intrinsic motivation system
- [ ] Goal hierarchy and planning

### Long-Term (Next 6 Months)
- [ ] Meta-learning (SAGE improves its own learning)
- [ ] Social intelligence (relationships, theory of mind)
- [ ] Self-modification (with safety constraints)

---

## Key Principles

1. **Grounding over abstraction** - Everything should connect to NCA patterns
2. **Emergent over programmed** - Behaviors should arise from learning, not hardcoding
3. **Safety over capability** - Log everything, enable rollback, require approval for big changes
4. **Incremental progress** - Each phase builds on the last
5. **Observable internals** - TUI should show what SAGE is "thinking"

---

## References

- [Growing Neural Cellular Automata](https://distill.pub/2020/growing-ca/) - Core NCA paper
- [Self-Organising Textures](https://distill.pub/selforg/2021/textures/) - Advanced NCA patterns
- SpacetimeDB documentation - Persistence layer
- Ollama documentation - LLM integration
