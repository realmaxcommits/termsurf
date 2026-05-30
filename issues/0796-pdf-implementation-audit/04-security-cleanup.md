# Experiment 4: Security Cleanup

## Description

This experiment implements the required security cleanup from Experiment 3. It
must harden the existing PDF viewer boundary without adding new PDF features,
changing protocol surface, revisiting native PDF printing, or broadening
extension access.

The main security model is: TermSurf's temporary extension stack exists only for
Chromium's built-in PDF component extension `mhjfbmdgcfjbbpaeojofohoefgiehjai`.
Any extension-scheme handling, process-policy decisions, process-map activation,
PDF extension APIs, and `chrome://resources` grants must be explicitly tied to
that fixed component extension. Non-PDF extension URLs and non-PDF sender
contexts must fall through or fail closed.

This experiment starts from Chromium branch `148.0.7778.97-issue-796-exp2` and
creates a fresh branch `148.0.7778.97-issue-796-exp4`.

## Changes

### 1. Create the Chromium branch

- Create Chromium branch `148.0.7778.97-issue-796-exp4` from
  `148.0.7778.97-issue-796-exp2`.
- Add the branch to `chromium/README.md` with a link to this experiment.
- Do not modify older PDF branches.

### 2. Add a single PDF-extension identity helper

In `chromium/src/content/libtermsurf_chromium/ts_pdf_browser_support.*` or a
small shared helper under `content/libtermsurf_chromium/extensions/`, add one
reusable predicate for the fixed PDF component extension identity.

The helper should answer whether a candidate extension, extension id, origin, or
extension URL is the built-in PDF component extension:

- extension id exactly equals `extension_misc::kPdfExtensionId`;
- extension URL host exactly equals `extension_misc::kPdfExtensionId`;
- extension object is a component extension when that information is available;
- non-extension schemes always return false.

Prefer one implementation source of truth with thin overloads. Do not duplicate
literal extension ids in multiple files.

### 3. Restrict extension URL handling and process policy

In `chromium/src/content/libtermsurf_chromium/ts_pdf_browser_support.cc`,
tighten the three broad extension-scheme hooks identified by Experiment 3:

- `MaybeUseTsPdfProcessPerSite()`
- `MaybeHandleTsPdfExtensionURL()`
- `MaybeActivateTsPdfSiteInstance()`

Required behavior:

- the real PDF component extension still uses process-per-site;
- the real PDF component extension URL is still handled by TermSurf's extension
  URL loader;
- the real PDF component extension still gets process-map activation,
  renderer-startup helper activation, and the `chrome://resources` request
  origin grant;
- any other `chrome-extension://<id>/...` URL returns false or declines handling
  so `ShellContentBrowserClient` remains the authority;
- any other enabled extension, if one is added later, does not get TermSurf PDF
  process-map insertion through this PDF helper.

Add a short code comment next to the `chrome://resources` grant explaining that
the grant is process-scoped and is safe only because this code now forces the
PDF component extension into its own PDF-only process policy.

### 4. Add explicit sender checks to PDF extension APIs

Add a small shared sender guard for TermSurf PDF extension functions. Use it in:

- `extensions/ts_resources_private_api.cc`
- `extensions/ts_pdf_viewer_private_api.cc`

Required behavior:

- `resourcesPrivate.getStrings(PDF)` succeeds only when the sender extension is
  the fixed PDF component extension;
- `pdfViewerPrivate.setPdfDocumentTitle` succeeds only when the sender extension
  is the fixed PDF component extension and the existing `application/pdf` MIME
  guard passes;
- non-PDF sender contexts fail closed with a stable error and a stable
  `[termsurf-pdf]` or `[termsurf-pdf-title]` log line;
- bad parameters and missing `WebContents` keep their current failure behavior.

Use the `ExtensionFunction` sender metadata if available (`extension()`,
`source_url()`, or the equivalent Chromium 148 API). Do not rely only on
manifest permissions or the fact that today's registry contains one extension.

### 5. Replace PDF wrapper data-pipe CHECKs with graceful failure

In
`chromium/src/content/libtermsurf_chromium/ts_plugin_response_interceptor_url_loader_throttle.cc`,
replace the two browser-process `CHECK_EQ` calls around the generated wrapper
data pipe with graceful failure if the change remains small and local.

Required behavior:

- if `mojo::CreateDataPipe()` fails, log a stable `[termsurf-pdf]` failure and
  cancel/fail this intercepted PDF load without crashing the browser process;
- if `WriteAllData()` fails, log a stable `[termsurf-pdf]` failure and
  cancel/fail this intercepted PDF load without crashing the browser process;
- the normal success path remains unchanged.

If implementation proves that graceful failure requires a risky navigation or
Mojo ownership rewrite, do not silently skip this item. Record the exact blocker
in the result and include the minimal safe code/comment change that was made
instead. The two required extension-boundary fixes above are not optional.

### 6. Add a negative security probe

Add automated coverage that proves a non-PDF extension identity cannot use the
PDF-only boundary. Prefer a lightweight Chromium-side test seam or a DevTools
probe that can run in the existing PDF harness.

The probe must check at least these facts:

- a fake extension URL such as
  `chrome-extension://aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/index.html` is not logged
  as a TermSurf PDF `handled-url`;
- the fake extension URL does not receive TermSurf PDF `process-per-site`,
  `process-map-insert`, `pdf-activate-request`, or `chrome-resources-grant`
  logs;
- the real PDF extension path still produces the expected positive PDF logs
  during a normal PDF load;
- a non-PDF sender context cannot successfully call
  `resourcesPrivate.getStrings(PDF)` or `pdfViewerPrivate.setPdfDocumentTitle`.

If direct API invocation from a fake extension context is not practical in this
experiment, the probe may verify that the APIs reject a web/non-extension
context and the result must explain why constructing a fake enabled extension is
deferred. The process-policy negative check is still required.

Suggested implementation options:

- extend `scripts/test-issue-794-pdf-toolbar.py` with a
  `--security-negative-probe` mode that launches Roamium, navigates to the fake
  extension URL and a normal PDF, then inspects the Chromium log for positive
  and negative stable labels;
- add a dedicated script such as `scripts/test-issue-796-pdf-security.py` if the
  harness changes would make the Issue 794 script harder to read;
- add a tiny C++ static seam only if it can be built and invoked without
  broadening production behavior.

The probe must fail if any fake-extension URL receives PDF-specific handling.

## Verification

Run all verification from the main repo unless a command explicitly says to run
inside `chromium/src`.

1. Build Chromium:

   ```bash
   cd chromium/src
   export PATH="$HOME/dev/termsurf/chromium/depot_tools:$PATH"
   autoninja -C out/Default libtermsurf_chromium
   cd ../..
   ```

2. Build Roamium so the harness cannot accidentally run stale Chromium-facing
   glue:

   ```bash
   ./scripts/build.sh roamium
   ```

3. Run the PDF toolbar/save/title/local harness without print intercept:

   ```bash
   python3 scripts/test-issue-794-pdf-toolbar.py \
     --log-dir logs/issue-796-exp4-save-title-local \
     --serve-bitcoin-pdf \
     --probe save-print-title-local
   ```

   Expected result: pass. The production print status must remain
   `print-production-available-not-clicked`; no native print dialog may appear,
   and no print-intercept file or intercept-only log evidence may be produced in
   this default run.

4. Run the same harness with contained print intercept enabled:

   ```bash
   python3 scripts/test-issue-794-pdf-toolbar.py \
     --log-dir logs/issue-796-exp4-save-title-local-print-intercept \
     --serve-bitcoin-pdf \
     --probe save-print-title-local \
     --enable-pdf-print-intercept
   ```

   Expected result: pass. The intercept must remain local env/switch gated and
   must produce the contained callback evidence used by Issue 794.

5. Run the protocol interaction harnesses:

   ```bash
   python3 scripts/test-issue-794-protocol-scroll.py \
     --log-dir logs/issue-796-exp4-protocol-scroll \
     --serve-bitcoin-pdf
   python3 scripts/test-issue-794-protocol-resize.py \
     --log-dir logs/issue-796-exp4-protocol-resize \
     --serve-bitcoin-pdf
   python3 scripts/test-issue-794-protocol-mouse.py \
     --log-dir logs/issue-796-exp4-protocol-mouse-click \
     --serve-bitcoin-pdf \
     --action click
   python3 scripts/test-issue-794-protocol-mouse.py \
     --log-dir logs/issue-796-exp4-protocol-mouse-select-copy \
     --serve-bitcoin-pdf \
     --action key-select-copy
   ```

   Expected result: pass.

6. Run the deterministic non-PDF HTML smoke test used by the current PDF
   harness, or an equivalent `text/html` navigation through the same Roamium
   binary.

   Expected result: normal HTML still renders and no PDF-only logs are emitted
   for the HTML page.

7. Run the new security negative probe.

   Expected result:
   - fake extension URL receives no TermSurf PDF handling/process/grant logs;
   - real PDF URL still receives the positive PDF extension logs needed for
     rendering;
   - non-PDF sender API attempts fail closed or are unavailable;
   - the probe writes a JSON summary under `logs/`.

8. Static checks:

   ```bash
   rg -n "MaybeUseTsPdfProcessPerSite|MaybeHandleTsPdfExtensionURL|MaybeActivateTsPdfSiteInstance|chrome-resources-grant|CHECK_EQ" \
     chromium/src/content/libtermsurf_chromium
   ```

   Expected result:
   - the three process/URL helpers visibly check the fixed PDF extension id;
   - the `chrome://resources` grant has the process-scoped invariant comment;
   - no `CHECK_EQ` remains in the PDF response interceptor data-pipe path unless
     the result documents a justified deferral.

9. Format checks:
   - run Chromium formatting on changed C++ files;
   - run Prettier on this experiment file and the issue README;
   - run `cargo fmt` only if Rust code is edited.

10. Review:
    - Codex must review the completed diff, test evidence, and result language;
    - real completion-review findings must be fixed before the experiment is
      marked complete.

## Pass Criteria

This experiment passes if the required extension-boundary fixes are implemented,
the positive PDF viewer behavior still works, the negative probe proves non-PDF
extension/context rejection, and Codex completion review accepts the result
after any real findings are fixed.

## Partial Criteria

This experiment is partial if the required security boundary is improved but one
defense-in-depth item cannot be completed safely in this experiment, or if the
negative probe proves process-policy rejection but cannot yet construct a
realistic fake extension sender for the API functions. The result must state the
remaining gap precisely.

## Failure Criteria

This experiment fails if:

- any non-PDF extension URL is still treated as a TermSurf PDF URL;
- PDF extension APIs still rely only on permission wiring without explicit
  sender identity checks;
- normal PDF rendering, scrolling, resizing, title propagation, local-file
  parity, save/download, or non-PDF HTML rendering regresses;
- the fix broadens extension, file, origin, or protocol access;
- native PDF printing is implemented or re-scoped into this issue;
- code is committed on an existing Chromium issue branch instead of a fresh
  Experiment 4 branch;
- Codex design or completion review is skipped.
