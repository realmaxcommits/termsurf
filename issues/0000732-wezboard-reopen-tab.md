# Issue 732: Wezboard cannot reopen browser after last tab closes

## Goal

After closing the last browser tab in Wezboard, opening a new webview should
work — launching a fresh browser process if needed. Currently, the second
webview never appears.

## Background

### Symptom

1. Open a webview in Wezboard (`web localhost:3000`).
2. Close that tab (the only browser tab).
3. Open a new webview (`web localhost:3000` again).
4. No webview appears. The browser overlay is missing.

### What happens under the hood

When the TUI disconnects (tab closes), `handle_disconnect` in `conn.rs:862-905`
cleans up the pane:

1. Removes the pane from `st.panes` (line 871).
2. Removes the tab-to-pane mapping (line 873).
3. Sends `CloseTab` to Chromium (lines 879-884).
4. Decrements `server.pane_count` (line 877).
5. Removes CALayers (lines 889-895).

But the **server entry is never removed** from `st.servers`. The `Server` struct
stays in the HashMap with `pane_count: 0` and potentially a stale `tx` channel.

When the user opens a new webview, `handle_set_overlay` (line 556) checks
`st.servers.contains_key(&key)`. Since the old server entry still exists, it
takes the `else` branch (line 568) — reusing the existing server instead of
spawning a new one. It increments `pane_count` and tries to send `CreateTab`
through the server's `tx` channel.

The problem: by this point, the Roamium process has likely exited (after its
last tab was closed), and the socket connection is dead. Two failure modes:

1. **Roamium exited, `ConnType::Chromium` disconnect was handled** — `server.tx`
   is `None`. The `CreateTab` message is never sent. The log shows:
   `"SetOverlay: server exists but tx is None — CreateTab not sent!"` (line
   583).

2. **Roamium exited but disconnect not yet processed** — `server.tx` is `Some`
   but the channel is closed. `try_send` silently fails or errors.

Either way, no `CreateTab` reaches Chromium, no `TabReady` comes back, and no
webview appears.

### The stale server problem

The root cause is that `st.servers` is never cleaned up. The server entry
persists indefinitely after the process exits. The get-or-create logic in
`handle_set_overlay` (line 556) trusts that an existing entry means a live,
reachable server — but that's not true after the process has exited.

### Relevant code paths

| Location          | What it does                                                         |
| ----------------- | -------------------------------------------------------------------- |
| `conn.rs:554-585` | Get-or-create server in `handle_set_overlay`                         |
| `conn.rs:862-905` | TUI disconnect cleanup (decrements pane_count, never removes server) |
| `conn.rs:906-919` | Chromium disconnect (sets `server.tx = None`, never removes server)  |
| `conn.rs:949-989` | `spawn_server()` — launches Roamium process                          |
| `state.rs:32-39`  | `Server` struct — holds process handle, tx, pane_count               |

### How Ghostboard handles this

In Ghostboard, the browser process lifecycle is managed differently — servers
are spawned per-connection and cleaned up when the connection drops. The stale
server problem doesn't exist because there's no persistent server registry that
outlives the connection.

### Fix strategy

When the get-or-create logic in `handle_set_overlay` finds an existing server
entry, it needs to check whether the server is actually alive and reachable. If
not, it should remove the stale entry and spawn a fresh server.

The simplest check: if `server.tx` is `None` and `server.pane_count == 0`, the
server is dead — remove it and fall through to the spawn path.

A more robust check: also verify the process handle (`server.process`) with
`try_wait()` to detect if the process has exited even before the socket
disconnect was processed.

## Experiments

### Experiment 1: Remove stale servers before spawning

#### Goal

When `handle_set_overlay` finds an existing server entry, check if the server is
actually alive. If it's dead (process exited or `tx` is `None` with no active
panes), remove the stale entry and spawn a fresh server.

#### Changes

**File: `wezboard/wezboard-gui/src/termsurf/conn.rs`** — `handle_set_overlay`,
lines 554–585

Replace the simple `contains_key` check with a liveness check. Before deciding
to reuse a server, verify it's actually reachable:

```rust
// Get or create server
let key = TermSurfState::server_key(&overlay.profile, &browser);
let needs_spawn = match st.servers.get_mut(&key) {
    None => true,
    Some(server) => {
        // Check if server process has exited
        let process_dead = match server.process {
            Some(ref mut child) => match child.try_wait() {
                Ok(Some(_)) => true,  // Process exited
                _ => false,           // Still running (or error)
            },
            None => true,             // No process handle
        };
        if process_dead || (server.tx.is_none() && server.pane_count == 0) {
            log::info!(
                "SetOverlay: removing stale server key={} process_dead={} tx={} pane_count={}",
                key, process_dead, server.tx.is_some(), server.pane_count
            );
            st.servers.remove(&key);
            true
        } else {
            false
        }
    }
};

if needs_spawn {
    // Must drop lock before spawning (spawn_server doesn't need state)
    drop(st);
    let server = spawn_server(&overlay.profile, &browser)?;
    let mut st = state.lock().unwrap();
    st.servers.insert(key.clone(), server);
    // If server already connected (unlikely for fresh spawn), send CreateTab
    let server = st.servers.get(&key).unwrap();
    if let Some(ref server_tx) = server.tx {
        let pane = st.panes.get(&overlay.pane_id).unwrap();
        send_create_tab(server_tx, pane)?;
    }
} else {
    let server = st.servers.get_mut(&key).unwrap();
    server.pane_count += 1;
    // ... existing reuse logic (log, send CreateTab if tx available) ...
}
```

The key insight: `try_wait()` on the `Child` process handle is non-blocking and
tells us if the process has already exited. Combined with the `tx` and
`pane_count` checks, this catches both failure modes described in the issue
background.

No other files need changes — this is entirely within the get-or-create logic in
`handle_set_overlay`.

#### Verification

1. `cd wezboard && cargo build` — compiles without errors
2. Open a webview (`web localhost:3000`), confirm it loads
3. Close the tab (Ctrl+D or close the pane)
4. Open a new webview (`web localhost:3000`) — it should appear and load
5. Verify logs show "removing stale server" on the second open
6. Repeat steps 2–4 multiple times to confirm reliability
