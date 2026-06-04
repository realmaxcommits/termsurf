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

# Experiment 433: the custom-shader from-state colors (update_state_colors)

## Description

Experiment 432 ported the palette loop of `updateCustomShaderUniformsFromState`.
This experiment ports its **colors** group: the `background_color` and
`foreground_color` (always set from the terminal colors) and the `cursor_color`,
`cursor_text`, `selection_background_color`, and `selection_foreground_color`
(set only when their config/state value is present). Each is the same
`Rgb → [r/255, g/255, b/255, 1.0]` normalization. The colors are parameters
(deferring the live terminal state / config, the `dirty` gate, and the
`cursor_visible` / `cursor_style` fields).

## Upstream behavior

In `updateCustomShaderUniformsFromState` (`renderer/generic.zig`), after the
palette:

```zig
uniforms.background_color = .{ colors.background.r/255, .g/255, .b/255, 1.0 };
uniforms.foreground_color = .{ colors.foreground.r/255, .g/255, .b/255, 1.0 };
if (colors.cursor) |c|            uniforms.cursor_color              = .{ c.r/255, c.g/255, c.b/255, 1.0 };
if (self.config.cursor_text) |c|  uniforms.cursor_text              = .{ c.color.r/255, …, 1.0 };
if (self.config.selection_background) |c| uniforms.selection_background_color = .{ … };
if (self.config.selection_foreground) |c| uniforms.selection_foreground_color = .{ … };
```

`background`/`foreground` are always set; the cursor and selection colors are
set only when present (a config/state `Option`) — when absent, the uniform keeps
its prior value. Each is the RGB normalized to `[0, 1]` with an opaque alpha.

## Rust mapping (`roastty/src/renderer/shadertoy.rs`)

A small `normalize_rgb` helper does the `Rgb → [f32; 4]` conversion;
`update_state_colors` sets the always-present and optional colors:

```rust
fn normalize_rgb(c: Rgb) -> [f32; 4] {
    [
        f32::from(c.r) / 255.0,
        f32::from(c.g) / 255.0,
        f32::from(c.b) / 255.0,
        1.0,
    ]
}

impl CustomShaderUniforms {
    /// Update the from-state color uniforms (the colors of upstream
    /// `updateCustomShaderUniformsFromState`): `background_color` and
    /// `foreground_color` always; `cursor_color`, `cursor_text`,
    /// `selection_background_color`, and `selection_foreground_color` only when
    /// their value is present (else the prior value is kept). Each is the RGB
    /// normalized to `[0, 1]` with an opaque alpha.
    pub(crate) fn update_state_colors(
        &mut self,
        background: Rgb,
        foreground: Rgb,
        cursor: Option<Rgb>,
        cursor_text: Option<Rgb>,
        selection_background: Option<Rgb>,
        selection_foreground: Option<Rgb>,
    ) {
        self.background_color = normalize_rgb(background);
        self.foreground_color = normalize_rgb(foreground);
        if let Some(c) = cursor {
            self.cursor_color = normalize_rgb(c);
        }
        if let Some(c) = cursor_text {
            self.cursor_text = normalize_rgb(c);
        }
        if let Some(c) = selection_background {
            self.selection_background_color = normalize_rgb(c);
        }
        if let Some(c) = selection_foreground {
            self.selection_foreground_color = normalize_rgb(c);
        }
    }
}
```

`background`/`foreground` are always normalized; the optional colors are set
only when `Some` (matching upstream's `if (…) |c|`), so a `None` leaves the
uniform's prior value — faithful to upstream.

## Scope / faithfulness notes

- **Ported (bridged)**: `CustomShaderUniforms::update_state_colors` (and the
  `normalize_rgb` helper) — the colors group of upstream's
  `updateCustomShaderUniformsFromState`.
- **Faithful**: `background`/`foreground` always set; the cursor and selection
  colors set only when present (else unchanged); each is
  `[r/255, g/255, b/255, 1.0]` — matching upstream.
- **Faithful adaptation**: the colors are parameters (upstream reads
  `self.terminal_state.colors` / `self.config`), each as an `Rgb` /
  `Option<Rgb>`; the config's `cursor_text` / `selection_*` are `.color`
  wrappers upstream, supplied here as the resolved `Rgb`.
- **Deferred**: the `cursor_visible` and cursor-style fields, the `dirty` gate,
  the live terminal state / config, and the `has_custom_shaders` gate. (Consumed
  by a later slice; this experiment lands and tests the colors group.)
- No C ABI/header/ABI-inventory change (internal Rust).

## Changes

1. `roastty/src/renderer/shadertoy.rs`:
   - add a private `fn normalize_rgb(c: Rgb) -> [f32; 4]` and
     `CustomShaderUniforms::update_state_colors(&mut self, background, foreground, cursor, cursor_text, selection_background, selection_foreground)`.
     Add `Rgb` to the `crate::terminal::color` import.
2. Tests (in `shadertoy.rs`):
   - **First call** with all four optional colors `Some`:
     `update_state_colors(Rgb(10,20,30), Rgb(40,50,60), Some(Rgb(255,0,0)), Some(Rgb(0,128,255)), Some(Rgb(0,255,0)), Some(Rgb(64,64,64)))`
     → `background_color == [10/255, 20/255, 30/255, 1.0]`,
     `foreground_color == [40/255, 50/255, 60/255, 1.0]`,
     `cursor_color == [1.0, 0.0, 0.0, 1.0]`,
     `cursor_text == [0.0, 128.0/255.0, 1.0, 1.0]`,
     `selection_background_color == [0.0, 1.0, 0.0, 1.0]`,
     `selection_foreground_color == [64.0/255.0, 64.0/255.0, 64.0/255.0, 1.0]` —
     proving every optional `Some` path updates its field.
   - **Second call** (after the first seeds the optional colors) with all four
     optionals `None` and different bg/fg:
     `update_state_colors(Rgb(1,2,3), Rgb(4,5,6), None, None, None, None)` →
     `background_color == [1/255, 2/255, 3/255, 1.0]`,
     `foreground_color == [4/255, 5/255, 6/255, 1.0]`; and the four optional
     colors keep their prior (seeded) values — proving `None` preserves the
     prior value.
   - The other fields (`palette[0]`, `focus`) untouched throughout.
3. Format and test (`cargo fmt`, accept output).

## Verification

```bash
cargo fmt
cargo fmt -- --check
cargo test -p roastty update_state_colors
cargo test -p roastty
cargo build -p roastty            # no warnings
rg -n 'ghostty|Ghostty|GHOSTTY' roastty/src/font roastty/src/renderer roastty/src/config && exit 1 || true
rg -n 'ghostty|Ghostty|GHOSTTY' roastty/src/lib.rs roastty/include/roastty.h roastty/tests/abi_harness.c && exit 1 || true
git diff --check
```

The experiment **passes** if:

- `update_state_colors` always sets `background_color`/`foreground_color` and
  sets the optional colors only when present (else unchanged), each
  `[r/255, g/255, b/255, 1.0]` — faithful to upstream's colors group;
- the test passes (the always-set and optional colors; the `None`-unchanged
  fields; the untouched others), and the existing tests still pass;
- the `cursor_visible` / cursor-style fields and the live state stay deferred;
- `cargo fmt` accepted, `cargo build -p roastty` has no warnings, and
  `cargo test -p roastty` passes with no regressions;
- the no-`ghostty`-name gates and `git diff --check` pass;
- Codex reviews the design before implementation and the result after, with all
  real findings fixed.

The experiment **fails** if a color is normalized wrong, an optional color is
set when absent (or not set when present), an unrelated field changes, or any
public C API/ABI changes.

## Design Review

Codex reviewed this design before implementation and approved it with **one Low
finding**, now fixed. It confirmed the design is faithful: `background` /
`foreground` are always set, the optional colors update only when present (else
the prior value is preserved), the normalization is `f32::from(u8) / 255.0` with
an opaque `1.0` alpha, and taking `Rgb` / `Option<Rgb>` parameters is a good
boundary while the live state / config wiring stays deferred.

- **Low (fixed)**: the originally planned test exercised only the `None` paths
  for `cursor_text` and `selection_foreground`, so a bug that never updated
  those two fields would still pass. Fixed by strengthening the test to two
  calls: a first call with all four optional colors `Some` (asserting each
  updates its field), then a second call with all four `None` (asserting each
  keeps its prior seeded value).

Review artifacts:

- Prompt: `logs/codex-review/20260604-094832-d433-prompt.md` (design)
- Result: `logs/codex-review/20260604-094832-d433-last-message.md` (design)

## Result

**Result:** Pass

The custom-shader from-state colors are now live.

- `roastty/src/renderer/shadertoy.rs`:
  `CustomShaderUniforms::update_state_colors(&mut self, background, foreground, cursor, cursor_text, selection_background, selection_foreground)`
  always sets `background_color` / `foreground_color` and sets `cursor_color` /
  `cursor_text` / `selection_background_color` / `selection_foreground_color`
  only when the corresponding `Option<Rgb>` is `Some` (else the prior value is
  kept). A private `fn normalize_rgb(c: Rgb) -> [f32; 4]` does the
  `[r/255, g/255, b/255, 1.0]` conversion. Extended the import to
  `use crate::terminal::color::{Palette, Rgb};`.

Test (in `shadertoy.rs`): `update_state_colors_sets_required_and_optional` — a
first call with all four optionals `Some` (asserting each updates its field:
`cursor_color == [1,0,0,1]`, `cursor_text == [0, 128/255, 1, 1]`,
`selection_background_color == [0,1,0,1]`,
`selection_foreground_color == [64/255, 64/255, 64/255, 1]`, plus the always-set
bg/fg), then a second call with all four `None` and different bg/fg (asserting
bg/fg change and the four optional colors keep their prior seeded values); and
`palette[0]` / `focus` untouched.

Gate results:

- `cargo fmt -p roastty` accepted; `--check` clean.
- `cargo test -p roastty` → 2916 passed, 0 failed (+1, no regressions).
- `cargo build -p roastty` → no warnings.
- No-`ghostty`-name gates (font + renderer + config +
  `lib.rs`/header/`abi_harness.c`) clean; `git diff --check` clean.

## Conclusion

The custom-shader uniforms now take their background, foreground, cursor,
cursor-text, and selection colors from a `Rgb` / `Option<Rgb>` set, with the
optional colors faithfully preserving their prior value when absent. The
remaining `updateCustomShaderUniformsFromState` work — the `cursor_visible` flag
and the cursor-style fields (`current_cursor_style` / `previous_cursor_style`,
driven by a `CursorStyle`-to-`i32` mapping) — plus the `dirty` gate stays
deferred (each a small param-driven slice), along with the `Target` enum, the
shader loading, the broader live per-frame call sites, and the `neverExtendBg`
terminal-core row/cell access; beyond the renderer, the other subsystems.

## Completion Review

Codex reviewed the completed implementation and result and **approved** with
**no findings**. It confirmed `update_state_colors` is faithful to upstream's
colors group: `background_color` and `foreground_color` are always updated, the
four optional colors update only on `Some`, and `None` preserves the prior
uniform value; `normalize_rgb` matches upstream's
`@floatFromInt(channel) / 255.0` with alpha `1.0`. It judged the prior Low
resolved — the test now covers all optional `Some` branches and then a second
all-`None` call proving the seeded optional values are preserved while bg/fg
still update, plus representative untouched fields. No public C ABI/header
impact; nothing needed to change before the result commit.

Review artifacts:

- Prompt: `logs/codex-review/20260604-095134-r433-prompt.md` (result)
- Result: `logs/codex-review/20260604-095134-r433-last-message.md` (result)
