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

# Experiment 821: Apply Custom Shader Frame Updates

## Description

Connect the prepared frame rebuild path to Roastty's existing custom shader
per-frame uniform helpers. Experiment 820 can clear/apply Metal block cursor
uniforms, but the tracker still calls out custom shader cursor animation
updates: upstream `updateCustomShaderUniformsForFrame` updates time/resolution,
cursor rectangle/color transition fields, and focus timing once per frame.

This experiment keeps those inputs prepared. It does not collect live clocks,
surface focus state, renderer sizes, custom shader enablement, cursor style, or
cursor glyphs from the live renderer. It does not upload custom shader buffers,
load custom shaders, submit draw calls, pace redraws, or add the live renderer
thread.

## Changes

- `roastty/src/renderer/frame_rebuild.rs`
  - Add `FrameCustomShaderInput` with prepared per-frame custom shader data:
    - `time_secs: f32`,
    - `time_delta_secs: f32`,
    - `screen_size: [u32; 2]`,
    - `cell_size: [u32; 2]`,
    - `padding: [u32; 2]`,
    - `cursor: Option<CellTextVertex>`,
    - `focused: bool`, and
    - `focus_changed: bool`.
  - Add `FrameCustomShaderValidationError` for zero cell dimensions only when a
    cursor glyph is present, because cell size is used for cursor-rect
    calculation. Zero screen dimensions remain valid prepared inputs for
    time/resolution updates, matching upstream and the existing helper.
  - Add `FrameCustomShaderApplication` recording:
    - whether the frame time/resolution fields were updated,
    - whether a cursor glyph was supplied,
    - whether focus changed was consumed, and
    - the returned `focus_changed` value for the next frame.
  - Add
    `FrameRebuildPlan::apply_custom_shader_frame(&self, uniforms: &mut CustomShaderUniforms, input: FrameCustomShaderInput) -> Result<FrameCustomShaderApplication, FrameCustomShaderValidationError>`.
  - Validate prepared cursor geometry before mutation. Missing cursor glyphs
    still allow time/resolution/focus updates even when cell dimensions are
    zero, because `CustomShaderUniforms::update_cursor(None, ...)` is a no-op.
  - Call `CustomShaderUniforms::update_for_frame` with prepared time and screen
    size.
  - Call `CustomShaderUniforms::update_cursor` with the prepared cursor glyph
    and prepared cell/padding geometry. A missing cursor glyph leaves the
    previous custom shader cursor fields untouched, matching the helper and
    upstream behavior.
  - Call `CustomShaderUniforms::update_focus` with prepared focus state and
    focus-changed flag, returning the next `focus_changed` value to the caller.
  - Add tests proving:
    - a frame update advances time, delta, frame count, resolution, and channel
      resolution,
    - a cursor glyph updates custom shader cursor rectangle/color and stamps
      `cursor_change_time`,
    - a missing cursor glyph leaves cursor rectangle/color/change-time fields
      untouched while time/resolution still update,
    - an unchanged cursor glyph does not move current cursor to previous or
      restamp `cursor_change_time`,
    - focus-changed/focused stamps `time_focus` and returns `false`,
    - unfocused or unchanged focus leaves `time_focus` untouched and returns the
      correct next `focus_changed`, and
    - zero screen dimensions still update frame fields,
    - zero cell dimensions with no cursor still update time/resolution/focus,
      and
    - zero cell dimensions with a cursor glyph reject before mutation.
- `issues/0801-roastty-libghostty-rewrite/README.md`
  - After implementation, update the renderer tracker to mention that prepared
    custom shader per-frame time, focus, and cursor animation inputs can update
    `CustomShaderUniforms`, while live terminal-state collection, custom shader
    enablement, glyph upload/draw calls, pacing, and renderer-thread integration
    remain open.

## Verification

- Inspect:
  - `vendor/ghostty/src/renderer/generic.zig`
    `updateCustomShaderUniformsForFrame`
  - `roastty/src/renderer/frame_rebuild.rs`
  - `roastty/src/renderer/shadertoy.rs`
- Run:
  - `cargo fmt -p roastty`
  - `cargo test -p roastty renderer::frame_rebuild -- --nocapture`
  - `cargo test -p roastty renderer::shadertoy -- --nocapture`
- Run markdown formatting:
  - `prettier --write --prose-wrap always --print-width 80 issues/0801-roastty-libghostty-rewrite/README.md issues/0801-roastty-libghostty-rewrite/821-apply-custom-shader-frame.md`
- Run:
  - `git diff --check`

The experiment passes if prepared custom shader per-frame inputs can update
time/resolution, cursor animation, and focus fields through the existing
`CustomShaderUniforms` helpers without live renderer state collection. It is
Partial if the driver lands but one prepared input has to be split into a
follow-up. It fails if custom shader frame updates cannot be separated from the
live renderer loop.

## Design Review

Codex reviewed the initial design and found that its validation boundary was
stricter than upstream and the existing helpers. Upstream accepts zero
screen-size values as plain resolution/channel-resolution inputs, and
`CustomShaderUniforms::update_cursor(None, ...)` is a no-op, so zero cell size
should only be rejected when a cursor glyph is present and cursor-rect geometry
would be computed. Codex also asked for tests proving that zero screen
dimensions still update frame fields, zero cell dimensions with no cursor still
update time/focus fields, and zero cell dimensions with a cursor reject before
mutation.

The design was amended to keep zero screen dimensions valid, reject zero cell
dimensions only when `cursor.is_some()`, and add tests for that validation
boundary.

Codex re-reviewed the amended design and approved it for implementation with no
remaining blockers. The re-review confirmed that the zero screen-size, zero
cell-size-without-cursor, and zero cell-size-with-cursor boundaries are now
specified and tested, and that the driver order matches upstream/helper
behavior.

## Result

**Result:** Pass

Roastty can now apply prepared custom shader per-frame updates through the frame
rebuild path:

- `roastty/src/renderer/frame_rebuild.rs` adds `FrameCustomShaderInput`,
  `FrameCustomShaderValidationError`, and `FrameCustomShaderApplication`.
- `FrameRebuildPlan::apply_custom_shader_frame` validates cursor geometry before
  mutation, then calls `CustomShaderUniforms::update_for_frame`,
  `CustomShaderUniforms::update_cursor`, and
  `CustomShaderUniforms::update_focus` in upstream order.
- The driver returns the next `focus_changed` value so a future live renderer
  can consume focus-change state the same way upstream does.
- Zero screen dimensions remain valid frame-resolution inputs.
- Zero cell dimensions remain valid when no cursor glyph is supplied.
- Zero cell dimensions with a cursor glyph reject before mutation because cursor
  rectangle calculation would be invalid.
- Tests cover time/resolution updates, cursor rectangle/color animation updates,
  missing-cursor no-op behavior, unchanged-cursor no-restamp behavior, focused
  focus-change consumption, unfocused focus-change preservation, zero screen
  size acceptance, zero cell size without cursor acceptance, and zero cell size
  with cursor rejection before mutation.

Verification:

- Inspected `vendor/ghostty/src/renderer/generic.zig`
  `updateCustomShaderUniformsForFrame`.
- Inspected `roastty/src/renderer/frame_rebuild.rs`.
- Inspected `roastty/src/renderer/shadertoy.rs`.
- `cargo fmt -p roastty` — passed.
- `cargo test -p roastty renderer::frame_rebuild -- --nocapture` — passed, 65
  tests.
- `cargo test -p roastty renderer::shadertoy -- --nocapture` — passed, 9 tests.
- `prettier --write --prose-wrap always --print-width 80 issues/0801-roastty-libghostty-rewrite/README.md issues/0801-roastty-libghostty-rewrite/821-apply-custom-shader-frame.md`
  — passed.
- `git diff --check` — passed.

## Conclusion

Experiment 821 connects prepared custom shader per-frame data to the existing
custom shader uniform helpers. The frame rebuild path can now update custom
shader time/resolution, cursor animation, and focus timing fields without live
renderer state collection. Remaining render-loop work still needs live
terminal-state collection, custom shader enablement/upload, cell/glyph upload,
draw-call submission, pacing, and renderer-thread integration.

## Completion Review

Codex reviewed the completed implementation and found no blocking issues. The
review confirmed that the driver validates before mutation, calls
`update_for_frame`, `update_cursor`, and `update_focus` in upstream order,
returns the next `focus_changed` value, and preserves the corrected
zero-dimension boundary: zero screen size is accepted, zero cell size is
accepted with no cursor, and zero cell size rejects only when a cursor glyph is
present.

Codex also confirmed that the result doc and README tracker match the diff, then
re-ran the focused `frame_rebuild` and `shadertoy` test filters plus
formatting/diff checks successfully. The experiment is approved for the result
commit.
