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

## Experiments

### Experiment 1: Compare Electron and Chromium CALayerHost usage to TermSurf

#### Problem

We don't understand why the overlay goes blank for ~10 seconds during
navigation. Issue 628 spent 8 experiments modifying the Chromium pipeline with
no effect. Before trying more fixes, we need to understand how our CALayerHost
usage differs from the working implementations.

#### Research questions

**R1: Does Electron use CALayerHost at all?**

How does Electron's off-screen rendering display compositor output on macOS?
Does it create a `CALayerHost` with a `ca_context_id`, or does it use a
completely different mechanism?

**R2: How does normal Chromium use CALayerHost?**

In a standard Chrome window, how does `DisplayCALayerTree` manage the
`CALayerHost`? What is the layer tree structure? What happens during navigation
when the `ca_context_id` changes?

**R3: How does TermSurf use CALayerHost?**

Trace the full pipeline: Chromium Profile Server sends `ca_context_id` over XPC
→ GUI creates `CALayerHost`. What layer tree structure do we use? How does it
differ from Chromium's `DisplayCALayerTree`?

**R4: What are the architectural differences?**

Compare the three approaches side by side. Identify anything TermSurf does
differently that could explain the 10-second blank.

#### Results

**R1: Electron does NOT use CALayerHost.**

Electron's off-screen rendering on macOS intercepts `CALayerParams` at the
`HostDisplayClient` level and extracts the IOSurface directly — it never creates
a `CALayerHost`:

```cpp
// vendor/electron/shell/browser/osr/osr_host_display_client_mac.mm
void OffScreenHostDisplayClient::OnDisplayReceivedCALayerParams(
    const gfx::CALayerParams& ca_layer_params) {
  if (!ca_layer_params.is_empty) {
    IOSurfaceRef io_surface = IOSurfaceLookupFromMachPort(
        ca_layer_params.io_surface_mach_port.get());
    void* pixels = IOSurfaceGetBaseAddress(io_surface);
    SkBitmap bitmap;
    bitmap.installPixels(..., pixels, stride);
    callback_.Run(ca_layer_params.damage, bitmap, {});
  }
}
```

Electron reads pixels from the IOSurface Mach port on every frame. It has two
rendering paths:

1. **Hardware accelerated:** `FrameSinkVideoCapturer` (the same pipeline
   TermSurf used before Issue 625).
2. **Software:** `CALayerParams` → extract IOSurface → read pixels → SkBitmap.

Neither path involves `CALayerHost`. Electron sidesteps the entire
CAContext/CALayerHost mechanism. This means **Electron cannot tell us anything
about CALayerHost navigation behavior** — they don't use it.

**R2: Normal Chromium uses CALayerHost inside a visible window.**

In stock Chrome, `DisplayCALayerTree` (in the browser process) creates a
`CALayerHost` inside the window's NSView layer tree:

```
RenderWidgetHostViewCocoa (NSView, wantsLayer=YES)
└─ background_layer_ (CALayer, view's backing layer)
   └─ maybe_flipped_layer_ (CALayer, geometryFlipped=YES)
      └─ remote_layer_ (CALayerHost, contextId = ca_context_id)
```

Key details from `display_ca_layer_tree.mm`:

- `GotCALayerFrame()` creates a **new** `CALayerHost` when `ca_context_id`
  changes (never updates `contextId` on an existing host).
- Uses `ScopedCAActionDisabler` to suppress CALayer animations during the swap.
- Adds the new host **before** removing the old one — atomic visual swap.
- The NSView is in a **visible** window on screen.

When `SetCALayerParams()` is called on the NSView, it calls
`DisplayCALayerTree::UpdateCALayerTree()` which calls `GotCALayerFrame()`. This
happens inside `AcceleratedWidgetCALayerParamsUpdated()`.

**R3: TermSurf uses CALayerHost cross-process with a hidden intermediary.**

TermSurf's pipeline:

1. Chromium Profile Server runs with a **hidden** NSWindow
   (`[window orderOut:nil]`).
2. Inside that hidden window, the standard Chromium pipeline runs:
   `RenderWidgetHostViewCocoa` → `DisplayCALayerTree` → `CALayerHost`. This
   `CALayerHost` lives inside the hidden window and points at the GPU process's
   `CAContext`.
3. We hook `SetCALayerParamsCallback` on the `RenderWidgetHostViewMac` to
   intercept the `ca_context_id`.
4. We send the `ca_context_id` over XPC to the TermSurf GUI (a completely
   separate process).
5. The GUI creates its **own** `CALayerHost` in its Metal renderer's layer tree:

```
IOSurfaceLayer (Metal renderer)
└─ flipped_layer (geometryFlipped=YES)
   └─ positioning_layer (explicit frame at overlay grid rect)
      └─ CALayerHost (contextId = ca_context_id from XPC)
```

So there are **two CALayerHosts** pointing at the same `CAContext`:

1. One inside the Chromium Profile Server's hidden window (created by
   `DisplayCALayerTree`, standard Chromium behavior).
2. One inside the TermSurf GUI (created by `Metal.zig`).

**R4: Architectural differences.**

| Aspect               | Normal Chrome            | Electron OSR            | TermSurf                          |
| -------------------- | ------------------------ | ----------------------- | --------------------------------- |
| Mechanism            | CALayerHost              | IOSurface pixel read    | CALayerHost                       |
| CALayerHost location | Browser window (visible) | N/A                     | GUI process (visible)             |
| Intermediate window  | None                     | Placeholder NSView      | Hidden NSWindow                   |
| CALayerHost count    | 1 per CAContext          | 0                       | **2 per CAContext**               |
| Window visibility    | Visible                  | N/A                     | Hidden                            |
| Process topology     | GPU → Browser            | GPU → Browser (extract) | GPU → Server (hidden) → XPC → GUI |

Three critical differences in TermSurf:

1. **Two CALayerHosts per CAContext.** The hidden window's `DisplayCALayerTree`
   creates one, and our GUI creates another. Both point at the same `CAContext`.
   macOS may not support multiple `CALayerHost` instances for the same
   `CAContext`, or the hidden window's host may interfere with the GUI's host.

2. **Hidden intermediary window.** The Chromium Profile Server's NSWindow is
   hidden via `[window orderOut:nil]`. The `DisplayCALayerTree` inside that
   window still runs and manages its own `CALayerHost`. The Window Server may
   deprioritize or defer compositing for off-screen windows, affecting the
   `CAContext` that both hosts share.

3. **No `ScopedCAActionDisabler`.** Chromium wraps CALayerHost creation in
   `ScopedCAActionDisabler` to suppress Core Animation's implicit animations
   (fade-in, position interpolation). Our GUI code does not do this. A 0.25s
   fade-in animation wouldn't explain a 10-second blank, but other implicit
   animation behaviors might.

#### Conclusion

Electron is irrelevant — they don't use CALayerHost at all. The comparison that
matters is TermSurf vs. normal Chrome.

The most suspicious difference is **two CALayerHosts pointing at the same
CAContext**. In normal Chrome, there is exactly one `CALayerHost` per
`CAContext`. In TermSurf, the hidden window's `DisplayCALayerTree` creates one,
and the GUI creates a second. This is an untested configuration — macOS may not
properly handle it.

The next experiment should test whether eliminating the hidden window's
`CALayerHost` (by disabling `DisplayCALayerTree` or by not calling
`SetCALayerParams` on the hidden NSView) resolves the blank. Alternatively, test
whether making the hidden window visible fixes the problem — this would confirm
that window visibility affects `CAContext` compositing.

#### Verification

Research is complete when all four questions are answered and we have a concrete
hypothesis for the next experiment.
