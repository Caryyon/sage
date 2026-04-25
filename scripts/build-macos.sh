#!/bin/bash
# Build SAGE for macOS (universal binary)

set -e

cd ~/Code/sage

echo "Building SAGE for macOS..."

# Build for Intel
rustup target add x86_64-apple-darwin 2>/dev/null || true
cargo build --release --target x86_64-apple-darwin

# Build for Apple Silicon
rustup target add aarch64-apple-darwin 2>/dev/null || true
cargo build --release --target aarch64-apple-darwin

# Create universal binary
mkdir -p target/release/macos
lipo -create \
  target/x86_64-apple-darwin/release/sage-cli \
  target/aarch64-apple-darwin/release/sage-cli \
  -output target/release/macos/sage

echo "✅ Universal binary created"
echo "   target/release/macos/sage"
