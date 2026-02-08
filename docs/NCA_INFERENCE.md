# NCA Token Prediction — Phase 1 Research

## Hypothesis

Neural Cellular Automata (NCA) can learn to predict next tokens in a sequence through local update rules alone, without attention mechanisms or transformer architecture.

## Architecture

```
Input tokens → Grid activation → N NCA steps → Read activations → Predicted token
```

### Grid Layout
- **181×181 grid** (~32K cells, one per vocabulary token)
- **8 channels per cell**: activation, position encoding, recency, 5 hidden
- Each token ID maps to a fixed (row, col) via `id / 181, id % 181`

### NCA Update Rule
- Small MLP: `3×3 neighborhood perception (72 inputs) → 64 hidden (ReLU) → 8 output (tanh)`
- Residual update: `cell += 0.1 * tanh(MLP(perception))`
- Applied uniformly to all cells each step
- ~5,192 trainable parameters

### Prediction Mechanism
1. Clear grid
2. Activate cells for input tokens (set activation=1, encode position)
3. Run 20 NCA update steps (cells communicate via 3×3 neighborhoods)
4. Read activation channel of all vocab cells
5. Highest activation = predicted next token

### Training
- **Evolution Strategy (ES)**: population of 50 weight perturbations
- **Fitness**: top-5 accuracy on next-token prediction
- **No backpropagation needed** — ES works with non-differentiable systems
- Weights saved to `~/.sage/nca_weights.bin`

## Usage

```bash
# Train on built-in Shakespeare excerpt
sage-train --demo --epochs 100

# Train on custom corpus
sage-train --corpus my_text.txt --epochs 200
```

## Hybrid Mode

Config in `sage.toml`:
```toml
[inference]
nca_weight = 0.3  # 0.0 = pure LLM, 1.0 = pure NCA
```

The `HybridTracker` monitors NCA accuracy and auto-promotes `nca_weight` when accuracy exceeds threshold.

## Roadmap

| Phase | Description | Status |
|-------|-------------|--------|
| **1** | NCA predicts tokens, any signal above random | **Current** |
| 2 | Hybrid NCA + SmolLM2 1.7B | Planned |
| 3 | Reduce to SmolLM2 500M | Planned |
| 4 | Reduce to 100M | Planned |
| 5 | Pure NCA inference | Planned |

## Key Questions

1. Can local NCA rules propagate information across the grid fast enough?
2. Does the ES training find useful gradients in 5K-parameter space?
3. What's the ceiling for NCA next-token prediction with this architecture?
4. Does increasing NCA steps improve accuracy (more communication rounds)?

## Files

- `src/inference/nca_predictor.rs` — Core NCA predictor, tokenizer, training
- `src/bin/sage_train.rs` — Training binary
- `docs/NCA_INFERENCE.md` — This document
