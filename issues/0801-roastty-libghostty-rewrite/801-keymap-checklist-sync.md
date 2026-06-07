+++
[implementer]
agent = "codex"
model = "gpt-5"
reasoning = "high"

[review.design]
agent = "codex"
model = "default"
reasoning = "medium"

[review.result]
agent = "codex"
model = "default"
reasoning = "medium"
+++

# Experiment 801: Keymap Checklist Sync

## Description

Issue 801's input checklist currently says keymaps (`keycodes`, `function_keys`,
`KeymapDarwin`, layouts) are missing. That is too broad: Roastty already has the
key enum/value layer, the upstream keyboard layout enum and Apple-layout ID
mapping, and the legacy/Kitty function-key encoding tables inside
`key_encode.rs`.

This experiment updates the checklist wording only. It should keep the row
unchecked because Roastty still lacks the macOS `KeymapDarwin` equivalent that
loads the active input source, translates native keycodes through Carbon, tracks
dead-key state, and reloads when the keyboard layout changes.

## Changes

- `issues/0801-roastty-libghostty-rewrite/README.md`
  - Update the keymaps row from "missing" to partial wording that names the
    existing key enum, layout mapping, and function-key encoding foundations.
  - Keep `KeymapDarwin` native keycode/layout translation, input-source reload,
    dead-key handling, and frontend integration open.
  - Add the Experiment 801 index entry.
- `issues/0801-roastty-libghostty-rewrite/801-keymap-checklist-sync.md`
  - Record verification evidence and review results.

## Verification

- Inspect:
  - `roastty/src/input/key.rs`
  - `roastty/src/input/keyboard.rs`
  - `roastty/src/input/key_encode.rs`
  - `vendor/ghostty/src/input/keycodes.zig`
  - `vendor/ghostty/src/input/keyboard.zig`
  - `vendor/ghostty/src/input/function_keys.zig`
  - `vendor/ghostty/src/input/KeymapDarwin.zig`
- Run:
  - `cargo test -p roastty key_ -- --nocapture --test-threads=1`
  - `cargo test -p roastty keyboard -- --nocapture --test-threads=1`
  - `cargo test -p roastty key_encode_legacy_completed_cursor_edit_and_function_tables -- --nocapture --test-threads=1`
  - `cargo test -p roastty key_encode_kitty_table_covers_upstream_supported_entries -- --nocapture --test-threads=1`
- Run markdown formatting:
  - `prettier --write --prose-wrap always --print-width 80 issues/0801-roastty-libghostty-rewrite/README.md issues/0801-roastty-libghostty-rewrite/801-keymap-checklist-sync.md`
- Run:
  - `git diff --check`

The experiment passes if the row accurately records the existing key/layout/
function-key foundations while remaining unchecked for `KeymapDarwin`. It is
Partial if only some evidence can be verified. It fails if the row should remain
an undifferentiated missing item.

## Design Review

Codex reviewed the design and initially found one blocking verification gap: the
plan claimed a key enum/value foundation but did not run a focused `key.rs`
test. After adding `cargo test -p roastty key_ -- --nocapture --test-threads=1`
to cover key enum boundaries, W3C round trips, ASCII/codepoint mapping, keypad
detection, and macOS modifier helpers, Codex re-reviewed the design and found a
second blocking inspection gap: the plan claimed upstream keyboard layout parity
but did not inspect `vendor/ghostty/src/input/keyboard.zig`. After adding that
source file to the inspection list, the design requires re-review before the
plan commit. Codex then re-reviewed the corrected design and approved it with no
blocking findings. The approval noted one non-blocking wording guard for
implementation: keep the checklist precise that Roastty has key enum/value,
layout mapping, and function-key encoding foundations, but still lacks the
native macOS keycode translation table/runtime path owned by `KeymapDarwin`.
