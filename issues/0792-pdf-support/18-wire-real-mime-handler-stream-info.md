# Experiment 18: Wire Real MIME Handler Stream Info

## Description

Experiment 17 moved the PDF pipeline past content_shell's download
classification gate:

```text
wrapper response intercepted_by_plugin=1
navigation-download-classification ... is_download=0
pvs-ready ...
pvs-claim claimed=1
```

The next expected Chrome/Electron step is that the PDF extension viewer asks for
stream information. Depending on the viewer path, this may happen through
`chrome.mimeHandlerPrivate.getStreamInfo()` or through
`chrome.pdfViewerPrivate.getStreamInfo()`.

In Chrome, the `MimeHandlerService` binder creates
`extensions::MimeHandlerServiceImpl` with a weak pointer to the claimed
`StreamContainer`, but Chrome gets that weak pointer from
`MimeHandlerViewGuest::GetStreamWeakPtr()`. TermSurf deliberately does not have
`MimeHandlerViewGuest`.

For OOPIF PDF, Chromium already has a canonical non-guest lookup in
`chrome/browser/extensions/api/pdf_viewer_private/pdf_viewer_private_api.cc`: it
walks from the PDF extension frame to its parent embedder frame, then asks
`PdfViewerStreamManager::GetStreamContainer(embedder_host)`. Experiment 18 uses
that canonical OOPIF PDF lookup inside TermSurf's `MimeHandlerService` binder,
then hands the resulting real `StreamContainer` weak pointer to Chrome's
canonical `MimeHandlerServiceImpl`. The stream lookup and service implementation
are Chromium code; the TermSurf-specific part is the binder seam that replaces
the absent `MimeHandlerViewGuest`.

TermSurf currently still has the Experiment 12 diagnostic stub in
`content/libtermsurf_chromium/ts_mime_handler_binders.cc`:

```text
mime-handler-get-stream-info ... stream_info=null
```

That stub was useful before the stream was claimed, but after Experiment 17 it
is now the next likely blocker. Experiment 18 replaces the stub with the real
Chrome implementation path for PDF extension frames:

```text
PDF extension frame requests MimeHandlerService
  -> locate the parent embedder frame
  -> locate PdfViewerStreamManager from the embedder frame's WebContents
  -> locate the claimed StreamContainer from the extension frame's parent
  -> bind extensions::MimeHandlerServiceImpl(stream_weak_ptr)
  -> getStreamInfo returns non-null stream info
  -> setPdfPluginAttributes stores attributes on the StreamContainer
```

This experiment is both a minimal fix and a diagnostic gate. If the binder is
never requested, the result is Partial and the next experiment targets PDF
extension API invocation rather than stream-info conversion.

This experiment must receive Claude design review before implementation. After
implementation and result recording, Claude must review the completed output
before any next experiment is designed.

## Changes

1. Create the Chromium implementation branch.

   Start from the accepted Experiment 17 branch:

   ```bash
   git -C chromium/src checkout 148.0.7778.97-issue-792-exp17
   git -C chromium/src checkout -b 148.0.7778.97-issue-792-exp18
   ```

   Add the branch to `chromium/README.md` only after the branch builds and the
   result is accepted.

2. Replace the diagnostic `TsMimeHandlerService` stub with Chrome's real stream
   service.

   In `content/libtermsurf_chromium/ts_mime_handler_binders.cc`:
   - include `chrome/browser/pdf/pdf_viewer_stream_manager.h`;
   - include
     `extensions/browser/api/mime_handler_private/mime_handler_private.h`;
   - include `extensions/browser/mime_handler/stream_container.h`;
   - remove the local `TsMimeHandlerService` class;
   - in `BindTsMimeHandlerService(...)`, after validating this is a PDF
     extension frame, find:

     ```cpp
     auto* embedder_host = render_frame_host->GetParent();
     auto* manager = embedder_host
         ? pdf::PdfViewerStreamManager::FromRenderFrameHost(embedder_host)
         : nullptr;
     base::WeakPtr<extensions::StreamContainer> stream =
         manager && embedder_host ? manager->GetStreamContainer(embedder_host)
                                  : nullptr;
     ```

   - if `stream` is present, bind:

     ```cpp
     extensions::MimeHandlerServiceImpl::Create(stream, std::move(receiver));
     ```

   The real implementation already returns `StreamInfo` from `GetStreamInfo()`
   and stores PDF plugin attributes from `SetPdfPluginAttributes(...)`.

   The no-op `TsMimeHandlerService` class can be removed because PDF extension
   frames now bind the real implementation, non-PDF frames are filtered before
   binding, and TermSurf has no other registered MIME-handler extensions.

3. Add issue-tagged logs around the new binder path.

   Required logs:

   ```text
   [issue-792-exp18] mime-handler-service-request frame_url=<url> site_url=<url> is_pdf_extension=<0|1> has_parent=<0|1> has_manager=<0|1> has_stream=<0|1>
   [issue-792-exp18] mime-handler-service-bound frame_url=<url> stream_url=<url> original_url=<url>
   [issue-792-exp18] mime-handler-service-no-stream frame_url=<url> reason=<not-pdf-extension|no-parent|no-manager|no-stream>
   ```

   Preserve the Experiment 12 binder-entry log if it still provides value, but
   do not leave a live null-returning service on the successful PDF path.

4. Add narrowly scoped logging to `MimeHandlerServiceImpl`.

   In `extensions/browser/api/mime_handler_private/mime_handler_private.cc`, add
   logs inside:
   - `GetStreamInfo(...)`;
   - `SetPdfPluginAttributes(...)`.

   Required logs:

   ```text
   [issue-792-exp18] real-mime-handler-get-stream-info has_stream=<0|1> stream_url=<url> original_url=<url>
   [issue-792-exp18] real-mime-handler-set-pdf-attributes has_stream=<0|1> background_color=<value> allow_javascript=<0|1>
   ```

   These logs must be issue-scoped and should not dump full response headers.

5. Add a companion diagnostic to `pdfViewerPrivate.getStreamInfo()`.

   In
   `chrome/browser/extensions/api/pdf_viewer_private/pdf_viewer_private_api.cc`,
   add a narrow log inside `PdfViewerPrivateGetStreamInfoFunction::Run()`:

   ```text
   [issue-792-exp18] pdf-viewer-private-get-stream-info has_stream=<0|1> stream_url=<url> original_url=<url>
   ```

   This is diagnostic only. It answers whether the OOPIF PDF viewer is using
   `pdfViewerPrivate.getStreamInfo()` instead of
   `mimeHandlerPrivate.getStreamInfo()`.

6. Add the minimal GN dependency if needed.

   If including `mime_handler_private.h` requires a more specific dependency
   than the current `//extensions/browser` dependency, add the narrowest
   required target to `content/libtermsurf_chromium/BUILD.gn`.

7. Preserve Experiment 17 logs.

   Do not remove the Experiment 17 download-classification logs yet. The proof
   for Experiment 18 depends on seeing the complete chain:

   ```text
   is_download=0
   pvs-claim claimed=1
   mime-handler-service-bound
   real-mime-handler-get-stream-info has_stream=1
   real-mime-handler-set-pdf-attributes has_stream=1
   ```

8. Build and archive only after the result is accepted.

   Build with:

   ```bash
   cd chromium/src
   export PATH="$HOME/dev/termsurf/chromium/depot_tools:$PATH"
   autoninja -C out/Default libtermsurf_chromium
   ```

   If the experiment passes or produces a coherent partial branch, commit the
   Chromium branch and regenerate:

   ```bash
   rm -rf ../../chromium/patches/issue-792/
   git format-patch 148.0.7778.97..HEAD -o ../../chromium/patches/issue-792/
   ```

## Verification

1. Build `libtermsurf_chromium` with `autoninja`.

2. Run the fake-GUI PDF smoke test against the local bitcoin PDF fixture:

   ```bash
   LOG_DIR="logs/issue-792-exp18-pdf-$(date +%Y%m%d-%H%M%S)"
   scripts/test-issue-792-fake-gui.py \
     http://127.0.0.1:9787/bitcoin.pdf \
     --serve-bitcoin-pdf \
     --log-dir "$LOG_DIR" \
     --seconds 18
   ```

3. Inspect `roamium.stderr` in the run directory.

   The required success chain is:

   ```text
   [issue-792-exp17] navigation-download-classification ... is_download=0
   [issue-792-exp16] pvs-claim ... claimed=1
   [issue-792-exp18] mime-handler-service-request ... is_pdf_extension=1 ... has_stream=1
   [issue-792-exp18] mime-handler-service-bound ... stream_url=<url> original_url=<url>
   [issue-792-exp18] real-mime-handler-get-stream-info has_stream=1 ...
   [issue-792-exp18] real-mime-handler-set-pdf-attributes has_stream=1 ...
   ```

   If the viewer uses `pdfViewerPrivate` instead, the run should show:

   ```text
   [issue-792-exp18] pdf-viewer-private-get-stream-info has_stream=1 ...
   ```

4. Classify the result.
   - If `mime-handler-service-request` never appears, the PDF extension did not
     request the Mojo service. Check the `pdf-viewer-private-get-stream-info`
     log before concluding. If `pdf-viewer-private-get-stream-info has_stream=1`
     fires, the viewer is using `pdfViewerPrivate`; record the result around
     that path instead. If neither API path fires, record Partial and design the
     next experiment around extension API invocation/resource startup.
   - If the request appears with `has_stream=0`, record Partial and use the
     logged reason to target frame-parent/manager lookup.
   - If `GetStreamInfo()` returns a real stream but PDF rendering still does not
     progress, record Partial and target the next renderer/plugin lifecycle
     blocker.
   - If the fake-GUI logs show the stream-info path succeeds and the browser
     starts producing normal layer/update messages for the PDF viewer, run a
     separate real-GUI visual smoke before claiming user-visible PDF support.

5. Run HTML and download regressions.

   Re-run the same three safety smokes from Experiment 17:
   - normal HTML;
   - unsupported non-PDF binary;
   - attachment PDF.

   The new real MIME-handler service must not be requested for HTML or non-PDF
   downloads, and attachment PDFs must still download.

## Pass Criteria

- `libtermsurf_chromium` builds.
- The PDF smoke reaches `pvs-claim claimed=1`.
- The PDF extension requests either `MimeHandlerService` or
  `pdfViewerPrivate.getStreamInfo()`.
- `MimeHandlerServiceImpl::GetStreamInfo()` or
  `PdfViewerPrivateGetStreamInfoFunction::Run()` returns a non-null stream for
  the claimed PDF.
- If the MIME-handler service path is used, `SetPdfPluginAttributes(...)` stores
  attributes on the same stream.
- HTML navigation remains normal.
- Non-PDF downloads and attachment PDFs are not converted into inline viewer
  navigations.

## Partial Criteria

- The service is never requested after stream claim.
- The service is requested from a frame that is not identified as the PDF
  extension frame.
- The service is requested but no claimed stream is found for the parent
  embedder frame.
- `GetStreamInfo()` succeeds, but rendering hits a later PDF plugin or
  subresource blocker.
- `pdfViewerPrivate.getStreamInfo()` succeeds instead of the MIME-handler Mojo
  service path, proving the viewer uses the other Chromium API.

## Failure Criteria

- The experiment reintroduces the old null-stream service on the successful PDF
  path.
- The experiment creates a fake `StreamInfo` instead of using the claimed
  `StreamContainer`.
- The experiment broadens MIME-handler service access to arbitrary extension or
  web frames.
- Attachment PDFs stop downloading.
- HTML navigation, DevTools, popups, or ordinary browser input regress.
