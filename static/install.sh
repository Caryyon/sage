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

# Get latest version
echo "📡 Fetching latest release..."
VERSION=$(curl -sL "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name"' | sed -E 's/.*"([^"]+)".*/\1/')
if [ -z "$VERSION" ]; then
  echo "❌ Could not determine latest version"; exit 1
fi
echo "📦 Version: ${VERSION}"

# Download
ARCHIVE="sage-${VERSION}-${OS}-${ARCH}.tar.gz"
URL="https://github.com/${REPO}/releases/download/${VERSION}/${ARCHIVE}"
TMPDIR=$(mktemp -d)
trap 'rm -rf "$TMPDIR"' EXIT

echo "⬇️  Downloading ${URL}..."
curl -fSL "$URL" -o "${TMPDIR}/${ARCHIVE}"

# Install
mkdir -p "$INSTALL_DIR"
echo "📂 Installing to ${INSTALL_DIR}..."
tar -xzf "${TMPDIR}/${ARCHIVE}" -C "$INSTALL_DIR"
chmod +x "$INSTALL_DIR"/*

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
bootstrap = ["bootstrap.sage.lattice.black:4001"]
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
echo "  Binaries: ${INSTALL_DIR}"
echo "  Version:  ${VERSION}"
echo ""
echo "  Next steps:"
echo "    sage-node       Start a SAGE node"
echo "    sage_chat        Interactive chat"
echo "    sage-api         Start the API server"
echo "    sage-bootstrap   Bootstrap your node"
echo ""
echo "  Restart your shell or run:"
echo "    export PATH=\"${INSTALL_DIR}:\$PATH\""
echo ""
