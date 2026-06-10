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

# Experiment 46: Phase D — paste-safe live A/B recipe input

## Description

Experiment 45 made the live A/B recipes repeatable, but inspecting recent
captures showed the next blocker is harness reliability rather than renderer
parity: a Ghostty capture can be blank, and the Roastty side can show a recipe
command mangled by synthetic typing and shell parsing (`printf` treating recipe
text as format directives, escape sequences arriving as control characters, and
the visible shell prompt entering quote-continuation state).

This experiment makes recipe delivery deterministic before using strict A/B
diffs as evidence. The harness should deliver each generated command to both
apps by paste, not character-by-character `System Events keystroke`, and the
recipes should avoid `printf` format-string hazards. The goal is not to make
Roastty match Ghostty visually yet; it is to ensure both apps are asked to
render the same intended recipe so future diffs measure terminal behavior, not
input injection artifacts.

## Changes

- `scripts/roastty-app/live-ab-smoke.sh`
  - Replace command entry through `System Events keystroke (read POSIX file …)`
    with pasteboard-based command delivery:
    - write the full command string to a temporary file;
    - load it into the macOS pasteboard;
    - activate the target app;
    - dismiss any partial prompt state with Escape;
    - press Command-V;
    - press Return.
  - Preserve the current app launch, window sizing, capture, diff, and exact
    launched-PID-tree cleanup behavior.
  - Rewrite recipe shell commands so ANSI payloads are emitted as data, not as
    the `printf` format string. `@#$%^…`, `%`, backslashes, and ESC sequences
    must not put either shell into quote-continuation or `printf` error states.
  - Keep `smoke` as the default recipe and preserve all existing recipe names.
- `scripts/roastty-app/README.md`
  - Document that live A/B commands are pasted as whole commands and that
    recipes are written to be shell-format safe.
- `issues/0802-libroastty-completion-and-mac-app/README.md`
  - Add this experiment to the index as `Designed`.
  - After implementation, update the Operating notes if paste delivery proves
    reliable.

## Verification

- Run shell syntax checks:
  - `bash -n scripts/roastty-app/live-ab-smoke.sh`
  - `bash -n scripts/roastty-app/live-ab-matrix.sh`
- Run non-GUI recipe discovery:
  - `scripts/roastty-app/live-ab-smoke.sh --list-recipes`
- Run the default one-recipe live A/B smoke with permissive thresholds:
  - `scripts/roastty-app/live-ab-smoke.sh --recipe smoke --max-mismatch-ratio 1 --max-mean-channel-delta 255`
  - Confirm both apps receive the command, the Ghostty capture is not blank, the
    Roastty capture does not show quote-continuation or `printf` error output,
    the JSON summary is emitted, and only launched PID trees are killed.
- Run the recipe that exposed the `%`/escaping problem:
  - `scripts/roastty-app/live-ab-smoke.sh --recipe ascii-grid --max-mismatch-ratio 1 --max-mean-channel-delta 255`
  - Confirm the captured terminal output contains the marker, uppercase row,
    lowercase row, digit row, and punctuation row without `printf` errors or
    shell quote prompts.
- Run the full default matrix:
  - `scripts/roastty-app/live-ab-matrix.sh`
  - Confirm it exits `0` with permissive thresholds, emits one JSON Lines object
    for every recipe reported by `live-ab-smoke.sh --list-recipes`, and no
    recipe output contains quote-continuation prompts, shell syntax errors, or
    `printf` errors.
- Run markdown formatting:
  - `prettier --write --prose-wrap always --print-width 80 issues/0802-libroastty-completion-and-mac-app/README.md issues/0802-libroastty-completion-and-mac-app/46-live-ab-paste-safe-recipes.md scripts/roastty-app/README.md`
- Run `git diff --check`.
- Run
  `pgrep -fl '[G]hostty.app/Contents/MacOS/ghostty|[R]oastty.app/Contents/MacOS/roastty' || true`
  and verify no launched app processes remain.
- Run `git status --short` and verify no screenshots or generated artifacts are
  in the repo.

**Pass** = recipe commands are delivered reliably by paste, the `%`/escape
recipe no longer produces shell/`printf` artifacts, live A/B JSON still emits,
the matrix still composes selected recipes, screenshots remain outside the repo,
and no app processes remain.

**Partial** = paste delivery is implemented and syntax-checked, but a local
accessibility, pasteboard, screen-recording, app-build, or live-window condition
prevents live verification; record the blocker and next command.

**Fail** = reliable recipe delivery requires a larger automation rewrite.

## Design Review

**Reviewer:** Codex-native adversarial subagent (`multi_agent_v1.spawn_agent`,
fresh context, read-only). **Verdict: APPROVED after fixes.**

The first review returned `CHANGES REQUIRED` with one Required finding: because
the experiment rewrites all recipe commands, verifying only `ascii-grid` and
`clear-after` would not prove `color-grid`, `alt-screen`, and `scroll-output`
survived the paste/shell/format changes. Fixed by replacing the two-recipe
matrix verification with the full default matrix, requiring one JSON Lines
object for every recipe from `live-ab-smoke.sh --list-recipes` and permissive
threshold exit `0`.

The focused re-review approved the fix and found no remaining Required issues.

## Result

**Result:** Partial

The experiment found that pasteboard delivery is not viable against the current
Roastty app: sending Command-V to Roastty terminates the app, even with a simple
`abc` pasteboard payload. The implementation therefore did not keep the planned
paste path. Instead, it made the harness safer and narrowed the real blocker:

- `scripts/roastty-app/live-ab-smoke.sh`
  - verifies the intended app is frontmost before input, preventing recipes from
    being sent to the wrong app;
  - scopes activation/window targeting to the lowercase app process names
    reported by System Events (`ghostty`, `roastty`);
  - dismisses the app-local macOS "Don't Reopen" crash dialog when earlier
    automation failures left one behind;
  - focuses the target terminal view by clicking inside the launched app window;
  - rewrites recipes so ANSI payloads are emitted as data arguments rather than
    `printf` format strings, so literal `%`, backslashes, and escape sequences
    are no longer shell format hazards.
- `scripts/ghostty-app/inject.swift`
  - adds keyboard helpers:
    - `key <virtual-key-code> [control] [command] [shift] [option]`;
    - `type <utf8-text-file>`.
- `scripts/roastty-app/README.md`
  - documents the guarded CGEvent command-entry path.
- `issues/0802-libroastty-completion-and-mac-app/README.md`
  - records the durable finding that live A/B command delivery itself remains
    unsolved.

Verification:

- `bash -n scripts/roastty-app/live-ab-smoke.sh`
- `bash -n scripts/roastty-app/live-ab-matrix.sh`
- `scripts/roastty-app/live-ab-smoke.sh --list-recipes`
  - Printed `smoke`, `ascii-grid`, `color-grid`, `clear-after`, `alt-screen`,
    and `scroll-output`.
- `swift scripts/ghostty-app/inject.swift key 8 control`
  - Compiled and ran the new key path.
- `scripts/roastty-app/live-ab-smoke.sh --recipe smoke --max-mismatch-ratio 1 --max-mean-channel-delta 255`
  - Exited `0`, emitted JSON, captured 1000x1000 Ghostty/Roastty images, and
    killed the launched PID trees.
  - The permissive screenshot diff passed, but visual inspection showed the
    recipe output was still missing, so this is not evidence of successful
    command delivery.
- Focused probes:
  - Command-V paste into Roastty terminated the app.
  - AppleScript text entry left Roastty alive but did not produce visible
    command output.
  - CGEvent text entry left Roastty alive but did not produce visible command
    output.
- `scripts/roastty-app/stop-app.sh || true`
- `scripts/ghostty-app/stop-app.sh || true`
- `pgrep -fl '[G]hostty.app/Contents/MacOS/ghostty|[R]oastty.app/Contents/MacOS/roastty' || true`
  - no output after cleanup.

## Completion Review

**Reviewer:** Codex-native adversarial subagent (`multi_agent_v1.spawn_agent`,
fresh context, read-only). **Verdict: APPROVED.**

The reviewer found no Required issues. It noted one Optional concern: activation
still starts from a process-name lookup, so multiple same-name debug instances
could confuse targeting. That is a real hardening follow-up, but it is not
blocking for this partial result because runs begin and end with scoped
Ghostty/Roastty cleanup. The reviewer also found one nit: `inject.swift`'s
top-level usage omitted the new `key` and `type` subcommands; fixed before the
result commit.

## Conclusion

Experiment 46 did not solve live A/B command delivery, so the full recipe matrix
verification was not meaningful to run. It did make the harness safer: it now
guards against wrong-frontmost-app input, avoids relaunching apps during
activation, handles the crash-reopen dialog, and removes the `printf` format
hazards from recipe payloads. The next experiment should make command delivery
observable and reliable, likely by fixing Roastty's paste/key input path or by
switching the A/B recipes to a launch-time shell bootstrap instead of synthetic
UI typing.
