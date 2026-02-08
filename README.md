# SAGE — Shared Adaptive Growing Experience

> **The People's AI.** Free. Local. Gets smarter together.

SAGE is a decentralized AI that runs on your machine. Every node contributes knowledge to the network, and the network makes every node smarter. No API keys. No cloud. No corporate middleman. Just people running AI together.

Think of it like **BitTorrent for intelligence** — each node learns locally, and knowledge propagates across the mesh via a gossip protocol. The more people who run SAGE, the smarter it gets for everyone.

## Quick Start

```bash
# Install SAGE
curl -fsSL https://sage.lattice.black/install.sh | sh

# Start chatting
sage chat

# Join the network (share & receive knowledge)
sage node start
```

That's it. You're now part of a distributed intelligence network.

## How It Works

1. **You use SAGE** — ask questions, have conversations, explore topics
2. **SAGE encodes knowledge** — patterns are compressed into NCA grid states
3. **Knowledge syncs** — encoded patterns propagate to nearby nodes via gossip protocol
4. **Everyone gets smarter** — your node absorbs knowledge from the network

Your raw data never leaves your machine. Only encoded neural patterns are shared — compressed, anonymous knowledge representations.

## Why SAGE?

| | **SAGE** | **ChatGPT** | **Ollama** | **Cloud APIs** |
|---|---|---|---|---|
| Free forever | ✅ | ❌ $20/mo | ✅ | ❌ pay/token |
| Runs locally | ✅ | ❌ | ✅ | ❌ |
| Gets smarter over time | ✅ (network) | ❌ | ❌ | ❌ |
| No account needed | ✅ | ❌ | ✅ | ❌ |
| Privacy-first | ✅ | ❌ | ✅ | ❌ |
| Decentralized | ✅ | ❌ | ❌ | ❌ |
| No rate limits | ✅ | ❌ | ✅ | ❌ |
| OpenAI compatible | ✅ | is OpenAI | ✅ | ✅ |

## Architecture

SAGE's core is a **32×32 Neural Cellular Automata grid** with 22 channels per cell. Knowledge is encoded as self-organizing, self-healing grid patterns — not stored as text. These patterns are compact (kilobytes, not gigabytes) and can be shared without revealing the original data.

Nodes discover each other and exchange knowledge via a gossip protocol. Sync is lazy, runs in the background, and never blocks your chat.

## Commands

```bash
sage chat                 # Start a conversation
sage node start           # Join the mesh network
sage node status          # See connected peers and sync state
sage node stop            # Disconnect from the mesh
sage dream                # Trigger a dream cycle manually
sage explore              # Trigger curiosity exploration
sage status               # Show local node health & metrics
sage export <file>        # Export your knowledge state
sage import <file>        # Import a knowledge snapshot
```

## OpenAI Compatible

```bash
export OPENAI_API_BASE=http://localhost:19176/v1
# Now any OpenAI-compatible tool uses SAGE
```

Works with [Continue](https://continue.dev), [Cursor](https://cursor.sh), [Open WebUI](https://openwebui.com), and anything that speaks the OpenAI API.

## Roadmap

- ✅ **Phase 1** — Local chat with NCA-based knowledge encoding
- ✅ **Phase 2** — Mesh network with gossip protocol knowledge sync
- 🔨 **Phase 3** — Kill the LLM dependency
- 🔮 **Phase 4** — Pure NCA intelligence

## Community

- 💬 [Discord](https://discord.gg/YXThZcrPHc)
- 📖 [Getting Started](docs/GETTING_STARTED.md)
- 📚 [Documentation](docs/)

## License

MIT
