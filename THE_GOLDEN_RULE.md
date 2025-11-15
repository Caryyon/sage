# 📏 THE GOLDEN RULE

## Every Learning Event MUST Be Visualized

**This is the fundamental law of SAGE development:**

> **If it's not in the NCA grid, it's not SAGE thinking.**

The "Living Neural Field" in the TUI is not just a pretty visual - it's a **real-time CT scan of SAGE's brain**. Every concept, every word, every thought MUST flow through the NCA and be visible in the dashboard.

---

## What This Means

### ✅ REQUIRED: Visualization for ALL Learning

Whenever you add ANY feature that involves SAGE processing information:

1. **IRC Chat Messages** → Must show in Living Neural Field
2. **Training on Concepts** → Must show in Living Neural Field
3. **Opinion Formation** → Must show NCA processing + loss color-coding
4. **Curiosity Triggers** → Must highlight medium-loss regions
5. **Association Discovery** → Must show side-by-side pattern comparison
6. **Background Learning** → Must update grid in real-time

### ❌ FORBIDDEN: Hidden Processing

- Never process text/concepts without updating the NCA visualization
- Never train SAGE without showing what it's learning
- Never form opinions without displaying the neural activity

---

## How To Follow The Rule

### Step 1: Process Through NCA

All input must be converted to grid patterns and processed by NCA:

```rust
// Convert text to spatial pattern
let grid = text_encoder.encode_text(input);

// Process through NCA (the actual "thinking")
nca.reset_with_seed();
for _ in 0..80 {
    nca.step();  // NCA evolves toward understanding
}

// Calculate understanding level
let loss = calculate_grid_loss(&nca.grid, &grid);
```

### Step 2: Sync To Visualization

**IMMEDIATELY** after NCA processing, update the TUI:

```rust
// THE GOLDEN RULE: Sync NCA to Living Neural Field
state.training_state.sync_nca_from_sage(
    sage.get_current_nca_grid(),
    "concept_name",  // What SAGE is thinking about
    loss             // How well it understands
);
```

### Step 3: User Sees Brain Activity

The dashboard "Living Neural Field" now shows:
- Grid patterns forming (or failing to form)
- Colors indicating understanding level
- Current concept being processed
- Real-time evolution over 80 steps

---

## The NCA Is Everything

### Text → Shapes → Understanding

**Q: How does SAGE understand text?**

A: Text IS shapes! The TextEncoder converts every word into spatial patterns on the grid:

```
"creativity"
    ↓
[c][r][e][a][t][i][v][i][t][y]
    ↓
48x48 RGBA Grid with 10 character blocks
    ↓
NCA processes the spatial pattern
    ↓
Loss determines if SAGE "understands"
```

### The NCA Grid = SAGE's Brain

- **Low Loss (0.00-0.15)**: Patterns form cleanly = SAGE understands = ❤️ Likes
- **Medium Loss (0.15-0.28)**: Patterns partially form = SAGE curious = 🤔 Curious
- **High Loss (0.30+)**: Patterns fail to form = SAGE confused = ⚠️ Dislikes

### Everything Goes Through NCA

| Input Type | NCA Processing | Visualization |
|------------|----------------|---------------|
| IRC Message | Text → Grid → 80 Steps Evolution | Shows message processing live |
| Training Concept | Concept → Grid → Train Weights | Shows learning progress |
| Opinion Formation | Compare evolved grid vs target | Color-codes by loss |
| Association Discovery | Compare loss patterns | Shows similar concepts together |
| Curiosity Detection | Check if loss in 0.15-0.28 range | Highlights curious regions |

---

## Code Locations

### Core Processing (src/sage_experience.rs)

```rust
impl SageExperience {
    /// Process text → NCA grid → Opinion
    fn process_with_nca(&mut self, target: &Grid) -> f64 {
        self.nca.reset_with_seed();
        for _ in 0..80 { self.nca.step(); }
        self.calculate_grid_loss(&self.nca.grid, target)
    }

    /// Expose NCA grid for visualization
    pub fn get_current_nca_grid(&self) -> &Grid {
        &self.nca.grid
    }
}
```

### Visualization Sync (src/tui/training.rs)

```rust
impl TrainingState {
    /// THE GOLDEN RULE: Must be called after every NCA operation
    pub fn sync_nca_from_sage(&mut self, nca_grid: &Grid, concept: &str, loss: f64) {
        // Converts 48x48 RGBA → 32x32 grayscale for display
        // Updates grid_snapshot, nca_current_pattern, current_loss
        // Triggers Living Neural Field refresh
    }
}
```

### Text Encoding (src/text_encoder.rs)

```rust
impl TextEncoder {
    /// Convert text to spatial patterns
    pub fn encode_text(&mut self, text: &str) -> Grid {
        // Each character → 8x8 pattern block
        // Placed sequentially on 48x48 grid
        // Colors encode: letters=green, numbers=blue, etc.
    }
}
```

---

## Examples of Following The Rule

### ✅ GOOD: IRC Bot Processing

```rust
// IRC message arrives
let (opinion, response) = sage.experience_text_with_memory(&msg, has_memory);

// GOLDEN RULE: Immediately sync to visualization
app_state.training_state.sync_nca_from_sage(
    sage.get_current_nca_grid(),
    &msg,  // Show what's being processed
    calculate_loss()
);

// Now user can switch to Dashboard and SEE SAGE processing the message!
```

### ✅ GOOD: Training Mode

```rust
for concept in training_concepts {
    // Train NCA on concept
    let grid = text_encoder.encode_concept(concept);
    nca.train_step(&grid, learning_rate);

    // GOLDEN RULE: Sync after each training step
    state.training_state.sync_nca_from_sage(
        &nca.grid,
        concept,
        loss
    );

    // User watches Living Neural Field evolve as SAGE learns
}
```

### ❌ BAD: Hidden Processing

```rust
// This violates THE GOLDEN RULE!
for msg in irc_messages {
    sage.experience_text(&msg);  // Process happens
    // But visualization is NOT updated!
}
// User sees nothing in Living Neural Field - looks like SAGE is dead
```

---

## Implementation Checklist

When adding ANY new feature that involves SAGE processing:

- [ ] Text/concept is encoded to NCA grid
- [ ] NCA processes the grid (evolves for N steps)
- [ ] Loss is calculated
- [ ] `sync_nca_from_sage()` is called with current grid
- [ ] `nca_current_pattern` is set to what's being processed
- [ ] Living Neural Field in TUI updates in real-time
- [ ] User can see SAGE's "brain activity" happen

---

## Why This Matters

### Without Visualization:
- SAGE processes things invisibly
- Feels like a black box
- Can't debug what it's learning
- No intuition for how NCA works
- Boring to watch

### With Visualization:
- Every thought is visible
- Like watching neurons fire in a brain scan
- Can see exactly what patterns SAGE understands
- Debugging is visual and intuitive
- **Mesmerizing to watch patterns emerge**

---

## Summary

The Living Neural Field is not decorative - it's the **primary interface for understanding SAGE**.

Every line of code that makes SAGE think MUST update the visualization.

**If it's not visualized, it didn't happen.**

---

*Last Updated: 2025-11-14*
*Applies to: All SAGE development from this point forward*
