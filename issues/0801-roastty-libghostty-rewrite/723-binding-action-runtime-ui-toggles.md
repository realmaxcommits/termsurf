+++
[implementer]
agent = "codex"
model = "gpt-5"
reasoning = "high"

[review.design]
agent = "codex"
model = "gpt-5"
reasoning = "medium"

[review.result]
agent = "codex"
model = "gpt-5"
reasoning = "medium"
+++

# Experiment 723: Binding Action Runtime UI Toggles

## Description

Experiments 721 and 722 added tab/window and tab-navigation runtime forwarding
for binding-action strings. Upstream Ghostty has another nearby low-risk group
of surface-scoped binding actions that forward no-storage UI/runtime commands to
the app runtime:

- `toggle_window_decorations`
- `toggle_command_palette`
- `toggle_background_opacity`
- `show_on_screen_keyboard`

This experiment adds those parser and callback forwarding paths only. It does
not implement the window decoration UI, command palette, opacity state changes,
on-screen keyboard presentation, or Swift frontend behavior. The
frontend/runtime remains responsible for consuming the forwarded action tags.

Actions with payload enums or local state changes, such as
`toggle_window_float_on_top`, `toggle_secure_input`, `inspector`, and
`toggle_mouse_reporting`, are intentionally left for later experiments because
they need additional storage conventions or surface state behavior.

## Changes

- `roastty/include/roastty.h`
  - Add action tags matching upstream `ghostty_action_tag_e` values:
    - `ROASTTY_ACTION_TOGGLE_WINDOW_DECORATIONS = 9`
    - `ROASTTY_ACTION_TOGGLE_COMMAND_PALETTE = 11`
    - `ROASTTY_ACTION_TOGGLE_BACKGROUND_OPACITY = 13`
    - `ROASTTY_ACTION_SHOW_ON_SCREEN_KEYBOARD = 57`
  - Document that all four actions leave `storage` zeroed.

- `roastty/src/lib.rs`
  - Add matching constants.
  - Extend `parse_binding_action` to accept:
    - `toggle_window_decorations`
    - `toggle_command_palette`
    - `toggle_background_opacity`
    - `show_on_screen_keyboard`
  - Reject empty-colon and non-empty parameters for all four no-parameter
    actions.
  - Forward all four actions through the existing runtime `action_cb`, returning
    `false` for null, detached, and no-callback surfaces and otherwise returning
    the callback result.
  - Keep all previously supported binding actions unchanged.

- `roastty/tests/abi_harness.c`
  - Add C ABI smoke coverage for the new action constants.
  - Add malformed runtime-UI toggle action rejection checks.
  - Add no-callback coverage that valid runtime-UI forwarding actions return
    `false` without crashing.

- Tests in `roastty/src/lib.rs`
  - Cover constants matching upstream values.
  - Cover invalid parser forms, including empty-colon and non-empty parameters.
  - Cover null, detached, and no-callback surfaces returning `false`.
  - Cover valid runtime-UI toggle actions forwarding expected tags, target,
    zeroed storage, and callback result.
  - Re-run existing binding-action tests to prove previous action semantics did
    not change.

## Verification

Run:

- `cargo fmt -p roastty`
- `cargo test -p roastty runtime_ui -- --nocapture --test-threads=1`
- `cargo test -p roastty binding_action -- --nocapture --test-threads=1`
- `cargo test -p roastty --test abi_harness`
- `cargo fmt -p roastty -- --check`
- `git diff --check`

## Design Review

Codex reviewed the Experiment 723 design and found no technical blockers. The
review approved the scope as no-storage runtime forwarding only, with payload
and stateful actions deferred to later experiments.

The review found one workflow blocker: this design-review section still said
`Pending.` This section now records the review outcome, and the README tuple is
`Codex/Codex/-`.

## Result

**Result:** Pass

Implemented no-storage runtime UI binding-action forwarding through the existing
runtime action callback path. Roastty now exposes upstream-matching action tags
for window decorations, command palette, background opacity, and on-screen
keyboard actions in `roastty/include/roastty.h`.

`parse_binding_action` now accepts:

- `toggle_window_decorations`
- `toggle_command_palette`
- `toggle_background_opacity`
- `show_on_screen_keyboard`

All four actions reject empty-colon and non-empty parameters, forward with
zeroed storage, and return the runtime callback result.

Verification passed:

- `cargo fmt -p roastty`
- `cargo test -p roastty runtime_ui -- --nocapture --test-threads=1`
- `cargo test -p roastty binding_action -- --nocapture --test-threads=1`
- `cargo test -p roastty --test abi_harness`
- `cargo fmt -p roastty -- --check`
- `git diff --check`

## Conclusion

Runtime UI toggle binding actions now reach the app runtime with stable
upstream-shaped action tags and zeroed storage. The remaining nearby forwarding
work includes payload/state actions such as `toggle_window_float_on_top`,
`toggle_secure_input`, `inspector`, and local surface state such as
`toggle_mouse_reporting`.

## Completion Review

Codex reviewed the completed Experiment 723 result and found no implementation
blockers. The review approved the action constants, header storage
documentation, zeroed-storage forwarding, parser rejection paths, no-callback
behavior, and Rust/C ABI test coverage.

The review found one workflow blocker: result-review provenance was missing from
the experiment frontmatter and README tuple. This section, the `[review.result]`
frontmatter, and the README tuple now record the completion review.

Codex re-reviewed the revised result and found no remaining findings. The
completion review approved the result for commit.
