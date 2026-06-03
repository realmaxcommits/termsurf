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

# Experiment 362: shaping the whole viewport

## Description

Experiment 360 produces a `Vec<RunOptions>` for the active viewport (one per
row); Experiment 361's `shape_row` shapes one row into `Vec<ShapedRun>`. This
experiment composes them: `shape_viewport` takes the viewport's per-row
`RunOptions` and the `CodepointResolver` and shapes **every** row, threading the
one resolver across all rows. The result is one `Vec<ShapedRun>` per row — the
complete shaped viewport the Metal draw path consumes. This is the font-side
entry the renderer calls with the output of `Terminal::shape_run_options()`.

## Upstream behavior

Upstream's renderer `rebuildCells` walks the viewport rows and, for each, runs
the shaper driver (`runIterator` → `shape`) to fill the GPU cell buffer
(`renderer/generic.zig`). roastty already has the per-row driver (`shape_row`,
Experiment 361) and the per-row `RunOptions` assembly
(`Terminal::shape_run_options`, Experiment 360); this experiment is the row loop
between them — shaping each row with the shared resolver, in row order.

## Rust mapping (`roastty/src/font/run.rs`)

```rust
/// Shape every row of the viewport: run [`shape_row`] over each row's
/// [`RunOptions`] with the shared `resolver`, in row order. Returns one
/// `Vec<ShapedRun>` per input row (same length and order as `rows`) — the
/// complete shaped viewport. Faithful port of upstream's renderer `rebuildCells`
/// row loop (the per-row driver is `shape_row`).
pub(crate) fn shape_viewport(
    rows: &[RunOptions],
    resolver: &mut CodepointResolver,
) -> Vec<Vec<ShapedRun>> {
    rows.iter().map(|row| shape_row(row, resolver)).collect()
}
```

The single `&mut resolver` is reborrowed per row by `shape_row` (each call
finishes its drain-then-shape before the next), so the resolver's font cache
accumulates across rows — exactly upstream's shared-grid behavior.

## Scope / faithfulness notes

- **Ported (bridged)**: the viewport row loop — shape each row's `RunOptions`
  with the shared resolver, producing one `Vec<ShapedRun>` per row, in row
  order. This is upstream's renderer `rebuildCells` row walk (per-row driver =
  `shape_row`).
- **Faithful**: the output has exactly one entry per input row, in the same
  order; each row is shaped identically to a standalone `shape_row` call; the
  resolver is shared across rows (its font cache accumulates), matching the
  shared font grid.
- **Faithful adaptation**: `shape_viewport` is a thin composition over
  `shape_row` — no new shaping behavior, only the row iteration. It takes the
  already-assembled `Vec<RunOptions>` (from `Terminal::shape_run_options`,
  Experiment 360) rather than re-deriving rows, keeping the terminal-side
  assembly and the font-side shaping cleanly separated.
- **Deferred**: the Metal draw-path wiring (placing each row's `ShapedRun`
  glyphs into the renderer's cell buffer at `run.offset + glyph.x` with the cell
  colors), the sprite/box-drawing draw path, and the shaped-run cache — all as
  in Experiment 361. (Consumed by tests now.)
- No C ABI/header/ABI-inventory change (internal Rust).

## Changes

1. `roastty/src/font/run.rs`: add the `shape_viewport` function.
2. Test (in `run.rs`): with the `menlo_resolver()` helper, build a two-row
   viewport — row 0 narrow `"AB"`, row 1 narrow `"CD"` — and assert
   `shape_viewport`:
   - returns exactly two rows (one `Vec<ShapedRun>` per input row, in order);
   - each row has one `ShapedRun` with `cells == 2` and two glyph cells with
     nonzero `glyph_index` at run-relative `x == 0` / `x == 1`;
   - the two rows shape distinct glyphs (`A`/`B` vs `C`/`D` differ), proving
     each row is shaped from its own cells.
3. Format and test (`cargo fmt`, accept output).

## Verification

```bash
cargo fmt
cargo fmt -- --check
cargo test -p roastty shape_viewport
cargo test -p roastty
cargo build -p roastty            # no warnings
rg -n 'ghostty|Ghostty|GHOSTTY' roastty/src/font && exit 1 || true
rg -n 'ghostty|Ghostty|GHOSTTY' roastty/src/lib.rs roastty/include/roastty.h roastty/tests/abi_harness.c && exit 1 || true
git diff --check
```

The experiment **passes** if:

- `shape_viewport` shapes every viewport row via `shape_row` with the shared
  resolver, returning one `Vec<ShapedRun>` per row in row order — faithful to
  upstream's `rebuildCells` row loop;
- the viewport test passes, and the existing tests still pass;
- the draw-path wiring, sprite path, and cache stay deferred;
- `cargo fmt` accepted, `cargo build -p roastty` has no warnings, and
  `cargo test -p roastty` passes with no regressions;
- the no-`ghostty`-name gates and `git diff --check` pass;
- Codex reviews the design before implementation and the result after, with all
  real findings fixed.

The experiment **fails** if the row order or count diverges from the input, a
row is shaped from the wrong cells, or any public C API/ABI changes.

## Design Review

Codex reviewed this design before implementation and **approved** it with **no
findings**. It confirmed: the `&mut resolver` reborrow through the `map` closure
is sound (each `shape_row` call completes before the next row is evaluated, so
there is no overlapping mutable borrow); sharing one resolver across all rows is
the faithful choice (it matches the shared font grid/cache rather than resetting
discovery per row); `Vec<Vec<ShapedRun>>` is the right return shape for this
layer (one output slot per input row, in order, including rows that shape to an
empty vector — flattening would force the caller to reconstruct row boundaries,
a draw-path responsibility); and the two-row `"AB"`/`"CD"` test is sufficient
for this thin composition (two input rows → two output rows, one run each,
run-relative glyph `x`, and distinct glyphs proving rows are not reused or
flattened). It judged this a reasonable standalone experiment in the incremental
chain — the font-side viewport entry without Metal placement, sprite handling,
or caching mixed in.

Review artifacts:

- Prompt: `logs/codex-review/20260603-172613-875252-prompt.md` (design)
- Result: `logs/codex-review/20260603-172613-875252-last-message.md` (design)

## Result

**Result:** Pass

The font-side viewport entry is in place — the renderer can now shape an entire
viewport in one call.

- `roastty/src/font/run.rs`:
  `shape_viewport(rows, resolver) -> Vec<Vec<ShapedRun>>` runs `shape_row` over
  each row's `RunOptions` with the shared resolver, in row order, returning one
  `Vec<ShapedRun>` per input row. A thin composition over the per-row driver —
  the resolver's font cache accumulates across rows, matching upstream's shared
  font grid.

Test (in `run.rs`): `shape_viewport_shapes_every_row` builds a two-row viewport
(`"AB"`, `"CD"`) with `menlo_resolver()` and asserts two output rows in order,
each one `ShapedRun` with `cells == 2` and two nonzero-`glyph_index` cells at
run-relative `x == 0`/`x == 1`, and that the two rows' glyph sequences differ —
proving each row is shaped from its own cells.

Gate results:

- `cargo fmt -p roastty` accepted; `--check` clean.
- `cargo test -p roastty` → 2809 passed, 0 failed (+1, no regressions).
- `cargo build -p roastty` → no warnings.
- No-`ghostty`-name gates clean; `git diff --check` clean.

## Conclusion

The full font-side path is now a single composition:
`Terminal::shape_run_options()` produces the viewport's `Vec<RunOptions>`
(Experiments 358–360), and `shape_viewport` turns it into `Vec<Vec<ShapedRun>>`
— every row's positioned glyphs (Experiments 361–362). From a live terminal
screen to shaped glyphs, the font subsystem is complete and renderer-ready.

The remaining renderer↔font work is entirely on the **Metal draw path**: take
the `Vec<Vec<ShapedRun>>`, place each glyph into the renderer's cell buffer at
`run.offset + glyph.x` (rasterizing each glyph into the atlas with the cell's
foreground/background colors), plus the deferred sprite/box-drawing draw path
and the shaped-run cache.

## Completion Review

Codex reviewed the completed implementation and result and **approved** with
**no findings**. It confirmed the implementation is faithful to the approved
design and upstream's `rebuildCells` row walk (one `shape_row` call per input
row, same order and length, one shared resolver threaded across the viewport),
that the `map` closure borrow is sound (each `shape_row` completes before the
next item is evaluated, so the mutable resolver borrow is reborrowed
sequentially rather than held across rows), and that the test adequately proves
the thin composition (two input rows → two output rows, each shaped
independently, run-relative glyph `x`, distinct glyph sequences for `"AB"` vs
`"CD"`). Nothing needed to change before the result commit.

Review artifacts:

- Result review: `logs/codex-review/20260603-172732-076725-last-message.md`
