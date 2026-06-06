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

# Experiment 685: Surface Mouse Captured

## Description

Experiment 684 finished worker-backed active selection reads. The next smallest
surface ABI gap is `ghostty_surface_mouse_captured(surface)`, which upstream
implements as a query over the terminal's mouse-event reporting state.

Roastty already tracks mouse-event reporting modes in `Terminal::mouse_tracking`
and exposes that state through the terminal C ABI. Surface terminals currently
live behind an attached `TermioWorker`, so this experiment adds only the surface
query that reads the worker terminal's mode state. It does not implement mouse
button/position/scroll/pressure dispatch, mouse-shift-capture policy, link hover
actions, selection routing, or frontend cursor behavior.

## Changes

- `roastty/include/roastty.h`
  - Add `ROASTTY_API bool roastty_surface_mouse_captured(roastty_surface_t);`
    near the other surface APIs.
- `roastty/src/lib.rs`
  - Implement `roastty_surface_mouse_captured(surface)`:
    - null, detached, or no-worker surfaces return `false`;
    - worker-backed surfaces return `terminal.mouse_tracking()`.
  - Add tests:
    - null, detached, no-worker, and default worker-backed surfaces return
      `false`;
    - a worker terminal that receives a DEC mouse-event mode enable sequence
      returns `true`;
    - disabling the mouse-event mode returns `false` again.
- `roastty/tests/abi_harness.c`
  - Exercise null/no-worker `roastty_surface_mouse_captured` through the public
    C header.

## Verification

- `prettier --write --prose-wrap always --print-width 80 issues/0801-roastty-libghostty-rewrite/README.md issues/0801-roastty-libghostty-rewrite/685-surface-mouse-captured.md`
- `cargo fmt -p roastty`
- `cargo fmt -p roastty -- --check`
- `cargo test -p roastty surface`
- `cargo test -p roastty --test abi_harness`
- `git diff --check`

## Design Review

**Result:** Approved.

Codex found no design blockers. It approved the scope as appropriately narrow:
read only the attached `TermioWorker` terminal state and exclude mouse dispatch,
shift-capture policy, link hover, selection routing, and frontend cursor
behavior.

Codex also confirmed the upstream-fidelity target: Ghostty's
`Surface.mouseCaptured()` returns whether the terminal mouse-event mode is not
none, and Roastty's `Terminal::mouse_tracking()` already aggregates the four DEC
mouse-event modes into the equivalent boolean. The planned null, detached,
no-worker, default, enable, disable, and C header checks are sufficient for this
slice.

## Result

**Result:** Pass.

Implemented `roastty_surface_mouse_captured(surface)` as a worker-terminal
query. Null, detached, and no-worker surfaces return `false`; worker-backed
surfaces return the attached terminal's `mouse_tracking()` state.

The C header now declares the new surface API, and the C ABI harness exercises
null/no-worker calls through the public header. Rust tests cover null, detached,
no-worker, default worker-backed false, DEC mouse-event mode enable returning
true, and disabling the mode returning false again.

Verification:

- `cargo fmt -p roastty`
- `cargo test -p roastty surface`
- `cargo test -p roastty --test abi_harness`
- `cargo fmt -p roastty -- --check`
- `git diff --check`

## Conclusion

Surface mouse capture querying now matches the worker-terminal portion of
upstream's `ghostty_surface_mouse_captured` behavior. Mouse button, position,
scroll, and pressure dispatch remain separate work, along with mouse-shift
capture policy, link hover actions, selection routing, and frontend cursor
behavior.

## Completion Review

**Result:** Approved after workflow updates.

Codex reviewed the staged result diff and found no code, ABI, regression, or
missing-test blockers. It confirmed the implementation is the intended narrow
query: null, detached, and no-worker surfaces return `false`, and worker-backed
surfaces forward the worker terminal's `mouse_tracking()` state. Header
placement and C/Rust tests cover the expected ABI and behavior.

Codex blocked the result commit only until review provenance was recorded. This
section, the `[review.result]` frontmatter, and the README reviewer tuple are
the requested workflow updates.
