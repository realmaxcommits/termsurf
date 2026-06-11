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

## Result

**Result:** Pass

Implemented the input-side `RemapSet` foundation in
`roastty/src/input/key_mods.rs`:

- added `RemapSet` and its `RemapMask` fast-path pre-check;
- added `from=to` parsing for shift/ctrl/alt/super, upstream aliases, optional
  left/right source and target sides, unsided-source expansion, and empty CLI
  reset behavior;
- added finalized right-side-first ordering, one-way non-transitive `apply()`,
  `is_remapped()`, and deterministic formatter output;
- added focused tests for unsided and sided mappings, aliases, reset/error
  cases, right-side ordering, one-way remapping, mask checks, clone/equality,
  order-independent equality, and formatter shape.

The implementation deliberately does not add the `key-remap` config field,
config parser routing, `Config::finalize()` wiring, app C ABI exposure, surface
key-event remapping, menu/main-loop behavior, or OS keyboard-layout integration.

Verification:

1. `cargo test -p roastty key_remap_set` — pass: 9 tests passed; filtered ABI
   harness passed.
2. `cargo test -p roastty key_mods_` — pass: 5 tests passed; filtered ABI
   harness passed.
3. `cargo test -p roastty` — pass before the completion-review fix: 4598 unit
   tests passed; ABI harness passed; doc tests passed. The ABI harness printed
   the known 10 enum-conversion warnings.
4. `cargo test -p roastty tests::surface_foreground_pid_reports_worker_foreground_pid_after_start`
   — pass after the completion-review fix: 1 test passed; filtered ABI harness
   passed.
5. `cargo test -p roastty -- --test-threads=1` — pass after the
   completion-review fix: 4599 unit tests passed; ABI harness passed; doc tests
   passed. The ABI harness printed the known 10 enum-conversion warnings.

## Conclusion

The reusable modifier-remap data structure is now available for the later
`key-remap` config slice. The next work can add config storage, parsing,
formatting, and finalization without also porting the core remapping algorithm.

## Completion Review

Codex-native adversarial review ran in fresh context with subagent
`019eb674-1839-7422-bd83-b22c335e7095`.

Initial verdict: **CHANGES REQUIRED**

Findings and fixes:

- Required: `RemapSet` equality was order-sensitive because it derived
  `PartialEq`/`Eq` over the ordered mapping vector, unlike upstream's
  key/value-content comparison. Fixed by implementing manual order-independent
  `PartialEq`/`Eq` for `RemapSet`.
- Required: tests did not cover the upstream order-independent equality
  semantics. Fixed by adding `key_remap_set_equality_is_order_independent`.
- Required: the reviewer's independent full `cargo test -p roastty` run hit
  `tests::surface_foreground_pid_reports_worker_foreground_pid_after_start`,
  while the same test passed alone, suggesting an unrelated parallel-test flake.
  The same parallel-only failure reproduced after the equality fix, the failing
  test passed alone, and the full roastty suite passed serially with
  `cargo test -p roastty -- --test-threads=1`.

The first re-review still required a stronger equality test because the initial
order-independent equality test used one left-side mapping and one right-side
mapping, allowing `finalize()` to normalize the vector order. Fixed by changing
the test to compare same-priority left-side mappings inserted in opposite order.

Final re-review ran in fresh context with subagent
`019eb682-c34a-7561-b788-5d92e3d5f9bb`.

Final verdict: **APPROVED**

Findings: None.
