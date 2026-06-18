+++
status = "closed"
opened = "2026-06-17"
closed = "2026-06-18"
+++

# Issue 816: Ghostboard Browser State and Interruption Walkthrough

## Goal

Prove or reject the medium-likelihood browser-state and interruption-flow gaps
from Issue 810.

## Background

Issue 810 grouped these as `Maybe` findings. Static or partial evidence exists
for many paths through direct Roamium sockets, but Ghostboard runtime proof is
missing for the full walkthrough.

Covered behaviors include:

- loading state;
- page title;
- hover target URL;
- console messages;
- JavaScript dialogs;
- HTTP auth;
- renderer crash recovery;
- color scheme;
- target blank;
- refresh/reload;
- copy-current-URL;
- default white page background.

## Analysis

This issue should start as a walkthrough and regression-design issue. It should
only fix app code after a focused experiment proves a specific missing behavior.
Because many flows are engine- or webtui-owned, each finding must identify the
owning component before any fix.

## Experiments

- [Experiment 1: Prove direct browser state smoke](01-prove-direct-browser-state-smoke.md)
  — **Partial** (initial load reports `progress`/`done` but not literal
  `loading`)
- [Experiment 2: Fix initial loading-state start](02-fix-initial-loading-state-start.md)
  — **Pass**
- [Experiment 3: Prove JavaScript dialog runtime flow](03-prove-javascript-dialogs.md)
  — **Pass**
- [Experiment 4: Prove HTTP auth runtime flow](04-prove-http-auth-runtime-flow.md)
  — **Pass**
- [Experiment 5: Prove renderer crash recovery](05-prove-renderer-crash-recovery.md)
  — **Pass**
- [Experiment 6: Prove runtime color scheme](06-prove-runtime-color-scheme.md) —
  **Pass**
- [Experiment 7: Prove copy-current-URL](07-prove-copy-current-url.md) —
  **Pass**

## Conclusion

Issue 816 is closed. The walkthrough proved the medium-likelihood browser-state
and interruption gaps from Issue 810 under the current debug Ghostboard +
webtui + Roamium path.

The final coverage is:

- loading state, including the initially missing first `loading` event;
- page title updates;
- hover target URL;
- console messages;
- target blank navigation;
- refresh/reload;
- default white page background;
- JavaScript `alert`, `confirm`, `prompt`, and `beforeunload` flows;
- HTTP auth success, cancel, post-cancel recovery, and password non-leakage;
- renderer crash state and recovery;
- runtime color scheme changes; and
- Cmd+C copy-current-URL with Ghostboard feedback plus Browse-mode
  non-interference.

The main ownership finding is that most browser-state and interruption flows are
normal direct webtui/Roamium behavior after `BrowserReady`, while the macOS
Cmd+C current-URL path must be Ghostboard-owned because AppKit can consume the
key before webtui sees it.
