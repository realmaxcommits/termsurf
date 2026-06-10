+++
[implementer]
agent = "codex"
model = "gpt-5"
reasoning = "high"

[review.design]
agent = "codex"
+++

# Experiment 51: Phase E — Unicode width properties

## Description

Experiment 50 added a live `unicode-width` A/B recipe, exposing the current
Roastty gap at the app level. The underlying cause was already identified by the
architecture audit: Roastty has page/cell support for wide cells and grapheme
storage, but `Terminal::print()` still writes narrow, single-codepoint cells and
there is no Rust `unicode/` property namespace equivalent to Ghostty's
`unicode.table.get(c)`.

This experiment adds the first Rust-side Unicode property API that
`Terminal::print()` can call in the next slice. It should mirror the upstream
shape closely enough to keep the later print rewrite mechanical:

- a `unicode` module;
- a compact `Properties` value containing at least `width`,
  `width_zero_in_grapheme`, `grapheme_break`, and `emoji_vs_base`;
- a `get(codepoint: u32)` lookup, so Rust can model Ghostty's explicit
  out-of-range fallback for values beyond the Unicode scalar range;
- representative tests for the codepoints exercised by the new live
  `unicode-width` recipe.

This is intentionally not the full grapheme-break table or the full
`Terminal::print()` rewrite. This slice should include a Ghostty-shaped
`grapheme_break` property classification for representative cases because the
print rewrite needs to inspect it, but it should defer the full
`unicode.graphemeBreak(cp1, cp2, state)` state-machine/table port to the next
grapheme-clustering slice. If direct use of a Rust Unicode crate differs from
Ghostty's generated table on a representative case, add a local compatibility
override and record the gap; the final target remains Ghostty's pinned
`vendor/ghostty/src/unicode/` semantics.

## Changes

- `roastty/Cargo.toml`
  - Add any direct Unicode crate dependency needed by `roastty` instead of
    relying on transitive dependencies.
- `roastty/src/unicode/`
  - Add a `mod.rs` with a Ghostty-shaped `Properties` struct and `get` function.
  - Implement width lookup clamped to Ghostty's `[0, 2]` terminal-cell range.
  - Implement `width_zero_in_grapheme`, `grapheme_break`, and `emoji_vs_base`
    for at least the representative variation-selector / combining / emoji cases
    needed by the next print slice.
  - Use a `u32` codepoint input and mirror Ghostty's out-of-range fallback:
    width `1`, `width_zero_in_grapheme = true`, `grapheme_break = other`, and
    `emoji_vs_base = false`.
  - Keep the API internal unless another module already needs it public.
- `roastty/src/lib.rs` or module declarations
  - Register the new module.
- Tests
  - Add unit tests for ASCII fast-path width, combining marks, CJK wide
    characters, emoji used in the live recipe, VS15/VS16, box/symbol glyphs,
    representative grapheme-break classes, and out-of-range/default behavior.
  - Where practical, cite the corresponding upstream Ghostty property behavior
    from `vendor/ghostty/src/unicode/props*.zig`.
- `issues/0802-libroastty-completion-and-mac-app/README.md`
  - Add this experiment to the index as `Designed`.
  - After implementation, record the durable fact and next Phase-E target.

## Verification

- Run formatting:
  - `cargo fmt`
  - `prettier --write --prose-wrap always --print-width 80 issues/0802-libroastty-completion-and-mac-app/README.md issues/0802-libroastty-completion-and-mac-app/51-unicode-width-properties.md`
- Run targeted tests:
  - `cargo test -p roastty unicode`
- Run full Roastty tests:
  - `cargo test -p roastty`
- Run shell syntax checks to ensure the live recipe harness remains valid:
  - `bash -n scripts/roastty-app/live-ab-smoke.sh`
  - `bash -n scripts/roastty-app/live-ab-matrix.sh`
- Run `git diff --check`.
- Run `git status --short` and verify no generated artifacts or screenshots are
  in the repo.

**Pass** = Roastty has a Ghostty-shaped Unicode property lookup with
representative width/variation tests passing, the full Roastty test suite
passes, formatting and diff checks pass, and the next experiment can rewrite
`Terminal::print()` against this API.

**Partial** = the property API and targeted tests exist, but full-suite
verification is blocked by an unrelated local failure; record the exact failure
and why it is unrelated.

**Fail** = the chosen Rust-side implementation cannot be made compatible with
the representative Ghostty width/property semantics without a generated-table
port first.

## Design Review

**Reviewer:** Codex-native adversarial subagent (`multi_agent_v1.spawn_agent`,
fresh context, read-only). **Initial verdict: CHANGES REQUIRED. Final verdict:
APPROVED.**

The reviewer found one Required issue: the first design under-specified
`grapheme_break`, even though upstream `Properties` includes it and
`Terminal.print` needs it for the next rewrite. Fixed by adding a Ghostty-shaped
`grapheme_break` property to this slice while explicitly deferring the full
`unicode.graphemeBreak(cp1, cp2, state)` table/state-machine port. The reviewer
also flagged an Optional ambiguity around Rust out-of-range behavior; fixed by
specifying a `u32` codepoint input and Ghostty's fallback properties for values
beyond the Unicode scalar range. Re-review approved with no remaining blockers.

## Result

**Result:** Pass.

Added a direct `unicode-width` dependency to `roastty`, registered an internal
`unicode` module, and implemented a Ghostty-shaped `Properties` lookup:

- `width` uses Ghostty-compatible standalone overrides for combining marks,
  spacing marks, and Hangul V/T, then falls back to `unicode-width` clamped to
  Ghostty's terminal-cell range `[0, 2]`;
- invalid / out-of-range `u32` codepoints return Ghostty's fallback properties
  (`width = 1`, `width_zero_in_grapheme = true`, `grapheme_break = other`,
  `emoji_vs_base = false`);
- `width_zero_in_grapheme`, `grapheme_break`, and `emoji_vs_base` cover the
  representative combining marks, variation selectors, emoji, CJK, box drawing,
  private-use, regional indicator, spacing mark, ZWJ, and Hangul classes needed
  by the next print rewrite.

Verification:

- `cargo fmt -- roastty/src/lib.rs roastty/src/unicode/mod.rs` — pass.
- `cargo test -p roastty unicode` — pass: 24 tests passed, including the 9 new
  Unicode property tests.
- `cargo test -p roastty` — pass: 4429 unit tests passed, the C ABI harness
  passed 1 test, and doc tests passed 0 tests. The C harness still emits the
  existing enum-conversion warnings.
- `bash -n scripts/roastty-app/live-ab-smoke.sh` — pass.
- `bash -n scripts/roastty-app/live-ab-matrix.sh` — pass.
- `git diff --check` — pass.

## Completion Review

**Reviewer:** Codex-native adversarial subagent (`multi_agent_v1.spawn_agent`,
fresh context, read-only). **Initial verdict: CHANGES REQUIRED. Final verdict:
APPROVED.**

The reviewer found three Required upstream-fidelity issues:

- combining marks were using raw `unicode-width` width `0`, but Ghostty's
  standalone width is `1`;
- Hangul V/T classes were not marked `width_zero_in_grapheme`;
- `emoji_vs_base` was too broad because it treated arbitrary emoji as variation
  sequence bases.

Fixed all three by adding a Ghostty-compatible `standalone_width` override,
marking Hangul V/T as zero-width inside graphemes, and narrowing `emoji_vs_base`
to a representative subset from `emoji-variation-sequences.txt` while keeping
`ExtendedPictographic` separate. Re-review approved the fixes with no new
Required findings. The reviewer independently verified
`cargo fmt --check -p roastty`, `cargo test -p roastty unicode`, both `bash -n`
harness checks, and `git diff --check`.

## Conclusion

Roastty now has the internal property API shape that Ghostty's `Terminal.print`
expects from `unicode.table.get(c)`. This is not yet the full generated Unicode
table or grapheme-break state machine; it is the mechanical surface and
representative compatibility layer needed to make the next slice small. The next
Phase-E experiment should rewrite `Terminal::print()` to use this module for
cell width, zero-width grapheme accumulation, variation selectors, and mode 2027
behavior.
