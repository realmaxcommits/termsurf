# Experiment 122: Phase G — chained keybinding actions

## Description

Implement upstream-style `chain=` entries for configured keybindings. A normal
keybind entry creates or replaces the current leaf, and a following
`chain=<action>` entry appends another action to that most recently stored leaf.

Upstream models this as `Binding.Set.Value.leaf_chained`: the first action keeps
the trigger and flags, later `chain=` entries add actions to the same leaf, and
the chained leaf dispatches all actions in order. `chain=` is intentionally not
an action that can stand alone: it has no flags, cannot be part of a trigger
sequence, must follow a valid prior keybind leaf, and cannot append `unbind`.

This experiment ports that behavior for Roastty's configured keybinding storage
and surface dispatch paths. It keeps native keymaps/global shortcut
registration, default catalog completion, and app-key sequence/table dispatch
out of scope.

## Changes

- `roastty/src/lib.rs`
  - Add a parsed keybind entry variant for `chain=<action>`.
  - Reject `chain=` with flags (`global:chain=...`), with table prefixes
    (`nav/chain=...`), as part of a sequence (`a>chain=...`), without a valid
    prior leaf, and with `chain=unbind`.
  - Track the most recent chain parent while storing configured keybind entries.
    The parent may be a root direct leaf, a root sequence leaf, a table direct
    leaf, or a table sequence leaf.
  - Extend configured keybind leaves so they can hold one or more action bytes
    while preserving the original trigger and flags from the first action.
  - Remove chained configured leaves from `roastty_config_trigger` reverse
    lookup parity: after a leaf becomes chained, neither its original action nor
    appended actions should reverse-map to that trigger. This matches upstream's
    `leaf_chained` behavior, where chained leaves are excluded from
    `Binding.Set.getTrigger`.
  - When a direct or sequence binding overwrites a previous trigger, replace the
    old action list with the new single action and update the chain parent.
    `unbind` and table clear entries clear the chain parent. If the replacement
    is a non-chained configured root binding, it becomes eligible for
    `roastty_config_trigger` again.
  - Preserve the existing root/table sequence precedence and direct/sequence
    override rules from Experiments 118–121.
  - Dispatch chained configured bindings by performing each action in order. The
    binding is considered performed if any action performs; `ignore` preserves
    its ignored-input behavior regardless of consumed flags; a configured
    `performable:` chained binding falls through only when no action performs,
    matching the existing configured-binding performability rule.
  - Preserve Exp121 sequence-control behavior inside chains:
    - `ignore` drops queued leader bytes and consumes the input.
    - `end_key_sequence` flushes queued leader bytes without encoding the
      triggering key.
  - Keep `roastty_app_key` ignoring chained actions for now if they are not a
    single configured app/surface action; app-key sequence/table handling
    remains later Phase G work.
- Tests in `roastty/src/lib.rs`
  - Parse/store `a=new_window` followed by `chain=new_tab` as one chained leaf.
  - Multiple `chain=` entries append in order.
  - `unconsumed:` and `performable:` flags are preserved from the parent leaf.
  - `roastty_config_trigger` returns the empty trigger for the original and
    appended actions of a chained configured root binding, then returns the
    trigger again after a later non-chained overwrite.
  - `chain=` without a prior leaf, after `unbind`, with flags, in a table
    namespace, in a sequence trigger, or with `unbind` is rejected and leaves
    prior storage unchanged where applicable.
  - Chaining works on root sequence leaves and active-table direct/sequence
    leaves.
  - Direct/sequence overwrites replace prior chained action lists and update the
    chain parent.
  - Surface dispatch runs chained runtime actions in order.
  - Chained text actions write in order to the child pty.
  - `ignore` in a chain consumes even for `unconsumed:` bindings.
  - `end_key_sequence` in a sequence chain flushes queued leader bytes without
    encoding the triggering key.
  - `roastty_app_key` ignores chained configured actions for now.

## Verification

- Run:
  - `cargo test -p roastty chain`
  - `cargo test -p roastty sequence`
  - `cargo test -p roastty key_table`
  - `cargo test -p roastty surface_key`
  - `cargo test -p roastty app_key`
  - `cargo test -p roastty config_trigger`
  - `cargo test -p roastty parse_config_keybind`
  - `cargo test -p roastty --test abi_harness`
  - `cargo test -p roastty -- --test-threads=1`
  - `cargo fmt`
  - `cargo fmt --check`
  - `git diff --check`
  - `prettier --check --prose-wrap always --print-width 80 issues/0802-libroastty-completion-and-mac-app/122-chained-keybind-actions.md issues/0802-libroastty-completion-and-mac-app/README.md`

## Design Review

**Reviewer:** Codex-native adversarial reviewer, fresh context
(`multi_agent_v1.spawn_agent`, agent `019eb7c0-e45b-78a0-bacb-68b405d255ad`)

**Initial verdict:** Changes required

**Required finding:** The original design did not specify upstream reverse
mapping behavior for chained leaves. Upstream removes `leaf_chained` entries
from `Binding.Set.getTrigger`, while Roastty currently implements
`roastty_config_trigger` by scanning configured root bindings.

**Fix:** Added explicit implementation and test requirements that chained
configured root bindings do not reverse-map through `roastty_config_trigger` for
the original or appended action, and that a later non-chained overwrite restores
normal reverse lookup eligibility.

**Final verdict:** Approved

**Final findings:** None.
