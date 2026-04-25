#!/bin/bash
# Build SAGE optimized for Raspberry Pi 4

set -e

cd ~/Code/sage

echo "Building SAGE for Raspberry Pi 4..."

# Cross-compile for ARM
cargo build --release --target aarch64-unknown-linux-gnu \
  2>&1 || echo "Install cross-compile toolchain:"
  echo "  sudo apt install gcc-aarch64-linux-gnu"
  echo "  rustup target add aarch64-unknown-linux-gnu"

echo ""
echo "For Pi 4 deployment:"
echo "  scp target/aarch64-unknown-linux-gnu/release/sage-cli pi@raspberrypi.local:"
echo "  ssh pi@raspberrypi.local ./sage-cli --version"
