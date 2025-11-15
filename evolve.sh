#!/bin/bash
# SAGE Evolution Script - Rebuild and restart with state preservation

echo "🧬 SAGE Evolution Initiated..."
echo "📸 State is auto-saved to /tmp/sage_state.json"
echo ""
echo "🔨 Rebuilding SAGE with new code..."

# Build the project
cargo build --release 2>&1 | tail -20

if [ $? -eq 0 ]; then
    echo ""
    echo "✅ Build successful!"
    echo "🔄 SAGE will restart and restore state automatically"
    echo ""
    echo "Press Ctrl+C in the SAGE terminal, then run: cargo run"
else
    echo ""
    echo "❌ Build failed. Please fix errors before restarting SAGE."
    exit 1
fi
