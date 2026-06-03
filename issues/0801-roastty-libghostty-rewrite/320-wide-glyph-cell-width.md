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

# Experiment 320: the wide-glyph cell-width factoring

## Description

`render_codepoint` (Experiment 318) always sizes its canvas to a single cell
(`metrics.cell_width`). Upstream's sprite `renderGlyph` instead widens the
canvas when the glyph spans multiple cells:
`width = metrics.cell_width × opts.cell_width` (with `cell_width` `null`/`0`/`1`
meaning a single cell). This experiment ports that factoring — the last deferred
piece of the sprite `renderGlyph` signature — so a wide sprite codepoint renders
into a wide canvas, and the draw families that upstream widens consume the
widened `width`.

## Upstream behavior (`sprite/Face.zig` `renderGlyph`)

```zig
const width = switch (opts.cell_width orelse 1) {
    0, 1 => metrics.cell_width,
    else => |width| metrics.cell_width * width,
};
const height = metrics.cell_height;
...
try draw(cp, &canvas, width, height, metrics);
```

The `draw(cp, canvas, width, height, metrics)` dispatch passes the (possibly
widened) `width` to every family. Crucially, **most families ignore it**
(`_ = width;`) and draw against `metrics.cell_width`; only some use the passed
`width`. A faithful port must reproduce that split exactly.

### Which families use the passed `width` (audited against upstream)

- **Use `width`** (widen): **braille** (`draw2800_28FF` — `w`, `x_spacing`,
  `x_margin`, the right-edge assertion all derive from the passed `width`);
  **powerline** (`powerline.zig` — solid/chevron/rounded/flame all scale to the
  passed `width`). In roastty the powerline draws already take an explicit `w`
  parameter, so only the _value passed_ changes.
- **Ignore `width`** (`_ = width;`, draw against `metrics.cell_width`): **box
  drawing** (`draw2500_257F`), **block** (`draw2580_259F`), **geometric shapes**
  (the corner triangles `draw25E2_25E5`/`draw25F8_25FA`/`draw25FF`), and the
  **sextant/octant/separated-quadrant** legacy-computing ranges (`draw1FB00_…`
  etc. are `_ = width;`). These stay on `metrics.cell_width` — unchanged.

So on the codepoint dispatch the only metrics-keyed family that must begin
consuming the widened width is **braille**; **powerline** already takes the
width and only needs the widened value threaded to it.

## Rust mapping

- `roastty/src/font/face/coretext.rs` `RenderOptions`: un-defer the `cell_width`
  field — add `pub cell_width: Option<u8>` (the number of cells the glyph spans;
  `None`/`Some(0)`/`Some(1)` ⇒ a single cell). Update the doc comment (it no
  longer lists `cell_width` as deferred; `thicken`/`thicken_strength` stay
  deferred-by-branch but already exist as fields). Set it at every construction
  site (the test helpers default it to `None`).
- `roastty/src/font/sprite/mod.rs` `render_codepoint`: add a
  `cell_width: Option<u8>` parameter. Compute
  `let width = match cell_width { None | Some(0) | Some(1) => metrics.cell_width, Some(n) => metrics.cell_width.saturating_mul(n as u32) };`
  size the `Canvas` to that `width` (height stays `metrics.cell_height`),
  `padding_x = width / 4`, and pass `width` into `draw_codepoint`.
- `roastty/src/font/sprite/draw.rs` `draw_codepoint`: add a `width: u32`
  parameter (the canvas content width). Pass `width` (not `metrics.cell_width`)
  to the widening families — `draw_braille(cp, width, metrics, canvas)` and the
  powerline draws (`draw_powerline_triangle(cp, width, h, canvas)` etc.). The
  ignoring families (`draw_box_*`, `draw_block`, `draw_sextant`, `draw_octant`,
  `draw_separated_quadrant`, `draw_corner_triangle*`) keep taking `metrics` and
  reading `metrics.cell_width` — unchanged, matching upstream's `_ = width;`.
- `roastty/src/font/sprite/draw.rs` `draw_braille`: change its signature to
  `draw_braille(cp: u32, width: u32, metrics: &Metrics, canvas: &mut Canvas)`
  and replace the horizontal `metrics.cell_width` reads (the local `width`, `w`,
  `x_spacing`, `x_margin`, the right-edge assertion) with the passed `width`.
  Note `w = min(width / 4, height / 8)` derives the **dot size** from the
  widened width, and `w` also feeds the **vertical** dot placement (`4 * w`, the
  `y_px_left` computation) — that is faithful: the dot size is shared, so `w`
  comes from the widened width while `height`, `y_spacing`, and `y_margin`
  continue to come from `metrics.cell_height`. The `0x2800..=0x28FF` range gate
  is unchanged.
- `draw_codepoint`'s other callers thread `width = metrics.cell_width`:
  `has_codepoint` (the scratch coverage render is single-cell) and the existing
  `draw_codepoint` tests.
- `roastty/src/font/codepoint_resolver.rs` `render_glyph`: pass
  `opts.cell_width` into
  `render_codepoint(glyph_index, m, opts.cell_width, atlas)`.

## Scope / faithfulness notes

- **Ported**: the wide-glyph `cell_width` factoring of sprite `renderGlyph` —
  the canvas widening and the per-family width/`metrics.cell_width` split,
  exactly as upstream (braille + powerline widen;
  box/block/geometric/sextant/octant ignore the width). The sprite `renderGlyph`
  signature is now complete.
- **Deferred**: the sprite-kind special glyphs (underlines/cursors via the
  `Sprite` enum), a range-only `has_codepoint` fast path, and the collection's
  own sprite coverage.
- No C ABI/header/ABI-inventory change (`RenderOptions`/`Glyph`/`Atlas` are
  internal Rust).

## Changes

1. `roastty/src/font/face/coretext.rs`: add `cell_width: Option<u8>` to
   `RenderOptions`; update its doc; set it (`None`) at every construction site.
2. `roastty/src/font/sprite/mod.rs`: `render_codepoint` gains the `cell_width`
   parameter and the canvas-widening computation; update its doc.
3. `roastty/src/font/sprite/draw.rs`: `draw_codepoint` gains the `width`
   parameter and threads it to braille + powerline; `draw_braille` gains the
   `width` parameter and uses it for the horizontal geometry; `has_codepoint`
   and the `draw_codepoint` tests pass `metrics.cell_width`.
4. `roastty/src/font/codepoint_resolver.rs`: `render_glyph` passes
   `opts.cell_width` into `render_codepoint`.
5. Tests:
   - `render_codepoint_wide`: `render_codepoint(0x28FF, &m, Some(2), atlas)` (an
     **inked** braille glyph — all eight dots — which widens) yields a `Glyph`
     laid out across `2 × cell_width`; assert its trimmed `width` exceeds the
     single-cell (`Some(1)`) `0x28FF` render's trimmed width (the dots spread
     wider) — proving the widened canvas and the braille width threading.
     (`0x2800` is blank and trims to width `0` in both, so it cannot prove
     widening — an inked pattern is required.)
   - `render_codepoint_wide_box_unchanged`:
     `render_codepoint(0x2500, &m, Some(2), atlas)` (box drawing — ignores
     width, uses `metrics.cell_width`) produces the **same trimmed glyph
     geometry** as the `Some(1)` render: assert
     `wide.width == single.width && wide.height == single.height`. A box glyph
     that ignores the cell-width factor must **not** span the widened canvas —
     it stays one cell wide (the offsets may differ because
     `padding_x = width / 4` grows with the canvas, so assert only the trimmed
     extent, not the bearings) — proving the ignoring families stay on
     `metrics.cell_width`.
   - `render_codepoint_single_is_default`: `Some(1)`, `Some(0)`, and `None` all
     produce the identical single-cell `Glyph` (the `orelse 1` / `0,1` arm).
   - `draw_braille_wide`: a direct
     `draw_braille(0x28FF, 2 * m.cell_width, &m, canvas)` (all eight dots) on a
     `2×`-wide canvas places ink in the right half of the canvas
     (`x ≥ cell_width`), which a single-cell render does not — proving the
     braille horizontal geometry follows the passed width.
   - Existing `render_codepoint_*` and `has_codepoint`/`draw_codepoint` tests
     updated for the new parameter (passing `None`/`metrics.cell_width`).
6. Format and test (`cargo fmt`, accept output).

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

- `render_codepoint` widens the canvas to `metrics.cell_width × cell_width`
  (with `None`/`0`/`1` ⇒ single cell), and `draw_codepoint` threads the widened
  `width` to braille and powerline while box/block/geometric/sextant/octant keep
  `metrics.cell_width` — matching upstream's per-family `_ = width;` split;
- the wide-braille, wide-box-unchanged, single-default, and direct
  `draw_braille_wide` tests confirm both the widening and the non-widening
  families;
- the resolver passes `opts.cell_width` through;
- `cargo fmt` accepted, `cargo build -p roastty` has no warnings, and
  `cargo test -p roastty` passes with no regressions;
- the no-`ghostty`-name gates and `git diff --check` pass;
- Codex reviews the design before implementation and the result after, with all
  real findings fixed.

The experiment is **partial** if a widening family beyond braille/powerline is
discovered on the codepoint path that the audit missed.

The experiment **fails** if the canvas widening or the per-family width split
diverges from upstream, or any public C API/ABI changes.

## Design Review

Codex reviewed this design before implementation and raised two **Required**
findings, both in the test plan:

1. `render_codepoint_wide` used `0x2800`, the **blank** braille pattern, whose
   trimmed width is `0` in both the single-cell and wide renders — it cannot
   prove widening. Fixed: the test now uses `0x28FF` (an inked, all-eight-dots
   pattern), whose trimmed width grows with the widened canvas.
2. `render_codepoint_wide_box_unchanged` wrongly expected the box line to "span
   the canvas". A box glyph ignores the cell-width factor (upstream
   `_ = width;`), so a widened canvas must still produce the **same one-cell
   trimmed geometry** as `Some(1)`. Fixed: the test now asserts
   `wide.width == single.width && wide.height == single.height` (trimmed extent
   only — the bearings differ because `padding_x = width / 4` grows with the
   canvas).

Codex confirmed the per-family audit is otherwise correct and complete for the
ported codepoint dispatch: braille and the non-diagonal powerline families
consume the passed render width; box/block/geometric/sextant/octant/
separated-quadrant stay on `metrics.cell_width`. It confirmed the braille
substitution is faithful with one clarification (now folded into the Rust
mapping): the dot size `w = min(width / 4, height / 8)` must be computed from
the **widened** width, and because `w` also feeds the vertical dot placement,
that shared `w` is correct — while `height`, `y_spacing`, and `y_margin`
continue to come from `metrics.cell_height`. No Optional findings.

Review artifacts:

- Prompt: `logs/codex-review/20260603-102320-874525-prompt.md`
- Result: `logs/codex-review/20260603-102320-874525-last-message.md`
