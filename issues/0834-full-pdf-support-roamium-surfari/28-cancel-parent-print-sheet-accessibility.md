# Experiment 28: Cancel Parent Print Sheet Through Accessibility

## Description

Experiment 27 proved that the native macOS print sheet is present and visible,
but the production watcher is looking in the wrong place:

- AppKit attaches a visible key `NSPanel` titled `Print` to Roamium's parent
  window.
- `NSPrintPanel` does not expose a dynamic `window` selector in this build.
- The current watcher searches the CoreGraphics window list for separate
  `"Print"` / `"Printer"` windows.
- CoreGraphics only reports the parent Roamium window, not the document-modal
  attached print sheet.
- The first failing hop is now
  `mac-print-parent-window-sheet-visible-watcher-missed`.

This experiment should make the native print watcher sheet-aware. It should use
Accessibility against the Roamium process/window hierarchy to find and press the
print sheet's Cancel button after the trace proves the sheet exists, instead of
depending only on CoreGraphics title matching.

The goal is to safely reach native print cancellation for Roamium PDFs without
submitting a print job.

## Changes

- Update `scripts/test-issue-834-pdf-native-print.py`.
- Reuse and extend the existing Swift Accessibility helper rather than adding a
  separate automation stack.
- Add a watcher path that can target a known process PID even when CoreGraphics
  does not expose a separate print-sheet window:
  - observe the native print trace for the parent-window sheet evidence from
    Experiment 27;
  - identify the Roamium process PID from the running browser process or trace
    lines;
  - walk that process's AX windows/sheets recursively;
  - find a `Cancel` button inside the print sheet or its attached window
    hierarchy;
  - invoke `AXPress` on that button.
- Preserve the existing CoreGraphics title watcher as a first attempt or
  diagnostic path, but do not require it for attached sheets.
- Record enough watcher output to distinguish:
  - AX permission failure;
  - Roamium process/window not found;
  - sheet found but Cancel button missing;
  - Cancel button pressed but callback not observed;
  - safe cancellation observed.
- Update classification so successful sheet-aware cancellation maps to
  `native-print-dialog-seen-cancelled`, and partial/failure cases get specific
  `first_failing_hop` values.
- Do not modify Chromium unless the harness proves a product-side callback bug
  after AX cancellation succeeds.

## Verification

Run the harness checks:

```bash
rm -rf scripts/__pycache__
PYTHONDONTWRITEBYTECODE=1 python3 -m py_compile \
  scripts/test-issue-834-pdf-native-print.py
rm -rf scripts/__pycache__
node --check scripts/probe-pdf-save-print-title-local.mjs
git diff --check
git -C chromium/src diff --check
```

Run the guarded native-print probe:

```bash
rm -rf logs/issue-834-exp28-sheet-ax-cancel
python3 scripts/test-issue-834-pdf-native-print.py \
  --log-dir logs/issue-834-exp28-sheet-ax-cancel \
  --probe native-dialog \
  --allow-native-dialog-click
```

Pass criteria:

- the native print safety preflight passes;
- the Roamium PDF print sheet is detected through the sheet-aware watcher;
- the watcher presses Cancel through Accessibility;
- Roamium receives a cancel path rather than `kSuccess`;
- no print job is submitted;
- Roamium remains alive until harness shutdown;
- the harness exits successfully with
  `first_failing_hop=native-print-dialog-seen-cancelled`.

Partial criteria:

- no print job is submitted and Roamium remains alive, but the sheet-aware
  watcher exposes a new specific failing hop that prevents safe cancellation.

Failure criteria:

- a print job is submitted;
- the native print safety gate is weakened;
- OK / printed / `kSuccess` is treated as safe;
- the watcher sends unbounded keyboard or mouse input to the wrong process;
- unrelated Chromium, Roamium, Ghostboard, or Surfari behavior is changed;
- the harness hides a permission failure as a product bug;
- the experiment claims native print is solved without proving queue state,
  cancellation, and Roamium liveness.

## Design Review

An adversarial Codex subagent reviewed the design with fresh context.

Verdict: **Approved**.

The reviewer found no findings. It confirmed that the design is linked from the
issue README as `Designed`, has the required Description, Changes, and
Verification sections, follows directly from Experiment 27's
`mac-print-parent-window-sheet-visible-watcher-missed` result, keeps scope to
the harness, preserves print safety, requires PID-targeted Accessibility
cancellation, distinguishes permission failures, and defines concrete
pass/partial/failure criteria for queue state, cancellation, and Roamium
liveness.
