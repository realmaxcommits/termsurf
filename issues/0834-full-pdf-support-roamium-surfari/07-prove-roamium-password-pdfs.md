# Experiment 7: Prove Roamium Password-Protected PDFs

## Description

Experiment 6 proved one PDF permission row and clarified that Chromium does not
disable original-file downloads for the restricted fixture after load. The next
unproven Roamium core row is password-protected PDFs.

This experiment should prove whether Roamium can open a user-password-protected
PDF through the real TermSurf protocol path. The important behavior is not only
that Chromium/PDFium can load encrypted PDFs, but that the TermSurf embedding
path preserves the Chromium PDF viewer's password prompt, accepts user input,
and reaches the loaded PDF after the correct password is entered.

## Changes

1. Create deterministic password PDF fixtures inside the harness log directory.

   Use `test-html/public/bitcoin.pdf` as the source and generate:

   - an unrestricted control PDF;
   - a user-password-protected PDF that requires a non-empty open password.

   Prefer `qpdf` because Experiment 6 already verified it is available on this
   VM. The generated password should be a fixed test fixture value, not a real
   secret. The harness may record the command and password length, but should
   avoid writing the raw password into result JSON or Roamium/Chromium logs.

2. Add a focused password PDF probe.

   Create `scripts/test-issue-834-pdf-password.py` plus a DevTools helper if
   useful, likely `scripts/probe-pdf-password.mjs`. Reuse the TermSurf
   socket/protobuf and DevTools patterns from Experiments 3 through 6.

   The harness should:

   - launch repo-built `chromium/src/out/Default/roamium`;
   - serve unrestricted and password-protected PDF fixtures from a local HTTP
     server;
   - create a Roamium tab through the TermSurf protocol;
   - resize the tab to a stable viewport;
   - discover the DevTools port and PDF extension child target;
   - detect the Chromium PDF viewer password dialog state without submitting a
     password through DevTools;
   - locate the password input field or dialog submit button coordinates through
     DevTools;
   - focus the password field through TermSurf protocol mouse input;
   - type passwords through TermSurf protocol keyboard input, including Enter;
   - write one summary JSON file at `<log-dir>/pdf-password-summary.json`.

3. Probe the unrestricted control first.

   The unrestricted control must prove the harness can observe normal PDF load
   success with no password prompt. If the control cannot load the same source
   PDF without encryption, classify the failure before testing the protected
   path.

4. Probe the password-protected path.

   The protected probe should prove:

   - before any password is entered, the PDF viewer reports password UI or an
     equivalent blocked load state;
   - the PDF content/plugin is not reported as successfully loaded before the
     password is submitted;
   - an incorrect password keeps the viewer in a password-required or invalid
     password state;
   - the correct password is entered through TermSurf protocol key events, not
     DevTools DOM mutation;
   - after the correct password, the viewer reports load success and exposes a
     real PDF plugin/viewport state;
   - the raw password does not appear in the harness summary, `messages.log`,
     `pdf-input.log`, `roamium.stdout`, or `roamium.stderr`.

5. Run the probes.

   Use fresh log directories:

   ```bash
   python3 scripts/test-issue-834-pdf-password.py \
     --log-dir logs/issue-834-exp7-password-control \
     --probe unrestricted-control
   python3 scripts/test-issue-834-pdf-password.py \
     --log-dir logs/issue-834-exp7-password-protected \
     --probe password-protected
   ```

6. If a probe fails, record the first failing layer.

   Classify the first failing layer before changing product code:

   - fixture generation;
   - fixture validity;
   - baseline PDF load;
   - DevTools target discovery;
   - password prompt discovery;
   - protocol mouse focus;
   - protocol keyboard delivery;
   - wrong-password rejection;
   - correct-password acceptance;
   - PDF plugin load after password;
   - password leak in logs;
   - Chromium PDF password plumbing;
   - Roamium integration;
   - evidence collection.

   If the failure is a real TermSurf/Roamium integration gap, fix only that gap
   in this experiment and rerun the relevant probe. If Chromium source must
   change, create a fresh Chromium branch for this experiment and update the
   Chromium branch table and patch archive according to `chromium/AGENTS.md`.

## Verification

Verification for the completed result is:

- fixture generation is deterministic and documented;
- the unrestricted control proves normal PDF load success without a password
  prompt;
- the protected run proves a password prompt before credential entry;
- the protected run sends password characters and Enter through TermSurf
  protocol key events;
- wrong-password rejection is observed if the viewer exposes a stable invalid
  password state;
- correct-password success is observed through PDF viewer state, plugin
  geometry, title/load state, or another stable Chromium/PDF observable;
- raw password text does not appear in the harness summary or logs;
- the summary records protocol key count, protocol mouse count, Roamium trace
  evidence, Chromium/PDF trace evidence when available, prompt state,
  wrong-password state, correct-password state, plugin state, password-leak
  checks, and first failing hop;
- the result cites command, exit status, summary file, summary status, first
  failing hop, and matrix rows proven or not proven;
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

Fresh-context adversarial review by Codex subagent `Hume`: **Approved**.

Findings: none.

The reviewer confirmed that the README links Experiment 7 as `Designed`, the
design includes the required sections and gates, the scope is narrow enough for
one experiment, password entry is required to use TermSurf protocol input rather
than DevTools DOM mutation, leak checks are included, and failure-layer
classification is concrete.

## Pass Criteria

This experiment passes if Roamium proves password-protected PDFs end to end:
prompt discovery, no premature load, TermSurf-protocol credential entry,
correct-password load success, and no raw password leakage in logs.

## Partial Criteria

This experiment is partial if the unrestricted control works and at least one
protected-path stage is proven, but a later stage records a concrete first
failing layer.

## Failure Criteria

This experiment fails if no valid password-protected fixture can be produced, if
the unrestricted control fails, if the password is submitted through DevTools
instead of TermSurf protocol input, or if the raw password leaks into logs or
result JSON.
