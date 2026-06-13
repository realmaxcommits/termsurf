# Experiment 180: Phase C — live draw ownership audit

## Description

Close the remaining Phase C ambiguity around `surface_draw` ownership and the
interim `render_state` pull path by auditing the copied app's actual render
loop.

Earlier Phase C experiments built the live Metal renderer, attached it to the
app-provided `NSView`, proved CoreVideo display-link presentation with live
smoke, threaded cursor blink state through live frame rendering, and propagated
focus / visibility / config options. The roadmap still has two unchecked items:

- `surface_draw` owns a Metal renderer bound to the app `NSView` / `CALayer`;
  attach the layer and present on-screen.
- Retire the interim `render_state` pull divergence.

Current source evidence suggests these are now mostly proof/documentation gaps:
the copied Swift app creates each surface with its `NSView`, drives size/content
scale/focus/input through the surface ABI, and does not call
`roastty_surface_render_state_update` or the C `render_state` row/cell iterator
APIs. The live Rust path stores `SurfaceLiveRenderer` on `Surface`,
`roastty_surface_draw` calls `Surface::draw`, and `Surface::present_live` lazily
builds the Metal compositor, attaches the IOSurface layer to the `NSView`, and
presents live frames.

This experiment should prove those statements against the current tree and
update the Issue 802 roadmap only when the evidence is strong enough. It should
not delete the generic C `render_state` ABI, because upstream still exposes
terminal render-state helpers through `lib_vt`; the Phase C concern is the
copied app render loop relying on an interim pull path instead of the live
surface renderer.

## Changes

- `issues/0802-libroastty-completion-and-mac-app/README.md`
  - Link this experiment as `Designed`.
  - After verification, mark it `Pass`, `Partial`, or `Fail`.
  - Check the `surface_draw` ownership item only if source inspection and live
    smoke prove the copied app path creates a live renderer owned by `Surface`,
    attached to the app `NSView`, and presents on-screen without a separate
    Swift renderer.
  - Check the `render_state` pull divergence item only if source inspection
    proves the copied app render path no longer calls
    `roastty_surface_render_state_update` or lower-level C render-state
    iterator/cell APIs.

- `issues/0802-libroastty-completion-and-mac-app/180-live-draw-ownership-audit.md`
  - Record the source evidence, command output, live artifact paths, result,
    conclusion, and AI completion review.

- `roastty/src/lib.rs`
  - No production change is expected.
  - Add a narrow test only if the design review identifies a missing,
    deterministic assertion needed to prove the ownership claim.

## Verification

Before implementation:

- Codex-native adversarial design review approves this experiment.
- Commit the reviewed plan separately from the result.

Source audit:

- Prove the copied Swift app has no render-state pull usage:

  ```bash
  rg -n "render_state|RenderState|surface_render_state_update|roastty_render_state" \
    roastty/macos/Sources
  ```

- Prove the copied Swift app creates surfaces with the AppKit view and routes
  resize/scale/focus through the surface ABI:

  ```bash
  rg -n "roastty_surface_new|roastty_surface_set_size|roastty_surface_set_content_scale|roastty_surface_set_focus" \
    "roastty/macos/Sources/Roastty/Surface View"
  ```

- Prove the Rust surface draw path owns and presents through the live renderer:

  ```bash
  rg -n "struct SurfaceLiveRenderer|fn build_live_renderer|fn draw\\(&mut self\\)|fn present_live|roastty_surface_draw|attach_to_nsview|render_and_present_frame" \
    roastty/src/lib.rs roastty/src/renderer
  ```

- Compare against upstream embedded draw ownership:

  ```bash
  sed -n '760,785p' vendor/ghostty/src/apprt/embedded.zig
  sed -n '875,887p' vendor/ghostty/src/Surface.zig
  ```

Regression checks:

- `cargo test -p roastty live_renderer_options -- --test-threads=1`
- `cargo test -p roastty live_cursor_blink -- --test-threads=1`
- `cargo test -p roastty --test abi_harness`
- `cargo fmt --check -p roastty`
- `prettier --check --prose-wrap always --print-width 80 issues/0802-libroastty-completion-and-mac-app/180-live-draw-ownership-audit.md issues/0802-libroastty-completion-and-mac-app/README.md`
- `git diff --check`

Live sanity:

- Rebuild the copied app:

  ```bash
  cd roastty && macos/build.nu --action build
  ```

- Re-run live smoke:

  ```bash
  scripts/roastty-app/stop-app.sh
  TERMSURF_AB_HOLD_SECONDS=10 \
  ROASTTY_PRESENT_DRIVER_LOG=1 \
    scripts/roastty-app/live-ab-smoke.sh \
      --recipe smoke \
      --comparison-region content \
      --max-mismatch-ratio 1 \
      --max-mean-channel-delta 255
  ```

**Pass** = source audit proves the copied app render path is live
`SurfaceLiveRenderer` ownership rather than Swift/render-state pulling,
regression checks pass, the copied app rebuilds, live smoke renders with
`present-driver=display-link reason=core-video`, and both remaining Phase C
ownership/divergence checklist items can be checked.

**Partial** = live rendering remains healthy, but source evidence shows any
copied-app render path still depends on `roastty_surface_render_state_update` or
the ownership claim remains too indirect to check a roadmap item. Record the
exact missing proof.

**Fail** = source audit contradicts the ownership claim, app startup/rendering
regresses, or the verification gates fail.

## Design Review

**Reviewer:** Codex-native adversarial review subagent `Harvey`, fresh context.

**Verdict:** Approved.

Findings: None. The reviewer confirmed the README links Experiment 180 as
`Designed`, the experiment has the required sections, an audit-only scope is
legitimate for closing the remaining Phase C proof gaps, the design explicitly
preserves the public `render_state` ABI because upstream still exposes it
through `lib_vt`, and the verification includes source-audit, live smoke,
regression, formatting, Prettier, and `git diff --check` gates.
