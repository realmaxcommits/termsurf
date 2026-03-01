# Issue 687: One DevTools Per Tab

Enforce that only one DevTools session can be open per inspected browser tab.
Opening a second DevTools for the same tab should be rejected with an error
message instead of silently creating a duplicate that crashes the renderer.

## Background

Issue 686 found that opening two DevTools panes for the same inspected tab
causes a DCHECK crash in Chromium's `PaintController`. Both DevTools sessions
attach an `InspectorOverlayAgent` to the same renderer, producing duplicate
`DisplayItem::Id` entries during overlay painting. Chromium enforces one
DevTools frontend per inspected page internally — TermSurf bypasses that by
creating independent `ShellDevToolsFrontend` instances.

## Where to Enforce

The check can happen at three levels:

1. **Chromium (`CreateDevToolsTab`)** — before creating the
   `ShellDevToolsFrontend`, check if any existing tab already has
   `inspected_tab_id == N`. If so, reject the request and send an error back.
   This is the safest level — it's impossible to bypass.

2. **GUI (`handleSetDevtoolsOverlay`)** — before forwarding
   `create_devtools_tab` to Chromium, check if any pane already has
   `inspected_tab_id == N` for the same server. Faster feedback — no round-trip
   to Chromium.

3. **TUI** — before sending the XPC message. Requires the TUI to know what
   DevTools sessions are already open, which it currently doesn't.

## Relevant Code

- `chromium/src/content/chromium_profile_server/browser/shell_browser_main_parts.cc`
  — `CreateDevToolsTab`, `tabs_` vector
- `gui/src/apprt/xpc.zig` — `handleSetDevtoolsOverlay`, `panes` map,
  `inspected_tab_id` field on `Pane`
- `tui/src/main.rs` — DevTools detection and `send_set_devtools_overlay`
