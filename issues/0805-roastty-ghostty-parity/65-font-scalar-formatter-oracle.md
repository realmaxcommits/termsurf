# Experiment 65: Font scalar formatter oracle

## Description

Experiment 64 promoted the six optional value formatter rows and left 92
formatter rows as `Audit covered`. The remaining `font` family is still mixed:
some rows are simple scalar/optional-scalar/metric formatter paths, while others
are font-family repeatables, feature maps, variation maps, style unions, or
codepoint maps.

This experiment isolates the scalar-shaped font rows so they can be proven
without claiming the more complex font-specific formatters.

The target rows are:

- `adjust-font-baseline`;
- `font-size`;
- `font-thicken`;
- `font-thicken-strength`;
- `window-inherit-font-size`;
- `window-title-font-family`.

CFG-218 should remain `Gap` because 86 formatter rows will still lack
non-default formatter oracles.

## Changes

- `roastty/src/config/mod.rs`
  - Add a focused `font_scalar_config_formatter_family_oracle` test.
  - Cover optional metric modifier void, absolute value, percent value, and
    raw-empty reset output for `adjust-font-baseline`.
  - Cover `font-size` float output.
  - Cover `font-thicken` boolean output.
  - Cover `font-thicken-strength` integer output.
  - Cover `window-inherit-font-size` boolean output.
  - Cover `window-title-font-family` optional void, string output, raw-empty
    reset, and byte-preserving string behavior.
  - Cover representative row order across the target rows.
- `issues/0805-roastty-ghostty-parity/config_formatter_inventory.py`
  - Classify exactly the six target rows as `font scalar`.
  - Detect `font_scalar_config_formatter_family_oracle`.
  - Promote only formatter rows whose family is `font scalar`.
  - Keep Experiment 65 as the CFG-218 owner when this oracle is present.
- `issues/0805-roastty-ghostty-parity/config-formatter-inventory.md`
  - Regenerate the formatter inventory.
  - Expected counts after implementation: 117 `Oracle complete` rows and 86
    `Audit covered` rows.
- `issues/0805-roastty-ghostty-parity/config-matrix.md`
  - Regenerate CFG-218. It should remain `Gap` and report the new promotion
    counts.

## Verification

Pass criteria:

- `cargo test --manifest-path roastty/Cargo.toml font_scalar_config_formatter_family_oracle`
  passes.
- Existing representative parser/formatter tests for the covered value shapes
  still pass:
  - `cargo test --manifest-path roastty/Cargo.toml metric_modifier_config_formatter_family_oracle`
  - `cargo test --manifest-path roastty/Cargo.toml config_font_thicken_parses_and_round_trips`
  - `cargo test --manifest-path roastty/Cargo.toml config_font_synthetic_style_and_size_parse_and_format`
  - `cargo test --manifest-path roastty/Cargo.toml window_scalar_config_parse_format_reset_and_diagnose`
- `cargo test --manifest-path roastty/Cargo.toml config_default_format_oracle`
  still passes.
- `PYTHONDONTWRITEBYTECODE=1 python3 issues/0805-roastty-ghostty-parity/config_formatter_inventory.py --upstream vendor/ghostty/src/config/Config.zig --upstream-formatter-file vendor/ghostty/src/config/formatter_file.zig --upstream-formatter vendor/ghostty/src/config/formatter.zig --roastty roastty/src/config/mod.rs --config-inventory issues/0805-roastty-ghostty-parity/config-inventory.md --output issues/0805-roastty-ghostty-parity/config-formatter-inventory.md --matrix issues/0805-roastty-ghostty-parity/config-matrix.md`
  reports:
  - `ghostty_canonical=203`;
  - `roastty_formatter_rows=203`;
  - `missing_canonical_formatter_rows=0`;
  - `extra_formatter_rows=0`;
  - `oracle_complete=117`;
  - `audit_covered=86`;
  - `gap=0`.
- Run this matrix assertion:

  ```bash
  python3 - <<'PY'
  from pathlib import Path

  matrix = Path('issues/0805-roastty-ghostty-parity/config-matrix.md').read_text()
  rows = Path('issues/0805-roastty-ghostty-parity/config-formatter-inventory.md').read_text().splitlines()

  def row_for(option: str) -> str:
      for line in rows:
          if not line.startswith('| FORMAT-'):
              continue
          cells = [cell.strip() for cell in line.strip('|').split('|')]
          if len(cells) > 1 and cells[1] == f'`{option}`':
              return line
      raise AssertionError(f'missing row for {option}')

  cfg218 = matrix.split('| CFG-218 |', 1)[1].split('\n', 1)[0]
  assert '| Gap    |' in cfg218 or '| Gap |' in cfg218, cfg218
  assert 'Experiment 65 inventories formatter coverage: 117 rows Oracle complete; 86 rows are not Oracle complete and 0 rows are formatter gaps.' in cfg218, cfg218

  for option in [
      'adjust-font-baseline',
      'font-size',
      'font-thicken',
      'font-thicken-strength',
      'window-inherit-font-size',
      'window-title-font-family',
  ]:
      row = row_for(option)
      assert 'font scalar' in row and 'Oracle complete' in row, row

  for option in [
      'font-family',
      'font-feature',
      'font-variation',
      'font-codepoint-map',
      'font-style',
      'font-synthetic-style',
  ]:
      row = row_for(option)
      assert 'font' in row and 'Audit covered' in row, row

  for option in ['cursor-style', 'window-theme', 'env', 'input']:
      row = row_for(option)
      assert 'custom format_entry' in row and 'Audit covered' in row, row

  print('matrix assertions passed')
  PY
  ```

- `cargo fmt --manifest-path roastty/Cargo.toml --check` passes.
- `prettier --write --prose-wrap always --print-width 80` is run on changed
  Markdown files.
- `git diff --check` passes.

## Design Review

Reviewed by a fresh-context Codex adversarial subagent.

Verdict: **Approved**.

Findings: none.

The reviewer verified that the README links Experiment 65 as `Designed`, the
experiment has the required sections, the scope is exactly the six current font
scalar rows, complex font formatter families remain unpromoted, and the
verification criteria cover counts, CFG-218 status, target promotion, adjacent
unpromoted families, existing/new test filters, and hygiene checks.
