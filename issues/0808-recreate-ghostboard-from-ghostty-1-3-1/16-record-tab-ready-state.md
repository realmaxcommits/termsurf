# Experiment 16: Record TabReady State

## Description

Experiment 15 sends pending `CreateTab` messages to a browser socket after
`ServerRegister` attaches. The browser's next response is `TabReady`, which
assigns the real browser tab id for a pane.

In Wezboard, `TabReady`:

- finds the pane by `pane_id`;
- stores `tab_id` on that pane;
- records a `(server_key, tab_id) -> pane_id` lookup;
- updates `last_browser_pane` for non-DevTools panes;
- then sends `BrowserReady` to the TUI if a browser listen socket is known.

Ghostboard does not yet track TUI write channels or browser listen sockets, so
this experiment will implement the state update only. `BrowserReady` remains out
of scope until the app has enough TUI/socket state to send it correctly.

The practical effect is important now: once a pane has a nonzero `tab_id`, a
later `ServerRegister` must not flush another `CreateTab` for that pane.

## Changes

- `ghostboard/src/apprt/termsurf.zig`
  - add bounded tab-to-pane lookup state keyed by `profile/browser/tab_id`;
  - add `last_browser_pane` state;
  - add `TabReady` to the message type name helper;
  - add an explicit `TabReady` branch in `handleClient`;
  - on known `pane_id`, store the nonzero `tab_id`, update the lookup, set
    `last_browser_pane`, and log the exact lookup key/value and lookup count;
  - log that the pane is no longer pending after `tab_id` becomes nonzero, so
    later `CreateTab` flushing can be verified to skip because of pane state
    rather than because registration failed for an unrelated reason;
  - on unknown `pane_id`, log `TabReady: unknown pane_id=...`;
  - leave `BrowserReady`, browser listen socket propagation, overlay
    presentation, and input forwarding out of scope.

No changes will be made to `webtui`, `roamium`, `proto/termsurf.proto`,
branding, app config paths, icon assets, Xcode project files, CLI install
behavior, browser process launch, `BrowserReady`, overlay presentation, or input
forwarding.

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
  - a TUI `SetOverlay` creates a pending `default/roamium` pane;
  - `ServerRegister(profile=default)` receives one decoded `CreateTab`;
  - a later `TabReady(pane_id=pane-a, tab_id=42)` on that browser socket logs
    `TabReady: pane_id=pane-a tab_id=42 tab_to_pane_count=1`;
  - the log shows
    `TabReady lookup: key=default/roamium tab_id=42 pane_id=pane-a` so the
    harness proves the lookup is keyed to the right server and tab id;
  - the log shows `last_browser_pane=pane-a`;
  - the log shows `TabReady pending=false pane_id=pane-a tab_id=42`, proving the
    pane state no longer satisfies the pending `tab_id == 0` condition used by
    `CreateTab` flushing;
  - no additional `sent CreateTab: pane_id=pane-a` log appears after the
    `TabReady pending=false` log;
  - `TabReady(pane_id=missing, tab_id=99)` logs
    `TabReady: unknown pane_id=missing`;
  - no `BrowserReady`, browser process launch, or overlay presentation logs are
    emitted by this experiment.
- The runtime harness also sends a normal TUI `HelloRequest` on a fresh socket
  and receives `HelloReply`, proving existing request/reply behavior still
  works.
- The harness verifies shutdown cleanup still removes the socket file and leaves
  no stale `TermSurf.app/Contents/MacOS/termsurf` process.
- `git diff --check` is clean.

Fail criteria:

- `TabReady` is still handled only by the generic ignored-message branch.
- Known `pane_id` does not get its `tab_id` stored.
- The logged lookup key/value does not prove `default/roamium + 42 -> pane-a`.
- `TabReady` does not log `pending=false` for a pane whose `tab_id` became
  nonzero.
- Another `CreateTab` for the same pane is logged after `pending=false`.
- Unknown `pane_id` does not log a warning.
- The implementation sends `BrowserReady`, launches a browser process, or
  creates overlay UI in this experiment.
- Browser/TUI classification or the synchronous request/reply paths from
  Experiments 8 through 15 regress.
- Any `webtui`, `roamium`, protocol schema, app branding, config path, icon, or
  CLI install behavior changes are needed for this experiment.

## Design Review

A fresh-context adversarial design review returned **CHANGES REQUIRED**.

Required finding accepted and fixed: the original duplicate-`CreateTab`
verification could pass for the wrong reason, because a second browser socket
would fail to attach after the first registration had already set `attached_fd`.
The design now requires a direct `TabReady pending=false` log and checks that no
later `sent CreateTab` for the pane appears after that state transition.

Required finding accepted and fixed: the original lookup verification only
checked `tab_to_pane_count=1`, which would not prove the lookup was keyed
correctly. The design now requires a concrete log proving
`default/roamium + 42 -> pane-a`.

Fresh-context adversarial re-review returned **APPROVED**. The reviewer
confirmed both required findings were resolved and that the fixes introduced no
new required issues.
