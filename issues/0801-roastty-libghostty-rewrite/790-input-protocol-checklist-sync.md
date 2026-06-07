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

# Experiment 790: Input Protocol Checklist Sync

## Description

The Issue 801 input checklist says Kitty keyboard protocol details, `Link`, and
mouse input structs are missing. That wording is stale for the current Roastty
tree: `input/link.rs` and `input/mouse.rs` exist, terminal mouse encoding and
surface mouse dispatch are covered, and Kitty keyboard protocol push/pop/set and
query state are implemented in the terminal stream/screen/terminal layers.

This is still not a complete input subsystem. There is no dedicated
`input/kitty` module, platform keymaps/layouts remain a separate missing row,
and frontend integration still has open selection/mouse/key dispatch work. This
experiment verifies the existing input protocol pieces and updates the checklist
row from "missing" to a scoped partial state without marking it complete.

## Changes

- `issues/0801-roastty-libghostty-rewrite/README.md`
  - Change the Kitty keyboard / `Link` / mouse input checklist row from
    "missing" to a partial summary of the implemented pieces.
  - Keep the row unchecked because dedicated `input/kitty`, keymaps/layouts, and
    frontend integration remain incomplete.
  - Add the Experiment 790 index entry.
- `issues/0801-roastty-libghostty-rewrite/790-input-protocol-checklist-sync.md`
  - Record the verification evidence and review result.

## Verification

- Inspect current input protocol modules and handlers:
  - `roastty/src/input/link.rs`
  - `roastty/src/input/mouse.rs`
  - `roastty/src/terminal/mouse.rs`
  - `roastty/src/terminal/mouse_encode.rs`
  - `roastty/src/terminal/stream.rs`
  - `roastty/src/terminal/screen.rs`
  - `roastty/src/terminal/terminal.rs`
- Run focused Kitty keyboard checks:
  - `cargo test -p roastty kitty_keyboard -- --nocapture --test-threads=1`
- Run focused link and mouse input struct checks:
  - `cargo test -p roastty input::link -- --nocapture --test-threads=1`
  - `cargo test -p roastty input::mouse -- --nocapture --test-threads=1`
- Run adjacent mouse encoding and surface dispatch checks:
  - `cargo test -p roastty mouse_encode -- --nocapture --test-threads=1`
  - `cargo test -p roastty surface_mouse -- --nocapture --test-threads=1`
- Run markdown formatting:
  - `prettier --write --prose-wrap always --print-width 80 issues/0801-roastty-libghostty-rewrite/README.md issues/0801-roastty-libghostty-rewrite/790-input-protocol-checklist-sync.md`
- Run:
  - `git diff --check`

The experiment passes if the current tree has the named Link/mouse input structs
and Kitty keyboard protocol state, focused tests pass, and the README row is
updated to a scoped partial state without overclaiming dedicated `input/kitty`,
keymaps/layouts, or frontend integration. It is Partial if only Link/mouse
structs or only Kitty keyboard protocol state verify. It fails if the original
"missing" wording is still accurate.

## Design Review

Codex reviewed the design and found no blocking findings. The review approved
the unchecked partial checklist update, scoped wording for the existing
Link/mouse/Kitty keyboard pieces, explicit open work for dedicated
`input/kitty`, keymaps/layouts, frontend integration, and full input policy, and
the non-empty focused test filters.
