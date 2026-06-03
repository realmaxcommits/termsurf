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

# Experiment 372: rebuilding the whole viewport

## Description

`rebuild_row` (Experiment 371) assembles one row's foreground text. This
experiment adds the **outer loop**, `rebuild_viewport`: given the viewport's
per-row `RunOptions` (from `Terminal::shape_run_options`), it shapes each row
(`shape_row` over the grid's resolver) and calls `rebuild_row` with the row's
cells — filling `Contents` for the whole screen. This closes the text-rendering
path end to end: from a live terminal screen's `RunOptions` to a fully-populated
foreground GPU cell buffer.

## Upstream behavior

`rebuildCells` (`renderer/generic.zig`) walks every viewport row: it runs the
shaper over the row and, per shaped cell, resolves color and `addGlyph`. roastty
already has the per-row body (`rebuild_row`) and the per-row shaper driver
(`shape_row`). This experiment is the row loop that drives them: for each row's
`RunOptions`, shape it into `ShapedRun`s and rebuild the row. The cursor,
backgrounds, and decorations (the other parts of upstream `rebuildCells`) remain
separate.

## Rust mapping (`roastty/src/renderer/cell.rs`)

```rust
use crate::font::run::{shape_row, RunOptions};

/// Rebuild every viewport row's foreground text into `contents` from the
/// viewport's per-row [`RunOptions`] (from `Terminal::shape_run_options`). For
/// each row, shape it into [`ShapedRun`]s ([`shape_row`] over the grid's resolver)
/// and assemble it ([`rebuild_row`]) with the row's cells. The row loop of upstream
/// `rebuildCells` (foreground text).
#[allow(clippy::too_many_arguments)]
pub(crate) fn rebuild_viewport(
    contents: &mut Contents,
    grid: &mut SharedGrid,
    rows: &[RunOptions],
    default_fg: Rgb,
    palette: &Palette,
    bold: Option<BoldColor>,
    alpha: u8,
    thicken: bool,
    thicken_strength: u8,
) -> Result<(), ResolverRenderError> {
    for (y, opts) in rows.iter().enumerate() {
        let y = u16::try_from(y).expect("viewport row fits u16");
        // Shape the row first (this borrows the grid's resolver), then assemble
        // it (this borrows the grid). The owned `runs` releases the resolver
        // borrow before `rebuild_row` runs.
        let runs = shape_row(opts, &mut grid.resolver);
        rebuild_row(
            contents,
            grid,
            y,
            &runs,
            &opts.cells,
            default_fg,
            palette,
            bold,
            alpha,
            thicken,
            thicken_strength,
        )?;
    }
    Ok(())
}
```

## Scope / faithfulness notes

- **Ported (bridged)**: the viewport row loop of upstream `rebuildCells`
  (foreground text) — for each row's `RunOptions`, shape it and assemble it into
  `Contents`, in row order.
- **Faithful**: row `y` is the row's viewport index; each row is shaped by
  `shape_row` (the same driver `shape_viewport` uses) and assembled by
  `rebuild_row` (Experiment 371); the row's cells are `opts.cells` (the same
  `RunCell`s the shaping read). The shared resolver is reused across rows (its
  font cache accumulates), matching the shared grid.
- **Faithful adaptation**: the two-step per row (shape, then rebuild) is a
  borrow-checker shape — `shape_row` borrows `grid.resolver` and returns owned
  `ShapedRun`s, releasing that borrow before `rebuild_row` borrows `grid`; the
  runs/cells are identical to shaping and assembling separately. (This mirrors
  `shape_viewport`, but interleaves the assembly so the shaped runs are consumed
  immediately rather than collected.) `rebuild_viewport` takes the renderer's
  color config (`default_fg`/`palette`/`bold`/`alpha`/thicken), as
  `rebuild_row`.
- **Deferred**: the renderer-layer color adjustments (reverse-video, selection,
  min-contrast, faint/dim alpha); the background, decoration (underline/
  strikethrough), and cursor cells; and the Metal upload of `Contents`.
  (Consumed by tests now.)
- No C ABI/header/ABI-inventory change (internal Rust).

## Changes

1. `roastty/src/renderer/cell.rs`: add the `rebuild_viewport` function; import
   `font::run::{shape_row, RunOptions}`.
2. Test (in `cell.rs`): with a Menlo `SharedGrid` and a 2×2 `Contents`, build a
   two-row viewport with **observably different** rows — row 0 = `'A'`/`'B'`
   (two visible glyphs), row 1 = `'C'` + an empty cell (one visible glyph; the
   empty cell shapes to a 0-size glyph and is skipped) — and call
   `rebuild_viewport`:
   - assert `fg_rows[1].len() == 2` (row 0) **and** `fg_rows[2].len() == 1`
     (row 1) — distinct counts proving each row is shaped from its own
     `RunOptions`, not row 0's reused for both;
   - assert the grid positions are `[0, 0]`/`[1, 0]` for row 0 and `[0, 1]` for
     row 1's single glyph (each row assembled at its own `y`).
3. Format and test (`cargo fmt`, accept output).

## Verification

```bash
cargo fmt
cargo fmt -- --check
cargo test -p roastty rebuild_viewport
cargo test -p roastty
cargo build -p roastty            # no warnings
rg -n 'ghostty|Ghostty|GHOSTTY' roastty/src/font roastty/src/renderer && exit 1 || true
rg -n 'ghostty|Ghostty|GHOSTTY' roastty/src/lib.rs roastty/include/roastty.h roastty/tests/abi_harness.c && exit 1 || true
git diff --check
```

The experiment **passes** if:

- `rebuild_viewport` shapes and assembles every viewport row into `Contents` in
  row order, reusing the shared resolver — faithful to the row loop of upstream
  `rebuildCells` (foreground text);
- the test passes (a two-row viewport fills both rows at the right positions),
  and the existing tests still pass;
- the color adjustments, backgrounds/decorations/cursor, and the Metal upload
  stay deferred;
- `cargo fmt` accepted, `cargo build -p roastty` has no warnings, and
  `cargo test -p roastty` passes with no regressions;
- the no-`ghostty`-name gates and `git diff --check` pass;
- Codex reviews the design before implementation and the result after, with all
  real findings fixed.

The experiment **fails** if a row is shaped/assembled at the wrong `y`, the
borrow ordering is wrong, or any public C API/ABI changes.

## Design Review

Codex reviewed this design before implementation and **approved** it with one
**Required** finding, now addressed:

- **Required (addressed):** the test used identical `"AB"`/`"AB"` rows, so a
  buggy loop that reused the first row's `RunOptions` for every `y` would still
  pass. The test now makes the rows observably different — row 0 `'A'`/`'B'`
  (two visible glyphs), row 1 `'C'` + an empty cell (one visible glyph) — and
  asserts `fg_rows[1].len() == 2` and `fg_rows[2].len() == 1`, proving each row
  is shaped from its own `RunOptions`.

Codex confirmed the loop design is sound: shaping each `RunOptions` with
`shape_row(opts, &mut grid.resolver)` then passing the owned `runs` plus
`&opts.cells` to `rebuild_row` is faithful to the foreground row walk and uses
the same cell data that was shaped; the borrow ordering is correct (the mutable
borrow of `grid.resolver` ends when `shape_row` returns, before `rebuild_row`
borrows `grid`); and `u16::try_from(y).expect(...)` is the right checked
adaptation for the viewport row index.

Review artifacts:

- Prompt: `logs/codex-review/20260603-183040-241273-prompt.md` (design)
- Result: `logs/codex-review/20260603-183040-241273-last-message.md` (design)

## Result

**Result:** Pass

The whole viewport's foreground text now rebuilds in one call — the capstone of
the text-rendering path.

- `roastty/src/renderer/cell.rs`:
  `rebuild_viewport(contents, grid, rows, default_fg, palette, bold, alpha, thicken, thicken_strength)`
  iterates the viewport's per-row `RunOptions` in order; for each row it checks
  the row index (`u16::try_from`), shapes the row into `ShapedRun`s (`shape_row`
  over `grid.resolver` — owned, releasing the borrow), and calls `rebuild_row`
  at that `y` with the row's `opts.cells`. Imported
  `font::run::{shape_row, RunOptions}`.

Test (in `cell.rs`): `rebuild_viewport_fills_each_row` builds a 2×2 viewport
with observably different rows — row 0 `'A'`/`'B'` (two visible glyphs), row 1
`'C'` + an empty cell (one visible glyph; the empty shapes to a 0-size glyph and
is skipped) — and asserts `fg_rows[1].len() == 2` and `fg_rows[2].len() == 1`
with grid positions `[0,0]`/`[1,0]` (row 0) and `[0,1]` (row 1) — the distinct
counts prove each row is shaped from its own `RunOptions`.

Gate results:

- `cargo fmt -p roastty` accepted; `--check` clean.
- `cargo test -p roastty` → 2822 passed, 0 failed (+1, no regressions).
- `cargo build -p roastty` → no warnings.
- No-`ghostty`-name gates (font + renderer) clean; `git diff --check` clean.

## Conclusion

The text-rendering path is complete end to end: a live terminal screen's
`RunOptions` (from `Terminal::shape_run_options`) → `rebuild_viewport` → a
`Contents` whose foreground rows are filled with correctly-placed,
correctly-colored `CellTextVertex`es. Every primitive in the chain — shaping,
rasterization, the glyph cache, the per-glyph/per-run/per-row/per-viewport
assembly — is ported and gated:

```
Terminal::shape_run_options() → Vec<RunOptions>
  → rebuild_viewport → (per row) shape_row + rebuild_row
      → cell_infos + resolve_fg + add_run → add_glyph → SharedGrid::render_glyph
  → Contents (whole viewport foreground)
```

The remaining renderer-bridge work is no longer on the foreground-text critical
path: the renderer-layer color adjustments (reverse-video, selection,
min-contrast, faint/dim alpha), the **background** cells, the **decorations**
(underline/strikethrough/overline), the **cursor** cell, and the **Metal
upload** of `Contents` to the GPU. Each is a separate, independently-gateable
experiment.

## Completion Review

Codex reviewed the completed implementation and result and **approved** with
**no findings**. It confirmed `rebuild_viewport` is faithful to the approved row
loop (iterates `rows` in order, checks the row index with `u16::try_from`,
shapes each row with `shape_row(opts, &mut grid.resolver)`, then passes the
owned runs and the same `opts.cells` into `rebuild_row` at that `y`), with sound
borrow ordering (`shape_row` returns owned `ShapedRun`s before `rebuild_row`
borrows the whole grid). It confirmed the Required test fix is applied — the
rows are observably different (row 0 → two visible glyphs, row 1 → one), so
reusing the first row's `RunOptions` would fail the `fg_rows` count assertions.
Nothing needed to change before the result commit.

Review artifacts:

- Result review: `logs/codex-review/20260603-183307-229808-last-message.md`
