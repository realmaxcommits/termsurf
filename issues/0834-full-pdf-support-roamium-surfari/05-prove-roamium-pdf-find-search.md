# Experiment 5: Prove Roamium PDF Find/Search

## Description

Experiment 4 proved current Roamium internal and external PDF link activation
through TermSurf protocol mouse input. The next unproven core Roamium workflow
is PDF find/search.

Chromium's PDF plugin has native find support, but Issue 796 did not prove that
TermSurf users can reach that path from Roamium. This experiment adds a focused
probe that drives the user-level TermSurf keyboard path first. Product code
should change only if the probe exposes a real integration gap between TermSurf
keyboard input, Chromium browser find, and the PDF viewer.

## Changes

1. Add a deterministic Roamium PDF find fixture.

   Prefer generating the fixture inside the harness log directory to avoid
   binary churn. The fixture should be a small searchable PDF with at least two
   pages:

   - page 1 contains ordinary text but not the unique target term;
   - page 2 contains a unique target term such as `TERMSURF_FIND_TARGET_EXP5`;
   - no external network dependency is required.

   A target term that appears only on page 2 gives the harness a stable
   observable: successful find should move the viewer from page 1 toward page 2,
   or otherwise expose selected-match evidence.

2. Add a narrow Roamium PDF find probe.

   Create `scripts/test-issue-834-pdf-find.py` plus a small DevTools helper,
   likely `scripts/probe-pdf-find.mjs`. Reuse the TermSurf socket/protobuf and
   DevTools patterns from the Issue 794 harnesses and from Experiments 3 and 4.

   The harness should:

   - launch `chromium/src/out/Default/roamium` with trace/log output under
     `--log-dir`;
   - serve the generated PDF fixture through a deterministic local HTTP server;
   - create a Roamium tab through the TermSurf protocol;
   - resize the tab to a stable viewport;
   - discover the DevTools port and PDF extension child target;
   - capture before/after screenshots and viewer state;
   - focus the PDF plugin through TermSurf protocol mouse input;
   - send the find command through TermSurf protocol keyboard input, not
     DevTools synthetic DOM input;
   - type the search term through TermSurf protocol keyboard input when the
     browser/PDF find UI accepts text;
   - advance or confirm the search with TermSurf protocol keyboard input if
     required;
   - write one summary JSON file at `<log-dir>/pdf-find-summary.json`.

   The first attempt should use the real user path: Command-F, the target term,
   and Enter or another normal find-next key as needed. DevTools may be used to
   observe state, discover geometry, and capture screenshots, but it must not be
   the mechanism that starts the find or types the query for the primary pass
   condition.

3. Record the first failing layer if the user path does not work.

   If the probe fails, classify the first failing layer before changing product
   code:

   - fixture generation or searchable-text problem;
   - TermSurf protocol key serialization;
   - Roamium key receipt;
   - Chromium key routing;
   - missing browser find command handling, such as Command-F not entering
     browser/PDF find mode;
   - PDF viewer/plugin find handler not receiving the search;
   - evidence collection gap.

   If the failure is a product integration gap, fix only that gap in this
   experiment and rerun the probe. If Chromium source must change, create a
   fresh Chromium branch for this experiment and update the Chromium branch
   table and patch archive according to `chromium/AGENTS.md`.

4. Run the new probe.

   Use a fresh log directory:

   ```bash
   python3 scripts/test-issue-834-pdf-find.py \
     --log-dir logs/issue-834-exp5-find-positive \
     --probe positive-search
   ```

   If the harness exposes a stable match-count or no-match observable cheaply,
   add a no-match probe as a follow-up inside this experiment. Do not let a
   no-match probe broaden the experiment into restrictions, password PDFs,
   malformed PDFs, forms, annotations, context menus, accessibility/searchify,
   Surfari, or final regression coverage.

## Verification

Verification for the completed result is:

- the PDF fixture is deterministic, searchable, and documented in the result;
- the primary positive-search probe starts find/search through TermSurf protocol
  keyboard input;
- the primary positive-search probe types the search query through TermSurf
  protocol keyboard input unless the result explicitly proves a lower-level
  product gap that prevents that from happening;
- the probe records protocol key count, protocol mouse count, Roamium key trace
  evidence, Chromium key routing evidence when available, PDF viewer/plugin find
  evidence when available, before/after viewer state, before/after screenshot
  hashes, and the pass/fail delta;
- the pass condition is based on stable PDF find evidence, such as selected page
  movement to the unique target page, selected-match or match-count state, PDF
  find callback logs, screenshot changes consistent with a highlighted match, or
  a combination of those signals;
- the probe writes `pdf-find-summary.json` under `logs/issue-834-exp5-*`;
- the experiment result cites command, exit status, summary file, summary
  status, first failing hop, and matrix rows proven or not proven;
- no product code is changed unless the probe exposes a real TermSurf
  integration bug and that fix is explicitly documented in this experiment;
- no Chromium source is changed unless a fresh Chromium branch and patch archive
  are created according to `chromium/AGENTS.md`;
- syntax checks pass for any new Python or Node scripts;
- markdown is formatted with Prettier;
- `git diff --check` passes;
- design review is recorded and the plan commit exists before implementation
  begins;
- completion review is recorded before the result commit.

## Design Review

Fresh-context adversarial review by Codex subagent `Volta`: **Approved**.

Findings: none.

## Pass Criteria

This experiment passes if Roamium PDF find/search works through TermSurf
protocol keyboard input and the probe records stable current evidence for the
find/search matrix row.

## Partial Criteria

This experiment is partial if the probe identifies a concrete first failing
layer but the product fix is larger than this experiment, or if find/search
works only through DevTools or another non-user path.

## Failure Criteria

This experiment fails if PDF find/search cannot be probed at all, if the probe
claims success without stable PDF find evidence, or if it bypasses the TermSurf
keyboard path for the primary pass condition.
