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

# Experiment 446: the config TerminalColor type and its terminal-RGB conversion (TerminalColor, to_terminal_rgb)

## Description

This experiment ports the config `TerminalColor` union — a color that is either
an explicit `Color` or one of the sentinels `cell-foreground` /
`cell-background` — and its `to_terminal_rgb` conversion. It builds directly on
the `Color` value type (Experiment 445). The renderer's `DerivedConfig` holds
several `?TerminalColor` keys (`cursor_color`, `cursor_text`,
`selection_background`, `selection_foreground`, and the `search_*` colors);
upstream's `toTerminalRGB` resolves an explicit `Color` to an `Rgb` and resolves
the cell sentinels to `null` (meaning "use the cell's own fg/bg"). This
experiment lands the union and that conversion; the renderer's resolution of the
`null`/`None` sentinels to the actual cell colors stays deferred.

## Upstream behavior

In `config/Config.zig`:

```zig
pub const TerminalColor = union(enum) {
    color: Color,
    @"cell-foreground",
    @"cell-background",

    pub fn parseCLI(input_: ?[]const u8) !TerminalColor {
        const input = input_ orelse return error.ValueRequired;
        if (std.mem.eql(u8, input, "cell-foreground")) return .@"cell-foreground";
        if (std.mem.eql(u8, input, "cell-background")) return .@"cell-background";
        return .{ .color = try Color.parseCLI(input) };
    }

    pub fn toTerminalRGB(self: TerminalColor) ?terminal.color.RGB {
        return switch (self) {
            .color => |v| v.toTerminalRGB(),
            .@"cell-foreground", .@"cell-background" => null,
        };
    }
    // ... formatEntry
};
```

`TerminalColor` is either an explicit `color` or the sentinel `cell-foreground`
/ `cell-background`. `toTerminalRGB` returns the explicit color's `Rgb` for
`.color` and `null` for the two sentinels — the `null` signals the consumer to
use the cell's own foreground / background instead of a fixed color.

## Rust mapping (`roastty/src/config/mod.rs`)

Building on `Color` (Experiment 445):

```rust
/// A config terminal-color value (upstream `Config.TerminalColor`): either an
/// explicit `Color` or a cell-relative sentinel (use the cell's own fg / bg).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TerminalColor {
    /// An explicit color.
    Color(Color),
    /// Use the cell's own foreground color.
    CellForeground,
    /// Use the cell's own background color.
    CellBackground,
}

impl TerminalColor {
    /// Resolve to the terminal-native `Rgb` (upstream `TerminalColor.toTerminalRGB`):
    /// an explicit `Color` resolves to its `Rgb`; the cell sentinels resolve to
    /// `None` (the consumer uses the cell's own fg / bg).
    pub(crate) fn to_terminal_rgb(self) -> Option<Rgb> {
        match self {
            TerminalColor::Color(c) => Some(c.to_terminal_rgb()),
            TerminalColor::CellForeground | TerminalColor::CellBackground => None,
        }
    }
}
```

`to_terminal_rgb` is upstream's `toTerminalRGB`: `Some(color.to_terminal_rgb())`
for an explicit color, `None` for the cell sentinels. The `match` is exhaustive.
`TerminalColor` is `Copy`/`Eq` (`Color` is `Copy`).

## Scope / faithfulness notes

- **Ported (bridged)**: the config `TerminalColor` union (`config/Config.zig`)
  and its `to_terminal_rgb` conversion (upstream `TerminalColor.toTerminalRGB`).
- **Faithful**: the union has the three upstream variants (`color`,
  `cell-foreground`, `cell-background`); `to_terminal_rgb` returns the explicit
  color's `Rgb` for `Color`, `None` for both sentinels — exactly upstream's
  `switch`.
- **Faithful adaptation**: the `color` payload is the `Color` value type
  (Experiment 445); the upstream tags `cell-foreground` / `cell-background` map
  to `CellForeground` / `CellBackground`. The `None` return preserves upstream's
  `null` sentinel (the consumer resolves it to the cell's own color).
- **Deferred**: the string parsing (`parseCLI`), the `formatEntry`, the `Config`
  struct that holds `?TerminalColor` keys, and the renderer / terminal
  resolution of the `None` sentinel to the cell's actual foreground / background
  color. (Consumed by a later slice; this experiment lands the union and the
  conversion.)
- No C ABI/header/ABI-inventory change (internal Rust).

## Changes

1. `roastty/src/config/mod.rs`:
   - add
     `pub(crate) enum TerminalColor { Color(Color), CellForeground, CellBackground }`
     (derive `Debug, Clone, Copy, PartialEq, Eq`) and
     `TerminalColor::to_terminal_rgb(self) -> Option<Rgb>`.
2. Tests (in `config/mod.rs`):
   - `to_terminal_rgb`: `TerminalColor::Color(Color { 10, 20, 30 })` resolves to
     `Some(Rgb::new(10, 20, 30))`; `CellForeground` and `CellBackground` both
     resolve to `None`; the variants distinct
     (`CellForeground != CellBackground`, and two `Color(_)` differ) and a
     `Copy`/`Eq` round-trip.
3. Format and test (`cargo fmt`, accept output).

## Verification

```bash
cargo fmt
cargo fmt -- --check
cargo test -p roastty terminal_color
cargo test -p roastty
cargo build -p roastty            # no warnings
rg -n 'ghostty|Ghostty|GHOSTTY' roastty/src/font roastty/src/renderer roastty/src/config && exit 1 || true
rg -n 'ghostty|Ghostty|GHOSTTY' roastty/src/lib.rs roastty/include/roastty.h roastty/tests/abi_harness.c && exit 1 || true
git diff --check
```

The experiment **passes** if:

- `TerminalColor` has the three upstream variants and `to_terminal_rgb` resolves
  `Color` to `Some(rgb)` and both sentinels to `None` via an exhaustive `match`
  — faithful to upstream's union and `toTerminalRGB`;
- the tests pass (the conversion; the sentinels; the distinct variants), and the
  existing tests still pass;
- the parsing, the `Config` struct, and the cell-sentinel resolution stay
  deferred;
- `cargo fmt` accepted, `cargo build -p roastty` has no warnings, and
  `cargo test -p roastty` passes with no regressions;
- the no-`ghostty`-name gates and `git diff --check` pass;
- Codex reviews the design before implementation and the result after, with all
  real findings fixed.

The experiment **fails** if a variant is missing/extra, `to_terminal_rgb`
resolves a sentinel to a color (or the explicit color to `None`), an unrelated
item changes, or any public C API/ABI changes.

## Design Review

Codex reviewed this design before implementation and **approved** it with **no
findings**. It verified against the vendored upstream: the variants match
exactly (`color: Color`, `cell-foreground`, `cell-background`,
`Config.zig:5549`); `to_terminal_rgb()` is an exact port of `toTerminalRGB`
(`Config.zig:5561`, the explicit `Color` maps through `Color::to_terminal_rgb()`
while both cell sentinels return `None`); preserving `None` as the sentinel is
the right boundary (the consumer with access to the active cell resolves
`CellForeground` / `CellBackground`, not this value type); `Copy` / `Eq` is
appropriate because `Color` is copyable; and the planned tests (explicit color
conversion, both sentinel cases, value semantics) are adequate.

Review artifacts:

- Prompt: `logs/codex-review/20260604-110050-d446-prompt.md` (design)
- Result: `logs/codex-review/20260604-110050-d446-last-message.md` (design)
