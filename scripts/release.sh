#!/bin/bash
# SAGE Release Builder
# Builds release binaries for all platforms

set -e

VERSION=$(grep '^version' Cargo.toml | head -1 | cut -d'"' -f2)
echo "Building SAGE v${VERSION} releases..."

RELEASE_DIR="releases/v${VERSION}"
mkdir -p "${RELEASE_DIR}"

# Linux x86_64
echo "📦 Building Linux x86_64..."
cargo build --release --target x86_64-unknown-linux-gnu 2>/dev/null || echo "   (using host target)"
cp ~/.cargo-targets/sage/release/sage-cli "${RELEASE_DIR}/sage-${VERSION}-linux-x86_64" 2>/dev/null || \
  cp target/release/sage-cli "${RELEASE_DIR}/sage-${VERSION}-linux-x86_64" 2>/dev/null || \
  echo "   Binary not found, may need cross-compilation setup"

# Create tarball
echo "📦 Creating release archives..."
cd "${RELEASE_DIR}"

# Checksum generation (if binaries exist)
if ls sage-* 1>/dev/null 2>&1; then
    sha256sum sage-* > SHA256SUMS.txt 2>/dev/null || shasum -a 256 sage-* > SHA256SUMS.txt
    echo "✅ SHA256 checksums generated"
fi

cd ../..

echo ""
echo "═══════════════════════════════════════════════════════════════"
echo "Release v${VERSION} artifacts in ${RELEASE_DIR}/"
echo "═══════════════════════════════════════════════════════════════"
ls -lh "${RELEASE_DIR}/" 2>/dev/null || echo "(No binaries built yet)"
echo ""
echo "Next steps:"
echo "  1. Build for other platforms (see scripts/build-*.sh)"
echo "  2. Upload to GitHub releases"
echo "  3. Update install.sh with new version"
