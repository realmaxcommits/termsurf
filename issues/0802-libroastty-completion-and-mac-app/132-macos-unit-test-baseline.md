# Experiment 132: Phase G — macOS unit-test baseline

## Description

Restore the copied Roastty macOS app's non-UI XCTest baseline after the Phase G
keybinding/config work. The normal CLI gate,
`cd roastty && macos/build.nu --action test`, now builds the app and runs unit
tests, but the partial `.xcresult` from the interrupted run shows 201 tests
started, 188 passed, 1 skipped, and 12 failing assertions in
`ConfigTests`/`MenuShortcutManagerTests`.

The failures cluster in two app-facing ABI surfaces:

- `roastty_config_get` does not yet expose several config keys that the copied
  Swift app already reads (`auto-update-channel`, `focus-follows-mouse`,
  `maximize`, `window-title-font-family`, `macos-titlebar-style`,
  `macos-window-shadow`, `resize-overlay`, and `scrollbar`).
- `roastty_config_trigger`/the keybind store do not yet model the menu shortcut
  behavior expected by the copied tests: configured keybinds must override
  built-in defaults, and `keybind = super+d=unbind` must suppress the built-in
  `new_split:right` menu shortcut.

This experiment is deliberately limited to the non-UI app-test gate. The
command-palette UI automation timeout from Experiment 129 remains a later UI
harness problem; this experiment should not broaden into XCUITest permissions or
visual automation.

## Changes

- In `roastty/src/lib.rs`, extend `roastty_config_get` with the missing scalar
  and enum getters needed by the copied Swift config tests, returning the same C
  types the Swift app already requests.
- In `roastty/src/lib.rs`, adjust config-trigger lookup semantics so app menu
  shortcut sync can distinguish three cases: explicit configured shortcut,
  explicit `unbind`, and default shortcut fallback.
- Add focused Rust tests for the new `roastty_config_get` keys and keybind
  lookup semantics, including uppercase/unicode normalization where relevant.
- If the existing Swift bridge needs no logic changes, leave it untouched; if a
  tiny bridge change is required to represent `unbind` distinctly, keep it
  mechanical and covered by the copied Swift tests.
- Update this experiment's result and Issue 802's operating notes/roadmap after
  verification.

## Verification

Pass criteria:

- `cargo fmt`
- `cargo test -p roastty -- --test-threads=1`
- `cd roastty && macos/build.nu --action test --only-testing RoasttyTests/ConfigTests`
- `cd roastty && macos/build.nu --action test --only-testing RoasttyTests/MenuShortcutManagerTests`
- `cd roastty && macos/build.nu --action test`
- `git diff --check`

The final full macOS unit-test gate must either pass or, if it still hangs after
all listed unit-test assertions are fixed, the result must identify the exact
remaining hanging test/process with evidence. UI tests are out of scope for this
experiment.

## Design Review

**Reviewer:** Codex-native adversarial subagent (`multi_agent_v1.spawn_agent`,
fresh context, `Ptolemy the 3rd`)

**Verdict:** Approved

**Findings:** None.
