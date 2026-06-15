# Experiment 172: Native Menu Runtime

## Description

`RUNTIME-011B2B` still includes native menu display/validation in the remaining
live macOS GUI gap. Earlier experiments proved AppleScript app/window/tab/split
commands and lower-level AppleScript keyboard/mouse input delivery, but they did
not prove that Roastty's real macOS menu bar is visible to the OS accessibility
tree, validates representative menu items correctly, or dispatches
representative native menu actions into the running app.

This experiment will split a narrow native-menu slice out of `RUNTIME-011B2B` by
adding a live debug-app guard that uses System Events against the exact launched
Roastty PID. The guard will prove:

- the native menu bar is present for the debug app process;
- expected top-level menus and representative menu items are visible;
- representative validation states are correct with a live terminal window;
- representative native menu actions mutate app state and are observable through
  the existing AppleScript object model.

This experiment will not claim titlebar/fullscreen/quick-terminal visuals,
screenshot/pixel evidence, split visual/layout parity, broader command-palette
GUI behavior, cursor/pointer pixels, broad keyboard/mouse walkthrough parity, or
notification/link/bell GUI effects.

## Changes

- `issues/0805-roastty-ghostty-parity/macos_native_menu_runtime.py`
  - Add a new live debug-app guard using the same absolute app bundle, isolated
    config, scoped process cleanup, and new-crash-report failure pattern used by
    `macos_applescript_workflow_runtime.py`.
  - Launch the debug app with `macos-applescript = true` and a controlled child
    command so the app has a real terminal window.
  - Use System Events to resolve the application process by exact Unix PID and
    fail if the frontmost process is not that PID before inspecting or clicking
    menus.
  - Assert the menu bar exposes the expected top-level menu names, including the
    application, File, Edit, View, Window, and Help menus.
  - Assert representative menu items exist and are enabled when a terminal
    window is active: New Window, New Tab, Split Right, Split Left, Split Down,
    Split Up, Close, Toggle Full Screen, Quick Terminal, and Command Palette.
  - Assert representative validated menu items reflect app state: Undo and Redo
    are disabled with no undo stack, and Float on Top / Use as Default are
    enabled only while a primary terminal window is key.
  - Click New Tab through the native File menu and assert the selected window's
    tab count increases through AppleScript.
  - Click Split Right through the native File menu and assert the selected tab's
    terminal count increases through AppleScript.
- `issues/0805-roastty-ghostty-parity/config_runtime_inventory.py`
  - Split a new Oracle-complete row from `RUNTIME-011B2B` for live native menu
    visibility, representative validation, and representative action dispatch.
  - Reduce the remaining `RUNTIME-011B2B` gap so it no longer lists native menu
    display/validation.
- `issues/0805-roastty-ghostty-parity/config-runtime-inventory.md`
  - Regenerate from the inventory script.
- `issues/0805-roastty-ghostty-parity/config-matrix.md`
  - Regenerate CFG-223 summary. It must remain `Gap`.
- Existing CFG-223/runtime guards
  - Update expected counts from 77 runtime rows, 70 Oracle-complete rows, and 73
    closed rows to 78 runtime rows, 71 Oracle-complete rows, and 74 closed rows.
    Incomplete and gap counts remain 4.
  - Update references that describe the remaining macOS app GUI gap so they no
    longer require native menu display/validation.
- `issues/0805-roastty-ghostty-parity/README.md`
  - Add the experiment link and update Learnings after the result.

## Verification

Pass criteria:

- The built debug Roastty app launches from the absolute app bundle path.
- The guard uses an isolated config with `macos-applescript = true` and does not
  depend on the user's normal config.
- System Events targets the exact launched Roastty Unix PID before inspecting or
  clicking menus.
- The native menu bar exposes expected top-level menus and representative menu
  items.
- Representative menu validation is observed through System Events, including
  disabled Undo/Redo and enabled terminal-window items when a primary terminal
  window is key.
- Clicking New Tab through the native menu increases the selected window's tab
  count.
- Clicking Split Right through the native menu increases the selected tab's
  terminal count.
- The live guard still fails if a new Roastty crash report appears during the
  workflow.
- The new runtime inventory row is `Oracle complete`.
- `RUNTIME-011B2B` remains `Gap` for titlebar/fullscreen/quick-terminal visuals,
  screenshot/pixel evidence, broader command-palette GUI behavior, split
  visual/layout parity, cursor/pointer pixels, and broader keyboard/mouse
  walkthroughs.
- CFG-223 remains `Gap`.

Commands:

```bash
(cd roastty && macos/build.nu --action build)
PYTHONDONTWRITEBYTECODE=1 python3 issues/0805-roastty-ghostty-parity/macos_native_menu_runtime.py
PYTHONDONTWRITEBYTECODE=1 python3 issues/0805-roastty-ghostty-parity/config_runtime_inventory.py --output issues/0805-roastty-ghostty-parity/config-runtime-inventory.md --matrix issues/0805-roastty-ghostty-parity/config-matrix.md
PYTHONDONTWRITEBYTECODE=1 python3 issues/0805-roastty-ghostty-parity/platform_runtime_classification.py --config-inventory issues/0805-roastty-ghostty-parity/config-inventory.md --output issues/0805-roastty-ghostty-parity/platform-runtime-classification.md
for f in issues/0805-roastty-ghostty-parity/*_runtime_parity.py issues/0805-roastty-ghostty-parity/terminal_runtime_residual_audit.py issues/0805-roastty-ghostty-parity/link_hover_preview_dispatch_parity.py issues/0805-roastty-ghostty-parity/link_hover_modifier_refresh_parity.py issues/0805-roastty-ghostty-parity/link_preview_context_runtime_parity.py; do
  PYTHONDONTWRITEBYTECODE=1 python3 "$f"
done
PYTHONDONTWRITEBYTECODE=1 python3 issues/0805-roastty-ghostty-parity/macos_app_workflow_plumbing_parity.py
prettier --write --prose-wrap always --print-width 80 issues/0805-roastty-ghostty-parity/README.md issues/0805-roastty-ghostty-parity/172-native-menu-runtime.md issues/0805-roastty-ghostty-parity/config-runtime-inventory.md issues/0805-roastty-ghostty-parity/config-matrix.md issues/0805-roastty-ghostty-parity/platform-runtime-classification.md
git diff --check
```

Fail criteria:

- The guard targets the app by process name only instead of exact Unix PID.
- The guard treats app launch or AppleScript command success as sufficient
  without inspecting the real native menu accessibility tree.
- The guard only checks menu item existence and does not check representative
  validation state.
- The guard clicks menu items without proving resulting app state changes.
- The guard depends on the user's normal config or leaves the debug app running.
- The inventory claims titlebar/fullscreen/quick-terminal visuals,
  screenshot/pixel evidence, split visual/layout parity, cursor/pointer pixels,
  broader command-palette GUI behavior, or broad keyboard/mouse walkthrough
  parity.
- CFG-223 is marked complete.

## Design Review

Adversarial review was performed by a fresh-context Codex subagent.

Verdict: Approved.

Findings: none.

## Result

**Result:** Pass

Experiment 172 implemented and verified a live native-menu runtime guard.

Changes:

- `issues/0805-roastty-ghostty-parity/macos_native_menu_runtime.py`
  - Added a live debug-app guard using the absolute `Roastty.app` bundle,
    isolated config, exact launched Unix PID targeting through System Events,
    scoped cleanup, and new-crash-report detection.
  - Proved the native menu bar exposes the expected top-level Roastty, File,
    Edit, View, Window, and Help menus.
  - Proved representative File/Edit/View/Window menu items are visible.
  - Proved representative validation states: Undo and Redo are disabled with no
    undo stack, and representative terminal-window menu items are enabled while
    a primary terminal window is key.
  - Clicked New Tab through the native File menu and observed the front window's
    tab count increase through AppleScript.
  - Clicked Split Right through the native File menu and observed the selected
    tab's terminal count increase through AppleScript.
  - Dismissed native menus with Escape before switching from System Events back
    to AppleScript object-model queries, avoiding open-menu stalls.
- `issues/0805-roastty-ghostty-parity/config_runtime_inventory.py`
  - Added `RUNTIME-011B2F` as Oracle complete for live native menu visibility,
    representative validation, and representative New Tab / Split Right
    dispatch.
  - Reduced the remaining `RUNTIME-011B2B` gap so native menu display/validation
    is no longer part of the open macOS app bucket.
  - Updated CFG-223 count assertions to `78` runtime rows, `71` Oracle-complete
    rows, `74` closed rows, `4` incomplete rows, and `4` gap rows.
- Existing CFG-223 runtime guard scripts
  - Updated shared CFG-223 count expectations from `70` Oracle-complete / `73`
    closed rows to `71` Oracle-complete / `74` closed rows.
  - Updated stale remaining-gap assertions that still expected native menu
    display/validation to remain open.
- Generated docs
  - Regenerated `config-runtime-inventory.md`, `config-matrix.md`, and
    `platform-runtime-classification.md`.

Verification:

```bash
(cd roastty && macos/build.nu --action build)
# passed

PYTHONDONTWRITEBYTECODE=1 python3 issues/0805-roastty-ghostty-parity/macos_native_menu_runtime.py
# macos_native_menu_runtime=pass

PYTHONDONTWRITEBYTECODE=1 python3 issues/0805-roastty-ghostty-parity/config_runtime_inventory.py --output issues/0805-roastty-ghostty-parity/config-runtime-inventory.md --matrix issues/0805-roastty-ghostty-parity/config-matrix.md
# runtime_rows=78
# oracle_complete=71
# closed=74
# audit_covered=0
# incomplete=4
# gap=4
# cfg223=Gap

PYTHONDONTWRITEBYTECODE=1 python3 issues/0805-roastty-ghostty-parity/platform_runtime_classification.py --config-inventory issues/0805-roastty-ghostty-parity/config-inventory.md --output issues/0805-roastty-ghostty-parity/platform-runtime-classification.md
# platform_options=32
# gap=15
# not_applicable=15
# oracle_complete=2

for f in issues/0805-roastty-ghostty-parity/*_runtime_parity.py issues/0805-roastty-ghostty-parity/terminal_runtime_residual_audit.py issues/0805-roastty-ghostty-parity/link_hover_preview_dispatch_parity.py issues/0805-roastty-ghostty-parity/link_hover_modifier_refresh_parity.py issues/0805-roastty-ghostty-parity/link_preview_context_runtime_parity.py; do
  PYTHONDONTWRITEBYTECODE=1 python3 "$f"
done
# all listed guards passed

PYTHONDONTWRITEBYTECODE=1 python3 issues/0805-roastty-ghostty-parity/macos_app_workflow_plumbing_parity.py
# macos_app_workflow_plumbing_parity=pass

PYTHONDONTWRITEBYTECODE=1 python3 issues/0805-roastty-ghostty-parity/macos_applescript_workflow_runtime.py && PYTHONDONTWRITEBYTECODE=1 python3 issues/0805-roastty-ghostty-parity/macos_native_menu_runtime.py
# macos_applescript_workflow_runtime=pass
# macos_native_menu_runtime=pass
```

## Conclusion

Native menu visibility, representative validation, and representative native
menu action dispatch are no longer part of the remaining CFG-223 macOS app gap.
The live guard now proves this through the real macOS menu bar, exact-PID System
Events targeting, and AppleScript-observed app state changes after menu clicks.

CFG-223 remains `Gap` because unrelated GUI work still needs proof:
titlebar/fullscreen/quick-terminal visuals, screenshot/pixel evidence, broader
command-palette GUI behavior, split visual/layout parity, cursor/pointer pixels,
and broader keyboard/mouse walkthrough parity.

## Completion Review

Fresh-context adversarial reviewer `Planck the 2nd` reviewed the completed
experiment and approved it with no findings.

The reviewer did not rerun `macos_native_menu_runtime.py` because the guard
writes a temporary config, launches/quits the app, and drives GUI state, which
was outside the reviewer's read-only discipline. The reviewer did inspect the
guard and verified that it targets the exact launched Unix PID before menu
inspection/clicks, checks menu visibility and validation, dismisses menus before
AppleScript object queries, and observes New Tab / Split Right state changes.

Read-only checks performed by the reviewer:

- `git status --short` and `git log` confirmed the result commit had not been
  made and only the plan commit existed.
- Scoped diffs matched the experiment scope.
- Other changed `*_runtime_parity.py` files only updated CFG-223 `70/73` to
  `71/74`; scoped exceptions removed stale native-menu gap assertions.
- `git diff --check` passed.
- `command_palette_runtime_parity.py` passed.
- `macos_app_workflow_plumbing_parity.py` passed.
- In-memory inventory validation reported `runtime_rows=78`,
  `oracle_complete=71`, `closed=74`, `audit_covered=0`, `incomplete=4`, `gap=4`,
  and `cfg223=Gap`.

Final verdict: **Approved**.
