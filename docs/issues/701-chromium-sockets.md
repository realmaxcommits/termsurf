# Issue 701: Replace GUI↔Chromium XPC with Unix Sockets

## Goal

Replace the GUI↔Chromium IPC channel with Unix domain sockets + protobuf. This
is the second half of the XPC removal — Issue 700 replaced TUI↔GUI. After this
issue, there is no XPC anywhere in the stack and the xpc-gateway daemon can be
deleted.

## Background

### What Issues 698–700 proved

Issue 698 proved protobuf wire compatibility across Zig (protobuf-c), Rust
(prost), and C++ (libprotobuf), and proved Unix socket round-trips across Zig
and Rust. Issue 699 solved the build system integration — protobuf-c compiles
into the GUI's xcframework via the `gui/src/protobuf/` stb.c pattern. Issue 700
replaced TUI↔GUI XPC with sockets end-to-end across three experiments.

### What exists now

- **Proto schema:** `proto/termsurf.proto` — 30 messages in a `oneof` wrapper,
  shared across all three build systems.
- **GUI socket listener:** `xpc.zig` listens on
  `$TMPDIR/termsurf/gui{-debug}.sock`. Currently handles one TUI connection.
  Uses `dispatch_source` on the serial `xpc_queue`.
- **GUI protobuf-c:** `gui/src/protobuf/` — runtime + generated code, linked
  into the final binary.
- **Wire format:** 4-byte LE length prefix + serialized `TermSurfMessage`.
- **Chromium protobuf:** `third_party/protobuf/` ships with Chromium. The
  `proto_library.gni` template compiles `.proto` → C++ at build time.

### What still uses XPC

The GUI↔Chromium channel — both directions:

**GUI → Chromium (commands, via `server.peer`):**

| Message             | Fields                                                                    |
| ------------------- | ------------------------------------------------------------------------- |
| `CreateTab`         | url, pane_id, pixel_width, pixel_height, dark                             |
| `CreateDevtoolsTab` | pane_id, inspected_tab_id, pixel_width, pixel_height, dark                |
| `Resize`            | tab_id, pixel_width, pixel_height                                         |
| `MouseEvent`        | tab_id, type, x, y, button, click_count, modifiers                        |
| `MouseMove`         | tab_id, x, y, modifiers                                                   |
| `ScrollEvent`       | tab_id, x, y, delta_x, delta_y, phase, momentum_phase, precise, modifiers |
| `KeyEvent`          | tab_id, type, windows_key_code, utf8, modifiers                           |
| `FocusChanged`      | tab_id, focused                                                           |
| `Navigate`          | tab_id, url                                                               |
| `SetColorScheme`    | tab_id, dark                                                              |
| `CloseTab`          | tab_id                                                                    |
| `QueryTabs`         | (reply expected)                                                          |

**Chromium → GUI (events, via per-tab XPC connection):**

| Message          | Fields                                           |
| ---------------- | ------------------------------------------------ |
| `ServerRegister` | profile                                          |
| `TabReady`       | pane_id, tab_id                                  |
| `CaContext`      | tab_id, ca_context_id, pixel_width, pixel_height |
| `UrlChanged`     | tab_id, url                                      |
| `LoadingState`   | tab_id, state, progress                          |
| `TitleChanged`   | tab_id, title                                    |
| `CursorChanged`  | tab_id, cursor_type                              |

### Current connection flow

```
GUI                       Gateway              Chromium
 |---register_app(endpoint)-->|                    |
 |                            |                    |
 |  (GUI spawns Chromium with --xpc-service=...)   |
 |                            |<--get_endpoint-----|
 |                            |---endpoint-------->|
 |                            |                    |
 |<========= XPC control connection (from endpoint) ========>|
 |<========= XPC per-tab connections (from endpoint) =======>|
```

### New connection flow

```
GUI                                     Chromium
 |  (GUI spawns Chromium with --ipc-socket=path)  |
 |                                                 |
 |<-------- socket connect to gui.sock ------------|
 |<-------- ServerRegister { profile } ------------|
 |                                                 |
 |========= single bidirectional socket ==========>|
```

No gateway, no endpoint handshake, no per-tab connections. One socket per
Chromium server process. The `tab_id` field in every message identifies the tab.

### Chromium source files to modify

All TermSurf code lives in `content/chromium_profile_server/browser/`:

- `shell_browser_main_parts.cc` (~900 lines) — XPC handshake, control connection
  handler, all GUI→Chromium command dispatch. **The main file.**
- `shell_browser_main_parts.h` — declares `TabState` with
  `xpc_connection_t tab_connection`, XPC handler methods.
- `shell_tab_observer.cc` — sends `url_changed`, `loading_state`,
  `title_changed`, `cursor_changed` via per-tab XPC connection.
- `shell_tab_observer.h` — holds `xpc_connection_t xpc_connection_`.
- `shell_switches.h` — defines `kXpcService` switch name.

### Chromium branch

Base: `146.0.7650.0-issue-694` (latest TermSurf branch). New branch:
`146.0.7650.0-issue-701`.

## Architecture decisions

### One socket per server process

Each Chromium server process (one per browser profile) opens a single
bidirectional socket to the GUI. Replaces the control connection + N per-tab
connections. Simpler, fewer file descriptors, same serialization.

### Multi-client accept in the GUI

The GUI's socket listener currently handles one TUI connection (`tui_fd`). For
Chromium, it needs to accept multiple concurrent connections — one TUI plus N
Chromium servers. The accept handler must create per-connection state (fd, read
buffer, dispatch_source) and distinguish connection types by the first message
received.

### Server.peer becomes Server.fd

In `xpc.zig`, `Server.peer: xpc_object_t` becomes `Server.fd: std.posix.fd_t`.
All `xpc_connection_send_message(server.peer, msg)` calls become
`sendProtobuf(server.fd, &wrapper)`. The XPC dictionary construction in
`sendCreateTab`, `sendResize`, `sendMouseEvent`, etc. is replaced with protobuf
struct initialization.

### Chromium uses C++ protobuf (not protobuf-c)

The GUI uses protobuf-c (C API, via `@cImport`). Chromium uses
`third_party/protobuf` (C++ API, via `proto_library.gni`). Both produce
identical wire format — that's protobuf's purpose. The C++ side uses
`ParseFromString` / `SerializeToString`.

### --xpc-service becomes --ipc-socket

Chromium currently receives `--xpc-service=com.termsurf.xpc-gateway`. This
becomes `--ipc-socket=/path/to/gui.sock`. The GUI passes the actual socket path
(from `sock_path_buf`) as a command-line argument to the server process.

## Ideas for experiments

- **Protobuf in Chromium BUILD.gn.** Copy `termsurf.proto` into the Chromium
  tree, add a `proto_library` target, verify `autoninja` compiles it to C++.
  Proof that the schema works in Chromium's build system.

- **Multi-client socket accept.** Extend the GUI's socket listener to handle
  multiple concurrent connections — per-connection read buffers,
  dispatch_sources, and connection type tagging (TUI vs Chromium). Replace the
  single `tui_fd` with a connection map.

- **Chromium socket client.** Replace the XPC handshake in
  `shell_browser_main_parts.cc` with a Unix socket connect to `--ipc-socket`.
  Send `ServerRegister` as the first message. Receive `CreateTab` and reply with
  `TabReady` + `CaContext`. Minimal viable round-trip proving the socket works.

- **Full Chromium message replacement.** Convert all remaining XPC messages to
  protobuf in both directions. Replace the XPC message handler with a socket
  reader. Replace per-tab XPC connections in `shell_tab_observer.cc` with sends
  over the shared server socket.

- **GUI → Chromium socket sends.** Replace all
  `xpc_connection_send_message( server.peer, msg)` calls in `xpc.zig` with
  `sendProtobuf(server.fd, &wrapper)`. Convert `sendCreateTab`, `sendResize`,
  `sendMouseEvent`, `sendKeyEvent`, etc. from XPC dict construction to protobuf
  struct initialization.

- **End-to-end integration.** Full runtime test — launch GUI, open a web page,
  verify browser renders via socket-only IPC. All 12 GUI→Chromium and 7
  Chromium→GUI message types working.

- **Remove xpc-gateway.** Delete the gateway daemon entirely. Remove all XPC
  client code from Chromium. Remove the gateway connection and endpoint
  registration from `xpc.zig`. Clean up `TERMSURF_XPC_SERVICE` env var and
  launchd plist.

## Experiments
