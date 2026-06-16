# Experiment 169: Config Facet Matrix Reconciliation

## Description

`config-matrix.md` still shows CFG-217 through CFG-222 as broad `Gap` rows even
though their facet inventories now report zero incomplete rows:

- `config-parser-inventory.md`: 203 `Oracle complete`, 0 audit, 0 gap;
- `config-formatter-inventory.md`: 203 `Oracle complete`, 0 audit, 0 gap;
- `config-diagnostic-inventory.md`: 203 `Oracle complete`, 0 audit, 0 gap;
- `config-finalization-inventory.md`: 17 `Oracle complete`, 0 audit, 0 gap;
- `config-load-inventory.md`: 18 `Oracle complete`, 0 audit, 0 gap;
- `config-reload-inventory.md`: 14 `Oracle complete`, 0 audit, 0 gap.

The top-level matrix is therefore stale for CFG-217 through CFG-222. Closing
Issue 805 requires zero unresolved `Gap` rows, so these completed facet rows
must be reconciled before the remaining work can focus honestly on CFG-223's
runtime/UI gaps.

This experiment will not modify parser, formatter, diagnostic, finalization,
load, reload, runtime, or app behavior. It only reconciles generated issue
bookkeeping with already-complete facet inventories.

## Changes

- `issues/0805-roastty-ghostty-parity/config_parser_inventory.py`
  - Fix CFG-217 generated matrix prose so a `Pass` row does not still say parser
    oracles are incomplete.
- `issues/0805-roastty-ghostty-parity/config_formatter_inventory.py`
  - Fix CFG-218 generated matrix prose so a `Pass` row does not still say
    formatter oracles are incomplete.
- `issues/0805-roastty-ghostty-parity/config_diagnostic_inventory.py`
  - Fix CFG-219 generated matrix prose so a `Pass` row does not still say
    diagnostic parity is incomplete.
- `issues/0805-roastty-ghostty-parity/config-matrix.md`
  - Regenerate CFG-217 through CFG-222 in dependency order from their facet
    inventories so they become `Pass`.
  - Leave CFG-223 as `Gap` with the current runtime/UI inventory counts.
- `issues/0805-roastty-ghostty-parity/README.md`
  - Add the experiment link and update Learnings after the result.

## Verification

Pass criteria:

- Regenerating each config facet inventory succeeds:
  - parser;
  - formatter;
  - diagnostic;
  - finalization;
  - load;
  - reload;
  - runtime.
- `config-matrix.md` marks CFG-217, CFG-218, CFG-219, CFG-220, CFG-221, and
  CFG-222 as `Pass`.
- `config-matrix.md` marks CFG-223 as `Gap`.
- The six reconciled CFG rows point to their generated inventory files and do
  not contain stale prose saying the completed facet is still incomplete.
- `config-runtime-inventory.md` remains at 75 rows, 68 Oracle-complete rows, 71
  closed rows, 4 incomplete rows, and 4 runtime gaps.
- No source code outside issue bookkeeping changes.

Commands:

```bash
PYTHONDONTWRITEBYTECODE=1 python3 issues/0805-roastty-ghostty-parity/config_parser_inventory.py --upstream vendor/ghostty/src/config/Config.zig --roastty roastty/src/config/mod.rs --config-inventory issues/0805-roastty-ghostty-parity/config-inventory.md --output issues/0805-roastty-ghostty-parity/config-parser-inventory.md --matrix issues/0805-roastty-ghostty-parity/config-matrix.md
PYTHONDONTWRITEBYTECODE=1 python3 issues/0805-roastty-ghostty-parity/config_formatter_inventory.py --upstream vendor/ghostty/src/config/Config.zig --upstream-formatter-file vendor/ghostty/src/config/formatter_file.zig --upstream-formatter vendor/ghostty/src/config/formatter.zig --roastty roastty/src/config/mod.rs --config-inventory issues/0805-roastty-ghostty-parity/config-inventory.md --output issues/0805-roastty-ghostty-parity/config-formatter-inventory.md --matrix issues/0805-roastty-ghostty-parity/config-matrix.md
PYTHONDONTWRITEBYTECODE=1 python3 issues/0805-roastty-ghostty-parity/config_diagnostic_inventory.py --config-inventory issues/0805-roastty-ghostty-parity/config-inventory.md --parser-inventory issues/0805-roastty-ghostty-parity/config-parser-inventory.md --roastty roastty/src/config/mod.rs --output issues/0805-roastty-ghostty-parity/config-diagnostic-inventory.md --matrix issues/0805-roastty-ghostty-parity/config-matrix.md
PYTHONDONTWRITEBYTECODE=1 python3 issues/0805-roastty-ghostty-parity/config_finalization_inventory.py --output issues/0805-roastty-ghostty-parity/config-finalization-inventory.md --matrix issues/0805-roastty-ghostty-parity/config-matrix.md
PYTHONDONTWRITEBYTECODE=1 python3 issues/0805-roastty-ghostty-parity/config_load_inventory.py --output issues/0805-roastty-ghostty-parity/config-load-inventory.md --matrix issues/0805-roastty-ghostty-parity/config-matrix.md
PYTHONDONTWRITEBYTECODE=1 python3 issues/0805-roastty-ghostty-parity/config_reload_inventory.py --output issues/0805-roastty-ghostty-parity/config-reload-inventory.md --matrix issues/0805-roastty-ghostty-parity/config-matrix.md
PYTHONDONTWRITEBYTECODE=1 python3 issues/0805-roastty-ghostty-parity/config_runtime_inventory.py --output issues/0805-roastty-ghostty-parity/config-runtime-inventory.md --matrix issues/0805-roastty-ghostty-parity/config-matrix.md
python3 - <<'PY'
from pathlib import Path
matrix = Path("issues/0805-roastty-ghostty-parity/config-matrix.md").read_text()
rows = {
    line.split("|")[1].strip(): [cell.strip() for cell in line.split("|")[1:-1]]
    for line in matrix.splitlines()
    if line.startswith("| CFG-")
}
for cfg in ["CFG-217", "CFG-218", "CFG-219", "CFG-220", "CFG-221", "CFG-222"]:
    assert rows[cfg][4] == "Pass", rows[cfg]
    row_text = " ".join(rows[cfg]).lower()
    for stale in [
        "not complete",
        "not proven",
        "not yet proven",
        "not fully audited",
        "explicit follow-up",
        "tbd",
        "unresolved config facet",
    ]:
        assert stale not in row_text, (cfg, stale, rows[cfg])
assert rows["CFG-223"][4] == "Gap", rows["CFG-223"]
assert "Runtime inventory coverage: 68 rows Oracle complete; 71 rows closed; 4 rows are incomplete and 4 rows are runtime gaps." in rows["CFG-223"][-1]
PY
prettier --write --prose-wrap always --print-width 80 issues/0805-roastty-ghostty-parity/README.md issues/0805-roastty-ghostty-parity/169-config-facet-matrix-reconciliation.md issues/0805-roastty-ghostty-parity/config-matrix.md issues/0805-roastty-ghostty-parity/config-parser-inventory.md issues/0805-roastty-ghostty-parity/config-formatter-inventory.md issues/0805-roastty-ghostty-parity/config-diagnostic-inventory.md issues/0805-roastty-ghostty-parity/config-finalization-inventory.md issues/0805-roastty-ghostty-parity/config-load-inventory.md issues/0805-roastty-ghostty-parity/config-reload-inventory.md issues/0805-roastty-ghostty-parity/config-runtime-inventory.md
git diff --check
```

Fail criteria:

- Any CFG-217 through CFG-222 row remains `Gap`.
- Any reconciled CFG row says the completed facet remains incomplete, unproven,
  not fully audited, explicitly needs follow-up work, is `TBD`, or is an
  unresolved config facet.
- CFG-223 is marked `Pass`.
- Runtime/UI gap counts change without a runtime/UI experiment.
- Any non-issue bookkeeping source changes are included.

## Design Review

Reviewed by a fresh-context Codex adversarial subagent.

Initial verdict: **Changes required**.

- Required: the verification only rejected literal `not complete`, which would
  miss the current stale CFG-219 phrase `full diagnostic parity is not proven`.

Fix:

- Strengthened the matrix assertion to reject the actual stale phrase classes in
  the reconciled CFG rows: `not complete`, `not proven`, `not yet proven`,
  `not fully audited`, `explicit follow-up`, `tbd`, and
  `unresolved config facet`.

Re-review verdict: **Approved**.

## Result

**Result:** Pass

The facet generators were rerun in dependency order and reconciled the stale
top-level matrix rows:

- CFG-217, CFG-218, CFG-219, CFG-220, CFG-221, and CFG-222 now report `Pass`.
- CFG-223 remains `Gap`.
- The matrix now has 222 `Pass` rows and 1 `Gap` row.
- The runtime inventory remains unchanged at 75 rows, 68 Oracle-complete rows,
  71 closed rows, 4 incomplete rows, and 4 runtime gaps.

The generator wording for CFG-217, CFG-218, and CFG-219 now changes with row
status, so regenerated `Pass` rows no longer claim the completed facet remains
incomplete or unproven.

Verification run:

```text
ghostty_canonical=203
roastty_parser_rows=203
missing_canonical_parser_rows=0
missing_dispatch_rows=0
extra_parser_rows=0
compatibility_only_parser_arms=5
noncanonical_noncompat_parser_arms=0
oracle_complete=203
audit_covered=0
gap=0
ghostty_canonical=203
roastty_formatter_rows=203
missing_canonical_formatter_rows=0
extra_formatter_rows=0
oracle_complete=203
audit_covered=0
gap=0
no_output_rows=1
ghostty_canonical=203
diagnostic_rows=203
missing_canonical_diagnostic_rows=0
extra_diagnostic_rows=0
oracle_complete=203
audit_covered=0
gap=0
finalization_rows=17
oracle_complete=17
audit_covered=0
gap=0
load_rows=18
oracle_complete=18
audit_covered=0
gap=0
reload_rows=14
oracle_complete=14
closed=14
audit_covered=0
incomplete=0
gap=0
cfg222=Pass
runtime_rows=75
oracle_complete=68
closed=71
audit_covered=0
incomplete=4
gap=4
cfg223=Gap
matrix assertions passed
{'Pass': 222, 'Gap': 1}
```

## Conclusion

Experiment 169 closed the stale aggregate config rows and confirmed the
remaining Issue 805 config parity work is concentrated in CFG-223 runtime/UI
effects. The next experiment should target one of the four remaining CFG-223
runtime gaps rather than parser, formatter, diagnostic, finalization, load, or
reload coverage.

## Completion Review

Reviewed by a fresh-context Codex adversarial subagent.

Verdict: **Approved**.

Findings: none.

The reviewer independently confirmed that the result commit had not yet been
made, the changed files were limited to issue docs, generated inventories, and
inventory generator scripts, CFG-217 through CFG-222 were `Pass`, CFG-223
remained `Gap`, the matrix count was 222 `Pass` rows and 1 `Gap` row, the
reconciled rows did not contain the stale phrase classes from the design, the
runtime count text matched CFG-223, the README marked Experiment 169 as `Pass`,
the experiment file had `Result` and `Conclusion`, and `git diff --check`
passed.
