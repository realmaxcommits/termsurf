# Vsync

**Status:** Good enough. Revisit later for efficiency.

See [Issue 512](issues/512-vsync.md) for the full analysis, research, and
experiment results.

## The problem

TermSurf composites Chromium frames inside terminal panes. Two independent 60fps
clocks drive the pipeline:

1. **Chromium's `FrameSinkVideoCapturer`** — produces frames on its own timer
2. **macOS CVDisplayLink** — drives the Metal renderer on the display's vsync

Even though both average 60fps, their ticks drift relative to each other. Some
vsyncs get two frames (one wasted), some get zero (duplicate displayed). The
result is micro-stutter — uneven frame intervals that the eye detects even when
the average framerate is correct.

## Options

| # | Approach                                           | Complexity      | Visual quality      | Efficiency      |
| - | -------------------------------------------------- | --------------- | ------------------- | --------------- |
| 1 | Fix `needs_redraw` for overlay changes             | Trivial         | Required baseline   | —               |
| 2 | 120fps capture (2x oversampling)                   | One-line change | Identical to Chrome | 2x capture cost |
| 3 | Demand-driven pull (RequestRefreshFrame per vsync) | Medium          | Identical to Chrome | Optimal         |
| 4 | In-process Chromium (single BeginFrameSource)      | Large           | Identical to Chrome | Optimal         |

Chromium solves this internally with a single authoritative clock — the
display's vsync — propagated to all frame producers via BeginFrame signals. No
producer runs its own timer. Electron's off-screen rendering has the same
two-clock problem we do and does not solve it.

## Current solution

**Options 1 + 2**, implemented in Issue 512 Experiment 1.

The `overlay_surface_changed` flag ensures every new IOSurface triggers a
redraw. The 120fps capture rate means there is always a frame no older than ~8ms
at every vsync. Side-by-side with native Chromium, TermSurf looks identical.

The cost is doubled GPU capture blits and XPC traffic (120 Mach port transfers
per second per pane instead of 60). For a handful of panes this is negligible.

## Future improvement

The correct long-term solution is option 4: a single vsync source of truth owned
by TermSurf's CVDisplayLink, with Chromium's frame production driven by
BeginFrame signals originating from that clock. This would require
rearchitecting the XPC protocol and possibly making significant changes inside
Chromium to wire the capturer into the BeginFrame pipeline.

The visual result would be the same — the 120fps solution already matches native
Chrome. The improvement is purely efficiency: no wasted captures, no redundant
XPC messages, no frames produced that are never displayed.

This is worth doing, but not urgent. It will be revisited once more fundamental
features are in place.
