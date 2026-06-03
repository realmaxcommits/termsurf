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

# Experiment 303: the curly underline (the first round-cap glyph)

## Description

The first consumer of **round caps** (Experiment 302): the curly underline
(undercurl). Upstream `special.zig`'s `underline_curly` draws a single-cycle
sine-like wave — two cubic Béziers — peaking at the cell center, stroked with
the underline thickness and **round** line caps. This experiment ports it as a
standalone `draw_underline_curly`, the first sprite glyph that exercises both
the curve stroke (Experiment 300) and round caps end to end. (The special
sprites — underlines, strikethrough, overline, cursors — are keyed by a sprite
kind, not a Unicode codepoint; the unifying dispatch is a later experiment, so
this ports the draw function standalone, as `draw_box_arc` was.)

## Upstream behavior (`special.zig` `underline_curly`)

With `width`/`height` the glyph dimensions and `metrics` the cell metrics:

- `line_width = underline_thickness`; `cap_mode = .round`.
- `amplitude = width / π`.
- `padding = canvas.padding_y`;
  `top = min(underline_position, height + padding − amplitude − line_width)`
  (clamped so the curl is not clipped); `bottom = top + amplitude`.
- `r = 0.4` (curvature multiplier); `center = 0.5 · width`.
- the path (one wave cycle, peaking at the center):
  - `move(0, bottom)`;
  - `curve((center·r, bottom), (center − center·r, top), (center, top))` — up to
    the center peak;
  - `curve((center + center·r, top), (width − center·r, bottom), (width, bottom))`
    — back down to the right edge;
  - `stroke()` with the round caps.

## Rust mapping (`roastty/src/font/sprite/draw.rs`, `canvas.rs`)

- `roastty/src/font/sprite/canvas.rs`: add
  `pub(crate) fn padding_y(&self) -> u32` (the draw function reads it; the field
  is private to the `canvas` module).
- `roastty/src/font/sprite/draw.rs`:
  `pub(crate) fn draw_underline_curly(canvas: &mut Canvas, width: u32, height: u32, metrics: &Metrics)`
  — compute `float_width`/`float_height`/`float_pos`,
  `line_width = underline_thickness as f64`, `amplitude = float_width / π`,
  `padding = canvas.padding_y() as f64`, `top`/`bottom`, `r = 0.4`, `center`,
  build the two-cubic node list, and
  `canvas.stroke_path(&nodes, line_width, raster::CapMode::Round)`.

`std::f64::consts::PI` for π. No codepoint dispatch yet (the function is invoked
directly, like `underline_curly`'s sprite-keyed call upstream).

## Scope / faithfulness notes

- **Ported**: the curly underline draw function — the first round-cap glyph.
- **Deferred**: the other special sprites (plain/double/dotted/dashed
  underlines, strikethrough, overline, cursors), the sprite-kind dispatch, the
  closed-path stroke, and dashes.
- No C ABI/header/ABI-inventory change.

## Changes

1. `roastty/src/font/sprite/canvas.rs`: add the `padding_y` accessor.
2. `roastty/src/font/sprite/draw.rs`: add `draw_underline_curly`; note it in the
   module doc.
3. Tests (deterministic; the fixture `9×18` cell, `underline_position 15`,
   `underline_thickness 1`, an **unpadded** canvas — `amplitude = 9/π ≈ 2.86`,
   `top = min(15, 18 − 2.86 − 1) ≈ 14.14`, `bottom ≈ 17.0`, so the wave runs
   between `y ≈ 14` (the center peak) and `y ≈ 17` (the ends), confirmed against
   the actual render):
   - `underline_curly_wave`: the **center** column near the peak (`x = 4`,
     `y ≈ 14`) is inked, and the **ends** near the trough (`x = 0` and `x = 8`,
     `y ≈ 16`) are inked — the wave spans the cell.
   - `underline_curly_shape`: the wave sits in the lower band — a row well above
     the curl (`y ≤ 12`) is entirely empty, confirming it does not fill the
     cell.
   - (The exact inked rows/cols are pinned against the render during
     implementation; a padded canvas is used if the bottom trough clips.)
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

- `draw_underline_curly` reproduces z2d's `underline_curly` — the amplitude/top/
  bottom geometry, the two-cubic wave, and the round-capped stroke;
- the wave/shape tests confirm the rendering (the center peak and the trough
  ends inked, the upper cell empty);
- the other special sprites, the sprite-kind dispatch, the closed-path stroke,
  and dashes stay deferred;
- `cargo fmt` accepted, `cargo build -p roastty` has no warnings, and
  `cargo test -p roastty` passes with no regressions;
- the no-`ghostty`-name gates and `git diff --check` pass;
- Codex reviews the design before implementation and the result after, with all
  real findings fixed.

The experiment is **partial** if the curl needs clip/padding handling beyond the
upstream `top` clamp to land in the cell.

The experiment **fails** if the curl geometry or the round-cap stroke diverges
from z2d, or any public C API/ABI changes.

## Design Review

Codex reviewed this design before implementation and found **no Required
changes**. It confirmed the geometry matches `underline_curly`
(`line_width = underline_thickness`, `amplitude = width / π`,
`top = min(underline_position, height + padding_y − amplitude − line_width)`,
`bottom = top + amplitude`, `r = 0.4`, `center = 0.5 · width`, the two cubics);
that `canvas.stroke_path(…, CapMode::Round)` is the right wiring (padding CTM,
MSAA stroke, round caps, and the curve-internal round joins — with no explicit
`line_to`, the only joins are from the flattened curve); that a
`pub(crate) padding_y()` accessor is acceptable; that the standalone
`draw_underline_curly` is a sound slice with the sprite dispatch deferred; and
that the unpadded `9×18` test canvas is fine since the upstream clamp keeps the
trough drawable (switch to padded only if the render clips — but then the
expected `y` shifts because `padding_y` participates in the `top` clamp). One
**Optional** suggestion — pin the exact rendered rows/columns after
implementation, since round caps + AA can shift which adjacent row has visible
alpha — which is already the plan (the pixels are confirmed against the render).

Review artifacts:

- Prompt: `logs/codex-review/20260603-080539-599586-prompt.md`
- Result: `logs/codex-review/20260603-080539-599586-last-message.md`
