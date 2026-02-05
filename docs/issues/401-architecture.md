# TermSurf 4.0: Architecture Reconsideration

## Problem

Issue 339 concluded that CEF cannot deliver the 60fps browser rendering TermSurf
requires. The only viable path forward is embedding Chromium directly, like
Electron and Steam have done.

This forces us to reconsider nearly every architectural choice made to date.

## Why This Changes Everything

### The Original Architecture (ts3)

TermSurf 3.0 was built on two Rust foundations:

| Component | Implementation | Language               |
| --------- | -------------- | ---------------------- |
| Terminal  | WezTerm (fork) | Rust                   |
| Browser   | CEF via cef-rs | Rust (bindings to C++) |

This worked because:

- WezTerm is a mature, full-featured terminal in Rust
- cef-rs provided Rust bindings to CEF
- Both could be "plugged together" in a unified Rust codebase

### The New Reality

Embedding Chromium directly means:

- **Chromium is C++** — No Rust bindings exist for direct embedding
- **Electron is C++** — Its OSR implementation is C++
- **The 240fps code path is C++** — `FrameSinkVideoCapturer`, `viz` layer, etc.

This breaks the Rust assumption.

## The Core Question

**What programming language and terminal should TermSurf use?**

### Option Space

| Approach | Terminal                  | Browser         | Language       |
| -------- | ------------------------- | --------------- | -------------- |
| A        | C++ terminal              | Chromium direct | C++            |
| B        | Rust terminal + C++ FFI   | Chromium direct | Rust + C++     |
| C        | Other language + bindings | Electron OSR    | ???            |
| D        | Electron-based terminal   | Electron        | TypeScript/C++ |

### Factors to Consider

1. **Terminal quality** — Must match or exceed current WezTerm functionality
2. **Browser integration** — Must achieve 60fps with GPU texture sharing
3. **Development velocity** — Team expertise, ecosystem, tooling
4. **Maintenance burden** — Long-term cost of each approach
5. **Cross-platform** — macOS now, Linux/Windows later

## Research

### What This Document Will Track

- [x] Survey of C++ terminals (Alacritty's C++ predecessors, etc.)
- [x] Feasibility of Rust + C++ FFI for Chromium embedding
- [ ] Electron as a platform (not just browser component)
- [ ] Alternative approaches (WebGPU terminals, etc.)
- [ ] Decision framework and recommendation

### Research 1: C++ Terminals

**Question:** Are there C++ terminals that support tabs, panes, cross-platform
(Windows/Linux/macOS), and GPU rendering?

**Answer:** No. No C++ terminal meets all criteria.

#### C++ Terminals Survey

| Terminal                                               | Tabs | Panes | Platforms           | GPU          |
| ------------------------------------------------------ | ---- | ----- | ------------------- | ------------ |
| [Contour](https://github.com/contour-terminal/contour) | ✅   | ❌    | Win/Linux/macOS/BSD | ✅ OpenGL    |
| [Konsole](https://konsole.kde.org/)                    | ✅   | ✅    | Linux only          | ❌ Qt/CPU    |
| [Terminator](https://gnome-terminator.org/)            | ✅   | ✅    | Linux only          | ❌ GTK/CPU   |
| cool-retro-term                                        | ❌   | ❌    | Linux/macOS         | ✅ Qt/OpenGL |

**Contour** is the closest — C++23, GPU-accelerated via OpenGL, cross-platform —
but splits/panes are
[still a feature request](https://github.com/contour-terminal/contour/issues/1170).
We could contribute splits ourselves, but that's significant work.

**Konsole** has tabs and splits but is Linux-only and CPU-rendered (Qt).

#### The Modern GPU Terminals Are Not C++

The terminals that meet all criteria are written in other languages:

| Terminal                                   | Tabs | Panes | Platforms       | GPU | Language     |
| ------------------------------------------ | ---- | ----- | --------------- | --- | ------------ |
| [WezTerm](https://wezfurlong.org/wezterm/) | ✅   | ✅    | Win/Linux/macOS | ✅  | **Rust**     |
| [Alacritty](https://alacritty.org/)        | ❌   | ❌    | Win/Linux/macOS | ✅  | **Rust**     |
| [Kitty](https://sw.kovidgoyal.net/kitty/)  | ✅   | ✅    | Linux/macOS     | ✅  | **Python/C** |
| [Ghostty](https://ghostty.org/)            | ✅   | ✅    | Linux/macOS     | ✅  | **Zig**      |

#### Alternative: Electron-Based Terminal

Since we're considering embedding Electron anyway, [Hyper](https://hyper.is/) is
notable — it's already Electron-based:

| Feature   | Hyper                     |
| --------- | ------------------------- |
| Language  | TypeScript/Electron       |
| Tabs      | ✅                        |
| Splits    | ✅ (Cmd+D / Cmd+Shift+D)  |
| Platforms | Win/Linux/macOS           |
| Rendering | Canvas (xterm.js)         |
| GPU       | ❌ (but runs in Chromium) |

If we go the Electron route, terminal and browser would share the same runtime.

#### Conclusion

The C++ terminal ecosystem doesn't have what we need. Options:

1. **Contour + contribute splits** — Significant C++ work
2. **Hyper (Electron)** — Already has everything, shares runtime with browser
3. **WezTerm + C++ FFI** — Keep current terminal, wrap Chromium in Rust
4. **Build from scratch** — Unified architecture, highest effort

### Research 2: High-FPS Chromium Integration Options

**Question:** What are our options for achieving 60fps browser rendering? Can CEF
ever work, or must we integrate Chromium directly?

**Answer:** CEF cannot deliver 60fps. Direct Chromium integration is required.
Our XPC architecture already supports swapping the profile server implementation.

#### The Definitive Answer on CEF

CEF's frame throttling is hard-coded in `CefCopyFrameGenerator::GenerateCopyFrame()`:

```cpp
if (frame_in_progress_)
    return;  // Discards >66% of frames
```

This is not configurable. Issue 338 documented five failed experiments:

| Experiment | Approach                       | Result                           |
| ---------- | ------------------------------ | -------------------------------- |
| 1          | Cache IOSurface texture        | Import is fast (0.37ms), not it  |
| 2          | Investigate frame pacing       | Confirmed CEF's 30fps cap        |
| 3          | `multi_threaded_message_loop`  | Incompatible with OSR on macOS   |
| 4          | `external_begin_frame_enabled` | Reduced stutters, still ~20fps   |
| 5          | Chrome command-line flags      | Flags don't affect OSR code path |

Someone achieved [1100 FPS on CEF 85 (2020)](https://magpcss.org/ceforum/viewtopic.php?f=6&t=19628)
with `--disable-frame-rate-limit --disable-gpu-vsync`. This stopped working in
newer versions—we're on CEF 143.

#### How Electron Achieves 240fps

Electron uses `FrameSinkVideoCapturer`, a Chromium internal API that CEF doesn't
expose:

```cpp
// Electron's approach (osr_video_consumer.cc)
video_capturer_->SetAutoThrottlingEnabled(false);  // No 30fps cap!
video_capturer_->Start(this,
    viz::mojom::BufferFormatPreference::kPreferMappableSharedImage);
```

On macOS, frames arrive as `IOSurfaceRef`—exactly what our GUI already consumes.

#### Our Architecture Supports This

The XPC profile server is process-isolated from the GUI:

```
GUI (Rust/WezTerm) ←—XPC—→ Profile Server (???) ←→ Browser Engine
```

The profile server doesn't need to be Rust. It just needs to:

1. Receive commands via XPC
2. Render web pages at 60fps
3. Send IOSurface Mach ports back

**We can rewrite the profile server in C++ to wrap Chromium directly.**

#### The Four Options

| Option                         | FPS | Effort    | Maintenance              | Risk    |
| ------------------------------ | --- | --------- | ------------------------ | ------- |
| **A. Embed Electron**          | 240 | Medium    | Low (Electron team)      | Low     |
| **B. Chromium direct (C++)**   | 240 | Very High | High (track Chromium)    | Medium  |
| **C. Patch CEF**               | 240 | High      | High (fork forever)      | Medium  |
| **D. OBS's CEF fork**          | ?   | Medium    | Medium (depends on OBS)  | Unknown |

##### Option A: Embed Electron (Recommended)

Use Electron as a headless OSR renderer in the profile server process:

- **Proven:** 240fps in VS Code, Figma, Slack
- **Maintained:** Large team tracking Chromium updates
- **Same pipeline:** IOSurface → Mach port → GUI (unchanged)
- **Reference code:** [Electron's OSR implementation](https://github.com/electron/electron/blob/main/shell/browser/osr/)

##### Option B: Chromium Direct (C++)

Write a C++ profile server wrapping Chromium's viz layer directly. This is what
Electron does internally—we'd reimplement their OSR layer for our use case.

Required Chromium components:

- `components/viz/host/client_frame_sink_video_capturer.h`
- `services/viz/privileged/mojom/compositing/frame_sink_video_capture.mojom`
- `ui/gfx/gpu_memory_buffer.h`

Build complexity: 25GB checkout, 5-40 minute builds, no stable API.

This is the "earn the pain" path that Steam took.

##### Option C: Patch CEF

Fork CEF and add `FrameSinkVideoCapturer` support:

1. Add new callback: `OnFrameCaptured()`
2. Wire it to Chromium's capturer instead of `CefCopyFrameGenerator`
3. Maintain the fork forever

Significant C++ work requiring deep CEF/Chromium knowledge.

##### Option D: OBS's CEF Fork

[OBS has a custom CEF fork](https://github.com/obsproject/cef) for streaming
overlays. However, [their PR discussions](https://github.com/obsproject/obs-browser/pull/310)
show `OnAcceleratedPaint` doesn't trigger on macOS arm64.

#### Conclusion

| Question                          | Answer                                    |
| --------------------------------- | ----------------------------------------- |
| Will CEF ever deliver 60fps?      | **No.** Architectural limit.              |
| Must we integrate Chromium directly? | **Yes**, for 60fps GPU OSR.            |
| Can we embed just Electron's OSR? | **Yes.** Headless Electron as profile server. |
| Can we write our own C++ wrapper? | **Yes.** Same as Electron but without Node. |

**Recommendation:** Embed Electron (Option A). It's proven, maintained, and our
XPC architecture already supports swapping the profile server implementation.

If we need maximum control later, Option B (Chromium direct) remains available—
it's the same path Steam and Brave took.

## Related Issues

- [Issue 338: Browser lag investigation](./338-lag.md) — Why CEF doesn't work
- [Issue 339: Electron study](./339-electron.md) — How Electron achieves 240fps
