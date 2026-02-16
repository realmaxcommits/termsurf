# Issue 508: Retina Checkerboard with Safe IOSurface Lifetime

## Background

Issue 507 proved the full Chromium integration pipeline works — IOSurface frames
streamed at 60fps from Chromium Profile Server through XPC to the Metal
renderer. But it crashed after ~3 seconds. The same crash occurred when resizing
the terminal with a static checkerboard overlay.

Both crashes have the same root cause: **IOSurface use-after-free across the
Swift/Zig boundary.** Swift passes the IOSurface to Zig as a raw pointer via
`Unmanaged.passUnretained().toOpaque()`. When Swift replaces or releases the
IOSurface (on resize or new frame), ARC frees it while the Zig renderer still
holds the dangling pointer.

This issue isolates the lifetime problem using a simple test case — a
checkerboard IOSurface — without any Chromium complexity. Once the checkerboard
survives resize without crashing, the same fix applies to live Chromium frames.

### What exists today (Issue 505)

- **Pink overlay pipeline** (`pink_overlay` in `shaders.zig` / `shaders.metal`)
  renders a solid hot-pink quad at grid coordinates. No IOSurface, no texture
  sampling — just a constant color from the fragment shader.
- **C API** (`ghostty_surface_set_overlay` / `clear_overlay`) sets grid
  coordinates on the renderer under `draw_mutex`.
- **`web` TUI** sends viewport grid coordinates via XPC. The pink quad appears
  at the exact viewport position and clears on disconnect.

### What Issue 507 added and reverted

- **IOSurface texture overlay** (`overlay` pipeline, `overlay_vertex` /
  `overlay_fragment` shaders) — samples from an IOSurface-backed Metal texture
  instead of returning a constant color.
- **`ghostty_surface_set_overlay_iosurface`** — C API to pass an IOSurface
  pointer to the renderer.
- **`ghostty_surface_get_cell_size`** — C API to query physical pixel dimensions
  of a terminal cell (already includes Retina scale factor via DPI-scaled font
  metrics).
- **IOSurface texture import** (`Texture.fromIOSurface`) — creates a Metal
  texture from an IOSurface reference.
- **Checkerboard test surface** — Swift code in `CompositorXPC.swift` that
  creates an IOSurface, fills it with a blue/dark checkerboard pattern, and
  passes it to the renderer.

All of this code was reverted to the pink overlay state. This issue will
reimplement it with proper IOSurface lifetime management.

## The Problem

The renderer runs on its own thread. The IOSurface pointer is set from the main
thread (or XPC queue) under `draw_mutex`. The mutex protects the pointer swap
but not the IOSurface lifetime:

```
Thread A (main/XPC):          Thread B (renderer):
───────────────────           ────────────────────
lock(draw_mutex)
  old = overlay_surface
  overlay_surface = new
  // ARC releases old         reading old surface's memory
unlock(draw_mutex)            → USE AFTER FREE
```

The Zig side stores a raw `*anyopaque` pointer. It has no way to prevent ARC
from releasing the IOSurface because it doesn't participate in reference
counting.

## The Fix: CFRetain/CFRelease on the Zig Side

The simplest fix: when the Zig renderer receives a new IOSurface pointer, it
calls `CFRetain` on the new one and `CFRelease` on the old one. This gives the
Zig side its own ownership stake — ARC on the Swift side can release freely
because the Zig retain keeps the surface alive.

```
Thread A (main/XPC):          Thread B (renderer):
───────────────────           ────────────────────
lock(draw_mutex)
  CFRelease(old)
  overlay_surface = new
  CFRetain(new)
unlock(draw_mutex)            reading surface → safe, Zig holds a retain
```

The `draw_mutex` serializes the swap, and the Zig-side retain prevents
deallocation until the renderer is done. On `clearOverlay`, the Zig side calls
`CFRelease` on the current surface.

### Why not double-buffering?

Double-buffering (two IOSurface slots, swap atomically) is more complex and
doesn't solve the fundamental problem — someone still needs to manage the
lifetime of the "old" slot. CFRetain/CFRelease is the direct solution.

### Why not Mach port lookup per frame?

Calling `IOSurfaceLookupFromMachPort` on the render thread would work but adds
per-frame overhead and is only relevant for the cross-process Chromium case. The
checkerboard doesn't use Mach ports. And even for Chromium, the Swift side
already does the lookup — passing the result with proper retain is cleaner.

## Current State (starting point)

| Component                               | State                                     |
| --------------------------------------- | ----------------------------------------- |
| `pink_overlay` pipeline                 | Working — solid color quad at grid coords |
| `overlay` pipeline (IOSurface texture)  | Reverted — needs reimplementation         |
| `ghostty_surface_set_overlay_iosurface` | Reverted — needs reimplementation         |
| `ghostty_surface_get_cell_size`         | Reverted — needs reimplementation         |
| `Texture.fromIOSurface`                 | Reverted — needs reimplementation         |
| Checkerboard test surface (Swift)       | Reverted — needs reimplementation         |
| CFRetain/CFRelease lifetime management  | Never existed — new work                  |

## Key Files

| File                                            | Role                                                                     |
| ----------------------------------------------- | ------------------------------------------------------------------------ |
| `ts5/src/renderer/shaders/shaders.metal`        | Metal shaders (add `overlay_vertex`/`overlay_fragment`)                  |
| `ts5/src/renderer/metal/shaders.zig`            | Pipeline definitions (add `overlay` pipeline, `OverlayParams` struct)    |
| `ts5/src/renderer/metal/Texture.zig`            | IOSurface → Metal texture import (`fromIOSurface`)                       |
| `ts5/src/renderer/generic.zig`                  | Renderer state (`overlay_iosurface` field, render step in `drawFrame()`) |
| `ts5/src/Surface.zig`                           | `setOverlayIOSurface()` / `clearOverlay()` with CFRetain/CFRelease       |
| `ts5/src/apprt/embedded.zig`                    | C API exports                                                            |
| `ts5/include/ghostty.h`                         | C API declarations                                                       |
| `ts5/macos/Sources/Ghostty/CompositorXPC.swift` | Checkerboard creation, `set_overlay` handler                             |

## Pass Criteria

1. `cargo run -p web -- http://example.com` shows a blue/dark checkerboard at
   Retina resolution in the viewport area. Each checker cell is exactly one
   terminal cell with sharp edges.
2. **Resizing the terminal window does not crash.** The checkerboard recreates
   at the new size and remains pixel-perfect.
3. Quitting `web` clears the overlay.
4. No crash during normal operation.

## Experiments

_To be designed._
