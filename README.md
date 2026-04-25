<p align="center">
  <img src="https://via.placeholder.com/200x200?text=SAGE" alt="SAGE Logo" width="200" height="200">
</p>

<h1 align="center">SAGE</h1>

<p align="center">
  <strong>The People's AI — Decentralized. Private. Free forever.</strong>
</p>

<p align="center">
  <a href="https://github.com/caryyon/sage/releases"><img src="https://img.shields.io/github/v/release/caryyon/sage?style=flat-square" alt="Version"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue?style=flat-square" alt="License"></a>
  <a href="https://discord.gg/U999zZUuUV"><img src="https://img.shields.io/discord/1234567890?style=flat-square&logo=discord&logoColor=white&label=Discord" alt="Discord"></a>
  <img src="https://img.shields.io/badge/rust-1.75+-orange?style=flat-square&logo=rust" alt="Rust">
  <a href="https://github.com/caryyon/sage/actions"><img src="https://img.shields.io/github/actions/workflow/status/caryyon/sage/ci.yml?style=flat-square" alt="Build"></a>
</p>

---

**BitTorrent for intelligence.** SAGE encodes knowledge into Neural Cellular Automata patterns and shares them across a peer-to-peer mesh network. Your raw conversations never leave your machine — only compact, encoded patterns sync between nodes.

The more people run SAGE, the smarter it gets for everyone.

## Quick Start

```bash
curl -fsSL https://whatssage.ai/install.sh | bash
sage chat
```

No accounts. No API keys. No setup wizards.

## Why SAGE?

| | **SAGE** | **ChatGPT** | **Ollama** | **Cloud APIs** |
|---|:---:|:---:|:---:|:---:|
| Free forever | ✅ | ❌ $20/mo | ✅ | ❌ pay/token |
| Runs 100% locally | ✅ | ❌ | ✅ | ❌ |
| Gets smarter over time | ✅ mesh | ❌ | ❌ | ❌ |
| No account needed | ✅ | ❌ | ✅ | ❌ |
| Privacy-first | ✅ | ❌ | ✅ | ❌ |
| Decentralized | ✅ | ❌ | ❌ | ❌ |
| Works offline | ✅ | ❌ | ✅ | ❌ |
| No rate limits | ✅ | ❌ | ✅ | ❌ |
| OpenAI compatible API | ✅ | is OpenAI | ✅ | ✅ |

## How It Works

1. **You chat** — Ask questions, have conversations, explore topics
2. **Knowledge encodes** — Patterns compress into NCA grid states (kilobytes, not gigabytes)
3. **Patterns sync** — Encoded knowledge propagates to peers via gossip protocol
4. **Everyone benefits** — Your node absorbs knowledge from the network

Your raw data never leaves your machine. Only encoded neural patterns are shared — compressed, anonymous knowledge representations that can't be reversed into the original text.

## Architecture

SAGE uses a **256×256 Neural Cellular Automata grid** with 16 channels per cell as its knowledge substrate. This is not a database — it's a living neural structure that evolves via local update rules.

- **Knowledge encoding**: Text → semantic embeddings → NCA grid patterns
- **Knowledge retrieval**: Cross-attention decoder (query = your question, keys/values = grid cells)
- **Network sync**: Gossip protocol exchanges Merkle-verified, Ed25519-signed diffs between peers
- **Trust model**: Validation tiers, quarantine for suspicious diffs, rate limiting
- **Retrieval quality**: ~96% hit rate with learned LinearProjection embeddings

For technical depth, see [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md).

## Commands

| Command | Description |
|---------|-------------|
| `sage chat` | Start a conversation |
| `sage chat --ollama` | Use Ollama as LLM backend |
| `sage node start` | Join the mesh network |
| `sage node status` | See connected peers |
| `sage node stop` | Disconnect from mesh |
| `sage status` | Show local health & metrics |
| `sage explore` | Trigger curiosity exploration |
| `sage export <file>` | Export knowledge state |
| `sage import <file>` | Import knowledge snapshot |

## Network Dashboard

See the live mesh at [whatssage.ai/network](https://whatssage.ai/network)

- Node count, peer connections, sync stats
- Knowledge diff throughput
- Network health and geographic distribution

## Manifesto

Read [docs/MANIFESTO.md](docs/MANIFESTO.md) for the full vision: *The Future of AI is Distributed.*

## Documentation

- [Getting Started](docs/GETTING_STARTED.md)
- [Architecture](docs/ARCHITECTURE.md)
- [API Reference](docs/API.md)
- [Distributed Systems](docs/DISTRIBUTED.md)
- [NCA Inference](docs/NCA_INFERENCE.md)
- [Roadmap](ROADMAP.md)

## Contributing

We welcome contributions! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## Community

- **Discord:** [https://discord.gg/U999zZUuUV](https://discord.gg/U999zZUuUV)
- **GitHub Issues:** Bug reports, feature requests, discussions
- **Network Dashboard:** [whatssage.ai/network](https://whatssage.ai/network)

## License

MIT — see [LICENSE](LICENSE)

---

*SAGE: The People's AI*
