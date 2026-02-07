# Issue 344: cef-test — Minimal Multi-Process CEF Test Harness

## Goal

Build a minimal, standalone test application that loads two CEF browsers side by
side in a single window, each running in a separate process with a separate
profile, communicating via XPC Mach port transfer. No terminal emulator, no
WezTerm, no pane management — just the core product requirement stripped to its
essence.

This isolates the single architectural variable that separates the working
cef-rs OSR example (60fps) from the struggling ts3 profile server (38fps):
**multi- process CEF with cross-process IOSurface sharing.**

## Why This Matters

Eight experiments in [Issue 343](./343-optimal-performance.md) failed to improve
the profile server's frame rate. The problem is clear (`do_message_loop_work()`
takes >1ms on 100% of calls in the headless profile server vs 5.7% in the
windowed cef-rs example), but the cause is buried under layers of ts3
complexity: WezTerm's event loop, the launcher lifecycle, terminal multiplexing,
pane management, the web command flow, and more.

cef-test eliminates all of that. If the performance problem reproduces in
cef-test, the root cause is inherent to the multi-process/headless architecture
and we can iterate here 10x faster. If it doesn't, the root cause is in ts3's
integration and we know where to look.

## Architecture Overview

### Process Topology

```
cef-test-gui (single window, wgpu rendering)
    │
    ├── Connects to cef-test-launcher (Mach service bootstrap)
    │
    ├── Requests profile "left" → launcher spawns cef-test-profile
    │   │
    │   └── Profile "left" ←─ XPC direct ──→ GUI
    │       (headless CEF, github.com)     (receives Mach ports)
    │
    └── Requests profile "right" → launcher spawns cef-test-profile
        │
        └── Profile "right" ←─ XPC direct ──→ GUI
            (headless CEF, google.com)     (receives Mach ports)
```

### Data Flow

```
Profile Server (headless CEF)                GUI (windowed)
─────────────────────────────                ──────────────
CEF renders to IOSurface
    │
    ▼
on_accelerated_paint callback
    │
    ▼
IOSurfaceCreateMachPort(handle)
    │
    ▼
XPC send: {                          ──▶    XPC receive
  action: "display_surface",                    │
  iosurface_port: <mach_port>,                  ▼
  width, height                          IOSurfaceLookupFromMachPort(port)
}                                               │
                                                ▼
                                         Metal: newTexture(iosurface:)
                                                │
                                                ▼
                                         wgpu bind group + render pass
                                                │
                                                ▼
                                         Draw to left or right half
                                                │
                                                ▼
                                         surface.present()


GUI (windowed)                           Profile Server
──────────────                           ──────────────
winit captures mouse/key event
    │
    ▼
Determine target (left/right)
based on cursor position
    │
    ▼
Translate coordinates to                 XPC receive
local profile space              ──▶         │
    │                                        ▼
XPC send: {                          CEF host.send_mouse_move_event()
  action: "mouse_move",             CEF host.send_key_event()
  x, y, modifiers                   etc.
}
```

### Why a Launcher is Required

XPC endpoints are opaque kernel objects that can only be transferred over
existing XPC connections. To establish the first connection between the GUI and
a profile server, both processes need a shared bootstrap point. A named Mach
service (registered with launchd) serves this role:

1. GUI connects to the launcher's named service
2. GUI creates an anonymous XPC listener, sends the endpoint to the launcher
3. Launcher spawns a profile server process
4. Profile server connects to the launcher, claims the endpoint
5. Profile server connects directly to the GUI via the endpoint
6. All further communication is direct GUI ↔ Profile (launcher not involved)

This is identical to ts3's pattern, proven to work. The launcher itself is ~150
lines — trivial plumbing, not complexity.

## Binaries

### cef-test-gui

The windowed process. Creates a single window, renders two browser textures side
by side, captures input, routes it to the correct profile server.

**Responsibilities:**

- Create a winit window (1600x800 logical, 3200x1600 physical on Retina)
- Initialize wgpu with Metal backend
- Connect to the launcher Mach service
- For each browser slot (left, right):
  - Create an anonymous XPC listener
  - Send the listener's endpoint + metadata to the launcher
  - Receive the profile server's direct connection
  - Receive IOSurface Mach ports from the profile server
  - Import IOSurface → Metal texture → wgpu texture → bind group
- Run the event loop:
  - `pump_app_events` (winit) — process window events
  - On `RedrawRequested`: render both textures to their respective halves
  - On mouse/key events: route to the correct profile server via XPC
- Log per-frame timing for performance measurement

**Does NOT run CEF.** No `do_message_loop_work()`, no CEF initialization. The
GUI is purely a renderer and input dispatcher.

### cef-test-profile

The headless CEF process. One instance per browser profile. Renders web pages
off-screen and sends IOSurface Mach ports to the GUI.

**Responsibilities:**

- Parse CLI args (session-id, url, profile, width, height, scale)
- Load CEF framework, run subprocess check
- Connect to launcher, claim the GUI endpoint for its session
- Connect directly to the GUI via the endpoint
- Initialize CEF with:
  - `windowless_rendering_enabled: true`
  - `shared_texture_enabled: true`
  - `root_cache_path: ~/.config/cef-test/{profile}/`
  - `windowless_frame_rate: 60`
- Create a render handler that:
  - On `on_accelerated_paint`: create Mach port from IOSurface, send to GUI
  - On `view_rect`: return stored width/height
  - On `screen_info`: return device_scale_factor
- Receive input events from GUI via XPC:
  - `mouse_move`, `mouse_click`, `mouse_wheel` → forward to CEF browser host
  - `key_event` → forward to CEF browser host
  - `resize` → update browser size
  - `focus` → set/kill browser focus
- Run the message loop: `do_message_loop_work()` + `cfrunloop::run_for(0.001)`
- Log `[FRAME-TX]` timing for performance measurement

**Matches ts3's profile server** in message loop structure and CEF
configuration. This is deliberate — we want to reproduce the same performance
characteristics so we can experiment from there.

### cef-test-launcher

The bootstrap service. Forwards XPC endpoints between the GUI and profile
servers. Exits when the GUI disconnects.

**Responsibilities:**

- Register as Mach service `com.cef-test.launcher`
- Handle `spawn_profile`:
  - Store GUI endpoint by session-id
  - Spawn `cef-test-profile` with CLI args
- Handle `claim_session`:
  - Look up and return stored GUI endpoint
- Handle `register_profile`:
  - Store profile connection for reuse (same profile, second browser)
- Exit when GUI connection closes

This is a simplified version of ts3's `termsurf-launcher` (~150 lines). The
simplifications:

- No multi-GUI support (single GUI connection)
- No crash recovery
- No log redirection complexity

## XPC Protocol

### Bootstrap Flow

```
GUI                          Launcher                     Profile
 │                              │                            │
 │── connect ──────────────────▶│                            │
 │                              │                            │
 │── spawn_profile ────────────▶│                            │
 │   {session_id, url,          │                            │
 │    profile, width, height,   │                            │
 │    scale, gui_endpoint}      │── spawn process ──────────▶│
 │                              │                            │
 │                              │◀────── connect ────────────│
 │                              │                            │
 │                              │◀── claim_session ──────────│
 │                              │   {session_id}             │
 │                              │                            │
 │                              │── reply ──────────────────▶│
 │                              │   {endpoint}               │
 │                              │                            │
 │◀───────── XPC direct connection (via endpoint) ──────────▶│
 │                              │                            │
 │◀── display_surface ──────────────────────────────────────│
 │   {iosurface_port, w, h}                                  │
 │                                                           │
 │── mouse_move ────────────────────────────────────────────▶│
 │   {x, y, modifiers}                                      │
```

### Messages: Profile → GUI

**display_surface** — sent on every CEF frame

| Field            | Type      | Description                         |
| ---------------- | --------- | ----------------------------------- |
| `action`         | string    | `"display_surface"`                 |
| `iosurface_port` | mach_send | IOSurface Mach port (set_mach_send) |
| `width`          | i64       | Physical pixel width                |
| `height`         | i64       | Physical pixel height               |

### Messages: GUI → Profile

**mouse_move**

| Field       | Type   | Description                     |
| ----------- | ------ | ------------------------------- |
| `action`    | string | `"mouse_move"`                  |
| `x`         | i64    | Logical x (relative to profile) |
| `y`         | i64    | Logical y (relative to profile) |
| `modifiers` | i64    | CEF modifier flags              |

**mouse_click**

| Field         | Type   | Description                        |
| ------------- | ------ | ---------------------------------- |
| `action`      | string | `"mouse_click"`                    |
| `x`           | i64    | Logical x (relative to profile)    |
| `y`           | i64    | Logical y (relative to profile)    |
| `button`      | i64    | 0=left, 1=middle, 2=right          |
| `is_up`       | i64    | 1 if button released, 0 if pressed |
| `click_count` | i64    | 1 for single, 2 for double-click   |
| `modifiers`   | i64    | CEF modifier flags                 |

**mouse_wheel**

| Field       | Type   | Description        |
| ----------- | ------ | ------------------ |
| `action`    | string | `"mouse_wheel"`    |
| `x`         | i64    | Logical cursor x   |
| `y`         | i64    | Logical cursor y   |
| `delta_x`   | i64    | Horizontal scroll  |
| `delta_y`   | i64    | Vertical scroll    |
| `modifiers` | i64    | CEF modifier flags |

**key_event**

| Field         | Type   | Description                |
| ------------- | ------ | -------------------------- |
| `action`      | string | `"key_event"`              |
| `key_is_down` | i64    | 1 for keydown, 0 for keyup |
| `key_type`    | i64    | CEF key event type         |
| `raw_code`    | i64    | Native macOS key code      |
| `char_code`   | i64    | Unicode character          |
| `shift`       | i64    | Shift modifier             |
| `ctrl`        | i64    | Control modifier           |
| `alt`         | i64    | Alt/Option modifier        |
| `meta`        | i64    | Command modifier           |

**resize**

| Field    | Type   | Description         |
| -------- | ------ | ------------------- |
| `action` | string | `"resize"`          |
| `width`  | i64    | Logical width       |
| `height` | i64    | Logical height      |
| `scale`  | string | Device scale factor |

**focus**

| Field     | Type   | Description             |
| --------- | ------ | ----------------------- |
| `action`  | string | `"focus"`               |
| `focused` | i64    | 1 for focus, 0 for blur |

## Window Layout & Rendering

### Layout

Single window, 1600x800 logical pixels (3200x1600 physical on Retina):

```
┌──────────────────────┬──────────────────────┐
│                      │                      │
│    Profile "left"    │   Profile "right"    │
│    (github.com)      │   (google.com)       │
│                      │                      │
│    800 x 800 logical │   800 x 800 logical  │
│                      │                      │
│                      │                      │
└──────────────────────┴──────────────────────┘
                1600 x 800 logical
```

Each profile server sees an 800x800 logical viewport (1600x1600 physical on
Retina). The GUI receives two independent IOSurface textures and composites them
side by side.

### Rendering Pipeline

Two draw calls per frame, one per browser. Each draw call renders a fullscreen
quad mapped to half the window:

**Left quad vertices (NDC):**

```
(-1, +1) → (0, 0)    // top-left of window
( 0, +1) → (1, 0)    // top-center of window
(-1, -1) → (0, 1)    // bottom-left of window
( 0, -1) → (1, 1)    // bottom-center of window
```

**Right quad vertices (NDC):**

```
( 0, +1) → (0, 0)    // top-center of window
(+1, +1) → (1, 0)    // top-right of window
( 0, -1) → (0, 1)    // bottom-center of window
(+1, -1) → (1, 1)    // bottom-right of window
```

Same shader, same pipeline, same sampler. Different vertex buffer and different
bind group (different texture) per draw call. This matches the cef-rs OSR
example's rendering approach — a pass-through fragment shader sampling from the
CEF texture.

### wgpu Setup

Identical to the cef-rs OSR example:

- Metal backend
- Bgra8UnormSrgb surface format (sRGB fix from cef-rs)
- Linear sampler, clamp to edge
- TriangleStrip topology, 4 vertices per quad
- Alpha blending (Over)

## Input Routing

### Focus Model

- **Mouse events** are routed based on cursor position:
  - `cursor.x < window_width / 2` → left profile
  - `cursor.x >= window_width / 2` → right profile
- **Keyboard events** go to the last-clicked side (focus follows click)
- **Scroll events** go to whichever side the cursor is over

### Coordinate Translation

Mouse coordinates must be translated from window-space to profile-local space:

```
Window space (logical):  (x, y) where x ∈ [0, 1600], y ∈ [0, 800]

Left profile:   (x, y)           → same coordinates, x ∈ [0, 800]
Right profile:  (x - 800, y)     → offset by half window width
```

Scale factor is applied when constructing CEF mouse events. The profile server
receives logical coordinates and multiplies by `device_scale_factor` internally
via CEF's `screen_info()` callback.

### Input Handling

Keyboard and mouse handling adapted directly from the cef-rs OSR example's input
code (main.rs lines 407-556), which already handles:

- Mouse movement with modifier tracking
- Mouse clicks with button state bitmask
- Scroll wheel with line-to-pixel conversion
- Keyboard events with native key code mapping
- Modifier state (Shift, Control, Alt, Command)

The only difference: instead of calling `host.send_mouse_move_event()` directly,
the GUI serializes the event into an XPC dictionary and sends it to the
appropriate profile server.

## Performance Measurement

### Profile Server Logging

Each profile server logs `[FRAME-TX]` on every `on_accelerated_paint` callback,
identical to ts3's format:

```
[FRAME-TX] frame=42 w=1600 h=1600 port=12345 url=github.com time=1234567890
```

### GUI Logging

The GUI logs frame intervals, measuring the time between consecutive
`display_surface` messages from each profile:

```
[LEFT]  frame=42 interval=16ms
[RIGHT] frame=37 interval=17ms
```

### Comparison Targets

| Source                  | fps   | 60fps % | Max streak |
| ----------------------- | ----- | ------- | ---------- |
| cef-rs OSR (in-process) | ~60   | ~95%    | ~400+      |
| ts3 profile server      | 38.2  | 71%     | 424        |
| **cef-test (target)**   | **?** | **?**   | **?**      |

If cef-test matches cef-rs: the problem is in ts3's integration. If cef-test
matches ts3: the problem is inherent to multi-process headless CEF.

## Directory Structure

```
cef-test/
├── Cargo.toml                      (workspace)
├── cef-test-gui/
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs                 (window, event loop, XPC manager)
│       └── webrender.rs            (wgpu pipeline, texture import, rendering)
├── cef-test-profile/
│   ├── Cargo.toml
│   └── src/
│       └── main.rs                 (CEF init, render handler, message loop)
├── cef-test-launcher/
│   ├── Cargo.toml
│   └── src/
│       └── main.rs                 (XPC bootstrap service)
└── scripts/
    └── build.sh                    (build all, bundle as macOS app)
```

### macOS App Bundle

CEF requires a proper macOS app bundle. The build script produces:

```
CefTest.app/
├── Contents/
│   ├── MacOS/
│   │   └── cef-test-gui                (main binary)
│   ├── Frameworks/
│   │   ├── Chromium Embedded Framework.framework/
│   │   └── cef-test-profile            (profile server binary)
│   ├── XPCServices/
│   │   └── com.cef-test.launcher.xpc/
│   │       └── Contents/
│   │           ├── MacOS/
│   │           │   └── cef-test-launcher
│   │           └── Info.plist          (XPC service registration)
│   └── Info.plist
```

The launcher's `Info.plist` registers the `com.cef-test.launcher` Mach service
with launchd, enabling both the GUI and profile servers to connect to it by
name.

## Dependencies

### Shared XPC Crate

Both cef-test and ts3 need XPC bindings. Currently `termsurf-xpc` lives inside
`ts3/`. To satisfy the "no dependency on ts3" constraint, extract it to a
top-level location:

```
termsurf/
├── termsurf-xpc/       ← extracted from ts3/termsurf-xpc/
├── cef-rs/
├── cef-test/
├── ts3/                (references ../termsurf-xpc/)
└── docs/
```

Both ts3 and cef-test reference it via `path = "../termsurf-xpc"`. The crate has
no dependencies on ts3 code — it's a standalone XPC bindings library with `libc`
and `block2` as its only dependencies.

### cef-test-gui Dependencies

```toml
[dependencies]
termsurf-xpc = { path = "../termsurf-xpc" }
cef = { path = "../cef-rs/cef", features = ["accelerated_osr"] }
wgpu = "..."
winit = "0.30"
pollster = "0.4"
bytemuck = "1"

[target.'cfg(target_os = "macos")'.dependencies]
metal = "..."
objc = "..."
io-surface = "..."
```

Note: cef-test-gui depends on the `cef` crate only for the `IOSurfaceImporter`
and texture import utilities. It does NOT initialize CEF or call any CEF browser
APIs.

### cef-test-profile Dependencies

```toml
[dependencies]
termsurf-xpc = { path = "../termsurf-xpc" }
cef = { path = "../cef-rs/cef", features = ["accelerated_osr"] }
clap = "..."
ctrlc = "3.4"
```

No wgpu, no winit, no window. Headless.

### cef-test-launcher Dependencies

```toml
[dependencies]
termsurf-xpc = { path = "../termsurf-xpc" }
```

Nothing else. The launcher is pure XPC plumbing.

## Key Simplifications vs ts3

| Aspect             | ts3                                     | cef-test                     |
| ------------------ | --------------------------------------- | ---------------------------- |
| GUI                | WezTerm (terminal emulator + webview)   | Bare winit window + wgpu     |
| Window management  | Tabs, splits, panes, multiplexing       | Fixed 2-panel layout         |
| Browser lifecycle  | Dynamic via `web` command               | Fixed at startup             |
| Profile reuse      | Launcher detects existing, forwards     | Launcher does same (simpler) |
| Input pipeline     | Terminal → web command → socket → XPC   | winit → XPC (direct)         |
| Rendering          | wgpu integrated into WezTerm's renderer | Standalone wgpu pipeline     |
| Event loop         | WezTerm's complex event loop            | Simple winit pump_app_events |
| Configuration      | WezTerm config, profiles, multiplexer   | CLI args only                |
| Total lines (est.) | ~100k+ (WezTerm fork)                   | ~2000                        |

## Build & Run

```bash
cd cef-test && ./scripts/build.sh
./CefTest.app/Contents/MacOS/cef-test-gui
```

The build script:

1. `cargo build` all three binaries
2. Bundle into CefTest.app with correct directory structure
3. Copy CEF framework into Frameworks/
4. Copy helper processes
5. Create Info.plist files

## Expected Outcomes

### If cef-test reproduces the problem (~38fps)

The root cause is inherent to headless CEF processes. Experiments to try here:

1. **`external_message_pump: true`** — The cef-rs example uses this and achieves
   60fps. ts3 couldn't use it due to a deadlock during init (Issue 342 Exp 4).
   cef-test may avoid the deadlock since it has a simpler init sequence.

2. **CVDisplayLink in profile server** — Create a CVDisplayLink (requires a
   hidden CAMetalLayer or IOSurface-based display link) to provide hardware
   vsync timing to the headless process.

3. **winit in profile server** — Add a hidden winit window to the profile server
   purely for `pump_app_events`. This is ugly but would definitively test
   whether the windowed event loop is what makes the cef-rs example fast.

4. **Vary the message loop** — Much easier to iterate on message loop
   experiments (cfrunloop timeout, NSApp pump, timer-based scheduling) with a
   2000-line codebase than a 100k-line one.

### If cef-test achieves ~60fps

The root cause is in ts3's integration. Suspects:

- WezTerm's event loop interfering with XPC message handling
- Additional latency in the web command → socket → GUI → XPC path
- Pane management overhead in the rendering path
- WezTerm's wgpu integration conflicting with IOSurface import

In this case, cef-test becomes the reference implementation and we progressively
add ts3 features until performance degrades, identifying the exact culprit.

### Either way, cef-test wins

A minimal reproduction is the gold standard for performance debugging. Whether
the problem reproduces or not, we learn something definitive and have a fast
iteration environment.
