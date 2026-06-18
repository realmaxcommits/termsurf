+++
reviewer = "adversarial-review"
+++

# Experiment 1: Restore Browse-Mode Command Navigation Forwarding

## Description

Restore the missing Ghostboard-legacy behavior that let browser-owned
Command-key shortcuts reach Chromium while a pane is in browse mode.

The current browser-side path already handles `Cmd+[` as Back if a `KeyEvent`
reaches Roamium/Chromium. Swift maps `[` to `0xDB`; Zig forwards only when
`snapshotBrowserInput(pane_id, true)` proves the pane is browsing with a real
browser tab and attached browser file descriptor; Chromium maps
`Meta + VKEY_OEM_4` to `GoBack()`.

The missing piece is AppKit routing. `performKeyEquivalent(with:)` can receive
Command-key events before `keyDown(with:)`. Current Ghostboard only forwards
events to `keyDown` when Ghostty recognizes them as bindings or when AppKit
redispatches a matching timestamp. Plain `Cmd+[` can be swallowed before
`keyDown`, so the existing TermSurf forwarding path never runs.

This experiment restores a narrow browser-navigation bypass at the top of
`performKeyEquivalent(with:)`: for key-down events on browser navigation
shortcuts, call the existing `forwardTermSurfKeyDown(event)` path before Ghostty
binding/menu fallback. That call is the safety gate. It returns true only if the
current TermSurf pane/browser state accepts the event; otherwise the method
continues through normal Ghostty/AppKit handling. This is intentionally narrower
than legacy's broad `self.keyDown(with:)` browse-mode bypass because the current
direct forwarding path can prove whether the browser accepted the event before
`performKeyEquivalent` consumes it.

## Changes

- `ghostboard/macos/Sources/Ghostty/Surface View/SurfaceView_AppKit.swift`
  - Add a private helper that recognizes browser-owned navigation shortcuts:
    `Cmd+[`, `Cmd+]`, and `Cmd+R`.
  - In `performKeyEquivalent(with:)`, after the existing focused guard and
    before `surface.keyIsBinding`, call `forwardTermSurfKeyDown(event)` only for
    those shortcuts.
  - Return `true` only if forwarding succeeds, so non-browse mode, unattached
    browsers, or non-browser panes fall through to the current behavior.
  - Add trace logging for forwarded and rejected browser key-equivalent attempts
    so the smoke can distinguish AppKit swallowing from browser forwarding.
- `scripts/ghostboard-geometry-matrix.sh`
  - Add a narrow `browser-command-navigation` scenario that opens a page, enters
    browse mode, navigates to a second URL, sends `Cmd+[` through real macOS
    keyboard injection, and verifies:
    - Ghostboard logs `perform_key_equivalent_browser_forwarded`;
    - Zig logs `KeyEvent` with `windows_key_code=219` and `modifiers=8`;
    - Roamium/Chromium reports the original URL after Back.
  - Send `Cmd+[` once before browse mode and verify no
    `perform_key_equivalent_browser_forwarded` log and no browser `KeyEvent` are
    produced for that pre-browse attempt.
  - Include a small `Cmd+]` forward check if it can reuse the same fixture
    cheaply; otherwise leave Forward/Reload for the existing browser-state and
    navigation smoke coverage.
- `docs/keybindings.md`
  - Update the Browser navigation notes only if the implementation details
    change from the current description.

## Verification

Pass criteria:

1. `swiftc`/build verification for the edited macOS Swift source succeeds as
   part of the Ghostboard build.
2. `./scripts/build.sh ghostboard` succeeds.
3. `./scripts/ghostboard-geometry-matrix.sh browser-command-navigation` succeeds
   and its logs prove `Cmd+[` reached the browser as:
   - `perform_key_equivalent_browser_forwarded`;
   - `KeyEvent ... windows_key_code=219 ... modifiers=8`;
   - browser URL/title state returned to the prior page.
4. The same scenario proves a pre-browse `Cmd+[` attempt does not emit
   `perform_key_equivalent_browser_forwarded` or a browser `KeyEvent`.
5. `./scripts/ghostboard-geometry-matrix.sh copy-current-url-smoke` succeeds,
   proving Control-mode `Cmd+C` still copies the URL and Browse-mode `Cmd+C`
   does not run the Ghostboard URL-copy action.
6. `git diff --check` reports no whitespace errors.

Fail criteria:

- `Cmd+[` still has no `KeyEvent` evidence in browse mode.
- `performKeyEquivalent` consumes `Cmd+[` outside browse mode.
- Control-mode `Cmd+C` regresses.
- The implementation forwards broad Command-key traffic without a browser-owned
  shortcut check or without requiring `forwardTermSurfKeyDown` to accept the
  event.

## Design Review

Reviewed by a fresh-context Codex adversarial subagent.

Initial verdict: **Changes Required**.

- Required: the original design called `forwardTermSurfKeyDown(event)` directly
  but still required the existing `key_down_forwarded` log, which is emitted
  only from `keyDown`. Fixed by making the design consistently require the new
  `perform_key_equivalent_browser_forwarded` log for direct
  `performKeyEquivalent` forwarding.
- Optional: outside-browse non-consumption was listed only as a fail criterion.
  Fixed by adding a concrete pre-browse `Cmd+[` negative check.

Re-review verdict: **Approved**. The reviewer confirmed both findings were
resolved and found no new required issues.
