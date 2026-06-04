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

# Experiment 459: the theme config type and its single-name constructor (Theme, single)

## Description

This experiment ports the `theme` config type — `Theme { light, dark }`, a pair
of theme names (one for light mode, one for dark mode) — **and the single-name
normalization** from its parser. Upstream's `Theme.parseCLI` has two paths: a
light/dark **pair** (when the input contains `,`/`=`/`:`) and a **single** name
(otherwise), in which case both `light` and `dark` are set to that one name.
This experiment lands the value type and captures the single-name path as a
`Theme::single` constructor; the full pair-detection parser, the
appearance-based light/dark selection (the conditional-config system), and the
rest stay deferred.

## Upstream behavior

In `config/Config.zig`:

```zig
pub const Theme = struct {
    light: []const u8,
    dark: []const u8,

    pub fn parseCLI(self: *Theme, alloc: Allocator, input_: ?[]const u8) !void {
        const input = input_ orelse return error.ValueRequired;
        if (input.len == 0) return error.ValueRequired;

        // If there is a comma, equal sign, or colon, parse a light/dark pair.
        // (… `has_colon` handling, Windows drive-letter guard …)
        if (std.mem.indexOf(u8, input, ",") != null or
            std.mem.indexOf(u8, input, "=") != null or has_colon)
        {
            self.* = try cli.args.parseAutoStruct(Theme, alloc, input, null);
            return;
        }

        // Otherwise a single name sets both light and dark.
        const trimmed = std.mem.trim(u8, input, cli.args.whitespace);
        self.* = .{ .light = try alloc.dupeZ(u8, trimmed), .dark = self.light };
    }
    // ... clone, formatEntry
};
```

`Theme` holds a `light` theme name and a `dark` theme name. The parser sets a
light/dark pair when the input is delimited; otherwise a single name is used for
both modes (`light = dark = name`). The appearance-based selection of `light` vs
`dark` happens elsewhere (the conditional-config system).

## Rust mapping (`roastty/src/config/mod.rs`)

```rust
/// The `theme` config (upstream `Theme`): the theme names for light mode and dark
/// mode.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Theme {
    /// The theme name used in light mode.
    pub light: String,
    /// The theme name used in dark mode.
    pub dark: String,
}

impl Theme {
    /// A single theme name used for both light and dark modes (upstream's
    /// `parseCLI` non-pair path: `light = dark = name`).
    pub(crate) fn single(name: String) -> Theme {
        Theme {
            light: name.clone(),
            dark: name,
        }
    }
}
```

`single` sets both `light` and `dark` to the one name — exactly upstream's
single-name path. The `light` / `dark` payloads are owned `String`s (upstream
`[]const u8`), so `Theme` derives `Clone`/`Eq` but not `Copy`.

## Scope / faithfulness notes

- **Ported (bridged)**: the `Theme` config type (`config/Config.zig`) and its
  single-name constructor (`Theme::single`, upstream's `parseCLI` non-pair
  path).
- **Faithful**: `Theme` has the two upstream fields (`light`, `dark`); `single`
  sets both to the one name — exactly the upstream single-name normalization
  (`self.* = .{ .light = name, .dark = self.light }`).
- **Faithful adaptation**: the `light` / `dark` payloads are owned `String`s
  (upstream `[]const u8`), so `Theme` is `Clone`/`Eq` but not `Copy`. Only the
  single-name normalization is extracted from `parseCLI`; the delimiter-based
  pair detection (`,`/`=`/`:`, the Windows drive-letter guard) and the trimming
  are the deferred parser.
- **Deferred**: the full `parseCLI` (the pair detection and trimming), the
  `formatEntry`, the `Config` struct that holds the `theme` key, and the
  appearance-based light/dark selection (the conditional-config system that
  picks `light` vs `dark` from the system scheme). (Consumed by a later slice;
  this experiment lands the value type and the single-name normalization.)
- No C ABI/header/ABI-inventory change (internal Rust).

## Changes

1. `roastty/src/config/mod.rs`:
   - add `pub(crate) struct Theme { pub light: String, pub dark: String }`
     (derive `Debug, Clone, PartialEq, Eq` — not `Copy`) and
     `Theme::single(name: String) -> Theme` (both fields = `name`).
2. Tests (in `config/mod.rs`):
   - `single`: `Theme::single("foo".to_string())` has `light == "foo"` and
     `dark == "foo"`; a pair `Theme { light: "a", dark: "b" }` has
     `light != dark` and differs from `single("a")`; a `Clone`/`Eq` round-trip.
3. Format and test (`cargo fmt`, accept output).

## Verification

```bash
cargo fmt
cargo fmt -- --check
cargo test -p roastty config_theme
cargo test -p roastty
cargo build -p roastty            # no warnings
rg -n 'ghostty|Ghostty|GHOSTTY' roastty/src/font roastty/src/renderer roastty/src/config && exit 1 || true
rg -n 'ghostty|Ghostty|GHOSTTY' roastty/src/lib.rs roastty/include/roastty.h roastty/tests/abi_harness.c && exit 1 || true
git diff --check
```

The experiment **passes** if:

- `Theme` has the two upstream fields and `single` sets both `light` and `dark`
  to the one name — faithful to upstream's type and the `parseCLI` single-name
  path;
- the tests pass (the single-name normalization; the pair; the `Clone`/`Eq`),
  and the existing tests still pass;
- the full parser, the `Config` struct, and the appearance-based selection stay
  deferred;
- `cargo fmt` accepted, `cargo build -p roastty` has no warnings, and
  `cargo test -p roastty` passes with no regressions;
- the no-`ghostty`-name gates and `git diff --check` pass;
- Codex reviews the design before implementation and the result after, with all
  real findings fixed.

The experiment **fails** if `single` does not set both fields to the name, the
struct is shaped wrong, an unrelated item changes, or any public C API/ABI
changes.

## Design Review

Codex reviewed this design before implementation and **approved** it with **no
findings**. It verified against the vendored upstream: `Theme { light, dark }`
matches the two-string struct (`Config.zig:9848`); `Theme::single(name)`
correctly extracts the single-name parse path (upstream trims the input, then
sets both `light` and `dark` to the same name, with upstream tests asserting
both become the name, `Config.zig:9880` / `:9922`); owning `String` is the right
Rust mapping for upstream `[]const u8` in an internal config value; deferring
the delimiter-based pair parsing, trimming, formatting, `Config` wiring, and
appearance selection is the right boundary; and the tests (the single-name
normalization, distinct light/dark pairs, value semantics) are adequate.

Review artifacts:

- Prompt: `logs/codex-review/20260604-115403-d459-prompt.md` (design)
- Result: `logs/codex-review/20260604-115403-d459-last-message.md` (design)
