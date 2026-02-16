# Issue 511: Three Profiles

## Background

Issue 510 proved two different browser profiles render side by side in the same
terminal window at 60fps, each with fully isolated sessions. But each pane still
spawns its own Chromium Profile Server process, even when two panes share the
same profile name. This works for the two-profile demo (one pane per profile),
but it breaks the moment two panes use `--profile work`.

Chromium acquires a lock on `--user-data-dir` at startup. A second process with
the same `--user-data-dir` will fail to initialize. Two panes sharing a profile
**must** share a single server process. Server reuse is not an optimization — it
is a correctness requirement.

The Chromium Profile Server already supports multiple tabs. Issue 503 Experiment
3 proved that one server process can host N WebContents from the same
BrowserContext, each with an independent FrameSinkVideoCapturer streaming at
60fps. The `CreateTab` method can be called multiple times, each adding a new
Shell + VideoConsumer + per-tab XPC connection. `CloseTab` fires automatically
when a tab's connection drops, removing the tab without shutting down the
server. The infrastructure exists — it just isn't wired up.

## Goal

Three panes in the same terminal window:

- Pane A: `--profile work` (server spawns)
- Pane B: `--profile personal` (second server spawns)
- Pane C: `--profile work` (reuses Pane A's server, sends `create_tab`)

This proves two capabilities:

1. **Two different profiles in the same window.** Already proven in Issue 510,
   but re-confirmed here with a third pane in the mix.
2. **Two panes sharing the same profile.** An ordinary feature that all users
   will expect to work flawlessly. Both panes share one server process, one
   BrowserContext, one `--user-data-dir` — but each gets its own WebContents
   rendering at 60fps.

## Product requirements

### Server lifecycle

Each profile gets exactly one Chromium Profile Server process. The lifecycle:

- **Spawn** when the first pane requests a profile that has no running server.
- **Add tab** when a subsequent pane requests a profile that already has a
  running server. No new process — send `create_tab` on the existing control
  connection.
- **Remove tab** when a pane disconnects. The server closes that tab's
  WebContents and video consumer.
- **Shut down** when the last tab closes (N reaches 0). The server process
  exits.

N can be any positive integer. A server with 5 tabs manages 5 independent
WebContents, each streaming frames to a different terminal pane.

### Frame routing

Each pane must receive only its own frames. The current architecture stamps
`pane_id` on every `display_surface` message, and the compositor routes by
`pane_id` to the correct Ghostty surface. This already works — the challenge is
making `pane_id` per-tab instead of per-process.

Currently the server receives `--pane-id` once from the command line and stamps
it on every video consumer. With server reuse, the server manages tabs for
multiple panes. Each tab's `pane_id` must come from the `create_tab` message,
not from the command line.

### Resize routing

The current `resize` message on the control connection resizes `tabs_[0]` — the
first (and only) tab. With multiple tabs, resize must route to the correct tab.
Each pane sends resize independently when the terminal is resized, so the
message must identify which tab to resize.

### Display

No UI changes. The URL bar already shows the profile name. Each pane renders
independently.

## Current state

### What already works

| Component                  | Status  | Notes                                               |
| -------------------------- | ------- | --------------------------------------------------- |
| `CreateTab` (server)       | Working | Adds Shell + VideoConsumer + per-tab XPC connection |
| `CloseTab` (server)        | Working | Fires when tab connection drops, server continues   |
| Multi-tab 60fps            | Proven  | Issue 503 Experiment 3: N WebContents, N capturers  |
| `display_surface` routing  | Working | `pane_id` in every message, compositor routes by it |
| Profile propagation        | Working | Issue 510: `web` sends profile, app extracts it     |
| Per-profile data directory | Working | Issue 510: `--user-data-dir` per profile            |
| XPC serialization          | Working | Issue 510: all peers on same serial queue           |

### What needs to change

**1. App: Profile-keyed server tracking.**

`CompositorXPC.swift` maps everything by pane UUID:

```swift
private var serverProcesses: [UUID: Process] = [:]
private var serverControlConnections: [UUID: xpc_connection_t] = [:]
```

A second pane requesting `--profile work` spawns a new server because
`serverProcesses[newUUID]` is nil. The mapping needs to change from pane UUID to
profile name so the app can detect an existing server for the same profile.

**2. App: Server reuse in `handleSetOverlay`.**

When `set_overlay` arrives with a URL and a profile name:

- If no server exists for this profile: spawn one, store the URL as pending.
- If a server already exists: send `create_tab` immediately on the existing
  control connection. No spawn needed.

**3. App: Disconnect logic.**

Currently `handleDisconnect` kills the server process when any web peer
disconnects. With server reuse, a disconnect should only remove one tab from the
server. The server should be killed (or allowed to exit) only when all panes for
that profile have disconnected.

**4. Server: Per-tab `pane_id`.**

`pane_id_` is a process-level field set once from `--pane-id`:

```cpp
std::string pane_id_;  // set from command line, shared by all tabs
```

Every tab's video consumer gets the same `pane_id_` via `SetPaneId(pane_id_)`.
For server reuse, each tab needs its own `pane_id` so the compositor can route
frames to the correct Ghostty surface. The `create_tab` message must include the
target pane UUID, and each video consumer must store its own.

**5. Server: Per-tab resize.**

`ResizeCapture` only operates on `tabs_[0]`:

```cpp
auto& tab = tabs_[0];
```

With multiple tabs, resize must accept a pane identifier and route to the
correct `TabState`. The `resize` message on the control connection must include
`pane_id`.

**6. Server: `--pane-id` removal.**

The server no longer belongs to a single pane. The `--pane-id` command-line flag
should be removed. The server identifies itself by its profile (the app already
knows which profile it spawned the server for). Each tab gets its `pane_id` from
`create_tab`.

**7. Server: `server_register` update.**

Currently `server_register` sends `pane_id`:

```cpp
xpc_dictionary_set_string(reg, "action", "server_register");
xpc_dictionary_set_string(reg, "pane_id", pane_id_.c_str());
```

With `--pane-id` removed, the server needs another way to identify itself to the
compositor. The simplest option: send `profile` (the basename of the
`--user-data-dir` path). Alternatively, since the compositor already knows which
profile it spawned the server for, `server_register` could be purely a handshake
with no routing information.

### What should work without changes

**xpc-gateway** — Pure stateless rendezvous. No profile or tab awareness.

**Metal renderer** — Composites overlays by pane UUID. Each pane independently
receives IOSurface frames. No changes needed.

**`web` CLI** — Already sends `--profile` in `set_overlay`. No changes needed.

**Profile name validation** — Already implemented in Issue 510.

## XPC protocol changes

### `web` -> app: `set_overlay`

Unchanged from Issue 510.

```
{ action: "set_overlay",
  pane_id: "<uuid>",
  col: N,
  row: N,
  width: N,
  height: N,
  url: "http://...",
  profile: "work" }
```

### app -> server: `create_tab`

Add `pane_id` so the server can stamp each tab's frames with the correct pane
UUID. Remove `tab_id` — each pane corresponds 1-to-1 with a tab, so `pane_id` is
the natural identifier for both.

```
{ action: "create_tab",
  url: "http://...",
  pane_id: "<uuid>",         // (new) identifies pane and tab
  pixel_width: N,
  pixel_height: N }
```

### app -> server: `resize`

Add `pane_id` so the server knows which tab to resize.

```
{ action: "resize",
  pane_id: "<uuid>",         // (new) which tab to resize
  pixel_width: N,
  pixel_height: N }
```

### server -> app: `server_register`

Replace `pane_id` with `profile`.

```
{ action: "server_register",
  profile: "<name>" }        // (changed) profile name instead of pane_id
```

### server -> app: `tab_ready`

Replace `tab_id` with `pane_id`.

```
{ action: "tab_ready",
  pane_id: "<uuid>" }
```

### server -> app: `display_surface`

Unchanged in structure — `pane_id` is already present. The difference is that
it's now per-tab instead of per-process.

```
{ action: "display_surface",
  iosurface_port: <mach_port>,
  pane_id: "<uuid>" }
```

## Architecture note: pane-to-tab mapping

Each terminal pane corresponds 1-to-1 with a Chromium tab (WebContents). There
is no scenario where one pane has multiple tabs or one tab spans multiple panes.
`pane_id` is the single identifier used everywhere — in XPC messages, in frame
routing, and in resize routing. There is no separate `tab_id`.

The server's `tabs_` vector currently stores `TabState` with Shell,
VideoConsumer, and tab_connection. For server reuse, each `TabState` also needs
the `pane_id` of the terminal pane it's rendering for:

```cpp
struct TabState {
    raw_ptr<Shell> shell;
    std::unique_ptr<ShellVideoConsumer> video_consumer;
    xpc_connection_t tab_connection = nullptr;
    std::string pane_id;    // (new) identifies pane and routes frames + resize
};
```

The flow for a shared server:

1. Pane A sends `set_overlay` with `profile=work`. Compositor spawns server.
2. Server sends `server_register`. Compositor stores control connection keyed by
   profile.
3. Compositor sends `create_tab` with `pane_id=A`. Server creates Tab 1, stamps
   A on its frames.
4. Pane C sends `set_overlay` with `profile=work`. Compositor finds existing
   server for `work`.
5. Compositor sends `create_tab` with `pane_id=C`. Server creates Tab 2, stamps
   C on its frames.
6. Both tabs stream at 60fps. Compositor routes by `pane_id` — no confusion.
7. Pane A disconnects. Compositor tells server to close Tab 1. Server keeps
   running for Tab 2.
8. Pane C disconnects. Compositor tells server to close Tab 2. Server has 0 tabs
   and exits.

## Ideas for future experiments

1. **Per-tab pane_id in Chromium.** Remove `--pane-id`, add `pane_id` to
   `create_tab`, store it per-TabState, pass to each VideoConsumer. Test with a
   single pane to verify frames still route correctly.

2. **Per-tab resize.** Add `pane_id` to `resize` message, look up the correct
   TabState, resize that tab's view and capturer. Test with a single pane.

3. **Profile-keyed server tracking in the app.** Restructure
   `CompositorXPC.swift` to map servers by profile name. When a second pane
   requests the same profile, send `create_tab` instead of spawning. Test with
   two panes, same profile.

4. **Server shutdown on last tab close.** The server currently idles forever
   with zero tabs. Add auto-exit when `tabs_` becomes empty after a `CloseTab`.
   Test by closing both panes and verifying the server process exits.

5. **Three panes, two profiles.** The full demo: panes A and C with `work`, pane
   B with `personal`. Two server processes, three frame streams, all at 60fps.
