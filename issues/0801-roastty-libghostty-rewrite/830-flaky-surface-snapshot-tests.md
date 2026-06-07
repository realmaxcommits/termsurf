+++
[implementer]
agent = "claude-code"
model = "claude-opus-4-8"
reasoning = "high"

[review.design]
agent = "claude-code"
model = "claude-opus-4-8"
reasoning = "high"

[review.result]
agent = "claude-code"
model = "claude-opus-4-8"
reasoning = "high"
+++

# Experiment 830: Fix the flaky surface_text PTY-snapshot tests

## Description

The deadlock-proof gate from Experiment 829 let the suite run to completion
under the in-process runner, which exposed **6 pre-existing flaky test
failures** (not regressions — confirmed by a fix-reverted baseline that
reproduced 2–5 of them with zero timeouts). Per this issue's hang/flake
convention, fixing them is the immediate next experiment, before any feature
work resumes.

### Root cause (diagnosed from the Exp 829 logs)

The 6 failures are **one flaky pattern plus a poison cascade**, not 6
independent bugs:

- **The real flake** is in the four `surface_text_*` tests (`roastty/src/lib.rs`
  ~28461–28545). Each spawns a real child, writes input with
  `roastty_surface_text`, then snapshots the screen with `surface_snapshot_text`
  and asserts the child's echoed output appears. `surface_snapshot_text`
  (`lib.rs:15310`) returns on the **first** `roastty_surface_needs_render` —
  which often fires on the initial render **before** the child's round-trip
  output arrives. Under load the snapshot is empty (e.g.
  `surface_text_bracketed_mode_wraps_paste_markers` panicked at `lib.rs:28541`
  with an all-newline snapshot `"\n\n\n…"`), so the
  `assert!(text.contains(...))` fails.
- **The cascade:** the failing test panics **while holding the global
  `PTY_COMMAND_LOCK`**, which **poisons** the mutex. Every subsequent PTY test
  then panics on `PTY_COMMAND_LOCK.lock().unwrap()` with `PoisonError` — that is
  exactly what the other five failures show (panics at column 46, the
  `.unwrap()`, including the two `surface_tty_name_*` tests). Under nextest's
  process-per-test model the lock is not shared, so there is no cascade and
  these tests pass in isolation — which is why they only fail under the
  in-process full suite.

So the two `surface_tty_name_*` failures and the three secondary
`surface_text_*` failures are **collateral**: fixing the snapshot race removes
the panic, the poison, and the cascade together.

## Changes

`roastty/src/lib.rs` — change the affected tests to wait for the **actual
expected output** instead of snapshotting on the first render. Replace
`surface_snapshot_text(app, surface)` with
`surface_snapshot_text_until(app, surface, <needle>)`.
`surface_snapshot_text_until` (`lib.rs:15325`) is a **pre-existing, already
widely-used** helper (13 current call sites) that ticks, re-renders only on a
fresh `needs_render`, returns on `contains(needle)`, and otherwise panics after
300 × 10 ms = 3 s with the latest snapshot — so the assertion runs against
output that has actually arrived. This is a minimal, proven, test-only change:

- `surface_text_unbracketed_reaches_child_pty` → needle `"out:hello"`.
- `surface_text_unbracketed_maps_newline_to_carriage_return` → needle
  `"line:hello"`.
- `surface_text_replaces_unsafe_control_bytes_with_spaces` → needle
  `"line:a b"`.
- `surface_text_bracketed_mode_wraps_paste_markers` → needle `"^[[201~"` (the
  closing paste marker, which arrives last, so both the `^[[200~hello` and
  `^[[201~` asserts that follow are satisfied). This test already waits for
  `bracketed_paste_enabled` before writing; only its output snapshot is racy.
- `surface_key_printable_utf8_reaches_child_pty` (`lib.rs:15890`) → needle
  `"a"`. Found in **design review**: same first-render round-trip race
  (`start → roastty_surface_key('a') → surface_snapshot_text → assert contains('a')`
  against a `dd bs=1 count=1` echo child) — it merely did not lose the race this
  run. Fixed here so the same latent flake cannot poison a later run.

Each needle is the complete child round-trip token, which cannot appear in a
blank/startup render, so there is no early-partial-render false positive. The
existing `assert!(text.contains(...))` lines stay (now guaranteed; `_until`
gives a clear panic with the latest snapshot if the needle never appears,
instead of a silent empty snapshot).

No production code changes; this is a test-robustness fix. The
`surface_snapshot_text` helper and its remaining call sites are left unchanged:
those either assert on content present at the first render or first gate on a
`surface_snapshot_text_after_start(...)` (whose render clears the dirty flag, so
the later `surface_snapshot_text` correctly waits for fresh output). This fixes
the five same-class round-trip-echo tests; any other caller that later proves
flaky under the gate gets the same one-line fix then.

### Considered and rejected

- **Poison-resilient `PTY_COMMAND_LOCK`**
  (`lock().unwrap_or_else(|e| e.into_inner())`): would hide the cascade but not
  the root flake, masks real panics, and touches ~160 call sites. Rejected — fix
  the race, not the symptom.
- **Readiness handshake for the unbracketed tests** (child prints a marker
  before `read`): unnecessary, because the PTY input buffer holds the written
  input until the child reads it; only the output snapshot was racy. If 3 s
  proves too tight under contention, the remedy is to raise the `_until` cap,
  not add handshakes.

## Verification

The flake only manifests under in-process full-suite CPU contention (the poison
cascade needs other PTY tests running). So:

- **Reproduce (pre-fix baseline):** already on record —
  `bare cargo test -p roastty` produced 6 failures (1 real assertion + 5
  `PoisonError`), and the fix-reverted baseline reproduced 2–5 per run.
- **Fixed:** `cargo test -p roastty` (bare, in-process — the configuration that
  fails) run **5×** with **0 failures** across all runs (clearing the observed
  pre-fix rate by a wide margin). Also run the four tests under a tight
  high-concurrency loop to stress the child round-trip.
- **Gate:**
  `cargo nextest run -p roastty -E 'test(/surface_text/) + test(/surface_tty_name/)'`
  clean (no terminations).
- `cargo build -p roastty` — no warnings. `cargo fmt -p roastty -- --check` —
  clean. No-ghostty grep on the touched source — clean. `git diff --check` —
  clean.

**Pass** = the snapshot race is removed in all five same-class tests,
`cargo test -p roastty` completes with zero failures across 5 repeats (no real
assertion failure, therefore no poison cascade), and the gate is clean. The 6
observed failures plus the one latent same-class caller are then fixed; any
other caller that surfaces under the gate later is a one-line follow-up. Feature
work (URI/regex, remaining `os/`) resumes once the suite runs clean.

## Design Review

**Reviewer:** `adversarial-reviewer` subagent (Claude Opus, fresh context,
read-only). Ran ~3.5 min; verified the root cause against
`logs/exp829/bare-cargo-test.log` and read both helpers + all affected tests.

**Verdict:** APPROVED, no Required findings. Confirmed: `surface_snapshot_text`
renders on the first `needs_render`; the lone real originator this run was
`surface_text_bracketed_mode_wraps_paste_markers` (`lib.rs:28541`, all-newline
snapshot); the other five panics are `PoisonError` at the `.unwrap()` (col 46),
including both `surface_tty_name_*` which do no snapshotting (pure cascade
collateral); `surface_snapshot_text_until` is the established 13-caller pattern;
each needle is a complete round-trip token (no early-render false positive); and
the 3 s cap already succeeds across 13 round-trip callers, with a clear panic
(not a hang) as the worst case.

Two findings, both adopted:

- **Optional — a 5th same-class test.**
  `surface_key_printable_utf8_reaches_child_pty` (`lib.rs:15890`) has the
  identical first-render round-trip race and only avoided losing it this run.
  **Adopted:** added to the fix (needle `"a"`), and the "flake-free" conclusion
  softened to "the observed + this latent caller are fixed; others get the same
  fix if they surface."
- **Nit — `surface_snapshot_text_until` is pre-existing.** **Adopted:** the
  Changes section now states it is an already-used 13-caller helper,
  strengthening the minimal/proven argument.
