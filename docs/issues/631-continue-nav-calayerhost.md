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
