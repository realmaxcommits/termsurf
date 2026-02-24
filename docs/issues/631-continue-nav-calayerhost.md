# Issue 631: Continue Navigation CALayerHost

## Goal

Eliminate the ~100ms flicker that occurs on every page navigation. The browser
overlay should transition seamlessly — no visible blank frame between the old
page and the new page.

## Background

### CALayerHost issue history

This is the seventh issue in the CALayerHost series. Each addressed a different
regression from the migration away from `FrameSinkVideoCapturer`:

- [Issue 625](625-calayerhost.md) — **CALayerHost migration.** Replaced the
  `FrameSinkVideoCapturer` pipeline with `CALayerHost`. Instead of capturing
  IOSurface frames at 120fps and transferring Mach ports over XPC every frame,
  Chromium now sends a `ca_context_id` (uint32) once per tab. The GUI creates a
  `CALayerHost` sublayer, and Window Server composites the remote content
  directly from GPU VRAM. Zero per-frame IPC, zero texture copies.

- [Issue 626](626-x-y-calayerhost.md) — **X/Y positioning.** The CALayerHost
  overlay had a ~10px Y and ~3px X offset. Fixed by adding a positioning layer
  inside a geometry-flipped layer, matching Chromium's `maybe_flipped_layer_`
  pattern.

- [Issue 627](627-resize-calayerhost.md) — **Resize.** The overlay stopped
  resizing when the user resized the window or pane. Fixed by propagating resize
  events through XPC to the Chromium capturer and updating the positioning
  layer's frame.

- [Issue 628](628-navigation-calayerhost.md) — **Navigation (first attempt).**
  Ran 8 experiments targeting the Chromium-side pipeline. All failed. Key
  finding from diagnostic logging: the new `ca_context_id` arrives within 100ms
  and the GUI replaces the `CALayerHost` immediately, yet the new host shows
  nothing for ~10 seconds.

- [Issue 629](629-understand-nav-calayerhost.md) — **Navigation (diagnosis).**
  Research issue. Five experiments: compared Electron/Chromium CALayerHost
  usage, traced the CAContext lifecycle, tested `DisableDisplay()` (made things
  worse), audited all 10-second delays in Chromium, and performed a full code
  audit of both the GUI and Chromium Profile Server. Produced the primary
  hypothesis and confirmed two latent bugs.

- [Issue 630](630-nav-calayerhost-6.md) — **Navigation (fix).** Resolved the
  permanent overlay disappearance with seven coordinated fixes across GUI (Zig)
  and Chromium (C++): transparent hidden window instead of `orderOut:` (C1),
  callback re-registration on view swap (C2), dedup gate reset (C3),
  `ResizeWebContentForTests` for correct `dfh_size_dip_` (C4), main-thread
  dispatch with CATransaction wrapping (G1), atomic CALayerHost swap (G2), and
  zero context ID guard (G3). Navigation no longer causes permanent
  disappearance, but a brief ~100ms flicker remains on every navigation.

### What we know

1. **The permanent blank is fixed.** Issue 630's seven fixes resolved the
   overlay vanishing forever on navigation.
2. **A ~100ms flicker remains.** On every navigation, the overlay briefly
   disappears then reappears. Visible and annoying, but not app-breaking.
3. **The flicker is likely compositor-side.** The CAContext's content tree is
   torn down and rebuilt during navigation. Even though the CALayerHost stays
   pointed at the right CAContext, there is a brief moment where the CAContext
   has no rendered content.
4. **The `ca_context_id` may not change.** During same-site navigation, the same
   `CALayerTreeCoordinator` may keep the same ID. If so, the CALayerHost swap
   triggered by our dedup reset (C3) is unnecessary and may itself cause the
   flicker.

### Untested CALayerHost changes

The CALayerHost migration (Issues 625–630) replaced fundamental rendering
infrastructure. The following features have not been retested since the
migration and may have regressions:

- **Mouse input**: clicks, drag, scroll, cursor changes (Issue 606)
- **Keyboard input**: key forwarding, Cmd+key bypass, clipboard, Tab (Issues
  607–609)
- **Loading progress**: progress bar, pulse animation (Issue 616)
- **Browser navigation keybindings**: Cmd+L, Cmd+R, back/forward (Issue 616)
- **Multi-pane multi-profile**: server reuse, independent tabs (Issues 604–605)
- **Dynamic resize**: pane resize propagation through XPC (Issue 627)
- **Text selection**: drag-to-select, cursor changes (Issue 606)

A comprehensive retest should be performed as part of this issue or immediately
after.

### Chromium branch

Continue from `146.0.7650.0-issue-630`.

### Possible approaches

- **Don't swap when `ca_context_id` is unchanged.** If same-site navigation
  keeps the same ID, skipping the CALayerHost replacement eliminates the
  GUI-side gap entirely.
- **Snapshot before swap.** Capture the current CALayerHost content as a
  `CGImage` and place it on a static `CALayer` behind the host. When the host
  goes blank during transition, the snapshot shows through.
- **Delay old host removal.** Keep the old CALayerHost for ~200ms after adding
  the new one, so the old content remains visible until the new host composites.
- **Debug logging.** Add timestamps at every stage (XPC arrival, host swap,
  Chromium callback) to confirm whether the gap is GUI-side or Chromium-side.

## Experiments

### Experiment 1: Audit code for navigation flicker smells

#### Purpose

Identify what causes the ~100ms blank frame during every page navigation. The
permanent blank is fixed (Issue 630), but a brief flicker remains. This audit
searches both the GUI (Zig) and Chromium Profile Server (C++) code for patterns
that could cause a momentary gap in content during navigation.

This is a research-only audit — no code modifications.

#### Code smells

**Flicker-specific smells (1–10):**

1. **Unnecessary CALayerHost swap on same ID.** The dedup gate reset (C3) forces
   `*last_ca_context_id_ = 0` on every navigation. If the `ca_context_id`
   doesn't actually change during same-site navigation, this causes the callback
   to fire with the same ID, triggering a full CALayerHost destroy-and-recreate
   in the GUI — a swap that produces a blank frame for no reason.

2. **CAContext content gap during compositor surface transition.** When the
   Chromium compositor processes a navigation, it may invalidate the old
   `LocalSurfaceId` and allocate a new one. During the transition, the
   `CAContext` exists but has no submitted frame — the CALayerHost renders as
   transparent. This is a Chromium-side gap that no GUI-side fix can address.

3. **Async main-thread dispatch adds latency to host swap.** The `ca_context`
   XPC message arrives on the XPC queue, then `dispatch_async_f` to the main
   queue adds scheduling latency. If the old host was already torn down
   Chromium-side but the new host isn't created until the main queue drains,
   there is a visible gap.

4. **draw_mutex contention during swap.** `setCAContextId()` acquires
   `draw_mutex` on the main thread. If the renderer thread holds it (mid-frame),
   the main thread blocks. The CALayerHost replacement is delayed until the
   frame completes, extending the blank window.

5. **No content readiness signal.** The GUI swaps the CALayerHost as soon as the
   new `ca_context_id` arrives. But the new CAContext may not have a submitted
   frame yet. The new host is added to the layer tree pointing at an empty
   context. A "content ready" signal from Chromium (e.g., after the first
   `SubmitCompositorFrame`) would allow delaying the swap until content exists.

6. **Old host removed too early in atomic swap.** The atomic swap (G2) adds the
   new host before removing the old one, but both happen in the same
   CATransaction. If the new host's CAContext has no content yet, removing the
   old host (which may still have stale content from the old page) eliminates
   the only visible content. The old host should stay until the new one has
   rendered.

7. **CATransaction commit flushes both add and remove simultaneously.** The
   single CATransaction wrapping the entire swap means Window Server sees "add
   new + remove old" as one atomic operation. If the new host's context is
   empty, Window Server transitions from "old content" to "nothing" in one
   commit.

8. **Chromium `DidNavigate()` surface ID churn.** During navigation,
   `BrowserCompositorMac::DidNavigate()` calls
   `InvalidateLocalSurfaceIdAndAllocationGroup()` which invalidates the current
   surface, then allocates a new one via `GetRendererLocalSurfaceId()`. Between
   invalidation and the first frame on the new surface, the CAContext has
   nothing to display.

9. **No fallback content during transition.** Unlike the old
   `FrameSinkVideoCapturer` pipeline (which always had the last captured frame
   as a texture), the CALayerHost pipeline has no fallback. When the CAContext
   goes empty, there is nothing to show — just transparency.

10. **RenderViewHostChanged re-registration triggers redundant swap.** If
    `RenderViewHostChanged` fires AND the CALayerParams callback also fires with
    a new ID, two host swaps happen in quick succession. The first swap may
    create a host pointing at a stale context, immediately replaced by the
    second.

**Structural smells (11–15):**

11. **No timestamp logging at swap boundaries.** We have log lines for "replaced
    CALayerHost" and "Sent ca_context_id" but no microsecond timestamps showing
    the gap between: (a) Chromium navigation commit, (b) CALayerParams callback
    fire, (c) XPC message send, (d) XPC message receive, (e) main-thread
    dispatch, (f) CALayerHost swap, (g) first visible frame. Without these, we
    cannot distinguish GUI-side from Chromium-side flicker.

12. **Dedup reset timing vs callback timing.** The dedup gate is reset in
    `DidFinishNavigation()`, which fires when the navigation commits. The
    CALayerParams callback fires when the compositor produces new params. If the
    compositor fires BEFORE `DidFinishNavigation` resets the gate, the callback
    is still blocked by the old dedup value and the new context ID is missed.

13. **No distinction between same-site and cross-site navigation.** The code
    treats all navigations identically — full dedup reset, potential host swap.
    Same-site navigations (where the CAContext survives) and cross-site
    navigations (where the RenderViewHost changes) may need different handling.

14. **CALayerHost replacement vs contextId update.** The code always destroys
    and recreates the CALayerHost when the context ID changes. Chromium's
    `DisplayCALayerTree::GotCALayerFrame()` does the same, but an alternative is
    to update the `contextId` property on the existing host. This avoids the
    remove/add cycle entirely. Issue 628 noted this "may not rebind Window
    Server compositing" but this was never tested in the post-630 codebase.

15. **Overlay visibility during loading state.** The `DidStartLoading` /
    `DidStopLoading` XPC messages are sent to the GUI, but the GUI does not use
    them to manage CALayerHost visibility. If the GUI knew a navigation was in
    progress, it could hold the old content visible until the new page's first
    frame arrives.

#### Files to audit

**GUI (Zig):**

- `gui/src/renderer/Metal.zig` — `setCALayerHostContextId()` swap logic,
  CATransaction wrapping, layer creation
- `gui/src/Surface.zig` — `setCAContextId()`, `draw_mutex` acquisition
- `gui/src/apprt/xpc.zig` — `handleCAContext()`, main-thread dispatch,
  `handleLoadingState()`

**Chromium (C++):**

- `content/chromium_profile_server/browser/shell_browser_main_parts.cc` —
  CALayerParams callback, dedup gate, `CreateTab()`
- `content/chromium_profile_server/browser/shell_tab_observer.cc` —
  `DidFinishNavigation()` dedup reset, `RenderViewHostChanged()` re-registration
- `content/chromium_profile_server/browser/shell_tab_observer.h` — observer
  interface, stored state

#### Steps

For each of the 6 files above:

1. Read the file in full.
2. Check each of the 15 code smells.
3. Record a verdict: **clean** (not present), **suspect** (possible but
   unconfirmed), or **confirmed** (definitely present).
4. Add a one-line note explaining the verdict.

#### Output format

A findings table per file:

```
#### File: `path/to/file.zig`

| # | Smell | Verdict | Note |
|---|-------|---------|------|
| 1 | Unnecessary swap on same ID | confirmed | Dedup reset forces swap even when ID unchanged |
| … | … | … | … |
```

After all files, a summary section listing every confirmed and suspect finding
with file path and line number.

#### Verification

Every confirmed and suspect finding has a file path, line number, and one-line
explanation. No smell is left unchecked for any file.
