# Experiment 19: Trace Roamium Native Print Plumbing

## Description

Experiment 18 safely clicked the real Roamium PDF print control. The PDF viewer
JavaScript emitted print records, Chromium reached
`pdf_view_web_plugin.cc event=handle-print`, and no print job was submitted, but
the macOS native Print/Printer dialog never appeared.

This experiment should identify the first missing native print hop inside
Roamium's Chromium embedding path. It is an audit/probe experiment, not a
product fix. The goal is to prove whether the missing integration is in:

- the PDF plugin `client_->Print()` path;
- `components/printing/renderer/PrintRenderFrameHelper`;
- browser-side `printing::PrintViewManager` ownership;
- print preview registration and WebContents delegate plumbing;
- macOS `PrintingContext` / system-dialog integration;
- build flags or command-line flags that disable the relevant path.

## Changes

1. Audit upstream Chromium's PDF print path.

   Read the current Chromium sources, especially:

   - `chrome/browser/pdf/pdf_extension_printing_test.cc`;
   - `pdf/pdf_view_web_plugin.cc`;
   - `components/printing/renderer/print_render_frame_helper.*`;
   - browser-side `printing::PrintViewManager` creation and ownership sites;
   - macOS printing context code under `printing/`.

   Record the expected upstream chain for the PDF toolbar Print button. The
   audit should explicitly distinguish print-preview mode from basic system
   dialog mode, because the upstream PDF tests exercise both.

2. Audit TermSurf's current Chromium embedding path.

   Read the local TermSurf-specific code, especially:

   - `content/libtermsurf_chromium/ts_pdf_renderer_support.cc`;
   - `content/libtermsurf_chromium/ts_browser_main_parts.cc`;
   - `content/libtermsurf_chromium/BUILD.gn`;
   - `chromium/src/out/Default/args.gn`;
   - the Issue 834 patch archive entries related to PDF print.

   Determine which expected upstream print objects are present, missing, or
   intentionally disabled in Roamium.

3. Add only diagnostic instrumentation if static audit is insufficient.

   If the missing hop cannot be proven by static audit and existing logs, add a
   narrow trace behind existing TermSurf print trace controls. The trace may
   record whether:

   - `PdfViewWebPlugin::OnInvokePrintDialog()` runs;
   - `client_->Print()` calls `PrintRenderFrameHelper::PrintNode()`;
   - `PrintRenderFrameHelper` exists for the relevant frame;
   - `PrintRenderFrameHelper` reaches its browser IPC call;
   - the browser has a `printing::PrintViewManager` for the WebContents;
   - print preview or basic print code is compiled/enabled.

   If Chromium code must be modified, create a fresh Chromium branch for Issue
   834 and update `chromium/README.md` according to the repo's Chromium branch
   rules.

4. Run the existing safe native print probe.

   Reuse Experiment 18's guarded command:

   ```bash
   python3 scripts/test-issue-834-pdf-native-print.py \
     --log-dir logs/issue-834-exp19-print-plumbing \
     --probe native-dialog \
     --allow-native-dialog-click
   ```

   The probe must still refuse unsafe native print attempts, must record the
   harmless preflight, must record print queue before/after state, and must not
   submit a print job.

5. Classify the first missing hop.

   Record the first objectively proven missing hop. Examples:

   - `plugin-print-not-invoked`;
   - `print-render-frame-helper-missing`;
   - `print-node-not-called`;
   - `print-render-frame-helper-stops-before-browser-ipc`;
   - `print-view-manager-missing`;
   - `print-preview-disabled`;
   - `basic-print-dialog-disabled`;
   - `mac-printing-context-not-reached`;
   - `native-dialog-observation-gap`;
   - `native-dialog-appears-and-cancels`.

   The classification must cite source lines and log evidence.

## Verification

Verification for the completed result is:

```bash
rm -rf scripts/__pycache__
PYTHONDONTWRITEBYTECODE=1 python3 -m py_compile \
  scripts/test-issue-834-pdf-native-print.py
rm -rf scripts/__pycache__

python3 scripts/test-issue-834-pdf-native-print.py \
  --log-dir logs/issue-834-exp19-print-plumbing \
  --probe native-dialog \
  --allow-native-dialog-click

git diff --check
```

If Chromium code is changed, also run the Chromium workspace verification from
`chromium/AGENTS.md`:

```bash
git status --short
git -C chromium/src status --short
git -C chromium/src rev-parse --abbrev-ref HEAD
git -C chromium/src rev-parse HEAD

cd chromium/src
export PATH="/Users/astrohacker/dev/termsurf/chromium/depot_tools:$PATH"
autoninja -C out/Default libtermsurf_chromium
```

Then regenerate the Issue 834 patch archive, update `chromium/README.md` if a
new Chromium branch was created, and return to the main repo before recording
the result.

Required evidence:

- the upstream expected print chain is documented;
- the TermSurf/Roamium actual print chain is documented;
- any added trace is gated and narrowly scoped;
- the guarded native print probe records its internal preflight;
- no print job is submitted;
- the first missing hop is classified from source and log evidence;
- the result explains whether the next experiment should implement a fix or
  improve observability;
- markdown is formatted with Prettier;
- Python bytecode cache is removed after compilation;
- `git diff --check` passes;
- Chromium status, branch, HEAD, build, branch table, and patch archive evidence
  is recorded if Chromium source is changed;
- design review is recorded, all real design-review findings are fixed, the
  design is approved, and the plan commit exists before implementation begins;
- completion review is recorded before the result commit.

## Pass Criteria

This experiment passes if it identifies the first missing native print hop with
source and runtime evidence, without submitting a print job.

## Partial Criteria

This experiment is partial if the audit narrows the problem but still cannot
identify the first missing hop without broader Chromium instrumentation or an
environment outside this VM.

## Failure Criteria

This experiment fails if it submits a print job, performs an unsafe native print
click, makes product behavior changes before identifying the missing hop, or
claims a root cause without source and log evidence.

## Design Review

An adversarial Codex subagent reviewed the design with fresh context.

Initial verdict: **Changes Required**.

Required finding:

- If Chromium source instrumentation is needed, the design did not include the
  full Chromium verification hygiene required by `chromium/AGENTS.md`.

Fix:

- Added the conditional Chromium status, branch, HEAD, `autoninja`, patch
  archive, and `chromium/README.md` evidence requirements.

Re-review verdict: **Approved**.

The reviewer found no remaining Required findings.
