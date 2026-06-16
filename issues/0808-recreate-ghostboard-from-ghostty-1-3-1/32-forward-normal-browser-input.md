# Experiment 32: Forward Normal Browser Input

## Description

Experiment 31 made real Roamium content visible inside the normal Ghostboard
terminal pane. The next ordinary-browsing parity gap is input. A visible browser
that cannot receive keyboard, mouse, and scroll events is not usable as a
browser.

This experiment will implement normal-pane browser input forwarding only:

- keyboard events while the pane is in browsing mode;
- mouse down/up events inside the visible browser overlay rectangle;
- mouse move events inside the visible browser overlay rectangle;
- scroll events inside the visible browser overlay rectangle.

The experiment will use the current TermSurf protobuf messages: `KeyEvent`,
`MouseEvent`, `MouseMove`, and `ScrollEvent`. It will send those messages from
Ghostboard to the already-attached Roamium browser server using the normal
pane's `tab_id`.

This experiment intentionally does not implement DevTools input forwarding,
browser state UI updates, JavaScript dialogs, HTTP auth, downloads, bookmarks,
history, or Roamium shutdown crash cleanup.

## Changes

Expected implementation files:

- `ghostboard/src/apprt/termsurf.zig`
  - add bridge-callable functions that accept normalized input from AppKit and
    send the corresponding TermSurf protobuf to the browser server;
  - resolve pane id to `PaneState`, require a nonzero normal-tab `tab_id`, and
    require an attached browser server fd;
  - only forward keyboard events when `pane.browsing` is true;
  - forward pointer events only when AppKit reports a point inside the overlay
    rectangle;
  - log every forwarded input message with pane id, tab id, and key/mouse
    details sufficient for runtime verification.
- `ghostboard/macos/Sources/Ghostty/Surface View/SurfaceView_AppKit.swift`
  - track the current overlay rectangle from Experiment 31;
  - add hit testing that converts an `NSEvent` location into overlay-relative
    coordinates;
  - intercept `keyDown`, `keyUp`, and repeat events while browsing mode is
    active and send them to Zig instead of Ghostty's terminal input path;
  - intercept mouse down/up, drag/move, and scroll events inside the overlay and
    send them to Zig with overlay-relative coordinates.
- `ghostboard/macos/Sources/App/macOS/AppDelegate+TermSurf.swift`
  - if needed, add C-callable bridge functions for input events, following the
    existing overlay/open-split bridge pattern.

Possible supporting file:

- `ghostboard/macos/Sources/Ghostty/Surface View/SurfaceView.swift`
  - only if the SwiftUI wrapper is the correct place to observe browsing-mode
    changes or focus state.

No changes will be made to `webtui`, `roamium`, Chromium,
`proto/termsurf.proto`, config paths, branding, CLI install behavior, DevTools
overlay presentation, browser state UI updates, or browser shutdown behavior in
this experiment.

## Verification

Pass criteria:

- `cargo build -p webtui` passes, with command, cwd, and exit status recorded in
  a log.
- `./scripts/build.sh roamium` passes, with command, cwd, and exit status
  recorded in a log.
- If Zig code is modified, run
  `zig fmt src/apprt/termsurf.zig src/main_c.zig src/build/SharedDeps.zig`
  inside `ghostboard/`, with command, cwd, and exit status recorded in a log.
- If Swift code is modified, run SwiftLint on touched Swift files, with command,
  cwd, and exit status recorded in a log. If SwiftLint reports warnings, either
  fix them or record why a targeted suppression is necessary.
- The native GhosttyKit framework build passes:
  `zig build -Demit-xcframework=true -Dxcframework-target=native -Demit-macos-app=false`,
  with command, cwd, and exit status recorded in a log.
- The macOS app build passes:
  `macos/build.nu --scheme Ghostty --configuration Debug --action build`, with
  command, cwd, and exit status recorded in a log.
- Runtime harness launches `TermSurf.app` with `GHOSTTY_LOG=stderr` and a
  temporary config whose command runs:

  ```text
  /Users/astrohacker/dev/termsurf/target/debug/web --browser /Users/astrohacker/dev/termsurf/chromium/src/out/Default/roamium https://example.com
  ```

- Runtime logs still prove the Experiment 31 lifecycle:
  - Chromium-output Roamium is used;
  - `ServerRegister`, `CreateTab`, `TabReady`, `BrowserReady`, direct browser
    connection, `CaContext`, `PresentOverlay`, and AppKit overlay presentation
    all occur;
  - visible `Example Domain` content is still captured in a screenshot.
- Runtime input proof:
  - switch the normal `webtui` pane into browsing mode using the real UI path,
    not by directly sending protocol messages from the harness;
  - send keyboard down, repeat, and up events that the browser can observably
    receive;
  - prove Ghostboard sent `KeyEvent(type=down)`, `KeyEvent(type=repeat)`, and
    `KeyEvent(type=up)` with a nonzero tab id;
  - prove the `windows_key_code` field uses the expected Chromium/Windows
    virtual-key value, not the raw macOS `NSEvent.keyCode`. For example, the `a`
    key must produce `0x41` / `65`;
  - prove Roamium received or dispatched those key events, using Roamium-side
    logs if available. If Roamium does not currently log `KeyEvent` dispatch,
    add Ghostboard-side proof of the exact serialized fields and use a visible
    browser effect as the receive proof;
  - preferred visible keyboard proof: navigate through normal `webtui`/browser
    behavior to a local test page with an input element, type a printable
    character, and prove typed text appears in the browser screenshot;
  - send mouse down/up inside the overlay rectangle and prove Ghostboard
    forwarded `MouseEvent` with overlay-relative coordinates and nonzero tab id;
  - prove Roamium received or dispatched that `MouseEvent`. Roamium currently
    logs `mouse-event ... ffi=ts_forward_mouse_event` from
    `roamium/src/dispatch.rs`, so the runtime log must include that line or an
    equivalent browser-side proof;
  - send mouse move inside the overlay rectangle and prove Ghostboard forwarded
    `MouseMove`;
  - prove Roamium received or dispatched that `MouseMove`. Roamium currently
    logs `mouse-move ... ffi=ts_forward_mouse_move`;
  - send scroll inside the overlay rectangle and prove Ghostboard forwarded
    `ScrollEvent`;
  - prove Roamium received or dispatched that `ScrollEvent`. Roamium currently
    logs `scroll-event ... ffi=ts_forward_scroll_event`;
  - send a control event outside the overlay rectangle and prove it is not
    forwarded to Roamium as browser input.
- `web last` still returns the normal Roamium tab after input forwarding.
- Runtime cleanup clears the overlay and leaves no stale matching
  `TermSurf.app/Contents/MacOS/termsurf`, `target/debug/web`, or
  `chromium/src/out/Default/roamium` processes, and removes the GUI socket.
- `git diff --check` is clean.
- `git diff --name-only` or `git diff --stat` is recorded, and the experiment
  fails if the implementation changes any forbidden path: `webtui/`, `roamium/`,
  `chromium/`, or `proto/termsurf.proto`.

Fail criteria:

- Input forwarding is implemented by modifying `webtui`, `roamium`, Chromium, or
  `proto/termsurf.proto`.
- Keyboard events are forwarded while the pane is not in browsing mode.
- Mouse or scroll events outside the overlay rectangle are forwarded to Roamium.
- Forwarded input uses pane ids without resolving the nonzero browser `tab_id`.
- Forwarded coordinates are terminal/window coordinates rather than
  overlay-relative coordinates.
- The implementation regresses visible overlay presentation or the Experiment 31
  normal Roamium lifecycle.
- The experiment adds DevTools input forwarding, browser state UI updates,
  JavaScript dialog handling, HTTP auth handling, browser shutdown fixes,
  Chromium changes, `webtui` changes, `roamium` changes, or protobuf schema
  changes.

## Design Review

A fresh-context adversarial Codex subagent reviewed the Experiment 32 design and
returned **CHANGES REQUIRED** with two required findings:

- mouse, move, and scroll verification could pass on Ghostboard-side attempted
  forwarding without proving Roamium received or handled the input;
- keyboard verification only required one keyboard event even though the design
  scoped `keyDown`, `keyUp`, and repeat forwarding, and it did not require
  proving that `windows_key_code` uses Chromium/Windows virtual-key values
  rather than raw macOS key codes.

Both findings were accepted. The design now requires Roamium-side receive or
dispatch proof for mouse, move, and scroll input. It also requires separate
keyboard down, repeat, and up verification, a nonzero tab id, and an expected
Windows virtual-key value such as `0x41` / `65` for the `a` key.

The same reviewer re-reviewed the updated design and returned **APPROVED**. The
reviewer confirmed that the prior mouse, move, scroll, and keyboard verification
findings were resolved and found no new required changes.
