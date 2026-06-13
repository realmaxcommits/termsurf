# Experiment 13: Non-Default Parser Facet Audit

## Description

Experiment 12 made the remaining config work explicit. The first open facet is
CFG-217, non-default parser semantics: Roastty must prove that non-default
values for pinned Ghostty's canonical config options parse, reject, and reset in
equivalent ways.

This experiment will build the audit surface before trying to close the facet.
The goal is to map every canonical Ghostty option to the parser path Roastty
uses, classify the parser family, attach existing test evidence, and identify
the smallest remaining parser gaps. The result should prevent accidental
overclaiming: this experiment keeps CFG-217 as `Gap` unless it can cite
upstream-derived parser-family or option-specific oracles that cover all
documented accepted variants/classes plus rejection and reset semantics. The
expected outcome is an audit map for the next parser experiments, not a broad
sample-based pass.

## Changes

- `issues/0805-roastty-ghostty-parity/config_parser_inventory.py`
  - Add a bounded source scanner for Roastty's `Config::set_from_source` match
    arms and Ghostty's canonical option list.
  - Emit a parser-facet inventory with one row per canonical option.
  - Classify each option by parser family where possible, such as boolean,
    integer scalar, float scalar, optional scalar, enum, string, path,
    repeatable path/list, keybind, command palette, color, duration, font,
    window padding, working directory, and custom parser.
  - Mark rows as `Audit covered` only when this experiment identifies the parser
    family and existing evidence. Mark rows as `Oracle complete` only if the
    evidence is upstream-derived and covers all documented accepted
    variants/classes plus rejection and reset semantics for that parser family
    or option.
- `issues/0805-roastty-ghostty-parity/config-parser-inventory.md`
  - Record the generated parser facet rows, counts by parser family, covered
    rows, and gap rows.
- `issues/0805-roastty-ghostty-parity/config-matrix.md`
  - Update CFG-217 with the parser audit evidence.
  - Keep CFG-217 as `Gap` unless every parser inventory row is
    `Oracle complete`. If any row is only `Audit covered` or `Gap`, point
    CFG-217 to the exact remaining parser rows.
- `issues/0805-roastty-ghostty-parity/README.md`
  - Add a learning only if the audit discovers a reusable parser-proof rule or a
    concrete parser mismatch.

## Verification

Pass criteria:

- The parser inventory generator exits successfully and reports:
  - `ghostty_canonical=203`;
  - `roastty_parser_rows=203`;
  - no missing canonical parser rows;
  - no extra parser rows outside the canonical inventory.
- Every generated parser row names:
  - the canonical config option;
  - the Roastty parser path or helper;
  - parser family;
  - current coverage status;
  - evidence artifact or the concrete missing evidence.
- A matrix assertion verifies that CFG-217 is internally consistent:
  - if every parser inventory row is `Oracle complete`, CFG-217 may be `Pass`;
  - if any parser inventory row is `Audit covered` or `Gap`, CFG-217 remains
    `Gap`;
  - CFG-217 points to `config-parser-inventory.md`.
- Any newly added Rust tests pass with the narrowest relevant `cargo test`
  filter.
- `cargo fmt --manifest-path roastty/Cargo.toml` is run if Rust files change.
- `prettier --write --prose-wrap always --print-width 80` is run on changed
  markdown files.
- `git diff --check` passes.

Suggested commands:

```bash
python3 issues/0805-roastty-ghostty-parity/config_parser_inventory.py \
  --upstream vendor/ghostty/src/config/Config.zig \
  --roastty roastty/src/config/mod.rs \
  --config-inventory issues/0805-roastty-ghostty-parity/config-inventory.md \
  --output issues/0805-roastty-ghostty-parity/config-parser-inventory.md \
  --matrix issues/0805-roastty-ghostty-parity/config-matrix.md
python3 - <<'PY'
from pathlib import Path

matrix_rows = []
for line in Path('issues/0805-roastty-ghostty-parity/config-matrix.md').read_text().splitlines():
    if not line.startswith('| CFG-'):
        continue
    matrix_rows.append([cell.strip() for cell in line.strip('|').split('|')])
cfg217 = next(row for row in matrix_rows if row[0] == 'CFG-217')
assert 'config-parser-inventory.md' in cfg217[6], cfg217

parser_rows = []
for line in Path('issues/0805-roastty-ghostty-parity/config-parser-inventory.md').read_text().splitlines():
    if not line.startswith('| PARSE-'):
        continue
    parser_rows.append([cell.strip() for cell in line.strip('|').split('|')])
assert len(parser_rows) == 203, len(parser_rows)
incomplete = [row for row in parser_rows if row[4] != 'Oracle complete']
assert (not incomplete and cfg217[4] == 'Pass') or (incomplete and cfg217[4] == 'Gap')
print(f'parser_rows={len(parser_rows)} incomplete={len(incomplete)} cfg217={cfg217[4]}')
PY
git diff --check
```

## Design Review

Fresh-context adversarial design review found two required issues:

- The draft could overclaim CFG-217 parser parity from sample coverage, even
  though CFG-217 requires the full documented non-default value range.
- The draft named nonexistent `Config::set_field_raw`; Roastty's parser dispatch
  is `Config::set_from_source`.

Fixes:

- Scoped this experiment as an audit map that keeps CFG-217 `Gap` unless every
  row reaches an upstream-derived `Oracle complete` standard.
- Updated the scanner target to `Config::set_from_source`.

Re-review approved the fixed design:

```text
VERDICT: APPROVED

Findings: none.
```
