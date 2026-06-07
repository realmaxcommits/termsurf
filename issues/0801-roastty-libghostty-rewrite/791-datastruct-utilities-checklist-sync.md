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

# Experiment 791: Datastruct Utilities Checklist Sync

## Description

The Issue 801 supporting-subsystems checklist still leaves `CircBuf`,
`IntrusiveLinkedList`, and other utility datastructures unchecked "as needed."
The current Roastty tree already contains the named utilities and several
adjacent terminal collection helpers under `roastty/src/terminal/`:
`circ_buf.rs`, `intrusive_linked_list.rs`, `array_list_collection.rs`,
`cache_table.rs`, `lru.rs`, and `segmented_pool.rs`.

This experiment verifies those existing utility datastructures and updates the
checklist row to complete for the current supporting datastruct set. It does not
claim that unrelated future utility types will never be needed.

## Changes

- `issues/0801-roastty-libghostty-rewrite/README.md`
  - Change the `CircBuf` / `IntrusiveLinkedList` datastruct row from unchecked
    "as needed" to checked with scoped wording for the implemented terminal
    utility collections.
  - Add the Experiment 791 index entry.
- `issues/0801-roastty-libghostty-rewrite/791-datastruct-utilities-checklist-sync.md`
  - Record the verification evidence and review result.

## Verification

- Inspect current utility datastructure modules:
  - `roastty/src/terminal/circ_buf.rs`
  - `roastty/src/terminal/intrusive_linked_list.rs`
  - `roastty/src/terminal/array_list_collection.rs`
  - `roastty/src/terminal/cache_table.rs`
  - `roastty/src/terminal/lru.rs`
  - `roastty/src/terminal/segmented_pool.rs`
- Run focused datastructure tests:
  - `cargo test -p roastty circ_buf -- --nocapture --test-threads=1`
  - `cargo test -p roastty intrusive_linked_list -- --nocapture --test-threads=1`
  - `cargo test -p roastty array_list_collection -- --nocapture --test-threads=1`
  - `cargo test -p roastty cache_table -- --nocapture --test-threads=1`
  - `cargo test -p roastty lru -- --nocapture --test-threads=1`
  - `cargo test -p roastty segmented_pool -- --nocapture --test-threads=1`
- Run markdown formatting:
  - `prettier --write --prose-wrap always --print-width 80 issues/0801-roastty-libghostty-rewrite/README.md issues/0801-roastty-libghostty-rewrite/791-datastruct-utilities-checklist-sync.md`
- Run:
  - `git diff --check`

The experiment passes if the named datastructure modules exist, focused tests
pass, and the README row is checked with wording scoped to the current utility
collection set. It is Partial if only `CircBuf` or only `IntrusiveLinkedList`
verifies. It fails if the original unchecked row is still accurate.

## Design Review

Codex reviewed the design and found no blocking findings. The review approved
the docs-only scope, checked row limited to the current terminal utility
collection set, explicit future-helper caveat, and non-empty focused test
filters.
