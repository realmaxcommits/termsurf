# Issue 629: Understand Navigation Blank in CALayerHost

## Goal

Understand **why** the browser overlay disappears for ~10 seconds when the user
clicks a link. This is a research issue — the goal is diagnosis, not a fix.

## Background

### The CALayerHost migration

[Issue 625](625-calayerhost.md) replaced the `FrameSinkVideoCapturer` pipeline
with `CALayerHost`. Instead of capturing IOSurface frames at 120fps and
transferring Mach ports over XPC every frame, Chromium now sends a
`ca_context_id` (uint32) once per tab. The GUI creates a `CALayerHost` sublayer,
and Window Server composites the remote content directly from GPU VRAM.

This migration broke several things that worked under the old pipeline:

- [Issue 626](626-x-y-calayerhost.md) — X/Y positioning was offset. Fixed.
- [Issue 627](627-resize-calayerhost.md) — Resize stopped working. Fixed.
- [Issue 628](628-navigation-calayerhost.md) — Navigation causes a ~10s blank.
  **Unresolved after 8 experiments.**

Under the old IOSurface pipeline, navigation was invisible — every frame
delivered a new Mach port, and the Metal shader re-read the texture every frame
in `drawFrame()`. The new surface just showed up. With CALayerHost, there is no
per-frame update. The `ca_context_id` is set once, and Window Server composites
from that context. When navigation produces a new `CAContext`, the old one
becomes invalid and the overlay goes blank.

There may be additional regressions from the CALayerHost migration that haven't
been tested yet.

### What Issue 628 tried and failed

Issue 628 ran 8 experiments targeting the Chromium-side pipeline. All failed to
fix the ~10-second blank:

| Exp | Approach                                               | Result             |
| --- | ------------------------------------------------------ | ------------------ |
| 1   | Re-register callback on view swap, replace CALayerHost | No effect on blank |
| 2   | Re-apply size in `RenderViewHostChanged`               | Fail               |
| 3   | Research: Electron/Chromium sizing                     | Research only      |
| 4   | Resize NSWindow instead of `view->SetSize()`           | No effect on blank |
| 5   | Research: navigation transitions, dedup gate           | Research only      |
| 6   | Set fallback surface before navigation                 | Fail               |
| 7   | Diagnostic logging                                     | Research only      |
| 8   | Reduce dedup gate to 100ms                             | Fail               |

Key finding from Experiment 7's diagnostic logging: Chromium sends the new
`ca_context_id` within 100ms of the click. The page loads in ~70ms. The GUI
receives the ID and replaces the `CALayerHost` immediately. Yet the new
`CALayerHost` shows nothing for ~10 seconds.

All code changes from Issue 628 should be reverted — none had any effect.

### What we know

1. **Chromium is fast.** The new `ca_context_id` arrives in ~100ms. The page
   loads in ~70ms.
2. **The GUI is fast.** The `CALayerHost` is replaced immediately upon receiving
   the new ID.
3. **The blank is ~10 seconds.** Suspiciously consistent.
4. **The problem is NOT:** callback lifecycle, compositor surface fallback,
   dedup gate timing, NSWindow sizing, or `SetSize()` vs `setContentSize:`.
5. **The problem likely IS:** something in the Window Server's handling of
   cross-process CAContext/CALayerHost connections, or something about how the
   hidden NSWindow interacts with CAContext compositing.

### What we don't know

- Does the blank happen with a **visible** Chromium window? If not, the hidden
  window is the cause.
- Is the new `CAContext` actually producing content when the `CALayerHost`
  connects to it? Or is it empty?
- Does macOS have an internal timeout for establishing cross-process CAContext
  connections (`CARemoteLayerServer` / `CARemoteLayerClient`)?
- Does `CALayerHost` need an explicit trigger (e.g., `setNeedsDisplay`,
  `CATransaction`) to start displaying a newly-connected remote context?
- Is the old `CAContext` torn down before the new one is ready, creating a gap
  where no context has content?

### Chromium branch

Start from `146.0.7650.0-issue-627` (discarding Issue 628's branch). Create
`146.0.7650.0-issue-629` if any Chromium changes are needed for diagnostics.

### GUI revert

Revert `a73f3e1` (`gui/src/renderer/Metal.zig` — CALayerHost replacement logic
from Issue 628 Experiment 1) before starting experiments.
