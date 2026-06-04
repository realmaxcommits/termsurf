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

# Experiment 418: the grid-size uniform update (update_grid_size)

## Description

Experiments 415–417 ported the screen-size, font-grid, and cursor uniform
groups. This experiment ports the last geometry uniform — **`grid_size`** (the
grid columns/rows) — which upstream sets in `rebuildCells` when the grid is
resized. It is a small, self-contained 1:1 port (one uniform field from a
`GridSize`), completing the geometry trio (`screen_size`, `cell_size`,
`grid_size`); the surrounding resize handling (the cell-buffer resize, the
padding-extend reset, the full rebuild) is separate and stays deferred.

## Upstream behavior

In `rebuildCells` (`renderer/generic.zig`), when the grid size changed, after
the cells are resized the `grid_size` uniform is updated:

```zig
if (grid_size_diff) {
    var new_size = self.cells.size;
    new_size.rows = state.rows;
    new_size.columns = state.cols;
    try self.cells.resize(self.alloc, new_size);

    // Update our uniforms accordingly, otherwise
    // our background cells will be out of place.
    self.uniforms.grid_size = .{ new_size.columns, new_size.rows };
}
```

So `grid_size` is set to `(columns, rows)` of the new grid.

## Rust mapping (`roastty/src/renderer/metal/shaders.rs`)

roastty's `GridSize` has `columns: u16` and `rows: u16` (`Unit = u16`), matching
the `grid_size: [u16; 2]` uniform field. `update_grid_size` sets it:

```rust
impl MetalUniforms {
    /// Update the grid-size uniform (upstream `rebuildCells`'s resize path): the
    /// `grid_size` (`[columns, rows]`), so the background cells stay in place when
    /// the grid is resized.
    pub(crate) fn update_grid_size(&mut self, grid: GridSize) {
        self.grid_size = [grid.columns, grid.rows];
    }
}
```

`grid_size` is `[columns, rows]`, the same order upstream writes
(`new_size.columns, new_size.rows`). Only `grid_size` is touched.

## Scope / faithfulness notes

- **Ported (bridged)**: `MetalUniforms::update_grid_size` — the `grid_size`
  uniform (`[columns, rows]`) from a `GridSize`, upstream's
  `rebuildCells`-resize `grid_size` assignment.
- **Faithful**: sets `grid_size = [columns, rows]` (the upstream order), the
  only uniform field that assignment touches.
- **Faithful adaptation**: `update_grid_size` mutates an existing
  `MetalUniforms` (upstream mutates `self.uniforms`) and takes the grid as a
  parameter (upstream reads the resized `self.cells.size`). The surrounding
  resize handling (the `Contents`/cell-buffer resize, the padding-extend reset,
  the full rebuild) is separate.
- **Deferred**: the rest of the resize path (the cell-buffer resize gating, the
  padding-extend reset, the dirty/rebuild), the config-derived uniform group
  (min-contrast, color-space and blending bools), the background color, a full
  production `MetalUniforms` constructor, and the live call sites. (Consumed by
  a later slice; this experiment lands and tests the grid-size update.)
- No C ABI/header/ABI-inventory change (internal Rust).

## Changes

1. `roastty/src/renderer/metal/shaders.rs`:
   - add `MetalUniforms::update_grid_size(&mut self, grid: GridSize)` setting
     `grid_size` from the grid columns/rows. (`GridSize` is already imported.)
2. Tests (in `shaders.rs`):
   - `update_grid_size` over a `GridSize { columns, rows }` sets `grid_size` to
     `[columns, rows]` (distinct columns ≠ rows to make the order meaningful),
     and leaves the other uniform fields (e.g. `screen_size`, `cell_size`,
     `bg_color`) untouched.
3. Format and test (`cargo fmt`, accept output).

## Verification

```bash
cargo fmt
cargo fmt -- --check
cargo test -p roastty update_grid_size
cargo test -p roastty
cargo build -p roastty            # no warnings
rg -n 'ghostty|Ghostty|GHOSTTY' roastty/src/font roastty/src/renderer && exit 1 || true
rg -n 'ghostty|Ghostty|GHOSTTY' roastty/src/lib.rs roastty/include/roastty.h roastty/tests/abi_harness.c && exit 1 || true
git diff --check
```

The experiment **passes** if:

- `update_grid_size` sets `grid_size` to `[columns, rows]` from the grid and
  touches nothing else — faithful to upstream's `rebuildCells` resize
  assignment;
- the test passes (the `grid_size` set in the right order, the other fields
  untouched), and the existing tests still pass;
- the rest of the resize path and the other uniform groups stay deferred;
- `cargo fmt` accepted, `cargo build -p roastty` has no warnings, and
  `cargo test -p roastty` passes with no regressions;
- the no-`ghostty`-name gates and `git diff --check` pass;
- Codex reviews the design before implementation and the result after, with all
  real findings fixed.

The experiment **fails** if `grid_size` is set in the wrong order, an unrelated
uniform field is changed, or any public C API/ABI changes.

## Design Review

Codex reviewed this design before implementation and **approved** it with **no
findings**. It confirmed the design is faithful to upstream's resize-path
assignment: `grid_size = [columns, rows]`, with no cast and no other uniform
fields touched; taking `GridSize` as a parameter and mutating `MetalUniforms`
matches the adaptation pattern already used for `update_screen_size` and
`update_font_grid`. It judged the scope thin but acceptable — the assignment is
embedded in upstream's resize path rather than a named function, but extracting
it as `update_grid_size` is consistent with the local uniform-update API and
keeps the surrounding resize concerns (cell-buffer resize, padding-extend reset,
dirty/full rebuild, live call-site wiring) separate and deferred. It judged the
planned test sufficient (distinct columns/rows to catch ordering mistakes, plus
untouched-field checks for the single-field boundary).

Review artifacts:

- Prompt: `logs/codex-review/20260604-082928-d418-prompt.md` (design)
- Result: `logs/codex-review/20260604-082928-d418-last-message.md` (design)
