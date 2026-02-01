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

## References

- `docs/issues/317-input.md` — Keyboard input (completed)
- `docs/issues/318-cmd.md` — Clipboard keybindings (completed)
