# Experiment 11: Reply To QueryDevtoolsRequest

## Description

Experiments 8 through 10 established Ghostboard's TermSurf socket and the
no-state synchronous request/reply paths for `HelloRequest`, `QueryTabsRequest`,
and `QueryLastRequest`. The remaining synchronous query that `webtui` can send
before normal TUI startup is `QueryDevtoolsRequest`, used when opening a
DevTools TUI for an existing browser tab.

Ghostboard does not yet launch Roamium, create browser panes, track browser
profiles, track tab ids, or maintain DevTools panes. Therefore this experiment
will implement only the baseline validation/error behavior that is correct
before browser state exists.

The reply should follow Wezboard's validation order:

- if `browser` is empty, return `error = "DevTools target browser is required"`;
- else if `profile` is empty, return
  `error = "DevTools target profile is required"`;
- else if `inspected_tab_id == 0`, return
  `error = "DevTools target tab id is required"`;
- otherwise return
  `error = "Inspected tab {id} not found in {browser}/{profile}"`.

All success fields should remain at protobuf defaults because there is no
browser/tab state to resolve yet.

## Changes

- `ghostboard/src/apprt/termsurf.zig`
  - recognize `QueryDevtoolsRequest` in the decoded `TermSurfMessage` switch;
  - log the request's `pane_id`, `inspected_tab_id`, `profile`, and `browser`;
  - send a length-prefixed `QueryDevtoolsReply` with the validation/no-state
    error described above;
  - add `QueryDevtoolsRequest` and `QueryDevtoolsReply` to the TermSurf message
    type name helper used by decoded-message logs.

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
  length-prefixed current-schema `QueryDevtoolsRequest` messages, and decodes
  length-prefixed `QueryDevtoolsReply` messages.
- The harness verifies all four no-state/validation replies:
  - empty `browser` returns `DevTools target browser is required`;
  - nonempty `browser` with empty `profile` returns
    `DevTools target profile is required`;
  - nonempty `browser` and `profile` with `inspected_tab_id = 0` returns
    `DevTools target tab id is required`;
  - nonempty `browser`, `profile`, and nonzero `inspected_tab_id` returns
    `Inspected tab {id} not found in {browser}/{profile}`.
- The runtime harness also sends `HelloRequest`, `QueryTabsRequest`, and
  `QueryLastRequest` to prove Experiments 8 through 10 still work on the same
  socket implementation.
- The app log contains `TermSurf message decoded type=QueryDevtoolsRequest` and
  a reply-sent log for `QueryDevtoolsReply`.
- Shutdown cleanup still removes the socket file and leaves no stale
  `TermSurf.app/Contents/MacOS/termsurf` process.
- `git diff --check` is clean.

Fail criteria:

- `QueryDevtoolsRequest` is ignored or returns no frame.
- The reply has the wrong oneof message type.
- The validation order differs from Wezboard.
- The nonzero tab-id no-state case falsely reports success before Ghostboard has
  implemented browser pane state.
- Any `webtui`, `roamium`, protocol schema, app branding, config path, icon, or
  CLI install behavior changes are needed for this experiment.

## Design Review

Fresh-context adversarial design review returned `APPROVED` with no required
findings.

Optional notes:

- The reviewer noted that the build verification names the native GhosttyKit and
  macOS app builds without spelling out exact commands. I will use the same
  commands as Experiments 8 through 10 and record the exact commands in the
  result.
- The reviewer noted that the runtime harness is described behaviorally rather
  than as a named script. I will record the exact harness behavior and logs in
  the result, as in the previous socket experiments.

The reviewer confirmed the README links Experiment 11 as `Designed`, the
experiment has the required sections, the scope is limited to
`ghostboard/src/apprt/termsurf.zig`, the validation order matches Wezboard's
handler, and the no-state fallback is faithful because without tab/server state
Wezboard reaches the `Inspected tab ... not found` branch.
