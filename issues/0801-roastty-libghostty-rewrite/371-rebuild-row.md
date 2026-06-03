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

# Experiment 371: assembling one row's text cells

## Description

Experiments 369–370 made both per-row inputs reachable from the shaping output:
`cell_infos` (a row's `CellInfo` slice) and `Style::resolve_fg` (a cell's
foreground). This experiment composes them with `add_run` (Experiment 368) into
**`rebuild_row`**: given a row's `RunCell`s and its `ShapedRun`s, it derives the
`CellInfo` slice and the per-column `fg_colors`, then places every run's glyphs
into `Contents`. This is the per-row body of upstream `rebuildCells` (the
foreground-text part). The outer loop (over the whole viewport) is the next
experiment.

## Upstream behavior

`rebuildCells` (`renderer/generic.zig`), for each viewport row, reads the row's
cells and shaped runs and emits the foreground glyphs (plus backgrounds,
decorations, cursor). The per-row foreground work is: for each shaped run of the
row, for each shaped cell, resolve the cell's color and `addGlyph`. roastty has
the pieces — `cell_infos` (the `CellInfo` view), `resolve_fg` (the foreground
color), and `add_run` (a run's glyphs) — and this experiment is the per-row
composition that wires them: derive the row's `CellInfo` slice and `fg_colors`
once, then `add_run` each run.

## Rust mapping (`roastty/src/renderer/cell.rs`)

```rust
use crate::terminal::color::{Palette, Rgb};
use crate::terminal::style::BoldColor;

/// Assemble one viewport row's foreground text cells into `contents`. Derives the
/// row's [`CellInfo`] slice ([`cell_infos`]) and per-column `fg_colors`
/// ([`Style::resolve_fg`] + `alpha`) from `row_cells`, then places every glyph of
/// each [`ShapedRun`] via [`add_run`]. The per-row foreground body of upstream
/// `rebuildCells`.
#[allow(clippy::too_many_arguments)]
pub(crate) fn rebuild_row(
    contents: &mut Contents,
    grid: &mut SharedGrid,
    y: u16,
    row_runs: &[ShapedRun],
    row_cells: &[RunCell],
    default_fg: Rgb,
    palette: &Palette,
    bold: Option<BoldColor>,
    alpha: u8,
    thicken: bool,
    thicken_strength: u8,
) -> Result<(), ResolverRenderError> {
    let cols = row_cells.len();
    let infos = cell_infos(row_cells);
    let fg_colors: Vec<[u8; 4]> = row_cells
        .iter()
        .map(|cell| {
            let rgb = cell.style.resolve_fg(default_fg, palette, bold);
            [rgb.r, rgb.g, rgb.b, alpha]
        })
        .collect();

    for run in row_runs {
        add_run(
            contents,
            grid,
            y,
            run,
            &infos,
            &fg_colors,
            cols,
            thicken,
            thicken_strength,
        )?;
    }
    Ok(())
}
```

## Scope / faithfulness notes

- **Ported (bridged)**: the per-row foreground body of upstream `rebuildCells` —
  derive the row's `CellInfo` slice and per-column `fg_colors` from its cells,
  then `add_run` each of the row's shaped runs.
- **Faithful**: `cols` is the row's column count (`row_cells.len()`, the same
  length as `infos`/`fg_colors`); each column's color is
  `style.resolve_fg(default_fg, palette, bold)` (the ported `Style::fg`); the
  runs are placed in order; the glyph placement (column math, atlas, bearings,
  zero-size skip) is `add_run`'s, unchanged.
- **Faithful adaptation**: `rebuild_row` takes the renderer's color config
  (`default_fg`, `palette`, `bold`) and a single `alpha` — the inputs the
  renderer holds. `alpha` is uniform per call (per-cell faint/dim alpha is a
  deferred renderer-layer adjustment, as in Experiment 370); it derives `infos`/
  `fg_colors` once per row (not per run), the natural place since they are
  row-wide.
- **Deferred**: the outer loop over the whole viewport (iterate
  `shape_viewport`'s rows, calling `rebuild_row` with each row's `RunCell`s);
  the renderer-layer color adjustments (reverse-video, selection, min-contrast,
  faint/dim alpha); the background/decoration/cursor cells; and the Metal
  upload. (Consumed by tests now.)
- No C ABI/header/ABI-inventory change (internal Rust).

## Changes

1. `roastty/src/renderer/cell.rs`: add the `rebuild_row` function; import
   `terminal::color::{Palette, Rgb}` and `terminal::style::BoldColor`.
2. Test (in `cell.rs`): with a Menlo `SharedGrid` and a `Contents`, build a row
   of `RunCell`s — `'A'` with the **default** style and `'B'` with a
   **non-default** foreground (`fg_color = Color::Rgb(Rgb::new(11, 22, 33))`) —
   and a matching `ShapedRun` (offset 0, glyphs at `x = 0/1`), and call
   `rebuild_row` on row `y = 1` with a distinct
   `default_fg = Rgb::new(200, 200, 200)`, `DEFAULT_PALETTE`, `bold = None`,
   `alpha = 255`:
   - assert two cells land in `fg_rows[2]` at `grid_pos [0, 1]`/`[1, 1]`, both
     `atlas == Grayscale`;
   - assert column 0's `color` is `[200, 200, 200, 255]` (the default style
     resolves to `default_fg`) **and** column 1's `color` is `[11, 22, 33, 255]`
     (its own style's color, **differing** from `default_fg`) — proving the
     per-column `fg_colors` are derived from each cell's style, not just
     `default_fg`.
3. Format and test (`cargo fmt`, accept output).

## Verification

```bash
cargo fmt
cargo fmt -- --check
cargo test -p roastty rebuild_row
cargo test -p roastty
cargo build -p roastty            # no warnings
rg -n 'ghostty|Ghostty|GHOSTTY' roastty/src/font roastty/src/renderer && exit 1 || true
rg -n 'ghostty|Ghostty|GHOSTTY' roastty/src/lib.rs roastty/include/roastty.h roastty/tests/abi_harness.c && exit 1 || true
git diff --check
```

The experiment **passes** if:

- `rebuild_row` derives the row's `CellInfo` slice and `fg_colors` from its
  cells and places every run's glyphs via `add_run` — faithful to the per-row
  foreground body of upstream `rebuildCells`;
- the test passes (a row's `"AB"` run adds two correctly-colored,
  correctly-placed cells), and the existing tests still pass;
- the outer loop, the color adjustments, the decorations/cursor, and the Metal
  upload stay deferred;
- `cargo fmt` accepted, `cargo build -p roastty` has no warnings, and
  `cargo test -p roastty` passes with no regressions;
- the no-`ghostty`-name gates and `git diff --check` pass;
- Codex reviews the design before implementation and the result after, with all
  real findings fixed.

The experiment **fails** if a row's colors or columns are mis-derived, a run is
dropped, or any public C API/ABI changes.

## Design Review

Codex reviewed this design before implementation and **approved** it with one
**Required** finding, now addressed:

- **Required (addressed):** the test used the default style for both cells, so
  it would still pass if `rebuild_row` ignored `cell.style.resolve_fg(...)` and
  filled every column with `default_fg`. Since building `fg_colors` from
  `row_cells` is the experiment's main new behavior, the test now gives `'B'` a
  **non-default** foreground (`Color::Rgb(Rgb::new(11, 22, 33))`) and asserts
  column 1's emitted color is that color, **differing** from `default_fg` —
  proving the per-column colors come from each cell's style.

Codex confirmed the rest is sound: the composition is faithful (derive
`cell_infos(row_cells)` and `fg_colors` once per row, then `add_run` each
`ShapedRun`); `cols = row_cells.len()` is consistent because `infos` and
`fg_colors` are both built from `row_cells`, so `add_run` receives matching
row-wide slices; and using base `resolve_fg` plus uniform alpha is the right
scope (inverse, selection, faint/dim alpha, and min-contrast are renderer-layer
follow-ups).

Review artifacts:

- Prompt: `logs/codex-review/20260603-182514-265914-prompt.md` (design)
- Result: `logs/codex-review/20260603-182514-265914-last-message.md` (design)

## Result

**Result:** Pass

One viewport row's foreground text now assembles end to end.

- `roastty/src/renderer/cell.rs`:
  `rebuild_row(contents, grid, y, row_runs, row_cells, default_fg, palette, bold, alpha, thicken, thicken_strength)`
  derives the row's `CellInfo` slice (`cell_infos`) and per-column `fg_colors`
  (each cell's `style.resolve_fg(default_fg, palette, bold)` + `alpha`) once,
  then calls `add_run` for each `ShapedRun` with the row-wide slices
  (`cols = row_cells.len()`). Imported `terminal::color::{Palette, Rgb}` and
  `terminal::style::BoldColor`.

Test (in `cell.rs`): `rebuild_row_derives_infos_and_colors` builds a row with
`'A'` (default style) and `'B'` (`Color::Rgb(11, 22, 33)`) and a matching
`ShapedRun`, calls `rebuild_row` with `default_fg = Rgb(200, 200, 200)`, and
asserts the two emitted cells land at `[0, 1]`/`[1, 1]` (Grayscale) with column
0's color `[200, 200, 200, 255]` (default → `default_fg`) and column 1's
`[11, 22, 33, 255]` — `assert_ne` against `default_fg` proving the per-column
colors come from each cell's style.

Gate results:

- `cargo fmt -p roastty` accepted; `--check` clean.
- `cargo test -p roastty` → 2821 passed, 0 failed (+1, no regressions).
- `cargo build -p roastty` → no warnings.
- No-`ghostty`-name gates (font + renderer) clean; `git diff --check` clean.

## Conclusion

The per-row foreground assembly is complete: a row's `RunCell`s + `ShapedRun`s
become correctly-placed, correctly-colored `CellTextVertex`es in `Contents`. The
last structural step is the **outer loop** that drives `rebuild_row` over the
whole viewport.

The remaining renderer-bridge work: the **full-viewport loop** (Experiment 372)
— iterate `shape_viewport`'s rows (each row's `ShapedRun`s) alongside each row's
`RunCell`s (from `Terminal::shape_run_options`'s `RunOptions.cells`), calling
`rebuild_row` per row — then the renderer-layer color adjustments
(reverse-video, selection, min-contrast, faint/dim alpha), the
background/decoration/cursor cells, and the Metal upload of `Contents`.

## Completion Review

Codex reviewed the completed implementation and result and **approved** with
**no findings**. It confirmed `rebuild_row` is faithful to the approved per-row
composition (derives `cols` from `row_cells.len()`, builds `cell_infos` and
per-cell `fg_colors` once, then feeds each `ShapedRun` through `add_run` with
matching row-wide slices), with base foreground resolution plus uniform alpha
the right scope (inverse, selection, faint/dim alpha, min-contrast, backgrounds,
cursor, and upload correctly deferred). It confirmed the Required test fix is
applied — `'B'` carries a non-default `Color::Rgb(11, 22, 33)` and the test
asserts its emitted column color differs from `default_fg`, proving `fg_colors`
comes from each cell's style rather than a flat default. Nothing needed to
change before the result commit.

Review artifacts:

- Result review: `logs/codex-review/20260603-182735-961437-last-message.md`
