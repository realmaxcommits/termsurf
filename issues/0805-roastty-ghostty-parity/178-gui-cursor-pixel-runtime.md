# Experiment 178: GUI Cursor Pixel Runtime

## Description

`RUNTIME-008B2B2B2B2B` still owns actual app/GUI cursor pixel proof. Earlier
experiments proved deterministic cursor renderer data, cursor priority,
terminal/default cursor runtime behavior, Metal shader cursor readback, and live
window-padding screenshot proof, but they did not prove that a real macOS
Roastty window visibly draws the configured cursor pixels in an exact app
screenshot.

This experiment will split out one focused live GUI slice: a debug Roastty app
window with an isolated config draws a non-blinking high-contrast block cursor
at a known terminal cell, and an exact CGWindowID screenshot contains the
expected cursor-colored pixel region at that cell while nearby background/text
regions do not. It will not claim broader GUI/pixel parity, broad font output
parity, native notification/link/bell GUI parity, full app walkthrough parity,
or CFG-223 completion.

## Changes

- New guard script:
  `issues/0805-roastty-ghostty-parity/macos_gui_cursor_pixel_runtime.py`
  - Launch the debug `roastty/macos/build/Debug/Roastty.app` with isolated
    config and defaults.
  - Use a controlled config with:
    - `macos-applescript = true`
    - `quit-after-last-window-closed = true`
    - `font-size = 16`
    - `cursor-style = block`
    - `cursor-style-blink = false`
    - `cursor-color = #ff00ff`
    - `cursor-text = #00ff00`
    - `background = #102030`
    - `foreground = #ffffff`
    - `window-padding-x = 0`
    - `window-padding-y = 0`
    - `macos-titlebar-style = hidden`
  - Create a terminal running a deterministic child process that writes a marker
    file, paints a controlled dark background with a bright non-magenta grid
    landmark, positions the visible block cursor at a known row/column, and then
    sleeps. The guard must wait for the marker file before screenshot capture
    and record the terminal id, command path, and marker path in debug JSON.
  - Prove the screenshot target is the exact launched debug-app PID and the
    captured CGWindowID maps to the focused accessibility window, following the
    stricter Experiment 176/177 pattern.
  - Capture the exact window with `screencapture -l`.
  - Derive the cursor cell rectangle from screenshot evidence instead of a lucky
    hard-coded crop:
    - detect the bright grid landmark in the captured image;
    - infer terminal cell width/height from repeated landmark spacing or another
      deterministic grid measurement;
    - compute the expected cursor cell rectangle from the detected terminal grid
      origin, measured cell geometry, and the known cursor row/column;
    - record all detected geometry, expected cursor rectangle, and sample
      rectangles in debug JSON.
  - Sample stable regions in the captured PNG:
    - the expected cursor cell must be magenta-dominant;
    - the adjacent same-row background cells must be background-dominant and not
      magenta-dominant;
    - the bright landmark cells must be bright-dominant and not
      magenta-dominant;
    - the screenshot must be nonblank and must fail if magenta appears only in a
      wrong window, wrong region, or full-screen color wash.
  - Save debug PNG/JSON artifacts under `/tmp/termsurf-issue805-exp178-*`.

- Inventory: `issues/0805-roastty-ghostty-parity/config_runtime_inventory.py`
  - Split `RUNTIME-008B2B2B2B2B` into:
    - new `RUNTIME-008B2B2B2B2D` as an `Oracle complete` row for focused live
      app/GUI block cursor pixel proof;
    - the remaining gap row for broader GUI/pixel parity and renderer-visible
      effects outside this focused cursor proof.
  - Update `EXPECTED_IDS` and CFG-223 counts only for the new passing row.
  - Keep CFG-223 as `Gap`.

- Existing guard scripts:
  - Update expected CFG-223 count text from 76/79 to the new generated counts if
    the new row is added and passing.
  - Narrow stale wording from "GUI cursor pixels" to broader remaining
    renderer/GUI pixel gaps in scripts that inspect the remaining gap row.

- Issue docs:
  - Update this experiment from `Designed` to `Pass`/`Partial`/`Fail` after
    verification.
  - Add a focused learning to the issue README only if the live cursor
    screenshot guard teaches a reusable technique or limitation.

## Verification

- Build the macOS app:

```bash
(cd roastty && macos/build.nu --action build)
```

- Regenerate CFG-223 inventory and matrix:

```bash
PYTHONDONTWRITEBYTECODE=1 python3 issues/0805-roastty-ghostty-parity/config_runtime_inventory.py \
  --output issues/0805-roastty-ghostty-parity/config-runtime-inventory.md \
  --matrix issues/0805-roastty-ghostty-parity/config-matrix.md
PYTHONDONTWRITEBYTECODE=1 python3 issues/0805-roastty-ghostty-parity/platform_runtime_classification.py \
  --config-inventory issues/0805-roastty-ghostty-parity/config-inventory.md \
  --output issues/0805-roastty-ghostty-parity/platform-runtime-classification.md
```

- Run the existing CFG-223 guard set:

```bash
for f in issues/0805-roastty-ghostty-parity/*_runtime_parity.py \
  issues/0805-roastty-ghostty-parity/terminal_runtime_residual_audit.py \
  issues/0805-roastty-ghostty-parity/link_hover_preview_dispatch_parity.py \
  issues/0805-roastty-ghostty-parity/link_hover_modifier_refresh_parity.py \
  issues/0805-roastty-ghostty-parity/link_preview_context_runtime_parity.py; do
  PYTHONDONTWRITEBYTECODE=1 python3 "$f" || exit 1
done
```

- Run the live macOS guard subset needed for this issue family:

```bash
PYTHONDONTWRITEBYTECODE=1 python3 issues/0805-roastty-ghostty-parity/macos_gui_cursor_pixel_runtime.py
PYTHONDONTWRITEBYTECODE=1 python3 issues/0805-roastty-ghostty-parity/macos_window_padding_pixel_runtime.py
PYTHONDONTWRITEBYTECODE=1 python3 issues/0805-roastty-ghostty-parity/macos_titlebar_runtime.py
```

- Syntax, formatting, and hygiene:

```bash
PYTHONDONTWRITEBYTECODE=1 python3 -m py_compile issues/0805-roastty-ghostty-parity/*.py
rm -rf issues/0805-roastty-ghostty-parity/__pycache__
prettier --write --prose-wrap always --print-width 80 \
  issues/0805-roastty-ghostty-parity/README.md \
  issues/0805-roastty-ghostty-parity/178-gui-cursor-pixel-runtime.md \
  issues/0805-roastty-ghostty-parity/config-runtime-inventory.md \
  issues/0805-roastty-ghostty-parity/config-matrix.md \
  issues/0805-roastty-ghostty-parity/platform-runtime-classification.md
git diff --check
```

Pass criteria:

- The new guard passes only after proving exact debug-app launch, isolated
  config/defaults, no new crash report, frontmost/main-window evidence,
  accessibility-focused-window to CGWindowID mapping, exact-window screenshot
  capture, command-marker evidence, geometry-derived cursor-cell detection,
  magenta-dominant cursor pixels at the expected cell, and negative neighboring
  samples that are not magenta-dominant.
- The pixel oracle cannot pass if the screenshot is blank, captures the wrong
  process/window, paints the whole window magenta, detects only the mouse
  pointer, or finds magenta away from the expected cursor cell.
- The new inventory row claims only focused live app/GUI block cursor pixel
  proof.
- The remaining `RUNTIME-008B2B2B2B2B` row still owns broader GUI/pixel parity
  and renderer-visible effects outside this focused cursor proof.
- CFG-223 remains `Gap`.

Fail criteria:

- The guard can pass without exact-CGWindowID screenshot evidence tied to the
  focused accessibility window.
- The guard can pass without command-marker evidence that the deterministic
  painter ran.
- The guard can pass without geometry-derived cursor-cell detection recorded in
  debug JSON.
- The guard can pass when magenta appears only outside the expected cursor cell.
- The guard relies on an installed app or non-isolated user config/defaults.
- The experiment claims full renderer pixel parity, broad font output parity,
  notification/link/bell GUI parity, full app walkthrough parity, or CFG-223
  completion.

## Design Review

Fresh-context adversarial reviewer `Bohr the 3rd` reviewed the initial design
and returned `APPROVED`.

Optional finding:

- The initial design said it would add a new inventory row but did not name the
  row ID. Since `RUNTIME-008B2B2B2B2C` is already used by Experiment 177, the
  reviewer recommended naming the next row explicitly.

Fix made:

- The design now names the planned cursor row as `RUNTIME-008B2B2B2B2D` and
  states that `EXPECTED_IDS` must include it.

Final design verdict: **Approved**.

## Result

**Result:** Pass

Experiment 178 added
`issues/0805-roastty-ghostty-parity/macos_gui_cursor_pixel_runtime.py`, a live
macOS GUI guard for focused app/GUI block cursor pixel proof.

The guard launches the debug Roastty app with isolated config/defaults and a
high-contrast, non-blinking cursor setup:

- `cursor-style = block`
- `cursor-style-blink = false`
- `cursor-color = #ff00ff`
- `cursor-text = #00ff00`
- `background = #102030`
- `window-padding-x = 0`
- `window-padding-y = 0`
- `macos-titlebar-style = hidden`

It creates a terminal running a deterministic Python painter. The painter writes
a marker file, paints a bright yellow background landmark at a known grid
rectangle, positions the visible block cursor at row 10, column 12, and sleeps.
The guard waits for the marker before screenshot capture.

The first live run failed usefully: the exact-window screenshot showed a visible
white cursor instead of the configured magenta cursor. That exposed a real
Roastty mismatch: the active frame renderer used OSC 12 cursor color and default
foreground, but did not thread `cursor-color` or `cursor-text` from `Config`
through the live render paths.

The implementation now threads config-derived cursor color state into
`FrameRenderState`:

- `cursor-color` drives the cursor overlay/sprite color;
- `cursor-text` drives the block-cursor text recolor uniform;
- both normal and custom-shader render entry points call
  `FrameCursorOptions::with_config(config)`;
- `render_state_cursor_colors_come_from_config` guards the deterministic Rust
  path.

The final live guard run passed after rebuilding the app. It proves the
screenshot target is the exact debug-app PID, maps the focused accessibility
window to a PID-owned layer-0 CGWindowID, captures that exact window with
`screencapture -l`, derives terminal grid geometry from the painted landmark,
and samples the expected cursor cell plus negative neighboring/background
regions.

Debug artifacts from the passing run:

- `/tmp/termsurf-issue805-exp178-gui-cursor.png`
- `/tmp/termsurf-issue805-exp178-gui-cursor.json`

Latest focused guard output:

```text
macos_gui_cursor_pixel_runtime=pass terminal=ED95DCCE-016A-47D9-9543-36B64C70A4E7
```

Representative metric summary from the passing debug JSON:

```text
landmark = {x: 64, y: 94, width: 319, height: 149}
cellSize = {width: 15.95, height: 29.8}
cursorCell = {x: 177, y: 335, width: 15, height: 26}
totalMagenta = 120
outsideCursorMagenta = 0
cursor sample: 195 / 195 magenta
left/right background samples: 0 magenta, 240 / 240 background
landmark sample: 0 magenta, 240 / 240 bright
```

Inventory impact:

- Added `RUNTIME-008B2B2B2B2D` for focused live app/GUI block cursor pixel
  proof.
- Narrowed `RUNTIME-008B2B2B2B2B` to remaining broader GUI/pixel parity.
- CFG-223 remains `Gap` with 77 Oracle-complete rows, 80 closed rows, 4
  incomplete rows, and 4 runtime gaps.

Verification performed:

```text
(cd roastty && macos/build.nu --action build)
cargo test --manifest-path roastty/Cargo.toml render_state_cursor -- --test-threads=1
PYTHONDONTWRITEBYTECODE=1 python3 issues/0805-roastty-ghostty-parity/macos_gui_cursor_pixel_runtime.py
PYTHONDONTWRITEBYTECODE=1 python3 issues/0805-roastty-ghostty-parity/config_runtime_inventory.py --output issues/0805-roastty-ghostty-parity/config-runtime-inventory.md --matrix issues/0805-roastty-ghostty-parity/config-matrix.md
PYTHONDONTWRITEBYTECODE=1 python3 issues/0805-roastty-ghostty-parity/platform_runtime_classification.py --config-inventory issues/0805-roastty-ghostty-parity/config-inventory.md --output issues/0805-roastty-ghostty-parity/platform-runtime-classification.md
for f in issues/0805-roastty-ghostty-parity/*_runtime_parity.py issues/0805-roastty-ghostty-parity/terminal_runtime_residual_audit.py issues/0805-roastty-ghostty-parity/link_hover_preview_dispatch_parity.py issues/0805-roastty-ghostty-parity/link_hover_modifier_refresh_parity.py issues/0805-roastty-ghostty-parity/link_preview_context_runtime_parity.py; do PYTHONDONTWRITEBYTECODE=1 python3 "$f" || exit 1; done
```

The macOS app build succeeded with existing Swift/linker warnings.

## Completion Review

Fresh-context adversarial reviewer `Dewey the 3rd` reviewed the completed
experiment and returned `APPROVED`.

Optional finding:

- `window_padding_layout_runtime_parity.py` still checked only that the
  remaining renderer row was `Gap`; the reviewer recommended also asserting the
  newly narrowed `broader GUI/pixel parity` wording.

Fix made:

- The guard now requires `broader GUI/pixel parity` in the remaining
  `RUNTIME-008B2B2B2B2B` row, and the focused guard was rerun successfully.

Final completion verdict: **Approved**.

## Conclusion

Focused live app/GUI block cursor pixel proof is now complete, and the guard
caught a real config propagation bug before passing. The remaining renderer row
no longer owns GUI cursor pixels; it is limited to broader GUI/pixel parity
outside this focused cursor proof. The next experiment should target one of the
four remaining CFG-223 gaps: broader renderer pixels, broad font output, broader
macOS app walkthrough, or notification/link/bell GUI effects.
