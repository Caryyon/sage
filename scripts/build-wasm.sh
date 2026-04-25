#!/bin/bash
# Build SAGE for WASM browser target
# Note: libp2p networking won't work in browser, but chat + NCA will

set -e

cd ~/Code/sage

echo "Building SAGE core for WASM..."

# Build only the NCA + knowledge modules (no networking)
cargo build --target wasm32-unknown-unknown --release \
  --no-default-features \
  2>&1 || echo "WASM target requires wasm32-unknown-unknown"

echo ""
echo "To install WASM target:"
echo "  rustup target add wasm32-unknown-unknown"
echo ""
echo "For browser demo, use wasm-bindgen:"
echo "  cargo install wasm-bindgen-cli"
