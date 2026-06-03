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

# Experiment 366: addGlyph — a shaped glyph into a render cell

## Description

The shaping path produces `ShapedRun`s (Experiments 358–362) and `SharedGrid`
rasterizes each glyph index into an atlas, returning a
`Render { glyph, presentation }` (Experiments 363–365). The renderer's
`Contents` builder (`renderer/cell.rs`) holds the per-row `CellTextVertex` lists
the GPU draws. The missing link is **`addGlyph`**: render one shaped glyph
through the grid and emit a `CellTextVertex` into `Contents` — choosing the
atlas from the presentation, placing the glyph at its grid cell, and combining
the glyph and shaper bearings. This is upstream's `renderer/generic.zig`
`addGlyph` (its emit half).

## Upstream behavior

`addGlyph` (`renderer/generic.zig`):

1. renders the glyph:
   `render = font_grid.renderGlyph(run.font_index, shaper_cell.glyph_index, opts)`;
2. **skips invisible glyphs**: if `render.glyph.width == 0` or `height == 0`,
   returns without adding (a zero glyph, e.g. a space, draws nothing);
3. emits a GPU cell via `self.cells.add(.text, .{ … })`:
   - `atlas`: `render.presentation` → `.emoji ⇒ color`, `.text ⇒ grayscale`;
   - `grid_pos`: `(x, y)`;
   - `color`: `(color.r, color.g, color.b, alpha)`;
   - `glyph_pos`: `(render.glyph.atlas_x, render.glyph.atlas_y)`;
   - `glyph_size`: `(render.glyph.width, render.glyph.height)`;
   - `bearings`:
     `(render.glyph.offset_x + shaper_cell.x_offset, render.glyph.offset_y + shaper_cell.y_offset)`;
   - `bools.no_min_contrast`: from the codepoint.

Upstream additionally builds the `RenderOptions` inside `addGlyph` (from the
config's `thicken`, the terminal cell's `gridWidth`, `getConstraint`/`isSymbol`,
and `constraintWidth`) and derives `color`/`alpha`/`no_min_contrast` from the
terminal cell. Those derivations need the terminal cell slice and the renderer
config — they belong to the not-yet-ported `rebuildCells` loop. This experiment
ports the **emit half**: given the resolved inputs, render and add the cell.

## Rust mapping (`roastty/src/renderer/cell.rs`)

`addGlyph` lives in `renderer/generic.zig` upstream; roastty has no generic
renderer yet, so it is co-located with `Contents` in `renderer/cell.rs` (which
already owns `Contents`, the `Key`/`CellTextVertex` types, and the codepoint
predicates). `rebuildCells` — the loop that derives `opts`/`color`/
`no_min_contrast` per cell and calls this — is a later experiment.

```rust
use crate::font::codepoint_resolver::ResolverRenderError;
use crate::font::collection::Index;
use crate::font::face::coretext::RenderOptions;
use crate::font::shape;
use crate::font::shared_grid::SharedGrid;
use crate::font::Presentation;
use super::shader::{CellTextAtlas, CellTextFlags};

/// Render one shaped glyph through `grid` and add it to `contents` as a text
/// `CellTextVertex` at `grid_pos`. Invisible glyphs (0 width/height) are skipped.
/// Faithful port of the emit half of upstream `addGlyph`: the atlas comes from
/// the render's presentation, and the bearings sum the glyph's own bearings and
/// the shaper cell's per-glyph offsets. (`opts`, `color`/`alpha`, and
/// `no_min_contrast` are derived by the caller — the future `rebuildCells`.)
pub(crate) fn add_glyph(
    contents: &mut Contents,
    grid: &mut SharedGrid,
    grid_pos: [u16; 2],
    font_index: Index,
    shaper_cell: &shape::Cell,
    color: [u8; 3],
    alpha: u8,
    no_min_contrast: bool,
    opts: &RenderOptions,
) -> Result<(), ResolverRenderError> {
    let render = grid.render_glyph(font_index, shaper_cell.glyph_index, opts)?;

    // A 0-size glyph (e.g. a space) is invisible — don't add it to the buffer.
    if render.glyph.width == 0 || render.glyph.height == 0 {
        return Ok(());
    }

    // The glyph's own bearings plus the shaper's per-glyph offsets.
    let bearings = [
        i16::try_from(render.glyph.offset_x + i32::from(shaper_cell.x_offset))
            .expect("glyph x bearing fits i16"),
        i16::try_from(render.glyph.offset_y + i32::from(shaper_cell.y_offset))
            .expect("glyph y bearing fits i16"),
    ];

    contents.add(
        Key::Text,
        CellTextVertex {
            glyph_pos: [render.glyph.atlas_x, render.glyph.atlas_y],
            glyph_size: [render.glyph.width, render.glyph.height],
            bearings,
            grid_pos,
            color: [color[0], color[1], color[2], alpha],
            atlas: match render.presentation {
                Presentation::Emoji => CellTextAtlas::Color,
                Presentation::Text => CellTextAtlas::Grayscale,
            },
            flags: CellTextFlags::new(no_min_contrast, false),
            _padding: [0, 0],
        },
    );
    Ok(())
}
```

## Scope / faithfulness notes

- **Ported (bridged)**: the emit half of upstream `addGlyph` — render the shaped
  glyph through the `SharedGrid`, skip invisible (0-size) glyphs, and add a
  `CellTextVertex` to `Contents` with the atlas (from presentation), grid
  position, color, atlas placement/size, and combined bearings.
- **Faithful**: `atlas` is `Emoji → Color`, `Text → Grayscale` (from
  `render.presentation`); `bearings` sum the glyph bearings and the shaper
  cell's `x_offset`/`y_offset`, as upstream; the 0-size skip matches upstream;
  the `CellTextVertex` field mapping is one-to-one with upstream's
  `cells.add(.text, …)`.
- **Faithful adaptation**: the bearings use a checked
  `i16::try_from(...).expect` (upstream's `@intCast`, which asserts on
  overflow); `is_cursor_glyph` is `false` (this is a text-run glyph, not the
  cursor glyph). The function takes the already-derived
  `opts`/`color`/`alpha`/`no_min_contrast` because deriving them needs the
  terminal cell slice and the renderer config — the `rebuildCells` loop's job,
  ported later. `add_glyph` is placed in `renderer/cell.rs` (with `Contents`)
  rather than a generic-renderer module that does not exist yet.
- **Deferred**: the `rebuildCells` loop (iterate the viewport's `ShapedRun`s,
  derive per-cell `opts`/`color`/`no_min_contrast`, and call `add_glyph`), the
  background/decoration cells, the cursor glyph, and the Metal upload of the
  `Contents`. (Consumed by tests now.)
- No C ABI/header/ABI-inventory change (internal Rust).

## Changes

1. `roastty/src/renderer/cell.rs`: add the `add_glyph` function; import the font
   types (`Index`, `RenderOptions`, `shape`, `SharedGrid`, `Presentation`,
   `ResolverRenderError`) and the shader types (`CellTextAtlas`,
   `CellTextFlags`). Update the module doc to note the `addGlyph` bridge.
2. Tests (in `cell.rs`): with a Menlo `SharedGrid` and a `Contents`:
   - render `'M'` (with non-zero shaper `x_offset`/`y_offset`) at
     `grid_pos [2, 1]`; assert one cell lands in `fg_rows[2]`, with
     `grid_pos == [2, 1]`, `atlas == Grayscale`, `color == [r, g, b, alpha]`,
     and `glyph_pos`/ `glyph_size`/`bearings` matching a direct `render_glyph`
     (bearings include the shaper offsets);
   - render a space (0-size glyph); assert no cell is added.
3. Format and test (`cargo fmt`, accept output).

## Verification

```bash
cargo fmt
cargo fmt -- --check
cargo test -p roastty add_glyph
cargo test -p roastty
cargo build -p roastty            # no warnings
rg -n 'ghostty|Ghostty|GHOSTTY' roastty/src/font roastty/src/renderer && exit 1 || true
rg -n 'ghostty|Ghostty|GHOSTTY' roastty/src/lib.rs roastty/include/roastty.h roastty/tests/abi_harness.c && exit 1 || true
git diff --check
```

The experiment **passes** if:

- `add_glyph` renders a shaped glyph through the grid, skips 0-size glyphs, and
  adds a `CellTextVertex` with the correct atlas, grid position, color, atlas
  placement/size, and combined bearings — faithful to upstream `addGlyph`'s
  emit;
- the tests pass (a visible glyph adds one correctly-built cell; a space adds
  none), and the existing tests still pass;
- the `rebuildCells` loop, decorations, cursor, and Metal upload stay deferred;
- `cargo fmt` accepted, `cargo build -p roastty` has no warnings, and
  `cargo test -p roastty` passes with no regressions;
- the no-`ghostty`-name gates and `git diff --check` pass;
- Codex reviews the design before implementation and the result after, with all
  real findings fixed.

The experiment **fails** if the atlas/grid position/bearings are wrong, an
invisible glyph is added (or a visible one skipped), or any public C API/ABI
changes.

## Design Review

Codex reviewed this design before implementation and **approved** it with **no
findings**. It confirmed `add_glyph` is faithful to upstream's emit half (render
through `SharedGrid`, skip zero-size glyphs, choose `CellTextAtlas` from
`render.presentation`, map atlas position/size directly, combine the glyph
bearings with the `shape::Cell` offsets, pass through grid position and RGBA
color, set `no_min_contrast` while leaving `is_cursor_glyph` false); that
`i16::try_from(...).expect(...)` is an acceptable Rust equivalent of upstream's
checked `@intCast` (a real overflow would mean a pathological rasterized bearing
or shaper offset far outside terminal glyph geometry, fine to surface as an
invariant failure); that placing the function in `renderer/cell.rs` is
acceptable for now (it builds `CellTextVertex` and writes to `Contents`, and the
generic renderer loop that would own it upstream does not exist yet); that
taking pre-derived `opts`/`color`/`alpha`/`no_min_contrast` is the right
boundary (deriving them needs the terminal cells and renderer config, the later
`rebuildCells` loop's job); and that the test plan is sufficient (a visible
glyph verifies field mapping, atlas selection, row routing, color, and bearing
math against a direct render; a space verifies the zero-size skip).

Review artifacts:

- Prompt: `logs/codex-review/20260603-175632-120915-prompt.md` (design)
- Result: `logs/codex-review/20260603-175632-120915-last-message.md` (design)
