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

# Experiment 833: Bound test parallelism to eliminate the contention flakes

## Description

After Exp 831/832, the suite's residual failures are **all CPU-contention
flakes**: they pass in isolation and fail only under the full suite's default
(~`ncpu`) parallelism. Exp 832's instrumentation showed the mechanism for
`surface_key_default_natural_text_editing_writes_legacy_bytes`: in isolation the
rendered screen is `"^E           05 …"` (the terminal's `^E` echo of the key
**and** od's `05` hex); under load the `^E` echo is **lost**, leaving only `05`,
so the assert fails. The child clearly runs — it is **starvation of the
render/echo path**, not the snapshot timing 832 already fixed.

A full-suite run at **`--test-threads=4`** is **clean**: `4360 passed; 0 failed`
in 171 s (whole wrapper 191 s, `STATUS=COMPLETED`, 0 panics, 0 `PoisonError`) —
versus 2–4 failures/run at default parallelism. Giving each real-child PTY test
enough CPU lets the echo render in time. So the fix is to **bound the roastty
suite's test parallelism**, the standard remedy for real-process integration
suites.

This is the last flake layer: with it, `cargo test -p roastty` should run green,
and feature work (URI/regex, remaining `os/`) can resume.

## Changes

This experiment also **resolves a gate incoherence** the design review surfaced:
the README currently mandates `cargo nextest run -p roastty` (for per-test
by-name kill-timeout), but every flake investigation (Exp 830–832) and the
threads=4 fix run used **bare `cargo test`**. Two reasons make bare `cargo test`
the right _routine_ gate now, with nextest kept as an on-demand pinpoint tool:

1. **The 15-min hard cap.** nextest runs each test in its own process with no
   shared CoreText cache, so the `font::` tests reload fonts every test and a
   full nextest run is **~12–15 min** (Exp 829/832) — dangerously close to
   `bounded-run.sh`'s 900 s ceiling; an occasional `HARD_TIMEOUT` would itself
   be a flaky gate. Bare `cargo test` shares the cache → font tests are fast → a
   full run is **~191 s**.
2. **Deadlock detection is preserved within the 15-min hard cap** — with two
   honestly-stated gaps versus nextest. `bounded-run.sh`'s 90 s no-progress kill
   `sample`s the wedged process and kills it, so a silent deadlock is still
   caught with a stack capture identifying the hung test. But (a) **catch
   latency grows**: under threads=4 the other 3 threads keep the log growing
   until the ~4357 other tests drain (~4 min), so the kill fires at ~drain+90 s
   ≈ 5 min, not nextest's per-test 30 s; and (b) an **output-emitting livelock**
   (a wedge that keeps printing) defeats the output-based idle kill entirely,
   whereas nextest's per-test wall-clock `slow-timeout` would still catch it —
   that is the one class the routine gate misses, so **use nextest on-demand for
   a suspected livelock**. Both stay inside the 900 s ceiling, so neither breaks
   the hard guarantee.

So:

- **Rewrite the README "Test execution gate"** so the canonical routine gate is
  **bare `cargo test -p roastty -- --test-threads=4`, run through
  `scripts/bounded-run.sh`** (15-min cap, Central stamps, watchdog+`sample` for
  hangs). nextest + `.config/nextest.toml` (kept) become the **on-demand**
  by-name deadlock-pinpointing tool, not the routine gate.
- `--test-threads=4` is the contention fix: ≈ half this machine's cores, enough
  CPU per real-child test so the echo/render path is not starved. It is
  **machine-relative** — on a host with few cores, 4 may _be_ full parallelism
  and the flake could return; the convention notes it as "≈ half the cores,
  recalibrate per machine," not a portable constant.
- **No source change.** (Considered: `.config/nextest.toml`
  `[profile.default] test-threads = 4` — keeps nextest but inherits its ~12–15
  min cost, rejected for routine use; and
  `.cargo/config.toml [env] RUST_TEST_THREADS=4` — rejected, it caps the whole
  workspace incl. the WezTerm fork.)
- **`bounded-run.sh` logs the invoked command** (so the thread count the gate
  hinges on is auditable from the artifact).

### Scope and alternative

- **`config_path_cli` disposition.** Exp 831 forecast 833 as the
  `config_path_cli` fix; that slot is repurposed to parallelism because the
  threads=4 run was clean _including_ config. But config failed only 1/5 at
  default, so a single clean run does not prove it is contention-driven. If it
  reappears in the 3× verification, it is a **non-contention env/path flake**
  that survives the cap and gets its own experiment (it manipulates `$HOME`/cwd,
  a different root cause than CPU starvation).
- **Alternative — per-test echo-render token-waits.** The `surface_key` failures
  are a lost-`^E`-echo race; in principle they could be made robust at _any_
  parallelism by waiting for the echo token (the Exp 830/832 template), instead
  of a suite-wide cap. Rejected as the primary fix here because the echo renders
  on a _different_ path than the child stdout already waited for (so it needs
  new per-test instrumentation for an unknown number of tests), whereas the
  thread cap fixes the whole class — including `surface_mouse` and any
  unenumerated sibling — in one change. The cap is the pragmatic suite-level
  fix; per-test hardening remains available if a specific test must stay robust
  at full parallelism.

## Verification

Per the bounded-run convention (15-min hard cap, Central-time stamped, single
tracked task, no poll-watcher):

- **Reproduce-the-fix:** `cargo test -p roastty -- --test-threads=4` run **3×**,
  **each iteration in its own `bounded-run.sh`** (so every run gets the full 900
  s headroom — a late-iteration deadlock can't be squeezed by earlier runs' time
  — and each is its own ≤15-min process) — **every run `4360 passed; 0 failed`,
  0 panics, 0 `PoisonError`**. This is "consistent with the fix" given the
  mechanism (2× CPU/test) plus the fully-clean motivating run, **not** a proof;
  if any of the 3 fails, the cap is insufficient (raise repeats / lower threads,
  or fall to the per-test fix below). The pre-fix default-parallelism baseline
  is on record (`logs/exp832/verify832b-*.log`: 2–4 failures in 4/5 runs).
- Each run reports `STATUS=COMPLETED` with `START=`/`END=` Central stamps and a
  `CMD=` line confirming `--test-threads=4`; none hits
  `HARD_TIMEOUT`/`IDLE_KILL`.
- **README rewrite is complete, not partial:** after editing, the "Test
  execution gate" section has **no leftover language mandating nextest as the
  routine gate** (grep the section for `nextest run -p roastty` / `--retries` /
  "run the full suite" — any remaining nextest mention must be explicitly framed
  as the on-demand pinpoint tool).
- `cargo build -p roastty --tests` — no warnings.
  `cargo fmt -p roastty -- --check` — clean. `git diff --check` — clean. (No
  source change, so no-ghostty grep is moot.)

**Pass** = 3/3 clean full-suite runs at `--test-threads=4` (zero failures, zero
poison, no timeout) — the suite is **flake-free**, and the convention records
the bounded-parallelism gate. **Partial/Fail** = any residual failure (e.g. if
`config_path_cli` proves to be a non-contention flake that survives the cap — it
then gets its own experiment).

## Design Review

**Reviewer:** `adversarial-reviewer` subagent (Claude Opus, fresh context,
read-only). Verified the `hyp-t4.log` clean run and the `verify832b` pre-fix
failures against the logs.

**Verdict (first pass):** CHANGES REQUIRED. The motivating evidence checked out;
one blocking defect plus four refinements:

- **Required — gate incoherence.** The original design enshrined **bare**
  `cargo test` into a README convention that mandates **nextest** (for per-test
  by-name deadlock detection), creating two contradictory gates and silently
  dropping nextest's guarantee. **Fixed:** the design now explicitly makes bare
  `cargo test -p roastty -- --test-threads=4` (via `bounded-run.sh`) the
  _routine_ gate and **rewrites** (not duplicates) the README gate, retiring
  nextest to an **on-demand** by-name pinpoint tool — justified by the 15-min
  cap (nextest's process-per-test makes a full run ~12–15 min, near the ceiling)
  and by `bounded-run.sh`'s watchdog+`sample` preserving deadlock detection.
- **Optional — strawman P-value.** **Fixed:** dropped; 3/3 clean is stated as
  "consistent with the fix," not proof.
- **Optional — `--test-threads=4` is machine-relative.** **Adopted:** noted as
  "≈ half the cores, recalibrate per machine," not a portable constant.
- **Optional — masks test-level fragility.** **Adopted:** the per-test
  echo-token-wait alternative is acknowledged and rejected with rationale (the
  cap fixes the whole class in one change; per-test hardening stays available).
- **Optional/Nit — auditability + config disposition.** **Adopted:**
  `bounded-run.sh` now logs `CMD=`; `config_path_cli`'s disposition (re-check in
  the 3× run; own experiment if it survives the cap) is stated up front.

**Re-review:** APPROVED, no Required findings. The reviewer confirmed the gate
is now coherent (one routine gate, nextest on-demand) and that the deadlock hard
guarantee survives (a wedged test → drain ~4 min + 90 s idle kill ≈ 5 min, well
under the 900 s cap). Three Optional findings, all adopted: (1) the "deadlock
detection preserved" claim now discloses its two gaps — ~5-min catch latency,
and an output-emitting **livelock** is the one class only nextest catches (use
it on-demand for that); (2) the 3× verification runs each in its **own**
`bounded-run.sh` (full headroom each); (3) a verification step greps the
rewritten README gate to ensure no leftover nextest-as-routine-gate language.
(Nit: the motivating `hyp-t4.log` predates `CMD=` logging — acceptable, it is
"consistent, not proof"; the 3× runs carry `CMD=`.)

## Result

**Result:** Partial

The README "Test execution gate" was rewritten to the bare
`cargo test -p roastty -- --test-threads=4` + `bounded-run.sh` routine gate
(nextest retired to on-demand), and `bounded-run.sh` now logs `CMD=`. The 3×
verification (each its own bounded-run, Central-stamped):

| run | STATUS                | result               | residual failure                    |
| --: | --------------------- | -------------------- | ----------------------------------- |
|   1 | COMPLETED rc=0 170s   | 4360 passed / 0 fail | —                                   |
|   2 | COMPLETED rc=101 146s | 4359 / **1 fail**    | `surface_start_uses_copied_config…` |
|   3 | COMPLETED rc=101 110s | 4359 / **1 fail**    | `surface_start_uses_copied_config…` |

All three reported `START=`/`END=`/`CMD=… --test-threads=4`, none hit
`HARD_TIMEOUT`/`IDLE_KILL` — the bounded-run gate worked exactly as designed.

**The thread cap SHIFTS the flake profile — it is not a clean elimination
(corrected in result review):**

- ✅ **The `surface_key`/`surface_mouse`/`config_path_cli` flakes did not
  appear** in any of the 3 threads=4 runs (they failed at default parallelism in
  Exp 832). Consistent with the CPU-starvation hypothesis for _those_ tests.
- ❌ **But a different test now fails that passed at default.**
  `surface_start_uses_copied_config_after_source_strings_drop` is **`... ok`
  15/15 across all Exp 831/832 logs** (default parallelism) and fails **2/3 only
  at threads=4**. So the failure is **correlated with the thread cap** — the cap
  plausibly _induced/exposed_ it. This contradicts a simple "contention fixed,
  one unrelated residual" story: parallelism tuning **moved the flake** from
  `surface_key` to `surface_start` rather than removing it.

**Honest limits of the residual diagnosis** (the earlier draft overstated this):

- The failing assert is `text.contains(current_dir.to_str().unwrap())` (passes
  the `"owned"` half) at `lib.rs:29170` — a **bare `assert!` with no
  `{text:?}`**, and the `_until` 30 s timeout dump (`lib.rs:15357`) did not
  fire. So **the failing snapshot was never captured**; the screen content is
  unknown.
- A geometry/width **wrap** was hypothesized (the test never sets geometry), but
  a wrap is **parallelism-independent** and so cannot, by itself, explain
  pass-at-default / fail-at-threads=4; and Exp 832 established this child is
  single-burst (path + `owned` render together). The real cause is
  **undiagnosed** and must be observed, not inferred.

So the gate-**runner** rewrite (bare `cargo test` + `bounded-run`, nextest
on-demand — vetted in design review for speed/15-min fit) is kept, but
`--test-threads=4` is **not validated as a clean fix**: the suite is not green
at threads=4, and the thread cap is implicated in the new failure. Recorded
**Partial**: the suite run is a gate **Fail** (not green) while this
experiment's _net contribution_ is partial (proved the `surface_key` class is
parallelism- sensitive; surfaced a thread-cap-correlated failure to diagnose).

## Conclusion

The honest lesson: parallelism tuning **moves** the timing flake rather than
removing it — at default the `surface_key` tests lose the race; at threads=4
`surface_start_uses_copied_config` does. The underlying fragility is per-test
render/snapshot timing, and a global thread count is not a reliable cure. The
gate-runner rewrite (fast bare `cargo test` + `bounded-run`, nextest on-demand)
stands; the thread _count_ is unsettled.

- **Exp 834 (next):** **diagnose before fixing.**
  `surface_start_uses_copied_config` must be **observed**, not inferred —
  instrument it to capture the failing snapshot (add `{text:?}` to the assert,
  or route through `_until`'s dump), reproduce under threads=4, and read what
  the screen actually contains (wrapped path? truncated? empty? partial `pwd`?).
  The fix follows the real cause — most likely the same
  wait-for-full-output-token robustness as Exp 832, applied to the `current_dir`
  half, **not** a speculative geometry change.
- **Open gate question for 834+:** whether `--test-threads=4` is the right
  routine value given it induced this failure, or whether the right answer is to
  make the handful of fragile round-trip tests robust at _any_ parallelism (the
  per-test token-wait route) and drop the global cap. 834's diagnosis informs
  this.

Feature work (URI/regex, remaining `os/`) resumes only once the suite runs green
across the 3× gate.

## Completion Review

**Reviewer:** `adversarial-reviewer` subagent (Claude Opus, fresh context,
read-only). Verified the v1/v2/v3 logs, the README gate rewrite, and the
workflow split, then caught two factual errors in the first result draft.

**Verdict:** CHANGES REQUIRED → corrected. The mechanical claims held (v1 clean;
v2/v3 each 1 fail on `surface_start_uses_copied_config`; `STATUS=COMPLETED`,
`CMD=…--test-threads=4`, no timeout; gate rewrite coherent). Two Required
corrections, applied above:

- **Required — false "failure originator at default" claim.** The reviewer's
  grep showed `surface_start_uses_copied_config` is `... ok` **15/15** across
  all Exp 831/832 logs and `FAILED` **only** at threads=4. **Fixed:** the result
  now states it passes at default and fails only under the cap, and that the cap
  is **implicated** (plausibly induced/exposed the failure) — parallelism tuning
  _shifts_ the flake (`surface_key` → `surface_start`), not a clean elimination.
- **Required — unsupported width-wrap diagnosis.** The failing snapshot was
  never captured (bare `assert!` with no `{text:?}`, no `_until` timeout dump),
  and a stable wrap is parallelism-independent so cannot explain pass-at-default
  / fail-at-threads=4 (and Exp 832 found the child single-burst). **Fixed:**
  downgraded to "cause undiagnosed, must be observed"; Exp 834 now leads with
  capturing the snapshot before any fix, and the gate's thread count is flagged
  as unsettled.
- **Optional — gate-Fail vs experiment-Partial vocabulary.** **Adopted:** the
  result states the suite run is a gate `Fail` (not green) while the
  experiment's net contribution is `Partial`. (Nit re `CMD=` in the plan commit:
  it is front-loaded instrumentation, acceptable.)
