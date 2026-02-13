# Issue 414: Two Profiles via XPC

## Goal

Two browser profiles rendering side by side in one window at an uncompromising
60fps, each profile running in its own process, communicating via XPC with
IOSurface Mach port transfer. This is the architecture that TermSurf will ship.

## Background

### The multi-profile problem

Issue 413 proved the core constraint: two `BrowserContext` instances in one
Chromium process drop rendering to 2fps (Experiment 4), while two `WebContents`
sharing one `BrowserContext` render at 60fps side by side (Experiment 6). The
boundary is clear — one profile per process.

### Multi-process rendering is proven

Three prior efforts proved that cross-process rendering via IOSurface Mach port
transfer works on macOS:

| Effort                           | Result                        | Bottleneck                                         |
| -------------------------------- | ----------------------------- | -------------------------------------------------- |
| **Issue 403** (Swift+Rust+C++)   | 60fps, <0.12ms composite time | None — architecture proven                         |
| **cef-test** (two CEF profiles)  | 50fps per profile             | CEF internal scheduling jitter (~15% vsync misses) |
| **ts3** (WezTerm + CEF profiles) | 38fps                         | Input pipeline + GUI rendering overhead            |

The cef-test result is the most relevant. Two independent CEF processes
rendering simultaneously achieved 50fps each with p50 = 16.7ms (exactly on
vsync). The ~10fps gap from 60fps is entirely due to CEF's
`do_message_loop_work()` jitter — not XPC, not IOSurface transfer, not
compositing. The Content API should eliminate this ceiling entirely.

### What we're building on

- **One Profile app** (Issue 412–413) — A Content Shell clone that renders at
  60fps. This becomes the basis for each profile server process.
- **cef-test** — Multi-process architecture with proven XPC protocol and
  IOSurface compositing. Port the frame delivery and compositing code, replace
  CEF with Content API, simplify bootstrap by eliminating the launcher.
- **termsurf-xpc** — Rust XPC bindings used by cef-test and ts3. Wraps
  `xpc_connection`, `xpc_dictionary`, Mach port transfer, IOSurface
  create/lookup.

## Architecture

```
Two Profiles GUI (Cocoa/Metal window + XPC Mach service)
├── Listens on com.termsurf.two-profiles
├── Spawns profile-a server → connects back to GUI
│   └── Left pane ◀── IOSurface Mach port ── Profile A server (Content API)
├── Spawns profile-b server → connects back to GUI
│   └── Right pane ◀── IOSurface Mach port ── Profile B server (Content API)
└── Composites both IOSurfaces into one window
```

Two process types (no launcher):

1. **GUI process** — Creates a single window with two Metal quads. Registers as
   a named XPC Mach service (`com.termsurf.two-profiles`). Spawns profile server
   processes as children, passing the service name and a session ID as CLI args.
   Receives IOSurface Mach ports from both profile servers via XPC. Imports each
   as a Metal texture and composites them side by side. No browser code runs
   here.

2. **Profile server process** (one per profile) — Runs the Content API with a
   single `BrowserContext`. Navigates a `WebContents` to the test page. Captures
   the composited output as an IOSurface. Connects to the GUI's Mach service by
   name and sends IOSurface Mach ports every frame.

### Why no launcher?

In cef-test and ts3, a separate launcher process acted as a middleman: the GUI
sent an anonymous XPC endpoint to the launcher, the launcher stored it, and the
profile server claimed it. This relay was necessary because XPC endpoints can
only be transferred over existing XPC connections — two processes with no shared
channel have no way to exchange endpoints.

The launcher solved the bootstrap problem, but it was a third process (~220
lines) that existed solely to relay one message per profile. A simpler
alternative: **make the GUI itself the named Mach service.** The GUI registers a
hard-coded service name (e.g., `com.termsurf.two-profiles`) via a launchd plist.
Profile servers receive this name as a CLI argument and connect directly. No
endpoint relay, no session claiming, no middleman.

The connection is bidirectional — once established, the profile server sends
IOSurface frames to the GUI, and the GUI sends input events (keyboard, mouse,
resize) back to the profile server over the same connection. This is the same
communication pattern as cef-test, just with one fewer process.

If the GUI-as-service approach hits problems (e.g., multiple GUI instances
conflicting on the service name), we can fall back to the launcher pattern. For
the PoC with a single window, the simpler approach should work.

### XPC protocol

**Bootstrap (simplified from cef-test):**

1. GUI registers as Mach service `com.termsurf.two-profiles` (via launchd plist)
2. GUI spawns profile-a server with args:
   `--service com.termsurf.two-profiles
   --session-id profile-a --profile profile-a --url <url>`
3. Profile-a server connects to `com.termsurf.two-profiles` by name
4. Profile-a server sends `register` message with its session ID
5. GUI maps the connection to the left pane
6. Repeat for profile-b (right pane)

No anonymous listeners, no endpoint relay, no claim handshake. Each profile
server connects directly to the GUI.

**Frame delivery (fast path, every frame):**

```
Profile server → GUI:
{
  action: "display_surface",
  iosurface_port: <mach_port_t>,  // set_mach_send()
  width: i64,                      // physical pixels
  height: i64,                     // physical pixels
}
```

**Input forwarding (GUI → profile server, same connection):**

```
GUI → Profile server:
{
  action: "key_event" | "mouse_click" | "mouse_move" | "resize" | ...,
  ... event-specific fields ...
}
```

**GUI import pipeline:**

1. `copy_mach_send("iosurface_port")` — extract Mach port from XPC message
2. `IOSurfaceLookupFromMachPort(port)` — reconstruct IOSurface in GUI process
3. Import as Metal texture
4. Composite into window
5. `mach_port_deallocate(port)` — release kernel resource

## Prior art: what to reuse

### From cef-test

cef-test used a three-process architecture (GUI, launcher, profile server) where
the launcher relayed XPC endpoints between the GUI and profile servers. We're
simplifying to two process types (GUI + profile servers) by making the GUI the
named Mach service, but the frame delivery and compositing code is directly
reusable:

- **Frame delivery protocol:** `display_surface` message with `iosurface_port`.
  One message per frame, ~100 bytes + Mach port. Identical to what we need.
- **GUI compositing:** wgpu render pipeline with two quads (left/right),
  IOSurface import via `IOSurfaceLookupFromMachPort`, sRGB texture views.
- **Background dispatch queue for XPC callbacks:** Critical discovery — XPC
  handlers must dispatch on a background queue, not the main queue, to avoid
  conflicts with the GUI event loop.
- **Benchmark harness:** 60-second automated run with frame interval statistics
  (avg fps, % at 60fps, p50/p95/p99, max consecutive streak).

### From termsurf-xpc

Reference implementation for the XPC patterns we need. The Rust code won't be
reused directly, but the patterns translate 1:1 to Apple's C API
(`<xpc/xpc.h>`):

- **Connection management:** `xpc_connection_create_mach_service()` for named
  services, `xpc_connection_set_event_handler()` for message dispatch.
- **Mach port transfer:** `xpc_dictionary_set_mach_send()` (sender) /
  `xpc_dictionary_copy_mach_send()` (receiver).
- **IOSurface sharing:** `IOSurfaceCreateMachPort()` (sender) /
  `IOSurfaceLookupFromMachPort()` (receiver) / `mach_port_deallocate()`
  (cleanup).

### From the One Profile app

- **Content API embedder:** Complete, buildable, 60fps Content Shell clone. This
  becomes the profile server with the addition of IOSurface capture and XPC
  frame delivery.
- **Profile path management:** `SHELL_DIR_USER_DATA` override for isolated
  profile storage. Each profile server process overrides to its own path.

## Language choice for the PoC

C++ for everything. Both the GUI and profile server are C++/Objective-C++.

- **Profile server:** C++. Links against Chromium. XPC calls use Apple's C API
  directly (`<xpc/xpc.h>`). Modified from the One Profile app.
- **GUI:** C++/Objective-C++. Metal rendering via Objective-C++
  (`<Metal/Metal.h>`, `<QuartzCore/QuartzCore.h>`). XPC Mach service
  registration via Apple's C API. IOSurface import via
  `<IOSurface/IOSurface.h>`.

This keeps the entire PoC in one language, avoids cross-language build
complexity, and matches Chromium's own codebase. The cef-test Rust code and
termsurf-xpc crate are useful as reference for the XPC protocol and IOSurface
transfer patterns, but the implementation will be native C++.

## How Electron captures GPU textures

Electron's off-screen rendering (OSR) solves the same problem we need to solve:
capture the composited output of a `WebContents` as a GPU texture, without
displaying it in a window. Studying Electron's approach reveals that Chromium
already has a built-in API for this.

### Two capture paths (GPU vs. software)

Electron has two capture paths, selected by whether GPU acceleration is enabled:

**GPU-accelerated (FrameSinkVideoCapturer):** When
`HardwareAccelerationEnabled()` returns true (the normal case), Electron creates
an `OffScreenVideoConsumer` backed by `ClientFrameSinkVideoCapturer`. This is
Chromium's built-in video capture API — the same mechanism Chrome uses for tab
capture, WebRTC screen sharing, and remote display. It issues
`CopyOutputRequest`s at the compositor level and delivers frames as
`GpuMemoryBufferHandle`s. On macOS, these handles are IOSurfaces.

**Software rasterization (HostDisplayClient):** When GPU acceleration is
disabled, Electron falls back to `OffScreenHostDisplayClient`. On macOS, this
receives `OnDisplayReceivedCALayerParams()` callbacks from Chromium's
compositor, which include `io_surface_mach_port`. This is the older, legacy
path.

The selection logic is straightforward (`osr_render_widget_host_view.cc`):

```cpp
if (content::GpuDataManager::GetInstance()->HardwareAccelerationEnabled()) {
  video_consumer_ = std::make_unique<OffScreenVideoConsumer>(...);
  video_consumer_->SetActive(is_painting());
} else {
  // Falls through to HostDisplayClient path
}
```

Only one path is active at a time. They are never used simultaneously.

### FrameSinkVideoCapturer: the GPU-accelerated path

This is the path that matters for TermSurf. GPU acceleration is not optional —
we need it for 60fps rendering.

How it works:

1. `CreateVideoCapturer()` on the `RenderWidgetHostView` creates a
   `ClientFrameSinkVideoCapturer` (host side) linked to a
   `FrameSinkVideoCapturerImpl` (renderer side)
2. Chromium's viz layer monitors frame damage and issues `CopyOutputRequest`s
3. Frames arrive in `OnFrameCaptured()` as `GpuMemoryBufferHandle`s
4. On macOS, the handle contains an IOSurface pointer
   (`OffscreenSharedTextureValue.shared_texture_handle`)
5. A buffer pool of 10 pre-allocated GPU textures eliminates per-frame
   allocation

Key properties:

- **Supported API.** Designed for continuous frame capture, not a hook into
  compositor internals.
- **Buffer pooling.** 10-frame ring buffer (`kFramePoolCapacity = 10`), no
  allocation per frame.
- **Frame rate control.** Built-in `SetMinCapturePeriod()`.
- **Damage tracking.** Only dirty regions flagged via `content_rect`.
- **Cross-platform.** IOSurface on macOS, D3D11 on Windows, DMA-BUF on Linux.

The tradeoff is that `CopyOutputRequest` involves a GPU-to-GPU copy — not true
zero-copy. But it's a GPU-side copy, fast enough for Chrome's real-time tab
capture.

### The `useSharedTexture` option

Within the FrameSinkVideoCapturer path, a separate `useSharedTexture` preference
controls the capture format:

- `true` → GPU shared texture (IOSurface on macOS). This is what we want.
- `false` → Shared memory bitmap (CPU-accessible pixels).

This preference does NOT select between the two capture paths — it only controls
the buffer format within the GPU-accelerated path.

### Why the CALayerParams path is irrelevant

The `OffScreenHostDisplayClient` / `OnDisplayReceivedCALayerParams()` path on
macOS is only active when GPU acceleration is disabled. Since TermSurf requires
GPU acceleration for 60fps rendering, this path is irrelevant to us. Early
research (before studying Electron's source) considered intercepting at
`DisplayCALayerTree::UpdateCALayerTree()` to grab
`CALayerParams.io_surface_mach_port`, but this is the wrong approach — it's the
software fallback, not the GPU-accelerated path.

### What this means for TermSurf

The profile server should use `FrameSinkVideoCapturer` to capture composited
frames as IOSurfaces, then create Mach ports from those IOSurfaces and send them
to the GUI via XPC. This is exactly what Electron does for its `paint` event
with `useSharedTexture = true`, except instead of delivering the texture to
JavaScript, we deliver the Mach port to a separate GUI process.

Key reference files:

- `electron/shell/browser/osr/osr_video_consumer.{h,cc}` — capture logic
- `electron/shell/browser/osr/osr_render_widget_host_view.{h,cc}` — OSR widget
- `electron/shell/browser/osr/osr_paint_event.h` — frame data structures
- `electron/shell/browser/osr/osr_host_display_client_mac.mm` — legacy macOS
  path (irrelevant but useful as reference)

## Ideas for Experiments

### Idea 1: FrameSinkVideoCapturer (Electron's primary path)

**Goal:** Capture composited frames as IOSurfaces at 60fps using Chromium's
built-in video capture API.

Electron's off-screen rendering uses `ClientFrameSinkVideoCapturer`, which
implements `viz::mojom::FrameSinkVideoConsumer`. This is a Chromium API designed
for exactly this use case — capturing compositor output for headless rendering,
screen sharing, and remote display. Chrome uses it for tab capture and WebRTC.

How it works:

1. Call `CreateVideoCapturer()` on the `RenderWidgetHostView`
2. Chromium's viz layer issues `CopyOutputRequest`s at the compositor level
3. Frames arrive in `OnFrameCaptured()` as `GpuMemoryBufferHandle`s — which on
   macOS are IOSurfaces
4. A buffer pool of 10 pre-allocated GPU textures eliminates per-frame
   allocation

Advantages:

- **Supported API.** Designed for continuous capture, not a hook into internals.
- **Buffer pooling.** 10-frame ring buffer, no allocation per frame.
- **Frame rate control.** Built-in `SetMinCapturePeriod()`.
- **Damage tracking.** Only dirty regions flagged.
- **Cross-platform.** IOSurface on macOS, D3D11 on Windows, DMA-BUF on Linux.

Tradeoff: involves a `CopyOutputRequest` (GPU-to-GPU copy), so not true
zero-copy. But it's a GPU-side copy — fast enough for Chrome's real-time tab
capture at 60fps.

Reference: `electron/shell/browser/osr/osr_video_consumer.{h,cc}`.

### Idea 2: Single profile server with XPC frame delivery

**Goal:** Prove IOSurface Mach port transfer from a Content API process to a
separate GUI process works at 60fps.

Two components:

1. **GUI** (C++/ObjC++) — registers as Mach service `com.termsurf.two-profiles`,
   spawns profile server, receives Mach ports, imports as Metal textures,
   renders to window
2. **Profile server** (modified One Profile app, C++) — captures frames as
   IOSurfaces, connects to GUI's Mach service, sends Mach ports via XPC

This proves the full pipeline: Content API → IOSurface → Mach port → XPC → GPU
texture → window. If this hits 60fps, the architecture is validated.

### Idea 3: Two profile servers, one window

**Goal:** Two profiles, two processes, one window, both at 60fps.

Run two profile server instances (profile-a and profile-b) with the GUI
displaying both side by side. This is the target architecture — identical to
cef-test but with Content API instead of CEF.

Success criteria: both panes rendering the spinning blue square at 60fps with
different localStorage identities (proving profile isolation).

### Idea 4: Stress test and benchmarking

**Goal:** Sustained 60fps under load, matching or exceeding cef-test's 50fps.

Run the two-profile setup for 60+ seconds with continuous animation. Measure:

- Average FPS per profile
- Percentage of frames at 60fps (within one vsync interval)
- p50, p95, p99 frame intervals
- CPU usage (must not be 100%)
- Max consecutive frames at 60fps

Compare against cef-test's benchmark (50fps, 80.8% at 60fps, p50=16.7ms,
p95=33.6ms). The Content API should beat these numbers since CEF's internal
scheduling jitter was the bottleneck.

## Success criteria

- Two panes in one window, each showing the spinning blue square
- Different localStorage identity in each pane (profile isolation)
- Both at 60fps sustained for 60+ seconds
- CPU usage well below 100% (no busy-wait loops)
- IOSurface transfer via XPC (not shared memory, not window capture)

## What this unlocks

Once this PoC works, the path to TermSurf is clear:

1. **Ghostty integration:** Replace the Rust/Swift GUI with Ghostty's Metal
   renderer. Ghostty composites IOSurfaces from profile servers alongside
   terminal panes.
2. **Input forwarding:** GUI sends keyboard and mouse events to profile servers
   via XPC (reverse direction of the frame pipeline).
3. **Process lifecycle:** Ghostty manages profile server processes. Multiple
   `web` commands for the same profile reuse the existing process.
4. **Multiple WebContents per profile:** Each profile server handles multiple
   WebContents (tabs). Issue 413 Experiment 6 proved this works at 60fps.

## Experiments

### Experiment 1: Intercept the compositor's IOSurface

#### Hypothesis

Chromium's macOS compositor already produces IOSurfaces as part of its normal
rendering pipeline. We don't need `CopyFromSurface()` or any capture mechanism —
we just need to intercept the existing IOSurface at the right point. If we can
grab it at 60fps, the entire frame delivery pipeline (IOSurface → Mach port →
XPC → GUI) is a solved problem.

#### Background: how Chromium presents frames on macOS

Research into Chromium's macOS rendering pipeline reveals a clean architecture:

```
GPU process (or GPU thread if in-process)
  ↓ Compositor produces frames
CALayerTreeCoordinator
  ↓ GetContentIOSurface() → raw IOSurface
  ↓ IOSurfaceCreateMachPort() → Mach port
  ↓ Packages into CALayerParams { ca_context_id OR io_surface_mach_port }
Browser process
  ↓ DisplayCALayerTree::UpdateCALayerTree(ca_layer_params)
  ↓ IOSurfaceLookupFromMachPort() → IOSurface
  ↓ Sets CALayer.contents = io_surface
CoreAnimation
  ↓ Presents to screen
```

Two rendering modes exist:

1. **Remote layers (`ca_context_id`):** The GPU process creates a `CAContext`
   with a layer tree. The browser process creates a `CALayerHost` with the
   context ID. WindowServer composites remotely. Used when GPU is in-process.

2. **IOSurface Mach port:** The GPU process extracts the IOSurface from the
   layer tree via `GetContentIOSurface()`, creates a Mach port via
   `IOSurfaceCreateMachPort()`, and sends it in `CALayerParams`. The browser
   process imports it via `IOSurfaceLookupFromMachPort()`. Used when GPU is
   out-of-process.

The key files:

- `ui/accelerated_widget_mac/ca_layer_tree_coordinator.{h,mm}` — GPU-side.
  Manages the layer tree, calls `GetContentIOSurface()`, creates Mach ports.
- `ui/accelerated_widget_mac/ca_renderer_layer_tree.{h,mm}` — Builds the
  hierarchy of CALayers from compositor output. Each `ContentLayer` holds an
  `io_surface_` member.
- `ui/accelerated_widget_mac/display_ca_layer_tree.{h,mm}` — Browser-side.
  Receives `CALayerParams`, imports IOSurfaces, assigns to CALayers.
- `ui/gfx/ca_layer_params.h` — The IPC structure: `ca_context_id` (uint32) or
  `io_surface_mach_port` (scoped Mach port), plus `pixel_size` and
  `scale_factor`.

The critical insight: **Chromium already creates IOSurface Mach ports.** We
don't need to invent a capture mechanism. We need to intercept what Chromium
already produces.

#### Design

Modify the One Profile app to intercept frames at the
`DisplayCALayerTree::UpdateCALayerTree()` call site and log what arrives.

##### Step 1: Determine which rendering path content_shell uses

Add logging to `DisplayCALayerTree::UpdateCALayerTree()` (or its equivalent in
the One Profile app's rendering path) to check which `CALayerParams` field is
populated:

- If `ca_layer_params.ca_context_id != 0` → remote layer path (in-process GPU)
- If `ca_layer_params.io_surface_mach_port` → IOSurface Mach port path

This tells us which intercept strategy to use.

##### Step 2: Grab the IOSurface

- **If IOSurface Mach port path:** Call
  `IOSurfaceLookupFromMachPort(ca_layer_params.io_surface_mach_port.get())` to
  get the IOSurface. Log its dimensions (`IOSurfaceGetWidth`,
  `IOSurfaceGetHeight`). Increment a frame counter.

- **If remote layer path:** We need to either:
  - (a) Force the IOSurface Mach port path by disabling remote layers (e.g.,
    `--disable-gpu-compositing` or similar flag), or
  - (b) Intercept earlier at `CALayerTreeCoordinator` where
    `GetContentIOSurface()` is called before the CA context ID path is chosen,
    or
  - (c) Intercept at the `CARendererLayerTree::ContentLayer` level where each
    layer holds an `io_surface_` member directly.

##### Step 3: Measure the rate

Add a frame counter and a periodic log (once per second) that reports:

- Frames in the last second
- IOSurface dimensions
- Which rendering path was used

#### What we're modifying

The One Profile app in `content/one_profile/`. The exact file depends on which
intercept point works best:

- **Minimal change:** Add logging to `display_ca_layer_tree.mm` in the
  `ui/accelerated_widget_mac/` directory. This is Chromium code (not One Profile
  code), so the change is a Chromium fork modification.
- **Cleaner change:** Override the relevant method in the One Profile app's
  `ShellBrowserMainParts` or platform delegate, if the rendering path is
  hookable from the embedder level.

The experiment will reveal which approach is practical.

#### Expected result

60 frames per second with IOSurface dimensions matching the window size (e.g.,
1600×1200 physical pixels for an 800×600 logical window at 2x Retina). This
proves that we can intercept Chromium's compositor output at full framerate
without any additional capture mechanism.

#### What a failure would mean

- **0 frames (wrong path):** Content_shell uses the remote layer path and we
  can't easily intercept IOSurfaces. Fix: force the IOSurface path or intercept
  earlier in the pipeline.
- **< 60fps:** The intercept itself is adding overhead (unlikely for a simple
  log). Investigate.
- **IOSurface is null or wrong size:** The compositor isn't producing
  single-surface frames (multi-layer tree). Need to understand the layer
  structure and potentially composite multiple IOSurfaces ourselves.
