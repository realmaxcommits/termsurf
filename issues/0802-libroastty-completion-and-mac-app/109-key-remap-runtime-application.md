# Experiment 109: Phase G — key-remap runtime application

## Description

Apply the finalized `key-remap` config to runtime surface key handling.

Exp 107 ported the reusable `RemapSet` parser/apply/formatter foundation, and
Exp 108 wired that set into `Config` parsing, formatting, and finalization.
However, runtime key events still flow through `Surface::key` and
`Surface::key_is_binding` with their original modifiers. Upstream `Surface.zig`
clones the finalized `key-remap` set into each surface's derived config, then
applies it at the start of both `keyCallback` and `keyEventIsBinding` before
keybinding lookup or terminal key encoding.

This experiment should add the same surface-local runtime application in
Roastty. It must not implement native keymaps, app-scoped `roastty_app_key`,
full upstream `input.Binding.Set` tables, app C ABI exposure for config strings,
or keyboard-layout change handling.

## Changes

- `roastty/src/lib.rs`
  - Import or reference `key_mods::RemapSet` where needed.
  - Add a `key_remaps: RemapSet` field to `Surface`.
  - In `roastty_surface_new`, initialize `Surface::key_remaps` from the app's
    finalized parsed config (`app.parsed_config.key_remap.clone()`).
  - In `Surface::apply_config`, refresh `self.key_remaps` from the finalized
    parsed config so `roastty_surface_update_config` and
    `roastty_app_update_config` update existing surfaces.
  - Add a small helper that takes a `KeyEvent`, applies `self.key_remaps` when
    `is_remapped(event.mods)` is true, and returns the remapped event without
    mutating the caller's event object.
  - Use that helper at the start of `Surface::key` so configured bindings,
    default bindings, VT KAM gating, terminal key encoding, release consumption,
    and `last_key_event` all observe the remapped modifiers.
  - Use the same helper in `Surface::key_is_binding` so by-value and handle
    `surface_key_is_binding` match the actual `surface_key` runtime path.
  - Leave `roastty_config_key_is_binding` and `roastty_app_key` unchanged
    because they do not have a surface-derived config object in the current
    local model.
  - Add focused tests proving:
    - remapped modifiers trigger configured bindings in `surface_key`;
    - `surface_key_is_binding` reports the remapped configured binding;
    - remapped modifiers affect default binding detection;
    - encoded terminal input uses remapped modifiers when no binding consumes
      the event;
    - app or surface config updates refresh the remap set on existing surfaces;
    - the original `KeyEvent` handle is not mutated by remap application.

## Verification

Pass criteria:

1. `cargo test -p roastty key_remap`
2. `cargo test -p roastty surface_key`
3. `cargo test -p roastty -- --test-threads=1`
4. `cargo fmt --check`
5. `git diff --check`
6. `prettier --check --prose-wrap always --print-width 80 issues/0802-libroastty-completion-and-mac-app/109-key-remap-runtime-application.md issues/0802-libroastty-completion-and-mac-app/README.md`

The serial full-suite gate remains the required broad gate because the known
unrelated
`tests::surface_foreground_pid_reports_worker_foreground_pid_after_start` flake
has reproduced only under parallel full-suite execution.

## Design Review

Codex-native adversarial review ran in fresh context with subagent
`019eb695-5752-7ac0-82a9-f4dcdf023845`.

Verdict: **APPROVED**

Findings and fixes:

- Nit: the initial design said `roastty_surface_new` should default to an empty
  remap set when no app exists, but the constructor already returns null for a
  null app. Removed the unreachable fallback phrase.
