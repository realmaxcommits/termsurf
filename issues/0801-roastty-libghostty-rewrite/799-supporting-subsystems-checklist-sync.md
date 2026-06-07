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

# Experiment 799: Supporting Subsystems Checklist Sync

## Description

Issue 801's supporting-subsystems checklist groups `cli/`, `inspector/`,
`crash/`, `terminfo/`, and `synthetic/` into one unchecked row with no progress
detail. That is stale for several scoped foundations: the C ABI exposes
surface-owned inspector handles and input-forwarding state, bundled resource
discovery can locate terminfo-bearing resource directories, and CoreText/font
code supports synthetic bold/italic style generation.

This experiment updates the checklist wording only. It keeps the row unchecked
because CLI/list tooling, inspector UI rendering and core data collection,
Sentry-style crash reporting, full terminfo install/tooling, and any broader
Ghostty `synthetic/` subsystem remain incomplete.

## Changes

- `issues/0801-roastty-libghostty-rewrite/README.md`
  - Update the supporting-subsystems row from an undifferentiated open list to
    scoped partial wording naming the existing inspector ABI foundation,
    terminfo resource lookup, and font synthetic style support.
  - Keep the row unchecked and explicitly leave CLI/list tools, inspector
    UI/data, crash reporting, full terminfo tooling, and broader synthetic work
    open.
  - Add the Experiment 799 index entry.
- `issues/0801-roastty-libghostty-rewrite/799-supporting-subsystems-checklist-sync.md`
  - Record verification evidence and review results.

## Verification

- Inspect:
  - `roastty/src/lib.rs`
  - `roastty/src/os/resources_dir.rs`
  - `roastty/src/font/face/coretext.rs`
  - `roastty/src/font/collection.rs`
- Run:
  - `cargo test -p roastty inspector -- --nocapture --test-threads=1`
  - `cargo test -p roastty resources_dir -- --nocapture --test-threads=1`
  - `cargo test -p roastty synthetic -- --nocapture --test-threads=1`
- Run markdown formatting:
  - `prettier --write --prose-wrap always --print-width 80 issues/0801-roastty-libghostty-rewrite/README.md issues/0801-roastty-libghostty-rewrite/799-supporting-subsystems-checklist-sync.md`
- Run:
  - `git diff --check`

The experiment passes if the supporting-subsystems row records the existing
inspector ABI, terminfo resource discovery, and font synthetic foundations while
remaining unchecked and leaving the larger
CLI/inspector/crash/terminfo/synthetic surface open. It is Partial if only one
or two foundations can be documented. It fails if the row should remain an
undifferentiated open list.

## Design Review

Codex reviewed the design and found no blocking findings. The review approved
the scope because the row remains unchecked, the wording is limited to partial
foundations, the open gaps prevent a full supporting-subsystem completion claim,
and the verification plan directly covers inspector ABI, resource discovery, and
font synthetic behavior.
