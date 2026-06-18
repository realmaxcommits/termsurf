# Experiment 2: Prove Active State Across Browser Tabs

## Description

Experiment 1 implemented Ghostboard `SetGuiActive` signaling, but it ended
partial for two reasons:

- the recorded runtime evidence used only one browser pane/tab;
- broad local test targets failed, and the experiment did not prove whether
  those failures were caused by the active-state implementation or were
  pre-existing test-harness/environment failures.

This experiment should close those verification gaps without broadening the
feature implementation. The core behavior to prove is:

- when two browser tabs exist, app deactivation broadcasts inactive state to the
  live browser process with `tab_id=0`;
- app activation targets only the currently focused browser tab;
- switching focus between browser tabs changes which tab receives the next
  active-state message;
- the unfocused browser tab does not receive a duplicate stale `active=true`
  message during the same activation cycle.

## Changes

Planned source changes:

- `scripts/ghostboard-geometry-matrix.sh`
  - Add a focused scenario such as `gui-active-multi-tab`, reusing the existing
    `open-browser-in-new-tab` setup flow:
    - open browser A in native tab 1;
    - open browser B in native tab 2;
    - put browser B into browse mode and verify normal input reaches only B;
    - hide/unhide or deactivate/reactivate the app and verify active-state logs
      target browser B only;
    - switch back to browser A, put it into browse mode, deactivate/reactivate,
      and verify active-state logs target browser A only;
    - verify no `set-gui-active ... active=true` line appears for the unfocused
      browser tab after each activation boundary.
  - Add small shell helpers if needed for:
    - waiting for `SetGuiActive` app-log lines after a boundary;
    - waiting for Roamium `set-gui-active` trace lines after a boundary;
    - asserting that no active `set-gui-active` trace was emitted for an
      unfocused tab after a boundary.
  - Keep the scenario narrow: do not change existing scenario behavior except
    where shared helpers are added.

Planned issue-doc changes:

- Record the baseline/current test-target comparison in this experiment.
- Record the `gui-active-multi-tab` runtime logs, pass/fail result, and
  conclusion.
- Update the Issue 812 README status line for Experiment 2.

## Verification

Baseline and static checks:

1. Confirm the working tree starts from the Experiment 1 result commit.
2. Create a temporary disposable baseline worktree at the Experiment 1 plan
   commit `bc9601cad` or another clearly documented pre-implementation commit.
3. In the baseline worktree, run enough checks to classify the known local
   failures:
   - `cd ghostboard && swiftlint lint --strict`
   - `cd ghostboard && zig build test`
   - `cd ghostboard && macos/build.nu --scheme Ghostty --configuration Debug --action test`
4. In the current worktree, run:
   - `prettier --write --prose-wrap always --print-width 80 issues/0812-ghostboard-gui-active-state/README.md issues/0812-ghostboard-gui-active-state/02-prove-active-state-across-browser-tabs.md`
   - `shellcheck scripts/ghostboard-geometry-matrix.sh` if available;
   - `cd ghostboard && zig build -Demit-macos-app=false`;
   - `cd ghostboard && macos/build.nu --scheme Ghostty --configuration Debug --action build`;
   - `cargo check -p roamium`;
   - `git diff --check`.
5. Re-run the broad test targets in the current worktree if practical:
   - `cd ghostboard && swiftlint lint --strict`;
   - `cd ghostboard && zig build test`;
   - `cd ghostboard && macos/build.nu --scheme Ghostty --configuration Debug --action test`.
6. If a check is unavailable or fails for the same reason in both baseline and
   current worktrees, record the exact failure and classify it as a pre-existing
   environment/test-harness gap rather than an Issue 812 regression.

Runtime checks:

1. Build the app and local Roamium used by the geometry harness.
2. Run `scripts/ghostboard-geometry-matrix.sh gui-active-multi-tab`.
3. Verify the harness opens browser A and browser B with distinct pane ids and
   browser tab ids.
4. With browser B focused/in browse mode:
   - deactivate/reactivate Ghostboard;
   - require Ghostboard app logs to show an inactive broadcast;
   - require Roamium trace logs to show
     `set-gui-active tab=0 active=false reason=gui_deactivated`;
   - require Ghostboard app logs and Roamium trace logs to show
     `active=true reason=gui_activated` for browser B's tab id;
   - require no `active=true` Roamium trace line for browser A's tab id after
     the browser B activation boundary.
5. Switch back to browser A and repeat the deactivate/reactivate cycle:
   - require active state for browser A's tab id;
   - require no `active=true` Roamium trace line for browser B's tab id after
     the browser A activation boundary.
6. After each activation cycle, send a deterministic keyboard marker and verify
   it reaches the focused browser tab only.

Pass criteria:

- The current Ghostboard app build passes.
- The current Roamium check/build needed by the harness passes.
- Any broad test-target failures are shown to be pre-existing or environmental,
  with matching baseline/current evidence.
- The new runtime scenario passes with two distinct browser tabs.
- Deactivation is observed as `tab=0 active=false`.
- Activation is observed exactly for the focused browser tab after each
  activation boundary.
- No stale or duplicate `active=true` message is observed for the unfocused
  browser tab after each activation boundary.
- Focused-browser keyboard input still reaches only the focused tab after each
  activation cycle.

Partial criteria:

- The active-state implementation still works for one tab, but the new multi-tab
  scenario cannot be completed because of automation or environment instability.
- The multi-tab scenario passes, but broad test-target failures cannot be
  confidently classified against a baseline.

Fail criteria:

- The app no longer builds.
- The multi-tab scenario shows activation targeting the wrong tab.
- The focused tab receives no active-state message after activation.
- The unfocused tab receives a stale or duplicate active-state message after an
  activation boundary.
- Keyboard input/focus regresses after activation.

## Design Review

Fresh-context adversarial review by Codex subagent `Herschel`:

- **Verdict:** Changes required.
- **Required finding:** The baseline verification plan described a read-only
  baseline worktree but used `swiftlint lint --strict --fix`, whose fix mode can
  mutate files and make the baseline comparison less reproducible.
- **Resolution:** Changed the baseline/current SwiftLint classification command
  to non-mutating `swiftlint lint --strict` and described the baseline worktree
  as temporary and disposable.
- **Re-review verdict:** Approved.
