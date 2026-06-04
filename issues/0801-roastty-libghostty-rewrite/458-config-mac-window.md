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

# Experiment 458: the macOS window config enums (MacWindowButtons, MacHidden)

## Description

This experiment ports two more macOS window config enums: `MacWindowButtons`
(whether the window traffic-light buttons are shown) and `MacHidden` (the
`macos-hidden` dock/app-hiding behavior). Both are consumed by the macOS
frontend imperatively — there is no pure-logic decision to extract — so this
slice ports the enums and their exact variant sets (no method); the frontend
handling stays deferred. roastty is macOS-only, so these are directly relevant.
It continues the macOS-window config family (Experiments 456–457).

## Upstream behavior

In `config/Config.zig`, the two enums and their `Config` fields:

```zig
@"macos-window-buttons": MacWindowButtons = .visible,
@"macos-hidden": MacHidden = .never,

pub const MacWindowButtons = enum {
    visible,
    hidden,
};

pub const MacHidden = enum {
    never,
    always,
};
```

`MacWindowButtons` toggles the window's traffic-light buttons: `visible` (the
default) or `hidden`. `MacHidden` selects the `macos-hidden` behavior (whether
the app starts hidden): `never` (the default) or `always`. Both are applied by
the macOS frontend.

## Rust mapping (`roastty/src/config/mod.rs`)

```rust
/// The `macos-window-buttons` config (upstream `MacWindowButtons`): whether the
/// window's traffic-light buttons are shown. The `Config` default is `Visible`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MacWindowButtons {
    /// Show the window buttons.
    Visible,
    /// Hide the window buttons.
    Hidden,
}

/// The `macos-hidden` config (upstream `MacHidden`): whether the app starts
/// hidden. The `Config` default is `Never`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MacHidden {
    /// Never start hidden.
    Never,
    /// Always start hidden.
    Always,
}
```

Both are plain enums (the macOS frontend applies them imperatively, ported with
the frontend window code later); the variant sets match upstream exactly.

## Scope / faithfulness notes

- **Ported (bridged)**: the `MacWindowButtons` and `MacHidden` config enums
  (`config/Config.zig`).
- **Faithful**: `MacWindowButtons` has the two upstream variants (`visible`,
  `hidden`); `MacHidden` has the two (`never`, `always`); the names map exactly.
- **Faithful adaptation**: the `Config` field defaults (`.visible` / `.never`)
  are documented on the enums but kept off them (the other config types keep
  defaults on the deferred `Config` struct). No method is extracted — the
  consumers are imperative macOS-frontend window handling, so they port with the
  frontend code.
- **Deferred**: the `Config` struct / parsing (and the field defaults), and the
  macOS frontend that applies these enums. (Consumed by a later slice; this
  experiment lands the enums.)
- No C ABI/header/ABI-inventory change (internal Rust).

## Changes

1. `roastty/src/config/mod.rs`:
   - add `pub(crate) enum MacWindowButtons { Visible, Hidden }` and
     `pub(crate) enum MacHidden { Never, Always }` (both derive
     `Debug, Clone, Copy, PartialEq, Eq`).
2. Tests (in `config/mod.rs`):
   - `MacWindowButtons`: an array listing **every** variant with
     `assert_eq!(len, 2)`; `assert_ne!(Visible, Hidden)` and a `Copy`/`Eq`
     round-trip.
   - `MacHidden`: an array listing **every** variant with `assert_eq!(len, 2)`;
     `assert_ne!(Never, Always)` and a `Copy`/`Eq` round-trip.
3. Format and test (`cargo fmt`, accept output).

## Verification

```bash
cargo fmt
cargo fmt -- --check
cargo test -p roastty mac_window
cargo test -p roastty mac_hidden
cargo test -p roastty
cargo build -p roastty            # no warnings
rg -n 'ghostty|Ghostty|GHOSTTY' roastty/src/font roastty/src/renderer roastty/src/config && exit 1 || true
rg -n 'ghostty|Ghostty|GHOSTTY' roastty/src/lib.rs roastty/include/roastty.h roastty/tests/abi_harness.c && exit 1 || true
git diff --check
```

The experiment **passes** if:

- `MacWindowButtons` has exactly the two upstream variants and `MacHidden`
  exactly the two — faithful to `config/Config.zig`;
- the tests pass (the exact variant sets), and the existing tests still pass;
- the `Config` struct and the macOS frontend handling stay deferred;
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
`MacWindowButtons { Visible, Hidden }` matches `visible/hidden`
(`Config.zig:8988`); `MacHidden { Never, Always }` matches `never/always`
(`Config.zig:9008`); the defaults are correctly documented as deferred
Config-field defaults (`.visible` / `.never`, `Config.zig:3219` / `:3358`);
plain enums are the right shape (behavior belongs in the later macOS frontend
integration); porting the pair together is appropriately bounded (adjacent macOS
window config leaves); and the exact-variant tests are adequate.

Review artifacts:

- Prompt: `logs/codex-review/20260604-114911-d458-prompt.md` (design)
- Result: `logs/codex-review/20260604-114911-d458-last-message.md` (design)
