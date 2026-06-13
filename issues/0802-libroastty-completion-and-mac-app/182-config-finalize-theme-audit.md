# Experiment 182: Phase F — config finalize and theme audit

## Description

Close the two remaining Phase F checklist items by proving the current
`Config::finalize()` and theme-loading implementation covers the intended
upstream behavior.

The Phase F roadmap still shows these broad items unchecked:

- `finalize()` — cross-field validation / derivation / clamping
- Theme loading — themes-dir locator + file read + palette/option application

Earlier experiments implemented the work in slices: scalar finalization,
absolute theme loading, named theme lookup, conditional theme reload,
working-directory defaulting, command/home resolution, GTK single-instance
defaulting, quit-delay warnings, link-url mutation, key-remap finalization, and
palette runtime application. Current source evidence suggests the remaining
roadmap entries are now proof/documentation gaps rather than new production-code
gaps.

This experiment should audit the current source and run the focused regression
tests that prove the broad Phase F checklist entries. It should check those
roadmap boxes only if the evidence shows the current implementation owns the
finalize/theme behaviors end to end. It should not claim broader Issue 802
completion or Phase G completion.

## Changes

- `issues/0802-libroastty-completion-and-mac-app/README.md`
  - Link this experiment as `Designed`.
  - After verification, mark it `Pass`, `Partial`, or `Fail`.
  - Check the `finalize()` roadmap item only if source audit and tests prove the
    current `Config::finalize()` path covers the cross-field derivations,
    defaults, clamps, warnings, link-url mutation, and key-remap finalization
    that previous experiments identified as the upstream finalize scope.
  - Check the theme-loading roadmap item only if source audit and tests prove
    default theme directories, absolute/named theme file loading,
    user-over-theme replay priority, light/dark conditional reload, and
    theme-driven option/palette application.

- `issues/0802-libroastty-completion-and-mac-app/182-config-finalize-theme-audit.md`
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

- Inspect the current finalize pipeline:

  ```bash
  sed -n '2160,2615p' roastty/src/config/mod.rs
  ```

- Confirm theme location discovery includes renamed user themes and app
  resources themes:

  ```bash
  sed -n '2968,3005p' roastty/src/config/mod.rs
  sed -n '1,90p' roastty/src/config/loader.rs
  ```

- Confirm key-remap finalization is called from `Config::finalize()` and has
  deterministic ordering tests:

  ```bash
  rg -n "key_remap\\.finalize|fn key_remap_set_finalize" \
    roastty/src/config/mod.rs roastty/src/input/key_mods.rs
  ```

Focused tests:

- `cargo test -p roastty config_finalize_scalar_tail`
- `cargo test -p roastty config_working_directory_finalize`
- `cargo test -p roastty config_command_home_finalize`
- `cargo test -p roastty config_gtk_single_instance_finalize`
- `cargo test -p roastty config_quit_delay_finalize_warning`
- `cargo test -p roastty config_link_url_finalize`
- `cargo test -p roastty key_remap_set_finalize`
- `cargo test -p roastty config_theme_loading`
- `cargo test -p roastty config_conditional_theme`
- `cargo test -p roastty palette`
- `cargo test -p roastty surface_apply_config_updates_palette`

Regression and hygiene:

- `cargo fmt --check -p roastty`
- `prettier --check --prose-wrap always --print-width 80 issues/0802-libroastty-completion-and-mac-app/182-config-finalize-theme-audit.md issues/0802-libroastty-completion-and-mac-app/README.md`
- `git diff --check`

**Pass** = source audit proves the broad finalize and theme-loading code paths
are wired, all focused tests pass, hygiene checks pass, and the two remaining
Phase F checklist items can be checked.

**Partial** = most evidence passes but a specific finalize/theme sub-behavior
remains unproved, stale, or too broad to check. Record the exact missing proof
or implementation gap.

**Fail** = source audit or focused tests contradict the claim that
`Config::finalize()` or theme loading are complete enough for the roadmap item.

## Design Review

**Reviewer:** Codex-native adversarial review subagent `Beauvoir`, fresh
context.

**Verdict:** Approved.

Findings: None. The reviewer confirmed the README links Experiment 182 as
`Designed`, the experiment has the required sections, the scope is limited to
the Phase F `finalize()` and theme-loading checklist items, and it does not
overclaim Phase G or broader Issue 802 completion. The reviewer also confirmed
the audit-only plan is legitimate because current source already shows
`Config::finalize()` wiring theme finalization, scalar finalization, and
key-remap finalization, with named/absolute theme loading and theme-location
discovery covered by the cited regions. Required hygiene checks are present.
