+++
status = "open"
opened = "2026-03-17"
+++

# Issue 759: Show link URL on hover

## Goal

When the user hovers over a link in the browser overlay, the TUI displays the
destination URL. When the mouse leaves the link, the URL disappears.

## Background

### The problem

Currently, hovering over a link gives the user no indication of where it leads.
In a normal browser, the destination URL appears in a status bar at the bottom
of the window. TermSurf has no equivalent.

### How Chromium handles it

Chromium has a well-defined pipeline for link hover URLs:

1. **Blink** detects the hover via hit testing in
   `ChromeClientImpl::ShowMouseOverURL()`. It extracts the URL from
   `HitTestResult::AbsoluteLinkURL()`.

2. **WebViewImpl** rate-limits updates using an async request-reply pattern. It
   won't send the next update until the browser process ACKs the previous one.
   This prevents flooding the IPC channel when the user moves the mouse quickly
   across many links.

3. **Mojo IPC** carries the URL from renderer to browser via
   `LocalMainFrameHost::UpdateTargetURL(url) => ()`.

4. **RenderFrameHostImpl** receives the message, notifies `WebContentsImpl`,
   which routes to `WebContentsDelegate::UpdateTargetURL(source, url)`.

5. **The embedder** decides what to do. Chrome sends it to `StatusBubble` (the
   translucent overlay in the bottom-left corner). Content Shell does nothing —
   it doesn't override `UpdateTargetURL()`.

### What we need

Three changes across three layers:

1. **Chromium (`shell.cc`)** — Override `UpdateTargetURL()` on the Shell class.
   Send a new protobuf message through the existing IPC socket to the GUI.

2. **Protocol (`termsurf.proto`)** — Add a `TargetUrlChanged` message. Follows
   the same pattern as `UrlChanged`, `TitleChanged`, etc. Sent from Chromium to
   the GUI, and from the GUI (via the browser socket) to the TUI.

3. **TUI (`webtui`)** — Receive the message and display the URL in the browser
   chrome. The URL bar area or a dedicated status line at the bottom are natural
   places.

### Message design

```protobuf
message TargetUrlChanged {
  int64 tab_id = 1;
  string url = 2;    // empty string = mouse left the link
}
```

An empty `url` means the hover ended. The TUI clears the display.

This follows the existing pattern: `UrlChanged` for the page URL, `TitleChanged`
for the page title, `TargetUrlChanged` for the link hover URL. All three are
Chromium → GUI → TUI state messages with the same shape.

### Message flow

```
Blink (hover detected)
  → Mojo IPC → RenderFrameHostImpl → WebContentsImpl
  → Shell::UpdateTargetURL(source, url)
  → protobuf TargetUrlChanged → Unix socket → GUI (Wezboard)
  → protobuf TargetUrlChanged → browser socket → TUI (webtui)
  → TUI displays URL in status area
```

### Rate limiting

Blink already rate-limits hover updates (one in-flight at a time, queues the
next). We don't need additional throttling — the messages arrive at a reasonable
rate.
