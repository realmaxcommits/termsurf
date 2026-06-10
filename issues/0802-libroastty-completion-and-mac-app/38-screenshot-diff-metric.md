+++
[implementer]
agent = "codex"
model = "gpt-5"
reasoning = "high"

[review.design]
agent = "codex-adversarial"
model = "gpt-5"
reasoning = "high"

[review.result]
agent = "codex-adversarial"
model = "gpt-5"
reasoning = "high"
+++

# Experiment 38: Phase D — screenshot diff metric for live A/B checks

## Description

Phase D needs a repeatable way to compare the real Ghostty app and the
roastty-backed app without committing screenshots. The issue policy already
requires live A/B visual checks: capture both apps in the same run under
identical input, diff them, and record only a verdict / metric. The missing
piece is the metric tool itself.

This experiment adds a small, deterministic PNG comparison helper for window
captures produced by the existing `scripts/ghostty-app/screenshot.sh` and
`scripts/roastty-app/screenshot.sh` wrappers. It is intentionally only the
visual-diff primitive: it does not yet launch both apps, drive identical shell
input, choose crop regions, or define the full feature matrix. Those later
experiments can use this helper as their shared oracle.

The helper must keep all screenshots outside the repo and print one
machine-readable JSON object to stdout suitable for experiment logs: dimensions,
compared pixels, mismatched pixels, mismatch ratio, mean absolute channel error,
max channel error, thresholds, and pass/fail verdict. Diagnostics and usage
errors go to stderr.

## Changes

- `scripts/roastty-app/pngdiff.swift`
  - Add a Swift/AppKit PNG diff helper:
    `swift scripts/roastty-app/pngdiff.swift <expected.png> <actual.png> [--max-mismatch-ratio N] [--max-mean-channel-delta N]`.
  - Load both PNGs as `NSBitmapImageRep`, normalize to RGBA bytes, and fail on
    dimension mismatch.
  - Compare every pixel/channel and print one JSON object to stdout:
    - width/height,
    - compared pixel count,
    - mismatched pixel count,
    - mismatch ratio,
    - mean absolute channel delta,
    - max absolute channel delta,
    - max mismatch ratio threshold,
    - max mean channel delta threshold,
    - verdict (`PASS`/`FAIL`).
  - Print diagnostics and usage errors to stderr only.
  - Exit `0` on pass and nonzero on fail or invalid input.
  - Never write images or artifacts.
- `scripts/roastty-app/README.md`
  - If absent, add a short helper README for Phase-D roastty-app automation.
  - Document the screenshot policy and `pngdiff.swift` usage.
- `issues/0802-libroastty-completion-and-mac-app/README.md`
  - Add the experiment to the index as `Designed`.
  - After implementation, update the Screenshots / Operating notes section to
    record the diff metric tool and the fact that it stores no images.

## Verification

- Run markdown formatting:
  - `prettier --write --prose-wrap always --print-width 80 issues/0802-libroastty-completion-and-mac-app/README.md issues/0802-libroastty-completion-and-mac-app/38-screenshot-diff-metric.md scripts/roastty-app/README.md`
- Run `git diff --check`.
- Create temporary PNG fixtures outside the repo under `/tmp/termsurf-pngdiff`:
  - identical images → `PASS`, exit `0`, mismatch ratio `0`;
  - one pixel changed with a strict threshold → `FAIL`, nonzero exit, nonzero
    mismatch ratio and channel deltas;
  - one pixel changed with permissive thresholds → `PASS`, exit `0`;
  - dimension mismatch → `FAIL`, nonzero exit.
- Run `swift scripts/roastty-app/pngdiff.swift --help` or invalid args to prove
  usage errors are clear.
- Run `git status --short` and verify no PNG or screenshot artifacts are in the
  repo.

**Pass** = the helper computes deterministic metrics, threshold pass/fail works,
stdout is a single JSON object, dimension mismatch fails, no images are written,
docs/index are updated, and the working tree contains no screenshot artifacts.

**Partial** = the helper compares images but thresholding or docs need
follow-up.

**Fail** = Swift/AppKit cannot provide a reliable image comparison helper on
this machine.

## Design Review

**Reviewer:** Codex-native adversarial subagent (`multi_agent_v1.spawn_agent`,
fresh context, read-only). **Verdict: APPROVED.** It verified the README links
Experiment 38 as `Designed`, the experiment has Description / Changes /
Verification, the scope is narrow and moves Phase D forward, the screenshot
policy is obeyed, and `git diff --check` passed for the design files.

The review found one Optional issue: "machine-readable metrics" should pin the
output format. Fixed by specifying one JSON object on stdout with diagnostics on
stderr and adding threshold fields to the output contract.

## Result

**Result:** Pass

Added the Phase-D screenshot diff primitive:

- `scripts/roastty-app/pngdiff.swift`
  - Loads two PNGs with Swift/AppKit.
  - Compares dimensions and every RGBA channel.
  - Emits exactly one JSON metrics object to stdout on compare paths.
  - Sends usage / load diagnostics to stderr.
  - Supports exact defaults plus `--max-mismatch-ratio` and
    `--max-mean-channel-delta` thresholds.
  - Writes no images or artifacts.
- `scripts/roastty-app/README.md`
  - Documents screenshot artifact policy and `pngdiff.swift` usage.
- `issues/0802-libroastty-completion-and-mac-app/README.md`
  - Records `pngdiff.swift` under Screenshots / Operating notes.
  - Marks Experiment 38 as `Pass`.

Verification:

- `swift scripts/roastty-app/pngdiff.swift --help`
  - printed usage to stderr and exited `0`.
- Created temporary PNG fixtures under `/tmp/termsurf-pngdiff`, then removed
  them after verification.
- Identical 2x2 images:
  - `verdict: PASS`, `mismatch_ratio: 0`, exit `0`.
- One-pixel-different 2x2 images with exact thresholds:
  - `verdict: FAIL`, `mismatched_pixels: 1`, `mismatch_ratio: 0.25`, nonzero
    channel deltas, exit `1`.
- Same one-pixel-different images with permissive thresholds:
  - `verdict: PASS`, exit `0`.
- Dimension mismatch:
  - `verdict: FAIL`, `error: dimension_mismatch`, exit `1`.
- `prettier --write --prose-wrap always --print-width 80 issues/0802-libroastty-completion-and-mac-app/README.md issues/0802-libroastty-completion-and-mac-app/38-screenshot-diff-metric.md scripts/roastty-app/README.md`
- `git diff --check`
- `git status --short`
  - no PNG or screenshot artifacts in the repo.

## Conclusion

Issue 802 now has a stable, machine-readable screenshot diff metric for Phase-D
live A/B checks. Later experiments can launch Ghostty and Roastty, drive
identical inputs, capture both windows through the existing screenshot wrappers,
and use `pngdiff.swift` to record a verdict without committing images.

This does not complete Phase D by itself: the full live app driver, input
recipes, crop policy, thresholds per feature, and behavior matrix still need
follow-up experiments.

## Completion Review

**Reviewer:** Codex-native adversarial subagent (`multi_agent_v1.spawn_agent`,
fresh context, read-only). **Verdict: APPROVED with no findings.**

The reviewer independently verified that:

- `swift scripts/roastty-app/pngdiff.swift --help` exits `0`, writes usage to
  stderr, and writes no stdout.
- invalid args exit `2` and write no stdout.
- `swiftc -parse scripts/roastty-app/pngdiff.swift` exits `0`.
- `git diff --check` exits `0`.
- `git status --short` shows only expected source/docs changes and no PNG or
  screenshot artifacts.
- the latest commit is the Experiment 38 plan commit and result changes are not
  committed before review.

The reviewer did not recreate the `/tmp/termsurf-pngdiff` fixture matrix because
the adversarial review was read-only by discipline.
