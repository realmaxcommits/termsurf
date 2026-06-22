# Experiment 1: Define the Cross-Engine PDF Matrix

## Description

This experiment creates the evidence-backed PDF feature matrix that will govern
the rest of Issue 834. It is documentation-only: no Roamium, Surfari,
Ghostboard, Chromium, WebKit, protocol, fixture, script, or runtime behavior may
change.

Issue 834 spans two engines and a large PDF surface. Before implementing fixes,
we need a concrete matrix that separates:

- product requirements from optional or engine-specific behavior;
- existing Roamium evidence from stale, weak, or missing evidence;
- Surfari/WebKit unknowns from known TermSurf integration gaps;
- feature gaps from automation gaps;
- cheap regression guards from expensive/manual workflows.

The output of this experiment is the first authoritative matrix for Issue 834
and the immediate Roamium-first backlog for Experiment 2.

## Changes

1. Audit current issue records and existing PDF automation.

   Read at least:

   - Issue 834 README;
   - Issues 776, 789, 790, 791, 792, 793, 794, and 796 for existing Roamium PDF
     evidence;
   - superseded Issues 795, 797, and 798 for remaining print, core workflow, and
     advanced-feature scope;
   - Issue 756, `surfari/`, `surfari/libtermsurf_webkit/`, `webkit/README.md`,
     and `webkit/AGENTS.md` for Surfari capability context.

   A deep WebKit source audit is deferred to the Surfari phase. Experiment 1 may
   inspect targeted WebKit source when it is cheap and directly clarifies a
   matrix row, but Surfari rows must remain `Unknown` or `Likely but unverified`
   unless current TermSurf/Surfari evidence proves the behavior.

   Inspect existing scripts and logs, including:

   - `scripts/test-issue-794-pdf-toolbar.py`;
   - `scripts/test-issue-794-protocol-scroll.py`;
   - `scripts/test-issue-794-protocol-resize.py`;
   - `scripts/test-issue-794-protocol-mouse.py`;
   - `scripts/test-issue-796-pdf-security.py`;
   - `scripts/probe-pdf-save-print-title-local.mjs`;
   - `scripts/probe-pdf-toolbar-events.mjs`;
   - `scripts/termsurf_pdf_protocol_harness.py`;
   - relevant `logs/issue-794-*` and `logs/issue-796-*` summaries when present.

2. Add a matrix section to this experiment's `## Result`.

   The matrix must include every feature listed in the Issue 834 README and any
   additional PDF workflow discovered during the audit. Each row must include:

   - feature/workflow name;
   - requirement level: `Required`, `Engine-specific acceptable`, `Optional`, or
     `Out of scope`;
   - Roamium status;
   - Surfari status;
   - existing evidence;
   - automation coverage;
   - missing fixtures or probes;
   - known engine-specific difference, if any;
   - next action.

   Status values must be one of:

   - `Proven`;
   - `Likely but unverified`;
   - `Weak evidence`;
   - `Missing`;
   - `Blocked by fixture/probe gap`;
   - `Unsupported by design`;
   - `Unknown`.

3. Classify existing Roamium evidence.

   For Roamium, distinguish evidence that is still strong enough for Issue 834
   from evidence that must be rerun on the current tree. Older issue results may
   be cited, but they must be marked weak when the current code or test harness
   has materially changed.

4. Classify Surfari unknowns.

   For Surfari, do not assume WebKit-native PDF support is sufficient. Mark a
   feature as proven only if TermSurf/Surfari evidence exists. Otherwise
   classify it as `Unknown`, `Likely but unverified`, or
   `Blocked by fixture/probe gap` with a concrete next probe.

5. Define regression tiers.

   Split the future regression strategy into:

   - fast smoke checks suitable for frequent development;
   - focused feature probes used while fixing a row;
   - full matrix checks for issue completion or release confidence;
   - manual or OS-contained checks for features that cannot safely be fully
     automated, especially native print.

6. Produce the Experiment 2 recommendation.

   The conclusion must identify the next experiment. Expected default:
   Roamium-first verification of the current baseline using existing probes,
   because Issue 834 says Surfari begins after Roamium's matrix is complete and
   protected. If the audit proves a different ordering is safer, explain why.

## Verification

This is a documentation-only design/audit experiment. Verification for the
completed result will be:

- no product/runtime source files changed;
- this experiment contains `## Result` and `## Conclusion`;
- the Issue 834 README experiment index is updated from `Designed` to the final
  result status;
- every matrix row has a requirement level, Roamium status, Surfari status,
  evidence, automation coverage, and next action;
- no row claims `Proven` without concrete evidence;
- native print is classified with an OS-contained test strategy and no real
  print submission;
- Roamium and Surfari statuses are evaluated independently;
- automation gaps are separated from product behavior gaps;
- the next experiment recommendation is concrete enough to design without
  redoing this audit;
- markdown is formatted with:

  ```bash
  prettier --write --prose-wrap always --print-width 80 \
    issues/0834-full-pdf-support-roamium-surfari/README.md \
    issues/0834-full-pdf-support-roamium-surfari/01-define-cross-engine-pdf-matrix.md
  ```

- `git diff --check` passes;
- required design and completion reviews are recorded in this file.

No Chromium, WebKit, Rust, Zig, Swift, Python, or JavaScript build is required
unless this experiment accidentally changes code. It must not change code.

## Design Review

Fresh-context adversarial review by Codex subagent `Helmholtz`: **Approved**.

Findings:

- Optional: Surfari/WebKit evidence sources were underspecified. Fixed by naming
  the local Surfari/WebKit docs and source areas to inspect, while explicitly
  deferring deep WebKit source audit to the Surfari phase.
- Nit: Prettier verification was implicit. Fixed by adding the exact Prettier
  command for the edited issue files.

## Pass Criteria

This experiment passes if it produces a complete, evidence-backed cross-engine
PDF matrix and a concrete next experiment recommendation.

## Partial Criteria

This experiment is partial if it improves the matrix but leaves major Issue 834
feature areas without evidence classification or next actions.

## Failure Criteria

This experiment fails if it changes product code, treats old Roamium evidence as
current proof without justification, assumes Surfari parity without TermSurf
evidence, or leaves native print without a contained testing strategy.
