# Issue 694: Replace pane_id with tab_id in Chromium

Remove all `pane_id` usage from the Chromium profile server. Chromium should
only know about tabs (identified by `tab_id`). The GUI manages the pane ↔ tab
relationship.

## Why

Currently there's a 1:1 relationship between panes and tabs — every pane has
exactly one Chromium tab, and Chromium identifies tabs by `pane_id`. This blocks
multiple tabs per pane.

Multiple tabs per pane enables:

- Split views (two webpages side by side in one pane)
- Webview scrolling with a shell session (webpage renders inline, scrolls up as
  shell output continues)
- Tab stacking (multiple tabs behind one pane, switched with keybindings)
- Picture-in-picture (small overlay webview inside a pane)

None of these are possible while Chromium uses `pane_id` as its primary
identifier, because Chromium can only have one tab per pane_id.

The fix is clean: Chromium already generates `tab_id` (auto-incrementing
integer). Use it everywhere. The GUI already stores `tab_id` in the Pane struct
(received from `tab_ready`). The GUI translates between pane_id (its domain) and
tab_id (Chromium's domain).

## How It Works Now

### The three layers

```
TUI (Rust)  ←→  GUI (Zig)  ←→  Chromium (C++)
  pane_id         pane_id         pane_id ← WRONG
                  tab_id          tab_id
```

### Chromium's pane_id usage (4 files, 78 occurrences)

**Storage:**

- `TabState::pane_id` (string) in `shell_browser_main_parts.h`
- `ShellTabObserver::pane_id_` (string) in `shell_tab_observer.h`

**Inbound messages (GUI → Chromium, on control connection):**

All use `pane_id` to identify the target tab:

| Message               | Uses pane_id | Uses tab_id     |
| --------------------- | ------------ | --------------- |
| `create_tab`          | YES          | NO              |
| `create_devtools_tab` | YES          | YES (inspected) |
| `resize`              | YES          | NO              |
| `mouse_event`         | YES          | NO              |
| `scroll_event`        | YES          | NO              |
| `mouse_move`          | YES          | NO              |
| `focus_changed`       | YES          | NO              |
| `key_event`           | YES          | NO              |
| `navigate`            | YES          | NO              |
| `set_color_scheme`    | YES          | NO              |
| `close_tab`           | YES          | NO              |
| `query_tabs`          | NO           | NO              |

**Outbound messages (Chromium → GUI, on per-tab connection):**

All echo `pane_id` back:

| Message          | Sends pane_id | Sends tab_id |
| ---------------- | ------------- | ------------ |
| `tab_ready`      | YES           | YES          |
| `ca_context`     | YES           | NO           |
| `cursor_changed` | YES           | NO           |
| `url_changed`    | YES           | NO           |
| `loading_state`  | YES           | NO           |
| `title_changed`  | YES           | NO           |

**Tab lookup pattern (repeated ~10 times):**

```cpp
TabState* tab = nullptr;
for (auto& t : tabs_) {
    if (t->pane_id == pane_id) {
        tab = t.get();
        break;
    }
}
```

### GUI's mapping (xpc.zig)

The GUI already maintains the pane_id ↔ tab_id relationship:

```zig
const Pane = struct {
    pane_id_key: []const u8,        // UUID string
    tab_id: i64,                     // From Chromium's tab_ready
    overlay_surface: ?*CoreSurface,
    server: ?*Server,                // Which Chromium process
    // ...
};

var panes: StringHashMap(*Pane);     // pane_id → Pane
```

When the GUI receives `tab_ready { pane_id, tab_id }`, it stores `tab_id` in the
Pane. All subsequent forwarding could use `tab_id` instead of `pane_id` — it
just doesn't yet.

## Target Architecture

```
TUI (Rust)  ←→  GUI (Zig)  ←→  Chromium (C++)
  pane_id         pane_id         tab_id ← CORRECT
                  tab_id          tab_id
```

The boundary is at the GUI. Pane_id stays in the GUI/TUI world. Tab_id is
Chromium's world. The GUI translates between them.

## Design

### Chromium changes (4 files)

#### 1. `shell_browser_main_parts.h`

- Remove `std::string pane_id` from `TabState`
- Remove `CloseTabByPaneId` declaration
- Add `FindTabById(int tab_id)` helper
- Add `CloseTabById(int tab_id)` to replace `CloseTabByPaneId`

#### 2. `shell_browser_main_parts.cc`

**Tab lookup:** Replace all `pane_id` string comparisons with `tab_id` integer
comparisons. The repeated pattern:

```cpp
// Before:
for (auto& t : tabs_) {
    if (t->pane_id == pane_id) { tab = t.get(); break; }
}

// After:
TabState* tab = FindTabById(tab_id);
```

**Inbound messages:** Change every `xpc_dictionary_get_string(event, "pane_id")`
to `xpc_dictionary_get_int64(event, "tab_id")`. Affects: `resize`,
`mouse_event`, `scroll_event`, `mouse_move`, `focus_changed`, `key_event`,
`navigate`, `set_color_scheme`, `close_tab`.

**`create_tab`:** Special case — the GUI doesn't know the tab_id yet (Chromium
assigns it). Two options:

- **Option A:** Return `tab_id` in the synchronous reply to `create_tab`. The
  GUI sends `create_tab { url, pixel_width, pixel_height, dark }`, Chromium
  assigns `tab_id`, returns it in the reply alongside the per-tab endpoint. No
  pane_id needed at all.
- **Option B:** Keep an opaque correlation ID (`request_id`) that Chromium
  echoes back in `tab_ready`.

Option A is simpler. The GUI already waits for the reply (to get the per-tab
endpoint). Adding `tab_id` to the reply is trivial.

**`create_devtools_tab`:** Same as `create_tab` — return `tab_id` in the reply.
`inspected_tab_id` is already a tab_id, so no change needed there.

**`tab_ready`:** Remove `pane_id` from the message. Keep `tab_id`. (Or remove
`tab_ready` entirely if `tab_id` is in the create_tab reply — but keep it for
now as a signal that rendering is ready.)

**Outbound messages:** Remove `pane_id` from `ca_context`, `cursor_changed`,
`url_changed`, `loading_state`, `title_changed`. These go on the per-tab
connection, so the GUI already knows which tab they belong to (by connection
identity). Include `tab_id` for robustness.

**`CloseTabByPaneId` → `CloseTabById`:** Change the lookup from string
comparison to integer comparison.

#### 3. `shell_tab_observer.h`

- Remove `std::string pane_id_`
- Remove `SetPaneId()`
- Add `int tab_id_` and `SetTabId(int tab_id)`

#### 4. `shell_tab_observer.cc`

- Replace all `pane_id_` with `tab_id_` in outbound messages
- Change `xpc_dictionary_set_string(msg, "pane_id", pane_id_.c_str())` to
  `xpc_dictionary_set_int64(msg, "tab_id", tab_id_)`

### GUI changes (xpc.zig)

#### Outbound (GUI → Chromium)

Every message that currently sends `pane_id` to Chromium must send `tab_id`
instead. The GUI already has `tab_id` in the Pane struct.

Functions affected: `sendCreateTab` (no pane_id, read tab_id from reply),
`sendCreateDevToolsTab` (same), all forwarding functions (resize, mouse, key,
navigate, etc.).

**`sendCreateTab` flow change:**

```zig
// Before:
// 1. Send create_tab { pane_id, url, ... }
// 2. Receive tab_ready { pane_id, tab_id } on per-tab connection
// 3. Store tab_id in Pane

// After:
// 1. Send create_tab { url, ... } — no pane_id
// 2. Reply includes { tab_id, endpoint }
// 3. Store tab_id in Pane immediately
// 4. Receive tab_ready { tab_id } on per-tab connection (rendering ready)
```

#### Inbound (Chromium → GUI)

Messages from Chromium now include `tab_id` instead of `pane_id`. The GUI needs
a reverse lookup: tab_id → Pane. Add a new map:

```zig
var tab_to_pane: AutoHashMap(i64, []const u8) = .init(alloc);  // tab_id → pane_id
```

Populated when `tab_id` is assigned (from create_tab reply). Used to route
`cursor_changed`, `url_changed`, `loading_state`, `title_changed` back to the
correct pane/TUI.

Alternatively, since per-tab connections already identify the tab, the GUI can
maintain a `tab_conn_to_pane` map (XPC connection → pane_id). This is already
partially done via `peer_to_pane`.

#### `handleDisconnect` change

Currently sends `close_tab { pane_id }` to Chromium. Change to
`close_tab { tab_id }`.

### TUI changes: None

The TUI only communicates with the GUI using `pane_id`. The TUI never talks to
Chromium directly. No TUI changes needed.

## Test

1. `web google.com` → page loads, URL bar updates
2. Navigate in Edit mode → URL changes, loading indicator works
3. Mouse clicks, scroll, text selection → all work
4. Keyboard input → typing in search bars works
5. `:devtools right` → DevTools opens, inspects correct tab
6. Close DevTools pane → pane closes, no crash
7. Reopen DevTools → works multiple times
8. Open two browser panes (Cmd+D split) → both work independently
9. Different profiles → each profile's Chromium process handles its tabs
10. Close a pane → tab cleaned up, other panes unaffected
11. `web status` → shows tab inventory with tab_ids
