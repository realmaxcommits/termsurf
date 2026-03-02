# Issue 696: Double Click Suppression

Clicking an unfocused browser pane requires three clicks to interact: one to
focus, one that gets eaten, and one that finally goes through. Should only
require two (focus + interact).

## Why

Two independent click suppression flags — `pane_activation` (Issue 670) and
`overlay_activation` (Issue 606 Experiment 10) — both fire on the same click
when refocusing a pane that's in browse mode. Each flag eats one click, so two
clicks are consumed instead of one.

## How It Happens

Clicking an unfocused pane that's already in browse mode (e.g. clicked away to
another pane, now clicking back):

```
1. becomeFirstResponder() → focusCallback(true)
     → pane_activation = true

2. paneFocusChanged(true) → isOverlayBrowsing? YES
     → overlay_activation = true          ← BOTH flags now set

3. mouseButtonCallback(press)
     → pane_activation TRUE → consumed    ← first click eaten

4. mouseButtonCallback(release)
     → pane_activation TRUE → consumed, cleared

5. mouseButtonCallback(press)             ← user's SECOND click
     → pane_activation FALSE, continue
     → hit-test overlay → isOverlayForwarding → YES
     → overlay_activation TRUE → consumed  ← second click ALSO eaten

6. mouseButtonCallback(release)
     → overlay_activation TRUE → consumed, cleared

7. mouseButtonCallback(press)             ← THIRD click finally goes through
```

## Root Cause

`paneFocusChanged` (Surface.zig:3499) sets `overlay_activation = true` when a
pane gains focus while in browse mode. This was added in Issue 606 Experiment
10, before `pane_activation` existed. Issue 670 later added `pane_activation`,
which runs first in `mouseButtonCallback` and already suppresses the focus click
for all cases (terminal and overlay). The `overlay_activation` set in
`paneFocusChanged` is now redundant — it stacks on top of `pane_activation`,
consuming a second click.

The other place `overlay_activation` is set — in `notifyOverlayClicked()` for
control→browse mode transitions — is correct and unrelated. That path handles
activating browse mode, not refocusing.

## Test

1. Open two split panes, both with browser overlays in browse mode
2. Click the unfocused pane → focuses (first click consumed, correct)
3. Click again → click goes through to Chromium (not consumed)
4. Verify: control→browse activation still works (click overlay in control mode,
   first click activates, second interacts)
5. Verify: terminal pane click-to-focus still works (Issue 670)
