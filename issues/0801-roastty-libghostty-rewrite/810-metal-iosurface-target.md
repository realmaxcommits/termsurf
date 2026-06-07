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

# Experiment 810: Metal IOSurface Target

## Description

Port upstream `renderer/metal/Target.zig` into Roastty as a focused
IOSurface-backed Metal render target.

The Metal renderer checklist still lists `Target` and `IOSurfaceLayer` as
missing. Roastty already has offscreen `MetalTexture` render targets and
render-pass execution, but no wrapper for the live-presentable path: an
IOSurface plus an `MTLTexture` created with
`newTextureWithDescriptor:iosurface:plane:`. This experiment adds the target
resource layer without creating a CALayer subclass, presenting a surface, or
starting the full live render loop.

## Changes

- `roastty/Cargo.toml`
  - Add a direct `objc2-io-surface` dependency with the `IOSurfaceRef`,
    `IOSurfaceTypes`, and `objc2-core-foundation` features needed for IOSurface
    creation and dimension inspection.
  - Enable the `objc2-io-surface` feature on `objc2-metal` so
    `MTLDevice::newTextureWithDescriptor_iosurface_plane` is available.
- `roastty/src/renderer/metal/target.rs`
  - Add `MetalTargetOptions` matching upstream `Target.Options`: device, width,
    height, pixel format, and storage mode.
  - Add `MetalTarget` owning a `CFRetained<IOSurfaceRef>` and a
    `Retained<ProtocolObject<dyn MTLTexture>>`, plus width/height accessors.
  - Build the IOSurface property dictionary from CoreFoundation values:
    `kIOSurfaceWidth`, `kIOSurfaceHeight`, `kIOSurfacePixelFormat` (`32BGRA`),
    `kIOSurfaceBytesPerElement = 4`, and `kIOSurfaceColorSpace` populated from
    `CGColorSpaceCreateWithName(kCGColorSpaceDisplayP3)` plus
    `CGColorSpaceCopyPropertyList`, matching upstream's Display P3 surface
    setup.
  - Accept only BGRA-compatible Metal formats (`Bgra8Unorm` and
    `Bgra8UnormSrgb`) because the IOSurface storage is always `32BGRA`; reject
    other formats before creating the surface/texture.
  - Create an `MTLTextureDescriptor`, set width, height, pixel format,
    render-target usage, and resource options using existing Metal API wrappers.
  - Create the texture with `newTextureWithDescriptor_iosurface_plane(..., 0)`.
  - Return explicit errors for invalid dimensions, IOSurface creation failure,
    unsupported target pixel format, Display P3 property-list creation failure,
    or Metal texture creation failure.
  - Expose `surface() -> &IOSurfaceRef` for the future `IOSurfaceLayer`
    experiment and `texture() -> &ProtocolObject<dyn MTLTexture>` for
    render-pass target usage.
  - Add device-backed tests that create a target, assert dimensions and
    IOSurface bytes-per-element, render a clear color into its texture with the
    existing render-pass path, and read back the bytes through the Metal
    texture.
- `roastty/src/renderer/metal/render_pass.rs`
  - Change `MetalRenderPassAttachment` to carry a borrowed
    `&ProtocolObject<dyn MTLTexture>` instead of `&MetalTexture`, so both
    offscreen `MetalTexture` and IOSurface-backed `MetalTarget` can be render
    attachments.
  - Keep `MetalRenderPassStep` texture bindings unchanged as
    `&[Option<&MetalTexture>]`; this experiment only generalizes render targets,
    not shader-read texture resources.
  - Update existing render-pass tests to pass `texture.texture()` for
    attachments.
- `roastty/src/renderer/metal/mod.rs`
  - Add the `target` module.
- `issues/0801-roastty-libghostty-rewrite/README.md`
  - After implementation, update the Metal checklist row to mention the
    IOSurface target wrapper while keeping `IOSurfaceLayer` and full live frame
    orchestration open.

## Verification

- Inspect:
  - `vendor/ghostty/src/renderer/metal/Target.zig`
  - `vendor/ghostty/src/renderer/metal/IOSurfaceLayer.zig`
  - `roastty/src/renderer/metal/texture.rs`
  - `roastty/src/renderer/metal/render_pass.rs`
  - local `objc2-io-surface` and `objc2-metal` generated bindings
- Run:
  - `cargo fmt -p roastty`
  - `cargo test -p roastty metal::target -- --nocapture --test-threads=1`
  - `cargo test -p roastty metal::render_pass -- --nocapture --test-threads=1`
- Run markdown formatting:
  - `prettier --write --prose-wrap always --print-width 80 issues/0801-roastty-libghostty-rewrite/README.md issues/0801-roastty-libghostty-rewrite/810-metal-iosurface-target.md`
- Run:
  - `git diff --check`

The experiment passes if Roastty has a tested IOSurface-backed Metal target that
can be rendered into by the existing render-pass path while the Metal renderer
row remains partial for `IOSurfaceLayer` and full live frame orchestration. It
is Partial if IOSurface creation lands but render-pass use or readback exposes a
follow-up binding issue. It fails if the target cannot be cleanly expressed with
the current `objc2` bindings. Negative tests should cover zero width, zero
height, and unsupported non-BGRA pixel formats; Metal texture creation failure
is still represented as an error but is not forced if valid local inputs do not
make Metal return null.

## Design Review

Codex reviewed the design and found five required fixes before implementation:
the original plan did not specify how an IOSurface-backed `MTLTexture` would fit
the current `MetalRenderPassAttachment` type, omitted upstream's Display P3
IOSurface color-space property, described IOSurface ownership too loosely, did
not constrain the hardcoded `32BGRA` IOSurface storage to BGRA-compatible Metal
formats, and did not require negative tests for the promised error paths.

The plan was updated to make `MetalRenderPassAttachment` borrow a common
`MTLTexture` protocol view, include `kIOSurfaceColorSpace` from a Display P3
CoreGraphics property list, specify `CFRetained<IOSurfaceRef>` ownership,
restrict target pixel formats to `Bgra8Unorm`/`Bgra8UnormSrgb`, and add zero
dimension plus unsupported-format negative tests. With those fixes, the review
said the design is implementable with the local `objc2` bindings and remains
properly scoped to `Target`, not `IOSurfaceLayer` or the live render loop.
