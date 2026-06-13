# Experiment 165: Phase F — macOS tail config

## Description

Remove the four remaining macOS scalar keys from the Phase F public-config tail:
`macos-dock-drop-behavior`, `macos-auto-secure-input`,
`macos-secure-input-indication`, and `macos-applescript`.

These are bounded parser/formatter fields. They should match pinned upstream
Ghostty's `Config.zig` defaults and keywords, but their runtime app behavior
dock drop routing, Secure Input heuristics/indicator, and AppleScript handling
remains copied-app/platform work outside this experiment.

## Changes

- `roastty/src/config/mod.rs`
  - Add a `MacOSDockDropBehavior` enum with upstream keywords `new-tab` and
    `new-window`, defaulting to `new-tab`.
  - Add `Config` fields for the enum plus the three bools:
    `macos_auto_secure_input`, `macos_secure_input_indication`, and
    `macos_applescript`, all defaulting to `true`.
  - Format `macos-dock-drop-behavior` in upstream order immediately after
    `macos-titlebar-proxy-icon` and before `macos-option-as-alt`.
  - Format `macos-auto-secure-input`, `macos-secure-input-indication`, and
    `macos-applescript` in upstream order after `macos-hidden` and before
    `macos-icon`.
  - Route `Config::set` for all four keys, including empty-value resets,
    `macos-dock-drop-behavior` missing-value diagnostics, bare-bool `true`
    handling, and invalid enum/bool diagnostics.
  - Preserve upstream's compatibility alias `macos-dock-drop-behavior = window`
    as `new-window`, matching `compatMacOSDockDropBehavior`.
  - Update config field-order tests and add focused parse/format/reset/load
    tests for the new keys.

- `issues/0802-libroastty-completion-and-mac-app/README.md`
  - Mark Experiment 165 as `Designed`.
  - After result, update the Phase F remaining-public-options count from 28 to
    24 and remove the remaining `macos-*` scalar wording if this passes.

## Verification

Before implementation:

- Codex-native adversarial design review approves this experiment.
- Commit the reviewed plan separately from the result.

After implementation:

- `cargo test -p roastty macos_tail_config`
- `cargo test -p roastty config_format_config_emits_fields_in_upstream_order`
- `cargo test -p roastty`
- `cargo fmt --check -p roastty`
- `prettier --check --prose-wrap always --print-width 80 issues/0802-libroastty-completion-and-mac-app/165-macos-tail-config.md issues/0802-libroastty-completion-and-mac-app/README.md`
- `git diff --check`

**Pass** = all four keys parse, format, reset, load, and report diagnostics with
upstream defaults/order/keywords, including the `window` compatibility alias,
and the full roastty test suite passes.

**Partial** = the direct parser/formatter fields land, but compatibility,
ordering, load/replay behavior, or full-suite verification remains incomplete.

**Fail** = the fields cannot be added without conflicting with existing config
storage, formatting, or copied-app expectations.

## Design Review

**Reviewer:** Codex-native adversarial review subagent `Dirac`, fresh context.

**Verdict:** Approved after one required upstream-order fix.

**Findings:**

- Required: the initial design put all four fields immediately after
  `macos-titlebar-proxy-icon`, but upstream only places
  `macos-dock-drop-behavior` there. The three bool fields belong after
  `macos-hidden` and before `macos-icon`.

**Fix:** Updated the design to specify the two upstream insertion points
separately.

The reviewer re-reviewed the fix and approved the design with no remaining
required findings.

## Result

**Result:** Pass

Roastty now carries the four remaining macOS scalar config keys from this
tranche:

- `macos-dock-drop-behavior` as an upstream-shaped enum with `new-tab` and
  `new-window`, defaulting to `new-tab`.
- `macos-auto-secure-input`, `macos-secure-input-indication`, and
  `macos-applescript` as bool fields defaulting to `true`.

The formatter preserves the upstream declaration order: dock-drop behavior is
between `macos-titlebar-proxy-icon` and `macos-option-as-alt`, while the three
bools are after `macos-hidden` and before `macos-icon`. Parsing supports
empty-value resets, invalid enum/bool diagnostics, bool bare flags as `true`,
and the upstream deprecated `macos-dock-drop-behavior = window` compatibility
alias as `new-window`.

The Phase F public-config tail is now 24 keys: language, font
feature/variation/metric/freetype knobs, cursor-click/mouse-hide, `input`, and
`keybind`.

Verification:

- `cargo test -p roastty macos_tail_config` passed 1 filtered unit test plus the
  ABI harness filter.
- `cargo test -p roastty config_format_config_emits_fields_in_upstream_order`
  passed 1 filtered unit test plus the ABI harness filter.
- The first `cargo test -p roastty` run passed 4,860 unit tests and failed only
  the known unrelated
  `surface_foreground_pid_reports_worker_foreground_pid_after_start` race.
- `cargo test -p roastty surface_foreground_pid_reports_worker_foreground_pid_after_start -- --nocapture`
  passed when rerun alone.
- Final `cargo test -p roastty` passed 4,861 Rust unit tests, 0 failed, 4
  ignored; the C ABI harness passed with the existing enum-conversion warnings;
  doc tests passed with 0 tests.
- `cargo fmt --check -p roastty` passed.
- `prettier --check --prose-wrap always --print-width 80 issues/0802-libroastty-completion-and-mac-app/165-macos-tail-config.md issues/0802-libroastty-completion-and-mac-app/README.md`
  passed.
- `git diff --check` passed.

## Completion Review

**Reviewer:** Codex-native adversarial review subagent `Newton`, fresh context.

**Verdict:** Approved with no findings.

The reviewer verified the diff from the plan commit, confirmed the result commit
had not been made, checked that implementation scope stayed within
parser/formatter/storage/tests/docs, and confirmed the macOS defaults, ordering,
keywords, and `window` alias match upstream `Config.zig`.

Read-only checks reproduced by the reviewer:

- `cargo test -p roastty macos_tail_config`
- `cargo test -p roastty config_format_config_emits_fields_in_upstream_order`
- `cargo test -p roastty`
- `cargo fmt --check -p roastty`
- Prettier check
- `git diff --check`

## Conclusion

The macOS scalar config tail is complete at the parser/formatter/storage layer.
The remaining Phase F public-config surface is smaller and more focused:
language, font feature/variation/metric/freetype knobs, cursor-click/mouse-hide,
`input`, and `keybind`. Runtime Dock drop behavior, Secure Input heuristics and
indication, and AppleScript handling remain app/platform integration work rather
than config-field gaps.
