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

# Experiment 575: config float formatter (background-image-opacity)

## Description

This experiment ports the config **float formatting** branch from upstream
`config/formatter.zig`'s `formatEntry` (the `.float` case, `{d}`), and uses it
to wire `background-image-opacity` into roastty's `Config::format_config` — the
one field that was left omitted as "float-formatting blocked (Experiment 509)".
roastty already stores `bg_image_opacity: f32`; this adds
`EntryFormatter::entry_float` and emits the field's line.

## Upstream behavior

`config/formatter.zig`'s `formatEntry`, for a float-typed field:

```zig
.float => {
    try writer.print("{s} = {d}\n", .{ name, value });
    return;
},
```

`{d}` formats the float as the **shortest round-trippable decimal**, in
**decimal notation** (never scientific), with **no trailing `.0`** for whole
values: `1.0` → `1`, `0.5` → `0.5`, `0.25` → `0.25`, `0` → `0`.
(`background-image-opacity` is `f32 = 1.0` upstream, so its default line is
`background-image-opacity = 1`.)

## Rust mapping

Rust's `Display` for `f32` is **also** the shortest round-trippable decimal,
always in decimal notation (never scientific), with no trailing `.0` — so
`format!("{}", value)` reproduces Zig's `{d}` for every finite value. The single
divergence is `NaN`: Rust prints `NaN`, Zig prints `nan`;
`background-image-opacity` is a finite opacity (never `NaN`), but `entry_float`
special-cases it to stay faithful to the general `.float` branch. `inf` / `-inf`
already match between Rust `Display` and Zig `{d}`.

```rust
// roastty/src/config/formatter.rs — a new EntryFormatter method:

/// `name = <shortest-decimal>\n` (upstream the `float` / `{d}` case). Rust's `f32` `Display` is the
/// shortest round-trippable decimal in decimal notation (never scientific, no trailing `.0`),
/// matching Zig's `{d}` for every finite value; `NaN` is written `nan` (Zig's spelling) rather than
/// Rust's `NaN`.
pub(crate) fn entry_float(&mut self, value: f32) {
    if value.is_nan() {
        let _ = writeln!(self.out, "{} = nan", self.name);
    } else {
        let _ = writeln!(self.out, "{} = {}", self.name, value);
    }
}
```

The `Config::format_config` deferral is then replaced with the real line, in
declaration order (after `foreground`, before `background-image-position`):

```rust
// was: // background-image-opacity (f32) — float-formatting blocked (Exp 509), deferred.
EntryFormatter::new("background-image-opacity", out).entry_float(self.bg_image_opacity);
```

## Scope / faithfulness notes

- **Ported**: the `.float` (`{d}`) branch of upstream `formatEntry` →
  `EntryFormatter::entry_float`, and the `background-image-opacity` line in
  `Config::format_config` (previously deferred).
- **Faithful**: `{d}` (shortest round-trippable decimal, decimal notation, no
  trailing `.0`) is reproduced by Rust's `f32` `Display`, which guarantees the
  same — the shortest decimal that round-trips to the same `f32` is unique, so
  the digit strings match. The field is emitted at the correct declaration-order
  position.
- **Faithful adaptation**: `NaN` is special-cased to `nan` (Zig's spelling) so
  `entry_float` matches the general `{d}` branch; `inf` / `-inf` already
  coincide between Rust `Display` and Zig `{d}`. `background-image-opacity`
  itself is always finite (an opacity), so in practice only the finite decimal
  path is exercised.
- **Unblocks**: Experiment 509's "float-formatting blocked" deferral for this
  field. No other config field uses the `.float` branch in roastty's current
  `format_config` (the other `background-image-*` fields are enums / a bool,
  already formatted).
- No C ABI/header/ABI-inventory change (internal Rust). Extends
  `config::formatter` and `Config::format_config`.

## Changes

1. `roastty/src/config/formatter.rs`: add
   `EntryFormatter::entry_float(&mut self, value: f32)`.
2. `roastty/src/config/mod.rs`: replace the `background-image-opacity` deferral
   comment with
   `EntryFormatter::new("background-image-opacity", out).entry_float(self.bg_image_opacity);`,
   and update the `format_config` doc comment (drop the "omitted / blocked"
   note).
3. Tests:
   - In `formatter.rs`: `entry_float` writes `name = 1` for `1.0`, `name = 0.5`
     for `0.5`, `name = 0.25` for `0.25`, `name = 0` for `0.0`, and `name = nan`
     for `f32::NAN`.
   - In `mod.rs` (or wherever `format_config` is tested): the formatted config
     now contains `background-image-opacity = 1` for the default config, in the
     right position.
4. Format and test (`cargo fmt`, accept output).

## Verification

```bash
cargo fmt
cargo fmt -- --check
cargo test -p roastty config
cargo test -p roastty
cargo build -p roastty            # no warnings
rg -n 'ghostty|Ghostty|GHOSTTY' roastty/src/font roastty/src/renderer roastty/src/config/formatter.rs roastty/src/config/mod.rs && exit 1 || true
rg -n 'ghostty|Ghostty|GHOSTTY' roastty/src/lib.rs roastty/include/roastty.h roastty/tests/abi_harness.c && exit 1 || true
git diff --check
```

The experiment **passes** if:

- `entry_float` reproduces Zig `{d}` (shortest round-trippable decimal, no
  trailing `.0`, `nan` spelling) and `background-image-opacity` is emitted in
  `format_config` at the correct position — faithful to `config/formatter.zig`
  and the upstream declaration order;
- the tests pass (`entry_float` cases / the default config line), and the
  existing tests still pass;
- `cargo fmt` accepted, `cargo build -p roastty` has no warnings, and
  `cargo test -p roastty` passes with no regressions;
- the no-`ghostty`-name gates and `git diff --check` pass;
- Codex reviews the design before implementation and the result after, with all
  real findings fixed.

The experiment **fails** if the float formatting diverges from Zig `{d}`, the
field is emitted in the wrong position, an unrelated item changes, or any public
C API/ABI changes.

## Design Review

Codex reviewed the design and **approved it with no findings**. It **verified
the risky formatter claim locally** against Zig `{d}` and Rust `f32` `Display`
for representative values — `1`, fractions, `0`, `-0`, `inf`, `-inf`, a very
large finite, a very small finite, and a subnormal — and confirmed the outputs
match for all finite/infinite cases; the only observed spelling divergence is
`NaN` vs Zig's `nan`, so the explicit `NaN` special-case is correct and
sufficient. Codex also confirmed `background-image-opacity = 1` is faithful for
the `1.0` default, that placing it after `foreground` and before
`background-image-position` matches upstream declaration order, and that the
scope (the previously deferred `.float` branch + the single existing float
field) is appropriately bounded. The test plan is sound.

Review artifacts:

- Prompt: `logs/codex-review/20260604-d575-prompt.md`
- Result: `logs/codex-review/20260604-d575-last-message.md`

## Result

**Result:** Pass

`EntryFormatter::entry_float(&mut self, value: f32)` was added (NaN ⇒
`name = nan`; otherwise `name = {value}` via Rust's shortest-round-trip
`Display`), and `Config::format_config` now emits `background-image-opacity` via
`entry_float(self.bg_image_opacity)` in declaration order (after `foreground`,
before `background-image-position`), replacing the Experiment-509 deferral
comment. The `format_config` doc comment dropped its "omitted / blocked" note.

Gates:

- `cargo fmt -p roastty` accepted; `--check` clean.
- `cargo test -p roastty`: 3174 passed, 0 failed (one net new test; no
  regressions, up from 3173; config subset 123 passed).
- `cargo build -p roastty`: no warnings.
- no-`ghostty`-name greps (font/renderer + config/formatter.rs + config/mod.rs +
  lib.rs/header/abi_harness.c) clean; `git diff --check` clean.

Tests: `entry_float_writes_shortest_decimal` (`1.0` ⇒ `a = 1`, `0.5`, `0.25`,
`0.0` ⇒ `a = 0`, `0.75`, and `f32::NAN` ⇒ `a = nan`); the `format_config`
declaration-order test now includes `background-image-opacity` at the right
position and asserts the default `background-image-opacity = 1` line (replacing
the old "intentionally absent" assertion).

## Completion Review

Codex reviewed the completed experiment and **approved** it with **no Required
or Optional findings** (one Nit: the `## Result` / `## Conclusion` sections were
not yet in the saved file — added here as part of result recording). Codex
confirmed `entry_float` matches the approved `{d}` mapping (Rust `Display` for
finite values, `inf` / `-inf` unchanged, explicit lowercase `nan`), the config
wiring is in the correct declaration-order position between `foreground` and
`background-image-position`, the default output `background-image-opacity = 1`
is faithful for `1.0`, and the updated formatter and config-order tests cover
the relevant behavior.

Review artifacts:

- Prompt: `logs/codex-review/20260604-r575-prompt.md` (result)
- Result: `logs/codex-review/20260604-r575-last-message.md` (result)

## Conclusion

This experiment **unblocks Experiment 509's deferred float formatter**: it ports
the `.float` (`{d}`) branch of upstream `config/formatter.zig` as
`EntryFormatter::entry_float` — Rust's `f32` `Display` being the faithful
equivalent of Zig's `{d}` (shortest round-trippable decimal, never scientific,
no trailing `.0`), with `NaN` special-cased to Zig's `nan` spelling — and wires
the one previously-omitted config field, `background-image-opacity`, into
`Config::format_config`. With the `half::f16` dependency (Experiment 573) and
now this float formatter, **both** items that the session had carried as
"float-blocked" are resolved. The remaining big-ticket subsystem is the terminal
**search subsystem** (coupled to `PageList` / `Pin` / `Screen` / `Selection` /
`PageFormatter`); the dependency-blocked helpers persist (regex/oniguruma for
`Link::oniRegex`, a URI parser for `os/uri`, the config-directory naming
decision for `file_load` / `edit` / `loadDefaultFiles`); and the remaining
split_tree work (the `Node`-over-`View`-generic arena and ref-counting, then
`nearest` / `nearestWrapped` and the `Spatial` container) is the next split_tree
design question.
