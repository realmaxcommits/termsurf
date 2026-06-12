# Experiment 151: Phase I — codepoint width fast path

## Description

Finish the width half of the Phase I SIMD/perf item left Partial by
Experiment 150. Upstream Ghostty exposes `simd.codepointWidth`, backed by
`src/simd/codepoint_width.cpp`, as a width-only helper that uses compact range
tables and vector lane comparisons instead of the full Unicode property lookup.

Roastty should mirror that shape without pretending a width-only result is a
full `Properties` value. The existing `unicode::get` table remains the
authoritative source for grapheme-break and emoji-variation metadata. This
experiment adds a dedicated width helper, ports the upstream range categories
into Rust, proves full-range width parity against the generated table, and wires
only the terminal print width hot path through the helper.

This experiment may be range-accelerated rather than explicit Rust SIMD if that
is the only maintainable way to match the upstream helper without adding a C++
build bridge. To close the SIMD/perf checklist item, the accepted result must
still demonstrate a release-mode speedup over the generated-table width lookup
on a representative non-ASCII print-width workload.

## Changes

- `roastty/src/unicode/mod.rs`
  - Add `pub(crate) fn codepoint_width(codepoint: u32) -> u8`.
  - Preserve upstream's width-only semantics:
    - printable/non-control codepoints `<= 0xff` return width `1`, matching
      `CodepointWidth`;
    - C0 controls and C1 controls (`U+0080..=U+009F`) must not be widened by
      this fast helper when they reach terminal print handling. Either reject
      them before `codepoint_width` in `TerminalStreamHandler::print`, or keep
      those codepoints on the existing table-width behavior;
    - out-of-range codepoints return the existing Ghostty fallback width `1`;
    - width `3` cases such as `U+2E3B` clamp to `2`, matching Ghostty's terminal
      width limit.
  - Port the upstream `codepoint_width.cpp` range data into Rust helper tables:
    fixed 16-bit/32-bit wide ranges, definite zero-width ranges, East Asian
    width ranges, and non-spacing-mark ranges.
  - Keep `unicode::get` and `table_properties` available for metadata-heavy
    callers; do not replace `Properties` with width-only guesses.
  - Replace the Experiment 150 ASCII-only perf probe with a release-mode
    `simd_fast_path_perf_codepoint_width` probe that compares `codepoint_width`
    against `table_properties(codepoint).width` on a mixed non-ASCII workload
    containing narrow, wide, zero-width, and supplementary codepoints. The probe
    must fail below a 1.05x speedup if the result is to be recorded as Pass.
  - Add correctness tests:
    - upstream basic cases from `src/simd/codepoint_width.zig`;
    - boundary tests around representative 16-bit and 32-bit zero/wide ranges;
    - `codepoint_width` matches `table_properties(...).width` for every valid
      Unicode scalar value in `0x0000..=0x10ffff`, skipping surrogate values
      `0xd800..=0xdfff`, with the documented `<= 0xff` caller-filter/control
      exception handled explicitly in the assertion.
- `roastty/src/terminal/terminal.rs`
  - In `TerminalStreamHandler::print`, compute the print width with
    `unicode::codepoint_width(codepoint)` for the width-only print/spacer logic.
  - Continue using `unicode::get(codepoint)` only when grapheme clustering,
    variation selectors, or `width_zero_in_grapheme` / `emoji_vs_base` /
    `grapheme_break` metadata are needed.
  - Add or adjust tests that prove C0 controls and C1 controls are still
    rejected or table-width-preserved before the `<= 0xff => 1` fast path can
    widen them, ASCII printable text remains width 1, wide CJK/emoji codepoints
    still occupy two cells, and zero-width marks still attach/ignore exactly as
    before.
- `issues/0802-libroastty-completion-and-mac-app/README.md`
  - If width parity and perf pass, update the Phase I SIMD checklist line to
    show that base64, index-of, VT ASCII, and codepoint width are now covered.
  - If the range helper is correct but not faster, keep the checklist open and
    record the next required bridge/intrinsics work instead of overclaiming.

## Verification

- `cargo fmt`
- `cargo test -p roastty unicode -- --test-threads=1`
- `cargo test -p roastty terminal -- --test-threads=1`
- `cargo test -p roastty -- --test-threads=1`
- `cd roastty && macos/build.nu --action test`
- `cargo fmt --check`
- `git diff --check`
- `cargo test --release -p roastty simd_fast_path_perf_codepoint_width -- --ignored --nocapture --test-threads=1`

**Pass** = Roastty has a width-only fast helper matching upstream
`simd.codepointWidth` semantics, terminal print width uses it without weakening
grapheme/emoji metadata behavior, exhaustive Unicode scalar parity against the
generated table passes, the release perf probe shows at least a 1.05x speedup,
all listed Rust and hosted macOS checks pass, and the README marks the Phase I
SIMD checklist item complete.

**Partial** = the helper is correct and wired but the release perf probe is
below 1.05x, or the implementation must leave a subset of width handling on the
generated table to preserve behavior; the README must keep the SIMD checklist
open and document the exact remaining bridge/intrinsics work.

**Fail** = the helper diverges from the generated table for valid scalar values,
changes terminal print/grapheme behavior, or cannot be verified without
weakening the existing Unicode metadata contract.

## Design Review

**Reviewer:** Codex-native adversarial subagent with fresh context, using the
`adversarial-review` skill's Codex path (`multi_agent_v1.spawn_agent`), not
Claude's named `adversarial-reviewer` agent.

**Initial verdict:** Changes required.

**Required finding:** The first plan only said callers filter controls before
the `<= 0xff => 1` width helper, but `TerminalStreamHandler::print` currently
rejects ASCII controls only. C1 controls (`U+0080..=U+009F`) could therefore be
widened if the fast helper were called unconditionally.

**Nit:** The exhaustive parity wording said every Unicode scalar from
`0x0000..=0x10ffff`, which imprecisely includes surrogate codepoints.

**Fixes:**

- Required the implementation to reject C0 and C1 controls before
  `codepoint_width`, or keep those codepoints on existing table-width behavior.
- Added explicit C1 control coverage to the terminal test plan.
- Tightened the exhaustive parity wording to valid Unicode scalar values,
  skipping `0xd800..=0xdfff`.

**Re-review:** Approved. The reviewer confirmed the C1 control requirement and
valid-scalar parity wording resolve the prior findings.

**Final verdict:** Approved.
