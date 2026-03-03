# Issue 702: Socket IPC Cleanup

## Goal

Remove all dead XPC code from both the GUI and Chromium, and remove the
fixed-size client connection limit in the GUI. Issues 698–701 replaced all IPC
with Unix sockets + protobuf. This issue cleans up the remnants.

## Background

- [Issue 698](698-unix-sockets.md) — Proved Unix socket + protobuf round-trips
  across Zig, Rust, and C++.
- [Issue 699](699-protobuf-build.md) — Built protobuf-c into the GUI.
- [Issue 700](700-tui-gui-sockets.md) — Replaced TUI↔GUI XPC with sockets.
- [Issue 701](701-chromium-sockets.md) — Replaced GUI↔Chromium XPC with sockets.

After Issue 701, no XPC messages flow at runtime. All IPC uses Unix domain
sockets with length-prefixed protobuf. But the XPC code is still in the
codebase, and the GUI's socket listener uses a fixed 16-slot connection pool.

## Part 1: Dead XPC Code Removal

### Chromium (`chromium/src/content/chromium_profile_server/`)

- `shell_browser_main_parts.cc`:
  - `StartDynamicMode()` — XPC gateway handshake. Dead.
  - `control_connection_` and `app_endpoint_` — XPC connection/endpoint storage.
    Dead.
  - Per-tab XPC connection creation in `CreateTab()` and `CreateDevToolsTab()` —
    the `else` branches that call `xpc_connection_create_from_endpoint`. Dead.
  - XPC message handler for the control connection. Dead.
  - `HandleQueryTabs()` XPC reply path. Dead.
- `shell_browser_main_parts.h`:
  - `xpc_connection_t control_connection_`, `xpc_object_t app_endpoint_`
    declarations. Dead.
  - `TabState::tab_connection` (per-tab XPC connection). Dead.
  - XPC handler method declarations. Dead.
- `shell_tab_observer.cc`:
  - XPC fallback branches in `OnCursorChanged`, `DidFinishNavigation`,
    `SendLoadingState`, `TitleWasSet` — the `else if (xpc_connection_)` paths.
    Dead.
- `shell_tab_observer.h`:
  - `xpc_connection_t xpc_connection_` member. Dead.
  - `SetConnection(xpc_connection_t)` method. Dead.
- `shell_switches.h`:
  - `kXpcService` switch. Dead.

### GUI (`gui/src/apprt/xpc.zig`)

- XPC gateway connection and anonymous listener — the `register_app(endpoint)`
  handshake. Dead.
- `server.peer` field and all `xpc_connection_send_message(server.peer, ...)`
  calls — the `else` branches in every send function. Dead.
- XPC fallback branches in all 10 GUI→Chromium send functions (`sendCreateTab`,
  `sendCreateDevToolsTab`, `sendResize`, `sendFocusMessage`, `sendMouseEvent`,
  `sendScrollEvent`, `sendMouseMove`, `sendKeyEvent`, `handleNavigate`,
  `handleSetColorScheme`). Dead.
- XPC fallback in close-tab sends in `handleDisconnect` and
  `handleClientDisconnect`. Dead.
- `peer_to_profile` and `peer_to_pane` maps (keyed by XPC peer address). Dead.
- `Server.peer` field. Dead.
- `Pane.web_peer` field. Dead.
- `handleServerRegister` XPC path (the non-socket branch). Dead.
- `TERMSURF_XPC_SERVICE` env var and launchd plist references. Dead.

### XPC Gateway Daemon

The entire gateway daemon can be deleted once all XPC code is removed. It was
the intermediary that brokered XPC connections between GUI and Chromium.

## Part 2: Unlimited Client Connections

The GUI's socket listener uses a fixed-size array:

```zig
const MAX_CLIENTS = 16;
var clients: [MAX_CLIENTS]ClientConn = [_]ClientConn{.{}} ** MAX_CLIENTS;
```

Each `ClientConn` has a 65KB read buffer, so 16 slots = 1MB pre-allocated. This
caps the number of simultaneous TUI + Chromium connections at 16.

Replace with heap-allocated `ClientConn`s (same pattern as `Pane` and `Server`)
so there is no fixed limit. Each connection is allocated on accept and freed on
disconnect.

## Experiments

_None yet._
