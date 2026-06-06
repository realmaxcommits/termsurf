+++
[implementer]
agent = "codex"
model = "gpt-5"
reasoning = "high"

[review.design]
agent = "codex"
model = "gpt-5"
reasoning = "medium"
+++

# Experiment 751: Surface CLI Keybind Is Binding

## Description

Make CLI-loaded root keybinds visible to `roastty_surface_key_is_binding`.
Experiment 750 made `roastty_config_key_is_binding` recognize configured
keybinds, but surfaces still only report static default bindings because `App`
does not carry configured keybind state and `Surface::key_is_binding` only asks
the default flag matcher.

This experiment copies configured root keybinds from `Config` into `App` and
teaches the surface query to recognize them before falling back to static
defaults. Because Roastty does not yet parse full binding flags or execute
configured binding actions, configured surface matches return the ordinary
consumed flag value, `ROASTTY_KEYBIND_FLAGS_DEFAULT`.

This remains query-only. It does not dispatch configured actions, parse action
flags, mark configured performable bindings, implement key tables, sequences,
`clear`, `unbind`, global/all prefixes, config-file loading, diagnostics, or
frontend action routing.

## Changes

- `roastty/src/lib.rs`
  - Add configured keybind trigger storage to `App`.
  - Clone configured root keybinds from `Config` into `App` during
    `roastty_app_new`.
  - Replace the app's configured keybind storage during
    `roastty_app_update_config`.
  - Add an app-level helper that checks whether a key event matches a configured
    root keybind trigger using the same matcher added in Experiment 750.
  - Make `Surface::key_is_binding` check the attached app's configured keybinds
    before default bindings.
  - Write `ROASTTY_KEYBIND_FLAGS_DEFAULT` for configured surface matches when
    the caller supplies a flags pointer, and tolerate null flags for true
    matches.
  - Preserve existing false-path behavior: null surface, null event, detached
    surface, and nonmatching events return `false` and zero the optional flags
    pointer.
  - Preserve existing static default behavior and default/performable flag
    values when no configured keybind matches.
- `roastty/tests/abi_harness.c`
  - Add C coverage that an app created from a CLI-loaded config makes
    `roastty_surface_key_is_binding` return true and
    `ROASTTY_KEYBIND_FLAGS_DEFAULT` for configured physical and Unicode keybind
    events.
  - Assert configured matches tolerate a null flags pointer.
  - Assert configured release events, modifier mismatches, null events, detached
    surfaces, and malformed CLI keybinds do not produce configured surface
    matches.
  - Assert static default surface matches still return their existing ordinary
    and performable flag values.
- Tests in `roastty/src/lib.rs`
  - Cover app construction from a CLI-loaded config, then free the config and
    prove surface queries still see the app-owned configured keybinds.
  - Cover `roastty_app_update_config` replacing the app's configured keybind
    storage.
  - Cover configured physical, Unicode, unshifted-codepoint, duplicate-action,
    release, modifier-mismatch, null-flags, and detached-surface cases.
  - Cover configured-over-static precedence with an overlap such as
    `cmd+c=some_action`, proving the configured match returns
    `ROASTTY_KEYBIND_FLAGS_DEFAULT` instead of the static command-C performable
    flags.
  - Cover default fallback flags after configured keybind support is present.

## Verification

- `cargo test -p roastty config_cli_keybind -- --nocapture --test-threads=1`
- `cargo test -p roastty config_key_is_binding -- --nocapture --test-threads=1`
- `cargo test -p roastty surface_key_is_binding -- --nocapture --test-threads=1`
- `cargo test -p roastty config_trigger -- --nocapture --test-threads=1`
- `cargo test -p roastty --test abi_harness -- --nocapture`
- `cargo fmt -p roastty`
- `cargo fmt -p roastty -- --check`
- `git diff --check`

## Design Review

Codex reviewed the initial Experiment 751 design and found one blocking
verification gap: the plan needed an explicit configured-over-static precedence
test. Since the design checks configured bindings before static defaults and
returns ordinary consumed flags for configured matches, an overlap such as
`cmd+c=some_action` must return `ROASTTY_KEYBIND_FLAGS_DEFAULT` instead of the
static command-C performable flags.

The design was updated to include that precedence case in the planned Rust test
coverage. Codex otherwise accepted the scope: app-owned configured keybind
storage is the right layer for surface queries, and ordinary consumed flags are
acceptable for configured matches until Roastty implements full action/flag
parsing and performability.

Codex re-reviewed the corrected design and approved it for the plan commit with
no remaining blocking findings.
