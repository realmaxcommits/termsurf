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

# Experiment 316: the unifying codepoint sprite dispatch (draw_codepoint)

## Description

Every codepoint-keyed sprite glyph family has its own standalone
`draw_*(cp, metrics, canvas) -> bool` dispatcher (each returns `false` and draws
nothing when the codepoint is not in its range). This experiment adds the
**unifying** `draw_codepoint`, a single entry point that tries every family in
turn and renders the first one that matches — the dispatch upstream's sprite
`Face` uses to turn a codepoint into a drawn glyph. It is the first half of
filling the resolver's deferred `SpriteUnavailable` arm (the atlas/resolver
wiring and the sprite-kind special glyphs are later experiments).

## Background

The codepoint-keyed draw families (all
`(cp: u32, metrics: &Metrics, canvas: &mut Canvas) -> bool`, except the
powerline ones, which also take the glyph `width`/`height`):

- box drawing: `draw_box_lines`, `draw_box_dashes`, `draw_box_diagonal`,
  `draw_box_arc`;
- braille: `draw_braille`;
- legacy computing: `draw_sextant`, `draw_octant`, `draw_separated_quadrant`,
  `draw_block`;
- geometric: `draw_corner_triangle`, `draw_corner_triangle_outline`;
- powerline: `draw_powerline_triangle(cp, width, height, canvas)`,
  `draw_powerline_chevron(cp, width, height, metrics, canvas)`,
  `draw_powerline_rounded(cp, width, height, metrics, canvas)`,
  `draw_powerline_diagonal(cp, metrics, canvas)`,
  `draw_powerline_flame(cp, width, height, metrics, canvas)`.

Each family's codepoint ranges are disjoint, and a non-matching family draws
nothing — so a short-circuit `||` chain dispatches each codepoint to exactly one
family.

## Rust mapping (`roastty/src/font/sprite/draw.rs`)

`pub(crate) fn draw_codepoint(cp: u32, metrics: &Metrics, canvas: &mut Canvas) -> bool`
— with `w = metrics.cell_width`, `h = metrics.cell_height`, return the `||`
chain of every codepoint-keyed family, passing `w`/`h` to the powerline
functions that need them:

```text
draw_box_lines(cp, metrics, canvas)
    || draw_box_dashes(cp, metrics, canvas)
    || draw_box_diagonal(cp, metrics, canvas)
    || draw_box_arc(cp, metrics, canvas)
    || draw_braille(cp, metrics, canvas)
    || draw_sextant(cp, metrics, canvas)
    || draw_octant(cp, metrics, canvas)
    || draw_separated_quadrant(cp, metrics, canvas)
    || draw_block(cp, metrics, canvas)
    || draw_corner_triangle(cp, metrics, canvas)
    || draw_corner_triangle_outline(cp, metrics, canvas)
    || draw_powerline_triangle(cp, w, h, canvas)
    || draw_powerline_chevron(cp, w, h, metrics, canvas)
    || draw_powerline_rounded(cp, w, h, metrics, canvas)
    || draw_powerline_diagonal(cp, metrics, canvas)
    || draw_powerline_flame(cp, w, h, metrics, canvas)
```

Because `||` short-circuits, the first family that returns `true` (and has
drawn) stops the chain; a codepoint no family matches leaves the canvas
untouched and returns `false`. Update the module doc.

## Scope / faithfulness notes

- **Ported**: the unifying codepoint dispatch (`draw_codepoint`).
- **Deferred**: a non-rendering `has_codepoint` predicate (the codepoint-range
  enumeration), the sprite-kind special glyphs (underlines/cursors, keyed by a
  `Sprite` enum, not a codepoint), and the resolver/atlas wiring that calls this
  to fill `SpriteUnavailable`.
- No C ABI/header/ABI-inventory change.

## Changes

1. `roastty/src/font/sprite/draw.rs`: add `draw_codepoint`; update the module
   doc.
2. Tests (deterministic — the fixture `9×18` cell):
   - `draw_codepoint_dispatches`: for a representative codepoint from **every**
     family — `0x2500` box line, `0x2504` box dashes, `0x2571` diagonal,
     `0x2570` arc, `0x2802` braille, `0x1FB00` sextant, `0x1CD00` octant,
     `0x1CC21` separated quadrant, `0x2588` block, `0x25E2` corner triangle,
     `0x25F8` outlined triangle, `0xE0B0` powerline triangle, `0xE0B1` chevron,
     `0xE0B4` rounded, `0xE0B9` powerline diagonal, `0xE0D2` flame —
     `draw_codepoint` returns `true`, and its rendered buffer (on a **fresh**
     canvas) equals the buffer from calling the **direct** family dispatcher on
     a **second fresh** canvas, confirming each arm is reached and not shadowed
     (per the design review: two separate canvases, not sequential draws on
     one).
   - `draw_codepoint_excludes`: an un-handled codepoint (`'M'`, `0x0041`,
     `0x20`) returns `false` and leaves the canvas blank.
   - (The exact representative codepoints are confirmed against the existing
     family dispatchers during implementation.)
3. Format and test (`cargo fmt`, accept output).

## Verification

```bash
cargo fmt
cargo fmt -- --check
cargo test -p roastty sprite
cargo test -p roastty
cargo build -p roastty            # no warnings
rg -n 'ghostty|Ghostty|GHOSTTY' roastty/src/font && exit 1 || true
rg -n 'ghostty|Ghostty|GHOSTTY' roastty/src/lib.rs roastty/include/roastty.h roastty/tests/abi_harness.c && exit 1 || true
git diff --check
```

The experiment **passes** if:

- `draw_codepoint` dispatches each codepoint to exactly the right family (the
  rendered buffer equals the direct family call) and returns `false` (drawing
  nothing) for un-handled codepoints;
- the dispatch and exclusion tests confirm the routing across all families;
- the `has_codepoint` predicate, the special-sprite glyphs, and the
  resolver/atlas wiring stay deferred;
- `cargo fmt` accepted, `cargo build -p roastty` has no warnings, and
  `cargo test -p roastty` passes with no regressions;
- the no-`ghostty`-name gates and `git diff --check` pass;
- Codex reviews the design before implementation and the result after, with all
  real findings fixed.

The experiment is **partial** if two families share a codepoint and the `||`
order shadows one (it should not — the ranges are disjoint).

The experiment **fails** if the dispatch routes a codepoint to the wrong family,
or any public C API/ABI changes.

## Design Review

Codex reviewed this design before implementation and raised one **Required**
finding: the representative-codepoint test list omitted three families in the
chain — `draw_box_dashes`, `draw_separated_quadrant`, and `draw_block` — so a
broken or omitted arm could pass undetected. Fixed: the test now covers
**every** family, adding `0x2504` (box dashes), `0x1CC21` (separated quadrant),
and `0x2588` (block). One **Optional** suggestion — make the buffer-equality
test explicitly use two fresh canvases (one through `draw_codepoint`, one
through the direct family call), since sequential draws on one canvas would not
be a valid equality check — folded into the test wording. Codex confirmed the
rest is sound: the codepoint families are disjoint; the dispatchers return
`false` before drawing on non-matches (so the `||` short-circuit is safe); the
`||` chain is a faithful equivalent of upstream's single-route switch; deferring
`has_codepoint` and the resolver/atlas wiring is reasonable; and the sample
codepoints are valid (`0x1FB00` sextant, `0x1CD00` octant, `0x2802` braille,
`0xE0B4` rounded).

Review artifacts:

- Prompt: `logs/codex-review/20260603-092259-788283-prompt.md`
- Result: `logs/codex-review/20260603-092259-788283-last-message.md`

## Result

**Result:** Pass

`roastty/src/font/sprite/draw.rs` gained
`draw_codepoint(cp, metrics, canvas) -> bool`: a short-circuit `||` chain over
all 16 codepoint-keyed families (the four box families, braille, the four
legacy-computing families, the two geometric corner triangles, and the five
powerline families), passing `cell_width`/ `cell_height` to the powerline
functions that need them. The first family that matches draws and stops the
chain; an un-handled codepoint leaves the canvas untouched and returns `false`.

Tests:

- `draw_codepoint_dispatches` — one representative codepoint per family
  (`0x2500` box line, `0x2504` box dashes, `0x2571` diagonal, `0x2570` arc,
  `0x2802` braille, `0x1FB00` sextant, `0x1CD00` octant, `0x1CC21` separated
  quadrant, `0x2588` block, `0x25E2` corner triangle, `0x25F8` outlined
  triangle, `0xE0B0` powerline triangle, `0xE0B1` chevron, `0xE0B4` rounded,
  `0xE0B9` powerline diagonal, `0xE0D2` flame): `draw_codepoint`'s buffer (fresh
  canvas) equals the direct family call's buffer (second fresh canvas), pixel
  for pixel.
- `draw_codepoint_excludes` — `'M'`, `0x0041`, `0x20` return `false` and leave
  the canvas blank.

Gate results:

- `cargo fmt -p roastty` accepted; `--check` clean.
- `cargo test -p roastty` → 2661 passed, 0 failed (+2, no regressions).
- `cargo build -p roastty` → no warnings.
- No-`ghostty`-name gates clean; `git diff --check` clean.

## Conclusion

The unifying codepoint dispatch lands: a single `draw_codepoint` entry point now
routes any sprite codepoint to its drawing family — the dispatch upstream's
sprite `Face` uses. With it, every codepoint-keyed sprite glyph (box drawing,
braille, the legacy-computing families, the geometric triangles, and the entire
powerline block) is reachable through one call.

The remaining sprite-font work is: a non-rendering **`has_codepoint`** predicate
(the codepoint-range classification, so the collection's sprite-coverage check
does not have to render); the **sprite-kind special glyphs** (the
underlines/strikethrough/overline/cursors, keyed by a `Sprite` enum rather than
a codepoint — they need a separate dispatch and the cursor/underline-style
plumbing); and the **resolver/atlas wiring** (a sprite `Face`-equivalent that
sizes a `Canvas`, calls `draw_codepoint`, and writes to the atlas, filling the
resolver's deferred `SpriteUnavailable` arm). After the sprite font: the
discovery consumer, the UCD emoji-presentation default, codepoint overrides, the
shaper, the Nerd Font attribute table, and SVG color detection.

## Completion Review

Codex reviewed the completed implementation and result and found **no Required
changes** (and no Optional). It confirmed `draw_codepoint` is faithful: it
short-circuits across the complete set of 16 codepoint-keyed families, uses
`metrics.cell_width`/`cell_height` for the powerline width/height arguments, and
keeps the special sprite-kind glyphs out of this codepoint path; and that the
test coverage addresses the prior design finding (one representative per family,
fresh-canvas pixel equality against the direct dispatcher, plus the false/blank
behavior for unsupported codepoints), with valid per-family codepoints and no
shadowing given the dispatchers' disjoint match sets and false-before-draw
behavior.

Review artifacts:

- Result review: `logs/codex-review/20260603-092539-863133-last-message.md`
