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

# Experiment 327: font discovery — the ranking score

## Description

`CoreText.discover` sorts its candidate descriptors (Experiment 326) by a
**ranking score**: upstream's `Score` is a _packed struct_ whose fields are
ordered by increasing precedence, so that bit-casting it to an integer gives a
single value where a higher number means a better match — and the sort just
compares those integers. This experiment ports that `Score` value type and its
integer projection (`int()`) **in isolation** — the pure bit-packing logic,
which is the most error-prone and the most exhaustively testable part. Computing
a `Score` from a font (`score()`, which loads the font and reads its tables) and
the `sortMatchingDescriptors` wiring are the next experiments.

## Upstream behavior (`discovery.zig` `CoreText.Score`)

```zig
/// Packed structs store fields least- to most-significant, so these are in
/// increasing order of precedence.
const Score = packed struct {
    glyph_count: u16 = 0,   // bits 0..16  — tie-breaker: more glyphs is better
    fuzzy_style: u8 = 0,    // bits 16..24 — fuzzy style-string match strength
    bold: bool = false,     // bit 24      — bold-ness matches the request
    italic: bool = false,   // bit 25      — italic-ness matches the request
    exact_style: bool = false, // bit 26   — exact (case-insensitive) style match
    monospace: bool = false,   // bit 27   — monospace (unless seeking a codepoint)
    codepoint: bool = false,   // bit 28   — has the requested codepoint (top)

    pub fn int(self: Score) Backing { return @bitCast(self); }
};
// lhs ranks before rhs iff lhs.int() > rhs.int()
```

The precedence (low → high) is: `glyph_count` < `fuzzy_style` < `bold` <
`italic` < `exact_style` < `monospace` < `codepoint`. So a font that has the
requested codepoint always outranks one that doesn't; among those, monospace
wins; then an exact style match; then italic-ness; then bold-ness; then the
fuzzy style score; and finally, all else equal, more glyphs.

## Rust mapping (`roastty/src/font/discovery.rs`)

- A `Score` struct mirroring the fields:
  ```rust
  #[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
  pub(crate) struct Score {
      pub glyph_count: u16,
      pub fuzzy_style: u8,
      pub bold: bool,
      pub italic: bool,
      pub exact_style: bool,
      pub monospace: bool,
      pub codepoint: bool,
  }
  ```
- `pub(crate) fn int(&self) -> u32` reproducing the packed layout by bit offset:
  ```rust
  self.glyph_count as u32
      | (self.fuzzy_style as u32) << 16
      | (self.bold as u32) << 24
      | (self.italic as u32) << 25
      | (self.exact_style as u32) << 26
      | (self.monospace as u32) << 27
      | (self.codepoint as u32) << 28
  ```
  (Upstream's backing integer is `u29`; `u32` is wider with the top bits always
  zero, so the **ordering is identical** — only relative comparison matters.)
- `impl Ord for Score` (and `PartialOrd`) so the candidate list can be sorted: a
  **higher** `int()` ranks **earlier**, i.e. `Score::cmp` orders by `int()`
  descending (or the sort uses `b.int().cmp(&a.int())`). The exact sort call
  into `discover_descriptors` is the later `sortMatchingDescriptors` experiment;
  this experiment lands the value type + projection + ordering.

## Scope / faithfulness notes

- **Ported**: the `Score` value type, its `int()` integer projection (the exact
  packed-struct bit layout and field precedence), and the "higher is earlier"
  ordering.
- **Deferred**: `score(desc, ct_desc)` — the function that _computes_ a `Score`
  by loading the candidate font and reading its glyph count, codepoint coverage,
  symbolic traits, `head`/`OS/2`/variation bold-italic, and style strings — and
  `sortMatchingDescriptors` (wiring the ordering into `discover_descriptors`).
  Those are the next experiments.
- No C ABI/header/ABI-inventory change (`Score` is internal Rust).

## Changes

1. `roastty/src/font/discovery.rs`: add the `Score` struct, `int()`, and the
   `Ord`/`PartialOrd` impls.
2. Tests (in `discovery.rs`):
   - `score_field_offsets`: each single field set in isolation projects to the
     expected bit (`glyph_count = 0xABCD → int() == 0xABCD`;
     `fuzzy_style = 0xEF → int() == 0xEF_0000`; `bold → 1<<24`;
     `italic → 1<<25`; `exact_style → 1<<26`; `monospace → 1<<27`;
     `codepoint → 1<<28`).
   - `score_precedence`: each higher-precedence field outranks **all** lower
     ones combined — e.g. a `codepoint`-only score beats a score with
     `monospace+exact_style+italic+bold+fuzzy_style=0xFF+glyph_count=0xFFFF`;
     likewise `monospace` beats everything below it, and so on down the chain.
   - `score_glyph_count_tiebreak`: two otherwise-equal scores order by
     `glyph_count` (more glyphs ranks earlier).
   - `score_ord_sorts_desc`: a `Vec<Score>` sorted with the `Ord` impl is in
     descending `int()` order (best first).
3. Format and test (`cargo fmt`, accept output).

## Verification

```bash
cargo fmt
cargo fmt -- --check
cargo test -p roastty score
cargo test -p roastty
cargo build -p roastty            # no warnings
rg -n 'ghostty|Ghostty|GHOSTTY' roastty/src/font && exit 1 || true
rg -n 'ghostty|Ghostty|GHOSTTY' roastty/src/lib.rs roastty/include/roastty.h roastty/tests/abi_harness.c && exit 1 || true
git diff --check
```

The experiment **passes** if:

- `Score::int()` reproduces upstream's packed-struct bit layout and field
  precedence exactly, and the `Ord` impl ranks a higher `int()` earlier;
- the field-offset, precedence, tie-break, and sort tests pass;
- `score()` (the font-reading computation) and `sortMatchingDescriptors` stay
  deferred;
- `cargo fmt` accepted, `cargo build -p roastty` has no warnings, and
  `cargo test -p roastty` passes with no regressions;
- the no-`ghostty`-name gates and `git diff --check` pass;
- Codex reviews the design before implementation and the result after, with all
  real findings fixed.

The experiment is **partial** if a precedence relation cannot be expressed
because of an unexpected `int()` overflow (none expected — `u32` holds all 29
bits).

The experiment **fails** if the bit layout, the field precedence, or the
ordering diverges from upstream.

## Design Review

Codex reviewed this design before implementation and found **no Required
findings**. It confirmed the bit layout is correct (`glyph_count` at `0..16`,
`fuzzy_style` at `16..24`, then `bold`/`italic`/`exact_style`/`monospace`/
`codepoint` at bits `24..28`, matching the Zig packed-field order with
`codepoint` the highest bit), that using `u32` instead of Zig's inferred `u29`
is safe (the maximum packed value is `0x1FFF_FFFF`, so the top three `u32` bits
stay zero and unsigned ordering is identical), and that the "higher `int()`
ranks earlier" direction is correct (upstream's sort treats
`lhs.int() > rhs.int()` as "lhs earlier", so a best-first sort compares
`other.int().cmp(&self.int())`). It confirmed isolating `Score` from
`score()`/`sortMatchingDescriptors` is a sensible slice and the proposed tests
cover the important failure modes.

Implementation choice (folded in): rather than a surprising _reversed_ `Ord`,
the implementation gives `Score` a **natural** `Ord` (higher `int()` is
`Greater`) and the consumer/sort reverses for best-first
(`sort_by(|a, b| b.cmp(a))`). The `score_ord_sorts_desc` test sorts descending
accordingly. This keeps `Ord` consistent with the derived `PartialEq` (the field
tuple ↔ `int()` is a bijection).

Review artifacts:

- Prompt: `logs/codex-review/20260603-113936-991480-prompt.md`
- Result: `logs/codex-review/20260603-113936-991480-last-message.md`
