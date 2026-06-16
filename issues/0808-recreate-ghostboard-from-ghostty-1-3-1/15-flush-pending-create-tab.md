# Experiment 15: Flush Pending CreateTab

## Description

Experiment 14 created pending pane/server state and matched
`ServerRegister(profile=...)` to an unattached pending server. The next Wezboard
behavior is to flush pending browser tabs to that registered browser connection.

In Wezboard, `handle_server_register` finds a matching server, stores the
browser connection sender, then sends `CreateTab` for each pending non-DevTools
pane whose `profile/browser` matches and whose `tab_id` is still zero.

Ghostboard does not yet launch Roamium. This experiment will still move the real
wire protocol forward by treating the runtime harness as the browser-engine
socket:

1. A TUI socket sends `SetOverlay`.
2. Ghostboard records a pending pane/server.
3. A browser-classified socket sends `ServerRegister`.
4. Ghostboard matches that socket and writes a length-prefixed `CreateTab`
   protobuf frame back to the browser socket.

This experiment must not launch a browser process, handle `TabReady`, send
`BrowserReady`, create overlay UI, or forward input. It only proves that
Ghostboard can deliver the correct browser-directed tab creation message once a
browser connection is attached.

## Changes

- `ghostboard/src/apprt/termsurf.zig`
  - add `CreateTab` to the message type name helper;
  - add `tab_id: i64 = 0` to `PaneState` so Ghostboard can distinguish pending
    panes from panes that will later be assigned a browser tab id by `TabReady`;
  - add a helper that sends a length-prefixed `CreateTab` for one pending pane;
  - after `ServerRegister` matches a pending server, iterate matching panes and
    send `CreateTab` for each pane with `tab_id == 0`;
  - derive `CreateTab.pixel_width` and `CreateTab.pixel_height` from
    `SetOverlay.width` and `SetOverlay.height` using Wezboard's fallback cell
    size of `10x20` pixels because Ghostboard does not yet expose live terminal
    cell metrics to this socket module;
  - keep the pane pending after sending, because `TabReady` is not implemented
    yet and will be responsible for assigning `tab_id`.

No changes will be made to `webtui`, `roamium`, `proto/termsurf.proto`,
branding, app config paths, icon assets, Xcode project files, CLI install
behavior, browser process launch, `TabReady`, `BrowserReady`, overlay
presentation, or input forwarding.

## Verification

Pass criteria:

- `zig fmt src/apprt/termsurf.zig src/main_c.zig src/build/SharedDeps.zig`
  passes inside `ghostboard/`.
- The native GhosttyKit framework build passes:
  `zig build -Demit-xcframework=true -Dxcframework-target=native -Demit-macos-app=false`.
- The macOS app build passes:
  `macos/build.nu --scheme Ghostty --configuration Debug --action build`.
- Runtime harness launches `TermSurf.app`, connects to `TERMSURF_SOCKET`, and
  proves:
  - two TUI `SetOverlay` messages create two pending panes for
    `default/roamium`;
  - one additional TUI `SetOverlay` creates a nonmatching pending pane for a
    different profile or browser;
  - a later browser-classified `ServerRegister(profile=default)` receives a
    length-prefixed `CreateTab` frame for each matching pending pane on that
    browser socket;
  - exactly two `CreateTab` frames are received for the two `default/roamium`
    panes, and the nonmatching pane is not flushed to that socket;
  - each decoded `CreateTab` has the original `url`, `pane_id`, fallback
    `pixel_width = width * 10`, fallback `pixel_height = height * 20`, and
    `dark = false`;
  - the app log contains one `sent CreateTab: pane_id=... url=...` entry for
    each flushed matching pane;
  - no `BrowserReady`, `TabReady`, browser process launch, or overlay
    presentation logs are emitted by this experiment.
- The runtime harness also sends a normal TUI `HelloRequest` on a fresh socket
  and receives `HelloReply`, proving existing request/reply behavior still
  works.
- The harness verifies shutdown cleanup still removes the socket file and leaves
  no stale `TermSurf.app/Contents/MacOS/termsurf` process.
- `git diff --check` is clean.

Fail criteria:

- `ServerRegister` matches but no `CreateTab` frame is sent.
- Fewer or more `CreateTab` frames are flushed than the number of matching
  pending panes.
- A nonmatching pending pane is flushed to the registered browser socket.
- Any `CreateTab` has the wrong oneof type or wrong `pane_id`, `url`,
  dimensions, or dark flag.
- `CreateTab` is sent before `ServerRegister` attaches the browser socket.
- The implementation launches a browser process, sends `BrowserReady`, handles
  `TabReady`, or creates overlay UI in this experiment.
- Browser/TUI classification or the synchronous request/reply paths from
  Experiments 8 through 14 regress.
- Any `webtui`, `roamium`, protocol schema, app branding, config path, icon, or
  CLI install behavior changes are needed for this experiment.

## Design Review

A fresh-context adversarial design review returned **CHANGES REQUIRED**.

Required finding accepted and fixed: the original plan referenced Wezboard's
`tab_id == 0` pending-pane filter but did not plan a `tab_id` field in
Ghostboard's `PaneState`. The plan now explicitly adds `tab_id: i64 = 0`, with
`TabReady` left out of scope except as the future owner of assigning nonzero tab
ids.

Required finding accepted and fixed: the original runtime proof only required
one pending pane and one `CreateTab`, which would not prove Wezboard's
flush-all-matching-pending-panes behavior. The plan now requires two matching
pending panes, one nonmatching pending pane, exactly two decoded `CreateTab`
frames for the matching panes, and proof that the nonmatching pane is not
flushed to that browser socket.

Fresh-context adversarial re-review returned **APPROVED**. The reviewer
confirmed both required findings were resolved and that the fixes introduced no
new required issues.
