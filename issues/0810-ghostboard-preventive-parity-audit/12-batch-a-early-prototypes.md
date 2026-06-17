# Experiment 12: Batch A Early Prototypes Audit

## Description

Classify Batch A from Experiment 4: issue folders `0001`-`0350`. This batch
covers the earliest TermSurf architecture and prototype work: competitor
research, merge and website setup, bookmarks, builds, console and keybinding
behavior, libghostty research, releases, target blank handling, webview and CEF
iterations, profile handling, WezTerm analysis, XPC, resize and tab behavior,
focus and input behavior, mouse/scroll/drag/cursor/caret behavior, process
lifecycle, multi-pane behavior, navigation, backgrounding, refresh, lag,
Electron evaluation, performance benchmarking, CEF ceilings, and callback-based
rendering.

This experiment should read every Batch A issue folder and map each durable
lesson to current Ghostboard risk using the schema defined in Experiment 4. The
output is a classification table, not fixes.

This is an audit/documentation experiment only. It must not change application
code, generated code, historical issue files, closed issue files, scripts, test
harnesses, screenshots, website assets, or build configuration.

## Changes

Planned files:

- `issues/0810-ghostboard-preventive-parity-audit/12-batch-a-early-prototypes.md`
  - record this experiment design, design review, Batch A classification result,
    completion review, and conclusion;
  - classify every issue folder in Batch A using the Experiment 4 historical
    audit row schema.
- `issues/0810-ghostboard-preventive-parity-audit/README.md`
  - add Experiment 12 to the `## Experiments` index with status `Designed`, then
    update status after the result.

No application code, generated protobuf code, historical issue files, closed
issue files, scripts, test harnesses, screenshots, website assets, or build
configuration should be edited.

## Verification

Design-gate pass criteria:

- The issue README links this experiment as `Designed`.
- A fresh-context adversarial design review approves the plan.
- The plan commit exists before implementation begins.

Implementation pass criteria:

- The result audits every Batch A issue folder exactly once:
  - `0001-competitors`
  - `0002-merge-upstream`
  - `0003-website`
  - `0100-bookmarks`
  - `0101-build`
  - `0102-console`
  - `0103-ctrl-z`
  - `0104-keybindings`
  - `0105-libghostty`
  - `0106-release`
  - `0107-target-blank`
  - `0108-webview`
  - `0200-architecture`
  - `0201-cef`
  - `0202-cef-mvp`
  - `0203-cef-mvp2`
  - `0204-cef-mvp3`
  - `0205-cef-mvp4`
  - `0206-cef-mvp5`
  - `0207-cef-wezterm`
  - `0208-profile`
  - `0209-web`
  - `0210-wezterm-analysis`
  - `0301-architecture`
  - `0302-webview`
  - `0303-xpc`
  - `0304-webpage`
  - `0305-profile`
  - `0306-resize`
  - `0307-profile`
  - `0308-resize`
  - `0309-resize`
  - `0310-tabs`
  - `0311-resize`
  - `0312-stealing-focus`
  - `0313-helper`
  - `0314-control`
  - `0315-mode`
  - `0316-dim`
  - `0317-input`
  - `0318-cmd`
  - `0319-mouse`
  - `0320-double-click`
  - `0321-scroll`
  - `0322-drag-selection`
  - `0323-shift-click`
  - `0324-cursor-feedback`
  - `0325-webview-framerate`
  - `0326-process-lifecycle`
  - `0327-scroll-speed`
  - `0328-caret`
  - `0329-focus-lifecycle`
  - `0330-multi-pane`
  - `0331-termsurf`
  - `0332-profile-reconnect`
  - `0333-control-panel-profile`
  - `0334-copy-url`
  - `0335-navigation`
  - `0336-background`
  - `0337-refresh`
  - `0338-lag`
  - `0339-electron`
  - `0340-architecture`
  - `0341-performance`
  - `0342-perf-no-win`
  - `0343-optimal-performance`
  - `0344-cef-test`
  - `0345-benchmark`
  - `0346-mouse-performance`
  - `0347-lingering-lag`
  - `0348-cef-test-ceiling`
  - `0349-bimodal`
  - `0350-callback`
- The result uses the Experiment 4 row schema for every classification: source
  issue, batch, subsystem, durable lesson, current Ghostboard relevance,
  evidence paths, likelihood, risk or impact, recommended follow-up, and
  historical classification note.
- The result classifies each row as `Highly likely`, `Maybe`, or `No`, and
  explains the classification from issue evidence plus current code/test/doc
  evidence.
- The result treats all Batch A issues as closed historical evidence and does
  not modify or reinterpret their closure state.
- The result distinguishes obsolete CEF, XPC, WebView, prototype, and benchmark
  implementation mechanisms from current socket/protobuf, Roamium, CALayerHost,
  and restored Ghostboard evidence.
- The result distinguishes Ghostboard GUI-owned parity findings from Roamium,
  Chromium, webtui, website, packaging, docs, and historical prototype findings.
- The result carries forward relevant Issue 810 findings where Batch A overlaps
  current Ghostboard risk, especially build and release workflow, keybindings,
  target blank handling, profile lifecycle, resize behavior, tab behavior,
  focus, mode and command behavior, mouse/scroll/drag/cursor/caret behavior,
  process lifecycle, multi-pane behavior, navigation, backgrounding, refresh,
  lag, and performance.
- The result groups or summarizes related repeated findings after the table, but
  the table itself must still contain one row per Batch A issue folder.
- The result identifies whether any historical audit slices remain after Batch
  A, and if none remain, identifies the next step as the final Issue 810
  conclusion.
- Markdown is formatted:

  ```bash
  prettier --write --prose-wrap always --print-width 80 \
    issues/0810-ghostboard-preventive-parity-audit/README.md \
    issues/0810-ghostboard-preventive-parity-audit/12-batch-a-early-prototypes.md
  ```

- Whitespace check passes:

  ```bash
  git diff --check
  ```

- A fresh-context completion review approves the completed result before the
  result commit.
- All real completion-review findings are fixed and recorded in this experiment
  file.
- The result commit is made after completion-review approval and before the
  final Issue 810 conclusion is written.

Fail criteria:

- Any Batch A issue folder is omitted or classified more than once.
- The experiment edits historical issue files, application code, generated code,
  scripts, tests, screenshots, website assets, or build configuration.
- The result treats obsolete CEF, XPC, WebView, benchmark, or prototype
  implementation details as current Ghostboard requirements without mapping them
  to the current socket/protobuf, Roamium, and CALayerHost architecture.
- The result treats Roamium, Chromium, webtui, website, packaging, docs, or
  prototype behavior as a Ghostboard GUI bug without a direct current Ghostboard
  ownership path.

## Design Review

Raman reviewed the design with fresh context and approved it with no findings.

The review verified that the README links Experiment 12 as `Designed`, the
experiment has `Description`, `Changes`, and `Verification` sections, the Batch
A list matches Experiment 4 exactly as range `0001`-`0350` with seventy-three
folders, `git diff --check` passes, and the scope, schema, ownership
distinctions, obsolete-vs-current architecture distinctions, review gates,
formatting, and commit gates are covered.
