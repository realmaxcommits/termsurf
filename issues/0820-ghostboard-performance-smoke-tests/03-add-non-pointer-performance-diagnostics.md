# Experiment 3: Add Non-Pointer Performance Diagnostics

## Description

Experiment 1 established a fast repeated-startup smoke. Experiment 2 proved
pointer-dependent diagnostics are blocked in this VM because neither CGEvent nor
System Events click delivery produces the generic AppKit hit-test record.

Issue 820 still needs more useful lightweight coverage than startup alone. This
experiment will add non-pointer resize and split diagnostics that use the
existing app/window automation and keyboard paths, while explicitly leaving
mouse, scroll, and pointer hot-path rows blocked until the VM/input-permission
problem is solved.

## Changes

Planned script changes:

- `scripts/ghostboard-geometry-matrix.sh`
  - Add a `performance-window-resize` scenario that skips only the generic
    initial hit-test prerequisite and then proves grow/shrink resize geometry,
    AppKit presented pixels, Zig presented-pixel records, and Roamium resize
    delivery without any pointer click assertions.
  - Add a `performance-split-right` scenario that skips only the generic initial
    hit-test prerequisite and then proves keyboard-driven split-right geometry,
    AppKit presented pixels, and Roamium resize delivery without pointer click
    assertions.
  - Keep existing correctness scenarios unchanged; do not weaken their hit-test
    assertions.
- `scripts/ghostboard-performance-smoke.sh`
  - Keep `--fast` as the repeated-startup smoke.
  - Change `--diagnostic` to include the fast startup rows plus the new
    non-pointer resize and split rows.
  - Leave pointer-dependent rows out of `--diagnostic` for now because
    Experiment 2 proved they fail before performance can be measured.

Planned issue-document changes:

- Add `## Result` and `## Conclusion` after verification.
- Update the Issue 820 README experiment status after verification.

Explicitly out of scope:

- Ghostboard, Roamium, webtui, protocol, or app source changes.
- Fixing VM pointer injection.
- Precise FPS, CPU, memory, or frame-time benchmarking.
- Adding generated logs or screenshots to git.

## Verification

Formatting actions:

```bash
prettier --write --prose-wrap always --print-width 80 \
  issues/0820-ghostboard-performance-smoke-tests/README.md \
  issues/0820-ghostboard-performance-smoke-tests/03-add-non-pointer-performance-diagnostics.md
```

Static checks:

```bash
bash -n scripts/ghostboard-geometry-matrix.sh
bash -n scripts/ghostboard-performance-smoke.sh
git diff --check
```

Runtime checks:

```bash
scripts/ghostboard-geometry-matrix.sh performance-window-resize
scripts/ghostboard-geometry-matrix.sh performance-split-right
scripts/ghostboard-performance-smoke.sh --fast
scripts/ghostboard-performance-smoke.sh --diagnostic
```

Pass criteria:

- The two new geometry scenarios pass and do not require pointer hit-test
  events.
- Existing correctness scenarios keep their generic hit-test assertions.
- `--fast` still passes the repeated-startup smoke.
- `--diagnostic` passes startup plus the non-pointer resize/split diagnostics
  with bounded-run log paths and elapsed seconds.
- Pointer-dependent mouse/scroll/input rows remain documented as blocked, not
  silently claimed as covered.
- No generated logs or screenshots are staged.

Partial criteria:

- Startup remains green, but one non-pointer diagnostic row exposes a
  scenario-specific app or harness failure.
- The non-pointer rows work individually, but the diagnostic wrapper needs a
  follow-up to classify elapsed thresholds or logs correctly.

Fail criteria:

- The new scenarios weaken existing correctness scenarios' hit-test assertions.
- The fast repeated-startup smoke regresses.
- The diagnostic profile cannot distinguish scenario failure from timeout or
  threshold failure.

## Design Review

This experiment is plan-only until a fresh-context adversarial design review
approves it. Record the reviewer verdict here, fix all real findings, and commit
the approved plan before implementation begins.

External Codex design review using `skills/codex-review`:

- **Initial verdict:** Changes required.
- **Required finding:** Static verification used `bash -n` with two script paths
  in one command, which only parses the first file. Accepted; split the
  verification into separate `bash -n scripts/ghostboard-geometry-matrix.sh` and
  `bash -n scripts/ghostboard-performance-smoke.sh` commands.
- **Final verdict:** Approved.
- **Required findings:** None.
- **Evidence checked:** The reviewer confirmed the README links Experiment 3 as
  `Designed`, the experiment has required sections and a completion gate, scope
  follows Experiment 2's Partial result, existing hit-test correctness scenarios
  are preserved, app/Roamium/webtui/protocol source changes are out of scope,
  and the corrected syntax checks cover both planned shell scripts.

## Completion Gate

After implementation and verification:

- add `## Result` and `## Conclusion` to this experiment file;
- update the Issue 820 README experiment status from `Designed` to `Pass`,
  `Partial`, or `Fail`;
- request a fresh-context completion review;
- fix all real completion-review findings and record the final verdict in this
  file; and
- commit the reviewed result separately before designing or implementing the
  next experiment.
