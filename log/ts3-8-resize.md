# TermSurf 3.0 Dynamic Browser Resize

## Background

### Progress So Far

ts3-7 completed the one-process-per-profile architecture:

- Multiple webviews can share a single CEF process
- Each webview renders within its pane bounds
- The launcher routes requests to existing profile processes
- IOSurface Mach ports are correctly sent to the right GUI panes

The webview rendering pipeline now works for the **initial render**. When a user
runs `web google.com`, the page renders at the correct pane size and displays
within the pane bounds.

### The Problem

When the user resizes the window or splits panes, the webview does not re-render
at the new size. Instead, the existing texture is stretched or compressed to fit
the new viewport dimensions. This produces a blurry, distorted image that does
not match what a user expects from a browser.

**Current behavior:**

1. User runs `web google.com` in a full-width pane
2. Page renders correctly at 1200x800
3. User splits the pane vertically (now 600x800)
4. The 1200x800 texture is squished into 600x800 viewport
5. Text becomes unreadable, images distorted

**Expected behavior:**

1. User runs `web google.com` in a full-width pane
2. Page renders correctly at 1200x800
3. User splits the pane vertically (now 600x800)
4. CEF re-renders the page at 600x800
5. Page reflows naturally, text remains crisp

This is how every browser works. When you resize a browser window, the page
reflows to fit the new width. TermSurf must do the same.

## Goal

When a pane containing a webview is resized, the browser must re-render at the
new dimensions so the page content reflows naturally.

## Product Requirements

### Core Requirement

**When a webview pane changes size, the browser must resize to match.**

This applies to all resize scenarios:

1. **Window resize** — User drags the window edge to make it larger or smaller
2. **Pane split** — User splits a pane, reducing the original pane's size
3. **Pane close** — User closes an adjacent pane, expanding the remaining pane
4. **Divider drag** — User drags the divider between panes to adjust sizes

In all cases, the webview must re-render at the new size, not stretch the old
texture.

### User Experience

**Resize should feel responsive.** The page should update quickly enough that
the user perceives it as "live" resizing, similar to resizing a Chrome or Safari
window.

**Content should reflow naturally.** Text should wrap to fit the new width.
Images should maintain aspect ratio. Responsive layouts should adapt.

**No visual artifacts.** During resize, it's acceptable to briefly show the
stretched old texture while waiting for the new render. However, the final state
must always be a correctly-sized render.

### Edge Cases

1. **Rapid resizing** — User drags window edge continuously. The system should
   debounce or throttle resize events to avoid overwhelming CEF with resize
   requests. A brief delay (e.g., 30-50ms settle time) before triggering
   re-render is acceptable.

2. **Multiple webviews** — When a window resize affects multiple webview panes,
   all of them must resize. Each webview is independent; they may complete their
   re-renders at different times.

3. **Minimum size** — There may be a minimum practical size for webviews. If a
   pane becomes too small (e.g., < 100px in either dimension), the webview
   behavior is undefined. It's acceptable to show a placeholder or simply not
   render.

4. **Profile process crash** — If the profile process crashes during resize, the
   GUI should handle this gracefully (e.g., show an error state in the pane).

### Non-Requirements (Deferred)

The following are explicitly **not** part of this task:

- **Zoom level** — Changing the page zoom (Cmd+/Cmd-) is separate from resize
- **DPI changes** — Moving window between Retina and non-Retina displays
- **Scroll position preservation** — Maintaining scroll position across resize
  (nice to have, but not required)

## Success Criteria

- [ ] Resize window → webview re-renders at new size
- [ ] Split pane → webview re-renders at new size
- [ ] Close adjacent pane → webview re-renders at new size
- [ ] Drag pane divider → webview re-renders at new size
- [ ] Text remains crisp and readable after resize
- [ ] Page content reflows naturally (responsive layouts work)
- [ ] Multiple webviews in same window all resize correctly
- [ ] Resize feels responsive (not sluggish)

## Tasks

- [ ] Detect pane resize events in the GUI
- [ ] Send new dimensions to the profile server
- [ ] Profile server calls CEF resize API
- [ ] CEF re-renders at new size
- [ ] New IOSurface sent to GUI
- [ ] GUI updates viewport to match new size
- [ ] Implement debounce/throttle for rapid resize events

## Research

### IPC Decision: XPC over Unix Sockets

ts3 uses two IPC mechanisms:

- **XPC** — GUI ↔ Launcher, Launcher → Profile, Profile → GUI (IOSurface transfer)
- **Unix domain sockets** — CLI → GUI (the `web` command)

For GUI → Profile resize communication, **XPC is the correct choice**:

1. Profile server already has an XPC command listener (used by launcher for
   `create_browser` commands)
2. XPC is already used for the Profile → GUI direction
3. The architecture is macOS-specific anyway (IOSurface, Mach ports)
4. Adding Unix sockets would require profile server to manage two IPC mechanisms

### Current Communication Gap

The current XPC flow is **one-way**:

```
Profile Server ──display_surface──▶ GUI (working)
GUI ──???──▶ Profile Server (not implemented)
```

The profile server creates a command listener in `main.rs:216-250` and registers
it with the launcher. The launcher connects to send `create_browser` commands.
But the GUI never receives this endpoint—it can only receive surfaces, not send
commands back.

**Solution:** Profile server must share its command endpoint with the GUI. The
simplest approach is to include it in the first `display_surface` message.

### How ts2 Handles Resize

ts2 implements resize in `ts2/wezterm-gui/src/cef_browser/mod.rs:262-291` and
`ts2/wezterm-gui/src/termwindow/render/pane.rs:813-880`:

1. **Debounce with 30ms settle delay** — Every frame, check if size changed. If
   so, record the pending size and mark the time. Only send resize after 30ms of
   no further changes.

2. **CEF resize API** — Call `host.was_resized()` to notify CEF that dimensions
   changed, then `host.invalidate()` to force a repaint.

3. **Message loop pump** — ts2 calls `cef::do_message_loop_work()` after resize
   because CEF runs in-process and shares the event loop. ts3 does NOT need this
   because the profile server has its own process and event loop.

Key code from ts2:

```rust
const SETTLE_DELAY: Duration = Duration::from_millis(30);

// In BrowserState::resize()
host.was_resized();
host.invalidate(PaintElementType::default());
```

### CEF Automatic Re-rendering

CEF automatically handles animation and content updates without explicit resize:

- `windowless_frame_rate: 60` causes CEF to render at 60 FPS
- When content changes (animations, scrolling, DOM updates), CEF renders a new
  frame and calls `on_accelerated_paint`
- The profile server's dedup logic detects when the IOSurface buffer pointer
  changes and sends the new Mach port to GUI

**This already works.** Animations and dynamic content automatically flow to the
GUI. The only missing piece is triggering a re-render when the **size** changes.

### ts3 Cell-Based Sizing

Unlike ts2 which uses exact pixel dimensions, ts3 sizes browsers to cell
boundaries (`cols × cell_width`, `rows × cell_height`). This means:

- Fewer resize events (size only changes when grid dimensions change)
- More predictable dimensions
- Slightly less precise fit, but acceptable for terminal integration

The 30ms debounce is still valuable for rapid window resizing, but cell-based
sizing naturally reduces resize frequency.

### CEF Thread Safety

XPC callbacks run on libdispatch queues, not the CEF UI thread. Browser
operations (including resize) must be marshalled to the CEF UI thread:

```rust
cef::post_task(cef::ThreadId::UI, move || {
    // Safe to call host.was_resized() here
});
```

### Key Files

| File | Role |
|------|------|
| `ts3/termsurf-profile/src/main.rs` | Command listener, resize handler |
| `ts3/wezterm-gui/src/termwindow/webview_xpc.rs` | Store command connection |
| `ts3/wezterm-gui/src/termwindow/render/draw.rs` | Detect size change, debounce |
| `ts2/wezterm-gui/src/cef_browser/mod.rs` | Reference implementation |
| `ts2/wezterm-gui/src/termwindow/render/pane.rs` | Reference debounce logic |

## Experiments

### Experiment 1: Implement Dynamic Resize via Bidirectional XPC

**Status:** PLANNED

**Goal:** Enable the GUI to send resize commands to the profile server so that
webviews re-render at the correct size when panes are resized.

#### Key Insight: XPC Connections Are Bidirectional

XPC connections are inherently bidirectional. When the profile server connects
to the GUI via `gui_endpoint`, that same connection can be used by the GUI to
send commands back. No separate command channel is needed.

Current state:
- Profile server connects to GUI → sends `display_surface`
- GUI stores connection in a `Vec` (index only, no pane mapping)
- Profile server's event handler only logs errors

To enable resize:
- GUI stores connection by `pane_id` in a `HashMap`
- Profile server's event handler processes incoming commands
- GUI sends `resize_browser` on the existing connection

#### Architecture

```
GUI                                     Profile Server
┌─────────────────────┐                 ┌─────────────────────────┐
│                     │                 │                         │
│  Pane 0 (google)    │                 │  Browser 0 (google)     │
│    ┌───────────────┐│                 │    ┌───────────────────┐│
│    │ peer_conn[0] ◄┼┼──display_surface┼────┤ gui connection    ││
│    │               ├┼┼──resize_browser─────►                   ││
│    └───────────────┘│                 │    └───────────────────┘│
│                     │                 │                         │
│  Pane 1 (github)    │                 │  Browser 1 (github)     │
│    ┌───────────────┐│                 │    ┌───────────────────┐│
│    │ peer_conn[1] ◄┼┼──display_surface┼────┤ gui connection    ││
│    │               ├┼┼──resize_browser─────►                   ││
│    └───────────────┘│                 │    └───────────────────┘│
│                     │                 │                         │
└─────────────────────┘                 └─────────────────────────┘
```

Each pane has its own bidirectional connection. No session_id routing needed—
the connection itself identifies the browser.

#### Changes

**1. GUI: Store peer connections by pane_id**

**File:** `ts3/wezterm-gui/src/termwindow/webview_xpc.rs`

Change from `Vec` to `HashMap`:

```rust
pub struct XpcManager {
    // ... existing fields ...

    /// Peer connections from profile servers, keyed by pane_id
    /// Used to send commands (resize, input) back to the browser
    peer_connections: Mutex<HashMap<u64, Arc<XpcConnection>>>,
}
```

In `new_connection_handler` (around line 227), store by pane_id instead of
pushing to Vec:

```rust
// Store the peer connection for sending commands back
// The pane_id comes from the display_surface message
manager.peer_connections.lock().unwrap().insert(pane_id, Arc::new(conn));
log::info!("[XPC] Stored peer connection for pane {}", pane_id);
```

**Issue:** The pane_id is extracted from the first `display_surface` message,
but we receive the connection before the first message. Two options:

1. Store connection temporarily, then move to HashMap when first message arrives
2. Have profile server send pane_id in an initial handshake message

**Simpler approach:** Store in the `display_surface` handler:

```rust
"display_surface" => {
    // ... existing extraction ...

    // Store peer connection if not already stored
    if !manager.peer_connections.lock().unwrap().contains_key(&pane_id) {
        // The 'conn' here is the connection that sent this message
        // We need to capture it in the handler closure
        manager.peer_connections.lock().unwrap().insert(pane_id, conn.clone());
        log::info!("[XPC] Stored peer connection for pane {}", pane_id);
    }
}
```

This requires passing `conn` into the message handler, which may need
restructuring of the event handler closure.

**Alternative:** Send pane_id in a separate `register_browser` message before
`display_surface`:

```rust
// In profile server, after connecting to GUI:
let register_msg = XpcDictionary::new();
register_msg.set_string("action", "register_browser");
register_msg.set_i64("pane_id", pane_id as i64);
gui.send(&register_msg);
```

Then GUI stores connection when `register_browser` arrives. This is cleaner.

**2. GUI: Add method to send resize command**

**File:** `ts3/wezterm-gui/src/termwindow/webview_xpc.rs`

```rust
impl XpcManager {
    /// Send a resize command to the browser in the given pane
    pub fn send_resize(&self, pane_id: u64, width: u32, height: u32) -> bool {
        let connections = self.peer_connections.lock().unwrap();
        if let Some(conn) = connections.get(&pane_id) {
            let msg = termsurf_xpc::XpcDictionary::new();
            msg.set_string("action", "resize_browser");
            msg.set_i64("width", width as i64);
            msg.set_i64("height", height as i64);
            conn.send(&msg);
            log::info!("[XPC] Sent resize to pane {}: {}x{}", pane_id, width, height);
            true
        } else {
            log::warn!("[XPC] No connection for pane {} to send resize", pane_id);
            false
        }
    }
}
```

**3. Profile Server: Send `register_browser` message**

**File:** `ts3/termsurf-profile/src/main.rs`

In `create_browser_on_ui_thread`, after connecting to GUI:

```rust
// Send registration so GUI can map connection to pane
let register_msg = XpcDictionary::new();
register_msg.set_string("action", "register_browser");
register_msg.set_i64("pane_id", pane_id as i64);
gui.send(&register_msg);
println!("Profile: Sent register_browser for pane {}", pane_id);
```

**4. Profile Server: Handle resize_browser command**

**File:** `ts3/termsurf-profile/src/main.rs`

Add event handler to the GUI connection to receive commands:

```rust
// After gui.resume():

let browser_state_clone = Arc::clone(&browser_state);
set_event_handler(&gui, move |event| {
    match event {
        Ok(msg) => {
            let action = msg.get_string("action").unwrap_or_default();
            match action.as_str() {
                "resize_browser" => {
                    let width = msg.get_i64("width") as u32;
                    let height = msg.get_i64("height") as u32;
                    println!("Profile: resize_browser {}x{}", width, height);

                    // Store pending resize and post to UI thread
                    let bs = Arc::clone(&browser_state_clone);
                    cef::post_task(cef::ThreadId::UI, move || {
                        resize_browser_on_ui_thread(width, height, &bs);
                    });
                }
                other => {
                    println!("Profile: Unknown command: {}", other);
                }
            }
        }
        Err(e) => {
            eprintln!("Profile: GUI connection error: {}", e);
        }
    }
});
```

**5. Profile Server: Store Browser reference for resize**

**File:** `ts3/termsurf-profile/src/main.rs`

`BrowserState` needs to hold the CEF `Browser` to call `was_resized()`:

```rust
struct BrowserState {
    session_id: String,
    gui: Arc<XpcConnection>,
    width: AtomicU32,
    height: AtomicU32,
    last_handle: AtomicPtr<c_void>,
    browser: Mutex<Option<Browser>>,  // NEW: for resize
}
```

In `create_browser_on_ui_thread`, store the browser:

```rust
match browser {
    Some(b) => {
        let browser_id = b.identifier();
        *browser_state.browser.lock().unwrap() = Some(b);
        // ... rest of existing code ...
    }
    None => { /* ... */ }
}
```

**6. Profile Server: Resize function**

**File:** `ts3/termsurf-profile/src/main.rs`

```rust
fn resize_browser_on_ui_thread(width: u32, height: u32, state: &BrowserState) {
    // Update stored dimensions
    state.width.store(width, Ordering::Relaxed);
    state.height.store(height, Ordering::Relaxed);

    // Notify CEF
    if let Some(ref browser) = *state.browser.lock().unwrap() {
        if let Some(host) = browser.host() {
            println!("Profile: Calling was_resized ({}x{})", width, height);
            host.was_resized();
            host.invalidate(cef::PaintElementType::View);
        }
    }
}
```

**7. GUI: Detect size change and debounce**

**File:** `ts3/wezterm-gui/src/termwindow/webview_socket.rs`

Add debounce state to `WebviewOverlay`:

```rust
pub struct WebviewOverlay {
    pub mach_port: u32,
    pub width: u32,
    pub height: u32,
    // Debounce state:
    pub last_sent_size: Option<(u32, u32)>,
    pub pending_size: Option<(u32, u32)>,
    pub last_resize_time: Option<std::time::Instant>,
}
```

**File:** `ts3/wezterm-gui/src/termwindow/render/draw.rs`

In `render_webview_overlays_webgpu`, after calculating viewport dimensions:

```rust
use std::time::{Duration, Instant};

const SETTLE_DELAY: Duration = Duration::from_millis(30);

// After calculating (viewport_x, viewport_y, viewport_w, viewport_h):

let current_size = (viewport_w as u32, viewport_h as u32);

// Check if size changed from what we last sent
let size_changed = overlay.last_sent_size != Some(current_size);

if size_changed {
    // Size changed - update pending and mark time
    if overlay.pending_size != Some(current_size) {
        overlay.pending_size = Some(current_size);
        overlay.last_resize_time = Some(Instant::now());
    }
}

// Check if we should send resize command
if let Some(pending) = overlay.pending_size {
    let should_send = overlay.last_resize_time
        .map(|t| t.elapsed() >= SETTLE_DELAY)
        .unwrap_or(false);

    if should_send {
        // Send resize via XPC
        if let Some(xpc_manager) = crate::termwindow::webview_xpc::get_xpc_manager() {
            xpc_manager.send_resize(*pane_id, pending.0, pending.1);
        }

        overlay.last_sent_size = Some(pending);
        overlay.pending_size = None;
        overlay.last_resize_time = None;
    }
}
```

**Note:** The overlay struct needs mutable access for debounce state. This may
require changing the overlay iteration to use mutable references, or storing
debounce state separately.

#### Files to Modify

| File | Changes |
|------|---------|
| `ts3/termsurf-profile/src/main.rs` | Send `register_browser`, handle `resize_browser` on gui connection, store Browser ref |
| `ts3/wezterm-gui/src/termwindow/webview_xpc.rs` | Store peer connections by pane_id, add `send_resize()` method |
| `ts3/wezterm-gui/src/termwindow/webview_socket.rs` | Add debounce state to WebviewOverlay |
| `ts3/wezterm-gui/src/termwindow/render/draw.rs` | Detect size change, 30ms debounce, call `send_resize()` |

#### Verification

```bash
cd ts3 && ./scripts/build-debug.sh --open

# Open a webview
web google.com

# Check that connection was registered
cat /tmp/termsurf-gui.log | grep "peer connection"
# Should show: "[XPC] Stored peer connection for pane 0"

cat /tmp/termsurf-profile-*.log | grep "register_browser"
# Should show: "Sent register_browser for pane 0"

# Split the pane (Cmd+Shift+D)
# The webview should re-render at the new smaller size

# Check profile logs for resize handling
cat /tmp/termsurf-profile-*.log | grep -i resize
# Should show: "resize_browser 640x768"
# Should show: "Calling was_resized (640x768)"

# Check GUI logs for resize commands
cat /tmp/termsurf-gui.log | grep "Sent resize"
# Should show: "[XPC] Sent resize to pane 0: 640x768"

# Open second webview
web github.com
# Should get its own connection
cat /tmp/termsurf-gui.log | grep "peer connection"
# Should show: "[XPC] Stored peer connection for pane 1"

# Both panes should resize correctly when window is resized

# Drag the window edge to resize
# Both webviews should update after 30ms settle delay
```

#### Success Criteria

- [ ] Profile server sends `register_browser` with pane_id
- [ ] GUI stores peer connection by pane_id in HashMap
- [ ] GUI can send `resize_browser` command via stored connection
- [ ] Profile server receives command on gui connection's event handler
- [ ] Profile server calls `was_resized()` and `invalidate()`
- [ ] CEF re-renders at new size
- [ ] New IOSurface is sent to GUI with correct dimensions
- [ ] Splitting a pane triggers resize command after 30ms
- [ ] Dragging window edge triggers resize after 30ms settle
- [ ] Text remains crisp after resize (not stretched)
- [ ] Multiple webviews each have independent connections
- [ ] Each webview resizes independently

#### Advantages Over Original Design

1. **Simpler** — No `register_commands`, no profile-level connections, no
   session_id routing
2. **Natural mapping** — Connection ↔ browser is 1:1, no lookup needed
3. **Reuses existing infrastructure** — Same connection used for both directions
4. **Less state to manage** — No separate command listener, no profile tracking

#### Risks and Mitigations

1. **Connection capture in handler** — Need to pass connection reference into
   the `display_surface` handler. May require restructuring closure capture.

2. **Debounce timing** — 30ms may be too short or too long. Start with 30ms (ts2
   default), adjust if needed based on feel.

3. **Race conditions** — Resize commands may arrive while CEF is already
   rendering. CEF should handle this gracefully, but watch for crashes or
   hangs.

4. **Browser reference lifetime** — Storing `Browser` in `BrowserState` requires
   careful lifetime management. The browser must outlive the state, or we need
   weak references.

5. **Mutable overlay access** — Debounce state requires mutable access to
   overlays during render. May need to store debounce state separately or
   restructure the render loop.
