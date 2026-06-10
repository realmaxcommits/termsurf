+++
[implementer]
agent = "codex"
model = "gpt-5"
reasoning = "high"

[review.design]
agent = "codex"
+++

# Experiment 49: Phase D — content-region live A/B diffs

## Description

Experiment 48 made the live A/B harness execute deterministic recipes and hold
their final frames through capture. The remaining full-window diff still
includes differences that are not the terminal rendering behavior under test:
the app titlebar icon/name and the debug-build banner text (`Ghostty` vs
`Roastty`). Those pixels are useful for diagnosing window setup, but they make
the Phase-D golden comparison too noisy to serve as the terminal conformance
oracle.

This experiment adds an explicit content-region diff mode to the live A/B
harness. The harness should keep producing the existing full-window crop and
full-window diff for continuity, but it should also crop both app captures to a
shared terminal-content region below the titlebar/debug banner and run the same
PNG diff there. The content diff becomes the metric used for stricter
Ghostty-vs-Roastty terminal behavior checks, while the full-window diff remains
available in JSON for context.

The initial crop can be a fixed pixel inset in the already-normalized 1600x1264
window crops. A quick measurement on the latest Exp-48 ASCII captures showed
that cropping from `y=132` for a 1600x900 region reduces `mismatch_ratio` from
`0.07274970332278481` to `0.02223125`, confirming that the crop removes mostly
chrome/banner noise while preserving the recipe text and cursor region. This
experiment should not try to fix the remaining rendering deltas; it should make
the comparison target precise enough that those deltas can be addressed in later
experiments.

## Changes

- `scripts/roastty-app/live-ab-smoke.sh`
  - Add a content-region diff mode enabled by default, with configurable pixel
    crop controls for the normalized app-window captures.
  - Preserve the existing full-window crop PNGs and full-window diff metrics.
  - Write content-region crop PNGs outside the repo next to the existing
    captures.
  - Emit both metrics in the JSON summary: full-window context and
    content-region verdict/metrics.
  - Make the script exit according to the content-region diff when the content
    mode is enabled, while retaining a way to force the legacy full-window-only
    behavior for compatibility/debugging.
- `scripts/roastty-app/live-ab-matrix.sh`
  - Forward any new content-region threshold/options needed by
    `live-ab-smoke.sh`, or inherit the smoke harness defaults if no forwarding
    is necessary.
- `scripts/roastty-app/README.md`
  - Document the content-region diff, the crop controls, and the meaning of the
    two metric sets.
- `issues/0802-libroastty-completion-and-mac-app/README.md`
  - Add this experiment to the index as `Designed`.
  - After implementation, update Operating notes and the Phase-D roadmap if the
    content-region A/B diff is the new golden comparison path.

## Verification

- Run shell syntax checks:
  - `bash -n scripts/roastty-app/live-ab-smoke.sh`
  - `bash -n scripts/roastty-app/live-ab-matrix.sh`
- Run non-GUI recipe discovery:
  - `scripts/roastty-app/live-ab-smoke.sh --list-recipes`
- Run a representative content-region smoke:
  - `scripts/roastty-app/live-ab-smoke.sh --recipe ascii-grid --max-mismatch-ratio 1 --max-mean-channel-delta 255`
  - Confirm the JSON contains both full-window and content-region metric
    objects, with stable paths to the two content crop PNGs outside the repo.
  - Confirm the content-region `mismatch_ratio` is lower than the full-window
    `mismatch_ratio` for the ASCII recipe, and record both values.
- Run the full default matrix:
  - `scripts/roastty-app/live-ab-matrix.sh`
  - Confirm it exits `0`, emits one JSON Lines object for every recipe, and each
    line includes both metric sets.
- Run a stricter content threshold probe:
  - Choose a threshold below the measured full-window `mismatch_ratio` and above
    the measured content-region `mismatch_ratio`, then run at least the ASCII
    recipe to prove content mode passes while legacy full-window-only mode
    fails.
- Run markdown formatting:
  - `prettier --write --prose-wrap always --print-width 80 issues/0802-libroastty-completion-and-mac-app/README.md issues/0802-libroastty-completion-and-mac-app/49-live-ab-content-region-diff.md scripts/roastty-app/README.md`
- Run `git diff --check`.
- Run cleanup checks:
  - `scripts/roastty-app/stop-app.sh || true`
  - `scripts/ghostty-app/stop-app.sh || true`
  - `pgrep -fl '[G]hostty.app/Contents/MacOS/ghostty|[R]oastty.app/Contents/MacOS/roastty' || true`
  - `find /tmp -maxdepth 1 -name 'termsurf-ab-bootstrap.*' -print`
- Run `git status --short` and verify no screenshots or generated artifacts are
  in the repo.

**Pass** = the harness reports both full-window and content-region diffs, exits
according to the content-region metric by default, the full matrix remains
repeatable, stricter content gating works for at least one representative
recipe, screenshots remain outside the repo, and no launched app processes or
bootstrap temp dirs remain.

## Design Review

**Reviewer:** Codex-native adversarial subagent (`multi_agent_v1.spawn_agent`,
fresh context, read-only). **Verdict: APPROVED.**

The reviewer found no Required issues. It noted one Optional improvement: make
the stricter threshold probe mechanically reproducible by choosing a threshold
between the measured full-window and content-region mismatch ratios. It also
noted one nit: replace the subjective phrase "materially lower" with a concrete
comparison and record both values. Both were fixed before the plan commit.

**Partial** = content-region metrics are emitted and useful, but full-matrix or
strict-threshold gating is blocked by local capture/window conditions; record
the exact blocker and next command.

**Fail** = fixed content-region cropping cannot reliably isolate the terminal
content across the current Ghostty and Roastty app captures.
