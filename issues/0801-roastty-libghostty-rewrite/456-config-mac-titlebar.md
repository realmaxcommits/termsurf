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

# Experiment 456: the macOS titlebar config enums (MacTitlebarStyle, MacTitlebarProxyIcon)

## Description

This experiment ports the two macOS titlebar config enums: `MacTitlebarStyle`
(the titlebar appearance) and `MacTitlebarProxyIcon` (whether the document proxy
icon is shown). Both are consumed by the macOS (Swift) frontend, which styles
the window's titlebar imperatively — there is no pure-logic decision to extract
— so this slice ports the enums and their exact variant sets (no method); the
frontend titlebar styling stays deferred. roastty is macOS-only, so these are
directly relevant. It continues diversifying the config-type family into the
macOS-window config.

## Upstream behavior

In `config/Config.zig`, the two enums and their `Config` fields:

```zig
@"macos-titlebar-style": MacTitlebarStyle = .transparent,
@"macos-titlebar-proxy-icon": MacTitlebarProxyIcon = .visible,

pub const MacTitlebarStyle = enum {
    native,
    transparent,
    tabs,
    hidden,
};

pub const MacTitlebarProxyIcon = enum {
    visible,
    hidden,
};
```

`MacTitlebarStyle` selects the titlebar appearance: `native` (the standard macOS
titlebar), `transparent` (a translucent titlebar, the default), `tabs` (a
tab-integrated titlebar), or `hidden` (no titlebar). `MacTitlebarProxyIcon`
toggles the document proxy icon: `visible` (the default) or `hidden`. Both are
applied by the macOS frontend when it builds the window.

## Rust mapping (`roastty/src/config/mod.rs`)

```rust
/// The `macos-titlebar-style` config (upstream `MacTitlebarStyle`): the macOS
/// titlebar appearance. The `Config` default is `Transparent`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MacTitlebarStyle {
    /// The standard macOS titlebar.
    Native,
    /// A translucent titlebar.
    Transparent,
    /// A tab-integrated titlebar.
    Tabs,
    /// No titlebar.
    Hidden,
}

/// The `macos-titlebar-proxy-icon` config (upstream `MacTitlebarProxyIcon`):
/// whether the document proxy icon is shown. The `Config` default is `Visible`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MacTitlebarProxyIcon {
    /// Show the document proxy icon.
    Visible,
    /// Hide the document proxy icon.
    Hidden,
}
```

Both are plain enums (the frontend applies them imperatively, ported with the
macOS window code later); the variant sets match upstream exactly.

## Scope / faithfulness notes

- **Ported (bridged)**: the `MacTitlebarStyle` and `MacTitlebarProxyIcon` config
  enums (`config/Config.zig`).
- **Faithful**: `MacTitlebarStyle` has the four upstream variants (`native`,
  `transparent`, `tabs`, `hidden`); `MacTitlebarProxyIcon` has the two
  (`visible`, `hidden`); the names map exactly.
- **Faithful adaptation**: the `Config` field defaults (`.transparent` /
  `.visible`) are documented on the enums but kept off them (the other config
  types keep defaults on the deferred `Config` struct). No method is extracted —
  the consumers are imperative macOS-frontend titlebar styling, not pure
  functions, so they port with the frontend window code.
- **Deferred**: the `Config` struct / parsing (and the field defaults), and the
  macOS (Swift) frontend that builds the window titlebar from these enums.
  (Consumed by a later slice; this experiment lands the enums.)
- No C ABI/header/ABI-inventory change (internal Rust).

## Changes

1. `roastty/src/config/mod.rs`:
   - add
     `pub(crate) enum MacTitlebarStyle { Native, Transparent, Tabs, Hidden }`
     and `pub(crate) enum MacTitlebarProxyIcon { Visible, Hidden }` (both derive
     `Debug, Clone, Copy, PartialEq, Eq`).
2. Tests (in `config/mod.rs`):
   - `MacTitlebarStyle`: an array listing **every** variant with
     `assert_eq!(len, 4)`; a representative `assert_ne!` and a `Copy`/`Eq`
     round-trip.
   - `MacTitlebarProxyIcon`: an array listing **every** variant with
     `assert_eq!(len, 2)`; `assert_ne!(Visible, Hidden)` and a `Copy`/`Eq`
     round-trip.
3. Format and test (`cargo fmt`, accept output).

## Verification

```bash
cargo fmt
cargo fmt -- --check
cargo test -p roastty mac_titlebar
cargo test -p roastty
cargo build -p roastty            # no warnings
rg -n 'ghostty|Ghostty|GHOSTTY' roastty/src/font roastty/src/renderer roastty/src/config && exit 1 || true
rg -n 'ghostty|Ghostty|GHOSTTY' roastty/src/lib.rs roastty/include/roastty.h roastty/tests/abi_harness.c && exit 1 || true
git diff --check
```

The experiment **passes** if:

- `MacTitlebarStyle` has exactly the four upstream variants and
  `MacTitlebarProxyIcon` exactly the two — faithful to `config/Config.zig`;
- the tests pass (the exact variant sets), and the existing tests still pass;
- the `Config` struct and the macOS frontend styling stay deferred;
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
findings**. It verified against the vendored upstream:
`MacTitlebarStyle { Native, Transparent, Tabs, Hidden }` matches
`native/transparent/tabs/hidden` (`Config.zig:8994`);
`MacTitlebarProxyIcon { Visible, Hidden }` matches `visible/hidden`
(`Config.zig:9002`); the defaults are correctly documented as deferred
Config-field defaults (`.transparent` / `.visible`, `Config.zig:3261` /
`:3282`); plain enums are the right shape (the consumers are macOS frontend
styling paths, not pure config logic); porting the pair together is
appropriately bounded (adjacent macOS titlebar config leaves); and the
exact-variant tests are adequate.

Review artifacts:

- Prompt: `logs/codex-review/20260604-114120-d456-prompt.md` (design)
- Result: `logs/codex-review/20260604-114120-d456-last-message.md` (design)

## Result

**Result:** Pass

The macOS titlebar config enums are now live.

- `roastty/src/config/mod.rs`:
  `pub(crate) enum MacTitlebarStyle { Native, Transparent, Tabs, Hidden }`
  (upstream `MacTitlebarStyle`) and
  `pub(crate) enum MacTitlebarProxyIcon { Visible, Hidden }` (upstream
  `MacTitlebarProxyIcon`), both deriving `Debug, Clone, Copy, PartialEq, Eq`.
  Plain enums (the consumers are imperative macOS-frontend titlebar styling,
  ported with the frontend window code later); the `Config` field defaults
  (`.transparent` / `.visible`) documented but kept off the enums.

Tests (in `config/mod.rs`):

- `mac_titlebar_style_has_the_four_upstream_variants` — an array of all four
  variants, `assert_eq!(len, 4)`, `assert_ne!(Native, Hidden)`, `Copy`/`Eq`.
- `mac_titlebar_proxy_icon_has_the_two_upstream_variants` — an array of both
  variants, `assert_eq!(len, 2)`, `assert_ne!(Visible, Hidden)`, `Copy`/`Eq`.

Gate results:

- `cargo fmt -p roastty` accepted; `--check` clean.
- `cargo test -p roastty` → 2945 passed, 0 failed (+2, no regressions).
- `cargo build -p roastty` → no warnings.
- No-`ghostty`-name gates (font + renderer + config +
  `lib.rs`/header/`abi_harness.c`) clean; `git diff --check` clean.

## Conclusion

The config layer now carries the macOS titlebar config enums `MacTitlebarStyle`
and `MacTitlebarProxyIcon` — the first config slice to reach the macOS-window
frontend config (directly relevant since roastty is macOS-only). These are
dispatch enums (no extracted method — the consumers are imperative
macOS-frontend titlebar styling), so they land as plain enums with
exact-variant-set tests, like the background-image placement pair
(Experiment 436) and the click-action pair (Experiment 443). The `Config` struct
/ parsing and the macOS (Swift) frontend that builds the titlebar stay deferred.
The config-type family — now eighteen enums/flag-structs with consumers plus
three color value types — remains a clean, gated way to advance the rewrite
while the larger coupled subsystems stay deferred.

## Completion Review

Codex reviewed the completed implementation and result and **approved** with
**no findings**. It confirmed `MacTitlebarStyle` and `MacTitlebarProxyIcon`
carry the exact upstream variant sets; keeping them as plain enums is
appropriate (behavior belongs in the later macOS frontend titlebar integration);
the defaults are documented but correctly left off the enums; and the tests
reference every variant. No public C ABI/header impact; nothing needed to change
before the result commit.

Review artifacts:

- Prompt: `logs/codex-review/20260604-114307-r456-prompt.md` (result)
- Result: `logs/codex-review/20260604-114307-r456-last-message.md` (result)
