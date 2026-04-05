#!/usr/bin/env bash
set -euo pipefail

# Package a release tarball for Homebrew distribution.
# Usage: scripts/release.sh [version]
# Default version: 0.1.0

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_DIR="$(dirname "$SCRIPT_DIR")"
VERSION="${1:-0.1.0}"
ARCH="aarch64-apple-darwin"
TARBALL_NAME="termsurf-${VERSION}-${ARCH}.tar.gz"
STAGING_DIR="$REPO_DIR/dist/release"
CHROMIUM_OUT="$REPO_DIR/chromium/src/out/Default"

echo "==> Packaging TermSurf v${VERSION} for ${ARCH}..."

# Check release builds exist
for f in \
  "$REPO_DIR/webtui/target/release/web" \
  "$REPO_DIR/wezboard/target/release/wezboard" \
  "$REPO_DIR/roamium/target/release/roamium"; do
  if [ ! -f "$f" ]; then
    echo "Error: Release build not found: $f"
    echo "Run: scripts/build.sh all --release"
    exit 1
  fi
done

# Clean and create staging directory
rm -rf "$STAGING_DIR"
mkdir -p "$STAGING_DIR/roamium"

# Copy binaries
echo "==> Copying binaries..."
cp "$REPO_DIR/webtui/target/release/web" "$STAGING_DIR/"
cp "$REPO_DIR/wezboard/target/release/wezboard" "$STAGING_DIR/"
cp "$REPO_DIR/roamium/target/release/roamium" "$STAGING_DIR/roamium/"

# Copy Chromium dylibs and resources
echo "==> Copying Chromium dylibs and resources..."
cp "$CHROMIUM_OUT"/*.dylib "$STAGING_DIR/roamium/"
cp "$CHROMIUM_OUT"/*.pak "$STAGING_DIR/roamium/"
cp "$CHROMIUM_OUT/icudtl.dat" "$STAGING_DIR/roamium/"
cp "$CHROMIUM_OUT"/v8_context_snapshot*.bin "$STAGING_DIR/roamium/"

# Create tarball
echo "==> Creating tarball..."
cd "$STAGING_DIR"
tar czf "$REPO_DIR/dist/$TARBALL_NAME" .

# Compute SHA256
SHA=$(shasum -a 256 "$REPO_DIR/dist/$TARBALL_NAME" | awk '{print $1}')
echo ""
echo "==> Release package: dist/$TARBALL_NAME"
echo "==> SHA256: $SHA"
echo ""
echo "Next steps:"
echo "  1. gh release create v${VERSION} dist/${TARBALL_NAME} --title 'v${VERSION}'"
echo "  2. Update homebrew/Formula/termsurf.rb with SHA256: ${SHA}"
echo "  3. cd homebrew && git add -A && git commit -m 'v${VERSION}' && git push"
