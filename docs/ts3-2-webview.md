# TermSurf 3.0 Webview Rendering

## Background

This document continues the work started in
[ts3-1-architecture.md](./ts3-1-architecture.md), which established the core
process model for TermSurf 3.0.

### What We Accomplished

TermSurf 3.0 uses a multi-process architecture for browser integration:

- **Profile servers**: Separate CEF processes, one per profile, providing true
  session isolation (different cookies, storage, login sessions)
- **Socket communication**: Unix domain sockets with JSON protocol for
  coordinator-to-profile-server and coordinator-to-GUI communication
- **Connection-based lifecycle**: Webview lifetime tied to coordinator
  connection, ensuring crash-proof cleanup

Through 8 experiments, we validated:

1. CEF initializes correctly with profile-specific cache paths
2. CEF enforces single-process-per-profile via `SingletonLock`
3. Socket communication works (ping/pong, open, get_status)
4. Multiple coordinators share one profile server
5. Webviews close automatically when coordinators disconnect
6. GUI socket server receives texture handle messages

**What failed**: Cross-process IOSurface sharing via global IDs.
`IOSurfaceLookup()` returns NULL for IOSurfaces created by CEF's GPU process.
The texture handle passes successfully between processes, but the receiving
process cannot access the actual texture data.

## Goal

**Display a webview texture rendered by the profile server in the GUI.**

The profile server (CEF) renders web content to a texture. The GUI (wezterm-gui)
must display that texture in a terminal pane. The challenge is sharing GPU
texture data between two separate processes on macOS.

### Requirements

1. Profile server renders webpage to a texture (already working)
2. Texture data becomes accessible to GUI process
3. GUI imports texture and renders it in the correct pane location
4. No visible latency or tearing during normal browsing

### Approaches to Explore

Since `IOSurfaceLookup()` failed, we need alternative approaches:

1. **Mach port-based IOSurface transfer**: Pass IOSurface references between
   processes using Mach ports instead of global IDs
2. **Shared memory with pixel copy**: Profile server copies pixel data to shared
   memory, GUI reads and uploads to GPU
3. **XPC services**: Use macOS XPC for structured cross-process communication
   with IOSurface support
4. **CALayerHost/CARemoteLayer**: Use Core Animation's built-in cross-process
   layer sharing

## Future Plans

Once texture display works, we will address (in rough order):

1. **Resize handling**: When the pane resizes, tell CEF to re-render at the new
   size (not just stretch the texture)
2. **Keyboard input**: Route keystrokes to CEF when webview pane is focused
3. **Mouse input**: Route clicks, scrolls, and hover events to CEF
4. **Keybindings**: Implement browse mode vs control mode (like ts1/ts2)
5. **Console output**: Stream `console.log` from browser to terminal
6. **Navigation controls**: Back, forward, reload, URL bar
7. **Multiple webviews**: Support multiple browser panes simultaneously
8. **Focus management**: Track which pane has focus, route input accordingly

These features are deferred until we solve the fundamental texture sharing
problem.

## Experiments

_Experiments will be added as we explore solutions to cross-process texture
sharing._
