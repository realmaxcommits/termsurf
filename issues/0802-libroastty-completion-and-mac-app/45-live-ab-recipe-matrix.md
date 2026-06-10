+++
[implementer]
agent = "codex"
model = "gpt-5"
reasoning = "high"

[review.design]
agent = "codex-adversarial"
model = "gpt-5"
reasoning = "high"
+++

# Experiment 45: Phase D — live A/B recipe matrix runner

## Description

Experiments 40–44 added individual live A/B recipes, but Phase D still needs a
repeatable in-session run wired so later work can regression-test the current
feature surface without manually invoking each recipe. This experiment adds a
small matrix runner over the existing `live-ab-smoke.sh` recipes.

The runner should execute a selected set of recipes, keep the permissive
thresholds configurable, print one machine-readable JSON Lines summary per
recipe, and exit nonzero if any selected recipe fails under the supplied
thresholds. It should not introduce new screenshot storage rules or new visual
judgment logic; it composes the existing harness and `pngdiff.swift` outputs.
Strict visual parity remains a separate per-recipe metric, not a pass
requirement for this matrix runner.

## Changes

- `scripts/roastty-app/live-ab-matrix.sh`
  - Add a Bash runner around `scripts/roastty-app/live-ab-smoke.sh`.
  - Default to running every recipe reported by
    `live-ab-smoke.sh --list-recipes`.
  - Support selecting a subset with repeated `--recipe <name>`.
  - Support threshold passthrough:
    - `--max-mismatch-ratio <N>`
    - `--max-mean-channel-delta <N>`
  - Default thresholds should be permissive (`1` and `255`) so the matrix proves
    harness mechanics and current coverage rather than strict parity.
  - For each recipe, run `live-ab-smoke.sh`, capture its single JSON summary,
    and print one JSON Lines object containing at least:
    - `recipe`,
    - `status` (`PASS` / `FAIL`),
    - child exit status,
    - nested harness JSON summary.
  - Continue running remaining recipes after a recipe fails, then exit nonzero
    if any recipe failed.
  - Preserve the existing screenshot policy: no screenshots or generated
    artifacts in the repo.
- `scripts/roastty-app/README.md`
  - Document the matrix runner and a one-recipe smoke invocation.
- `issues/0802-libroastty-completion-and-mac-app/README.md`
  - Add this experiment to the index as `Designed`.
  - After implementation, record the matrix command under Operating notes if the
    live run succeeds.

## Verification

- Run shell syntax checks:
  - `bash -n scripts/roastty-app/live-ab-matrix.sh`
  - `bash -n scripts/roastty-app/live-ab-smoke.sh`
- Run a non-GUI recipe discovery check:
  - `scripts/roastty-app/live-ab-smoke.sh --list-recipes`
- Run markdown formatting:
  - `prettier --write --prose-wrap always --print-width 80 issues/0802-libroastty-completion-and-mac-app/README.md issues/0802-libroastty-completion-and-mac-app/45-live-ab-recipe-matrix.md scripts/roastty-app/README.md`
- Run `git diff --check`.
- If both debug apps are built, run a one-recipe matrix smoke:
  - `scripts/roastty-app/live-ab-matrix.sh --recipe smoke`
  - Confirm it exits `0`, prints one JSON Lines object, includes
    `"recipe":"smoke"`, includes nested harness JSON, and cleans up only the
    launched PID trees.
- If the one-recipe smoke passes, run a two-recipe matrix:
  - `scripts/roastty-app/live-ab-matrix.sh --recipe ascii-grid --recipe clear-after`
  - Confirm it exits `0` with permissive defaults and prints exactly two JSON
    Lines objects.
- Run an intentional failure aggregation check with strict thresholds:
  - `bash -lc 'scripts/roastty-app/live-ab-matrix.sh --recipe ascii-grid --recipe clear-after --max-mismatch-ratio 0 --max-mean-channel-delta 0; rc=$?; echo matrix_exit=$rc; exit 0'`
  - Confirm the matrix prints one JSON Lines object for `ascii-grid` and one for
    `clear-after`, at least one has `status:"FAIL"`, the later recipe still ran
    after the first failure, and the wrapper prints a nonzero `matrix_exit`.
- Run
  `pgrep -fl '[G]hostty.app/Contents/MacOS/ghostty|[R]oastty.app/Contents/MacOS/roastty' || true`
  and verify no launched app processes remain.
- Run `git status --short` and verify no screenshots or generated artifacts are
  in the repo.

**Pass** = the matrix runner can execute selected recipes, emit JSON Lines
summaries, continue after failures, report aggregate failure by exit status,
preserve screenshot hygiene, and leave no app processes running.

**Partial** = the runner is syntax-checked and documented, but a local
app-build, accessibility, screen-recording, or live-window condition prevents a
full live run; the blocker and next command are recorded.

**Fail** = composing recipes into a reliable runner requires a larger harness
rewrite.

## Design Review

**Reviewer:** Codex-native adversarial subagent (`multi_agent_v1.spawn_agent`,
fresh context, read-only). **Verdict: APPROVED after fixes.**

The first review returned `CHANGES REQUIRED` with one Required finding: the
design promised continue-after-failure and aggregate nonzero exit behavior, but
only verified permissive passing runs. Fixed by adding an intentional
strict-threshold failure aggregation check that must emit JSON Lines for both
selected recipes, show at least one `FAIL`, prove the later recipe still ran,
and report nonzero `matrix_exit`.

The focused re-review approved the fix and found no new Required issues.

## Result

**Result:** Pass

Added `scripts/roastty-app/live-ab-matrix.sh`, a JSON Lines matrix runner around
`live-ab-smoke.sh`:

- Defaults to every recipe from `live-ab-smoke.sh --list-recipes`.
- Supports repeated `--recipe <name>` for subsets.
- Supports `--max-mismatch-ratio` and `--max-mean-channel-delta`.
- Defaults to permissive thresholds (`1` and `255`).
- Emits one JSON Lines object per recipe with:
  - `recipe`,
  - `status`,
  - `child_exit_status`,
  - nested `summary` from `live-ab-smoke.sh`.
- Continues after recipe failures and exits nonzero if any selected recipe
  fails.

Updated `scripts/roastty-app/README.md` and the Issue 802 Operating notes with
the matrix command. The Issue 802 experiment index now marks Experiment 45
`Pass`, and the Phase D roadmap now marks the repeatable in-session run item
complete.

Verification:

- `bash -n scripts/roastty-app/live-ab-matrix.sh`
- `bash -n scripts/roastty-app/live-ab-smoke.sh`
- `scripts/roastty-app/live-ab-matrix.sh --help`
  - Exited `0`.
- `scripts/roastty-app/live-ab-smoke.sh --list-recipes`
  - Printed `smoke`, `ascii-grid`, `color-grid`, `clear-after`, `alt-screen`,
    and `scroll-output`.
- One-recipe matrix smoke:
  - `scripts/roastty-app/live-ab-matrix.sh --recipe smoke`
  - Exited `0`.
  - Printed one JSON Lines object with `recipe: smoke`, `status: PASS`,
    `child_exit_status: 0`, and nested harness summary.
  - Launched Ghostty PID `59577` and Roastty PID `59592`.
  - The child harness killed Ghostty descendants `59585`, `59586`, Ghostty PID
    `59577`, Roastty descendant `59599`, and Roastty PID `59592`.
- Two-recipe permissive matrix:
  - `scripts/roastty-app/live-ab-matrix.sh --recipe ascii-grid --recipe clear-after`
  - Exited `0`.
  - Printed two JSON Lines objects: `ascii-grid` then `clear-after`, both with
    `status: PASS` and `child_exit_status: 0`.
  - The child harnesses killed all launched Ghostty/Roastty PID trees.
- Strict failure aggregation check:
  - `bash -lc 'scripts/roastty-app/live-ab-matrix.sh --recipe ascii-grid --recipe clear-after --max-mismatch-ratio 0 --max-mean-channel-delta 0; rc=$?; echo matrix_exit=$rc; exit 0'`
  - Printed two JSON Lines objects: `ascii-grid` then `clear-after`.
  - Both objects had `status: FAIL` and `child_exit_status: 1`.
  - The second recipe still ran after the first failure.
  - Wrapper printed `matrix_exit=1`.
  - The child harnesses killed all launched Ghostty/Roastty PID trees.
- `pgrep -fl '[G]hostty.app/Contents/MacOS/ghostty|[R]oastty.app/Contents/MacOS/roastty' || true`
  - no output after cleanup.
- `prettier --write --prose-wrap always --print-width 80 issues/0802-libroastty-completion-and-mac-app/README.md issues/0802-libroastty-completion-and-mac-app/45-live-ab-recipe-matrix.md scripts/roastty-app/README.md`
- `git diff --check`
- `git status --short`
  - no screenshot or PNG artifacts in the repo.

## Completion Review

**Reviewer:** Codex-native adversarial subagent (`multi_agent_v1.spawn_agent`,
fresh context, read-only). **Verdict: APPROVED.**

The reviewer found no Required issues. Notes recorded that the review did not
launch the GUI live matrix runs, but read-only checks passed for both scripts,
the matrix help output, recipe listing, `git diff --check`, status/artifact
checks, and the scoped app-process `pgrep`. After the approval, the runner's
default-all recipe collection was changed from `mapfile` to a Bash 3.2
compatible `while read` loop so macOS system Bash can run that path.

A focused Codex-native re-review of the final state approved that compatibility
edit, the roadmap checkbox update, and the documentation wording with no
Required findings.

## Conclusion

Phase D now has a repeatable in-session recipe matrix runner instead of only
individual recipe commands. Later experiments can use it to run the current live
visual conformance surface as a regression check, add recipes incrementally, and
intentionally run strict thresholds when recording current visual differences.
