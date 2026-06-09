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

# Experiment 16: Phase C — `surface_new` auto-starts the IO (the shell-start divergence)

## Description

Exp 15 found the surface's shell never starts: the renamed app calls
`roastty_surface_new` but **never `roastty_surface_start`**
(`SurfaceView_AppKit.swift:352` is the only surface call), and `ghostty.h` has
**no `surface_start`** at all — `ghostty_surface_new` starts the IO itself
(`embedded.zig` → `Surface.init`/`core_surface.init`). roastty split new/start
(the interim API), so the shell never runs → `termio_worker` is `None` → the
(already-wired) live present skips → blank terminal.

## Approach

Make `roastty_surface_new` start the surface IO, matching ghostty — but
reconcile with roastty's **test harness**, which injects `termio_worker`
manually (`new_test_surface` → `= Some(test_worker(...))`) and must NOT spawn
real shells.

1. **At the end of `roastty_surface_new`** (after the `Surface` is fully built +
   registered + `app` set), call `surface.start_termio()` — the existing method
   (lib.rs:2273) that spawns the surface's stored command/working-dir/env and
   sets `termio_worker`.
2. **Gate it on a RUNTIME signal — `platform_tag == ROASTTY_PLATFORM_MACOS`
   (1)** — NOT `#[cfg(not(test))]`. The design review showed `cfg(test)` is
   **not** hermetic: the `roastty/tests/abi_harness.rs` integration test links
   the **cdylib** (where `cfg(test)` is OFF → auto-start ON) and calls
   `roastty_surface_new` 20+ times, which would spawn real shells and flip
   worker-gated FFIs. Instead, auto-start only for **real macOS app surfaces**:
   the app sets `platform_tag == MACOS` + a real `nsview`; the abi_harness +
   unit tests use the default `platform_tag == 0` (verified) and inject
   `termio_worker` manually. This is the same condition that already gates the
   nsview capture, and it's faithful (ghostty's real surfaces all carry a
   platform → all auto-start; the `platform_tag == 0` surfaces are roastty-test
   artifacts ghostty has no equivalent of). `start_termio` guards re-entry, and
   the app calls `surface_new` once and never `surface_start`, so there is no
   double-start.
3. **Re-launch the app** (Exp-14/15 harness): the shell now runs, so
   `present_live` reaches `render_and_present_frame` with a live
   `termio_worker`. Note: `start_termio`'s present fires during `surface_new`
   when `size` is still 0 → a clamped **1×1 throwaway frame** (the compositor
   reallocates its target on the later real-size `set_size` present, so it's a
   no-op, not fatal) — the meaningful present is the subsequent `set_size` one.
   Verify via the live-present log / window capture.

This touches **only `roastty/src/lib.rs`** (one runtime-gated call). No app
source changes. It does **not** yet make text appear — Exp 17 (atlas coherence)
is still required — but it unblocks the present path so it actually renders the
terminal's background/cells path.

## Verification

1. **Full `cargo test -p roastty`** (NOT `--lib` — must include the
   `abi_harness` integration test, which links the cdylib) green, AND **no shell
   processes spawned/leaked by the harness** (the harness surfaces use
   `platform_tag == 0`, so the runtime gate excludes them). The unit tests
   inject `termio_worker` manually and use null nsview / `platform_tag == 0`, so
   the auto-start is skipped there too.
2. **App launch:** the shell starts (a real `termio_worker`), so the live
   present no longer skips on `worker is None`. Confirm `present_live` reaches
   `render_and_present_frame` (no "worker is None" path) — e.g. the window shows
   the terminal background frame, or the live present error log is clean.
   Capture out-of-repo; **kill the spawned app** (0 dangling PIDs).
3. **No regression / no double-start** in the app (the surface starts exactly
   once).

**Pass** = `surface_new` auto-starts the IO in non-test builds (test build
unchanged, suite green), and the launched app's surface has a running shell so
the wired present reaches `render_and_present_frame` (worker present). (Text
still needs Exp 17.)

**Partial** = the shell starts + tests green, but an unexpected interaction
surfaces (e.g. the present still skips for another reason) — documented.

**Fail** = auto-starting in `surface_new` can't be reconciled with the test
harness or breaks launch (documented).

## Design Review

**Reviewer:** `adversarial-reviewer` subagent (Claude Opus, fresh context,
read-only). **Verdict: CHANGES REQUIRED → addressed.** It confirmed faithfulness
(ghostty's `Surface.init` spawns the IO thread unconditionally —
`Surface.zig:723` via `embedded.zig:248/580` — so auto-starting matches
upstream), no double-start (app calls `surface_new` once, never `surface_start`;
`start_termio` guards re-entry), and that `start_termio` reads only fields the
`surface_new` `Box` already populates. Two Required + one Optional, all
addressed:

- **Required — `#[cfg(not(test))]` is NOT hermetic.** `roastty` is a `cdylib`;
  the `tests/abi_harness.rs` integration test links it (cfg(test) OFF →
  auto-start ON) and `abi_harness.c` calls `surface_new` 20+ times → would fork
  20+ real shells per `cargo test` and flip worker-gated FFIs. **Fixed:** gate
  on the **runtime** `platform_tag == MACOS` (the harness uses
  `platform_tag == 0`), not `cfg`.
- **Required — verification used `--lib`,** masking the abi_harness regression.
  **Fixed:** verify with the **full** `cargo test -p roastty` + zero shell
  leaks.
- **Optional — `start_termio`'s present fires during `surface_new`** at size 0 →
  a 1×1 throwaway frame (compositor reallocates later). **Fixed:** documented.

## Result

**Result:** Pass — `surface_new` now auto-starts the IO for real macOS app
surfaces, the launched app spawns a live shell, and the full suite (lib **+ the
abi_harness**) is green with no shell leaks. Verifying this surfaced — and
required fixing — a **second, larger** thing: the `abi_harness` C conformance
test had been silently broken since Exp 8.

### The auto-start (the planned change)

`roastty_surface_new` calls `surface.start_termio()` gated on
`platform_tag == MACOS` (1). One `lib.rs` call; no app changes.

- **Verified live:** launching `Roastty.app` (PID _N_) now spawns a child
  **`/bin/zsh`** (`ps`: `<shell-pid> <N> /bin/zsh`) — before Exp 16 there was no
  shell child. So the surface's IO is running and `present_live` (from
  `start_termio`/`set_size`) now reaches `render_and_present_frame` with a live
  `termio_worker` instead of skipping on `worker is None`. App + child killed
  cleanly (0 dangling). (Text still won't render — Exp 17 atlas coherence.)

### The discovered regression (the larger fix): restoring `abi_harness`

The design review correctly forced verification with the **full**
`cargo test -p roastty` (not `--lib`). That immediately exposed that
**`tests/abi_harness.c` had not compiled since Exp 8** — my Exp 8–15 ABI changes
(renames + signature/layout changes) were only ever validated with
`cargo test --lib`, which skips the C-linking integration test. **141 compile
errors + 1 runtime assert**, all from prior experiments, fixed here:

- **Keys (Exp 8/13):** the harness specifies keys by W3C enum via `key_event_t`,
  but by-value `surface_key`/`surface_key_is_binding` take a native keycode — so
  the harness must use the opaque `_handle` variants. Those Rust impls already
  existed but were **never declared in `roastty.h`** — added the two decls
  (mirroring `config_key_is_binding_handle`). Also the
  `ROASTTY_KEY_KEY_*`→`ROASTTY_KEY_*`, `DIGIT0`→`DIGIT_0`, `NUMPAD0`→`NUMPAD_0`
  renames.
- **Action union (Exp 9):** `action_s.storage` (old opaque byte array) →
  `action_s.action` (typed union). Added the `uintptr_t raw[3]` member to the C
  `roastty_action_u` (mirroring the Rust `raw: [usize;3]`) so the harness can
  read roastty-only tags like NAVIGATE_SEARCH (`.action.raw[0]`); fixed the
  zero-fill loop bounds (`<8` bytes → `<3` words).
- **Readonly values (Exp 9 review):** the harness asserted the _old swapped_
  `ON==0/OFF==1`; Exp 9 fixed these to upstream `OFF==0/ON==1` — updated the
  asserts (the runtime failure).
- **Point/selection (Exp 11):** grid
  `point_s`/`point_value_u`/`point_coordinate_s`/ `selection_s` → their `grid_*`
  names; the `read_text` selections → the embedded `selection_s`
  (`{top_left, bottom_right, rectangle}`, no `.size`).
- **Target (Exp 12):** `target_s.surface` → `target_s.target.surface`.

### Verification

- **Full `cargo test -p roastty`:** lib **4401 passed**, `abi_harness` **1
  passed** (was: failed to compile). No `--lib` scoping.
- **No shell leaks:** `/bin/sh` count 12→12 across the run; the harness surfaces
  are `platform_tag == 0`, so the auto-start gate excludes them (no real shells
  in tests).
- **Live shell:** the launched app has a `/bin/zsh` child (above).
- **Known minor follow-up:** 10 benign `-Wimplicit-enum-enum-cast` warnings
  remain where the harness passes terminal-mouse/key enum constants
  (`ROASTTY_MOUSE_BUTTON_LEFT`, `ROASTTY_KEY_ACTION_*`) to the input functions
  (identical values; no `-Werror`, test passes). A blanket rename risks the
  terminal-mouse call sites, so it is left as a targeted cleanup.

## Conclusion

The app's surface now **runs a real shell** (the planned auto-start), and — more
consequentially — the **C ABI conformance harness compiles and passes again**,
closing a regression that had been invisible since Exp 8 because I validated
with `cargo test --lib`. **Lesson (added to the issue README):** always verify
ABI work with the **full** `cargo test -p roastty`; `--lib` skips the
`abi_harness` integration test that links the cdylib against `roastty.h`.

**Next (Exp 17) — atlas coherence:** the present now reaches
`render_and_present_frame` with a live terminal, but still samples standalone
empty atlases instead of the `SharedGrid`'s glyph atlases, so it renders
backgrounds, not text. Make the sampled atlas be (or be synced from) the grid's
rasterized atlas. Then real terminal text finally appears. (Exp 18: the
continuous `CVDisplayLink` driver for live updates.)

## Result Review

**Reviewer:** `adversarial-reviewer` subagent (Claude Opus, fresh context,
read-only). **Verdict: APPROVED.** It independently reproduced the load-bearing
claims — `clang -fsyntax-only` on `abi_harness.c` → **0 errors** (10 benign
warnings), `cargo build -p roastty` clean,
`cargo test -p roastty --test abi_harness` → **1 passed** — and validated all
six review points: (1) the auto-start gate is safe (`register_surface` only
stores the raw `NonNull`, so `as_ptr().as_mut()` has exclusive access;
`start_termio` guards re-entry; no double-start); (2) `action_u.raw[3]`
(24B/align8) matches the Rust `raw: [usize;3]`, and since `open_url` is already
24B it's **co-largest and cannot grow the union** — guarded by the existing
`_Static_assert(sizeof(roastty_action_u) == 24)` (so the size is verified, not
silent); (3) the `_handle` decls match their Rust impls; (4) the readonly fix
(`OFF==0/ON==1`) is upstream-faithful across header/Rust/harness; (5) the
harness was **not weakened** — the `raw[0]` NAVIGATE_SEARCH read is genuinely
exercised by a real key binding driven through `surface_key_handle` (Rust writes
`u.raw = [storage[0], 0, 0]`); (6) the honesty checks hold (`--lib` truly skips
the integration test; warnings benign / no `-Werror`). Two minor findings, both
fixed: an Optional ("ghostty" → "upstream" in the new comment) and a Nit (the
`raw[3]` comment overstated "forces the size" — it's the typed accessor; the
union is already 24/8 via the largest member).
