# Issue 685: Multi-Profile Tracking

Fix `web last` and `web devtools` auto-targeting to work correctly when multiple
browser profiles are open simultaneously.

## Background

Issue 684 introduced `last_browser_pane` — a single global variable that tracks
the most recently active browser pane. It's updated in two places:

1. `handleTabReady` — when a browser tab is created (`tab_id > 0`)
2. `handlePaneFocusChanged` — when a non-DevTools pane with `tab_id > 0` gains
   focus

Both `web last` and `web devtools` auto-targeting depend on this global.

## Problem

The single global breaks with multiple profiles:

1. **`web last` fails entirely with multiple profiles open.** Open a browser
   with the default profile, then open another with the "work" profile.
   `web last` (no filter) returns "No active browser tab found." instead of the
   work profile's pane info. The root cause needs investigation — the global
   should point to the most recent pane regardless of profile.
2. **`web last --profile default` fails when "work" was opened last.** The
   profile filter only checks `last_browser_pane`. If that pane belongs to
   "work", the filter rejects it and returns nothing. It does not search other
   panes.
3. **`web last --profile work` works** only because the global happens to point
   to the work pane (most recently created).
4. **`web devtools` auto-targeting has the same limitation.** It uses the same
   `last_browser_pane` global, so it can only target the single most recent
   browser pane.

## Relevant Code

- `gui/src/apprt/xpc.zig` — `last_browser_pane` global (line 119),
  `handleTabReady` (line 614), `handlePaneFocusChanged` (line 900),
  `handleQueryLast` (line 790), DevTools auto-targeting (line 490)
- `tui/src/main.rs` — `Commands::Last` subcommand
- `tui/src/xpc.rs` — `send_query_last`
