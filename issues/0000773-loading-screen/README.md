+++
status = "open"
opened = "2026-04-05"
+++

# Issue 773: Loading screen for browser startup

## Goal

Show a loading indicator in the Web TUI viewport while the browser engine
starts, and display errors if something goes wrong.

## Background

Chromium's first launch after a fresh install takes ~60 seconds. During this
time, the user sees a blank terminal pane with no feedback. It looks broken.
Even subsequent launches take several seconds while the GPU process initializes.

The Web TUI already occupies the full terminal pane viewport. There is plenty of
space to display loading status, progress, and errors.

## Requirements

1. **Loading indicator** — show something immediately when `web` starts. The
   user should see feedback within the first frame.

2. **Status messages** — update as the browser progresses through startup:
   - Connecting to GUI
   - Spawning browser engine
   - Waiting for browser to initialize
   - Browser ready / page loading

3. **Error display** — if something goes wrong, show the error in the viewport
   instead of silently failing:
   - Roamium not found (not installed)
   - Roamium crashed (code signature, sandbox failure)
   - Connection timeout (browser never connected)
   - Socket error

4. **Disappear on success** — once the browser overlay appears, the loading
   screen should be replaced by the normal TUI chrome (URL bar, mode indicator).

## Analysis

The Web TUI (`webtui/src/main.rs`) already renders a TUI interface with ratatui.
The loading screen would be rendered in the same viewport before the browser
overlay appears.

The TUI currently waits for `BrowserReady` from the GUI before it knows the
browser is connected. The sequence of events that could be surfaced:

1. TUI sends `HelloRequest` → show "Connecting..."
2. TUI sends `SetOverlay` → show "Starting browser..."
3. TUI waits for `BrowserReady` → show "Waiting for Chromium..." with elapsed
   time
4. TUI receives `BrowserReady` → show "Loading page..."
5. Browser renders first frame → loading screen disappears

If step 3 takes more than ~30 seconds, show a warning that first launch is slow.
If it takes more than ~120 seconds, suggest checking if Roamium is installed.

The TUI already has a main event loop that redraws on every event. Adding a
loading state that renders a centered message in the viewport should be
straightforward.
