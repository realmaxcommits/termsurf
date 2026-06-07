+++
[implementer]
agent = "codex"
model = "gpt-5"
reasoning = "high"

[review.design]
agent = "codex"
model = "default"
reasoning = "medium"

[review.result]
agent = "codex"
model = "default"
reasoning = "medium"
+++

# Experiment 798: UTF-8 Dependency Checklist Sync

## Description

Issue 801's dependency checklist still says `simdutf` is not started. That is
stale for the current UTF-8 behavior: the terminal stream uses a ported
DFA-based `UTF8Decoder`, and several C ABI/config/input/preedit/render-state
paths validate or expose UTF-8 through Rust's standard library. The terminal
core checklist already says `UTF8Decoder` is folded into `stream.rs`, so the
dependency row should not imply no UTF-8 validation work exists.

This experiment updates the checklist wording only. It keeps the row unchecked
because Roastty has not selected a SIMD UTF-8 crate, has not implemented
SIMD-accelerated validation/transcoding, and has not audited every Ghostty
`simdutf` call site as a dependency replacement.

## Changes

- `issues/0801-roastty-libghostty-rewrite/README.md`
  - Update the `simdutf` dependency row from "not started" to scoped partial
    wording that names the terminal `UTF8Decoder` and standard-library UTF-8
    validation/exposure paths.
  - Keep the row unchecked and explicitly leave SIMD acceleration, transcoding,
    and a full call-site audit open.
  - Add the Experiment 798 index entry.
- `issues/0801-roastty-libghostty-rewrite/798-utf8-dependency-checklist-sync.md`
  - Record verification evidence and review results.

## Verification

- Inspect:
  - `roastty/src/terminal/utf8_decoder.rs`
  - `roastty/src/terminal/stream.rs`
  - `roastty/src/lib.rs`
  - `roastty/src/terminal/osc.rs`
- Inspect the broader standard-library UTF-8 validation/exposure call sites:
  - `rg -n "from_utf8|String::from_utf8|utf8" roastty/src/lib.rs roastty/src/config roastty/src/input roastty/src/renderer roastty/src/terminal`
- Run:
  - `cargo test -p roastty utf8 -- --nocapture --test-threads=1`
  - `cargo test -p roastty key_event_utf8 -- --nocapture --test-threads=1`
- Run markdown formatting:
  - `prettier --write --prose-wrap always --print-width 80 issues/0801-roastty-libghostty-rewrite/README.md issues/0801-roastty-libghostty-rewrite/798-utf8-dependency-checklist-sync.md`
- Run:
  - `git diff --check`

The experiment passes if the dependency checklist stops saying UTF-8
validation/transcoding work is not started while still keeping the row unchecked
and leaving SIMD acceleration/transcoding parity open. It is Partial if only the
terminal decoder wording can be corrected. It fails if the original "not
started" wording remains accurate.

## Design Review

Codex's first design review found one blocking issue: the verification plan
claimed broader standard-library UTF-8 validation/exposure paths without a
direct call-site check. The design was fixed by adding an `rg` verification step
across `lib.rs`, config, input, renderer, and terminal sources.

Codex re-reviewed the fixed design and found no blocking findings. The review
approved the scope because the row remains unchecked, the wording is scoped to
partial coverage, the broader call-site claim now has direct verification, and
SIMD acceleration, explicit transcoding replacement, and a full Ghostty
`simdutf` call-site audit remain open.

## Result

**Result:** Pass

The `simdutf` dependency row no longer says UTF-8 validation/transcoding work is
not started. The README now records the existing partial coverage:

- `roastty/src/terminal/utf8_decoder.rs` ports Ghostty's DFA-based
  `UTF8Decoder`, and `roastty/src/terminal/stream.rs` integrates it for split,
  pending, invalid, replacement, and raw C1 byte handling.
- C ABI, config, input, preedit, render-state, OSC, Kitty, tmux, mouse encoding,
  and related terminal paths use standard-library UTF-8 validation/exposure
  helpers where needed.

The row remains unchecked because Roastty still has no selected SIMD UTF-8
crate, no SIMD-accelerated validation/transcoding replacement, and no full
Ghostty `simdutf` call-site audit.

Verification:

- Inspected:
  - `roastty/src/terminal/utf8_decoder.rs`
  - `roastty/src/terminal/stream.rs`
  - `roastty/src/lib.rs`
  - `roastty/src/terminal/osc.rs`
- Ran call-site search:
  - `rg -n "from_utf8|String::from_utf8|utf8" roastty/src/lib.rs roastty/src/config roastty/src/input roastty/src/renderer roastty/src/terminal`
- `cargo test -p roastty utf8 -- --nocapture --test-threads=1` — 75 passed
- `cargo test -p roastty key_event_utf8 -- --nocapture --test-threads=1` — 1
  passed
- `prettier --write --prose-wrap always --print-width 80 issues/0801-roastty-libghostty-rewrite/README.md issues/0801-roastty-libghostty-rewrite/798-utf8-dependency-checklist-sync.md`
  — passed
- `git diff --check` — passed

## Conclusion

The `simdutf` dependency row was stale as "not started." Roastty has tested
UTF-8 decoding and validation/exposure foundations, but the dependency remains
partial until a SIMD/transcoding strategy and full Ghostty call-site audit are
completed.

## Completion Review

Codex reviewed the completed experiment and found no blocking findings. The
review approved the result because the row remains unchecked and partial, SIMD
acceleration, explicit transcoding replacement, selected crate choice, and full
Ghostty call-site audit remain open, and the verification evidence records the
call-site search, focused tests, Prettier, and `git diff --check`.
