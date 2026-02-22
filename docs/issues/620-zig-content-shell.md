# Issue 620: Zig Content Shell

## Goal

Build a minimal Chromium embedder using a thin C++ shim and Zig logic that can
load web pages and support multiple browser profiles in a single process. This
replaces the 14,000-line Content Shell fork with about 1,400 lines and
determines whether the browser can run in-process inside the GUI.

## Background

Issue 619 investigated input latency and traced it to three sources: the
FrameSinkVideoCapturer (a recording API, not the display path), asynchronous XPC
message-passing, and a double-vsync penalty from out-of-process streaming.
Research revealed that:

1. **Content Shell uses Chrome's native display path.** CALayerHost, zero-copy
   GPU compositing, compositor-thread input handling. Our FrameSinkVideoCapturer
   bypasses all of this — it is a recording API bolted onto the side of the
   display compositor.

2. **The multi-process architecture is a CEF artifact.** CEF required one
   process per browser profile (`SingletonLock` on `root_cache_path`). The
   Content API has no such limitation — `content::BrowserContext` supports
   multiple instances in one process with full isolation. ts4 proved this
   (Issues 406–413): two profiles at 60fps in a single content_shell process.

3. **The Content API is C++, but Zig can drive it.** A thin C++ shim (about 800
   lines) subclasses the required virtual classes (`ContentMainDelegate`,
   `ContentBrowserClient`, `BrowserMainParts`, `WebContentsDelegate`,
   `WebContentsObserver`) and forwards all calls to C functions. Zig implements
   those C functions — tab lifecycle, input routing, profile management. All
   logic lives in Zig; the C++ shim is mechanical glue.

### What the Zig Content Shell replaces

The current Chromium Profile Server is a fork of Content Shell: 13,000 lines of
unmodified boilerplate + 1,050 lines of TermSurf logic. Of those 1,050 lines,
590 are XPC gateway connection and input routing in
`shell_browser_main_parts.cc` and 460 are the `ShellVideoConsumer` (frame
capture + IOSurface transfer). The rest of the 100+ files are copied verbatim
and never modified.

The Zig Content Shell replaces all of this with two components:

- **C++ shim** (about 800 lines) — Subclasses Content API virtual classes,
  exposes C functions for Zig. Built inside `chromium/src/` with GN/autoninja.
- **Zig embedder** (about 600 lines) — Tab lifecycle, profile management, input
  routing. Built separately, linked against the C++ shim.

### What we strip from Content Shell

- Web test infrastructure (`IsRunWebTestsSwitchPresent()` paths) — roughly 30%
  of Content Shell's code
- Android, iOS, Fuchsia, ChromeOS platform code — macOS only
- Aura/Ozone UI toolkit code — macOS doesn't use these
- The `Shell` class window management — replaced by the C shim
- `ShellPlatformDelegate` platform abstraction — single platform, no abstraction
  needed
- DevTools HTTP server — can be re-added later via the shim if needed

### What we keep (via the C++ shim)

- `ContentMain()` — entry point
- `ContentMainDelegate` — app initialization (5 overrides)
- `ContentBrowserClient` — browser configuration (start with minimal overrides,
  add incrementally)
- `BrowserMainParts` — initialization pipeline
- `BrowserContext` — profile storage and isolation
- `WebContents` — page lifecycle, navigation
- `RenderWidgetHost` — input forwarding
- `NavigationController` — back/forward/reload
- `WebContentsObserver` — navigation events, loading state

### The critical experiment

Can two different browser profiles (`BrowserContext` instances with different
storage paths) coexist in the same Zig process? ts4 proved this works in a
native C++ content_shell. The experiment confirms it works through the C++
shim + Zig bridge.

If two profiles work: in-process is the answer. The GUI binary becomes the
browser process. The entire multi-process architecture (xpc-gateway, profile
server spawning, XPC connections, IOSurface Mach port transfer, frame capture,
120fps oversampling) goes away.

If two profiles fail: out-of-process with the Zig Content Shell as a separate
binary. Still a major improvement — 1,400 lines instead of 14,000, and the
codebase is understandable and modifiable.

## Architecture

### C++ shim (3 files in the Chromium fork)

The shim lives in `chromium/src/content/zig_content_shell/` — just 3 files
(BUILD.gn, one `.h`, one `.cc`). It must be inside `chromium/src/` because GN
can only see files rooted there. Built with autoninja, produces a shared library
(component build). This is the same pattern as the current
`chromium_profile_server/`, but 3 files instead of 100+.

```
content_api_shim.h    — C header (Zig-callable)
content_api_shim.cc   — C++ implementation
├── TsContentMainDelegate : ContentMainDelegate
├── TsContentBrowserClient : ContentBrowserClient
├── TsBrowserMainParts : BrowserMainParts
├── TsWebContentsDelegate : WebContentsDelegate
├── TsWebContentsObserver : WebContentsObserver
├── TsBrowserContext : BrowserContext
│
├── Initialization:
│   ts_content_main(argc, argv)
│
├── Profile management:
│   ts_create_browser_context(path) → context handle
│   ts_destroy_browser_context(handle)
│
├── Tab management:
│   ts_create_web_contents(context, url) → contents handle
│   ts_destroy_web_contents(handle)
│   ts_load_url(handle, url)
│
├── Navigation:
│   ts_go_back(handle)
│   ts_go_forward(handle)
│   ts_reload(handle)
│   ts_can_go_back(handle) → bool
│   ts_can_go_forward(handle) → bool
│
├── Input:
│   ts_forward_mouse_event(handle, type, x, y, button, mods)
│   ts_forward_scroll_event(handle, x, y, dx, dy, phase, mods)
│   ts_forward_key_event(handle, type, keycode, text, mods)
│   ts_set_focus(handle, focused)
│
├── Display:
│   ts_get_ca_context_id(handle) → uint32_t
│   ts_set_view_size(handle, width, height)
│
└── Callbacks (Zig → C function pointers, set at init):
    on_navigation_committed(handle, url)
    on_loading_state_changed(handle, state, progress)
    on_cursor_changed(handle, cursor_type)
    on_title_changed(handle, title)
```

### Zig embedder (`browser/`)

Top-level directory in the main repo, separate from `gui/`. Builds a standalone
binary for the experiment phase. If in-process wins, the Zig logic migrates into
`gui/src/` and the standalone binary goes away.

```
browser/
├── build.zig          — Build system, links against C++ shim
├── src/
│   ├── main.zig       — Entry point, initializes Content API
│   ├── profile.zig    — BrowserContext lifecycle
│   ├── tab.zig        — WebContents lifecycle
│   └── callbacks.zig  — Handles Content API callbacks
```

### Directory layout

```
~/dev/termsurf/
├── browser/                                    ← Zig embedder (main repo)
│   ├── build.zig
│   └── src/*.zig
├── chromium/src/content/zig_content_shell/     ← C++ shim (Chromium fork, 3 files)
│   ├── BUILD.gn
│   ├── content_api_shim.h
│   └── content_api_shim.cc
├── gui/                                        ← TermSurf GUI (Ghostty fork)
└── tui/                                        ← web TUI (Rust/ratatui)
```

### Build

Step 1 — Build the C++ shim (produces shared library in `out/Default/`):

```bash
cd chromium/src
autoninja -C out/Default zig_content_shell
```

Step 2 — Build the Zig embedder (links against the shim):

```bash
cd browser
zig build
```

### Display path

The Zig Content Shell does NOT use `FrameSinkVideoCapturer`. It uses Content
Shell's normal display path:

1. Content API renders into a `CAContext` (GPU process)
2. `AcceleratedWidgetMac` receives `CALayerParams` with `ca_context_id`
3. The C++ shim forwards the `ca_context_id` to Zig via callback
4. For the standalone experiment: create a window with a `CALayerHost`
5. For in-process (future): pass the `ca_context_id` to the GUI's Metal renderer

No frame capture. No IOSurface Mach port transfer. No recording API. The same
display path Chrome uses.
