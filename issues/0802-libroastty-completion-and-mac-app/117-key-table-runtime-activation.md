# Experiment 117: Phase G — key-table runtime activation

## Description

Wire the first runtime slice of upstream key-table behavior for Roastty's
configured single-key surface bindings.

Experiment 116 added parser/storage support for `table-name/trigger=action`, but
the stored table bindings remain inert. Upstream Ghostty activates named tables
through keybinding actions such as `activate_key_table:<name>` and then searches
active tables from innermost to outermost before falling back to the root
binding set. This experiment makes that stored table data usable for surface key
handling while keeping the scope deliberately narrower than full upstream
keybinding parity.

This experiment does not implement multi-key sequences/chords, leader keys,
`chain=`, table-local chains, `ignore`, native keymaps, native global shortcut
registration, app-level table handling in `roastty_app_key`, or the full
upstream default binding catalog.

## Changes

- `roastty/src/lib.rs`
  - Add surface-owned active key-table stack state with upstream's bounded depth
    of 8 entries.
    - Each stack entry stores the table name and whether it was activated in
      one-shot mode.
    - The stack is per surface, not global app state.
  - Add helpers to look up configured table bindings by table name from the
    owning `App`'s cloned `keybind_tables`.
  - Add configured binding lookup for surface key paths in this order:
    1. active tables as complete sets, innermost to outermost: exact binding
       first, then that same table's `catch_all` fallback;
    2. root configured exact bindings;
    3. built-in default exact bindings;
    4. root configured catch-all bindings.
  - Keep built-in defaults separate from configured bindings. Since Roastty does
    not have upstream's unified binding trie yet, this preserves Exp 115's rule
    that root configured catch-all bindings do not shadow exact built-in
    defaults. Active table catch-all bindings are table-local and intentionally
    shadow root/default bindings while the table is active, matching upstream's
    table-before-root lookup.
  - Extend `ParsedBindingAction` and `parse_config_binding_action` for:
    - `activate_key_table:<name>`;
    - `activate_key_table_once:<name>`;
    - `deactivate_key_table`;
    - `deactivate_all_key_tables`.
  - Implement surface action behavior for those parsed actions:
    - activating a missing table returns false and leaves the stack unchanged;
    - activating the currently innermost table returns false and leaves the
      stack unchanged;
    - activating a table when the stack already has 8 entries returns false and
      leaves the stack unchanged;
    - otherwise push the table and notify the app with
      `ROASTTY_ACTION_KEY_TABLE` / `ROASTTY_KEY_TABLE_ACTIVATE`;
    - `deactivate_key_table` pops one active table, notifies
      `ROASTTY_KEY_TABLE_DEACTIVATE`, and returns true only when a table was
      active;
    - `deactivate_all_key_tables` clears all active tables, notifies
      `ROASTTY_KEY_TABLE_DEACTIVATE_ALL`, and returns true only when any table
      was active.
  - Add the existing key-table C ABI payload to `action_u_from_storage` and the
    test-only inverse so runtime action callbacks receive a typed
    `roastty_action_key_table_s`.
  - Preserve existing root/default binding behavior for surfaces when no table
    is active.
  - Keep `Config::key_event_is_binding` and `roastty_app_key` scoped to root
    configured/default behavior in this experiment; active tables are
    surface-local runtime state.
- `roastty/tests/abi_harness.c`
  - Add C ABI coverage for a surface using CLI key-table config:
    - inactive table bindings do not match root binding checks;
    - root `activate_key_table:<name>` activates a table through
      `roastty_surface_key_handle`;
    - a table binding then matches `roastty_surface_key_is_binding_handle` and
      dispatches through `roastty_surface_key_handle`;
    - `deactivate_key_table` removes the table and restores root-only behavior.
- Tests in `roastty/src/lib.rs`
  - Parsing/canonicalization for the four key-table actions.
  - Active table exact bindings dispatch before root configured bindings and
    built-in defaults.
  - Root exact bindings remain available when active tables do not match.
  - Active table catch-all bindings shadow root exact bindings and exact
    built-in defaults while the table is active.
  - `roastty_surface_key_is_binding` and the handle variant consult active
    tables and return the table binding's flags.
  - `activate_key_table_once` pops the innermost table after any valid binding
    from that table is invoked, including table `catch_all` bindings.
  - Duplicate activation of the current innermost table is ignored.
  - The same table can appear again when it is not currently innermost.
  - The active table stack caps at 8.
  - `deactivate_key_table` and `deactivate_all_key_tables` return false/noop
    when no table is active, and true with the correct app action notification
    when they change active state.
  - `roastty_app_update_config` refreshes table storage used by existing
    surfaces without preserving removed table bindings as usable runtime data.

## Verification

- Run:
  - `cargo test -p roastty key_table`
  - `cargo test -p roastty parse_config_keybind`
  - `cargo test -p roastty surface_key`
  - `cargo test -p roastty app_key`
  - `cargo test -p roastty --test abi_harness`
  - `cargo test -p roastty -- --test-threads=1`
  - `cargo fmt`
  - `cargo fmt --check`
  - `git diff --check`
  - `prettier --check --prose-wrap always --print-width 80 issues/0802-libroastty-completion-and-mac-app/117-key-table-runtime-activation.md issues/0802-libroastty-completion-and-mac-app/README.md`

## Design Review

Codex-native adversarial review ran in a fresh-context subagent
(`multi_agent_v1.spawn_agent`, agent `019eb745-918d-75a1-b69a-de75d6939fa1`).

Initial verdict: **Changes required.** The reviewer found two upstream-fidelity
problems in the first design:

- Active table `catch_all` bindings were planned after root/default exact
  bindings, but upstream evaluates each active table as a complete binding set
  before falling back to root bindings.
- `activate_key_table_once` was planned to ignore table `catch_all` matches for
  one-shot deactivation, but upstream deactivates the current one-shot table
  after any valid binding from that table, including `catch_all`.

Fix: the design now evaluates active tables innermost-to-outermost as complete
sets before root/default lookup, explicitly requires active table `catch_all` to
shadow root/default bindings, and requires one-shot tables to pop after table
`catch_all` matches.

Final verdict after re-review: **Approved.** The reviewer confirmed both prior
findings were resolved and reported no new required findings.

## Completion Review

Codex-native adversarial review ran in a fresh-context subagent
(`multi_agent_v1.spawn_agent`, agent `019eb758-8d09-7a43-acde-0334d36b76d4`).

Verdict: **Approved.** The reviewer reported no required, optional, or nit
findings. It independently confirmed the result was still uncommitted, reran the
verification suite successfully, and confirmed the implementation matches the
approved Exp117 scope while keeping app-key table behavior out of scope.

## Result

**Result:** Pass

Roastty now activates configured key tables on surface key paths. Each surface
owns an active table stack capped at 8 entries, table activation stores table
names rather than borrowing app config storage, and lookup resolves against the
current `App` table storage so `roastty_app_update_config` refreshes usable
table bindings for existing surfaces.

Implemented behavior:

- `activate_key_table:<name>` activates an existing table and ignores missing
  tables, duplicate activation of the current innermost table, and stack-depth
  overflow.
- `activate_key_table_once:<name>` activates a one-shot table and pops it after
  any valid binding from that table, including table-local `catch_all`.
- `deactivate_key_table` pops one active table; `deactivate_all_key_tables`
  clears the stack; both return false/noop when no table is active.
- Active tables are searched innermost-to-outermost as complete sets, so a
  table-local exact or `catch_all` binding shadows root configured bindings and
  built-in defaults while the table is active.
- Root configured exact bindings, built-in exact defaults, and root configured
  `catch_all` continue to behave as before when no active table matches.
- `roastty_surface_key_is_binding` and the handle variant consult active tables
  and return table binding flags.
- `ROASTTY_ACTION_KEY_TABLE` now carries the typed C ABI payload for app
  activation/deactivation notifications.
- `roastty_app_key` deliberately ignores key-table actions in this slice, so
  table runtime behavior remains surface-local.

Verification run:

- `cargo test -p roastty key_table` — pass
- `cargo test -p roastty parse_config_keybind` — pass
- `cargo test -p roastty surface_key` — pass
- `cargo test -p roastty app_key` — pass
- `cargo test -p roastty --test abi_harness` — pass
  - The harness still emits the existing enum-conversion warnings in unrelated
    mouse/inspector calls.
- `cargo test -p roastty -- --test-threads=1` — pass
  - 4,654 unit tests passed.
  - ABI harness passed.
  - Doc tests passed.
- `cargo fmt --check` — pass
- `git diff --check` — pass
- `prettier --check --prose-wrap always --print-width 80 issues/0802-libroastty-completion-and-mac-app/117-key-table-runtime-activation.md issues/0802-libroastty-completion-and-mac-app/README.md`
  — pass

## Conclusion

The first key-table runtime slice is complete for configured single-key surface
bindings. Remaining Phase G keybinding work is now narrower: multi-key
sequences/chords, `chain=`, `ignore`, native keymaps/global shortcuts,
app-key-level table behavior, and the full upstream binding/action catalog.
