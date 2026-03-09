# Issue 731: Wezboard scroll crashes Roamium

## Goal

Fix scrolling inside Wezboard's browser overlay — currently any scroll event
causes Roamium (the Chromium browser engine) to crash, making the webview
vanish.

## Background

### Symptom

In Wezboard, loading a page works fine. As soon as the user scrolls (trackpad or
mouse wheel), the webview disappears. The Roamium process crashes.

### How Ghostboard handles scroll (working)

Ghostboard captures raw macOS `NSEvent` scroll data in Swift
(`SurfaceView_AppKit.swift:1002-1017`), stores it in the Zig `CoreSurface`
struct (`Surface.zig:70-78`), and forwards it via protobuf
(`xpc.zig:1236-1268`).

Key details from the working implementation:

- **Raw NSEvent phases** are passed through. `NSEventPhase` values are bitmask
  flags: `none=0`, `began=1`, `changed=2`, `stationary=4`, `ended=8`,
  `cancelled=16`, `mayBegin=32`.
- **`precise`** is set from `NSEvent.hasPreciseScrollingDeltas` — true for
  trackpad, false for mouse wheel.
- **`delta_x`/`delta_y`** are raw `NSEvent.scrollingDeltaX/Y` values (pixels for
  trackpad, lines for mouse wheel).

### How Wezboard handles scroll (broken)

Wezboard's `input.rs:192-208` handles `VertWheel(delta)`:

```rust
WMEK::VertWheel(delta) => {
    send_to_chromium(
        &pane_id_str,
        Msg::ScrollEvent(proto::ScrollEvent {
            tab_id: 0,
            x: rel_x,
            y: rel_y,
            delta_x: 0.0,
            delta_y: *delta as f64,
            phase: 4,
            momentum_phase: 0,
            precise: false,
            modifiers: mods,
        }),
    );
}
```

### The phase value mismatch

The C API header documents phase values as
`0=none, 1=began, 2=changed, 3=ended`. The Chromium implementation
(`ForwardScrollEvent` in `ts_browser_main_parts.cc`) casts phase directly to
`blink::WebMouseWheelEvent::Phase`:

```cpp
wheel_event.phase =
    static_cast<blink::WebMouseWheelEvent::Phase>(phase);
```

Blink's `WebMouseWheelEvent::Phase` enum:

| Value | Name             |
| ----- | ---------------- |
| 0     | kPhaseNone       |
| 1     | kPhaseBegan      |
| 2     | kPhaseStationary |
| 4     | kPhaseChanged    |
| 8     | kPhaseEnded      |
| 16    | kPhaseCancelled  |
| 32    | kPhaseMayBegin   |

These are **bitmask values**, matching macOS `NSEventPhase` exactly. So the C
API header comment (`0=none, 1=began, 2=changed, 3=ended`) is **wrong** — the
implementation accepts raw NSEventPhase bitmask values.

Ghostboard passes raw `NSEventPhase` values and it works. Wezboard sends
`phase: 4` which is `kPhaseChanged` — not inherently wrong, but sending a single
`kPhaseChanged` without a preceding `kPhaseBegan` may confuse Chromium's scroll
state machine.

### Likely crash cause

WezTerm's `VertWheel` is a discrete mouse wheel event, not a trackpad gesture.
For discrete scrolling:

- Ghostboard would send raw NSEvent data with `phase: 0` (none) and
  `precise: false`, meaning `delta_units = kScrollByLine`
- Wezboard sends `phase: 4` (`kPhaseChanged`) which tells Chromium this is a
  mid-gesture trackpad scroll, but there was never a `kPhaseBegan` (1) to start
  the gesture — Chromium's scroll handling may crash or enter an invalid state

The fix is likely: for discrete wheel events, send `phase: 0` (none) and
`momentum_phase: 0` (none). This tells Chromium it's a standalone wheel tick,
not part of a gesture sequence.

### Additional concern: HorzWheel

WezTerm also has `HorzWheel(delta)` for horizontal scrolling. Wezboard's
`input.rs` currently falls through to the `_ => return true` catch-all, silently
dropping horizontal scroll events. This should also be forwarded.

## Experiments

### Experiment 1: Fix phase values for discrete scroll

#### Goal

Change Wezboard's scroll event to use `phase: 0` and `momentum_phase: 0` for
discrete wheel events. Add `HorzWheel` support.

#### Design

**1. Fix `VertWheel` in `input.rs`**

```rust
WMEK::VertWheel(delta) => {
    let mods = modifiers_to_termsurf(event.modifiers);
    send_to_chromium(
        &pane_id_str,
        Msg::ScrollEvent(proto::ScrollEvent {
            tab_id: 0,
            x: rel_x,
            y: rel_y,
            delta_x: 0.0,
            delta_y: *delta as f64,
            phase: 0,
            momentum_phase: 0,
            precise: false,
            modifiers: mods,
        }),
    );
    return true;
}
```

**2. Add `HorzWheel` in `input.rs`**

```rust
WMEK::HorzWheel(delta) => {
    let mods = modifiers_to_termsurf(event.modifiers);
    send_to_chromium(
        &pane_id_str,
        Msg::ScrollEvent(proto::ScrollEvent {
            tab_id: 0,
            x: rel_x,
            y: rel_y,
            delta_x: *delta as f64,
            delta_y: 0.0,
            phase: 0,
            momentum_phase: 0,
            precise: false,
            modifiers: mods,
        }),
    );
    return true;
}
```

#### Verification

1. Wezboard builds without errors
2. Load a page in Wezboard — scrolling with mouse wheel doesn't crash
3. Page scrolls smoothly in the expected direction
4. Horizontal scrolling works on pages with horizontal overflow
5. Compare scroll behavior with Ghostboard as reference
