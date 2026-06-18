+++
status = "closed"
opened = "2026-06-17"
closed = "2026-06-17"
+++

# Issue 812: Ghostboard GUI Active State

## Goal

Implement and verify Ghostboard GUI active/inactive signaling to Roamium using
`SetGuiActive`.

## Background

Issue 810 classified this as a `Highly likely` Ghostboard gap. `SetGuiActive` is
GUI-owned state: webtui's direct Roamium socket cannot know whether the macOS
app or window is active. Wezboard sends app/window activation state, while
Ghostboard did not show an equivalent runtime path during the audit.

## Analysis

The work should identify the correct Ghostty/AppKit activation and deactivation
hooks, translate them into TermSurf `SetGuiActive` messages for relevant browser
tabs or servers, and verify Roamium receives the state transitions.

Verification should include:

- app activation sends active state;
- app deactivation sends inactive state;
- window focus changes do not produce stale or duplicate state;
- browser input/focus behavior still works after activation changes.

## Experiments

- [Experiment 1: Wire GUI active state into Roamium](01-wire-gui-active-state.md)
  — **Partial**
- [Experiment 2: Prove active state across browser tabs](02-prove-active-state-across-browser-tabs.md)
  — **Pass**

## Conclusion

Ghostboard now sends GUI active/inactive state to Roamium using `SetGuiActive`.
Experiment 1 wired AppKit activation and deactivation into Ghostboard's browser
state path and proved the one-browser path, while Experiment 2 added a
multi-native-tab regression scenario proving that deactivation broadcasts
inactive state, activation targets only the focused browser tab, switching tabs
changes the next activation target, and browser keyboard input remains scoped to
the focused tab after activation.

The remaining broad Ghostty test-target failures are documented as pre-existing
local test-harness/environment gaps because the same failures reproduce in the
baseline worktree.
