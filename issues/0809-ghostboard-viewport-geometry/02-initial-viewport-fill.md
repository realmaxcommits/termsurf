# Experiment 2: Initial viewport fill

## Description

Experiment 1 reproduced the first viewport geometry failure with correlated
evidence. The initial browser content is visible but smaller than the AppKit
overlay frame. The passing `initial-open` run recorded:

- TUI/AppKit grid: `78x16+1+1`;
- AppKit cell size: `8.0x17.0` points;
- AppKit overlay frame: `624x272` points;
- backing scale: `2.0`;
- Roamium/browser pixel size: `780x320`.

That means the native overlay frame is `1248x544` physical pixels, while the
browser surface is only `780x320` physical pixels. This explains the screenshot:
Roamium is rendering at the wrong initial viewport size.

The likely root cause is that current Ghostboard still calculates browser
viewport pixels from hardcoded fallback cell dimensions in Zig:

- `ghostboard/src/apprt/termsurf.zig:1175-1176` sends initial `CreateTab` pixels
  as `pane.width * 10` and `pane.height * 20`;
- `ghostboard/src/apprt/termsurf.zig:1193-1194` does the same for DevTools tab
  creation;
- `ghostboard/src/apprt/termsurf.zig:1229-1230` does the same for `Resize`.

The legacy Ghostboard implementation solved this by reading the actual terminal
cell metrics from the owning surface before calculating browser pixels:

- `ghostboard-legacy/src/apprt/xpc.zig:280-283` computes
  `new_pixel_w/new_pixel_h` from `width/height * cell.width/cell.height`;
- `ghostboard-legacy/src/Surface.zig:2506-2515` exposes `getCellSize()`;
- `ghostboard-legacy/src/renderer/generic.zig:849-862` updates the CALayerHost
  frame from the same grid metrics and padding.

The new Ghostboard implementation has a different Swift/AppKit bridge shape, so
this experiment should adapt the legacy invariant rather than copying the legacy
XPC design. AppKit already knows the actual overlay frame and backing scale when
`presentTermSurfOverlay` runs. The smallest viable fix is to report the actual
presented overlay pixel size back to Zig and have Zig send Roamium a normal
`Resize` for the mapped browser tab when that actual size differs from the
browser size currently recorded for the pane.

Roamium already has an authoritative resize trace source for verification:
`roamium/src/dispatch.rs:174-199` writes
`resize tab_id=... pixel_width=... pixel_height=... ffi=ts_set_view_size` when
`TERMSURF_PDF_INPUT_TRACE=1` is set. The harness must enable that trace and
parse it from a run-specific `TERMSURF_PDF_INPUT_TRACE_FILE`. AppKit/Zig
requested-size logs alone are not sufficient proof that Roamium applied the
resize.

This experiment is scoped to the initial browser-open viewport-fill row of the
matrix. It should not attempt split, tab, window, or fullscreen behavior yet.

## Changes

Planned files:

- `ghostboard/src/apprt/termsurf.zig`
  - add pane state for the latest actual AppKit overlay pixel size;
  - add a function that receives `pane_id`, `pixel_width`, and `pixel_height`
    from the macOS bridge, snapshots the affected pane/server state under
    `state_mutex`, then releases the mutex before sending any `Resize`;
  - when a browser tab is mapped and the actual AppKit pixel size is known, send
    a `Resize` to Roamium if the actual size differs from the last browser size;
  - suppress unchanged duplicate AppKit-size callbacks so a stable overlay does
    not loop continuously on identical resizes;
  - keep fallback `CreateTab` sizes as a cold-start fallback only, unless the
    actual AppKit size is already known;
  - add geometry trace records for the AppKit-size callback and any corrective
    resize it triggers.
- `ghostboard/src/main_c.zig`
  - export this exact C ABI function:

    ```zig
    pub export fn termsurf_overlay_presented_pixels(
        pane_id: [*:0]const u8,
        pixel_width: u64,
        pixel_height: u64,
    ) void
    ```

    It forwards the callback into `src/apprt/termsurf.zig`.

- `ghostboard/macos/Sources/Ghostty/Surface View/SurfaceView_AppKit.swift`
  - after `presentTermSurfOverlay` computes `frame` and backing scale, calculate
    actual browser viewport pixels from the presented AppKit frame:
    `round(frame.width * backingScale)` and
    `round(frame.height * backingScale)`;
  - call the new Zig callback with the owning pane id and actual pixel size;
  - keep the CALayerHost frame calculation behavior unchanged except where the
    final fix requires a proven geometry correction.
- `scripts/ghostboard-geometry-matrix.sh`
  - strengthen `initial-open` so it fails unless a post-fix run observes:
    - the AppKit-presented pixel size;
    - a Zig corrective resize record when the initial fallback size differs;
    - a Roamium trace record from the run-specific
      `TERMSURF_PDF_INPUT_TRACE_FILE` showing
      `ffi=ts_set_view_size pixel_width=<appkit-pixel-width> pixel_height=<appkit-pixel-height>`;
    - a screenshot where the browser fills the visible viewport area.
- `issues/0809-ghostboard-viewport-geometry/02-initial-viewport-fill.md`
  - record the design, reference audit, implementation, verification, review,
    result, and conclusion.
- `issues/0809-ghostboard-viewport-geometry/README.md`
  - add Experiment 2 to the experiment index.

Reference files:

- `ghostboard-legacy/src/apprt/xpc.zig:280-283`
- `ghostboard-legacy/src/Surface.zig:2506-2515`
- `ghostboard-legacy/src/renderer/generic.zig:849-862`
- `ghostboard/src/apprt/termsurf.zig:1170-1236`
- `ghostboard/macos/Sources/Ghostty/Surface View/SurfaceView_AppKit.swift:527-594`
- `ghostboard/src/main_c.zig:160-230`

## Verification

Pass criteria:

- Markdown is formatted:

  ```bash
  prettier --write --prose-wrap always --print-width 80 \
    issues/0809-ghostboard-viewport-geometry/README.md \
    issues/0809-ghostboard-viewport-geometry/02-initial-viewport-fill.md
  ```

- Zig formatting is run if Zig files are changed:

  ```bash
  cd ghostboard
  zig fmt src/apprt/termsurf.zig src/main_c.zig
  ```

- Swift formatting/linting is run if Swift files are changed:

  ```bash
  cd ghostboard
  swiftlint lint --strict --fix \
    "macos/Sources/Ghostty/Surface View/SurfaceView_AppKit.swift"
  swiftlint lint --strict \
    "macos/Sources/Ghostty/Surface View/SurfaceView_AppKit.swift"
  ```

- The underlying Ghostboard library and macOS app build:

  ```bash
  cd ghostboard
  zig build -Demit-macos-app=false
  macos/build.nu --scheme Ghostty --configuration Debug --action build
  ```

- The `initial-open` harness passes:

  ```bash
  scripts/ghostboard-geometry-matrix.sh initial-open
  ```

- The passing harness run proves all of the following:
  - the screenshot no longer shows unused terminal space to the right or below
    the browser content inside the expected browser viewport;
  - AppKit reports a presented overlay pixel size derived from
    `overlay_frame * backing_scale`;
  - Zig records the AppKit-presented pixel size for the owning pane;
  - Zig sends a corrective `Resize` when the old fallback browser size differs
    from the AppKit-presented pixel size;
  - the run-specific Roamium trace file records `ffi=ts_set_view_size` with the
    same tab id, pane id, pixel width, and pixel height as the AppKit-presented
    size;
  - hit testing still reports `hit=true` and a webview-relative point inside the
    resized overlay.
- `git diff --check` passes.

Fail criteria:

- The fix changes `webtui` or Roamium without evidence that Ghostboard cannot
  correct its own viewport sizing.
- The fix only enlarges the native AppKit frame while leaving the browser
  viewport at the fallback size.
- The fix removes or weakens Experiment 1 geometry correlation.
- The fix depends on the hardcoded fallback `10x20` cell dimensions as the
  normal path.
- The screenshot is visually improved but logs do not prove browser pixel size
  and AppKit-presented pixel size agree.
- A corrective resize loops continuously or sends duplicate resizes with
  unchanged dimensions.

## Design Review

A fresh-context adversarial design reviewer returned **CHANGES REQUIRED** with
one required finding, one optional finding, and one nit:

- **Required:** verification could pass without proving Roamium actually
  resized, because AppKit/Zig `browser_pixel` logs can represent requested size
  rather than a browser-applied viewport.
- **Optional:** callback state and timing were underspecified around
  `state_mutex`, asynchronous AppKit presentation, and duplicate resize
  suppression.
- **Nit:** the exported callback name/signature was unstable because the design
  said "likely".

The design was updated to require the existing Roamium
`TERMSURF_PDF_INPUT_TRACE_FILE` resize trace as the authoritative browser-side
proof, to define the Zig callback lock/snapshot/send sequencing, to suppress
duplicate same-size callbacks, and to name the exact exported C ABI function:
`termsurf_overlay_presented_pixels(pane_id, pixel_width, pixel_height)`.

A focused re-review returned **APPROVED**. The reviewer confirmed the
browser-side proof, callback sequencing, duplicate suppression requirement, and
stable C ABI signature all resolved the prior findings. No new required findings
were reported.
