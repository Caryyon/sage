<p align="center">
  <!-- Logo placeholder: Replace with actual logo when available -->
  <img src="https://via.placeholder.com/200x200?text=SAGE" alt="SAGE Logo" width="200" height="200">
</p>

<h1 align="center">SAGE</h1>

<p align="center">
  <strong>Decentralized AI that runs locally and gets smarter together.</strong>
</p>

<p align="center">
  <a href="https://github.com/caryyon/sage/releases"><img src="https://img.shields.io/github/v/release/caryyon/sage?style=flat-square" alt="Version"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue?style=flat-square" alt="License"></a>
  <a href="https://discord.gg/U999zZUuUV"><img src="https://img.shields.io/discord/1234567890?style=flat-square&logo=discord&logoColor=white&label=Discord" alt="Discord"></a>
  <img src="https://img.shields.io/badge/rust-1.75+-orange?style=flat-square&logo=rust" alt="Rust">
  <a href="https://github.com/caryyon/sage/actions"><img src="https://img.shields.io/github/actions/workflow/status/caryyon/sage/ci.yml?style=flat-square" alt="Build"></a>
</p>

---

**BitTorrent for intelligence.** SAGE encodes knowledge into Neural Cellular Automata patterns and shares them across a peer-to-peer mesh network. Your raw conversations never leave your machine — only compact, encoded patterns sync between nodes. The more people run SAGE, the smarter it gets for everyone.

## Quick Start

```bash
curl -fsSL https://whatssage.ai/install.sh | bash
sage chat
```

That's it. No accounts, no API keys, no setup wizards.

<!-- Demo GIF placeholder: Add a terminal recording showing sage chat in action -->

## Why SAGE?

| | **SAGE** | **ChatGPT** | **Ollama** | **Cloud APIs** |
|---|:---:|:---:|:---:|:---:|
| Free forever | ✅ | ❌ $20/mo | ✅ | ❌ pay/token |
| Runs 100% locally | ✅ | ❌ | ✅ | ❌ |
| Gets smarter over time | ✅ mesh | ❌ | ❌ | ❌ |
| No account needed | ✅ | ❌ | ✅ | ❌ |
| Privacy-first | ✅ | ❌ | ✅ | ❌ |
| Decentralized | ✅ | ❌ | ❌ | ❌ |
| No rate limits | ✅ | ❌ | ✅ | ❌ |
| OpenAI compatible | ✅ | is OpenAI | ✅ | ✅ |

## How It Works

1. **You chat** — Ask questions, have conversations, explore topics
2. **Knowledge encodes** — Patterns compress into NCA grid states (kilobytes, not gigabytes)
3. **Patterns sync** — Encoded knowledge propagates to peers via gossip protocol
4. **Everyone benefits** — Your node absorbs knowledge from the network

Your raw data never leaves your machine. Only encoded neural patterns are shared — compressed, anonymous knowledge representations that can't be reversed into the original text.

## Architecture

SAGE uses a **256×256 Neural Cellular Automata grid** with 38 channels per cell as its knowledge substrate. This is not a database — it's a living neural structure that evolves via local update rules.

- **Knowledge encoding**: Text → semantic embeddings → NCA grid patterns
- **Knowledge retrieval**: Cross-attention decoder (query = your question, keys/values = grid cells)
- **Network sync**: Gossip protocol exchanges Merkle-verified diffs between peers

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
| `sage dream` | Trigger a dream cycle |
| `sage explore` | Trigger curiosity exploration |
| `sage export <file>` | Export knowledge state |
| `sage import <file>` | Import knowledge snapshot |
| `/help` | Show help in chat |

## OpenAI Compatible

```bash
export OPENAI_API_BASE=http://localhost:19176/v1
export OPENAI_API_KEY=not-needed
```

Works with [Continue](https://continue.dev), [Cursor](https://cursor.sh), [Open WebUI](https://openwebui.com), and anything that speaks the OpenAI API. No rate limits. No billing.

## Current Status

**Version:** 0.2.9

- ✅ **Phase 1** — Local chat with NCA knowledge encoding
- ✅ **Phase 2** — Mesh network with gossip protocol
- 🔨 **Phase 3** — Reduce LLM dependency (bundled embeddings done, 96% retrieval)
- 🔮 **Phase 4** — Pure NCA intelligence

See [ROADMAP.md](ROADMAP.md) for details.

## Requirements

- **macOS or Linux** (x86_64 or arm64)
- **Ollama** (optional) — For best retrieval quality. Without Ollama, SAGE uses bundled fastembed (96% retrieval) or hash fallback.

## Contributing

We welcome contributions. See [CONTRIBUTING.md](CONTRIBUTING.md) for setup instructions, code style, and PR guidelines.

Join [#dev on Discord](https://discord.gg/U999zZUuUV) to discuss ideas.

## Community

- [Discord](https://discord.gg/U999zZUuUV) — Chat, support, dev discussions
- [Documentation](docs/) — Architecture, API, getting started
- [Website](https://whatssage.ai) — Project home

## License

MIT
