#!/usr/bin/env bash
# Issue 802 / Exp 5 — drive the keyboard + mouse input matrix against the real
# Ghostty app and collect oracles (PTY byte log, pasteboard, window state).
# Keyboard = osascript System Events; mouse = inject.swift (CGEvent). Activate-first.
#
#   input-matrix.sh <stage>
#     check | bootstrap | probe-start[ modes] | probe-stop | bytes |
#     keyA-printable | keyA-edit | keyA-arrows | keyA-nav | keyA-fn |
#     keyA-ctrl | keyA-meta | mouse-move | mouse-select | mouse-scroll
#
# Pauses use `osascript delay` (the agent's bash blocks `sleep`).
set -uo pipefail
TS=/tmp/ghostty-exp5
DIR="$(cd "$(dirname "$0")" && pwd)"
APP="${GHOSTTY_APP:-/Users/ryan/dev/termsurf/vendor/ghostty/macos/build/Debug/Ghostty.app}"
LOG="$TS/bytes.log"
mkdir -p "$TS"

p() { osascript -e "delay ${1:-0.35}" >/dev/null 2>&1; }
act() { osascript -e "tell application \"$APP\" to activate" >/dev/null 2>&1; p 0.5; }
ktype() { printf '%s' "$1" >"$TS/_kt"; osascript -e 'tell application "System Events" to keystroke (read POSIX file "/tmp/ghostty-exp5/_kt")' >/dev/null; }
kret() { osascript -e 'tell application "System Events" to key code 36' >/dev/null; }
kkey() { osascript -e "tell application \"System Events\" to key code $1" >/dev/null; }
kmod() { osascript -e "tell application \"System Events\" to key code $1 using {$2}" >/dev/null; }
kchar() { printf '%s' "$1" >"$TS/_kt"; osascript -e "tell application \"System Events\" to keystroke (read POSIX file \"/tmp/ghostty-exp5/_kt\") using {$2}" >/dev/null; }
runcmd() { ktype "$1"; kret; p 0.4; }       # type a shell command + Return
mark() { printf -- "--- %s\n" "$1" >>"$LOG"; }  # label the byte log

case "${1:-}" in
check)
  pgrep -fl "Ghostty.app/Contents/MacOS/ghostty" || { echo "launching"; open "$APP"; p 5; pgrep -fl "MacOS/ghostty"; }
  ;;

bootstrap)
  act
  runcmd "exec bash --norc --noprofile"
  runcmd "export TS=/tmp/ghostty-exp5; export PS1=READY\\ ; stty sane; mkdir -p \$TS"
  rm -f "$TS/marker"
  runcmd "echo MARKER > \$TS/marker"
  p 0.5
  echo "marker file:"; cat "$TS/marker" 2>/dev/null || echo "  (absent — text+Return injection FAILED)"
  ;;

probe-start)
  act
  : >"$LOG"
  runcmd "python3 $DIR/byteprobe.py \$TS/bytes.log ${2:-}"
  p 0.8
  echo "probe started (modes=${2:-none}); log header:"; head -1 "$LOG"
  ;;

probe-stop)
  pkill -f byteprobe.py && echo "probe killed" || echo "no probe running"
  ;;

bytes) tail -40 "$LOG" ;;

keyA-printable)
  act; mark "printable"
  ktype "abcXYZ 012 ~!@#%^&*()_+{}|:<>?"
  p 0.4; echo "see byte log"
  ;;
keyA-edit)
  act; mark "edit"
  kkey 49; mark "space"        # space
  kkey 48; mark "tab"          # tab
  kkey 36; mark "return"       # return
  kkey 51; mark "backspace"    # delete (backspace)
  kkey 117; mark "fwd-delete"  # forward delete
  p 0.4 ;;
keyA-arrows)
  act; mark "arrows"
  kkey 126; kkey 125; kkey 123; kkey 124   # up down left right
  p 0.4 ;;
keyA-nav)
  act; mark "nav"
  kkey 115; kkey 119; kkey 116; kkey 121   # home end pgup pgdn
  p 0.4 ;;
keyA-fn)
  act; mark "fn"
  for c in 122 120 99 118 96 97 98 100 101 109 103 111; do kkey "$c"; done
  p 0.4 ;;
keyA-esc)
  act; mark "esc"; kkey 53; p 0.3 ;;
keyA-ctrl)
  act; mark "ctrl"
  for ch in a e b f u k w l; do kchar "$ch" "control down"; done   # avoid c/d/z here
  p 0.4 ;;
keyA-ctrl-danger)
  act; mark "ctrl-cdz"
  kchar "c" "control down"; mark "ctrl-c"
  p 0.3 ;;
keyA-meta)
  act; mark "meta"
  kchar "b" "option down"; kchar "f" "option down"   # alt-b alt-f
  p 0.4 ;;
keyA-chord)
  act; mark "chord"
  kmod 124 "shift down"; kmod 124 "option down"; kmod 123 "command down"
  p 0.4 ;;

*) echo "unknown stage: ${1:-}"; exit 2 ;;
esac
