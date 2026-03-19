+++
status = "open"
opened = "2026-03-19"
+++

# Issue 761: Browser engine label missing when using default

## Goal

The viewport bottom-right should always show the browser engine name, even when
the user doesn't pass `--browser`.

## Background

### The problem

When the user runs `web --browser /path/to/roamium`, the TUI extracts the last
path component ("roamium") and displays it in the viewport bottom-right. But
when the user runs just `web localhost:3000` without `--browser`, the label is
empty — the user gets no indication of which engine is rendering the page.

### Root cause

In `webtui/src/main.rs`, the `browser` variable is set from the CLI argument:

```rust
let mut browser = cli.browser.unwrap_or_default();  // "" when omitted
```

The TUI passes this empty string to the GUI in `SetOverlay.browser`. The GUI
resolves the default engine internally and launches it, but never tells the TUI
what it chose. The `BrowserReady` message (GUI → TUI) contains `pane_id`,
`tab_id`, and `browser_socket` — but not the browser name.

### Fix

Add a `browser` field to the `BrowserReady` message:

```protobuf
message BrowserReady {
  string pane_id = 1;
  int64 tab_id = 2;
  string browser_socket = 3;
  string browser = 4;          // resolved browser binary path
}
```

When the GUI sends `BrowserReady`, it includes the actual browser path it
launched. The TUI updates its `browser` variable from this field, and the
viewport label updates on the next render.

### Scope

Three layers:

1. **Protocol (`termsurf.proto`)** — Add `browser` field to `BrowserReady`.
2. **Wezboard (GUI)** — Populate the new field when sending `BrowserReady`.
3. **TUI (`webtui`)** — Read the field from `BrowserReady` and update the
   `browser` variable.
