# Experiment 5: Completeness Audit

## Description

This experiment audits whether TermSurf's non-print PDF viewer is complete
enough after Issues 792-794 and the Issue 796 security cleanup. It is diagnostic
only. It must not change Chromium, Rust, JavaScript, Python, protocol, fixtures,
or runtime behavior.

The audit starts from Chromium branch `148.0.7778.97-issue-796-exp4`. The goal
is to produce a concrete Experiment 6 cleanup plan that either implements the
remaining non-print gaps or explicitly documents them as follow-up issues.

Native PDF printing is out of scope. Issue 795 owns native PDF printing. This
audit may mention print only to confirm it remains intentionally deferred and
does not block the non-print PDF viewer scope.

This experiment must receive Codex design review before it runs. After the audit
result is recorded, Codex must review the completed audit before Experiment 6 is
designed.

## Scope

Audit only the PDF implementation and test coverage created or materially
changed by Issues 792, 793, 794, and 796.

Primary Chromium scope:

- `chromium/src/content/libtermsurf_chromium/` PDF viewer, extension,
  MimeHandler, stream, resource, title, toolbar, print-containment, and
  input/resize helper code;
- TermSurf Chromium patches under `chromium/patches/issue-792/`,
  `chromium/patches/issue-793/`, `chromium/patches/issue-794-*`, and
  `chromium/patches/issue-796-exp*`;
- any PDF-specific patches under `chromium/src/pdf/` and
  `chromium/src/components/printing/`.

Primary Rust, JavaScript, and automation scope:

- Roamium PDF/input/resize dispatch paths touched for PDF behavior;
- Wezboard PDF input/resize routing touched for PDF behavior;
- `scripts/test-issue-794-*.py`;
- `scripts/termsurf_pdf_protocol_harness.py`;
- `scripts/probe-pdf-*.mjs`;
- `scripts/capture-pdf-interactions.mjs`;
- `scripts/test-issue-796-pdf-security.py`;
- logs from the latest passing Issue 794 and Issue 796 PDF runs where needed.

Out of scope:

- native PDF printing implementation;
- unrelated browser features;
- broad upstream Chromium/PDFium feature parity outside the TermSurf embedder
  integration;
- large accessibility work unless TermSurf specifically broke or omitted a
  required PDF viewer integration point;
- changing code during this audit.

## Audit Method

### 1. Build a feature inventory

Create a table of expected non-print PDF viewer behavior. At minimum include:

- full-page PDF rendering;
- embedded PDF rendering;
- HTTP and HTTPS PDFs;
- `file://` PDFs;
- extensionless local PDFs;
- extensionless HTTP PDFs;
- titled and untitled PDFs;
- restricted PDFs;
- PDF permission-restricted documents;
- copy restrictions;
- save/download restrictions;
- disabled toolbar states for document restrictions;
- scroll wheel;
- keyboard scroll/navigation;
- mouse click and focus;
- text selection;
- copy;
- resize and reflow;
- toolbar visibility;
- toolbar page navigation;
- zoom in/out;
- fit-to-page / fit-to-width;
- rotate;
- save/download;
- title propagation;
- internal PDF links and external links;
- find/search within PDF;
- forms;
- annotations;
- password-protected or error-page PDFs;
- context menu behavior;
- accessibility/searchify behavior if applicable;
- normal HTML regression behavior.

For each item, classify the current status as one of:

- `Implemented and automated`;
- `Implemented but manual only`;
- `Implemented but weakly automated`;
- `Missing`;
- `Intentionally out of scope`;
- `Unknown`.

Each row must cite evidence: issue result, test script, log file, code path, or
local source comparison. If evidence is weak or old, mark it weak instead of
treating it as proof.

### 2. Compare against Chrome and Electron where useful

Use local source references to compare TermSurf's PDF behavior against Chrome
and Electron for missing or uncertain features. Focus on the embedder
integration points TermSurf owns, not PDFium internals.

Reference areas include:

- PDF viewer extension UI/resources;
- stream handoff and MimeHandler behavior;
- toolbar actions;
- save/download;
- title propagation;
- find/search;
- link handling;
- forms and annotations;
- accessibility/searchify setup;
- context menu hooks;
- error-page or password handling.

The audit does not need exact Chrome parity for every feature. It must identify
whether a missing feature is:

- required for a normal usable PDF viewer;
- nice-to-have but acceptable as follow-up;
- already provided by upstream PDF viewer code once TermSurf wiring is correct;
- blocked by a TermSurf embedder gap.

### 3. Audit automation coverage

Map every non-print feature to existing automation.

Questions:

- Which features are covered by `scripts/test-issue-794-pdf-toolbar.py`?
- Which features are covered by protocol scroll/resize/mouse harnesses?
- Which features are only covered by old one-off probes?
- Which features are manually tested but automatable?
- Which existing harnesses produce partial or ambiguous summaries?
- Which assertions are too broad, too weak, or too dependent on screenshots?

The audit must specifically address the known `save-print-title-local` harness
nuance from Experiment 4: it exited successfully and verified save/title/print,
but its local-parity DevTools wheel subcheck reported `partial` while the
dedicated protocol scroll harness passed. Decide whether Experiment 6 should fix
that harness, document it, or replace the local-parity subcheck with a better
automated assertion.

### 4. Audit user-facing completeness gaps

For each missing or weak feature, determine the user impact:

- Does the user lose core PDF viewing ability?
- Does the user lose common document workflow behavior?
- Is the behavior browser-standard but rarely needed?
- Is it blocked by native UI, OS permissions, or cross-process architecture?
- Can it be solved in the TermSurf embedder layer without large new
  infrastructure?

Native print must be classified as `Intentionally out of scope` and linked to
Issue 795, not pulled into Experiment 6.

### 5. Produce the Experiment 6 cleanup backlog

The conclusion must split the audit output into:

- required fixes for Experiment 6;
- automation cleanup required for Experiment 6;
- acceptable follow-up issues outside this issue;
- intentional non-print scope exclusions;
- non-findings where current behavior is sufficient.

If the audit finds no missing required non-print behavior, Experiment 6 should
still be designed to add the smallest useful documentation or automation cleanup
that makes the completeness boundary easier to maintain.

## Commands and Evidence

Use `rg` first for source searches. Suggested starting points:

```bash
rg -n "pdf|PDF|viewer-toolbar|viewer-page-selector|zoom|fit|rotate|find|search|annotation|form|password|download|save|copy|selection|link|context|accessibility|searchify|print" \
  chromium/src/content/libtermsurf_chromium \
  chromium/src/pdf \
  chromium/src/chrome/browser/resources/pdf \
  chromium/src/components/pdf \
  roamium/src \
  wezboard/wezboard-gui/src/termsurf \
  scripts
```

```bash
rg -n "status|first_failing_hop|localParity|titlePropagation|saveDownload|print|toolbar|scroll|resize|selection|copy|link|find|form|annotation" \
  scripts/test-issue-794-*.py \
  scripts/probe-pdf-*.mjs \
  scripts/capture-pdf-interactions.mjs \
  scripts/termsurf_pdf_protocol_harness.py
```

Suggested local reference searches:

```bash
rg -n "pdfViewerPrivate|PDFViewer|viewer-toolbar|viewer-page-selector|find|search|annotation|form|password|download|save|copy|selection|contextMenu|accessibility|searchify" \
  vendor/electron \
  chromium/src/chrome/browser/resources/pdf \
  chromium/src/chrome/browser/pdf \
  chromium/src/components/pdf \
  chromium/src/pdf
```

If a local Electron checkout is unavailable, note that in the result and rely on
the local Chromium source plus existing issue records.

Useful current logs:

- `logs/issue-796-exp4-security-rerun/`
- `logs/issue-796-exp4-save-title-local-rerun/`
- `logs/issue-796-exp4-save-title-local-print-intercept/`
- `logs/issue-796-exp4-protocol-scroll/`
- `logs/issue-796-exp4-protocol-resize/`
- `logs/issue-796-exp4-protocol-mouse-click/`
- `logs/issue-796-exp4-protocol-mouse-select-copy/`
- `logs/issue-796-exp4-non-pdf-html/`

Do not treat these logs as proof for features they do not assert. Use them only
where the script summaries and captured artifacts directly cover the feature.

## Verification

This is a documentation-only audit experiment. Verification is:

- Codex design review completed and real design findings fixed;
- no runtime code changed;
- the audit result is appended to this file under `## Result`;
- the feature inventory table is present;
- every feature status cites evidence or explicitly says evidence is missing;
- automation gaps are separated from product/feature gaps;
- native print is left to Issue 795 and not re-scoped;
- the Experiment 6 cleanup backlog is concrete enough to implement;
- Codex completion review completed and real findings fixed;
- Prettier run on this file and the issue README.

No Chromium, Rust, Roamium, or Wezboard build is required unless the audit
accidentally changes code. It must not change code.

## Pass Criteria

This experiment passes if it produces an evidence-backed completeness audit that
identifies the actual non-print PDF cleanup backlog for Experiment 6, or proves
that no required non-print feature work remains and defines the minimum
documentation/automation cleanup needed to preserve that conclusion.

## Partial Criteria

This experiment is partial if it identifies likely completeness gaps but lacks
enough evidence, user-impact classification, or verification guidance to safely
design Experiment 6.

## Failure Criteria

This experiment fails if:

- it changes runtime behavior;
- it combines audit and cleanup;
- it treats native PDF printing as in scope;
- it claims completeness without checking rendering, input, toolbar, save,
  title, local-file, embedded, and normal-HTML behavior;
- it treats old issue notes as proof without checking whether current code and
  current harnesses still support the claim;
- it omits Codex design or completion review;
- it produces a cleanup backlog too vague to implement safely.

## Result

**Result:** Pass

This audit reviewed the current non-print PDF viewer after Experiment 4 on
Chromium branch `148.0.7778.97-issue-796-exp4`. No runtime code was changed.

### Feature Inventory

| Feature                                  | Status                           | Evidence                                                                                                                                       | Notes                                                                                                                                                                   |
| ---------------------------------------- | -------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Full-page PDF rendering                  | Implemented and automated        | `logs/issue-796-exp4-security-rerun/positive-pdf/pdf-toolbar-summary.json`; `logs/issue-796-exp4-protocol-scroll/protocol-scroll-summary.json` | Real PDF loads create the PDF extension target and protocol scroll succeeds.                                                                                            |
| Embedded PDF rendering                   | Implemented but weakly automated | `logs/issue-796-exp4-save-title-local-rerun/save-print-title-local/save-print-title-local-summary.json`                                        | `embeddedTitle.status = "pass"` proves the embedded path/title case, but there is no standalone embedded scroll/input matrix.                                           |
| HTTP PDF                                 | Implemented and automated        | Issue 796 Exp 4 security probe and save/title/local runs use `http://127.0.0.1:9799/bitcoin.pdf`.                                              | Covered by current local fixture server.                                                                                                                                |
| HTTPS PDF                                | Implemented but weakly automated | Issue 794 history used web PDFs; current deterministic runs use local HTTP.                                                                    | Current automation avoids the network. A future smoke can add a local TLS fixture if needed, but HTTP exercises the same PDF viewer path after response classification. |
| `file://` PDF                            | Implemented but weakly automated | `logs/issue-796-exp4-save-title-local-rerun/save-print-title-local/save-print-title-local-summary.json` local parity rows.                     | Title/render evidence exists; local parity scroll subcheck is weak/partial.                                                                                             |
| Extensionless local PDF                  | Implemented but weakly automated | Same local parity summary, `file-extensionless` row.                                                                                           | Rendering/title evidence exists; scroll proof comes from the general protocol scroll harness rather than local parity.                                                  |
| Extensionless HTTP PDF                   | Implemented but weakly automated | Same local parity summary, `http-extensionless` row.                                                                                           | Same local-parity weakness as above.                                                                                                                                    |
| Titled PDFs                              | Implemented and automated        | `titlePropagationPass = true` in save/title/local summaries; `ts_pdf_viewer_private_api.cc` title path.                                        | Current title propagation is covered.                                                                                                                                   |
| Untitled PDFs                            | Implemented but weakly automated | `http-untitled` and `file-untitled` local parity rows; Issue 794 Exp 14/15 records.                                                            | Evidence exists, but the local parity harness status is partial.                                                                                                        |
| Permission-restricted PDFs               | Implemented but weakly automated | `ts_pdf_document_helper_client.cc` logs `pdf-content-restrictions`; save/title/local print statuses include restricted-document handling code. | No dedicated restricted fixture currently proves every disabled state.                                                                                                  |
| Copy restrictions                        | Unknown                          | `chromium/src/pdf/content_restriction.h`; protocol selection/copy harness proves normal copy only.                                             | No restricted-copy fixture or negative copy assertion.                                                                                                                  |
| Save/download restrictions               | Unknown                          | Save/download probe proves ordinary download; Chromium plugin exposes content restrictions.                                                    | No restricted-save fixture or disabled download-control assertion.                                                                                                      |
| Disabled toolbar states for restrictions | Unknown                          | Toolbar inventory captures disabled flags, but current fixture is not restriction-specific.                                                    | Needs a restricted fixture check.                                                                                                                                       |
| Scroll wheel                             | Implemented and automated        | `logs/issue-796-exp4-protocol-scroll/protocol-scroll-summary.json` reports `first_failing_hop = "no-failure-observed"`.                        | Protocol-level wheel path is covered.                                                                                                                                   |
| Keyboard scroll/navigation               | Unknown                          | `scripts/test-issue-794-protocol-mouse.py --action key-select-copy` proves keyboard delivery for selection/copy only.                          | Keyboard page/scroll navigation is not separately asserted.                                                                                                             |
| Mouse click and focus                    | Implemented and automated        | `logs/issue-796-exp4-protocol-mouse-click/protocol-mouse-summary.json`.                                                                        | No failure observed.                                                                                                                                                    |
| Text selection                           | Implemented and automated        | `logs/issue-796-exp4-protocol-mouse-select-copy/protocol-mouse-summary.json`; PDFium selection log checks in script.                           | Covered through key-select-copy.                                                                                                                                        |
| Copy                                     | Implemented and automated        | Same protocol mouse summary.                                                                                                                   | Normal unrestricted copy is covered.                                                                                                                                    |
| Resize and reflow                        | Implemented and automated        | `logs/issue-796-exp4-protocol-resize/protocol-resize-summary.json`.                                                                            | No failure observed.                                                                                                                                                    |
| Toolbar visibility                       | Implemented and automated        | `logs/issue-796-exp4-security-rerun/positive-pdf/pdf-toolbar-summary.json`; control inventories.                                               | Toolbar target and controls are visible.                                                                                                                                |
| Toolbar page navigation                  | Implemented but weakly automated | `probe-pdf-toolbar.mjs` inventories `viewer-page-selector`; old Issue 794 toolbar runs covered controls.                                       | Current post-security run did not separately assert page selector changes.                                                                                              |
| Zoom in/out                              | Implemented but weakly automated | `logs/issue-794-exp20-regression-toolbar-events-20260530-151446/toolbar-events/toolbar-events-summary.json` status `pass`.                     | Evidence is recent but pre-Experiment 4. Experiment 6 should rerun this on the current branch.                                                                          |
| Fit-to-page / fit-to-width               | Implemented but weakly automated | Same toolbar-events summary status `pass`.                                                                                                     | Same caveat as zoom.                                                                                                                                                    |
| Rotate                                   | Implemented but weakly automated | Same toolbar-events summary status `pass`; print-intercept run also observed rotate callback plumbing.                                         | Same caveat as zoom.                                                                                                                                                    |
| Save/download                            | Implemented and automated        | Save/title/local summary reports `saveDownload.status = "download-file-created"`.                                                              | Ordinary download is covered.                                                                                                                                           |
| Title propagation                        | Implemented and automated        | Save/title/local summaries report `titlePropagationPass = true`; `ts_pdf_viewer_private_api.cc`.                                               | Covered.                                                                                                                                                                |
| Internal PDF links                       | Unknown                          | No current harness fixture or log asserts clicking links inside a PDF.                                                                         | Likely upstream PDFium/plugin behavior, but TermSurf input routing needs a fixture.                                                                                     |
| External links from PDFs                 | Unknown                          | No current harness fixture asserts URL/navigation behavior after clicking a PDF link.                                                          | Important user workflow; should be automated.                                                                                                                           |
| Find/search within PDF                   | Unknown                          | Chromium PDF plugin has find APIs (`pdf_view_web_plugin.h`), but TermSurf has no current PDF find harness.                                     | This is a common PDF viewer feature and should be probed.                                                                                                               |
| Forms                                    | Unknown                          | Chromium plugin has form loader/filler code; no TermSurf fixture or harness.                                                                   | Needs at least a simple form fixture decision: support now or follow-up.                                                                                                |
| Annotations                              | Unknown / likely follow-up       | Chromium PDF resources include Ink/text annotation managers; TermSurf does not have dedicated annotation automation.                           | More complex than core viewing; likely follow-up unless current UI already works trivially.                                                                             |
| Password-protected PDFs                  | Unknown                          | Chromium viewer has password prompt plumbing; no TermSurf fixture or native/HTML prompt assertion.                                             | Needs probe or explicit follow-up.                                                                                                                                      |
| Error-page PDFs                          | Unknown                          | No malformed/404 PDF fixture in current harness matrix.                                                                                        | Needs small fixture.                                                                                                                                                    |
| Context menu behavior                    | Unknown / likely follow-up       | No TermSurf PDF context-menu harness.                                                                                                          | TermSurf context-menu story is broader than PDF; should not block core PDF viewing unless user workflow requires it.                                                    |
| Accessibility/searchify                  | Unknown / likely follow-up       | Chromium plugin has accessibility/searchify code; TermSurf has no coverage.                                                                    | Large surface; document as follow-up unless a concrete non-print requirement appears.                                                                                   |
| Normal HTML regression                   | Implemented and automated        | `logs/issue-796-exp4-non-pdf-html/protocol-resize-summary.json` reports no failure.                                                            | HTML resize works; logs include normal PDF subsystem startup but no fake-extension process/grant handling.                                                              |
| Native PDF printing                      | Intentionally out of scope       | Issue 795.                                                                                                                                     | Do not pull into Experiment 6.                                                                                                                                          |

### Chrome and Electron Comparison

TermSurf now follows the Electron-style embedder-owned PDF stack for the core
viewer path: component extension resources, stream handoff, MimeHandler/OOPIF
setup, PDF viewer APIs, and plugin creation. Local Chromium source confirms
Chrome's PDF viewer contains additional non-print feature surfaces that TermSurf
has not proven end to end:

- find/search hooks in `chromium/src/pdf/pdf_view_web_plugin.h`;
- save and content-restriction plumbing in
  `chromium/src/pdf/pdf_view_web_plugin.h` and
  `chromium/src/pdf/content_restriction.h`;
- forms and accessibility state in `chromium/src/pdf/pdf_view_web_plugin.h`;
- annotation UI/resource code under
  `chromium/src/chrome/browser/resources/pdf/ink2_manager.ts`;
- password prompt handling references in
  `chromium/src/chrome/browser/resources/pdf/pdf_viewer_base.ts`.

The audit does not prove these features are missing in product code. Many are
likely already available through upstream PDFium/Chrome viewer code once the
TermSurf input and resource path is active. The gap is evidence: TermSurf has no
current fixtures or automated assertions for several common PDF workflows.

### Automation Coverage Findings

Current automation is strongest for the core browser-embedding plumbing:

- full-page rendering and toolbar inventory;
- protocol scroll;
- protocol resize;
- protocol mouse click;
- selection/copy;
- title propagation;
- ordinary save/download;
- default print non-click behavior and contained print intercept;
- security boundary for fake extension URLs;
- normal HTML resize smoke.

Current automation is weak or missing for:

- local parity scroll inside `probe-pdf-save-print-title-local.mjs`;
- keyboard scroll/page navigation;
- page selector/page navigation after security cleanup;
- current post-security toolbar event run for zoom/fit/rotate;
- PDF links;
- find/search;
- permission-restricted copy/save/disabled toolbar states;
- forms;
- password/error pages;
- annotations;
- context menu;
- accessibility/searchify.

The Experiment 4 local-parity nuance is real. The save/title/local harness exits
0 and proves save/title/embedded title/print states, but marks overall status
`partial` because its DevTools wheel subcheck does not observe local-parity
scroll movement. The protocol scroll harness separately proves TermSurf
scrolling through Roamium and Chromium. Experiment 6 should fix or replace this
local-parity assertion so future runs are not ambiguous.

### User-Facing Gap Classification

Required for a normal non-print PDF viewer:

- reliable full-page/embedded rendering;
- scrolling;
- resizing;
- selection/copy;
- save/download;
- toolbar zoom/fit/rotate/page controls;
- title propagation;
- local-file and extensionless parity;
- links inside PDFs;
- find/search;
- password/error handling;
- restrictions/disabled states for copy and save.

Acceptable as follow-up if not already trivial:

- annotations;
- forms beyond simple display/interact;
- context menu;
- accessibility/searchify.

Intentionally out of scope:

- native PDF printing, tracked by Issue 795.

### Experiment 6 Backlog

Required cleanup for Experiment 6:

1. Repair or replace the local-parity scroll assertion in
   `probe-pdf-save-print-title-local.mjs` so the save/title/local harness can
   report a clean pass when the underlying protocol scroll path works.
2. Add keyboard scroll/page navigation assertions for the PDF viewer.
3. Add a current post-security toolbar-events verification for zoom in/out, fit,
   rotate, and page selector/page navigation.
4. Add a small link fixture and automated probe for internal PDF links and
   external PDF links.
5. Add a find/search probe for a known text string in the Bitcoin PDF or a small
   deterministic fixture.
6. Add restricted-document fixtures or a clear fixture-generation plan for copy
   and save/download restrictions, and assert disabled toolbar states where
   applicable.
7. Add password/error fixtures or explicitly open follow-up issues if Chromium's
   viewer path requires more infrastructure than fits in Experiment 6.

Acceptable follow-up issues outside Experiment 6 if they prove large:

- forms;
- annotations;
- context menu;
- accessibility/searchify.

Non-findings:

- No evidence from this audit suggests the core stream/resource/plugin
  architecture needs another rewrite.
- No evidence suggests the Experiment 4 security cleanup regressed the core PDF
  viewer path.
- Native print remains correctly deferred to Issue 795.

## Conclusion

The non-print PDF viewer is usable and the core embedding path is covered, but
"complete" is not yet proven. Experiment 6 should focus on tightening automation
and filling common workflow probes, especially local parity scroll, toolbar
event coverage after the security branch, links, find/search, document
restrictions, and password/error behavior. Large advanced surfaces such as
annotations, forms, context menus, and accessibility/searchify can be documented
as follow-up issues if they exceed this audit issue's cleanup budget.

Codex completion review passed after two fixes: keyboard scroll/navigation was
downgraded to `Unknown` and added to the Experiment 6 backlog, and
zoom/fit/rotate were downgraded to `Implemented but weakly automated` until
rerun on the current branch.
