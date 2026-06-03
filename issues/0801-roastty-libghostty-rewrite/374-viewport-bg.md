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

# Experiment 374: backgrounds in the viewport pass

## Description

`rebuild_viewport` (Experiment 372) fills the foreground text; `rebuild_bg_row`
(Experiment 373) writes one row's backgrounds. This experiment folds the
background write into the viewport pass: for each row, `rebuild_viewport` now
writes the row's backgrounds **and** its foreground glyphs — one pass per row
filling both buffers, as upstream `rebuildCells` does. No new parameters: the
background write reuses the `palette`/`alpha` `rebuild_viewport` already takes.

## Upstream behavior

`rebuildCells` (`renderer/generic.zig`) processes each viewport row once,
emitting both the background cell and the foreground glyph(s) for each column.
roastty already has both per-row halves — `rebuild_bg_row` and `rebuild_row` —
and this experiment calls both inside the existing row loop, so a single
`rebuild_viewport` fills `Contents`'s background **and** foreground for the
whole screen.

## Rust mapping (`roastty/src/renderer/cell.rs`)

The `rebuild_viewport` loop gains the background write (no signature change):

```rust
for (y, opts) in rows.iter().enumerate() {
    let y = u16::try_from(y).expect("viewport row fits u16");

    // Backgrounds first (behind the glyphs); needs no shaping or grid.
    rebuild_bg_row(contents, y, &opts.cells, palette, alpha);

    // Then the foreground: shape the row (borrows the grid's resolver — `runs`
    // is owned, releasing that borrow) and assemble it.
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
```

## Scope / faithfulness notes

- **Ported (bridged)**: the single per-row pass of upstream `rebuildCells` that
  fills both the background and the foreground — `rebuild_viewport` now calls
  `rebuild_bg_row` then `rebuild_row` for each row.
- **Faithful**: the background is written before the foreground (it sits behind
  the glyph), matching upstream's per-cell order; both halves use the row's
  `opts.cells`; the foreground path (shape → assemble) is unchanged from
  Experiment 372. The two writes target disjoint buffers (`bg_cells` vs
  `fg_rows`), so the order is immaterial to correctness but kept
  background-first for clarity.
- **Faithful adaptation**: `rebuild_bg_row` needs no grid and borrows only
  `contents` + `opts.cells` + `palette`, so it runs before the shape/assemble
  step with no borrow interaction; `rebuild_viewport`'s signature is unchanged
  (the `palette`/`alpha` it already takes feed the background).
- **Deferred**: the renderer-layer background adjustments (default-background
  fill, opacity, reverse-video, selection); the decorations
  (underline/strikethrough/overline); the cursor cell; and the Metal upload.
  (Consumed by tests now.)
- No C ABI/header/ABI-inventory change (internal Rust).

## Changes

1. `roastty/src/renderer/cell.rs`: add the
   `rebuild_bg_row(contents, y, &opts.cells, palette, alpha)` call to the
   `rebuild_viewport` row loop (before the shape/assemble step).
2. Test (in `cell.rs`): a 2×2 viewport whose row 0 has a cell with an explicit
   background (`bg_color = Color::Palette(1)`) and visible glyphs; after
   `rebuild_viewport`, assert **both**:
   - the foreground cells are present (`fg_rows[1]` non-empty, as before);
   - `bg_cell(0, 0)` holds the resolved background
     `CellBg([palette[1].r, .g, .b,   alpha])` — proving the single viewport
     pass now fills backgrounds too. (The existing
     `rebuild_viewport_fills_each_row` test still passes — its default-style
     cells write transparent backgrounds, which it does not assert.)
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

- `rebuild_viewport` fills both the background and the foreground for every row
  in one pass — faithful to upstream `rebuildCells`;
- the new test passes (foreground present **and** the explicit background
  written), and the existing tests still pass;
- the background adjustments, decorations, cursor, and Metal upload stay
  deferred;
- `cargo fmt` accepted, `cargo build -p roastty` has no warnings, and
  `cargo test -p roastty` passes with no regressions;
- the no-`ghostty`-name gates and `git diff --check` pass;
- Codex reviews the design before implementation and the result after, with all
  real findings fixed.

The experiment **fails** if a row's background or foreground is skipped, the
borrow ordering breaks, or any public C API/ABI changes.

## Design Review

Codex reviewed this design before implementation and **approved** it with one
**Low** finding, to be addressed in implementation:

- **Low (to address):** `rebuild_viewport`'s doc comment describes rebuilding
  "foreground text" only; after this experiment it should say the viewport pass
  fills both background cells and foreground text (with
  decorations/cursor/upload still deferred). The implementation updates the doc
  accordingly.

Codex confirmed the plan is otherwise sound: calling
`rebuild_bg_row(contents, y, &opts.cells, palette, alpha)` before shaping and
`rebuild_row` is faithful to upstream's per-row rebuild flow (backgrounds into
`bg_cells`, foreground glyphs into `fg_rows`, both using the same row
`opts.cells`); the borrow ordering is clean (`rebuild_bg_row` finishes before
`shape_row` borrows `grid.resolver`, and `shape_row` returns owned runs before
`rebuild_row` borrows the full grid); reusing the existing `palette`/`alpha`
parameters is correct; and the test is sufficient if it asserts both buffers
from the same `rebuild_viewport` call (foreground in `fg_rows[1]`, the explicit
background in `bg_cell(0, 0)`).

Review artifacts:

- Prompt: `logs/codex-review/20260603-184123-778140-prompt.md` (design)
- Result: `logs/codex-review/20260603-184123-778140-last-message.md` (design)

## Result

**Result:** Pass

One viewport pass now fills both buffers.

- `roastty/src/renderer/cell.rs`: `rebuild_viewport`'s row loop now calls
  `rebuild_bg_row(contents, y, &opts.cells, palette, alpha)` (backgrounds, no
  grid needed) before shaping the row and assembling its foreground
  (`shape_row` + `rebuild_row`). The doc comment was updated to state the pass
  fills both background and foreground (decorations/cursor/upload still
  separate). No signature change — the existing `palette`/`alpha` feed the
  background.

Test (in `cell.rs`): `rebuild_viewport_fills_background_and_foreground` builds a
2×1 viewport with `'A'` (`bg = Palette(1)`) and `'B'` (`bg = None`); after one
`rebuild_viewport` call it asserts the foreground glyphs are present
(`fg_rows[1].len() == 2`) **and** the explicit background is written
(`bg_cell(0, 0) == CellBg([p1.r, p1.g, p1.b, 255])`) with the default cell
transparent (`bg_cell(0, 1) == CellBg([0, 0, 0, 0])`).

Gate results:

- `cargo fmt -p roastty` accepted; `--check` clean.
- `cargo test -p roastty` → 2825 passed, 0 failed (+1, no regressions; the
  existing `rebuild_viewport_fills_each_row` still passes).
- `cargo build -p roastty` → no warnings.
- No-`ghostty`-name gates (font + renderer) clean; `git diff --check` clean.

## Conclusion

`rebuild_viewport` is now the single entry that turns a viewport's `RunOptions`
into a `Contents` with both its background colors and its foreground glyphs —
upstream `rebuildCells`'s per-row background+foreground pass, ported and gated.
A renderer can call it once per frame to fill the text and background GPU
buffers.

The remaining renderer-bridge work: the **decorations**
(underline/strikethrough/ overline cells), the **cursor** cell, the
renderer-layer **color adjustments** (reverse-video, selection, min-contrast,
faint/dim alpha, default-background fill, opacity), and the **Metal upload** of
`Contents` to the GPU.

## Completion Review

Codex reviewed the completed implementation and result and **approved** with
**no findings**. It confirmed `rebuild_viewport` now calls `rebuild_bg_row`
before shaping and foreground assembly (faithful to the single per-row rebuild
flow — backgrounds into `bg_cells`, foreground glyphs into `fg_rows`, both using
the same `opts.cells`), with sound borrow ordering (the background write
finishes before `shape_row` borrows `grid.resolver`, and `shape_row` returns
owned runs before `rebuild_row` borrows the full grid). It confirmed the Low doc
finding was addressed (the doc now says the pass fills both buffers) and that
the new test proves one `rebuild_viewport` call fills both buffers (foreground
in `fg_rows[1]`, the explicit background in `bg_cell(0, 0)`). Nothing needed to
change before the result commit.

Review artifacts:

- Result review: `logs/codex-review/20260603-184302-877387-last-message.md`
