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

# Experiment 430: the custom-shader cursor update (update_cursor)

## Description

Experiment 429 ported the time/resolution half of
`updateCustomShaderUniformsForFrame`. This experiment ports the **cursor-glyph**
half: from the cursor glyph (a `CellTextVertex`, supplied by
`Contents::get_cursor_glyph` — already ported) plus the cell size and padding,
it computes the cursor's pixel rectangle and color and, when they changed,
shifts `current_cursor` to `previous_cursor` and stamps `cursor_change_time`.
roastty is Metal-only and Metal's `custom_shader_y_is_down = true`, so the
OpenGL y-flip branch (the only use of `screen.height`) is omitted. The focus
uniforms and the live timing/state remain deferred.

## Upstream behavior

In `updateCustomShaderUniformsForFrame` (`renderer/generic.zig`), after the time
fields, the cursor glyph drives the cursor uniforms (the Metal —
`custom_shader_y_is_down = true` — path):

```zig
if (self.cells.getCursorGlyph()) |cursor| {
    var pixel_x: f32 = @floatFromInt(cursor.grid_pos[0] * cell.width + padding.left);
    var pixel_y: f32 = @floatFromInt(cursor.grid_pos[1] * cell.height + padding.top);
    // (the `!custom_shader_y_is_down` screen-height flip is OpenGL-only)
    pixel_x += @floatFromInt(cursor.bearings[0]);
    // custom_shader_y_is_down (Metal):
    pixel_y += @floatFromInt(cell.height);
    pixel_y -= @floatFromInt(cursor.bearings[1]);
    pixel_y += @floatFromInt(cursor.glyph_size[1]);

    const new_cursor = .{ pixel_x, pixel_y, cursor.glyph_size[0], cursor.glyph_size[1] };
    const cursor_color = .{ cursor.color[0]/255, cursor.color[1]/255, cursor.color[2]/255, cursor.color[3]/255 };

    if (new_cursor != uniforms.current_cursor or cursor_color != uniforms.current_cursor_color) {
        uniforms.previous_cursor = uniforms.current_cursor;
        uniforms.previous_cursor_color = uniforms.current_cursor_color;
        uniforms.current_cursor = new_cursor;
        uniforms.current_cursor_color = cursor_color;
        uniforms.cursor_change_time = uniforms.time;
    }
}
```

The pixel rect is
`[left+bearingX, top+cellHeight-bearingY+glyphHeight, glyphW, glyphH]`; the
color is the glyph color normalized to `[0, 1]`. On a change, the previous
cursor is preserved and `cursor_change_time = time` (the frame time set by the
time update). No cursor glyph → no update.

## Rust mapping (`roastty/src/renderer/shadertoy.rs`)

`update_cursor` takes the cursor glyph (`Option<CellTextVertex>`) and the cell
size / padding:

```rust
impl CustomShaderUniforms {
    /// Update the cursor uniforms from the cursor glyph (upstream
    /// `updateCustomShaderUniformsForFrame`'s cursor half, Metal
    /// `custom_shader_y_is_down = true`): compute the cursor's pixel rect
    /// (`[left+bearingX, top+cellH-bearingY+glyphH, glyphW, glyphH]`) and its
    /// normalized color; on a change, shift `current` → `previous` and stamp
    /// `cursor_change_time = time`. No cursor glyph → no update.
    pub(crate) fn update_cursor(
        &mut self,
        cursor: Option<CellTextVertex>,
        cell_width: u32,
        cell_height: u32,
        padding_left: u32,
        padding_top: u32,
    ) {
        let Some(cursor) = cursor else {
            return;
        };
        let mut pixel_x = (cursor.grid_pos[0] as u32 * cell_width + padding_left) as f32;
        let mut pixel_y = (cursor.grid_pos[1] as u32 * cell_height + padding_top) as f32;
        pixel_x += f32::from(cursor.bearings[0]);
        // Metal: custom_shader_y_is_down = true.
        pixel_y += cell_height as f32;
        pixel_y -= f32::from(cursor.bearings[1]);
        pixel_y += cursor.glyph_size[1] as f32;

        let new_cursor = [
            pixel_x,
            pixel_y,
            cursor.glyph_size[0] as f32,
            cursor.glyph_size[1] as f32,
        ];
        let cursor_color = [
            f32::from(cursor.color[0]) / 255.0,
            f32::from(cursor.color[1]) / 255.0,
            f32::from(cursor.color[2]) / 255.0,
            f32::from(cursor.color[3]) / 255.0,
        ];

        if new_cursor != self.current_cursor || cursor_color != self.current_cursor_color {
            self.previous_cursor = self.current_cursor;
            self.previous_cursor_color = self.current_cursor_color;
            self.current_cursor = new_cursor;
            self.current_cursor_color = cursor_color;
            self.cursor_change_time = self.time;
        }
    }
}
```

The pixel math, the `glyph_size` width/height, the `/255.0` color normalization,
and the change-tracking match upstream; `cursor_change_time` is `self.time` (set
by `update_for_frame`).

## Scope / faithfulness notes

- **Ported (bridged)**: `CustomShaderUniforms::update_cursor` — the cursor-glyph
  half of upstream's per-frame custom-shader update (the cursor pixel rect, the
  normalized color, and the change-tracking), Metal path.
- **Faithful**: the pixel rect (`pixel_x = left + bearingX`,
  `pixel_y = top + cellH − bearingY + glyphH`), the `[glyphW, glyphH]` size, the
  `/255.0` color, and the change-tracking (preserve previous, stamp
  `cursor_change_time = time`) match upstream; no cursor glyph (`None`) is a
  no-op.
- **Faithful adaptation**: roastty is Metal-only and
  `custom_shader_y_is_down = true`, so the OpenGL `screen.height` y-flip and the
  alternate y-bearing branch are omitted (they are the `false` path); the cursor
  glyph, cell size, and padding are parameters (upstream reads
  `self.cells.getCursorGlyph()` / `self.size`). The cursor glyph type is
  roastty's `CellTextVertex` (the upstream cursor cell).
- **Deferred**: the focus uniforms (`focus` / `time_focus`, from `self.focused`
  / `custom_shader_focused_changed`), the live timing source, the
  `has_custom_shaders` gate, and the live call site that supplies the glyph /
  sizes. (Consumed by a later slice; this experiment lands and tests the cursor
  update.)
- No C ABI/header/ABI-inventory change (internal Rust).

## Changes

1. `roastty/src/renderer/shadertoy.rs`:
   - add
     `CustomShaderUniforms::update_cursor(&mut self, cursor: Option<CellTextVertex>, cell_width, cell_height, padding_left, padding_top)`.
     Import `CellTextVertex` from `crate::renderer::shader`.
2. Tests (in `shadertoy.rs`):
   - a cursor glyph (`grid_pos = [2, 3]`, `glyph_size = [10, 20]`,
     `bearings = [1, 2]`, `color = [255, 0, 0, 255]`), `cell 8×16`,
     `padding left 4 / top 5`, after setting `time = 5.0` →
     `current_cursor == [21.0, 87.0, 10.0, 20.0]` (`x = 2·8+4+1 = 21`;
     `y = 3·16+5+16−2+20 = 87`), `current_cursor_color == [1.0, 0.0, 0.0, 1.0]`,
     `previous_cursor == [0; 4]` (the old current), and
     `cursor_change_time == 5.0`;
   - a second call with the **same** glyph → no change (`previous_cursor` and
     `cursor_change_time` unchanged even after bumping `time`);
   - a call with a **different** glyph → `previous_cursor` becomes the prior
     `current_cursor`;
   - a `None` cursor → no-op (the cursor fields unchanged).
3. Format and test (`cargo fmt`, accept output).

## Verification

```bash
cargo fmt
cargo fmt -- --check
cargo test -p roastty update_cursor
cargo test -p roastty
cargo build -p roastty            # no warnings
rg -n 'ghostty|Ghostty|GHOSTTY' roastty/src/font roastty/src/renderer roastty/src/config && exit 1 || true
rg -n 'ghostty|Ghostty|GHOSTTY' roastty/src/lib.rs roastty/include/roastty.h roastty/tests/abi_harness.c && exit 1 || true
git diff --check
```

The experiment **passes** if:

- `update_cursor` computes the Metal cursor pixel rect and normalized color from
  the glyph + sizes, and shifts `current` → `previous` + stamps
  `cursor_change_time = time` only on a change (no glyph → no-op) — faithful to
  upstream's cursor half;
- the tests pass (the computed rect/color/change-tracking; the unchanged and
  different and `None` cases), and the existing tests still pass;
- the focus uniforms, the timing source, and the live call site stay deferred;
- `cargo fmt` accepted, `cargo build -p roastty` has no warnings, and
  `cargo test -p roastty` passes with no regressions;
- the no-`ghostty`-name gates and `git diff --check` pass;
- Codex reviews the design before implementation and the result after, with all
  real findings fixed.

The experiment **fails** if the pixel math (the bearing/glyph-height terms), the
color normalization, or the change-tracking differs from upstream's Metal path,
an unrelated field changes, or any public C API/ABI changes.

## Design Review

Codex reviewed this design before implementation and **approved** it with **no
findings**, verifying the Metal pixel math: for Metal
`custom_shader_y_is_down = true`, so omitting the OpenGL
`screen.height − pixel_y` flip is faithful, and the formula matches upstream —
`x = grid_x · cell_width + padding_left + bearing_x`,
`y = grid_y · cell_height + padding_top + cell_height − bearing_y + glyph_height`.
It confirmed the worked example (`x = 2·8 + 4 + 1 = 21`;
`y = 3·16 + 5 + 16 − 2 + 20 = 87` → `[21, 87, 10, 20]`), the color normalization
(RGBA / `255.0`), and the change-tracking (a change shifts current → previous
and stamps `cursor_change_time = self.time`; `None` is a no-op). It noted that
taking `Option<CellTextVertex>` by value is fine because `CellTextVertex` is
`Copy` and matches `Contents::get_cursor_glyph()`, and judged the planned tests
to cover the main risks (pixel math, normalized color, no-change behavior,
changed-cursor shifting, and the `None` no-op).

Review artifacts:

- Prompt: `logs/codex-review/20260604-093334-d430-prompt.md` (design)
- Result: `logs/codex-review/20260604-093334-d430-last-message.md` (design)
