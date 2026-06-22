# Experiment 27: Inspect Parent Print Sheet Visibility

## Description

Experiment 26 proved that Roamium now calls
`beginSheetWithPrintInfo:modalForWindow:delegate:didEndSelector:contextInfo:`
with the right high-level preconditions:

- `NSApp.activationPolicy=regular`;
- `NSApp.active=true`;
- the parent window is present, key, main, visible, and matches
  `NSApp.keyWindow` / `NSApp.mainWindow`;
- the `beginSheetWithPrintInfo` call enters and exits;
- no native dialog is observed;
- no delegate response arrives;
- no print job is submitted.

The next unknown is whether AppKit creates and attaches any visible print panel
window after the begin call. This experiment is a trace-only probe that keeps
the guarded no-print safety posture and records what happens to the
`NSPrintPanel` and parent window immediately after presentation and on later
main-run-loop turns.

The goal is diagnostic, not a fix. The result should identify the next failing
hop more precisely than `mac-print-parent-window-sheet-response-missing`, for
example:

- `NSPrintPanel.window` never exists;
- the panel window exists but is never visible;
- the parent window never reports an attached sheet;
- the sheet appears only after a later run-loop turn that the current harness
  fails to wait through;
- a sheet/window appears but the watcher cannot observe or cancel it.

## Changes

- Create a new Chromium branch from `148.0.7778.97-issue-834-exp26`, named
  `148.0.7778.97-issue-834-exp27`.
- Update `chromium/README.md` with the new branch row after implementation.
- In `chromium/src/printing/printing_context_mac.mm`, add trace-only AppKit
  inspection around the parent-window sheet path:
  - trace the panel window state immediately before and after
    `beginSheetWithPrintInfo`;
  - trace the parent window's `attachedSheet` state after the begin call;
  - schedule one or more `dispatch_async(dispatch_get_main_queue(), ...)`
    inspections on later main-run-loop turns and trace the same state again;
  - use weak or otherwise non-retaining references for delayed inspections so
    the trace probe does not keep the autoreleased `NSPrintPanel` or related
    `NSWindow` objects alive longer than the current implementation would;
  - trace a distinct outcome when a delayed weak reference is already gone, so
    the result can distinguish "object was deallocated before delayed
    inspection" from "object still exists but is hidden or unattached";
  - trace enough identifiers to correlate whether `panel.window`, the parent
    `attachedSheet`, and `NSApp.orderedWindows` refer to the same window.
- Keep the existing delegate callback and activation-policy restoration logic
  unchanged except for any trace-only additions required to report state.
- Update `scripts/test-issue-834-pdf-native-print.py` only if needed to classify
  the new trace outcomes into more specific `first_failing_hop` values.
- Regenerate `chromium/patches/issue-834/` so the archive includes the
  Experiment 27 Chromium commit.
- Record the Chromium commit hash and probe result in this experiment file.

## Verification

Run the hygiene checks:

```bash
git status --short
git -C chromium/src status --short
git -C chromium/src rev-parse --abbrev-ref HEAD
git -C chromium/src rev-parse HEAD
git diff --check
git -C chromium/src diff --check

rm -rf scripts/__pycache__
PYTHONDONTWRITEBYTECODE=1 python3 -m py_compile \
  scripts/test-issue-834-pdf-native-print.py
rm -rf scripts/__pycache__
node --check scripts/probe-pdf-save-print-title-local.mjs
```

Build the Chromium library:

```bash
cd chromium/src
export PATH="/Users/astrohacker/dev/termsurf/chromium/depot_tools:$PATH"
autoninja -C out/Default libtermsurf_chromium
```

Run the guarded native-print probe:

```bash
cd /Users/astrohacker/dev/termsurf
rm -rf logs/issue-834-exp27-print-sheet-visibility
python3 scripts/test-issue-834-pdf-native-print.py \
  --log-dir logs/issue-834-exp27-print-sheet-visibility \
  --probe native-dialog \
  --allow-native-dialog-click
```

Pass criteria:

- no print job is submitted;
- Roamium remains alive until harness shutdown;
- the probe captures the new sheet/window visibility trace lines;
- the result identifies whether the panel window and/or parent attached sheet
  exists, becomes visible, or remains missing across later main-run-loop turns;
- delayed inspections distinguish deallocated weak references from hidden or
  unattached live objects;
- if a native print dialog is observed, the watcher cancels it and the result is
  classified as safe cancellation.

Partial criteria:

- the probe remains safely non-printing and captures enough new trace evidence
  to identify the next failing hop, but native print still does not reach safe
  observed cancellation.

Failure criteria:

- a print job is submitted;
- the native print safety gate is weakened;
- OK / printed / `kSuccess` is treated as safe;
- unrelated Chromium, Roamium, Ghostboard, or Surfari behavior is changed;
- the trace addition changes print behavior instead of only observing it;
- delayed trace blocks retain `NSPrintPanel` or related `NSWindow` objects in a
  way that could change their lifetime;
- the Chromium branch, README, patch archive, or experiment result is left
  inconsistent.

## Design Review

An adversarial Codex subagent reviewed the design with fresh context.

Initial verdict: **Changes Required**.

The reviewer found that the delayed `dispatch_async` trace plan was not
guaranteed to be trace-only. Because Objective-C blocks retain captured objects
by default, capturing the autoreleased `NSPrintPanel` or related windows for
later inspection could keep them alive longer than the current implementation
does and change the behavior under diagnosis.

The design now requires weak or otherwise non-retaining delayed inspection, a
distinct trace outcome when a delayed weak reference is already gone, and a
failure criterion that rejects delayed trace blocks which retain the panel or
related windows in a behavior-changing way.

Final verdict: **Approved**.

The reviewer confirmed that the prior Required finding is resolved, no new
Required finding was introduced by the fix, and the README still links
Experiment 27 as `Designed`.
