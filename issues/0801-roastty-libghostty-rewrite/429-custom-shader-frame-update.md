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

# Experiment 429: the custom-shader per-frame time/resolution update (update_for_frame)

## Description

Experiment 428 ported the `CustomShaderUniforms` value type. This experiment
ports the **time/resolution** group of upstream's per-frame custom-shader update
(`updateCustomShaderUniformsForFrame`): `time`, `time_delta`, the `frame`
counter, `resolution`, and `channel_resolution[0]`. These are pure given the
elapsed/delta seconds and the screen size, so the method takes them as
parameters — deferring the live timing source (`Instant::now` and the
first/last-frame bookkeeping) and the cursor-glyph half of the function (which
reads `Contents`).

## Upstream behavior

In `updateCustomShaderUniformsForFrame` (`renderer/generic.zig`), the time and
resolution fields are set each frame:

```zig
const since_ns: f32 = @floatFromInt(now.since(first_frame_time));
uniforms.time = since_ns / std.time.ns_per_s;

const delta_ns: f32 = @floatFromInt(now.since(last_frame_time));
uniforms.time_delta = delta_ns / std.time.ns_per_s;

uniforms.frame += 1;

const screen = self.size.screen;
uniforms.resolution = .{ screen.width, screen.height, 1 };
uniforms.channel_resolution[0] = .{ screen.width, screen.height, 1, 0 };
```

`time` is seconds since the first frame; `time_delta` is seconds since the last
frame; `frame` increments; `resolution` is the screen size with `z = 1`;
`channel_resolution[0]` is the same with a trailing `0`. (The function then
updates the cursor fields from `getCursorGlyph` / the cursor state — deferred.)

## Rust mapping (`roastty/src/renderer/shadertoy.rs`)

`update_for_frame` takes the already-computed elapsed/delta seconds (the caller
owns the clock) and the screen size:

```rust
impl CustomShaderUniforms {
    /// Update the per-frame time and resolution fields (the time/resolution
    /// group of upstream `updateCustomShaderUniformsForFrame`): `time` (seconds
    /// since the first frame), `time_delta` (seconds since the last frame), the
    /// `frame` counter (incremented), `resolution` (the screen size, `z = 1`),
    /// and `channel_resolution[0]`. The cursor-glyph update and the timing source
    /// are the caller's / a later slice.
    pub(crate) fn update_for_frame(
        &mut self,
        time_secs: f32,
        time_delta_secs: f32,
        screen_width: u32,
        screen_height: u32,
    ) {
        self.time = time_secs;
        self.time_delta = time_delta_secs;
        self.frame += 1;
        let (w, h) = (screen_width as f32, screen_height as f32);
        self.resolution = [w, h, 1.0];
        self.channel_resolution[0] = [w, h, 1.0, 0.0];
    }
}
```

`time` / `time_delta` are the seconds the caller computed (upstream divides the
`Instant` deltas by `ns_per_s`); `frame` increments; `resolution` and
`channel_resolution[0]` are the screen size with the `1`/`1, 0` trailing values
— matching upstream.

## Scope / faithfulness notes

- **Ported (bridged)**: `CustomShaderUniforms::update_for_frame` — the
  time/resolution group of upstream's per-frame custom-shader update (`time`,
  `time_delta`, `frame`, `resolution`, `channel_resolution[0]`).
- **Faithful**: `time` / `time_delta` set from the elapsed/delta seconds;
  `frame` incremented; `resolution = [w, h, 1]`;
  `channel_resolution[0] = [w, h, 1, 0]` — matching the upstream assignments;
  only these fields are touched.
- **Faithful adaptation**: the elapsed/delta seconds are parameters (upstream
  computes them from `Instant::now` and the first/last-frame times); the screen
  size is a parameter (upstream reads `self.size.screen`). The caller owns the
  clock.
- **Deferred**: the live timing source (the `Instant` / first-last-frame
  bookkeeping), the cursor-glyph half of the function (`getCursorGlyph` → the
  `current_cursor` / cursor fields, which read `Contents`), the
  `updateCustomShaderUniformsFromState` group, and the `has_custom_shaders`
  gate. (Consumed by a later slice; this experiment lands and tests the
  time/resolution update.)
- No C ABI/header/ABI-inventory change (internal Rust).

## Changes

1. `roastty/src/renderer/shadertoy.rs`:
   - add
     `CustomShaderUniforms::update_for_frame(&mut self, time_secs, time_delta_secs, screen_width, screen_height)`.
2. Tests (in `shadertoy.rs`):
   - from `new()`, `update_for_frame(1.5, 0.016, 800, 600)` → `time == 1.5`,
     `time_delta == 0.016`, `frame == 1`, `resolution == [800.0, 600.0, 1.0]`,
     `channel_resolution[0] == [800.0, 600.0, 1.0, 0.0]`; a second call →
     `frame == 2` (the counter increments); and the other fields (`focus`,
     `palette[0]`, `channel_resolution[1]`) untouched.
3. Format and test (`cargo fmt`, accept output).

## Verification

```bash
cargo fmt
cargo fmt -- --check
cargo test -p roastty update_for_frame
cargo test -p roastty
cargo build -p roastty            # no warnings
rg -n 'ghostty|Ghostty|GHOSTTY' roastty/src/font roastty/src/renderer roastty/src/config && exit 1 || true
rg -n 'ghostty|Ghostty|GHOSTTY' roastty/src/lib.rs roastty/include/roastty.h roastty/tests/abi_harness.c && exit 1 || true
git diff --check
```

The experiment **passes** if:

- `update_for_frame` sets `time` / `time_delta` from the given seconds,
  increments `frame`, and sets `resolution` / `channel_resolution[0]` from the
  screen size (with the `1` / `1, 0` trailers), touching nothing else — faithful
  to upstream's per-frame time/resolution update;
- the tests pass (the field values; the `frame` increment; the untouched
  fields), and the existing tests still pass;
- the timing source, the cursor-glyph update, and the from-state group stay
  deferred;
- `cargo fmt` accepted, `cargo build -p roastty` has no warnings, and
  `cargo test -p roastty` passes with no regressions;
- the no-`ghostty`-name gates and `git diff --check` pass;
- Codex reviews the design before implementation and the result after, with all
  real findings fixed.

The experiment **fails** if a field is set wrong (e.g. `channel_resolution[0]`'s
trailer, or `frame` not incrementing), an unrelated field is changed, or any
public C API/ABI changes.

## Design Review

Codex reviewed this design before implementation and **approved** it with **no
findings**. It confirmed `update_for_frame` is faithful to the time/resolution
portion of upstream `updateCustomShaderUniformsForFrame`: it sets `time`,
`time_delta`, increments `frame`, writes `resolution = [width, height, 1.0]`,
and writes `channel_resolution[0] = [width, height, 1.0, 0.0]`; the `1.0` /
`1.0, 0.0` trailers match upstream exactly, and leaving
`channel_resolution[1..]` untouched is correct. It judged taking the precomputed
seconds and screen dimensions as parameters a sound slice boundary — upstream's
clock bookkeeping and cursor-glyph half are separate dependencies, and deferring
them while porting this pure field group is consistent with the prior
uniform-group splits. It judged the planned test to cover the field assignments,
the frame increment across calls, and representative untouched fields.

Review artifacts:

- Prompt: `logs/codex-review/20260604-092820-d429-prompt.md` (design)
- Result: `logs/codex-review/20260604-092820-d429-last-message.md` (design)

## Result

**Result:** Pass

The custom-shader per-frame time/resolution update is now live.

- `roastty/src/renderer/shadertoy.rs`:
  `CustomShaderUniforms::update_for_frame(&mut self, time_secs, time_delta_secs, screen_width, screen_height)`
  sets `time` / `time_delta` from the given seconds, increments `frame`, and
  sets `resolution = [w, h, 1.0]` and
  `channel_resolution[0] = [w, h, 1.0, 0.0]`.

Test (in `shadertoy.rs`): `update_for_frame_sets_time_and_resolution` — from
`new()`, `update_for_frame(1.5, 0.016, 800, 600)` → `time == 1.5`,
`time_delta == 0.016`, `frame == 1`, `resolution == [800, 600, 1]`,
`channel_resolution[0] == [800, 600, 1, 0]`; a second call → `frame == 2`; and
`focus` / `palette[0]` / `channel_resolution[1]` untouched.

Gate results:

- `cargo fmt -p roastty` accepted; `--check` clean.
- `cargo test -p roastty` → 2911 passed, 0 failed (+1, no regressions).
- `cargo build -p roastty` → no warnings.
- No-`ghostty`-name gates (font + renderer + config +
  `lib.rs`/header/`abi_harness.c`) clean; `git diff --check` clean.

## Conclusion

The custom-shader uniforms now carry their per-frame time and resolution (built
on the value type from Experiment 428). The remaining custom-shader work — the
cursor-glyph half of `updateCustomShaderUniformsForFrame` (needs `Contents` /
`getCursorGlyph`), the `updateCustomShaderUniformsFromState` group (needs the
live render `State`), the live timing source, the `Target` enum, and the shader
loading — stays deferred, along with the broader live per-frame call sites and
the `neverExtendBg` terminal-core row/cell access; beyond the renderer, the
other subsystems.

## Completion Review

Codex reviewed the completed implementation and result and **approved** with
**no findings**. It confirmed `update_for_frame` matches the upstream
time/resolution block: it assigns `time` and `time_delta` from the provided
seconds, increments `frame`, sets `resolution` to `[w, h, 1.0]`, and sets
`channel_resolution[0]` to `[w, h, 1.0, 0.0]` (the `1.0` and trailing `0.0`
match upstream exactly). It judged the test to cover the first-frame
assignments, the second-call frame increment, and representative untouched
fields (including `channel_resolution[1]`), and the deferral of the timing
source and cursor-glyph half a clean boundary. No public C ABI/header impact;
nothing needed to change before the result commit.

Review artifacts:

- Prompt: `logs/codex-review/20260604-092948-r429-prompt.md` (result)
- Result: `logs/codex-review/20260604-092948-r429-last-message.md` (result)
