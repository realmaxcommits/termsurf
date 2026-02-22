# Issue 625: CALayerHost

## Goal

Replace the `FrameSinkVideoCapturer` with `CALayerHost` so that browser panes
display with the same latency as native Chrome — zero per-frame IPC, zero
application-side compositing, Window Server composites directly from GPU VRAM.

## Background

### The current pipeline

TermSurf runs Chromium out-of-process. A Chromium Profile Server renders web
content and streams frames to the GUI (a Ghostty fork) over XPC. The current
frame delivery path:

```
Chromium renders → compositor composites → FrameSinkVideoCapturer (timer) →
CopyOutputRequest → IOSurface → Mach port via XPC → GUI imports IOSurface →
Metal shader composites → CVDisplayLink vsync → screen
```

This adds 15–25ms of latency versus native Chrome
([Issue 619](619-input-latency.md)). The single biggest contributor is the
`FrameSinkVideoCapturer` — a recording API that runs on its own timer, adding
~5-7ms per frame from timer wait and GPU readback. On top of that, every frame
requires a Mach port transfer over XPC.

### What we learned in Issue 624

[Issue 624](624-chromium-ipc.md) mapped Chromium's full IPC architecture across
three experiments:

1. **Chrome's normal display path uses `CALayerHost`.** The GPU process creates
   a `CAContext`, sends a `ca_context_id` (uint32) once, and the browser process
   creates a `CALayerHost` pointing to that ID. Window Server composites the GPU
   process's CALayer tree directly from VRAM. Zero per-frame IPC, zero pixel
   copies.

2. **Our Chromium Profile Server already produces `CALayerParams` every frame.**
   The capturer is purely observational — the normal display path runs alongside
   it. We've been ignoring `CALayerParams` output at
   `RenderWidgetHostViewMac::AcceleratedWidgetCALayerParamsUpdated()`.

3. **Electron validates this approach.** Electron's normal `BrowserWindow` uses
   stock Chromium — CALayerHost, unmodified, zero custom display code.
   Electron's off-screen rendering mode uses the same `FrameSinkVideoCapturer`
   that TermSurf currently uses, with the same latency penalty. CALayerHost is
   the architecturally correct way to display Chromium content.

### Why CALayerHost

The `ca_context_id` is a uint32 that identifies a `CAContext` in the GPU
process. `CALayerHost` is a `CALayer` subclass that displays a remote
`CAContext` from another process — Window Server handles the compositing. This
is the same mechanism Chrome uses between its own GPU process and browser
process. Adding one more process boundary (Chromium server → TermSurf GUI) is
the same pattern.

**What it eliminates:**

- `FrameSinkVideoCapturer` and `ShellVideoConsumer` (~460 lines)
- Per-frame `CopyOutputRequest` GPU readback
- Per-frame IOSurface Mach port transfer over XPC
- Per-frame Metal texture import and shader compositing in the GUI
- The ~5-7ms capturer timer latency

**What it adds:**

- One XPC message per tab containing a uint32 `ca_context_id` (sent once, not
  per frame)
- A `CALayerHost` sublayer in the GUI, positioned at browser pane coordinates
- Dimming of inactive browser panes via a sibling `CALayer` with
  semi-transparent background

**Architectural change:** Browser pane content moves out of the Metal shader
pipeline. Terminal panes render via Metal (unchanged). Browser panes render via
`CALayerHost` — Window Server composites them. Both coexist as sibling
`CALayer`s in the same NSView layer tree. The GUI still controls positioning and
z-order, but does not touch browser pixels.

### What needs to change

Two sides:

**Chromium Profile Server** (in `chromium/src/`):

- Intercept `CALayerParams` at
  `RenderWidgetHostViewMac::AcceleratedWidgetCALayerParamsUpdated()`
- Extract `ca_context_id` from the params
- Send it over XPC to the GUI (once per tab, re-send on context change)
- Remove `ShellVideoConsumer` and all capturer setup

**TermSurf GUI** (in `gui/`):

- Receive `ca_context_id` over XPC
- Create `CALayerHost` with that `contextId`
- Add as sublayer of the window's content view, positioned at the browser pane's
  pixel coordinates
- Update position/size on pane resize and split changes
- Add dimming overlay `CALayer` for inactive browser panes
- Remove the IOSurface overlay pipeline from the Metal renderer (the pink
  texture proof-of-concept from [Issue 602](602-pink-texture.md), the IOSurface
  import from [Issue 603](603-box-demo.md))

### Open questions

Before implementing, we need to research the TermSurf codebase to understand:

1. **Where does the GUI currently receive IOSurface frames?** Trace the XPC
   message path from reception to Metal rendering. What code handles the Mach
   port, imports the IOSurface, and passes it to the renderer?

2. **How does the Metal renderer currently composite browser content?** What
   shaders, pipeline state, and draw calls are involved? What gets deleted?

3. **Where in the Zig/Swift layer hierarchy should `CALayerHost` live?** The GUI
   uses a `CAMetalLayer` for terminal rendering. `CALayerHost` needs to be a
   sibling or sublayer. What's the current NSView/CALayer structure?

4. **How does pane positioning work?** When a split pane resizes, how do pixel
   coordinates propagate? The `CALayerHost` needs to track these coordinates.

5. **Can `CALayerHost` be created from Zig?** Zig already calls Objective-C
   runtime APIs for Metal. `CALayerHost` is another `CALayer` subclass — same
   pattern. But we need to verify the specific API calls.
