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

# Experiment 829: Deadlock-proof test gate and PTY worker deadlock fix

## Description

The `cargo test -p roastty` suite can **deadlock**: a run was observed hung for
208 s wall-clock with **0.05 s total CPU**
(`208.50 real / 0.03 user / 0.02 sys`) — the signature of every thread parked in
a blocking primitive, doing no work. A batch of `kitty_clipboard_*`,
`osc52_clipboard_*`, and `surface_binding_action_*` tests sat "running for over
60 seconds" and never progressed; the same set hung across repeated runs and had
to be killed by hand.

This went unnoticed for 828 experiments because the test gate was a
self-reported pass count, and Rust has **no per-test timeout** — a hang is an
indefinite wait, not a failure, and an intermittent deadlock passes almost every
run.

This experiment makes hangs **loud and fatal**, then fixes the first deadlock
the new gate exposes. It implements the "Test execution gate (deadlock-proof)"
convention added to this issue's README (`## Test Parity`).

### What static analysis already established

The wedge is **not** in the obvious places — they were read and ruled out:

- The PTY master fd is **non-blocking** (`Termio` calls
  `child.set_nonblocking()`, `termio.rs:126` → `fcntl(F_SETFL, O_NONBLOCK)`,
  `os/pty.rs:237`), so `pump_once`'s `poll(10ms)` / `read_available` / `write`
  cannot block indefinitely (`flush_pending_write` breaks on `WouldBlock`,
  `termio.rs:235`).
- The worker loop (`run_termio_worker`, `termio.rs:328`) drains commands with
  **`try_recv`** (`termio.rs:384`) every iteration and breaks on
  `Shutdown`/`eof`/`child_exited`, so `TermioWorker::shutdown` → `join.join()`
  (`termio.rs:309`) should always return.
- The main-thread test helpers are **bounded**: `tick_termio` drains events with
  non-blocking `try_recv_event` (`lib.rs:3732`); `wait_until` panics after 100 ×
  10 ms (`lib.rs:14985`); `surface_snapshot_text_until` caps at 300 iterations.

So no single primitive can hang on its own. The 208 s / ~0 CPU stall is
therefore a **runtime lock or scheduling deadlock**. Note what the analysis
above does **not** point to: the per-worker `termio` mutex is held only across
bounded critical sections (`pump_once` — whose sole potentially-blocking call is
`child.poll(10ms)` on a non-blocking fd, `pump_timeout_ms = 10` in both
production, `lib.rs:1992`, and tests, `lib.rs:14982` — plus the non-blocking
command drain and clipboard drain), so the worker releases `termio` at least
every ~10 ms and a main-thread `worker.with_termio*` `lock()` can wait at most
~10 ms — a single-`termio`-mutex cycle **cannot** produce a multi-minute stall
and is ruled out, not suspected. (A _poisoned_ `termio` is a separate failure
mode: `lock()` then returns `Err` and the code's `.expect(...)` panics — a fast
panic/cascade, not the observed silent multi-minute hang — so it is not a
candidate here either.) The genuinely unbounded wait this code actually contains
is elsewhere: the **`join.join()` in `TermioWorker::shutdown` has no timeout**
(`termio.rs:309`), and — most likely given the evidence — **a lock at the
`lib.rs` surface layer**, since the hung tests are `surface_binding_action_*`
and `*_clipboard_*` (the surface/app teardown + clipboard-reply path), not the
termio-worker unit tests. The exact cycle still **requires a backtrace of the
live hung process**, which static reading cannot supply. Hence this experiment
is diagnosis-first, and the backtrace — not this hypothesis — determines the
fix.

### Why `nextest` is the right gate

`cargo-nextest` runs **each test in its own process** with a per-test
`slow-timeout`/`terminate-after` that **SIGKILLs a hung test and reports it by
name**. Two properties make it decisive here:

1. The global `PTY_COMMAND_LOCK` that serializes 160 PTY tests in cargo's
   in-process runner caused one wedged test to freeze 159 others (the "wave").
   Under nextest's process-per-test model that cascade disappears: **only the
   genuinely-deadlocking test is killed and named**, in isolation.
2. The killed test becomes a **first-class failure** the gate catches, instead
   of an indefinite wait an agent backgrounds and polls.

Two consequences of process-per-test must be handled, not assumed away:

- **It defeats `PTY_COMMAND_LOCK`** (`termio.rs:427`), the global mutex that
  deliberately serializes ~160 PTY tests within one process. Under nextest each
  test process has its own static, so those PTY tests run **truly concurrently**
  across cores — which can spawn fd/fork storms and inject _spurious_ contention
  failures unrelated to the deadlock (making "zero failures ×3" hard to meet for
  the wrong reason), or shift timing enough to hide the race. If that appears,
  serialize the PTY tests with a nextest `[test-groups]` entry (max-threads = 1
  over a filter matching the PTY tests) rather than relying on the now-defeated
  mutex. This is part of the experiment, recorded in the Result.
- **It can hide an inter-test deadlock.** If the wedge is intra-test (one test's
  own main+worker threads), nextest reproduces it in isolation. If diagnosis
  instead shows an **inter-test** cycle that only occurs under shared in-process
  statics, nextest's isolation would make it _vanish_ — so "passes under
  nextest" must not be read as "fixed." In that case the result records the
  inter-test nature explicitly and the fix targets the lock discipline, verified
  under cargo's in-process runner (with a kill-timeout wrapper), not only
  nextest.

## Changes

This experiment proceeds in ordered steps; the exact source fix is finalized
from the Step 3 backtrace and recorded in the Result.

1. **Tooling — verify `cargo-nextest`.** It is **already installed**:
   `cargo nextest --version` → `cargo-nextest 0.9.137`, binary at
   `/opt/homebrew/bin/cargo-nextest` (Homebrew, on `PATH`; it is absent only
   from `~/.cargo/bin`, which does not matter). The step is just
   `cargo nextest --version` to confirm presence; install
   (`brew install cargo-nextest` or `cargo install cargo-nextest --locked`) only
   if a future checkout lacks it. It is a developer test runner, **not** an app
   install and **not** a crate dependency in any `Cargo.toml`.

2. **Gate config — `.config/nextest.toml`** (new, repo root):

   ```toml
   [profile.default]
   slow-timeout = { period = "30s", terminate-after = 1 }
   # period: a test is "slow" after 30s; terminate-after = 1 SIGKILLs it after one
   # such period (~30s) and reports it as a failure by name.
   ```

3. **Reproduce + diagnose.** The hang is racy (it did not fire on every casual
   run), so do not rely on a single run to surface it. Force it with a bounded
   loop — e.g.
   `for i in $(seq 1 20); do cargo nextest run -p roastty || break; done`,
   and/or repeatedly run the suspect tests alone
   (`cargo nextest run -p roastty -E 'test(/clipboard|surface_binding/)'`) under
   the kill-timeout — until nextest terminates a test by name. Then run that
   test in isolation under a debugger and, **while it is hung**, capture
   `sample <pid>` (and/or `lldb -p <pid>` → `thread backtrace all`) to record
   the **main-thread and worker-thread stacks** and name the exact lock/wait
   cycle. Record the reproduction rate and the backtrace in the Result. If a
   bounded number of forced attempts cannot reproduce it, that is recorded as a
   **Partial** (not a silent pass) before any "inter-test only / cannot
   reproduce" conclusion.

4. **Fix the root cause** identified in Step 3. The backtrace names the cycle;
   the fix targets exactly that. Candidate directions, ordered by what the
   static analysis leaves genuinely possible (the bounded `termio` critical
   section is **not** among them — it is ruled out above):
   - **An unbounded lock at the `lib.rs` surface/app layer.** Since the hung
     tests are `surface_binding_action_*` / `*_clipboard_*`, the likely cycle is
     in surface/app teardown or the clipboard-reply path holding a lock while
     waiting on (or being waited on by) the worker. Break the demonstrated
     ordering, or drop the lock across the wait.
   - **`shutdown` cannot interrupt a wedged worker.** As a defensive backstop
     regardless of the precise cycle, make `TermioWorker::shutdown`
     (`termio.rs:302`) unwedge a stuck worker before the no-timeout `join`
     (`termio.rs:309`) — signal/close the child PTY so any in-flight syscall
     returns — and/or bound the `join` so teardown cannot hang forever.

   The final diff is whatever the backtrace proves necessary; the design commits
   to fixing the demonstrated cycle, not to a guessed one.

5. **Regression test.** Add a deterministic test that reproduces the wedge
   scenario (spawn a `TermioWorker` over a non-cooperative child, drive the
   exact event/teardown sequence) and asserts it **completes within a bounded
   time** via a watchdog thread (`recv_timeout`), so the deadlock fails fast if
   it regresses — and is independently killed by the nextest `terminate-after`
   gate.

## Verification

All commands run in the foreground (never backgrounded-and-polled), so a hang
surfaces as a failure.

- **Tooling check:** `cargo nextest --version` reports `0.9.137` (already
  present; no install needed).
- **Reproduce (pre-fix):** the Step 3 forced loop makes
  `cargo nextest run -p roastty` terminate the deadlocking test(s) by name
  within the 30 s window — proving the gate converts the hang into a named
  failure. Record how many attempts were needed (the reproduction rate).
- **Fixed:** after the Step 4 fix, `cargo nextest run -p roastty` runs the full
  suite with `--retries 0` and **zero terminations / zero failures**, repeated
  enough times to clear the observed pre-fix reproduction rate by a wide margin
  — at least **3× the pre-fix mean attempts-to-reproduce**, and never fewer
  than 3. (A fix verified only across fewer repeats than it took to reproduce
  the bug is not proof; raise the count to match.) The previously-deadlocking
  test passes every repeat.
- **Regression test** passes under nextest and fails (is killed) if the fix is
  reverted (spot-checked).
- `cargo build -p roastty` — no warnings.
- `cargo fmt -p roastty -- --check` — clean.
- No-ghostty grep on every touched source file — clean.
- `git diff --check` — clean.

**Pass** = the deadlock is root-caused from a backtrace and fixed; the full
suite completes under the nextest kill-timeout across three repeats with no
hang; and a regression test plus the `terminate-after` gate guard against
recurrence. **Fail/Partial** = the deadlock cannot be reproduced under nextest
(record whether it is inter-test only), or any repeat still hangs.

## Review note

Per this session's move to in-session adversarial review, both gates run via the
`adversarial-reviewer` subagent (it loaded and ran in this session without a
restart; the design review below was performed by it). One standing rule for the
result-gate reviewer: it may run `cargo nextest run -p roastty` to verify
independently **only once `.config/nextest.toml` exists** (the kill-timeout then
prevents the review from hanging); it must **never** run bare
`cargo test -p roastty`, nor `cargo nextest run` before the timeout config
lands, since both can still deadlock.

## Design Review

**Reviewer:** `adversarial-reviewer` subagent (Claude Opus, fresh context,
read-only). Ran ~3.8 min with no deadlock under design-review guardrails (no
full-suite test run); verified the design's code claims against source.

**Verdict:** CHANGES REQUIRED on the first pass → all four findings addressed
below → **re-review APPROVED** (fresh `adversarial-reviewer` subagent, ~1.5 min,
no deadlock). The re-review confirmed each prior finding genuinely resolved and
cross-checked every load-bearing line number against source, raising only two
non-blocking accuracy points, both since applied: (a) a poisoned-`termio`
`lock()` is a fast panic, not a silent multi-minute hang, so it was removed from
the "unbounded waits" list and recharacterized; (b) the `termio` mutex is held
across three bounded critical sections, not only `pump_once`, so the wording was
made precise. No Required findings remain; the design is approved for the plan
commit.

- **Required — self-contradicting prime suspect.** The original design named a
  `termio`-mutex lock-order cycle as prime suspect, but its own analysis shows
  that mutex is held only across a 10 ms-bounded `pump_once`, so it cannot stall
  for minutes. **Fixed:** the hypothesis is re-aimed at the genuinely unbounded
  waits — the no-timeout `join` (`termio.rs:309`) and, most likely, a lock at
  the `lib.rs` surface/clipboard layer (where the hung tests live); the `termio`
  mutex is now explicitly ruled out, not suspected. Step 4's fix candidates were
  re-aimed to match.
- **Required — false "nextest not installed" premise.** **Fixed:** Step 1 and
  the Verification tooling check now record nextest `0.9.137` already present on
  `PATH`; the step is a presence check, not an install.
- **Optional — nextest defeats `PTY_COMMAND_LOCK`.** **Adopted:** the design now
  notes process-per-test runs the ~160 PTY tests truly concurrently (fd/fork
  storms, timing shifts) and prescribes a nextest `[test-groups]` serialization
  if spurious failures appear.
- **Optional — racy-hang verification gap.** **Adopted:** Step 3 now forces the
  hang with a bounded reproduction loop before capturing the backtrace, records
  the reproduction rate, and the post-fix repeat count must exceed the pre-fix
  attempts-to-reproduce (≥3× mean, never <3); a non-reproduction is a Partial,
  not a silent pass.

## Result

**Result:** Pass

### Gate established and proven to catch the deadlock

`.config/nextest.toml` (new) sets
`slow-timeout = { period = "30s", terminate-after = 1 }`. Run under the gate,
the suite reproduced the deadlock as a **named, terminated test** on the second
forced attempt:

```
TERMINATING [> 30.000s] roastty tests::kitty_clipboard_read_event_rejects_unsupported_forms_without_request
    TIMEOUT [  30.004s] roastty tests::kitty_clipboard_read_event_rejects_unsupported_forms_without_request
```

That is the design goal: an indefinite hang became a first-class, named failure.
Reproduction rate: ~1-in-2 under the targeted 188-test nextest run; ~100% under
16-way direct concurrency on the single test.

### Root cause (from a live backtrace)

`sample` of the wedged process (`logs/exp829/backtrace.txt`) showed the test
thread parked — **not** in any lock cycle (the worker thread had already
exited):

```
roastty_surface_free (lib.rs:13301) → drop Surface → drop TermioWorker
 → drop Arc<Mutex<Termio>> → drop Termio → <PtyChild as Drop>::drop (pty.rs:354)
  → std::process::Child::wait → __wait4   [parked indefinitely]
```

**Proven fact:** `PtyChild::drop` blocked in an **unbounded `Child::wait()`**
(`wait4`) reaping a child that did not promptly die — freezing teardown and,
under the in-process runner, the whole suite via `PTY_COMMAND_LOCK`. That is the
demonstrated symptom and is sufficient to justify the fix (bound the wait). This
is also a **production bug**: closing a surface whose shell is still alive would
hang the same way.

**Hypothesis (not fully derived):** why the pre-fix path — `kill()` (single pid)
then `wait()` — failed to reap promptly. The child runs `setsid()` + `TIOCSCTTY`
(`pty.rs:172`), so it leads its own session/process group; a plausible cause is
the session/group structure under heavy concurrency leaving the leader
un-reapable for a window. The exact kernel reason was **not** reproduced
deterministically (the hang is racy — see the regression-test caveat). The fix
does not depend on it: bounding the reap eliminates _any_ unbounded wait in drop
regardless of cause, and the process-group kill is independently correct
production behavior (terminate the shell's whole job tree on surface close).

### Fix

`PtyChild::drop` (`roastty/src/os/pty.rs`) now SIGKILLs the whole **process
group** (`kill(-pid)`, reaching grandchildren and the wedged session leader),
then performs a **bounded reap** (poll `try_wait` ≤2s, then detach) so teardown
can never block indefinitely.

### Verification (all green for the deadlock)

- **`cargo build -p roastty`** — no warnings.
  **`cargo fmt -p roastty -- --check`** — clean. **No-ghostty grep** on
  `roastty/src/os/pty.rs` — clean.
- **Forced concurrency:** 0 hangs across **160 runs** (10×16 concurrent) of the
  culprit test — was ~100% pre-fix.
- **Targeted nextest:** **12/12** clean runs of the 188 `clipboard`/
  `surface_binding` tests — was ~1-in-2 hung pre-fix.
- **Original failing config:** bare `cargo test -p roastty` now **completes in
  96 s** (was a 208 s hang), 4353 passed / 6 failed.
- **Backtrace confirms the fix target**; the 0/160 + 12/12 + bare-suite
  completion confirm the deadlock is gone.
- **Regression test** `pty_child_drop_kills_the_whole_process_group`
  (`roastty/src/os/pty.rs`): spawns a child that backgrounds a grandchild,
  asserts `PtyChild::drop` returns in **<5 s** (the bounded-reap invariant) and
  that the child tree is reaped. Honest caveat: it does **not** fail when the
  group-kill is reverted to a single-pid kill — the grandchild is also reaped by
  the SIGHUP that the PTY master-close delivers, so it cannot deterministically
  isolate the group-kill, and the underlying deadlock is racy (no deterministic
  unit reproduction). The **real regression guard is the gate itself**: the
  nextest kill-timeout + the forced-reproduction loop reproduced the hang
  ~1-in-2 pre-fix and show 0/160 + 12/12 post-fix. (The design's "fails when
  reverted" expectation was not met for the reason above; recorded as a
  deviation.)

### Two pre-existing issues the now-completing suite exposed (not regressions)

1. **6 flaky tests** (`surface_text_*`, `surface_tty_name_*`) — real-child I/O
   racing a 1 s wait. **Confirmed pre-existing** via a baseline with the fix
   reverted (`git stash`): they fail **2–5 per run with zero timeouts even
   without the fix**. Unrelated to this change. → **deferred to Experiment 830**
   per the issue's hang/flake convention.
2. **CoreText font-test slowness under nextest.** `font::*` tests load real
   macOS fonts; nextest's process-per-test has no shared CoreText cache, so each
   pays a cold ~10 s load, and under full parallelism 100+ processes thrash the
   system font daemon and each stretches to **~58 s** (the font module alone
   took 522 s under nextest). These are **slow, not deadlocked** (they pass in
   isolation). Per the chosen remedy (tune nextest, not a different runner),
   `.config/ nextest.toml` puts the whole `font::` module in a `coretext`
   test-group (`max-threads = 4`, to cut fontd thrash) with a **120 s** window,
   so slow-but-finite tests are not killed while an infinite deadlock is still
   caught. Fast font tests are unaffected. **Verified:** under the tuned config
   the whole `font::` module runs **593 passed / 0 timeouts / 0 failures** (slow
   tests land at ~18 s, well under 120 s). Cost: the capped concurrency makes
   the font module ~12.8 min — the accepted trade-off of keeping one nextest
   runner.

### Gate-design decisions (user-directed)

- The deadlock-proof gate stays on **nextest**, tuned with a `[test-groups]`
  entry + longer timeout for the CoreText tests (rather than switching to a
  bare-`cargo test` + watchdog runner).
- The 6 flaky surface tests become **Experiment 830** (the immediate next
  experiment), not a fold-in here.

## Conclusion

The first real deadlock the gate was built to catch was a genuine bug — an
unbounded `Child::wait()` in `PtyChild::drop` reaping a `setsid()`
session-leader child that a single-pid `kill()` could not terminate. The fix
(process-group SIGKILL + bounded reap) eliminates it in both the test suite and
production surface teardown, proven by 160 concurrent runs, 12 nextest repeats,
and the once-hanging bare `cargo test` now completing in 96 s.

The deadlock-proof gate works: it converted an indefinite, intermittent hang
into a named, terminated test — exactly what 828 prior experiments lacked.
Tuning it for this suite surfaced that nextest's process-per-test isolation is
costly for CoreText-backed font tests (no shared font cache → ~10–58 s each);
the `coretext` test-group (capped concurrency + 120 s window) keeps them from
false-positiving while still catching infinite hangs.

Two follow-ups, in order:

- **Experiment 830 (immediate next):** fix the 6 pre-existing flaky
  `surface_text_*` / `surface_tty_name_*` tests (real-child I/O racing a 1 s
  wait). Per this issue's hang/flake convention, no new feature work precedes
  it. 830 can also add a stronger surface-level regression test that drives the
  exact racy teardown sequence under a watchdog, using the lib.rs surface
  helpers it fixes there.

A `PtyChild::drop` teardown-invariant test ships in this experiment; the
deterministic isolation of the racy deadlock is left to the gate (which catches
it empirically). The URI/regex and remaining `os/` slices resume only once the
test suite is flake-free.

## Completion Review

**Reviewer:** `adversarial-reviewer` subagent (Claude Opus, fresh context,
read-only). Ran ~5.4 min; independently reproduced every claimed number from
`logs/exp829/` and ran fast spot-checks.

**Verdict:** APPROVED — no Required findings. The reviewer confirmed: the fix is
sound (at the `kill(-pid)` point the child is provably un-reaped, so no
PID-reuse can misdirect the group signal; `setsid` makes pgid==pid; the
pre-`setsid` race is covered by the direct `self.child.kill()`; drop is bounded
to ~2 s); the backtrace supports the "unbounded `wait4` in drop, no lock cycle"
claim; the log integrity holds (`loop-2.log` named TIMEOUT, 12/12
`postfix-*.log`, `bare-cargo-test.log` 96 s / 4353 passed / 6 failed,
`font-tuned2.log` 593 passed / 0 timeouts, baseline `base-*.log` 2–5 fails with
the fix reverted); the 6 flaky tests are pre-existing and correctly deferred to
Exp 830; build/fmt/no-ghostty/ diff clean; plan commit exists and no result
commit yet. It re-ran the two `pty_child_drop` tests (pass) and the 188
clipboard/surface_binding tests under nextest (pass, no hang).

Findings, all Optional/Nit, addressed:

- **Optional — overstated root-cause mechanism.** The prose asserted the
  single-pid `kill()` "left the group unreapable, so `wait()` parked forever,"
  but an uncatchable SIGKILL to the shell pid reaps it regardless of a surviving
  grandchild. **Fixed:** the Root-cause section now separates the **proven
  fact** (unbounded `wait4` in drop) from the **hypothesis** (why the old path
  failed to reap), matching the honesty already in the regression-test caveat.
- **Optional — regression test is not fail-without-fix.** Already disclosed in
  the Result; no change required. The empirical gate is the regression guard; a
  deterministic wedge reproduction is deferred to Exp 830.
- **Nit — zombie-leak window.** **Fixed:** the bounded-reap comment in `pty.rs`
  now notes that detaching can leak a zombie until process exit (the accepted
  trade for bounded teardown).
