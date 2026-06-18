# Experiment 5: Prove Two-Browser Split-Pane Routing

## Description

Experiment 1 left multi-pane routing only partially covered: the existing split
rows prove a single browser pane next to a terminal sibling, but they do not
prove two simultaneous browser overlays in one split layout. Experiments 2
through 4 proved multi-profile isolation, same-profile server reuse,
close/reopen, and final server cleanup in native-tab layouts, but the Issue 818
matrix still needs a direct split-pane lifecycle row.

This experiment will add and run a focused runtime scenario for two browser
panes in one split. It should open browser A, create a right split, launch
browser B in the sibling pane, prove both panes have distinct pane/tab/context
identity, prove both overlays render at separate split frames, prove mouse
hit-testing targets the pane under the cursor, prove keyboard input reaches only
the focused browser, close one browser pane, and prove the surviving browser
pane remains interactive without receiving stale events from the closed pane.

The experiment is proof-first. No app source changes are planned. If the
scenario exposes a Ghostboard-owned routing or cleanup bug, record the result as
`Partial` or `Fail` and make the fix a later design-reviewed experiment.

## Changes

Planned harness changes:

- `scripts/ghostboard-geometry-matrix.sh`
  - Add a `two-browser-split-routing` scenario.
  - Reuse the existing split-right automation shape and existing browser routing
    helpers where possible.
  - Launch browser A in the initial pane with
    `web --browser "$ROAMIUM" --profile default "$URL"`.
  - Create a right split, focus the sibling terminal pane, and launch browser B
    with `web --browser "$ROAMIUM" --profile default "$URL_B"`.
  - Assert browser B reuses the same `default/${ROAMIUM}` server/pid as browser
    A, because both panes use the same profile and browser.
  - Assert browser A and browser B have distinct pane ids, browser tab ids, CA
    context ids, terminal surface ids, and split overlay frames.
  - Assert both AppKit presented frames and pixels correspond to their split
    locations and do not overlap incoherently.
  - Click inside browser A and browser B, and assert hit-testing targets the
    expected CA context and selected pane for each click.
  - Enter Browse mode in browser A and browser B in turn, send keyboard markers,
    and assert each marker reaches only the active browser tab/pane.
  - Close browser B's split pane and assert `CloseTab` reaches Roamium for
    browser B while browser A and the shared server remain alive.
  - Click and type in browser A after browser B closes, and assert browser A
    remains interactive while closed browser B receives no input.

Planned issue-document changes:

- Record the result in this experiment file.
- Update the Issue 818 README status for Experiment 5 after verification.

Planned app source changes:

- None.

## Verification

Formatting actions:

1. `prettier --write --prose-wrap always --print-width 80 issues/0818-ghostboard-profile-tab-lifecycle-matrix/README.md issues/0818-ghostboard-profile-tab-lifecycle-matrix/05-prove-two-browser-split-routing.md`.

Static checks:

1. `git diff --check`.
2. `bash -n scripts/ghostboard-geometry-matrix.sh`.

Runtime checks:

1. `scripts/ghostboard-geometry-matrix.sh two-browser-split-routing`.

Pass criteria:

- Browser A launches successfully in the initial pane.
- Browser B launches successfully in the right split pane.
- Browser B reuses the existing `default/${ROAMIUM}` server and Roamium pid
  instead of spawning a second default-profile process.
- Browser A and browser B have distinct pane ids, browser tab ids, CA context
  ids, terminal surface ids, and non-overlapping split overlay frames.
- AppKit presents both overlays at their expected split-frame locations and
  pixel sizes.
- Clicking inside browser A produces a hit-test against browser A's CA context
  and selected pane, not browser B's.
- Clicking inside browser B produces a hit-test against browser B's CA context
  and selected pane, not browser A's.
- Keyboard input reaches browser A only when browser A is focused.
- Keyboard input reaches browser B only when browser B is focused.
- Closing browser B's split pane sends timely `CloseTab` for browser B while
  Roamium is still attached.
- Browser A remains interactive after browser B closes.
- Closed browser B receives no keyboard or mouse input after close.
- The shared server remains alive while browser A is still open.

Partial criteria:

- Two-browser split routing and input isolation pass, but the close-B cleanup
  portion exposes a separate cleanup gap.
- Geometry and hit-testing pass, but one keyboard-routing assertion is
  inconclusive because automation cannot reliably focus the desired split pane.
- The scenario exposes a distinct lifecycle bug that should be fixed in the next
  experiment.

Fail criteria:

- Browser B cannot launch in the split pane.
- Browser B spawns a second same-profile Roamium process instead of reusing the
  existing server.
- Browser A and browser B reuse the same pane id, browser tab id, CA context id,
  or terminal surface id.
- The two browser overlays overlap incorrectly or are presented in the wrong
  split locations.
- Mouse hit-testing targets the wrong browser pane.
- Keyboard input leaks between browser panes.
- Closing browser B kills or disconnects browser A.
- Closed browser B continues receiving input.

## Design Review

Fresh-context adversarial design review by Codex subagent `Lorentz the 2nd`:

- **Verdict:** Approved.
- **Required findings:** None.
- **Reviewer checks:** The reviewer confirmed the README links Experiment 5 as
  `Designed`, the experiment has the required sections, the scope is one
  proof-first harness scenario plus issue docs with no app source changes, and
  the verification criteria cover same-profile server reuse, distinct identity,
  non-overlapping split geometry, AppKit presentation, mouse hit-testing,
  keyboard isolation, browser B close cleanup, browser A post-close
  interactivity, and no closed-browser B input.
