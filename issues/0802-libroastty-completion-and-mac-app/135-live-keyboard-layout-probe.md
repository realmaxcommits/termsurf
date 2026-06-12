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

## Result

**Result:** Pass

Implemented the live keyboard-layout diagnostics path:

- Added public C ABI enum values for Roastty's small keyboard-layout set:
  `unknown`, `us_standard`, and `us_international`.
- Added `roastty_current_keyboard_layout()`, which calls the production
  `input_keyboard::Layout::current()` path and maps the result to the ABI enum.
- Added Rust unit coverage proving the ABI converter follows the current layout
  provider for all three enum variants without claiming to exercise the host
  Carbon/TIS path.
- Added `KeyboardLayoutTests` to the macOS hosted test suite. The test calls the
  normal staticlib ABI, verifies the result is a valid public enum case, then
  independently reads the host layout-source ID through
  `TISCopyCurrentKeyboardLayoutInputSource` plus `kTISPropertyInputSourceID`. On
  this host, the independent layout-source ID was `com.apple.keylayout.US`, and
  the ABI returned `ROASTTY_KEYBOARD_LAYOUT_US_STANDARD`.

Verification:

- `cargo fmt`
- `cargo build -p roastty`
- `cargo test -p roastty keyboard_layout` — 3 unit tests passed; ABI harness
  filter passed with 0 tests
- `cargo test -p roastty key_translation_mods` — 10 unit tests passed; ABI
  harness filter passed with 0 tests
- `cd roastty && macos/build.nu --action test --only-testing RoasttyTests/KeyboardLayoutTests`
  — 1 Swift Testing test passed
- Independent host layout-source check:
  `TISCopyCurrentKeyboardLayoutInputSource` reported `com.apple.keylayout.US`
- `cd roastty && macos/build.nu --action test` — 202 tests in 19 suites passed
- `cargo test -p roastty -- --test-threads=1` — 4,751 unit tests passed, the C
  ABI harness passed, and doc-tests passed
- `cargo fmt --check`
- `git diff --check`
- `prettier --check --prose-wrap always --print-width 80 issues/0802-libroastty-completion-and-mac-app/135-live-keyboard-layout-probe.md issues/0802-libroastty-completion-and-mac-app/README.md`

The full macOS test run still emits pre-existing SwiftLint/Main Thread Checker
and App Intents logs; they are non-failing and unrelated to this experiment. The
C ABI harness still emits the pre-existing enum conversion warnings.

## Conclusion

The production macOS Carbon/TIS layout probe is now validated from the hosted
app-test environment through the normal staticlib ABI, with an independent
same-API oracle for recognized Apple US layouts. This closes the live host-probe
validation gap left by Experiment 131. The remaining native-key work is full
Rust-side `KeymapDarwin` text translation, dead-key/preedit handling, and native
global shortcut registration behavior.

## Completion Review

**Reviewer:** Codex-native adversarial review subagent `Dirac`, fresh context.

**Verdict:** Approved.

**Findings:** None.

The reviewer verified that the result commit had not been made yet and that the
implementation/result changes were still uncommitted on top of plan commit
`cd7f7ec9b45bd`. The reviewer independently ran `cargo fmt --check`,
`git diff --check`, the Prettier check, `cargo build -p roastty`,
`cargo test -p roastty keyboard_layout`,
`cargo test -p roastty key_translation_mods`, and
`cd roastty && macos/build.nu --action test --only-testing RoasttyTests/KeyboardLayoutTests`;
all passed. The reviewer also confirmed the independent host layout-source query
returned `com.apple.keylayout.US`, matching the experiment's `Pass` condition.
