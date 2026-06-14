# Experiment 106: Runtime UI Effects Inventory

## Description

CFG-223 remains a broad gap after parser, formatter, diagnostic, finalization,
load, and reload parity were completed. Static config parity does not prove that
config options actually affect the running app, terminal, renderer, input,
clipboard, fonts, windows, notifications, and macOS bridge the same way as
pinned Ghostty.

This experiment will split CFG-223 into a generated runtime/UI effect inventory.
It is an inventory experiment, not a broad implementation pass. The goal is to
turn the unresolved row into explicit effect rows with source anchors, current
Roastty evidence, guard tier, and follow-up status.

The initial runtime/UI effect manifest is:

- `RUNTIME-001`: app-level clipboard policy effects (`clipboard-read`,
  `clipboard-write`);
- `RUNTIME-002`: clipboard copy/paste transformation effects
  (`clipboard-paste-protection`, `clipboard-paste-bracketed-safe`,
  `clipboard-codepoint-map`, `clipboard-trim-trailing-spaces`);
- `RUNTIME-003`: selection behavior effects (`selection-clear-on-typing`,
  `selection-clear-on-copy`, `selection-word-chars`, `copy-on-select`);
- `RUNTIME-004`: mouse reporting and mouse behavior effects (`mouse-reporting`,
  `mouse-shift-capture`, `mouse-scroll-multiplier`, `click-repeat-interval`,
  `cursor-click-to-move`, `mouse-hide-while-typing`, `right-click-action`,
  `middle-click-action`);
- `RUNTIME-005`: keyboard remap and keybind dispatch effects (`key-remap`,
  `keybind`, `command-palette-entry`);
- `RUNTIME-006`: color, palette, theme, and color-scheme runtime effects;
- `RUNTIME-007`: font selection, shaping, fallback, metrics, and font-size
  runtime effects;
- `RUNTIME-008`: renderer presentation effects such as vsync, opacity, blur,
  padding, cursor style, window padding color, and generated palette state;
- `RUNTIME-009`: terminal behavior toggles such as VT KAM, scrollback, alternate
  screen, shell integration, terminfo, and title reporting;
- `RUNTIME-010`: PTY/process launch effects such as command, working directory,
  environment, wait-after-command, abnormal-command-exit-runtime, and quit
  policy;
- `RUNTIME-011`: macOS app/window/tab/split/menu effects;
- `RUNTIME-012`: notifications, bell, command-finish notification, app
  notifications, and URL/link opening effects;
- `RUNTIME-013`: platform-specific or unsupported runtime effects that may be
  not applicable to Roastty, such as GTK/Linux-only settings;
- `RUNTIME-014`: already accepted runtime divergences that must be cross-linked
  to `divergences.md`.

Rows may be split during implementation if one manifest row is too broad for a
single guard, but the inventory must keep CFG-223 honest: CFG-223 can pass only
when every runtime/UI row is `Oracle complete`, `Not applicable`, or an accepted
documented divergence.

## Changes

- Add `issues/0805-roastty-ghostty-parity/config_runtime_inventory.py` to
  generate a runtime/UI effect inventory with explicit row IDs, pinned Ghostty
  source anchors, Roastty evidence anchors, status, guard tier, guard command,
  and notes.
- Add generated
  `issues/0805-roastty-ghostty-parity/config-runtime-inventory.md`.
- Update `issues/0805-roastty-ghostty-parity/config-matrix.md` row `CFG-223` to
  point at the runtime inventory, keep it `Gap` while any runtime row is
  incomplete, and report row counts.
- Update `issues/0805-roastty-ghostty-parity/README.md` learnings with the
  concrete runtime/UI effect groups and next gap priority.
- Do not change runtime behavior in this experiment unless a row is already
  implemented and only needs a narrow existing-test citation or matrix
  promotion.

## Verification

Pass/fail criteria:

- The generated runtime inventory includes every manifest row above, with
  explicit row IDs and pinned Ghostty/Roastty anchors.
- The generator fails for missing IDs, duplicate IDs, invalid status, missing
  evidence anchors, missing guard fields, or `CFG-223` marked `Pass` while any
  runtime row is incomplete.
- Existing CFG-217 through CFG-222 matrix rows remain unchanged.
- The inventory clearly distinguishes `Oracle complete`, `Gap`,
  `Not applicable`, and `Intentional divergence` rows.
- Any promoted `Oracle complete` row names a guard that would catch the runtime
  regression.

Commands:

```bash
PYTHONDONTWRITEBYTECODE=1 python3 issues/0805-roastty-ghostty-parity/config_runtime_inventory.py \
  --output issues/0805-roastty-ghostty-parity/config-runtime-inventory.md \
  --matrix issues/0805-roastty-ghostty-parity/config-matrix.md

PYTHONDONTWRITEBYTECODE=1 python3 - <<'PY'
from pathlib import Path

matrix = Path("issues/0805-roastty-ghostty-parity/config-matrix.md").read_text()
line = next(row for row in matrix.splitlines() if row.startswith("| CFG-223 "))
assert "config-runtime-inventory.md" in line
assert ("| Pass " in line) == (
    "0 rows are incomplete" in line and "0 rows are runtime gaps" in line
)
PY

python3 -m py_compile issues/0805-roastty-ghostty-parity/config_runtime_inventory.py
rm -rf issues/0805-roastty-ghostty-parity/__pycache__

prettier --check issues/0805-roastty-ghostty-parity/README.md \
  issues/0805-roastty-ghostty-parity/106-runtime-ui-effects-inventory.md \
  issues/0805-roastty-ghostty-parity/config-matrix.md \
  issues/0805-roastty-ghostty-parity/config-runtime-inventory.md

git diff --check
```

The result is `Pass` if the inventory is generated, validated, and closes
CFG-223 with every runtime/UI row complete, not applicable, or accepted as a
documented divergence. The result is `Partial` if the inventory exists but
identifies one or more unresolved runtime/UI gaps. The result is `Fail` if the
inventory cannot be grounded in pinned Ghostty source anchors or cannot be
checked deterministically.

## Design Review

Adversarial design review by fresh-context Codex subagent `Plato`:

- **Initial verdict:** Changes required.
- **Required finding:** The initial manifest omitted major config-driven
  clipboard, selection, and mouse runtime effects, including
  `clipboard-codepoint-map`, `selection-clear-on-copy`,
  `clipboard-trim-trailing-spaces`, `copy-on-select`, `right-click-action`,
  `middle-click-action`, `cursor-click-to-move`, and `mouse-hide-while-typing`.
- **Fix:** The manifest now explicitly names those copy transformation,
  selection/copy, and click/cursor mouse effects in the runtime rows.
- **Re-review verdict:** Approved. The reviewer confirmed the prior finding is
  resolved and no new required findings were introduced.
