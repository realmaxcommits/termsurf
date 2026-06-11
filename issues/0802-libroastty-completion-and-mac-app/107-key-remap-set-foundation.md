# Experiment 107: Phase G — key-remap set foundation

## Description

Port the core input-side modifier remapping data structure that upstream uses
for `key-remap` before wiring the config field itself.

Upstream declares `key-remap` in `Config.zig` as an `input/key_mods.zig`
`RemapSet`, then calls `self.@"key-remap".finalize()` at the end of
`Config.finalize()`. Roastty currently has `Mods`, `Mod`, `Side`, and modifier
translation helpers, but it does not yet have `RemapSet`, its `Mask`, or the
parse/finalize/apply behavior needed for the later config-finalize slice.

This experiment should port the reusable `RemapSet` core in
`roastty/src/input/key_mods.rs` so the next config experiment can add the
`key-remap` field and call `finalize()` without also introducing the full
remapping algorithm.

This is an input data-structure slice only. It must not add the `key-remap`
config field, `Config::set` parsing, config formatting, app C ABI exposure,
surface key-event remapping, menu/main-loop behavior, or OS keyboard-layout
integration.

## Changes

- `roastty/src/input/key_mods.rs`
  - Add a `RemapSet` type that stores ordered mappings from `Mods` to `Mods` and
    a `Mask` fast pre-check matching upstream's `RemapSet.Mask`.
  - Add parse support for one `from=to` remap string:
    - accepted modifiers: `shift`, `ctrl`, `alt`, `super`;
    - accepted aliases matching upstream: `control`/`cmd`/`command`/`opt`/
      `option`;
    - optional sides: `left_` and `right_`;
    - unsided `from` expands to both left and right sides;
    - unsided `to` defaults to left, matching upstream's current app runtime
      modifier layout.
  - Add empty-value reset behavior for a future config parser entry point.
  - Add `finalize()` ordering so mappings with right-side modifiers are checked
    before left-side mappings, matching upstream's sorted map behavior.
  - Add `is_remapped()` and `apply()` with upstream semantics:
    - return the original modifiers when the mask does not match;
    - apply the first finalized mapping whose modifier keys and sides match;
    - perform one-way, non-transitive remaps.
  - Add formatter support for the remap set so a later config field can reuse
    it.
  - Add focused tests ported from upstream examples proving:
    - unsided remaps create both left and right mappings;
    - sided `from` maps only that side;
    - sided `to` preserves the target side;
    - multiple parses accumulate;
    - empty reset clears mappings and mask;
    - missing assignment and invalid modifier aliases error;
    - `finalize()` orders right-side mappings first;
    - `is_remapped()` uses the mask correctly;
    - `apply()` is one-way and non-transitive;
    - clone/equality and formatter output are deterministic.

## Verification

Pass criteria:

1. `cargo test -p roastty key_remap_set`
2. `cargo test -p roastty key_mods_`
3. `cargo test -p roastty`
4. `cargo fmt --check`
5. `git diff --check`
6. `prettier --check --prose-wrap always --print-width 80 issues/0802-libroastty-completion-and-mac-app/107-key-remap-set-foundation.md issues/0802-libroastty-completion-and-mac-app/README.md`

The full `cargo test -p roastty` run must pass. The existing ABI harness may
print its known enum-conversion warnings, but no new failures are acceptable.

## Design Review

Codex-native adversarial review ran in fresh context with subagent
`019eb66a-0c5f-73c2-86a1-7b1867eb33df`.

Verdict: **APPROVED**

Findings: None.
