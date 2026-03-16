# Issue 755: Scroll broken in neovim when webviews are open

## Goal

Mouse scroll works in fullscreen TUIs like neovim at all times — whether browser
overlays are open or not.

## Background

### The bug

Scrolling with the Apple Magic Mouse in neovim does not work in Wezboard when
any browser overlay is open. It works fine in three other cases:

1. Scrolling in neovim in **WezTerm** (upstream) — works
2. Scrolling in neovim in **Wezboard with no webviews open** — works
3. Scrolling in neovim in **Wezboard with webviews open** — broken

The mere presence of a browser overlay somewhere in the window breaks scroll
event delivery to fullscreen TUIs, even when the cursor is not over the overlay.

### What we changed

Issue 731 added `RawScrollEvent` to the window layer (`window.rs`) to forward
scroll phase data to browser overlays. This event is dispatched before the
normal `VertWheel`/`HorzWheel` mouse events that drive terminal scrolling.
WezTerm does not have `RawScrollEvent` at all.

Issue 752 changed the scroll handler to iterate all overlay panes
(`try_forward_scroll_any_pane`) instead of only the active pane. The handler
sets `raw_scroll_consumed` based on whether any overlay consumed the scroll. A
flag in `mouseevent.rs` checks `raw_scroll_consumed` and suppresses the
duplicate wheel event if the raw scroll was already forwarded to a browser.

### What needs to happen

Find why the presence of browser overlays interferes with scroll event delivery
to terminal panes and fix it. The scroll path must work correctly whether zero,
one, or many browser overlays are open.
