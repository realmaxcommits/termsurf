# Experiment 43: Prove Surfari PDF Links

## Description

Experiment 42 proved that real TermSurf-routed scroll, click, `PageDown`, and
`PageUp` input reaches Surfari's PDF surface and produces visible PDF movement.
The next Surfari matrix slice should prove PDF link activation:

- internal PDF links;
- external PDF links from a PDF to a normal web page.

This experiment should test Surfari only. It should start as a probe and make no
product changes unless the probe exposes a real TermSurf integration gap.

## Changes

- Add a focused harness, tentatively
  `scripts/test-issue-834-surfari-pdf-links.sh`.
- Generate deterministic fixtures in the harness:
  - a PDF with at least two pages;
  - page 1 has a visible internal link annotation to page 2;
  - page 1 has a visible external link annotation to a local HTTP target page,
    such as `/pdf-link-target.html`;
  - page colors are distinct enough to prove internal navigation with
    overlay-cropped pixel counts;
  - the external target page uses colors and text that cannot be confused with
    the PDF page colors, with a fixture-specific target color suitable for
    overlay-cropped pixel proof.
- Serve the PDF as `application/pdf` and the target HTML as
  `text/html; charset=utf-8` from a deterministic local server.
- Launch repo-built Ghostboard with repo-built WebTUI and repo-built Surfari. Do
  not use installed artifacts.
- Run with `TERMSURF_SURFARI_CACONTEXT_LAYER` unset so the test proves the
  default Surfari presentation path.
- Use separate fresh-load scenarios for internal and external link activation so
  each scenario has an unambiguous starting state.
- Establish the baseline for each scenario:
  - WebTUI requested `browser=surfari`;
  - Surfari emitted `BrowserReady`;
  - WebTUI reached ready state;
  - Surfari trace recorded the PDF URL;
  - Surfari emitted nonzero CAContext;
  - Surfari internal render proof passed;
  - Ghostboard-window overlay-cropped visible proof shows page 1's target color;
  - the fixture server recorded the PDF request with `application/pdf`.
- Activate links through real TermSurf-routed mouse input:
  - compute global click coordinates from the Ghostboard overlay frame and known
    fixture link rectangles;
  - record the PDF-space annotation rectangles, the expected page-space click
    point, the overlay frame/crop, and the computed global click point for each
    link;
  - tie each post-click result to the annotation rectangle that was targeted so
    a coordinate-mapping failure cannot be mistaken for a WebKit link failure;
  - send real CGEvent mouse movement and click through
    `scripts/ghostty-app/inject.swift`;
  - require Ghostboard input/geometry evidence for the click;
  - require Surfari `mouse-event` trace evidence for the click;
  - do not use WebKit internals, JavaScript DOM clicks, or direct Surfari state
    mutation to activate the links.
- Internal-link pass evidence:
  - after clicking the internal link, Surfari refreshes;
  - Ghostboard-window overlay-cropped pixels transition from page 1's target
    color toward page 2's target color;
  - the transition uses explicit thresholds: page 1 target starts at least 5,000
    pixels, page 2 target ends at least 5,000 pixels, and the expected target
    rises by at least 5,000 pixels or 10% of the overlay crop, whichever is
    smaller.
- External-link pass evidence:
  - after clicking the external link, the fixture server records a request for
    `/pdf-link-target.html` with `text/html; charset=utf-8`;
  - Surfari trace or WebTUI state records navigation away from the PDF URL to
    the target URL, when that evidence is available;
  - Ghostboard-window overlay-cropped pixels show the target page, not the PDF
    page;
  - the target page proof uses explicit thresholds: the HTML target color starts
    below 5,000 pixels before the click, ends at least 5,000 pixels after the
    click, and rises by at least 5,000 pixels or 10% of the overlay crop,
    whichever is smaller;
  - if WebKit opens the external link in a new window, tab, or system browser
    instead of the same Surfari/Ghostboard overlay, record the exact behavior
    and classify the result as Partial.
- Record a JSON summary with:
  - env state proving `TERMSURF_SURFARI_CACONTEXT_LAYER` was unset;
  - repo binary paths;
  - PDF and target URL request/content-type evidence;
  - link rectangle and global click coordinates for each scenario;
  - PDF-space annotation rectangles and overlay/frame mapping evidence for each
    click;
  - matched Ghostboard, WebTUI, Surfari, and server evidence lines;
  - pre/post screenshot paths and target color counts;
  - pass/partial/fail classification for internal and external links;
  - cleanup status for Ghostboard, Surfari, WebTUI, and the fixture server.
- Update this experiment file with the result.

## Verification

Run build and hygiene checks:

```bash
./surfari/libtermsurf_webkit/build.sh
cargo fmt -p surfari
cargo build -p surfari
cargo build -p webtui
(cd ghostboard && macos/build.nu --configuration Debug --action build)
bash -n scripts/test-issue-834-surfari-pdf-links.sh
git diff --check
git -C webkit/src status --short
```

Run the links harness:

```bash
rm -rf logs/issue-834-exp43-surfari-pdf-links
env -u TERMSURF_SURFARI_CACONTEXT_LAYER \
  scripts/test-issue-834-surfari-pdf-links.sh
```

Pass criteria:

- `TERMSURF_SURFARI_CACONTEXT_LAYER` is unset;
- all scenarios use repo-built Ghostboard, WebTUI, Surfari, and WebKit
  artifacts;
- the PDF request is recorded with `application/pdf`;
- Surfari internal render proof passes before link activation;
- baseline Ghostboard-window overlay-cropped visible proof shows page 1 target
  pixels;
- internal-link activation uses real TermSurf-routed mouse input and produces
  Surfari mouse-event trace evidence;
- internal-link activation visibly moves to page 2 according to the
  fixture-specific color thresholds;
- external-link activation uses real TermSurf-routed mouse input and produces
  Surfari mouse-event trace evidence;
- external-link activation requests and displays the local HTML target in the
  same Surfari/Ghostboard overlay;
- external-link target display is proven with fixture-specific HTML target-color
  thresholds, not only URL or request evidence;
- each click records PDF-space annotation rectangle, overlay frame/crop, and
  computed global click point evidence;
- every visible proof is bounded to the Ghostboard app window and overlay crop;
- source/helper-window pixels cannot satisfy the proof;
- cleanup leaves no running Ghostboard, Surfari, WebTUI, or fixture server
  process;
- `webkit/src` remains clean;
- design review and completion review are recorded.

Partial criteria:

- internal links pass but external links do not;
- external links pass but internal links do not;
- mouse input reaches Surfari, but WebKit's PDF link behavior differs from
  Roamium in a concrete, documented way;
- WebKit opens external links outside the same Surfari/Ghostboard overlay;
- coordinate mapping cannot yet be proven well enough to distinguish a missed
  annotation from a WebKit link behavior gap;
- link activation works but one evidence source, such as WebTUI state, is
  unavailable while another authoritative source proves the behavior.

Failure criteria:

- baseline PDF visibility regresses;
- the harness requires `TERMSURF_SURFARI_CACONTEXT_LAYER=snapshot`;
- link activation proof can only pass by directly mutating Surfari internals or
  using JavaScript DOM clicks;
- no real mouse path reaches the Surfari PDF overlay;
- visible proof can be satisfied by a helper/source window;
- cleanup leaves running processes.

## Design Review

An external Codex review checked the design.

Initial verdict: **Changes required**.

Findings:

- full Pass allowed external-link behavior outside the Surfari/Ghostboard
  overlay if accepted as an engine-specific difference, which conflicted with
  the experiment goal of proving Surfari PDF links to a normal web page;
- external target-page proof did not define fixture-specific pixel thresholds,
  leaving room to pass on URL/request evidence or residual PDF pixels;
- the click-coordinate plan did not require enough evidence that computed global
  click points mapped into the intended PDF annotation rectangles.

Resolution:

- full Pass now requires the external target page to request and display in the
  same Surfari/Ghostboard overlay;
- external target display must be proven with fixture-specific HTML target-color
  thresholds;
- non-overlay WebKit behavior is explicitly Partial unless a later experiment
  changes product requirements;
- the plan now requires PDF-space annotation rectangles, overlay frame/crop, and
  computed global click point evidence for each link activation.

Follow-up verdict: **Approved**.

The reviewer found no remaining required design findings and approved the plan
for the Experiment 43 plan commit and implementation.
