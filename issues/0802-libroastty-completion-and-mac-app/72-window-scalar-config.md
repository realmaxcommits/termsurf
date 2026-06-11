+++
[implementer]
agent = "codex"
model = "gpt-5"
reasoning = "high"

[review.design]
agent = "codex"
+++

# Experiment 72: Phase F — window scalar config

## Description

Experiment 71 completed the config-level window padding block. The next upstream
fields in `vendor/ghostty/src/config/Config.zig` are a set of simple
window-scoped scalar fields before the already-ported decoration/theme/subtitle
group:

- `window-vsync`
- `window-inherit-working-directory`
- `tab-inherit-working-directory`
- `split-inherit-working-directory`
- `window-inherit-font-size`
- `window-title-font-family`

The first five are booleans with upstream default `true`.
`window-title-font-family` is an optional string with upstream default `null`.
This experiment wires those fields into the aggregating `Config`: fields,
defaults, parsing/reset behavior, formatting, diagnostics, formatter order, and
focused tests.

Runtime renderer synchronization, working-directory inheritance behavior, font
size inheritance behavior, and native app title font application are out of
scope. This is only the config parser/formatter surface.

## Changes

- `roastty/src/config/mod.rs`
  - Add boolean fields and upstream defaults:
    - `window_vsync = true`
    - `window_inherit_working_directory = true`
    - `tab_inherit_working_directory = true`
    - `split_inherit_working_directory = true`
    - `window_inherit_font_size = true`
  - Add `window_title_font_family: Option<String> = None`.
  - Route all six keys through defaults, `Config::set`, `format_config`,
    diagnostics, clone/equality, and formatter-order tests.
  - Preserve upstream order after the padding block:
    - `window-padding-color`
    - `window-vsync`
    - `window-inherit-working-directory`
    - `tab-inherit-working-directory`
    - `split-inherit-working-directory`
    - `window-inherit-font-size`
    - `window-decoration`
    - `window-title-font-family`
    - `window-subtitle`

Out of scope:

- Runtime renderer vsync behavior.
- Runtime inheritance behavior for windows, tabs, splits, or font size.
- Applying the title font in the macOS app.
- `keybind` and `key-remap`.

## Verification

- Run formatting:
  - `cargo fmt`
  - `prettier --write --prose-wrap always --print-width 80 issues/0802-libroastty-completion-and-mac-app/README.md issues/0802-libroastty-completion-and-mac-app/72-window-scalar-config.md`
- Run targeted tests:
  - `cargo test -p roastty window_scalar_config`
  - `cargo test -p roastty config_format_config`
- Add concrete test cases proving:
  - all five boolean defaults are `true` and format as `true`;
  - each boolean parses `false`, bare CLI `None` as `true`, and empty value as
    default reset;
  - invalid boolean values return `InvalidValue`;
  - `window-title-font-family` defaults/formats as empty, parses a string,
    resets on empty, reports `ValueRequired` on missing value, and reports
    `InvalidValue` for interior NUL;
  - `Config::load_str` records diagnostics for invalid neighboring scalar lines
    while preserving valid values;
  - formatter order matches the upstream sequence around these fields;
  - clone/equality preserves all six values.
- Run full Roastty tests:
  - `cargo test -p roastty`
- Run `cargo fmt --check`.
- Run `git diff --check`.
- Run `git status --short` and verify only intended source/docs are present.

**Pass** = the six scalar window fields are represented faithfully on `Config`,
round-trip through config loading/formatting, match upstream defaults and parser
behavior, and have targeted and full tests passing.

**Partial** = some fields land faithfully but one parser/diagnostic/order edge
requires a follow-up.

**Fail** = these fields cannot be represented faithfully without first porting
runtime window inheritance or title-font application.

## Design Review

Codex adversarial reviewer `019eb40e-9b57-77f3-acbb-571ad103ebf5` returned
**Approved** with no findings.

The reviewer verified the README link/status, required experiment sections,
scope, upstream field defaults/types/order, existing Rust config parser/reset
helpers, optional string behavior, verification criteria, `prettier --check`,
and `git diff --check`.
