# Issue 729: Overlay reposition on resize and remaining protocol

## Goal

Fix overlay positioning during window resize so multi-pane layouts stay aligned,
and implement the remaining unhandled TermSurf protocol messages (DevTools and
OpenSplit).

## Background

Issue 728 brought Wezboard to interactive parity with Ghostboard for single-pane
browsing — input forwarding, cursor changes, and focus management all work. But
a positioning bug remains: when the window is resized with two side-by-side
browser panes, both panes resize correctly but the second pane's x/y origin
doesn't track its terminal pane. The overlay stays anchored to its original
pixel position instead of moving with the pane.

### Root cause: resize path skips repositioning

The `SetOverlay` handler in `conn.rs` has two paths:

1. **New overlay** (line 506+) — Creates CALayerHost, calls
   `update_ca_layer_frame()` which computes pixel x/y from grid coordinates +
   cell metrics + padding + border. Correct.

2. **Resize** (line 472-503) — Updates `pane.pixel_width`, `pane.pixel_height`,
   `pane.col`, `pane.row`, sends `Resize` to Chromium, then **returns early**.
   It never calls `update_ca_layer_frame()`, so the positioning layer's frame
   stays at the old x/y values.

When the window resizes, the TUI detects viewport changes and sends a new
`SetOverlay` with updated cell dimensions. This hits the resize path, which
updates pixel dimensions but not the frame position. For pane 1 (at column 0),
this is invisible — x stays at 0. For pane 2 (at column N), the x position
should shift because cell metrics changed, but it doesn't.

### How Ghostboard handles this

Ghostboard stores grid coordinates in the renderer and recomputes pixel
positions dynamically in `updateCALayerHostFrame()` every render frame:

```zig
const x: f64 = @as(f64, grid_col) * cw / scale + pl / scale;
const y: f64 = @as(f64, grid_row) * ch / scale + pt / scale;
```

Wezboard's `update_ca_layer_frame()` does the same math but is only called on
new overlay creation, not on resize.

### Remaining protocol messages

After Issue 728, two functional areas remain unimplemented:

| Message            | Direction        | What it does                           |
| ------------------ | ---------------- | -------------------------------------- |
| SetDevtoolsOverlay | TUI → Board      | Create DevTools pane linked to tab     |
| CreateDevtoolsTab  | Board → Chromium | Send DevTools tab creation to Chromium |
| OpenSplit          | TUI → Board      | Create a split pane in the terminal    |

These are feature extensions beyond core browsing. DevTools requires
coordinating a second overlay with an `inspected_tab_id`. OpenSplit requires
calling WezTerm's split pane API.

## Analysis

### The reposition fix

The resize path in `handle_set_overlay()` needs to call
`update_ca_layer_frame()` after updating pane state, just like the new-overlay
path does. The function already handles all the math — grid-to-pixel conversion
using cell metrics, padding, border, scale, and pane cell position from the mux.
It just isn't called.

The challenge is that `update_ca_layer_frame()` requires:

1. A mutable reference to the `Pane`
2. The root layer pointer (stored in the pane as `ca_layer_root`)
3. The state mutex to be held (for the pane lookup)

The resize path already has the state mutex locked and the pane available, so
the fix should be straightforward — call `update_ca_layer_frame()` before
returning.

### DevTools

Ghostboard's `handleSetDevtoolsOverlay` creates a pane with `inspected_tab_id`
set, then sends `CreateDevtoolsTab` to Chromium instead of `CreateTab`. The TUI
triggers this via the `:devtools` command. This requires understanding how
WezTerm creates new panes and how to associate a DevTools overlay with an
existing tab.

### OpenSplit

The TUI sends `OpenSplit` with a direction (horizontal/vertical) to create a new
terminal split pane. The board needs to call WezTerm's split pane API.
Ghostboard implements this by spawning a new terminal pane in the specified
direction.

## Experiments

### Experiment 1: Reposition overlay on resize

#### Goal

When the window resizes with multiple browser panes, each overlay's x/y position
must track its terminal pane. Currently only dimensions update; position stays
stale.

#### Root cause

The resize path in `handle_set_overlay()` (line 472-503) updates
`pane.pixel_width`, `pane.pixel_height`, `pane.col`, and `pane.row`, sends
`Resize` to Chromium, then returns at line 503 without calling
`update_ca_layer_frame()`. The positioning layer's frame keeps its old x/y.

#### Design

After sending `Resize` to Chromium, look up the mux window for this pane, get
the root layer via `get_or_create_overlay()`, get a mutable pane reference, and
call `update_ca_layer_frame()`. This mirrors exactly what `handle_ca_context`
does at lines 954-1053.

The `get_or_create_overlay(&mut st, mux_window_id)` call returns a raw pointer,
ending the mutable borrow on `st`, so `st.panes.get_mut()` can borrow again
afterward — same pattern as `handle_ca_context`.

#### Changes

**1. `termsurf/conn.rs` — Call `update_ca_layer_frame` in resize path**

Replace the early return at line 503 with:

```rust
// Reposition the overlay (x/y may have changed due to cell metric changes)
#[cfg(target_os = "macos")]
{
    if let Some(mux_window_id) = get_pane_mux_window(&overlay.pane_id) {
        if let Some(root_layer) = get_or_create_overlay(&mut st, mux_window_id) {
            if let Some(pane) = st.panes.get_mut(&overlay.pane_id) {
                unsafe {
                    update_ca_layer_frame(pane, root_layer);
                }
            }
        }
    }
}
return Ok(());
```

#### Verification

1. `cd wezboard && cargo build -p wezboard-gui` — zero errors
2. Open two side-by-side panes: `web lite.duckduckgo.com` in both
3. Resize the window horizontally — both overlays track their panes (second
   pane's left edge stays aligned with its terminal pane)
4. Resize vertically — overlays stay aligned
5. Single pane still works (regression check)
