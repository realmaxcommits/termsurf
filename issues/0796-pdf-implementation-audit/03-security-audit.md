# Experiment 3: Security Audit

## Description

This experiment audits the current PDF implementation for security issues. It is
diagnostic only. It must not change Chromium, Rust, JavaScript, Python, protocol
surface, fixtures, or runtime behavior.

The goal is to produce a concrete security cleanup plan for Experiment 4. The
audit should identify any way an untrusted PDF or web page could escape the
intended PDF viewer path, gain broader extension/API/resource/file access than
intended, confuse stream/frame ownership, trigger unsafe lifetime behavior, or
activate automation-only code in production.

This audit runs after the code organization cleanup, so it should review the
current helper structure on branch `148.0.7778.97-issue-796-exp2`, not the older
Issue 792-794 source layout. Native PDF printing remains out of scope except for
the existing print-containment and print-intercept guards that touch PDF viewer
safety.

This experiment must receive Codex design review before it runs. After the audit
result is recorded, Codex must review the completed audit before Experiment 4 is
designed.

## Scope

Audit only PDF-related security surfaces introduced or materially changed by
Issues 792, 793, 794, and the Issue 796 organization cleanup.

Primary Chromium scope:

- PDF-relevant call sites in
  `chromium/src/content/libtermsurf_chromium/ts_browser_client.*`,
  `chromium/src/content/libtermsurf_chromium/ts_content_renderer_client.*`, and
  `chromium/src/content/libtermsurf_chromium/ts_content_client.*`, because these
  thin dispatchers still decide when the extracted PDF helpers run;
- `chromium/src/content/libtermsurf_chromium/ts_pdf_browser_support.*`
- `chromium/src/content/libtermsurf_chromium/ts_pdf_renderer_support.*`
- `chromium/src/content/libtermsurf_chromium/ts_pdf_stream_delegate.*`
- `chromium/src/content/libtermsurf_chromium/ts_pdf_iframe_navigation_throttle.*`
- `chromium/src/content/libtermsurf_chromium/ts_plugin_response_interceptor_url_loader_throttle.*`
- `chromium/src/content/libtermsurf_chromium/ts_plugin_utils.*`
- `chromium/src/content/libtermsurf_chromium/ts_mime_handler_binders.*`
- `chromium/src/content/libtermsurf_chromium/ts_pdf_document_helper_client.*`
- `chromium/src/content/libtermsurf_chromium/extensions/ts_pdf_*`
- `chromium/src/content/libtermsurf_chromium/extensions/ts_resources_private_api.*`
- `chromium/src/content/libtermsurf_chromium/extensions/ts_component_extension_resource_manager.*`
- `chromium/src/content/libtermsurf_chromium/extensions/ts_extension_resource_loader.*`
- `chromium/src/content/libtermsurf_chromium/extensions/ts_extensions_*`
- PDF-specific TermSurf patches in `chromium/src/pdf/` and
  `chromium/src/components/printing/`.

Primary Rust, JavaScript, and automation scope:

- Roamium PDF/input/resize dispatch paths if they accept PDF-originated IDs,
  URLs, dimensions, or commands;
- Wezboard PDF input/resize routing only where it could expose browser-process
  trust or file access;
- `scripts/test-issue-794-*.py`;
- `scripts/termsurf_pdf_protocol_harness.py`;
- `scripts/probe-pdf-*.mjs`;
- `scripts/capture-pdf-interactions.mjs`.

Out of scope:

- unrelated browser security work;
- native PDF printing implementation from Issue 795;
- normal upstream Chromium/PDFium memory-safety audit outside TermSurf patches;
- broad extension-system hardening unrelated to the PDF viewer path;
- completeness/user-experience gaps that are not security relevant.

## Audit Method

### 1. Build a trust-boundary map

Map the security-relevant PDF flow from the original URL to the rendered viewer:

- original PDF URL classification and MIME detection;
- top-level and embedded PDF navigation throttles;
- stream claiming and lookup keys;
- frame tree node, render frame host, tab, and process identifiers;
- PDF component-extension registration and manifest permissions;
- extension-scheme resource serving;
- renderer-side PDF plugin creation and externalization;
- `resourcesPrivate` and `pdfViewerPrivate` API entrypoints;
- file, extensionless, HTTP, and HTTPS PDF paths;
- print-containment and print-intercept flags.

The map must distinguish trusted actors, untrusted actors, and data crossing
between them. It should explicitly state which code decides that a request is
"the PDF viewer" rather than an arbitrary extension page or web page.

### 2. Compare TermSurf's PDF security model to Chrome/Electron

Use local open-source copies where available. Compare TermSurf's implementation
against the relevant Chrome/Electron patterns for:

- PDF component-extension manifest permissions and web-accessible resources;
- extension process recognition and process-map grants;
- stream-manager ownership checks;
- `resourcesPrivate.getStrings(PDF)`;
- `pdfViewerPrivate.setPdfDocumentTitle`;
- internal PDF plugin origin checks;
- file access handling for `file://` and extensionless local PDFs;
- print containment or print interception.

The audit does not need to prove exact Chrome parity everywhere. It must record
where TermSurf intentionally differs, whether the difference is security
neutral, and what cleanup or test would make the boundary clearer.

### 3. Review URL, origin, and resource access

Inspect every path that accepts or constructs:

- `chrome-extension://mhjfbmdgcfjbbpaeojofohoefgiehjai/...`;
- `chrome://resources/...`;
- original PDF URLs;
- stream URLs;
- local `file://` URLs;
- extensionless local file URLs;
- resource bundle paths or resource IDs.

Questions to answer:

- Can a non-PDF web page or a different extension invoke PDF-only APIs?
- Can a PDF origin get access to broader `chrome://resources` or extension
  resources than the PDF viewer needs?
- Can a crafted URL, redirect, path, query, fragment, or extensionless local
  path bypass MIME or origin checks?
- Are resource lookups bounded to known resources rather than path-derived disk
  access?
- Are `file://` grants restricted to the loaded PDF file and viewer needs?

### 4. Review stream, frame, tab, and process ownership

Inspect every path that stores, retrieves, forwards, or trusts:

- stream IDs;
- frame tree node IDs;
- render process and render frame IDs;
- tab IDs;
- `RenderFrameHost*`, `RenderFrame*`, `WebContents*`, and `BrowserContext*`;
- extension process-map grants;
- PDF title/update messages derived from extension frames.

Questions to answer:

- Can a stale frame or reused ID retrieve another PDF's stream?
- Can one tab or PDF viewer influence another tab's stream/title/state?
- Are null and lifecycle checks sufficient around frame and WebContents lookup?
- Are process-map grants scoped to the PDF extension process and revoked or
  naturally bounded by Chromium lifetime rules?
- Are renderer-originated calls validated in the browser before changing
  browser-visible state?

### 5. Review automation-only and diagnostic paths

Audit the env-var and command-line switch gates for:

- PDF input tracing;
- PDF resize tracing;
- PDF print bridge tracing;
- PDF print intercept;
- screenshot/devtools/automation harness paths.

Questions to answer:

- Are dangerous automation paths off by default?
- Does enabling a trace only log, or can it alter runtime behavior?
- Can production users or untrusted content trigger print interception without
  an explicit local env var/switch?
- Are trace file paths controlled only by local process environment, not web
  content?
- Do logs include sensitive URLs or local paths, and if so, is that acceptable
  for explicit debug traces?

### 6. Review C++ safety assumptions in TermSurf-owned PDF code

This is not a full upstream Chromium or PDFium memory-safety audit. Focus on
TermSurf-owned code and patches. Look for:

- unchecked null pointers;
- raw pointer lifetime assumptions;
- callbacks or posted tasks that can outlive frames/WebContents;
- unsafe casts;
- integer truncation or unbounded size conversion;
- unbounded string or byte logging;
- file path handling;
- fallthrough defaults that fail open;
- `CHECK`/`DCHECK` choices that could turn untrusted content into a browser or
  renderer crash.

### 7. Classify findings

Each finding must include:

- **Severity:** `Critical`, `High`, `Medium`, `Low`, or `Defense-in-depth`.
- **Confidence:** `High`, `Medium`, or `Low`.
- **Files:** exact file paths and line references where practical.
- **Threat:** what an untrusted PDF, web page, or local environment could do.
- **Current guard:** what guard exists today, if any.
- **Gap:** why the guard may be too broad, absent, unclear, or unverifiable.
- **Recommended cleanup:** the concrete fix or hardening step for Experiment 4.
- **Verification needed:** static check, automated test, negative test, or
  manual security reasoning needed to prove the cleanup.

### 8. Separate non-findings and intentional differences

Record notable non-findings where code looks risky but is acceptable because of
Chromium invariants, Electron parity, explicit local-only debug gating, or
closed-world resource IDs. These prevent Experiment 4 from chasing cosmetic or
non-security churn.

### 9. Produce the Experiment 4 cleanup backlog

The conclusion must split the audit output into:

- security fixes that must be implemented in Experiment 4;
- defense-in-depth improvements that should be included if small;
- findings that need a separate follow-up issue because they exceed this issue's
  scope;
- non-findings or rejected concerns;
- minimum verification matrix for Experiment 4.

If the audit finds no exploitable issues, Experiment 4 should still be designed
to add the smallest useful set of assertions, comments, or negative tests that
make the security boundary easier to maintain.

## Commands and Evidence

Use `rg` first for searches. Suggested starting points:

```bash
rg -n "chrome-extension://|chrome://resources|PDF_EXTENSION|pdfViewerPrivate|resourcesPrivate|GetStream|StreamInfo|frame_tree|FrameTree|process_map|Grant|Allow|Origin|file://|FilePath|TERMSURF_PDF|PRINT_INTERCEPT|CHECK|DCHECK|raw_ptr|Unretained|WeakPtr" \
  chromium/src/content/libtermsurf_chromium \
  chromium/src/pdf \
  chromium/src/components/pdf \
  chromium/src/components/printing \
  roamium/src \
  wezboard/wezboard-gui/src/termsurf \
  scripts
```

```bash
rg -n "CanExecute|ExtensionFunction|Run\\(|Respond|GetBrowserContext|GetWebContents|FromFrame|FromRenderFrameHost|RenderFrameHost|RenderProcessHost|SiteInstance|ChildProcessSecurityPolicy|URLLoader|NavigationThrottle|MimeHandler|CreateInternalPlugin|IsPdfInternalPluginAllowedOrigin" \
  chromium/src/content/libtermsurf_chromium \
  chromium/src/pdf \
  chromium/src/components/pdf \
  chromium/src/components/printing
```

```bash
rg -n "TERMSURF_PDF|--termsurf|print-intercept|trace-file|user-data-dir|file-pdf-url|file-extensionless-url" \
  scripts \
  roamium/src \
  wezboard/wezboard-gui/src/termsurf
```

Suggested local reference searches:

```bash
rg -n "pdfViewerPrivate|getStreamInfo|resourcesPrivate|PdfViewerStreamManager|ChromePdfStreamDelegate|IsPdfExtensionOrigin|IsPdfInternalPluginAllowedOrigin" \
  chromium/src/chrome \
  chromium/src/components/pdf \
  chromium/src/extensions
```

If using Electron as a reference, use the local open-source research workflow
and cite exact files/lines from the local checkout or note if a local checkout
is unavailable.

The final audit must cite current worktree files and line numbers. Patch names
alone are not sufficient.

## Verification

This is a documentation-only audit experiment. Verification is:

- Codex design review completed and real design findings fixed;
- no runtime code changed;
- the audit result is appended to this file under `## Result`;
- the trust-boundary map is present;
- findings cite current files and line references where practical;
- findings are separated into required fixes, defense-in-depth improvements,
  follow-up issues, rejected concerns, and non-findings;
- every required Experiment 4 cleanup item has a concrete verification plan;
- Codex completion review completed and real findings fixed;
- Prettier run on this file and the issue README.

No Chromium, Rust, or Roamium build is required unless the audit accidentally
changes code. It must not change code.

## Pass Criteria

This experiment passes if it produces an evidence-backed security audit that
identifies the actual security cleanup backlog for Experiment 4, or proves that
no exploitable security issues were found and defines the minimum
defense-in-depth cleanup needed to preserve that boundary.

## Partial Criteria

This experiment is partial if it identifies likely security issues but lacks
enough line-level evidence, threat modeling, or verification guidance to safely
design Experiment 4.

## Failure Criteria

This experiment fails if:

- it changes runtime behavior;
- it combines audit and cleanup;
- it audits broad upstream Chromium/PDFium code instead of TermSurf's PDF
  integration;
- it treats native PDF printing as in scope;
- it relies on old Issue 792-794 layouts instead of the current organized code;
- it claims safety without checking URL/origin/resource, stream/frame ownership,
  extension API, file access, automation gates, and C++ lifetime surfaces;
- it omits Codex design or completion review;
- it produces a cleanup backlog too vague to implement safely.
