# Issue 747: Overlay doesn't reposition on split (second screen)

## Goal

When a pane is split, the webview overlay must reposition immediately — not just
resize. This must work on all screens, not just the primary display.

## Background

Issue 746 replaced the overlay's broken positioning system with one that
piggybacks on the terminal's own render pass. The old system
(`reposition_all_overlays`, `get_pane_cell_position`, global `metrics` atomics)
computed overlay coordinates independently and was wrong in multiple ways. The
new system returns `(left_pixel_x, top_pixel_y)` from `paint_pane()` and passes
those coordinates directly to `set_overlay_frame()`, which converts backing
pixels to logical points (`dpi / 72.0`) and sets the CALayer frame inside a
`CATransaction` (to suppress animation).

Issue 746 also added a `MuxNotification::WindowInvalidated` notification after
the async split completes (spawn.rs), ensuring the GUI repaints with the updated
split tree.

This works on the primary screen. On a secondary screen, it doesn't.

### What works

- Single window, single screen: open, split, resize, tab switch — all correct.
- Scale correct on Retina (`dpi / 72.0` = 2.0).
- No animation artifacts (CATransaction suppresses implicit animations).
- Split triggers immediate overlay reposition via `WindowInvalidated`.

### The bug

Steps to reproduce:

1. Open a new window.
2. Move the window to the second screen.
3. Open a webview. It positions correctly.
4. Split pane to the left.
5. The webview **resizes** (width shrinks correctly) but does **not reposition**
   (x stays at 0, left edge, instead of moving to the right pane).
6. Press any key — the overlay snaps to the correct position.

This does not happen on the primary screen. The same window on the primary
screen repositions correctly on split. The bug is specific to a window on a
secondary display.

### What this tells us

The overlay's width and x position are both set in the same
`set_overlay_frame()` call from `paint_pass()`. The width comes from
`pane.pixel_width` (updated by the TUI's protocol resize message after
SIGWINCH). The x comes from `pane_pixel_x` (returned by `paint_pane()`, which
computes `padding_left + border.left + pos.left * cell_width`).

Since the width updates but x doesn't, the paint pass is running — but with
`pos.left` still at 0 (the pre-split value). Then a keypress triggers another
paint which sees the correct `pos.left`.

The `WindowInvalidated` notification travels through a 4-hop async chain:

1. `mux.notify()` → subscriber calls `spawn_into_main_thread`
2. `mux_pane_output_event_callback` → `window.notify()` →
   `Connection::with_window_inner` → `spawn_into_main_thread`
3. Event handler dispatches `WindowInvalidated` → `window.invalidate()` →
   `Connection::with_window_inner` → `spawn_into_main_thread`
4. Inner invalidate runs `setNeedsDisplay: true`

On macOS, the spawn queue processes one task per `CFRunLoopObserver` invocation.
Each display has its own `CVDisplayLink` at its own phase. The hypothesis is
that on the second screen, display timing causes a repaint between the tree
update and the `setNeedsDisplay: true` arriving — so the repaint sees old
positions but new sizes (since the TUI's resize message updates
`pane.pixel_width` through a different path).

However, `get_panes_to_render()` reads directly from `tab.iter_panes()`, which
walks the live split tree. The tree is updated before the notification is sent.
So any paint after the split should see the correct `pos.left`. This contradicts
the observed behavior and suggests the root cause may be elsewhere.

### Key files

- `wezboard/wezboard-gui/src/termwindow/render/paint.rs` — `paint_pass()`,
  overlay positioning block (lines 263–288)
- `wezboard/wezboard-gui/src/termwindow/render/pane.rs` — `paint_pane()`,
  returns `(left_pixel_x, top_pixel_y)`
- `wezboard/wezboard-gui/src/termsurf/conn.rs` — `set_overlay_frame()`,
  `update_ca_layer_frame()` (initial placement)
- `wezboard/wezboard-gui/src/spawn.rs` — `WindowInvalidated` notification after
  split
- `wezboard/wezboard-gui/src/termwindow/mod.rs` — `WindowInvalidated` handler
  (line 1328), notification filter (line 1552)
- `wezboard/window/src/os/macos/window.rs` — `invalidate()` →
  `setNeedsDisplay: true`
- `wezboard/window/src/os/macos/connection.rs` — `with_window_inner` →
  `spawn_into_main_thread`

### Analysis

The root cause is not yet identified. The theoretical analysis in Issue 746
explored display link phase differences, spawn queue hop counts, and layer
coordinate systems, but none fully explain why `pos.left` would be stale when
the split tree has already been updated. Logging is needed to determine:

1. Whether `set_overlay_frame` is called with the wrong x on the second screen,
   or not called at all after the split.
2. What `pos.left` value `paint_pass` sees on the second screen after the split.
3. Whether the `WindowInvalidated` notification reaches the second window and
   triggers `setNeedsDisplay: true`.
4. The timing relationship between the split completion, the notification chain,
   and the display refresh on the second screen.
