+++
[implementer]
agent = "codex"
model = "gpt-5"
reasoning = "high"

[review.design]
agent = "codex"
+++

# Experiment 81: Phase F — quick terminal size config

## Description

Experiment 80 wired `quick-terminal-position`. The next unported upstream config
field is:

- `quick-terminal-size`

Upstream declares this as `QuickTerminalSize = .{}` in
`vendor/ghostty/src/config/Config.zig`. The struct has optional `primary` and
`secondary` sizes. Each size is either a percentage (`50%`) or pixels (`200px`).
The parser accepts one or two comma-separated values, trims whitespace around
each segment, rejects missing units, rejects negative percentages/pixels, and
formats only when `primary` is present.

This experiment adds the Rust config parser/formatter surface and the standalone
sizing calculation helper. Runtime quick-terminal window behavior and the
existing app C ABI accessor gap for `quick-terminal-size` are out of scope.

## Changes

- `roastty/src/config/mod.rs`
  - Add `QuickTerminalSize` with optional `primary` and `secondary` values.
  - Add `QuickTerminalSizeValue::{Percentage(f32), Pixels(u32)}`.
  - Add parse behavior matching upstream:
    - missing input and empty primary are `ValueRequired`;
    - one size sets `primary` only;
    - two sizes set `primary` and `secondary`;
    - three or more sizes are `TooManyArguments`;
    - bare values without `px` or `%` are `MissingUnit`;
    - malformed/negative values are `InvalidValue`.
  - Add formatter behavior matching upstream:
    - default/no-primary writes no config entry;
    - one size formats as `primary`;
    - two sizes format as `primary,secondary` without spaces.
  - Add `calculate(position, dims)` parity with upstream default/fallback
    dimensions for `top`, `bottom`, `left`, `right`, and `center`.
  - Add `Config::quick_terminal_size` with upstream default empty.
  - Route `quick-terminal-size` through `Config::set`, config loading
    diagnostics, clone/equality, and formatting.
  - Preserve the current local formatter convention by inserting the key after
    `quick-terminal-position`.

Out of scope:

- Runtime quick-terminal creation, positioning, restart behavior, and toggle
  actions.
- C ABI `roastty_config_get` exposure for `quick-terminal-size`; Exp 10
  documented that the app accessor is currently inert and that remains a later
  feature-completion item.
- GTK-only quick-terminal layer/namespace fields.
- Any broader formatter reordering of already-ported keys.

## Verification

- Run formatting:
  - `cargo fmt`
  - `prettier --write --prose-wrap always --print-width 80 issues/0802-libroastty-completion-and-mac-app/README.md issues/0802-libroastty-completion-and-mac-app/81-quick-terminal-size-config.md`
- Run targeted tests:
  - `cargo test -p roastty quick_terminal_size`
  - `cargo test -p roastty config_format_config`
- Add concrete test cases proving:
  - default/no-primary omits `quick-terminal-size` from `format_config`;
  - `50%`, `200px`, and `50%,200px` parse and format;
  - whitespace around comma-separated segments is trimmed;
  - empty reset returns to the default empty struct;
  - missing input and empty primary return the internal `ValueRequired` error,
    and map to `ConfigSetError::ValueRequired` where appropriate;
  - `69px,` returns `ValueRequired`;
  - `69px,42%,69px` returns `TooManyArguments`;
  - bare values such as `420` and `bobr` return `MissingUnit`;
  - malformed/negative units such as `bobr%`, `-32%`, and `-69px` return
    `InvalidValue`;
  - `Config::load_str` records diagnostics for invalid neighboring
    `quick-terminal-size` lines while preserving valid parsed values;
  - `calculate` matches upstream default, primary-only, pixel-only, and
    primary+secondary examples for `top`, `left`, and `center`;
  - formatter order includes the key immediately after `quick-terminal-position`
    when the key is present;
  - clone/equality preserves the struct value.
- Run full Roastty tests:
  - `cargo test -p roastty`
- Run `cargo fmt --check`.
- Run `git diff --check`.
- Run `git status --short` and verify only intended source/docs are present.

**Pass** = the quick-terminal-size field and standalone type are represented
faithfully on `Config`, round-trip through config loading/formatting, match
upstream default/parser/formatter/calculation behavior for this slice, and have
targeted and full tests passing.

**Partial** = the parser/formatter field lands, but calculation, diagnostics, or
formatter-order coverage requires a follow-up.

**Fail** = the key cannot be represented faithfully without first implementing
quick-terminal runtime behavior or C ABI accessors.

## Design Review

Codex adversarial reviewer `019eb478-a828-7a91-a3d7-f99edb819366` initially
returned **Changes Required** with one required finding: the design incorrectly
said that the upstream no-primary/default formatter behavior should write an
empty `quick-terminal-size = ` entry. The reviewer pointed to upstream
`QuickTerminalSize.formatEntry`, which returns before formatting when `primary`
is null.

The design was fixed to require upstream-faithful no-output behavior for the
default/no-primary case, and the verification plan now requires proving that
`quick-terminal-size` is omitted from `format_config` when absent.

Codex re-reviewer `019eb479-fce2-7252-ad81-44bca7624393` returned **Approved**
with no findings. The reviewer confirmed the prior finding is resolved, the
verification now checks omission rather than an empty entry, and the behavior
matches `vendor/ghostty/src/config/Config.zig`.
