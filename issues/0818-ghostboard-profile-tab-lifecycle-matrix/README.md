+++
status = "closed"
opened = "2026-06-17"
closed = "2026-06-18"
+++

# Issue 818: Ghostboard Profile, Tab, and Lifecycle Matrix

## Goal

Design and run a focused Ghostboard matrix for multi-profile, multi-pane,
multi-tab, DevTools, reconnect, close/reopen, and process cleanup behavior.

## Background

Issue 810 grouped profile, tab, and process lifecycle as a `Maybe` finding.
Current Ghostboard has credible code shape for pane, server, tab, profile, and
DevTools state, but the full runtime matrix is not proven.

The matrix should cover:

- multi-profile isolation;
- multi-pane routing;
- multi-tab routing;
- warm reconnect;
- server reuse;
- close/reopen behavior;
- stale process cleanup;
- DevTools target lookup;
- profile display or user-visible profile identity.

## Analysis

This issue should prove the lifecycle invariants before making fixes. Tests
should include enough logging or screenshots to distinguish wrong pane routing,
wrong profile routing, stale tab lookup, duplicate server spawn, and premature
process exit.

## Experiments

- [Experiment 1: Establish lifecycle baseline](01-establish-lifecycle-baseline.md)
  — **Partial**
- [Experiment 2: Prove multi-profile isolation](02-prove-multi-profile-isolation.md)
  — **Pass**
- [Experiment 3: Prove same-profile server reuse and cleanup](03-prove-same-profile-server-reuse-cleanup.md)
  — **Partial**
- [Experiment 4: Fix native tab-close TermSurf cleanup](04-fix-native-tab-close-termsurf-cleanup.md)
  — **Pass**
- [Experiment 5: Prove two-browser split-pane routing](05-prove-two-browser-split-routing.md)
  — **Pass**
- [Experiment 6: Prove TUI disconnect reconnect](06-prove-tui-disconnect-reconnect.md)
  — **Pass**
- [Experiment 7: Prove visible profile identity](07-prove-visible-profile-identity.md)
  — **Pass**

## Conclusion

Issue 818 is closed. The Ghostboard profile, tab, pane, DevTools, reconnect,
close/reopen, and lifecycle matrix is now covered by focused runtime proofs and
targeted fixes.

Final coverage:

- multi-profile isolation: Experiment 2 proved distinct profile server keys,
  Roamium processes, profile-specific user-data directories, storage isolation,
  and input routing isolation;
- multi-tab routing: Experiment 1 carried forward the passing native-tab routing
  rows, and Experiments 3 and 4 proved same-profile A/B/C native-tab routing
  through close and reopen;
- same-profile server reuse, normal close/reopen, stale tab cleanup, and final
  process cleanup: Experiments 3 and 4 proved reuse, exposed/fixed the
  native-tab close cleanup path, prevented late closed-pane `SetOverlay`
  recreation, removed closed tabs in Roamium, and proved the final shared
  Roamium pid exited after the last browser closed;
- multi-pane routing: Experiment 5 proved two simultaneous browser panes in one
  split route mouse and keyboard input independently;
- warm reconnect: Experiment 6 proved TUI disconnect cleanup, shared server
  preservation, and later reconnect on the warm server;
- DevTools target lookup: Experiment 1 carried forward the passing
  `devtools-singleton-guard` runtime row from the baseline matrix;
- user-visible profile identity: Experiment 7 proved the rendered webtui
  viewport identity label for default profile, non-default profile, and DevTools
  inspected-tab identity.

The main source fixes made during this issue were:

- Ghostboard native tab-close cleanup now sends `CloseTab` while the shared
  profile server is writable and keeps the server alive until the final pane
  closes.
- Roamium now keeps browser FFI handles in a tab table so `CloseTab` can destroy
  the correct tab by id.
- webtui now traces the same identity label string it renders in the viewport
  footer, enabling durable regression coverage for visible profile identity.

The remaining cleanup noise observed in some harness runs is process-teardown
related and is not part of Issue 818's lifecycle invariants. Future work should
track that as a separate issue only if it becomes user-visible or blocks
automated runs.
