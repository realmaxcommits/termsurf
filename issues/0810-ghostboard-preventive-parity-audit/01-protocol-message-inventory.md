# Experiment 1: Protocol Message Inventory

## Description

Start the preventive parity audit with the protocol itself.
`proto/termsurf.proto` is the most compact description of the TermSurf behavior
surface: every browser, GUI, and TUI feature that crosses process boundaries is
represented by a protobuf message or message group.

This experiment will not decide final Ghostboard parity for every feature. Its
job is to build the audit backbone:

- enumerate every `TermSurfMessage` variant and concrete protobuf message;
- group messages into logical feature areas;
- infer the feature each message group implies;
- identify the primary reference code paths to inspect in later experiments;
- create the initial audit table schema that later Wezboard/Ghostboard and
  historical-issue passes will fill in.

This is an audit/documentation experiment only. It must not change application
code.

## Changes

Planned files:

- `issues/0810-ghostboard-preventive-parity-audit/01-protocol-message-inventory.md`
  - record the protocol inventory design, review, result, and conclusion;
  - record the initial message-group table after the experiment runs.
- `issues/0810-ghostboard-preventive-parity-audit/README.md`
  - add Experiment 1 to the `## Experiments` index with status `Designed`, then
    update status after the result.

No application code, generated protobuf code, or historical issue files should
be edited.

## Verification

Design-gate pass criteria:

- The issue README links this experiment as `Designed`.
- A fresh-context adversarial design review approves the plan.
- The plan commit exists before implementation begins.

Implementation pass criteria:

- The experiment result enumerates every `TermSurfMessage` oneof variant from
  `proto/termsurf.proto`.
- The concrete message inventory covers all message definitions in
  `proto/termsurf.proto`, including helper reply payloads such as `TabInfo`.
- Every message is assigned to a logical feature group, such as:
  - tab lifecycle and browser process lifecycle;
  - viewport geometry and native layer presentation;
  - navigation and browser chrome state;
  - input forwarding and focus;
  - GUI/TUI handshake and browser discovery;
  - DevTools;
  - split/pane orchestration;
  - dialogs, console capture, HTTP auth, and crash reporting.
- The result records an audit table schema with the required Issue 810 fields:
  source, inferred feature, reference behavior, Ghostboard evidence, likelihood,
  risk or impact, and recommended follow-up.
- The result identifies the next audit slice. Expected next slice: compare the
  protocol message groups against Wezboard and Ghostboard implementation
  evidence.
- Markdown is formatted:

  ```bash
  prettier --write --prose-wrap always --print-width 80 \
    issues/0810-ghostboard-preventive-parity-audit/README.md \
    issues/0810-ghostboard-preventive-parity-audit/01-protocol-message-inventory.md
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

- Any `TermSurfMessage` variant or concrete message definition is omitted.
- The result makes final parity claims without Wezboard/Ghostboard evidence.
- The experiment edits application code.
- The result collapses protocol messages into vague categories that cannot guide
  later audits.

## Design Review

Fresh-context adversarial design review initially returned **CHANGES REQUIRED**.

Required finding:

- The design did not explicitly require the result workflow gate: completed
  result recording, fresh-context completion review, fixing and recording real
  review findings, and result commit before the next experiment.

Fix:

- Added implementation pass criteria requiring completion-review approval,
  recording/fixing real review findings, and committing the result before the
  next experiment is designed.

Re-review verdict: **APPROVED**.

The reviewer confirmed the prior required finding is resolved and no new
required findings were introduced.
