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

# Experiment 312: Canvas::flip_horizontal + the outlined powerline chevrons (E0B1/E0B3)

## Description

The outlined powerline arrows `E0B1` (`>`) and `E0B3` (`<`) are stroked open
chevrons. Upstream `powerline.zig`'s `drawE0B1` strokes a two-segment open path
(the chevron) with butt caps and the light box thickness; `drawE0B3` draws the
same and then `flipHorizontal`s the canvas. This experiment ports
`Canvas::flip_horizontal` (a left-right mirror of the surface) and
`draw_powerline_chevron` (the two-glyph dispatch) over the already-wired
`Canvas::stroke_path`.

## Upstream behavior

- `drawE0B1(width, height, metrics)`: an **open** path
  `move(0, 0) → line(width, height/2) → line(0, height)` (the `>` chevron),
  `strokePath` with `line_cap = butt`,
  `line_width = Thickness.light.height(box_thickness)`, `.on`.
- `drawE0B3`: `drawE0B1(…)` then `canvas.flipHorizontal()`.
- `Canvas.flipHorizontal()`: mirror the surface columns —
  `buf[y·w + x] = clone[y·w + (w − x − 1)]` for every pixel — and swap
  `clip_left`/`clip_right`.

## Rust mapping

- `roastty/src/font/sprite/canvas.rs`:
  `pub(crate) fn flip_horizontal(&mut self)` — clone the buffer, then for each
  `(x, y)` set `self.buf[y·w + x] = clone[y·w + (w − x − 1)]` (the full padded
  surface, `w = self.width`), and
  `std::mem::swap(&mut self.clip_left, &mut self.clip_right)`.
- `roastty/src/font/sprite/draw.rs`:
  `pub(crate) fn draw_powerline_chevron(cp: u32, width: u32, height: u32, metrics: &Metrics, canvas: &mut Canvas) -> bool`
  — for `0xE0B1`/`0xE0B3`: build the open chevron node list (`move(0,0)`,
  `line(width, height/2)`, `line(0, height)`),
  `let thick = Thickness::Light.height(metrics.box_thickness) as f64`,
  `canvas.stroke_path( &nodes, thick, raster::CapMode::Butt)`; for `0xE0B3` then
  `canvas.flip_horizontal()`. `_ => false`. Update the module doc.

## Scope / faithfulness notes

- **Ported**: `Canvas::flip_horizontal` and the two outlined powerline chevrons.
- **Deferred**: the inner-stroke powerline glyphs (`E0B9`/`E0BB`/`E0BD`/`E0BF`),
  the rounded separators (`E0B4`–`E0B7`), the flames (`E0D2`/`E0D4`), and the
  sprite dispatch.
- No C ABI/header/ABI-inventory change.

## Changes

1. `roastty/src/font/sprite/canvas.rs`: add `Canvas::flip_horizontal`.
2. `roastty/src/font/sprite/draw.rs`: add `draw_powerline_chevron`; update the
   module doc.
3. Tests (deterministic — the fixture `9×18` cell, `box_thickness 2` → light
   thickness 2; the chevron point at `(9, 9)`):
   - `powerline_e0b1_chevron`: `E0B1` strokes the `>` outline — a point near the
     right tip (`(8, 9)`) is inked, the chevron **interior** (`(4, 9)`, between
     the two arms) is empty, and the chevron is in the right half (the left edge
     `(0, 9)` is empty — the arms start at the corners `(0,0)`/`(0,18)`, not the
     mid-left).
   - `powerline_e0b3_flipped`: `E0B3` is `E0B1` mirrored — the tip is now near
     the **left** (`(0, 9)` inked) and the right-mid (`(8, 9)`) is empty;
     equivalently, `E0B3`'s buffer equals `E0B1`'s column-mirrored.
   - `flip_horizontal_mirrors`: a **padded** `Canvas` (so the mirror is tested
     on the padded surface) with an asymmetric mark (`pixel(0, 0)` set) mirrors
     the mark to the opposite cell column after `flip_horizontal()` (the mark
     moves to `(width − 1, 0)`).
   - (The exact pixels are confirmed against the render during implementation.)
4. Format and test (`cargo fmt`, accept output).

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

- `Canvas::flip_horizontal` mirrors the surface columns (and swaps the clip
  margins), and `draw_powerline_chevron` strokes the `>` chevron for `E0B1` and
  the flipped `<` for `E0B3`, returning `false` otherwise;
- the chevron, flip, and mirror tests confirm the rendering;
- the inner-stroke/rounded/flame powerline glyphs and the sprite dispatch stay
  deferred;
- `cargo fmt` accepted, `cargo build -p roastty` has no warnings, and
  `cargo test -p roastty` passes with no regressions;
- the no-`ghostty`-name gates and `git diff --check` pass;
- Codex reviews the design before implementation and the result after, with all
  real findings fixed.

The experiment is **partial** if the chevron needs join/cap handling the
butt-cap open stroke does not cover.

The experiment **fails** if the chevron geometry, the flip, or the
`Canvas::stroke_path` wiring diverges from z2d, or any public C API/ABI changes.

## Design Review

Codex reviewed this design before implementation and found **no Required
changes**. It confirmed: `Canvas::flip_horizontal` mirrors the full padded alpha
surface and swaps the left/right clip margins, matching upstream; `E0B1` uses
the explicit glyph `width`/`height`, the light box thickness from metrics, butt
caps, and the default miter join through `Canvas::stroke_path`; `E0B3` as "draw
E0B1, then flip" matches upstream exactly (flipping the whole canvas); the
chevron tests target the right geometry (tip side inked, center interior empty,
opposite mid-edge empty, plus a direct mirror test); and deferring the remaining
powerline glyphs and the dispatch is cleanly scoped. One **Optional** suggestion
— make `flip_horizontal_mirrors` use a padded canvas (since the mirror is on the
padded surface) — folded into the test plan.

Review artifacts:

- Prompt: `logs/codex-review/20260603-090217-384765-prompt.md`
- Result: `logs/codex-review/20260603-090217-384765-last-message.md`

## Result

**Result:** Pass

**Note on `Canvas::flip_horizontal`:** the design planned to add it, but it was
**already present** in `roastty/src/font/sprite/canvas.rs` (ported in an earlier
session, matching upstream exactly: clone the buffer, mirror the columns
`buf[y·w + x] = clone[y·w + (w − x − 1)]`, swap `clip_left`/`clip_right`). So
this experiment added only `draw_powerline_chevron` and reused the existing
flip.

`roastty/src/font/sprite/draw.rs` gained
`draw_powerline_chevron(cp, width, height, metrics, canvas)`: for `E0B1` it
strokes the open `>` chevron path (`move(0,0)`, `line(width, height/2)`,
`line(0, height)`) with `thick = Thickness::Light.height(box_thickness)` and
`CapMode::Butt` (the default miter join); `E0B3` does the same then
`canvas.flip_horizontal()`; `_ => false`.

Tests (the fixture `9×18` cell), confirmed against the render:

- `powerline_e0b1_chevron` — the `>` outline: tip `(8,9)` inked, interior
  `(4,9)` empty (hollow), mid-left `(0,9)` empty.
- `powerline_e0b3_flipped` — the mirrored `<`: left tip `(0,9)` inked, mid-right
  `(8,9)` empty.
- `flip_horizontal_mirrors` — on a padded canvas, a mark at cell `(0,0)` mirrors
  to `(8,0)` after `flip_horizontal()`.
- `draw_powerline_chevron_excludes` — `0x2500`, `0xE0B0`, `'M'` return `false`
  and draw nothing.

Gate results:

- `cargo fmt -p roastty` accepted; `--check` clean.
- `cargo test -p roastty` → 2646 passed, 0 failed (+4, no regressions).
- `cargo build -p roastty` → no warnings.
- No-`ghostty`-name gates clean; `git diff --check` clean.

## Conclusion

The outlined powerline chevrons render faithfully — the first stroked (vs
filled) powerline glyphs, and the first consumers of the (pre-existing)
`Canvas::flip_horizontal`. The powerline family now covers the six solid
triangles (Experiment 311) and the two outlined chevrons.

The remaining powerline glyphs are the **inner-stroke** arrows (`E0B9`/`E0BB`/
`E0BD`/`E0BF` — `Canvas::inner_stroke_path`), the **rounded** separators
(`E0B4`–`E0B7` — half-discs via the `arc` primitive + `flip`), and the
**flames** (`E0D2`/`E0D4`). The larger remaining integration is the unifying
sprite `has_codepoint`/draw and **sprite-kind dispatch** (mapping the codepoint
tables and a `Sprite` enum to all the standalone `draw_*` functions, filling the
resolver's deferred `SpriteUnavailable` arm). After the sprite font: the
discovery consumer, the UCD emoji-presentation default, codepoint overrides, the
shaper, the Nerd Font attribute table, and SVG color detection.

## Completion Review

Codex reviewed the completed implementation and result and found **no Required
changes**. It confirmed `draw_powerline_chevron` matches upstream for both
glyphs: the explicit glyph dimensions, the light box thickness, the open
two-segment path, butt caps, the default miter join, and `E0B3` as `E0B1`
followed by `flip_horizontal`; and that the padded flip test covers the
surface-mirror behavior. No Optional findings.

Review artifacts:

- Result review: `logs/codex-review/20260603-090521-638531-last-message.md`
