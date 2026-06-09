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

# Experiment 23: Phase C — scrollback navigation (deferred Exp-20 probe)

## Description

Exp 20 deferred **scrollback navigation** (scrolling up to view history). The
design review corrected the premise: the path **does not exist**.
`roastty_surface_mouse_scroll` → `Surface::mouse_scroll` (`lib.rs:3789`) calls
**only** `dispatch_scroll_reports`, which no-ops unless the terminal is in
**mouse-reporting** mode. The `scroll_viewport_*` functions are reachable
**only** from the explicit keybinding/AppleScript FFIs — never from the wheel.
So in a plain shell, scrolling the wheel does nothing. Upstream `scrollCallback`
(`vendor/ghostty/src/Surface.zig:3505-3573`) has **three** branches that
roastty's `mouse_scroll` is missing two of:

1. **alt-screen + `mouse_alternate_scroll` mode + no mouse-reporting** →
   translate the wheel to **cursor keys** (`\x1bOA`/`\x1bOB` app-mode or
   `\x1b[A`/`\x1b[B`) written to the PTY;
2. **mouse-reporting** → button-4/5/6/7 reports (roastty has this —
   `dispatch_scroll_reports`);
3. **otherwise (plain shell)** → `scrollViewport(.delta = -y.delta)` —
   **scrollback navigation**.

This experiment **ports branches (1) and (3)** into `mouse_scroll`. The
relative-scroll primitive already exists (`Scroll::DeltaRow` /
`screen.scroll_delta_row`, `page_list.rs:4933`); it's just not exposed to the
wheel path.

## Approach

**Phase 1 — the fix (port branches 1 & 3 into `mouse_scroll`).** Faithful to
upstream `scrollCallback`: after the existing mouse-reporting handling, when
**not** mouse-reporting —

- if alt-screen + `mouse_alternate_scroll` mode → write cursor-key sequences to
  the PTY (branch 1);
- else → scroll the viewport by the wheel delta (branch 3). Expose a `Terminal`
  viewport-delta scroll (wrapping `screen.scroll_delta_row`) and call it;
  compute the line delta from the wheel `y` (line-mode; precision/fractional
  fidelity can be a follow-up — get the line-step behavior right first, matching
  upstream sign: viewport delta = `-y`).

**Phase 2 — headless regression test (per the Exp-22 lesson).** Drive
`Surface::mouse_scroll(...)` on a **non-mouse-reporting** surface with
scrollback content (`seq`-like fill) and assert, **via
`shape_run_options()`/`FrameTerminalSnapshot::collect`** (the render read-path,
not a generic dump), that scroll-up shows earlier rows and scroll-to-bottom
shows the tail. A separate test asserts branch 1 (alt-screen + alt-scroll → the
cursor-key bytes are queued to the PTY). This fails pre-port and passes after.

**Phase 3 — live confirmation (the scroll driver).** Build
`scripts/roastty-app/scroll.swift` (`CGEventCreateScrollWheelEvent` +
`CGEventPost(.cghidEventTap)` at the window center — the review confirmed
`.cghidEventTap` routes to the window **under the cursor**, avoiding the
frontmost-keystroke pitfall; raise + warp cursor over the window, **restore the
cursor after**). **Validate the driver independently** of the (newly-ported)
viewport path — against a mouse-reporting program where scroll has a
current-code effect — then probe: `ZDOTDIR/.zshrc` `seq 1 200`, capture the
tail, scroll up, capture (earlier lines render), scroll down (tail returns).
Scrollback is retained: the live surface uses `Terminal::init(.., None)` =
`usize::MAX` (`termio.rs:114`, unlimited), not disabled.

This touches **only `libroastty`** (`mouse_scroll` + a `Terminal` scroll-delta
accessor) + a test-only `scroll.swift`. No app changes.

## Verification

1. **Headless regression test** through `mouse_scroll` + the render read-path:
   fails pre-port, passes after (scroll-up shows earlier rows; scroll-bottom
   shows the tail; alt-scroll → cursor keys). **`cargo test -p roastty`** (full)
   green.
2. **Live confirmation:** the scroll driver is validated independently; then the
   captures (out-of-repo) show **history on scroll-up + the tail on
   scroll-down**. App + descendant tree killed (0 dangling); screen **unlocked**
   (check `CGSSessionScreenIsLocked` first — `screencapture` is black when
   locked); cursor restored.
3. Faithful to upstream `scrollCallback` (cite the branches ported).

**Pass** = branches (1) & (3) are ported, the headless regression test passes,
the suite is green, and the live app shows scrollback navigation (history on
scroll-up, tail on scroll-down).

**Partial** = viewport scrolling (branch 3) works + tested, but a sub-aspect is
deferred (e.g. precision/fractional scroll fidelity, or branch 1 alt-scroll if
it proves larger) — documented.

**Fail** = the port can't be made to scroll the viewport (documented with the
blocker).

## Design Review

**Reviewer:** `adversarial-reviewer` subagent (Claude Opus, fresh context,
read-only). **Verdict: CHANGES REQUIRED → addressed.** It **corrected the
central premise**: `mouse_scroll` calls only `dispatch_scroll_reports`
(mouse-reporting only); the `scroll_viewport_*` functions are unreachable from
the wheel — upstream `scrollCallback` has three branches and roastty is
**missing two** (the alt-scroll cursor-keys branch and the plain-shell
viewport-scroll branch). So this is a **feature port**, not a
verify-existing-path probe. Two Required + two Optional + a Nit, folded in:

- **Required — wrong premise.** Reframed to "port the missing `scrollCallback`
  branches".
- **Required — the driver-validation gate was unsound** (a CGEvent scroll has no
  viewport effect in a plain shell _because the code is missing_, so "scroll
  moves viewport" can't validate the driver). **Fixed:** validate the driver
  independently against a mouse-reporting program; lead with a **headless**
  regression test through `mouse_scroll` + the render read-path.
- **Optional — regression-test layer** (`scroll_viewport_to_row` already works →
  a direct test is vacuous). **Fixed:** assert through `Surface::mouse_scroll` +
  `render_rows_snapshot`.
- **Optional — scrollback capacity** unstated. **Fixed:** noted
  `Terminal::init(.., None)` = `usize::MAX` (unlimited, `termio.rs:114`).
- **Nit — restore the cursor** after warping. **Fixed.**

It also confirmed `.cghidEventTap` scroll routes to the window-under-cursor (so
the driver itself is sound, unlike the frontmost-keystroke path).

## Result

_(to be added after the run.)_

## Conclusion

_(to be added after the run.)_
