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

# Experiment 423: the perfect-fit powerline predicate (is_perfect_fit_powerline)

## Description

The per-row `padding_extend` refinement (Experiment 422's deferred half)
decides, for `extend` padding, whether a row should extend its background into
the padding — it calls `neverExtendBg`, which (among other checks) returns
"never extend" if any cell is a **perfect-fit powerline glyph**. This experiment
ports that specific codepoint predicate, `is_perfect_fit_powerline`, as a
self-contained, test-driven classification (a precursor to the full
`neverExtendBg`). It is a **distinct, narrower** set from roastty's existing
broad `is_powerline` (`0xE0B0..=0xE0D7`, used for symbol classification): the
perfect-fit subset is `0xE0B0..=0xE0C8`, `0xE0CA`, `0xE0CC..=0xE0D2`, `0xE0D4`.
`neverExtendBg` itself is coupled to the terminal-core cell representation
(`content_tag`, `semantic_prompt`, `hasStyling`, `bg`) not yet in the renderer's
row data, so it stays deferred.

## Upstream behavior

In `neverExtendBg` (`renderer/row.zig`), for a codepoint cell, a powerline glyph
means "never extend" (the glyphs are perfect-fit, so extending looks bad):

```zig
switch (cell.codepoint()) {
    // Powerline
    0xE0B0...0xE0C8,
    0xE0CA,
    0xE0CC...0xE0D2,
    0xE0D4,
    => return true,

    else => {},
}
```

This is a **narrower** set than the general powerline range: it excludes
`0xE0C9`, `0xE0CB`, `0xE0D3`, and `0xE0D5..=0xE0D7` (glyphs that are not
perfect-fit separators).

## Rust mapping (`roastty/src/renderer/cell.rs`)

roastty's `cell.rs` already holds the codepoint classification predicates
(`is_covering`, `is_symbol`, the broad `is_powerline`, …). The perfect-fit
predicate joins them:

```rust
/// Whether `cp` is a "perfect-fit" powerline glyph — the subset upstream's
/// `neverExtendBg` treats as a reason to never extend a row's background (these
/// separators are perfect-fit, so extending looks bad). A **narrower** set than
/// the general [`is_powerline`] range: `0xE0B0..=0xE0C8`, `0xE0CA`,
/// `0xE0CC..=0xE0D2`, `0xE0D4`.
pub(crate) fn is_perfect_fit_powerline(cp: u32) -> bool {
    matches!(
        cp,
        0xE0B0..=0xE0C8 | 0xE0CA | 0xE0CC..=0xE0D2 | 0xE0D4,
    )
}
```

The ranges and singletons match upstream's `switch` arms exactly.

## Scope / faithfulness notes

- **Ported (bridged)**: `is_perfect_fit_powerline` — the perfect-fit powerline
  codepoint subset upstream's `neverExtendBg` uses, a precursor classification.
- **Faithful**: the set is upstream's exact arms (`0xE0B0..=0xE0C8`, `0xE0CA`,
  `0xE0CC..=0xE0D2`, `0xE0D4`) — narrower than the general powerline range,
  excluding `0xE0C9`, `0xE0CB`, `0xE0D3`, `0xE0D5..=0xE0D7`.
- **Faithful adaptation**: a standalone `cp -> bool` predicate (upstream's
  inline `switch` arm), distinct from roastty's broad `is_powerline` (symbol
  classification) — they are different concepts and must not be conflated.
- **Deferred**: the full `neverExtendBg` (the semantic-prompt check, the
  per-cell `content_tag` / `bg` / default-background checks), which needs the
  renderer's terminal-core row/cell representation; the per-row
  `padding_extend.up`/`.down` refinement that consumes it; and the live call
  site. (`is_perfect_fit_powerline` is consumed by a later slice; `cell.rs` is
  `#![allow(dead_code)]`.)
- No C ABI/header/ABI-inventory change (internal Rust).

## Changes

1. `roastty/src/renderer/cell.rs`:
   - add `pub(crate) fn is_perfect_fit_powerline(cp: u32) -> bool` (the
     perfect-fit subset), near the other codepoint predicates.
2. Tests (in `cell.rs`):
   - `is_perfect_fit_powerline` is true at the boundaries and singletons
     (`0xE0B0`, `0xE0C8`, `0xE0CA`, `0xE0CC`, `0xE0D2`, `0xE0D4`) and false in
     the gaps and just outside (`0xE0AF`, `0xE0C9`, `0xE0CB`, `0xE0D3`,
     `0xE0D5`, `0xE0D7`) — proving the narrower set vs the general powerline
     range.
3. Format and test (`cargo fmt`, accept output).

## Verification

```bash
cargo fmt
cargo fmt -- --check
cargo test -p roastty perfect_fit_powerline
cargo test -p roastty
cargo build -p roastty            # no warnings
rg -n 'ghostty|Ghostty|GHOSTTY' roastty/src/font roastty/src/renderer roastty/src/config && exit 1 || true
rg -n 'ghostty|Ghostty|GHOSTTY' roastty/src/lib.rs roastty/include/roastty.h roastty/tests/abi_harness.c && exit 1 || true
git diff --check
```

The experiment **passes** if:

- `is_perfect_fit_powerline` is true exactly on upstream's set
  (`0xE0B0..=0xE0C8`, `0xE0CA`, `0xE0CC..=0xE0D2`, `0xE0D4`) and false elsewhere
  — faithful to `neverExtendBg`'s powerline arm, distinct from the broad
  `is_powerline`;
- the test passes (the boundaries / singletons true; the gaps and just-outside
  false), and the existing tests still pass;
- the full `neverExtendBg`, the per-row refinement, and the live call site stay
  deferred;
- `cargo fmt` accepted, `cargo build -p roastty` has no warnings, and
  `cargo test -p roastty` passes with no regressions;
- the no-`ghostty`-name gates and `git diff --check` pass;
- Codex reviews the design before implementation and the result after, with all
  real findings fixed.

The experiment **fails** if the set includes a non-perfect-fit codepoint (e.g.
`0xE0C9`, `0xE0D3`) or excludes one of upstream's arms, or any public C API/ABI
changes.

## Design Review

Codex reviewed this design before implementation and **approved** it with **no
findings**. It confirmed the predicate matches upstream's `neverExtendBg`
powerline arm exactly (`0xE0B0..=0xE0C8 | 0xE0CA | 0xE0CC..=0xE0D2 | 0xE0D4`),
correctly excluding the broad-powerline gaps `0xE0C9`, `0xE0CB`, `0xE0D3`, and
`0xE0D5..=0xE0D7`, so it is not conflated with the existing broad
`is_powerline(0xE0B0..=0xE0D7)` used for symbol/graphics classification; the
name `is_perfect_fit_powerline` is clear and appropriately distinct. It judged
the precursor scoping acceptable — the full `neverExtendBg` depends on
terminal-core row/cell details `RunCell` does not currently carry, so porting
only the codepoint subset now is a clean, testable bridge — and the planned
true/false tests sufficient (boundaries, singleton inclusions, and excluded
gaps).

Review artifacts:

- Prompt: `logs/codex-review/20260604-085454-d423-prompt.md` (design)
- Result: `logs/codex-review/20260604-085454-d423-last-message.md` (design)

## Result

**Result:** Pass

The perfect-fit powerline predicate is now live.

- `roastty/src/renderer/cell.rs`:
  `pub(crate) fn is_perfect_fit_powerline(cp: u32) -> bool` —
  `matches!(cp, 0xE0B0..=0xE0C8 | 0xE0CA | 0xE0CC..=0xE0D2 | 0xE0D4)`, the
  perfect-fit subset upstream's `neverExtendBg` uses, distinct from the broad
  `is_powerline(0xE0B0..=0xE0D7)`.

Test (in `cell.rs`): `is_perfect_fit_powerline_is_the_narrow_subset` — true at
`0xE0B0`, `0xE0C8`, `0xE0CA`, `0xE0CC`, `0xE0D2`, `0xE0D4`; false at `0xE0AF`,
`0xE0C9`, `0xE0CB`, `0xE0D3`, `0xE0D5`, `0xE0D7` (the gaps the broad
`is_powerline` includes but the perfect-fit subset excludes).

Gate results:

- `cargo fmt -p roastty` accepted; `--check` clean.
- `cargo test -p roastty` → 2902 passed, 0 failed (+1, no regressions).
- `cargo build -p roastty` → no warnings.
- No-`ghostty`-name gates (font + renderer + config +
  `lib.rs`/header/`abi_harness.c`) clean; `git diff --check` clean.

## Conclusion

The perfect-fit powerline codepoint subset is ported as a self-contained
predicate, distinct from the broad `is_powerline`. It is the codepoint half of
upstream's `neverExtendBg` powerline check. The full `neverExtendBg` — the
semantic-prompt check and the per-cell `content_tag` / `bg` / default-background
checks — needs the renderer's terminal-core row/cell representation (`RunCell`
does not yet carry `semantic_prompt` / `content_tag`), so it and the per-row
`padding_extend` refinement that consumes it stay deferred. The other remaining
renderer-bridge work: the macOS-glass `bg_color` override (needs a
`background_blur` config enum), a production `MetalUniforms` constructor, and
the live per-frame call sites.

## Completion Review

Codex reviewed the completed implementation and result and **approved** with
**no findings**. It confirmed the predicate matches upstream's `neverExtendBg`
powerline arm exactly (`0xE0B0..=0xE0C8 | 0xE0CA | 0xE0CC..=0xE0D2 | 0xE0D4`)
and is correctly distinct from the broad `is_powerline(0xE0B0..=0xE0D7)`
classifier, with the test covering the included boundaries/singletons and the
excluded gaps (`0xE0C9`, `0xE0CB`, `0xE0D3`, `0xE0D5`, `0xE0D7`). It confirmed
the full `neverExtendBg` and the padding refinement remain properly deferred. No
public C ABI/header impact; nothing needed to change before the result commit.

Review artifacts:

- Prompt: `logs/codex-review/20260604-085641-r423-prompt.md` (result)
- Result: `logs/codex-review/20260604-085641-r423-last-message.md` (result)
