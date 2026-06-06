+++
[implementer]
agent = "codex"
model = "gpt-5"
reasoning = "high"

[review.design]
agent = "codex"
model = "gpt-5"
reasoning = "medium"
+++

# Experiment 713: Binding Action Clear Screen

## Description

Experiment 712 completed the remaining scroll-family binding action by adding
`scroll_to_selection`. Upstream Ghostty's `performBindingAction` also supports
the void `clear_screen` action:

- the action has no parameter;
- if the active screen is alternate, it returns `false` and does not clear;
- on the primary screen it clears the active selection;
- when history clearing is enabled, it erases scrollback;
- when the cursor is not at a prompt, it deletes active rows above the cursor
  and preserves the current input row;
- when the cursor is at a prompt, it clears the visible screen and sends a form
  feed (`0x0c`) to the child process so the shell can repaint;
- when the action performs work on the primary screen, it returns `true`.

Roastty already has active-screen detection, semantic prompt tracking, selection
clearing, scrollback erase, visible-screen erase, active-row erase, and PTY
write helpers. This experiment wires those existing primitives through the
binding-action path.

The binding-action path should always request history clearing. Upstream's
`Surface.performBindingAction(.clear_screen)` queues
`.clear_screen = .{ .history = true }`; the `history` flag is an internal termio
clear-screen option, not a user-supplied binding-action parameter. This
experiment therefore calls `Terminal::clear_screen(true)` from the surface
binding-action helper.

This does not implement Kitty graphics deletion during clear-screen. Upstream
also clears Kitty graphics for the active screen in the non-prompt branch, but
Roastty's renderer/Kitty graphics cleanup path is a separate missing graphics
integration slice and should not be mixed into this binding-action foundation.
This also does not implement prompt jumping, search actions, clipboard actions,
cursor-key actions, full keybind storage/lookup, frontend selection routing, or
app-scoped actions.

## Changes

- `roastty/src/terminal/page_list.rs`
  - Expose the existing active-row erase helper within the terminal module so
    the screen layer can delete active rows above the cursor without clearing
    the current row.

- `roastty/src/terminal/screen.rs`
  - Add a helper for clear-screen active-row deletion: if the cursor is below
    row zero, erase active rows `0..=cursor.y - 1`; if the cursor is on row
    zero, leave the active area unchanged.

- `roastty/src/terminal/terminal.rs`
  - Add `Terminal::clear_screen(history: bool) -> ClearScreenResult`.
  - Return `ClearScreenResult::NotPerformed` on the alternate screen.
  - On the primary screen, clear the active selection, optionally erase
    scrollback, then:
    - if not at a prompt, erase only active rows above the cursor and return
      `Performed`;
    - if at a prompt, erase the complete visible screen and return
      `SendFormFeed`.
  - Keep current prompt detection semantics unchanged.

- `roastty/src/lib.rs`
  - Extend the internal parsed binding-action enum with `ClearScreen`.
  - Extend `parse_binding_action` to accept bare `clear_screen` and reject any
    parameter, including `clear_screen:`.
  - Add/use a surface helper that locks the active termio worker, calls
    `Terminal::clear_screen(true)`, requests a render when the terminal action
    is performed, queues `0x0c` to the child when the terminal result requests a
    form feed, and returns whether the action was performed.
  - Return `false` for null, detached, no-worker, alternate-screen, and
    malformed-parameter cases.
  - Keep split, close, `text:`, `csi:`, `esc:`, `reset`, and scroll action
    semantics unchanged.

- `roastty/tests/abi_harness.c`
  - Add C ABI smoke coverage that parameterized `clear_screen` forms are
    rejected and the bare action returns `false` without crashing on the
    no-worker harness surface.

- Tests in `roastty/src/lib.rs`
  - Cover invalid parameter forms returning false, including both
    `clear_screen:` and `clear_screen:now`.
  - Cover null, detached, and no-worker surfaces returning false.
  - Cover primary-screen non-prompt clear removing scrollback and active rows
    above the cursor while preserving the current row.
  - Cover clear-screen clearing the active selection.
  - Cover alternate-screen clear returning false and preserving primary and
    alternate content.
  - Cover prompt clear erasing the visible screen and queueing a single form
    feed byte to the child process.
  - Re-run existing binding-action tests to prove previous action semantics did
    not change.

## Verification

Run:

- `cargo fmt -p roastty`
- `cargo test -p roastty clear_screen -- --nocapture`
- `cargo test -p roastty binding_action -- --nocapture --test-threads=1`
- `cargo test -p roastty --test abi_harness`
- `cargo fmt -p roastty -- --check`
- `git diff --check`

## Design Review

Codex reviewed the Experiment 713 design and found the technical slice otherwise
well scoped: void parser semantics, alternate-screen false behavior, selection
clearing, non-prompt row preservation, prompt form-feed behavior, and no-worker
false returns are all addressed.

The review raised one technical blocker before plan commit: the design needed to
explicitly justify why the surface helper calls `Terminal::clear_screen(true)`
instead of threading a configurable history flag. That is now documented in the
Description: upstream `Surface.performBindingAction(.clear_screen)` always
queues `.clear_screen = .{ .history = true }`, so the binding-action value for
Roastty is also always `true`.

The review accepted leaving Kitty graphics deletion out of this slice because
Roastty lacks the renderer/graphics cleanup integration, with the requirement
that the result preserve this as a documented parity gap rather than claiming
complete clear-screen parity. It also confirmed that the prompt test should
verify both local visible-screen erasure and exactly one queued `0x0c` byte.

The remaining required fix before plan commit was workflow provenance: adding
the design-review frontmatter, recording this design-review section, and
updating the README provenance tuple to `Codex/Codex/-`. Result-review
provenance will be added only after implementation and completion review.

Codex re-reviewed the updated design and found no remaining blockers. The
re-review confirmed that `history = true` is now explicitly tied to upstream
`Surface.performBindingAction(.clear_screen)`, design-review provenance is
recorded, and the scoped Kitty graphics parity gap remains documented. The
design is approved for the plan commit.
