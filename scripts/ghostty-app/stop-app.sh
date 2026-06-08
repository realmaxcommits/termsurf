#!/usr/bin/env bash
# Issue 802 — cleanly stop the debug Ghostty app this harness spawned, plus the byte
# probe. Kills by PID, scoped to the build-output path, with SIGKILL so there is NO
# graceful-quit confirmation dialog. Touches nothing but the debug build under
# vendor/ghostty/macos/build/ — never an installed/stable Ghostty or any other app.
set -uo pipefail
SCOPE="${1:-vendor/ghostty/macos/build/.*Ghostty.app/Contents/MacOS/ghostty}"

# the in-Ghostty byte probe first (so its atexit restore runs via the SIGTERM handler)
pkill -f "scripts/ghostty-app/byteprobe.py" 2>/dev/null && echo "stopped byteprobe"

PIDS=$(pgrep -f "$SCOPE" || true)
if [ -n "$PIDS" ]; then
  echo "killing debug Ghostty PIDs: $PIDS"
  kill -9 $PIDS
else
  echo "no debug Ghostty running"
fi
