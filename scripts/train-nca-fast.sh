#!/bin/bash
# Fast NCA training script — optimized for speed over accuracy
# Uses small grid, few epochs, but produces functional weights

set -e

cd ~/Code/sage

echo "🧠 Fast NCA Training"
echo "==================="

# Use backprop with minimal config for speed
cargo run --bin sage-train \
  -- --corpus data/training/qa-corpus.txt \
  --epochs 50 \
  --grid-size 4 \
  --max-examples 30 \
  --optimizer backprop \
  2>&1 | tail -20

echo ""
echo "✅ Training complete!"
echo "Weights: ~/.sage/nca_weights.bin"
ls -lh ~/.sage/nca_weights.bin
