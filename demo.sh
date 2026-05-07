#!/bin/bash
# SAGE Demo Script
# Shows the core value proposition: personal AI that learns from you

set -e

echo "╔════════════════════════════════════════════════════════════╗"
echo "║                    SAGE Interactive Demo                     ║"
echo "║          Your personal AI that learns from you             ║"
echo "╚════════════════════════════════════════════════════════════╝"
echo ""

# Check if sage is installed
if ! command -v sage &> /dev/null; then
    echo "❌ SAGE not found. Install with:"
    echo "   curl -fsSL https://whatssage.ai/install.sh | bash"
    exit 1
fi

echo "✅ SAGE is installed"
echo ""

# Demo Part 1: Personal Knowledge Store
echo "═══════════════════════════════════════════════════════════════"
echo "DEMO 1: Personal Knowledge Store"
echo "═══════════════════════════════════════════════════════════════"
echo ""
echo "SAGE remembers what you tell it and retrieves it when relevant."
echo ""
echo "Let's encode some knowledge..."
echo ""

# Simulate encoding knowledge
echo "User: My favorite programming language is Rust"
echo "SAGE: [Encoding knowledge into neural grid...]"
echo ""
echo "User: I live in Milwaukee and work on AI systems"
echo "SAGE: [Encoding knowledge into neural grid...]"
echo ""
echo "User: My dog's name is Baxter, he's a golden retriever"
echo "SAGE: [Encoding knowledge into neural grid...]"
echo ""

# Demo Part 2: Retrieval
echo "═══════════════════════════════════════════════════════════════"
echo "DEMO 2: Contextual Retrieval"
echo "═══════════════════════════════════════════════════════════════"
echo ""
echo "Later, when you ask related questions, SAGE remembers:"
echo ""
echo "User: What language should I use for a systems project?"
echo "SAGE: [Retrieving relevant knowledge...]"
echo "       Based on your preferences, I'd recommend Rust."
echo "       You've mentioned it's your favorite language."
echo ""
echo "User: Tell me about my pets"
echo "SAGE: [Retrieving relevant knowledge...]"
echo "       You have a golden retriever named Baxter."
echo ""

# Demo Part 3: Feedback Stats
echo "═══════════════════════════════════════════════════════════════"
echo "DEMO 3: Learning From Usage"
echo "═══════════════════════════════════════════════════════════════"
echo ""
echo "SAGE tracks how well it's serving you:"
echo ""
echo "$ sage feedback stats"
echo ""
# Show example stats
sage feedback stats 2>/dev/null || cat << 'EOF'
╔══════════════════════════════════════════╗
║         SAGE Feedback Statistics         ║
╚══════════════════════════════════════════╝

Total queries tracked: 42
NCA attempts:          38
NCA satisfaction rate: 84.2%
LLM fallback rate:     9.5%

Pattern breakdown:
  factual_lookup: 15 queries, 93.3% NCA success
  definitional: 8 queries, 87.5% NCA success
  procedural: 6 queries, 66.7% NCA success
  comparative: 5 queries, 60.0% NCA success
  causal: 4 queries, 50.0% NCA success
  temporal: 3 queries, 100.0% NCA success
  other: 1 queries, 100.0% NCA success
EOF
echo ""

# Demo Part 4: Network Sync
echo "═══════════════════════════════════════════════════════════════"
echo "DEMO 4: Decentralized Sync"
echo "═══════════════════════════════════════════════════════════════"
echo ""
echo "Your knowledge syncs across devices via peer-to-peer network:"
echo ""
echo "$ sage node start"
echo "🌐 SAGE node starting..."
echo "   Node ID: swift-harbor-7a3f"
echo "   Gossip port: 4001"
echo "   Chat port: 19175"
echo ""
echo "   📡 Broadcasting presence..."
echo "   🔗 Connected peers: 3"
echo "   📤 Knowledge diffs sent: 12"
echo "   📥 Knowledge diffs received: 8"
echo ""
echo "✅ Your knowledge is now syncing across your devices"
echo ""

# Summary
echo "═══════════════════════════════════════════════════════════════"
echo "SUMMARY: What SAGE Does For You"
echo "═══════════════════════════════════════════════════════════════"
echo ""
echo "1. 🧠 Remembers Everything You Tell It"
echo "   - Conversations, facts, preferences"
echo "   - Stored locally on your machine"
echo ""
echo "2. 🔍 Retrieves Relevant Context Automatically"
echo "   - No need to search your notes"
echo "   - Brings up what matters when it matters"
echo ""
echo "3. 📈 Learns From Your Feedback"
echo "   - Gets better at serving you over time"
echo "   - Tracks satisfaction per query type"
echo ""
echo "4. 🌐 Syncs Across Your Devices"
echo "   - Peer-to-peer, no cloud required"
echo "   - Your data stays yours"
echo ""
echo "═══════════════════════════════════════════════════════════════"
echo ""
echo "Ready to try it yourself?"
echo "   $ sage chat          # Start chatting"
echo "   $ sage node start    # Join the network"
echo "   $ sage feedback stats # See your stats"
echo ""
echo "Documentation: https://whatssage.ai/docs"
echo "Discord: https://discord.gg/U999zZUuUV"
echo ""
