# Experiment 25: Mouse Input After Geometry

## Description

Experiment 24 proved that DevTools split overlays follow normal browser overlay
presentation, resize, mouse, focus, and keyboard routing rules. The next matrix
row is mouse input after geometry changes.

Many earlier experiments already clicked after a specific geometry transition,
but this row should be proven directly as a durable mouse-input regression
guard. The goal is not to repeat every viewport scenario. The goal is to prove
that after representative geometry changes, Ghostboard derives browser
hit-testing and Roamium mouse coordinates from the current AppKit overlay frame,
not from stale pre-change geometry.

This experiment should focus on one browser window and one normal browser pane.
It should drive a small set of high-signal geometry changes that cover the main
coordinate failure modes:

- window resize larger and smaller, where the pane keeps ownership but its
  root/window dimensions change;
- split-right, where the browser frame shrinks and a sibling pane appears;
- split-right divider resize or equalize, where the browser frame changes again
  without reopening the browser;
- TUI overlay shrink/reset, where the browser overlay frame changes inside the
  same terminal pane.

For each transition, the harness must click inside the current overlay frame and
prove both:

- AppKit hit testing reports the current post-change overlay frame and
  webview-relative point;
- Roamium receives a mouse event for the owning browser tab/pane with
  coordinates matching the AppKit webview-relative point for that same
  post-change click.

It must also click at at least one former/stale coordinate after a shrink/split
and prove that the stale region no longer routes to the browser. If current
Ghostboard already passes, the experiment should record that and avoid product
changes. If it fails, the harness must first localize whether the stale state is
in AppKit hit testing, Zig routing, or Roamium event delivery before any product
fix is designed.

## Changes

Planned files:

- `scripts/ghostboard-geometry-matrix.sh`
  - add a `mouse-after-geometry-change` scenario, or extend existing reusable
    helpers if the scenario can be implemented without duplicating large blocks;
  - launch one normal browser in one Ghostboard window using the repo-built
    `web` and Roamium binaries;
  - record the baseline identity tuple, AppKit frame/pixels, backing scale, and
    Roamium tab/pane ids;
  - perform the representative geometry transitions listed in the Description;
  - after each transition, compute a click point from the current AppKit overlay
    frame and window/content offset;
  - require AppKit `hit_test` evidence with the current overlay frame and
    webview-relative point;
  - require Roamium `mouse-event` evidence for the same browser tab/pane after
    each click;
  - parse the AppKit `web_point` and matching Roamium `mouse-event coords=(x,y)`
    for each post-change click and compare them within a small documented
    tolerance, expected to be no more than one CSS pixel to account for integer
    rounding between AppKit doubles and Chromium integer event coordinates;
  - require the matching Roamium event type/button evidence for at least the
    click down event, so a hover-only mouse move cannot satisfy click delivery;
  - require at least one stale post-shrink/post-split coordinate to produce no
    browser mouse event;
  - capture screenshots for the baseline and final changed state;
  - fail if a mouse assertion accepts pre-transition hit-test or Roamium records
    as post-transition proof.
- `roamium/src/dispatch.rs`
  - change only if existing trace output is insufficient to distinguish mouse
    event tab id, pane id, coordinates, button, and event type;
  - any change must be trace-only under the existing trace mechanism.
- `ghostboard/src/apprt/termsurf.zig`
  - change only if runtime evidence proves Zig input routing uses stale or wrong
    pane state after a geometry change.
- `ghostboard/macos/Sources/Ghostty/Surface View/SurfaceView_AppKit.swift`
  - change only if runtime evidence proves AppKit hit testing uses stale or
    wrong overlay frames after a geometry change.
- `issues/0809-ghostboard-viewport-geometry/25-mouse-input-after-geometry.md`
  - record the design review, implementation, verification, completion review,
    result, and conclusion.
- `issues/0809-ghostboard-viewport-geometry/README.md`
  - add Experiment 25 to the experiment index.

Reference files:

- `scripts/ghostboard-geometry-matrix.sh`
- `scripts/ghostty-app/inject.swift`
- `ghostboard/src/apprt/termsurf.zig`
- `ghostboard/macos/Sources/Ghostty/Surface View/SurfaceView_AppKit.swift`
- `roamium/src/dispatch.rs`
- `issues/0809-ghostboard-viewport-geometry/03-window-resize-follow.md`
- `issues/0809-ghostboard-viewport-geometry/04-split-right-pane-attachment.md`
- `issues/0809-ghostboard-viewport-geometry/06-split-right-divider-resize.md`
- `issues/0809-ghostboard-viewport-geometry/07-split-right-equalize-rebalance.md`
- `issues/0809-ghostboard-viewport-geometry/21-tui-overlay-resize-command.md`
- `issues/0809-ghostboard-viewport-geometry/24-devtools-split-geometry.md`

## Verification

Pass criteria:

- Markdown is formatted:

  ```bash
  prettier --write --prose-wrap always --print-width 80 \
    issues/0809-ghostboard-viewport-geometry/README.md \
    issues/0809-ghostboard-viewport-geometry/25-mouse-input-after-geometry.md
  ```

- Shell syntax is valid:

  ```bash
  bash -n scripts/ghostboard-geometry-matrix.sh
  ```

- If Rust files are changed:

  ```bash
  cargo fmt
  cargo check -p roamium
  cargo build -p roamium
  ```

- If Zig files are changed:

  ```bash
  cd ghostboard
  zig fmt src/apprt/termsurf.zig
  zig build -Demit-macos-app=false
  ```

- If Swift files are changed:

  ```bash
  cd ghostboard
  macos/build.nu --scheme Ghostty --configuration Debug --action build
  ```

- If the copied Roamium binary is needed by the harness after Rust changes:

  ```bash
  scripts/build.sh roamium
  ```

- The new mouse scenario passes:

  ```bash
  scripts/ghostboard-geometry-matrix.sh mouse-after-geometry-change
  ```

- The passing run proves, after each representative geometry transition:
  - the current AppKit overlay frame is used for hit testing;
  - the webview-relative point is inside the current frame;
  - Roamium receives a click mouse event for the owning browser tab/pane;
  - the Roamium click coordinates match the AppKit `web_point` for the same
    post-change click within the documented tolerance;
  - stale coordinates outside the current frame do not route to the browser;
  - no assertion accepts records from before the transition boundary.
- Adjacent geometry regressions still pass:

  ```bash
  scripts/ghostboard-geometry-matrix.sh split-right
  scripts/ghostboard-geometry-matrix.sh tui-overlay-resize-command
  ```

- `git diff --check` passes.
- The design review is recorded in this experiment file and the plan is
  committed before implementation begins.
- After implementation, verification, and result recording, the completion
  review is recorded in this experiment file and the result commit is made
  before designing or implementing Experiment 26.

Fail criteria:

- The scenario only proves AppKit hit testing but not Roamium mouse event
  delivery.
- The scenario proves Roamium event delivery but does not compare Roamium event
  coordinates to the AppKit webview-relative point for the same click.
- The scenario accepts pre-change hit-test or mouse-event logs as post-change
  proof.
- Stale post-shrink or post-split coordinates still route to the browser.
- The experiment expands into keyboard routing, DevTools behavior, tab/window
  switching, final matrix regression, or unrelated product changes before mouse
  input after geometry changes is isolated.

## Design Review

Fresh-context adversarial design review initially returned **CHANGES REQUIRED**.

Required finding:

- The design required AppKit `web_point` evidence and separate Roamium mouse
  event delivery evidence, but did not require the Roamium event coordinates to
  match the AppKit webview-relative point for the same post-change click.

Fix:

- The design now requires parsing AppKit `web_point`, parsing Roamium
  `mouse-event coords=(x,y)`, comparing them within a documented one CSS pixel
  tolerance, and requiring click down/button evidence so hover-only movement
  cannot satisfy click delivery.

Fresh-context adversarial re-review returned **APPROVED**.

Findings: none.
