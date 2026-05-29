# Experiment 25: Capture PDF via DevTools

## Description

Experiment 24 could not answer whether PDFs visually render because macOS
`screencapture` returned all-black images from this agent process, including the
permission-test image. That failure invalidates OS-level screenshot evidence,
but the repaired harness logs proved the real debug path can still launch:

```text
SetOverlay: pane_id=0 profile=default browser=/Users/ryan/dev/termsurf/chromium/src/out/Default/roamium url=https://example.com
CALayerHost created at (14.0,40.0,1106.0,1056.0): pane_id=0 contextId=2935611265 ...
```

Experiment 25 replaces the broken visual capture mechanism with Chromium's
DevTools Protocol `Page.captureScreenshot`. Roamium already exposes a DevTools
endpoint during the debug harness run; Experiment 24's repaired HTML run logged:

```text
DevTools listening on ws://127.0.0.1:59083/devtools/browser/e60a48bb-d552-473a-aea0-394f095fea8a
```

Capturing from that endpoint bypasses macOS screen-recording permissions while
still capturing Chromium's compositor output for the page, including child
frames and plugin pixels if they are present.

This experiment must not change PDF behavior. It only repairs verification
automation, re-proves the Experiment 23 stream-info plumbing, and captures the
rendered page image through DevTools.

This experiment must receive Claude design review before it runs. After the
result is recorded, Claude must review the completed output before any cleanup,
closure, or next experiment.

## Changes

1. Repair the fake-GUI harness observability in
   `scripts/test-issue-792-fake-gui.py`.

   When `--serve-bitcoin-pdf` is passed:
   - fail fast if the embedded HTTP server cannot bind;
   - write a short `http-server.log` containing the bound address/port;
   - keep logging every fixture request to `http.log`;
   - set `allow_reuse_address = true` so repeated local runs do not fail on a
     recently released port.

   This does not change browser behavior. It prevents Experiment 24's invalid
   fake-GUI result, where the run reached `chrome-error` and no HTTP request was
   logged, from being ambiguous.

2. Add a DevTools screenshot helper, preferably
   `scripts/capture-devtools-screenshot.mjs`.

   Use Node's built-in `fetch` and `WebSocket` support; do not add npm
   dependencies. The helper should:
   - accept `--devtools-port`, `--url-contains`, `--out`, optional
     `--timeout-seconds`, and optional `--settle-seconds`;
   - poll `http://127.0.0.1:{port}/json/list` every 250 ms up to
     `--timeout-seconds` until a `type == "page"` target whose URL contains
     `--url-contains` appears;
   - choose the newest matching target if more than one matches;
   - connect to that target's `webSocketDebuggerUrl`;
   - send `Page.enable`;
   - send `Page.bringToFront`;
   - wait for `Page.loadEventFired` or `--settle-seconds`, whichever is reached
     first for HTML; use the fixed settle interval for PDF because PDFium/plugin
     rendering may continue after document load;
   - call `Page.captureScreenshot` with `format: "png"` and `fromSurface: true`;
   - write the decoded PNG to `--out`;
   - write a small JSON sidecar next to the PNG containing the selected target
     id, URL, title, and screenshot byte count.

   If the matching page target does not exist, fail clearly and write the
   available targets to the sidecar or stderr.

3. Add `scripts/test-issue-792-devtools-screenshot.sh` so it captures DevTools
   output while the real debug app is still running.

   The harness should:
   - launch debug `wezboard-gui`;
   - launch debug `web`;
   - pass repo-built Roamium with `--browser`;
   - parse the DevTools port from the live log line:

     ```text
     DevTools listening on ws://127.0.0.1:{port}/devtools/browser/{id}
     ```

   - run the DevTools screenshot helper before shutting down Wezboard/Roamium;
   - keep the existing `screencapture` artifact as optional diagnostic output,
     but do not use it for pass/fail classification;
   - close the DevTools WebSocket/helper cleanly, then let the existing harness
     teardown shut down Wezboard/Roamium.

   This new script becomes the canonical visual harness for Issue 792. The older
   `scripts/test-issue-776-pdf.sh` remains useful for legacy OS-screenshot
   diagnostics, but it is not reliable for pass/fail visual evidence from this
   agent context.

4. Do not modify Chromium, Roamium, Wezboard rendering, webtui, the TermSurf
   protocol, or PDF plumbing in this experiment.

## Verification

1. Re-prove the Experiment 23 fake-GUI stream-info chain first:

   ```bash
   LOG_DIR="logs/issue-792-exp25-fakegui-$(date +%Y%m%d-%H%M%S)"
   scripts/test-issue-792-fake-gui.py \
     http://127.0.0.1:9787/bitcoin.pdf \
     --serve-bitcoin-pdf \
     --log-dir "$LOG_DIR" \
     --seconds 18
   ```

   Required log evidence:

   ```text
   real-mime-handler-get-stream-info has_stream=1
   ```

   If this chain does not reproduce, stop. The next problem is plumbing
   regression or fake-GUI fixture setup, not visual verification.

2. Run the real-GUI DevTools screenshot sanity check on ordinary HTML:

   ```bash
   TERMSURF_PDF_SETTLE_SECONDS=8 \
   LOG_DIR="logs/issue-792-exp25-html-devtools-$(date +%Y%m%d-%H%M%S)" \
   scripts/test-issue-792-devtools-screenshot.sh https://example.com
   ```

   Inspect the DevTools PNG with `view_image`. It must show normal rendered HTML
   content from `example.com`.

   Required log evidence:
   - debug `web` launched with
     `--browser /Users/ryan/dev/termsurf/chromium/src/out/Default/roamium`;
   - `SetOverlay` references `https://example.com`;
   - `CALayerHost created` appears for `pane_id=0`;
   - a DevTools page target was selected for `https://example.com`.

3. Run the real-GUI DevTools screenshot against the vendored Bitcoin PDF:

   ```bash
   TERMSURF_PDF_SETTLE_SECONDS=18 \
   LOG_DIR="logs/issue-792-exp25-pdf-devtools-$(date +%Y%m%d-%H%M%S)" \
   scripts/test-issue-792-devtools-screenshot.sh http://localhost:9616/bitcoin.pdf
   ```

4. Inspect the PDF DevTools PNG with `view_image`.

   Classify it as one of:
   - **Rendered PDF:** recognizable Bitcoin PDF content is visible, such as the
     paper title or first-page body text.
   - **Viewer shell only:** the PDF viewer UI appears, but the document page is
     blank, loading forever, or shows a viewer error.
   - **Wrong target:** DevTools captured a non-PDF target or stale page.
   - **Blank renderer output:** DevTools captured a valid target but the image
     is blank/black/white.
   - **Automation failure:** no DevTools PNG was produced.

5. Cross-check logs for the PDF run.

   A Pass requires the Experiment 23 success chain in the real-GUI PDF run, not
   only in the fake-GUI preflight:

   ```text
   real-mime-handler-get-stream-info has_stream=1
   ```

   It also requires real-GUI log evidence that the run used the repo-built
   Roamium binary and created the TermSurf overlay.

6. Record the result in this file.

   Include:
   - fake-GUI log directory and stream-info result;
   - HTML DevTools log directory, PNG path, sidecar path, and visual
     classification;
   - PDF DevTools log directory, PNG path, sidecar path, and visual
     classification;
   - whether the PDF run used the repo-built Roamium binary;
   - whether `SetOverlay` and `CALayerHost created` appear in the real-GUI logs;
   - whether the Experiment 23 stream-info chain appears in the PDF logs;
   - Pass/Partial/Fail status;
   - next action.

## Pass Criteria

Experiment 25 passes only if:

- fake-GUI preflight reproduces
  `real-mime-handler-get-stream-info has_stream=1`;
- real-GUI PDF run logs also show
  `real-mime-handler-get-stream-info has_stream=1`;
- the HTML DevTools sanity PNG shows normal `example.com` content;
- the PDF DevTools PNG shows recognizable Bitcoin PDF content;
- the real-GUI logs prove the run used repo-built Roamium and created the
  TermSurf overlay;
- logs do not contradict the run.

## Partial Criteria

Experiment 25 is partial if:

- fake-GUI and real-GUI logs show healthy stream-info plumbing;
- the HTML DevTools sanity screenshot works;
- the PDF DevTools screenshot captures the correct PDF target;
- the image shows viewer shell, blank renderer output, or a viewer/plugin error
  instead of recognizable PDF content.

In that case, the next experiment should instrument PDF extension viewer
JavaScript and PDFium plugin startup/rendering.

## Failure Criteria

Experiment 25 fails if:

- the fake-GUI stream-info chain does not reproduce;
- the DevTools endpoint cannot be discovered from the real-GUI run;
- the HTML DevTools sanity screenshot does not show `example.com`;
- the PDF screenshot captures the wrong target;
- the run uses an installed/stable Roamium instead of the repo-built debug
  Roamium;
- no reliable visual artifact is produced.

## Result

Not run yet.

## Conclusion

Pending verification.
