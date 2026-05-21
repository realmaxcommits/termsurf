+++
status = "open"
opened = "2026-05-21"
+++

# Issue 782: Remaining native popup bugs

## Goal

Fix the native popup bugs that remain after Issue 779 without reopening that
large investigation. Each remaining bug should be isolated, logged, fixed, and
verified one at a time.

## Background

Issue 779 fixed the primary PagePopup y-axis placement bug for date-family
controls. Date, time, date-time, and color controls now appear at the correct y
position in the TermSurf webview overlay.

That work also showed that native widgets are not one unified system in
Chromium:

- date, time, date-time, and color controls use Blink PagePopup widgets;
- `<select>` uses Chromium's AppKit menu path;
- datalist still needs to be isolated because testing was blocked by later popup
  failures.

The remaining failures are separate enough that they should be handled in this
new issue instead of extending Issue 779.

## Remaining Bugs

### PagePopup remains visible after alt-tab

Date, time, date-time, and color popups can remain visible after the user
alt-tabs away from Wezboard. The owning TermSurf window is no longer visible or
active, but the native popup stays on screen.

This is likely a popup lifecycle, owner-window, focus, or deactivation issue. It
should be investigated with logs around window deactivation, PagePopup
visibility, popup widget ownership, and dismissal.

### Select dropdown has the wrong x position

The `<select>` dropdown now has the correct y position, but its x position is
wrong. This path is different from PagePopup:

```text
RenderFrameHostImpl::ShowPopupMenu
PopupMenuHelper::ShowPopupMenu
RenderWidgetHostNSViewBridge::DisplayPopupMenu
WebMenuRunner::runMenuInView
NSPopUpButtonCell
```

Issue 779 confirmed that Chromium logs the select anchor, but AppKit owns the
final menu placement after `NSPopUpButtonCell` takes over. The next select
experiment needs to capture or infer the final menu x position and compare it
against the anchor.

### Native widgets stop opening after select/datalist interactions

After interacting with `<select>` once or twice, native widgets stop opening for
the rest of the session. Later mouse movement still produces cursor updates, so
the browser is not dead, but new popup-open paths stop firing.

This may be an activation, focus, event dispatch, AppKit menu-tracking, or
popup-state cleanup bug. It should be treated independently from positioning
until logs identify where popup requests stop.

### Datalist does not work

Datalist could not be tested reliably because the post-select failure prevents
further native widgets from opening. Once the session-stopping popup failure is
understood, datalist should get its own clean trace and fix path.

## Approach

Do not try to fix every remaining bug in one experiment. Start with one bug,
design the smallest experiment that can identify its cause, record the result,
and only then move to the next bug.

The likely order is:

1. Post-select native-widget shutdown, because it contaminates multi-control
   test runs and can block every later popup experiment.
2. PagePopup alt-tab visibility, because it affects every PagePopup-family
   control that now has correct y placement.
3. Select dropdown x placement.
4. Datalist behavior.

The order may change if new logs show that two symptoms share one root cause.

## Experiments

### Experiment 1: Trace post-select popup shutdown

#### Description

After interacting with a `<select>` dropdown once or twice, later native widgets
stop opening for the rest of the session. Cursor updates still arrive, so the
browser process and basic mouse routing are alive. The missing signal is where a
later click stops:

- before Chromium receives mouse down/up;
- after Chromium receives input but before Blink activates the control;
- after Blink activation but before popup-open IPC;
- inside Chromium because popup/menu state still says a popup is active;
- inside AppKit because menu tracking or window activation did not unwind.

This experiment is logs-only. It must not change popup behavior. The goal is to
capture one clean sequence:

```text
open select -> close select -> click date -> click select again
```

If native widgets stop opening, the logs must identify the first missing
boundary in the second popup-open attempt.

#### Changes

1. **Keep the Issue 779 popup trace hooks.**

   Preserve existing logs for:
   - `RenderFrameHostImpl::ShowPopupMenu`;
   - `PopupMenuHelper::ShowPopupMenu`;
   - `RenderWidgetHostNSViewBridge::DisplayPopupMenu`;
   - `WebMenuRunner::runMenuInView`;
   - `DateTimeChooserImpl`;
   - `WebPagePopupImpl::SetWindowRect`;
   - `WebContentsImpl::ShowCreatedWidget`;
   - `RenderWidgetHostViewMac::InitAsPopup`.

2. **Log select menu lifecycle cleanup.**

   In Chromium's select/AppKit path, add trace lines for:
   - menu open entry;
   - menu selection callback;
   - menu cancel/dismiss callback;
   - `PopupMenuHelper` close/destructor cleanup;
   - renderer/browser notification that the popup menu closed.

   Each line should include enough join fields to follow one select menu:

   ```text
   path=select
   popup_sequence=...
   webcontents=...
   rfh=...
   helper=...
   view/window pointer
   selected_or_cancelled=...
   helper_alive_before/after=...
   ```

   If Chromium already has a process-local popup/menu counter or helper pointer,
   log it. Do not add protocol fields.

3. **Log the top of the popup-open decision points.**

   Add trace lines before any early return or suppression in:
   - `RenderFrameHostImpl::ShowPopupMenu`;
   - the PagePopup open path used by date/time/color;
   - any known "popup already active" or "suppress popup" guard near those
     paths.

   These logs must answer whether the failed post-select click reaches the
   popup-open functions and, if it does, why the open is rejected.

4. **Log mouse click delivery after select closes.**

   Add trace lines for the macOS `RenderWidgetHostViewMac` input path that sees
   mouse down/up events for the main webview after the select menu closes.

   Include:

   ```text
   event type
   location in window/view
   target RenderWidgetHostViewMac pointer
   webcontents pointer if available
   window isKey/isMain/isVisible
   firstResponder class if cheap to log
   ```

   The purpose is not to trace every cursor move. Log clicks only, or keep move
   logs out of the experiment trace, so the result is readable.

5. **Log window activation state around AppKit menu tracking.**

   In the AppKit select menu path, log before opening the menu and after it
   returns:

   ```text
   window isKey/isMain/isVisible
   app isActive
   firstResponder
   currentEvent type
   ```

   If AppKit leaves the hidden/transparent Chromium shell window in a different
   activation state after the select menu closes, this should make it visible.

6. **Add one concise summary line for each attempted popup.**

   Emit a summary line at each attempted popup boundary:

   ```text
   native_popup_attempt
     attempt=N
     control=select|date|unknown
     boundary=input|blink|browser-popup-open|appkit-open|cleanup
     outcome=entered|opened|closed|cancelled|suppressed|missing
     reason=...
   ```

   The summary does not need to be perfect automation. It only needs to make the
   trace easy to scan and compare with the detailed lines.

#### Verification

0. Build through the project scripts:

   ```bash
   cd /Users/ryan/dev/termsurf
   scripts/build.sh chromium
   scripts/build.sh roamium
   scripts/build.sh webtui --release
   scripts/build.sh wezboard
   ```

1. Start the test page server:

   ```bash
   cd /Users/ryan/dev/termsurf
   bun test-html/server.ts
   ```

2. Start Wezboard with deterministic logs:

   ```bash
   cd /Users/ryan/dev/termsurf
   mkdir -p logs/issue-782-exp1-state/termsurf

   TERMSURF_ISSUE_779_TRACE=1 \
   XDG_STATE_HOME="$PWD/logs/issue-782-exp1-state" \
   RUST_LOG=info \
   ./wezboard/target/debug/wezboard-gui \
   2>&1 | tee logs/issue-782-exp1-wezboard.log
   ```

3. Launch the TUI:

   ```bash
   /Users/ryan/dev/termsurf/webtui/target/release/web \
     --browser /Users/ryan/dev/termsurf/chromium/src/out/Default/roamium \
     http://localhost:9616/test-native-popups.html
   ```

4. Run one controlled interaction sequence:
   - click the date control and confirm it opens;
   - close it;
   - click the `<select>` dropdown and choose or dismiss one item;
   - click the date control again;
   - click the `<select>` dropdown again;
   - if widgets stop opening, stop the test immediately and preserve the logs.

5. Extract the trace:

   ```bash
   rg -a "\[issue-779-trace\]|native_popup_attempt|ShowPopupMenu|PopupMenuHelper|DisplayPopupMenu|WebMenuRunner|DateTimeChooserImpl|WebPagePopupImpl|ShowCreatedWidget|InitAsPopup|mouse.*down|mouse.*up|firstResponder|isKey|isMain|app isActive|menu.*close|menu.*cancel|menu.*dismiss" \
     logs/issue-782-exp1-wezboard.log \
     logs/issue-782-exp1-state/termsurf/webtui-trace.log \
     logs/issue-782-exp1-state/termsurf/roamium-trace.log \
     logs/issue-782-exp1-state/termsurf/chromium-server.log \
     > logs/issue-782-exp1-trace.log
   ```

6. Pass criteria:
   - the first date click shows the normal PagePopup open chain;
   - the select click shows the full select menu open and cleanup chain;
   - after select closes, the next failed click shows exactly where the chain
     stops: no mouse click delivered, no Blink activation, popup-open
     suppressed, AppKit/menu state stuck, or another concrete boundary;
   - logs are quiet enough to read without cursor-move floods.

7. Partial criteria:
   - the failure reproduces and the trace narrows the cause to a subsystem, but
     another experiment is needed to identify the exact function or state flag;
   - the failure does not reproduce, but the trace proves repeated date/select
     interactions can work in a clean run.

8. Fail criteria:
   - the logs still only show cursor movement after the failure;
   - the trace cannot distinguish input delivery, Blink activation, Chromium
     popup suppression, and AppKit menu cleanup;
   - the experiment changes popup behavior instead of only adding logs.
