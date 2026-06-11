# Experiment 112: Phase G — default binding table foundation

## Description

Replace Roastty's split, ad hoc default-keybinding lookup with a shared macOS
default-binding table foundation.

Upstream `Config.Keybinds.init` builds an ordered default binding set. That
order matters for two separate behaviors:

- runtime key handling uses the default set when no user-configured binding
  overrides the key;
- `ghostty_config_trigger` performs reverse action-to-trigger lookup and returns
  the last trigger registered for an action, which the macOS app uses for menu
  shortcut labels.

Roastty currently duplicates this data in two forms:

- `default_config_trigger` is a hardcoded action-to-trigger match;
- `default_physical_key_binding` / `default_unicode_key_binding` separately
  hardcode runtime key-to-action matching.

This experiment introduces a single table for the existing macOS default
single-key bindings and rewires both directions to use it. It does not add new
actions, multi-key sequences/chords, key tables, non-macOS default bindings,
global/all app routing, `roastty_app_key`, native keymaps, command-palette
catalog data, or the remaining upstream binding table entries that require
unported actions.

## Changes

- `roastty/src/lib.rs`
  - Add a compact static/default table representation for one-trigger,
    one-action bindings:
    - trigger kind: physical key or unicode codepoint;
    - modifier mask;
    - action bytes;
    - keybind flag byte.
  - Populate the table with the macOS default bindings that Roastty already
    supports today, preserving upstream insertion order for duplicate
    action-to-trigger cases:
    - config open/reload;
    - copy/paste and paste-from-selection;
    - font-size controls;
    - write-screen-file actions;
    - tab/window/split/search/navigation actions already supported by
      `parse_binding_action`;
    - macOS natural text-editing defaults (`text:\x01`, `text:\x05`,
      `text:\x15`, `esc:b`, `esc:f`).
  - Change `default_config_trigger` to scan the shared table in reverse order
    for the requested action, skipping rows with
    `ROASTTY_KEYBIND_FLAG_PERFORMABLE`, and return the matching trigger. This
    should preserve upstream's "last non-performable binding wins for menu
    labels" behavior without a separate action match.
  - Change `default_key_event_binding` to find the first matching table entry
    for physical-key and unicode/unshifted-codepoint events, carrying the entry
    flags and action through the existing `DefaultBindingMatch` path.
  - Remove or shrink the old duplicated hardcoded default trigger/action matches
    after the shared table covers them.
  - Keep configured keybind precedence unchanged: configured bindings still
    shadow default bindings before the default table is consulted.

## Verification

- Add unit coverage for the table and both lookup directions:
  - every table row has a non-empty action and a valid trigger;
  - reverse lookup returns the last table row for duplicate actions, including
    `increase_font_size:1` returning `super++` rather than `super+=`;
  - runtime default lookup and reverse lookup agree for representative
    non-performable rows: write-screen-file, goto tab, split navigation, resize
    split, and natural text editing;
  - reverse lookup skips performable default rows such as
    `copy_to_clipboard:mixed`, `paste_from_clipboard`, `start_search`,
    `end_search`, and `navigate_search:*`, while runtime lookup still uses them;
  - configured keybinds still override default table matches in
    `roastty_config_trigger` and `Surface::key`;
  - default binding flag behavior remains unchanged for performable and consumed
    bindings.
- Run:
  - `cargo test -p roastty default_config_trigger`
  - `cargo test -p roastty surface_key`
  - `cargo test -p roastty keybind`
  - `cargo test -p roastty -- --test-threads=1`
  - if the known foreground-PID race fails, rerun
    `cargo test -p roastty -- --test-threads=1 --skip surface_foreground_pid_reports_worker_foreground_pid_after_start`
  - `cargo fmt`
  - `cargo fmt --check`
  - `git diff --check`
  - `prettier --check --prose-wrap always --print-width 80 issues/0802-libroastty-completion-and-mac-app/112-default-binding-table-foundation.md issues/0802-libroastty-completion-and-mac-app/README.md`

## Design Review

Codex-native adversarial review ran in a fresh-context subagent
(`multi_agent_v1.spawn_agent`, agent `019eb6d9-6fe9-7522-baf9-11cbf3c9c899`).

Initial verdict: **Changes required.**

Required findings:

- Reverse lookup would have exposed performable default bindings as menu
  shortcuts. The design now specifies that `default_config_trigger` skips rows
  with `ROASTTY_KEYBIND_FLAG_PERFORMABLE`, and verification must assert
  performable defaults return an empty reverse trigger unless user-configured.
- Verification omitted the required `cargo fmt` command before
  `cargo fmt --check`.

Fixes: updated the plan and verification for both findings.

Re-review verdict: **Approved.** The reviewer reported no remaining required
findings.

## Result

**Result:** Pass.

Implemented a shared default-binding table for Roastty's existing macOS
single-key default bindings. Runtime default lookup and reverse
action-to-trigger lookup now both scan the same ordered table, while preserving
the existing alias behavior for `close_tab`/`close_tab:this` and
`copy_to_clipboard`/`copy_to_clipboard:mixed`.

The reverse lookup scans from the end of the table and skips rows marked
`ROASTTY_KEYBIND_FLAG_PERFORMABLE`. This means performable-only defaults such as
search navigation no longer appear as menu-label shortcuts, while actions that
also have a separate non-performable default row, such as copy/paste, still
reverse-map to that non-performable trigger. Configured keybind precedence was
kept unchanged, and the runtime table now includes the Shift+Insert
`paste_from_selection` default that the old physical-key ladder covered.

Verification:

- `cargo test -p roastty default_config_trigger` — passed.
- `cargo test -p roastty surface_key` — passed.
- `cargo test -p roastty keybind` — passed.
- `cargo test -p roastty config_key_is_binding_matches_default_physical_keys` —
  passed.
- `cargo test -p roastty --test abi_harness` — passed with the existing enum
  conversion warnings from the C harness.
- `cargo test -p roastty -- --test-threads=1` — passed: 4,625 unit tests, the
  ABI harness, and doc tests.

## Conclusion

The default keybinding data now has a single source for the macOS single-key
subset that Roastty already supports. That removes the old drift risk between
runtime key handling and app-facing reverse shortcut labels, and leaves the next
Phase G work clearly bounded: multi-key sequences/chords, key tables, non-macOS
defaults, global/all app routing, `roastty_app_key`, native keymaps, the
command-palette catalog, and the remaining upstream default bindings that depend
on unported actions.

## Completion Review

Codex-native adversarial review ran in a fresh-context subagent
(`multi_agent_v1.spawn_agent`, agent `019eb6ef-3d4e-7ca1-8238-3ebc873dd69f`).

Verdict: **Approved.** The reviewer reported no required findings.

Evidence checked by the reviewer:

- the shared table drives both reverse lookup and runtime default lookup;
- reverse lookup scans in reverse and skips performable rows;
- configured bindings still precede defaults;
- the ABI harness asserts performable search defaults reverse-map to the empty
  trigger;
- this experiment has Result and Conclusion sections, the issue README marks
  Experiment 112 as Pass, and the result commit had not been made before review.

The reviewer independently verified `cargo fmt --check`, `git diff --check`, the
Prettier check for the touched markdown files, targeted Roastty tests, and the
full serial `cargo test -p roastty -- --test-threads=1` gate.
