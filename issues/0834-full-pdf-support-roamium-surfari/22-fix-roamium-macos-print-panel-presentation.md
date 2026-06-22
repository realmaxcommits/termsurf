# Experiment 22: Fix Roamium macOS Print Panel Presentation

## Description

Experiment 21 proved that Roamium reaches Chromium's macOS print implementation
and enters `[panel runModalWithPrintInfo:]`, but the call never produces an
observable `Print` / `Printer` window and never returns a modal response before
the guarded harness terminates Roamium.

The known good state before the failure is:

- `TsPdfPrintManager::ScriptedPrint()` calls `AskUserForSettings()`;
- `PrintingContextMac::AskUserForSettings()` runs on the main thread;
- the delegate has a native parent view;
- that view has a native parent window;
- the `CATransaction` completion block is installed and entered;
- `[panel runModalWithPrintInfo:]` is entered;
- no modal response is recorded;
- the watcher sees Roamium's existing `Content Shell` window but no separate
  print-panel candidate;
- the print queues remain empty.

This experiment should determine whether the stuck modal is caused by AppKit
presentation state in Roamium's content-shell embedding, then apply the
narrowest safe fix if one is proven. Candidate causes include inactive
application/window state at the moment of presentation, `runModal` being a poor
fit for this embedding, and the need to use a sheet/app-modal presentation path
similar to content shell's JavaScript dialog path.

## Changes

1. Create a fresh Chromium branch for this issue experiment.

   ```bash
   cd chromium/src
   git checkout 148.0.7778.97-issue-834-exp21
   git checkout -b 148.0.7778.97-issue-834-exp22
   ```

   Update the branch table in `chromium/README.md`.

2. Add pre-presentation AppKit state trace points immediately before showing the
   print panel.

   Extend the Experiment 21 trace with:

   - `NSApp.activationPolicy`;
   - `NSApp.isActive`;
   - whether the parent window is key, main, visible, miniaturized, and ordered;
   - whether the parent window can become key/main;
   - whether `NSApp.keyWindow` and `NSApp.mainWindow` match the parent window;
   - the print panel class and whether it is visible before presentation.

   These are diagnostic events only; they must not click, submit, or dismiss a
   print dialog.

3. Try one presentation adjustment at a time, guarded by trace evidence.

   The preferred order is:

   1. If the app/window is inactive, make the parent window key/front and
      activate the app immediately before print-panel presentation.
   2. If activation does not fix the stuck modal, try presenting the print panel
      as a sheet on the parent window using
      `beginSheetWithPrintInfo: modalForWindow:delegate:didEndSelector:contextInfo:`
      or the modern block equivalent if available in this SDK.
   3. If sheet presentation is used, preserve the asynchronous callback contract
      and return `kCanceled` on watcher cancellation. Do not treat OK /
      `kSuccess` as a safe pass unless queue evidence proves no job was
      submitted, and do not count OK as cancellation.

   The experiment should stop after the first proven improvement. Do not pile
   multiple unproven AppKit workarounds into one result.

4. Keep the native-print safety gate intact.

   The harness must still:

   - require `--allow-native-dialog-click` for any production print click;
   - run the harmless preflight before the production click;
   - capture print queues before and after;
   - cancel any observed dialog;
   - hard-fail if a print job is submitted unexpectedly;
   - classify modal OK / callback `kSuccess` as a safety failure unless no-job
     evidence proves otherwise, and never count it as a safe cancellation.

5. Run the guarded native-print probe after each attempted presentation change.

   A passing result requires an observed native print panel, successful
   automated cancellation, unchanged print queues, and trace evidence showing
   the print path returned through the cancellation callback.

## Verification

Verification for the completed result is:

```bash
git status --short
git -C chromium/src status --short
git -C chromium/src rev-parse --abbrev-ref HEAD
git -C chromium/src rev-parse HEAD
git diff --check

cd chromium/src
export PATH="/Users/astrohacker/dev/termsurf/chromium/depot_tools:$PATH"
autoninja -C out/Default libtermsurf_chromium

cd /Users/astrohacker/dev/termsurf
rm -rf scripts/__pycache__
PYTHONDONTWRITEBYTECODE=1 python3 -m py_compile \
  scripts/test-issue-834-pdf-native-print.py
rm -rf scripts/__pycache__
node --check scripts/probe-pdf-save-print-title-local.mjs

rm -rf logs/issue-834-exp22-macos-print-panel-presentation
python3 scripts/test-issue-834-pdf-native-print.py \
  --log-dir logs/issue-834-exp22-macos-print-panel-presentation \
  --probe native-dialog \
  --allow-native-dialog-click

git diff --check
```

After committing Chromium branch changes inside `chromium/src`, regenerate the
cumulative Issue 834 patch archive from the local Chromium 148.0.7778.97 shallow
base:

```bash
cd /Users/astrohacker/dev/termsurf/chromium/src
rm -rf ../patches/issue-834
git format-patch 6b3fa66a923a9442c8ab0bc71b4b41ff24528d3b..HEAD \
  -o ../patches/issue-834
```

Required evidence:

- `chromium/README.md` records the new Chromium branch;
- Chromium source changes are committed inside `chromium/src`;
- `autoninja -C out/Default libtermsurf_chromium` passes;
- the Issue 834 patch archive is regenerated;
- the guarded native print probe records AppKit activation/window/panel state;
- no print job is submitted;
- if a native dialog appears, it is cancelled and queue state remains unchanged;
- if the dialog still does not appear, the result identifies the next precise
  failing sub-hop and whether activation/sheet presentation changed behavior;
- markdown is formatted with Prettier;
- Python bytecode cache is removed after compilation;
- `git diff --check` passes;
- design review is recorded, all real design-review findings are fixed, the
  design is approved, and the plan commit exists before implementation begins;
- completion review is recorded before the result commit.

## Pass Criteria

This experiment passes if Roamium native PDF print opens a native macOS print
panel, the safety watcher cancels it, the modal/callback path reports
cancellation rather than OK, and print queue evidence proves no job was
submitted.

## Partial Criteria

This experiment is partial if native print still does not pass but the result
proves a more precise AppKit presentation cause than Experiment 21, or proves
that one attempted presentation adjustment changes the failing sub-hop without
completing safe cancellation.

## Failure Criteria

This experiment fails if it submits a print job, weakens the native print safety
gate, treats OK / `kSuccess` as a safe cancellation, leaves Chromium
branch/patch records inconsistent, or makes broad AppKit changes without trace
evidence that they target the current modal presentation failure.

## Design Review

An adversarial Codex subagent reviewed the design with fresh context.

Verdict: **Approved**.

The reviewer found no Required findings.
