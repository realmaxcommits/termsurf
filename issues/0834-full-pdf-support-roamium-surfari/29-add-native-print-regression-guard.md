# Experiment 29: Add Native Print to Roamium Regression Guards

## Description

Experiment 28 proved that Roamium native PDF print cancellation is now
automatable on this macOS VM:

- the native print safety preflight passes;
- the production native print control can be clicked by the guarded harness;
- document-modal print sheets are detected by live AppKit trace evidence;
- the watcher presses the sheet's Cancel button through PID-targeted
  Accessibility traversal constrained to a sheet/dialog-like AX subtree;
- Chromium reports the cancel callback;
- no print job is submitted;
- Roamium remains alive.

The durable Roamium PDF regression runner still reflects the old state from
Experiment 15: native print is listed only under `unsafe-manual` as skipped
because a safe native-dialog watcher was not yet proven. This experiment should
update the regression guard so native print is covered by an explicit
safety-gated tier without making it part of normal fast/focused runs.

The goal is durability, not new product behavior. The experiment should preserve
the existing smoke/focused/form tiers and add a clear opt-in tier for native
print cancellation.

## Changes

- Update `scripts/test-issue-834-roamium-pdf-regression.py`.
- Add an explicit native print regression tier, tentatively named
  `native-print`.
- The `native-print` tier should run:

  ```bash
  python3 scripts/test-issue-834-pdf-native-print.py \
    --log-dir <child-log-dir> \
    --probe native-dialog \
    --allow-native-dialog-click
  ```

- Classify the native print check as passing only when the child summary reports
  all of:
  - `first_failing_hop = "native-print-dialog-seen-cancelled"`;
  - `safety_gate_passed = true`;
  - `roamium_exited_before_shutdown = false`;
  - unchanged print queue before/after;
  - `print_dialog_watch.cancel_sent = true`;
  - `print_dialog_watch.sheet_evidence.observed = true`;
  - `print_dialog_watch.sheet_cancel.requireSheet = true`;
  - native trace includes `ts-scripted-print-callback-result-canceled`.
- Keep `smoke`, `focused`, and `forms` unchanged so routine runs do not open
  native OS UI.
- Update `unsafe-manual` so it no longer claims native print is unproven. It
  should either:
  - point users to the explicit `native-print` tier, or
  - list native print as skipped from `unsafe-manual` because it has its own
    guarded tier.
- Extend the runner summary if needed so future automation can tell that native
  print used the explicit safety gate.
- Do not modify Chromium, Roamium, Ghostboard, Surfari, or protocol code.

## Verification

Run hygiene checks:

```bash
rm -rf scripts/__pycache__
PYTHONDONTWRITEBYTECODE=1 python3 -m py_compile \
  scripts/test-issue-834-roamium-pdf-regression.py \
  scripts/test-issue-834-pdf-native-print.py
rm -rf scripts/__pycache__
git diff --check
git -C chromium/src diff --check
```

Run the explicit native print tier:

```bash
rm -rf logs/issue-834-exp29-native-print-regression
python3 scripts/test-issue-834-roamium-pdf-regression.py \
  --log-dir logs/issue-834-exp29-native-print-regression \
  --tier native-print
```

Run the dry unsafe tier to prove native print does not run there:

```bash
rm -rf logs/issue-834-exp29-unsafe-manual
python3 scripts/test-issue-834-roamium-pdf-regression.py \
  --log-dir logs/issue-834-exp29-unsafe-manual \
  --tier unsafe-manual
```

Run at least one cheap existing tier to prove it is not disrupted:

```bash
rm -rf logs/issue-834-exp29-smoke
python3 scripts/test-issue-834-roamium-pdf-regression.py \
  --log-dir logs/issue-834-exp29-smoke \
  --tier smoke
```

Pass criteria:

- `native-print` exits 0 and records `overall_result = "pass"`;
- the native print child summary proves safe cancellation, unchanged print
  queue, Roamium liveness, sheet evidence, `requireSheet=true`, and Chromium's
  canceled callback trace;
- `unsafe-manual` exits 0 without running a production native print click;
- `smoke` still exits 0;
- generated summaries are current-run summaries, not stale reused files;
- README status and this experiment result accurately describe the tiering.

Partial criteria:

- the runner is updated, but the explicit native print tier exposes a new
  concrete failing hop while still preserving print safety and Roamium liveness.

Failure criteria:

- a print job is submitted;
- native print is added to `smoke` or `focused` by default;
- `unsafe-manual` clicks the production native print control;
- the runner reports success without proving the safety-gate fields listed
  above;
- stale child summaries can make the native print tier pass;
- unrelated product code is changed.

## Design Review

An adversarial Codex subagent reviewed the design with fresh context.

Verdict: **Approved**.

The reviewer found no findings. It confirmed that the design is linked from the
issue README as Experiment 29 with status `Designed`, has the required
Description, Changes, and Verification sections, follows directly from
Experiment 28, keeps native OS UI behind an explicit `native-print` tier,
preserves `smoke` / `focused` / `forms`, includes stale-summary and cheap-tier
checks, and has no implementation changes beyond the README link and new
experiment file.
