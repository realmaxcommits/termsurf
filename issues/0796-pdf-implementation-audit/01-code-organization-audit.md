# Experiment 1: Code Organization Audit

## Description

This experiment audits the PDF implementation for code organization,
readability, and ease of understanding. It is diagnostic only. It must not
change runtime behavior, rename symbols, move files, delete traces, or clean up
code directly.

The goal is to produce a precise cleanup plan for Experiment 2. The audit should
identify organization issues in the PDF code created across Issues 792, 793, and
794, then classify each issue by severity, confidence, affected files, and
recommended cleanup action.

This audit must focus on maintainability, not correctness or security. Security
gets its own audit after the organization cleanup is complete. Completeness gets
its own audit after the security cleanup is complete.

This experiment must receive Codex design review before it runs. After the
result is recorded, Codex must review the completed audit before Experiment 2 is
designed.

## Scope

Audit only PDF-related implementation and test code introduced or materially
changed in the recent PDF work.

Primary Chromium scope:

- `chromium/src/content/libtermsurf_chromium/ts_browser_client.*`
- `chromium/src/content/libtermsurf_chromium/ts_content_client.*`
- `chromium/src/content/libtermsurf_chromium/ts_content_renderer_client.*`
- `chromium/src/content/libtermsurf_chromium/ts_pdf_*`
- `chromium/src/content/libtermsurf_chromium/ts_plugin_*`
- `chromium/src/content/libtermsurf_chromium/extensions/ts_*pdf*`
- `chromium/src/content/libtermsurf_chromium/extensions/ts_*resource*`
- `chromium/src/content/libtermsurf_chromium/extensions/ts_*extension*`
- PDF-specific edits in Chromium PDF/printing components, including
  `pdf_view_web_plugin.cc`, `pdf_view_web_plugin_client.cc`, and
  `print_render_frame_helper.cc`.

Primary Rust and automation scope:

- `roamium/src/dispatch.rs` PDF/input paths touched for PDF work;
- Wezboard TermSurf input/resize paths touched for PDF work;
- `scripts/test-issue-794-*.py`;
- `scripts/probe-pdf-*.mjs`;
- `scripts/capture-pdf-interactions.mjs`;
- issue records for Issues 792, 793, and 794 only when needed to understand why
  code exists.

Out of scope:

- native PDF printing as a feature; Issue 795 owns that;
- behavior changes;
- security conclusions;
- completeness conclusions;
- broad refactors outside the PDF implementation;
- formatting-only churn unrelated to specific audit findings.

## Audit Method

1. Inventory the PDF implementation.

   Produce a concise map of the current PDF code by responsibility:
   - component-extension setup;
   - resource and template serving;
   - stream and MIME handling;
   - PDF viewer private/resources private APIs;
   - browser-side binders and helpers;
   - renderer-side plugin and extension setup;
   - input, resize, selection, toolbar, title, save/download, local-file, and
     print-containment paths;
   - automation harnesses.

2. Find organization issues with current evidence.

   Search for and inspect:
   - duplicated helpers or env-var parsing;
   - repeated resource lookup or URL/origin helper logic;
   - misleading file names, function names, comments, or issue-numbered log
     labels;
   - experiment-only names or traces that now read like permanent API;
   - large mixed-responsibility files or functions;
   - code that makes ownership/lifetime/call order hard to understand;
   - automation scripts that duplicate launch, fixture, or DevTools probing
     logic.

3. Categorize each finding.

   Each finding must include:
   - **Severity:** `High`, `Medium`, or `Low`.
   - **Confidence:** `High`, `Medium`, or `Low`.
   - **Files:** exact file paths and line references where practical.
   - **Problem:** what makes the code harder to understand.
   - **Why now:** why this should or should not be cleaned before later audits.
   - **Recommended cleanup:** a behavior-preserving cleanup action for
     Experiment 2.
   - **Verification needed:** the build/test/log check that would prove the
     cleanup preserved behavior.

4. Separate findings from non-findings.

   Some rough-looking code may be intentionally shaped by Chromium embedder
   constraints. Do not force cleanup if the local shape is the clearest safe
   option. Record notable non-findings where useful, especially if they prevent
   Experiment 2 from chasing cosmetic churn.

5. Produce the Experiment 2 cleanup backlog.

   The audit conclusion must list:
   - cleanup items that should be in Experiment 2;
   - cleanup items that should be deferred;
   - cleanup items that should be rejected as not worth changing;
   - the minimum verification matrix for the cleanup.

## Commands and Evidence

Use `rg` first for searches. Suggested starting points:

```bash
rg -n "issue-79[234]|TERMSURF_PDF|pdf-print|pdf-input|PdfViewer|resourcesPrivate|pdfViewerPrivate|MimeHandler|Stream|CreateInternalPlugin|PrintRenderFrameHelper" \
  chromium/src/content/libtermsurf_chromium \
  chromium/src/pdf \
  chromium/src/components/pdf \
  chromium/src/components/printing \
  roamium/src \
  wezboard/wezboard-gui/src/termsurf \
  scripts
```

```bash
rg -n "TODO|FIXME|HACK|temporary|probe|trace|issue-" \
  chromium/src/content/libtermsurf_chromium \
  chromium/src/pdf \
  chromium/src/components/pdf \
  chromium/src/components/printing \
  roamium/src \
  wezboard/wezboard-gui/src/termsurf \
  scripts
```

Also inspect the patch history in:

```bash
find chromium/patches -maxdepth 2 -path '*issue-79[234]*/*.patch' -print
```

The final audit should cite actual files and line numbers from the current
worktree, not only patch names.

## Verification

This is a documentation-only audit experiment. Verification is:

- Codex design review completed and any real design findings fixed;
- the audit result is appended to this file under `## Result`;
- findings cite current files and line references where practical;
- findings are separated into actionable cleanup, deferred cleanup, rejected
  cleanup, and non-findings;
- the conclusion defines the exact intended scope of Experiment 2;
- Codex completion review completed and any real findings fixed;
- Prettier run on this file and the issue README.

No Chromium build or Rust build is required unless the audit process changes
code, which it must not do.

## Pass Criteria

This experiment passes if it produces a clear, evidence-backed organization
audit with a concrete, behavior-preserving cleanup backlog for Experiment 2, and
Codex agrees the audit is sufficient to proceed.

## Partial Criteria

This experiment is partial if the audit identifies the right general cleanup
areas but lacks enough line-level evidence, prioritization, or verification
guidance for a safe cleanup experiment.

## Failure Criteria

This experiment fails if:

- it changes runtime behavior;
- it combines audit and cleanup;
- it drifts into security or completeness decisions beyond noting that they
  belong to later tracks;
- it proposes broad non-PDF refactors;
- it omits Codex design or completion review;
- it produces a cleanup backlog too vague to implement safely.

## Result

**Result:** Pass

The organization audit found enough line-level evidence to drive a
behavior-preserving cleanup experiment. No runtime code was changed.

### Inventory

The current PDF implementation is split across these responsibilities:

- **Extension system bootstrap:** `TsBrowserClient` wires extension process
  policy, extension URL loader factories, PDF extension frame binders, and
  extension process activation in
  `chromium/src/content/libtermsurf_chromium/ts_browser_client.cc:233`,
  `chromium/src/content/libtermsurf_chromium/ts_browser_client.cc:397`,
  `chromium/src/content/libtermsurf_chromium/ts_browser_client.cc:478`, and
  `chromium/src/content/libtermsurf_chromium/ts_browser_client.cc:514`.
- **Renderer extension and plugin setup:** `TsContentRendererClient` installs
  the extensions renderer client, grants PDF viewer access to
  `chrome://resources`, binds MimeHandlerView, installs `ExtensionFrameHelper`,
  installs the print helper delegate, externalizes PDF plugins, and creates the
  internal PDF plugin in
  `chromium/src/content/libtermsurf_chromium/ts_content_renderer_client.cc:62`,
  `chromium/src/content/libtermsurf_chromium/ts_content_renderer_client.cc:94`,
  `chromium/src/content/libtermsurf_chromium/ts_content_renderer_client.cc:109`,
  `chromium/src/content/libtermsurf_chromium/ts_content_renderer_client.cc:130`,
  and
  `chromium/src/content/libtermsurf_chromium/ts_content_renderer_client.cc:172`.
- **Resource and template serving:** the PDF component extension resource
  manager serves resource IDs and template replacements in
  `chromium/src/content/libtermsurf_chromium/extensions/ts_component_extension_resource_manager.cc:47`
  and
  `chromium/src/content/libtermsurf_chromium/extensions/ts_component_extension_resource_manager.cc:240`.
  `resourcesPrivate.getStrings` returns nearly the same load-time data as a
  dictionary in
  `chromium/src/content/libtermsurf_chromium/extensions/ts_resources_private_api.cc:41`
  and
  `chromium/src/content/libtermsurf_chromium/extensions/ts_resources_private_api.cc:242`.
- **Stream and MIME handling:** `TsPdfStreamDelegate` maps stream URLs back to
  original PDFs and controls PDF frame navigation in
  `chromium/src/content/libtermsurf_chromium/ts_pdf_stream_delegate.cc:61`,
  `chromium/src/content/libtermsurf_chromium/ts_pdf_stream_delegate.cc:128`, and
  `chromium/src/content/libtermsurf_chromium/ts_pdf_stream_delegate.cc:169`.
  `TsBrowserClient` adds `PdfNavigationThrottle`,
  `PdfURLLoaderRequestInterceptor`, and the TermSurf plugin response throttle in
  `chromium/src/content/libtermsurf_chromium/ts_browser_client.cc:163`,
  `chromium/src/content/libtermsurf_chromium/ts_browser_client.cc:176`, and
  `chromium/src/content/libtermsurf_chromium/ts_browser_client.cc:197`.
- **PDF extension APIs:** `pdfViewerPrivate.setPdfDocumentTitle` is implemented
  in
  `chromium/src/content/libtermsurf_chromium/extensions/ts_pdf_viewer_private_api.cc:32`.
  `resourcesPrivate.getStrings` is implemented in
  `chromium/src/content/libtermsurf_chromium/extensions/ts_resources_private_api.cc:242`.
- **Input, resize, and selection traces:** Wezboard and Roamium both write the
  PDF input trace in `wezboard/wezboard-gui/src/termsurf/input.rs:37` and
  `roamium/src/dispatch.rs:20`. Chromium-side PDF input/geometry probes live in
  upstream PDF files at `chromium/src/pdf/pdf_view_web_plugin.cc:182`,
  `chromium/src/pdf/pdf_view_web_plugin.cc:976`,
  `chromium/src/pdf/pdf_view_web_plugin.cc:2783`, and
  `chromium/src/pdf/pdf_view_web_plugin.cc:3098`, plus PDFium selection logs in
  `chromium/src/pdf/pdfium/pdfium_engine.cc`.
- **Print containment:** print state, print guard trace, bridge trace, and
  native-print trace code exists in
  `chromium/src/content/libtermsurf_chromium/ts_browser_client.cc:357`,
  `chromium/src/pdf/pdf_view_web_plugin.cc:157`,
  `chromium/src/components/pdf/renderer/pdf_view_web_plugin_client.cc:62`, and
  `chromium/src/components/printing/renderer/print_render_frame_helper.cc:134`.
  Native print remains out of scope for this issue.
- **Automation:** PDF regression harnesses are currently issue-numbered:
  `scripts/test-issue-794-pdf-toolbar.py`,
  `scripts/test-issue-794-protocol-scroll.py`,
  `scripts/test-issue-794-protocol-resize.py`,
  `scripts/test-issue-794-protocol-mouse.py`, and several
  `scripts/probe-pdf-*.mjs` files.

### Findings

#### 1. Permanent PDF diagnostics still use experiment-number labels

**Severity:** High  
**Confidence:** High

**Files:**

- `chromium/src/content/libtermsurf_chromium/ts_content_renderer_client.cc:74`
- `chromium/src/content/libtermsurf_chromium/ts_content_renderer_client.cc:106`
- `chromium/src/content/libtermsurf_chromium/ts_content_renderer_client.cc:117`
- `chromium/src/content/libtermsurf_chromium/ts_content_renderer_client.cc:139`
- `chromium/src/content/libtermsurf_chromium/ts_content_renderer_client.cc:178`
- `chromium/src/content/libtermsurf_chromium/ts_browser_client.cc:170`
- `chromium/src/content/libtermsurf_chromium/ts_browser_client.cc:190`
- `chromium/src/content/libtermsurf_chromium/ts_browser_client.cc:211`
- `chromium/src/content/libtermsurf_chromium/ts_browser_client.cc:252`
- `chromium/src/content/libtermsurf_chromium/ts_browser_client.cc:404`
- `chromium/src/content/libtermsurf_chromium/ts_pdf_stream_delegate.cc:65`
- `chromium/src/content/libtermsurf_chromium/ts_pdf_stream_delegate.cc:122`
- `chromium/src/content/libtermsurf_chromium/extensions/ts_pdf_viewer_private_api.cc:36`
- `chromium/src/content/libtermsurf_chromium/extensions/ts_component_extension_resource_manager.cc:233`
- `chromium/src/pdf/pdf_view_web_plugin.cc:981`
- `chromium/src/pdf/pdf_view_web_plugin.cc:2788`
- `chromium/src/pdf/pdf_view_web_plugin.cc:3107`
- `scripts/test-issue-794-protocol-resize.py:599`
- `scripts/test-issue-794-protocol-mouse.py:851`

**Problem:** Diagnostics that are now part of the maintained PDF stack still
look like temporary probes from Issues 792 and 794. The automation depends on
those historical names, so readers must know the entire experiment history to
understand current behavior. The labels also make it harder to tell which logs
are durable product diagnostics and which are stale experiment probes.

**Why now:** Security and completeness audits will need reliable log names. If
those audits keep citing `[issue-792-exp14]` and `[issue-794-exp7]`, they will
further entrench experiment history as API.

**Recommended cleanup:** In Experiment 2, rename durable PDF log labels to
stable prefixes such as `[termsurf-pdf]`, `[termsurf-pdf-input]`,
`[termsurf-pdf-resize]`, `[termsurf-pdf-title]`, and `[termsurf-pdf-print]`.
Update the automation scripts in the same change so the test contract follows
the stable names. Do not delete useful diagnostics unless they are replaced by
an equivalent stable event.

**Verification needed:** Run the PDF toolbar, protocol scroll, protocol resize,
and protocol mouse harnesses, or at minimum run the renamed log-matching parts
against a fresh Roamium run and prove each previous required hop still has a
stable replacement.

#### 2. PDF trace/env parsing is duplicated across layers

**Severity:** High  
**Confidence:** High

**Files:**

- `wezboard/wezboard-gui/src/termsurf/input.rs:37`
- `wezboard/wezboard-gui/src/termsurf/input.rs:41`
- `roamium/src/dispatch.rs:20`
- `roamium/src/dispatch.rs:26`
- `chromium/src/content/libtermsurf_chromium/ts_browser_client.cc:65`
- `chromium/src/content/libtermsurf_chromium/ts_browser_client.cc:77`
- `chromium/src/pdf/pdf_view_web_plugin.cc:157`
- `chromium/src/pdf/pdf_view_web_plugin.cc:182`
- `chromium/src/pdf/pdf_view_web_plugin.cc:188`
- `chromium/src/pdf/pdf_view_web_plugin.cc:217`
- `chromium/src/pdf/pdf_view_web_plugin.cc:238`
- `chromium/src/components/pdf/renderer/pdf_view_web_plugin_client.cc:62`
- `chromium/src/components/pdf/renderer/pdf_view_web_plugin_client.cc:70`
- `chromium/src/components/pdf/renderer/pdf_view_web_plugin_client.cc:97`
- `chromium/src/components/printing/renderer/print_render_frame_helper.cc:134`
- `chromium/src/content/libtermsurf_chromium/extensions/ts_component_extension_resource_manager.cc:28`
- `chromium/src/content/libtermsurf_chromium/extensions/ts_resources_private_api.cc:28`

**Problem:** Each layer reimplements env-var probing, default paths, command
line switch names, path validation, and append logic. The Rust trace helpers are
nearly identical but differ in truncation behavior: Wezboard truncates the trace
file during initialization, while Roamium only appends. Chromium has several
independent C++ helpers for the same print trace switches and env vars. That is
hard to reason about and makes future trace changes easy to apply
inconsistently.

**Why now:** This is organization debt, not a behavior bug. It should be cleaned
before the security audit so the later audit can review one trace/env contract
instead of several local copies.

**Recommended cleanup:** In Experiment 2, introduce small shared helpers without
changing behavior:

- a Rust helper for PDF input trace path initialization and append behavior,
  used by Wezboard and Roamium while preserving the existing prefix
  (`wezboard`/`roamium`) and truncation semantics;
- Chromium-side constants/helpers for PDF print trace switch/env names and trace
  path resolution, placed in the narrowest reusable PDF support file available
  to the touched targets.

If moving helpers across Chromium component boundaries would force broad BUILD
dependency churn, keep the shared helper inside the smallest existing target and
only deduplicate callsites within that target in Experiment 2.

**Verification needed:** Run `cargo fmt` after Rust edits. Run the PDF input
trace harnesses and the print-guard toolbar harness with tracing enabled, then
confirm the same trace files are created and contain the same event classes.

#### 3. `TsContentRendererClient` mixes too many PDF responsibilities

**Severity:** Medium  
**Confidence:** High

**Files:**

- `chromium/src/content/libtermsurf_chromium/ts_content_renderer_client.cc:41`
- `chromium/src/content/libtermsurf_chromium/ts_content_renderer_client.cc:62`
- `chromium/src/content/libtermsurf_chromium/ts_content_renderer_client.cc:94`
- `chromium/src/content/libtermsurf_chromium/ts_content_renderer_client.cc:109`
- `chromium/src/content/libtermsurf_chromium/ts_content_renderer_client.cc:130`
- `chromium/src/content/libtermsurf_chromium/ts_content_renderer_client.cc:172`

**Problem:** The renderer client now owns unrelated setup details: a print
helper delegate, origin grants for `chrome://resources`, extension renderer
startup, MimeHandlerView binding, extension frame helper lifetime, PDF plugin
externalization rules, and direct internal plugin creation. The file is not
large, but the responsibilities are different enough that it reads as a
chronological experiment log rather than as intentional embedder structure.

**Why now:** This cleanup makes later security review easier. The security audit
needs to inspect the PDF renderer boundary; currently that boundary is spread
through callback overrides and local helpers in the generic renderer client.

**Recommended cleanup:** Extract PDF-specific renderer support into a small
`ts_pdf_renderer_support.*` helper or similarly named file. Keep
`TsContentRendererClient` as the Chromium override owner, but move the
implementation details for PDF origin grants, MimeHandlerView binding, print
helper delegate construction, `IsPluginHandledExternally`, and
`OverrideCreatePlugin` helper logic behind clearly named functions.

**Verification needed:** Build Chromium/Roamium after the extraction and run the
automated PDF render, toolbar, scroll, resize, and selection tests. Because this
touches Chromium code, Experiment 2 needs a fresh Chromium branch.

#### 4. `TsBrowserClient` is an overloaded PDF orchestration point

**Severity:** Medium  
**Confidence:** High

**Files:**

- `chromium/src/content/libtermsurf_chromium/ts_browser_client.cc:163`
- `chromium/src/content/libtermsurf_chromium/ts_browser_client.cc:176`
- `chromium/src/content/libtermsurf_chromium/ts_browser_client.cc:197`
- `chromium/src/content/libtermsurf_chromium/ts_browser_client.cc:233`
- `chromium/src/content/libtermsurf_chromium/ts_browser_client.cc:306`
- `chromium/src/content/libtermsurf_chromium/ts_browser_client.cc:357`
- `chromium/src/content/libtermsurf_chromium/ts_browser_client.cc:397`
- `chromium/src/content/libtermsurf_chromium/ts_browser_client.cc:441`
- `chromium/src/content/libtermsurf_chromium/ts_browser_client.cc:478`
- `chromium/src/content/libtermsurf_chromium/ts_browser_client.cc:514`

**Problem:** `TsBrowserClient` now contains navigation throttles, URL loader
throttles, PDF URL request interceptors, extension frame binders, PDF host
binders, renderer print env forwarding, extension URL loader factories, process
per site policy, extension URL handling, process map insertion, renderer
activation, and `chrome://resources` grants. That makes the generic browser
client the place to understand every PDF browser-side concern.

**Why now:** A later security audit will need to reason about browser-side
origin and process grants. Extracting a named PDF browser support layer first
will make that audit more precise and less error-prone.

**Recommended cleanup:** Extract behavior-preserving PDF browser helpers into
`ts_pdf_browser_support.*` or a small set of narrowly named helpers. Leave the
Chromium virtual override methods in `TsBrowserClient`, but delegate PDF
specific work to functions such as `AddTsPdfNavigationThrottles`,
`AddTsPdfURLLoaderThrottles`, `RegisterTsPdfFrameBinders`,
`AppendTsPdfRendererSwitches`, and `MaybeGrantTsPdfExtensionProcessAccess`.

**Verification needed:** Build Roamium and run the same PDF regression matrix as
Finding 3. The cleanup should also smoke-test a normal non-PDF page to prove the
generic browser client still delegates to `ShellContentBrowserClient` correctly.

#### 5. PDF load-time data is duplicated between template replacements and `resourcesPrivate.getStrings`

**Severity:** Medium  
**Confidence:** High

**Files:**

- `chromium/src/content/libtermsurf_chromium/extensions/ts_component_extension_resource_manager.cc:47`
- `chromium/src/content/libtermsurf_chromium/extensions/ts_component_extension_resource_manager.cc:209`
- `chromium/src/content/libtermsurf_chromium/extensions/ts_component_extension_resource_manager.cc:217`
- `chromium/src/content/libtermsurf_chromium/extensions/ts_component_extension_resource_manager.cc:218`
- `chromium/src/content/libtermsurf_chromium/extensions/ts_resources_private_api.cc:41`
- `chromium/src/content/libtermsurf_chromium/extensions/ts_resources_private_api.cc:205`
- `chromium/src/content/libtermsurf_chromium/extensions/ts_resources_private_api.cc:227`
- `chromium/src/content/libtermsurf_chromium/extensions/ts_resources_private_api.cc:228`

**Problem:** The two files contain parallel lists of PDF localized strings and
the same TermSurf-specific load-time flags (`presetZoomFactors`,
`pdfOopifEnabled`, `printingEnabled`, and `termsurfPdfPrintBridgeTrace`). One
path builds `ui::TemplateReplacements`, the other builds `base::DictValue`.
Keeping the lists in sync by hand is brittle and obscures which values are
required by the viewer at template load time versus which are returned by the
extension API.

**Why now:** This is a pure organization cleanup that reduces future
completeness risk. The completeness audit will later ask whether toolbar,
strings, and feature flags are complete; it should audit a single source of
truth rather than two hand-maintained lists.

**Recommended cleanup:** Create one shared PDF load-time-data helper that owns
the resource-name/resource-id table and TermSurf feature flag values, with
adapters for `ui::TemplateReplacements` and `base::DictValue`. Keep both
existing public call paths intact.

**Verification needed:** Run the toolbar probe that verifies resource strings,
controls, zoom factors, and print bridge trace visibility.

#### 6. PDF automation harnesses duplicate protocol, fixture, and DevTools plumbing

**Severity:** Medium  
**Confidence:** High

**Files:**

- `scripts/test-issue-794-protocol-scroll.py:1`
- `scripts/test-issue-794-protocol-scroll.py:36`
- `scripts/test-issue-794-protocol-scroll.py:620`
- `scripts/test-issue-794-protocol-resize.py:1`
- `scripts/test-issue-794-protocol-resize.py:36`
- `scripts/test-issue-794-protocol-resize.py:806`
- `scripts/test-issue-794-protocol-mouse.py:1`
- `scripts/test-issue-794-protocol-mouse.py:35`
- `scripts/test-issue-794-protocol-mouse.py:1076`
- `scripts/test-issue-794-pdf-toolbar.py:300`
- `scripts/test-issue-794-pdf-toolbar.py:389`
- `scripts/probe-pdf-toolbar.mjs:6`
- `scripts/probe-pdf-toolbar-events.mjs:7`
- `scripts/capture-pdf-interactions.mjs:8`
- `scripts/probe-pdf-save-print-title-local.mjs:6`

**Problem:** The Python protocol harnesses repeat protobuf wire helpers, socket
setup, fixture server startup, Roamium launch, trace env setup, and DevTools
discovery. The JavaScript probes repeat argument parsing, DevTools connection,
`evaluate`, screenshot capture, and state-collection helpers. The duplication
makes behavior changes expensive and increases the chance that a future
completeness test exercises a slightly different harness than earlier tests.

**Why now:** This should be cleaned during the organization track. It is not a
runtime product change, and later completeness work will likely add more PDF
tests. Adding tests on top of the current duplication would make the cleanup
larger.

**Recommended cleanup:** Add shared PDF harness modules for Python and
JavaScript, then migrate existing scripts incrementally. Keep compatibility
wrappers with the old issue-numbered filenames if that avoids breaking existing
commands in issue records. Use durable names for the shared modules, such as
`scripts/termsurf_pdf_protocol_harness.py` and
`scripts/termsurf_pdf_devtools.mjs`.

**Verification needed:** Run each migrated harness once, or run old wrappers
that delegate to the new shared helpers and compare their summary JSON shape to
the current output.

### Deferred Cleanup

- **Native print implementation:** Print containment and print trace code is
  present in the PDF stack, but native PDF printing is owned by Issue 795. This
  issue should only rename or deduplicate print diagnostics when that is needed
  for organization. It must not attempt to make native print functional.
- **Broad upstream Chromium TODOs:** The `rg` sweep found many upstream Chromium
  TODOs in `chromium/src/pdf` and `chromium/src/components/printing`. They are
  not TermSurf PDF organization debt unless they were added or made ambiguous by
  the recent TermSurf work.
- **Patch archive layout:** The PDF patch archives under `chromium/patches`
  contain cumulative issue branches and are noisy by nature. That is expected
  for the current Chromium fork workflow and should not be cleaned up in
  Experiment 2.

### Rejected Cleanup

- Do not move PDF stream, MimeHandler, or extension support out of Chromium just
  to make the main repo smaller. The recent PDF work intentionally mirrors
  Electron's Chromium-side embedder approach, so organization cleanup should
  improve names and boundaries inside that approach rather than replace it.
- Do not remove diagnostics merely because they mention old experiments. The
  useful events should be renamed to stable labels, not deleted.
- Do not combine behavior-preserving cleanup with security tightening. If
  Experiment 2 notices a security concern while moving code, record it for the
  security audit instead of changing behavior in the organization cleanup.

## Conclusion

Experiment 1 passes. The PDF implementation is functional but still carries
several signs of its experiment-driven growth: issue-numbered diagnostics,
duplicated trace/env helpers, duplicated load-time PDF data, overloaded browser
and renderer clients, and copied automation harness plumbing.

Experiment 2 should be a behavior-preserving organization cleanup with this
scope:

1. Rename durable PDF diagnostics from issue-numbered labels to stable TermSurf
   PDF labels and update matching automation.
2. Deduplicate PDF trace/env helpers where that can be done without broad
   Chromium dependency churn.
3. Extract PDF renderer support from `TsContentRendererClient`.
4. Extract PDF browser support from `TsBrowserClient`.
5. Centralize PDF load-time data generation for template replacements and
   `resourcesPrivate.getStrings`.
6. Begin harness consolidation, prioritizing the duplicated Python protocol
   helpers and keeping old script entrypoints as compatibility wrappers if
   needed.

Minimum verification for Experiment 2:

- Chromium/Roamium builds successfully on the new Chromium branch.
- Rust formatting runs if Rust trace helper code is touched.
- PDF render, toolbar, scroll, resize, mouse/selection, title/save/local-file,
  and print-containment harnesses still pass or produce the same known non-print
  result.
- A normal non-PDF page still loads.
- The updated logs prove that every old required PDF diagnostic event has a
  stable replacement.
