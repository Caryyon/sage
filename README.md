# SAGE — The People's AI

<img src="https://raw.githubusercontent.com/Caryyon/sage/main/sage-logo.svg" alt="SAGE Logo" width="200" height="200">

**Decentralized AI that runs on your machine. No API keys. No cloud. No gatekeepers.**

SAGE is a Neural Cellular Automata intelligence system that learns, grows, and connects — all on your hardware. Your knowledge stays on your machine. Your conversations are private. Your AI is yours.

## Why SAGE?

| | Centralized AI | SAGE |
|---|---|---|
| **Privacy** | ❌ Your data trains their models | ✅ Everything stays local |
| **Cost** | ❌ $20/month subscription | ✅ Free forever |
| **Offline** | ❌ Requires internet | ✅ Works without connection |
| **Ownership** | ❌ They can shut it down | ✅ You own the software |
| **Decentralized** | ❌ Single point of failure | ✅ Peer-to-peer mesh |

## Quick Start

```bash
# Install
curl -fsSL https://whatssage.ai/install.sh | bash

# Chat with your personal AI
sage chat
> What is SAGE?
SAGE is a decentralized AI system...

# See how well it's learning from you
sage feedback stats

# Join the mesh to sync across devices
sage node start
```

## What SAGE Does

**1. Remembers Everything You Tell It**
Chat with SAGE about anything — your projects, ideas, preferences, notes. It stores knowledge in a 256×256 neural grid, retrieving relevant context when you need it later.

**2. Learns From Your Feedback**  
SAGE tracks which responses satisfy you and which fall back to LLM. Over time, it gets better at serving you faster with local inference for queries it handles well.

**3. Syncs Across Your Devices**
Run SAGE on multiple machines. Your knowledge automatically syncs peer-to-peer via libp2p gossip — no cloud required, your data stays yours.

**Try the Demo**
```bash
./demo.sh  # See SAGE in action
```

## Features

- **Grid-Based Knowledge Store** — 256×256 grid stores semantic embeddings with attention-based retrieval
- **Intelligent Query Router** — Learning-based routing (v0.3.7): detects 12 query patterns, tracks NCA vs LLM accuracy, adapts over time
- **Attention-Based Retrieval** — Cross-attention decoder finds relevant knowledge with delta attention spreading
- **Peer-to-Peer Mesh** — Nodes share knowledge diffs via libp2p gossip protocols
- **Retrieval Feedback Learning** — System improves from relevance signals using Adam optimizer
- **Bootstrap Peer Config** — Configure WAN bootstrap peers via `~/.sage/config.toml` (v0.3.8)
- **Raspberry Pi Ready** — Runs on a $35 board
- **Zero API Keys** — No OpenAI, no monthly fees

> **Note:** The "Neural Cellular Automata" branding is aspirational for the token-prediction
> grid (NcaPredictor). The knowledge grid now has real NCA-style dynamics (v0.5.0) through
> `consolidate_knowledge()` — Hebbian reinforcement, decay, and spreading activation.
> See [Architecture docs](docs/architecture/nca-brain.md) for details.

## Architecture

- **Rust** — Memory-safe, fast, no Python dependency hell
- **Grid Store** — Knowledge stored as semantic embeddings in 256×256 grid
- **libp2p** — Decentralized peer-to-peer networking
- **Ollama** — LLM fallback for complex reasoning
- **Ed25519** — Cryptographic identity and signed updates
- **Semantic Hashing** — 0% collision feature-to-position mapping

## Platforms

- ✅ Linux (x86_64, ARM64)
- ✅ macOS (Intel, Apple Silicon)
- ✅ Windows
- ✅ Raspberry Pi 4
- 🔄 Browser (WASM — in progress)
- ✅ Docker

## Documentation

- [Install Guide](docs/getting-started/install.md)
- [Offline Mode](docs/getting-started/offline.md)
- [Architecture](docs/architecture/nca-brain.md)
- [Contributing](CONTRIBUTING.md)

## Community

- [Discord](https://discord.gg/U999zZUuUV)
- [GitHub Issues](https://github.com/Caryyon/sage/issues)
- [Network Dashboard](https://whatssage.ai/network)

## License

MIT — use it, modify it, share it.

---

**SAGE is the future of AI — distributed, owned by its users, and getting smarter together.**
