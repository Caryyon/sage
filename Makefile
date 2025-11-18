# SAGE 2.0 - Neural Cellular Automata with LLM Integration
# Makefile for easy development and deployment

.PHONY: help setup build run irc tui test clean logs status stop all dev

# Default target
.DEFAULT_GOAL := help

# Colors for output
CYAN := \033[0;36m
GREEN := \033[0;32m
YELLOW := \033[1;33m
RED := \033[0;31m
NC := \033[0m # No Color

##@ General

help: ## Display this help message
	@echo "╔════════════════════════════════════════════════════════════╗"
	@echo "║              SAGE 2.0 - Development Commands               ║"
	@echo "╚════════════════════════════════════════════════════════════╝"
	@echo ""
	@awk 'BEGIN {FS = ":.*##"; printf "Usage: make $(CYAN)<target>$(NC)\n"} /^[a-zA-Z_-]+:.*?##/ { printf "  $(CYAN)%-15s$(NC) %s\n", $$1, $$2 } /^##@/ { printf "\n$(YELLOW)%s$(NC)\n", substr($$0, 5) } ' $(MAKEFILE_LIST)
	@echo ""

##@ Setup & Installation

setup: ## Install all dependencies (Ollama, model, SpacetimeDB)
	@echo "$(CYAN)Installing SAGE dependencies...$(NC)"
	@if ! command -v ollama &> /dev/null; then \
		echo "$(YELLOW)Installing Ollama...$(NC)"; \
		brew install ollama; \
	else \
		echo "$(GREEN)✓ Ollama already installed$(NC)"; \
	fi
	@if ! command -v spacetime &> /dev/null; then \
		echo "$(YELLOW)⚠ SpacetimeDB not found. Install from: https://spacetimedb.com$(NC)"; \
	else \
		echo "$(GREEN)✓ SpacetimeDB already installed$(NC)"; \
	fi
	@$(MAKE) setup-ollama
	@$(MAKE) setup-db
	@echo "$(GREEN)✓ Setup complete!$(NC)"

setup-ollama: ## Start Ollama and pull LLM model
	@echo "$(CYAN)Setting up Ollama...$(NC)"
	@brew services start ollama 2>/dev/null || true
	@sleep 2
	@if ! ollama list | grep -q "llama3.2:3b"; then \
		echo "$(YELLOW)Pulling llama3.2:3b model (this may take a few minutes)...$(NC)"; \
		ollama pull llama3.2:3b; \
	else \
		echo "$(GREEN)✓ Model llama3.2:3b already downloaded$(NC)"; \
	fi

setup-db: ## Initialize and publish SpacetimeDB schema
	@echo "$(CYAN)Setting up SpacetimeDB...$(NC)"
	@if command -v spacetime &> /dev/null; then \
		spacetime start --listen-addr 127.0.0.1:4000 2>/dev/null & \
		sleep 3; \
		cd sage-db && spacetime publish sage-db --project-path . && cd ..; \
		echo "$(GREEN)✓ Database published$(NC)"; \
	else \
		echo "$(YELLOW)⚠ SpacetimeDB not installed (optional)$(NC)"; \
	fi

publish-db: ## Republish SpacetimeDB schema (after changes)
	@echo "$(CYAN)Publishing SpacetimeDB schema...$(NC)"
	@cd sage-db && spacetime publish sage-db --project-path . && cd ..
	@echo "$(GREEN)✓ Database schema published$(NC)"

reset-db: ## Delete and recreate database (WARNING: loses all data!)
	@echo "$(RED)⚠️  WARNING: This will delete all SAGE data!$(NC)"
	@read -p "Are you sure? [y/N] " -n 1 -r; \
	echo; \
	if [[ $$REPLY =~ ^[Yy]$$ ]]; then \
		echo "$(CYAN)Deleting sage-db...$(NC)"; \
		spacetime delete sage-db 2>/dev/null || true; \
		sleep 2; \
		echo "$(CYAN)Creating fresh database...$(NC)"; \
		cd sage-db && spacetime publish sage-db --project-path .; \
		echo "$(GREEN)✓ Database reset complete$(NC)"; \
	else \
		echo "$(YELLOW)Cancelled$(NC)"; \
	fi

##@ Build & Run

build: ## Build SAGE in release mode
	@echo "$(CYAN)Building SAGE...$(NC)"
	@cargo build --release
	@echo "$(GREEN)✓ Build complete$(NC)"

build-dev: ## Build SAGE in debug mode (faster compilation)
	@echo "$(CYAN)Building SAGE (debug)...$(NC)"
	@cargo build
	@echo "$(GREEN)✓ Debug build complete$(NC)"

run: tui ## Alias for 'make tui'

tui: build ## Launch SAGE Mission Control TUI
	@echo "$(CYAN)Launching SAGE Mission Control...$(NC)"
	@echo "$(YELLOW)Tip: Press [Tab] to cycle screens, [N] to train, [Q] to quit$(NC)"
	@cargo run --release

sage: build setup-ollama setup-db ## Start SAGE with all core features (autonomous IRC + vision)
	@echo "$(CYAN)╔════════════════════════════════════════════════════════════╗$(NC)"
	@echo "$(CYAN)║              Starting SAGE - Full Consciousness            ║$(NC)"
	@echo "$(CYAN)╚════════════════════════════════════════════════════════════╝$(NC)"
	@echo ""
	@echo "$(GREEN)Features enabled:$(NC)"
	@echo "  ✓ IRC Bot (#sage-ai on Libera.Chat)"
	@echo "  ✓ Autonomous Consciousness (dreams + curiosity)"
	@echo "  ✓ Vision System (camera + visual memory)"
	@echo "  ✓ LLM Integration (Ollama)"
	@echo "  ✓ Conversation Summarization (NEW!)"
	@echo ""
	@echo "$(YELLOW)Press Ctrl+C to stop$(NC)"
	@cargo run --release --example sage_irc_autonomous

irc: sage ## Alias for 'make sage'

discord: build setup-ollama setup-db ## Start SAGE Discord bot
	@echo "$(CYAN)Starting SAGE Discord Bot...$(NC)"
	@if [ ! -f .env.local ]; then \
		echo "$(RED)Error: .env.local file not found$(NC)"; \
		echo "$(YELLOW)Create .env.local with your Discord token:$(NC)"; \
		echo "  echo 'DISCORD_TOKEN=your-token-here' > .env.local"; \
		echo "$(YELLOW)See .env.local.example for template$(NC)"; \
		exit 1; \
	fi
	@cargo run --release --example sage_discord_autonomous

vision: build ## Start SAGE Vision mode (real-time camera perception)
	@echo "$(CYAN)Starting SAGE Vision Mode...$(NC)"
	@echo "$(YELLOW)Features:$(NC)"
	@echo "  ✓ Real-time camera capture"
	@echo "  ✓ Visual feature extraction"
	@echo "  ✓ Concept generation (brightness, color, edges)"
	@echo "  ✓ Visual memory storage"
	@echo "  ✓ NCA grid conversion"
	@echo ""
	@echo "$(YELLOW)Press Ctrl+C to stop$(NC)"
	@cargo run --release --bin sage_vision

##@ Development

dev: ## Run SAGE + TUI in parallel using tmux
	@echo "$(CYAN)Starting SAGE development environment...$(NC)"
	@if ! command -v tmux &> /dev/null; then \
		echo "$(RED)Error: tmux not installed$(NC)"; \
		echo "Install with: brew install tmux"; \
		exit 1; \
	fi
	@$(MAKE) setup-ollama
	@tmux new-session -d -s sage "make setup-db; read"
	@tmux split-window -h -t sage "sleep 5; make sage"
	@tmux split-window -v -t sage "sleep 8; make tui"
	@tmux select-layout -t sage tiled
	@echo "$(GREEN)✓ SAGE development environment started in tmux$(NC)"
	@echo "$(YELLOW)Layout: DB | IRC Bot | TUI$(NC)"
	@echo "$(YELLOW)Attach with: tmux attach -t sage$(NC)"
	@echo "$(YELLOW)Detach with: Ctrl+B then D$(NC)"
	@echo "$(YELLOW)Kill with: make stop$(NC)"
	@tmux attach -t sage

all: setup build ## Setup, build, and run everything
	@$(MAKE) dev

##@ Testing & Debugging

test: ## Run integration tests
	@./test_llm_integration.sh

test-llm: ## Quick test of LLM connection
	@echo "$(CYAN)Testing LLM connection...$(NC)"
	@curl -s -X POST http://localhost:11434/api/generate \
		-d '{"model":"llama3.2:3b","prompt":"Say hello in 3 words","stream":false}' \
		| grep -o '"response":"[^"]*"' \
		&& echo "$(GREEN)✓ LLM responding$(NC)" \
		|| echo "$(RED)✗ LLM not responding$(NC)"

test-db: ## Test SpacetimeDB connection
	@echo "$(CYAN)Testing SpacetimeDB connection...$(NC)"
	@if spacetime server list 2>/dev/null | grep -q "sage-db"; then \
		echo "$(GREEN)✓ Database connected$(NC)"; \
	else \
		echo "$(RED)✗ Database not running$(NC)"; \
	fi

check: ## Run cargo check (fast validation)
	@cargo check

fmt: ## Format code with rustfmt
	@cargo fmt

clippy: ## Run clippy linter
	@cargo clippy -- -D warnings

##@ Monitoring

status: ## Show status of all SAGE services
	@echo "$(CYAN)SAGE Service Status:$(NC)"
	@echo ""
	@printf "$(YELLOW)Ollama:$(NC)         "
	@if brew services list | grep ollama | grep -q started; then \
		echo "$(GREEN)✓ Running$(NC)"; \
	else \
		echo "$(RED)✗ Stopped$(NC)"; \
	fi
	@printf "$(YELLOW)LLM Model:$(NC)      "
	@if ollama list 2>/dev/null | grep -q llama3.2:3b; then \
		echo "$(GREEN)✓ Downloaded$(NC)"; \
	else \
		echo "$(RED)✗ Not found$(NC)"; \
	fi
	@printf "$(YELLOW)SpacetimeDB:$(NC)    "
	@if pgrep -x spacetime > /dev/null; then \
		echo "$(GREEN)✓ Running$(NC)"; \
	else \
		echo "$(RED)✗ Stopped$(NC)"; \
	fi
	@printf "$(YELLOW)IRC Bot:$(NC)        "
	@if pgrep -f sage_irc_autonomous > /dev/null; then \
		echo "$(GREEN)✓ Running$(NC)"; \
	else \
		echo "$(RED)✗ Stopped$(NC)"; \
	fi
	@printf "$(YELLOW)Discord Bot:$(NC)    "
	@if pgrep -f sage_discord_autonomous > /dev/null; then \
		echo "$(GREEN)✓ Running$(NC)"; \
	else \
		echo "$(RED)✗ Stopped$(NC)"; \
	fi
	@printf "$(YELLOW)SAGE TUI:$(NC)       "
	@if pgrep -f "cargo run --release" > /dev/null || pgrep -f "target/release/sage" > /dev/null; then \
		echo "$(GREEN)✓ Running$(NC)"; \
	else \
		echo "$(RED)✗ Stopped$(NC)"; \
	fi
	@echo ""

logs: ## Show SAGE logs (if using systemd/launchd)
	@echo "$(YELLOW)Note: SAGE currently logs to stdout. Run in tmux to capture logs.$(NC)"
	@echo "Use: make dev (to run in tmux)"

##@ Cleanup

clean: ## Clean build artifacts
	@echo "$(CYAN)Cleaning build artifacts...$(NC)"
	@cargo clean
	@echo "$(GREEN)✓ Clean complete$(NC)"

clean-state: ## Clean SAGE runtime state (preferences, knowledge, etc.)
	@echo "$(CYAN)Cleaning SAGE state files...$(NC)"
	@rm -f sage_preferences.json sage_associations.json sage_curiosity.json
	@rm -f sage_positive_knowledge.json /tmp/sage_state.json
	@echo "$(GREEN)✓ State cleaned$(NC)"

clean-all: clean clean-state ## Clean everything (build + state)

stop: ## Stop all SAGE services
	@echo "$(CYAN)Stopping SAGE services...$(NC)"
	@pkill -f sage_irc_autonomous 2>/dev/null || true
	@pkill -f sage_discord_autonomous 2>/dev/null || true
	@pkill -f sage_irc_llm_bot 2>/dev/null || true
	@pkill -f "cargo run --release" 2>/dev/null || true
	@pkill -f "target/release/sage" 2>/dev/null || true
	@tmux kill-session -t sage 2>/dev/null || true
	@echo "$(GREEN)✓ Services stopped$(NC)"
	@echo "$(YELLOW)Note: Ollama and SpacetimeDB left running. Stop manually if needed:$(NC)"
	@echo "  brew services stop ollama"
	@echo "  pkill spacetime"

##@ Documentation

docs: ## Open documentation in browser
	@echo "$(CYAN)Opening SAGE documentation...$(NC)"
	@if command -v open &> /dev/null; then \
		open SAGE_LLM_QUICKSTART.md || true; \
		open IMPLEMENTATION_SUMMARY.md || true; \
	else \
		echo "$(YELLOW)Documentation files:$(NC)"; \
		echo "  - SAGE_LLM_QUICKSTART.md"; \
		echo "  - IMPLEMENTATION_SUMMARY.md"; \
		echo "  - SAGE_ARCHITECTURE.html"; \
	fi

readme: ## Display quick start info
	@cat SAGE_LLM_QUICKSTART.md | head -50

##@ Quick Commands

# Convenience aliases
quick: setup build tui ## Quick setup and launch (setup + build + run)

demo: ## Run a quick demo (train + visualize)
	@echo "$(CYAN)Running SAGE demo...$(NC)"
	@echo "$(YELLOW)1. Starting TUI$(NC)"
	@echo "$(YELLOW)2. Press [N] to start baseline training$(NC)"
	@echo "$(YELLOW)3. Press [Tab] to see different screens$(NC)"
	@echo "$(YELLOW)4. Press [Q] to quit$(NC)"
	@$(MAKE) tui

hot-reload: ## Test hot-reload system
	@echo "$(CYAN)Testing hot-reload...$(NC)"
	@cargo run --release --example test_hot_reload

chat: irc ## Alias for 'make irc'

watch: ## Watch mode - rebuild on file changes
	@cargo watch -x "build --release"

##@ Information

version: ## Show version information
	@echo "$(CYAN)SAGE Version Information:$(NC)"
	@echo ""
	@cargo --version
	@rustc --version
	@echo -n "Ollama: "; ollama --version 2>/dev/null || echo "not installed"
	@echo -n "SpacetimeDB: "; spacetime version 2>/dev/null || echo "not installed"
	@echo ""
	@echo "$(YELLOW)SAGE 2.0$(NC) - Neural Cellular Automata with LLM Integration"

info: status version ## Show all system information
