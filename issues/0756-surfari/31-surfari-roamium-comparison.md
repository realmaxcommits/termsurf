# Experiment 31: Compare Surfari against the Roamium matrix

## Description

Experiment 30 proved Surfari crash handling. The only remaining
`real-app-matrix.md` row is `Roamium comparison`.

This experiment should perform the final Surfari parity comparison against the
Ghostboard/Roamium behavior matrix. The point is not to port the large
Roamium-specific `scripts/ghostboard-geometry-matrix.sh` wholesale. Instead, use
that script as the authoritative list of comparable real-app behaviors, then
prove each comparable behavior with the focused Surfari harnesses created in
Experiments 23 and 25-30.

If the comparison finds a real Surfari parity gap, this experiment should record
`Partial` or `Fail` and the next experiment should fix the gap. If every
comparable row is proven or intentionally engine-specific, this experiment can
mark `Roamium comparison` as `Proven` and close Issue 756.

## Changes

- Add a comparison artifact under `issues/0756-surfari/`, tentatively
  `surfari-roamium-comparison.md`, that maps Roamium scenarios to Surfari
  evidence.
- Create a focused aggregate harness under `scripts/`, tentatively
  `test-issue-756-surfari-final-comparison.sh`, that reruns the Surfari real-app
  evidence suite:
  - `scripts/test-issue-756-surfari-input-regression.sh`;
  - `scripts/test-issue-756-surfari-lifecycle-tranche.sh`;
  - `scripts/test-issue-756-surfari-pane-split-geometry.sh`;
  - `scripts/test-issue-756-surfari-tab-window-focus-geometry.sh`;
  - `scripts/test-issue-756-surfari-click-drag-input-details.sh`;
  - `scripts/test-issue-756-surfari-profile-isolation.sh`;
  - `scripts/test-issue-756-surfari-crash-handling.sh`.
- The aggregate harness should write its own log under
  `logs/issue-756-exp31-surfari-roamium-comparison/` and record each child
  harness run ID or log path.
- The aggregate harness must fail before running child harnesses if the Debug
  `TermSurf.app` binary is missing or stale relative to Ghostboard source/build
  inputs. Reuse the freshness guard pattern from
  `scripts/test-issue-756-surfari-crash-handling.sh`.
- The comparison artifact should classify each relevant Roamium scenario from
  `scripts/ghostboard-geometry-matrix.sh` as one of:
  - `Equivalent` — Surfari has direct current evidence for the same user-visible
    behavior;
  - `Covered by focused Surfari harness` — the exact Roamium scenario name is
    not reused, but the behavior is directly proven by a focused Surfari
    harness;
  - `Engine-specific difference` — the behavior is intentionally different
    because Surfari uses WebKit instead of Chromium/Roamium;
  - `Not applicable` — the scenario tests Roamium launch/resolver behavior or
    another Chromium-specific path that is outside Surfari parity;
  - `Gap` — Surfari lacks direct evidence or fails the comparable behavior.
- At minimum, compare the Roamium scenario groups already listed in
  `real-app-matrix.md`:
  - lifecycle/navigation/resize: `browser-command-navigation`, `window-resize`,
    `browser-navigation-geometry`;
  - pane and split geometry: `split-right`, `split-down`, `split-right-resize`,
    `split-right-equalize`, `split-right-zoom`, `split-right-close-sibling`,
    `split-right-close-browser-pane`;
  - tabs/windows/focus: `new-terminal-tab-visibility`,
    `open-browser-in-new-tab`, `close-browser-tab`,
    `open-browser-in-new-window`, `multiple-windows-with-browsers`,
    `keyboard-after-tab-window-switch`, `gui-active-multi-tab`;
  - input details: `browser-input-granularity`, `mouse-after-geometry-change`;
  - profiles/lifecycle/crash: `multi-profile-isolation`,
    `same-profile-server-lifecycle`, `tui-disconnect-reconnect`,
    `renderer-crash-smoke`.
- Also scan the full scenario list in `scripts/ghostboard-geometry-matrix.sh`
  and explicitly account for any omitted scenario as non-comparable,
  engine-specific, already covered, or a gap. This prevents the comparison from
  silently ignoring a Roamium behavior.
- Update `issues/0756-surfari/real-app-matrix.md` only if the fresh aggregate
  evidence proves the `Roamium comparison` row.
- If all matrix rows become `Proven`, update the Issue 756 README conclusion and
  close the issue only after completion review approves the result.

## Verification

Pass criteria:

- Build or confirm required artifacts:

```bash
surfari/libtermsurf_webkit/build.sh
cargo build -p surfari
cargo build -p webtui
(cd ghostboard && zig build)
(cd ghostboard && macos/build.nu --configuration Debug --action build)
```

- Run the aggregate comparison harness.
- The aggregate harness must fail if any child Surfari harness fails.
- The aggregate harness must fail if any child harness log path is missing.
- The aggregate harness must fail if the Debug `TermSurf.app` binary it launches
  is missing or older than Ghostboard source/build inputs.
- The comparison artifact must include every Roamium scenario from the
  `scripts/ghostboard-geometry-matrix.sh` scenario list or explain why a
  scenario is intentionally excluded from Surfari parity.
- The comparison artifact must contain no `Gap` rows for a `Pass` result.
- The `Roamium comparison` row in `real-app-matrix.md` may become `Proven` only
  if the aggregate harness passes and the comparison artifact has no unresolved
  gaps.
- Run hygiene checks:

```bash
git diff --check
bash -n scripts/test-issue-756-surfari-final-comparison.sh
prettier --check --prose-wrap always --print-width 80 \
  issues/0756-surfari/README.md \
  issues/0756-surfari/31-surfari-roamium-comparison.md \
  issues/0756-surfari/real-app-matrix.md \
  issues/0756-surfari/surfari-roamium-comparison.md
```

Run formatting/checks for any source files touched:

```bash
cargo fmt -p surfari -p webtui -- --check
zig fmt <zig-files>
```

Result classification:

- `Pass` means the aggregate Surfari harness suite passes, every comparable
  Roamium scenario is mapped to current Surfari evidence or an intentional
  engine-specific/non-applicable difference, no `Gap` rows remain, and Issue 756
  can be closed after review.
- `Partial` means most comparison evidence passes, but at least one comparable
  behavior remains unproven, flaky, or too weakly mapped.
- `Fail` means the aggregate harness cannot complete or the comparison exposes a
  fundamental Surfari parity gap.

## Design Review

Adversarial design review initially returned `CHANGES REQUIRED` with one
Required finding: the verification plan could run the aggregate comparison
against a stale Debug `TermSurf.app` bundle because it only required
`zig build`, while the child Surfari harnesses launch
`ghostboard/macos/build/Debug/TermSurf.app`.

The design was updated to require building the Debug app bundle with
`macos/build.nu --configuration Debug --action build` and to require the
aggregate harness to fail if the app binary is missing or stale relative to
Ghostboard source/build inputs.

Focused re-review confirmed that stale-bundle finding was resolved, then found
one new command-sequence issue: two consecutive `cd ghostboard && ...` commands
would not be runnable if copied as a single shell block. The design was updated
to use independent subshells:

```bash
(cd ghostboard && zig build)
(cd ghostboard && macos/build.nu --configuration Debug --action build)
```

Final focused re-review returned `APPROVED` with no Required findings. The
reviewer confirmed the command block is now runnable and no new Required finding
was introduced by the fix.
