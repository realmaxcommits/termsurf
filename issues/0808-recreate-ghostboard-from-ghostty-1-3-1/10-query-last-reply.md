# Experiment 10: Reply To QueryLastRequest

## Description

Experiment 9 added the second synchronous request/reply path by answering
`QueryTabsRequest` with an empty successful tab inventory. The next smallest
`webtui` request/reply path is `QueryLastRequest`, used by `web last` to ask the
GUI for the last active browser pane.

Ghostboard does not yet create browser panes, launch Roamium, track tab ids, or
record a last active browser pane. Therefore this experiment should implement
only the baseline no-state reply that Wezboard returns before any browser pane
exists: a `QueryLastReply` with default `pane_id`, `tab_id`, and `profile`, and
`error = "No browser pane yet"`.

This advances protocol parity without inventing browser state early. Later
experiments can replace this no-state reply with real pane tracking once
`SetOverlay`, browser launch, `ServerRegister`, and `TabReady` are implemented.

## Changes

- `ghostboard/src/apprt/termsurf.zig`
  - recognize `QueryLastRequest` in the decoded `TermSurfMessage` switch;
  - log the request's `pane_id` and `profile`;
  - send a length-prefixed `QueryLastReply` whose `error` field is
    `No browser pane yet`;
  - add `QueryLastRequest` and `QueryLastReply` to the TermSurf message type
    name helper used by decoded-message logs.

No changes will be made to `webtui`, `roamium`, `proto/termsurf.proto`,
branding, app config paths, icon assets, Xcode project files, or CLI install
behavior.

## Verification

Pass criteria:

- `zig fmt src/apprt/termsurf.zig src/main_c.zig src/build/SharedDeps.zig`
  passes inside `ghostboard/`.
- The native GhosttyKit framework build passes.
- The macOS app build passes.
- Runtime harness launches `TermSurf.app`, connects to `TERMSURF_SOCKET`, sends
  a length-prefixed current-schema `QueryLastRequest`, and decodes a
  length-prefixed `QueryLastReply`.
- The decoded reply has empty `pane_id`, `tab_id = 0`, empty `profile`, and
  `error = "No browser pane yet"`.
- The runtime harness also sends `HelloRequest` and `QueryTabsRequest` to prove
  Experiments 8 and 9 still work on the same socket implementation.
- The app log contains `TermSurf message decoded type=QueryLastRequest` and a
  reply-sent log for `QueryLastReply`.
- Shutdown cleanup still removes the socket file and leaves no stale
  `TermSurf.app/Contents/MacOS/termsurf` process.
- `git diff --check` is clean.

Fail criteria:

- `QueryLastRequest` is ignored or returns no frame.
- The reply has the wrong oneof message type.
- The reply falsely reports a pane id, tab id, or profile before Ghostboard has
  implemented browser pane state.
- The reply omits the no-state error string.
- Any `webtui`, `roamium`, protocol schema, app branding, config path, icon, or
  CLI install behavior changes are needed for this experiment.

## Design Review

Fresh-context adversarial design review returned `CHANGES REQUIRED`.

Required finding accepted and fixed:

- The initial plan omitted the `msgTypeName` update needed by the planned log
  verification. Without explicit `QueryLastRequest` and `QueryLastReply` message
  names, the decoded-message log would report `Other`, while the verification
  expected `TermSurf message decoded type=QueryLastRequest`. The plan now
  includes the required message-name helper update.

Re-review returned `APPROVED`. The reviewer confirmed the prior finding is
resolved and that no new required finding was introduced by the fix.
