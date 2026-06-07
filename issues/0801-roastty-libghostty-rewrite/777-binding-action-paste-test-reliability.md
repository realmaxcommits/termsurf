+++
[implementer]
agent = "codex"
model = "gpt-5"
reasoning = "high"

[review.design]
agent = "codex"
model = "default"
reasoning = "medium"

[review.result]
agent = "codex"
model = "default"
reasoning = "medium"
+++

# Experiment 777: Binding Action Paste Test Reliability

## Description

Investigate and fix the PTY-backed
`surface_binding_action_write_*_paste_queues_path` tests that prevented
Experiment 776 from completing its broad `surface_binding_action_` verification.

Experiment 776 found that surface key dispatch and binding-action parsing are
likely complete enough to remove stale checklist wording, but the broad
binding-action test filter spent several minutes in the final file-paste tests
and had to be terminated. This experiment focuses on that reliability problem
before attempting another checklist sync.

## Changes

- `roastty/src/lib.rs`
  - Inspect the three PTY-backed paste tests and their shared helpers:
    `surface_binding_action_write_selection_file_paste_queues_path`,
    `surface_binding_action_write_screen_file_paste_queues_path`,
    `surface_binding_action_write_scrollback_file_paste_queues_path`, and
    `surface_snapshot_text_until`.
  - If the investigation finds a real test harness issue, timing bug, cleanup
    gap, or overly slow child command, make the smallest code or test change
    needed to make these tests deterministic.
  - If the tests are already deterministic when run in isolation, record that
    evidence and leave code unchanged.

## Verification

- Run the three previously blocking tests individually, twice each, with output
  enabled and elapsed time recorded:
  - `/usr/bin/time -p cargo test -p roastty surface_binding_action_write_selection_file_paste_queues_path -- --nocapture --test-threads=1`
  - `/usr/bin/time -p cargo test -p roastty surface_binding_action_write_screen_file_paste_queues_path -- --nocapture --test-threads=1`
  - `/usr/bin/time -p cargo test -p roastty surface_binding_action_write_scrollback_file_paste_queues_path -- --nocapture --test-threads=1`
- Run the broader write-action cluster that contains the paste tests and nearby
  write/copy/open false-path coverage:
  - `/usr/bin/time -p cargo test -p roastty surface_binding_action_write_ -- --nocapture --test-threads=1`
- If a code or Rust test change is made, run:
  - `cargo fmt -p roastty`
  - `cargo fmt -p roastty -- --check`
- Run the markdown formatter:
  - `prettier --write --prose-wrap always --print-width 80 issues/0801-roastty-libghostty-rewrite/README.md issues/0801-roastty-libghostty-rewrite/777-binding-action-paste-test-reliability.md`
- Run:
  - `git diff --check`

The experiment passes if the previously blocking paste tests complete twice in
isolation and in the broader write-action cluster with stable enough timings to
support using them as verification, with any root cause and fix recorded. It is
Partial if the tests remain slow or hanging without a confirmed fix, and Fail
only if the investigation proves the current binding-action path is incorrect
rather than merely unreliable.

## Design Review

Codex reviewed the initial design and found three issues: the file recorded
result-review metadata before implementation, the grouped verification command
was described as only the three paste tests even though it matched a broader
write-action cluster, and the pass criterion claimed deterministic behavior
after only one run.

The design was updated to remove premature result-review metadata, accurately
name the broader grouped filter, and require each previously blocking paste test
to run twice with elapsed timing recorded before calling the tests stable. Codex
reviewed the revision, found no blockers, and approved the Experiment 777 plan
commit.

## Result

**Result:** Partial

The three previously blocking PTY-backed paste tests passed when run
individually, but the timings were too slow and variable to call the broad
`surface_binding_action_` filter reliable:

- `surface_binding_action_write_selection_file_paste_queues_path`
  - First run: passed, `finished in 66.73s`, `real 66.79`
  - Second run: passed, `finished in 53.55s`, `real 53.60`
- `surface_binding_action_write_screen_file_paste_queues_path`
  - First run: passed, `finished in 45.94s`, `real 45.99`
  - Second run: passed, `finished in 61.48s`, `real 61.54`
- `surface_binding_action_write_scrollback_file_paste_queues_path`
  - First run: passed, `finished in 61.86s`, `real 61.92`
  - Second run: passed, `finished in 8.75s`, `real 8.81`

Two attempted test-harness changes did not solve the reliability problem and
were reverted:

- Shortening the child command's post-output sleep from 5 seconds to 0.1 seconds
  still left the selection-file test at `finished in 57.19s`, `real 68.69`.
- Rendering every poll in `surface_snapshot_text_until` still left the
  selection-file test at `finished in 55.23s`, `real 64.48`.

A larger attempt to reuse one PTY child for all four paste formats also did not
help. After fixing the generated Python command syntax, the selection-file test
passed but became slower: `finished in 88.83s`, `real 97.91`. That refactor was
also reverted.

The planned broader write-action cluster was intentionally not run:

- `/usr/bin/time -p cargo test -p roastty surface_binding_action_write_ -- --nocapture --test-threads=1`

The individual paste-path timings and the failed harness attempts already showed
that the PTY-heavy paste cases were not reliable enough to use as a broad
cluster proof in this experiment. Running the broader cluster would have mixed
that known timing problem with unrelated write/copy/open coverage without
changing the Partial conclusion.

No Rust code changes were kept. The experiment leaves the Issue 801 checklist
unchanged.

Documentation checks passed after recording the partial result:

- `prettier --write --prose-wrap always --print-width 80 issues/0801-roastty-libghostty-rewrite/README.md issues/0801-roastty-libghostty-rewrite/777-binding-action-paste-test-reliability.md`
- `git diff --check`

## Conclusion

The paste tests are not failing logically: targeted runs show that they can
pass. The blocker is that their PTY/process timing is expensive and unstable
enough to make the broad `surface_binding_action_` filter impractical as a
checklist proof.

The next experiment should avoid trying to prove binding-action completeness
with the entire broad filter. A better reviewed strategy is to run focused
action-family filters that exclude the PTY-heavy paste-path tests, pair that
with the already-passing targeted key dispatch tests, and separately record the
paste-path tests as known slow targeted coverage.

## Completion Review

Codex reviewed the completed Partial result and agreed that Partial is the right
status: the paste tests pass logically, but the timing evidence is too slow and
variable to use them as broad checklist-proof coverage.

The initial completion review found one documentation gap: the planned broader
`surface_binding_action_write_` cluster status was not recorded. The result was
updated to state that the broader cluster was intentionally skipped because the
individual paste-test timings and failed harness attempts had already
invalidated the pass criterion. Codex reviewed the revision, found no blockers,
and approved the Experiment 777 result commit.
