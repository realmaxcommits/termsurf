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

# Experiment 413: the per-frame resource state (FrameState)

## Description

The per-frame GPU resources are now all ported as individual units: the cell
buffers (`FrameCells` — Experiment 408), the atlas textures with the modified
gate (`FrameAtlasTexture` — Experiment 412), and the uniform buffer (a
`MetalBuffer<MetalUniforms>` synced via `sync`). Upstream owns all of these
together in one per-frame `FrameState` and syncs them as a block at the top of
`drawFrame`. This experiment ports that owner, `FrameState`, with a single
`sync` that runs upstream's full per-frame sync — uniforms, cells, and both
atlas textures — returning the foreground cell count. The frame-target
acquisition and the draw issue (`draw_cells`) stay deferred (they need the live
`begin_frame` plumbing).

## Upstream behavior

Each frame in upstream's `FrameState` owns the uniform buffer, the two cell
buffers, and the grayscale/color atlas textures; `drawFrame` syncs them as a
block:

```zig
try frame.uniforms.sync(&.{self.uniforms});
try frame.cells_bg.sync(self.cells.bg_cells);
const fg_count = try frame.cells.syncFromArrayLists(self.cells.fg_rows.lists);
// … then the two atlas `texture:` modified-gates (Experiment 412) …
```

So one frame state holds every per-frame GPU resource, and the sync is uniforms
→ cells (bg then fg, returning `fg_count`) → atlas textures.

## Rust mapping (`roastty/src/renderer/metal/frame.rs`, new)

`FrameState` bundles the four resources and composes their syncs:

```rust
pub(crate) struct FrameState {
    uniforms: MetalBuffer<MetalUniforms>,
    cells: FrameCells,
    grayscale: FrameAtlasTexture,
    color: FrameAtlasTexture,
}

impl FrameState {
    pub(crate) fn new(
        options: MetalBufferOptions<'_>,
        grayscale_atlas: &Atlas,
        color_atlas: &Atlas,
    ) -> Result<Self, FrameStateError> {
        let device = options.device;
        let storage = options.resource_options.storage_mode;
        Ok(Self {
            uniforms: MetalBuffer::new(options, 1)?,
            cells: FrameCells::new(options)?,
            grayscale: FrameAtlasTexture::new(device, storage, grayscale_atlas)?,
            color: FrameAtlasTexture::new(device, storage, color_atlas)?,
        })
    }

    /// Sync the per-frame GPU resources (upstream's `drawFrame` sync block):
    /// the uniforms, the cells (background + foreground), and both atlas
    /// textures (each gated on its `modified` counter). Returns the foreground
    /// cell count.
    pub(crate) fn sync(
        &mut self,
        options: MetalBufferOptions<'_>,
        uniforms: &MetalUniforms,
        contents: &Contents,
        grayscale_atlas: &Atlas,
        color_atlas: &Atlas,
    ) -> Result<usize, FrameStateError> {
        let device = options.device;
        let storage = options.resource_options.storage_mode;
        self.uniforms.sync(options, std::slice::from_ref(uniforms))?;
        let fg_count = self.cells.sync(options, contents)?;
        self.grayscale.sync_if_modified(device, storage, grayscale_atlas)?;
        self.color.sync_if_modified(device, storage, color_atlas)?;
        Ok(fg_count)
    }

    pub(crate) fn uniforms_buffer(&self) -> &ProtocolObject<dyn MTLBuffer> { self.uniforms.buffer() }
    pub(crate) fn cells(&self) -> &FrameCells { &self.cells }
    pub(crate) fn grayscale_texture(&self) -> &MetalTexture { self.grayscale.texture() }
    pub(crate) fn color_texture(&self) -> &MetalTexture { self.color.texture() }
}
```

The sync order is upstream's (uniforms → cells → atlas textures); the uniforms
are one element (`from_ref`); the cells return `fg_count`; the atlas textures
use the modified gate (Experiment 412). `device` and `storage_mode` are taken
from the shared `options` (upstream's per-frame buffer options + storage mode).
A `FrameStateError` unifies `MetalBufferError` (uniforms/cells) and
`MetalTextureError` (atlas) via `From`.

## Scope / faithfulness notes

- **Ported (bridged)**: `FrameState` — the per-frame owner of the uniform
  buffer, the cell buffers (`FrameCells`), and the grayscale/color atlas
  textures (`FrameAtlasTexture`), with a `sync` running upstream's `drawFrame`
  sync block (uniforms, cells, atlas textures) and returning `fg_count`.
- **Faithful**: the resources owned and the sync order (uniforms → cells bg+fg →
  atlas textures); the uniforms synced as one element; `fg_count` from the cell
  sync; the atlas modified-gates (Experiment 412).
- **Faithful adaptation**: `device` / `storage_mode` come from the shared
  `MetalBufferOptions` (upstream reads them off the renderer); a
  `FrameStateError` enum unifies the buffer and texture error types for `?`
  (upstream's single error union). The accessors expose the buffers/textures for
  the later `draw_cells` binding.
- **Deferred**: the frame-target acquisition (`begin_frame`), the render-pass
  setup and the `draw_cells` issue, the custom-shader/bg-image/kitty/overlay
  passes, and the live call site that assembles `Contents` and the uniforms from
  the render `State` each frame. (Consumed by a later slice; this experiment
  lands and tests the resource owner + its sync.)
- No C ABI/header/ABI-inventory change (internal Rust).

## Changes

1. `roastty/src/renderer/metal/frame.rs` (new): the `FrameState` struct and a
   `FrameStateError` (`Buffer(MetalBufferError)` / `Texture(MetalTextureError)`
   with `From` impls), per the mapping above.
2. `roastty/src/renderer/metal/mod.rs`: add `pub(crate) mod frame;`.
3. Tests (in `frame.rs`, live Metal device):
   - assemble a grayscale `Atlas` and a `Bgra` color `Atlas` (each with a
     reserved pixel `set`), a small `Contents` (a background cell + a foreground
     vertex), and a `MetalUniforms`; `FrameState::new` then `sync` → the return
     is the foreground count (`1`); the uniforms buffer holds the uniforms
     bytes; the cells' background and cell-text buffers hold the synced data;
     and the grayscale and color textures hold their atlas data (`read_bytes()`
     equals each `atlas.data()`).
4. Format and test (`cargo fmt`, accept output).

## Verification

```bash
cargo fmt
cargo fmt -- --check
cargo test -p roastty frame_state
cargo test -p roastty
cargo build -p roastty            # no warnings
rg -n 'ghostty|Ghostty|GHOSTTY' roastty/src/font roastty/src/renderer && exit 1 || true
rg -n 'ghostty|Ghostty|GHOSTTY' roastty/src/lib.rs roastty/include/roastty.h roastty/tests/abi_harness.c && exit 1 || true
git diff --check
```

The experiment **passes** if:

- `FrameState` owns the uniform buffer, the cell buffers, and the two atlas
  textures, and `sync` runs upstream's sync block (uniforms → cells → atlas
  textures) returning `fg_count` — faithful to `drawFrame`'s per-frame sync;
- the test passes (the foreground count; the uniforms, cell, and atlas-texture
  data all correct after one sync), and the existing tests still pass;
- the frame-target acquisition and the `draw_cells` issue stay deferred;
- `cargo fmt` accepted, `cargo build -p roastty` has no warnings, and
  `cargo test -p roastty` passes with no regressions;
- the no-`ghostty`-name gates and `git diff --check` pass;
- Codex reviews the design before implementation and the result after, with all
  real findings fixed.

The experiment **fails** if the sync order differs from upstream, a resource is
not synced (or synced wrong), the `fg_count` is wrong, or any public C API/ABI
changes.

## Design Review

Codex reviewed this design before implementation and **approved** it with **no
findings**. It confirmed the design is faithful to upstream's per-frame sync
block: `FrameState` owns the same resource categories (one uniform buffer, the
frame cell buffers, the grayscale/color atlas textures); the sync order
`uniforms → cells → atlas textures` matches `drawFrame`; and syncing the
uniforms via `std::slice::from_ref(uniforms)` is the right Rust equivalent of
upstream `sync(&.{self.uniforms})`. It confirmed returning `fg_count` from
`FrameCells::sync` is correct, and that taking `device` plus `storage_mode` from
the shared `MetalBufferOptions` is sound (`options` is `Copy`, the buffer syncs
still receive the full options, and the atlas textures only need
device/storage). It judged `FrameStateError` an appropriate wrapper for
composing the buffer and texture operations, agreed a dedicated `frame.rs`
module is the right home (the object depends on buffer, texture, shaders, and
cell/atlas state), noted the single-load atlas gate was already scoped/approved
in Experiment 412 (no need to re-flag), and judged the planned first-sync test
sufficient for this slice.

Review artifacts:

- Prompt: `logs/codex-review/20260604-075539-d413-prompt.md` (design)
- Result: `logs/codex-review/20260604-075539-d413-last-message.md` (design)

## Result

**Result:** Pass

The per-frame resource state is now live.

- `roastty/src/renderer/metal/frame.rs` (new module, registered in `mod.rs`): a
  `FrameStateError` enum (`Buffer(MetalBufferError)` /
  `Texture(MetalTextureError)` with `From` impls) and a `FrameState` struct
  owning the uniform buffer (`MetalBuffer<MetalUniforms>`), the cell buffers
  (`FrameCells`), and the grayscale/color atlas textures (`FrameAtlasTexture`).
  `new` creates them (the uniform/cell buffers at capacity one; the atlas
  textures sized to their atlases, device/storage from the shared `options`);
  `sync` runs upstream's `drawFrame` sync block — uniforms (one element via
  `from_ref`) → cells (background + foreground) → both atlas textures (each
  modified-gated) — and returns the foreground cell count. Accessors expose the
  uniform buffer, the cells, and the two textures for the later `draw_cells`
  binding.

Test (in `frame.rs`, live Metal device): a grayscale atlas and a `Bgra` color
atlas (each with a written pixel), a 1×1 `Contents` (an explicit background cell

- a foreground vertex), and a `MetalUniforms` → `FrameState::new` then `sync`
  returns `1` (the foreground count); the uniform buffer holds the uniforms
  bytes; the cells' background and cell-text buffers hold the background cell
  and the vertex; and the grayscale and color textures hold their
  `atlas.data()`. (The otherwise-private buffer bytes are read via a local
  `contents()` helper, sound for the shared-storage buffers.)

Gate results:

- `cargo fmt -p roastty` accepted; `--check` clean.
- `cargo test -p roastty` → 2887 passed, 0 failed (+1, no regressions).
- `cargo build -p roastty` → no warnings.
- No-`ghostty`-name gates (font + renderer + `lib.rs`/header/`abi_harness.c`)
  clean; `git diff --check` clean.

## Conclusion

The renderer bridge now has a single per-frame resource owner that runs the
whole `drawFrame` sync block in upstream's order and hands back the foreground
count plus the buffers/textures the draw needs. Every per-frame GPU building
block is now in place: `FrameState` (uniforms + cells + atlas textures), the
sync, and `draw_cells` (the render-pass sequence). The remaining renderer-bridge
work is the outer per-frame loop that ties `FrameState::sync` to `draw_cells` —
acquiring the frame target (`begin_frame`), opening the render pass, and binding
`state.uniforms_buffer()` / `state.cells()` / `state.grayscale_texture()` /
`state.color_texture()` with `fg_count` — which depends on the live render
`State` and target plumbing; plus the deferred bg-image / kitty / overlay draws,
the custom-shader passes, and the `rebuild_viewport` cursor/preedit assembly.

## Completion Review

Codex reviewed the completed implementation and result and **approved** with
**no findings**. It confirmed the implementation matches the approved design and
the upstream sync block: `FrameState` owns the uniform buffer, `FrameCells`, and
both atlas textures; `sync` runs in the correct order (uniforms → cells →
grayscale atlas → color atlas); the uniforms are synced as a single element; and
the returned `fg_count` is the cell-text upload count from `FrameCells`. It
judged the test to genuinely exercise all four resource categories and the local
raw `MTLBuffer` reads sound (shared storage, exactly the synced byte lengths).
Internal Rust only — no public C ABI/header impact; nothing needed to change
before the result commit.

Review artifacts:

- Prompt: `logs/codex-review/20260604-075918-r413-prompt.md` (result)
- Result: `logs/codex-review/20260604-075918-r413-last-message.md` (result)
