# NCA Brain Architecture

SAGE stores knowledge in a Neural Cellular Automata (NCA) grid.

## Grid Structure
- 256×256 cells
- 12 channels per cell
- Channels 0-3: knowledge storage
- Channel 10: recency tracking
- Channel 11: overflow probing

## Encoding Process
1. Text → hash → grid position
2. Write semantic embedding into cell channels
3. Run NCA update steps for local communication

## Retrieval Process
1. Hash query → find candidate cells
2. Semantic search via attention decoder
3. Delta attention for spreading activation
4. Return top-K results

## Training
- CMA-ES or backprop optimization
- Loss: cross-entropy on token prediction
- Weights: ~836KB for 107K parameters
