# Experiment 26: Route Internal PDF Plugin

## Description

Experiment 25 proved that visual capture now works and narrowed the product
failure to the internal PDF plugin instantiation path. The PDF wrapper and
viewer extension both load, the stream is claimed, and the viewer receives
`mimeHandlerPrivate.getStreamInfo`. The visible failure is Chromium's plugin
fallback:

```text
Couldn't load plugin.
```

The decisive renderer logs are:

```text
[issue-792-exp18] real-mime-handler-get-stream-info has_stream=1 ... original_url=http://localhost:9616/bitcoin.pdf
[issue-792-exp18] real-mime-handler-set-pdf-attributes has_stream=1 ...
[issue-792-exp15] is-plugin-handled-externally mime_type=application/x-google-chrome-pdf ... plugin_lookup=missing handled=0
[issue-792-exp19] renderer-plugin-external ... mime_type=application/x-google-chrome-pdf has_internal_id=0 handled=0
[issue-792-exp19] renderer-override-create-plugin ... mime_type=application/x-google-chrome-pdf ... delegated_to_extensions=1
```

Electron handles this exact layer directly in its renderer client:

```cpp
if (params.mime_type.Utf8() == pdf::kInternalPluginMimeType) {
  *plugin = pdf::CreateInternalPlugin(std::move(params), render_frame, {});
  return true;
}
```

Experiment 26 ports that shape into `TsContentRendererClient`, before the
extensions renderer-client delegation. The goal is to instantiate
`application/x-google-chrome-pdf` as the internal PDF plugin instead of letting
Blink fall through to "Couldn't load plugin."

This experiment may reveal the next process-model gate:
`pdf::CreateInternalPlugin()` requires the current renderer process to be a PDF
renderer and will `CHECK()` if the process is wrong. Therefore the
implementation must log `pdf::IsPdfRenderer()` before calling the helper. If
that predicate is false, do not crash the renderer; record a Partial and design
the next experiment around Chromium's PDF renderer process assignment.

This experiment must receive Claude design review before it runs. After the
result is recorded, Claude must review the completed output before any cleanup,
closure, or next experiment.

## Changes

1. Create a new Chromium branch from the current Issue 792 Chromium branch.

   ```bash
   cd chromium/src
   git checkout 148.0.7778.97-issue-792-exp23
   git checkout -b 148.0.7778.97-issue-792-exp26
   ```

   Add the branch to `chromium/README.md`.

2. Update `content/libtermsurf_chromium/ts_content_renderer_client.cc`.

   Add the includes needed for:
   - `pdf::CreateInternalPlugin`
   - `pdf::IsPdfRenderer`

   In `TsContentRendererClient::OverrideCreatePlugin()`, before delegating to
   `extensions_renderer_client_`, handle
   `params.mime_type.Utf8() == pdf::kInternalPluginMimeType`.

   Required behavior:
   - log `[issue-792-exp26] internal-plugin-create-check` with:
     - document URL;
     - plugin URL;
     - MIME type;
     - `has_pdf_renderer=0/1` from `pdf::IsPdfRenderer()`;
   - if `pdf::IsPdfRenderer()` is false, return `false` after logging
     `[issue-792-exp26] internal-plugin-create-skipped reason=missing-pdf-renderer`;
   - if `pdf::IsPdfRenderer()` is true, call
     `pdf::CreateInternalPlugin(std::move(params), render_frame, {})`;
   - set `*plugin` to the result;
   - log `[issue-792-exp26] internal-plugin-create-result created=0/1`;
   - return `true` after taking this route, even if the returned plugin pointer
     is null, matching Electron's renderer-client semantics. The `created=0/1`
     log is the diagnostic signal.

   This route must run before
   `extensions_renderer_client_->OverrideCreatePlugin` so the internal PDF
   plugin does not get delegated away.

3. Update `content/libtermsurf_chromium/BUILD.gn`.

   Add the dependency that exports
   `components/pdf/renderer/internal_plugin_renderer_helpers.h`:

   ```gn
   "//components/pdf/renderer",
   ```

4. Do not change:
   - the PDF wrapper throttle;
   - `PdfViewerStreamManager`;
   - stream-info APIs;
   - extension resource loading;
   - Wezboard, Roamium Rust, webtui, or the TermSurf protocol.

5. Build Chromium:

   ```bash
   cd chromium/src
   export PATH="$HOME/dev/termsurf/chromium/depot_tools:$PATH"
   autoninja -C out/Default libtermsurf_chromium
   ```

6. Regenerate the Issue 792 Chromium patch archive only after the Chromium
   branch commit:

   ```bash
   cd chromium/src
   rm -rf ../../chromium/patches/issue-792/
   git format-patch 148.0.7778.97..HEAD -o ../../chromium/patches/issue-792/
   ```

## Verification

1. Run the fake-GUI stream-info preflight:

   ```bash
   LOG_DIR="logs/issue-792-exp26-fakegui-$(date +%Y%m%d-%H%M%S)"
   scripts/test-issue-792-fake-gui.py \
     http://127.0.0.1:9787/bitcoin.pdf \
     --serve-bitcoin-pdf \
     --log-dir "$LOG_DIR" \
     --seconds 18
   ```

   Required:

   ```text
   real-mime-handler-get-stream-info has_stream=1
   ```

2. Run the real-GUI DevTools HTML sanity check:

   ```bash
   TERMSURF_PDF_SETTLE_SECONDS=8 \
   LOG_DIR="logs/issue-792-exp26-html-devtools-$(date +%Y%m%d-%H%M%S)" \
   scripts/test-issue-792-devtools-screenshot.sh https://example.com
   ```

   The DevTools screenshot must show rendered `example.com`.

3. Run the real-GUI PDF DevTools capture:

   ```bash
   TERMSURF_PDF_SETTLE_SECONDS=18 \
   LOG_DIR="logs/issue-792-exp26-pdf-devtools-$(date +%Y%m%d-%H%M%S)" \
   scripts/test-issue-792-devtools-screenshot.sh http://localhost:9616/bitcoin.pdf
   ```

4. Inspect the PDF DevTools PNG with `view_image`.

   Classify it as:
   - **Rendered PDF:** recognizable Bitcoin PDF content is visible.
   - **Plugin fallback:** "Couldn't load plugin" still appears.
   - **Renderer crash:** the PDF renderer crashes after the internal plugin
     route.
   - **Wrong target:** DevTools captured the wrong page.
   - **Automation failure:** no reliable DevTools PNG was produced.

5. Inspect PDF logs.

   Required for Pass:

   ```text
   real-mime-handler-get-stream-info has_stream=1
   [issue-792-exp26] internal-plugin-create-check ... has_pdf_renderer=1
   [issue-792-exp26] internal-plugin-create-result created=1
   ```

   If logs show:

   ```text
   [issue-792-exp26] internal-plugin-create-skipped reason=missing-pdf-renderer
   ```

   then Experiment 26 is Partial, and the next experiment must wire the PDF
   renderer process model.

6. Record the result in this file.

   Include:
   - Chromium branch name and commit;
   - build command and result;
   - fake-GUI log directory and stream-info result;
   - HTML DevTools screenshot path and classification;
   - PDF DevTools screenshot path and classification;
   - whether `pdf::IsPdfRenderer()` returned true;
   - whether `pdf::CreateInternalPlugin()` returned a plugin;
   - Pass/Partial/Fail status;
   - next action.

## Pass Criteria

Experiment 26 passes only if:

- Chromium builds;
- fake-GUI stream-info preflight passes;
- HTML DevTools sanity capture passes;
- real-GUI PDF logs show the internal plugin route ran with
  `has_pdf_renderer=1`;
- `pdf::CreateInternalPlugin()` returns a non-null plugin;
- the PDF DevTools screenshot shows recognizable Bitcoin PDF content.
- logs do not contradict the run.

## Partial Criteria

Experiment 26 is partial if:

- stream-info remains healthy;
- the internal plugin route is reached;
- but the renderer lacks `--pdf-renderer`, `pdf::CreateInternalPlugin()` returns
  null, or the screenshot still shows "Couldn't load plugin."

In that case, the next experiment should target the precise missing sublayer
shown by the logs.

## Failure Criteria

Experiment 26 fails if:

- Chromium does not build;
- the patch changes PDF wrapper/stream-manager behavior instead of the renderer
  plugin creation path;
- the fake-GUI or real-GUI stream-info chain regresses;
- HTML DevTools sanity capture fails;
- the renderer crashes during or after the internal plugin route runs;
- the run uses an installed/stable Roamium instead of the repo-built binary.

## Result

Not run yet.

## Conclusion

Pending verification.
