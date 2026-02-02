# Issue 337: Browser Refresh (Cmd+R)

## Product Requirements

### User Story

As a user browsing the web in TermSurf, I want to refresh the current page using
the familiar Cmd+R keyboard shortcut, so that I can reload content without
retyping the URL.

### Acceptance Criteria

1. **Cmd+R** reloads the current page
2. **Cmd+Shift+R** performs a hard reload (ignore cache)
3. Refresh works in both Browse mode and Control mode
4. Works when a webview pane is focused

### Keybindings

| Shortcut    | Action                     | Notes                        |
| ----------- | -------------------------- | ---------------------------- |
| Cmd+R       | Reload page                | Standard browser shortcut    |
| Cmd+Shift+R | Reload page (ignore cache) | Hard refresh, bypasses cache |

### Non-Requirements (Out of Scope)

- Loading indicator during refresh (future enhancement)
- Pull-to-refresh gesture (not applicable to terminal)

## Technical Context

This follows the same pattern as issue 335 (back/forward navigation):

1. GUI intercepts Cmd+R / Cmd+Shift+R in `keyevent.rs`
2. Sends XPC message to profile server
3. Profile server calls CEF's `browser.reload()` or
   `browser.reload_ignore_cache()`

### CEF Methods

From `cef-rs` bindings, the Browser object has:

- `reload()` — Normal reload
- `reload_ignore_cache()` — Hard reload, bypasses cache

## Files Involved

- `ts3/wezterm-gui/src/termwindow/webview_xpc.rs` — Add `send_reload()` method
- `ts3/wezterm-gui/src/termwindow/keyevent.rs` — Intercept Cmd+R / Cmd+Shift+R
- `ts3/termsurf-profile/src/main.rs` — Add ReloadTask and XPC handler

---

## Experiments

(To be designed)
