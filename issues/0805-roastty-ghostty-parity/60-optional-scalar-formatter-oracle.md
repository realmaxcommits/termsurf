# Experiment 60: Optional scalar formatter oracle

## Description

Experiment 59 promoted the single `keybind` formatter row and left 123 formatter
rows as `Audit covered`. The remaining rows are broad families: `font`,
`optional value`, and `custom format_entry`.

The next coherent slice is the optional scalar formatter path: optional values
that format through `entry_optional` and then a primitive scalar helper
(`entry_bool`, `entry_int`, or `entry_str`). These rows share the pinned Ghostty
formatter behavior for optionals: `null` formats as a void line (`name = `), and
present values recurse into the inner scalar formatter using the same key name.

This experiment will split those rows out of the broad `optional value` family,
add a focused oracle for them, and promote only that new optional scalar family.

The target rows are:

- `cursor-style-blink`;
- `class`;
- `language`;
- `macos-custom-icon`;
- `macos-option-as-alt`;
- `title`;
- `x11-instance-name`;
- `linux-cgroup-memory-limit`;
- `linux-cgroup-processes-limit`;
- `window-position-x`;
- `window-position-y`.

CFG-218 should remain `Gap` because 112 formatter rows will still lack
non-default formatter oracles.

## Changes

- `roastty/src/config/mod.rs`
  - Add a focused `optional_scalar_config_formatter_family_oracle` test.
  - Cover optional `None` void output, `Some` bool output, signed and unsigned
    integer output, byte-preserving string output, `macos-option-as-alt` keyword
    string output, raw-empty reset back to defaults, and representative order
    checks across the optional scalar rows.
- `issues/0805-roastty-ghostty-parity/config_formatter_inventory.py`
  - Classify `entry_optional` plus `entry_bool`, `entry_int`, or `entry_str` as
    `optional scalar` before the generic `optional value` family.
  - Detect `optional_scalar_config_formatter_family_oracle`.
  - Promote only formatter rows whose family is `optional scalar`.
  - Keep Experiment 60 as the CFG-218 owner when this oracle is present.
- `issues/0805-roastty-ghostty-parity/config-formatter-inventory.md`
  - Regenerate the formatter inventory.
  - Expected counts after implementation: 91 `Oracle complete` rows and 112
    `Audit covered` rows.
- `issues/0805-roastty-ghostty-parity/config-matrix.md`
  - Regenerate CFG-218. It should remain `Gap` and report the new promotion
    counts.

## Verification

Pass criteria:

- `cargo test --manifest-path roastty/Cargo.toml optional_scalar_config_formatter_family_oracle`
  passes.
- `cargo test --manifest-path roastty/Cargo.toml cursor_style_blink_config_parser_family_oracle`
  still passes.
- `cargo test --manifest-path roastty/Cargo.toml string_config_parser_family_oracle`
  still passes.
- `cargo test --manifest-path roastty/Cargo.toml integer_config_parser_family_oracle`
  still passes.
- `cargo test --manifest-path roastty/Cargo.toml config_default_format_oracle`
  still passes.
- `PYTHONDONTWRITEBYTECODE=1 python3 issues/0805-roastty-ghostty-parity/config_formatter_inventory.py --upstream vendor/ghostty/src/config/Config.zig --upstream-formatter-file vendor/ghostty/src/config/formatter_file.zig --upstream-formatter vendor/ghostty/src/config/formatter.zig --roastty roastty/src/config/mod.rs --config-inventory issues/0805-roastty-ghostty-parity/config-inventory.md --output issues/0805-roastty-ghostty-parity/config-formatter-inventory.md --matrix issues/0805-roastty-ghostty-parity/config-matrix.md`
  reports:
  - `ghostty_canonical=203`;
  - `roastty_formatter_rows=203`;
  - `missing_canonical_formatter_rows=0`;
  - `extra_formatter_rows=0`;
  - `oracle_complete=91`;
  - `audit_covered=112`;
  - `gap=0`.
- A matrix assertion confirms:
  - CFG-217 remains `Pass`;
  - CFG-218 remains `Gap`;
  - all previously promoted formatter families remain `Oracle complete`;
  - the 11 optional scalar formatter rows are `Oracle complete`;
  - optional custom `format_entry` rows and font rows are not promoted by this
    oracle.
- `cargo fmt --manifest-path roastty/Cargo.toml --check` passes.
- `prettier --write --prose-wrap always --print-width 80` is run on changed
  Markdown files.
- `git diff --check` passes.

## Design Review

Reviewed by a fresh-context Codex adversarial subagent.

Verdict: **Approved**.

Findings: none.

The reviewer verified the README link, required sections, 11-row target set,
expected 91/112/0 count movement, exclusion of optional custom and font rows,
and verification criteria.
