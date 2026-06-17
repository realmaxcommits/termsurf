# Experiment 24: DevTools Split Geometry

## Description

Experiment 23 proved that browser navigation does not reset or corrupt overlay
geometry. The next matrix row is DevTools split or tab.

Opening DevTools should create a second browser-backed overlay that inspects the
original browser tab without stealing or corrupting the original browser pane's
geometry. The normal browser overlay should remain attached to its original
pane. The DevTools overlay should attach to its new split pane, carry the
inspected normal browser tab id, receive its own native context, and follow the
same AppKit frame/pixel, Roamium resize, mouse hit-test, and keyboard routing
rules as a normal browser overlay.

This experiment should isolate one normal browser pane and one DevTools split
created from that pane through public `webtui` behavior. It must use command
mode `:devtools right` or `:de right` from the normal browser pane. It must not
launch a standalone `devtools://<tab>` TUI directly, inject private protocol
messages, change windows, change terminal tabs, use final matrix regression, or
combine this with unrelated DevTools product features. If current Ghostboard
already passes, the experiment should record that and avoid product changes. If
it fails, the harness must first localize whether the failure is command
dispatch, `QueryDevtoolsRequest`/reply, split creation, `SetDevtoolsOverlay`,
`CreateDevtoolsTab`, AppKit frame/pixel state, Roamium resize/focus, or input
routing before any product fix is designed.

## Changes

Planned files:

- `scripts/ghostboard-geometry-matrix.sh`
  - add a `devtools-split-geometry` scenario;
  - launch one normal browser in one Ghostboard window using the repo-built
    `web` and Roamium binaries;
  - record the normal browser baseline canonical identity tuple:
    `window_id + surface_id + selected_tab_id + pane_id + browser_tab_id`, plus
    `context_id + grid + cell size + AppKit frame + AppKit pixels + backing_scale`;
  - enter command mode using real keyboard input, type `devtools right`, and
    submit it;
  - wait for public split creation evidence and fresh DevTools-specific protocol
    evidence after the command boundary: `QueryDevtoolsRequest`,
    `QueryDevtoolsReply`, `OpenSplit`, `SetDevtoolsOverlay`,
    `CreateDevtoolsTab`, and Roamium DevTools tab creation/resize evidence where
    available;
  - identify the DevTools pane id, inspected normal browser tab id, DevTools
    browser tab id, DevTools context id, AppKit frame, AppKit pixels, grid, and
    backing scale;
  - require the normal browser overlay to remain mapped to the original pane and
    browser tab, with the expected split-resized frame/pixels;
  - require the DevTools overlay to be mapped to the new split pane and to
    inspect the original normal browser tab id;
  - require both overlays to have distinct pane ids and distinct native context
    ids;
  - require Roamium resize evidence for both the normal browser and DevTools
    views when the split changes their pixel sizes;
  - capture screenshots after DevTools opens;
  - click inside the normal browser pane and prove hit testing routes to the
    normal context and not the DevTools context;
  - click inside the DevTools pane and prove hit testing routes to the DevTools
    context and not the normal context;
  - enter Browse mode in the DevTools pane and prove keyboard input reaches the
    DevTools browser tab/pane only;
  - return to the normal pane, enter Browse mode, and prove keyboard input
    reaches the normal browser tab/pane only;
  - fail if assertions accept pre-DevTools records as post-DevTools proof.
- `roamium/src/dispatch.rs`
  - change only if existing trace output cannot distinguish DevTools tab
    creation, inspected tab id, DevTools tab id, resize, focus, mouse, or key
    routing;
  - any change must be trace-only under the existing trace mechanism.
- `ghostboard/src/apprt/termsurf.zig`
  - change only if runtime evidence proves Ghostboard needs additional
    scenario-gated geometry trace for DevTools overlay creation or mapping;
  - any behavior fix must be preceded by logs that localize the failure.
- `ghostboard/macos/Sources/Ghostty/Surface View/SurfaceView_AppKit.swift`
  - change only if runtime evidence proves AppKit frame/pixel or hit-test state
    is wrong for normal or DevTools overlays after the split.
- `webtui/src/main.rs`
  - change only if runtime evidence proves the public `:devtools` command cannot
    be automated deterministically or sends incomplete protocol data.
- `issues/0809-ghostboard-viewport-geometry/24-devtools-split-geometry.md`
  - record the design review, implementation, verification, completion review,
    result, and conclusion.
- `issues/0809-ghostboard-viewport-geometry/README.md`
  - add Experiment 24 to the experiment index.

Reference files:

- `scripts/ghostboard-geometry-matrix.sh`
- `scripts/ghostty-app/inject.swift`
- `webtui/src/main.rs`
- `webtui/src/ipc.rs`
- `roamium/src/dispatch.rs`
- `ghostboard/src/apprt/termsurf.zig`
- `ghostboard/macos/Sources/Ghostty/Surface View/SurfaceView_AppKit.swift`
- `issues/0809-ghostboard-viewport-geometry/23-browser-navigation-geometry.md`
- `issues/0809-ghostboard-viewport-geometry/13-open-browser-in-new-tab.md`
- `issues/0809-ghostboard-viewport-geometry/04-split-right-pane-attachment.md`

## Verification

Pass criteria:

- Markdown is formatted:

  ```bash
  prettier --write --prose-wrap always --print-width 80 \
    issues/0809-ghostboard-viewport-geometry/README.md \
    issues/0809-ghostboard-viewport-geometry/24-devtools-split-geometry.md
  ```

- Shell syntax is valid:

  ```bash
  bash -n scripts/ghostboard-geometry-matrix.sh
  ```

- If Rust files are changed:

  ```bash
  cargo fmt
  cargo check -p webtui
  cargo check -p roamium
  cargo build -p webtui
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

- The new scenario passes:

  ```bash
  scripts/ghostboard-geometry-matrix.sh devtools-split-geometry
  ```

- The passing run proves:
  - DevTools is opened through public `webtui` command mode using real keyboard
    input;
  - the normal browser keeps the same original browser tab id and remains mapped
    to the original pane;
  - the DevTools overlay appears in the new split pane and carries the inspected
    original browser tab id;
  - normal and DevTools overlays have distinct pane ids and context ids;
  - AppKit frame, AppKit pixels, backing scale, and Roamium view size agree for
    both overlays after the split;
  - mouse hit testing in each pane routes only to that pane's overlay context;
  - Browse-mode keyboard input in each pane routes only to that pane's browser
    tab/pane;
  - screenshots show the normal browser and DevTools split state.
- Adjacent geometry regressions still pass:

  ```bash
  scripts/ghostboard-geometry-matrix.sh browser-navigation-geometry
  scripts/ghostboard-geometry-matrix.sh split-right
  ```

- `git diff --check` passes.
- The design review is recorded in this experiment file and the plan is
  committed before implementation begins.
- After implementation, verification, and result recording, the completion
  review is recorded in this experiment file and the result commit is made
  before designing or implementing Experiment 25.

Fail criteria:

- The harness fakes DevTools by launching a standalone `devtools://<tab>` TUI
  instead of invoking the normal browser pane's public DevTools command.
- DevTools creation cannot be proven with fresh evidence after the command
  boundary.
- The normal browser loses its original pane, browser tab, context, or expected
  split geometry.
- The DevTools overlay is missing, attached to the wrong pane, lacks an
  inspected original tab id, or shares the normal browser context id.
- Mouse or keyboard input after DevTools opens reaches the wrong overlay, both
  overlays, no overlay, or stale coordinates.
- The experiment expands into terminal tabs, multiple windows, DevTools feature
  testing, browser navigation, scrollback movement, or final matrix regression
  before DevTools split geometry is isolated.

## Design Review

Fresh-context adversarial design review returned **APPROVED**.

Findings: none.
