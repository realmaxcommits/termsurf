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

# Experiment 409: the core cell-draw sequence (draw_cells)

## Description

The frame cell buffers (`FrameCells` — Experiment 408) hold the synced
background and cell-text vertices, and the standard pipelines
(`MetalStandardPipelines { bg_color, cell_bg, cell_text }`) compile the three
cell shaders. This experiment composes them into the per-frame **cell-draw
sequence** — `MetalRenderPass::draw_cells` — that issues upstream's three cell
render-pass steps in order: the background color, the opaque cell backgrounds,
and the cell text (sized by the foreground count). This is the core of
`drawFrame`'s cell rendering (the no-background-image path); the background
image, kitty images, the debug overlay, and the custom-shader passes stay
deferred.

## Upstream behavior

In `drawFrame` (`renderer/generic.zig`), within one render pass, the cells are
drawn as three steps (the relevant no-bg-image path):

```zig
// Background color (when there is no background image): a single triangle that
// fills the target, reading the bg cells.
pass.step(.{
    .pipeline = self.shaders.pipelines.bg_color,
    .uniforms = frame.uniforms.buffer,
    .buffers = &.{ null, frame.cells_bg.buffer },
    .draw = .{ .type = .triangle, .vertex_count = 3 },
});

// Opaque cell backgrounds.
pass.step(.{
    .pipeline = self.shaders.pipelines.cell_bg,
    .uniforms = frame.uniforms.buffer,
    .buffers = &.{ null, frame.cells_bg.buffer },
    .draw = .{ .type = .triangle, .vertex_count = 3 },
});

// Text — one instanced quad per foreground cell.
pass.step(.{
    .pipeline = self.shaders.pipelines.cell_text,
    .uniforms = frame.uniforms.buffer,
    .buffers = &.{ frame.cells.buffer, frame.cells_bg.buffer },
    .textures = &.{ frame.grayscale, frame.color },
    .draw = .{ .type = .triangle_strip, .vertex_count = 4, .instance_count = fg_count },
});
```

Between these upstream also draws kitty images (`kitty_below_bg`,
`kitty_below_text`, `kitty_above_text`) and the overlay; those are deferred. The
text step's `instance_count` is the `fg_count` returned by the cell-text sync —
when it is `0`, nothing is drawn.

## Rust mapping (`roastty/src/renderer/metal/render_pass.rs`)

`MetalRenderPass` already has
`step(MetalRenderPassStep { pipeline, buffers, textures, uniforms, draw })` and
short-circuits a step whose `draw.instance_count == 0`. `draw_cells` issues the
three steps from a `FrameCells`, the standard pipelines, the uniform buffer, the
two atlas textures, and the foreground count:

```rust
pub(crate) fn draw_cells(
    &self,
    pipelines: &MetalStandardPipelines,
    uniforms: &ProtocolObject<dyn MTLBuffer>,
    cells: &FrameCells,
    grayscale: &MetalTexture,
    color: &MetalTexture,
    fg_count: usize,
) {
    // Background color: a full-target triangle reading the bg cells.
    self.step(MetalRenderPassStep {
        pipeline: &pipelines.bg_color,
        buffers: &[None, Some(cells.bg_buffer())],
        textures: &[],
        uniforms: Some(uniforms),
        draw: MetalDraw {
            primitive_type: MetalPrimitiveType::Triangle,
            vertex_count: 3,
            instance_count: 1,
        },
    });
    // Opaque cell backgrounds.
    self.step(MetalRenderPassStep {
        pipeline: &pipelines.cell_bg,
        buffers: &[None, Some(cells.bg_buffer())],
        textures: &[],
        uniforms: Some(uniforms),
        draw: MetalDraw {
            primitive_type: MetalPrimitiveType::Triangle,
            vertex_count: 3,
            instance_count: 1,
        },
    });
    // Cell text: one instanced quad per foreground cell.
    self.step(MetalRenderPassStep {
        pipeline: &pipelines.cell_text,
        buffers: &[Some(cells.text_buffer()), Some(cells.bg_buffer())],
        textures: &[Some(grayscale), Some(color)],
        uniforms: Some(uniforms),
        draw: MetalDraw {
            primitive_type: MetalPrimitiveType::TriangleStrip,
            vertex_count: 4,
            instance_count: fg_count,
        },
    });
}
```

The buffer/texture/draw arguments mirror upstream exactly: `[null, cells_bg]`
for the two background steps; `[cells (text), cells_bg]` plus
`[grayscale, color]` for the text step; `triangle`/3 for the backgrounds,
`triangle_strip`/4 with `instance_count = fg_count` for the text. A `fg_count`
of `0` makes `step` skip the text draw (upstream draws nothing for zero
instances).

## Scope / faithfulness notes

- **Ported (bridged)**: `draw_cells` — the three core cell render-pass steps
  (background color, cell backgrounds, cell text) issued in upstream's order
  from a `FrameCells`, the standard pipelines, the uniform buffer, and the atlas
  textures, sized by the foreground count.
- **Faithful**: each step's pipeline, buffer bindings (`[null, cells_bg]` for
  the backgrounds; `[text, cells_bg]` for the text), textures
  (`[grayscale, color]` for the text), primitive type and vertex count, and the
  text step's `instance_count = fg_count` match `drawFrame`; the order is
  bg-color → cell-bg → cell-text; a zero `fg_count` skips the text step
  (upstream draws nothing).
- **Faithful adaptation**: this ports the **no-background-image** path (the
  common case). Upstream draws the background via the `bg_image` pipeline when a
  background image is present, else the `bg_color` step — only the latter is
  ported here. The interleaved kitty-image draws (`kitty_below_bg`,
  `kitty_below_text`, `kitty_above_text`) and the debug overlay are omitted (no
  image subsystem yet).
- **Deferred**: the background-image branch and the kitty/overlay image draws;
  the uniform/atlas sync and the `begin_frame` / target / custom-shader plumbing
  around the pass; the live call site that assembles `Contents`, syncs
  `FrameCells`, and invokes `draw_cells`. (Consumed by a later slice; this
  experiment lands and tests the cell-draw sequence against a render target.)
- No C ABI/header/ABI-inventory change (internal Rust).

## Changes

1. `roastty/src/renderer/metal/render_pass.rs`:
   - add
     `MetalRenderPass::draw_cells(&self, pipelines, uniforms, cells: &FrameCells, grayscale, color, fg_count)`
     issuing the three steps above. Import `FrameCells` (from `buffer`) and
     `MetalStandardPipelines` (from `shaders`).
2. Tests (in `render_pass.rs`, live Metal device):
   - assemble a 1×1 `Contents` with an opaque green background cell and a
     foreground vertex whose grayscale mask is fully on (a red glyph); sync a
     `FrameCells`; run `draw_cells` against a 1×1 render target with the
     cell-text uniforms and a grayscale atlas → the target pixel is the text red
     over the green background (the text step drew the one instance on top of
     the cell background);
   - a `fg_count = 0` case (a `Contents` with only a background cell, no
     foreground): `draw_cells` with `fg_count = 0` draws the background color /
     cell background but **skips** the text step, leaving the background pixel
     (proves the zero-instance text step is skipped);
   - a **bg-color** case that proves the first step runs: a 1×1 `Contents` with
     a **transparent** cell background (`CellBg([0, 0, 0, 0])`), `fg_count = 0`,
     and a nonzero uniform `bg_color`, against a target cleared to a different
     color → the pixel is the uniform `bg_color` (an implementation that omitted
     the `bg_color` step would leave the clear color, since the transparent cell
     background draws nothing over it).
3. Format and test (`cargo fmt`, accept output).

## Verification

```bash
cargo fmt
cargo fmt -- --check
cargo test -p roastty draw_cells
cargo test -p roastty
cargo build -p roastty            # no warnings
rg -n 'ghostty|Ghostty|GHOSTTY' roastty/src/font roastty/src/renderer && exit 1 || true
rg -n 'ghostty|Ghostty|GHOSTTY' roastty/src/lib.rs roastty/include/roastty.h roastty/tests/abi_harness.c && exit 1 || true
git diff --check
```

The experiment **passes** if:

- `draw_cells` issues the background-color, cell-background, and cell-text steps
  in order with upstream's pipeline / buffer / texture / draw parameters, the
  text step sized by `fg_count` — faithful to `drawFrame`'s no-bg-image cell
  path;
- the tests pass (the text-over-background pixel; the `fg_count = 0` skip), and
  the existing tests still pass;
- the background-image branch, the kitty/overlay draws, and the surrounding
  frame plumbing stay deferred;
- `cargo fmt` accepted, `cargo build -p roastty` has no warnings, and
  `cargo test -p roastty` passes with no regressions;
- the no-`ghostty`-name gates and `git diff --check` pass;
- Codex reviews the design before implementation and the result after, with all
  real findings fixed.

The experiment **fails** if the steps are issued out of order or with wrong
bindings, the text step is not sized by `fg_count` (or is not skipped when
zero), or any public C API/ABI changes.

## Design Review

Codex reviewed this design before implementation and **approved** it with one
**Low** finding (no Required), now addressed:

- **Low (addressed):** the planned pixel tests did not actually prove
  `draw_cells` issues the `bg_color` step — in both cases an opaque cell
  background could hide the background-color pass, so an implementation that
  omitted the first step but still drew `cell_bg` and `cell_text` would pass.
  Added a feature-level test where the cell background is **transparent**
  (`CellBg([0, 0, 0, 0])`), `fg_count = 0`, the uniform `bg_color` is nonzero,
  and the target is cleared to a different color → the expected pixel is the
  uniform `bg_color`, directly protecting the first step and the bg-color →
  cell-bg ordering.

Codex confirmed the rest is faithful and well scoped: the proposed bindings, the
draw parameters, the single-pass order, `instance_count = 1` for the background
steps, `instance_count = fg_count` for the text, and the deferral of the
bg-image / kitty / overlay paths.

Review artifacts:

- Prompt: `logs/codex-review/20260604-072610-d409-prompt.md` (design)
- Result: `logs/codex-review/20260604-072610-d409-last-message.md` (design)

## Result

**Result:** Pass

The core cell-draw sequence is now live.

- `roastty/src/renderer/metal/render_pass.rs`:
  `MetalRenderPass::draw_cells(&self, pipelines, uniforms, cells: &FrameCells, grayscale, color, fg_count)`
  issues the three cell steps via `self.step` in order — `bg_color` (buffers
  `[None, Some(bg)]`, no textures, `Triangle`/3/1), `cell_bg` (same bindings),
  and `cell_text` (buffers `[Some(text), Some(bg)]`, textures
  `[grayscale, color]`, `TriangleStrip`/4/`fg_count`). The existing
  zero-instance guard in `step` skips the text step when `fg_count == 0`. Added
  imports `objc2_metal::MTLBuffer`, `buffer::FrameCells`,
  `shaders::MetalStandardPipelines`.

Tests (in `render_pass.rs`, live Metal device, render-to-target + pixel
read-back):

- `draw_cells_renders_text_over_cells` — a 1×1 `Contents` (transparent bg cell +
  a masked red glyph) synced into `FrameCells` (`fg_count == 1`); `draw_cells`
  over a 2×2 target → the pixel grid `[red, transparent, transparent, red]` (the
  known cell-text mask, routed through `draw_cells`).
- `draw_cells_draws_cell_background_and_skips_zero_foreground` — a 1×1
  `Contents` (opaque green bg cell, no foreground; `fg_count == 0`) → all pixels
  green `[0, 255, 0, 255]` (the cell-bg step drew, the text step was skipped).
- `draw_cells_draws_background_color_step` — a 1×1 `Contents` (transparent bg
  cell, no foreground), a nonzero uniform `bg_color`, a different clear color →
  all pixels `[128, 64, 32, 255]` (the bg-color step ran; an omitted bg-color
  step would leave the clear color).

Gate results:

- `cargo fmt -p roastty` accepted; `--check` clean.
- `cargo test -p roastty` → 2877 passed, 0 failed (+3, no regressions).
- `cargo build -p roastty` → no warnings.
- No-`ghostty`-name gates (font + renderer + `lib.rs`/header/`abi_harness.c`)
  clean; `git diff --check` clean.

## Conclusion

The renderer bridge now runs the full cell pipeline end-to-end on the GPU:
per-cell assembly (`rebuildCells`) → `Contents` read views → the upload
primitives → `FrameCells` → `draw_cells` (bg-color, cell-bg, cell-text). A small
`Contents` assembled in a test renders to a Metal target with the expected
pixels. The remaining renderer-bridge work is the surrounding per-frame plumbing
— the uniform/atlas sync, the `begin_frame` / target acquisition / custom-shader
passes, the background-image and kitty/overlay image draws — and the live call
site that assembles `Contents` from terminal state, syncs `FrameCells`, and
invokes `draw_cells` each frame (which depends on the live render `State`); plus
the `rebuild_viewport` cursor/preedit assembly.

## Completion Review

Codex reviewed the completed implementation and result and **approved** with
**no findings**. It confirmed the implementation matches the approved design and
upstream's no-background-image cell sequence: `draw_cells` issues the steps in
the correct order (`bg_color` → `cell_bg` → `cell_text`) with upstream's
bindings and draw parameters (`[None, Some(cells_bg)]` for both background
steps; `[Some(text), Some(cells_bg)]` plus `[grayscale, color]` for the text;
`Triangle`/3/1 for the backgrounds; `TriangleStrip`/4/`fg_count` for the text),
and the existing zero-instance guard correctly skips the text draw when
`fg_count == 0`. It confirmed the Low finding is addressed —
`draw_cells_draws_background_color_step` (transparent cell background, no
foreground, nonzero uniform bg color, different clear color) would fail if the
bg-color step were omitted — and that the other two tests exercise the text
rendering and the cell-background / zero-foreground paths. Internal Rust only —
no public C ABI/header impact; nothing needed to change before the result
commit.

Review artifacts:

- Prompt: `logs/codex-review/20260604-073220-r409-prompt.md` (result)
- Result: `logs/codex-review/20260604-073220-r409-last-message.md` (result)
