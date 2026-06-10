+++
[implementer]
agent = "claude-code"
model = "claude-opus-4-8"
reasoning = "high"

[review.design]
agent = "claude-code"
model = "claude-opus-4-8"
reasoning = "high"
+++

# Experiment 28: Phase C — drag-selection autoscroll past the edge

## Description

Exp 25 wired drag selection, but a drag that goes **past the top/bottom edge**
doesn't scroll to extend the selection into off-screen content — a common,
expected behavior (select more than one screen). The gesture machinery is
complete: `SelectionGesture::drag` already sets `self.autoscroll` (`Up` when the
drag `y <= 1`, `Down` when `y > screen_height - 1`, `selection_gesture.rs`), and
`autoscroll_tick` scrolls the viewport by ±1 and re-drags to the edge cell. But
**nothing calls `autoscroll_tick`** — so the autoscroll state is computed and
never acted on. Upstream drives it on a timer while the button is held past the
edge.

## Approach

1. **`Surface::selection_autoscroll_tick`** (`lib.rs`): when the left button is
   held and the gesture's `autoscroll()` is `Up`/`Down`, compute the **clamped**
   viewport cell at the current mouse position — `geometry.pos_to_cell(pos)`
   directly (it clamps row/col to the grid), **not** `position_to_cell` (which
   returns `None` past the edge via `pos_out_of_viewport`) — plus
   `selection_geometry`, then call `gesture.autoscroll_tick(...)` and apply the
   returned selection, mark dirty. No-op when `autoscroll == None`, no left
   button, **or `mouse_report_context().is_some()`** (symmetry with
   `selection_drag`'s gate — else a program enabling reporting mid-drag leaves a
   retained `autoscroll` scrolling until button-up; the gesture also self-guards
   on `click_count == 0`). **Borrow:** mirror `selection_drag`'s
   read-then-mutate split exactly — read the cell/geometry in one `with_termio`,
   mutate in a separate `with_termio_mut`; **never nest** them
   (`with_termio_mut` is a non-reentrant `Mutex::lock` → self-deadlock on the
   main thread).
2. **Drive it from the present loop.** `start_present_driver` (Exp 19) already
   ticks ~16ms on the main thread (`tick_termio` + `present_live`); add
   `surface.selection_autoscroll_tick()` to that tick **before** the
   `if surface.dirty` present check (so the scrolled row presents the same
   frame, not 16ms late — the tick sets `dirty`). So while a drag is held past
   the edge, the viewport scrolls ~1 row/tick and the selection extends — and
   stops the moment the button releases (the release sets
   `buttons[Left]=Release` → `left_button_pressed()` false, **and**
   `gesture.release` sets `autoscroll = None`) or the mouse returns inside
   (autoscroll → None on the next `drag`).

Faithful to upstream's held-past-edge autoscroll. **Only `libroastty`**
(`lib.rs`: the tick method + the present-driver hook). No app change.

## Verification

1. **Headless regression test:** fill past the screen; drag from a cell **up
   past the top edge** (`mouse_pos` with `y <= 1` so `drag` sets
   `autoscroll = Up`); then call `selection_autoscroll_tick()` a few times
   (simulating the present ticks); assert the **viewport scrolled up into
   history** (a previously-off-screen row is now selected / visible via
   `render_rows_snapshot` or the selection text grew to include history). A
   control: with the mouse **inside** the viewport (no autoscroll), the tick is
   a no-op (selection unchanged). Asserts via
   `active_selection()`/`selection_format`. Fails pre-fix (tick never scrolls —
   there's no tick), passes after. `cargo test -p roastty` (full) green,
   deterministic (no wall-clock dependence — the test calls the tick directly).
2. **No regression:** the present-driver hook is a no-op unless a drag
   autoscroll is active (guarded by `autoscroll()`/left-button/`click_count`),
   so normal rendering + the Exp-25/27 selection tests are unaffected.
3. **Live confirmation** (screen unlocked — check `CGSSessionScreenIsLocked`):
   launch with content past one screen; drag from mid-screen up to the **top
   edge and hold** (a `drag.swift` variant that pauses with the button down at
   the edge); the viewport **auto-scrolls into history** and the selection
   extends. App + descendant tree killed (0 dangling); shots out-of-repo.
4. Faithful to upstream autoscroll (cite).

**Pass** = `autoscroll_tick` is wired (tick method + present-driver hook), the
headless test (drag past edge → viewport scrolls + selection extends) passes,
the suite is green, and the live app auto-scrolls a held past-edge drag.

**Partial** = the tick method + headless test pass, but the live hold-at-edge
can't be driven from the harness (documented; the headless proves the logic).

**Fail** = autoscroll can't be driven from the present loop (documented).

## Design Review

**Reviewer:** `adversarial-reviewer` subagent (Claude Opus, fresh context,
read-only). **Verdict: APPROVED.** It traced the full path and **refuted the
runaway-after-release risk** (two independent guards: Release sets
`buttons[Left]=Release` before any branch → `left_button_pressed()` false, and
`gesture.release` sets `autoscroll = None`; the serial `dispatch2` main queue
means no tick interleaves the FFI release), **confirmed the present-driver hook
is safe** (main thread, ~16ms, `platform_tag == 1` only so tests/abi_harness
with tag 0 never start the driver; `tick_termio` returns before the new call —
no nested lock), **the clamped cell is correct** (`pos_to_cell` clamps to row 0
past the top / `rows-1` past the bottom — exactly the edge cell
`autoscroll_tick` re-pins after scrolling; `pos_to_cell` over `position_to_cell`
is right since past-edge is when a cell is still needed), **direction correct**
(Up→`-1`→toward history), and **the test is sound + deterministic**. Three
Optional/Nit, folded in: mirror `selection_drag`'s read-then-mutate split (never
nest `with_termio_mut` → deadlock); add the `mouse_report_context().is_none()`
gate for symmetry; place the tick before the dirty check (present same frame).

## Result

_(to be added after the run.)_

## Conclusion

_(to be added after the run.)_
