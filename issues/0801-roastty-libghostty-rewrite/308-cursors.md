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

# Experiment 308: the cursor sprites (block, hollow, bar, underline)

## Description

The cursor special sprites — `cursor_rect` (block), `cursor_hollow_rect`
(outline), `cursor_bar` (vertical bar), and `cursor_underline` (underline bar) —
complete the rect-based special-sprite family (Experiment 307). Each is a plain
`canvas.rect` shape (the hollow one punches an `.off` interior); the bar shifts
left over the cell edge and the underline reuses the underline clamp. This
experiment ports the four as standalone draw functions (the special sprites are
sprite-kind-keyed; the dispatch is a later experiment).

## Upstream behavior (`special.zig`)

With `width`/`height` the glyph dimensions and `metrics` the cell metrics:

- `cursor_rect`: a full-cell rect `(0, 0, width, height)`, `.on`.
- `cursor_hollow_rect`: fill the full cell `.on`, then punch an `.off` inner
  rect at `(cursor_thickness, cursor_thickness)` of size
  `(width -| 2·thickness, height -| 2·thickness)` — leaving a hollow box
  outline.
- `cursor_bar`: a vertical bar at `x = -((cursor_thickness + 1) / 2)`, `y = 0`,
  width `cursor_thickness`, height `height`, `.on` — shifted half a thickness
  (rounded up) over the left cell edge so it sits centered between cells.
- `cursor_underline`:
  `y = min(underline_position, (height + padding_y) -| underline_thickness)`
  (the underline clamp); a full-width rect at `y`, height `cursor_thickness`,
  `.on`.

## Rust mapping (`roastty/src/font/sprite/draw.rs`)

Four `pub(crate)` functions, each
`(canvas: &mut Canvas, width: u32, height: u32, metrics: &Metrics)`, using
`canvas.rect(Rect { x, y, width, height }, color)` (`Rect`/`Color` already
imported):

- `draw_cursor_rect`:
  `rect(Rect { x: 0, y: 0, width: width as i32, height: height as i32 }, Color::ON)`.
- `draw_cursor_hollow_rect`: the full `.on` rect, then
  `rect(Rect { x: thick as i32, y: thick as i32, width: width.saturating_sub(thick.saturating_mul(2)) as i32, height: height.saturating_sub(thick.saturating_mul(2)) as i32 }, Color::OFF)`
  where `thick = metrics.cursor_thickness`.
- `draw_cursor_bar`:
  `rect(Rect { x: -(((metrics.cursor_thickness + 1) / 2) as i32), y: 0, width: metrics.cursor_thickness as i32, height: height as i32 }, Color::ON)`
  (the negative `x` shifts the bar into the left padding; clipped by `pixel()`).
- `draw_cursor_underline`:
  `let limit = height.saturating_add(canvas.padding_y()).saturating_sub(metrics.underline_thickness); let y = metrics.underline_position.min(limit);`
  then
  `rect(Rect { x: 0, y: y as i32, width: width as i32, height: metrics.cursor_thickness as i32 }, Color::ON)`.

(`cursor_thickness`/`underline_*` are `u32`. The `u32` clamps saturate; the
bar's `+ 1` matches upstream's plain `+`. `Color::OFF` is the transparent
source.)

## Scope / faithfulness notes

- **Ported**: the four cursor sprites.
- **Deferred**: the dotted/dashed underlines (need the dash stroke) and the
  sprite-kind dispatch.
- No C ABI/header/ABI-inventory change.

## Changes

1. `roastty/src/font/sprite/draw.rs`: add `draw_cursor_rect`,
   `draw_cursor_hollow_rect`, `draw_cursor_bar`, `draw_cursor_underline`; note
   them in the module doc.
2. Tests (deterministic — the fixture `9×18` cell, `cursor_thickness 1`,
   `underline_position 15`):
   - `cursor_rect_full`: `draw_cursor_rect` inks the entire cell (every pixel).
   - `cursor_hollow_border`: `draw_cursor_hollow_rect` inks the border (the four
     edges) but leaves the interior empty (the `.off` punch).
   - `cursor_bar_left`: on a canvas with `padding_x ≥ 1`, `draw_cursor_bar` inks
     a one-column bar shifted left over the cell edge (cell `x = -1`), with the
     cell columns `x = 0` and `x ≥ 1` empty (pinning the full left shift — per
     the design review).
   - `cursor_underline_row`: `draw_cursor_underline` inks the full width at
     `y = 15` (the clamped underline position), `cursor_thickness` tall.
   - `cursor_underline_clamp`: a large `underline_position` clamps the cursor
     underline to the saturating limit instead of drawing off the bottom (per
     the design review).
   - (The exact pixels are confirmed against the render during implementation.)
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

- the four functions reproduce z2d's `cursor_rect`/`cursor_hollow_rect`/
  `cursor_bar`/`cursor_underline` (the block, the punched hollow, the shifted
  bar, the clamped underline bar);
- the cursor tests confirm the rendering;
- the dotted/dashed underlines and the sprite dispatch stay deferred;
- `cargo fmt` accepted, `cargo build -p roastty` has no warnings, and
  `cargo test -p roastty` passes with no regressions;
- the no-`ghostty`-name gates and `git diff --check` pass;
- Codex reviews the design before implementation and the result after, with all
  real findings fixed.

The experiment is **partial** if a cursor needs position/clip handling the
fixture does not exercise.

The experiment **fails** if a cursor's geometry diverges from z2d, or any public
C API/ABI changes.

## Design Review

Codex reviewed this design before implementation and found **no Required
changes**. It confirmed: `cursor_rect` is the full-cell `.on` rect;
`cursor_hollow_rect` fills `.on` then punches the inner `.off` rect (direct
`pixel()` writes make `Color::OFF` a real transparent punch, not alpha
blending); `cursor_bar` matches the upstream left shift (with
`cursor_thickness = 1`, `x = -1`/width `1` lands entirely in the left padding —
the intended faithful behavior); and `cursor_underline` uses the underline clamp
(on `underline_thickness`) then draws height `cursor_thickness` (upstream's
slightly mixed metrics). Two **Optional** suggestions, both folded in: assert
`x = 0` is empty in the bar test (pinning the full left shift), and add a
`cursor_underline` clamp test.

Review artifacts:

- Prompt: `logs/codex-review/20260603-083740-655197-prompt.md`
- Result: `logs/codex-review/20260603-083740-655197-last-message.md`

## Result

**Result:** Pass

`roastty/src/font/sprite/draw.rs` gained the four cursor sprites:
`draw_cursor_rect` (the full-cell `.on` block), `draw_cursor_hollow_rect` (the
`.on` block with an `.off` inner punch at `(thick, thick)` of size
`width -| 2·thick × height -| 2·thick`), `draw_cursor_bar` (a vertical bar at
`x = -((cursor_thickness + 1) / 2)`, width `cursor_thickness`, full height), and
`draw_cursor_underline` (a full-width bar at the clamped underline position,
`cursor_thickness` tall).

Tests (the fixture `9×18` cell, `cursor_thickness 1`):

- `cursor_rect_full` — every cell pixel inked.
- `cursor_hollow_border` — the four edges inked, the interior (and `(1,1)`)
  empty (the `.off` punch).
- `cursor_bar_left` — on a `padding_x = 1` canvas, the 1px bar sits at cell
  `x = -1` (over the left edge), with cols `0` and `4` empty.
- `cursor_underline_row` — the underline bar at `y = 15`.
- `cursor_underline_clamp` — a large `underline_position` clamps to row 17.

Gate results:

- `cargo fmt -p roastty` accepted; `--check` clean.
- `cargo test -p roastty` → 2632 passed, 0 failed (+5, no regressions).
- `cargo build -p roastty` → no warnings.
- No-`ghostty`-name gates clean; `git diff --check` clean.

## Conclusion

The cursor sprites render faithfully, completing the rect-based special-sprite
family. The sprite font's special sprites now cover the underlines (plain,
double, curly), strikethrough, overline, and the four cursors — only the
dotted/dashed underlines (which need the dash stroke) remain.

The remaining sprite-font work is the **dotted/dashed** decorations (the dash
stroke — z2d's `dashed_plotter`, the one deferred stroke feature) and then the
unifying sprite `has_codepoint`/draw and **sprite-kind dispatch** (mapping a
`Sprite` enum and the box/braille/etc. codepoint tables to all the standalone
`draw_*` functions, filling the resolver's deferred `SpriteUnavailable` arm).
After the sprite font: the discovery consumer, the UCD emoji-presentation
default, codepoint overrides, the shaper, the Nerd Font attribute table, and SVG
color detection.

## Completion Review

Codex reviewed the completed implementation and result and found **no Required
changes**. It confirmed the cursor implementations are faithful to upstream: the
full block, the hollow punch with `OFF`, the left-shifted bar, and the underline
cursor using the underline clamp but the `cursor_thickness` height; and that the
tests cover the important geometry, including the 1px bar living at cell
`x = -1` and the underline clamp. No Optional findings.

Review artifacts:

- Result review: `logs/codex-review/20260603-084004-824921-last-message.md`
