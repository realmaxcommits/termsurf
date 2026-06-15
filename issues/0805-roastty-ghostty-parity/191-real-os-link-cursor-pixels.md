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
