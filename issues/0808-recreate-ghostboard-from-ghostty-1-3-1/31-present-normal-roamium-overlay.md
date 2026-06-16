# Experiment 31: Present the Normal Roamium Overlay

## Description

Experiment 30 proved that Ghostboard can launch the real repo-built Roamium
artifact, complete the normal `webtui` lifecycle, receive `CaContext`, and keep
the app and browser process lifecycle clean enough for a smoke test. The next
major parity gap is visual: Ghostboard currently logs browser-originated
messages such as `CaContext`, but it does not attach the Chromium
`CAContext`/`CALayerHost` output to the macOS terminal surface.

This experiment will implement native macOS overlay presentation for the normal
Roamium tab only. It will route
`CaContext(tab_id, ca_context_id, pixel_width, pixel_height)` to the owning
terminal pane, ask the AppKit side to create or update a `CALayerHost`, and
position that layer over the terminal cell rectangle from the latest
`SetOverlay`/`Resize` state.

This experiment intentionally stops at visual presentation. Browser keyboard and
mouse input forwarding, DevTools overlay presentation, shutdown crash cleanup,
and richer page-state UI updates are separate experiments.

## Changes

Expected implementation files:

- `ghostboard/src/apprt/termsurf.zig`
  - handle `TERMSURF__TERM_SURF_MESSAGE__MSG_CA_CONTEXT` instead of only logging
    it;
  - map the browser server plus `tab_id` back to the pane recorded by
    `TabReady`;
  - store the latest `ca_context_id`, browser pixel size, and pending overlay
    frame in normal pane state;
  - call a macOS bridge function after `CaContext`, `SetOverlay`, and `Resize`
    updates so Swift can create or reposition the host layer;
  - log successful and rejected overlay presentation attempts with pane id, tab
    id, context id, pixel size, and frame.
- `ghostboard/macos/Sources/App/macOS/AppDelegate+TermSurf.swift`
  - add a new `@_cdecl` bridge, likely `termsurf_present_overlay`, following the
    existing `termsurf_open_split` pattern;
  - resolve the pane id to `Ghostty.SurfaceView` via `AppDelegate.findSurface`;
  - dispatch AppKit layer mutation to the main queue;
  - log accepted/rejected overlay bridge calls.
- `ghostboard/macos/Sources/Ghostty/Surface View/SurfaceView_AppKit.swift`
  - own the normal browser overlay layers for a surface;
  - create a root `CALayer` if needed, then a flipped/positioning layer and a
    `CALayerHost` with the received `contextId`;
  - update the frame without implicit animations when `SetOverlay` or `Resize`
    changes the terminal-cell rectangle;
  - replace the hosted context safely if Roamium sends a new `CaContext`;
  - tear down overlay layers when the surface deinitializes or the pane clears.

Possible supporting files, only if required by the existing macOS source layout:

- `ghostboard/macos/Sources/Ghostty/Surface View/SurfaceView.swift`
  - expose enough geometry or lifecycle state to position the overlay relative
    to the rendered terminal surface.
- `ghostboard/macos/Sources/Ghostty/Surface View/SurfaceScrollView.swift`
  - update overlay frame when scrolling or visible bounds changes if the
    existing `SurfaceView` frame alone is insufficient.

No changes will be made to `webtui`, `roamium`, Chromium,
`proto/termsurf.proto`, config paths, branding, CLI install behavior, DevTools
behavior, browser input forwarding, or browser shutdown in this experiment.

## Verification

Pass criteria:

- `cargo build -p webtui` passes, with command, cwd, and exit status recorded in
  a log.
- `./scripts/build.sh roamium` passes, with command, cwd, and exit status
  recorded in a log.
- The real browser artifact remains
  `/Users/astrohacker/dev/termsurf/chromium/src/out/Default/roamium`, and the
  runtime harness uses that path rather than `target/debug/roamium`, an
  installed browser, or a fake helper.
- If Zig code is modified, run
  `zig fmt src/apprt/termsurf.zig src/main_c.zig src/build/SharedDeps.zig`
  inside `ghostboard/`, with command, cwd, and exit status recorded in a log.
- If Swift code is modified, run the nested Ghostboard SwiftLint fix and
  non-mutating lint checks for the touched Swift files, with command, cwd, and
  exit status recorded in logs. If SwiftLint cannot run in this environment,
  record the exact failure and run the macOS app build as the required compiler
  check.
- The native GhosttyKit framework build passes:
  `zig build -Demit-xcframework=true -Dxcframework-target=native -Demit-macos-app=false`,
  with command, cwd, and exit status recorded in a log.
- The macOS app build passes:
  `macos/build.nu --scheme Ghostty --configuration Debug --action build`, with
  command, cwd, and exit status recorded in a log.
- Runtime harness launches `TermSurf.app` with `GHOSTTY_LOG=stderr` and a
  temporary config whose command runs:

  ```text
  /Users/astrohacker/dev/termsurf/target/debug/web --browser /Users/astrohacker/dev/termsurf/chromium/src/out/Default/roamium https://example.com
  ```

- Runtime logs still prove the Experiment 30 lifecycle:
  - Ghostboard spawns the Chromium-output Roamium with `--ipc-socket`,
    `--user-data-dir`, and `--listen-socket`;
  - Roamium sends `ServerRegister(profile=default)`;
  - Ghostboard sends `CreateTab`;
  - Roamium sends `TabReady`;
  - Ghostboard sends `BrowserReady`;
  - `webtui` connects to Roamium's direct browser socket;
  - `web last` returns the normal Roamium tab.
- Runtime logs prove the new overlay path:
  - Ghostboard receives `CaContext` with nonzero `ca_context_id`;
  - Ghostboard maps the `CaContext` tab id to the normal pane id;
  - Ghostboard calls the macOS overlay bridge with pane id, context id, and a
    nonzero frame;
  - AppKit creates or updates a `CALayerHost` with that context id;
  - AppKit positions the host over the expected terminal-cell rectangle.
- Visual verification is mandatory. Capture a screenshot of the launched app and
  prove that browser content is visible inside the terminal pane. The screenshot
  check should be automated if possible:
  - capture the app window after `TitleChanged("Example Domain")` or after
    `LoadingState` completes;
  - crop or inspect the expected overlay rectangle;
  - require non-terminal browser pixels or recognizable Example Domain content
    in that rectangle.
- If automated screenshot validation is not reliable in this macOS VM, the
  result must include recorded manual screenshot inspection with the screenshot
  path, the inspected rectangle, and a pass/fail statement that recognizable
  `Example Domain` content or non-terminal browser content is visible inside the
  expected overlay rectangle. App logs proving `CALayerHost` creation are
  necessary but not sufficient for this experiment to pass.
- Runtime cleanup leaves no stale matching
  `TermSurf.app/Contents/MacOS/termsurf`, `target/debug/web`, or
  `chromium/src/out/Default/roamium` processes, and removes the GUI socket.
- `git diff --check` is clean.
- `git diff --name-only` or `git diff --stat` is recorded, and the experiment
  fails if the implementation changes any forbidden path: `webtui/`, `roamium/`,
  `chromium/`, or `proto/termsurf.proto`.

Fail criteria:

- The runtime uses a fake helper, installed browser, or `target/debug/roamium`
  instead of `/Users/astrohacker/dev/termsurf/chromium/src/out/Default/roamium`.
- `webtui`, `roamium`, Chromium, or `proto/termsurf.proto` are modified to make
  visual presentation work.
- `CaContext` is received but not mapped to the pane that owns the normal tab.
- The AppKit bridge mutates layers off the main thread.
- A `CALayerHost` is created without a nonzero `contextId`.
- The overlay frame ignores the `SetOverlay`/`Resize` cell rectangle or is not
  updated when overlay geometry changes.
- The implementation breaks the Experiment 30 normal Roamium lifecycle.
- The experiment adds browser keyboard/mouse forwarding, DevTools overlay
  presentation, browser shutdown fixes, Chromium changes, `webtui` changes,
  `roamium` changes, or protobuf schema changes.

## Design Review

A fresh-context adversarial Codex subagent reviewed the Experiment 31 design and
returned **CHANGES REQUIRED** with two required findings:

- visual verification could pass with logs only, which would not prove that
  Chromium pixels are visible inside the terminal pane;
- the hygiene checks did not explicitly require `git diff --name-only` or
  `git diff --stat` to prove forbidden paths were untouched.

Both findings were accepted. The design now makes visual proof mandatory through
automated screenshot validation or recorded manual screenshot inspection with
explicit pass/fail criteria, and it requires a recorded diff-name or diff-stat
check that fails if `webtui/`, `roamium/`, `chromium/`, or
`proto/termsurf.proto` changed.

The same reviewer re-reviewed the fixes and returned **APPROVED**. The reviewer
confirmed that visual proof is now mandatory, logs are explicitly necessary but
not sufficient, the forbidden-path diff check is required, no new required
findings were introduced, and the issue README still links Experiment 31 as
`Designed`.
