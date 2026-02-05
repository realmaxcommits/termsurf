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

- [ ] Survey of C++ terminals (Alacritty's C++ predecessors, etc.)
- [ ] Feasibility of Rust + C++ FFI for Chromium embedding
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

## Related Issues

- [Issue 338: Browser lag investigation](./338-lag.md) — Why CEF doesn't work
- [Issue 339: Electron study](./339-electron.md) — How Electron achieves 240fps
