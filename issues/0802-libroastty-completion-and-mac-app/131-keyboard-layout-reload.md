# Experiment 131: Phase G — keyboard layout reload

## Description

Finish the `macos-option-as-alt` auto-detection path left partial by
Experiment 130.

Experiment 130 wired explicit `macos-option-as-alt` config values and made
`roastty_surface_key_translation_mods` fall back to `App.keyboard_layout`, but
`roastty_app_keyboard_changed` still leaves that layout at `Layout::Unknown`
outside tests. Upstream `ghostty_app_keyboard_changed` reloads the app keymap,
and `keyboardLayout()` maps the current Carbon input-source ID to the small
`input.keyboard.Layout` enum before deriving the option-as-alt default.

This experiment ports only that layout-ID probe and reload behavior. It does not
port full `KeymapDarwin` text translation, dead-key composition, or native
global shortcut registration.

## Changes

- `roastty/src/input/keyboard.rs`
  - Add a public-to-crate `current()` helper that returns the current keyboard
    `Layout`.
  - On macOS, call Carbon/TextInputSources to read the current keyboard layout
    input-source ID, convert it to UTF-8, and feed it through
    `Layout::map_apple_id`.
  - On non-macOS, return `Layout::Unknown`.
  - Keep unknown/unreadable IDs as `Layout::Unknown`, matching upstream's
    fallback behavior.
  - Add unit coverage for the mapping/helper boundary. Avoid making CI depend on
    the host's real keyboard layout; host-probe tests should assert only that
    the call is safe and returns one of the known enum variants.
- `roastty/src/lib.rs`
  - Initialize `App.keyboard_layout` from `input_keyboard::Layout::current()`
    instead of always `Unknown`.
  - Change `roastty_app_keyboard_changed(app)` to refresh `app.keyboard_layout`
    from `Layout::current()`.
  - Keep explicit `macos-option-as-alt` surface config precedence unchanged.
  - Add ABI-level tests proving:
    - app creation uses the current-layout provider;
    - `roastty_app_keyboard_changed` refreshes the layout used by
      `roastty_surface_key_translation_mods`;
    - explicit surface config still overrides the refreshed app layout.

Out of scope:

- Rust-side `UCKeyTranslate` / full `KeymapDarwin` text translation.
- Dead-key/preedit composition.
- Native platform global shortcut registration.
- Changing copied Swift app key-event delivery.
- Reworking the command-palette UI automation gate.

## Verification

- Run formatting:
  - `cargo fmt`
  - `prettier --write --prose-wrap always --print-width 80 issues/0802-libroastty-completion-and-mac-app/131-keyboard-layout-reload.md issues/0802-libroastty-completion-and-mac-app/README.md`
- Run targeted tests:
  - `cargo test -p roastty keyboard_layout`
  - `cargo test -p roastty key_translation_mods`
  - `cargo test -p roastty option_as_alt`
- Run full Roastty tests:
  - `cargo test -p roastty -- --test-threads=1`
- Run `cargo fmt --check`.
- Run `git diff --check`.
- Run the same Prettier command with `--check`.

**Pass** = app creation and `roastty_app_keyboard_changed` refresh the layout
used by `roastty_surface_key_translation_mods`, the macOS current-layout probe
is safe, and explicit config values still take precedence.

**Partial** = deterministic tests prove reload plumbing, but the real macOS
Carbon/TIS probe cannot be safely linked or executed in this repo without a
larger keymap port.

**Fail** = layout reload cannot be separated from the full `KeymapDarwin`
translation port.

## Design Review

**Reviewer:** Codex-native adversarial review subagent, fresh context.

**Verdict:** Approved.

**Findings:** No required, optional, or nit findings. The reviewer approved the
scope, upstream fidelity, README linkage, verification plan, and host-layout
flakiness avoidance.

## Result

**Result:** Partial

Implemented the reload plumbing and production macOS layout probe:

- Added `Layout::current()` in `roastty/src/input/keyboard.rs`.
- On macOS production builds, `Layout::current()` calls Carbon
  `TISCopyCurrentKeyboardLayoutInputSource` /
  `TISGetInputSourceProperty(kTISPropertyInputSourceID)`, copies the returned
  `CFString` into Rust, releases the retained input source, and maps recognized
  Apple IDs through `Layout::map_apple_id`.
- On non-macOS, and in tests without an explicit override, the current layout is
  `Layout::Unknown`.
- Added a thread-local test override so deterministic unit tests can prove
  layout-dependent behavior without depending on the host keyboard layout or
  calling Carbon from arbitrary test threads.
- Initialized `App.keyboard_layout` from `Layout::current()`.
- Changed `roastty_app_keyboard_changed(app)` to refresh the stored app layout.
- Added ABI-level tests proving app creation and `roastty_app_keyboard_changed`
  update the layout used by `roastty_surface_key_translation_mods`, and explicit
  `macos-option-as-alt` surface config still takes precedence.

Verification:

- `cargo fmt`
- `cargo build -p roastty`
- `cargo test -p roastty keyboard_layout`
- `cargo test -p roastty key_translation_mods`
- `cargo test -p roastty option_as_alt`
- `cargo test -p roastty -- --test-threads=1`
- `cargo fmt --check`
- `git diff --check`
- `prettier --check --prose-wrap always --print-width 80 issues/0802-libroastty-completion-and-mac-app/131-keyboard-layout-reload.md issues/0802-libroastty-completion-and-mac-app/README.md`

All commands passed. The full Roastty run passed 4,740 unit tests, the C ABI
harness, and doc-tests; the ABI harness still emits the pre-existing enum
conversion warnings.

This is Partial rather than Pass because the deterministic unit-test build does
not execute the real Carbon/TIS host-layout probe. The production build compiles
and links that path, and the reload behavior is covered through the test
override, but a live app or purpose-built main-thread probe should validate the
real host layout before declaring the native keymap/layout gap fully closed.

## Conclusion

Roastty now has the same app-level reload shape as upstream for the
option-as-alt default: app creation captures a layout, keyboard-change
notifications refresh it, and surface translation uses explicit config before
falling back to the app layout. The remaining native key work is no longer
layout-state plumbing; it is live validation of the Carbon/TIS probe, full
`KeymapDarwin` text translation, dead-key/preedit handling, and native global
shortcut registration.

## Completion Review

**Reviewer:** Codex-native adversarial review subagent, fresh context.

**Verdict:** Approved.

**Findings:** No findings. The reviewer verified the result commit had not been
made yet, the working-tree changes were limited to the expected four paths,
`cargo fmt --check`, `git diff --check`, `prettier --check`,
`cargo build -p roastty`, `cargo test -p roastty keyboard_layout`,
`cargo test -p roastty key_translation_mods`, and
`cargo test -p roastty option_as_alt` passed. The reviewer also confirmed the
README marks Experiment 131 as `Partial`, the experiment file has `Result` and
`Conclusion`, app creation and `roastty_app_keyboard_changed` call
`Layout::current()`, the test override is `#[cfg(test)]`, and the macOS
Carbon/TIS path is present only in the production implementation path.
