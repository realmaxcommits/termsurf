# Experiment 35: Prove Basic Surfari PDF Rendering

## Description

Experiment 34 audited WebKit and Surfari PDF capabilities and concluded that the
first Surfari implementation step should be a real-app PDF load/render proof.
Before testing PDF input, links, forms, print, or WebKit-specific hooks, we need
objective evidence that a PDF can load through `web --browser surfari` and
render visibly inside the actual TermSurf/Ghostboard overlay.

This experiment should follow the Issue 756 real-app Surfari harness style. It
should not modify WebKit, Surfari product code, Ghostboard, WebTUI, Roamium,
protobuf, or Chromium unless the basic proof exposes a concrete integration bug
and a follow-up experiment is designed.

## Changes

- Add a focused harness, tentatively
  `scripts/test-issue-834-surfari-pdf-render.sh`.
- The harness should:
  - require the same repo-built artifacts as the Issue 756 Surfari real-app
    harnesses:
    - `ghostboard/macos/build/Debug/TermSurf.app/Contents/MacOS/termsurf`;
    - `target/debug/web`;
    - `target/debug/surfari`;
    - `webkit/src/WebKitBuild/Debug/WebKit.framework`;
    - `surfari/libtermsurf_webkit/build/libtermsurf_webkit.dylib`;
  - create a temporary deterministic PDF fixture with high-contrast visual
    content and a known title/marker;
  - serve it over local HTTP with `Content-Type: application/pdf`;
  - launch the Debug TermSurf app with:
    - `TERMSURF_SURFARI_PATH` pointing to the repo-built Surfari binary;
    - `DYLD_FRAMEWORK_PATH` pointing to `webkit/src/WebKitBuild/Debug`;
    - Surfari/geometry/WebTUI trace files under
      `logs/issue-834-exp35-surfari-pdf-render/`;
  - run the repo-built `web --browser surfari <pdf-url>` as the initial command;
  - wait for Ghostboard `SetOverlay`, Surfari `ServerRegister`, `BrowserReady`,
    AppKit presentation, Surfari `create-tab`, loading/title callbacks, and a
    nonzero CAContext;
  - capture a screenshot of the target app window;
  - perform a deterministic pixel or crop analysis that proves the PDF fixture,
    not just a blank WebKit view, is visible in the overlay;
  - close the Surfari browser tab cleanly through the TermSurf socket and clean
    up temporary files/processes.
- The harness should write a compact machine-readable summary, tentatively
  `surfari-pdf-render-summary.json`, with:
  - `overall_result`;
  - `first_failing_hop`;
  - artifact paths;
  - fixture URL/path;
  - Surfari/WebTUI/Ghostboard trace evidence;
  - CAContext and presented overlay evidence;
  - screenshot path;
  - pixel-proof statistics;
  - cleanup result.
- Update this experiment file with the result.
- Do not add a broad Surfari PDF regression runner yet. That should wait until
  at least basic render and one or two input/navigation rows are proven.

## Verification

Run syntax/hygiene checks:

```bash
bash -n scripts/test-issue-834-surfari-pdf-render.sh
git diff --check
git -C webkit/src status --short
```

Run the focused probe:

```bash
rm -rf logs/issue-834-exp35-surfari-pdf-render
scripts/test-issue-834-surfari-pdf-render.sh
```

Inspect the summary:

```bash
python3 - <<'PY'
import json
from pathlib import Path

summary = json.loads(
    Path(
        "logs/issue-834-exp35-surfari-pdf-render/"
        "surfari-pdf-render-summary.json"
    ).read_text()
)
print(json.dumps({
    "overall_result": summary.get("overall_result"),
    "first_failing_hop": summary.get("first_failing_hop"),
    "ca_context": summary.get("ca_context"),
    "pixel_proof": summary.get("pixel_proof"),
    "cleanup": summary.get("cleanup"),
}, indent=2, sort_keys=True))
PY
```

Pass criteria:

- the harness exits `0`;
- the summary records `overall_result = "pass"` and
  `first_failing_hop = "no-failure-observed"`;
- Ghostboard launches repo-built Surfari through `TERMSURF_SURFARI_PATH`;
- Surfari runs with repo WebKit through `DYLD_FRAMEWORK_PATH`;
- Surfari registers as browser `surfari` and receives a `CreateTab` for the PDF
  URL;
- WebTUI renders Surfari ready state;
- Surfari emits a nonzero CAContext and Ghostboard presents a nonzero overlay;
- the screenshot/pixel proof shows deterministic PDF fixture content visible
  inside the overlay;
- title/URL/loading state is recorded if WebKit exposes it for the PDF;
- the browser tab closes cleanly and no native OS print/menu UI is opened;
- `webkit/src` remains clean;
- markdown is formatted with Prettier;
- design review and completion review are recorded.

Partial criteria:

- Surfari launches and receives the PDF URL, but rendering is blank or pixel
  proof is inconclusive;
- or the PDF renders but title/loading state is missing in a way that needs a
  follow-up classification.

Failure criteria:

- the harness uses installed Surfari/WebTUI/Ghostboard instead of repo-built
  artifacts;
- the proof is based only on process logs and does not show visible PDF pixels;
- the harness loads HTML instead of a real `application/pdf` response;
- WebKit, Surfari product code, Ghostboard, WebTUI, Roamium, protobuf, or
  Chromium source is changed;
- native print or context-menu UI is opened;
- cleanup leaves a running app/browser process.

## Design Review

An external Codex review checked the design.

Verdict: **Approved**.

The review found no findings. It confirmed that Experiment 35 is the correct
next step after the Surfari/WebKit audit, that the scope is narrow and safe, and
that the pass/failure criteria require repo-built artifacts, a real
`application/pdf` response, nonzero CAContext/overlay evidence, visible
deterministic PDF pixels, clean shutdown, no native print/menu UI, and no
WebKit/product source changes.
