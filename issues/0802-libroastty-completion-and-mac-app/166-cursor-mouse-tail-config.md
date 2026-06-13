# Experiment 166: Phase F — cursor mouse tail config

## Description

Remove `cursor-click-to-move` and `mouse-hide-while-typing` from the remaining
Phase F public-config tail.

Both are upstream bool config fields. This experiment wires their
parser/formatter/storage behavior and keeps runtime behavior out of scope:
prompt-click cursor movement and platform mouse hiding remain separate
terminal/app integration work.

## Changes

- `roastty/src/config/mod.rs`
  - Add `Config` fields `cursor_click_to_move` and `mouse_hide_while_typing`.
  - Use upstream defaults: `cursor-click-to-move = true` and
    `mouse-hide-while-typing = false`.
  - Format both fields in upstream declaration order immediately after
    `cursor-text` and before `scroll-to-bottom`.
  - Route `Config::set` for both keys using upstream bool semantics:
    bare/missing bool values set `true`, empty values reset to defaults, and
    invalid bool strings report `InvalidValue`.
  - Update config field-order/default tests and add focused
    parse/format/reset/load/clone coverage.

- `issues/0802-libroastty-completion-and-mac-app/README.md`
  - Mark Experiment 166 as `Designed`.
  - After result, update the Phase F remaining-public-options count from 24 to
    22 and remove cursor-click/mouse-hide wording if this passes.

## Verification

Before implementation:

- Codex-native adversarial design review approves this experiment.
- Commit the reviewed plan separately from the result.

After implementation:

- `cargo test -p roastty cursor_mouse_tail_config`
- `cargo test -p roastty config_format_config_emits_fields_in_upstream_order`
- `cargo test -p roastty`
- `cargo fmt --check -p roastty`
- `prettier --check --prose-wrap always --print-width 80 issues/0802-libroastty-completion-and-mac-app/166-cursor-mouse-tail-config.md issues/0802-libroastty-completion-and-mac-app/README.md`
- `git diff --check`

**Pass** = both keys parse, format, reset, load, and report diagnostics with
upstream defaults/order/bool semantics, and the full roastty test suite passes.

**Partial** = the direct parser/formatter fields land, but ordering, load/replay
behavior, diagnostics, or full-suite verification remains incomplete.

**Fail** = the fields cannot be added without conflicting with existing cursor,
mouse, or config storage behavior.

## Design Review

**Reviewer:** Codex-native adversarial review subagent `Peirce`, fresh context.

**Verdict:** Approved with no findings.

The reviewer verified that the README links Experiment 166 as `Designed`, the
experiment has the required sections, upstream default/order semantics match
`Config.zig`, local bool helper semantics match the plan, and no implementation
was done before design review.

## Result

**Result:** Pass

Roastty now stores, parses, and formats `cursor-click-to-move` and
`mouse-hide-while-typing` as upstream bool config fields. The defaults match
pinned upstream Ghostty: cursor click-to-move enabled, mouse hide-while-typing
disabled.

The formatter places both fields in upstream order after `cursor-text` and
before `scroll-to-bottom`. The parser uses the existing upstream-style bool
semantics: a bare key sets `true`, an empty value resets to the default, and an
invalid value reports `InvalidValue`.

The Phase F public-config tail is now 22 keys: language, font
feature/variation/metric/freetype knobs, `input`, and `keybind`.

Verification:

- `cargo test -p roastty cursor_mouse_tail_config` passed 1 filtered unit test
  plus the ABI harness filter.
- `cargo test -p roastty config_format_config_emits_fields_in_upstream_order`
  passed 1 filtered unit test plus the ABI harness filter.
- `cargo test -p roastty` passed 4,862 Rust unit tests, 0 failed, 4 ignored; the
  C ABI harness passed with the existing enum-conversion warnings; doc tests
  passed with 0 tests.
- `cargo fmt --check -p roastty` passed.
- `prettier --check --prose-wrap always --print-width 80 issues/0802-libroastty-completion-and-mac-app/166-cursor-mouse-tail-config.md issues/0802-libroastty-completion-and-mac-app/README.md`
  passed.
- `git diff --check` passed.

## Completion Review

**Reviewer:** Codex-native adversarial review subagent `Halley`, fresh context.

**Verdict:** Approved with no findings.

The reviewer verified that the diff is limited to the experiment doc, issue
README, and `roastty/src/config/mod.rs`; the result commit had not been made;
upstream defaults/order match `Config.zig`; implementation stayed within
parser/formatter/storage/tests/docs scope; and all claimed checks passed.

## Conclusion

The cursor/mouse bool config tail is complete at the parser/formatter/storage
layer. Prompt-click cursor movement and platform mouse hiding remain
terminal/app runtime behavior, not missing public config fields.
