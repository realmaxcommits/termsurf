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

# Experiment 677: Surface Draw Wakeup

## Description

Experiment 676 made no-command surfaces use Roastty's default-shell resolution.
The next surface lifecycle gap is draw/wakeup signaling. The upstream embedded
ABI exposes `ghostty_surface_draw(surface)`, and Roastty's ABI inventory already
lists the renamed `roastty_surface_draw`, but the Roastty header and library do
not implement it yet.

This experiment adds the `roastty_surface_draw(surface)` ABI as a narrow
renderer-wakeup slice. Roastty does not yet have the full renderer-thread
machinery that upstream `Surface.draw()` uses, so this slice maps draw requests
onto the behavior the current library can represent: mark the surface as needing
render and invoke the app runtime `wakeup_cb` when the live surface still has an
attached app with a wakeup callback.

This experiment does not implement renderer frame drawing, renderer mailbox
messages, Metal renderer integration, refresh callbacks, frontend presentation,
or animation scheduling.

## Changes

- `roastty/include/roastty.h`
  - Add `ROASTTY_API void roastty_surface_draw(roastty_surface_t);` alongside
    the other surface lifecycle functions.
- `roastty/src/lib.rs`
  - Add `roastty_surface_draw(surface)`.
  - Null surfaces are a no-op.
  - For a live surface, set `surface.dirty = true`.
  - If `surface.app` is non-null and the app runtime has `wakeup_cb`, invoke it
    with runtime userdata.
  - If the surface has been detached by `roastty_app_free`, keep the call a
    no-op beyond marking the live surface dirty; do not dereference the old app
    pointer.
  - Keep `roastty_surface_render_state_update` as the operation that clears
    dirty state after a successful snapshot.
  - Add tests:
    - null draw is a no-op;
    - drawing a live surface marks `roastty_surface_needs_render(surface)`;
    - drawing a live surface invokes `wakeup_cb` with app userdata;
    - drawing a live surface twice invokes `wakeup_cb` twice, even when the
      surface is already dirty;
    - drawing a detached surface marks it dirty without invoking a wakeup;
    - successful render-state update still clears a draw-requested dirty flag
      when a worker exists. Use `os::pty::PTY_COMMAND_LOCK` for this
      subprocess-backed test.
- `roastty/tests/abi_harness.c`
  - Exercise `roastty_surface_draw(surface)` through the C header and assert
    `roastty_surface_needs_render(surface)` becomes true for the existing
    skeleton surface.

## Verification

- `prettier --write --prose-wrap always --print-width 80 issues/0801-roastty-libghostty-rewrite/README.md issues/0801-roastty-libghostty-rewrite/677-surface-draw-wakeup.md`
- `cargo fmt -p roastty`
- `cargo fmt -p roastty -- --check`
- `cargo test -p roastty surface`
- `cargo test -p roastty --test abi_harness`
- `git diff --check`

## Design Review

**Result:** Approved after amendments.

Codex found two test-plan gaps. First, upstream `Surface.draw()` forces a render
on every call, so the plan should prove repeated draw calls invoke wakeup even
when the surface is already dirty. Second, the worker-backed dirty-clear test
will spawn a PTY process and should explicitly use the shared PTY command lock.

The design now includes a repeated-draw wakeup test and requires
`os::pty::PTY_COMMAND_LOCK` for the worker-backed render-state update test.
