# Experiment 99: Phase F — absolute theme loading

## Description

Use the replay foundation from Experiment 98 to add the first real theme-loading
slice to Roastty config finalization.

Upstream `Config.finalize()` loads `theme` before later derivation so theme
values can establish defaults, then replays the user's original config inputs on
top so explicit user config wins. This experiment should implement that ordering
for existing absolute theme-file paths only. That keeps the slice small while
proving the hard part: theme config is lower priority than user file/CLI config,
and the selected light/dark theme follows the current conditional theme state.

This is not the full upstream `config/theme.zig` port. Named theme lookup,
`themes` directory discovery, bundled resource themes, conditional reload,
diagnostic-message parity, theme-file path-expansion replay, and conversion of
theme replay entries into conditional replay steps remain later work.

## Changes

- `roastty/src/config/mod.rs`
  - Add private conditional theme state storage to `Config`, defaulting to
    upstream's light theme state.
  - Add a small finalization report type for theme loading results so tests can
    inspect whether a theme loaded, whether theme parse diagnostics happened,
    and whether an absolute theme path failed to open.
  - Add `Config::finalize_with_report`, with existing `Config::finalize`
    delegating to it and discarding the report to preserve current callers.
  - During finalization, if `theme` is set:
    - choose `theme.light` or `theme.dark` from the current conditional theme
      state;
    - if the chosen name is an absolute path to a regular readable file, load it
      into a fresh default `Config`;
    - replay the current config's recorded file/CLI entries onto that fresh
      config so user config overrides theme values;
    - preserve the original replay entries on the final config, not the theme
      file's replay entries;
    - when light and dark theme names differ, change `window-theme = auto` to
      `system`, matching upstream's guard against auto-selecting a window theme
      from the newly-loaded terminal theme;
    - continue with the existing scalar/finalize derivations on the resulting
      config.
  - Preserve current behavior when no theme is set.
  - For this slice, leave non-absolute theme names unloaded and reported as
    unsupported rather than searching user/resource theme directories.
  - Add tests proving:
    - an absolute theme file applies during finalization;
    - explicit file/CLI config values override theme-file values after replay;
    - light/dark theme pairs select the expected absolute path from conditional
      state;
    - different light/dark theme names turn `window-theme = auto` into
      `window-theme = system`;
    - missing, unreadable, or non-regular absolute theme paths are reported
      without panicking;
    - finalization preserves the user replay list and does not replace it with
      theme-file entries;
    - no-theme finalization keeps existing scalar behavior unchanged.

No named-theme locator, bundled resource themes, full upstream diagnostic text,
conditional reload, `changeConditionalState`, theme replay conditionalization,
or app C ABI exposure should be implemented in this experiment.

## Verification

Pass criteria:

1. `cargo test -p roastty config_theme_loading`
2. `cargo test -p roastty config_finalize_scalar_tail`
3. `cargo test -p roastty config_replay`
4. `cargo test -p roastty`
5. `cargo fmt --check`
6. `git diff --check`

The full `cargo test -p roastty` run must pass. The existing ABI harness may
print its known enum-conversion warnings, but no new failures are acceptable.

## Design Review

Codex-native adversarial review ran in fresh context with subagent
`019eb5e7-00c1-71c1-9994-e36e2bcc786e`.

Initial verdict: **CHANGES REQUIRED**

- Required: the design omitted upstream behavior where different light/dark
  theme names change `window-theme = auto` to `system`.
- Optional: non-regular absolute paths were planned but not explicitly verified.

Fix:

- Added the `window-theme = auto` to `system` behavior to scope and
  verification.
- Added explicit verification for missing, unreadable, and non-regular absolute
  theme paths.

Final verdict after re-review: **APPROVED**

Findings: None remaining.
