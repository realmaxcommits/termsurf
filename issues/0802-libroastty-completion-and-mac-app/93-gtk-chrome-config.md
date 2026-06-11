+++
implementer = "codex"
review_design = "codex-adversarial"
+++

# Experiment 93: Phase F — GTK chrome config

## Description

Port the pinned upstream GTK window/chrome config subgroup from
`vendor/ghostty/src/config/Config.zig` into `roastty/src/config/mod.rs`.

Upstream defines these fields after the Linux cgroup group:

- `gtk-opengl-debug: bool = builtin.mode == Debug`
- `gtk-single-instance: GtkSingleInstance = detect`
- `gtk-titlebar: bool = true`
- `gtk-tabs-location: GtkTabsLocation = top`
- `gtk-titlebar-hide-when-maximized: bool = false`
- `gtk-toolbar-style: GtkToolbarStyle = raised`
- `gtk-titlebar-style: GtkTitlebarStyle = native`
- `gtk-wide-tabs: bool = true`

This experiment is parser/formatter-only. Runtime GTK OpenGL logging, GTK
single-instance behavior, titlebar/tabbar rendering, maximized-titlebar hiding,
toolbar styling, app C ABI exposure, and GTK app integration remain later work.

`gtk-custom-css`, `desktop-notifications`, and `progress-style` are the next
upstream fields, but they are intentionally left to later experiments:
`gtk-custom-css` is repeatable path loading/expansion, while desktop
notifications and progress sequences are runtime terminal/app behavior gates.

## Changes

- `roastty/src/config/mod.rs`
  - Add `Config` fields for the eight GTK chrome options after the Linux cgroup
    fields and before the font-family group in the current local struct/default
    region.
  - Initialize defaults to upstream values:
    - `gtk_opengl_debug = cfg!(debug_assertions)`
    - `gtk_single_instance = GtkSingleInstance::Detect`
    - `gtk_titlebar = true`
    - `gtk_tabs_location = GtkTabsLocation::Top`
    - `gtk_titlebar_hide_when_maximized = false`
    - `gtk_toolbar_style = GtkToolbarStyle::Raised`
    - `gtk_titlebar_style = GtkTitlebarStyle::Native`
    - `gtk_wide_tabs = true`
  - Format the eight fields after `linux-cgroup-hard-fail` and before
    `bold-color`, preserving the current local formatter gap before terminal
    color fields.
  - Route `Config::set` for bool fields through `set_bool_field`.
  - Route ordinary enum fields through `set_enum_field`.
  - Preserve upstream compatibility shims for the two renamed/removed values:
    - `gtk-tabs-location = hidden` updates
      `window_show_tab_bar = WindowShowTabBar::Never` and does not change
      `gtk_tabs_location`; other values parse as `GtkTabsLocation`.
    - `gtk-single-instance = desktop` maps to
      `gtk_single_instance = GtkSingleInstance::Detect`; other values parse as
      `GtkSingleInstance`.
  - Add enum variants and exact upstream keywords:
    - `GtkSingleInstance`: `false`, `true`, `detect`
    - `GtkTabsLocation`: `top`, `bottom`
    - `GtkToolbarStyle`: `flat`, `raised`, `raised-border`
    - `GtkTitlebarStyle`: `native`, `tabs`
  - Extend default-value, enum-route, bool-route, compatibility-route,
    format-order, and enum keyword round-trip tests.
  - Add a focused test for default formatter output, enum parsing, bool parsing,
    `gtk-tabs-location = hidden` compatibility, `gtk-single-instance = desktop`
    compatibility, empty reset, missing/invalid diagnostics, and clone/equality.

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
- `cargo test -p roastty gtk_chrome`
- `cargo test -p roastty config_format_config`
- `cargo test -p roastty`
- `cargo fmt --check`
- `git diff --check`

Pass criteria:

- The eight GTK chrome config fields are present in defaults, formatter output,
  `Config::set`, and format-order tests in the current local formatter region.
- Enum parsing and formatting matches upstream keywords exactly, including
  keyword tags that are Rust identifiers only after mapping (`false`, `true`,
  `raised-border`).
- Compatibility parsing matches upstream: `gtk-tabs-location = hidden` still
  maps to `window-show-tab-bar = never`, while `top`/`bottom` populate
  `gtk-tabs-location`; `gtk-single-instance = desktop` maps to `detect`.
- Bool fields keep local config semantics: bare key sets `true`, empty values
  reset to default, and invalid values diagnose as `InvalidValue`.
- Runtime GTK behavior is not claimed or changed by this experiment.

## Design Review

Codex-native adversarial reviewer `019eb568-a02b-7043-a84c-af0db7b5a5ec`
returned **Changes Required**: the initial design routed GTK enum fields
directly through `set_enum_field` and omitted upstream compatibility handling
for `gtk-tabs-location = hidden` and `gtk-single-instance = desktop`. The design
was updated to require those compatibility shims before normal enum parsing and
to test both compatibility values and normal enum values.

Codex-native adversarial re-reviewer `019eb56a-cdae-71c3-99a9-bc6a7551a3a9`
returned **Approved** with no findings.

## Result

**Result:** Pass

Implemented the parser/formatter-only GTK chrome config surface in
`roastty/src/config/mod.rs`:

- Added the eight GTK chrome fields to `Config`, defaults, formatter output, and
  `Config::set`.
- Added `GtkSingleInstance`, `GtkTabsLocation`, `GtkToolbarStyle`, and
  `GtkTitlebarStyle` enums with exact upstream keywords.
- Preserved upstream compatibility behavior:
  - `gtk-single-instance = desktop` maps to `detect`.
  - `gtk-tabs-location = hidden` sets `window-show-tab-bar = never` without
    changing the `gtk-tabs-location` value.
- Extended default, format-order, enum-route, bool-route, compatibility,
  diagnostic, keyword round-trip, and clone/equality tests.

Verification:

- `cargo fmt` — pass.
- `cargo test -p roastty gtk_chrome` — pass: 1 passed, 0 failed.
- `cargo test -p roastty config_format_config` — pass: 1 passed, 0 failed.
- Initial `cargo test -p roastty` — transient fail: 4536 passed, 1 failed. The
  failure was
  `tests::surface_foreground_pid_reports_worker_foreground_pid_after_start`,
  with `left: 82986`, `right: 82981`.
- Initial
  `cargo test -p roastty surface_foreground_pid_reports_worker_foreground_pid_after_start`
  — transient fail in isolation, with `left: 4244`, `right: 4240`.
- Rerun
  `cargo test -p roastty surface_foreground_pid_reports_worker_foreground_pid_after_start`
  — pass: 1 passed, 0 failed.
- Rerun `cargo test -p roastty` — pass: 4537 unit tests passed, 1 ABI harness
  test passed, 0 doc tests. The ABI harness emitted the existing 10 enum
  conversion warnings.
- `cargo fmt --check` — pass.
- `git diff --check` — pass.

Codex-native adversarial completion reviewer
`019eb575-3adc-7990-9ff3-0fce3b2ee1c2` returned **Changes Required** only for
stale result documentation: the reviewer independently reran
`cargo test -p roastty` and the isolated foreground-PID test, both passed, and
the source implementation had no required findings. This result section and the
README status were updated to reflect the current passing verification.

Codex-native adversarial re-reviewer `019eb575-3adc-7990-9ff3-0fce3b2ee1c2`
returned **Approved**: the result now records `Pass`, preserves the initial
transient foreground-PID failures, records the passing isolated and full-suite
reruns, and the README marks Experiment 93 as `Pass`. The re-review found no new
required findings.

## Conclusion

The GTK chrome parser/formatter surface is implemented and covered by targeted
tests, including the two upstream compatibility shims missed by the first design
draft. Runtime GTK behavior remains out of scope for this experiment. The next
experiment can continue with the remaining upstream config surface after the
completion re-review approves the corrected result docs and this result is
committed.
