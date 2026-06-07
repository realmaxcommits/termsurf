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

# Experiment 814: Metal Frame Compositor

## Description

Add the first reusable live-frame orchestration object for Roastty's Metal
renderer. Experiments 808-813 built the pieces needed for one frame: standard
pipelines, render passes, frame-state GPU sync, IOSurface-backed targets, and
IOSurfaceLayer presentation. What is still missing is the object that wires
those pieces together into a frame-sized render target, draws a synced
`FrameState`, commits the command buffer, and presents the resulting IOSurface.

This experiment does not port the full upstream `generic.zig` render loop. It
does not rebuild terminal cells from `terminal.RenderState`, upload images,
drive custom shaders, own the renderer thread, or implement redraw pacing. It
adds the narrow Metal frame compositor that later renderer integration can call
once cells/uniforms/atlases have already been prepared.

## Changes

- `roastty/src/renderer/metal/compositor.rs`
  - Add a `MetalFrameCompositor` that owns:
    - a retained Metal command queue,
    - standard Metal pipelines for the target pixel format,
    - one `FrameState`,
    - a `MetalIOSurfaceLayer`, and
    - an optional current `MetalTarget`.
  - Add `MetalFrameCompositorOptions` with device, initial width/height, target
    pixel format, storage mode, buffer resource options, and grayscale / color
    atlases.
  - Add `MetalFrameInput` with width/height, uniforms, prepared cell contents,
    grayscale / color atlases, and `contents_scale`.
  - Add
    `MetalFrameCompositor::draw_frame(&mut self, input: MetalFrameInput<'_>) -> Result<MetalFramePresentation, MetalFrameCompositorError>`.
  - `draw_frame` creates or replaces the current `MetalTarget` when dimensions
    change, syncs `FrameState`, records a `MetalCommandFrame`, begins a render
    pass against the target texture with transparent clear, calls
    `MetalRenderPass::draw_frame`, commits and waits for completion, updates the
    IOSurfaceLayer bounds/contents scale from the pixel dimensions and
    `contents_scale`, and presents the target surface through a presentation
    function.
  - Add a private presentation seam for tests. Production `draw_frame` uses
    `MetalIOSurfaceLayer::set_surface`; tests can force deterministic immediate
    presentation with `set_surface_if_size_matches`, avoiding real main-queue
    work in the Rust test harness while still proving target pixels and layer
    contents identity.
  - Return `MetalFramePresentation` with the foreground cell count, presentation
    mode (`Immediate`/`Queued`), target dimensions, and a boolean indicating
    whether a new target was allocated. This gives later integration enough
    evidence to drive redraw decisions without exposing raw Metal objects.
  - Keep image draws, background-image draw path, custom shader passes, and real
    swap-chain buffering out of scope. This compositor owns one target and one
    frame state as a foundation; the full upstream multi-frame swap chain can
    replace or wrap it later.
  - Constrain the single-target compositor contract: it is valid for
    single-in-flight immediate/main-thread presentation and deterministic tests.
    Off-main queued multi-frame use, including keeping retired/resized targets
    alive until queued presentation completion, remains out of scope until a
    swap-chain/in-flight-target experiment adds a presentation completion model.
- `roastty/src/renderer/metal/mod.rs`
  - Add the `compositor` module.
- `issues/0801-roastty-libghostty-rewrite/README.md`
  - After implementation, update the Metal checklist row to mention a
    single-target frame compositor while keeping terminal-state rebuild,
    images/background/custom shaders, swap-chain pacing, renderer thread, and
    full live frame orchestration open.

## Verification

- Inspect:
  - `vendor/ghostty/src/renderer/generic.zig` `drawFrame`
  - `roastty/src/renderer/metal/frame.rs`
  - `roastty/src/renderer/metal/render_pass.rs`
  - `roastty/src/renderer/metal/target.rs`
  - `roastty/src/renderer/metal/iosurface_layer.rs`
- Run:
  - `cargo fmt -p roastty`
  - `cargo test -p roastty metal::compositor -- --nocapture --test-threads=1`
  - `cargo test -p roastty metal::render_pass::tests::draw_frame -- --nocapture --test-threads=1`
  - `cargo test -p roastty metal::iosurface_layer -- --nocapture --test-threads=1`
- Run markdown formatting:
  - `prettier --write --prose-wrap always --print-width 80 issues/0801-roastty-libghostty-rewrite/README.md issues/0801-roastty-libghostty-rewrite/814-metal-frame-compositor.md`
- Run:
  - `git diff --check`

The experiment passes if a caller can build a `MetalFrameCompositor`, submit a
prepared one-frame input, get a committed IOSurface-backed render target
presented through a deterministic IOSurfaceLayer seam, observe target reuse
versus resize, verify layer bounds/scale math, and verify the rendered target
pixels for at least a background-color frame and a foreground glyph frame. The
production path may report `Queued` through `MetalIOSurfaceLayer::set_surface`,
but this experiment does not claim queued multi-frame target lifetime safety;
that is reserved for a later swap-chain/in-flight-target experiment. It is
Partial if the compositor lands but deterministic presentation or pixel
verification needs follow-up. It fails if the current Metal wrappers cannot be
wired into a reusable frame entry point without sound single-in-flight lifetime
contracts or strong test coverage.

## Design Review

Codex reviewed the initial design and found three blockers before
implementation. First, the single-target ownership model was unsafe for off-main
queued multi-frame presentation: a later frame could overwrite the same
IOSurface before a queued presentation ran, and resize replacement could drop a
target whose IOSurface was still retained by a pending presentation. Second, the
verification repeated Experiment 813's async problem by potentially enqueueing
real main-queue work that unit tests do not drain. Third, the layer size
contract did not specify contents scale, so the plan risked looking HiDPI-ready
while only testing 1x bounds.

The plan was updated to scope this compositor to a single-in-flight immediate /
main-thread presentation foundation, add a deterministic presentation seam for
tests, include `contents_scale` in `MetalFrameInput`, verify layer
`bounds * contentsScale` behavior, and explicitly leave queued multi-frame
target lifetime safety for the later swap-chain/in-flight-target experiment.
