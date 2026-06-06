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

# Experiment 743: Config Default Trigger Menu Actions

## Description

Experiment 742 added the first upstream-compatible default trigger lookups for
`open_config` and `reload_config`. Upstream Ghostty's default keybind set also
contains menu-visible non-performable defaults for clipboard actions, font-size
actions, and `write_screen_file` copy/paste/open actions. These are exposed
through the same `config_trigger` reverse lookup and are already represented by
Roastty's public key/modifier enums and binding-action string parser.

This experiment expands the built-in default trigger lookup to those macOS-only
menu actions. It still does not add user keybind parsing, keybind storage, key
tables, sequences, `roastty_config_key_is_binding`, or surface key dispatch.

## Changes

- `roastty/src/lib.rs`
  - Extend `default_config_trigger` with upstream macOS default triggers:
    - `copy_to_clipboard` and `copy_to_clipboard:mixed` return physical
      `ROASTTY_KEY_COPY` with no modifiers.
    - `paste_from_clipboard` returns physical `ROASTTY_KEY_PASTE` with no
      modifiers.
    - `increase_font_size:1` returns unicode `+` with `ROASTTY_MODS_SUPER`. This
      follows upstream's reverse-map behavior where the later `+` binding wins
      over the earlier `=` binding for the same action.
    - `decrease_font_size:1` returns unicode `-` with `ROASTTY_MODS_SUPER`.
    - `reset_font_size` returns unicode `0` with `ROASTTY_MODS_SUPER`.
    - `write_screen_file:copy` returns unicode `j` with
      `ROASTTY_MODS_SHIFT | ROASTTY_MODS_CTRL | ROASTTY_MODS_SUPER`.
    - `write_screen_file:paste` returns unicode `j` with
      `ROASTTY_MODS_SHIFT | ROASTTY_MODS_SUPER`.
    - `write_screen_file:open` returns unicode `j` with
      `ROASTTY_MODS_SHIFT | ROASTTY_MODS_ALT | ROASTTY_MODS_SUPER`.
  - Keep formatted write-file variants such as `write_screen_file:copy,html` and
    non-default clipboard formats such as `copy_to_clipboard:plain` and
    `copy_to_clipboard:html` on the empty trigger because upstream default
    keybinds only bind the mixed/plain default action forms listed above.
  - Keep performable defaults such as command-C / command-V and shift-arrow
    selection adjustments out of the reverse lookup, preserving Experiment 742's
    empty-trigger behavior for performable actions.
  - Keep malformed action strings and unsupported actions returning the empty
    trigger.
  - Keep `roastty_config_key_is_binding` unchanged.

- `roastty/tests/abi_harness.c`
  - Add C ABI checks for each new default trigger.
  - Keep empty-trigger checks for formatted write-screen variants, non-default
    clipboard formats, parameterized malformed forms, and performable
    `adjust_selection:left`.

- Tests in `roastty/src/lib.rs`
  - Cover every new physical and unicode default trigger.
  - Cover default aliases such as `copy_to_clipboard` and
    `copy_to_clipboard:mixed`.
  - Cover non-default clipboard formats, non-default formatted write-screen
    variants, and malformed parameterized forms returning the empty trigger.
  - Keep existing `config_trigger`, `config_key_is_binding`, binding-action, and
    ABI harness tests passing.

## Verification

Run:

- `cargo fmt -p roastty`
- `cargo test -p roastty config_trigger -- --nocapture --test-threads=1`
- `cargo test -p roastty config_key_is_binding -- --nocapture --test-threads=1`
- `cargo test -p roastty binding_action -- --nocapture --test-threads=1`
- `cargo test -p roastty --test abi_harness`
- `cargo fmt -p roastty -- --check`
- `git diff --check`

## Design Review

Codex reviewed the Experiment 743 design and found no technical blockers. The
review approved the narrow scope: expand only the built-in default reverse
lookup, without adding keybind storage, config parsing, key tables, sequences,
key dispatch, or `roastty_config_key_is_binding`.

The review confirmed the listed macOS defaults are consistent with upstream's
default keybind reverse-map behavior: physical Copy/Paste for clipboard menu
actions, command-plus/minus/zero for font size, `increase_font_size:1` resolving
to the later `+` binding, and command-shift `j` variants for `write_screen_file`
copy/paste/open. It also confirmed performable bindings should stay excluded
from the reverse lookup.

The review requested explicit empty-trigger checks for non-default clipboard
formats such as `copy_to_clipboard:plain` and `copy_to_clipboard:html`; the plan
now includes those cases.

The review initially raised a stale process concern that Experiment 742 still
needed completion-review metadata and a result commit. Current git history shows
Experiment 742 has both required commits:
`8a334b9d14860 Teach commas their doors` and
`37732b91e34ee Let commas find the menu`. No Experiment 742 blocker remains.

The remaining workflow requirement from the review was to record
`[review.design]`, this review section, and the README tuple before the
Experiment 743 plan commit; those records are now present.

## Result

**Result:** Pass

Experiment 743 expanded `roastty_config_trigger` with the next
upstream-compatible macOS default menu-action triggers. The built-in default
lookup now returns:

- physical Copy with no modifiers for `copy_to_clipboard` and
  `copy_to_clipboard:mixed`;
- physical Paste with no modifiers for `paste_from_clipboard`;
- command-plus for `increase_font_size:1`, preserving upstream's reverse-map
  behavior where the later `+` binding wins over the earlier `=` binding;
- command-minus for `decrease_font_size:1`;
- command-zero for `reset_font_size`;
- command-control-shift-`j` for `write_screen_file:copy`;
- command-shift-`j` for `write_screen_file:paste`;
- command-option-shift-`j` for `write_screen_file:open`.

Non-default clipboard formats (`copy_to_clipboard:plain`,
`copy_to_clipboard:html`), non-default formatted write-screen actions
(`write_screen_file:copy,html`, `write_screen_file:paste,vt`,
`write_screen_file:open,plain`), malformed config actions, unknown actions, and
performable actions continue to return the empty physical-unidentified trigger.
`roastty_config_key_is_binding` remains unchanged.

The C ABI harness now checks the same default and empty-trigger behavior at the
public C boundary.

Verification passed:

- `cargo fmt -p roastty`
- `cargo test -p roastty config_trigger -- --nocapture --test-threads=1`
  - 3 passed
- `cargo test -p roastty config_key_is_binding -- --nocapture --test-threads=1`
  - 1 passed
- `cargo test -p roastty binding_action -- --nocapture --test-threads=1`
  - 129 passed
- `cargo test -p roastty --test abi_harness`
  - 1 passed
- `cargo fmt -p roastty -- --check`
- `git diff --check`

## Conclusion

Roastty's default trigger ABI now covers the first menu-visible macOS keybinds
needed by the frontend: config open/reload, clipboard copy/paste, font size, and
write-screen-file actions. The remaining keybind work is to move from this
static default lookup toward a real default keybind set, user keybind
parsing/storage, key-event lookup, key tables, sequences, and dispatch.

## Completion Review

Codex reviewed the completed Experiment 743 implementation and result diff. It
found no implementation blockers.

The review confirmed that the default trigger mappings match the approved macOS
menu-action plan: physical Copy/Paste for clipboard menu actions,
command-plus/minus/zero for font sizing, and the command-shift `j` variants for
`write_screen_file` copy/paste/open. It also confirmed that non-default
clipboard formats, formatted write-screen variants, malformed strings, unknown
actions, and performable actions continue to return the empty
physical-unidentified trigger. Rust tests and ABI harness coverage both exercise
the new defaults and empty-trigger fallbacks.

The review's only blocker was missing workflow metadata: `[review.result]`, this
completion-review section, and the README tuple update. Those records are now
present.
