# Experiment 152: Command palette runtime

## Description

`RUNTIME-011` still groups macOS app/window/tab/split/menu and command palette
UI effects together. This experiment isolates the command palette slice:

- the copied command palette SwiftUI view surface;
- the terminal command palette overlay and focus-return behavior;
- config-derived `command-palette-entry` options, unsupported-action filtering,
  shortcut display, and action callback dispatch;
- the app action path that posts the command-palette toggle notification;
- the terminal controller state that receives the notification and toggles
  `commandPaletteIsShowing`;
- command palette keyboard-event shielding in `SurfaceView_AppKit`.

This is narrower than a full command palette GUI walkthrough. It will not claim
pixel-level command palette presentation, actual keyboard navigation inside the
running app, windows/tabs/splits/titlebar/fullscreen behavior, quick terminal
behavior, or broader menu parity.

## Changes

- Add a focused static parity guard:
  - `issues/0805-roastty-ghostty-parity/command_palette_runtime_parity.py`
  - Assert that pinned Ghostty and Roastty `CommandPalette.swift` and
    `CommandPaletteIntent.swift` match after expected Ghostty-to-Roastty
    renames.
  - Assert that `TerminalCommandPalette.swift` preserves the pinned command
    option behavior, allowing only Roastty's existing helper extraction
    (`terminalCommandOptions`) that makes the custom-command slice testable.
  - Assert copied app/controller/surface markers for toggle action dispatch,
    notification delivery, `commandPaletteIsShowing`, focus return, and
    command-palette keyboard shielding.
  - Assert the hosted macOS command palette test markers.
- Update `config_runtime_inventory.py` to split `RUNTIME-011` into:
  - an Oracle complete command palette runtime/UI plumbing row owned by this
    experiment;
  - a remaining macOS app/window/tab/split/menu gap row for launch/window, tabs,
    splits, menus, titlebar, fullscreen, quick terminal, and broader command
    palette GUI walkthrough behavior.
- Regenerate `config-runtime-inventory.md` and `config-matrix.md`.
- Update existing runtime parity guards and `terminal_runtime_residual_audit.py`
  for the new CFG-223 row counts and remaining macOS app gap id.
- Update Issue 805 learnings with the command palette runtime finding after the
  result is known.

## Verification

Pass criteria:

- The static command palette parity guard passes:

```bash
PYTHONDONTWRITEBYTECODE=1 python3 issues/0805-roastty-ghostty-parity/command_palette_runtime_parity.py
```

- The hosted command palette macOS unit tests pass:

```bash
xcodebuild test \
  -project roastty/macos/Roastty.xcodeproj \
  -scheme Roastty \
  -testPlan Roastty \
  -only-testing:RoasttyTests/CommandPaletteHostedTests
```

- The runtime inventory generator reports one additional Oracle complete row and
  the same total number of unresolved CFG-223 gaps unless this experiment
  discovers a real fixable discrepancy:

```bash
PYTHONDONTWRITEBYTECODE=1 python3 issues/0805-roastty-ghostty-parity/config_runtime_inventory.py --output issues/0805-roastty-ghostty-parity/config-runtime-inventory.md --matrix issues/0805-roastty-ghostty-parity/config-matrix.md
```

- All runtime parity guards still pass:

```bash
for guard in issues/0805-roastty-ghostty-parity/*_runtime_parity.py; do
  PYTHONDONTWRITEBYTECODE=1 python3 "$guard" || exit 1
done
```

- The terminal residual audit still passes:

```bash
PYTHONDONTWRITEBYTECODE=1 python3 issues/0805-roastty-ghostty-parity/terminal_runtime_residual_audit.py
```

- Markdown and diff hygiene pass:

```bash
prettier --write --prose-wrap always --print-width 80 \
  issues/0805-roastty-ghostty-parity/README.md \
  issues/0805-roastty-ghostty-parity/152-command-palette-runtime.md \
  issues/0805-roastty-ghostty-parity/config-runtime-inventory.md \
  issues/0805-roastty-ghostty-parity/config-matrix.md
git diff --check
```

## Design Review

Adversarial subagent `019ec9fe-dfd0-7780-88e9-67fa59018508` reviewed the design
with fresh context and returned `VERDICT: APPROVED`.

Findings: none.

The reviewer confirmed that the README links Experiment 152 as `Designed`, the
experiment has the required design sections, the scope excludes full GUI/pixel
parity and broader app behavior, the planned `terminalCommandOptions` helper
allowance preserves Ghostty's inline filter/map behavior, the toggle
notification and controller state plumbing match after renames, the `Roastty`
scheme and `RoasttyTests` target exist for the planned xcodebuild test command,
and `git diff --check` passed.
