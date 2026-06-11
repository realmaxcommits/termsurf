# Experiment 135: Phase G — live keyboard layout probe

## Description

Validate the production macOS keyboard-layout probe left partial by
Experiment 131.

Experiment 131 added `input_keyboard::Layout::current()` and made production
macOS builds call Carbon/TextInputSources, but Rust unit tests intentionally use
a `#[cfg(test)]` layout override instead of querying the host input source. The
result was correct plumbing with one remaining weak spot: no hosted test proved
that the normal staticlib path can execute the real Carbon/TIS probe from the
macOS app test environment.

This experiment adds a narrow diagnostics ABI that exposes only Roastty's small
layout enum, then adds an XCTest that calls that ABI from the copied macOS app.
It does not port full `KeymapDarwin` text translation, dead-key translation, or
new global shortcut behavior.

## Changes

- `roastty/include/roastty.h`
  - Add a small `roastty_keyboard_layout_e` enum with stable values for
    `unknown`, `us_standard`, and `us_international`.
  - Declare `roastty_current_keyboard_layout()`.
- `roastty/src/lib.rs`
  - Add ABI constants matching `roastty_keyboard_layout_e`.
  - Add a local converter from `input_keyboard::Layout` to the ABI enum.
  - Implement `roastty_current_keyboard_layout()` by calling
    `input_keyboard::Layout::current()`.
  - Add Rust unit coverage for the enum converter. The unit test should still
    use the existing test override and should not claim to exercise the host
    Carbon/TIS path.
- `roastty/macos/Tests/Roastty/KeyboardLayoutTests.swift`
  - Add a hosted XCTest that calls `roastty_current_keyboard_layout()` from the
    macOS staticlib.
  - Assert the returned value is one of the public enum cases.
  - Add a private test helper that independently reads the host layout ID using
    the same macOS API as Rust and upstream `KeymapDarwin`:
    `TISCopyCurrentKeyboardLayoutInputSource` plus `kTISPropertyInputSourceID`.
  - If that independent layout-source ID is recognized as an Apple US layout,
    assert the Rust ABI returns the matching enum value.
  - If the layout-source ID is unreadable or unrecognized, keep the test
    non-flaky by asserting only that the ABI call is safe and returns a valid
    public enum value; record the experiment result as `Partial`, not `Pass`,
    because that branch cannot distinguish a correct unknown mapping from a
    constant fallback.
- `issues/0802-libroastty-completion-and-mac-app/README.md`
  - Link this experiment as `Designed`.
  - After implementation, narrow the native-key note so live host-probe
    validation is no longer listed as remaining if the hosted XCTest passes.

Out of scope:

- Full Rust-side `UCKeyTranslate` / `KeymapDarwin` text translation.
- Dead-key or IME preedit generation changes.
- Native platform global shortcut registration changes.
- Changing copied Swift key-event delivery beyond adding the focused XCTest.
- Closing Phase G's native-key item.

## Verification

- Run formatting:
  - `cargo fmt`
  - `prettier --write --prose-wrap always --print-width 80 issues/0802-libroastty-completion-and-mac-app/135-live-keyboard-layout-probe.md issues/0802-libroastty-completion-and-mac-app/README.md`
- Run targeted Rust tests:
  - `cargo build -p roastty`
  - `cargo test -p roastty keyboard_layout`
  - `cargo test -p roastty key_translation_mods`
- Run the targeted macOS hosted test:
  - `cd roastty && macos/build.nu --action test --only-testing RoasttyTests/KeyboardLayoutTests`
- Run broader macOS coverage:
  - `cd roastty && macos/build.nu --action test`
- Run full Roastty tests:
  - `cargo test -p roastty -- --test-threads=1`
- Run checks:
  - `cargo fmt --check`
  - `git diff --check`
  - `prettier --check --prose-wrap always --print-width 80 issues/0802-libroastty-completion-and-mac-app/135-live-keyboard-layout-probe.md issues/0802-libroastty-completion-and-mac-app/README.md`

**Pass** = the macOS hosted XCTest calls the normal staticlib
`roastty_current_keyboard_layout()` successfully, independently reads a
recognized host Apple US layout ID through
`TISCopyCurrentKeyboardLayoutInputSource`, verifies the matching ABI enum, and
existing layout-dependent Rust behavior still passes.

**Partial** = the diagnostics ABI and Rust tests work, but the macOS hosted test
cannot execute reliably in the local app-test environment, or the independent
host layout-source ID is unreadable/unrecognized so the live probe cannot prove
more than safe execution plus valid enum output.

**Fail** = the production Carbon/TIS probe cannot be safely exposed or executed
without first porting full `KeymapDarwin`.

## Design Review

**Reviewer:** Codex-native adversarial review subagent `Ohm`, fresh context.

**Verdict:** Approved after fixes.

**Findings and fixes:**

- Required: the first design used `KeyboardLayout.id`, which reads
  `TISCopyCurrentKeyboardInputSource`, while Rust and upstream `KeymapDarwin`
  use `TISCopyCurrentKeyboardLayoutInputSource`. Fixed by requiring the hosted
  XCTest to use a private helper that reads the layout-source ID with
  `TISCopyCurrentKeyboardLayoutInputSource` plus `kTISPropertyInputSourceID`.
- Required: the first design overclaimed `Pass` for unknown host layouts because
  a constant `.unknown` fallback could satisfy that branch. Fixed by making
  `Pass` require an independently read, recognized Apple US layout ID and a
  matching ABI enum; unreadable or unrecognized host layouts now produce
  `Partial`.
- Optional: verification omitted `cargo build -p roastty`. Fixed by adding it to
  the targeted Rust checks.

The re-review approved the fixes and reported no new required findings.
