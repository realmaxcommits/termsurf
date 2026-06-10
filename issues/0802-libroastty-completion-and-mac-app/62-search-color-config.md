+++
[implementer]
agent = "codex"
model = "gpt-5"
reasoning = "high"

[review.design]
agent = "codex"
+++

# Experiment 62: Phase F — search color config

## Description

Experiment 61 landed the split visual config group and intentionally left the
adjacent search highlight colors for their own slice. The next narrow Phase-F
slice ports the four upstream search color config keys:

- `search-foreground`
- `search-background`
- `search-selected-foreground`
- `search-selected-background`

These are non-optional `TerminalColor` values in upstream `Config.zig`, with
concrete defaults and support for `cell-foreground` / `cell-background`
sentinels. Roastty already has renderer-side `SelectionConfig` search color
slots with matching hard-coded defaults; this experiment makes `Config` the
authoritative source and adds the renderer conversion path, while keeping
search-state discovery, match threading, UI actions, and live highlight
population out of scope.

## Changes

- `roastty/src/config/mod.rs`
  - Add upstream defaults:
    - `search-foreground = #000000`
    - `search-background = #ffe082`
    - `search-selected-foreground = #000000`
    - `search-selected-background = #f2a57e`
  - Store all four fields as non-optional `TerminalColor`.
  - Route the keys through `Config::set`, `format_config`, default construction,
    clone/equality, and diagnostics.
  - Preserve upstream declaration/formatter order immediately after
    `split-preserve-zoom` and before the command/shell group:
    `search-foreground`, `search-background`, `search-selected-foreground`,
    `search-selected-background`.
  - Use the existing `TerminalColor` parser/formatter so hex/named colors and
    `cell-foreground` / `cell-background` sentinels behave like upstream.
  - Empty values reset each field to its concrete default; missing values return
    `ValueRequired`; invalid colors return `InvalidValue`.

- `roastty/src/renderer/cell.rs`
  - Add a small conversion from config `TerminalColor` to renderer
    `SelectionColor`.
  - Add `SelectionConfig::from_config(&Config)` that preserves existing optional
    `selection-*` behavior and sources the four non-optional search colors from
    `Config`.
  - Keep `SelectionConfig::default()` equivalent to `Config::default()` so
    existing tests and stubs remain behaviorally unchanged.

- `roastty/src/renderer/frame_renderer.rs`
  - Source the live frame selection/search color config from
    `SelectionConfig::from_config(&Config)` through the config-derived render
    knobs, instead of relying only on `SelectionConfig::default()`.
  - Do not add search match discovery or UI search behavior in this experiment;
    the renderer will still only draw search highlights when later code supplies
    highlight ranges.

Out of scope:

- Search action dispatch, search threads, match navigation, and app callbacks.
- Populating live `highlights` / search match ranges.
- Live A/B search UI recipes.
- Theme loading or broader color palette config.

## Verification

- Run formatting:
  - `cargo fmt -- roastty/src/config/mod.rs roastty/src/renderer/cell.rs roastty/src/renderer/frame_renderer.rs`
  - `prettier --write --prose-wrap always --print-width 80 issues/0802-libroastty-completion-and-mac-app/README.md issues/0802-libroastty-completion-and-mac-app/62-search-color-config.md`
- Run targeted tests:
  - `cargo test -p roastty search_color_config`
  - `cargo test -p roastty selection_config`
  - `cargo test -p roastty config_format_config`
- Add concrete test cases proving:
  - defaults match upstream exactly;
  - formatter order places the four fields after `split-preserve-zoom`;
  - each field accepts hex, named colors, `cell-foreground`, and
    `cell-background`;
  - empty values reset to the correct concrete default;
  - missing values and invalid colors produce the expected diagnostics;
  - `SelectionConfig::from_config` maps explicit colors and sentinels into
    renderer `SelectionColor` values, while preserving optional selection
    colors;
  - config-derived render knobs carry the search selection config into a frame
    rebuild without changing the default frame output when defaults are used.
- Run full Roastty tests:
  - `cargo test -p roastty`
- Run `cargo fmt --check`.
- Run `git diff --check`.
- Run `git status --short` and verify only intended source/docs are present.

**Pass** = the four search color config fields are represented faithfully on
`Config`, round-trip through config loading/formatting, feed renderer
`SelectionConfig`, and have targeted and full tests passing.

**Partial** = the config fields land but renderer selection-config plumbing
exposes a larger ownership prerequisite that should be split out.

**Fail** = the fields cannot be represented faithfully without broader search or
theme infrastructure.

## Design Review

Codex adversarial reviewer `019eb391-7e5a-7cf2-9fc9-1a687e94d642` returned
**Approved** with no Required, Optional, or Nit findings.

The reviewer verified that the README links Exp62 as `Designed`, the experiment
has the required sections and pass/partial/fail criteria, the planned
defaults/order match upstream `Config.zig`, upstream renderer derived config
copies the four search fields from `Config`, existing Roastty renderer defaults
already match those values, and Exp61's result commit exists before Exp62
implementation.
