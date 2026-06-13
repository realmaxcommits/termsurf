# Experiment 184: Phase G — trigger-prefix finality audit

## Description

Close, or precisely fail to close, the Phase G trigger-prefix checklist item by
auditing the current configured keybinding prefix implementation end to end.

Prior experiments landed the work in slices: parser/storage/query flags
(`global:`, `all:`, `unconsumed:`, `performable:`), surface-local consumption,
focused and global app-key dispatch, surface-path `all:` / `global:` fanout,
macOS event-tap captured-event dispatch, and event-tap installation state. The
README item still appears open because those proofs have not been gathered into
one finality gate.

This is an audit/proof experiment. It should check the trigger-prefix roadmap
item only if source inspection and focused tests prove the implemented prefix
surface is complete enough. It must not claim native keymap correctness,
permission-granted live global keystroke receipt, or broader Issue 802
completion; those remain represented by the separate native-keymap/global
shortcut item.

## Changes

- `issues/0802-libroastty-completion-and-mac-app/README.md`
  - Link this experiment as `Designed`.
  - After verification, mark it `Pass`, `Partial`, or `Fail`.
  - Check the trigger-prefix roadmap item only if the audit proves configured
    prefix parsing/storage/query, surface consumption, app-key global/focused
    dispatch, surface `all:` / `global:` routing, hosted captured-event
    dispatch, and non-permission live tap state are all covered.
  - Leave the native-keymap/global-shortcut roadmap item unchecked unless a
    later experiment specifically proves permission-dependent live global
    shortcut receipt and native keymap behavior.

- `issues/0802-libroastty-completion-and-mac-app/184-trigger-prefix-finality-audit.md`
  - Record source evidence, command output, test results, result, conclusion,
    and AI completion review.

- Production code
  - No code change is expected. If the audit finds a real missing behavior,
    record the gap and design a follow-up implementation experiment.

## Verification

Before verification:

- Codex-native adversarial design review approves this experiment.
- Commit the reviewed plan separately from the result.

Source audit:

- Confirm configured prefix flag constants, parser/storage, configured binding
  match flags, app global-key predicate, and config trigger/query paths exist:

  ```bash
  rg -n "ROASTTY_KEYBIND_FLAG_|parse_config_keybind_flags|ConfigKeybind|ConfiguredBindingMatch|has_global_keybinds|roastty_app_has_global_keybinds|roastty_config_trigger" \
    roastty/src/lib.rs
  ```

- Confirm the runtime surface and app paths consume and dispatch prefix-marked
  bindings:

  ```bash
  rg -n "dispatch_configured_binding|ROASTTY_KEYBIND_FLAG_GLOBAL|ROASTTY_KEYBIND_FLAG_ALL|ROASTTY_KEYBIND_FLAG_PERFORMABLE|ROASTTY_KEYBIND_FLAG_CONSUMED|roastty_app_key|perform_app_key" \
    roastty/src/lib.rs
  ```

- Confirm macOS global event-tap dispatch and installation state remain hosted
  testable without requiring Accessibility permission:

  ```bash
  rg -n "GlobalEventTap|handleCapturedEvent|tapFactory|retryScheduler|isInstalled|isRetryPending|roastty_app_has_global_keybinds|roastty_app_key" \
    roastty/macos/Sources roastty/macos/Tests
  ```

Focused tests:

- `cargo test -p roastty keybind`
- `cargo test -p roastty parse_config_keybind`
- `cargo test -p roastty config_trigger_ -- --test-threads=1`
- `cargo test -p roastty app_has_global_keybinds`
- `cargo test -p roastty app_key`
- `cargo test -p roastty app_key_global`
- `cargo test -p roastty surface_key_configured`
- `cargo test -p roastty surface_key_configured_global_all`
- `cargo test -p roastty surface_key_all`
- `cargo test -p roastty --test abi_harness`
- `cd roastty && macos/build.nu --action test --only-testing RoasttyTests/GlobalEventTapTests`

Regression and hygiene:

- `cargo fmt --check -p roastty`
- `prettier --check --prose-wrap always --print-width 80 issues/0802-libroastty-completion-and-mac-app/184-trigger-prefix-finality-audit.md issues/0802-libroastty-completion-and-mac-app/README.md`
- `git diff --check`

**Pass** = source audit proves the configured trigger-prefix storage/runtime
paths are wired, all focused tests pass, hosted macOS event-tap tests pass, and
the trigger-prefix roadmap item can be checked while leaving the
native-keymap/global-shortcut item open.

**Partial** = most prefix behavior is proved, but a specific parser, query,
surface, app-key, or event-tap state behavior remains unproved or stale. Record
the exact missing proof or implementation gap.

**Fail** = source audit or focused tests contradict the claim that the
trigger-prefix roadmap item is complete enough to check.

## Design Review

**Reviewer:** Codex-native adversarial review subagent `McClintock the 2nd`,
fresh context.

**Verdict:** Approved.

Findings: None. The reviewer confirmed the README links Experiment 184 as
`Designed`, the experiment has the required sections, the scope is limited to
the trigger-prefix finality audit, overclaims are explicitly excluded, the
verification commands cover parser/storage/query flags, surface behavior,
app-key dispatch, routing, and non-permission event-tap paths, hygiene checks
are present, and the plan/result commit separation is stated.

## Result

**Result:** Pass.

This audit found the configured trigger-prefix surface wired broadly enough to
close the Phase G trigger-prefix checklist item while leaving the native
keymap/global-shortcut item open.

Source inspection confirmed:

- `roastty/src/lib.rs` defines the configured prefix flag byte:
  `ROASTTY_KEYBIND_FLAG_CONSUMED`, `ROASTTY_KEYBIND_FLAG_ALL`,
  `ROASTTY_KEYBIND_FLAG_GLOBAL`, and `ROASTTY_KEYBIND_FLAG_PERFORMABLE`.
- `parse_config_keybind_flags`, `ConfigKeybind`, `ConfiguredBindingMatch`,
  `Config::store_keybind_entry`, `ConfigKeybindSet::has_global_keybinds`,
  `roastty_app_has_global_keybinds`, and `roastty_config_trigger` carry the
  parsed prefix metadata through config storage and query paths.
- `Surface::dispatch_configured_binding` uses the same flags for surface
  consumption, `performable:` fallthrough, and `all:` / `global:` app-wide
  routing.
- `roastty_app_key`, `perform_app_key_app_action`, and
  `perform_app_key_surface_action` cover global app-key dispatch plus focused
  app-scoped non-global dispatch.
- `GlobalEventTap` and `GlobalEventTapTests` keep captured-event dispatch,
  event-tap creation, retry scheduling, installed state, and pending-retry state
  testable without requiring Accessibility permission.

Focused verification passed:

- Initial `cargo test -p roastty keybind` failed under the default parallel test
  runner: `config_c_abi_cli_config_args_preserve_keybind_parsing` observed one
  diagnostic instead of zero, and the remaining failures were poisoned shared
  test state. The failing test passed in isolation with `--test-threads=1`.
- `cargo test -p roastty keybind -- --test-threads=1` — 48 passed.
- `cargo test -p roastty parse_config_keybind` — 23 passed.
- `cargo test -p roastty config_trigger_ -- --test-threads=1` — 12 passed.
- `cargo test -p roastty app_has_global_keybinds` — 2 passed.
- `cargo test -p roastty app_key` — 31 passed.
- `cargo test -p roastty app_key_global` — 11 passed.
- `cargo test -p roastty surface_key_configured` — 11 passed.
- `cargo test -p roastty surface_key_configured_global_all` — 2 passed.
- `cargo test -p roastty surface_key_all` — 6 passed.
- `cargo test -p roastty --test abi_harness` — 1 passed, with the existing 10
  enum-conversion warnings and `[unknown](scope): message`.
- `cd roastty && macos/build.nu --action test --only-testing RoasttyTests/GlobalEventTapTests`
  — 10 Swift Testing tests in 1 suite passed; Xcode also reported the selected
  XCTest bundle with 0 XCTest tests and `** TEST SUCCEEDED **`.

Regression and hygiene checks passed:

- `cargo fmt --check -p roastty`
- `prettier --check --prose-wrap always --print-width 80 issues/0802-libroastty-completion-and-mac-app/184-trigger-prefix-finality-audit.md issues/0802-libroastty-completion-and-mac-app/README.md`
- `git diff --check`

No production code changed. The trigger-prefix roadmap checkbox is now justified
for configured parser/storage/query flags, surface consumption, app-key
global/focused dispatch, surface `all:` / `global:` routing, hosted
captured-event dispatch, and non-permission event-tap state. Permission-granted
live global keystroke receipt and native keymap behavior remain tracked by the
separate native-keymap/global-shortcut item.

## Completion Review

**Reviewer:** Codex-native adversarial review subagent `Nash the 2nd`, fresh
context.

**Verdict:** Approved.

Findings: None.

The reviewer independently verified that only the two expected markdown files
are modified, the diff from plan commit `27cb8f808bc98` has no production-code
changes, the three source-audit `rg` commands find the expected Rust and macOS
surfaces, and the result commit had not been made.

The reviewer also reran and confirmed:

- `cargo fmt --check -p roastty`
- `prettier --check --prose-wrap always --print-width 80 issues/0802-libroastty-completion-and-mac-app/184-trigger-prefix-finality-audit.md issues/0802-libroastty-completion-and-mac-app/README.md`
- `git diff --check`
- `cargo test -p roastty keybind -- --test-threads=1` — 48 passed.
- `cargo test -p roastty app_key_global` — 11 passed.
- `cargo test -p roastty surface_key_configured` — 11 passed.
- `cargo test -p roastty --test abi_harness` — 1 passed, with the recorded
  existing warnings.
- `cd roastty && macos/build.nu --action test --only-testing RoasttyTests/GlobalEventTapTests`
  — 10 Swift Testing tests passed and Xcode reported `** TEST SUCCEEDED **`.

## Conclusion

The trigger-prefix item was stale rather than missing implementation. The
current code and tests prove the configured prefix behavior from config parsing
through surface/app dispatch and the copied macOS app's testable event-tap
state. The remaining required Phase G work is the native keymap/global-shortcut
item, specifically the host-permission live global-key receipt caveat and native
keymap evidence; the optional debug overlay remains optional.
