+++
[implementer]
agent = "codex"
model = "gpt-5"
reasoning = "high"

[review.design]
agent = "codex"
+++

# Experiment 84: Phase F — quick terminal space and keyboard config

## Description

Experiment 83 wired `quick-terminal-screen`,
`quick-terminal-animation-duration`, and `quick-terminal-autohide`. The next
unported upstream quick-terminal config fields are:

- `quick-terminal-space-behavior`
- `quick-terminal-keyboard-interactivity`

Upstream declares `quick-terminal-space-behavior` as
`QuickTerminalSpaceBehavior = .move`, with enum tags `remain` and `move`.
Upstream declares `quick-terminal-keyboard-interactivity` as
`QuickTerminalKeyboardInteractivity = .@"on-demand"`, with enum tags `none`,
`on-demand`, and `exclusive`.

This experiment adds the Rust config parser/formatter surface for both fields.
Runtime macOS Spaces behavior, Wayland keyboard interactivity behavior, and app
C ABI accessors are out of scope.

## Changes

- `roastty/src/config/mod.rs`
  - Add `Config::quick_terminal_space_behavior` with upstream default `move`.
  - Add `QuickTerminalSpaceBehavior::{Remain, Move}`.
  - Route `quick-terminal-space-behavior` through `Config::set`, config loading
    diagnostics, clone/equality, and formatting.
  - Add `Config::quick_terminal_keyboard_interactivity` with upstream default
    `on-demand`.
  - Add `QuickTerminalKeyboardInteractivity::{None, OnDemand, Exclusive}`.
  - Route `quick-terminal-keyboard-interactivity` through `Config::set`, config
    loading diagnostics, clone/equality, and formatting.
  - Preserve the current local formatter convention by inserting both keys after
    `quick-terminal-autohide`, matching upstream declaration order.

Out of scope:

- Runtime macOS Spaces behavior.
- Runtime Wayland keyboard interactivity behavior.
- Runtime quick-terminal creation, focus, autohide, or toggle actions.
- C ABI `roastty_config_get` exposure for these fields; Exp 10 documented that
  the app accessor is currently inert and that remains a later
  feature-completion item.
- Shell integration fields that follow these quick-terminal options.
- Any broader formatter reordering of already-ported keys.

## Verification

- Run formatting:
  - `cargo fmt`
  - `prettier --write --prose-wrap always --print-width 80 issues/0802-libroastty-completion-and-mac-app/README.md issues/0802-libroastty-completion-and-mac-app/84-quick-terminal-space-keyboard-config.md`
- Run targeted tests:
  - `cargo test -p roastty quick_terminal_space_keyboard`
  - `cargo test -p roastty config_format_config`
- Add concrete test cases proving:
  - defaults are `QuickTerminalSpaceBehavior::Move` and
    `QuickTerminalKeyboardInteractivity::OnDemand`;
  - default `format_config` emits both keys after `quick-terminal-autohide` and
    before `font-family`;
  - both space-behavior keywords parse and format;
  - all three keyboard-interactivity keywords parse and format;
  - empty values reset to their defaults;
  - unknown keywords are `ConfigSetError::InvalidValue`;
  - missing values are `ConfigSetError::ValueRequired`;
  - `Config::load_str` records diagnostics for invalid neighboring
    quick-terminal lines while preserving valid parsed values;
  - clone/equality preserves both field values.
- Run full Roastty tests:
  - `cargo test -p roastty`
- Run `cargo fmt --check`.
- Run `git diff --check`.
- Run `git status --short` and verify only intended source/docs are present.

**Pass** = both quick-terminal enum fields are represented faithfully on
`Config`, round-trip through config loading/formatting, match upstream defaults
and parser behavior for this slice, and have targeted and full tests passing.

**Partial** = one field lands completely, but the other requires a follow-up.

**Fail** = either key cannot be represented faithfully without first
implementing runtime quick-terminal behavior or C ABI accessors.

## Design Review

Codex adversarial reviewer `019eb4aa-67a9-7ec1-b945-8bb3e7cb40a9` returned
**Approved** with no findings.

The reviewer verified read-only that Experiment 84 is linked from the issue
README as `Designed`, matches upstream defaults and enum values, covers enum
parsing, empty reset, missing/invalid errors, and formatter placement, and has a
focused verification plan. The reviewer also confirmed `git diff --check` passed
for the issue docs.
