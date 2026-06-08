#!/usr/bin/env bash
# Issue 802 — launch the debug Ghostty app and wait until its window is up. Pairs with
# stop-app.sh. Prints the main process PID. Use start → drive → stop in one flow so the
# app is never left running on the user's screen across turns.
set -uo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
APP="${GHOSTTY_APP:-$ROOT/vendor/ghostty/macos/build/Debug/Ghostty.app}"
[ -d "$APP" ] || { echo "app not built: $APP" >&2; exit 1; }

open "$APP"
for _ in $(seq 1 20); do
  pid=$(pgrep -f "$APP/Contents/MacOS/ghostty" | head -1 || true)
  [ -n "$pid" ] && { osascript -e 'delay 1' >/dev/null 2>&1; echo "$pid"; exit 0; }
  osascript -e 'delay 0.5' >/dev/null 2>&1
done
echo "launch timed out" >&2
exit 1
