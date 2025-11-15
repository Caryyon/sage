# SAGE 2.0 - Quick Reference Card

## 🚀 Most Common Commands

```bash
# First time setup
make setup              # Install everything (Ollama, model, DB)

# Start SAGE
make tui                # Launch Mission Control TUI only
make irc                # Launch IRC bot only
make dev                # Launch everything in tmux (recommended!)

# Quick start (all-in-one)
make quick              # Setup + build + run TUI
make all                # Setup + build + run everything in tmux

# Status & health
make status             # Check all services
make test               # Run integration tests
```

## 📋 Makefile Targets (All Available)

### Setup & Installation
- `make setup` - Install Ollama, pull model, setup database
- `make setup-ollama` - Just setup Ollama + model
- `make setup-db` - Just setup SpacetimeDB

### Build & Run
- `make build` - Build release version
- `make build-dev` - Build debug (faster)
- `make run` - Run TUI (alias for `make tui`)
- `make tui` - Launch Mission Control TUI
- `make irc` - Launch LLM-enhanced IRC bot
- `make irc-original` - Launch original template-based bot
- `make dev` - Run everything in tmux (DB + IRC + TUI)
- `make all` - Setup + build + run everything

### Testing & Debugging
- `make test` - Run integration test suite
- `make test-llm` - Quick LLM connection test
- `make test-db` - Quick database test
- `make check` - Fast validation (cargo check)
- `make fmt` - Format code
- `make clippy` - Run linter

### Monitoring
- `make status` - Show status of all services
- `make logs` - View logs (reminder to use tmux)

### Cleanup
- `make clean` - Clean build artifacts
- `make clean-state` - Clean runtime state files
- `make clean-all` - Clean everything
- `make stop` - Stop all SAGE services

### Documentation
- `make docs` - Open documentation
- `make readme` - Quick start preview
- `make help` - Show all commands (default)

### Quick Commands
- `make quick` - Setup + build + run TUI
- `make demo` - Run demo (just TUI)
- `make chat` - Alias for `make irc`
- `make hot-reload` - Test hot-reload system
- `make watch` - Watch mode (rebuild on changes)

### Information
- `make version` - Version info
- `make info` - Full system info (status + version)

## 🎯 Recommended Workflows

### First Time User
```bash
make setup      # Install everything
make quick      # Build and run TUI
# Press [N] to start training
# Press [Tab] to see screens
```

### Daily Development
```bash
make dev        # Start tmux session with all services
# Tmux session will have 3 panes:
#   - SpacetimeDB
#   - IRC bot
#   - Mission Control TUI
```

### IRC Chat Testing
```bash
# Terminal 1: Start IRC bot
make irc

# Terminal 2: Start TUI to monitor
make tui

# Connect IRC client to:
#   Server: irc.libera.chat:6667
#   Channel: #sage-ai
```

### Quick Check
```bash
make status     # See what's running
make test       # Run all tests
```

### Clean Restart
```bash
make stop           # Stop everything
make clean-state    # Reset SAGE memory
make quick          # Fresh start
```

## 🔧 Tmux Controls (when using `make dev`)

```
Ctrl+B then D    - Detach from session
Ctrl+B then ←→   - Switch panes
Ctrl+B then [    - Scroll mode (Q to exit)
Ctrl+B then "    - Split horizontal
Ctrl+B then %    - Split vertical

tmux attach -t sage    - Reattach to session
make stop              - Kill session
```

## ⌨️ TUI Keyboard Shortcuts

```
[Tab]      - Cycle screens (Mission Control → Dashboard → Database → Chat)
[N]        - Start baseline training (35 concepts)
[Space]    - Pause/Resume training
[M]        - Toggle mission control screen
[Q]        - Quit

# Screens:
Mission Control - Main dashboard with neural CT scan + IRC feed
Dashboard       - Original unified dashboard
Database        - Historical metrics and analytics
Chat            - Chat interface (experimental)
```

## 🤖 IRC Commands

```
SAGE, <message>        - Natural conversation
!personality           - Show SAGE's personality
!likes                 - What SAGE resonates with
!dislikes              - What SAGE struggles with
!memory or !context    - Current emotional state
!help                  - List commands
```

## 📊 Understanding the Neural CT Scan

### Colors (Understanding Level)
- 🟢 **Green** - SAGE has mastered this (loss < 0.05)
- 🟡 **Yellow** - SAGE is learning (loss 0.05-0.15)
- 🔴 **Red** - SAGE is exploring (loss > 0.15)

### Character Intensity
- `█` - Maximum activity (> 0.8)
- `▓` - High activity (0.6-0.8)
- `▒` - Medium activity (0.4-0.6)
- `░` - Low activity (0.2-0.4)
- `·` - Trace activity (0.05-0.2)
- ` ` - No activity (< 0.05)

### Visual Effects
- **Pulsing** - Active learning (300ms cycle)
- **Sparkles** - High-energy cells (150ms cycle)
- **Status**: ⚡ SCANNING = training active

## 🔍 Troubleshooting Quick Fixes

### IRC bot not responding
```bash
brew services restart ollama
make irc
```

### TUI shows black screen
```bash
# Press [N] to start training
# Or send IRC message to trigger learning
```

### Build errors
```bash
make clean
make build
```

### SpacetimeDB errors
```bash
pkill spacetime
make setup-db
```

### Everything broken
```bash
make stop
make clean-all
make setup
make quick
```

## 📂 Important Files

### Configuration
- `Cargo.toml` - Dependencies
- `Makefile` - Build automation

### Documentation
- `SAGE_LLM_QUICKSTART.md` - Full user guide
- `IMPLEMENTATION_SUMMARY.md` - Technical docs
- `QUICK_REFERENCE.md` - This file
- `HOT_RELOAD.md` - Hot-reload system docs

### Source Code
- `src/llm_client.rs` - LLM integration
- `src/sage_experience.rs` - SAGE consciousness
- `src/tui/screens/mission_control.rs` - Main dashboard
- `examples/sage_irc_llm_bot.rs` - IRC bot

### Runtime State
- `sage_preferences.json` - SAGE's likes/dislikes
- `sage_associations.json` - Concept connections
- `sage_curiosity.json` - Curiosity tracking
- `sage_positive_knowledge.json` - Trained patterns
- `/tmp/sage_state.json` - TUI state

## 🎓 Learning Path

1. **Day 1**: `make quick` → Watch training → Explore TUI
2. **Day 2**: `make irc` → Connect IRC → Chat with SAGE
3. **Day 3**: `make dev` → Multi-user chat → Watch patterns evolve
4. **Day 4**: Experiment with pattern capacity (how many concepts?)
5. **Day 5**: Modify baseline_concepts, test custom training

## 💡 Pro Tips

- Use `make dev` for full experience (tmux session with all panes)
- Let baseline training complete (35 concepts, ~10 min) before chatting
- Watch neural colors shift as SAGE learns (red → yellow → green)
- Frequently mentioned concepts get stronger (visible in TUI)
- Use IRC commands (!personality, !memory) to understand SAGE's state
- Check `make status` if anything seems wrong
- The neural field is synchronized with conversation - watch it pulse when you chat!

## 🆘 Getting Help

```bash
make help       # List all commands
make status     # Check service health
make test       # Run diagnostics
```

Read full docs: `SAGE_LLM_QUICKSTART.md`

---

**Quick start**: `make quick` then press `[N]` to train, `[Tab]` to explore screens!
