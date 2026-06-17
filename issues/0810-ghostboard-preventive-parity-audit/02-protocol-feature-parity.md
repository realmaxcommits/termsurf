# Experiment 2: Protocol Feature Parity

## Description

Use the protocol feature groups from Experiment 1 as the first concrete parity
audit slice. This experiment compares the current mature Wezboard/TermSurf
behavior against current Ghostboard evidence for every protocol feature group,
then classifies each group as `Highly likely`, `Maybe`, or `No` for Ghostboard
risk.

This experiment should not prove every individual edge case. Its job is to find
the most likely Ghostboard gaps before ordinary app usage finds them, and to
create a ranked evidence table that later experiments or issues can use for
focused verification.

This is an audit/documentation experiment only. It must not change application
code.

## Changes

Planned files:

- `issues/0810-ghostboard-preventive-parity-audit/02-protocol-feature-parity.md`
  - record this experiment design, design review, result, completion review, and
    conclusion;
  - record the protocol feature parity audit table after implementation.
- `issues/0810-ghostboard-preventive-parity-audit/README.md`
  - add Experiment 2 to the `## Experiments` index with status `Designed`, then
    update status after the result.

No application code, generated protobuf code, historical issue files, or closed
issue files should be edited.

## Verification

Design-gate pass criteria:

- The issue README links this experiment as `Designed`.
- A fresh-context adversarial design review approves the plan.
- The plan commit exists before implementation begins.

Implementation pass criteria:

- The result covers every protocol feature group from Experiment 1:
  - browser tab and process lifecycle;
  - viewport geometry and native presentation;
  - navigation and browser chrome state;
  - input forwarding and focus;
  - appearance and environment state;
  - GUI/TUI handshake and discovery;
  - DevTools orchestration;
  - pane/split orchestration;
  - dialogs and browser-interruption flows;
  - crash reporting and recovery.
- For each group, the result records:
  - source messages;
  - inferred feature;
  - Wezboard/reference behavior evidence;
  - current Ghostboard evidence;
  - likelihood: `Highly likely`, `Maybe`, or `No`;
  - risk or impact;
  - recommended follow-up.
- Each audit row justifies its likelihood with this evidence rubric:
  - `Highly likely`: reference behavior exists, but the Ghostboard runtime path
    appears absent, clearly incomplete, parse-only, log-only, or disconnected
    from the required behavior.
  - `Maybe`: Ghostboard has partial, ambiguous, platform-specific, or untested
    runtime evidence, and the audit cannot prove whether the feature works
    without a focused experiment.
  - `No`: Ghostboard has concrete runtime implementation evidence or durable
    test/experiment evidence for the required behavior. Generated protobuf
    structs, message names, or unpack-only code do not qualify by themselves.
- The audit distinguishes generated protobuf support from implemented runtime
  behavior. Generated message structs alone are not enough to classify a feature
  as implemented.
- The result explicitly calls out any group where Ghostboard appears to parse,
  log, or name a message but does not appear to perform the corresponding
  runtime behavior.
- The result preserves uncertainty where evidence is incomplete; it should not
  label an item `Highly likely` unless the implementation evidence supports that
  risk.
- The result identifies the next audit slice. Expected next slice: a focused
  deep dive on the highest-risk `Highly likely` or `Maybe` protocol findings,
  unless the protocol comparison shows the historical issue audit should begin
  first.
- Markdown is formatted:

  ```bash
  prettier --write --prose-wrap always --print-width 80 \
    issues/0810-ghostboard-preventive-parity-audit/README.md \
    issues/0810-ghostboard-preventive-parity-audit/02-protocol-feature-parity.md
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

- Any protocol feature group from Experiment 1 is omitted.
- The result treats generated protobuf types as sufficient runtime parity
  evidence.
- The audit edits application code.
- The result makes vague parity claims without file references or evidence.
- The experiment starts the historical issue audit before completing the
  protocol feature comparison.

## Design Review

Fresh-context adversarial design review initially returned **CHANGES REQUIRED**.

Required finding:

- The design required `Highly likely`, `Maybe`, and `No` labels, but did not
  define an operational evidence rubric for those labels. In particular, it did
  not state what Ghostboard evidence is sufficient for `No`, or what separates
  missing, partial, parse-only, logged-only, and runtime behavior evidence.

Fix:

- Added explicit classification rules requiring every audit row to justify its
  likelihood:
  - `Highly likely` for absent, incomplete, parse-only, log-only, or
    disconnected Ghostboard runtime paths when reference behavior exists;
  - `Maybe` for partial, ambiguous, platform-specific, or untested runtime
    evidence;
  - `No` only for concrete Ghostboard runtime implementation evidence or durable
    test/experiment evidence, excluding generated protobuf structs, message
    names, or unpack-only code by themselves.

Re-review verdict: **APPROVED**.

The reviewer confirmed the prior required finding is resolved and no new
required findings were introduced.
