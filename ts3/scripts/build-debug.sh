#!/bin/bash
# Build TermSurf 3.0 in Debug mode
#
# Usage:
#   ./scripts/build-debug.sh [--clean] [--open] [--open-web]
#
# Flags:
#   --clean     Clear build caches and do a fresh build
#   --open      Run wezterm-gui after building
#   --open-web  Run web CLI after building (for testing CEF)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_DIR="$(dirname "$SCRIPT_DIR")"
CEF_RS_DIR="$(dirname "$REPO_DIR")/cef-rs"

# Parse flags
CLEAN=false
OPEN=false
OPEN_WEB=false
for arg in "$@"; do
    case $arg in
        --clean) CLEAN=true ;;
        --open) OPEN=true ;;
        --open-web) OPEN_WEB=true ;;
    esac
done

# Clean if requested
if [ "$CLEAN" = true ]; then
    echo "=== Cleaning build caches ==="
    rm -rf "$REPO_DIR/target/debug"
    echo "Cleared target/debug"
fi

echo "=== Building TermSurf 3.0 (Debug) ==="

# 1. Build cef-rs helpers first (needed for bundle)
echo "Building CEF helpers..."
cd "$CEF_RS_DIR"
cargo build -p cef-osr
cargo run -p cef --bin bundle-cef-app -- cef-osr -o cef-osr.app

# Verify cef-osr.app was created
if [ ! -d "$CEF_RS_DIR/cef-osr.app" ]; then
    echo "ERROR: cef-osr.app not found at $CEF_RS_DIR/cef-osr.app"
    exit 1
fi

# 2. Build ts3 workspace
echo "Building workspace..."
cd "$REPO_DIR"
cargo build

# 3. Create app bundle
APP_BUNDLE="$REPO_DIR/target/debug/wezterm-gui.app"
echo "Creating app bundle at $APP_BUNDLE..."

rm -rf "$APP_BUNDLE"
mkdir -p "$APP_BUNDLE/Contents/MacOS"
mkdir -p "$APP_BUNDLE/Contents/Frameworks"
mkdir -p "$APP_BUNDLE/Contents/Resources"

# 4. Copy executables
cp "$REPO_DIR/target/debug/wezterm-gui" "$APP_BUNDLE/Contents/MacOS/"
cp "$REPO_DIR/target/debug/wezterm" "$APP_BUNDLE/Contents/MacOS/"
cp "$REPO_DIR/target/debug/web" "$APP_BUNDLE/Contents/MacOS/"

# 5. Copy CEF framework
echo "Copying CEF framework..."
cp -R "$CEF_RS_DIR/cef-osr.app/Contents/Frameworks/Chromium Embedded Framework.framework" \
      "$APP_BUNDLE/Contents/Frameworks/"

# 6. Copy and rename CEF helper apps
echo "Copying CEF helper apps..."
for suffix in "" " (GPU)" " (Renderer)" " (Plugin)" " (Alerts)"; do
    src="$CEF_RS_DIR/cef-osr.app/Contents/Frameworks/cef-osr Helper${suffix}.app"
    dst="$APP_BUNDLE/Contents/Frameworks/WezTerm Helper${suffix}.app"
    if [ -d "$src" ]; then
        cp -R "$src" "$dst"
        # Update Info.plist to rename from cef-osr to WezTerm
        sed -i '' 's/cef-osr/WezTerm/g' "$dst/Contents/Info.plist"
        # Rename the binary inside the helper app
        mv "$dst/Contents/MacOS/cef-osr Helper${suffix}" "$dst/Contents/MacOS/WezTerm Helper${suffix}"
    else
        echo "WARNING: Helper not found: $src"
    fi
done

# 7. Create Info.plist
cat > "$APP_BUNDLE/Contents/Info.plist" << 'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>wezterm-gui</string>
    <key>CFBundleIdentifier</key>
    <string>org.wezfurlong.wezterm</string>
    <key>CFBundleName</key>
    <string>WezTerm</string>
    <key>CFBundleVersion</key>
    <string>1.0</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>LSEnvironment</key>
    <dict>
        <key>MallocNanoZone</key>
        <string>0</string>
    </dict>
</dict>
</plist>
PLIST

# 8. Sign the bundle
echo "Signing bundle..."
codesign --sign - --force --deep "$APP_BUNDLE"

echo ""
echo "=== Debug Build Complete ==="
echo "App bundle: $APP_BUNDLE"
echo "  Contents/MacOS/wezterm-gui  (terminal)"
echo "  Contents/MacOS/wezterm      (CLI)"
echo "  Contents/MacOS/web          (web coordinator)"
echo ""

# Open if requested
if [ "$OPEN" = true ]; then
    echo "Running wezterm-gui..."
    "$APP_BUNDLE/Contents/MacOS/wezterm-gui"
fi

if [ "$OPEN_WEB" = true ]; then
    echo "Running web CLI..."
    "$APP_BUNDLE/Contents/MacOS/web"
fi
