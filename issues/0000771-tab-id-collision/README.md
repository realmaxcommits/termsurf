+++
status = "open"
opened = "2026-04-05"
+++

# Issue 771: Tab ID collision across browser profiles

## Goal

Fix the bug where having two browser profiles open simultaneously causes one
pane to visually "clone" the other when navigating.

## Background

This is the same bug as Issue 769, reopened now that the browser is working
again (Issue 770 was a macOS SDK mismatch unrelated to the tab_id fix).

Issue 769's experiment 1 had the correct approach (composite key) but failed
because it was tested while the browser was broken due to the macOS 26.4 sandbox
issue. The experiments were reverted and the issue was closed. Now that Chromium
has been rebuilt against 26.4, we can attempt the fix again.

### The bug

Each browser profile runs as a separate Roamium process. Chromium assigns each
tab a `tab_id` — a per-process integer. Two separate Chromium processes
independently generate the same `tab_id` values.

The GUI maintains `tab_to_pane: HashMap<i64, String>` mapping `tab_id` →
`pane_id`. When two profiles produce the same `tab_id`, the second insert
overwrites the first. All subsequent `CaContext` messages with that `tab_id`
route to the wrong pane, causing one profile to visually display the other's
content.

Refreshing fixes it temporarily because the correct browser re-renders and sends
a new `CaContext` that gets routed to the right pane.

### Reproduction

1. Open two panes with different profiles (e.g., "default" and "work").
2. Navigate to different URLs in each.
3. Navigate in pane 1 → pane 2 visually shows pane 1's page.
4. Refresh pane 2 → correct page returns until next navigation in pane 1.

## Analysis

### The fix

Change `tab_to_pane` from `HashMap<i64, String>` to
`HashMap<(String, i64), String>` where the `String` is the server key
(`"{profile}\0{browser}"`). Every pane stores `profile` and `browser`, so the
key is available at every site.

### All code sites

**Declaration** (`state.rs:52`):

```rust
pub tab_to_pane: HashMap<i64, String>,
// → HashMap<(String, i64), String>,
```

**Insert** (`conn.rs` `handle_tab_ready`):

```rust
st.tab_to_pane.insert(ready.tab_id, ready.pane_id.clone());
// → use pane.profile + pane.browser to build composite key
```

**Lookups** (`conn.rs`):

- `handle_ca_context` — routes CaContext to pane
- CursorChanged handler — routes cursor type to pane
- DevTools lookup — finds inspected pane

**Remove** (`conn.rs` `handle_disconnect`):

```rust
st.tab_to_pane.remove(&pane.tab_id);
// → use pane.profile + pane.browser to build composite key
```

### How to thread the server_key

Messages from browser sockets (CaContext, CursorChanged) need the server_key to
build the composite lookup key. The connection reader loop must track which
server this connection belongs to.

**Approach:** When `handle_server_register` runs, it matches a server by
profile. Store the matched server_key on the connection. Pass it to
`handle_message` so browser-originated lookups can use it.

### Lessons from Issue 769

Issue 769 experiment 1 failed for two reasons:

1. The implementation intercepted `ServerRegister` in the connection loop with a
   double-match pattern (`matches!` then `if let`) and removed it from
   `handle_message`. This restructuring may have introduced a subtle bug.

2. The failure could not be diagnosed because the browser was simultaneously
   broken by the macOS 26.4 sandbox issue (Issue 770). All testing showed
   "browser doesn't load" which was blamed on the code changes but was actually
   the OS.

For this attempt: keep `ServerRegister` inside `handle_message` (don't
restructure the message loop). Instead, have `handle_server_register` store the
server_key in shared state keyed by the connection's `tx` channel, so other
handlers can look it up without needing a parameter threaded through the loop.

Alternatively, since `handle_tab_ready` already builds the key from the pane
(not the connection), and `handle_ca_context` / CursorChanged receive messages
from a browser whose `tab_id` is already in `tab_to_pane` — we could do a
**reverse lookup**: for browser-originated messages, iterate `tab_to_pane` to
find any entry matching the `tab_id` on the connection's server. But this
defeats the purpose of the HashMap.

The simplest correct approach: add a `server_key: Option<String>` field to the
connection state, set it when `ServerRegister` is processed, and pass it to
`handle_message`. This is what 769 tried. The key difference: don't remove
`ServerRegister` from `handle_message`'s match — just add a side effect that
also stores the key on the connection.
