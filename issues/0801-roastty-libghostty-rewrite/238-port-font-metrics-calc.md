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

# Experiment 238: Port Font `Metrics::calc` and `clamp`

## Description

Port the metric-derivation core of `font/Metrics.zig`: `calc`, which derives a
`Metrics` from a `FaceMetrics`, and the private `clamp`, which enforces the
`Minimums` table. `Metrics` (Exp 235), `FaceMetrics`, and all its effective
accessors (Exps 236–237) are in place, so `calc` is unblocked.

The `apply`/`ModifierSet`/`Modifier` machinery and font constraint application
remain deferred (they depend on config modifier types).

### `calc` derivation (upstream lines 227–334), preserved exactly

```
face_width  = face.cell_width
face_height = face.lineHeight()
cell_width  = round(face_width)
cell_height = round(face_height)
half_line_gap = line_gap / 2
face_baseline = half_line_gap - descent
cell_baseline = round(face_baseline - (cell_height - face_height) / 2)
face_y        = cell_baseline - face_baseline
top_to_baseline = cell_height - cell_baseline
cap_height = face.capHeight()
underline_thickness     = max(1, ceil(face.underlineThickness()))
strikethrough_thickness = max(1, ceil(face.strikethroughThickness()))
underline_position      = round(top_to_baseline - face.underlinePosition())
strikethrough_position  = round(top_to_baseline - face.strikethroughPosition())
icon_height        = face_height
icon_height_single = (2 * cap_height + face_height) / 3
Metrics{
  cell_width, cell_height, cell_baseline,
  underline_position, underline_thickness,
  strikethrough_position, strikethrough_thickness,
  overline_position = 0, overline_thickness = underline_thickness,
  box_thickness = underline_thickness,
  cursor_height = cell_height,            // cursor_thickness uses the struct default 1
  icon_height, icon_height_single,
  face_width, face_height, face_y,
}.clamp()
```

Rust mapping:

- `round`/`ceil` → `f64::round`/`f64::ceil`; `max(1, …)` on the thicknesses →
  `…ceil().max(1.0)`. `cap_height`/`underlineThickness`/… are the `effective_*`
  accessors from Exps 236–237.
- The `@intFromFloat` conversions go through a private helper
  `f64_to_u32(value)` that
  `debug_assert!(value.is_finite() && value >= 0.0 && value <= u32::MAX as f64)`
  before `value as u32`. Bare `as u32` silently saturates a negative/NaN `f64`
  to `0`, which differs from Zig's checked conversion and would not be caught by
  `clamp` (the unsigned position fields
  `cell_baseline`/`underline_position`/`strikethrough_position` are unclamped).
  The helper catches an invalid derivation in debug/test builds while remaining
  a plain truncation in release. The sources are all `round`/`ceil`/`max(1, …)`
  outputs, so the truncation of these integer-valued `f64`s is exact. The signed
  `overline_position` is a literal `0`. The `f64` fields (`icon_height`,
  `icon_height_single`, `face_width`, `face_height`, `face_y`) stay `f64`.
- `cursor_thickness` is set to `1` explicitly in `calc` (the upstream `Metrics`
  struct default, which the deferred Exp 235 noted `calc` applies).
- `calc` is `pub(crate) fn Metrics::calc(face: FaceMetrics) -> Metrics`.

### `clamp` (upstream lines 434–443)

Upstream's comptime loop sets each `Metrics` field that has a matching
`Minimums` entry to `max(field, minimum)`. Rust has no field reflection, so it
is written explicitly for the **twelve** clamped fields (the others —
`cell_baseline`, `underline_position`, `strikethrough_position`,
`overline_position`, `face_y` — have no minimum and are untouched):

- `u32` min `1`: `cell_width`, `cell_height`, `underline_thickness`,
  `strikethrough_thickness`, `overline_thickness`, `box_thickness`,
  `cursor_thickness`, `cursor_height`.
- `f64` min `1.0`: `icon_height`, `icon_height_single`, `face_height`,
  `face_width`.

`clamp` is `fn clamp(&mut self)` (private), called at the end of `calc`.

### Faithfulness and scope notes

- `calc` has **no isolated upstream test** (the `Metrics.zig` tests cover
  `apply` with `ModifierSet`, deferred). Per the issue's test-parity policy,
  this slice adds **equivalent Roastty tests** that exercise `calc`/`clamp` with
  hand-computed expected values.
- No `apply`/`ModifierSet`/`Modifier`/constraint behavior.
- No C ABI, header, or ABI inventory changes; no new dependencies.

## Changes

1. `roastty/src/font/metrics.rs`:
   - Add
     `impl Metrics { pub(crate) fn calc(face: FaceMetrics) -> Metrics { … } fn clamp(&mut self) { … } }`
     reproducing the formulas above.

2. Tests in `roastty/src/font/metrics.rs`:
   - `calc_derives_clean_metrics`: with
     `cell_width 8, ascent 12, descent -4, line_gap 0` (all optionals `None`) →
     `cell_width 8`, `cell_height 16`, `cell_baseline 4`, `face_y 0`,
     `top_to_baseline 12`, `underline_thickness 2` (`ceil(0.15·6.75=1.0125)=2`),
     `strikethrough_thickness 2`, `underline_position 13`
     (`round(12 − (−1.0125))`), `strikethrough_position 8`
     (`round(12 − 3.88125)`), `overline_position 0`, `overline_thickness 2`,
     `box_thickness 2`, `cursor_thickness 1`, `cursor_height 16`,
     `icon_height 16.0`, `icon_height_single (2·9+16)/3 = 34/3` (epsilon),
     `face_width 8.0`, `face_height 16.0`.
   - `calc_clamps_minimums`: a degenerate face
     (`cell_width 0, ascent 0, descent 0, line_gap 0`) → `cell_width ≥ 1`,
     `cell_height ≥ 1`, `cursor_height ≥ 1`, `icon_height ≥ 1.0`,
     `icon_height_single ≥ 1.0`, `face_width ≥ 1.0`, `face_height ≥ 1.0` (the
     `Minimums` were applied), while the un-clamped `cell_baseline`/positions
     are left as derived.
   - `calc_line_gap_splits_evenly`: a non-zero `line_gap` shifts the baseline
     (half on top, half on bottom) — verify `cell_baseline` moves by the
     expected half-gap relative to the zero-gap case.
   - `clamp_raises_all_twelve_minimum_fields`: a direct `clamp` test (constructs
     a `Metrics` with all twelve clamped fields below their minimum — the eight
     `u32` fields `0`, the four `f64` fields `0.0` — and the five un-clamped
     fields set to recognizable sentinels), calls `clamp`, and asserts all
     twelve are raised to `1`/`1.0` while the five un-clamped fields
     (`cell_baseline`, `underline_position`, `strikethrough_position`,
     `overline_position`, `face_y`) are unchanged. This guards the hand-written
     clamp list (which upstream generates reflectively) against an omitted
     field.

3. Format and test (`cargo fmt`, accept output). Use a `1e-9` epsilon helper for
   `f64` field assertions.

## Verification

```bash
cargo fmt
cargo test -p roastty font
cargo test -p roastty
rg -n 'ghostty|Ghostty|GHOSTTY' roastty/src/font && exit 1 || true
rg -n 'ghostty|Ghostty|GHOSTTY' roastty/src/lib.rs roastty/include/roastty.h roastty/tests/abi_harness.c && exit 1 || true
git diff --check
```

The experiment **passes** if:

- `calc` reproduces the upstream derivation exactly (rounding, baseline
  centering, `max(1, ceil)` thicknesses, the icon-height formula, the position
  derivations, `cursor_thickness = 1`);
- `clamp` enforces the twelve `Minimums` and leaves the five un-clamped fields
  untouched;
- the hand-computed `calc`/`clamp` tests pass;
- no `apply`/`ModifierSet`/constraint scope is pulled in;
- no C ABI, header, or ABI inventory changes;
- `cargo fmt` accepted and `cargo test -p roastty` passes with no regressions;
- the no-`ghostty`-name gates and `git diff --check` pass;
- Codex reviews the design before implementation and the result after, with all
  real findings fixed.

The experiment is **partial** if a derivation step turns out to need a face
accessor or behavior not yet ported.

The experiment **fails** if a formula, rounding, or clamp diverges from
upstream, if `apply`/modifier behavior leaks in, or if any public C API/ABI
changes.

## Design Review

Codex reviewed this design before implementation.

Review artifacts:

- Prompt: `logs/codex-review/20260602-083509-043516-prompt.md`
- Result: `logs/codex-review/20260602-083509-043516-last-message.md`

Codex confirmed the `calc` formulas and the clean expected values are correct
(`cell_baseline = 4`, `underline_position = 13`, `strikethrough_position = 8`,
`icon_height_single = 34/3`), and that the clamp field set is correct (the five
un-clamped fields are `cell_baseline`, `underline_position`,
`strikethrough_position`, `overline_position`, `face_y`).

Two findings, fixed in the design above before this commit:

1. **(Medium)** bare `as u32` would silently saturate a negative/NaN derivation
   to `0`, which `clamp` does not cover for the unsigned position fields. Routed
   the unsigned conversions through a `f64_to_u32` helper that `debug_assert!`s
   the value is finite and in `[0, u32::MAX]` before truncating.
2. **(Low)** added `clamp_raises_all_twelve_minimum_fields`, a direct test that
   all twelve hand-written clamp fields are raised and the five un-clamped
   fields are untouched, guarding the hand-written list against omissions.

## Result

**Result:** Pass

Added `impl Metrics` with `pub(crate) fn calc(face: FaceMetrics) -> Metrics` and
private `fn clamp(&mut self)`, plus the private `f64_to_u32` helper
(debug-asserts finite/in-domain before truncating), to
`roastty/src/font/metrics.rs`. `calc` reproduces the upstream derivation exactly
— `round`ed cell sizes, the line-gap split and baseline centering,
`max(1, ceil(...))` thicknesses, the position derivations, `overline`/`box`
thickness = underline thickness, `cursor_thickness = 1`, the icon-height formula
— then calls `clamp`, which raises the twelve `Minimums` fields and leaves the
five un-clamped ones (`cell_baseline`, the positions, `face_y`) untouched.

Tests added (4): `calc_derives_clean_metrics` (a known face → all derived fields
hand-verified, e.g. `cell_baseline 4`, `underline_position 13`,
`strikethrough_position 8`, `icon_height_single 34/3`), `calc_clamps_minimums`
(degenerate face → minimums applied), `calc_line_gap_splits_evenly` (a 4px gap
grows cell height by 4 and shifts the baseline by 2), and
`clamp_raises_all_twelve_minimum_fields` (direct clamp test of all twelve fields
plus the five un-clamped). All hand-computed expectations passed on the first
run.

### Verification

```bash
cargo fmt -p roastty
cargo test -p roastty font
cargo test -p roastty
```

Observed:

- `font`: 23 passed (19 prior + 4 new).
- Full `roastty`: 2299 unit tests passed (2295 prior + 4 new), plus the C ABI
  harness passed.
- `cargo fmt -p roastty -- --check`: clean.
- `cargo build -p roastty`: no warnings.
- No-`ghostty`-name gates passed for `roastty/src/font` and for
  `roastty/src/lib.rs`, `roastty/include/roastty.h`,
  `roastty/tests/abi_harness.c`.
- `git diff --check`: clean.

No C ABI, header, or ABI inventory changes; no `apply`/`ModifierSet`/constraint
scope pulled in.

### Completion Review

Codex reviewed the completed implementation and found **no issues** ("nothing
needs to change before the result commit").

Review artifacts:

- Prompt: `logs/codex-review/20260602-083949-010383-prompt.md`
- Result: `logs/codex-review/20260602-083949-010383-last-message.md`

Codex confirmed `calc` matches upstream (unrounded face metrics, rounded cell
dimensions, baseline centering, `ceil(...).max(1.0)` thicknesses, rounded
positions, the icon-height formula, `cursor_thickness = 1`, final `clamp`), that
`clamp` clamps exactly the twelve `Minimums` fields and leaves the five
unclamped, that `f64_to_u32` is a sound scoped helper used for all unsigned
conversions, and that the four tests (including the hand-computed clean values)
are correct.

## Conclusion

Experiment 238 succeeds — the substantive derivation core of `font/Metrics.zig`
is ported. `Metrics::calc` turns a `FaceMetrics` into a `Metrics` with the
upstream rounding/centering/clamping, validated by hand-computed equivalent
tests (upstream has no isolated `calc` test). Both Codex gates passed (two
design findings fixed; zero result findings).

What remains of `font/Metrics.zig` is the `ModifierSet`/`Modifier` config types
and `Metrics::apply` (the runtime metric-adjustment path, which carries the
file's actual upstream tests), plus the font-constraint application. Those
depend on a config-modifier representation and are the next Metrics slices.
Beyond `Metrics`, the font stack continues toward the glyph `Atlas` and the
CoreText face/rasterization core.
