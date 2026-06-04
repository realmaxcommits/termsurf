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

# Experiment 414: the frame draw (draw_frame)

## Description

The two halves of `drawFrame`'s cell rendering are now in place: `FrameState`
(Experiment 413) owns and syncs the per-frame GPU resources (uniforms, cells,
atlas textures), and `MetalRenderPass::draw_cells` (Experiment 409) issues the
bg-color / cell-bg / cell-text steps from those resources. This experiment joins
them with a thin `MetalRenderPass::draw_frame` that draws a synced
`FrameState`'s cells in one call (binding the frame's own uniform buffer, cell
buffers, and atlas textures), and — the real value — an **end-to-end integration
test** that assembles a `Contents`, syncs a `FrameState`, and renders it to an
offscreen Metal target, asserting the produced pixels. The live drawable/target
acquisition (`begin_frame`) and the bg-image / kitty / overlay / custom-shader
passes stay deferred.

## Upstream behavior

In `drawFrame` (`renderer/generic.zig`), after the per-frame sync (uniforms,
cells, atlas textures — Experiment 413), the cells are drawn within the frame's
render pass using the frame's own resources:

```zig
var pass = frame_ctx.renderPass(&.{ … frame.target … });
// bg-color (no bg image) / cell-bg / cell-text steps, all bound from
// frame.uniforms, frame.cells_bg, frame.cells, frame.grayscale, frame.color,
// sized by fg_count …
```

So the cell draw reads exactly the per-frame `FrameState` resources that were
just synced.

## Rust mapping (`roastty/src/renderer/metal/render_pass.rs`)

`draw_frame` forwards to `draw_cells` (Experiment 409) with the `FrameState`'s
own resources:

```rust
pub(crate) fn draw_frame(
    &self,
    pipelines: &MetalStandardPipelines,
    state: &FrameState,
    fg_count: usize,
) {
    self.draw_cells(
        pipelines,
        state.uniforms_buffer(),
        state.cells(),
        state.grayscale_texture(),
        state.color_texture(),
        fg_count,
    );
}
```

The binding is exactly upstream's (the frame's uniform buffer, cell buffers, and
grayscale/color atlas textures), and `fg_count` is the value returned by
`FrameState::sync`. `draw_frame` is the cell-drawing portion of `drawFrame`'s
pass body, driven by a `FrameState`.

## Scope / faithfulness notes

- **Ported (bridged)**: `MetalRenderPass::draw_frame` — the cell draw of a
  synced `FrameState` (binding its uniform buffer, cell buffers, and atlas
  textures), forwarding to `draw_cells`. Joins Experiment 413 (sync) and
  Experiment 409 (draw) into the per-frame cell render.
- **Faithful**: the draw binds the frame's own resources
  (`uniforms_buffer`/`cells`/`grayscale_texture`/`color_texture`) sized by
  `fg_count` — exactly what `drawFrame` binds after the sync; the steps and
  their order are `draw_cells`' (bg-color → cell-bg → cell-text).
- **Faithful adaptation**: `draw_frame` is a thin convenience over `draw_cells`
  that supplies the `FrameState` accessors, so the render-pass step logic stays
  in one place. The end-to-end test drives the full assemble → sync → draw path
  against an offscreen render target (production acquires the target from the
  live drawable).
- **Deferred**: the live frame-target acquisition (`begin_frame` / the
  drawable), the bg-image / kitty / overlay image draws, the custom-shader
  passes, and the call site that assembles `Contents` and the uniforms from the
  render `State` each frame. (This experiment lands and tests the
  `FrameState`-driven cell draw.)
- No C ABI/header/ABI-inventory change (internal Rust).

## Changes

1. `roastty/src/renderer/metal/render_pass.rs`:
   - add
     `MetalRenderPass::draw_frame(&self, pipelines, state: &FrameState, fg_count)`
     forwarding to `draw_cells` with the `FrameState` accessors. Import
     `FrameState` (from `frame`).
2. Tests (in `render_pass.rs`, live Metal device, end-to-end):
   - assemble a 1×1 `Contents` with an **opaque green** background cell and **no
     foreground**, into a `FrameState` (built with a grayscale `Atlas` and a
     `Bgra` color `Atlas`); `FrameState::sync` → `fg_count == 0`; then render
     the frame to a 2×2 offscreen target (cleared to transparent) via
     `draw_frame` → all pixels are green (`[0, 255, 0, 255]`), proving the
     cell-background buffer synced into the `FrameState` binds and renders
     through `draw_frame` (the bg-color step draws transparent, the cell-bg step
     draws the green cell, and the text step is skipped at `fg_count == 0`).
     Using a cell background — whose bytes came from `Contents` through
     `FrameState::sync` — proves the frame's own cell buffer is bound (not a
     stray buffer), and exercises the uniform binding (the cell-bg shader reads
     the frame's uniforms for the grid layout).
   - a **nonzero-foreground** smoke test exercising the text buffer + grayscale
     atlas-texture forwarding: a grayscale `Atlas` (size 8) with a **reserved**
     2×2 region `set` fully on (`[255; 4]`), and a 1×1 `Contents` whose
     foreground vertex samples that region (`glyph_pos = [region.x, region.y]`,
     `glyph_size = [2, 2]`, the Experiment 409 cell-filling `bearings = [0, 2]`,
     red) over a transparent background cell; `FrameState::sync` →
     `fg_count == 1`; `draw_frame` to a 2×2 target → all pixels are red
     (`[0, 0, 255, 255]`), proving the frame's cell-text buffer and grayscale
     atlas texture bind through `draw_frame` (the region origin comes from the
     `reserve`, not a hardcoded atlas coordinate, and the fully-on mask avoids
     depending on a partial pattern).
3. Format and test (`cargo fmt`, accept output).

## Verification

```bash
cargo fmt
cargo fmt -- --check
cargo test -p roastty draw_frame
cargo test -p roastty
cargo build -p roastty            # no warnings
rg -n 'ghostty|Ghostty|GHOSTTY' roastty/src/font roastty/src/renderer && exit 1 || true
rg -n 'ghostty|Ghostty|GHOSTTY' roastty/src/lib.rs roastty/include/roastty.h roastty/tests/abi_harness.c && exit 1 || true
git diff --check
```

The experiment **passes** if:

- `draw_frame` draws a synced `FrameState`'s cells by binding its uniform
  buffer, cell buffers, and atlas textures (sized by `fg_count`), forwarding to
  `draw_cells` — faithful to `drawFrame`'s cell draw;
- the end-to-end test passes (assemble → `FrameState::sync` → `draw_frame` → the
  expected rendered pixels), and the existing tests still pass;
- the live target acquisition and the deferred image/custom-shader passes stay
  deferred;
- `cargo fmt` accepted, `cargo build -p roastty` has no warnings, and
  `cargo test -p roastty` passes with no regressions;
- the no-`ghostty`-name gates and `git diff --check` pass;
- Codex reviews the design before implementation and the result after, with all
  real findings fixed.

The experiment **fails** if `draw_frame` binds the wrong resources, the rendered
pixels are wrong, or any public C API/ABI changes.

## Design Review

Codex reviewed this design before implementation and **approved** it with one
**Low** finding (no Required), now addressed:

- **Low (addressed):** the planned end-to-end test (green cell-bg,
  `fg_count == 0`) proves the uniform and background-cell binding but does not
  exercise the text-buffer / atlas-texture forwarding (the text step is skipped
  at `fg_count == 0`), even though the `draw_frame` claim includes
  `state.cells().text_buffer()`, `state.grayscale_texture()`, and
  `state.color_texture()`. A nonzero-foreground smoke test was added: a
  grayscale `Atlas` with a **reserved** 2×2 region `set` fully on, a foreground
  vertex sampling that region (its origin taken from the `reserve`, not
  hardcoded), and a fully-on mask → `draw_frame` renders an all-red glyph,
  exercising the text-buffer + grayscale-texture forwarding without depending on
  a fragile partial-mask pattern.

Codex confirmed the `draw_frame` shape is otherwise faithful: a render-pass
method is the right home, the dependency direction is acyclic (`frame.rs` does
not import `render_pass`), forwarding to `draw_cells` keeps the step logic
centralized, and binding the `FrameState` resources sized by `fg_count` matches
upstream's post-sync cell draw; the deferred bg-image / kitty / overlay /
custom-shader / live frame-target plumbing are reasonable scope boundaries.

Review artifacts:

- Prompt: `logs/codex-review/20260604-080328-d414-prompt.md` (design)
- Result: `logs/codex-review/20260604-080328-d414-last-message.md` (design)
