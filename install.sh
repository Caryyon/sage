#!/bin/bash
# SAGE Installer
# One-liner: curl -fsSL https://whatssage.ai/install.sh | bash

set -e

REPO="Caryyon/sage"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"
SAGE_HOME="${SAGE_HOME:-$HOME/.sage}"

# Detect OS and architecture
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

case "$ARCH" in
  x86_64) ARCH="x86_64" ;;
  aarch64|arm64) ARCH="aarch64" ;;
  *) echo "Unsupported architecture: $ARCH"; exit 1 ;;
esac

case "$OS" in
  linux) PLATFORM="unknown-linux-gnu" ;;
  darwin) PLATFORM="apple-darwin" ;;
  *) echo "Unsupported OS: $OS"; exit 1 ;;
esac

echo "🔮 Installing SAGE..."
echo "   OS: $OS"
echo "   Arch: $ARCH"

# Get latest release
LATEST=$(curl -s "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')
if [ -z "$LATEST" ]; then
  echo "❌ Could not determine latest release"
  exit 1
fi

echo "   Version: $LATEST"

# Download
FILENAME="sage-${LATEST}-${ARCH}-${PLATFORM}.tar.gz"
URL="https://github.com/$REPO/releases/download/$LATEST/$FILENAME"

echo "   Downloading from GitHub..."
TMPDIR=$(mktemp -d)
curl -fsSL "$URL" -o "$TMPDIR/$FILENAME" || {
  echo "❌ Download failed. Build from source instead:"
  echo "   git clone https://github.com/$REPO.git && cd sage && cargo build --release"
  exit 1
}

# Extract
tar -xzf "$TMPDIR/$FILENAME" -C "$TMPDIR"

# Install binaries
echo "   Installing to $INSTALL_DIR (may need sudo)..."
if [ -w "$INSTALL_DIR" ]; then
  cp "$TMPDIR"/sage* "$INSTALL_DIR/" 2>/dev/null || true
else
  sudo cp "$TMPDIR"/sage* "$INSTALL_DIR/" 2>/dev/null || {
    echo "⚠️  Could not install to $INSTALL_DIR"
    echo "   Install manually: cp $TMPDIR/sage* ~/bin/"
  }
fi

# Create SAGE_HOME
mkdir -p "$SAGE_HOME"

# Cleanup
rm -rf "$TMPDIR"

echo ""
echo "✅ SAGE $LATEST installed!"
echo ""
echo "Quick start:"
echo "   sage chat          # Start chatting"
echo "   sage node start    # Join the mesh"
echo "   sage status        # Check health"
echo ""
echo "Dashboard: https://whatssage.ai/network"
echo "Discord:   https://discord.gg/U999zZUuUV"
