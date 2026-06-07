+++
[implementer]
agent = "codex"
model = "gpt-5"
reasoning = "high"
+++

# Experiment 768: Config Get App Lifecycle Bools

## Description

Wire the app lifecycle boolean config keys `initial-window` and
`quit-after-last-window-closed` into Roastty's aggregate config and the public
`roastty_config_get` C ABI.

Upstream defaults are macOS-stable for this slice: `initial-window = true` and
`quit-after-last-window-closed = false` because the latter defaults to true only
on Linux. Roastty currently returns those hard-coded defaults from
`roastty_config_get`, and `config::Config` has no aggregate fields for either
key.

This experiment only ports direct config state and lookup behavior. It does not
wire startup window creation or app shutdown policy; those runtime behaviors
remain separate app lifecycle slices.

## Changes

- `roastty/src/config/mod.rs`
  - Add `initial_window: bool` and `quit_after_last_window_closed: bool` to
    `config::Config`.
  - Set macOS defaults: `initial_window = true`,
    `quit_after_last_window_closed = false`.
  - Include both keys in `format_config` output.
  - Route `Config::set("initial-window", ...)` and
    `Config::set("quit-after-last-window-closed", ...)` through the existing
    boolean parser.
  - Add aggregate tests for defaults, formatting, set routing, empty reset, and
    invalid values.
- `roastty/src/lib.rs`
  - Make `roastty_config_get("initial-window")` and
    `roastty_config_get("quit-after-last-window-closed")` read parsed config
    fields instead of returning hard-coded defaults.
  - Add C ABI tests proving both keys reflect default, file-loaded, CLI-loaded,
    cloned, and reset-to-default values.

## Verification

- `cargo test -p roastty config_get_app_lifecycle -- --nocapture --test-threads=1`
- `cargo test -p roastty config_ -- --nocapture --test-threads=1`
- `cargo fmt -p roastty`
- `cargo fmt -p roastty -- --check`
- `git diff --check`

The experiment passes if both boolean fields are stored in aggregate config, can
be set through file and CLI loading, format consistently with other boolean
fields, reject unknown boolean values as `InvalidValue`, and are returned by
`roastty_config_get` from parsed state.

## Design Review

Codex reviewed the design and approved it with no blocking findings. The review
confirmed that both keys have bool-shaped ABI getters and map cleanly to the
existing boolean config parser: bare flags set `true`, empty values reset to the
field default, and unknown values become `InvalidValue`.

Non-blocking suggestions from the review: include bare CLI coverage, especially
for `--quit-after-last-window-closed`; include reset coverage for both fields;
and note in the result that `quit-after-last-window-closed = false` is the
macOS/default-for-this-port behavior.
