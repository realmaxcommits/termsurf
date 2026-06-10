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

# Experiment 32: Phase C — widen the reporting-mode selection clear+reset

## Description

Exp 25's review noted the reporting-mode `selection_clear_and_reset` is
**narrower than upstream**. Currently (`lib.rs::mouse_button`) the clear runs
only for **Left + Press**:

```rust
if matches!(button, Left) {
    if !reporting { /* selection press/drag/release/extend */ }
    else if Press { self.selection_clear_and_reset(); }
}
```

Upstream `Surface.zig:3879-3895` clears+resets on **any** button and **either**
press or release while mouse-reporting (`isMouseReporting()` runs for every
button event): `setSelection(null)` + `selection_gesture.reset(...)`, so a stale
selection from before reporting was enabled can't linger (or resume on a
report→no-report transition). roastty leaves the selection highlighted on a
non-Left button or a Left release while reporting.

## Approach

Hoist the reporting clear out of the Left-only branch so it runs for **any**
button + **any** state when mouse-reporting; keep the selection
(press/drag/extend/release) on the not-reporting + Left path:

```rust
if self.mouse_report_context().is_some() {
    // Reporting: clear+reset on any button event (upstream Surface.zig:3879-3895). Shift-while-
    // reporting override is deferred (separate follow-up).
    self.selection_clear_and_reset();
} else if matches!(button, MouseButton::Left) {
    match state {
        Press => if self.should_shift_extend() { self.selection_drag() } else { self.selection_press() },
        Release => self.selection_release(),
    }
}
self.dispatch_mouse_report(action, Some(button))
```

Behavior-identical for the Left+Press-while-reporting case (still clears); newly
also clears on a non-Left button or a release while reporting — matching
upstream. **Only `libroastty`** (`lib.rs`, restructure the existing branch). No
app change. The `selection_clear_and_reset` body is unchanged.

## Verification

1. **Headless regression test:** put the terminal in a mouse-reporting mode
   (`\x1b[?1000h` → `mouse_event_mode != None`, so `mouse_report_context()` is
   `Some`); set an active selection directly (`set_selection`); send a
   **non-Left** button press (and, separately, a **Left release**) via
   `mouse_button` → assert the selection is **cleared**
   (`active_selection().is_none()`). Pre-fix these do **not** clear (only
   Left+Press did); post-fix they do. A control: a Left+Press while reporting
   still clears (unchanged). `cargo test -p roastty` (full) green.
2. **No regression:** the **not-reporting** selection path is unchanged (Exp
   25/27/28/30 tests still pass) — the reporting clear is gated on
   `mouse_report_context().is_some()`, which is `None` in those tests (no mouse
   mode set), so they take the selection branch exactly as before.
3. **No live confirmation needed** — this is an edge-case behavioral fix
   (stale-selection clearing under reporting); the headless model assertion
   (selection cleared) is the proof.

**Pass** = the clear+reset runs for any button + press/release while reporting,
the headless test (non-Left/release clears; not-reporting unaffected) passes,
and the suite is green. (Fully headless — no Partial-pending-live.)

**Partial** = the widening works for press but release needs more (unlikely —
same call site).

**Fail** = the restructure breaks the not-reporting selection path (documented).

## Design Review

**Reviewer:** `adversarial-reviewer` subagent (Claude Opus, fresh context,
read-only). **Verdict: APPROVED.** Confirmed: **not-reporting selection path
preserved exactly** (when not reporting, `is_some()` false → Left takes the
identical match, non-Left falls through to nothing as before; the only new call
for non-Left is the read-only `mouse_report_context()` query, already run
unconditionally in `mouse_scroll`/`mouse_pos`); **clearing on every reporting
button is faithful** (upstream runs the `isMouseReporting()` block for every
button + press/release, `setSelection(null)`

- `reset` unconditionally incl. `dirty`; `gesture.reset` can't collide with a
  drag since drags are not-reporting-only); **test feasible**
  (`new_test_surface` defaults `mouse_reporting:true`; feed `\x1b[?1000h` via
  `next_slice` on the worker's terminal → `mouse_event_mode != None`;
  `set_surface_worker_active_selection`; `mouse_button_from_int` 2→Right; the
  clear runs before the position-gated `dispatch_mouse_report`); **no-regression
  honest** (Exp 25/27/28/30 tests never feed `[?1000h` →
  `mouse_report_context()` None → unchanged else-branch). Optional/Nit folded
  in: keep the shift-while-reporting deferral in the impl comment; feed
  reporting mode via `next_slice` on the worker terminal (not a separate
  handle).

## Result

**Result:** Pass — the reporting-mode selection clear+reset now runs for any
button + press/release (fully headless; no live needed).

### Change (only `libroastty`)

`lib.rs::mouse_button` restructured:
`if self.mouse_report_context().is_some() { self.selection_clear_and_reset() } else if matches!(button, Left) { Press => extend-or-press, Release => release }`.
The reporting clear is hoisted out of the Left-only branch (any button + any
state); the not-reporting selection path (Left only) is byte-for-byte the same
match. The shift-while-reporting override stays deferred (noted in the comment).

### Verification

- **Headless regression test** `reporting_clear_widens_to_any_button`
  (`lib.rs`): in mouse-reporting mode (`[?1000h`) with an active selection — a
  **non-Left** (Right) press, a **Left release**, and a Left press each clear
  the selection (`active_selection().is_none()`). Pre-fix only Left+Press
  cleared (a non-Left/release left it lingering). Fails pre-fix, passes after.
- **Full `cargo test -p roastty`:** lib **4415 passed**, 0 failures — the
  not-reporting selection tests (Exp 25/27/28/30 + copy) all still pass (they
  never enter reporting mode → unchanged else-branch).
- **No live confirmation needed** — an edge-case behavioral fix; the headless
  model assertion proves it. **Completes fully while the screen is locked.**

## Conclusion

Reporting-mode selection clearing is now faithful to upstream (clear on any
button event), so a stale selection can't linger or resume across a
report→no-report transition. A clean fully-headless Pass. Remaining: the live
re-confirmations (Exp 29 CJK, Exp 30 shift-click) + closing await the screen
unlock; the deeper minors (shift-while-reporting — needs
`mouse-shift-capture`/`XTSHIFTESCAPE` modeling, CVDisplayLink vsync, DPI-change
rebuild) remain documented follow-ups.

## Result Review

**Reviewer:** `adversarial-reviewer` subagent (Claude Opus, fresh context,
read-only). **Verdict: APPROVED.** Verified: the test is **load-bearing +
discriminating** ((a) non-Left press and (b) Left release would each fail
pre-fix since only Left+Press cleared; the `assert!(has_selection)` precondition
proves a real selection was set, and a non-engaged reporting mode would also
fail (a) — so the pass confirms `\x1b[?1000h` genuinely made
`mouse_report_context()` Some); **no regression** (full lib **4415 passed, 0
failed**; the not-reporting selection tests untouched — the clear is gated on
`mouse_report_context().is_some()`, None when no mouse mode is fed); the **diff
is exactly the restructure** (the not-reporting Left arm is byte-for-byte
identical, only re-parented under `else if`; `selection_clear_and_reset` body
unchanged; truth table holds); **Pass honest/headless**; hygiene clean
(`fmt --check` 0, no "ghostty" literals). Upstream fidelity confirmed against
`Surface.zig:3879-3892`. Optional (tracked follow-up): upstream guards the clear
with `if (mods.shift and !shift_capture) break` — roastty clears
unconditionally; the shift-while-reporting override is the documented deferral
(needs `mouse-shift-capture`/XTSHIFTESCAPE modeling).
