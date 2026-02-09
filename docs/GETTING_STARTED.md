# 🌿 Getting Started with SAGE

SAGE is a decentralized AI that runs locally on your machine and shares knowledge across a peer-to-peer network. No accounts, no API keys, no monthly fees.

---

## 📦 Installation

One command:

```bash
curl -fsSL https://sage.lattice.black/install.sh | bash
```

This downloads the `sage` binary for your platform (Linux/macOS, x86_64/arm64) and installs it to `~/.sage/bin/sage`. It also adds `~/.sage/bin` to your PATH.

After install, open a new terminal (or `source ~/.bashrc`) and verify:

```
$ sage version
sage 0.1.0
```

---

## 💬 First Chat

```bash
sage chat
```

On first run, SAGE downloads **SmolLM2 1.7B** (~1 GB) — a small but capable language model that runs entirely on your CPU. No GPU needed.

```
SAGE Chat — engine: SmolLM2-1.7B
Brain: /home/you/.sage/brain.bin (0 active cells)
Type your message, or /quit to exit.

you> What is SAGE?
sage> I'm SAGE — a decentralized AI running locally on your machine...
```

**Commands in chat:**
- Type your message and hit Enter
- `/quit` or `/exit` to leave

### Using Ollama Instead

If you have [Ollama](https://ollama.com) installed, you can use any Ollama model as the backend:

```bash
sage chat --ollama                          # Uses qwen2.5:14b by default
sage chat --ollama --model llama3.2:3b      # Pick a different model
sage chat --ollama --ollama-url http://remote:11434  # Remote Ollama
```

---

## 🔗 Join the Network

```bash
sage node start
```

This starts your SAGE node and connects it to the decentralized network:

```
🧠 SAGE Node starting...
   Identity: 12D3KooW...
   Brain: /home/you/.sage/brain.bin (42 active cells)
   Gossip port: 0
   Chat port: 19175
   mDNS: enabled

✅ Node running. Press Ctrl+C to stop.
```

**What happens when your node starts:**

1. **LAN Discovery** — mDNS finds other SAGE nodes on your local network automatically
2. **Bootstrap Connection** — Connects to `bootstrap.sage.lattice.black:4001` to find peers on the internet
3. **Knowledge Sync** — Your node exchanges compressed knowledge diffs with peers
4. **Brain Saves** — Your brain file is periodically saved to disk

### Node Options

```bash
sage node start --port 9000          # Set gossip port (default: random)
sage node start --chat-port 19175    # Set chat port (default: 19175)
sage node start --sync-interval 600  # Sync every 10 min (default: 300s)
sage node start --no-mdns            # Disable LAN discovery
```

### Check Node Status

```bash
sage node status
```

```
SAGE Node Status
─────────────────
  Home:     /home/you/.sage
  Config:   /home/you/.sage/config.toml ✓
  Brain:    /home/you/.sage/brain.bin ✓
  Running:  PID 12345
```

### Stop the Node

```bash
sage node stop
```

---

## 🧠 How It Works

1. **You chat** with your local SAGE instance
2. **Conversations become knowledge** — encoded into a Neural Cellular Automata (NCA) grid, a compact 32×32 grid of multi-channel cells (~128 KB)
3. **Knowledge is shared as diffs** — tiny compressed updates (200–2000 bytes each) that represent *what was learned*, not what was said
4. **Diffs flow via gossip** — peer-to-peer, no central server, reaching the whole network in seconds
5. **Your raw conversations never leave your machine** — only abstract, lossy knowledge patterns are shared

The more people run SAGE, the more knowledge flows through the network, and the smarter every node becomes.

---

## ⚙️ Configuration

SAGE stores its config at `~/.sage/config.toml`. View it with:

```bash
sage config          # Print current config
sage config --path   # Print config file path
```

### Example Config

```toml
[network]
bootstrap = "bootstrap.sage.lattice.black:4001"
listen_port = 0          # 0 = random
chat_port = 19175
sync_interval = 300      # seconds
mdns = true

[privacy]
# Channels to share (semantic only by default)
shared_channels = [8, 9, 10, 11, 12, 13, 14, 15]
# Don't share until N conversations contribute
min_conversations_before_share = 5
# Differential privacy noise level
dp_epsilon = 1.0
# Topics to never share
private_topics = ["health", "finance"]
```

Set `SAGE_HOME` environment variable to change the data directory (default: `~/.sage`).

---

## 🔌 OpenAI-Compatible API

SAGE exposes an OpenAI-compatible API at `localhost:19176/v1`. Point any tool that speaks OpenAI protocol at it:

```bash
# Environment variable (works with most tools)
export OPENAI_API_BASE=http://localhost:19176/v1
export OPENAI_API_KEY=not-needed

# curl example
curl http://localhost:19176/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "sage",
    "messages": [{"role": "user", "content": "Hello!"}]
  }'
```

### Works With

- **Continue** — set base URL in config
- **Open WebUI** — add as an OpenAI-compatible endpoint
- **LangChain** — `ChatOpenAI(base_url="http://localhost:19176/v1")`
- **Any OpenAI SDK** — just change the base URL

No API key required. No rate limits. No billing.

---

## 📋 Commands Reference

| Command | Description |
|---------|-------------|
| `sage chat` | Interactive chat (default if no subcommand) |
| `sage chat --ollama` | Chat using Ollama backend |
| `sage chat --ollama --model <name>` | Chat with specific Ollama model |
| `sage node start` | Start the SAGE network node |
| `sage node stop` | Stop the running node |
| `sage node status` | Show node status |
| `sage config` | Show current configuration |
| `sage config --path` | Print config file path |
| `sage update` | Update SAGE to latest version |
| `sage update --quiet` | Update without showing changelog |
| `sage version` | Print version info |

---

## 🔄 Updating

```bash
sage update
```

This downloads the latest binary, shows the changelog, and migrates your brain file if needed. Your data in `~/.sage` is always preserved.

```
🔄 Checking for updates...

📋 Changelog:
## v0.2.0
- Improved knowledge sync
- Better NCA encoding
...

⬇️  Downloading...
✅ Binary updated
🧠 Brain file up to date

🎉 Update complete! Data in /home/you/.sage preserved.
```

Skip the changelog with `sage update --quiet`.

---

## 🔧 Troubleshooting

### "command not found: sage"

Make sure `~/.sage/bin` is in your PATH. Re-run the install script or add manually:

```bash
export PATH="$HOME/.sage/bin:$PATH"
```

### Model download is slow

The SmolLM2 model (~1 GB) downloads on first `sage chat`. If it's slow, check your internet connection. The download only happens once.

### Node won't connect to peers

- Check your firewall allows outbound connections on the gossip port
- Make sure `bootstrap.sage.lattice.black:4001` is reachable
- Try `sage node start --no-mdns` if mDNS is causing issues on your network

### "No running SAGE node found"

`sage node stop` requires a running node. Check with `sage node status`.

### Brain file corruption

If your brain file gets corrupted, delete it and SAGE will create a fresh one:

```bash
rm ~/.sage/brain.bin
```

You'll lose local knowledge but will re-sync from the network when you start a node.

### Change data directory

```bash
export SAGE_HOME=/path/to/custom/sage
sage chat  # Uses /path/to/custom/sage/brain.bin etc.
```

---

## 🔗 Links

- **GitHub:** [github.com/Caryyon/sage](https://github.com/Caryyon/sage)
- **Discord:** [discord.gg/YXThZcrPHc](https://discord.gg/YXThZcrPHc)
- **Website:** [sage.lattice.black](https://sage.lattice.black)
- **Technical Details:** [DISTRIBUTED.md](./DISTRIBUTED.md)
