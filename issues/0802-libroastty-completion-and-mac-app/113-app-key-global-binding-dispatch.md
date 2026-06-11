# Experiment 113: Phase G — app key global binding dispatch

## Description

Wire Roastty's embedded `roastty_app_key` path to configured app-level
keybindings.

Upstream `ghostty_app_key` sends global key captures through
`app.keyEvent(.app, event.keyEvent())`. Roastty currently tracks
`global:`-prefixed keybinds for `roastty_app_has_global_keybinds`, but
`roastty_app_key` always returns `false`, so the copied app cannot dispatch the
global key events that it registers with the platform.

This experiment implements the narrow app-key dispatch foundation for `global:`
configured single-trigger bindings. It will use the already-parsed
`ConfigKeybind` list, app surface registration, and existing action dispatch
helpers. Plain `all:` bindings remain surface-key behavior only; upstream
`ghostty_app_key` fans out only global bindings. This experiment does not add
multi-key sequences/chords, key tables, native keymaps, keyboard-layout reload,
default global bindings, or the remaining upstream action catalog.

## Changes

- `roastty/src/lib.rs`
  - Add an app-level key-dispatch helper used by `roastty_app_key`.
  - Match only configured bindings whose flags include `global:`. Unprefixed and
    plain `all:` configured bindings remain surface-only in this path.
  - For `global:` matches, dispatch surface-scoped actions to the currently
    registered surfaces:
    - `global:` implies all-surface fanout for this path, matching upstream
      app-key behavior;
    - app-scoped actions such as `quit` dispatch once to the app target.
  - Reuse the existing configured-binding consumption semantics: `global:`
    bindings are consumed when matched, including `unconsumed:global:` forms,
    and `performable:` only suppresses consumption when a non-global action is
    not performed.
  - Skip stale registered surface pointers whose `surface.app` no longer matches
    the app handle.
  - Keep `roastty_surface_key` behavior unchanged.
- Tests in `roastty/src/lib.rs`
  - `roastty_app_key` returns `false` for null apps, unmatched events, and
    unprefixed or plain `all:` configured bindings.
  - A `global:` configured app action such as `global:ctrl+x=quit` invokes the
    runtime app action callback once and returns `true`.
  - A `global:` configured surface action dispatches to each live registered
    surface and returns `true`.
  - Stale/detached surfaces are ignored by app-key dispatch.
  - Updating app config replaces the app-level keybind list used by
    `roastty_app_key`.

## Verification

- Add unit coverage for the helper behavior above.
- Run:
  - `cargo test -p roastty app_key`
  - `cargo test -p roastty app_has_global_keybinds`
  - `cargo test -p roastty surface_key_configured_global_all_consume_even_when_unconsumed`
  - `cargo test -p roastty --test abi_harness`
  - `cargo test -p roastty -- --test-threads=1`
  - if the known foreground-PID race fails, rerun
    `cargo test -p roastty -- --test-threads=1 --skip surface_foreground_pid_reports_worker_foreground_pid_after_start`
  - `cargo fmt`
  - `cargo fmt --check`
  - `git diff --check`
  - `prettier --check --prose-wrap always --print-width 80 issues/0802-libroastty-completion-and-mac-app/113-app-key-global-binding-dispatch.md issues/0802-libroastty-completion-and-mac-app/README.md`

## Design Review

Codex-native adversarial review ran in a fresh-context subagent
(`multi_agent_v1.spawn_agent`, agent `019eb6f5-8618-7bd3-83a2-b29b76fe5c58`).

Initial verdict: **Changes required.**

Required findings:

- The first design routed plain `all:` bindings through `roastty_app_key`, but
  upstream app-key routing fans out only `global:` bindings. Plain `all:`
  surface-action fanout belongs to the surface-key path.
- The verification would have locked in that upstream-inconsistent `all:`
  app-key behavior.

Fixes: narrowed the app-key design to `global:` bindings, documented that plain
`all:` remains surface-key-only behavior, replaced the `all:` fanout test with a
`global:` fanout test, and added a negative plain-`all:` app-key expectation.

Re-review verdict: **Approved.** The reviewer reported no remaining required
findings and confirmed the revised design matches upstream's split: app-key
dispatch is limited to `global:` bindings, while plain `all:` fanout belongs to
the surface-key path.

## Result

**Result:** Pass.

Implemented `roastty_app_key` for configured `global:` bindings. The app path
now matches only global keybinds, consumes matched global captures, dispatches
app-scoped runtime actions once to the app target, and fans out surface-scoped
actions across live surfaces registered to the app. Plain `all:` bindings remain
surface-key-only behavior, matching upstream's split between `App.keyEvent` and
`Surface.maybeHandleBinding`.

The implementation skips stale surface registrations whose `surface.app` no
longer matches the app handle, keeps `roastty_surface_key` unchanged, and uses
the current app config after `roastty_app_update_config`.

Verification:

- `cargo test -p roastty app_key` — passed: 5 unit tests, ABI harness filtered
  out.
- `cargo test -p roastty app_has_global_keybinds` — passed: 2 unit tests, ABI
  harness filtered out.
- `cargo test -p roastty surface_key_configured_global_all_consume_even_when_unconsumed`
  — passed.
- `cargo test -p roastty --test abi_harness` — passed with the existing 10
  enum-conversion warnings from the C harness.
- First `cargo test -p roastty -- --test-threads=1` run passed the new app-key
  tests but failed the existing
  `surface_mouse_button_reporting_honors_surface_mouse_reporting_gate` test. The
  failing test passed immediately when rerun in isolation.
- Second `cargo test -p roastty -- --test-threads=1` run passed: 4,630 unit
  tests, the ABI harness, and doc tests.

## Conclusion

The copied app can now hand platform global-key captures to `roastty_app_key`
and have configured `global:` bindings perform through Roastty instead of always
falling back to the app. This closes the first app-level key-dispatch gap while
leaving the larger Phase G items intact: native keymaps, keyboard-layout reload,
multi-key sequences/chords, key tables, default global bindings, full app action
coverage, and plain `all:` surface-path fanout beyond the existing surface
consumption behavior.

## Completion Review

Codex-native adversarial review ran in a fresh-context subagent
(`multi_agent_v1.spawn_agent`, agent `019eb703-2a9a-7793-a449-2956adf56e94`).

Verdict: **Approved.** The reviewer reported no findings.

Evidence checked by the reviewer:

- the result commit had not been made before review;
- the issue README marks Experiment 113 as Pass;
- this experiment has Result and Conclusion sections;
- `roastty_app_key` matches only configured `global:` bindings;
- plain `all:` is negative-tested for the app-key path;
- app actions dispatch once, surface actions fan out to live app-owned surfaces,
  stale detached surfaces are skipped, and app config update replaces the
  keybind list;
- `roastty_surface_key` behavior remains unchanged.

The reviewer independently verified `cargo fmt --check`, `git diff --check`, the
Prettier check for the touched markdown files, the targeted Exp113 tests, the
ABI harness, and the full serial `cargo test -p roastty -- --test-threads=1`
gate.
