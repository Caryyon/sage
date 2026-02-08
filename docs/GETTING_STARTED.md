# Getting Started with SAGE

> **SAGE** — Shared Adaptive Growing Experience

## Install (one command, 30 seconds)

```bash
curl -fsSL https://sage.lattice.black/install.sh | sh
```

## Chat with SAGE

```bash
sage chat
```

That's it! Your node automatically:
- Downloads a small AI model (~1GB, first time only)
- Creates your brain at `~/.sage/brain.bin`
- Connects to the SAGE network
- Gets smarter with every conversation

---

## Commands

| Command | Description |
|---------|-------------|
| `sage-node --port 19175` | Start your node |
| `sage chat --port 19175` | Chat with SAGE |
| `sage-api --api-port 19176` | OpenAI-compatible API |

## Chat Commands

| Command | Description |
|---------|-------------|
| `/status` | Your node stats |
| `/peers` | Connected nodes |
| `/knowledge` | What your brain knows |
| `/private` | Toggle private mode (no sync) |
| `/help` | All commands |

## Use with Other Tools

SAGE exposes an OpenAI-compatible API. Point any tool at it:

```bash
export OPENAI_API_BASE=http://localhost:19176/v1
# Now any OpenAI-compatible tool uses SAGE — no API key needed
```

Works with [Continue](https://continue.dev), [Cursor](https://cursor.sh), [Open WebUI](https://openwebui.com), and anything that speaks the OpenAI API.

## Run Multiple Nodes

```bash
SAGE_HOME=~/.sage  sage-node --port 19175    # Node A
SAGE_HOME=~/.sage2 sage-node --port 19176    # Node B
# They find each other automatically on your network
```

## Configuration

Your config lives at `~/.sage/config.toml`:

```toml
[network]
bootstrap = ["bootstrap.sage.lattice.black:4001"]

[privacy]
share_knowledge = true   # sync learned knowledge with peers
private_mode = false      # start in private mode (no sync)
```

## Troubleshooting

**"Connection refused" on chat** — Your node isn't running. Start it first: `sage-node`

**"No peers found"** — Check your internet connection and make sure `~/.sage/config.toml` has the bootstrap node configured.

**Model download is slow** — First run downloads ~1GB. Subsequent starts are instant.

**Port already in use** — Pick a different port: `sage-node --port 19177`

**Reset everything** — `rm -rf ~/.sage` and re-run the installer.

## Community

- 💬 [Discord](https://discord.gg/YXThZcrPHc)
- 📚 [Documentation](.)
- 🌐 [SAGE Website](https://sage.lattice.black)
