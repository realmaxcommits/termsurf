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

# Experiment 796: Renderer Metal Checklist Sync

## Description

Issue 801's renderer checklist still says several renderer/Metal pieces are
missing even though current Roastty has substantial tested foundations in
`roastty/src/renderer/`. The stale wording is most visible in the `image.rs`,
Metal pipeline/frame/texture, and custom-shader rows.

This experiment updates the checklist wording only. It keeps the rows unchecked
because the live renderer remains incomplete: there is no full frame build/dirty
tracking/glyph-upload/draw-call pacing loop, no window `Target` or
`IOSurfaceLayer` presentation path, no full custom-shader file loading, and no
full frontend renderer integration.

## Changes

- `issues/0801-roastty-libghostty-rewrite/README.md`
  - Update the renderer heading from "data + Metal primitives only" to a scoped
    partial description that reflects offscreen render-pass and frame-resource
    foundations while still naming the missing live render loop.
  - Update the `Image state` row to name pending/ready/replace/unload tracking,
    Kitty placement buckets, RGBA preparation, Metal texture upload, and image
    draw-call foundations.
  - Update the Metal pipeline row to name the existing `pipeline`, standard
    shader library/pipelines, `texture`, `FrameState`, atlas/cell/uniform sync,
    and offscreen render-pass foundations while leaving `Sampler`, window
    `Target`, `IOSurfaceLayer`, and full live frame orchestration open.
  - Update the z2d debug/link/render-thread/custom-shader row to name
    custom-shader uniforms/target/per-frame update support and leave shader file
    loading, debug overlay, renderer thread, and full link-highlight rendering
    open.
  - Add the Experiment 796 index entry.
- `issues/0801-roastty-libghostty-rewrite/796-renderer-metal-checklist-sync.md`
  - Record verification evidence and review results.

## Verification

- Inspect:
  - `roastty/src/renderer/image.rs`
  - `roastty/src/renderer/shadertoy.rs`
  - `roastty/src/renderer/metal/pipeline.rs`
  - `roastty/src/renderer/metal/frame.rs`
  - `roastty/src/renderer/metal/texture.rs`
  - `roastty/src/renderer/metal/render_pass.rs`
  - `roastty/src/renderer/metal/shaders.rs`
- Run:
  - `cargo test -p roastty renderer::image -- --nocapture --test-threads=1`
  - `cargo test -p roastty renderer::metal::pipeline -- --nocapture --test-threads=1`
  - `cargo test -p roastty renderer::metal::frame -- --nocapture --test-threads=1`
  - `cargo test -p roastty renderer::metal::texture -- --nocapture --test-threads=1`
  - `cargo test -p roastty renderer::metal::render_pass -- --nocapture --test-threads=1`
  - `cargo test -p roastty renderer::shadertoy -- --nocapture --test-threads=1`
- Run markdown formatting:
  - `prettier --write --prose-wrap always --print-width 80 issues/0801-roastty-libghostty-rewrite/README.md issues/0801-roastty-libghostty-rewrite/796-renderer-metal-checklist-sync.md`
- Run:
  - `git diff --check`

The experiment passes if the README stops calling existing renderer/Metal
foundations missing while still keeping the renderer rows unchecked and clearly
leaving the live renderer and presentation path open. It is Partial if only
image/Metal pipeline wording can be corrected. It fails if the original
"missing" wording remains accurate.

## Design Review

Codex reviewed the design and found no blocking findings. The review approved
the scope because the renderer rows remain unchecked, the claims are limited to
tested foundations, and the missing live render loop, window `Target`,
`IOSurfaceLayer`, `Sampler`, full frame orchestration, shader file loading,
renderer thread, debug overlay, and frontend presentation path remain explicit.
The review also confirmed the planned test filters and counts.
