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

# Experiment 457: the fullscreen config enums (Fullscreen, NonNativeFullscreen)

## Description

This experiment ports the two fullscreen config enums: `Fullscreen` (the
`fullscreen` startup mode) and `NonNativeFullscreen` (the
`macos-non-native-fullscreen` style). Both are consumed by the macOS frontend,
which enters fullscreen imperatively — there is no pure-logic decision to
extract — so this slice ports the enums and their exact variant sets (no
method); the frontend fullscreen handling stays deferred. roastty is macOS-only,
so these are directly relevant. It continues diversifying the config-type family
into the macOS-window config.

## Upstream behavior

In `config/Config.zig`, the two enums and their `Config` fields (both default
`.false`):

```zig
fullscreen: Fullscreen = .false,
@"macos-non-native-fullscreen": NonNativeFullscreen = .false,

/// Valid values for fullscreen config option
/// c_int because it needs to be extern compatible
/// If this is changed, you must also update ghostty.h
pub const Fullscreen = enum(c_int) {
    false,
    true,
    @"non-native",
    @"non-native-visible-menu",
    @"non-native-padded-notch",
};

pub const NonNativeFullscreen = enum(c_int) {
    false,
    true,
    @"visible-menu",
    @"padded-notch",
};
```

`Fullscreen` selects the startup fullscreen mode: `false` (windowed), `true`
(native fullscreen), or one of the three non-native variants (`non-native`,
`non-native-visible-menu`, `non-native-padded-notch`). `NonNativeFullscreen`
selects the non-native fullscreen style independently: `false`, `true`,
`visible-menu`, or `padded-notch`. Both are applied by the macOS frontend.

## Rust mapping (`roastty/src/config/mod.rs`)

```rust
/// The `fullscreen` config (upstream `Fullscreen`): the startup fullscreen mode.
/// The `Config` default is `False`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Fullscreen {
    /// Windowed (not fullscreen).
    False,
    /// Native fullscreen.
    True,
    /// Non-native fullscreen.
    NonNative,
    /// Non-native fullscreen with the menu bar visible.
    NonNativeVisibleMenu,
    /// Non-native fullscreen padded around the notch.
    NonNativePaddedNotch,
}

/// The `macos-non-native-fullscreen` config (upstream `NonNativeFullscreen`): the
/// non-native fullscreen style. The `Config` default is `False`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NonNativeFullscreen {
    /// Disabled.
    False,
    /// Enabled.
    True,
    /// Enabled with the menu bar visible.
    VisibleMenu,
    /// Enabled, padded around the notch.
    PaddedNotch,
}
```

Both are plain enums (the macOS frontend applies them imperatively, ported with
the frontend window code later); the variant sets match upstream exactly. The
hyphenated tags map to `CamelCase` (`non-native-visible-menu` →
`NonNativeVisibleMenu`, `padded-notch` → `PaddedNotch`).

## Scope / faithfulness notes

- **Ported (bridged)**: the `Fullscreen` and `NonNativeFullscreen` config enums
  (`config/Config.zig`).
- **Faithful**: `Fullscreen` has the five upstream variants (`false`, `true`,
  `non-native`, `non-native-visible-menu`, `non-native-padded-notch`);
  `NonNativeFullscreen` has the four (`false`, `true`, `visible-menu`,
  `padded-notch`); the CamelCase names map the tags exactly.
- **Faithful adaptation**: upstream declares both `enum(c_int)` for `ghostty.h`
  extern compatibility; in roastty these are internal (`pub(crate)`, not yet
  crossing roastty's C ABI), so plain Rust enums are the faithful internal
  mapping (a `#[repr(C)]` would be added if/when roastty exposes them across its
  C boundary). The `Config` field defaults (`.false`) are documented on the
  enums but kept off them. No method is extracted — the consumers are imperative
  macOS-frontend fullscreen handling, so they port with the frontend window
  code.
- **Deferred**: the `Config` struct / parsing (and the field defaults), and the
  macOS frontend that enters fullscreen from these enums. (Consumed by a later
  slice; this experiment lands the enums.)
- No C ABI/header/ABI-inventory change (internal Rust).

## Changes

1. `roastty/src/config/mod.rs`:
   - add
     `pub(crate) enum Fullscreen { False, True, NonNative, NonNativeVisibleMenu, NonNativePaddedNotch }`
     and
     `pub(crate) enum NonNativeFullscreen { False, True, VisibleMenu, PaddedNotch }`
     (both derive `Debug, Clone, Copy, PartialEq, Eq`).
2. Tests (in `config/mod.rs`):
   - `Fullscreen`: an array listing **every** variant with `assert_eq!(len, 5)`;
     a representative `assert_ne!` and a `Copy`/`Eq` round-trip.
   - `NonNativeFullscreen`: an array listing **every** variant with
     `assert_eq!(len, 4)`; a representative `assert_ne!` and a `Copy`/`Eq`
     round-trip.
3. Format and test (`cargo fmt`, accept output).

## Verification

```bash
cargo fmt
cargo fmt -- --check
cargo test -p roastty fullscreen
cargo test -p roastty
cargo build -p roastty            # no warnings
rg -n 'ghostty|Ghostty|GHOSTTY' roastty/src/font roastty/src/renderer roastty/src/config && exit 1 || true
rg -n 'ghostty|Ghostty|GHOSTTY' roastty/src/lib.rs roastty/include/roastty.h roastty/tests/abi_harness.c && exit 1 || true
git diff --check
```

The experiment **passes** if:

- `Fullscreen` has exactly the five upstream variants and `NonNativeFullscreen`
  exactly the four — faithful to `config/Config.zig`;
- the tests pass (the exact variant sets), and the existing tests still pass;
- the `Config` struct and the macOS frontend fullscreen handling stay deferred;
- `cargo fmt` accepted, `cargo build -p roastty` has no warnings, and
  `cargo test -p roastty` passes with no regressions;
- the no-`ghostty`-name gates and `git diff --check` pass;
- Codex reviews the design before implementation and the result after, with all
  real findings fixed.

The experiment **fails** if either enum is missing a variant or has an extra/
misnamed one, a default is wrongly encoded onto an enum, an unrelated item
changes, or any public C API/ABI changes.

## Design Review

Codex reviewed this design before implementation and **approved** it with **no
findings**. It verified against the vendored upstream: `Fullscreen`'s variant
set is exact (`false`, `true`, `non-native`, `non-native-visible-menu`,
`non-native-padded-notch`, `Config.zig:5263`); `NonNativeFullscreen`'s is exact
(`false`, `true`, `visible-menu`, `padded-notch`, `Config.zig:5253`); the
CamelCase mappings are faithful; the defaults are correctly documented as
deferred Config-field defaults (both `.false`, `Config.zig:1469` / `:3198`);
plain internal enums are appropriate (`repr(C)` can wait until these cross
roastty's C ABI); no helper method is needed (the consumers are imperative
fullscreen frontend paths); porting the pair together is appropriately bounded;
and the exact-variant tests are adequate.

Review artifacts:

- Prompt: `logs/codex-review/20260604-114518-d457-prompt.md` (design)
- Result: `logs/codex-review/20260604-114518-d457-last-message.md` (design)
