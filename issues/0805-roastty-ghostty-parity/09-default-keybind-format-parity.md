# Experiment 9: Default Keybind Format Parity

## Description

Experiment 8 proved default config formatting parity for every comparable line
except `keybind` and `command-palette-entry`. The largest remaining default
config gap is `keybind`: pinned Ghostty emits 93 normalized default keybind
lines, while Roastty emits 86, with 135 normalized multiset mismatches.

This experiment should make Roastty's default keybind formatter and default
keybind set match the pinned Ghostty macOS default output for
`+show-config --default --no-pager`.

The scope is default keybind config formatting only:

- default keybind entries created by `Keybinds::default()`;
- keybind line order as emitted by `format_config`;
- key trigger text formatting, including physical-key casing, modifier order,
  and aliases such as `copy`, `paste`, `arrow_left`, and `digit_1`;
- keybind flag formatting when the pinned Ghostty output does or does not show
  `performable:`;
- action formatting for default keybinds, including
  `write_screen_file:{mode},plain`.

The scope is not general keybind parser parity, user-defined keybind round-trip
parity, menu shortcut lookup, runtime shortcut execution, GUI keyboard delivery,
or non-default platform keybind behavior. If the implementation has to touch a
shared keybind formatter, add focused tests proving the default macOS output and
record any remaining parser/runtime risks as gaps rather than claiming them.

## Changes

- `roastty/src/config/keybind.rs`
  - Audit Roastty default keybind storage and formatting against
    `vendor/ghostty/src/config/Config.zig` `Keybinds.init` and
    `Keybinds.formatEntryDocs`.
  - Fix default keybind content and/or formatter syntax so the normalized
    default `keybind = ...` lines match the pinned Ghostty fixture exactly.
  - Preserve existing parser behavior unless a mismatch is directly required to
    make default formatter parity true.
- `roastty/src/config/mod.rs`
  - Tighten `config_default_format_oracle` so `keybind` is no longer excluded as
    a gap once parity is achieved.
  - Keep `command-palette-entry` excluded and explicitly asserted as the only
    remaining repeatable default-format gap unless this experiment also proves a
    safe incidental fix there.
- `roastty/testdata/issue805-ghostty-default-config.txt`
  - Keep using the pinned Ghostty fixture from Experiment 8; do not regenerate
    it unless the pinned Ghostty executable output is being revalidated and the
    command is recorded.
- `issues/0805-roastty-ghostty-parity/default-config-oracle.md`
  - Update counts and gap notes so `keybind` moves from gap to proven default
    formatting parity.
- `issues/0805-roastty-ghostty-parity/config-matrix.md`
  - Mark `CFG-214` as `Pass` only after the oracle proves exact default keybind
    parity.
  - Update `CFG-213` notes if the comparable surface expands to include
    keybinds.
  - Leave parser, diagnostic, precedence, reload, and runtime-effect rows as
    `Gap` unless separately proven.
- `issues/0805-roastty-ghostty-parity/README.md`
  - Add a learning if the experiment identifies a reusable rule for Ghostty
    keybind formatting.

## Verification

Pass/fail criteria:

- `cargo test --manifest-path roastty/Cargo.toml config_default_format_oracle`
  passes with `keybind` included in the exact default-format comparison.
- The normalized default `keybind` line counts match pinned Ghostty exactly: 93
  Ghostty lines and 93 Roastty lines.
- The normalized default `keybind` multiset mismatch count is 0.
- The ordered normalized default `keybind` lines match the pinned Ghostty
  fixture exactly.
- The remaining full default-config diff is limited to the already-recorded
  `command-palette-entry` escaped-text gap, unless that gap is also fixed and
  documented.
- Matrix updates do not mark general parser, runtime shortcut execution, menu
  shortcut lookup, or GUI key handling as passing from formatter-only evidence.
- Rust formatting, markdown formatting, focused tests, and diff hygiene pass.

Suggested commands:

```bash
ROASTTY_DEFAULT_CONFIG_OUT=/Users/astrohacker/dev/termsurf/logs/issue805-exp9-roastty-default-config.txt \
  cargo test --manifest-path roastty/Cargo.toml config_default_format_oracle -- --nocapture
cargo test --manifest-path roastty/Cargo.toml config_format_config_emits_fields_in_upstream_order -- --nocapture
cargo fmt --manifest-path roastty/Cargo.toml --check
prettier --write --prose-wrap always --print-width 80 \
  issues/0805-roastty-ghostty-parity/09-default-keybind-format-parity.md \
  issues/0805-roastty-ghostty-parity/README.md \
  issues/0805-roastty-ghostty-parity/default-config-oracle.md \
  issues/0805-roastty-ghostty-parity/config-matrix.md
git diff --check
```

## Design Review

Fresh-context adversarial design review approved the design with no findings.

Reviewer verdict:

```text
VERDICT: APPROVED

Findings: None.
```
