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

# Experiment 440: the font-style config union and its enabled predicate (FontStyle, enabled)

## Description

This experiment ports the `font-style*` config union —
`FontStyle { Default, False, Name(String) }` — **and the predicate** the
renderer uses to derive its per-style enabled flags. Upstream's
`DerivedConfig.init` sets each entry of the font `StyleStatus` from
`config.@"font-style-bold" != .false` (and italic / bold-italic); this
experiment captures that `!= .false` check as a `FontStyle::enabled` method.
roastty already has the `StyleStatus` analog
(`CodepointResolver.styles: [bool; 4]`); the `DerivedConfig.init` wiring that
fills it stays deferred.

## Upstream behavior

In `config/Config.zig`, the union and its `Config` fields (each default
`.default`):

```zig
@"font-style": FontStyle = .{ .default = {} },
@"font-style-bold": FontStyle = .{ .default = {} },
// ... italic, bold-italic

pub const FontStyle = union(enum) {
    /// Use the default font style that font discovery finds.
    default: void,
    /// Disable this font style completely. This will fall back to using
    /// the regular font when this style is encountered.
    false: void,
    /// A specific named font style to use for this style.
    name: [:0]const u8,
    // ... parseCLI / formatEntry
};
```

In `renderer/generic.zig`'s `DerivedConfig.init`, each style's enabled flag is
derived by comparing the config against `.false`:

```zig
var font_styles = font.CodepointResolver.StyleStatus.initFill(true);
font_styles.set(.bold, config.@"font-style-bold" != .false);
font_styles.set(.italic, config.@"font-style-italic" != .false);
font_styles.set(.bold_italic, config.@"font-style-bold-italic" != .false);
```

A style is enabled (`true`) unless its `font-style-*` is `.false`; `.default`
and a `.name` both leave the style enabled. The `!= .false` compares the active
union tag, so only the `false` variant disables the style.

## Rust mapping (`roastty/src/config/mod.rs`)

```rust
/// The `font-style*` config (upstream `FontStyle`): how a font style (bold,
/// italic, …) is selected. The `Config` default is `Default`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum FontStyle {
    /// Use the default font style that font discovery finds.
    Default,
    /// Disable this style completely; fall back to the regular font.
    False,
    /// Use a specific named font style.
    Name(String),
}

impl FontStyle {
    /// Whether this style is enabled (upstream `DerivedConfig.init`'s
    /// `config.@"font-style-*" != .false`): enabled unless `False` — `Default`
    /// and `Name` both leave the style enabled.
    pub(crate) fn enabled(&self) -> bool {
        !matches!(self, FontStyle::False)
    }
}
```

`enabled` is the `!= .false` check: `true` for `Default` and `Name(_)`, `false`
for `False`. The `Name` payload is an owned `String` (upstream `[:0]const u8`),
so `FontStyle` derives `Clone`/`PartialEq`/`Eq` but not `Copy`.

## Scope / faithfulness notes

- **Ported (bridged)**: the `FontStyle` config union (`config/Config.zig`) and
  its enabled predicate (`FontStyle::enabled`, upstream's `DerivedConfig.init`
  `!= .false` derivation).
- **Faithful**: the union has the three upstream variants (`default`, `false`,
  `name`); `enabled` returns `false` only for `False`, `true` for `Default` and
  `Name` — exactly the `!= .false` comparison (which tests the active tag).
- **Faithful adaptation**: the `name` payload is an owned `String` (upstream
  `[:0]const u8`), so `FontStyle` is `Clone`/`Eq` but not `Copy`. The `Config`
  field default (`.default`) is documented on the enum but kept off it (the
  other config types keep defaults on the deferred `Config` struct). The
  consumer is modeled as a method (upstream inlines the `!= .false` comparison
  in `DerivedConfig.init`).
- **Deferred**: the `Config` struct / parsing (`parseCLI` / `formatEntry` and
  the field defaults), and the `DerivedConfig.init` wiring that fills the
  `StyleStatus` (`CodepointResolver.styles`) from `enabled`. (Consumed by a
  later slice; this experiment lands the union and the predicate.)
- No C ABI/header/ABI-inventory change (internal Rust).

## Changes

1. `roastty/src/config/mod.rs`:
   - add `pub(crate) enum FontStyle { Default, False, Name(String) }` (derive
     `Debug, Clone, PartialEq, Eq` — not `Copy`) and
     `FontStyle::enabled(&self) -> bool` (`!matches!(self, FontStyle::False)`).
2. Tests (in `config/mod.rs`):
   - `enabled`: `Default.enabled() == true`, `Name("…").enabled() == true`,
     `False.enabled() == false`; the variants distinct (`Default != False`,
     `Name("a") != Name("b")`) and a `Clone`/`Eq` round-trip.
3. Format and test (`cargo fmt`, accept output).

## Verification

```bash
cargo fmt
cargo fmt -- --check
cargo test -p roastty font_style
cargo test -p roastty
cargo build -p roastty            # no warnings
rg -n 'ghostty|Ghostty|GHOSTTY' roastty/src/font roastty/src/renderer roastty/src/config && exit 1 || true
rg -n 'ghostty|Ghostty|GHOSTTY' roastty/src/lib.rs roastty/include/roastty.h roastty/tests/abi_harness.c && exit 1 || true
git diff --check
```

The experiment **passes** if:

- `FontStyle` has the three upstream variants and `enabled` returns `false` only
  for `False` (`true` for `Default` / `Name`) — faithful to upstream's union and
  the `!= .false` derivation;
- the tests pass (the predicate; the distinct variants; the `Name` payload), and
  the existing tests still pass;
- the `Config` struct and the `DerivedConfig.init` `StyleStatus` wiring stay
  deferred;
- `cargo fmt` accepted, `cargo build -p roastty` has no warnings, and
  `cargo test -p roastty` passes with no regressions;
- the no-`ghostty`-name gates and `git diff --check` pass;
- Codex reviews the design before implementation and the result after, with all
  real findings fixed.

The experiment **fails** if a variant is missing/extra, `enabled` treats
`Default` or `Name` as disabled (or `False` as enabled), an unrelated item
changes, or any public C API/ABI changes.

## Design Review

Codex reviewed this design before implementation and **approved** it with **no
findings**. It verified against the vendored upstream: the four `font-style*`
config fields default to `.{ .default = {} }` (`Config.zig:186`), so keeping the
default on the deferred `Config` struct rather than implementing `Default` on
the enum is consistent with the existing config pattern; the union variants are
exactly `default`, `false`, `name: [:0]const u8` (`Config.zig:8431`), so
`FontStyle::{Default, False, Name(String)}` is the right internal Rust
representation (losing `Copy` because `String` is owned is expected); and
`enabled()` as "anything except `False`" exactly extracts the
`DerivedConfig.init` logic (`generic.zig:596`,
`config.@"font-style-*" != .false`, a tag-based comparison, so `Default` and
`Name(_)` both mean enabled). It judged the test plan (the predicate, payload
equality, distinctness, `Clone`/`Eq`) adequate.

Review artifacts:

- Prompt: `logs/codex-review/20260604-103154-d440-prompt.md` (design)
- Result: `logs/codex-review/20260604-103154-d440-last-message.md` (design)
