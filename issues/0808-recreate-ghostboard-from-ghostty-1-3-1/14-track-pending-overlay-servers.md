# Experiment 14: Track Pending Overlay Servers

## Description

Experiment 13 made `ServerRegister` explicit, but because Ghostboard has no
pending server records yet every browser registration correctly reports
`no matching server`. The next step is to create the minimal in-memory state
that Wezboard has before a browser process registers.

In Wezboard, `SetOverlay` creates pane state and a server record keyed by
`profile + browser`. Later, `ServerRegister` matches a browser connection to a
server record with the same profile whose transport is not attached yet.

This experiment will add the same state transition in Ghostboard without
launching a browser process and without sending `CreateTab` yet:

1. A TUI sends `SetOverlay`.
2. Ghostboard records pane metadata from the overlay.
3. Ghostboard creates or reuses a pending server record for `profile + browser`.
4. A browser-classified socket sends `ServerRegister`.
5. Ghostboard matches that registration to the pending server by profile and
   marks the server as attached.

This is the smallest useful server-registry step after Experiment 13. It proves
that Ghostboard can remember TUI overlay intent and later associate a browser
connection with it, while keeping browser process launch, `CreateTab`,
`TabReady`, `BrowserReady`, CALayerHost presentation, and input forwarding out
of scope.

## Changes

- `ghostboard/src/apprt/termsurf.zig`
  - add small process-local state for panes and pending browser servers;
  - protect that state with a mutex;
  - add an explicit `SetOverlay` branch in `handleClient`;
  - default an empty `SetOverlay.browser` to `roamium`, matching Wezboard;
  - store the overlay's `pane_id`, `profile`, `browser`, `url`, terminal-cell
    geometry, and browsing flag;
  - if `pane_id` already exists, update that pane's overlay metadata without
    creating a new pane and without incrementing server `pane_count`, matching
    Wezboard's resize/update path;
  - if `pane_id` is new, create a pending server record when `profile + browser`
    is new, or increment the existing server's `pane_count` only when the new
    pane attaches to an already-existing server;
  - update `ServerRegister` handling so a matching pending server is marked
    attached and logs the matched server key;
  - keep the Experiment 13 unmatched warning for registrations that still have
    no pending server.

No changes will be made to `webtui`, `roamium`, `proto/termsurf.proto`,
branding, app config paths, icon assets, Xcode project files, CLI install
behavior, browser launch, `CreateTab`, `TabReady`, `BrowserReady`, overlay
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
  - a TUI `SetOverlay` with `pane_id=pane-a`, `profile=default`, empty browser,
    and a URL logs `SetOverlay: pane_id=pane-a profile=default browser=roamium`;
  - that `SetOverlay` creates a pending server record for
    `profile=default browser=roamium`;
  - a second `SetOverlay` for the same `pane_id=pane-a` logs an update path,
    does not create a second pane, and does not increment server `pane_count`;
  - a `SetOverlay` for a new `pane_id=pane-b` with the same profile and browser
    reuses the pending server and increments `pane_count`;
  - a later browser-classified `ServerRegister(profile=default)` logs
    `ServerRegister: matched server key=default/roamium`;
  - that matched registration does not log
    `ServerRegister: no matching server for profile=default`;
  - a standalone `ServerRegister(profile=other)` still logs the unmatched
    warning;
  - no `CreateTab`, `BrowserReady`, `TabReady`, or overlay presentation logs are
    emitted by this experiment.
- The runtime harness also sends a normal TUI `HelloRequest` on a fresh socket
  and receives `HelloReply`, proving the new state path did not break existing
  request/reply behavior.
- The harness verifies shutdown cleanup still removes the socket file and leaves
  no stale `TermSurf.app/Contents/MacOS/termsurf` process.
- `git diff --check` is clean.

Fail criteria:

- `SetOverlay` is still handled only by the generic ignored-message branch.
- Empty `SetOverlay.browser` does not default to `roamium`.
- `ServerRegister(profile=default)` cannot match a prior pending server for the
  same profile.
- A duplicate `SetOverlay` for the same `pane_id` creates a duplicate pane or
  increments `pane_count`.
- The implementation sends `CreateTab`, launches a browser process, or creates
  overlay UI in this experiment.
- Browser/TUI classification or the synchronous request/reply paths from
  Experiments 8 through 13 regress.
- Any `webtui`, `roamium`, protocol schema, app branding, config path, icon, or
  CLI install behavior changes are needed for this experiment.

## Design Review

A fresh-context adversarial design review returned **CHANGES REQUIRED**.

Required finding accepted and fixed: the original design did not distinguish
between a duplicate `SetOverlay` for an existing `pane_id` and a new pane that
reuses an existing server. That could overcount panes and diverge from
Wezboard's resize/update behavior. The design now requires existing `pane_id`
messages to update pane metadata without creating a new pane or incrementing
`pane_count`, and it adds runtime pass/fail checks for duplicate `SetOverlay`.

Optional finding accepted and fixed: the design now lists the exact native
GhosttyKit and macOS app build commands in the verification section.

Fresh-context adversarial re-review returned **APPROVED**. The reviewer
confirmed that the duplicate-pane update path, duplicate `SetOverlay` runtime
checks, duplicate-pane fail criterion, and exact build commands resolve the
prior findings without introducing new required issues.
