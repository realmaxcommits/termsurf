# Issue 503: One, Two, Three

## Background: One Process Per Profile

The constraint that two processes cannot share the same profile data directory
has been discovered, re-discovered, and documented across eight issue documents
spanning three generations of TermSurf. This section consolidates those
findings.

### ts2: Discovery (Issues 208, 209)

Issue 208 found that CEF's Chrome runtime (post-M128) deliberately ignores
custom `cache_path` settings. The `root_cache_path` IS the profile — one
process, one profile, no exceptions. Issue 209 confirmed this by attempting to
use Chrome's native profile naming (`Default`, `Profile 1`, etc.) with CEF.
Custom profiles fail silently; only the Default profile works. This is
documented CEF behavior, not a bug.

### ts3: Architecture around the constraint (Issues 301, 305, 306, 307)

Issue 301 ("Lessons from ts2") identified this as the core constraint that
necessitated the entire ts3 architecture: out-of-process CEF, one process per
profile. Issue 305 confirmed the mechanism — CEF uses a `SingletonLock` file in
the profile directory; a second process will crash or fail to initialize. Issue
306 discovered that the ts3 code was violating this constraint by spawning a new
`termsurf-profile` process for every `web` command. Running `web google.com`
then `web github.com` with the same profile would crash the second process on
SingletonLock. The fix: detect an existing profile process and send a "create
browser" command to it. Issue 307 formalized this as "the foundational
architectural constraint of ts3" — exactly one `termsurf-profile` process per
browser profile, with multiple webviews within that process sharing cookies and
storage like tabs in a browser.

### ts4: The CEF vs Chromium distinction (Issues 406, 407)

Issue 406 made the critical discovery: **the one-profile-per-process constraint
is CEF-specific, not a Chromium limitation.** Chromium's Content API
(`content::BrowserContext`) fully supports multiple profiles with different
storage paths in the same process. Chrome itself does this routinely. Electron
proves it via `session.fromPartition()`. CEF adds its own constraints on top of
Chromium. This finding killed CEF and led to ts4.

Issue 407 proved it in practice — the in-process Chromium PoC ran two
`BrowserContext` instances with different storage paths, each with its own
cookies, localStorage, and cache, all in one process at 60fps.

### What this means for Issue 503

Multiple `BrowserContext` instances (different profiles) coexist in one process
— proven in Issue 407. But Issue 503 asks a different question: can multiple
`WebContents` from the **same** `BrowserContext` each have their own
`FrameSinkVideoCapturer` delivering independent IOSurface streams? This is the
multi-tab case. The profile server must host an unlimited number of WebContents
per profile, each captured independently and sent over its own XPC connection.

## Problem

The ts4 proof-of-concept demonstrated two **different** browser profiles
rendering side by side at 60fps (Issue 414, Issue 501). But TermSurf needs to
support a harder case: two tabs from the **same** profile in the same window.

There is exactly **one profile server process per profile** — this is a hard
constraint, not a design preference. Two Chromium processes cannot share the
same user data directory (SingletonLock). This was proven in ts2/ts3 with CEF
and remains true with the Content API. A profile server must therefore handle
**multiple WebContents** within a single process, each with its own
`FrameSinkVideoCapturer` delivering an independent IOSurface stream via XPC.

We've never tested this. The current Chromium Profile Server creates exactly one
`Shell` with one `WebContents` and one `ShellVideoConsumer` in
`InitializeMessageLoopContext()`. It needs to support an unlimited number.

Additionally, the working Swift compositor and box-demo test page live in
`ts4/`, but ts5 is the active development directory. These need to be ported to
`ts5/` so that ts5 has its own self-contained test infrastructure.

## Goals

1. **Port the box-demo and Swift compositor from ts4 to ts5.** The ts5 directory
   should have its own working copies that don't depend on ts4.

2. **Validate one profile, one tab.** A single Chromium Profile Server process
   with one WebContents rendering one page to one pane — the simplest case.

3. **Validate two profiles, one tab each.** Two Chromium Profile Server
   processes, each with a different `--user-data-dir`, each hosting one
   WebContents. Two panes in one window. This is the Issue 414/501 case,
   re-validated with the ts5 test infrastructure.

4. **Validate two profiles, three tabs.** Two Chromium Profile Server processes:
   one hosting **two** WebContents (same profile, two tabs), one hosting **one**
   WebContents (different profile). Three panes in one window. The compositor
   opens two XPC connections to the first profile server and one to the second.

## Architecture

### One profile server per profile

```
Profile Server A (profile-a data dir)
├── WebContents 1 → FrameSinkVideoCapturer → IOSurface → XPC connection 1
├── WebContents 2 → FrameSinkVideoCapturer → IOSurface → XPC connection 2
└── WebContents N → ...

Profile Server B (profile-b data dir)
├── WebContents 1 → FrameSinkVideoCapturer → IOSurface → XPC connection 1
└── WebContents N → ...
```

The compositor (Swift app) connects to profile servers via XPC. Each connection
represents one tab. A profile server can accept an unlimited number of
connections, creating a new WebContents and capturer for each.

### XPC connection model: one connection per tab

Two options were considered:

**Option A: One XPC connection per tab.** The compositor opens a new connection
to the profile server for each tab it wants. The profile server's listener fires
once per connection, creating a WebContents and capturer for each.

**Option B: One shared connection, multiplexed.** The compositor opens a single
connection to the profile server. All messages include a tab identifier. Frames
for all tabs flow over the same pipe.

**Decision: Option A.** Three reasons:

1. **Lifecycle is free.** Closing a connection = closing a tab. The profile
   server sees the connection die and tears down the WebContents + capturer. No
   need for explicit "close tab" messages or lifecycle protocol.

2. **No head-of-line blocking.** Each XPC connection has its own dispatch queue.
   If one tab's IOSurface Mach port transfer is slow, it doesn't delay another
   tab's frame delivery. With a shared connection, all tabs compete for the same
   pipe.

3. **Natural XPC model.** When the compositor creates two connections to the
   same Mach service, the profile server's listener fires twice with two
   separate `xpc_connection_t` peers. Each peer naturally maps to one
   WebContents. The existing single-tab code already handles one connection —
   multiple connections is a generalization, not a new abstraction.

The shared-connection approach would require adding a tab identifier to every
message, demuxing on both sides, and explicit lifecycle commands — all
complexity that XPC's connection model provides for free.

### What changes in the Chromium Profile Server

Currently, the profile server creates one WebContents at startup:

```cpp
void ShellBrowserMainParts::InitializeMessageLoopContext() {
  Shell* shell = Shell::CreateNewWindow(...);
  video_consumer_ = std::make_unique<ShellVideoConsumer>();
  video_consumer_->ObserveContents(shell->web_contents());
}
```

For multi-tab support, the profile server needs to:

1. **Listen for incoming XPC connections** requesting new tabs.
2. **Create a new WebContents** (via `Shell::CreateNewWindow()`) for each
   connection.
3. **Create a new ShellVideoConsumer** per WebContents, each sending frames over
   its own XPC connection back to the compositor.

The existing single-tab startup path can remain as a default, but the server
must also accept "create tab" commands over XPC.

## What needs to be ported

### Box demo (`ts4/box-demo/` -> `ts5/box-demo/`)

The test page: a spinning blue square with a localStorage-based identity string
and FPS counter. Served by a Bun HTTP server on port 9407.

Files:

- `server.ts` — Bun HTTP server (12 lines)
- `public/index.html` — Test page (105 lines)

No changes needed — copy as-is.

### Swift compositor (`ts4/two-profiles-swift/` -> adapted for ts5)

The receiver app: an XPC Mach service that accepts IOSurface Mach ports from
Chromium Profile Server processes and composites them into a Metal window using
CADisplayLink.

Source files:

- `Package.swift` — SwiftPM manifest
- `Sources/Receiver/main.swift` — XPC listener, Metal pipeline, rendering (328
  lines)
- `Sources/Receiver/Shaders.metal` — Vertex + fragment shaders (33 lines)
- `com.termsurf.two-profiles-swift.plist` — Launchd agent definition

The two-pane compositor from ts4 must be adapted into three separate apps for
the three test cases.

## Three Swift apps

### `ts5/one-profile/` — One pane, one profile server

The simplest case. One Chromium Profile Server process with one WebContents
sends IOSurface frames to a Swift compositor that renders a single pane filling
the whole window.

- XPC service name: `com.termsurf.one-profile`
- Window: single pane (800x600)
- Profile servers: 1
- WebContents per server: 1
- Session IDs: `profile-a`

### `ts5/two-profiles/` — Two panes, two profile servers

Two Chromium Profile Server processes, each with a different `--user-data-dir`,
each hosting one WebContents. Two panes side by side. The two panes should show
different localStorage identities, proving profile isolation.

- XPC service name: `com.termsurf.two-profiles`
- Window: two panes side by side (1600x600)
- Profile servers: 2
- WebContents per server: 1
- Session IDs: `profile-a`, `profile-b`
- User data dirs: `~/.config/termsurf/poc/profile-a`,
  `~/.config/termsurf/poc/profile-b`

### `ts5/three-profiles/` — Three panes, two profile servers

The new test case. Two Chromium Profile Server processes: the first hosts
**two** WebContents (same profile, two tabs), the second hosts **one**
WebContents (different profile). Three panes in one window.

The two same-profile panes should show the **same** localStorage identity (they
share a `BrowserContext` with the same storage). The third pane should show a
**different** identity.

- XPC service name: `com.termsurf.three-profiles`
- Window: three panes side by side (2400x600)
- Profile servers: 2
- WebContents per server: 2 (profile-a), 1 (profile-b)
- Session IDs: `profile-a1`, `profile-a2`, `profile-b`
- User data dirs: `~/.config/termsurf/poc/profile-a`,
  `~/.config/termsurf/poc/profile-b`

## Success criteria

All three apps:

1. Build with `swift build` (+ `make` for Metal shaders).
2. Render at ~60fps sustained.
3. No Dock icon for Chromium Profile Server processes.
4. Correct profile isolation (same-profile panes share identity, different
   profiles have different identities).
5. For three-profiles: two panes from the same profile show the same
   localStorage identity, confirming they share the same `BrowserContext`.

## Experiments

### Experiment 1: One profile — port box-demo and build one-pane compositor

#### Goal

Port the box-demo test page from ts4 to ts5. Build a one-pane Swift compositor
(`ts5/one-profile/`) that receives IOSurface frames from a single Chromium
Profile Server process and renders them in a Metal window. This establishes the
ts5 test infrastructure and validates the simplest case.

#### Branch

No Chromium changes — this experiment only adds files to the main repo.

#### Changes

##### `ts5/box-demo/` — Copy from ts4

Copy `ts4/box-demo/` to `ts5/box-demo/` as-is:

- `server.ts` — Bun HTTP server on port 9407
- `public/index.html` — Spinning blue square with localStorage identity and FPS

No modifications needed.

##### `ts5/one-profile/` — New Swift app

Adapt `ts4/two-profiles-swift/` into a single-pane compositor:

- `Package.swift` — SwiftPM manifest, target name `OneProfile`
- `Sources/OneProfile/main.swift` — XPC listener, Metal pipeline, rendering
- `Sources/OneProfile/Shaders.metal` — Vertex + fragment shaders (unchanged)
- `com.termsurf.one-profile.plist` — Launchd agent definition
- `Makefile` — Compile Metal shaders + `swift build`

Changes from the ts4 two-profiles-swift source:

1. **One pane, not two.** Remove the left/right pane split. The single pane
   fills the entire window. Remove the `Pane` enum, the `paneForSession()`
   mapping, and the dual-viewport rendering logic.
2. **Window size.** 800x600 (single pane) instead of 1600x600 (two panes).
3. **XPC service name.** `com.termsurf.one-profile`.
4. **Target name.** `OneProfile` instead of `Receiver`.
5. **Log path.** `~/dev/termsurf/logs/one-profile.log`.
6. **Single texture.** One `gCurrentTexture` instead of an array of two.

#### Build and Run

```bash
# 1. Start test page server
cd ts5/box-demo && bun run server.ts &

# 2. Build one-profile compositor
cd ts5/one-profile && make

# 3. Register as launchd service
launchctl bootstrap gui/$(id -u) \
  ~/dev/termsurf/ts5/one-profile/com.termsurf.one-profile.plist

# 4. Start one Chromium Profile Server
cd chromium/src
out/Default/Chromium\ Profile\ Server.app/Contents/MacOS/Chromium\ Profile\ Server \
  --hidden \
  --xpc-service=com.termsurf.one-profile \
  --session-id=profile-a \
  --user-data-dir=$HOME/.config/termsurf/poc/profile-a \
  http://localhost:9407
```

#### Pass Criteria

1. Box-demo server runs on port 9407.
2. One-profile compositor builds with `make` (shaders + `swift build`).
3. Compositor window shows the spinning blue square at ~60fps.
4. No Dock icon for the Chromium Profile Server process.
5. localStorage identity string is visible in the rendered page.

#### Result: Pass

Build: `make` compiled Metal shaders and Swift app with zero errors (one warning
about `.metallib` file, fixed by adding it to the exclude list in
`Package.swift`).

Compositor log (receiver side):

```
[OneProfile] 60 frames (59.0 fps) | IOSurface 1600x1200
[OneProfile] 60 frames (60.0 fps) | IOSurface 1600x1200
[OneProfile] 61 frames (60.0 fps) | IOSurface 1600x1200
[OneProfile] 60 frames (59.7 fps) | IOSurface 1600x1200
[OneProfile] 61 frames (60.3 fps) | IOSurface 1600x1200
```

Profile server log (sender side):

```
[ShellVideoConsumer] Attached to FrameSinkId FrameSinkId(5, 3), starting capture
[ShellVideoConsumer] 62 frames in 1.00931s (61.4283 fps) | IOSurface 1600x1200
[ShellVideoConsumer] 60 frames in 1.01486s (59.1215 fps) | IOSurface 1600x1200
[ShellVideoConsumer] 61 frames in 1.01645s (60.0127 fps) | IOSurface 1600x1200
[ShellVideoConsumer] 61 frames in 1.01622s (60.0261 fps) | IOSurface 1600x1200
```

60fps on both sides. No Dock icon. The spinning blue square and localStorage
identity rendered correctly in the single-pane compositor window.

#### Conclusion

The ts5 test infrastructure is established. Box-demo is ported, and
`ts5/one-profile/` is a working single-pane Swift compositor (~240 lines)
adapted from the ts4 two-profiles-swift source. The one-profile baseline
validates the full pipeline: Chromium Profile Server → FrameSinkVideoCapturer →
IOSurface → XPC Mach port → Metal texture → CADisplayLink rendering at 60fps.

### Experiment 2: Two profiles — port the two-pane compositor

#### Goal

Port the ts4 two-profiles-swift compositor to `ts5/two-profiles/`. Two Chromium
Profile Server processes, each with a different `--user-data-dir`, each hosting
one WebContents. Two panes side by side in one window. The two panes should show
different localStorage identities, proving profile isolation.

This is the Issue 414/501 case, re-validated with the ts5 test infrastructure.

#### Branch

No Chromium changes — this experiment only adds files to the main repo.

#### Changes

##### `ts5/two-profiles/` — New Swift app

Port `ts4/two-profiles-swift/` with naming updates:

- `Package.swift` — SwiftPM manifest, target name `TwoProfiles`
- `Sources/TwoProfiles/main.swift` — XPC listener, Metal pipeline, two-pane
  rendering
- `Sources/TwoProfiles/Shaders.metal` — Vertex + fragment shaders (copy from
  one-profile)
- `com.termsurf.two-profiles.plist` — Launchd agent definition
- `Makefile` — Compile Metal shaders + `swift build`

Changes from the ts4 two-profiles-swift source:

1. **XPC service name.** `com.termsurf.two-profiles` instead of
   `com.termsurf.two-profiles-swift`.
2. **Target name.** `TwoProfiles` instead of `Receiver`.
3. **Log prefix.** `[TwoProfiles]` instead of `[Receiver]`.
4. **Log path.** `~/dev/termsurf/logs/two-profiles.log`.
5. **Window title.** `Two Profiles`.
6. **Exclude metallib.** Add `shaders.metallib` to the Package.swift exclude
   list (lesson from Experiment 1).
7. **Binary path in plist.** Points to
   `ts5/two-profiles/.build/debug/TwoProfiles`.

The two-pane rendering logic (left/right viewports, `Pane` enum,
`paneForSession()` mapping) carries over unchanged from ts4.

#### Build and Run

```bash
# 1. Start test page server (if not already running)
cd ts5/box-demo && bun run server.ts &

# 2. Build two-profiles compositor
cd ts5/two-profiles && make

# 3. Register as launchd service
launchctl bootstrap gui/$(id -u) \
  ~/dev/termsurf/ts5/two-profiles/com.termsurf.two-profiles.plist

# 4. Start two Chromium Profile Servers
cd chromium/src
out/Default/Chromium\ Profile\ Server.app/Contents/MacOS/Chromium\ Profile\ Server \
  --hidden \
  --xpc-service=com.termsurf.two-profiles \
  --session-id=profile-a \
  --user-data-dir=$HOME/.config/termsurf/poc/profile-a \
  http://localhost:9407 &

out/Default/Chromium\ Profile\ Server.app/Contents/MacOS/Chromium\ Profile\ Server \
  --hidden \
  --xpc-service=com.termsurf.two-profiles \
  --session-id=profile-b \
  --user-data-dir=$HOME/.config/termsurf/poc/profile-b \
  http://localhost:9407
```

#### Pass Criteria

1. Two-profiles compositor builds with `make` (shaders + `swift build`).
2. Compositor window shows two side-by-side panes, each with a spinning blue
   square at ~60fps.
3. The two panes show **different** localStorage identity strings (profile
   isolation).
4. No Dock icon for either Chromium Profile Server process.
5. Both profile servers log ~60fps on the sender side.

#### Result: Pass

Build: `make` compiled Metal shaders and Swift app with zero errors.

Compositor log (receiver side):

```
[TwoProfiles] L: 60 (60.0 fps) R: 60 (60.0 fps) | IOSurface 1600x1200
[TwoProfiles] L: 61 (60.0 fps) R: 60 (59.0 fps) | IOSurface 1600x1200
[TwoProfiles] L: 60 (60.0 fps) R: 61 (61.0 fps) | IOSurface 1600x1200
[TwoProfiles] L: 60 (60.0 fps) R: 61 (60.0 fps) | IOSurface 1600x1200
[TwoProfiles] L: 60 (59.7 fps) R: 60 (59.7 fps) | IOSurface 1600x1200
```

Profile server A log (sender side):

```
[ShellVideoConsumer] 61 frames in 1.01659s (60.0046 fps) | IOSurface 1600x1200
[ShellVideoConsumer] 61 frames in 1.01691s (59.9855 fps) | IOSurface 1600x1200
[ShellVideoConsumer] 61 frames in 1.01829s (59.9042 fps) | IOSurface 1600x1200
```

Profile server B log (sender side):

```
[ShellVideoConsumer] 61 frames in 1.01659s (60.0044 fps) | IOSurface 1600x1200
[ShellVideoConsumer] 60 frames in 1.00022s (59.9866 fps) | IOSurface 1600x1200
[ShellVideoConsumer] 60 frames in 1.00021s (59.9873 fps) | IOSurface 1600x1200
```

60fps on all three streams (compositor left, compositor right, both senders). No
Dock icons. Two side-by-side panes with different localStorage identities,
confirming profile isolation.

#### Conclusion

`ts5/two-profiles/` is a working two-pane Swift compositor (~280 lines) ported
from ts4. The Issue 414/501 two-profiles case is re-validated with the ts5 test
infrastructure. Both profile servers deliver independent IOSurface streams at
60fps, composited into a single Metal window with left/right viewports.

### Experiment 3: Three panes — dynamic XPC tab creation

#### Goal

Validate that a single Chromium Profile Server process can dynamically create
**multiple** WebContents from the same `BrowserContext`, each with its own
`FrameSinkVideoCapturer` delivering an independent IOSurface stream. This is the
multi-tab case — the core question of Issue 503.

This experiment also reverses the XPC connection direction to match the target
architecture. The profile server becomes an XPC Mach service (listener). The
compositor becomes the XPC client, opening connections on demand. Each
connection = one tab. Connection close = tab close.

Two profile server processes:

- **Profile server A** (`com.termsurf.cps.profile-a`): Mach service listening
  for connections. The compositor opens **two** connections to it, each
  triggering the creation of a WebContents + capturer.
- **Profile server B** (`com.termsurf.cps.profile-b`): Mach service listening
  for connections. The compositor opens **one** connection.

The compositor renders three panes. The left and center panes (profile-a) should
show the **same** localStorage identity. The right pane (profile-b) should show
a **different** identity.

#### Connection direction reversal

In Experiments 1 and 2, the compositor was the Mach service (listener) and
profile servers connected to it as clients. The profile server used
`--xpc-service` and `--session-id` to tag frames. This made the number of tabs
static — baked into command-line arguments at launch.

In Experiment 3, the roles reverse:

```
Before (Experiments 1–2):
  Compositor (Mach service, listener) ← Profile server (client)
  Profile server decides what to send at startup.

After (Experiment 3):
  Profile server (Mach service, listener) ← Compositor (client)
  Compositor decides when to open/close tabs dynamically.
```

The compositor drives the lifecycle. Opening a connection creates a tab. Closing
a connection destroys it. The profile server is a passive service that responds
to incoming connections.

#### XPC protocol

Compositor → Profile server (per connection):

```
{"action": "navigate", "url": "http://localhost:9407"}
```

Profile server → Compositor (per connection, 60fps):

```
{"action": "display_surface", "iosurface_port": <mach_port>, "width": N, "height": N}
```

No `session_id` field — the connection itself is the identity. The compositor
knows which pane each connection maps to because it created the connection for
that pane.

#### Branch

`146.0.7650.0-issue-503`, branched off `146.0.7650.0-issue-502`.

#### Chromium changes

The profile server currently creates one WebContents at startup and connects to
an external Mach service as an XPC client. It needs to become an XPC Mach
service listener that dynamically creates WebContents when connections arrive.

##### `content/chromium_profile_server/common/shell_switches.h`

Add one new switch:

```cpp
// Mach service name to listen on for incoming tab connections.
inline constexpr char kServiceName[] = "service-name";
```

The existing `--xpc-service` and `--session-id` switches remain for backward
compatibility with Experiments 1–2 but are not used in this experiment.

##### `content/chromium_profile_server/browser/shell_browser_main_parts.h`

Replace the single `video_consumer_` with per-connection state:

```cpp
struct TabState {
  raw_ptr<Shell> shell;
  std::unique_ptr<ShellVideoConsumer> video_consumer;
};

// Per-connection tab state, keyed by xpc_connection_t.
std::vector<std::pair<xpc_connection_t, std::unique_ptr<TabState>>> tabs_;

// XPC listener (retained to prevent ARC release).
xpc_connection_t xpc_listener_ = nullptr;
```

Add methods:

```cpp
void StartXPCListener(const std::string& service_name);
void HandleNewConnection(xpc_connection_t peer);
void CreateTab(xpc_connection_t peer, const GURL& url);
void CloseTab(xpc_connection_t peer);
```

##### `content/chromium_profile_server/browser/shell_browser_main_parts.cc`

`InitializeMessageLoopContext()` checks for `--service-name`. If present, it
starts the XPC listener instead of creating a WebContents at startup. If absent,
the existing single-tab startup path runs unchanged (backward compatible).

`StartXPCListener()` calls `xpc_connection_create_mach_service()` with the
listener flag. Each incoming peer gets an event handler that watches for
`"navigate"` messages and connection errors.

`HandleNewConnection()` sets up per-peer event handling. On a `"navigate"`
dictionary, it extracts the URL and PostTasks `CreateTab()` to the UI thread. On
XPC error (connection close), it PostTasks `CloseTab()` to the UI thread.

`CreateTab()` creates a Shell + WebContents via `Shell::CreateNewWindow()`,
creates a ShellVideoConsumer, hands it the peer connection via
`SetConnection()`, and calls `ObserveContents()`. Stores the TabState.

`CloseTab()` finds the TabState for the peer, stops the capturer, closes the
Shell, and removes the entry.

`PostMainMessageLoopRun()` clears `tabs_` and releases the listener.

##### `content/chromium_profile_server/browser/shell_video_consumer.h`

Add a new method alongside the existing `ConnectToService()`:

```cpp
// Use an existing XPC connection (for listener mode — the connection was
// accepted by the profile server's XPC listener, not created by us).
void SetConnection(xpc_connection_t conn);
```

##### `content/chromium_profile_server/browser/shell_video_consumer.cc`

`SetConnection()` stores the connection (with `xpc_retain`) without creating a
new one. `OnFrameCaptured()` sends frames on whichever connection is set — the
existing code works unchanged since it just uses `xpc_connection_`.

The `session_id_` field is no longer set in listener mode. Frame messages omit
it. The compositor identifies panes by connection, not by session-id.

#### Swift compositor: `ts5/three-profiles/`

A three-pane compositor. Unlike Experiments 1–2, this is a **regular app** (not
a launchd Mach service). It opens XPC client connections to profile servers.

- `Package.swift` — SwiftPM manifest, target name `ThreeProfiles`
- `Sources/ThreeProfiles/main.swift` — XPC client connections, Metal pipeline,
  three-pane rendering
- `Sources/ThreeProfiles/Shaders.metal` — Vertex + fragment shaders (copy)
- `Makefile` — Compile Metal shaders + `swift build`

No launchd plist for the compositor — it's launched directly.

##### Connection setup

The compositor opens three XPC client connections, each bound to a pane:

```swift
func connectToProfile(serviceName: String, pane: Pane, url: String) {
    let conn = xpc_connection_create_mach_service(serviceName, queue, 0)
    xpc_connection_set_event_handler(conn) { event in
        if xpc_get_type(event) == XPC_TYPE_DICTIONARY {
            handleFrame(event, pane: pane)
        }
    }
    xpc_connection_resume(conn)

    // Send navigate command.
    let msg = xpc_dictionary_create(nil, nil, 0)
    xpc_dictionary_set_string(msg, "action", "navigate")
    xpc_dictionary_set_string(msg, "url", url)
    xpc_connection_send_message(conn, msg)
}

// Two connections to profile-a (two tabs, same profile):
connectToProfile("com.termsurf.cps.profile-a", pane: .left, url: boxDemoURL)
connectToProfile("com.termsurf.cps.profile-a", pane: .center, url: boxDemoURL)

// One connection to profile-b (one tab, different profile):
connectToProfile("com.termsurf.cps.profile-b", pane: .right, url: boxDemoURL)
```

Each closure captures its pane, so frame routing is implicit — no session-id
parsing needed.

##### Rendering

Three panes side by side. `Pane` enum has `.left`, `.center`, `.right`.
Viewports split into thirds. Otherwise identical to two-profiles.

- Window: 2400x600
- Three textures, three surfaces, three frame counters
- FPS logging: `L: N (fps) C: N (fps) R: N (fps)`

#### Launchd plists for profile servers

Each profile server registers as a Mach service via a launchd plist. These live
in `ts5/three-profiles/` alongside the compositor.

##### `com.termsurf.cps.profile-a.plist`

```xml
<key>Label</key>
<string>com.termsurf.cps.profile-a</string>
<key>MachServices</key>
<dict>
    <key>com.termsurf.cps.profile-a</key>
    <true/>
</dict>
<key>ProgramArguments</key>
<array>
    <string>.../Chromium Profile Server</string>
    <string>--service-name=com.termsurf.cps.profile-a</string>
    <string>--user-data-dir=~/.config/termsurf/poc/profile-a</string>
    <string>--hidden</string>
</array>
```

##### `com.termsurf.cps.profile-b.plist`

Same structure, different service name and data dir.

When the compositor opens a connection to `com.termsurf.cps.profile-a`, launchd
launches the profile server process. Chromium initializes, the XPC listener
starts in `InitializeMessageLoopContext()`, and queued connections are
delivered.

#### Build and Run

```bash
# 1. Start test page server (if not already running)
cd ts5/box-demo && bun run server.ts &

# 2. Build Chromium with dynamic tab support
cd chromium/src
export PATH="$HOME/dev/termsurf/chromium/depot_tools:$PATH"
autoninja -C out/Default one_profile

# 3. Build three-profiles compositor
cd ts5/three-profiles && make

# 4. Register profile servers as launchd services
launchctl bootstrap gui/$(id -u) \
  ~/dev/termsurf/ts5/three-profiles/com.termsurf.cps.profile-a.plist
launchctl bootstrap gui/$(id -u) \
  ~/dev/termsurf/ts5/three-profiles/com.termsurf.cps.profile-b.plist

# 5. Launch compositor (opens connections, triggers profile server launch)
ts5/three-profiles/.build/debug/ThreeProfiles
```

No manual profile server launch needed — launchd starts them on first connection
from the compositor.

#### Pass Criteria

1. Chromium builds with dynamic tab changes (autoninja).
2. Three-profiles compositor builds with `make`.
3. Running the compositor triggers launchd to launch both profile servers.
4. Compositor window shows three side-by-side panes at ~60fps.
5. Left and center panes show the **same** localStorage identity (same profile,
   same `BrowserContext`).
6. Right pane shows a **different** localStorage identity (different profile).
7. No Dock icon for either Chromium Profile Server process.
8. Both profile servers log ~60fps per WebContents on the sender side.
9. Closing the compositor window closes all XPC connections, causing both
   profile servers to tear down their WebContents.
