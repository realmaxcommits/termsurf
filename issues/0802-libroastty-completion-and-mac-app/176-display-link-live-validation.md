# Experiment 176: Phase C — display-link live validation

## Description

Run the live copied-app verification that Experiment 175 intentionally left
open.

Experiment 175 replaced the ad hoc sleep-thread present driver with an owned
driver abstraction, added a macOS `CVDisplayLink` scheduler for
`window-vsync = true`, preserved the fallback scheduler, and routed
`roastty_surface_set_display_id` into active display-link drivers. The Rust
tests prove the state machine and fallback behavior, but they do not prove the
real copied app uses the CoreVideo path successfully on the desktop.

This experiment should rebuild the current `RoasttyKit`, rebuild and launch the
copied `Roastty.app`, drive live terminal output, capture evidence outside the
repo, and verify the app exits cleanly. If the live display-link path is proven,
update the Phase C render-thread/frame-pacing roadmap item. If the app fails to
launch, render, update, or clean up, record the precise blocker and keep the
roadmap unchecked.

This is a validation experiment with one planned observability hook: add an
environment-gated present-driver diagnostic so the live run can prove whether
the copied app selected the real CoreVideo display-link scheduler or fell back
to the timer scheduler. Do not change rendering behavior unless the live run
exposes a small, directly related display-link bug. If code changes beyond the
diagnostic are needed, keep them scoped to the Experiment 175 present-driver
path and rerun the same live verification.

## Changes

- `issues/0802-libroastty-completion-and-mac-app/README.md`
  - Link this experiment as `Designed`.
  - After the run, mark this experiment `Pass`, `Partial`, or `Fail`.
  - If live display-link rendering, display-ID update routing, and cleanup are
    proven, update the Phase C render-thread/frame-pacing roadmap item to
    checked with an Experiment 176 note.

- `issues/0802-libroastty-completion-and-mac-app/176-display-link-live-validation.md`
  - Record the exact commands run, screenshot/log paths, result, conclusion, and
    AI completion review.

- `roastty/src/lib.rs`
  - Add an environment-gated diagnostic around present-driver selection and
    display-ID updates, enabled only when `ROASTTY_PRESENT_DRIVER_LOG=1`.
  - Log whether `PresentDriver::start` selected the CoreVideo display-link path
    or fallback path.
  - Log display-ID updates routed to an active display-link driver.
  - Do not change scheduling, rendering, fallback, or dirty-pump behavior unless
    the live run exposes a directly related bug.

## Verification

Before live verification:

- Codex-native adversarial design review approves this experiment.
- Commit the reviewed plan separately from the result.

Build:

- `cd roastty && macos/build.nu --action build`

Live run:

- Stop any stale debug Roastty process: `scripts/roastty-app/stop-app.sh`.
- Run the existing live A/B harness with the smoke recipe and driver logging:

  ```bash
  TERMSURF_AB_HOLD_SECONDS=5 \
  ROASTTY_PRESENT_DRIVER_LOG=1 \
    scripts/roastty-app/live-ab-smoke.sh \
      --recipe smoke \
      --comparison-region content \
      --max-mismatch-ratio 1 \
      --max-mean-channel-delta 255
  ```

  This launches the debug copied `Roastty.app` binary directly, bootstraps a zsh
  config that prints a unique `ISSUE802_AB_SMOKE_...` marker, captures
  Ghostty/Roastty screenshots outside the repo, and writes Roastty stdout/stderr
  logs under `${TERMSURF_SHOT_DIR:-$HOME/.cache/termsurf/shots}`.

- Identify the run's Roastty stderr log from the harness output or shot
  directory, then assert it contains the display-link selection diagnostic, for
  example:

  ```bash
  grep -F 'present-driver=display-link' "$ROASTTY_STDERR_LOG"
  ```

- If a safe display-change trigger is available in the current desktop session,
  run it and assert the stderr log also contains the display-ID update
  diagnostic. If not, explicitly record that display-ID routing remains covered
  by the Experiment 175 unit test
  `present_driver_display_id_update_reaches_active_display_link` and was not
  contradicted by the live run.
- Prove no debug Roastty app PID remains after the harness cleanup:

  ```bash
  if pgrep -f 'roastty/macos/build/.*Roastty.app/Contents/MacOS/roastty'; then
    exit 1
  fi
  ```

Regression checks after any code change:

- `cargo test -p roastty present_driver -- --test-threads=1`
- `cargo test -p roastty --test abi_harness`
- `cargo test -p roastty -- --test-threads=1`
- `cargo fmt --check -p roastty`
- `prettier --check --prose-wrap always --print-width 80 issues/0802-libroastty-completion-and-mac-app/176-display-link-live-validation.md issues/0802-libroastty-completion-and-mac-app/README.md`
- `git diff --check`

Run `cargo fmt -p roastty` after any Rust edit before the verification checks.
If there are no code changes beyond documentation, the focused/full Rust
regression checks from Experiment 175 may be referenced as already current, but
the macOS app build and live run remain mandatory for this experiment.

**Pass** = the copied `Roastty.app` rebuilds against the current `libroastty`,
launches, renders the live smoke marker, logs that it selected the vsync-enabled
CoreVideo display-link driver, captures live evidence outside the repo, and
stops with zero dangling debug Roastty app PIDs. Display-ID update routing is
either live-triggered and logged, or explicitly covered by the Experiment 175
unit proof and not contradicted by the live run.

**Partial** = the app builds and some live evidence is captured, but a desktop
automation limitation prevents proving live terminal output, real display-link
selection, display-ID update routing, or cleanup. Record the limitation and
exact evidence.

**Fail** = the rebuilt copied app does not launch, does not render, does not
update after terminal output, crashes in the display-link path, or leaves
dangling debug app processes after cleanup.

## Design Review

**Reviewer:** Codex-native adversarial review subagent `Faraday`, fresh context.

**Initial verdict:** Changes required.

Findings and fixes:

- Required: the live terminal-output step was too vague. Fixed by using the
  concrete `scripts/roastty-app/live-ab-smoke.sh --recipe smoke` harness
  invocation.
- Required: the Pass criteria overclaimed real display-link selection without an
  observable proof. Fixed by adding a planned `ROASTTY_PRESENT_DRIVER_LOG=1`
  diagnostic and a required `present-driver=display-link` stderr assertion.
- Optional: the cleanup `pgrep` proof was implicit. Fixed by spelling out the
  inverted `pgrep` check.
- Nit: mutating `cargo fmt -p roastty` was listed as a verification check. Fixed
  by moving it to a pre-check instruction and keeping
  `cargo fmt --check -p roastty` in the verification list.

**Final verdict:** Approved.

Final findings: None.
