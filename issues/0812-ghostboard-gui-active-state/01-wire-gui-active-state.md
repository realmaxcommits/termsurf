# Experiment 1: Wire GUI Active State Into Roamium

## Description

Ghostboard should tell Roamium when the visible macOS GUI activates and
deactivates. Roamium already consumes `SetGuiActive`, and Wezboard already sends
that message from application activation events:

- deactivation sends `active=false`, `reason="gui_deactivated"`, `tab_id=0` to
  every connected browser server so every tab in each browser process learns the
  GUI is inactive;
- activation sends `active=true`, `reason="gui_activated"` only to the focused
  browser tab so Chromium restores focus/activity for the tab the user returned
  to.

Ghostboard currently has AppKit activation hooks and TermSurf browser state, but
`SetGuiActive` only appears as a protobuf type/name in Ghostboard. This
experiment will add the smallest Ghostboard-side equivalent of Wezboard's
`set_gui_active` behavior.

## Changes

Planned source changes:

- `ghostboard/src/apprt/termsurf.zig`
  - Add a public function such as `guiActiveChanged(active: bool)` that can be
    called from the macOS delegate bridge.
  - Snapshot send targets while holding `state_mutex`, then send after releasing
    the mutex.
  - For `active=false`, target every live attached browser server with
    `tab_id=0`, `active=false`, and `reason="gui_deactivated"`.
  - For `active=true`, target only the currently focused browser pane whose
    `browsing` state is true, whose `tab_id` is nonzero, and whose server has an
    attached browser fd. Send `active=true`, `reason="gui_activated"` for that
    tab id.
  - Log skipped activation sends, skipped deactivation sends, and successful
    sends with reason, tab id, pane id when applicable, and target count.
  - Encode and send protobuf `SetGuiActive` using the existing `sendProtobuf`
    framing helper.
- `ghostboard/src/main_c.zig`
  - Export a C bridge such as `termsurf_gui_active_changed(int active)` that
    calls the Zig TermSurf function.
- `ghostboard/macos/Sources/App/macOS/ghostty-bridging-header.h`
  - Declare the new C bridge for Swift.
- `ghostboard/macos/Sources/App/macOS/AppDelegate.swift`
  - Call the bridge from the existing `applicationDidBecomeActive` and
    `applicationDidResignActive` delegate hooks.
  - Preserve existing Ghostty launch/focus behavior.

Planned issue-doc changes:

- Add the design review result before implementation.
- After verification, append `## Result` and `## Conclusion` here and update the
  Issue 812 README status line.

## Verification

Baseline check before implementation:

1. Build current Ghostboard before source edits:
   - `cd ghostboard && zig build -Demit-macos-app=false`
   - `cd ghostboard && macos/build.nu --scheme Ghostty --configuration Debug --action build`
2. Launch Ghostboard with local Roamium through the existing geometry harness.
3. Force an app deactivate/reactivate cycle using the same AppKit/System Events
   automation already used by the Ghostboard geometry harness.
4. Record the current failure evidence:
   - no Ghostboard `SetGuiActive` send log exists;
   - Roamium/Chromium trace logs do not show `set_gui_active` for Ghostboard's
     deactivate/reactivate cycle.

Static and build checks:

1. `prettier --write --prose-wrap always --print-width 80 issues/0812-ghostboard-gui-active-state/README.md issues/0812-ghostboard-gui-active-state/01-wire-gui-active-state.md`
2. `zig fmt ghostboard/src/apprt/termsurf.zig ghostboard/src/main_c.zig`
3. `cd ghostboard && zig build -Demit-macos-app=false`
4. `cd ghostboard && zig build test`
5. `cd ghostboard && swiftlint lint --strict --fix`
6. `cd ghostboard && macos/build.nu --scheme Ghostty --configuration Debug --action build`
7. `cd ghostboard && macos/build.nu --scheme Ghostty --configuration Debug --action test`
8. `git diff --check`

If a local tool required by these checks is unavailable in the VM, record the
exact command failure in the result instead of silently skipping it, and rely on
the closest build-phase coverage only as secondary evidence.

Runtime checks:

1. Launch Ghostboard with local Roamium and a browser page through the existing
   TermSurf/web flow.
2. Confirm a browser tab is ready, focused, and receiving normal keyboard or
   mouse input before activation testing.
3. Deactivate Ghostboard by activating another normal macOS app, then wait for
   logs.
4. Confirm Ghostboard sends `SetGuiActive` with:
   - `active=false`;
   - `reason="gui_deactivated"`;
   - `tab_id=0`;
   - one send target per live attached browser server.
5. Confirm Roamium/Chromium receives the inactive state, either through
   Roamium's `Msg::SetGuiActive` path or the Chromium `ts_set_gui_active` trace
   already emitted by the local Roamium/Chromium build.
6. Reactivate Ghostboard, then wait for logs.
7. Confirm Ghostboard sends `SetGuiActive` with:
   - `active=true`;
   - `reason="gui_activated"`;
   - the focused browser tab id, not `0`.
8. Confirm Roamium/Chromium receives the active state for that tab.
9. After reactivation, prove browser input/focus still works by sending a
   deterministic keyboard or mouse marker to the browser and verifying it
   reaches the same tab/pane.
10. In a two-browser-tab or two-pane run, confirm deactivation broadcasts
    inactive state to all live browser servers while reactivation targets only
    the focused browser tab.

Pass criteria:

- Baseline evidence proves Ghostboard did not send or Roamium/Chromium did not
  receive `SetGuiActive` before this experiment.
- All static/build checks pass.
- App deactivation sends
  `SetGuiActive(active=false, reason="gui_deactivated", tab_id=0)` to every live
  attached browser server.
- App activation sends `SetGuiActive(active=true, reason="gui_activated")` only
  to the focused browser tab.
- Roamium/Chromium logs prove both inactive and active states were received.
- Browser input/focus still works after the deactivate/reactivate cycle.
- Multi-tab or multi-pane verification proves activation does not send stale or
  duplicate active-state messages to an unfocused tab.

Partial criteria:

- Static/build checks pass and Ghostboard send logs are correct, but the local
  Roamium/Chromium build lacks enough trace output to independently prove
  receipt.

Fail criteria:

- Ghostboard still does not send `SetGuiActive`, source does not build,
  deactivate/reactivate sends target the wrong tab/server, duplicate stale
  active messages are emitted, or browser input/focus regresses after
  reactivation.

## Design Review

Fresh-context adversarial review by Codex subagent `Pasteur`:

- **Verdict:** Changes required.
- **Required finding:** The verification plan omitted local Swift lint and
  Zig/macOS unit test commands even though the experiment plans to touch Swift,
  the bridging header, and Zig.
- **Resolution:** Added `zig build test`, `swiftlint lint --strict --fix`, and
  `macos/build.nu --action test` to the static/build checks, plus an explicit
  requirement to record exact unavailable-tool failures instead of silently
  skipping checks.
- **Re-review verdict:** Approved. The reviewer confirmed the missing hygiene
  checks were added and found no new required issues.

## Result

**Result:** Partial

Implemented Ghostboard GUI active-state signaling and verified Roamium receives
the state changes during the one-browser-pane minimize/hide/restore runtime
scenario. The experiment remains partial because the planned broad test targets
did not pass in this VM and the recorded runtime evidence did not cover the
planned two-tab or two-pane stale/duplicate-active case.

Source changes:

- `ghostboard/src/apprt/termsurf.zig`
  - added `guiActiveChanged(active: bool)`;
  - snapshots send targets under `state_mutex` and sends after unlocking;
  - sends `SetGuiActive(active=false, reason="gui_deactivated", tab_id=0)` to
    each live attached browser server on deactivation;
  - sends `SetGuiActive(active=true, reason="gui_activated")` to the focused
    browser tab on activation;
  - records a pending activation when AppKit reports activation before the
    focused browsing pane has a tab/server ready, then flushes it from mode or
    pane-focus updates;
  - logs skipped and successful sends.
- `ghostboard/src/main_c.zig`
  - exported `termsurf_gui_active_changed`.
- `ghostboard/macos/Sources/App/macOS/ghostty-bridging-header.h`
  - declared the new bridge.
- `ghostboard/macos/Sources/App/macOS/AppDelegate.swift`
  - calls the bridge from `applicationDidBecomeActive` and
    `applicationDidResignActive`.
- `roamium/src/dispatch.rs`
  - added trace logging in `Msg::SetGuiActive` so the runtime harness can prove
    Roamium received the message.

Baseline evidence before implementation:

- `cd ghostboard && zig build -Demit-macos-app=false` passed.
- `cd ghostboard && macos/build.nu --scheme Ghostty --configuration Debug --action build`
  passed.
- `scripts/ghostboard-geometry-matrix.sh minimize-hide-restore` passed, but the
  logs showed only existing focus messages:
  - app log:
    `logs/ghostboard-geometry-minimize-hide-restore-app-20260617-193906.log`
  - Roamium trace:
    `logs/ghostboard-geometry-minimize-hide-restore-roamium-20260617-193906.log`
- The baseline logs had `FocusChanged` / `focus-changed` evidence but no
  `SetGuiActive` / `set-gui-active` evidence.

Static and build verification:

- `zig fmt ghostboard/src/apprt/termsurf.zig ghostboard/src/main_c.zig` passed.
- `cargo fmt -- roamium/src/dispatch.rs` passed.
- `cargo check -p roamium` passed.
- `./scripts/build.sh roamium` passed and copied the rebuilt local Roamium
  binary into `chromium/src/out/Default/roamium`.
- `cd ghostboard && zig build -Demit-macos-app=false` passed.
- `cd ghostboard && macos/build.nu --scheme Ghostty --configuration Debug --action build`
  passed after removing stale test bundle artifacts left behind by the failed
  test command:
  - `ghostboard/macos/build/Debug/TermSurf.app/Contents/PlugIns/GhosttyTests.xctest`
  - `ghostboard/macos/build/Debug/GhosttyUITests-Runner.app`
- `git diff --check` passed.

Verification caveats:

- `cd ghostboard && swiftlint lint --strict --fix` failed because the VM shell
  cannot find `swiftlint`:

  ```text
  zsh:1: command not found: swiftlint
  ```

  The macOS Xcode build still ran its SwiftLint build phase.

- `cd ghostboard && macos/build.nu --scheme Ghostty --configuration Debug --action test`
  failed in the existing `GhosttyTests` test target with linker errors. The
  failed result bundle was:
  `/Users/astrohacker/Library/Developer/Xcode/DerivedData/Ghostty-argaxstpsdjkjudxknbvwcoaqdrq/Logs/Test/Test-Ghostty-2026.06.17_19-42-30--0500.xcresult`.

- `cd ghostboard && zig build test` failed in the existing test path. The Zig
  test target first reported an Xcode test dependency failure and then failed
  linking the `ghostty-test` target because several AppKit bridge symbols were
  undefined:

  ```text
  error: undefined symbol: _termsurf_clear_overlay
  error: undefined symbol: _termsurf_open_split
  error: undefined symbol: _termsurf_present_overlay
  error: undefined symbol: _termsurf_set_cursor
  ```

Runtime verification after implementation:

- Final runtime command:
  `scripts/ghostboard-geometry-matrix.sh minimize-hide-restore`
- Result: `PASS: scenario minimize-hide-restore`
- App log:
  `logs/ghostboard-geometry-minimize-hide-restore-app-20260617-194802.log`
- Roamium trace:
  `logs/ghostboard-geometry-minimize-hide-restore-roamium-20260617-194802.log`

Ghostboard send evidence:

```text
SetGuiActive: tab_id=0 active=false reason=gui_deactivated
SetGuiActive: pane_id=E24AE127-9AE3-43F7-AE96-179686971BC1 tab_id=1 active=true reason=gui_activated
SetGuiActive: tab_id=0 active=false reason=gui_deactivated
SetGuiActive: pane_id=E24AE127-9AE3-43F7-AE96-179686971BC1 tab_id=1 active=true reason=gui_activated
```

Roamium receipt and input-preservation evidence:

```text
roamium set-gui-active tab=0 active=false reason=gui_deactivated target_count=1
roamium set-gui-active tab=1 pane=E24AE127-9AE3-43F7-AE96-179686971BC1 active=true reason=gui_activated target_count=1
roamium focus-changed tab=1 pane=E24AE127-9AE3-43F7-AE96-179686971BC1 ffi=ts_set_focus focused=true
roamium key-event tab=1 pane=E24AE127-9AE3-43F7-AE96-179686971BC1 ffi=ts_forward_key_event type=down windows_key_code=65 utf8_len=1 modifiers=0
```

Observed final runtime counts:

```text
app_deactivated=2
app_activated=2
trace_deactivated=2
trace_activated=2
trace_focus_true=2
trace_key=120
```

## Conclusion

Ghostboard now sends GUI activation state to Roamium. Deactivation broadcasts
`SetGuiActive(active=false, tab_id=0)` to the attached browser server,
activation targets the focused browser tab, Roamium trace logs prove receipt,
and keyboard input continues after the deactivate/reactivate cycle.

The implementation is in place and the one-pane runtime behavior is proven, but
Issue 812 should not close yet. The next experiment must either make the broad
test targets pass or explicitly bound them as pre-existing harness failures, and
it must add two-tab or two-pane runtime evidence that active-state messages do
not go stale or duplicate onto an unfocused browser tab.

## Completion Review

Fresh-context adversarial review by Codex subagent `Hooke`:

- **Verdict:** Changes required.
- **Required finding:** The experiment result and README were marked `Pass` even
  though the planned pass criteria required all static/build checks to pass, and
  `macos/build.nu --action test` plus `zig build test` failed.
- **Required finding:** The recorded runtime evidence only covered one
  browser-pane/tab and did not prove the planned two-tab or two-pane
  stale/duplicate-active behavior.
- **Resolution:** Downgraded the experiment result and README status to
  `Partial`, and recorded the remaining verification gaps as required follow-up
  work for the next experiment.
- **Re-review verdict:** Approved. The reviewer confirmed the result and README
  now accurately mark the experiment as `Partial` and explicitly preserve the
  unresolved test-target and two-pane/two-tab verification gaps.
