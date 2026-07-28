# SAGE — The People's AI

<img src="https://raw.githubusercontent.com/Caryyon/sage/main/sage-logo.svg" alt="SAGE Logo" width="200" height="200">

**Free, local AI that reads your documents and answers questions. No API keys. No cloud. No subscription.**

SAGE runs entirely on your machine. Feed it PDFs, text files, or web pages, and it remembers what it read. Ask questions in plain English and get answers grounded in your documents. Your data never leaves your computer.

## Quick Start

```bash
# Install
curl -fsSL https://whatssage.ai/install.sh | bash

# Feed it a document
sage learn ~/Documents/notes.txt

# Ask a question
sage search "What did I write about project timelines?"

# Interactive chat
sage chat
```

That's it. No API keys to set up. No account to create. No monthly bill.

## What SAGE Does

**1. Learns from your documents**
Feed SAGE text files, markdown, or any plain text. It chunks the content, embeds it, and stores it locally in a knowledge base that persists across sessions.

```bash
sage learn ~/Documents/report.txt
sage learn ~/notes.md --fastembed   # No Ollama needed (384-dim mode)
```

**2. Answers questions about what it's read**
Ask questions in plain English. SAGE retrieves relevant passages from its knowledge base and synthesizes answers using a local LLM.

```bash
sage search "What are the key findings in the Q3 report?"
sage chat   # Interactive mode with brain visualization
```

**3. Works completely offline**
No API keys. No cloud calls. No telemetry. The LLM (SmolLM2 1.7B) runs in-process, or you can use Ollama with any local model.

**4. Shares knowledge peer-to-peer**
Run SAGE on multiple machines. Knowledge syncs over libp2p — no central server, no cloud. Your data stays yours.

```bash
sage node start    # Join the P2P network
```

## How It Works

SAGE has three layers:

1. **HDC Knowledge Store** — Every chunk of text you feed it gets embedded (768-dim via Ollama, or 384-dim via fastembed) and stored in a Hyperdimensional Computing store. Retrieval is cosine similarity + keyword fusion. This is the memory.

2. **LLM Synthesis** — When you ask a question, SAGE retrieves the top relevant passages and a local LLM (SmolLM2 1.7B bundled, or any Ollama model) reads them and generates an answer. This is the reasoning.

3. **NCA Grid** — A Neural Cellular Automata grid learns patterns from the knowledge store during "sleep" cycles. This is experimental — it provides intuition and association, not generation. (Work in progress.)

## Platforms

- ✅ Linux (x86_64, ARM64)
- ✅ macOS (Intel, Apple Silicon)
- 🚧 Windows (build from source)

## Requirements

- **Minimal mode:** 500MB RAM, no GPU, no Ollama (fastembed + embedded SmolLM2)
- **Full mode:** 2GB RAM, Ollama installed (for larger models and 768-dim embeddings)
- **Disk:** ~250MB for SAGE + ~1GB for SmolLM2 model (auto-downloaded on first chat)

## Trained AI Specialists (New in v0.6.0)

SAGE comes with 8 pre-trained specialist brains. Each one is a ~40MB trained brain that downloads on demand and answers questions locally — no API keys, no cloud, no training required.

```bash
# See available specialists
sage specialist list

# Hire one (downloads the trained brain)
sage specialist hire accounting

# Ask it a question
sage specialist ask accounting "What is a balance sheet?"

# See brain details
sage specialist info accounting
```

Available specialists:

| Specialist | Trained On |
|------------|------------|
| accounting | Financial statements, bookkeeping, GAAP |
| customer-support | Support best practices, escalation, CSAT |
| data-analyst | SQL, statistics, data visualization |
| high-school-graduate | General knowledge across subjects |
| marketing | Marketing strategy, channels, branding |
| paralegal | Legal research, contracts, civil procedure |
| software-engineer | Data structures, algorithms, system design |
| cs-fundamentals | CS theory, complexity, automata |

Each specialist works with local synthesis (no Ollama needed) or falls back to Ollama for complex queries.

## Commands

| Command | Description |
|---------|-------------|
| `sage chat` | Interactive chat with brain visualization |
| `sage learn <file>` | Feed a text file into the knowledge base |
| `sage search "query"` | Search the knowledge base (non-interactive) |
| `sage specialist list` | List available trained specialists |
| `sage specialist hire <name>` | Download a trained specialist brain |
| `sage specialist info <name>` | Show specialist brain details |
| `sage specialist ask <name> "query"` | Ask a specialist a question |
| `sage status` | Show brain stats (entries, memory usage, grid state) |
| `sage node start` | Start P2P networking and sync with other nodes |
| `sage export <file>` | Export knowledge to a .sage file (for sneakernet sharing) |
| `sage import <file>` | Import knowledge from a .sage file |
| `sage insights` | Show knowledge statistics and insights |
| `sage feedback stats` | Show learning statistics |
| `sage dream` | Run a consolidation cycle (NCA sleep) |
| `sage update` | Update SAGE to the latest version |
| `sage version` | Print version info |

## OpenAI-Compatible API

SAGE includes an API server that's compatible with the OpenAI chat completions endpoint:

```bash
sage-api --port 19175 --api-port 19176

# Use with any OpenAI client
OPENAI_API_BASE=http://localhost:19176/v1 python3 your_script.py
```

Endpoints:
- `POST /v1/chat/completions` — Chat completions (streaming supported)
- `POST /v1/embeddings` — Text embeddings
- `GET /v1/sage/status` — Brain stats
- `GET /v1/sage/brain` — NCA grid state
- `GET /health` — Health check

## Build from Source

```bash
git clone https://github.com/Caryyon/sage.git
cd sage
cargo build --release

# With CUDA support (for GPU inference)
cargo build --release --features cuda

# With local LLM (llama.cpp)
cargo build --release --features local-llm
```

## Project Status

SAGE is in active development. The core RAG pipeline (HDC retrieval + LLM synthesis) is stable and working. The NCA grid is experimental research. See [docs/v0.6.0-the-actual-plan.md](docs/v0.6.0-the-actual-plan.md) for the roadmap.

**Current version:** v0.6.0

## License

MIT

## Community

- Discord: https://discord.gg/U999zZUuUV
- Website: https://whatssage.ai
- GitHub: https://github.com/Caryyon/sage