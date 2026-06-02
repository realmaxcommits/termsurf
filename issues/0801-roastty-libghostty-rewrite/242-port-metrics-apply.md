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

# Experiment 242: Port `Metrics::apply` (modifier dispatch + cell-height re-centering)

## Description

Port the struct-level `Metrics::apply` from upstream `font/Metrics.zig` — the
method that walks a `ModifierSet` and applies each `Modifier` to the metric it
keys, plus the `addFloatToInt` helper it uses and the private `init`
zero-constructor its tests use. This completes the `Metrics.zig` modifier
behavior: with `Modifier`/`parse`/`apply_*` (Exps 239–240), `Key`/`ModifierSet`
(Exp 241), and now `Metrics::apply`, the metric-adjustment path is whole, and
this slice carries the actual upstream `Metrics.zig` modifier tests.

This is the one behavior-heavy slice of the file, but it is a single coherent
method over types that are already in place, with the upstream tests as the
exact pass criteria, so it stays one experiment.

### Upstream `apply` (lines 337–416)

```zig
pub fn apply(self: *Metrics, mods: ModifierSet) void {
    var it = mods.iterator();
    while (it.next()) |entry| {
        switch (entry.key_ptr.*) {
            inline .cell_width, .cell_height => |tag| {
                const original = @field(self, @tagName(tag));
                const new = @max(entry.value_ptr.apply(original), 1);
                if (new == original) continue;
                @field(self, @tagName(tag)) = new;
                if (comptime tag == .cell_height) {
                    // … baseline re-centering (see below) …
                }
            },
            inline .icon_height => {
                self.icon_height = entry.value_ptr.apply(self.icon_height);
                self.icon_height_single = entry.value_ptr.apply(self.icon_height_single);
            },
            inline else => |tag| {
                @field(self, @tagName(tag)) = entry.value_ptr.apply(@field(self, @tagName(tag)));
            },
        }
    }
    self.clamp();
}
```

Three cases:

1. **`cell_width` / `cell_height`** — compute `new = max(apply(original), 1)`
   (the `max(…, 1)` prevents a divide-by-zero downstream); if unchanged, skip.
   For `cell_height` only, also re-center the baseline (below).
2. **`icon_height`** — the one modifier key that fans out to two fields:
   `icon_height` **and** `icon_height_single` both get the modifier applied.
3. **every other key** (`inline else`) — apply the modifier to the single field
   the key names, using the field's own numeric type.

After the loop, `clamp()` re-bounds everything.

### Cell-height baseline re-centering (lines 358–401)

When `cell_height` changes, positions measured from the cell edges must move so
the baseline stays vertically centered:

```zig
const original_f64: f64 = @floatFromInt(original);
const new_f64: f64 = @floatFromInt(new);
const diff = new_f64 - original_f64;
const half_diff = diff / 2.0;
const position_with_respect_to_center =
    self.face_y - (original_f64 - self.face_height) / 2;
const diff_top, const diff_bottom =
    if (position_with_respect_to_center > 0)
        .{ @ceil(half_diff), @floor(half_diff) }
    else
        .{ @floor(half_diff), @ceil(half_diff) };
addFloatToInt(&self.cell_baseline, diff_bottom);
self.face_y += diff_bottom;
addFloatToInt(&self.underline_position, diff_top);
addFloatToInt(&self.strikethrough_position, diff_top);
self.overline_position +|= @as(i32, @intFromFloat(diff_top));
```

The odd extra pixel (when `diff` is odd) goes to whichever edge "needs it" —
decided by whether the face currently sits above or below the centered position.
`cell_baseline` and `face_y` are bottom-relative (get `diff_bottom`);
`underline_position`, `strikethrough_position`, `overline_position` are
top-relative (get `diff_top`). `cursor_height` is deliberately **not** touched —
it does not follow cell height.

### `addFloatToInt` (lines 424–431)

```zig
inline fn addFloatToInt(int: *u32, float: f64) void {
    assert(@floor(float) == float);
    int.* = if (float >= 0.0)
        int.* +| @as(u32, @intFromFloat(float))
    else
        int.* -| @as(u32, @intFromFloat(-float));
}
```

Saturating add (or saturating subtract, for a negative float) of an
integer-valued `f64` to a `u32`.

### `init` (lines 609–628)

A private all-zero `Metrics` constructor. Upstream keeps it non-`pub` so callers
use struct-literal syntax; its only callers are the modifier tests in this file.

### Rust mapping

- `pub(crate) fn apply(&mut self, mods: &ModifierSet)` on `impl Metrics`:
  - Iterate the keys in a **fixed `Key` order** rather than over the `HashMap`'s
    randomized order:
    `for key in Key::ALL { let Some(modifier) = mods.get(&key) else { continue }; … }`
    (`modifier: &Modifier`; `Modifier` is `Copy`, so `modifier.apply_*(…)`
    copies it out). This is the determinism fix from the design review (see
    Faithfulness notes): a few keys (`CellHeight`, `IconHeight`) write fields
    that another key can also target, so iteration order is observable for
    overlapping sets; a fixed order makes `apply` reproducible. `Key::ALL` is
    the seventeen variants in discriminant order (lifted from the existing
    test-only `ALL_KEYS` to a `pub(crate) const ALL: [Key; 17]` on `Key`, now
    also used by the `key_discriminants` test).
  - `match key`:
    - `Key::CellWidth | Key::CellHeight`: read `original` from the right field;
      `let new = modifier.apply_u32(original).max(1); if new == original { continue; }`;
      write it back; if the key is `CellHeight`, run the re-centering block.
    - `Key::IconHeight`: apply `apply_f64` to **both** `icon_height` and
      `icon_height_single`.
    - one arm per remaining field, applying the field's typed method
      (`apply_u32` for the `u32` fields, `apply_i32` for `overline_position`,
      `apply_f64` for `icon_height_single`/`face_width`/`face_height`/`face_y`).
    - The match is **exhaustive with no wildcard** — all 17 `Key` variants are
      named, so adding/removing a `Key` forces updating `apply`.
  - After the loop, `self.clamp()`.
- Re-centering block (only for `CellHeight`):
  - `let original_f64 = original as f64; let new_f64 = new as f64;`
  - `let half_diff = (new_f64 - original_f64) / 2.0;`
  - `let position_with_respect_to_center = self.face_y - (original_f64 - self.face_height) / 2.0;`
  - `let (diff_top, diff_bottom) = if position_with_respect_to_center > 0.0 { (half_diff.ceil(), half_diff.floor()) } else { (half_diff.floor(), half_diff.ceil()) };`
  - `add_float_to_int(&mut self.cell_baseline, diff_bottom); self.face_y += diff_bottom;`
  - `add_float_to_int(&mut self.underline_position, diff_top); add_float_to_int(&mut self.strikethrough_position, diff_top);`
  - `self.overline_position = self.overline_position.saturating_add(diff_top as i32);`
    (`diff_top` is integer-valued from `ceil`/`floor`; `as i32` matches
    `@intFromFloat`'s truncation, and `saturating_add` matches `+|=`).
- Module-private free function `fn add_float_to_int(int: &mut u32, float: f64)`:
  - `debug_assert!(float.floor() == float);` (upstream's `assert` is a Zig
    safe-mode precondition; the only callers pass `ceil`/`floor` outputs, which
    are integer-valued by construction, so `debug_assert!` — the Rust analog,
    active in debug, compiled out in release — is the faithful mapping and can
    never fire here);
  - `*int = if float >= 0.0 { int.saturating_add(float as u32) } else { int.saturating_sub((-float) as u32) };`
    (`float as u32` is a saturating cast standing in for `@intFromFloat`; the
    `saturating_add`/`saturating_sub` are `+|`/`-|`).
- Module-private `fn zeroed() -> Metrics` mirroring upstream `init()`: every
  field `0`/`0.0` **except `cursor_thickness: 1`**. Upstream `init()` omits
  `cursor_thickness` ([Metrics.zig:609](#)), so the field's struct default
  (`cursor_thickness: u32 = 1`, [Metrics.zig:34](#)) applies; `zeroed()` must
  set it to `1` to match. The `font` module is `#![allow(dead_code)]`, so a
  non-`pub` helper used only by tests does not warn. (Named `zeroed` rather than
  `init` for Rust clarity; semantics identical.)

### Faithfulness and scope notes

- Iteration order **is** observable for overlapping modifier sets: most keys
  target a distinct field, but `CellHeight` also writes `cell_baseline`,
  `face_y`, `underline_position`, `strikethrough_position`, and
  `overline_position`, and `IconHeight` also writes `icon_height_single` — so a
  set containing, e.g., both `CellHeight` and `UnderlinePosition` (or
  `IconHeight` and `IconHeightSingle`) yields order-dependent results. Upstream
  iterates an `AutoHashMapUnmanaged` ([Metrics.zig:337](#)), leaving that order
  unspecified; Rust's `HashMap` randomizes it per-run, which would be
  nondeterministic. The port therefore iterates in a **fixed `Key` order**
  (`Key::ALL`, discriminant order), looking each key up in the set —
  deterministic and within upstream's unspecified-order contract. For the
  single-key sets the config and these tests use, the order is immaterial.
- `apply_u32`/`apply_i32`/`apply_f64` (Exp 240) already encode the per-type
  rounding/saturation, so `apply` only routes keys to fields and carries the
  cell-height geometry.
- No `parseCLI`/`formatEntry`/`hash` (config/formatter integration) and no
  constraint application — those remain later slices.
- No C ABI, header, or ABI inventory changes; no new dependencies (std only).

## Changes

1. `roastty/src/font/metrics.rs`:
   - Add `pub(crate) fn apply(&mut self, mods: &ModifierSet)` to `impl Metrics`.
   - Add module-private `fn add_float_to_int(int: &mut u32, float: f64)`.
   - Add module-private `fn zeroed() -> Metrics` (`cursor_thickness: 1`, rest
     `0`/`0.0`).
   - Lift the test-only `ALL_KEYS: [Key; 17]` to a `pub(crate) const Key::ALL`
     (so `apply` iterates a fixed order) and point the `key_discriminants` test
     at it.
   - Refresh the module doc comment to note `Metrics::apply` is now ported.

2. Tests in `roastty/src/font/metrics.rs` — port the five upstream `Metrics`
   modifier tests (build a `ModifierSet`, set fields via `zeroed()`, `apply`,
   assert):
   - `apply_modifiers`: `cell_width = 100`, `{CellWidth: Percent(1.2)}` →
     `cell_width == 120`.
   - `apply_cell_height_smaller`: the upstream odd-pixel-down case
     (`{CellHeight: Percent(0.75)}`, `face_y = 0.33`, `cell_baseline = 50`,
     `underline_position = 55`, `strikethrough_position = 30`,
     `overline_position = 0`, `cell_height = 100`, `face_height = 99.67`,
     `cursor_height = 100`) → `face_y ≈ -12.67` (epsilon), `cell_height == 75`,
     `cell_baseline == 37`, `underline_position == 43`,
     `strikethrough_position == 18`, `overline_position == -12`,
     `cursor_height == 100` (untouched).
   - `apply_cell_height_larger`: the upstream odd-pixel-up case
     (`{CellHeight: Percent(1.75)}`, same starting fields) → `face_y ≈ 37.33`,
     `cell_height == 175`, `cell_baseline == 87`, `underline_position == 93`,
     `strikethrough_position == 68`, `overline_position == 38`,
     `cursor_height == 100`.
   - `apply_icon_height_percent`: `{IconHeight: Percent(0.75)}`,
     `icon_height = 100`, `icon_height_single = 80`, `face_height = 100`,
     `face_y = 1` → `icon_height == 75`, `icon_height_single == 60`,
     `face_height == 100`, `face_y == 1` (face untouched).
   - `apply_icon_height_absolute`: `{IconHeight: Absolute(-5)}`, same starting
     fields → `icon_height == 95`, `icon_height_single == 75`, face untouched.
   - (`Modifier` is built directly, e.g. `Modifier::Percent(0.75)`, matching the
     upstream tests' direct `.{ .percent = 0.75 }` construction — not via
     `parse`.)
   - `add_float_to_int_saturates` (a direct helper test, since the five upstream
     `apply` tests exercise only ordinary add/subtract): a positive float that
     overflows saturates to `u32::MAX` (`int = u32::MAX - 1`, `+2.0` →
     `u32::MAX`), and a negative float that underflows saturates to `0`
     (`int = 1`, `-3.0` → `0`); plus an ordinary add and subtract.

3. Format and test (`cargo fmt`, accept output).

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

- `Metrics::apply` reproduces upstream exactly — the `max(…, 1)`-and-skip for
  `cell_width`/`cell_height`, the `icon_height` fan-out to both icon fields, the
  typed per-field application for every other key, the cell-height baseline
  re-centering (the `diff_top`/`diff_bottom` split, `cell_baseline`/`face_y`
  bottom-relative, `underline`/`strikethrough`/`overline` top-relative,
  `cursor_height` untouched), and the trailing `clamp`;
- `add_float_to_int` saturates correctly for positive (add) and negative
  (subtract) integer-valued floats;
- all five ported upstream tests pass with the exact expected values;
- no `parseCLI`/`formatEntry`/`hash`/constraint scope is pulled in;
- no C ABI, header, or ABI inventory changes;
- `cargo fmt` accepted and `cargo test -p roastty` passes with no regressions;
- the no-`ghostty`-name gates and `git diff --check` pass;
- Codex reviews the design before implementation and the result after, with all
  real findings fixed.

The experiment is **partial** if applying the upstream tests reveals a field
type or rounding edge that needs its own reconciling change.

The experiment **fails** if any of the five upstream tests produce a value other
than the expected one (especially the odd-pixel split or the `face_y`/
`overline_position` updates), if the `cursor_height`-untouched invariant breaks,
if out-of-scope behavior leaks in, or if any public C API/ABI changes.

## Design Review

Codex reviewed this design before implementation.

Review artifacts:

- Prompt: `logs/codex-review/20260602-090246-700280-prompt.md`
- Result: `logs/codex-review/20260602-090246-700280-last-message.md`

Codex traced the re-centering math and confirmed all five upstream expected
value sets are correct (smaller: `face_y -12.67`, `cell_height 75`,
`cell_baseline 37`, `underline 43`, `strikethrough 18`, `overline -12`; larger:
`37.33`, `175`, `87`, `93`, `68`, `38`; icon fan-out `75/60` and `95/75`).

Three findings, all fixed in the design above before this commit:

1. **Medium — false order-independence claim.** The original Faithfulness note
   said every key targets a distinct field, so order was immaterial. That is
   wrong: `CellHeight` and `IconHeight` write extra fields another key can also
   target, and Rust's `HashMap` randomizes iteration per-run. Fixed: `apply` now
   iterates a **fixed `Key::ALL` order** (lifted from the test-only `ALL_KEYS`
   to a `pub(crate) const`), making it deterministic and within upstream's
   unspecified-order contract; the note was rewritten to state this.
2. **Low — `zeroed()` must set `cursor_thickness: 1`.** Upstream `init()` omits
   `cursor_thickness`, so its struct default of `1` applies. `zeroed()` now sets
   `cursor_thickness: 1` (rest `0`/`0.0`).
3. **Low — no saturation coverage.** The five upstream tests exercise only
   ordinary add/subtract. Added a direct `add_float_to_int_saturates` test for
   the saturating-add (to `u32::MAX`) and saturating-subtract (to `0`) paths.

## Result

**Result:** Pass

Added `Metrics::apply(&mut self, mods: &ModifierSet)` to `impl Metrics`, the
module-private `add_float_to_int` and `zeroed` helpers, and the
`pub(crate) const Key::ALL` (lifted from the test-only `ALL_KEYS`) to
`roastty/src/font/metrics.rs`, and refreshed the module doc comment. `apply`
iterates `Key::ALL` in discriminant order, looking each key up in the set:
`CellWidth`/`CellHeight` clamp to a minimum of 1 and skip when unchanged, a
`CellHeight` change re-centers the baseline-relative positions (the
`diff_top`/`diff_bottom` ceil/floor split, `cell_baseline`/`face_y`
bottom-relative, `underline`/`strikethrough`/`overline_position` top-relative,
`cursor_height` untouched), `IconHeight` fans out to both icon fields, and every
other key applies the field's typed `apply_*`; then a trailing `clamp`.
`zeroed()` sets `cursor_thickness: 1` (rest `0`/`0.0`) to mirror upstream
`init()`.

Tests added (6): the five upstream `Metrics` modifier tests (`apply_modifiers`,
`apply_cell_height_smaller`, `apply_cell_height_larger`,
`apply_icon_height_percent`, `apply_icon_height_absolute`) and the direct
`add_float_to_int_saturates` helper test. `face_y` comparisons use the `1e-9`
epsilon helper.

### Verification

```bash
cargo fmt -p roastty
cargo test -p roastty font
cargo test -p roastty
```

Observed:

- `font`: 42 passed (36 prior + 6 new).
- Full `roastty`: 2318 unit tests passed (2312 prior + 6 new), plus the C ABI
  harness passed.
- `cargo fmt -p roastty -- --check`: clean.
- `cargo build -p roastty`: no warnings.
- No-`ghostty`-name gates passed for `roastty/src/font` and for
  `roastty/src/lib.rs`, `roastty/include/roastty.h`,
  `roastty/tests/abi_harness.c`.
- `git diff --check`: clean.

All five upstream expected value sets reproduced exactly (smaller:
`face_y ≈ -12.67`, `cell_height 75`, `cell_baseline 37`, `underline 43`,
`strikethrough 18`, `overline -12`, `cursor 100`; larger: `face_y ≈ 37.33`,
`175`, `87`, `93`, `68`, `38`, `100`; icon percent `75/60`, icon absolute
`95/75`, face untouched). No C ABI, header, or ABI inventory changes; no
`parseCLI`/`formatEntry`/`hash`/constraint scope pulled in.

### Completion Review

Codex reviewed the completed implementation and found **no issues** ("nothing
needs to change before the result commit").

Review artifacts:

- Prompt: `logs/codex-review/20260602-090824-566698-prompt.md`
- Result: `logs/codex-review/20260602-090824-566698-last-message.md`

Codex verified against upstream and the approved design that `apply` iterates
`Key::ALL` in deterministic discriminant order with map lookups, that the
`cell_width`/`cell_height` `max(…, 1)`-and-skip and the `cell_height` ceil/floor
re-centering update the correct top/bottom-relative fields with `cursor_height`
untouched, that `IconHeight` fans out to both icon fields, that the per-field
dispatch is exhaustive with no wildcard, that `add_float_to_int` matches the
upstream saturating add/subtract, that `zeroed()` preserves the implicit
`cursor_thickness = 1`, and that `apply` ends with `clamp()`. It confirmed the
six tests' expected values match upstream.

## Conclusion

Experiment 242 succeeds, completing the `font/Metrics.zig` behavior: the full
metric-modifier path — `Modifier`/`parse`/`apply_*` (239–240),
`Key`/`ModifierSet` (241), and now `Metrics::apply` with its cell-height
re-centering — is ported, along with the `addFloatToInt` helper and the `init`
zero-constructor. Both Codex gates passed (three design findings fixed — the
deterministic `Key::ALL` iteration, the `cursor_thickness: 1` default, and the
saturation test; zero result findings). The design-review catch of the `HashMap`
non-determinism made the port stricter than a literal transliteration would have
been.

With `Metrics.zig` complete, the next slice moves to the remaining font layer.
The candidates are the font `Collection`/`face` plumbing and the CoreText face
metrics extraction that feeds `FaceMetrics` into `calc`, then glyph
rasterization and the Atlas. The next experiment will port the smallest coherent
next type in that path (likely the face/`FaceMetrics` source or the `Collection`
entry types), keeping the same one-surface, predictable-tests sizing.
