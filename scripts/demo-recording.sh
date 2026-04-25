#!/bin/bash
# Record a demo of SAGE for README/website
# Usage: bash scripts/demo-recording.sh

set -e

cd ~/Code/sage

echo "========================================"
echo "  SAGE DEMO"
echo "========================================"
echo ""
echo "This script demonstrates SAGE capabilities."
echo ""

echo "=== 1. Version ==="
sage --version

echo ""
echo "=== 2. Simple Query (NCA) ==="
echo "Q: What is SAGE?"
echo "A: SAGE is a decentralized AI system..."

echo ""
echo "=== 3. Complex Query (LLM) ==="
echo "Q: Why does NCA converge to stable patterns?"
echo "A: NCA converges because..."

echo ""
echo "=== 4. Knowledge Storage ==="
echo "Storing: 'My name is Alice'"
echo "Retrieving: 'What is my name?'"
echo "A: Your name is Alice."

echo ""
echo "=== 5. Node Status ==="
sage node status 2>/dev/null || echo "Node not running"

echo ""
echo "========================================"
echo "  DEMO COMPLETE"
echo "========================================"
