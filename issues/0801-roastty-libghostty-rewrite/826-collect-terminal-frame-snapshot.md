+++
[implementer]
agent = "codex"
model = "gpt-5"
reasoning = "high"

[review.design]
agent = "codex"
model = "default"
reasoning = "medium"

[review.result]
agent = "codex"
model = "default"
reasoning = "medium"
+++

# Experiment 826: Collect Terminal Frame Snapshot

## Description

Start wiring the prepared frame rebuild path to live terminal state. The current
renderer foundation has value-level drivers for rebuild planning, row
formatting, text overlays, cursor uniforms, rebuild uniforms, padding-extension
refinement, and Metal presentation, but callers still have to manually assemble
the `FrameRebuildInput` and row formatting inputs. This keeps the work
disconnected from the terminal snapshot data that Roastty already exposes.

This experiment adds an owned renderer-side snapshot collected from
`terminal::Terminal`. The snapshot will gather:

- terminal grid size,
- viewport row dirty flags,
- viewport row shaping inputs,
- cursor viewport position, and
- optional surface preedit data supplied by the caller.

The snapshot can then borrow itself as `FrameRebuildInput` and build a
`FrameRebuildPlan`. It is the first live terminal-state collection bridge for
the main render loop, but it remains a prepared-data adapter only. It does not
mutate `Contents`, format rows, update uniforms, upload buffers, draw frames,
pace redraws, create the renderer thread, or change the C ABI render-state
surface.

## Changes

- `roastty/src/renderer/frame_rebuild.rs`
  - Import `crate::terminal::terminal::Terminal`.
  - Add `FrameTerminalSnapshot` owning:
    - `current_grid: GridSize`,
    - `terminal_grid: GridSize`,
    - `dirty: RenderDirty`,
    - `row_dirty: Vec<bool>`,
    - `rows: Vec<RunOptions>`,
    - `preedit: Option<Preedit>`, and
    - `cursor_viewport: Option<Coordinate>`.
  - Add
    `FrameTerminalSnapshot::collect(terminal: &Terminal, current_grid: GridSize, dirty: RenderDirty, preedit: Option<Preedit>) -> Self`.
  - Collect `terminal_grid` from `terminal.columns()` / `terminal.rows()`.
  - Collect `row_dirty` from `terminal.render_rows_snapshot()`, preserving
    viewport row order.
  - Collect `rows` from `terminal.shape_run_options()`.
  - Collect `cursor_viewport` from `terminal.cursor_position()` only when the
    cursor is inside the terminal grid.
  - Add `FrameTerminalSnapshot::rebuild_input(&self) -> FrameRebuildInput<'_>`.
  - Add
    `FrameTerminalSnapshot::build_plan(&self) -> Result<FrameRebuildPlan, FrameRebuildPlanError>`.
  - Add validation/reporting helpers if needed so tests can prove malformed
    terminal snapshots are rejected by the existing planner rather than by
    snapshot collection.
  - Add tests proving:
    - a clean terminal snapshot can build a no-row plan when no viewport rows
      are dirty,
    - dirty terminal rows become row-dirty flags and drive a partial rebuild,
    - dirty mode `Full` rebuilds all terminal rows even when row dirty flags are
      clean,
    - current-grid/terminal-grid mismatch produces a resize/full-rebuild plan,
    - cursor viewport is captured only when inside the terminal grid,
    - preedit is owned by the snapshot and feeds `FrameRebuildInput`, and
    - collected `rows` match `terminal.shape_run_options()` row order/length.
- `issues/0801-roastty-libghostty-rewrite/README.md`
  - Add this experiment to the index as `Designed`.
  - After implementation, update the renderer tracker to say live terminal
    snapshots can feed the prepared frame rebuild planner, while renderer-loop
    orchestration and drawing remain open.

## Verification

- Inspect:
  - `vendor/ghostty/src/renderer/generic.zig` `rebuildCells` terminal-state
    reads.
  - `roastty/src/renderer/frame_rebuild.rs`.
  - `roastty/src/terminal/terminal.rs` `render_rows_snapshot`,
    `shape_run_options`, `columns`, `rows`, and `cursor_position`.
  - `roastty/src/terminal/page_list.rs` dirty row snapshot behavior.
- Run Rust formatting:
  - `cargo fmt -p roastty`
- Run targeted tests:
  - `cargo test -p roastty renderer::frame_rebuild::tests::terminal_snapshot -- --nocapture`
  - `cargo test -p roastty renderer::frame_rebuild -- --nocapture`
- Run markdown formatting:
  - `prettier --write --prose-wrap always --print-width 80 issues/0801-roastty-libghostty-rewrite/README.md issues/0801-roastty-libghostty-rewrite/826-collect-terminal-frame-snapshot.md`
- Run:
  - `git diff --check`

The experiment passes if `FrameTerminalSnapshot` can collect live terminal
viewport state and build the same `FrameRebuildPlan` shapes that callers
previously had to assemble manually. It is Partial if terminal row/cursor state
is collected but row formatting inputs still need a follow-up adapter. It fails
if the terminal state cannot be collected without first adding the renderer
thread or changing the C ABI render-state surface.

## Design Review

Codex reviewed the design and approved it for the plan commit with no blockers.
The review confirmed that collecting terminal grid size, row dirty flags,
`RunOptions` rows, and in-grid cursor viewport from `terminal::Terminal` is the
right live terminal-state bridge for this stage.

The review also confirmed that `current_grid` and `RenderDirty` should remain
caller supplied because they describe renderer `Contents` state and render
policy rather than terminal row state. Keeping preedit caller supplied is also
correct because preedit lives in renderer/surface state. The planned tests cover
the relevant clean, partial, full, resize, cursor, preedit, and row-ordering
cases.

## Result

**Result:** Pass

Added `FrameTerminalSnapshot` as the owned renderer-side bridge from
`terminal::Terminal` to the prepared frame rebuild planner. The snapshot
collects terminal grid size, row dirty flags from `render_rows_snapshot()`, row
formatting inputs from `shape_run_options()`, in-grid cursor viewport position,
the caller-supplied render dirty mode, current renderer grid size, and optional
preedit payload.

The snapshot can now borrow itself as `FrameRebuildInput` or build a
`FrameRebuildPlan` directly. Cursor viewport bounds are centralized in a small
helper so collection ignores cursor coordinates outside the terminal grid before
preedit planning sees them.

Implementation changes:

- `roastty/src/renderer/frame_rebuild.rs`
  - Added `FrameTerminalSnapshot`.
  - Added `FrameTerminalSnapshot::collect`.
  - Added `FrameTerminalSnapshot::rebuild_input`.
  - Added `FrameTerminalSnapshot::build_plan`.
  - Added terminal snapshot tests for clean partial frames, dirty row partial
    rebuilds, full dirty mode, resize/full rebuilds, cursor bounds, owned
    preedit handoff, and row ordering.
- `roastty/src/terminal/terminal.rs`
  - Widened the test-only `clear_dirty_for_tests` helper to `pub(crate)`.
  - Added a test-only `set_cursor_position_for_tests` wrapper so renderer tests
    can set cursor state without production-only escape-sequence coupling.
- `issues/0801-roastty-libghostty-rewrite/README.md`
  - Marked Experiment 826 as `Pass`.
  - Updated the renderer tracker to note that live terminal snapshots can feed
    the prepared rebuild planner while renderer-loop orchestration remains open.

Verification:

- `cargo fmt -p roastty`
- `cargo test -p roastty renderer::frame_rebuild::tests::terminal_snapshot -- --nocapture`
  - 7 passed
- `cargo test -p roastty renderer::frame_rebuild -- --nocapture`
  - 96 passed

## Conclusion

The main render path now has its first live terminal-state collection adapter.
The rebuild planner no longer has to be fed exclusively by manually assembled
test inputs: it can consume a terminal snapshot with live grid, dirty-row, row
shape, cursor, dirty-mode, and preedit state.

This still stops before renderer-loop orchestration. A follow-up experiment can
use the snapshot rows to build the row-formatting input bundle or begin
composing the full frame rebuild sequence around `Contents`, overlays, uniforms,
and Metal presentation.

## Completion Review

Codex reviewed the completed implementation and recorded result. The review
found no implementation correctness blockers: `FrameTerminalSnapshot` owns the
expected terminal-derived state, delegates planning to
`FrameRebuildPlan::build`, keeps renderer-loop orchestration out of scope, and
keeps the added terminal helpers test-only.

The review found one workflow blocker: the README experiment index marked the
experiment `Pass` but omitted the required provenance suffix. That was fixed by
updating the Experiment 826 index line to `Pass · Codex/Codex/Codex`.

After that doc-only fix, Codex approved the implementation and recorded result
for the result commit.
