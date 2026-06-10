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

# Experiment 43: Phase D — alt-screen live A/B recipe

## Description

The live A/B harness now has recipes for text, colors, and clear-screen
behavior. The next feature from Experiment 20's conformance map is the alternate
screen plus cursor addressing: enter the alt screen, clear it, draw fixed text
at specific cursor positions, and capture while the command is sleeping.

This experiment adds an `alt-screen` recipe. It is self-terminating in the same
sense as the earlier Experiment 20 probe: the shell command enters the alternate
screen and sleeps so the harness can capture the alt-screen content; the harness
then kills the launched app PID trees, which tears down the sleeping command. As
with the other Phase-D recipes, strict visual parity is recorded but not
required yet.

## Changes

- `scripts/roastty-app/live-ab-smoke.sh`
  - Add `alt-screen` to `--list-recipes`.
  - Add `--recipe alt-screen`.
  - Update the `--help` / usage text to include `alt-screen`.
  - The recipe command:
    - enters alternate screen mode with `DECSET 1049`,
    - clears the screen,
    - prints a timestamped marker at a fixed row/column,
    - prints additional fixed text at at least two other cursor-addressed
      positions,
    - sleeps before the prompt returns so the capture sees the alt screen.
  - Include the existing `recipe` JSON field with value `alt-screen`.
  - Preserve `smoke`, `ascii-grid`, `color-grid`, and `clear-after`; screenshot
    policy; IOSurface-safe Roastty capture; `swift pngdiff.swift`; and exact
    launched-PID-tree cleanup.
- `scripts/roastty-app/README.md`
  - Document `alt-screen`.
- `issues/0802-libroastty-completion-and-mac-app/README.md`
  - Add this experiment to the index as `Designed`.
  - After implementation, record `alt-screen` under Operating notes if the live
    run succeeds.

## Verification

- Run shell syntax checks:
  - `bash -n scripts/roastty-app/live-ab-smoke.sh`
- Run recipe discovery:
  - `scripts/roastty-app/live-ab-smoke.sh --list-recipes`
  - Confirm it exits `0`, prints `smoke`, `ascii-grid`, `color-grid`,
    `clear-after`, and `alt-screen`, and does not launch either app.
- Run help:
  - `scripts/roastty-app/live-ab-smoke.sh --help`
  - Confirm it exits `0` and usage includes `alt-screen`.
- Run markdown formatting:
  - `prettier --write --prose-wrap always --print-width 80 issues/0802-libroastty-completion-and-mac-app/README.md issues/0802-libroastty-completion-and-mac-app/43-live-ab-alt-screen-recipe.md scripts/roastty-app/README.md`
- Run `git diff --check`.
- If both debug apps are built, run the alt-screen recipe with permissive
  thresholds:
  - `scripts/roastty-app/live-ab-smoke.sh --recipe alt-screen --max-mismatch-ratio 1 --max-mean-channel-delta 255`
  - Confirm the harness exits `0`, prints one JSON summary object, includes
    `"recipe":"alt-screen"`, includes same-sized captures, and cleans up only
    the launched PID trees.
- Run the alt-screen recipe with strict thresholds:
  - `bash -lc 'scripts/roastty-app/live-ab-smoke.sh --recipe alt-screen; rc=$?; echo strict_exit=$rc; exit 0'`
  - Record the current strict verdict and metrics. Strict visual parity is not
    required for this experiment unless the current app state already achieves
    it.
- Run
  `pgrep -fl '[G]hostty.app/Contents/MacOS/ghostty|[R]oastty.app/Contents/MacOS/roastty' || true`
  and verify no launched app processes remain.
- Run `git status --short` and verify no screenshots or generated artifacts are
  in the repo.

**Pass** = `alt-screen` is discoverable, runs live through the A/B harness, JSON
identifies the recipe, screenshots stay outside the repo, strict metrics are
recorded without overclaiming parity, and launched app processes are cleaned up.

**Partial** = the recipe is syntax-checked and documented, but a local
app-build, accessibility, screen-recording, or live-window condition prevents a
full live run; the blocker and next command are recorded.

**Fail** = the recipe makes the harness unreliable or cannot be added without a
larger rewrite.

## Design Review

**Reviewer:** Codex-native adversarial subagent (`multi_agent_v1.spawn_agent`,
fresh context, read-only). **Verdict: APPROVED with no findings.**

## Result

**Result:** Pass

Added `alt-screen` to the live A/B harness recipe layer:

- `--list-recipes` now prints `smoke`, `ascii-grid`, `color-grid`,
  `clear-after`, and `alt-screen`.
- `--help` / usage text now lists
  `smoke|ascii-grid|color-grid|clear-after|alt-screen`.
- `--recipe alt-screen` enters alternate screen mode (`DECSET 1049`), clears the
  screen, writes a timestamped marker at row 5 / column 10, writes fixed text at
  two additional cursor-addressed positions, and sleeps before the prompt
  returns so the capture sees the alt screen.
- Existing recipes, IOSurface-safe Roastty capture, `swift pngdiff.swift`,
  screenshot policy, and exact launched-PID cleanup are preserved.

Updated `scripts/roastty-app/README.md` and the Issue 802 Operating notes with
`alt-screen`. The Issue 802 experiment index now marks Experiment 43 `Pass`.

Verification:

- `bash -n scripts/roastty-app/live-ab-smoke.sh`
- `scripts/roastty-app/live-ab-smoke.sh --list-recipes`
  - Exited `0`.
  - Printed `smoke`, `ascii-grid`, `color-grid`, `clear-after`, and
    `alt-screen`.
  - Did not launch either app.
- `scripts/roastty-app/live-ab-smoke.sh --help`
  - Exited `0`.
  - Printed usage including
    `--recipe smoke|ascii-grid|color-grid|clear-after|alt-screen`.
- Alt-screen recipe permissive run:
  - `scripts/roastty-app/live-ab-smoke.sh --recipe alt-screen --max-mismatch-ratio 1 --max-mean-channel-delta 255`
  - Exited `0`.
  - Launched Ghostty PID `56114` and Roastty PID `56129`.
  - Captured both comparison images at `1000x1000`.
  - Printed one JSON summary object with `recipe: alt-screen`, `verdict: PASS`,
    `diff_exit_status: 0`, `mismatch_ratio: 1`, and
    `mean_channel_delta: 108.093655`.
  - The trap killed Ghostty descendants `56122`, `56123`, Ghostty PID `56114`,
    Roastty descendant `56136`, and Roastty PID `56129`.
- Alt-screen recipe strict run:
  - `bash -lc 'scripts/roastty-app/live-ab-smoke.sh --recipe alt-screen; rc=$?; echo strict_exit=$rc; exit 0'`
  - Harness exited `1`, wrapper printed `strict_exit=1`.
  - Launched Ghostty PID `56369` and Roastty PID `56383`.
  - Captured both comparison images at `1600x1264`.
  - Printed one JSON summary object with `recipe: alt-screen`, `verdict: FAIL`,
    `diff_exit_status: 1`, `mismatch_ratio: 0.9541193631329113`, and
    `mean_channel_delta: 8.774260779272153`.
  - The trap killed Ghostty descendants `56376`, `56377`, Ghostty PID `56369`,
    Roastty descendant `56390`, and Roastty PID `56383`.
- `pgrep -fl '[G]hostty.app/Contents/MacOS/ghostty|[R]oastty.app/Contents/MacOS/roastty' || true`
  - no output after cleanup.
- `prettier --write --prose-wrap always --print-width 80 issues/0802-libroastty-completion-and-mac-app/README.md issues/0802-libroastty-completion-and-mac-app/43-live-ab-alt-screen-recipe.md scripts/roastty-app/README.md`
- `git diff --check`
- `git status --short`
  - no screenshot or PNG artifacts in the repo.

## Conclusion

The live A/B harness now has an alternate-screen fixture with cursor-addressed
content. This moves another Experiment 20 conformance probe into the reusable
Phase-D visual comparison surface.

Strict parity still fails, and this experiment does not claim otherwise. The
important progress is that alternate screen and cursor addressing now have a
repeatable live app comparison recipe with machine-readable metrics.

## Completion Review

**Reviewer:** Codex-native adversarial subagent (`multi_agent_v1.spawn_agent`,
fresh context, read-only). **Verdict: APPROVED with no findings.**

The reviewer independently ran `bash -n scripts/roastty-app/live-ab-smoke.sh`,
`scripts/roastty-app/live-ab-smoke.sh --list-recipes`,
`scripts/roastty-app/live-ab-smoke.sh --help`, `git diff --check`, the scoped
`pgrep` process check, `git status --short`, and source/diff inspection. It did
not run the live GUI harness.
