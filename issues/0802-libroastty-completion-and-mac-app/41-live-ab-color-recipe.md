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

# Experiment 41: Phase D — color live A/B recipe

## Description

Experiment 40 added the recipe layer for the live Ghostty/Roastty A/B harness.
The next useful feature recipe is color rendering: ANSI palette colors,
background colors, bold brightening, and truecolor are core terminal behaviors
that later conformance work needs to compare against the real app.

This experiment adds a `color-grid` recipe to `live-ab-smoke.sh`. The recipe is
a visual oracle only: it prints deterministic ANSI / truecolor rows and records
the current A/B screenshot-diff metrics. It does not require strict visual
parity yet, because the existing strict A/B recipes still fail. The value is
creating a repeatable live color fixture that later renderer/config work can use
as a regression target.

## Changes

- `scripts/roastty-app/live-ab-smoke.sh`
  - Add `color-grid` to `--list-recipes`.
  - Add `--recipe color-grid`.
  - Update the `--help` / usage text to include `color-grid`.
  - The recipe clears the screen, moves the cursor home, prints a timestamped
    marker, then prints deterministic rows covering:
    - basic ANSI foreground colors,
    - ANSI background colors,
    - bold/bright foreground colors,
    - truecolor foreground/background samples.
  - Keep the command self-contained and sleeping long enough for capture before
    the shell prompt returns.
  - Include the existing `recipe` JSON field with value `color-grid`.
  - Preserve `smoke` default compatibility, the `ascii-grid` recipe, screenshot
    policy, IOSurface-safe Roastty capture path, `swift pngdiff.swift`, and
    exact launched-PID-tree cleanup.
- `scripts/roastty-app/README.md`
  - Document `color-grid`.
- `issues/0802-libroastty-completion-and-mac-app/README.md`
  - Add this experiment to the index as `Designed`.
  - After implementation, record `color-grid` under Operating notes if the live
    run succeeds.

## Verification

- Run shell syntax checks:
  - `bash -n scripts/roastty-app/live-ab-smoke.sh`
- Run recipe discovery:
  - `scripts/roastty-app/live-ab-smoke.sh --list-recipes`
  - Confirm it exits `0`, prints `smoke`, `ascii-grid`, and `color-grid`, and
    does not launch either app.
- Run markdown formatting:
  - `prettier --write --prose-wrap always --print-width 80 issues/0802-libroastty-completion-and-mac-app/README.md issues/0802-libroastty-completion-and-mac-app/41-live-ab-color-recipe.md scripts/roastty-app/README.md`
- Run `git diff --check`.
- If both debug apps are built, run the color recipe with permissive thresholds:
  - `scripts/roastty-app/live-ab-smoke.sh --recipe color-grid --max-mismatch-ratio 1 --max-mean-channel-delta 255`
  - Confirm the harness exits `0`, prints one JSON summary object, includes
    `"recipe":"color-grid"`, includes same-sized captures, and cleans up only
    the launched PID trees.
- Run the color recipe with strict thresholds:
  - `bash -lc 'scripts/roastty-app/live-ab-smoke.sh --recipe color-grid; rc=$?; echo strict_exit=$rc; exit 0'`
  - Record the current strict verdict and metrics. Strict visual parity is not
    required for this experiment unless the current app state already achieves
    it.
- Run
  `pgrep -fl '[G]hostty.app/Contents/MacOS/ghostty|[R]oastty.app/Contents/MacOS/roastty' || true`
  and verify no launched app processes remain.
- Run `git status --short` and verify no screenshots or generated artifacts are
  in the repo.

**Pass** = `color-grid` is discoverable, runs live through the A/B harness, JSON
identifies the recipe, screenshots stay outside the repo, strict metrics are
recorded without overclaiming parity, and launched app processes are cleaned up.

**Partial** = the recipe is syntax-checked and documented, but a local
app-build, accessibility, screen-recording, or live-window condition prevents a
full live run; the blocker and next command are recorded.

**Fail** = the recipe makes the harness unreliable or cannot be added without a
larger rewrite.

## Design Review

**Reviewer:** Codex-native adversarial subagent (`multi_agent_v1.spawn_agent`,
fresh context, read-only). **Verdict: APPROVED.**

The reviewer found no Required issues. It noted one Optional completeness issue:
the design mentioned `--list-recipes`, docs, and `--recipe color-grid`, but did
not explicitly call out the script's `--help` / usage text, which still listed
only `smoke|ascii-grid`. Fixed by adding the usage text update to the planned
changes.
