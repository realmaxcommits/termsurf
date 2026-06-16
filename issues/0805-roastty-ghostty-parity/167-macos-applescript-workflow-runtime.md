# Experiment 167: macOS AppleScript Workflow Runtime

## Description

`RUNTIME-011B2` still groups the remaining live macOS
app/window/tab/split/menu/titlebar/fullscreen/quick-terminal and broader command
palette GUI effects. Experiment 166 proved copied workflow plumbing by source
parity and focused Swift tests, but it intentionally did not prove that the
built Roastty app can be driven through the live macOS automation surface.

A narrow live slice is available through Roastty's copied AppleScript dictionary
and handlers:

- launch the built debug app by absolute bundle path;
- enable AppleScript through an isolated debug config using
  `macos-applescript = true`;
- ask the running app for windows, tabs, and terminals;
- create a new window and tab;
- create a split terminal whose command records a temp-file marker;
- send a small text input marker to the selected terminal through the
  AppleScript `input text` command, then prove the controlled child process
  received that marker.

This experiment will split `RUNTIME-011B2` into:

- `RUNTIME-011B2A`: **Oracle complete** for live AppleScript-driven Roastty app
  workflow automation covering launch, app dictionary access, window creation,
  tab creation/selection/close, split-terminal creation with a command side
  effect, and terminal text input command dispatch.
- `RUNTIME-011B2B`: **Gap** for remaining live macOS GUI behavior: native menu
  display/validation, titlebar/fullscreen/quick-terminal visual behavior,
  screenshot/pixel evidence, broader command-palette GUI behavior, and deeper
  input navigation/pixel walkthroughs, including returned split-terminal object
  re-resolution and focus/close commands.

This experiment will not claim visual parity with Ghostty, native menu display
or validation parity, fullscreen parity, quick-terminal parity, screenshot/pixel
parity, or complete keyboard/mouse walkthrough parity.

## Changes

- `issues/0805-roastty-ghostty-parity/macos_applescript_workflow_runtime.py`
  - Add a live guard that builds on the macOS AppleScript testing instructions:
    target `roastty/macos/build/Debug/Roastty.app` by absolute app path, launch
    it, drive it with `osascript`, and quit/clean up in a `finally` path.
  - Create an isolated temporary config with `macos-applescript = true` and
    launch the debug binary with `ROASTTY_CONFIG_PATH` so the test does not
    depend on the user's normal `~/.config/roastty/config`.
  - Assert the live scripting surface can query windows/tabs/terminals, create a
    new window, create/select/close a tab, create a split terminal with a
    side-effect marker command, and dispatch `input text` to a live terminal.
  - For `input text`, create a terminal from a temporary surface configuration
    whose `command` waits for one stdin line and writes it to a temp file. Send
    the marker plus newline with AppleScript `input text`, then assert the file
    contains the marker before claiming input dispatch parity.
  - For split creation, create the split from a temporary surface configuration
    whose `command` writes a second temp-file marker. Assert that marker exists
    before claiming split creation parity.
  - Keep assertions structural and command-based; do not infer pixel or visual
    parity from AppleScript object counts.
- `issues/0805-roastty-ghostty-parity/config_runtime_inventory.py`
  - Split `RUNTIME-011B2` into the complete live AppleScript workflow runtime
    row and the reduced remaining live macOS GUI gap.
- `issues/0805-roastty-ghostty-parity/config-runtime-inventory.md`
  - Regenerate from the inventory script.
- `issues/0805-roastty-ghostty-parity/config-matrix.md`
  - Regenerate CFG-223 summary. It must remain `Gap`.
- Existing CFG-223/runtime guards
  - Update expected counts from 73 runtime rows, 66 Oracle-complete rows, and 69
    closed rows to 74 runtime rows, 67 Oracle-complete rows, and 70 closed rows.
    Incomplete and gap counts remain 4.
  - Update references from `RUNTIME-011B2` to `RUNTIME-011B2B` where they mean
    the remaining visual/native-menu/fullscreen/quick-terminal GUI gap.
- `issues/0805-roastty-ghostty-parity/README.md`
  - Add the experiment link and update Learnings after the result.

## Verification

Pass criteria:

- The built debug Roastty app launches from an absolute app bundle path.
- The guard enables `macos-applescript` using an isolated debug config path, not
  the user's normal config.
- AppleScript can address the built app by absolute bundle path and read the
  expected application/window/tab/terminal object model.
- AppleScript can create a new window.
- AppleScript can create and select a tab in that window.
- AppleScript can create a split terminal and the split's controlled child
  process records its exact marker in a temp file.
- AppleScript can dispatch `input text` to the focused terminal and the
  controlled child process records the exact marker in a temp file.
- The guard always quits or kills only the debug app process it launched.
- `RUNTIME-011B2A` is `Oracle complete` and cites the live guard.
- `RUNTIME-011B2B` remains `Gap` for native menu display/validation,
  titlebar/fullscreen/quick-terminal visuals, screenshot/pixel evidence, broader
  command-palette GUI behavior, returned split-terminal object re-resolution and
  focus/close commands, and deeper input walkthroughs.
- CFG-223 remains `Gap`.

Commands:

```bash
(cd roastty && macos/build.nu --action build)
PYTHONDONTWRITEBYTECODE=1 python3 issues/0805-roastty-ghostty-parity/macos_applescript_workflow_runtime.py
PYTHONDONTWRITEBYTECODE=1 python3 issues/0805-roastty-ghostty-parity/config_runtime_inventory.py --output issues/0805-roastty-ghostty-parity/config-runtime-inventory.md --matrix issues/0805-roastty-ghostty-parity/config-matrix.md
PYTHONDONTWRITEBYTECODE=1 python3 issues/0805-roastty-ghostty-parity/platform_runtime_classification.py --config-inventory issues/0805-roastty-ghostty-parity/config-inventory.md --output issues/0805-roastty-ghostty-parity/platform-runtime-classification.md
for f in issues/0805-roastty-ghostty-parity/*_runtime_parity.py issues/0805-roastty-ghostty-parity/terminal_runtime_residual_audit.py issues/0805-roastty-ghostty-parity/link_hover_preview_dispatch_parity.py issues/0805-roastty-ghostty-parity/link_hover_modifier_refresh_parity.py issues/0805-roastty-ghostty-parity/link_preview_context_runtime_parity.py; do
  PYTHONDONTWRITEBYTECODE=1 python3 "$f"
done
prettier --write --prose-wrap always --print-width 80 issues/0805-roastty-ghostty-parity/README.md issues/0805-roastty-ghostty-parity/167-macos-applescript-workflow-runtime.md issues/0805-roastty-ghostty-parity/config-runtime-inventory.md issues/0805-roastty-ghostty-parity/config-matrix.md
git diff --check
```

Fail criteria:

- The guard targets the app by name instead of absolute bundle path.
- The guard depends on the user's normal config or leaves user defaults/config
  state behind.
- The guard leaves the debug app running after failure.
- The guard treats `input text` returning without error as sufficient evidence
  without asserting the marker reached the child process.
- The guard treats `split` returning without error as sufficient evidence
  without asserting the split terminal's controlled command ran.
- The guard claims visual, menu, fullscreen, quick-terminal, screenshot, or
  broad command-palette parity from AppleScript object-count assertions.
- `RUNTIME-011B2B` omits any remaining live GUI visual or native-menu gaps.
- CFG-223 is marked complete.

## Design Review

Reviewed by a fresh-context Codex adversarial subagent.

Initial verdict: **Changes required**.

- Required: the `input text` pass criterion only required no AppleScript/runtime
  error. That would not prove the marker reached the terminal child process.

Fix:

- Tightened the design so the future live guard must create a terminal from a
  temporary surface configuration whose `command` reads one stdin line and
  writes it to a temp file. The guard must send the marker plus newline with
  AppleScript `input text` and assert the file contains the marker before
  claiming input dispatch parity.

Re-review verdict: **Approved**. The reviewer confirmed the pass/fail criteria
now require the controlled child process to record the exact marker and reject
treating `input text` returning without error as sufficient evidence.

## Result

**Result:** Pass

`RUNTIME-011B2` was split into:

- `RUNTIME-011B2A`: **Oracle complete** for live AppleScript-driven Roastty app
  workflow automation covering absolute-path debug app launch, isolated
  `ROASTTY_CONFIG_PATH`, AppleScript dictionary access, window creation, tab
  creation/selection/close, split creation with a command side effect, scoped
  cleanup, and side-effect-proven terminal `input text` dispatch.
- `RUNTIME-011B2B`: **Gap** for remaining live macOS GUI behavior: native menu
  display/validation, titlebar/fullscreen/quick-terminal visuals,
  screenshot/pixel evidence, returned split-terminal object re-resolution and
  focus/close commands, broader command-palette GUI behavior, and deeper input
  navigation/pixel walkthroughs.

Implementation found and fixed two Roastty automation blockers:

- `roastty_config_get` did not expose `macos-applescript`, so Swift's
  `Roastty.Config.macosAppleScript` getter always returned its local default
  `false` and disabled AppleScript even when the parsed config enabled it.
- `ScriptWindow` used AppKit tab-group or window object identity for scripting
  IDs. Those IDs stopped resolving after tab-group creation. Window scripting
  IDs now use the primary terminal controller identity so a window reference can
  survive creating/selecting/closing a tab.

The live guard deliberately does not claim native menu display/validation,
titlebar/fullscreen/quick-terminal visual behavior, screenshot/pixel parity,
returned split-terminal object re-resolution, split focus/close command parity,
or broader command-palette GUI parity.

Verification run:

```text
(cd roastty && macos/build.nu --action build)
** BUILD SUCCEEDED **

PYTHONDONTWRITEBYTECODE=1 python3 issues/0805-roastty-ghostty-parity/macos_applescript_workflow_runtime.py
macos_applescript_workflow_runtime=pass

PYTHONDONTWRITEBYTECODE=1 python3 issues/0805-roastty-ghostty-parity/config_runtime_inventory.py --output issues/0805-roastty-ghostty-parity/config-runtime-inventory.md --matrix issues/0805-roastty-ghostty-parity/config-matrix.md
runtime_rows=74
oracle_complete=67
closed=70
audit_covered=0
incomplete=4
gap=4
cfg223=Gap

PYTHONDONTWRITEBYTECODE=1 python3 issues/0805-roastty-ghostty-parity/platform_runtime_classification.py --config-inventory issues/0805-roastty-ghostty-parity/config-inventory.md --output issues/0805-roastty-ghostty-parity/platform-runtime-classification.md
platform_options=32
gap=15
not_applicable=15
oracle_complete=2

for f in issues/0805-roastty-ghostty-parity/*_runtime_parity.py issues/0805-roastty-ghostty-parity/terminal_runtime_residual_audit.py issues/0805-roastty-ghostty-parity/link_hover_preview_dispatch_parity.py issues/0805-roastty-ghostty-parity/link_hover_modifier_refresh_parity.py issues/0805-roastty-ghostty-parity/link_preview_context_runtime_parity.py; do
  PYTHONDONTWRITEBYTECODE=1 python3 "$f"
done
parity_guards=pass

cargo test --manifest-path roastty/Cargo.toml config_get_macos_unit_test_scalar_keys
test tests::config_get_macos_unit_test_scalar_keys ... ok

cargo fmt --manifest-path roastty/Cargo.toml --check

(cd roastty && macos/build.nu --action test)
Test run with 219 tests in 23 suites passed after 2.002 seconds.
** TEST SUCCEEDED **
```

The macOS build/test runs emitted existing Swift 6/Main Thread Checker,
pasteboard, and linker deployment-version warnings, but both `xcodebuild`
actions reported success.

## Conclusion

Roastty now has a durable live AppleScript workflow guard that proves the debug
app can be launched with isolated config, addressed by absolute app path, driven
through window/tab/split/input commands, and cleaned up. The guard uses
temp-file side effects for input and split creation so successful command
returns are not treated as sufficient evidence.

The remaining live macOS app gap is narrower: visual/native-menu/fullscreen/
quick-terminal behavior, screenshot/pixel evidence, returned split-terminal
object re-resolution and focus/close commands, broader command-palette GUI
behavior, and deeper input navigation remain in `RUNTIME-011B2B`.

## Completion Review

Reviewed by a fresh-context Codex adversarial subagent.

Initial verdict: **Changes required**.

- Required: `platform-runtime-classification.md` still classified
  `macos-applescript` as a `Gap` owned by `RUNTIME-011B2B`, contradicting the
  completed `RUNTIME-011B2A` claim.
- Optional: `wait_for_app` did not retry the `AssertionError` raised by
  `run_osascript` on transient AppleScript readiness failures.

Fix:

- Updated `platform_runtime_classification.py` so `macos-applescript` is an
  `Oracle complete` platform row owned by `RUNTIME-011B2A`, then regenerated
  `platform-runtime-classification.md`.
- Updated `config_runtime_inventory.py` and the regenerated
  `config-runtime-inventory.md` so `RUNTIME-013` explicitly points
  `macos-applescript` to `RUNTIME-011B2A`.
- Updated `wait_for_app` to retry `AssertionError` readiness failures from
  `run_osascript`.

Re-review verdict: **Approved**.
