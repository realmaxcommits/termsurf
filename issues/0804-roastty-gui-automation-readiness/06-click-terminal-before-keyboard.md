# Experiment 6: Click Terminal Before Keyboard

## Description

Test the strongest remaining non-instrumented hypothesis: Roastty may be
frontmost, but the terminal surface may not be the first responder. Experiments
2 through 5 made Roastty frontmost before posting keyboard events, but they did
not reliably mimic XCTest's successful sequence of clicking `"Terminal pane"`
before typing.

This experiment explicitly clicks inside the live Roastty terminal content
before retrying both external keyboard routes:

1. System Events `keystroke` / `key code`.
2. CGEvent keyboard via `scripts/ghostty-app/inject.swift`.

The experiment should use the same marker-file oracle as the prior keyboard
experiments. If this works, update the issue learnings with the required focus
sequence. If it fails, the next experiment should instrument Roastty's AppKit
keyboard entry points.

Per user instruction, this issue skips adversarial review.

## Changes

- `issues/0804-roastty-gui-automation-readiness/06-click-terminal-before-keyboard.md`
  - Record this focused rerun and result.
- `issues/0804-roastty-gui-automation-readiness/README.md`
  - Add Experiment 6 to the issue index.

No product code or harness code should change in this experiment.

## Verification

Run from the repo root. Store transcripts in `logs/` with the `issue804-exp6-`
prefix.

### 1. Preflight and Launch

Commands:

```bash
git status --short
swift -e 'import ApplicationServices; print(AXIsProcessTrusted())'
osascript -e 'tell application "System Events" to count processes'
scripts/roastty-app/stop-app.sh || true
ROASTTY_PID="$(scripts/roastty-app/start-app.sh)"
export ROASTTY_PID
pgrep -fl 'Roastty.app/Contents/MacOS/roastty'
swift scripts/roastty-app/list-windows.swift "$ROASTTY_PID"
swift scripts/roastty-app/winid.swift "$ROASTTY_PID"
osascript -e 'tell application "System Events" to set frontmost of first process whose unix id is '"$ROASTTY_PID"' to true'
osascript -e 'tell application "System Events" to name of first process whose frontmost is true'
scripts/roastty-app/screenshot.sh "$ROASTTY_PID" issue-804-exp6-before-keyboard
```

Pass criteria:

- Accessibility and Apple Events preflights pass.
- Debug Roastty launches.
- The visible `800x632` terminal window is discovered.
- Roastty is frontmost.
- Screenshot capture works.

### 2. Compute Focus Coordinates

Commands:

```bash
LINE="$(swift scripts/roastty-app/list-windows.swift "$ROASTTY_PID" | awk '/layer=0/ { print; exit }')"
read -r X Y W H < <(printf '%s\n' "$LINE" |
  sed -E 's/.*bounds=\(([0-9.-]+),([0-9.-]+) ([0-9.-]+)x([0-9.-]+)\).*/\1 \2 \3 \4/' |
  awk '{ printf "%d %d %d %d\n", $1, $2, $3, $4 }')
FOCUS_X=$((X + 40))
FOCUS_Y=$((Y + 72))
SAFE_X=$((X + 120))
SAFE_Y=$((Y + 140))
printf 'window=%s,%s %sx%s focus=%s,%s safe=%s,%s\n' "$X" "$Y" "$W" "$H" "$FOCUS_X" "$FOCUS_Y" "$SAFE_X" "$SAFE_Y"
```

Pass criteria:

- Coordinates are inside the visible Roastty terminal window.
- `FOCUS_Y` targets the text row area that worked for Experiment 4 drag
  selection.
- `SAFE_Y` targets the terminal content area away from titlebar/debug banner.

### 3. System Events Keyboard After Terminal Click

Commands:

```bash
TS=/tmp/termsurf-issue804-exp6-system-events
mkdir -p "$TS"
rm -f "$TS/marker.txt"
osascript -e 'tell application "System Events" to set frontmost of first process whose unix id is '"$ROASTTY_PID"' to true'
swift scripts/ghostty-app/inject.swift click "$SAFE_X" "$SAFE_Y" left 1
swift scripts/ghostty-app/inject.swift click "$FOCUS_X" "$FOCUS_Y" left 1
osascript -e 'delay 0.7'
osascript -e 'tell application "System Events" to key code 49'
printf 'exec bash --norc --noprofile' > "$TS/type.txt"
osascript -e 'tell application "System Events" to keystroke (read POSIX file "'"$TS"'/type.txt")'
osascript -e 'tell application "System Events" to key code 36'
osascript -e 'delay 0.7'
printf 'printf "ISSUE804_EXP6_SYSTEM_EVENTS\n" > '"$TS"'/marker.txt' > "$TS/type.txt"
swift scripts/ghostty-app/inject.swift click "$FOCUS_X" "$FOCUS_Y" left 1
osascript -e 'delay 0.3'
osascript -e 'tell application "System Events" to keystroke (read POSIX file "'"$TS"'/type.txt")'
osascript -e 'tell application "System Events" to key code 36'
osascript -e 'delay 0.7'
cat "$TS/marker.txt"
```

Pass criteria:

- `marker.txt` exists and contains `ISSUE804_EXP6_SYSTEM_EVENTS`.

### 4. CGEvent Keyboard After Terminal Click

Commands:

```bash
TS=/tmp/termsurf-issue804-exp6-cgevent
mkdir -p "$TS"
rm -f "$TS/marker.txt"
osascript -e 'tell application "System Events" to set frontmost of first process whose unix id is '"$ROASTTY_PID"' to true'
swift scripts/ghostty-app/inject.swift click "$SAFE_X" "$SAFE_Y" left 1
swift scripts/ghostty-app/inject.swift click "$FOCUS_X" "$FOCUS_Y" left 1
osascript -e 'delay 0.7'
swift scripts/ghostty-app/inject.swift key 49
printf 'exec bash --norc --noprofile' > "$TS/type.txt"
swift scripts/ghostty-app/inject.swift type "$TS/type.txt"
swift scripts/ghostty-app/inject.swift key 36
osascript -e 'delay 0.7'
printf 'printf "ISSUE804_EXP6_CGEVENT\n" > '"$TS"'/marker.txt' > "$TS/type.txt"
swift scripts/ghostty-app/inject.swift click "$FOCUS_X" "$FOCUS_Y" left 1
osascript -e 'delay 0.3'
swift scripts/ghostty-app/inject.swift type "$TS/type.txt"
swift scripts/ghostty-app/inject.swift key 36
osascript -e 'delay 0.7'
cat "$TS/marker.txt"
```

Pass criteria:

- `marker.txt` exists and contains `ISSUE804_EXP6_CGEVENT`.

### 5. Investigation and Cleanup

If either route fails, capture post-attempt state:

```bash
scripts/roastty-app/screenshot.sh "$ROASTTY_PID" issue-804-exp6-after-keyboard
osascript -e 'tell application "System Events" to name of first process whose frontmost is true'
osascript -e 'tell application "System Events" to tell (first process whose unix id is '"$ROASTTY_PID"') to get {name, frontmost, visible, enabled}'
scripts/roastty-app/stop-app.sh
pgrep -fl 'Roastty.app/Contents/MacOS/roastty' || true
```

Pass criteria:

- Cleanup leaves no debug Roastty process running.
- Any failure is classified as "terminal-click focus did not fix external
  keyboard delivery" with screenshot and frontmost evidence.

Overall result:

- **Pass** if either external keyboard route creates its marker file after the
  explicit terminal clicks.
- **Partial** if keyboard still fails but the experiment proves the
  first-responder click hypothesis is insufficient.
- **Fail** if Roastty cannot be launched, focused, clicked, or observed.

## Result

Not run yet.
