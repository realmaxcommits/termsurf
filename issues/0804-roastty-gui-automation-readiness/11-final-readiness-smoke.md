# Experiment 11: Final Readiness Smoke

## Description

Run one post-fix end-to-end Roastty GUI automation smoke that covers the issue's
remaining readiness requirements in the current VM:

- launch the debug Roastty macOS app;
- prove the app is frontmost before synthetic input;
- drive external System Events keyboard input into the live terminal and verify
  command execution with a marker file;
- drive mouse click, drag, and scroll input into the terminal window;
- capture full-window screenshots;
- use deterministic non-OCR oracles where available;
- clean up every debug Roastty process.

Experiments 1 through 10 proved the individual routes and fixed the external
keyboard blocker. This experiment is the final integration proof that those
routes work together after the fix.

Per user instruction, this issue skips adversarial review.

## Changes

Planned issue-doc changes only:

- `issues/0804-roastty-gui-automation-readiness/11-final-readiness-smoke.md`
  - Record the plan, commands, result, and conclusion.
- `issues/0804-roastty-gui-automation-readiness/README.md`
  - Add Experiment 11 to the issue index.
  - If the smoke passes, update the learnings and close the issue.

No product-code changes are planned. If the smoke uncovers a blocker, fix only
the narrow blocker required to prove automation readiness, then record the
actual result here before moving on.

## Verification

Run from the repo root. Write all transcripts to `logs/` with the prefix
`issue804-exp11-`. Write screenshots with the existing Roastty screenshot helper
under `$TERMSURF_SHOT_DIR` or `~/.cache/termsurf/shots`.

### 1. Build and Launch

Commands:

```bash
git status --short
cargo test --manifest-path roastty/Cargo.toml managed_cell
cargo test --manifest-path roastty/Cargo.toml pending_wrap_managed
scripts/roastty-app/build-roastty-kit.sh
cd roastty/macos
xcodebuild build \
  -project Roastty.xcodeproj \
  -scheme Roastty \
  -configuration Debug \
  -derivedDataPath build
cd ../..
scripts/roastty-app/stop-app.sh || true
ROASTTY_PID="$(scripts/roastty-app/start-app.sh)"
export ROASTTY_PID
swift scripts/roastty-app/winid.swift "$ROASTTY_PID"
scripts/roastty-app/screenshot.sh "$ROASTTY_PID" issue-804-exp11-launch
```

Pass criteria:

- Focused Rust tests pass.
- The Rust kit and macOS debug app build.
- A debug Roastty process launches.
- A visible layer-0 window is found.
- Full-window screenshot capture succeeds.

### 2. Focus and Keyboard Marker

Commands:

```bash
TS=/tmp/termsurf-issue804-exp11-keyboard
mkdir -p "$TS"
rm -f "$TS/marker.txt"
osascript -e 'tell application "System Events" to set frontmost of first process whose unix id is '"$ROASTTY_PID"' to true'
osascript -e 'tell application "System Events" to name of first process whose frontmost is true'
osascript -e 'tell application "System Events" to unix id of first process whose frontmost is true'
printf 'printf "ISSUE804_EXP11_KEYBOARD" > %s/marker.txt' "$TS" > "$TS/type.txt"
osascript -e 'tell application "System Events" to keystroke (read POSIX file "'"$TS"'/type.txt")'
osascript -e 'tell application "System Events" to key code 36'
for _ in $(seq 1 20); do
  [ -f "$TS/marker.txt" ] && break
  sleep 0.25
done
cat "$TS/marker.txt"
scripts/roastty-app/screenshot.sh "$ROASTTY_PID" issue-804-exp11-after-keyboard
```

Pass criteria:

- Roastty is the frontmost process immediately before typing.
- `marker.txt` exists and contains `ISSUE804_EXP11_KEYBOARD`.
- The post-keyboard screenshot captures the live Roastty window.

### 3. Mouse Click Focus Oracle

Commands:

```bash
IFS=$'\t' read -r WID X Y W H < <(swift scripts/roastty-app/winid.swift "$ROASTTY_PID")
CX=$((X + W / 2))
CY=$((Y + H / 2))
swift scripts/roastty-app/click.swift "$CX" "$CY" 1
osascript -e 'tell application "System Events" to name of first process whose frontmost is true'
osascript -e 'tell application "System Events" to unix id of first process whose frontmost is true'
scripts/roastty-app/screenshot.sh "$ROASTTY_PID" issue-804-exp11-after-click
```

Pass criteria:

- The click helper returns success.
- Roastty remains frontmost after the click.
- A screenshot after the click succeeds.

This proves the basic click route with a focus/window oracle. Drag and scroll
use stronger content oracles below.

### 4. Mouse Drag Selection Oracle

Use external keyboard to place a known token on the screen, then use CGEvent
drag and menu-driven copy to prove the selected text reached the system
pasteboard.

Commands:

```bash
osascript -e 'tell application "System Events" to set frontmost of first process whose unix id is '"$ROASTTY_PID"' to true'
osascript -e 'tell application "System Events" to keystroke "printf \"DRAGSELECTME_EXP11_TARGET\\n\""'
osascript -e 'tell application "System Events" to key code 36'
sleep 1
scripts/roastty-app/screenshot.sh "$ROASTTY_PID" issue-804-exp11-before-drag
SX=$((X + 25))
SY=$((Y + 72))
EX=$((X + 325))
EY=$SY
swift scripts/roastty-app/drag.swift "$SX" "$SY" "$EX" "$EY" 16
osascript -e 'tell application "System Events" to keystroke "c" using command down'
pbpaste
```

Pass criteria:

- `pbpaste` contains `DRAGSELECTME_EXP11_TARGET`.

If the fixed row misses on this VM, rerun once after recomputing the text-row
coordinate from the current screenshot and record both attempts.

### 5. Mouse Scroll Oracle

Use external keyboard to print a long sequence, then scroll up and back down
over the terminal window while capturing screenshots at each state.

Commands:

```bash
osascript -e 'tell application "System Events" to set frontmost of first process whose unix id is '"$ROASTTY_PID"' to true'
osascript -e 'tell application "System Events" to keystroke "seq 1 200"'
osascript -e 'tell application "System Events" to key code 36'
sleep 1
scripts/roastty-app/screenshot.sh "$ROASTTY_PID" issue-804-exp11-scroll-bottom
swift scripts/roastty-app/scroll.swift "$CX" "$CY" 24
sleep 1
scripts/roastty-app/screenshot.sh "$ROASTTY_PID" issue-804-exp11-scroll-up
swift scripts/roastty-app/scroll.swift "$CX" "$CY" -24
sleep 1
scripts/roastty-app/screenshot.sh "$ROASTTY_PID" issue-804-exp11-scroll-down
```

Pass criteria:

- Scroll helpers return success.
- The three screenshots are created.
- The scroll-up screenshot differs from the bottom/down screenshots in the
  visible terminal content, proving the viewport moved.

### 6. Cleanup

Commands:

```bash
scripts/roastty-app/stop-app.sh || true
pgrep -fl 'roastty/macos/build/.*/Roastty.app/Contents/MacOS/roastty' || true
```

Pass criteria:

- No debug Roastty process remains after cleanup.

Overall result:

- **Pass** if every section above satisfies its pass criteria.
- **Partial** if keyboard, screenshot, drag, and scroll pass but basic click has
  only the focus/window oracle.
- **Fail** if any launched debug Roastty process cannot be cleaned up or if
  external keyboard input regresses.
