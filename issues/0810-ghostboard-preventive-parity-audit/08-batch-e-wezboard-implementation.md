# Experiment 8: Batch E Wezboard Implementation Audit

## Description

Classify Batch E from Experiment 4: issues `0715`-`0742`. This batch covers the
initial Wezboard implementation era and the Ghostboard archive transition:
WezTerm fork setup, build warnings, Cocoa/objc2 migration, wgpu/dependency
updates, split pane borders, TermSurf protocol implementation, CALayerHost
overlay rendering and lifecycle, multi-webview behavior, remaining protocol
coverage, Roamium install/process lifecycle, build scripts, branding, text
selection, split protocol, and the decision to archive Ghostboard.

This experiment should read every Batch E issue and map each durable lesson to
current Ghostboard risk using the schema defined in Experiment 4. The output is
a classification table, not fixes.

This is an audit/documentation experiment only. It must not change application
code, generated code, historical issue files, closed issue files, scripts, test
harnesses, screenshots, website assets, or build configuration.

## Changes

Planned files:

- `issues/0810-ghostboard-preventive-parity-audit/08-batch-e-wezboard-implementation.md`
  - record this experiment design, design review, Batch E classification result,
    completion review, and conclusion;
  - classify every issue in Batch E using the Experiment 4 historical audit row
    schema.
- `issues/0810-ghostboard-preventive-parity-audit/README.md`
  - add Experiment 8 to the `## Experiments` index with status `Designed`, then
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

- The result audits every Batch E issue exactly once:
  - `0715-wezboard`
  - `0716-wezboard-warnings`
  - `0717-remove-cocoa-crate`
  - `0718-finish-cocoa-removal`
  - `0719-wezboard-code-smells`
  - `0720-wezboard-manual-test`
  - `0721-wgpu-upgrade`
  - `0722-cargo-deps`
  - `0723-pane-borders`
  - `0724-wezboard-protocol`
  - `0725-wezboard-overlay`
  - `0726-wezboard-overlay-lifecycle`
  - `0727-wezboard-second-webview`
  - `0728-wezboard-remaining-protocol`
  - `0729-wezboard-reposition-and-protocol`
  - `0730-roamium-standalone-install`
  - `0731-wezboard-scroll-crash`
  - `0732-wezboard-reopen-tab`
  - `0733-ghostboard-shutdown`
  - `0734-build-scripts`
  - `0735-ghostboard-release-icon`
  - `0736-roamium-process-leak`
  - `0737-wezboard-icon`
  - `0738-wezboard-text-selection`
  - `0739-build-warnings`
  - `0740-wezboard-display-name`
  - `0741-protocol-split`
  - `0742-archive-ghostboard`
- The result uses the Experiment 4 row schema for every classification: source
  issue, batch, subsystem, durable lesson, current Ghostboard relevance,
  evidence paths, likelihood, risk or impact, recommended follow-up, and
  historical classification note.
- The result classifies each row as `Highly likely`, `Maybe`, or `No`, and
  explains the classification from issue evidence plus current code/test/doc
  evidence.
- The result treats all Batch E issues as closed historical evidence and does
  not modify or reinterpret their closure state.
- The result distinguishes Wezboard implementation lessons from current
  Ghostboard evidence. Wezboard protocol or overlay work is not automatically
  proof that restored Ghostboard has parity, and Wezboard-specific build/UI work
  is not automatically a Ghostboard bug.
- The result carries forward relevant Issue 810 findings where Batch E overlaps
  current Ghostboard risk, especially protocol message coverage, overlay
  lifecycle, multi-webview/tab routing, browser process cleanup, shutdown
  semantics, split protocol assumptions, text selection/input behavior, and
  branding/build-script evidence.
- The result explicitly evaluates Issue `0742` as historical evidence about why
  Ghostboard was archived, without treating the archive decision itself as a
  current defect.
- The result groups or summarizes related repeated findings after the table, but
  the table itself must still contain one row per Batch E issue.
- The result identifies the next audit slice after Batch E.
- Markdown is formatted:

  ```bash
  prettier --write --prose-wrap always --print-width 80 \
    issues/0810-ghostboard-preventive-parity-audit/README.md \
    issues/0810-ghostboard-preventive-parity-audit/08-batch-e-wezboard-implementation.md
  ```

- Whitespace check passes:

  ```bash
  git diff --check
  ```

- A fresh-context completion review approves the completed result before the
  result commit.
- All real completion-review findings are fixed and recorded in this experiment
  file.
- The result commit is made after completion-review approval and before any next
  experiment is designed.

Fail criteria:

- Any Batch E issue is omitted or classified more than once.
- The experiment edits historical issue files, application code, generated code,
  scripts, tests, screenshots, website assets, or build configuration.
- The result treats Wezboard historical fixes as current Ghostboard proof
  without current Ghostboard evidence.
- The result treats the historical Ghostboard archive as proof that restored
  Ghostboard is defective without current restored-Ghostboard evidence.
- The result labels build/branding/dependency issues as Ghostboard runtime bugs
  without a direct current product path.
- The result expands into other historical batches before Batch E is concluded.

## Design Review

Fresh-context adversarial design review returned **APPROVED**.

Reviewer checks confirmed:

- The issue README links Experiment 8 as `Designed`.
- The experiment has `Description`, `Changes`, and `Verification`.
- Batch E matches Experiment 4 exactly: `0715`-`0742`, twenty-eight issues.
- Scope is audit-only and limited to Issue 810 docs.
- Issue `0742-archive-ghostboard` is handled as historical archive evidence, not
  proof of current restored-Ghostboard defects.
- Verification includes the Experiment 4 schema, pass/fail criteria, markdown
  formatting, `git diff --check`, completion review, and separate plan/result
  commit gates.
- `git diff --check` passed.
- The plan commit had not yet been made before review.

Findings: none.
