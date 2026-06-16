# Experiment 191: Real OS link cursor pixels

## Description

Experiment 190 split the copied bell title/border UI effects out of the
remaining `RUNTIME-012B2B2B2B2B3C` gap. The residual row still includes real OS
cursor pixels, notification delivery/banner/sound, audible bell output, dock
attention, Quick Look/native link preview display, and external Launch Services
handler delivery.

Experiment 188 already proved the live app requests the link cursor shape when
Command-modified mouse movement hovers a deterministic URL. That did not prove
the OS-rendered cursor image. This experiment targets that exact missing slice:
use `screencapture -C` to capture the cursor pixels around the mouse position,
subtract a cursorless capture of the same rectangle, and compare the resulting
cursor mask against a non-link cursor mask.

The expected outcome is a new Oracle-complete runtime row for real OS link
cursor pixels, or a documented failure explaining why macOS cursor-inclusive
screenshot capture is not deterministic in this VM.

## Changes

- Add a focused guard, tentatively
  `issues/0805-roastty-ghostty-parity/macos_real_link_cursor_pixels.py`.
  - Reuse the Experiment 188/189 live link-hover setup: isolated config,
    deterministic URL at a known terminal row/column, fixed window size, stable
    background/foreground, hidden titlebar, no cursor blink, and exact focused
    CGWindowID/window bounds evidence.
  - Print a deterministic URL and a separate blank/background probe area in a
    real terminal surface.
  - Move the mouse to the non-link probe area and capture a small screen
    rectangle around the mouse twice: once without `screencapture -C` and once
    with `screencapture -C`. The pixel difference is the non-link cursor mask.
  - Move the mouse over the deterministic URL with the Command modifier, require
    trace evidence for `cursorShape raw=3 pointerStyle=link` and the exact
    `mouseOverLink url=...`, then capture the same kind of cursorless and
    cursor-included screen rectangle around the mouse. The pixel difference is
    the link cursor mask.
  - Compare the isolated masks with a Swift sampler:
    - both states must contain at least 150 cursor-mask pixels with RGB delta
      > = 45 between cursorless and cursor-included captures;
    - both cursor masks must have sane bounding boxes: width between 6 and 80
      px, height between 10 and 100 px, and the bounding box must intersect the
      center half of the captured rectangle so the mask is plausibly the cursor
      near the injected mouse position;
    - the two cursorless captures for a given state, if sampled twice for
      stability, must have fewer than 80 changed pixels with RGB delta >= 45 in
      the capture rectangle before cursor subtraction;
    - the link and non-link masks must be quantitatively different: symmetric
      mask difference must be at least 25% of the larger mask's changed-pixel
      count, or their bounding boxes must differ by at least 8 px in width,
      height, or center position;
    - the link mask must not be accepted solely because underlying terminal text
      differs; all mask metrics must come from cursor-included minus cursorless
      captures for the same mouse position/state.
  - Store debug screenshots and JSON evidence under `/tmp`, following the
    existing issue guard pattern.
  - Check for new Roastty crash reports.
- Update `config_runtime_inventory.py` according to the outcome:
  - If the guard passes, split a new Oracle-complete row from
    `RUNTIME-012B2B2B2B2B3C` for real OS link cursor pixels.
  - Keep `RUNTIME-012B2B2B2B2B3C` as a `Gap` for actual OS notification
    delivery/banner/sound, audible bell output, measurable dock-attention state,
    Quick Look/native link preview display, and external Launch Services handler
    delivery.
  - Do not overclaim all cursor behavior; this experiment only proves the
    OS-rendered link cursor for the live URL-hover path.
- Update residual guards and stale CFG-223 counts if a new runtime row is split.
- Regenerate `config-runtime-inventory.md` and `config-matrix.md`.
- Update Issue 805 `README.md` Learnings and Experiments index after the result
  is known.

## Verification

Pass criteria:

- The guard proves exact debug-app launch, isolated config/defaults, focused
  window-to-CGWindowID mapping, terminal marker evidence, and no new Roastty
  crash report.
- The guard proves the live link-hover state with trace evidence for the
  expected URL and link cursor request.
- The screenshot oracle captures cursorless and cursor-included screen
  rectangles for both non-link and link-hover states, isolates cursor-only masks
  by subtraction, and proves the link cursor mask is present and materially
  different from the non-link cursor mask with the numeric thresholds from the
  sampler design above.
- The experiment result does not claim notification delivery, audible bell
  output, dock attention, Quick Look/native preview behavior, or external URL
  delivery.
- Inventory counts and remaining gap IDs are updated exactly and asserted by
  guards.

Commands:

```bash
(cd roastty && macos/build.nu --action build)
PYTHONDONTWRITEBYTECODE=1 python3 issues/0805-roastty-ghostty-parity/macos_real_link_cursor_pixels.py
PYTHONDONTWRITEBYTECODE=1 python3 issues/0805-roastty-ghostty-parity/notification_link_bell_gui_residual_parity.py
PYTHONDONTWRITEBYTECODE=1 python3 issues/0805-roastty-ghostty-parity/config_runtime_inventory.py --output issues/0805-roastty-ghostty-parity/config-runtime-inventory.md --matrix issues/0805-roastty-ghostty-parity/config-matrix.md
for guard in issues/0805-roastty-ghostty-parity/*_parity.py issues/0805-roastty-ghostty-parity/*_residual_audit.py issues/0805-roastty-ghostty-parity/macos_*_runtime.py; do
  PYTHONDONTWRITEBYTECODE=1 python3 "$guard" || exit 1
done
python3 -m py_compile issues/0805-roastty-ghostty-parity/*.py
rm -rf issues/0805-roastty-ghostty-parity/__pycache__
prettier --check issues/0805-roastty-ghostty-parity/README.md issues/0805-roastty-ghostty-parity/191-real-os-link-cursor-pixels.md issues/0805-roastty-ghostty-parity/config-runtime-inventory.md issues/0805-roastty-ghostty-parity/config-matrix.md
git diff --check
```

The result must state the exact runtime row count, Oracle-complete count, closed
count, incomplete count, gap count, CFG-223 status, and remaining gap IDs.

## Design Review

Fresh-context Codex adversarial reviewer `Ptolemy the 3rd` reviewed the design
against the Issue 805 workflow, the remaining CFG-223 residual gap, the prior
live link-hover guards, and the cursor-inclusive `screencapture -C` strategy.

Initial verdict: **Changes required**.

Required finding resolved: the reviewer found that the first draft's screenshot
subtraction oracle was underspecified and could be implemented with vacuous
thresholds. The design now gives concrete criteria for cursor-mask changed
pixels, cursor-mask bounding boxes, cursorless-background stability, and
link-versus-non-link mask dissimilarity.

Final verdict after re-review: **Approved**.

## Result

**Result:** Partial

The focused guard passed after one implementation fix.

The first live run failed before sampling because the guard multiplied the
global mouse point by the measured Retina screenshot scale before passing it to
`screencapture -R`:

```text
rect (2058.0, 942.0, 180.0, 180.0) does not intersect any displays
```

That showed `screencapture -R` expects global display coordinates in points,
even though the output PNG is Retina-scaled. The guard now preserves the
measured scale in the evidence JSON but captures the source rectangle in global
display points.

The second live run passed:

```text
macos_real_link_cursor_pixels=pass
```

Evidence from `/tmp/termsurf-issue805-exp191-real-cursor-latest.json`:

- `scale = 2.0`
- terminal resize: `cols = 100`, `rows = 37`, `width_px = 1600`,
  `height_px = 1136`
- trace tail included `cursorShape raw=3 pointerStyle=link` and
  `mouseOverLink url=https://example.com/issue805-exp191-real-cursor`
- non-link cursor mask: 350 changed pixels, stable cursorless background, bbox
  `16x38`
- link cursor mask: 701 changed pixels, stable cursorless background, bbox
  `30x38`
- link/non-link mask difference: 721-pixel symmetric difference and 15-pixel
  bbox delta

The inventory now splits `RUNTIME-012B2B2B2B2B3C6` as Oracle complete for live
macOS real OS link cursor pixels. `RUNTIME-012B2B2B2B2B3C` remains a `Gap`, but
only for actual OS notification delivery/banner/sound after authorization is
available, audible bell output, measurable dock-attention state, Quick
Look/native link preview display beyond the copied SwiftUI URLHoverBanner, and
external Launch Services handler delivery.

The regenerated CFG-223 counts are:

- runtime rows: 95
- Oracle complete: 91
- closed: 94
- audit covered: 0
- incomplete: 1
- runtime gaps: 1
- CFG-223 status: `Gap`

Verification logs:

- `logs/issue805-exp191-build-1.log`
- `logs/issue805-exp191-real-cursor-1.log` for the initial point/scale failure
- `logs/issue805-exp191-real-cursor-2.log` for the passing live run
- `logs/issue805-exp191-config-runtime-inventory-1.log`
- `logs/issue805-exp191-residual-guard-1.log`
- `logs/issue805-exp191-broad-guard-sweep-1.log`
- `logs/issue805-exp191-py-compile-2.log`
- `logs/issue805-exp191-prettier-check-1.log`
- `logs/issue805-exp191-diff-check-1.log`

## Conclusion

Experiment 191 proves the real OS-rendered macOS link cursor over the live
Roastty window. The remaining CFG-223 work is now narrower: notification
delivery/banner/sound, audible bell output, dock attention, Quick Look/native
preview display, and external Launch Services handler delivery.

## Completion Review

Fresh-context Codex adversarial reviewer `Epicurus the 3rd` reviewed the
completed experiment, implementation diff, inventory split, residual guard, and
verification logs.

Initial verdict: **Changes required**.

Required finding resolved: the reviewer found that `RUNTIME-012B2B2B2B2A` still
said real pointer/link cursor pixels remained tracked by
`RUNTIME-012B2B2B2B2B3C`. The inventory source now points real OS link cursor
pixels at `RUNTIME-012B2B2B2B2B3C6`, leaves Quick Look/native preview in
`RUNTIME-012B2B2B2B2B3C`, and the generated inventory was regenerated.

Optional finding resolved: the result log list omitted the successful residual
guard, broad sweep, py_compile, prettier, and diff-check logs. Those logs are
now listed in the result.

Final verdict after re-review: **Approved**.
