# Experiment 1: Establish Input/Focus Baseline Matrix

## Description

Issue 817 needs a focused input/focus regression matrix before adding new
automation or fixing app code. Current evidence is spread across Issue 809
geometry scenarios, Issue 811 cursor feedback, Issue 812 GUI active state, and
Issue 816 browser-state/copy behavior. This experiment will turn that scattered
evidence into an explicit baseline for Issue 817 and run a small representative
set of existing scenarios to prove the current automation still works.

The experiment is intentionally evidence-first. It should classify each
requested input/focus behavior as:

- `Covered`: current issue docs plus a current rerun prove it well enough for
  the fast regression matrix;
- `Partially covered`: current evidence exists, but the behavior needs a focused
  follow-up or a slower/manual bucket;
- `Uncovered`: no useful current Ghostboard runtime proof exists;
- `Blocked`: automation cannot test the behavior yet and the blocker is
  concrete.

## Changes

Planned issue-document changes:

- Add an Issue 817 baseline matrix section recording each requested behavior:
  keyboard text input and special keys, Cmd/menu shortcuts, clipboard behavior,
  mode transitions, focus stealing and pane focus, inactive visual feedback,
  caret visibility, mouse click/hover/scroll/double-click/triple-click/
  modifier-click, drag selection and terminal-selection suppression, and mouse
  hot-path performance.
- Link every baseline row to current evidence where it exists:
  - Issue 809 geometry/input scenarios;
  - Issue 811 cursor feedback;
  - Issue 812 GUI active state;
  - Issue 816 browser state and copy-current-URL;
  - current `scripts/ghostboard-geometry-matrix.sh` scenario names.
- Record which rows should become fast automated smokes, which rows should be
  slower screenshot/manual checks, and which rows require new harness support.

Planned runtime checks:

- Run a compact current baseline using existing scenarios rather than the full
  slow matrix:
  1. `scripts/ghostboard-geometry-matrix.sh mouse-after-geometry-change`;
  2. `scripts/ghostboard-geometry-matrix.sh keyboard-after-tab-window-switch`;
  3. `scripts/ghostboard-geometry-matrix.sh gui-active-multi-tab`;
  4. `scripts/ghostboard-geometry-matrix.sh copy-current-url-smoke`.
- If one of those scenarios fails, stop and record the failing row, owner, logs,
  and next experiment recommendation instead of masking it with additional
  scenarios.

Planned source changes:

- None unless the baseline run proves the existing harness cannot distinguish a
  required pass/fail condition. If that happens, limit implementation changes to
  this issue's docs and, at most, `scripts/ghostboard-geometry-matrix.sh`
  assertions/logging needed to make the baseline trustworthy, then rerun the
  affected scenario. If Ghostboard, webtui, Roamium, or protocol source changes
  are needed, record `Partial` or `Fail` and make the source fix the next
  experiment.

## Verification

Formatting actions:

1. `prettier --write --prose-wrap always --print-width 80 issues/0817-ghostboard-input-focus-regression-matrix/README.md issues/0817-ghostboard-input-focus-regression-matrix/01-establish-input-focus-baseline.md`.

Static checks:

1. `git diff --check`.
2. If `scripts/ghostboard-geometry-matrix.sh` changes, run
   `bash -n scripts/ghostboard-geometry-matrix.sh`.
3. If Rust files change, run `cargo fmt` and `cargo check` for the affected
   package.
4. If Ghostboard Zig or Swift files change, run the relevant `zig build` or
   `macos/build.nu` command before runtime testing.

Runtime checks:

1. `scripts/ghostboard-geometry-matrix.sh mouse-after-geometry-change`.
2. `scripts/ghostboard-geometry-matrix.sh keyboard-after-tab-window-switch`.
3. `scripts/ghostboard-geometry-matrix.sh gui-active-multi-tab`.
4. `scripts/ghostboard-geometry-matrix.sh copy-current-url-smoke`.

Pass criteria:

- The Issue 817 baseline matrix exists and maps every requested behavior from
  the issue README to `Covered`, `Partially covered`, `Uncovered`, or `Blocked`.
- Every `Covered` row cites concrete current evidence.
- The compact runtime baseline passes, or failures are classified with log paths
  and a specific next experiment.
- The result recommends the smallest next experiment based on the weakest
  uncovered or failing row.

Partial criteria:

- The matrix exists, but one or more existing scenarios fail for reasons that
  require a focused fix experiment.
- Runtime automation is available for the main keyboard/mouse/focus paths, but
  slower behaviors such as triple-click, drag selection, caret visibility, or
  hot-path performance remain only classified and not yet implemented.

Fail criteria:

- The experiment cannot map the Issue 817 requested behaviors to concrete rows.
- The runtime baseline cannot launch Ghostboard, webtui, or Roamium.
- The harness cannot produce logs specific enough to identify the owner of a
  failure.

## Design Review

This experiment is plan-only until a fresh-context adversarial design review
approves it. Record the reviewer verdict here, fix all real findings, then
commit the approved plan before implementation begins.

Fresh-context adversarial design review by Codex subagent `Boole`:

- **Initial verdict:** Changes required.
- **Finding 1:** The Rust formatting check incorrectly narrowed formatting to
  changed files. Fixed by requiring `cargo fmt` after any Rust edit.
- **Finding 2:** The experiment was missing an explicit completion/result gate.
  Fixed by adding the Completion Gate section below.
- **Optional finding:** The source-change escape hatch was wider than the
  baseline experiment needed. Fixed by constraining implementation changes to
  docs and, at most, harness assertions/logging; Ghostboard, webtui, Roamium, or
  protocol source fixes must become a follow-up experiment.
- **Final verdict:** Approved. The reviewer confirmed the prior findings were
  resolved and no Required findings remained.

## Completion Gate

After implementation and verification:

- add `## Result` and `## Conclusion` to this experiment file;
- update the Issue 817 README experiment status from `Designed` to `Pass`,
  `Partial`, or `Fail`;
- request a fresh-context completion review;
- fix all real completion-review findings and record the final verdict in this
  file; and
- commit the reviewed result separately before designing or implementing the
  next experiment.

## Result

**Result:** Partial

The compact runtime baseline passed, and Issue 817 now has an explicit baseline
matrix for the requested input/focus behaviors. The result is `Partial` because
the matrix still exposes several important rows that are not yet covered by a
focused Ghostboard runtime test.

Current baseline runs:

| Scenario                           | Result | Evidence                                                                                                                                                                                                                                                                                                                                                 |
| ---------------------------------- | ------ | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `mouse-after-geometry-change`      | Pass   | `logs/ghostboard-geometry-mouse-after-geometry-change-harness-20260618-002509.log`; app log `logs/ghostboard-geometry-mouse-after-geometry-change-app-20260618-002509.log`; Roamium trace `logs/ghostboard-geometry-mouse-after-geometry-change-roamium-20260618-002509.log`                                                                             |
| `keyboard-after-tab-window-switch` | Pass   | `logs/ghostboard-geometry-keyboard-after-tab-window-switch-harness-20260618-002550.log`; app log `logs/ghostboard-geometry-keyboard-after-tab-window-switch-app-20260618-002550.log`; Roamium trace `logs/ghostboard-geometry-keyboard-after-tab-window-switch-roamium-20260618-002550.log`                                                              |
| `gui-active-multi-tab`             | Pass   | `logs/ghostboard-geometry-gui-active-multi-tab-harness-20260618-002807.log`; app log `logs/ghostboard-geometry-gui-active-multi-tab-app-20260618-002807.log`; Roamium trace `logs/ghostboard-geometry-gui-active-multi-tab-roamium-20260618-002807.log`                                                                                                  |
| `copy-current-url-smoke`           | Pass   | `logs/ghostboard-geometry-copy-current-url-smoke-harness-20260618-002920.log`; app log `logs/ghostboard-geometry-copy-current-url-smoke-app-20260618-002920.log`; Roamium trace `logs/ghostboard-geometry-copy-current-url-smoke-roamium-20260618-002920.log`; webtui trace `logs/ghostboard-geometry-copy-current-url-smoke-webtui-20260618-002920.log` |

Baseline matrix:

| Behavior                                                      | Status            | Evidence                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                               | Next action                                                                                                                      |
| ------------------------------------------------------------- | ----------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------- |
| Keyboard text input and ordinary key forwarding               | Covered           | `keyboard-after-tab-window-switch` proves Browse-mode keyboard markers reach only the active browser after tab and window switches. It passed in `logs/ghostboard-geometry-keyboard-after-tab-window-switch-harness-20260618-002550.log`.                                                                                                                                                                                                                                                                                                                              | Keep this as a fast smoke row.                                                                                                   |
| Keyboard special keys                                         | Partially covered | The baseline proves `enter` switches to Browse mode and `escape` returns to Control mode across browser tabs and windows. It does not yet prove browser-received editing/navigation keys such as arrows, backspace, delete, tab, or return inside a page input.                                                                                                                                                                                                                                                                                                        | Add a focused page-input key matrix.                                                                                             |
| Cmd/menu shortcuts                                            | Partially covered | `copy-current-url-smoke` proves Control-mode `Cmd+C` is handled by Ghostboard, and Browse-mode `Cmd+C` does not run Ghostboard copy-current-URL. Issue 816 also covered refresh/reload behavior, but this experiment did not rerun all menu shortcuts.                                                                                                                                                                                                                                                                                                                 | Keep copy as a fast smoke; add a small shortcut matrix only for browser-owned shortcuts that are easy to assert.                 |
| Clipboard behavior                                            | Covered           | `copy-current-url-smoke` sets a sentinel clipboard value, verifies Control-mode `Cmd+C` copies the current URL, and verifies Browse-mode `Cmd+C` leaves the guard sentinel intact.                                                                                                                                                                                                                                                                                                                                                                                     | Keep as a fast smoke row.                                                                                                        |
| Mode transitions                                              | Covered           | `keyboard-after-tab-window-switch`, `gui-active-multi-tab`, and `copy-current-url-smoke` all prove `enter=Mode::Browse`, `escape=Mode::Control`, and matching Roamium focus true/false transitions.                                                                                                                                                                                                                                                                                                                                                                    | Keep as a fast smoke row.                                                                                                        |
| Focus stealing and pane focus                                 | Covered           | `keyboard-after-tab-window-switch` proves keyboard routing follows selected tabs and windows without leaking to inactive browsers; `gui-active-multi-tab` proves activation targets only the focused browser.                                                                                                                                                                                                                                                                                                                                                          | Keep as a fast smoke row.                                                                                                        |
| Dimming or inactive visual feedback                           | Partially covered | Issue 812 and `gui-active-multi-tab` prove `SetGuiActive` active/inactive delivery to Roamium. This does not screenshot-assert any visual dimming state.                                                                                                                                                                                                                                                                                                                                                                                                               | Add a slow screenshot/manual row if visual dimming is required product behavior.                                                 |
| Caret visibility                                              | Uncovered         | No current baseline row checks caret visibility in a browser text field or terminal/web focus handoff.                                                                                                                                                                                                                                                                                                                                                                                                                                                                 | Add a page-input screenshot row after special-key input is covered.                                                              |
| Mouse click                                                   | Covered           | `mouse-after-geometry-change` proves click hit testing and webview-relative coordinates after grow, shrink, TUI viewport resize/reset, split, divider resize, and equalize.                                                                                                                                                                                                                                                                                                                                                                                            | Keep as a fast smoke row.                                                                                                        |
| Mouse hover                                                   | Covered           | Issue 811 Experiment 1 proves link, input/text, default background, and two-tab cursor isolation through Roamium `CursorChanged` and AppKit cursor application. Exact evidence is recorded in `issues/0811-ghostboard-cursor-feedback/01-wire-browser-cursor-updates.md`, including `logs/ghostboard-geometry-initial-open-app-20260617-192326.log`, `logs/ghostboard-geometry-initial-open-app-20260617-192332.log`, `logs/ghostboard-geometry-initial-open-app-20260617-192402.log`, and `logs/ghostboard-geometry-open-browser-in-new-tab-app-20260617-192430.log`. | Keep Issue 811 cursor scenario as the hover evidence; add to fast matrix only if runtime cost is acceptable.                     |
| Mouse scroll                                                  | Partially covered | Issue 809 Experiment 22 proves terminal scrollback movement with stable browser geometry and continued input routing, but this experiment did not rerun a browser-page scroll row.                                                                                                                                                                                                                                                                                                                                                                                     | Reuse the existing scrollback scenario for terminal scroll; add browser-page wheel scroll only if not already covered elsewhere. |
| Double-click                                                  | Uncovered         | No current Ghostboard runtime evidence proves double-click delivery or selection semantics inside browser content.                                                                                                                                                                                                                                                                                                                                                                                                                                                     | Add to a mouse interaction granularity experiment.                                                                               |
| Triple-click                                                  | Uncovered         | No current Ghostboard runtime evidence proves triple-click delivery or line/paragraph selection semantics.                                                                                                                                                                                                                                                                                                                                                                                                                                                             | Add to the slower mouse interaction bucket.                                                                                      |
| Modifier-click                                                | Uncovered         | No current Ghostboard runtime evidence proves modifier flags are forwarded with mouse events.                                                                                                                                                                                                                                                                                                                                                                                                                                                                          | Add to a focused mouse-event modifier test.                                                                                      |
| Drag selection                                                | Uncovered         | No current Ghostboard runtime evidence proves drag selection inside browser content.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                   | Add to the mouse interaction granularity experiment.                                                                             |
| Terminal-selection suppression while dragging browser content | Uncovered         | No current evidence proves browser drag does not accidentally create terminal selection.                                                                                                                                                                                                                                                                                                                                                                                                                                                                               | Add to the mouse interaction granularity experiment with screenshot or AppKit selection evidence.                                |
| Mouse hot-path performance                                    | Uncovered         | The current harness asserts correctness, not event-rate, coalescing, latency, or CPU behavior.                                                                                                                                                                                                                                                                                                                                                                                                                                                                         | Defer to Issue 820 performance smoke tests.                                                                                      |

## Conclusion

The current automation can reliably drive Ghostboard, webtui, Roamium, mouse
input, keyboard input, app activation, tab/window switching, and clipboard
shortcuts on this VM. That is enough to keep the existing keyboard/mouse/focus
smokes as fast regression guards.

The weakest remaining Issue 817 area is browser-input granularity. The next
experiment should add a focused page-input and mouse-granularity test that
asserts browser-received special keys, caret behavior, double-click,
triple-click, modifier-click, drag selection, and terminal-selection
suppression. Mouse hot-path performance should remain classified here but be
handled by Issue 820.

## Completion Review

Fresh-context adversarial completion review by Codex subagent `James`:

- **Verdict:** Approved.
- **Optional finding:** The mouse-hover row cited Issue 811 but did not include
  a precise experiment or log path. Fixed by adding the exact Issue 811
  experiment file and cursor evidence logs to the baseline matrix row.
- **Re-review verdict:** Approved. The reviewer confirmed the optional finding
  is resolved and no new Required finding was introduced.
- **Required findings:** None.
