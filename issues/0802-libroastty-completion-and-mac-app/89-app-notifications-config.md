+++
implementer = "codex"
review_design = "codex-adversarial"
+++

# Experiment 89: Phase F — app notifications config

## Description

Port the pinned upstream `app-notifications` config surface from
`vendor/ghostty/src/config/Config.zig` into `roastty/src/config/mod.rs`.

Upstream defines `app-notifications` after `bell-audio-volume` as a packed bool
struct:

- `clipboard-copy = true`
- `config-reload = true`

Its CLI/config syntax is upstream's packed-struct bool-flag syntax: a standalone
bool sets every flag, and comma-separated `[no-]flag` names override individual
fields while omitted fields keep their defaults. Empty assigned values reset to
the default value, and missing values diagnose as `ValueRequired`.

This experiment is parser/formatter-only. GTK toast delivery, clipboard-copy
notification UI, config-reload notification UI, app C ABI exposure, and any
runtime notification behavior remain later work.

## Changes

- `roastty/src/config/mod.rs`
  - Add `Config::app_notifications: AppNotifications` after `bell_features` and
    before `background` in the current local struct/default region, leaving the
    pre-existing local `bell_audio_path` / `bell_audio_volume` placement
    untouched.
  - Initialize the default to `AppNotifications::default()`.
  - Format `app-notifications` after `bell-features` and before
    `macos-non-native-fullscreen`, using the current local upstream-order slot
    for this config region.
  - Route `Config::set("app-notifications", ...)` through the existing
    `set_packed_field` helper.
  - Add an `AppNotifications` struct with the two upstream flags, `Default`,
    `parse_cli`, and `format_entry`, reusing the local `parse_packed_flags` /
    `EntryFormatter::entry_flags` pattern.
  - Extend default-value, format-order, and aggregate config-set route tests.
  - Add focused tests for:
    - upstream defaults (`clipboard-copy,config-reload` enabled);
    - formatting order and canonical `[no-]flag` output;
    - individual flag enable/disable parsing;
    - standalone bool setting both flags;
    - empty value resetting to defaults;
    - missing value returning `ValueRequired`;
    - unknown flags returning `InvalidValue`;
    - clone/equality preserving values.

- `issues/0802-libroastty-completion-and-mac-app/README.md`
  - Link this experiment as `Designed` in the experiment index.
  - After implementation, add an operating note describing the parser-only
    status and runtime work left open.

## Verification

Before implementation:

- Codex-native adversarial design review approves the experiment.
- Plan commit exists before source edits begin.

After implementation:

- `cargo fmt`
- `cargo test -p roastty app_notifications`
- `cargo test -p roastty config_format_config`
- `cargo test -p roastty`
- `cargo fmt --check`
- `git diff --check`

Pass criteria:

- `app-notifications` is present in defaults, formatter output, `Config::set`,
  and format-order tests in the same upstream-order region as `bell-features`.
- The packed-flag semantics match upstream's `AppNotifications` defaults and
  `parsePackedStruct` behavior for bool-all, `[no-]flag` lists, empty reset,
  missing values, and invalid names.
- Runtime notification behavior is not claimed or changed by this experiment.

## Design Review

Codex adversarial reviewer `019eb513-4106-7033-8c4e-c916ba975882` returned
**Approved** with no required findings. The reviewer confirmed the README link,
required sections, parser/formatter-only scope, upstream defaults, packed-struct
semantics, local placement plan, and verification checklist.
