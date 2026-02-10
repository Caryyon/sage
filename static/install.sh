#!/usr/bin/env bash
set -euo pipefail

REPO="Caryyon/sage"
INSTALL_DIR="$HOME/.sage/bin"

# Detect OS
case "$(uname -s)" in
  Linux*)  OS="linux" ;;
  Darwin*) OS="darwin" ;;
  *) echo "Unsupported OS: $(uname -s)"; exit 1 ;;
esac

# Detect arch
case "$(uname -m)" in
  x86_64)          ARCH="x86_64" ;;
  arm64|aarch64)   ARCH="arm64" ;;
  *) echo "Unsupported architecture: $(uname -m)"; exit 1 ;;
esac

echo "🔍 Detected: ${OS} ${ARCH}"

# GitHub token for private repo (optional)
GH_TOKEN="${SAGE_GH_TOKEN:-${GITHUB_TOKEN:-}}"
AUTH_HEADER=""
if [ -n "$GH_TOKEN" ]; then
  AUTH_HEADER="Authorization: token ${GH_TOKEN}"
  echo "🔑 Using GitHub token for authentication"
else
  echo "⚠️  No SAGE_GH_TOKEN or GITHUB_TOKEN set — this only works for public repos"
fi

curl_auth() {
  if [ -n "$AUTH_HEADER" ]; then
    curl -fSL -H "$AUTH_HEADER" "$@"
  else
    curl -fSL "$@"
  fi
}

# Get latest version
echo "📡 Fetching latest release..."
VERSION=$(curl_auth -s "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name"' | sed -E 's/.*"([^"]+)".*/\1/')
if [ -z "$VERSION" ]; then
  echo "❌ Could not determine latest version (is the repo public or SAGE_GH_TOKEN set?)"; exit 1
fi
echo "📦 Version: ${VERSION}"

# Download binary
BINARY="sage-${OS}-${ARCH}"
URL="https://github.com/${REPO}/releases/download/${VERSION}/${BINARY}"
TMPDIR=$(mktemp -d)
trap 'rm -rf "$TMPDIR"' EXIT

echo "⬇️  Downloading ${BINARY}..."
curl_auth "$URL" -o "${TMPDIR}/sage"

# Install
mkdir -p "$INSTALL_DIR"
echo "📂 Installing to ${INSTALL_DIR}..."
cp "${TMPDIR}/sage" "$INSTALL_DIR/sage"
chmod +x "$INSTALL_DIR/sage"

# PATH setup
if [[ ":$PATH:" != *":${INSTALL_DIR}:"* ]]; then
  SHELL_NAME=$(basename "$SHELL")
  case "$SHELL_NAME" in
    zsh)  RC="$HOME/.zshrc" ;;
    bash) RC="$HOME/.bashrc" ;;
    *)    RC="$HOME/.profile" ;;
  esac
  echo "export PATH=\"${INSTALL_DIR}:\$PATH\"" >> "$RC"
  export PATH="${INSTALL_DIR}:$PATH"
  echo "✅ Added ${INSTALL_DIR} to PATH in ${RC}"
fi

# Create default config with bootstrap node
SAGE_HOME="${SAGE_HOME:-$HOME/.sage}"
CONFIG_FILE="${SAGE_HOME}/config.toml"
if [ ! -f "$CONFIG_FILE" ]; then
  mkdir -p "$SAGE_HOME"
  cat > "$CONFIG_FILE" <<'TOML'
[network]
bootstrap = ["bootstrap.whatssage.ai:4001"]
TOML
  echo "📝 Created default config at ${CONFIG_FILE}"
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  🌿 SAGE installed successfully!"
echo "  Shared Adaptive Growing Experience"
echo "  The People's AI — Free. Local. Gets smarter together."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "  Binary:  ${INSTALL_DIR}/sage"
echo "  Version: ${VERSION}"
echo ""
echo "  Get started:"
echo "    sage --help"
echo ""
echo "  Restart your shell or run:"
echo "    export PATH=\"${INSTALL_DIR}:\$PATH\""
echo ""
