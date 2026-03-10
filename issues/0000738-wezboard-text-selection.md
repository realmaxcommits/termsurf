# Issue 738: Wezboard text selection

## Goal

Click+drag text selection should work in Wezboard browser overlays — single
click to place cursor, drag to select, double-click for word, triple-click for
line.

## Background

Text selection works in Ghostboard but not in Wezboard. The GUI is responsible
for forwarding mouse events to Chromium via the TermSurf protocol (MouseEvent
and MouseMove messages). The TUI (webtui) is not involved in mouse forwarding —
it intentionally drops all mouse events.

### How Ghostboard does it

Ghostboard's working implementation (`Surface.zig` + `xpc.zig`) has three key
mechanisms:

1. **Click count tracking.** `mouseButtonCallback` (Surface.zig:4021–4080)
   tracks `left_click_count` by measuring time and distance between clicks. If
   the next click is within the timing window and close enough, the count
   increments (1→2→3→1). The count is sent in `MouseEvent.click_count`, enabling
   Chromium's double-click (word) and triple-click (line) selection.

2. **Button-down flags in MouseMove.** `sendMouseMove` (xpc.zig:1272–1304) reads
   `click_state[LEFT]` and sets `modifiers |= 64` when the left button is held.
   This lets Chromium distinguish drag (selecting text) from hover (just moving
   the cursor).

3. **Persistent click state.** `click_state` is updated in `mouseButtonCallback`
   _before_ the overlay hit-test, so it persists across move events. A press
   sets `.press`, a release sets `.release`. Move events read this state to
   encode button-down flags.

### What Wezboard gets wrong

Three bugs in `wezboard/wezboard-gui/src/termsurf/input.rs`:

1. **Click count always 1.** Lines 131, 152, and 173 hardcode `click_count: 1`.
   No timing or distance tracking exists. Double-click and triple-click
   selection are impossible.

2. **No button-down flags in MouseMove.** Lines 179–191 send MouseMove with only
   keyboard modifiers (shift/ctrl/alt/super). The `modifiers_to_termsurf`
   function (lines 252–271) doesn't encode button state. Chromium receives move
   events but can't tell if a button is held, so it treats every move as a hover
   — no drag selection.

3. **MouseMove stops at overlay boundary.** The `hit_test_overlay` check at line
   109 gates all event forwarding. If the user clicks inside the overlay and
   drags outside it, MouseMove events stop. Selection freezes mid-drag.

### Fix approach

Add click state tracking to `input.rs` (or `state.rs`):

- Track which buttons are currently pressed.
- Track left-click timestamp and position for click count calculation.
- On MouseMove, encode button-down flags in modifiers (bit 6 = left, bit 8 =
  right), matching Ghostboard's convention.
- On MouseMove outside the overlay while a button is held, clamp coordinates to
  the overlay bounds and continue sending events.
