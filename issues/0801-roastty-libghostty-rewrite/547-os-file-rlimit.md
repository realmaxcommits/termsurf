+++
[implementer]
agent = "claude-code"
model = "claude-opus-4-8"
reasoning = "high"

[review.design]
agent = "codex"
model = "gpt-5.5"
reasoning = "medium"

[review.result]
agent = "codex"
model = "gpt-5.5"
reasoning = "medium"
+++

# Experiment 547: maximize the file-descriptor limit (os::file rlimit)

## Description

This experiment completes the `os::file` port (Experiment 544 did its temp-path
helpers) with the remaining two functions of upstream `os/file.zig`:
`fix_max_files` and `restore_max_files`. `fix_max_files` raises the process's
open-file-descriptor soft limit (`RLIMIT_NOFILE`) toward its hard limit —
necessary because each terminal window/pane consumes several fds — and returns
the previous limit so it can be put back; `restore_max_files` restores it. The
algorithm is the binary search lifted from the Zig compiler.

## Upstream behavior

`os/file.zig`:

```zig
pub const rlimit = if (@hasDecl(posix.system, "rlimit")) posix.rlimit else struct {};

/// Maximize the number of open file descriptors. Returns the old limit (to restore later).
pub fn fixMaxFiles() ?rlimit {
    if (!@hasDecl(posix.system, "rlimit") or posix.system.rlimit == void) return null;

    const old = posix.getrlimit(.NOFILE) catch {
        log.warn("failed to query file handle limit, may limit max windows", .{});
        return null;
    };

    // If we're already at the max, we're done.
    if (old.cur >= old.max) return old;

    // Binary search for the limit.
    var lim = old;
    var min: posix.rlim_t = lim.cur;
    var max: posix.rlim_t = 1 << 20;
    // If there's a defined upper bound, don't search, just set it.
    if (lim.max != posix.RLIM.INFINITY) { min = lim.max; max = lim.max; }

    while (true) {
        lim.cur = min + @divTrunc(max - min, 2);
        if (posix.setrlimit(.NOFILE, lim)) |_| { min = lim.cur; } else |_| { max = lim.cur; }
        if (min + 1 >= max) break;
    }

    return old;
}

pub fn restoreMaxFiles(lim: rlimit) void {
    if (!@hasDecl(posix.system, "rlimit")) return;
    posix.setrlimit(.NOFILE, lim) catch {};
}
```

- `fixMaxFiles`: query `RLIMIT_NOFILE`; if the soft limit (`cur`) already equals
  the hard limit (`max`), return it unchanged. Otherwise binary-search the
  highest settable `cur`: range `[cur, 1<<20)`, or — if the hard limit is not
  `INFINITY` — just `[max, max]` (set `cur = max` directly). Each step tries
  `setrlimit`, moving `min` up on success and `max` down on failure, until
  `min + 1 >= max`. Returns the **old** limit.
- `restoreMaxFiles`: `setrlimit(RLIMIT_NOFILE, old)`, ignoring errors.
- On a platform without `rlimit`, both are no-ops (`null` / nothing); a
  `getrlimit` failure logs and returns `null`.

## Rust mapping (`roastty/src/os/file.rs`)

`libc::rlimit` + `libc::getrlimit` / `libc::setrlimit` on `RLIMIT_NOFILE`, a
faithful port of the binary search:

```rust
/// Maximize the number of open file descriptors (`RLIMIT_NOFILE`) and return the previous
/// limit so it can be restored (upstream `os.file.fixMaxFiles`). Each window/pane consumes
/// several fds, so we raise the soft limit toward the hard limit. `None` if the limit can't
/// be queried.
pub(crate) fn fix_max_files() -> Option<libc::rlimit> {
    let mut old = libc::rlimit { rlim_cur: 0, rlim_max: 0 };
    // Oh well; we tried. (Upstream logs a warning that max windows may be limited.)
    if unsafe { libc::getrlimit(libc::RLIMIT_NOFILE, &mut old) } != 0 {
        return None;
    }

    // If we're already at the max, we're done.
    if old.rlim_cur >= old.rlim_max {
        return Some(old);
    }

    // Binary search for the limit.
    let mut min: libc::rlim_t = old.rlim_cur;
    let mut max: libc::rlim_t = 1 << 20;
    // If there's a defined upper bound, don't search — just set it.
    if old.rlim_max != libc::RLIM_INFINITY {
        min = old.rlim_max;
        max = old.rlim_max;
    }

    loop {
        let mut lim = old;
        lim.rlim_cur = min + (max - min) / 2;
        if unsafe { libc::setrlimit(libc::RLIMIT_NOFILE, &lim) } == 0 {
            min = lim.rlim_cur;
        } else {
            max = lim.rlim_cur;
        }
        if min + 1 >= max {
            break;
        }
    }

    Some(old)
}

/// Restore a file-descriptor limit previously returned by `fix_max_files` (upstream
/// `os.file.restoreMaxFiles`). Errors are ignored.
pub(crate) fn restore_max_files(lim: libc::rlimit) {
    unsafe { libc::setrlimit(libc::RLIMIT_NOFILE, &lim) };
}
```

`libc::rlimit` mirrors `posix.rlimit` (`rlim_cur` / `rlim_max`); the
`while (true)` do-while with the `min + 1 >= max` exit becomes a Rust `loop`.
`@divTrunc(max - min, 2)` is plain `(max - min) / 2` on the unsigned `rlim_t`.
The `@hasDecl` / `void` platform guards drop (macOS always has `rlimit`). The
`getrlimit`-failure path returns `None` (upstream also logs — roastty has no
logging in this module).

## Scope / faithfulness notes

- **Ported (bridged)**: `os.file.fixMaxFiles` → `os::file::fix_max_files`;
  `os.file.restoreMaxFiles` → `os::file::restore_max_files`. With this,
  `os::file` is fully ported on the macOS arm.
- **Faithful**: query `RLIMIT_NOFILE`; the already-maxed early return; the
  binary search (`[cur, 1<<20)` or `[max, max]` when the hard limit isn't
  `INFINITY`; `setrlimit` moves `min` up / `max` down until `min + 1 >= max`);
  return the old limit; `restore` sets the old limit ignoring errors.
- **Faithful adaptation**: `posix.getrlimit` / `setrlimit` → `libc::getrlimit` /
  `libc::setrlimit`; `posix.rlimit` → `libc::rlimit`; `posix.RLIM.INFINITY` →
  `libc::RLIM_INFINITY`; `?rlimit` → `Option<libc::rlimit>`; the `while (true)`
  → `loop`; the platform `rlimit`-absent guards drop (macOS-only); the warn-log
  on `getrlimit` failure → a comment (no logger here).
- **Deferred**: nothing — this completes `os::file`.
- No C ABI/header/ABI-inventory change (internal Rust).

## Changes

1. `roastty/src/os/file.rs`: add `fix_max_files`, `restore_max_files`.
2. Tests (in `file.rs`):
   - **fix then restore**: `fix_max_files()` returns `Some(old)`; after it, the
     queried soft limit is `>= old.rlim_cur` (never lowered);
     `restore_max_files(old)` then returns the queried limit to exactly `old`
     (both `rlim_cur` and `rlim_max`). (The test mutates and restores the
     process `RLIMIT_NOFILE` — raising the fd limit is benign, and it is
     restored.)
3. Format and test (`cargo fmt`, accept output).

## Verification

```bash
cargo fmt
cargo fmt -- --check
cargo test -p roastty os::file
cargo test -p roastty
cargo build -p roastty            # no warnings
rg -n 'ghostty|Ghostty|GHOSTTY' roastty/src/font roastty/src/renderer roastty/src/config roastty/src/os/file.rs roastty/src/os/mod.rs && exit 1 || true
rg -n 'ghostty|Ghostty|GHOSTTY' roastty/src/lib.rs roastty/include/roastty.h roastty/tests/abi_harness.c && exit 1 || true
git diff --check
```

The experiment **passes** if:

- `fix_max_files` queries `RLIMIT_NOFILE`, returns the old limit unchanged when
  already maxed, otherwise binary-searches the soft limit upward (never below
  the old `cur`) and returns the old limit; `restore_max_files` restores it —
  faithful to `os/file.zig`;
- the test passes (raise then restore exactly), and the existing tests still
  pass;
- `cargo fmt` accepted, `cargo build -p roastty` has no warnings, and
  `cargo test -p roastty` passes with no regressions;
- the no-`ghostty`-name gates and `git diff --check` pass;
- Codex reviews the design before implementation and the result after, with all
  real findings fixed.

The experiment **fails** if the search/limit semantics diverge from upstream, an
unrelated item changes, or any public C API/ABI changes.

## Design Review

Codex reviewed this design before implementation and **approved** it with **no
findings**. Codex confirmed the search is faithful to upstream: it returns the
old limit, preserves the already-at-max early return, handles the bounded
hard-limit branch by attempting `cur = hard` once, and otherwise mirrors the
`min`-up-on-success / `max`-down-on-failure loop with the `min + 1 >= max`
termination; `libc::rlimit` / `rlim_cur` / `rlim_max` / `RLIMIT_NOFILE` /
`RLIM_INFINITY` are the right macOS libc equivalents and plain `/ 2` on `rlim_t`
matches `@divTrunc` for this unsigned case; dropping the platform guards and
replacing the warning log with a `None` return is acceptable for the macOS-only
slice; and the test's process-wide mutation is reasonable since it only raises
the soft limit and restores the old value afterward.

Review artifacts:

- Prompt: `logs/codex-review/20260604-d547-prompt.md` (design)
- Result: `logs/codex-review/20260604-d547-last-message.md` (design)

## Result

**Result:** Pass

`fix_max_files` and `restore_max_files` were added to `os::file`, completing the
`os/file.zig` port on the macOS arm. `fix_max_files` queries `RLIMIT_NOFILE` via
`libc::getrlimit` (`None` on failure), returns the old limit when already maxed,
otherwise binary-searches the soft limit upward (`[cur, 1<<20)` or
`[hard, hard]` when the hard limit isn't `RLIM_INFINITY`, moving `min` up on
`setrlimit` success and `max` down on failure until `min + 1 >= max`), and
returns the old limit; `restore_max_files` sets it back, ignoring errors. One
test raises the limit (asserting the soft limit is never lowered) then restores
it (asserting the queried limit returns to exactly `old`).

Gates:

- `cargo fmt -p roastty` accepted; `--check` clean.
- `cargo test -p roastty`: 3058 passed, 0 failed (one new test; no regressions,
  up from 3057).
- `cargo build -p roastty`: no warnings.
- no-`ghostty`-name greps (font/renderer/config + os/file.rs + os/mod.rs +
  lib.rs/header/abi_harness.c) clean; `git diff --check` clean.

## Completion Review

Codex reviewed the completed experiment and **approved** it with **one Nit** (no
Required or Optional findings): the doc had `## Result` but no `## Conclusion` —
fixed by adding the conclusion below. Codex confirmed the implementation matches
upstream `file.zig` and the approved design: it returns the old limit, preserves
the already-maxed early return, implements the bounded hard-limit branch as a
single attempt to set `cur = hard`, and otherwise follows the same min/max
binary search and termination condition; `restore_max_files` correctly ignores
`setrlimit` errors; and the test soundly confirms the limit is not lowered and
is then restored to exactly the old `rlim_cur` / `rlim_max`.

Review artifacts:

- Prompt: `logs/codex-review/20260604-r547-prompt.md` (result)
- Result: `logs/codex-review/20260604-r547-last-message.md` (result)

## Conclusion

`os::file` is now **fully ported** on the macOS arm: the temp-path helpers
(Experiment 544) plus `fix_max_files` / `restore_max_files` — the binary-search
fd-limit raiser that lets roastty open enough descriptors for many windows/panes
(wiring into startup deferred). With this the `os` module spans `hostname`,
`path`, `env`, `file`, `temp_dir`, and `pipe`. The OS-utility frontier still has
a few self-contained slices (`i18n_locales`, `kernel_info`, `resourcesdir`). The
config `loadDefaultFiles` stays deferred pending roastty's naming decision;
`background-image-opacity` stays float-blocked.
