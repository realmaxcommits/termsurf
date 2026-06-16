# Experiment 28: Validate Webtui DevTools Flow

## Description

Experiments 25 through 27 implemented the protocol pieces needed for DevTools:

- `QueryDevtoolsRequest` can resolve an existing browser tab.
- `SetDevtoolsOverlay` can create a browser-side DevTools tab for an attached
  browser server.
- `TERMSURF_PANE_ID` is propagated into terminal child processes.
- `OpenSplit` can create a native Ghostboard split running a requested command.

Those experiments proved the individual protocol pieces with direct socket
harnesses. The remaining question is whether the real `webtui` binary can drive
the whole workflow inside Ghostboard without any changes to `webtui` or
`roamium`.

This experiment will launch the actual `target/debug/web` binary inside
`TermSurf.app`, using a fake browser helper only in place of Roamium. The helper
will connect as a browser server, receive the normal `CreateTab`, send
`TabReady(tab_id=42)`, and keep the browser socket open. The harness will then
drive the normal `webtui` command flow by sending the user-level `:devtools`
command into the terminal. The expected chain is:

```text
webtui normal pane
  -> QueryDevtoolsRequest(tab_id=42)
  -> OpenSplit(command="<same web binary> --browser <helper> --profile default devtools://42")
  -> Ghostboard native split
  -> webtui DevTools pane
  -> QueryDevtoolsRequest(tab_id=42)
  -> SetDevtoolsOverlay(inspected_tab_id=42)
  -> CreateDevtoolsTab(inspected_tab_id=42)
  -> TabReady(devtools-pane, tab_id=99)
  -> BrowserReady(devtools-pane, tab_id=99)
```

If this flow fails, this experiment may make the smallest necessary Ghostboard
fix to make the existing `webtui` and helper-compatible browser behavior work.
It will not change `webtui`, `roamium`, or `proto/termsurf.proto`.

## Changes

Expected code changes are none unless the runtime validation discovers a
Ghostboard-side defect.

If a fix is needed, keep it limited to the smallest relevant Ghostboard files,
likely one of:

- `ghostboard/src/apprt/termsurf.zig`
  - protocol state, routing, query, `SetDevtoolsOverlay`, or logging fixes;
- `ghostboard/macos/Sources/App/macOS/AppDelegate+TermSurf.swift`
  - native split command propagation fixes;
- `ghostboard/macos/Sources/Ghostty/Surface View/SurfaceView_AppKit.swift`
  - pane environment propagation fixes.

No changes will be made to `webtui`, `roamium`, `proto/termsurf.proto`,
branding, icon assets, Xcode project files, CLI install behavior, native browser
overlay presentation, keyboard/mouse browser input forwarding, browser process
lifecycle, or DevTools duplicate detection in this experiment.

## Verification

Pass criteria:

- Build the real `webtui` binary with `cargo build -p webtui`, with the command,
  cwd, and exit status recorded in a log.
- If Rust code is modified, run `cargo fmt` as required by `AGENTS.md`. If no
  Rust code is modified, explicitly record that no Rust formatting was required.
- If Zig code is modified, run
  `zig fmt src/apprt/termsurf.zig src/main_c.zig src/build/SharedDeps.zig`
  inside `ghostboard/`, with the command, cwd, and exit status recorded in a
  log.
- If Swift code is modified, run the nested Ghostboard `swiftlint` fix and
  non-mutating lint checks for touched Swift files, with commands, cwd, and exit
  statuses recorded in logs.
- If Ghostboard code is modified, the native GhosttyKit framework build passes:
  `zig build -Demit-xcframework=true -Dxcframework-target=native -Demit-macos-app=false`,
  with the command, cwd, and exit status recorded in a log.
- The macOS app build passes:
  `macos/build.nu --scheme Ghostty --configuration Debug --action build`, with
  the command, cwd, and exit status recorded in a log.
- Runtime harness launches `TermSurf.app` with a temporary config whose command
  runs the actual `target/debug/web --browser <helper> https://example.com`
  inside the first terminal surface.
- The fake browser helper:
  - receives Ghostboard's browser launch arguments including `--ipc-socket` and
    `--listen-socket`;
  - listens on the requested `--listen-socket`;
  - connects back with `ServerRegister(profile=default)`;
  - receives `CreateTab` for the normal webtui pane;
  - sends `TabReady(normal-pane, tab_id=42)`;
  - accepts the normal webtui process's direct browser connection on
    `--listen-socket` after `BrowserReady(tab_id=42)`;
  - later receives `CreateDevtoolsTab(inspected_tab_id=42)` for the DevTools
    pane;
  - sends `TabReady(devtools-pane, tab_id=99)`;
  - accepts the DevTools webtui process's direct browser connection on
    `--listen-socket` after `BrowserReady(tab_id=99)`.
- The normal `webtui` process receives `BrowserReady(tab_id=42)` and becomes
  ready enough for the `:devtools` command to proceed. This can be proven by the
  following downstream events rather than by screen scraping.
- The harness sends the literal user command `:devtools` or `:devtools right`
  into the normal webtui terminal using the same System Events keyboard
  automation proven in Experiment 26. Keyboard automation is allowed only for
  simulating the user's command entry; the split itself must be caused by
  `OpenSplit`.
- App logs show `QueryDevtoolsRequest` for `tab_id=42` from the normal pane.
- App logs show `OpenSplit` for the normal pane and a successful native split
  bridge log.
- The DevTools split launches the actual `target/debug/web` binary with
  `devtools://42`, `--browser <helper>`, and `--profile default`.
- App logs show the DevTools pane sends
  `SetDevtoolsOverlay(inspected_tab_id=42)`.
- The helper receives `CreateDevtoolsTab(inspected_tab_id=42)` for the DevTools
  pane.
- The DevTools webtui process receives `BrowserReady(tab_id=99)`, proven by the
  app log line for `BrowserReady: pane_id=<devtools-pane> tab_id=99`.
- `QueryLastRequest(profile=default)` still returns the normal browser pane with
  `tab_id=42` after the DevTools pane becomes ready.
- Runtime shutdown removes the GUI socket file and leaves no stale matching
  `TermSurf.app/Contents/MacOS/termsurf`, `target/debug/web`, or fake helper
  processes.
- `git diff --check` is clean.

Fail criteria:

- The real `webtui` binary is not used for the normal pane.
- The DevTools split does not launch the real `webtui` binary.
- The harness sends `OpenSplit` directly instead of issuing the user-level
  `:devtools` command to the normal `webtui`.
- The split is created by System Events keyboard shortcuts instead of Ghostboard
  handling `OpenSplit`.
- `webtui` sends `QueryDevtoolsRequest` but receives an error for an existing
  attached tab.
- `OpenSplit` is not emitted by `webtui` or not handled by Ghostboard.
- The DevTools split launches but does not send `SetDevtoolsOverlay`.
- The browser helper does not receive `CreateDevtoolsTab`.
- DevTools `TabReady` makes `QueryLastRequest(profile=default)` return the
  DevTools pane instead of the normal pane.
- The implementation changes `webtui`, `roamium`, `proto/termsurf.proto`, Xcode
  project files, native browser overlay presentation, browser input forwarding,
  browser process lifecycle, or DevTools duplicate detection in this experiment.

## Design Review

A fresh-context adversarial Codex subagent reviewed the Experiment 28 design and
returned **APPROVED** with one optional finding: the runtime verification should
cover Ghostboard's `--listen-socket` browser launch argument and the direct
browser connections that real `webtui` opens after `BrowserReady`.

The design was updated to require the helper to verify both `--ipc-socket` and
`--listen-socket`, listen on the browser socket, and accept the normal and
DevTools webtui direct browser connections.
