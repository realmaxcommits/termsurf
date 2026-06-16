# Experiment 190: Live bell title and border pixels

## Description

Experiment 189 split the copied SwiftUI URL hover banner display out of the
remaining `RUNTIME-012B2B2B2B2B3C` gap. The same residual row still includes
bell border/title visible effects, audible bell output, measurable dock
attention, notification delivery, real OS cursor pixels, Quick Look/native link
preview display, and external Launch Services delivery.

This experiment targets the deterministic bell UI subset: when a real terminal
emits BEL with `bell-features = no-system,no-audio,no-attention,title,border`,
Roastty should set the surface bell state, prefix the window title through the
copied title computation path, and render the copied SwiftUI
`BellBorderOverlay`. It will not claim audible bell output, dock attention,
system notification delivery, OS cursor pixels, Quick Look/native preview UI, or
external URL-handler delivery.

The expected outcome is a new Oracle-complete runtime row for live bell
title/border UI proof, or a documented failure explaining which part of the bell
UI path is not deterministic in this VM.

## Changes

- Add a focused guard, tentatively
  `issues/0805-roastty-ghostty-parity/macos_live_bell_title_border_pixels.py`.
  - Launch the built debug app with an isolated config:
    `macos-applescript = true`, `quit-after-last-window-closed = true`,
    `bell-features = no-system,no-audio,no-attention,title,border`, fixed window
    size, stable dark background, stable foreground, no cursor blink, and
    `macos-titlebar-style = hidden` so rendered titlebar text cannot contaminate
    the screenshot oracle.
  - Create a real terminal surface through AppleScript and run a small painter
    that first sets a deterministic title with OSC 2, prints stable content,
    writes a ready marker, waits briefly, emits BEL, writes a bell marker, and
    then stays alive.
  - Capture the exact focused CGWindowID before BEL and after BEL using
    `screencapture -l`.
  - Require trace evidence from the live app/core path:
    `ringBell target=surface`, `surfaceBell state=true`, and an `appBell` branch
    showing `system=false audio=false attention=false`.
  - Require a window-title oracle after BEL through AppleScript/accessibility
    state, not pixels: the focused window title should contain the configured
    title prefixed by the bell marker produced by `computeTitle(title:bell:)`.
  - Compare before/after screenshots with a Swift sampler. The sampler should
    prove localized deltas in narrow surface-edge bands consistent with the
    copied 3 px golden `BellBorderOverlay` stroke, while central control regions
    and any titlebar/control area remain mostly unchanged. The border oracle
    must exclude or mask the titlebar/title area and must not count the title
    prefix as evidence for the border.
  - Store debug screenshots and JSON evidence under `/tmp`, following the
    existing issue guard pattern.
  - Check for new Roastty crash reports.
- Update `config_runtime_inventory.py` according to the outcome:
  - If the guard passes, split a new Oracle-complete row from
    `RUNTIME-012B2B2B2B2B3C` for live bell title/border UI effects.
  - Keep `RUNTIME-012B2B2B2B2B3C` as a `Gap` for actual OS notification
    delivery/banner/sound, audible bell output, measurable dock-attention state,
    real OS cursor pixels, Quick Look/native link preview display, and external
    Launch Services handler delivery.
  - Do not overclaim audible output or dock attention just because app branches
    are requested or title/border pixels are visible.
- Update residual guards and stale CFG-223 counts if a new runtime row is split.
- Regenerate `config-runtime-inventory.md` and `config-matrix.md`.
- Update Issue 805 `README.md` Learnings and Experiments index after the result
  is known.

## Verification

Pass criteria:

- The guard proves exact debug-app launch, isolated config/defaults, focused
  window-to-CGWindowID mapping, terminal marker evidence, and no new Roastty
  crash report.
- The guard proves the live BEL path with trace evidence for the expected
  surface and app branches.
- The guard proves the bell-title path with a window-title oracle that includes
  the deterministic title and the bell prefix after BEL. This title oracle is
  state/accessibility evidence, not screenshot evidence.
- The screenshot oracle captures the exact Roastty window before and after BEL
  and proves a bounded, visible perimeter delta attributable to
  `BellBorderOverlay`. The screenshot oracle must mask or exclude titlebar/title
  pixels and require the edge-band delta to be independent of the title-prefix
  state change.
- The experiment result does not claim audible bell output, dock attention, OS
  notification delivery, real OS cursor pixels, external URL delivery, or Quick
  Look/native preview behavior.
- Inventory counts and remaining gap IDs are updated exactly and asserted by
  guards.

Commands:

```bash
(cd roastty && macos/build.nu --action build)
PYTHONDONTWRITEBYTECODE=1 python3 issues/0805-roastty-ghostty-parity/macos_live_bell_title_border_pixels.py
PYTHONDONTWRITEBYTECODE=1 python3 issues/0805-roastty-ghostty-parity/notification_link_bell_gui_residual_parity.py
PYTHONDONTWRITEBYTECODE=1 python3 issues/0805-roastty-ghostty-parity/config_runtime_inventory.py --output issues/0805-roastty-ghostty-parity/config-runtime-inventory.md --matrix issues/0805-roastty-ghostty-parity/config-matrix.md
for guard in issues/0805-roastty-ghostty-parity/*_parity.py issues/0805-roastty-ghostty-parity/*_residual_audit.py issues/0805-roastty-ghostty-parity/macos_*_runtime.py; do
  PYTHONDONTWRITEBYTECODE=1 python3 "$guard" || exit 1
done
python3 -m py_compile issues/0805-roastty-ghostty-parity/*.py
rm -rf issues/0805-roastty-ghostty-parity/__pycache__
prettier --check issues/0805-roastty-ghostty-parity/README.md issues/0805-roastty-ghostty-parity/190-live-bell-title-border-pixels.md issues/0805-roastty-ghostty-parity/config-runtime-inventory.md issues/0805-roastty-ghostty-parity/config-matrix.md
git diff --check
```

The result must state the exact runtime row count, Oracle-complete count, closed
count, incomplete count, gap count, CFG-223 status, and remaining gap IDs.

## Design Review

Fresh-context Codex adversarial reviewer `Boyle the 3rd` reviewed the design
against the Issue 805 workflow, the remaining CFG-223 residual gap, the copied
bell title/border source paths, and the prior live notification/banner guard
patterns.

Initial verdict: **Changes required**.

Required finding resolved: the reviewer found that the first draft could let the
border pixel oracle pass on titlebar/title-prefix pixels instead of the copied
`BellBorderOverlay`. The design now uses `macos-titlebar-style = hidden`, proves
the title prefix through AppleScript/accessibility state rather than screenshot
pixels, and requires the screenshot sampler to mask or exclude titlebar/title
pixels while proving narrow surface-edge deltas independent of the title change.

Final verdict after re-review: **Approved**.

## Result

**Result:** Partial

The focused guard passed and split the deterministic copied bell title/border UI
effects out of the remaining residual GUI gap.

Evidence:

- `macos_live_bell_title_border_pixels.py` launches the built debug app with an
  isolated config using
  `bell-features = no-system,no-audio,no-attention,title,border`.
- The guard creates a real terminal surface, sets the deterministic OSC 2 title
  `Issue805Exp190BellTitle`, captures the exact focused CGWindowID before BEL,
  then triggers BEL from the live terminal.
- The live trace records `ringBell target=surface`,
  `appBell system=false audio=false attention=false`, and
  `surfaceBell state=true title=Issue805Exp190BellTitle`.
- The AX title oracle changes from `Issue805Exp190BellTitle` before BEL to
  `🔔 Issue805Exp190BellTitle` after BEL.
- The screenshot sampler masks the titlebar/control area and proves a localized
  edge-band delta: 6375 changed pixels on each side edge, 9390 changed pixels on
  the bottom edge, 0 changed pixels in the center control region, and 0 changed
  pixels in the titlebar/control mask.
- No new Roastty crash report was written during the workflow.

Updated inventory counts:

- `runtime_rows=94`
- `oracle_complete=90`
- `closed=93`
- `audit_covered=0`
- `incomplete=1`
- `gap=1`
- `cfg223=Gap`
- Remaining gap ID: `RUNTIME-012B2B2B2B2B3C`

Verification logs:

- `logs/issue805-exp190-build-1.log`
- `logs/issue805-exp190-live-bell-title-border-2.log`
- `logs/issue805-exp190-config-runtime-inventory-2.log`
- `logs/issue805-exp190-residual-guard-1.log`
- `logs/issue805-exp190-py-compile-1.log`
- `logs/issue805-exp190-prettier-check-2.log`
- `logs/issue805-exp190-diff-check-2.log`
- `logs/issue805-exp190-broad-guard-sweep-1.log`

This result does not claim audible bell output, dock attention, OS notification
delivery, real OS cursor pixels, external URL delivery, or Quick Look/native
preview behavior.

## Conclusion

Roastty now has live app proof for copied bell title-prefix state and copied
SwiftUI `BellBorderOverlay` pixels. `RUNTIME-012B2B2B2B2B3C5` is Oracle
complete, while `RUNTIME-012B2B2B2B2B3C` remains a `Gap` for actual OS
notification delivery/banner/sound, audible bell output, measurable
dock-attention state, real OS cursor pixels, Quick Look/native link preview
display beyond the copied SwiftUI URLHoverBanner, and external Launch Services
handler delivery.

## Completion Review

Fresh-context Codex adversarial reviewer `Einstein the 3rd` reviewed the
completed experiment, working-tree diff, relevant Swift/app sources, and
verification logs.

Verdict: **Approved**.

Findings: none.

The reviewer confirmed the result was still uncommitted, the README status was
`Partial`, CFG-223 remained `Gap`, the counts matched `runtime_rows=94`,
`oracle_complete=90`, `closed=93`, `incomplete=1`, `gap=1`, and the remaining
gap stayed `RUNTIME-012B2B2B2B2B3C`. The reviewer also confirmed that the guard
uses AX title state for the title-prefix proof, masks/excludes titlebar/title
pixels for the border oracle, and does not overclaim audible bell output, dock
attention, OS notification delivery, cursor pixels, Quick Look/native preview,
or external URL delivery.
