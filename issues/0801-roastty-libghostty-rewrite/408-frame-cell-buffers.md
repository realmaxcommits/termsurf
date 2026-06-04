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

# Experiment 408: the frame cell buffers (FrameCells)

## Description

The two cell-upload primitives (`MetalBuffer::sync`, `sync_from_array_lists` —
Experiment 406) and the two `Contents` read views (`bg_cells`, `fg_rows` —
Experiment 407) are now in place. This experiment composes them into the
frame-owned cell-buffer pair, `FrameCells`: a persistent background buffer and
cell-text buffer that sync from a `Contents` in one call, returning the
foreground cell count — exactly what upstream's per-frame state holds and what
`drawFrame` does:

```zig
try frame.cells_bg.sync(self.cells.bg_cells);
const fg_count = try frame.cells.syncFromArrayLists(self.cells.fg_rows.lists);
```

## Upstream behavior

Each frame in upstream's `FrameState` owns two cell buffers, both created with
an initial capacity of one element and the shared buffer options
(`bgBufferOptions == fgBufferOptions == bufferOptions` — same device, same
resource options):

```zig
var cells = try CellTextBuffer.init(api.fgBufferOptions(), 1);     // foreground
var cells_bg = try CellBgBuffer.init(api.bgBufferOptions(), 1);    // background
```

Per frame, `drawFrame` syncs the assembled `Contents` into them: the background
slice 1:1 via `sync`, and the foreground row lists concatenated via
`syncFromArrayLists`, whose return is the foreground vertex count (`fg_count`)
used later to size the cell-text draw.

## Rust mapping (`roastty/src/renderer/metal/buffer.rs`)

`FrameCells` owns the two typed `MetalBuffer`s and composes the primitives:

```rust
pub(crate) struct FrameCells {
    cells_bg: MetalBuffer<CellBg>,
    cells: MetalBuffer<CellTextVertex>,
}

impl FrameCells {
    /// Create the frame's cell buffers, each at the initial capacity of one
    /// element (upstream `init(api.{bg,fg}BufferOptions(), 1)`). Background and
    /// foreground share the same buffer options (upstream
    /// `bgBufferOptions == fgBufferOptions`).
    pub(crate) fn new(options: MetalBufferOptions<'_>) -> Result<Self, MetalBufferError> {
        let cells_bg = MetalBuffer::new(options, 1)?;
        let cells = MetalBuffer::new(options, 1)?;
        Ok(Self { cells_bg, cells })
    }

    /// Sync the assembled `Contents` into the GPU buffers — the background slice
    /// 1:1, the foreground row lists concatenated — returning the foreground
    /// vertex count (upstream `drawFrame`: `cells_bg.sync(bg_cells)` then
    /// `fg_count = cells.syncFromArrayLists(fg_rows.lists)`).
    pub(crate) fn sync(
        &mut self,
        options: MetalBufferOptions<'_>,
        contents: &Contents,
    ) -> Result<usize, MetalBufferError> {
        self.cells_bg.sync(options, contents.bg_cells())?;
        self.cells.sync_from_array_lists(options, contents.fg_rows())
    }

    /// The background cell buffer (bound at the bg / cell-bg draw steps).
    pub(crate) fn bg_buffer(&self) -> &ProtocolObject<dyn MTLBuffer> {
        self.cells_bg.buffer()
    }

    /// The cell-text (foreground) buffer (bound at the cell-text draw step).
    pub(crate) fn text_buffer(&self) -> &ProtocolObject<dyn MTLBuffer> {
        self.cells.buffer()
    }
}
```

Background and foreground share one `options` argument because upstream's
`bgBufferOptions` and `fgBufferOptions` are the same `bufferOptions` (the
per-frame device + resource options). The `sync` order matches `drawFrame`
(background first, then foreground), and the return is the foreground count from
`sync_from_array_lists`.

## Scope / faithfulness notes

- **Ported (bridged)**: `FrameCells` — the frame-owned background + cell-text
  buffer pair and its `sync(contents) -> fg_count`, composing Experiment 406's
  upload primitives with Experiment 407's `Contents` read views. This is
  upstream's `frame.cells_bg` / `frame.cells` and the `drawFrame` cell sync.
- **Faithful**: both buffers start at capacity one (upstream `init(..., 1)`);
  the sync writes the background 1:1 then concatenates the foreground row lists
  (reserved cursor lists included, so the cursor glyph is uploaded), returning
  the foreground vertex count; background and foreground share the buffer
  options (upstream `bgBufferOptions == fgBufferOptions`).
- **Faithful adaptation**: roastty's `MetalBuffer` takes the options per call
  (for the device handle) rather than storing them, so `FrameCells::sync`
  threads one `options` to both buffers — matching the single upstream
  `bufferOptions`. The `bg_buffer` / `text_buffer` accessors expose the
  `MTLBuffer`s for the later draw-step binding (the draw wiring itself stays
  deferred).
- **Deferred**: the per-frame draw that binds these buffers and issues the
  bg-color / cell-bg / cell-text steps (the render-pass step wiring), the
  uniform/atlas sync, and the live `Contents` assembly call. (Consumed by a
  later slice; this experiment lands and tests the buffer pair and its sync.)
- No C ABI/header/ABI-inventory change (internal Rust); the Metal buffer module
  is already `#![allow(dead_code)]`.

## Changes

1. `roastty/src/renderer/metal/buffer.rs`:
   - add a `FrameCells` struct (the `cells_bg: MetalBuffer<CellBg>` and
     `cells: MetalBuffer<CellTextVertex>` pair) with `new(options)`,
     `sync(options, contents) -> Result<usize, MetalBufferError>`, and
     `bg_buffer()` / `text_buffer()` accessors. Imports
     `crate::renderer::cell::Contents`.
2. Tests (in `buffer.rs`, live Metal device):
   - assemble a small `Contents` (a 2×1 grid: two background cells, a foreground
     vertex on the real row, and a block cursor glyph in the reserved list);
     `FrameCells::new` then `sync` → the return is the total foreground vertex
     count across **all** lists (`2`: the real-row vertex **and** the cursor
     glyph), the background buffer holds the two `bg_cells` bytes, and the
     cell-text buffer holds the concatenated foreground vertices (cursor glyph
     first — reserved list `0` — then the real-row vertex);
   - a reallocation check: syncing a larger `Contents` grows the buffers and the
     data stays correct (covered by reusing the primitives, asserted via the
     foreground count and read-back bytes).
3. Format and test (`cargo fmt`, accept output).

## Verification

```bash
cargo fmt
cargo fmt -- --check
cargo test -p roastty frame_cells
cargo test -p roastty
cargo build -p roastty            # no warnings
rg -n 'ghostty|Ghostty|GHOSTTY' roastty/src/font roastty/src/renderer && exit 1 || true
rg -n 'ghostty|Ghostty|GHOSTTY' roastty/src/lib.rs roastty/include/roastty.h roastty/tests/abi_harness.c && exit 1 || true
git diff --check
```

The experiment **passes** if:

- `FrameCells::new` creates both buffers at capacity one, and `sync` writes the
  background 1:1 then concatenates the foreground row lists (reserved cursor
  lists included), returning the foreground vertex count — faithful to
  upstream's per-frame cell buffers and the `drawFrame` cell sync;
- the tests pass (the foreground count counts the cursor glyph; the background
  bytes match; the cell-text bytes are the concatenation, cursor glyph first),
  and the existing tests still pass;
- the draw-step wiring and the rest of the per-frame sync stay deferred;
- `cargo fmt` accepted, `cargo build -p roastty` has no warnings, and
  `cargo test -p roastty` passes with no regressions;
- the no-`ghostty`-name gates and `git diff --check` pass;
- Codex reviews the design before implementation and the result after, with all
  real findings fixed.

The experiment **fails** if the foreground count drops the cursor glyph, the
background and foreground are synced out of order or with the wrong options, the
buffers do not start at capacity one, or any public C API/ABI changes.

## Design Review

Codex reviewed this design before implementation and **approved** it with **no
findings**. It confirmed the design is faithful to upstream's per-frame cell
buffers and the `drawFrame` sync: both buffers starting at capacity `1` matches
upstream initialization, and `sync` doing background first then foreground
list-concatenation — with the returned `usize` coming from
`sync_from_array_lists` — matches the upstream `fg_count` behavior. It confirmed
that using one `MetalBufferOptions` for both buffers is correct (upstream's
bg/fg buffer options are the same shared options, and roastty's buffer API takes
options per creation/sync call), and that the cursor ordering claim holds:
`Contents::fg_rows()` returns list order
`[0 cursor-reserved, real rows…, last cursor-reserved]`, so a block cursor in
list `0` uploads before the row text and non-block cursor styles in the last
list upload after it. It judged `buffer.rs` an acceptable home (a Metal buffer
composition object; the dependency on `Contents` is one-way, no cycle) and the
planned tests sufficient (count, background bytes, foreground concatenation
including the cursor, and growth behavior), with the draw-binding follow-up
reasonable.

Review artifacts:

- Prompt: `logs/codex-review/20260604-071821-d408-prompt.md` (design)
- Result: `logs/codex-review/20260604-071821-d408-last-message.md` (design)

## Result

**Result:** Pass

The frame cell buffers are now live.

- `roastty/src/renderer/metal/buffer.rs`: a `FrameCells` struct owning
  `cells_bg: MetalBuffer<CellBg>` and `cells: MetalBuffer<CellTextVertex>`, with
  `new(options)` (both buffers at capacity one),
  `sync(options, contents) -> Result<usize, MetalBufferError>` (background 1:1
  via `sync`, then foreground concatenated via `sync_from_array_lists`,
  returning the foreground vertex count), and `bg_buffer()` / `text_buffer()`
  accessors exposing the `MTLBuffer`s for the later draw-step binding. Added
  `use crate::renderer::cell::Contents;`.

Tests (in `buffer.rs`, live Metal device):

- `frame_cells_sync_uploads_background_and_foreground` — a 2×1 `Contents` (two
  background cells, a foreground vertex on the real row, a block cursor glyph
  via `set_cursor`) → `sync` returns `2` (the cursor glyph **and** the real-row
  vertex); the background buffer holds the two `bg_cells` bytes (capacity grew
  to `4`); the cell-text buffer holds the concatenation, cursor glyph first
  (reserved list `0`) then the real-row vertex (capacity grew to `4`).
- `frame_cells_sync_grows_for_larger_contents` — a 3×1 `Contents` with three
  foreground vertices (no cursor) → `sync` returns `3`, the cell-text buffer
  holds `[v0, v1, v2]` and grew to capacity `6`.

Gate results:

- `cargo fmt -p roastty` accepted; `--check` clean.
- `cargo test -p roastty` → 2874 passed, 0 failed (+2, no regressions).
- `cargo build -p roastty` → no warnings.
- No-`ghostty`-name gates (font + renderer + `lib.rs`/header/`abi_harness.c`)
  clean; `git diff --check` clean.

## Conclusion

The frame now owns its background + cell-text buffers and syncs an assembled
`Contents` into them in one call, returning the foreground vertex count — a
faithful port of upstream's per-frame cell buffers and the `drawFrame` cell sync
(generic.zig:1560–1561). The cell-upload path is now complete end-to-end:
per-cell assembly (`rebuildCells`) → `Contents` read views → the two upload
primitives → the frame buffer pair. The next renderer-bridge slice is the
per-frame draw that binds these buffers (`bg_buffer` / `text_buffer`) and issues
the bg-color / cell-bg / cell-text render-pass steps, plus the uniform/atlas
sync — which depends on the live frame / render-pass plumbing; the
`rebuild_viewport` cursor/preedit assembly and the live `Contents` assembly call
stay deferred.

## Completion Review

Codex reviewed the completed implementation and result and **approved** with
**no findings**. It confirmed the implementation matches the approved design and
upstream shape: `FrameCells::new` creates both typed buffers at capacity `1`;
`sync` uploads the background first via `contents.bg_cells()` then the
foreground via `contents.fg_rows()`, returning the `sync_from_array_lists`
foreground count; and because `fg_rows()` exposes the full list array, the count
and upload include the block cursor in reserved list `0` (the byte-order test
verifies the cursor-first concatenation). It confirmed that using the same
`MetalBufferOptions` for both buffers is faithful to upstream's shared bg/fg
buffer options, that the `bg_buffer()` / `text_buffer()` accessors expose the
underlying `MTLBuffer`s for later draw binding, and that the tests cover the
background bytes, foreground bytes, cursor inclusion, and growth behavior.
Internal Rust only — no public C ABI/header impact; nothing needed to change
before the result commit.

Review artifacts:

- Prompt: `logs/codex-review/20260604-072135-r408-prompt.md` (result)
- Result: `logs/codex-review/20260604-072135-r408-last-message.md` (result)
