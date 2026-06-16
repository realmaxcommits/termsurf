# Experiment 2: Remove key-path termio mutex waits

## Description

Experiment 1 reproduced the `ReleaseLocal` input delay and localized the root
cause: synchronous AppKit key handling enters `Surface::write_encoded_key_event`
and blocks on the shared `TermioWorker` mutex while the worker owns that mutex
inside `pump_once`. Rapid keyboard input turns many per-key mutex waits into a
34-37 second visible delay.

This experiment will remove `Termio` mutex access from the synchronous keyboard
path. Key handling should encode and enqueue input without waiting for the
worker's PTY poll loop.

## Changes

- Add cached terminal input state to `Surface`:
  - cached `key_encode::Options`;
  - cached terminal KAM mode (`mode_get(2, true)`), used only when
    `vt_kam_allowed` is enabled.
- Initialize the cache to upstream/default terminal values when the surface is
  created.
- Add an input-state snapshot to the worker pump event, populated while the
  worker already owns `Termio` during `pump_once`. The snapshot must include the
  terminal's `key_encode::Options` and KAM mode. `Surface` must refresh its
  cache only from this snapshot, without calling `TermioWorker::with_termio`
  from the main thread during key handling, `tick_termio`, or
  `apply_termio_event`.
- Emit/update the snapshot when relevant terminal state changes. It is
  acceptable for the first implementation to include the snapshot on every pump
  event if that keeps the update path simple and still avoids main-thread locks.
- Change `Surface::key_encode_options()` and `Surface::terminal_kam_enabled()`
  so they read the cached values instead of calling `TermioWorker::with_termio`
  from key handling.
- Add or update focused Rust tests that prove:
  - `write_encoded_key_event` no longer needs to lock `Termio`;
  - cached KAM still blocks input when `vt_kam_allowed=true` and terminal KAM is
    enabled by real terminal output/escape-sequence processing;
  - cached key-encoding options update after real terminal output/escape
    sequences change a representative key-encoding option, such as cursor-key
    application mode, modify-other-keys, or Kitty keyboard flags.
- Rerun the Issue 806 Experiment 1 latency harness against `ReleaseLocal`.

## Verification

Pass criteria:

- The sampled main thread no longer shows the dominant
  `keyDown -> write_encoded_key_event -> TermioWorker::with_termio -> Mutex::lock`
  stack during the latency harness.
- The sampled/trace evidence also shows the fix did not move the same dominant
  main-thread `TermioWorker::with_termio`/mutex wait into `tick_termio`,
  `apply_termio_event`, render, or presentation.
- `scripts/roastty-app/issue806-exp1-latency.sh` observes both marker file and
  visible terminal output within a practical interactive budget. For this VM,
  the target is under `2000ms`; if the first fixed run is higher, the result
  must explain the remaining bottleneck with evidence and this experiment is not
  complete.
- The trace shows `keyAction result=true` durations are no longer dominated by
  hundreds-of-milliseconds mutex waits.
- KAM behavior and terminal key-encoding mode behavior remain covered by focused
  Rust tests.
- Hygiene checks run and are recorded:
  - `cargo fmt -- roastty/src/lib.rs roastty/src/termio.rs`;
  - `cargo fmt --check --manifest-path roastty/Cargo.toml`;
  - focused Rust tests for the cache/KAM/key-encoding behavior;
  - `cd roastty/macos && ./build.nu --configuration ReleaseLocal`;
  - `bash -n scripts/roastty-app/issue806-exp1-latency.sh`;
  - `git diff --check`.

Fail criteria:

- Key handling still locks `TermioWorker` for key-encoding or KAM checks.
- The latency harness still exceeds `2000ms` without a newly identified
  downstream root cause.
- KAM or key-encoding mode behavior regresses.
- The fix broadens into unrelated render, PTY, shell, configuration, or GUI
  refactoring.

## Design Review

Fresh-context adversarial review returned `CHANGES REQUIRED`.

- Required: cache refresh was underspecified and could have been implemented by
  taking the same `TermioWorker` lock from `tick_termio` or
  `apply_termio_event`. Fixed by requiring the worker to include input-state
  snapshots in pump events while it already owns `Termio`, and by forbidding
  main-thread `with_termio` refreshes.
- Required: tests did not explicitly require real escape-sequence updates for
  cached KAM/key-encoding state. Fixed by requiring tests that feed real
  terminal output/escape sequences through the stream and verify cached KAM plus
  at least one representative key-encoding option.
- Optional: latency criteria only checked the old `keyDown` stack. Fixed by
  requiring sample/trace evidence that equivalent main-thread mutex waits were
  not moved into event drain, render, or presentation.

Fresh-context adversarial re-review returned `APPROVED`.

- The reviewer confirmed the worker-populated snapshot requirement resolves the
  cache-refresh lock risk.
- The reviewer confirmed the test plan now requires real terminal
  output/escape-sequence updates.
- The reviewer confirmed the latency criteria now check for moved main-thread
  mutex contention.
- No new required findings were reported.
