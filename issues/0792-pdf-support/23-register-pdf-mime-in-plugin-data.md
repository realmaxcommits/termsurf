# Experiment 23: Register PDF MIME in Plugin Data

## Description

Experiment 22 proved that Blink's body-loader and parser path is healthy. The
problem is that Blink receives Chromium's 76-byte unsupported-plugin fallback
instead of TermSurf's 536-byte PDF wrapper:

```text
[issue-792-exp19] wrapper-payload ... bytes=536 has_template=1 has_iframe=1 has_about_blank=1
[issue-792-exp22] body-loader-start ... is_static_data=1 ...
[issue-792-exp22] body-data-received ... encoded_size=76 ... sample=<html><body><!-- no enabled plugin supports this MIME type --></
```

Claude traced that exact fallback string to
`third_party/blink/renderer/core/loader/frame_loader.cc::FillStaticResponseIfNeeded(...)`.
That function synthesizes the fallback when the response MIME is not supported
by Blink and renderer-side `PluginData::SupportsMimeType(mime_type)` returns
false.

TermSurf's `TsContentClient::AddPlugins(...)` currently registers the internal
PDF plugin only for `pdf::kInternalPluginMimeType`
(`application/x-google-chrome-pdf`). The top-level PDF navigation is checked as
`pdf::kPDFMimeType` (`application/pdf`) before the wrapper body can reach the
parser. Experiment 23 registers the PDF plugin for both MIME types and verifies
that the renderer no longer replaces the wrapper body with static fallback HTML.

This is the renderer-side counterpart of Experiment 17's browser-side
`intercepted_by_plugin` fix: the browser must not download the PDF, and the
renderer must not synthesize the unsupported-plugin fallback.

This experiment must receive Claude design review before implementation. After
implementation and result recording, Claude must review the completed output
before any next experiment is designed.

## Changes

1. Create the Chromium implementation branch.

   Start from the accepted Experiment 22 branch:

   ```bash
   git -C chromium/src checkout 148.0.7778.97-issue-792-exp22
   git -C chromium/src checkout -b 148.0.7778.97-issue-792-exp23
   ```

   Add the branch to `chromium/README.md` only after the branch builds and the
   result is accepted.

2. Register both PDF MIME types in TermSurf plugin data.

   In `content/libtermsurf_chromium/ts_content_client.cc`, update
   `TsContentClient::AddPlugins(...)` so the internal PDF plugin advertises both
   MIME types:

   ```text
   application/x-google-chrome-pdf
   application/pdf
   ```

   Use the same extension and description for both entries:

   ```text
   extension=pdf
   description=Portable Document Format
   ```

   Keep the plugin type as:

   ```text
   content::WebPluginInfo::PLUGIN_TYPE_BROWSER_INTERNAL_PLUGIN
   ```

   Preserve the existing internal MIME entry. Do not replace it with
   `application/pdf`; the extension viewer still uses
   `application/x-google-chrome-pdf`.

3. Add narrow diagnostic logging around static fallback.

   In `third_party/blink/renderer/core/loader/frame_loader.cc`, add temporary
   `[issue-792-exp23]` logs inside `FillStaticResponseIfNeeded(...)` for the
   local bitcoin PDF navigation only.

   Required logs:

   ```text
   [issue-792-exp23] static-response-check url=<url> mime_type=<mime> supported_mime=<0|1> has_plugin_data=<0|1> plugin_supports_mime=<0|1> action=<return|fallback>
   ```

   This log must prove whether `PluginData::SupportsMimeType("application/pdf")`
   becomes true after the plugin registration change.

4. Preserve Experiment 19-22 diagnostics.

   The verification needs the existing chain:

   ```text
   wrapper-payload bytes=536 has_iframe=1
   document-commit ...
   body-loader-start ...
   body-data-received ...
   parser-append-string ...
   declarative-shadow-root / first later missing gate
   ```

5. Do not change the PDF wrapper or stream manager.

   This experiment is specifically about renderer-side plugin MIME visibility.
   Do not change:
   - `TsPluginResponseInterceptorURLLoaderThrottle`;
   - `PdfViewerStreamManager`;
   - `MimeHandlerServiceImpl`;
   - data pipe ownership or completion ordering;
   - wrapper HTML generation;
   - extension resources;
   - parser scheduling.

6. Build and archive only after the result is accepted.

   Build with:

   ```bash
   cd chromium/src
   export PATH="$HOME/dev/termsurf/chromium/depot_tools:$PATH"
   autoninja -C out/Default libtermsurf_chromium
   ```

   If the experiment passes or produces a coherent diagnostic branch, commit the
   Chromium branch and regenerate:

   ```bash
   rm -rf ../../chromium/patches/issue-792/
   git format-patch 148.0.7778.97..HEAD -o ../../chromium/patches/issue-792/
   ```

## Verification

1. Build `libtermsurf_chromium` with `autoninja`.

2. Run the fake-GUI PDF smoke test against the local bitcoin PDF fixture:

   ```bash
   LOG_DIR="logs/issue-792-exp23-pdf-$(date +%Y%m%d-%H%M%S)"
   scripts/test-issue-792-fake-gui.py \
     http://127.0.0.1:9787/bitcoin.pdf \
     --serve-bitcoin-pdf \
     --log-dir "$LOG_DIR" \
     --seconds 18
   ```

3. Inspect `roamium.stderr`.

   Required success chain:

   ```text
   [issue-792-exp23] static-response-check ... mime_type=application/pdf ... plugin_supports_mime=1 action=return
   [issue-792-exp21] document-type-selected mime_type=application/pdf result=html is_for_external_handler=1
   [issue-792-exp22] body-loader-start ... is_static_data=0 ...
   [issue-792-exp22] body-data-received ... encoded_size=536 ... has_template=1 has_iframe=1 has_shadowrootmode=1 has_internal_id=1 ...
   [issue-792-exp22] parser-append-string ... has_template=1 has_iframe=1 has_shadowrootmode=1 ...
   [issue-792-exp21] declarative-shadow-root ...
   ```

   If the wrapper body reaches the parser but a later gate still fails, classify
   the first later missing transition:
   - no `declarative-shadow-root`: tokenizer/tree-builder issue;
   - shadow root attaches but no `frame-owner-inserted`: iframe in shadow root
     is not becoming a live frame owner;
   - frame owner inserts but no `load-or-redirect-subframe`: iframe insertion is
     not triggering subframe load;
   - `load-or-redirect-subframe result=1` but no child `pvs-finish`:
     browser-side child navigation is lost;
   - child `pvs-finish` appears but no extension viewer startup or stream-info:
     resume the extension-viewer diagnostics from Experiments 18-19.

4. Explicit failure criteria:
   - If `static-response-check ... plugin_supports_mime=0 action=fallback`
     remains, the new MIME was not visible to renderer-side plugin data.
   - If `body-data-received` still shows `encoded_size=76` or the unsupported
     MIME fallback sample, the static fallback is not fixed.
   - If `document-type-selected ... result=plugin` fires for the PDF navigation,
     the `application/pdf` MIME entry was not marked as an external handler;
     check that `GetPluginMimeTypesWithExternalHandlers(...)` still returns
     `application/pdf`.
   - If `application/pdf` reaches `OverrideCreatePlugin(...)` and creates a
     direct `PluginDocument` instead of the wrapper path, this fix is at the
     wrong layer; record the trace and redesign rather than adding more MIME
     aliases.

5. Run the normal HTML smoke test:

   ```bash
   LOG_DIR="logs/issue-792-exp23-html-$(date +%Y%m%d-%H%M%S)"
   scripts/test-issue-792-fake-gui.py \
     http://localhost:9616/index.html \
     --log-dir "$LOG_DIR" \
     --seconds 8
   ```

   The HTML control must not emit `[issue-792-exp23]` fallback logs and must not
   emit `[issue-792-exp22]` body/parser logs.

6. Record the result in this file.

   The result must include:
   - the exact PDF and HTML log directories;
   - whether renderer-side plugin data supports `application/pdf`;
   - whether `is_static_data` changes from `1` to `0`;
   - whether body bytes change from the 76-byte fallback to the 536-byte
     wrapper;
   - whether declarative shadow root and iframe creation advance;
   - the first remaining missing transition, if any;
   - the concrete next experiment implied by that transition.

## Result

**Result:** Pass

Build:

```text
autoninja -C out/Default libtermsurf_chromium
Build Succeeded: 5 steps
```

PDF log:

```text
logs/issue-792-exp23-pdf-20260529-155540
```

HTML control log:

```text
logs/issue-792-exp23-html-20260529-155603
```

The HTML control emitted no `[issue-792-exp23]` fallback logs and no
`[issue-792-exp22]` body/parser logs.

Important PDF trace lines:

```text
[issue-792-exp15] internal-pdf-plugin-registered mime_type=application/x-google-chrome-pdf document_mime_type=application/pdf path=internal-pdf-viewer
[issue-792-exp19] wrapper-payload ... bytes=536 has_template=1 has_iframe=1 has_about_blank=1 has_internal_id=1 has_pdf_extension_url=1
[issue-792-exp23] static-response-check url=http://127.0.0.1:9787/bitcoin.pdf mime_type=application/pdf supported_mime=0 has_plugin_data=1 plugin_supports_mime=1 action=return
[issue-792-exp21] document-type-check mime_type=application/pdf has_frame=1 allow_plugins=1 has_plugin_data=1 supports_mime=1 is_external=1 result=html
[issue-792-exp21] document-type-selected mime_type=application/pdf result=html is_for_external_handler=1
[issue-792-exp21] document-commit url=http://127.0.0.1:9787/bitcoin.pdf mime_type=application/pdf document_class=html is_for_external_handler=1 child_count=0
[issue-792-exp21] document-parser-open url=http://127.0.0.1:9787/bitcoin.pdf mime_type=application/pdf parser=html
[issue-792-exp22] body-loader-start url=http://127.0.0.1:9787/bitcoin.pdf mime_type=application/pdf reason=resume has_body_loader=1 is_static_data=0 loading_empty=0 is_main_frame=1
[issue-792-exp22] body-data-received url=http://127.0.0.1:9787/bitcoin.pdf encoded_size=536 ... has_template=1 has_iframe=1 has_shadowrootmode=1 has_internal_id=1 ...
[issue-792-exp22] parser-append-string url=http://127.0.0.1:9787/bitcoin.pdf length=536 ... has_template=1 has_iframe=1 has_shadowrootmode=1 ...
[issue-792-exp21] declarative-shadow-root url=http://127.0.0.1:9787/bitcoin.pdf host_tag=BODY mode=closed success=1 should_attach_template=0
[issue-792-exp21] frame-owner-inserted document_url=http://127.0.0.1:9787/bitcoin.pdf tag=IFRAME name=DE6EFBAA58A6CE20E4696463BBB4E2BB src=about:blank type=application/pdf internalid=DE6EFBAA58A6CE20E4696463BBB4E2BB is_connected=1
[issue-792-exp19] pvs-finish-about-blank frame_tree_node_id=2 parent_frame_tree_node_id=1 url=about:blank
[issue-792-exp15] pdf-extension-about-blank frame_tree_node_id=2 embedder=1
[issue-792-exp15] pdf-extension-navigate handler_url=chrome-extension://mhjfbmdgcfjbbpaeojofohoefgiehjai/index.html
[issue-792-exp21] load-or-redirect-subframe document_url=http://127.0.0.1:9787/bitcoin.pdf tag=IFRAME frame_name=DE6EFBAA58A6CE20E4696463BBB4E2BB url=about:blank internalid=DE6EFBAA58A6CE20E4696463BBB4E2BB result=1
[issue-792-exp19] pvs-finish-extension-already-started frame_tree_node_id=2 url=chrome-extension://mhjfbmdgcfjbbpaeojofohoefgiehjai/index.html matches_extension_url=1 matches_tracked_frame=1 has_committed=1 is_error_page=0
[issue-792-exp18] mime-handler-service-bound frame_url=chrome-extension://mhjfbmdgcfjbbpaeojofohoefgiehjai/index.html stream_url=chrome-extension://mhjfbmdgcfjbbpaeojofohoefgiehjai/2db89b22-e758-41e2-a1bb-16639feaeddd original_url=http://127.0.0.1:9787/bitcoin.pdf
[issue-792-exp18] real-mime-handler-get-stream-info has_stream=1 stream_url=chrome-extension://mhjfbmdgcfjbbpaeojofohoefgiehjai/2db89b22-e758-41e2-a1bb-16639feaeddd original_url=http://127.0.0.1:9787/bitcoin.pdf
```

This experiment fixed the renderer-side static fallback. The decisive changes
are:

- renderer plugin data now supports `application/pdf`;
- `FillStaticResponseIfNeeded(...)` returns instead of synthesizing fallback
  HTML;
- `DocumentInit` classifies the PDF navigation as external-handler HTML;
- `DocumentLoader` no longer uses static fallback data;
- the 536-byte wrapper body reaches the parser;
- the declarative shadow root attaches;
- the wrapper iframe inserts and starts the `about:blank` child navigation;
- `PdfViewerStreamManager` navigates the child frame to the PDF extension;
- the extension viewer requests and receives real stream info.

## Conclusion

Experiment 23 was the renderer-side counterpart to Experiment 17. Experiment 17
prevented browser-side download classification; Experiment 23 prevented
renderer-side unsupported-plugin fallback synthesis.

The PDF plumbing has now advanced through the full wrapper and stream-info path.
The remaining question is no longer whether the wrapper/stream handoff exists;
it does. The next experiment should verify the final user-visible outcome: does
the PDF viewer actually render the PDF page in the pane? If it does not, the
next failure is inside the PDF extension viewer or PDFium rendering path after
`mimeHandlerPrivate.getStreamInfo(...)` succeeds.
