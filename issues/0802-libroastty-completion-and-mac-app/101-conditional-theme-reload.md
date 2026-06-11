# Experiment 101: Phase F — conditional theme reload

## Description

Add the next upstream theme-finalization behavior: track that different
light/dark themes depend on the conditional theme state, and rebuild the config
from replay entries when that state changes.

Upstream `Config.finalize()` inserts `.theme` into `_conditional_set` when the
configured light and dark theme names differ. `changeConditionalState()` then
compares only the conditional keys used by the current config; if the relevant
state changed, it creates a fresh config, sets the new conditional state,
replays the original in-memory config inputs, finalizes, and returns the new
config. If no relevant conditional key changed, it returns `null`.

Roastty already has the selected conditional state and in-memory replay entries
from Experiments 98–100. This experiment should use those pieces to make
light/dark theme switching rebuild the finalized config without rereading
default config files. Theme files may still be read during finalization,
matching the current theme-loading path.

This is still a config-internal slice. It should not add app C ABI exports,
runtime OS-theme notification plumbing, general `if = ...` conditional config
syntax, conditionalized theme replay entries, or live surface/app propagation.

## Changes

- `roastty/src/config/conditional.rs`
  - Make `conditional::Key` hashable so `Config` can track which conditional
    keys affect the current finalized config.
- `roastty/src/config/mod.rs`
  - Add a private `conditional_set` to `Config`, defaulting empty and cloning
    with the config.
  - During theme finalization, if `theme.light != theme.dark`, insert
    `conditional::Key::Theme` into `conditional_set` after preserving Exp99/100
    `window-theme = auto` to `system` behavior.
  - Add an internal `change_conditional_state` method that:
    - returns `Ok(None)` if no key in `conditional_set` changes between the
      current and requested conditional states;
    - builds a fresh default config with the requested conditional state;
    - replays the stored file/CLI replay entries into it without recording
      duplicates;
    - finalizes the rebuilt config, including theme loading and scalar finalize;
    - returns `Ok(Some(new_config))` on relevant changes;
    - propagates replay failure as `ConfigSetError`, matching existing replay
      error handling.
  - Add a test-only variant that accepts explicit theme locations, reusing the
    Exp100 deterministic theme-dir mechanism.
  - Add focused tests proving:
    - same conditional state returns `None`;
    - changing the theme state is ignored when the config does not use a
      different light/dark theme;
    - a different light/dark theme marks the theme conditional as relevant;
    - changing light to dark reloads the dark theme and preserves user override
      priority;
    - changing dark back to light works after cloning/reloading, matching the
      upstream regression test shape;
    - replay entries are preserved on the rebuilt config and are not duplicated;
    - replay failure during conditional rebuild returns the expected
      `ConfigSetError`.

No general conditional config syntax, conditionalized theme-file replay steps,
resource packaging, app ABI, runtime notification, or surface propagation should
be implemented in this experiment.

## Verification

Pass criteria:

1. `cargo test -p roastty config_conditional_theme`
2. `cargo test -p roastty config_theme_loading`
3. `cargo test -p roastty config_replay`
4. `cargo test -p roastty`
5. `cargo fmt --check`
6. `git diff --check`

The full `cargo test -p roastty` run must pass. The existing ABI harness may
print its known enum-conversion warnings, but no new failures are acceptable.

## Design Review

Codex-native adversarial review ran in fresh context with subagent
`019eb613-5c7c-7660-bb50-6271576f50d7`.

Verdict: **APPROVED**

Findings:

- Optional: the test list did not directly cover the planned replay-failure
  branch in `change_conditional_state`.

Fix:

- Added a focused replay-failure test requirement to the experiment design.
