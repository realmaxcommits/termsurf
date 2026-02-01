# 328: Blinking Text Cursor (Caret)

The blinking text cursor (caret) does not appear in webview text inputs.

## Status

**Open.** Hypothesis identified from ts2 implementation.

## Problem

When a text input has focus in the webview (e.g., Google's search box
auto-focuses on load), the blinking text cursor does not appear. Users can type
and see their text, but there's no visual caret indicating the insertion point.

**Symptoms:**

- Google.com auto-focuses the search box
- User can type and text appears correctly
- No blinking cursor is ever visible
- This affects all text inputs, not just Google

**Impact:** Users have no visual feedback about cursor position, making text
editing difficult.

## Background

### ts3 Current Behavior

In `ts3/termsurf-profile/src/main.rs`, focus is set immediately after browser
creation (line 1061-1063):

```rust
// Ensure browser has focus for clipboard operations (experiment 4)
if let Some(host) = b.host() {
    println!("[FOCUS-DEBUG] Sending initial focus event to browser {}", browser_id);
    host.set_focus(1); // 1 = focused
}
```

This is called inside `create_browser_on_ui_thread`, right after
`create_browser()` returns. The browser may not be fully initialized at this
point.

### ts2 Working Behavior

ts2 handles focus differently in `ts2/wezterm-gui/src/cef_browser/mod.rs` (lines
616-625):

```rust
// Set initial focus on first paint (browser is now ready)
// We unfocus then refocus to properly initialize the focus state,
if !*self.handler.initial_focus_set.borrow() {
    if let Some(browser) = &self.handler.browser {
        if let Some(host) = browser.host() {
            log::info!("[CEF] Setting initial focus on first paint (unfocus then refocus)");
            host.set_focus(0);
            host.set_focus(1);
            *self.handler.initial_focus_set.borrow_mut() = true;
        }
    }
}
```

Key differences:

| Aspect         | ts3                                  | ts2                                        |
| -------------- | ------------------------------------ | ------------------------------------------ |
| When           | Immediately after `create_browser()` | On first `on_paint` callback               |
| How            | Single `set_focus(1)`                | Toggle: `set_focus(0)` then `set_focus(1)` |
| State tracking | None                                 | `initial_focus_set` flag                   |

### Why Timing Matters

CEF's browser initialization is asynchronous. When `create_browser()` returns,
the browser object exists but may not be fully initialized internally. The first
`on_paint` or `on_accelerated_paint` callback indicates the browser has
completed initialization and is ready to render.

Setting focus before the browser is ready may result in CEF's internal focus
state not being properly initialized, causing the caret to never appear.

### Why Toggle Matters

The ts2 comment says "unfocus then refocus to properly initialize the focus
state." This suggests CEF may have an edge case where calling `set_focus(1)` on
a browser that was never unfocused doesn't fully activate focus features like
the caret. The toggle forces CEF through both code paths.

## Proposed Solution

Modify `ts3/termsurf-profile/src/main.rs` to:

1. Add an `initial_focus_set` flag to `BrowserState`
2. Remove the `set_focus(1)` call from `create_browser_on_ui_thread`
3. In `on_accelerated_paint`, on the first paint, do the unfocus/refocus toggle

### Changes

**Add flag to BrowserState:**

```rust
pub struct BrowserState {
    // ... existing fields ...
    /// Whether initial focus has been set (must wait for first paint)
    pub initial_focus_set: AtomicBool,
}
```

**Initialize in create_browser_on_ui_thread:**

```rust
let browser_state = Arc::new(BrowserState {
    // ... existing fields ...
    initial_focus_set: AtomicBool::new(false),
});

// Remove the set_focus(1) call that's currently here
```

**Add to on_accelerated_paint in ProfileRenderHandler:**

```rust
fn on_accelerated_paint(
    &self,
    _browser: Option<&mut Browser>,
    type_: PaintElementType,
    _dirty_rects: Option<&[Rect]>,
    info: Option<&AcceleratedPaintInfo>,
) {
    // ... existing code ...

    // Set initial focus on first paint (browser is now ready)
    // Toggle unfocus/refocus to properly initialize focus state (from ts2)
    if !self.inner.state.initial_focus_set.load(Ordering::Relaxed) {
        if let Some(browser) = self.inner.state.browser.lock().unwrap().as_ref() {
            if let Some(host) = browser.host() {
                println!("[FOCUS] Setting initial focus on first paint (unfocus then refocus)");
                host.set_focus(0);
                host.set_focus(1);
                self.inner.state.initial_focus_set.store(true, Ordering::Relaxed);
            }
        }
    }

    // ... rest of existing code ...
}
```

## Verification

```bash
cd ts3 && ./scripts/build-debug.sh --open

# Test 1: Google search box
web google.com
# Expected: Blinking caret appears in auto-focused search box

# Test 2: Type and observe caret
# Type "hello"
# Expected: Caret visible after each character

# Test 3: Click in different text field
# Navigate to a page with multiple inputs
# Click in a text field
# Expected: Caret appears at click position

# Test 4: Check logs
cat /tmp/termsurf-profile-*.log | grep "FOCUS"
# Expected: "Setting initial focus on first paint" message
```

## Success Criteria

- [ ] Caret appears in auto-focused text inputs (Google search)
- [ ] Caret appears when clicking in text fields
- [ ] Caret blinks at normal rate (~500ms)
- [ ] Caret position updates correctly when typing
- [ ] No regression in keyboard input functionality

## References

- `ts2/wezterm-gui/src/cef_browser/mod.rs` — Working focus implementation (lines
  616-625)
- `ts3/termsurf-profile/src/main.rs` — Current broken implementation (lines
  1061-1063)
- Issue 317 — Keyboard input forwarding (works, but caret missing)
