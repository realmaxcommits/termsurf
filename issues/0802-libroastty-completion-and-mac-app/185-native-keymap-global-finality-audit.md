# Experiment 185: Phase G — native keymap and global finality audit

## Description

Close, or precisely fail to close, the last required Phase G checklist item:
native keymaps (`keycodes`, `KeymapDarwin`) plus app-level key handling.

Earlier experiments split and proved most of this surface: key remapping,
`macos-option-as-alt`, keyboard-layout reload, live layout probing,
`KeymapDarwin`, app-owned keymap state, copied-app text scope, hosted preedit
state, dead-key route synthesis, app-key dispatch, global event-tap dispatch,
and event-tap installation state. After Experiments 183 and 184, the README says
the remaining native-key gap is permission-dependent live global shortcut
receipt on hosts where macOS grants Accessibility permission.

This experiment is an audit/proof gate for that final required item. It should
check the native-keymap/global-shortcut roadmap item only if current source
evidence and focused tests prove the native keymap/app-key surface enough for
Issue 802, and if the live-global-shortcut caveat is either directly validated
on this host or explicitly resolved as a host-permission boundary already
covered by dispatch plus installation-state tests. It must not claim that the
optional debug overlay is complete.

## Changes

- `issues/0802-libroastty-completion-and-mac-app/README.md`
  - Link this experiment as `Designed`.
  - After verification, mark it `Pass`, `Partial`, or `Fail`.
  - Check the native-keymap/global-shortcut roadmap item only if the audit
    proves the remaining required native-key scope is complete enough to close.
  - Leave the optional debug overlay unchanged unless a later experiment
    explicitly chooses to implement it.

- `issues/0802-libroastty-completion-and-mac-app/185-native-keymap-global-finality-audit.md`
  - Record source evidence, host-permission evidence if checked, command output,
    test results, result, conclusion, and AI completion review.

- Production code
  - No code change is expected. If the audit finds a real missing behavior,
    record the gap and design a follow-up implementation experiment.

## Verification

Before verification:

- Codex-native adversarial design review approves this experiment.
- Commit the reviewed plan separately from the result.

Source audit:

- Confirm native keycode mapping, key-remap application, option-as-alt
  translation, layout reload, app-owned keymap state, and `KeymapDarwin` are
  present:

  ```bash
  rg -n "NATIVE_TO_KEY|RemapSet|key_remap|roastty_surface_key_translation_mods|macos_option_as_alt|AppKeymap|KeymapDarwin|roastty_app_keyboard_changed|roastty_current_keyboard_layout" \
    roastty/src
  ```

- Confirm copied-app text handling remains AppKit-provided and the raw ABI
  handoff preserves app-provided UTF-8/composing state:

  ```bash
  rg -n "interpretKeyEvents|setMarkedText|insertText|committedPreeditText|withCValue|surface_key_by_value_utf8|roastty_surface_preedit|roastty_surface_ime_point" \
    roastty/src roastty/macos/Sources roastty/macos/Tests roastty/macos/RoasttyUITests
  ```

- Confirm app-level key handling and global event-tap dispatch/installation
  state are wired:

  ```bash
  rg -n "roastty_app_key|roastty_app_has_global_keybinds|GlobalEventTap|handleCapturedEvent|tapFactory|retryScheduler|isInstalled|isRetryPending" \
    roastty/src roastty/macos/Sources roastty/macos/Tests
  ```

Focused tests:

- `cargo test -p roastty key_remap`
- `cargo test -p roastty key_translation_mods`
- `cargo test -p roastty keyboard_layout`
- `cargo test -p roastty keymap_darwin`
- `cargo test -p roastty app_keymap`
- `cargo test -p roastty app_key`
- `cargo test -p roastty surface_key_by_value_utf8_reaches_child_pty`
- `cargo test -p roastty preedit`
- `cargo test -p roastty surface_preedit`
- `cargo test -p roastty --test abi_harness`
- `cd roastty && macos/build.nu --action test --only-testing RoasttyTests/KeyboardLayoutTests`
- `cd roastty && macos/build.nu --action test --only-testing RoasttyTests/SurfaceKeyTextTests`
- `cd roastty && macos/build.nu --action test --only-testing RoasttyTests/SurfaceViewAppKitTests`
- `cd roastty && macos/build.nu --action test --only-testing RoasttyTests/GlobalEventTapTests`

Live-global-shortcut host check:

- Inspect whether this host can support a permission-dependent live global
  shortcut receipt check without changing product behavior:
  - whether the current process/app is Accessibility-trusted;
  - whether existing UI/event-tap tests already include a live receipt selector;
  - whether a focused live receipt test can be run safely with the existing
    harness.
- If permission or harness support is absent, record that as host evidence and
  decide whether the checklist item can still close from the non-permission
  state-machine plus captured-event dispatch proofs. Do not fabricate live
  receipt proof.

Regression and hygiene:

- `cargo fmt --check -p roastty`
- `prettier --check --prose-wrap always --print-width 80 issues/0802-libroastty-completion-and-mac-app/185-native-keymap-global-finality-audit.md issues/0802-libroastty-completion-and-mac-app/README.md`
- `git diff --check`

**Pass** = source audit and focused tests prove the native keymap/app-key
surface listed in the roadmap, hosted macOS tests prove layout/text/preedit and
event-tap dispatch/state, and the remaining permission-dependent live global
shortcut receipt question is either directly validated on this host or
explicitly resolved as a host-permission boundary that should not keep Issue 802
open.

**Partial** = native keymap/app-key behavior is mostly proved, but live global
shortcut receipt still lacks a direct or accepted boundary proof, a focused
hosted test remains Partial in a way that blocks the checklist item, or a
specific native-key behavior remains unproved.

**Fail** = source audit or focused tests contradict the claim that the
native-keymap/global-shortcut roadmap item is complete enough to check.

## Design Review

**Reviewer:** Codex-native adversarial review subagent `Helmholtz the 2nd`,
fresh context.

**Verdict:** Approved.

Findings: None. The reviewer confirmed the README links Experiment 185 as
`Designed`, the experiment has the required sections, the scope is limited to
the final Phase G native-keymap/global-shortcut audit, optional debug overlay
and broader Issue 802 completion are not overclaimed, verification covers native
keymap, app text/preedit, app-key, event-tap, live permission-boundary, hygiene,
and plan/result commit gates, and the pass/partial criteria are honest about
live Accessibility permission and the existing Partial UI-oracle history.

## Result

**Result:** Pass.

This audit found the native keymap/app-key surface complete enough to close the
last required Phase G checklist item. No production code changed.

Source inspection confirmed:

- `roastty/src/input/key.rs` owns `NATIVE_TO_KEY` native keycode mapping.
- `roastty/src/input/key_mods.rs`, `roastty/src/config/mod.rs`, and
  `roastty/src/lib.rs` own `RemapSet`, `key-remap` config, surface remap
  application, `macos-option-as-alt`, and
  `roastty_surface_key_translation_mods`.
- `roastty/src/input/keymap_darwin.rs` owns the Rust `KeymapDarwin` translation
  foundation, while `AppKeymap`, `roastty_app_keyboard_changed`, and
  `roastty_current_keyboard_layout` wire app-owned keymap state, reload, and
  hosted layout diagnostics.
- `SurfaceView_AppKit.swift` keeps copied-app text handling AppKit-provided via
  `interpretKeyEvents`, `setMarkedText`, `insertText`, and
  `committedPreeditText`, while `Roastty.Input.withCValue` and
  `surface_key_by_value_utf8_reaches_child_pty` prove the raw ABI preserves
  app-provided text/composing state.
- `roastty_surface_preedit`, `roastty_surface_ime_point`,
  `SurfaceViewAppKitTests`, and the preedit Rust tests cover hosted marked-text
  and IME geometry.
- `roastty_app_key`, `roastty_app_has_global_keybinds`, `GlobalEventTap`,
  `globalEventTapHandleKeyEvent`, the event-tap dependency seams, and
  `GlobalEventTapTests` cover app-key dispatch, captured-event dispatch,
  installation success, idempotent enable, failure retry, retry success, and
  disable.

Focused verification passed:

- `cargo test -p roastty key_remap` — 18 passed.
- `cargo test -p roastty key_translation_mods` — 10 passed.
- `cargo test -p roastty keyboard_layout` — 4 passed.
- `cargo test -p roastty keymap_darwin` — 5 passed, including the host smoke
  translation test.
- `cargo test -p roastty app_keymap` — 4 passed.
- `cargo test -p roastty app_key` — 31 passed.
- `cargo test -p roastty surface_key_by_value_utf8_reaches_child_pty` — 1
  passed.
- `cargo test -p roastty preedit` — 46 passed.
- `cargo test -p roastty surface_preedit` — 10 passed.
- `cargo test -p roastty --test abi_harness` — 1 passed, with the existing 10
  enum-conversion warnings and `[unknown](scope): message`.
- `cd roastty && macos/build.nu --action test --only-testing RoasttyTests/KeyboardLayoutTests`
  — 1 Swift Testing test passed and Xcode reported `** TEST SUCCEEDED **`.
- `cd roastty && macos/build.nu --action test --only-testing RoasttyTests/SurfaceKeyTextTests`
  — 2 Swift Testing tests passed and Xcode reported `** TEST SUCCEEDED **`.
- `cd roastty && macos/build.nu --action test --only-testing RoasttyTests/SurfaceViewAppKitTests`
  — 5 Swift Testing tests passed and Xcode reported `** TEST SUCCEEDED **`.
- `cd roastty && macos/build.nu --action test --only-testing RoasttyTests/GlobalEventTapTests`
  — 10 Swift Testing tests passed and Xcode reported `** TEST SUCCEEDED **`.

Live-global-shortcut boundary check:

- `swift -e 'import ApplicationServices; print(AXIsProcessTrusted())'` returned
  `true` on this host.
- `rg` found no existing live global-shortcut UI receipt selector. Existing
  coverage is the hosted, non-permission-sensitive path from Experiments 136 and
  163: captured `CGEvent` dispatch is proven, and event-tap enable/retry/disable
  state is proven with fake dependencies while production defaults still use
  `CGEvent.tapCreate`, `CFRunLoopAddSource`, and `CGEvent.tapEnable`.

The audit resolves the live receipt caveat as an OS/TCC delivery boundary rather
than an unimplemented Issue 802 requirement. Direct live receipt would require a
new permission-sensitive UI selector and global event synthesis, but the
Roastty-owned behavior on both sides of that boundary is already covered:
request/maintain the tap, translate captured events, dispatch through
`roastty_app_key`, and suppress handled inactive-app global captures.

Regression and hygiene checks passed:

- `cargo fmt --check -p roastty`
- `prettier --check --prose-wrap always --print-width 80 issues/0802-libroastty-completion-and-mac-app/185-native-keymap-global-finality-audit.md issues/0802-libroastty-completion-and-mac-app/README.md`
- `git diff --check`

## Completion Review

**Reviewer:** Codex-native adversarial review subagent `Copernicus the 2nd`,
fresh context.

**Verdict:** Approved.

Findings: None.

The reviewer independently verified that only the experiment doc and issue
README were modified, the diff from plan commit `b705eda4393d9` was
documentation-only, no result commit had been made, the source-audit `rg`
commands found the expected native keymap, AppKit text/preedit, app-key, and
global event-tap evidence, and `AXIsProcessTrusted()` returned `true`.

The reviewer reran and confirmed:

- `cargo fmt --check -p roastty`
- `prettier --check --prose-wrap always --print-width 80 issues/0802-libroastty-completion-and-mac-app/185-native-keymap-global-finality-audit.md issues/0802-libroastty-completion-and-mac-app/README.md`
- `git diff --check`
- `cargo test -p roastty key_remap`
- `cargo test -p roastty keymap_darwin`
- `cargo test -p roastty app_key`
- `cargo test -p roastty preedit`
- `cargo test -p roastty --test abi_harness`
- `cd roastty && macos/build.nu --action test --only-testing RoasttyTests/GlobalEventTapTests`

The reviewer did not independently rerun the remaining claimed focused tests
(`key_translation_mods`, `keyboard_layout`, `app_keymap`,
`surface_key_by_value_utf8_reaches_child_pty`, `surface_preedit`) or the other
hosted Swift suites (`KeyboardLayoutTests`, `SurfaceKeyTextTests`,
`SurfaceViewAppKitTests`).

## Conclusion

The native-keymap/global-shortcut checklist item is complete for Issue 802.
Native keycodes, key remapping, option-as-alt translation, keyboard layout
reload, live host layout probing, `KeymapDarwin`, app-owned keymap state,
AppKit-provided text handoff, hosted preedit/IME geometry, app-level key
handling, global event-tap dispatch, and event-tap installation state all have
focused source and test evidence.

The only unchecked Issue 802 roadmap line now is the optional debug overlay. It
is explicitly optional and should not block closing the required Issue 802
completion work unless we choose to implement it as a separate enhancement.
