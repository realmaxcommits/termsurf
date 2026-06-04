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

# Experiment 391: plumb the per-row highlights into the passes

## Description

`selected_state` is highlight-aware (Experiment 390), but the row passes pass an
empty highlight slice, so search highlighting never fires. This experiment
plumbs the **per-row** search highlights into the passes: `rebuild_viewport`
takes a per-row `&[Vec<Highlight>]` (parallel to `rows`, mirroring upstream's
`row_highlights`), and threads each row's slice to `rebuild_bg_row` and
`rebuild_row`, which pass it to `selected_state`. A cell inside a search-match
highlight now draws the search colors (Experiment 388) — search highlighting is
**live** in the rebuild. The highlight ranges themselves still originate from
the caller (the search engine that produces them is outside roastty's renderer
bridge); this experiment wires the renderer to honor them.

## Upstream behavior

In `rebuildCells` (`renderer/generic.zig`), the per-row highlights come from the
render state, parallel to the row data:

```zig
const row_highlights = row_data.items(.highlights);
// … per row y …
const highlights = row_highlights[y];   // this row's highlight list
// … passed to the `selected` derivation for each cell in the row …
```

So highlights are indexed by row, alongside the row's cells/selection, and each
row's highlight list feeds that row's per-cell `selected` derivation. They are a
renderer input (not a shaper/`RunOptions` field, per Experiment 390).

## Rust mapping (`roastty/src/renderer/cell.rs`)

`rebuild_bg_row` and `rebuild_row` gain a `highlights: &[Highlight]` param (the
row's highlights), passed straight to `selected_state`:

```rust
// rebuild_bg_row / rebuild_row, per cell:
let state = selected_state(selection, highlights, x, cell.wide);
```

`rebuild_viewport` gains a `highlights: &[Vec<Highlight>]` param (per row), and
threads each row's slice (defaulting to empty for a row beyond the array):

```rust
for (y, opts) in rows.iter().enumerate() {
    let row_highlights = highlights.get(y).map(Vec::as_slice).unwrap_or(&[]);
    let y = u16::try_from(y).expect("viewport row fits u16");

    rebuild_bg_row(
        contents, y, &opts.cells, opts.selection, row_highlights,
        selection_config, default_fg, default_bg, palette, bold, alpha,
    );
    let runs = shape_row(opts, &mut grid.resolver);
    rebuild_row(
        contents, grid, y, &runs, &opts.cells, opts.selection, row_highlights,
        selection_config, default_fg, default_bg, palette, bold, alpha,
        faint_opacity, thicken, thicken_strength,
    )?;
}
```

`Highlight` is the type from Experiment 390 (already in `cell.rs`). A row with
no highlights (the common case, or a row index beyond the `highlights` array)
passes an empty slice, so its behavior is unchanged.

## Scope / faithfulness notes

- **Ported (bridged)**: the per-row highlight plumbing — `rebuild_viewport`
  threads each row's highlight list (parallel to `rows`, like upstream's
  `row_highlights`) to the passes, which feed it to `selected_state`. A
  search-match cell now draws the search colors, completing live search
  highlighting in the rebuild.
- **Faithful**: highlights are indexed by row (the `highlights[y]` slice feeds
  row `y`'s per-cell `selected` derivation), matching upstream's
  `row_highlights[y]`; a row's highlights drive only that row; the derivation
  and colors are the already-faithful `selected_state` (Experiment 390) and
  `selected_colors` (Experiment 388). A row beyond the array (or an empty list)
  contributes no highlights — equivalent to upstream's empty per-row list.
- **Faithful adaptation**: highlights are a separate `&[Vec<Highlight>]`
  parameter (not a `RunOptions` field), because `RunOptions` mirrors the font
  shaper's `RunOptions` (no highlights — Experiment 390); upstream likewise
  sources highlights from the render state, separate from the shaper options.
  The `.get(y).unwrap_or(&[])` lookup tolerates a shorter array (a row with no
  highlights), avoiding a length-coupling panic.
- **Deferred**: the origin of the highlight ranges (the search engine that
  computes search-match ranges feeds them in from outside the renderer bridge);
  the lock-cursor glyph + under-cursor recolor; the column-ordered decoration
  merge + link double-underline; the Metal upload. (Consumed by tests now.)
- No C ABI/header/ABI-inventory change (internal Rust).

## Changes

1. `roastty/src/renderer/cell.rs`:
   - `rebuild_bg_row`: add a `highlights: &[Highlight]` param; pass it to
     `selected_state`. Update its doc comment.
   - `rebuild_row`: add a `highlights: &[Highlight]` param; pass it to
     `selected_state`. Update its doc comment.
   - `rebuild_viewport`: add a `highlights: &[Vec<Highlight>]` param; thread
     each row's slice (`.get(y).map(Vec::as_slice).unwrap_or(&[])`) to both
     passes. Update its doc comment.
   - Update the existing `rebuild_bg_row`/`rebuild_row`/`rebuild_viewport` test
     call sites (`&[]` for the row highlights; `&[]` for the viewport's per-row
     array).
2. Tests (in `cell.rs`):
   - `rebuild_bg_row` with a `SearchMatch` highlight over a no-explicit-bg cell
     → its background is the search background (amber `#FFE082`, opaque); a
     `SearchMatchSelected` highlight → the salmon `#F2A57E`; a cell outside the
     highlight is unchanged;
   - `rebuild_row` with a `SearchMatch` highlight over a glyph cell → the glyph
     foreground is the search foreground (black);
   - a `rebuild_viewport` end-to-end test: a per-row highlights array selects
     one column as `SearchMatch` → that column's background is amber and its
     glyph is black, an un-highlighted column unchanged.
3. Format and test (`cargo fmt`, accept output).

## Verification

```bash
cargo fmt
cargo fmt -- --check
cargo test -p roastty rebuild
cargo test -p roastty
cargo build -p roastty            # no warnings
rg -n 'ghostty|Ghostty|GHOSTTY' roastty/src/font roastty/src/renderer && exit 1 || true
rg -n 'ghostty|Ghostty|GHOSTTY' roastty/src/lib.rs roastty/include/roastty.h roastty/tests/abi_harness.c && exit 1 || true
git diff --check
```

The experiment **passes** if:

- `rebuild_viewport` threads each row's highlight list to the passes, and a
  search-match cell draws the search colors (background opaque, foreground the
  search foreground) — faithful to upstream's per-row `row_highlights`;
- the tests pass (the search-match/search-selected backgrounds, the search
  foreground, the end-to-end viewport case, an un-highlighted cell unchanged),
  and the existing tests still pass (updated for the new signatures, passing
  `&[]`);
- the highlight-range origin, the lock-cursor recolor, and the Metal upload stay
  deferred;
- `cargo fmt` accepted, `cargo build -p roastty` has no warnings, and
  `cargo test -p roastty` passes with no regressions;
- the no-`ghostty`-name gates and `git diff --check` pass;
- Codex reviews the design before implementation and the result after, with all
  real findings fixed.

The experiment **fails** if a highlighted cell does not draw the search colors,
a row's highlights bleed into another row, the empty/shorter-array case panics,
or any public C API/ABI changes.

## Design Review

Codex reviewed this design before implementation and **approved** it with **no
findings**. It confirmed the per-row `&[Vec<Highlight>]` parameter is a faithful
adaptation — upstream highlights are render-state row data parallel to the row
cells, not shaper `RunOptions`, so keeping them separate from `RunOptions` is
the right boundary. It confirmed
`highlights.get(y).map(Vec::as_slice).unwrap_or(&[])` is sound (it mirrors
`row_highlights[y]` when present while treating missing rows as "no highlights"
instead of coupling the API to an exact length), and that indexing with the
`usize` enumerate index before converting `y` to `u16` is correct. It agreed
that passing the same `row_highlights` slice to both `rebuild_bg_row` and
`rebuild_row` keeps the background/foreground state consistent, and that the
tests are sufficient (search-match and search-selected backgrounds, the search
foreground, unchanged outside cells, and an end-to-end viewport case proving the
row-indexed plumbing).

Review artifacts:

- Prompt: `logs/codex-review/20260603-203729-176260-prompt.md` (design)
- Result: `logs/codex-review/20260603-203729-176260-last-message.md` (design)

## Result

**Result:** Pass

Search highlighting is now live in the rebuild.

- `roastty/src/renderer/cell.rs`:
  - `rebuild_bg_row` and `rebuild_row` gain a `highlights: &[Highlight]` param,
    passed to `selected_state(selection, highlights, x, cell.wide)`.
  - `rebuild_viewport` gains a `highlights: &[Vec<Highlight>]` param (per-row,
    parallel to `rows`); per row it computes
    `row_highlights = highlights.get(y) .map(Vec::as_slice).unwrap_or(&[])` (the
    `usize` enumerate index, before the `y: u16` shadow) and passes the **same**
    slice to both passes — so a row with no highlights (or beyond the array) is
    unchanged.
  - The doc comments now describe the highlight inputs. The existing
    `rebuild_bg_row`/`rebuild_row`/`rebuild_viewport` test call sites are
    updated for the new signatures (`&[]`).

Tests (in `cell.rs`):

- `rebuild_bg_row_recolors_highlighted_cells` — a `SearchMatch` highlight over a
  no-explicit-bg cell → opaque amber `#FFE082`; a `SearchMatchSelected`
  highlight → salmon `#F2A57E`; an un-highlighted column → transparent.
- `rebuild_row_recolors_highlighted_foreground` — a `SearchMatch` highlight over
  a glyph cell → the glyph draws with the search foreground (black).
- `rebuild_viewport_threads_per_row_highlights` — a per-row highlights array
  highlights column 1 (`SearchMatch`) → that column's background is amber and
  its glyph is black, column 0 unchanged — end-to-end row-indexed plumbing.

Gate results:

- `cargo fmt -p roastty` accepted; `--check` clean.
- `cargo test -p roastty` → 2850 passed, 0 failed (+3, no regressions; the
  existing 13 rebuild tests pass with the new `&[]` signatures).
- `cargo build -p roastty` → no warnings.
- No-`ghostty`-name gates (font + renderer) clean; `git diff --check` clean.

## Conclusion

Search highlighting is now **live** in the rebuild: a cell inside a per-row
search highlight draws the search colors (amber/salmon background opaque, black
foreground) — the full chain from a highlight range to the final `Contents` is
faithful (`selected_state` derivation → `selected_colors` → both passes). The
selection **and** search recolor are complete; the only external dependency left
is the origin of the highlight ranges (the search engine that computes
search-match ranges, outside the renderer bridge).

The remaining renderer-bridge work: the lock-cursor glyph + under-cursor text
recolor; the column-ordered decoration merge + link double-underline; and the
**Metal upload** of `Contents`.

## Completion Review

Codex reviewed the completed implementation and result and **approved** with
**no findings**. It confirmed the implementation matches the approved design and
is faithful: highlights stay a renderer input (not on `RunOptions`);
`rebuild_viewport` indexes the per-row `highlights` with the `usize` row index
before the `u16` shadow, missing rows default to `&[]`, and the same
`row_highlights` slice is passed to both the background and foreground passes
(consistent bg/fg state, no row bleed); both passes route that slice into
`selected_state`, so search highlighting is live. It confirmed the new tests
cover the search-match amber background, the search-selected salmon background,
the search foreground black, un-highlighted cells unchanged, and the
viewport-level per-row plumbing, that the existing empty-slice call sites
preserve prior behavior, and that there is no public C ABI/header change.
Nothing needed to change before the result commit.

Review artifacts:

- Result review: `logs/codex-review/20260603-204215-693382-last-message.md`
