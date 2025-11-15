#!/bin/bash
# Quick test script for SAGE 2.0 LLM integration

echo "╔════════════════════════════════════════════════════════════╗"
echo "║         SAGE 2.0 LLM Integration Test Suite               ║"
echo "╚════════════════════════════════════════════════════════════╝"
echo ""

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test 1: Check Ollama is installed
echo -n "Test 1: Checking Ollama installation... "
if command -v ollama &> /dev/null; then
    echo -e "${GREEN}✓ PASS${NC}"
else
    echo -e "${RED}✗ FAIL${NC} - Ollama not found"
    echo "Install with: brew install ollama"
    exit 1
fi

# Test 2: Check Ollama is running
echo -n "Test 2: Checking Ollama service... "
if curl -s http://localhost:11434/api/version &> /dev/null; then
    echo -e "${GREEN}✓ PASS${NC}"
else
    echo -e "${YELLOW}⚠ WARN${NC} - Ollama not running"
    echo "Start with: brew services start ollama"
fi

# Test 3: Check if model is pulled
echo -n "Test 3: Checking llama3.2:3b model... "
if ollama list | grep -q "llama3.2:3b"; then
    echo -e "${GREEN}✓ PASS${NC}"
else
    echo -e "${YELLOW}⚠ WARN${NC} - Model not found"
    echo "Pull with: ollama pull llama3.2:3b"
fi

# Test 4: Check Rust build
echo -n "Test 4: Checking Rust build... "
if cargo build --release --quiet 2>/dev/null; then
    echo -e "${GREEN}✓ PASS${NC}"
else
    echo -e "${RED}✗ FAIL${NC} - Build errors detected"
    echo "Run: cargo build --release"
    exit 1
fi

# Test 5: Check SpacetimeDB
echo -n "Test 5: Checking SpacetimeDB... "
if command -v spacetime &> /dev/null; then
    echo -e "${GREEN}✓ PASS${NC}"
else
    echo -e "${YELLOW}⚠ WARN${NC} - SpacetimeDB not installed (optional)"
    echo "Install from: https://spacetimedb.com"
fi

# Test 6: Check key files exist
echo -n "Test 6: Checking implementation files... "
if [ -f "src/llm_client.rs" ] && \
   [ -f "src/tui/screens/mission_control.rs" ] && \
   [ -f "examples/sage_irc_llm_bot.rs" ]; then
    echo -e "${GREEN}✓ PASS${NC}"
else
    echo -e "${RED}✗ FAIL${NC} - Missing implementation files"
    exit 1
fi

# Test 7: Quick LLM test (if Ollama is running)
if curl -s http://localhost:11434/api/version &> /dev/null; then
    echo -n "Test 7: Testing LLM generation... "
    response=$(curl -s -X POST http://localhost:11434/api/generate \
        -d '{"model":"llama3.2:3b","prompt":"Test","stream":false}' 2>/dev/null)

    if echo "$response" | grep -q "response"; then
        echo -e "${GREEN}✓ PASS${NC}"
    else
        echo -e "${YELLOW}⚠ WARN${NC} - LLM test inconclusive"
    fi
else
    echo "Test 7: Skipped (Ollama not running)"
fi

echo ""
echo "╔════════════════════════════════════════════════════════════╗"
echo "║                    Test Summary                            ║"
echo "╚════════════════════════════════════════════════════════════╝"
echo ""
echo "Next steps:"
echo ""
echo "1. Start Ollama (if needed):"
echo "   ${YELLOW}brew services start ollama${NC}"
echo ""
echo "2. Pull model (if needed):"
echo "   ${YELLOW}ollama pull llama3.2:3b${NC}"
echo ""
echo "3. Start IRC bot:"
echo "   ${YELLOW}cargo run --release --example sage_irc_llm_bot${NC}"
echo ""
echo "4. Launch Mission Control TUI:"
echo "   ${YELLOW}cargo run --release${NC}"
echo ""
echo "5. Connect to IRC and chat!"
echo "   Server: irc.libera.chat:6667"
echo "   Channel: #sage-ai"
echo ""
echo "See ${GREEN}SAGE_LLM_QUICKSTART.md${NC} for detailed instructions."
echo ""
