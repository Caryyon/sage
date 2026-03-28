# Getting Started with SAGE

SAGE is a decentralized AI that runs locally and shares knowledge across a peer-to-peer network. No accounts, no API keys, no monthly fees.

**Current version:** 0.2.9

---

## Installation

One command:

```bash
curl -fsSL https://whatssage.ai/install.sh | bash
```

This downloads the `sage` binary for your platform (Linux/macOS, x86_64/arm64) and installs it to `~/.sage/bin/sage`. It also adds `~/.sage/bin` to your PATH.

After install, open a new terminal (or `source ~/.bashrc`) and verify:

```bash
sage version
# sage 0.2.9
```

---

## First Run

```bash
sage chat
```

On first run, SAGE will:

1. Create `~/.sage/` directory for config and data
2. Initialize an empty brain (NCA grid)
3. Download bundled embedding model (~22MB) to `~/.cache/fastembed/`
4. Start the chat interface

You'll see something like:

```
SAGE v0.2.9 — The People's AI
Brain: ~/.sage/brain.bin (0 active cells)
Embeddings: bundled (AllMiniLML6V2, 384-dim)

Type /help for commands, /quit to exit.

you>
```

### First Run with Ollama

If you have [Ollama](https://ollama.com) installed:

```bash
sage chat --ollama                      # Uses qwen2.5:14b by default
sage chat --ollama --model llama3.2:3b  # Pick a different model
```

With Ollama, you get:
- Higher quality LLM responses
- Semantic embeddings via nomic-embed-text (60% retrieval hit rate)

Without Ollama, SAGE uses:
- Bundled fastembed for embeddings (96% retrieval hit rate)
- A fallback response mode (limited without LLM backend)

### In-Chat Commands

| Command | Description |
|---------|-------------|
| `/help` | Show available commands |
| `/quit` or `/exit` | Exit chat |
| `/status` | Show node health and retrieval stats |
| `/dream` | Trigger a dream cycle |
| `/explore` | Trigger curiosity exploration |

---

## How Knowledge Retrieval Works

SAGE uses three retrieval strategies, selected automatically based on availability:

### 1. Semantic Retrieval (Best)

**When:** Ollama running with nomic-embed-text
**Hit rate:** ~60%
**How:** Cross-attention decoder queries NCA grid using semantic embeddings

### 2. Contrast Retrieval (Good)

**When:** Bundled fastembed available (default since v0.2.9)
**Hit rate:** ~96%
**How:** Query-conditioned delta attention over local grid regions

### 3. Hash Fallback (Basic)

**When:** No embedding model available
**Hit rate:** ~12%
**How:** Hash-based feature matching with cosine similarity

Check which mode is active:

```bash
sage status
# Shows: Embeddings: bundled (AllMiniLML6V2, 384-dim)
```

---

## Joining the Mesh Network

```bash
sage node start
```

Your node connects to the decentralized network:

```
SAGE Node starting...
  Identity: 12D3KooW...
  Brain: ~/.sage/brain.bin (42 active cells)
  Gossip: listening
  mDNS: enabled

Node running. Press Ctrl+C to stop.
```

### What Happens

1. **LAN Discovery** — mDNS finds other SAGE nodes on your local network
2. **Bootstrap** — Connects to `bootstrap.whatssage.ai:4001` for internet peers
3. **Knowledge Sync** — Exchanges Merkle-verified diffs with peers
4. **Brain Saves** — Periodically saves brain state to disk

### Node Commands

```bash
sage node start               # Start node
sage node start --port 9000   # Custom gossip port
sage node status              # Check peers and sync state
sage node stop                # Stop node
```

---

## OpenAI-Compatible API

SAGE exposes an API at `localhost:19176/v1`. Point any OpenAI-compatible tool at it:

```bash
export OPENAI_API_BASE=http://localhost:19176/v1
export OPENAI_API_KEY=not-needed
```

### Works With

- **[Continue](https://continue.dev)** — Set base URL in config
- **[Open WebUI](https://openwebui.com)** — Add as OpenAI-compatible endpoint
- **[Cursor](https://cursor.sh)** — Configure API settings
- **LangChain** — `ChatOpenAI(base_url="http://localhost:19176/v1")`

### Example API Call

```bash
curl http://localhost:19176/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "sage",
    "messages": [{"role": "user", "content": "Hello!"}]
  }'
```

No API key required. No rate limits. No billing.

---

## Troubleshooting

### "command not found: sage"

`~/.sage/bin` isn't in your PATH. Fix:

```bash
export PATH="$HOME/.sage/bin:$PATH"
# Add to ~/.bashrc or ~/.zshrc for persistence
```

Or re-run the install script.

### Brain version mismatch

If you see a warning about brain.bin format, SAGE now handles this gracefully since v0.2.8. It will either migrate the brain or start fresh if migration fails.

To manually reset:

```bash
rm ~/.sage/brain.bin
sage chat  # Creates fresh brain
```

### Ollama not running

SAGE works without Ollama since v0.2.9. You'll see:

```
Ollama not available, using bundled embeddings
Embeddings: bundled (AllMiniLML6V2, 384-dim)
```

This is fine — bundled embeddings actually have higher retrieval accuracy (96% vs 60%).

### Node won't connect to peers

- Check firewall allows outbound connections
- Verify `bootstrap.whatssage.ai:4001` is reachable
- Try `sage node start --no-mdns` if mDNS causes issues

### Slow first run

The bundled embedding model (~22MB) downloads on first use. This only happens once.

### Change data directory

```bash
export SAGE_HOME=/path/to/custom/sage
sage chat  # Uses /path/to/custom/sage/brain.bin
```

---

## Configuration

Config lives at `~/.sage/config.toml`:

```bash
sage config          # Print current config
sage config --path   # Print config file path
```

### Example Config

```toml
[network]
bootstrap = "bootstrap.whatssage.ai:4001"
listen_port = 0          # 0 = random
chat_port = 19175
sync_interval = 300      # seconds
mdns = true

[privacy]
shared_channels = [8, 9, 10, 11, 12, 13, 14, 15]
min_conversations_before_share = 5
dp_epsilon = 1.0
private_topics = ["health", "finance"]
```

---

## Next Steps

- **Join the network** — `sage node start`
- **Explore docs** — [ARCHITECTURE.md](./ARCHITECTURE.md) for technical depth
- **Join Discord** — [discord.gg/U999zZUuUV](https://discord.gg/U999zZUuUV)
- **Contribute** — See [CONTRIBUTING.md](../CONTRIBUTING.md)
