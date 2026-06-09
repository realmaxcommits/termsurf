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

# Experiment 19: Phase C — a continuous present driver (live updates)

## Description

Exp 18 put the first frame (the shell prompt) on screen, but it's **static**:
`present_live` fires only from `set_size`/`start_termio`/`draw` (Exp 16), and
the async shell output that arrives later sets `self.dirty = true`
(`apply_termio_event`, `lib.rs:4146/4155`) **without** re-presenting —
`wakeup_app` → the app's `wakeup` is an empty stub, and nothing calls `draw`. So
typing and new output don't update the screen. Upstream drives presents from a
library-internal **`CVDisplayLink`** render loop (`renderer/generic.zig`);
roastty has none. This experiment adds a continuous present driver so the
terminal is **live**.

## Approach

Add a library-internal **main-thread repeating present driver** that presents
`present_live` whenever the surface is dirty, then clears the dirty flag — using
`dispatch2` (already a dep) — a self-rescheduling
`DispatchQueue::main().exec_after` (a new use; the IOSurface layer uses the
higher-level `exec_async`), which avoids raw `CVDisplayLink` CoreVideo FFI
(there is no `objc2-core-video` dep) and the `DISPATCH_SOURCE_TYPE_TIMER` +
`block2` surface. A vsync-locked `CVDisplayLink` is a later refinement; a ~60
fps main-queue timer presenting **only on dirty** is the first functional,
efficient driver (idle = no GPU work).

1. **Driver state on `Surface`**: a `running: Arc<AtomicBool>` (the stop flag) —
   the tick is a self-rescheduling `DispatchQueue::main().exec_after(~16ms, …)`
   closure that checks `running` first; this avoids the
   `DISPATCH_SOURCE_TYPE_TIMER` FFI and the `block2` dep (a cancellable
   `DispatchSource` timer is the alternative, also supported by `dispatch2`
   0.3).
2. **The tick must DRAIN the worker, not just read `dirty` (Required, from the
   review).** `self.dirty` is set ONLY by `apply_termio_event`, reachable ONLY
   via `tick_termio` ← `roastty_app_tick` — the termio worker thread updates the
   terminal + pushes to an `mpsc` channel but **never touches `self.dirty`**.
   The app's `wakeup` is an empty stub, so nothing pumps the tick. So each
   driver tick, **on the main thread**, must: (a) call `self.tick_termio()` to
   **drain the worker channel** (which applies output to the terminal, sets
   `self.dirty`, and processes clipboard events) — this is the event-loop pump
   the app isn't providing; (b) if `self.dirty`, call `present_live()` then set
   `self.dirty = false`; (c) reschedule. Without (a) the screen stays static
   (exactly the bug Exp 19 fixes).
   - **Start** on `surface_new` for real macOS surfaces (the same
     `platform_tag == MACOS` gate as the Exp-16 auto-start, after the surface is
     registered).
3. **Stop** on `surface_free`: flip `running = false` and cancel the timer
   **before** the `Box` is dropped, so no tick dereferences a freed `Surface`.
   The driver runs only on the main thread (where `free`/`new`/the present all
   run), so the dirty read + the cancel are not cross-thread data races — but
   the design must prove the tick cannot fire after free (e.g. cancel the
   DispatchSource synchronously, or guard every tick on `running` + never
   reschedule once false, and ensure the captured surface pointer is only
   dereferenced while `running`).
4. **The captured surface pointer + `Send` bridge.** `exec_after`/`exec_async`
   closures are `Send + 'static`, but `*mut Surface` is `!Send`. Wrap it in a
   move-only newtype with `unsafe impl Send`, justified by **main-thread-only**
   dereference — mirroring the existing `MainQueueSurfacePresentation` pattern
   (`renderer/metal/iosurface_layer.rs:162`). The closure also captures the
   `Arc<AtomicBool> running` independently.
   - **Lifetime safety (the main risk — but achievable).** Everything — the
     ticks, `present_live`, `surface_new`, and `surface_free` — runs on the
     **main thread**, which is a **serial** queue. `surface_free` flips
     `running = false` before `Box::from_raw`/drop. Because the queue is serial,
     `free` cannot interleave with a tick: a tick that observed
     `running == true` derefs the still-alive surface, completes, and
     reschedules; only _then_ can `free` run, set the flag, and free the box;
     the next scheduled tick observes `running == false` and **returns before
     any deref** (and does not reschedule). So no tick ever dereferences a freed
     `Surface`. The closure dereferences the pointer **only** after confirming
     `running` is true, within a single main-thread turn.

This touches **only `libroastty`** (`Surface` + `surface_new`/`surface_free` + a
driver module). No app changes. `present_live` already presents the live
terminal Retina-correctly (Exp 17/18); this just calls it on a cadence.

## Verification

1. **`cargo test -p roastty`** (full) green — the driver is inert in tests (null
   nsview / `platform_tag == 0`, like the Exp-16 auto-start), so no timer is
   started in the suite; add a headless unit test for the start/stop bookkeeping
   if feasible without a run loop.
2. **App launch (the payoff):** rebuild + launch; **type into the terminal and
   run a command (e.g. `ls`, `echo hi`) and confirm the output appears live**
   (not just the initial prompt). Capture the window (full-screen
   `screencapture` + `crop.swift`, window by `list-windows.swift` `name="👻"`).
   The captured frame shows post-launch content (typed text / command output)
   that only a re-present could produce. Kill the spawned app + children (0
   dangling PIDs); shots out-of-repo.
3. **No busy-spin when idle:** confirm the tick presents only on dirty (idle
   CPU/GPU stays low — spot-check, not a hard gate).
4. **Clean shutdown:** the app quits / the surface frees without a crash or a
   use-after-free (the driver stopped first). Re-launch twice to shake out
   lifetime races.

**Pass** = a main-thread driver presents on dirty, the suite is green, and the
launched app shows **live updates** (typed input + command output render after
the first frame), with clean start/stop (no UAF, no idle busy-spin).

**Partial** = live updates work but with a caveat (e.g. presents every tick
regardless of dirty, or a vsync/tearing artifact) — documented, with the
refinement (CVDisplayLink) named.

**Fail** = the driver can't be made lifetime-safe with `dispatch2` from this
harness (documented; fall back to a `CVDisplayLink` FFI design).

## Design Review

**Reviewer:** `adversarial-reviewer` subagent (Claude Opus, fresh context,
read-only). **Verdict: CHANGES REQUIRED → addressed.** It **confirmed** the
lifetime safety is achievable (main-thread serial queue + `running` flag flipped
in `surface_free` before the box drops → no tick derefs a freed surface;
`exec_after` self-reschedule is safe with the `Arc<AtomicBool>` checked before
any deref), that there is **no cross-thread `dirty` race** (`dirty` is
main-thread only; the worker never touches it), and that the
size-0-at-`surface_new` build self-corrects. One Required + two Optional, folded
in:

- **Required — `dirty` is never set by shell output in the running app.** It's
  written only by `apply_termio_event` ← `tick_termio` ← `roastty_app_tick`; the
  worker thread only fills an `mpsc` channel, and the app's `wakeup` is an empty
  stub. A driver that only _reads_ `dirty` presents nothing. **Fixed:** each
  tick now calls `self.tick_termio()` to **drain the worker** (which sets
  `dirty` + processes events) before the present-on-dirty — the driver IS the
  event pump.
- **Optional — the `Send` bridge** for the captured `*mut Surface` was
  under-specified (won't compile as worded). **Fixed:** named the move-only
  `unsafe impl Send` newtype mirroring `MainQueueSurfacePresentation`.
- **Optional — overstated dispatch2 reuse** (`exec_async` ≠ the timer path).
  **Fixed:** clarified it's a new `exec_after` self-reschedule.

## Result

_(to be added after the run.)_

## Conclusion

_(to be added after the run.)_
