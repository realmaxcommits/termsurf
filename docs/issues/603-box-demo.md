# Issue 603: Box Demo in Ghost

## Goal

Render live Chromium frames in Ghost. The `web` TUI opens a URL, Ghost spawns a
Chromium Profile Server, receives IOSurface Mach ports at 60fps, and renders
them as a textured overlay at the correct grid coordinates. The box demo
(spinning blue square) is the test page.

## Background

Issue 602 proved the pink overlay pipeline works — a GPU quad renders at grid
coordinates specified by `web`, survives resize, and clears on disconnect. This
issue replaces the pink quad with live Chromium frames.

### What we have

**Ghost (from Issues 601–602):**

- XPC gateway connection, anonymous listener, endpoint registration
- Message parsing (`set_overlay`, `mode_changed`)
- Pane ID on Surface, propagated as `TERMSURF_PANE_ID`
- Surface lookup by pane ID
- Pink overlay shader, pipeline, renderer state, render step in `drawFrame()`
- `setOverlay()` / `clearOverlay()` with `draw_mutex` thread safety
- XPC handler wired to surface methods

**Chromium Profile Server (from ts5 Issues 503–515):**

- Full XPC protocol: gateway connect, `server_register`, `create_tab`,
  `tab_ready`, `display_surface`, `resize`, mouse/scroll/focus forwarding
- 120fps IOSurface capture via `FrameSinkVideoCapturer`
- Mach port transfer: `IOSurfaceCreateMachPort` → `xpc_dictionary_set_mach_send`
- Per-tab pane routing, auto-exit on last tab close
- Current branch: `146.0.7650.0-issue-515` (latest, all features)

**`web` TUI (already sends URL):**

- `send_set_overlay` includes `url`, `profile`, `browsing` fields
- No changes needed to `web`

### What we need to build

1. **Copy box demo** — Move `ts4/box-demo/` to top-level `box-demo/`
2. **Fork Chromium branch** — Create `146.0.7650.0-issue-603` from the latest
   working branch. No Chromium source changes expected.
3. **IOSurface overlay shader** — Textured overlay vertex/fragment in
   `shaders.metal` (samples IOSurface texture instead of returning pink)
4. **IOSurface overlay pipeline** — `overlay` pipeline in `shaders.zig`
5. **IOSurface texture creation** — `Texture.fromIOSurface()` in Ghost's
   `Texture.zig` using `MTLDevice.newTextureWithDescriptor:iosurface:plane:`
6. **IOSurface state on renderer** — `overlay_iosurface` pointer field,
   `overlay_surface_changed` flag
7. **`setOverlayIOSurface()` on Surface** — Thread-safe IOSurface update with
   `CFRetain` / `CFRelease` under `draw_mutex`
8. **Render path in `drawFrame()`** — If IOSurface present, use textured overlay
   pipeline; otherwise fall back to pink
9. **Chromium server lifecycle in XPC** — Handle `server_register`, send
   `create_tab`, handle `display_surface` (Mach port → IOSurface → renderer)
10. **Server spawning** — Launch `Chromium Profile Server.app` with the right
    flags when `set_overlay` arrives with a URL

### Chromium server XPC protocol

Messages Ghost must handle from the Chromium server:

| Message           | Fields                          | Frequency   |
| ----------------- | ------------------------------- | ----------- |
| `server_register` | action, profile                 | Once        |
| `tab_ready`       | action, tab_id                  | Once/tab    |
| `display_surface` | action, pane_id, iosurface_port | 60fps       |
| `url_changed`     | action, pane_id, url            | On navigate |
| `cursor_changed`  | action, pane_id, cursor_type    | On change   |

Messages Ghost must send to the Chromium server:

| Message      | Fields                                          | When           |
| ------------ | ----------------------------------------------- | -------------- |
| `create_tab` | action, url, pane_id, pixel_width, pixel_height | After register |
| `resize`     | action, pane_id, pixel_width, pixel_height      | On resize      |

The server connects to the xpc-gateway, sends `{ action: "connect" }`, receives
Ghost's endpoint, connects directly, and sends `server_register`. Ghost replies
with `create_tab`. The server then streams `display_surface` at 60fps.

### Mach port transfer in Zig

The `display_surface` message carries an IOSurface Mach port. In Zig:

```zig
extern "c" fn xpc_dictionary_copy_mach_send(xdict: xpc_object_t, key: [*:0]const u8) u32;
extern "c" fn IOSurfaceLookupFromMachPort(port: u32) ?*anyopaque;
extern "c" fn mach_port_deallocate(task: u32, name: u32) i32;
extern "c" fn mach_task_self() u32;
```

Flow:

1. `xpc_dictionary_copy_mach_send(msg, "iosurface_port")` → Mach port
2. `IOSurfaceLookupFromMachPort(port)` → IOSurfaceRef
3. `mach_port_deallocate(mach_task_self(), port)` — clean up kernel reference
4. Pass IOSurfaceRef to `surface.setOverlayIOSurface()`

### IOSurface texture creation

`MTLDevice.newTextureWithDescriptor:iosurface:plane:` creates a zero-copy
MTLTexture view into the IOSurface's GPU memory. From ts5's Texture.zig:

```zig
pub fn fromIOSurface(device: objc.Object, iosurface: *anyopaque) ?Self {
    const width: usize = IOSurfaceGetWidth(iosurface);
    const height: usize = IOSurfaceGetHeight(iosurface);
    // Create MTLTextureDescriptor with bgra8unorm, shader-read usage
    // Call device.newTextureWithDescriptor:iosurface:plane:
}
```

### Server spawning

The Chromium Profile Server binary lives at:

```
chromium/src/out/Default/Chromium Profile Server.app/Contents/MacOS/Chromium Profile Server
```

Launch arguments:

```
--xpc-service=com.termsurf.xpc-gateway
--user-data-dir=~/.config/termsurf/chromium-profiles/{profile}
--hidden
```

Ghost spawns it via `std.process.Child` (Zig's process API). One server per
profile — multiple panes with the same profile share one server.

### Key technical details

**Pixel dimensions for `create_tab`:** The server needs physical pixel
dimensions, not grid cells. Ghost computes them:
`pixel_width = grid_width * cell_width`,
`pixel_height = grid_height * cell_height`. Cell size comes from
`renderer.grid_metrics.cell_width/cell_height`.

**Thread safety:** `display_surface` arrives at 60fps on the XPC queue
(background thread). `setOverlayIOSurface()` locks `draw_mutex`, swaps the
IOSurface pointer with `CFRetain`/`CFRelease`, sets
`overlay_surface_changed = true`, and queues a render. `drawFrame()` holds
`draw_mutex` and creates an MTLTexture from the current IOSurface each frame.

**Server peer vs web peer:** Ghost's XPC listener now accepts two kinds of
peers: `web` processes (send `set_overlay`) and Chromium servers (send
`server_register`). The listener handler must distinguish them by the first
message received.

## Ideas for experiments

1. **IOSurface texture pipeline** — Add the textured overlay shader, pipeline,
   `fromIOSurface()`, and renderer state. Test with a programmatically created
   IOSurface (no Chromium needed). Proves the texture path works in Zig.

2. **Chromium server lifecycle** — Spawn the server, handle `server_register`,
   send `create_tab`, handle `display_surface`. Box demo renders in the terminal
   at 60fps. Full end-to-end proof.

3. **Resize** — Resize the terminal, Ghost sends `resize` to the server, the
   server adjusts capture resolution, frames continue at the new size.
