# Issue 319: Mouse Input for Webview

## Goal

Enable mouse interaction with webviews in ts3. Users should be able to click,
drag, and scroll within browser panes using their mouse or trackpad.

## Status

Not started.

## Requirements

### Clicking

| Requirement  | Description                                       |
| ------------ | ------------------------------------------------- |
| Left click   | Click links, buttons, form elements               |
| Right click  | No action (context menu deferred to future issue) |
| Middle click | No action for now                                 |
| Double-click | Select word                                       |
| Triple-click | Select line/paragraph                             |

### Dragging

| Requirement    | Description                                |
| -------------- | ------------------------------------------ |
| Text selection | Click and drag to select text              |
| Drag handles   | Drag slider controls, resize handles, etc. |
| Scroll by drag | Drag scrollbars to scroll content          |

### Scrolling

| Requirement       | Description                                               |
| ----------------- | --------------------------------------------------------- |
| Scroll wheel      | Vertical scrolling with mouse wheel                       |
| Horizontal scroll | Shift+scroll or horizontal wheel for horizontal scrolling |
| Trackpad scroll   | Two-finger scroll gesture                                 |
| Smooth scrolling  | Scrolling should feel native and smooth                   |

### Hover

| Requirement    | Description                                                                     |
| -------------- | ------------------------------------------------------------------------------- |
| Hover effects  | CSS :hover states should activate on mouse over                                 |
| Tooltips       | Native browser tooltips should appear                                           |
| Cursor changes | Cursor should change based on element (pointer for links, text for input, etc.) |

## Out of Scope

- Context menus (right-click menu) — separate issue
- Drag and drop between applications
- Pinch to zoom
- Force touch / pressure sensitivity

## Research: ts2 Mouse Input

ts2 handles mouse input in-process. ts3 must forward events via XPC, but the
CEF API calls and coordinate transformations are the same.

### Key Files (ts2)

| File | Purpose |
|------|---------|
| `ts2/wezterm-gui/src/cef_browser/mod.rs` | CEF browser API wrappers |
| `ts2/wezterm-gui/src/termwindow/mouseevent.rs` | Mouse event routing |

### Event Flow

1. Window system → `mouse_event_impl()` → `mouse_event_browser()`
2. Transform coordinates: physical window → browser-relative → logical (DIP)
3. Call CEF host methods

### Coordinate Transformation

```rust
// Physical to browser-relative
let rel_x = event.coords.x - pane_x;
let rel_y = event.coords.y - pane_y;

// Physical to logical (CEF expects DIP)
let scale = dpi / 72.0;  // macOS base DPI = 72
let cef_x = (rel_x / scale) as i32;
let cef_y = (rel_y / scale) as i32;
```

### CEF APIs

| Method | Purpose |
|--------|---------|
| `host.send_mouse_move_event()` | Mouse movement, hover |
| `host.send_mouse_click_event()` | Press/release, click count |
| `host.send_mouse_wheel_event()` | Scroll (delta × 120) |

### Modifier Flags

CEF uses a bitmask for modifiers and button state:

```rust
EVENTFLAG_SHIFT_DOWN: u32 = 1 << 1;
EVENTFLAG_CONTROL_DOWN: u32 = 1 << 2;
EVENTFLAG_ALT_DOWN: u32 = 1 << 3;
EVENTFLAG_LEFT_MOUSE_BUTTON: u32 = 1 << 4;
EVENTFLAG_MIDDLE_MOUSE_BUTTON: u32 = 1 << 5;
EVENTFLAG_RIGHT_MOUSE_BUTTON: u32 = 1 << 6;
EVENTFLAG_COMMAND_DOWN: u32 = 1 << 7;
```

### Key Patterns

1. **Button state tracking** — Track which buttons are pressed across events
2. **Modifier composition** — Combine keyboard modifiers + button state
3. **Wheel delta** — Multiply by 120 (Windows scroll standard)
4. **Mode switching** — Click on browser area switches Control → Browse mode

### ts3 Adaptation

The main difference for ts3: events must be serialized and sent via XPC to the
profile server, which then calls the CEF host methods. The XPC message would
include:

- Event type (move, click, wheel)
- Coordinates (already transformed to logical pixels)
- Button type and state (for clicks)
- Modifiers bitmask
- Click count (for double/triple click)
- Wheel deltas (for scroll)

## Hypothesis: Forward Transformed Events via XPC

Do coordinate transformation in the GUI (where we have pane bounds and DPI),
then send logical coordinates via XPC. The profile server just calls CEF
methods — no layout knowledge needed.

### GUI Side (wezterm-gui)

**1. Intercept in `mouse_event_impl()` (mouseevent.rs)**

Check if mouse is over a webview pane in Browse mode. If so, intercept instead
of normal handling.

**2. Transform coordinates**

```rust
// Get pane bounds (already available from render state)
let rel_x = event.coords.x - pane_x;
let rel_y = event.coords.y - pane_y;

// Convert to logical pixels
let scale = dpi / 72.0;
let cef_x = (rel_x / scale) as i32;
let cef_y = (rel_y / scale) as i32;
```

**3. Send via XPC (webview_xpc.rs)**

One method for each event type:

```rust
pub fn send_mouse_move(&self, pane_id, x, y, modifiers);
pub fn send_mouse_click(&self, pane_id, x, y, button, is_up, click_count, modifiers);
pub fn send_mouse_wheel(&self, pane_id, x, y, delta_x, delta_y, modifiers);
```

**4. Track button state on GUI side**

Maintain `mouse_buttons: u32` flag field, update on press/release, combine with
keyboard modifiers.

### Profile Server Side (termsurf-profile)

**1. Handle XPC messages**

```rust
"mouse_move" => { post MouseMoveTask }
"mouse_click" => { post MouseClickTask }
"mouse_wheel" => { post MouseWheelTask }
```

**2. Tasks call CEF host methods**

```rust
host.send_mouse_move_event(Some(&mouse_event), mouse_leave);
host.send_mouse_click_event(Some(&mouse_event), button, mouse_up, click_count);
host.send_mouse_wheel_event(Some(&mouse_event), delta_x, delta_y);
```

### Why This Should Work

1. **Same pattern as keyboard** — We already do XPC message → post_task → CEF
2. **GUI has all layout info** — Pane bounds, DPI, mode state already available
3. **Profile server stays simple** — Just receives coordinates and calls CEF
4. **No new architecture** — Extends existing XpcManager methods

### Potential Complications

1. **Click counting** — Double/triple click detection needs timeout logic on GUI
2. **Mouse leave events** — Need to detect when mouse exits pane bounds
3. **Cursor changes** — CEF may need to send cursor type back to GUI (reverse XPC)
4. **Latency** — XPC round-trip for every mouse move could feel sluggish

### Suggested First Experiment

Start with just `send_mouse_move` and `send_mouse_click` for left button. Verify
clicking links works before adding wheel, modifiers, and click counting.

## Success Criteria

- [ ] Can click links to navigate
- [ ] Can click buttons and form elements
- [ ] Can double-click to select words
- [ ] Can click and drag to select text
- [ ] Can scroll with mouse wheel
- [ ] Can scroll with trackpad gestures
- [ ] Hover effects work (CSS :hover, tooltips)
- [ ] Cursor changes appropriately (pointer, text, etc.)

---

## Experiment 1: Mouse Move and Left Click

**Status: FAILED**

Start with the minimal implementation: mouse movement and left-button clicks. Verify
that clicking links works before adding scrolling, modifiers, or click counting.

### Goal

- Mouse hover over webview pane triggers CEF hover effects
- Left-click on links navigates to the link target

### Files to Modify

| File                                                 | Changes                                    |
| ---------------------------------------------------- | ------------------------------------------ |
| `ts3/wezterm-gui/src/termwindow/webview_xpc.rs`      | Add `send_mouse_move`, `send_mouse_click`  |
| `ts3/wezterm-gui/src/termwindow/mouseevent.rs`       | Intercept mouse events for webview panes   |
| `ts3/termsurf-profile/src/main.rs`                   | Handle XPC messages, call CEF host methods |

### Part 1: XPC Methods (webview_xpc.rs)

Add two methods to XpcManager after the existing `send_select_all` method:

```rust
/// Send mouse move event to the browser (issue 319, experiment 1)
pub fn send_mouse_move(&self, pane_id: PaneId, x: i32, y: i32, modifiers: u32) -> bool {
    let msg = XpcDictionary::new();
    msg.set_string("action", "mouse_move");
    msg.set_i64("x", x as i64);
    msg.set_i64("y", y as i64);
    msg.set_i64("modifiers", modifiers as i64);

    if self.send_command(pane_id, &msg) {
        log::trace!("[XPC] Sent mouse_move to pane {}: ({}, {})", pane_id, x, y);
        true
    } else {
        false
    }
}

/// Send mouse click event to the browser (issue 319, experiment 1)
pub fn send_mouse_click(
    &self,
    pane_id: PaneId,
    x: i32,
    y: i32,
    button: u32,
    is_up: bool,
    click_count: i32,
    modifiers: u32,
) -> bool {
    let msg = XpcDictionary::new();
    msg.set_string("action", "mouse_click");
    msg.set_i64("x", x as i64);
    msg.set_i64("y", y as i64);
    msg.set_i64("button", button as i64);
    msg.set_bool("is_up", is_up);
    msg.set_i64("click_count", click_count as i64);
    msg.set_i64("modifiers", modifiers as i64);

    if self.send_command(pane_id, &msg) {
        log::debug!(
            "[XPC] Sent mouse_click to pane {}: ({}, {}) btn={} up={} count={}",
            pane_id, x, y, button, is_up, click_count
        );
        true
    } else {
        false
    }
}
```

### Part 2: Intercept Mouse Events (mouseevent.rs)

Add a new method to TermWindow and call it early in `mouse_event_impl`:

**2a. Add helper method to check webview pane bounds**

```rust
/// Check if mouse event is over a webview pane in Browse mode.
/// Returns Some((pane_id, rel_x, rel_y, scale)) if so, None otherwise.
#[cfg(target_os = "macos")]
fn mouse_over_webview(&self, event: &MouseEvent) -> Option<(mux::pane::PaneId, f32, f32, f32)> {
    use crate::termwindow::webview_socket::{get_server, WebviewMode};

    let server = get_server()?;
    let state = server.state();
    let overlays = state.read().unwrap();

    // Check each pane to find if mouse is over a webview
    for pos in self.get_panes_to_render() {
        let pane_id = pos.pane.pane_id();

        // Only consider panes with webview overlays in Browse mode
        let overlay = overlays.overlays.get(&pane_id)?;
        if overlay.mode != WebviewMode::Browse {
            continue;
        }

        // Calculate viewport bounds (same logic as render_webview_overlays_webgpu)
        let border = self.get_os_border();
        let tab_bar_height = if self.show_tab_bar && !self.config.tab_bar_at_bottom {
            self.tab_bar_pixel_height().unwrap_or(0.)
        } else {
            0.
        };

        let pane_x = pos.left as f32 * self.render_metrics.cell_size.width as f32
            + border.left.get() as f32;
        let pane_y = pos.top as f32 * self.render_metrics.cell_size.height as f32
            + border.top.get() as f32
            + tab_bar_height;
        let pane_w = pos.width as f32 * self.render_metrics.cell_size.width as f32;
        let pane_h = pos.height as f32 * self.render_metrics.cell_size.height as f32;

        // Check if mouse is within pane bounds
        let mx = event.coords.x as f32;
        let my = event.coords.y as f32;

        if mx >= pane_x && mx < pane_x + pane_w && my >= pane_y && my < pane_y + pane_h {
            // Calculate relative position within pane
            let rel_x = mx - pane_x;
            let rel_y = my - pane_y;

            // Get scale factor
            let scale = self.dimensions.dpi as f32 / 72.0;
            let scale = if scale <= 0.0 { 2.0 } else { scale };

            return Some((pane_id, rel_x, rel_y, scale));
        }
    }

    None
}
```

**2b. Add method to handle webview mouse events**

```rust
/// Handle mouse events for webview panes in Browse mode.
/// Returns true if the event was consumed.
#[cfg(target_os = "macos")]
fn handle_webview_mouse_event(&mut self, event: &MouseEvent) -> bool {
    use ::window::MouseEventKind as WMEK;
    use ::window::MousePress;

    let (pane_id, rel_x, rel_y, scale) = match self.mouse_over_webview(event) {
        Some(info) => info,
        None => return false,
    };

    // Convert to logical (CEF DIP) coordinates
    let cef_x = (rel_x / scale) as i32;
    let cef_y = (rel_y / scale) as i32;

    let xpc_manager = match crate::termwindow::webview_xpc::get_xpc_manager() {
        Some(m) => m,
        None => return false,
    };

    match &event.kind {
        WMEK::Move => {
            xpc_manager.send_mouse_move(pane_id, cef_x, cef_y, 0);
            true
        }
        WMEK::Press(MousePress::Left) => {
            xpc_manager.send_mouse_click(pane_id, cef_x, cef_y, 0, false, 1, 0);
            true
        }
        WMEK::Release(MousePress::Left) => {
            xpc_manager.send_mouse_click(pane_id, cef_x, cef_y, 0, true, 1, 0);
            true
        }
        _ => false, // Let other events pass through for now
    }
}
```

**2c. Add early intercept in mouse_event_impl**

At the start of `mouse_event_impl`, after getting the pane:

```rust
pub fn mouse_event_impl(&mut self, event: MouseEvent, context: &dyn WindowOps) {
    log::trace!("{:?}", event);
    let pane = match self.get_active_pane_or_overlay() {
        Some(pane) => pane,
        None => return,
    };

    // Check for webview mouse event (issue 319)
    #[cfg(target_os = "macos")]
    if self.handle_webview_mouse_event(&event) {
        return; // Event consumed by webview
    }

    self.current_mouse_event.replace(event.clone());
    // ... rest of existing code
```

### Part 3: CEF Mouse Event Handling (main.rs)

**3a. Add XPC message handlers in the event handler**

In `create_browser_on_ui_thread`, add cases for mouse events:

```rust
"mouse_move" => {
    let state_guard = deferred_for_handler.lock().unwrap();
    let Some(bs) = state_guard.as_ref() else {
        return;
    };

    let x = msg.get_i64("x") as i32;
    let y = msg.get_i64("y") as i32;
    let modifiers = msg.get_i64("modifiers") as u32;

    let bs = Arc::clone(bs);
    drop(state_guard);

    let mut task = MouseMoveTask::new(bs, x, y, modifiers);
    cef::post_task(cef::ThreadId::UI, Some(&mut task));
}
"mouse_click" => {
    let state_guard = deferred_for_handler.lock().unwrap();
    let Some(bs) = state_guard.as_ref() else {
        return;
    };

    let x = msg.get_i64("x") as i32;
    let y = msg.get_i64("y") as i32;
    let button = msg.get_i64("button") as u32;
    let is_up = msg.get_bool("is_up");
    let click_count = msg.get_i64("click_count") as i32;
    let modifiers = msg.get_i64("modifiers") as u32;

    let bs = Arc::clone(bs);
    drop(state_guard);

    let mut task = MouseClickTask::new(bs, x, y, button, is_up, click_count, modifiers);
    cef::post_task(cef::ThreadId::UI, Some(&mut task));
}
```

**3b. Add MouseMoveTask**

```rust
// ====== Mouse Move Task ======
//
// Task for sending mouse move events to CEF on the UI thread.
// Issue 319, experiment 1.

wrap_task! {
    pub struct MouseMoveTask {
        state: Arc<BrowserState>,
        x: i32,
        y: i32,
        modifiers: u32,
    }

    impl Task {
        fn execute(&self) {
            if let Some(browser) = self.state.browser.lock().unwrap().as_ref() {
                if let Some(host) = browser.host() {
                    let mouse_event = cef::MouseEvent {
                        x: self.x,
                        y: self.y,
                        modifiers: self.modifiers,
                    };
                    // mouse_leave = false (mouse is over the view)
                    host.send_mouse_move_event(Some(&mouse_event), 0);
                }
            }
        }
    }
}
```

**3c. Add MouseClickTask**

```rust
// ====== Mouse Click Task ======
//
// Task for sending mouse click events to CEF on the UI thread.
// Issue 319, experiment 1.

wrap_task! {
    pub struct MouseClickTask {
        state: Arc<BrowserState>,
        x: i32,
        y: i32,
        button: u32,
        is_up: bool,
        click_count: i32,
        modifiers: u32,
    }

    impl Task {
        fn execute(&self) {
            if let Some(browser) = self.state.browser.lock().unwrap().as_ref() {
                if let Some(host) = browser.host() {
                    let mouse_event = cef::MouseEvent {
                        x: self.x,
                        y: self.y,
                        modifiers: self.modifiers,
                    };
                    // button: 0=left, 1=middle, 2=right (CEF MouseButtonType)
                    let button_type = match self.button {
                        0 => cef::MouseButtonType::MBT_LEFT,
                        1 => cef::MouseButtonType::MBT_MIDDLE,
                        2 => cef::MouseButtonType::MBT_RIGHT,
                        _ => cef::MouseButtonType::MBT_LEFT,
                    };
                    let mouse_up = if self.is_up { 1 } else { 0 };
                    host.send_mouse_click_event(
                        Some(&mouse_event),
                        button_type,
                        mouse_up,
                        self.click_count,
                    );
                }
            }
        }
    }
}
```

### Verification

```bash
cd ts3 && ./scripts/build-debug.sh --open

# Test 1: Hover effects
web google.com
# Move mouse over search button
# Expected: Cursor changes, hover effects visible

# Test 2: Click links
web example.com
# Click the "More information..." link
# Expected: Navigates to IANA page

# Test 3: Click form elements
web google.com
# Click in search box
# Expected: Text cursor appears, can type

# Log verification
tail -f /tmp/termsurf-gui.log | grep -E "\[XPC\] Sent mouse"
tail -f /tmp/termsurf-profile-*.log | grep -i mouse
```

### Success Criteria for Experiment 1

- [ ] Mouse movement sends events to CEF (visible in logs)
- [ ] Hover over links shows pointer cursor
- [ ] Click on links navigates to URL
- [ ] Click in text fields focuses them
- [ ] Click on buttons activates them

### Known Limitations (Experiment 1)

These will be addressed in later experiments:

- No scroll wheel support
- No modifiers (Shift-click, Cmd-click)
- No click counting (double/triple click)
- No drag support (text selection)
- No right-click support
- No middle-click support

### Conclusion (Experiment 1)

**Result: Failed.** Mouse events are not being delivered to CEF.

#### What's Broken

The GUI successfully intercepts mouse events, transforms coordinates, and calls
`send_command()` via XPC. The connection appears valid (no "No connection for pane"
warnings). However, the profile server never receives the mouse events — zero
`mouse_move` or `mouse_click` actions appear in the profile logs.

Observed behavior: hover highlights appear briefly then disappear, clicks work once
then stop. This suggests messages may be delivered initially but the connection
enters a broken state where `send()` silently fails.

Confusingly, keyboard input uses the identical `send_command()` path and works
reliably. The XPC connection works in both directions for other message types:

| Direction         | Message Type      | Status  |
| ----------------- | ----------------- | ------- |
| Profile → GUI     | `display_surface` | Works   |
| Launcher → Profile| `create_browser`  | Works   |
| GUI → Profile     | `key_event`       | Works   |
| GUI → Profile     | `mouse_move`      | Broken  |
| GUI → Profile     | `mouse_click`     | Broken  |

The profile logs show repeated "XPC connection interrupted" errors and unexpected
`create_browser` commands, suggesting connection instability that may be related.

#### Ideas for Fixing

1. **Debug XPC connection state**: Add logging to verify the connection stored in
   `peer_connections` is the same object the profile has its event handler on.
   Multiple reconnections may cause GUI to send on a connection the profile isn't
   listening to.

2. **Verify event handler registration**: Confirm the profile's event handler for
   `mouse_move`/`mouse_click` is actually registered. Add a catch-all log in the
   handler's `_ => {}` branch to see if messages arrive with unexpected action names.

3. **Test with synchronous reply**: Use `send_with_reply_sync()` instead of `send()`
   for mouse events temporarily. If this works, the issue is with async message
   delivery. If it fails, we'll get an actual error message.

4. **Compare with keyboard path**: Trace exactly what happens for a keyboard event
   vs a mouse event. Find where the paths diverge.

5. **Check for connection replacement**: The logs show many "New connection for
   session" messages after errors. If the GUI stores a new connection but the
   profile's event handler is on the old one, messages would be lost. May need to
   re-register handlers on reconnection.

6. **Simplify**: Strip mouse handling down to the absolute minimum — send a single
   test message on click and verify it arrives. Remove all the coordinate
   transformation and throttling to isolate the core IPC issue.

---

## Experiment 2: Diagnostic Logging

**Status: FAILED**

Add comprehensive logging to trace exactly where mouse events are lost in the
XPC pipeline. The goal is to determine whether messages are:
1. Not being sent by GUI
2. Sent but not arriving at profile
3. Arriving but not being handled

### Goal

Understand why keyboard events work via `send_command()` but mouse events don't.
Produce log output that pinpoints the failure location.

### Files to Modify

| File | Changes |
|------|---------|
| `ts3/wezterm-gui/src/termwindow/webview_xpc.rs` | Log connection state on send |
| `ts3/termsurf-profile/src/main.rs` | Log all incoming XPC messages before parsing |

### Part 1: GUI-Side Logging (webview_xpc.rs)

**1a. Add connection state logging in `send_command`**

Find the `send_command` method and add logging before the send:

```rust
fn send_command(&self, pane_id: PaneId, msg: &XpcDictionary) -> bool {
    let connections = self.peer_connections.lock().unwrap();
    let Some(conn) = connections.get(&pane_id) else {
        log::warn!("[XPC] No connection for pane {}", pane_id);
        return false;
    };

    // NEW: Log connection pointer and message action for debugging
    let action = msg.get_string("action").unwrap_or("unknown");
    log::info!(
        "[XPC-SEND] pane={} action={} conn={:p}",
        pane_id, action, conn.as_ptr()
    );

    conn.send(msg);
    true
}
```

**1b. Log connection storage in `handle_new_connection`**

When storing a new connection, log its pointer:

```rust
// In handle_new_connection, after inserting into peer_connections:
log::info!(
    "[XPC-CONN] Stored connection for pane {}: {:p}",
    pane_id, conn.as_ptr()
);
```

**1c. Log when connection is replaced**

If a connection already exists for a pane, log that it's being replaced:

```rust
// In handle_new_connection, before inserting:
if let Some(old_conn) = peer_connections.get(&pane_id) {
    log::warn!(
        "[XPC-CONN] Replacing connection for pane {}: old={:p} new={:p}",
        pane_id, old_conn.as_ptr(), conn.as_ptr()
    );
}
peer_connections.insert(pane_id, conn);
```

### Part 2: Profile-Side Logging (main.rs)

**2a. Log ALL incoming messages at handler entry**

At the very first line of the XPC event handler, before any action matching:

```rust
// In the XPC event_handler closure, first thing:
let action = msg.get_string("action").unwrap_or("none");
log::info!("[XPC-RECV] Received message: action={}", action);

// Then the existing match on action...
match action.as_deref() {
    // ...
}
```

**2b. Add catch-all logging for unhandled actions**

In the action match, add a default case:

```rust
match action.as_deref() {
    Some("create_browser") => { /* existing */ }
    Some("key_event") => { /* existing */ }
    Some("mouse_move") => { /* existing */ }
    Some("mouse_click") => { /* existing */ }
    // ... other cases ...
    other => {
        log::warn!("[XPC-RECV] Unhandled action: {:?}", other);
    }
}
```

**2c. Log connection events**

Add logging for connection lifecycle:

```rust
// In XPC listener setup, after creating the connection handler:
log::info!("[XPC] Event handler registered on connection");

// If there are connection error callbacks:
// log::error!("[XPC] Connection error: ...");
```

### Part 3: Test Procedure

```bash
cd ts3 && ./scripts/build-debug.sh --open

# Terminal 1: Watch GUI logs
tail -f /tmp/termsurf-gui.log | grep -E "\[XPC-(SEND|CONN)\]"

# Terminal 2: Watch profile logs
tail -f /tmp/termsurf-profile-*.log | grep -E "\[XPC-RECV\]"

# Terminal 3: Run TermSurf
# 1. Start TermSurf
# 2. Type: web google.com
# 3. Wait for page to load
# 4. Move mouse over the webview
# 5. Click once on a link

# After test, examine both log outputs
```

### Expected Log Patterns

**If messages are being sent but not received:**
```
# GUI log shows:
[XPC-SEND] pane=0 action=mouse_move conn=0x12345678
[XPC-SEND] pane=0 action=mouse_move conn=0x12345678

# Profile log shows nothing, or only:
[XPC-RECV] Received message: action=create_browser
# No mouse_move entries
```

**If connection is being replaced:**
```
# GUI log shows:
[XPC-CONN] Stored connection for pane 0: 0x12345678
[XPC-CONN] Replacing connection for pane 0: old=0x12345678 new=0xABCDEF00
[XPC-SEND] pane=0 action=mouse_move conn=0xABCDEF00

# Profile's handler may still be on 0x12345678 (the old connection)
```

**If messages arrive but aren't handled:**
```
# Profile log shows:
[XPC-RECV] Received message: action=mouse_move
# But no further processing logs from MouseMoveTask
```

**If everything works (baseline with keyboard):**
```
# GUI log:
[XPC-SEND] pane=0 action=key_event conn=0x12345678

# Profile log:
[XPC-RECV] Received message: action=key_event
```

### Analysis Guide

| GUI Log | Profile Log | Diagnosis |
|---------|-------------|-----------|
| SEND appears | RECV appears | Handler bug (action parsing) |
| SEND appears | No RECV | XPC transport issue |
| CONN replaced | No RECV | Handler on wrong connection |
| No SEND | — | GUI interception bug |

### Success Criteria

- [ ] Can trace the full path of a keyboard event (control)
- [ ] Can see where mouse events diverge from keyboard
- [ ] Logs reveal whether messages arrive at profile
- [ ] Connection pointer logging reveals any mismatch

### Next Steps After Diagnosis

Based on what the logs reveal:

1. **If messages don't arrive**: Focus on XPC connection management. May need
   to re-register handlers on connection replacement.

2. **If messages arrive but aren't handled**: Check action string matching,
   possibly encoding issue or typo.

3. **If connection is replaced**: Need to either prevent replacement or
   re-register the event handler on the new connection.

### Conclusion (Experiment 2)

**Result: Failed.** The diagnostic logging was insufficient to identify the root cause.

#### What We Learned

1. **XPC transport works**: Messages ARE being delivered. GUI logs show `[XPC-SEND]`
   and profile logs show `[XPC-RECV]` for all mouse_move and mouse_click events.

2. **Connection is stable during sends**: The same connection pointer (`0xb02e80df0`)
   is used consistently. No `[XPC-CONN] Replacing connection` warnings appeared.

3. **Connection errors occur later**: Many "XPC connection interrupted" errors appear
   in the profile logs AFTER mouse events are received.

#### What We Didn't Learn

1. **Why handlers don't execute**: Messages arrive at `[XPC-RECV]` but we have no
   visibility into whether the action matching succeeds or whether `deferred_for_handler`
   contains a valid BrowserState.

2. **CEF task execution**: No logging confirms whether `post_task` is called or
   whether `MouseMoveTask.execute()` runs.

3. **CEF API response**: No logging shows if `send_mouse_move_event` is called or
   if CEF acknowledges the events.

#### Why This Experiment Failed

The logging was too shallow. We only logged at the entry point (`[XPC-RECV]`) but
not at the critical decision points inside the handlers:
- Is `deferred_for_handler.as_ref()` returning None?
- Is `post_task` being called?
- Is the task executing?
- Is CEF receiving the events?

A deeper experiment would need logging at each of these stages to pinpoint where
the chain breaks.

---

## Experiment 3: Deep Handler Logging

**Status: FAILED**

Add logging at every decision point in the mouse event handler chain. Experiment 2
showed messages arrive at `[XPC-RECV]` but gave no visibility into what happens next.
This experiment adds logging inside the handlers to trace the complete execution path.

### Goal

Determine exactly where mouse event handling fails:
- Does `deferred_for_handler` contain a valid BrowserState?
- Does `post_task` get called?
- Does the task's `execute()` method run?
- Does CEF's `send_mouse_*_event()` get called?

### Files to Modify

| File | Changes |
|------|---------|
| `ts3/termsurf-profile/src/main.rs` | Add logging inside action handlers and tasks |

### Part 1: Handler Entry Logging

In the XPC event handler, add logging inside each mouse action case BEFORE checking
`deferred_for_handler`:

```rust
"mouse_move" => {
    println!("[MOUSE] mouse_move handler entered");

    let state_guard = deferred_for_handler.lock().unwrap();
    let Some(bs) = state_guard.as_ref() else {
        println!("[MOUSE] FAIL: deferred_for_handler is None");
        return;
    };
    println!("[MOUSE] BrowserState available, posting task");

    let x = msg.get_i64("x") as i32;
    let y = msg.get_i64("y") as i32;
    let modifiers = msg.get_i64("modifiers") as u32;
    println!("[MOUSE] mouse_move coords: ({}, {}) mods={}", x, y, modifiers);

    let bs = Arc::clone(bs);
    drop(state_guard);

    let mut task = MouseMoveTask::new(bs, x, y, modifiers);
    println!("[MOUSE] Calling post_task for MouseMoveTask");
    cef::post_task(cef::ThreadId::UI, Some(&mut task));
    println!("[MOUSE] post_task returned");
}
"mouse_click" => {
    println!("[MOUSE] mouse_click handler entered");

    let state_guard = deferred_for_handler.lock().unwrap();
    let Some(bs) = state_guard.as_ref() else {
        println!("[MOUSE] FAIL: deferred_for_handler is None");
        return;
    };
    println!("[MOUSE] BrowserState available, posting task");

    let x = msg.get_i64("x") as i32;
    let y = msg.get_i64("y") as i32;
    let button = msg.get_i64("button") as u32;
    let is_up = msg.get_bool("is_up");
    let click_count = msg.get_i64("click_count") as i32;
    let modifiers = msg.get_i64("modifiers") as u32;
    println!(
        "[MOUSE] mouse_click coords: ({}, {}) btn={} up={} count={}",
        x, y, button, is_up, click_count
    );

    let bs = Arc::clone(bs);
    drop(state_guard);

    let mut task = MouseClickTask::new(bs, x, y, button, is_up, click_count, modifiers);
    println!("[MOUSE] Calling post_task for MouseClickTask");
    cef::post_task(cef::ThreadId::UI, Some(&mut task));
    println!("[MOUSE] post_task returned");
}
```

### Part 2: Task Execution Logging

Add logging inside the task `execute()` methods:

**MouseMoveTask:**

```rust
fn execute(&self) {
    println!("[MOUSE-TASK] MouseMoveTask::execute() called");

    let browser_guard = self.state.browser.lock().unwrap();
    let Some(browser) = browser_guard.as_ref() else {
        println!("[MOUSE-TASK] FAIL: browser is None");
        return;
    };
    println!("[MOUSE-TASK] Browser obtained");

    let Some(host) = browser.host() else {
        println!("[MOUSE-TASK] FAIL: browser.host() is None");
        return;
    };
    println!("[MOUSE-TASK] Host obtained, calling send_mouse_move_event");

    let mouse_event = cef::MouseEvent {
        x: self.x,
        y: self.y,
        modifiers: self.modifiers,
    };
    host.send_mouse_move_event(Some(&mouse_event), 0);
    println!("[MOUSE-TASK] send_mouse_move_event returned");
}
```

**MouseClickTask:**

```rust
fn execute(&self) {
    println!("[MOUSE-TASK] MouseClickTask::execute() called");

    let browser_guard = self.state.browser.lock().unwrap();
    let Some(browser) = browser_guard.as_ref() else {
        println!("[MOUSE-TASK] FAIL: browser is None");
        return;
    };
    println!("[MOUSE-TASK] Browser obtained");

    let Some(host) = browser.host() else {
        println!("[MOUSE-TASK] FAIL: browser.host() is None");
        return;
    };
    println!("[MOUSE-TASK] Host obtained, calling send_mouse_click_event");

    let mouse_event = cef::MouseEvent {
        x: self.x,
        y: self.y,
        modifiers: self.modifiers,
    };
    let button_type = match self.button {
        0 => cef::MouseButtonType::MBT_LEFT,
        1 => cef::MouseButtonType::MBT_MIDDLE,
        2 => cef::MouseButtonType::MBT_RIGHT,
        _ => cef::MouseButtonType::MBT_LEFT,
    };
    let mouse_up = if self.is_up { 1 } else { 0 };
    host.send_mouse_click_event(
        Some(&mouse_event),
        button_type,
        mouse_up,
        self.click_count,
    );
    println!("[MOUSE-TASK] send_mouse_click_event returned");
}
```

### Part 3: Control Comparison

Add identical logging to the keyboard handler to establish a working baseline:

```rust
"key_event" => {
    println!("[KEY] key_event handler entered");

    let state_guard = deferred_for_handler.lock().unwrap();
    let Some(bs) = state_guard.as_ref() else {
        println!("[KEY] FAIL: deferred_for_handler is None");
        return;
    };
    println!("[KEY] BrowserState available, posting task");

    // ... existing key event handling ...

    println!("[KEY] post_task returned");
}
```

### Test Procedure

```bash
cd ts3 && ./scripts/build-debug.sh --open

# Terminal 1: Watch profile logs for mouse events
tail -f /tmp/termsurf-profile-*.log | grep -E "\[(MOUSE|KEY)"

# Terminal 2: Run TermSurf
# 1. Type: web google.com
# 2. Wait for page to load
# 3. Type a few characters (keyboard control test)
# 4. Move mouse over the webview
# 5. Click once

# After test, analyze the full log sequence
```

### Expected Log Patterns

**Pattern A: Handler never entered (action matching failed)**
```
[XPC-RECV] Received message: action=mouse_move
# No [MOUSE] handler entered log follows
```

**Pattern B: BrowserState not available**
```
[XPC-RECV] Received message: action=mouse_move
[MOUSE] mouse_move handler entered
[MOUSE] FAIL: deferred_for_handler is None
```

**Pattern C: Task not executed (post_task failed or task dropped)**
```
[XPC-RECV] Received message: action=mouse_move
[MOUSE] mouse_move handler entered
[MOUSE] BrowserState available, posting task
[MOUSE] Calling post_task for MouseMoveTask
[MOUSE] post_task returned
# No [MOUSE-TASK] MouseMoveTask::execute() called
```

**Pattern D: Browser/Host unavailable**
```
[MOUSE-TASK] MouseMoveTask::execute() called
[MOUSE-TASK] FAIL: browser is None
```

**Pattern E: Everything works (expected for keyboard)**
```
[KEY] key_event handler entered
[KEY] BrowserState available, posting task
[KEY] post_task returned
[KEY-TASK] KeyTask::execute() called
[KEY-TASK] send_key_event returned
```

### Analysis Guide

| Log Sequence | Diagnosis | Fix Direction |
|--------------|-----------|---------------|
| No handler log | Action string mismatch | Check action parsing |
| deferred_for_handler is None | BrowserState not initialized | Check initialization timing |
| No task execute log | post_task failing | Check CEF thread state |
| browser is None | Browser not created yet | Check creation timing |
| host is None | Browser destroyed | Check lifecycle management |
| All logs present but CEF silent | CEF API issue | Check CEF event format |

### Success Criteria

- [ ] Can trace keyboard events through entire chain (control)
- [ ] Can identify exact failure point for mouse events
- [ ] Logs clearly show which pattern matches our failure

### Next Steps After Diagnosis

Based on which pattern emerges:

1. **Pattern A**: Debug action string matching — possibly encoding issue or whitespace
2. **Pattern B**: Debug BrowserState initialization timing — deferred_for_handler may be set too late
3. **Pattern C**: Debug CEF task posting — UI thread may not be running or task may be dropped
4. **Pattern D/E (browser/host None)**: Debug browser lifecycle — timing issue between creation and first mouse event

### Conclusion (Experiment 3)

**Result: Failed.** The logging showed the entire pipeline working, but mouse input remains
broken with highly inconsistent, non-deterministic behavior.

#### What We Learned

1. **Pipeline is complete**: Every stage executes successfully:
   - Handler enters
   - BrowserState is available
   - post_task is called and returns
   - Task execute() runs
   - Browser and host are obtained
   - CEF APIs (send_mouse_move_event, send_mouse_click_event) are called and return

2. **Coordinates are correct**: Mouse positions are within view_rect bounds after
   proper physical-to-logical conversion.

3. **Behavior is inconsistent**: User observed:
   - Cursor does NOT change to pointer on links
   - Links highlight only briefly and incorrectly
   - Visual feedback is highly inconsistent, "more broken than not"

4. **XPC connection errors**: Logs show "XPC connection interrupted" errors occurring
   during mouse event processing, but the profile continues running.

#### Why This Experiment Failed

The deep logging proved the pipeline executes correctly, but the mouse input is still
broken. This means the issue is NOT in the execution path we instrumented. Possible
causes that remain uninvestigated:

1. **CEF internal state**: CEF receives events but doesn't process them correctly
   (focus, hit testing, or rendering state issues)
2. **Timing/threading**: Events arrive but CEF's internal state machine rejects them
3. **Coordinate system mismatch**: CEF might expect different coordinate semantics
4. **Connection lifecycle**: XPC errors may indicate deeper connection instability

The experiment successfully ruled out pipeline failures but did not identify the
actual root cause of the inconsistent mouse behavior.

## References

- `docs/issues/317-input.md` — Keyboard input (completed)
- `docs/issues/318-cmd.md` — Clipboard keybindings (completed)
