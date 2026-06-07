+++
[implementer]
agent = "codex"
model = "gpt-5"
reasoning = "high"

[review.design]
agent = "codex"
model = "default"
reasoning = "medium"
+++

# Experiment 779: Surface Lifecycle Checklist Sync

## Description

Sync the broader Issue 801 `Surface` lifecycle checklist wording with the recent
C ABI surface-key and binding-action evidence.

Experiment 778 updated the C ABI checklist because focused verification proved
that configured/default surface key dispatch and binding-action parsing are no
longer missing at the C ABI boundary. The broader `Surface` lifecycle checklist
still says `full binding-action parsing` is missing, which appears stale after
the same evidence.

This experiment is documentation-only. It does not claim frontend selection
routing, renderer display-ID delivery, split tree/frontend mutations, Quicklook
UI/font integration, clipboard request allocation/handling, or full frontend
presentation are complete.

## Changes

- `issues/0801-roastty-libghostty-rewrite/README.md`
  - Remove or rewrite the stale `full binding-action parsing` missing-work
    phrase in the broader `Surface` lifecycle checklist item if Experiment 778's
    evidence applies cleanly there.
  - If adding a done phrase, use the same scoped wording as the C ABI checklist:
    `focused binding-action parsing coverage and targeted slow PTY paste-path coverage done`.
  - Preserve the remaining missing-work phrases that still describe actual
    incomplete frontend/lifecycle behavior.

## Verification

- Inspect the broader `Surface` lifecycle checklist item in
  `issues/0801-roastty-libghostty-rewrite/README.md`.
- Confirm Experiment 778's source and test evidence covers the stale
  `full binding-action parsing` wording:
  - public C ABI binding-action string invocation exists;
  - configured/default key dispatch routes through the parser/executor;
  - focused binding-action family filters passed;
  - PTY paste-path parsing/execution is covered by Experiment 777's targeted
    slow tests.
- Record in the result whether the broader `Surface` lifecycle phrase refers to
  the same parser/executor and string-invocation coverage proven by Experiments
  777 and 778, rather than unverified frontend mutation/application behavior.
- Run markdown formatting:
  - `prettier --write --prose-wrap always --print-width 80 issues/0801-roastty-libghostty-rewrite/README.md issues/0801-roastty-libghostty-rewrite/779-surface-lifecycle-checklist-sync.md`
- Run:
  - `git diff --check`

The experiment passes if the README update is documentation-only, removes only
the stale binding-action parsing missing-work claim from the broader `Surface`
lifecycle item, and leaves the genuinely missing frontend/lifecycle work listed.
It is Partial if the wording cannot be updated without overclaiming. It fails if
the broader checklist uses `full binding-action parsing` for a different
unverified concept than Experiment 778 covered.

## Design Review

Codex reviewed the initial design and found two issues: the plan did not propose
exact replacement wording, and the verification steps did not explicitly require
recording whether the broader `Surface` lifecycle phrase meant parser/executor
coverage rather than frontend mutation behavior.

The design was updated to constrain any added done phrase to
`focused binding-action parsing coverage and targeted slow PTY paste-path coverage done`.
It was also updated to require the result to record that the stale phrase is
being treated as parser/executor and string-invocation coverage, not frontend
mutation/application behavior. Codex reviewed the revision, found no blockers,
and approved the Experiment 779 plan commit.
