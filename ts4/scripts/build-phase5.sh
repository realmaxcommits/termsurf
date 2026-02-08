#!/bin/bash
# Build and run Phase 5: XPC between Swift window and Rust terminal
#
# Usage:
#   ./scripts/build-phase5.sh [--clean]

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TS4_DIR="$(dirname "$SCRIPT_DIR")"

# Parse flags
CLEAN=false
for arg in "$@"; do
    case $arg in
        --clean) CLEAN=true ;;
    esac
done

# Clean stale XPC service registration
echo "=== Cleaning stale XPC service ==="
launchctl bootout "gui/$(id -u)/com.termsurf.ts4.terminal" 2>/dev/null || true
rm -f /tmp/com.termsurf.ts4.terminal.plist

if [ "$CLEAN" = true ]; then
    echo "=== Cleaning build caches ==="
    rm -rf "$TS4_DIR/target/debug"
    rm -rf "$TS4_DIR/termsurf-window/.build"
fi

# 1. Build Rust terminal
echo ""
echo "=== Building Rust terminal ==="
cd "$TS4_DIR"
cargo build -p termsurf-terminal

TERMINAL_BIN="$TS4_DIR/target/debug/termsurf-terminal"
if [ ! -f "$TERMINAL_BIN" ]; then
    echo "ERROR: termsurf-terminal binary not found at $TERMINAL_BIN"
    exit 1
fi
echo "Built: $TERMINAL_BIN"

# 2. Build Swift window
echo ""
echo "=== Building Swift window ==="
cd "$TS4_DIR/termsurf-window"
swift build

WINDOW_BIN="$TS4_DIR/termsurf-window/.build/debug/termsurf-window"
if [ ! -f "$WINDOW_BIN" ]; then
    echo "ERROR: termsurf-window binary not found at $WINDOW_BIN"
    exit 1
fi
echo "Built: $WINDOW_BIN"

# 3. Register XPC service with launchd
echo ""
echo "=== Registering XPC service ==="

PLIST_PATH="/tmp/com.termsurf.ts4.terminal.plist"

cat > "$PLIST_PATH" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN"
  "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.termsurf.ts4.terminal</string>
    <key>MachServices</key>
    <dict>
        <key>com.termsurf.ts4.terminal</key>
        <true/>
    </dict>
    <key>ProgramArguments</key>
    <array>
        <string>$TERMINAL_BIN</string>
    </array>
    <key>StandardOutPath</key>
    <string>/tmp/termsurf-terminal.log</string>
    <key>StandardErrorPath</key>
    <string>/tmp/termsurf-terminal.log</string>
</dict>
</plist>
EOF

launchctl bootstrap "gui/$(id -u)" "$PLIST_PATH"
echo "Registered com.termsurf.ts4.terminal with launchd"

# 4. Run Swift window
echo ""
echo "=== Running Swift window ==="
echo "Terminal logs: /tmp/termsurf-terminal.log"
echo ""
"$WINDOW_BIN"
