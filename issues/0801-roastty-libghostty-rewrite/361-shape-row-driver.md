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

# Experiment 361: the shape-a-row driver

## Description

Experiments 359–360 produce a `RunOptions` per viewport row and expose them to
the renderer. The `RunIterator` (Exp 352–357) groups a row's cells into runs,
and `Face::shape_run` (Exp 339–349) shapes a run's codepoints into positioned
glyphs — but nothing connects them. This experiment adds the **driver** that
ties the two together: given a row's `RunOptions` and the `CodepointResolver`,
it runs the `RunIterator` to completion, resolves each run's face, and shapes
each run's codepoints into glyph cells. The result is one `ShapedRun` per text
run — the last font-side piece before the Metal draw path consumes it.

## Upstream behavior

Upstream's renderer drives the shaper per row:
`var run_iter = self.font_grid.shaper.runIterator(opts); while (run_iter.next()) |run| { const cells = try self.font_grid.shaper.shape(run); … }`
(`renderer/generic.zig`). The `runIterator` yields `TextRun`s;
`shaper.shape(run)` shapes the run's accumulated codepoints with the run's face
into `[]shaper.Cell` (glyph index + x + offsets). roastty already has both
halves — `RunIterator::next` yields a `RunOutput` (`TextRun` +
`Vec<Codepoint>`), and `Face::shape_run(&[Codepoint]) -> Vec<shape:: Cell>`
shapes them. This experiment is the `while` loop between them.

## Rust mapping (`roastty/src/font/run.rs`)

```rust
use crate::font::shape;

/// One run's shaped output: the run descriptor (with its `offset` column and
/// content `hash`) and the positioned glyph cells `Face::shape_run` produced.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ShapedRun {
    pub run: TextRun,
    pub glyphs: Vec<shape::Cell>,
}

/// Shape a terminal row end to end: drive the [`RunIterator`] over `opts`, then
/// shape each run's codepoints with its resolved face. Returns one [`ShapedRun`]
/// per text run, in column order. Runs whose font index is **special**
/// (sprite/box-drawing) are skipped — that draw path is separate (deferred).
pub(crate) fn shape_row(opts: &RunOptions, resolver: &mut CodepointResolver) -> Vec<ShapedRun> {
    // Drain the iterator first so its `&mut resolver` borrow is released before
    // we re-borrow the resolver's collection to fetch faces.
    let mut runs = Vec::new();
    let mut iter = RunIterator::new(opts, resolver);
    while let Some(out) = iter.next() {
        runs.push(out);
    }
    drop(iter);

    let mut shaped = Vec::with_capacity(runs.len());
    for out in runs {
        // Special (sprite/box-drawing) indices have no face — the sprite draw
        // path shapes them separately (a later experiment).
        if out.run.font_index.special_kind().is_some() {
            continue;
        }
        // A non-special index from the run iterator must be face-backed
        // (`resolve_font` resolves it through the resolver, and `get_face` only
        // rejects special/out-of-bounds indices). A non-special error means a
        // broken invariant, not skippable text — fail loudly rather than drop it.
        let face = resolver
            .collection()
            .get_face(out.run.font_index)
            .expect("a text run's font index must be face-backed");
        let glyphs = face.shape_run(&out.codepoints);
        shaped.push(ShapedRun {
            run: out.run,
            glyphs,
        });
    }
    shaped
}
```

The two-pass shape (drain, then shape) is required by the borrow checker:
`RunIterator::new` holds `&mut resolver` for the whole iteration, while
`get_face` needs a shared borrow of the same resolver's collection. Draining
into a `Vec<RunOutput>` first releases the mutable borrow.

## Scope / faithfulness notes

- **Ported (bridged)**: the per-row driver loop — run the `RunIterator`, resolve
  each run's face, shape its codepoints — producing one `ShapedRun` per text
  run. This is upstream's renderer `while (run_iter.next()) |run| shape(run)`
  loop.
- **Faithful**: the run order is column order (the iterator's order); each run's
  glyphs come from `Face::shape_run` over exactly the `RunOutput.codepoints` the
  iterator accumulated; the `TextRun` (with `offset`, `cells`, `hash`,
  `font_index`) is carried through so the caller can place the run (absolute
  `x = run.offset + glyph.x`) and cache by `hash`.
- **Faithful adaptation**: the two-pass drain-then-shape is a borrow-checker
  necessity, not a behavior change — the runs and their codepoints are identical
  to a single-pass `while` loop. roastty omits the shaping cache (upstream
  caches shaped runs by `hash`); the `hash` is carried on the `TextRun` so the
  cache is a later experiment, not a faithfulness gap here.
- **Special/sprite indices deferred**: a run whose `font_index` is special
  (`special_kind().is_some()` — sprite/box-drawing/underline) has no face and is
  **skipped**; those glyphs are produced by the sprite font / box-drawing draw
  path, a separate later experiment. A non-special index from the iterator must
  be face-backed (`resolve_font` resolves it through the resolver, and
  `get_face` rejects only special/out-of-bounds indices), so a non-special
  `get_face` error is a broken invariant — the driver `expect`s a face rather
  than silently dropping text.
- **Deferred**: the Metal draw-path wiring (placing each `ShapedRun`'s glyphs
  into the renderer's cell buffer at `run.offset + glyph.x`, with the cell
  background and foreground colors) and the shaped-run cache. (Consumed by tests
  now.)
- No C ABI/header/ABI-inventory change (internal Rust).

## Changes

1. `roastty/src/font/run.rs`: add the `ShapedRun` struct and the `shape_row`
   driver; import `crate::font::shape`.
2. Test (in `run.rs`): with the existing `menlo_resolver()` helper, build a
   `RunOptions` for a narrow `"AB"` row (no selection, no cursor) and assert
   `shape_row`:
   - returns exactly one `ShapedRun` (one run, since `A`/`B` share Menlo and
     style);
   - the run's `offset == 0` and `cells == 2`;
   - the run shaped two glyph cells with nonzero `glyph_index` (Menlo has
     `A`/`B`) at run-relative `x == 0` and `x == 1`.
3. Format and test (`cargo fmt`, accept output).

## Verification

```bash
cargo fmt
cargo fmt -- --check
cargo test -p roastty shape_row
cargo test -p roastty
cargo build -p roastty            # no warnings
rg -n 'ghostty|Ghostty|GHOSTTY' roastty/src/font && exit 1 || true
rg -n 'ghostty|Ghostty|GHOSTTY' roastty/src/lib.rs roastty/include/roastty.h roastty/tests/abi_harness.c && exit 1 || true
git diff --check
```

The experiment **passes** if:

- `shape_row` drives the `RunIterator` over a row's `RunOptions` and shapes each
  text run's codepoints via its resolved face into `ShapedRun`s, in column
  order, carrying the `TextRun` for placement/caching — faithful to upstream's
  per-row driver loop;
- the driver test passes, and the existing tests still pass;
- special/sprite runs and the draw-path wiring stay deferred;
- `cargo fmt` accepted, `cargo build -p roastty` has no warnings, and
  `cargo test -p roastty` passes with no regressions;
- the no-`ghostty`-name gates and `git diff --check` pass;
- Codex reviews the design before implementation and the result after, with all
  real findings fixed.

The experiment **fails** if the driver mis-orders runs, shapes the wrong
codepoints, resolves the wrong face, or any public C API/ABI changes.

## Design Review

Codex reviewed this design before implementation and **approved** it with one
**Required** finding, now addressed:

- **Required (addressed):** the driver originally `continue`d on any `get_face`
  error after the `special_kind()` skip. But a non-special index from the run
  iterator must be face-backed (`resolve_font` resolves it through the resolver,
  and `get_face` rejects only special/out-of-bounds indices), so a non-special
  error means an internal invariant broke — silently `continue`ing would drop
  text. The driver now `expect`s a face for a non-special index
  (`expect("a text run's font index must be face-backed")`) rather than
  skipping.

Codex confirmed: the two-pass drain-then-shape is faithful (`RunOutput` owns the
`TextRun` and codepoints, so draining preserves exactly the same runs/codepoints
as a single-pass `while` loop) and the borrow reasoning is correct
(`RunIterator` holds `&mut resolver`; `get_face` needs a later shared borrow);
skipping `special_kind()` runs is the right scope boundary (sprite/box-drawing
draw path deferred), kept explicit and not conflated with an unexpected lookup
failure; the driver should **not** apply absolute `x` — `shape::Cell.x` is
run-relative by contract and `TextRun.offset` is the caller's placement input,
so carrying both is faithful; omitting the shaped-run cache is acceptable
because `TextRun.hash` is preserved; and the Menlo `"AB"` test covers the
bridge's happy path.

Review artifacts:

- Prompt: `logs/codex-review/20260603-172148-968800-prompt.md` (design)
- Result: `logs/codex-review/20260603-172148-968800-last-message.md` (design)

## Result

**Result:** Pass

The shape-a-row driver ties the run iterator to the shaper — the loop that has
been missing between them.

- `roastty/src/font/run.rs`:
  - `ShapedRun { run: TextRun, glyphs: Vec<shape::Cell> }` — one run's
    descriptor plus its positioned glyph cells.
  - `shape_row(opts, resolver) -> Vec<ShapedRun>` drains the `RunIterator` over
    a row's `RunOptions` into `Vec<RunOutput>` (releasing the `&mut resolver`
    borrow), then shapes each run: skips special (sprite/box-drawing) indices,
    `expect`s a face for every non-special index (a non-special `get_face` error
    is a broken invariant, not skippable text), and shapes its codepoints via
    `Face::shape_run`. Returns the `ShapedRun`s in column order, each carrying
    the `TextRun` (with `offset`/`hash`) for placement and caching. Imported
    `crate::font::shape` (`use crate::font::shape::{self, Codepoint};`).

Test (in `run.rs`): `shape_row_drives_iterator_and_shapes` builds a narrow
`"AB"` `RunOptions` (no selection, no cursor) with the `menlo_resolver()` helper
and asserts `shape_row` returns exactly one `ShapedRun` (`A`/`B` share Menlo and
style), with `run.offset == 0`, `run.cells == 2`, and two shaped glyph cells
with nonzero `glyph_index` at run-relative `x == 0` and `x == 1`.

Gate results:

- `cargo fmt -p roastty` accepted; `--check` clean.
- `cargo test -p roastty` → 2808 passed, 0 failed (+1, no regressions).
- `cargo build -p roastty` → no warnings.
- No-`ghostty`-name gates clean; `git diff --check` clean.

## Conclusion

The font subsystem can now shape a whole terminal row end to end: `shape_row`
turns a `RunOptions` (decoded cells + selection + cursor) into positioned
glyphs, run by run. Combined with Experiments 358–360, the path is complete from
a live terminal page row all the way to shaped glyphs:
`Terminal::shape_run_options` → `RunIterator` → `Face::shape_run` →
`Vec<ShapedRun>`.

The remaining renderer↔font work is the **Metal draw-path wiring**: place each
`ShapedRun`'s glyphs into the renderer's cell buffer at `run.offset + glyph.x`
(rasterizing each glyph into the atlas, with the cell's foreground/background
colors), plus the deferred sprite/box-drawing draw path and the shaped-run
cache.

## Completion Review

Codex reviewed the completed implementation and result and **approved** with
**no findings**. It confirmed the implementation matches the approved design and
upstream's per-row driver loop: it drains `RunIterator` in column order,
releases the mutable resolver borrow, then shapes each owned
`RunOutput.codepoints` with the resolved face; `ShapedRun` carries `TextRun`
through unchanged, so glyph `x` stays run-relative and the caller keeps
`run.offset` for placement. It confirmed the `expect()` is correct (after the
explicit `special_kind()` skip, a non-special index must be face-backed; a
non-special `get_face` error is an internal invariant failure, not a legitimate
drop), and that the test adequately proves the bridge's happy path (iterator
grouping, face lookup, shaping, carried run metadata, run-relative glyph
columns), with special-index skipping and cache behavior explicitly deferred.
Nothing needed to change before the result commit.

Review artifacts:

- Result review: `logs/codex-review/20260603-172418-662029-last-message.md`
