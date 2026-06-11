# Experiment 130: Phase G — key translation option-as-alt

## Description

Wire the macOS key-translation modifier ABI to Roastty's config and keyboard
layout state.

The Phase G checklist still lists native keymaps/global shortcuts as remaining
work. A fresh read shows the physical native-keycode table is already present:
`roastty/src/input/key.rs` has `NATIVE_TO_KEY`, and the copied Swift app sends
macOS native `NSEvent.keyCode` values through `roastty_surface_key`. The
remaining divergence in the active ABI path is narrower: upstream
`ghostty_surface_key_translation_mods` applies `macos-option-as-alt` from the
surface config, falling back to keyboard-layout detection when the config is
unset, while Roastty currently hardcodes `OptionAsAlt::False`.

This experiment adds the smallest state and tests needed for the
`roastty_surface_key_translation_mods` behavior to match upstream for configured
and detected option-as-alt policy. Full Carbon `KeymapDarwin` text translation,
dead-key composition, and OS-level global shortcut registration remain separate
work.

## Changes

- `roastty/src/config/mod.rs`
  - Add the upstream `macos-option-as-alt` config field if it is still absent.
  - Parse and format `false`, `true`, `left`, and `right` using Roastty's
    existing `OptionAsAlt` enum semantics.
  - Preserve upstream's unset/auto behavior: an explicit value overrides
    keyboard-layout detection, while the empty/reset value returns to auto.
  - Add config parse/format/reset tests.
- `roastty/src/lib.rs`
  - Store the finalized `macos-option-as-alt` value on app/surface state or
    otherwise make it reachable from `roastty_surface_key_translation_mods`.
  - Track a minimal keyboard-layout value on `App`, defaulting to
    `Layout::Unknown`.
  - Make `roastty_app_keyboard_changed` refresh that layout state when the
    platform can provide it. If Carbon/TIS lookup is too large for this
    experiment, keep reload as a documented no-op but add a test-only setter or
    helper so detected-layout fallback is still covered deterministically.
  - Change `roastty_surface_key_translation_mods(surface, mods)` so it:
    - sanitizes unknown modifier bits for invalid or detached surfaces;
    - uses the explicit surface/app `macos-option-as-alt` value when configured;
    - otherwise uses the app's detected keyboard layout
      (`UsStandard`/`UsInternational` => option-as-alt true, `Unknown` =>
      false);
    - preserves side-specific `left`/`right` behavior already implemented in
      `key_mods::Mods::translation`.
  - Keep `roastty_surface_key` physical-key handling unchanged.
- `roastty/src/lib.rs` tests
  - Add ABI-level tests for `roastty_surface_key_translation_mods` covering:
    - explicit `macos-option-as-alt = false`;
    - explicit `true`;
    - explicit `left`;
    - explicit `right`;
    - unset config with detected US layout;
    - unset config with unknown layout;
    - invalid/detached surface fallback.
- `issues/0802-libroastty-completion-and-mac-app/README.md`
  - If the experiment passes, update the Phase G notes so native keymaps are no
    longer described as wholly missing; remaining native/global work should
    narrow to full Carbon `KeymapDarwin` text translation, keyboard-layout
    reload fidelity if not completed here, and native global shortcut
    registration.

Out of scope:

- Replacing Swift `NSEvent.characters` text delivery with Rust-side
  `KeymapDarwin` / `UCKeyTranslate`.
- Dead-key composition and preedit text generation.
- OS-level native global shortcut registration.
- Reworking app-key sequence/table ownership.
- Command-palette UI automation.

## Verification

- Run formatting:
  - `cargo fmt`
  - `prettier --write --prose-wrap always --print-width 80 issues/0802-libroastty-completion-and-mac-app/130-key-translation-option-as-alt.md issues/0802-libroastty-completion-and-mac-app/README.md`
- Run targeted tests:
  - `cargo test -p roastty option_as_alt`
  - `cargo test -p roastty key_translation_mods`
  - `cargo test -p roastty config_macos_option_as_alt`
  - `cargo test -p roastty app_key`
  - `cargo test -p roastty surface_key`
- Run full Roastty tests:
  - `cargo test -p roastty -- --test-threads=1`
- Run `cargo fmt --check`.
- Run `git diff --check`.
- Run the same Prettier command with `--check`.

**Pass** = `roastty_surface_key_translation_mods` matches upstream option-as-alt
behavior for explicit config values and detected/unknown keyboard layout
fallback, without changing physical keycode dispatch or app/surface key binding
behavior.

**Partial** = explicit config values work, but true keyboard-layout reload or
auto detection needs a follow-up experiment.

**Fail** = correct option-as-alt translation requires first porting the full
Carbon `KeymapDarwin` implementation.

## Design Review

**Reviewer:** Codex-native adversarial review subagent, fresh context.

**Verdict:** Approved.

**Findings:** No required findings. The reviewer confirmed that the README links
Experiment 130 as `Designed`, Roastty currently hardcodes `OptionAsAlt::False`
in `roastty_surface_key_translation_mods`, upstream uses config-or-layout
fallback, `macos-option-as-alt` is absent from Roastty config but present
upstream, the minimal layout mapping already exists, and the verification plan
includes the required formatting, targeted tests, full Roastty test run, and
diff checks.

## Result

**Result:** Partial

Implemented the explicit config and ABI translation path:

- Added `macos-option-as-alt` to Roastty config with upstream keywords `false`,
  `true`, `left`, and `right`, including parse/format/reset diagnostics
  coverage.
- Stored the finalized option-as-alt value on surfaces and refreshed it from
  surface config updates.
- Added minimal app keyboard layout state defaulting to `Layout::Unknown`.
- Updated `roastty_surface_key_translation_mods` to use explicit surface config
  first, then the app layout fallback, while preserving the existing unknown-bit
  sanitizer for invalid or detached surfaces.
- Left `roastty_app_keyboard_changed` as a documented no-op because the real
  Carbon/TIS reload path is larger than this experiment.

Verification:

- `cargo fmt`
- `cargo test -p roastty option_as_alt`
- `cargo test -p roastty key_translation_mods`
- `cargo test -p roastty config_macos_option_as_alt`
- `cargo test -p roastty app_key`
- `cargo test -p roastty surface_key`
- `cargo test -p roastty -- --test-threads=1`
- `cargo fmt --check`
- `git diff --check`

All commands passed. The full Roastty run passed 4,736 unit tests, the C ABI
harness, and doc-tests; the ABI harness still emits the pre-existing enum
conversion warnings.

## Conclusion

The modifier-translation ABI now matches upstream for explicit
`macos-option-as-alt` policy and for deterministic layout fallback in tests.
This narrows the remaining native key work to real keyboard layout reload via
Carbon/TIS, full `KeymapDarwin` text translation/dead-key handling, and native
global shortcut registration.

## Completion Review

**Reviewer:** Codex-native adversarial review subagent, fresh context.

**Verdict:** Approved.

**Findings:** No required findings. The reviewer verified that the result commit
had not been made yet, `cargo fmt --check`, `git diff --check`,
`prettier --check`, `cargo test -p roastty key_translation_mods`, and
`cargo test -p roastty config_macos_option_as_alt` passed. The reviewer also
confirmed the README status matches the `Partial` result, the experiment file
has `Result` and `Conclusion`, and the working-tree diff does not show
unrequested scope expansion or a correctness issue in option-as-alt parsing,
formatting, or modifier translation behavior.
