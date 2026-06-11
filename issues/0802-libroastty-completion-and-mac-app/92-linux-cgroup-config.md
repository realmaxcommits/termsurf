+++
implementer = "codex"
review_design = "codex-adversarial"
+++

# Experiment 92: Phase F — Linux cgroup config

## Description

Port the pinned upstream Linux cgroup config group from
`vendor/ghostty/src/config/Config.zig` into `roastty/src/config/mod.rs`.

Upstream defines this group after `macos-shortcuts`:

- `linux-cgroup: LinuxCgroup = single-instance` on Linux, otherwise `never`
- `linux-cgroup-memory-limit: ?u64 = null`
- `linux-cgroup-processes-limit: ?u64 = null`
- `linux-cgroup-hard-fail: bool = false`

This experiment is parser/formatter-only. Runtime transient `systemd` scope
creation, per-surface resource limits, single-instance interaction, reload
behavior for existing surfaces, app C ABI exposure, and Linux app integration
remain later work.

## Changes

- `roastty/src/config/mod.rs`
  - Add `Config` fields for the four Linux cgroup options after
    `macos_shortcuts` and before the font-family group in the current local
    struct/default region.
  - Initialize defaults to upstream values:
    - `linux_cgroup = LinuxCgroup::SingleInstance` on Linux, otherwise
      `LinuxCgroup::Never`
    - `linux_cgroup_memory_limit = None`
    - `linux_cgroup_processes_limit = None`
    - `linux_cgroup_hard_fail = false`
  - Format the four fields after `macos-shortcuts` and before `bold-color`,
    preserving the current local formatter gap before terminal color fields.
  - Route `Config::set` for:
    - `linux-cgroup` through `set_enum_field`;
    - `linux-cgroup-memory-limit` through `set_optional_value_field` with a new
      `u64` scalar parser;
    - `linux-cgroup-processes-limit` through `set_optional_value_field` with the
      same `u64` scalar parser;
    - `linux-cgroup-hard-fail` through `set_bool_field`.
  - Add `LinuxCgroup` enum variants and exact upstream keywords:
    - `never`
    - `always`
    - `single-instance`
  - Add `parse_u64_scalar_field` using the existing
    `parse_uint(value, 0, u64::MAX)` helper, matching the local base-0 scalar
    integer parsers.
  - Extend default-value, enum-route, format-order, scalar/optional formatting,
    and enum keyword round-trip tests.
  - Add a focused test for default formatter output, enum parsing, optional
    `u64` parsing/formatting, empty reset, missing/invalid diagnostics, bool
    parsing, and clone/equality.

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
- `cargo test -p roastty linux_cgroup`
- `cargo test -p roastty config_format_config`
- `cargo test -p roastty`
- `cargo fmt --check`
- `git diff --check`

Pass criteria:

- The four Linux cgroup config fields are present in defaults, formatter output,
  `Config::set`, and format-order tests in the current local formatter region.
- Enum parsing and formatting matches upstream keywords exactly.
- Optional `u64` parsing accepts normal/base-0 scalar values, resets empty
  values to `None`, diagnoses missing values as `ValueRequired`, and diagnoses
  invalid or overflowing values as `InvalidValue`.
- Runtime cgroup behavior is not claimed or changed by this experiment.

## Design Review

Codex-native adversarial reviewer `019eb551-b370-7f43-aa89-3bb8d113b8c6`
reviewed the design with fresh context and returned **Approved** with no
findings.

## Result

**Result:** Pass

Implemented parser/formatter-only support for the upstream Linux cgroup config
group in `roastty/src/config/mod.rs`:

- added `Config` storage/defaults for `linux-cgroup`,
  `linux-cgroup-memory-limit`, `linux-cgroup-processes-limit`, and
  `linux-cgroup-hard-fail`;
- defaulted `linux-cgroup` to `single-instance` on Linux and `never` elsewhere,
  matching upstream;
- formatted the four fields after `macos-shortcuts` and before `bold-color`;
- routed `Config::set` for the enum, optional `u64` limits, and bool hard-fail
  flag;
- added `LinuxCgroup::{Never,Always,SingleInstance}` with exact upstream
  keywords;
- added `parse_u64_scalar_field` using the existing base-0 `parse_uint` helper;
- extended default audits, enum-route coverage, format-order coverage, enum
  keyword round trips, and a focused parse/format/reset/diagnostic test.

Verification:

- `cargo fmt`
- `cargo test -p roastty linux_cgroup` — pass
- `cargo test -p roastty config_format_config` — pass
- `cargo test -p roastty surface_mouse_button_mode_drag_motion_uses_pressed_button`
  — pass after the first full-suite run hit that unrelated surface test once
- `cargo test -p roastty surface_mouse_button_reporting_honors_readonly_gate` —
  pass after the first full-suite run hit that unrelated surface test once
- `cargo test -p roastty` — pass on rerun: 4536 unit tests, C ABI harness pass,
  doc tests pass; the ABI harness still emits the pre-existing 10
  enum-conversion warnings
- `cargo fmt --check`
- `git diff --check`

## Conclusion

The upstream Linux cgroup config group now exists in roastty's parser and
formatter with the expected platform-dependent default, keywords, optional `u64`
semantics, reset behavior, diagnostics, and format-order placement. This remains
intentionally parser/formatter-only: runtime transient `systemd` scope creation,
resource-limit application, single-instance interaction, app C ABI exposure, and
Linux app integration remain later work.

## Completion Review

Codex-native adversarial reviewer `019eb562-e643-7883-bfca-739e7157e6e7`
reviewed the completed experiment with fresh context and returned **Approved**
with no findings.
