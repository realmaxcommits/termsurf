+++
[implementer]
agent = "codex"
model = "gpt-5"
reasoning = "high"

[review.design]
agent = "codex"
model = "gpt-5"
reasoning = "medium"

[review.result]
agent = "codex"
model = "gpt-5"
reasoning = "medium"
+++

# Experiment 738: Binding Action Write Scrollback File Copy

## Description

Experiments 734 through 737 implemented selection-file and screen-file
copy/paste actions. Upstream Ghostty also supports `write_scrollback_file`,
which formats the history area above the active screen and applies the same
write-file actions.

Roastty has history points, history grid refs, and PageList history bounds, but
the current terminal formatter helper defaults `None` selections to the active
screen. This experiment adds the smallest history-specific formatter hook needed
for the first scrollback target action, then implements
`write_scrollback_file:copy`.

`write_scrollback_file:paste`, `write_scrollback_file:open`, and all remaining
`open` actions stay out of scope.

## Changes

- `roastty/src/terminal/screen.rs`
  - Add an internal helper that returns the current history selection, using the
    page-list history top-left and bottom-right bounds.
  - Return `None` when there is no scrollback history, matching upstream's
    behavior of doing no work for an empty history selection.

- `roastty/src/terminal/terminal.rs`
  - Add a terminal-level helper that formats the current scrollback history with
    the existing formatter options.
  - Use unwrap enabled and trim disabled for parity with existing write-file
    targets.

- `roastty/src/lib.rs`
  - Extend the write-file target enum with `Scrollback`.
  - For the scrollback target, write `scrollback.txt` for plain/vt and
    `scrollback.html` for html.
  - Parse `write_scrollback_file:copy`, `copy,plain`, `copy,vt`, and
    `copy,html`.
  - Keep rejecting malformed `write_scrollback_file` forms plus `paste` and
    `open` until those actions are implemented for the scrollback target.
  - Dispatch scrollback copy through the same retained-temp-directory and
    standard clipboard path as other copy targets.

- `roastty/tests/abi_harness.c`
  - Add malformed `write_scrollback_file` parser rejection coverage.
  - Add valid no-worker / no-callback false-path coverage for the new copy forms
    returning `false`.

- Tests in `roastty/src/lib.rs`
  - Build known terminal content with both scrollback history and visible active
    rows, then assert scrollback-file output includes only the history above the
    active screen and excludes visible screen rows.
  - Cover `write_scrollback_file:copy`, `copy,plain`, `copy,vt`, and `copy,html`
    writing a readable temp file with the expected filename extension and
    copying its canonical path as `text/plain` without confirmation.
  - Assert each written file's contents match the new scrollback formatter
    output for its requested format.
  - Cover parser rejection for missing parameter, empty parameter, empty
    action/format components, unknown action, unsupported `paste` and `open`,
    unknown format, whitespace-padded values, extra comma fields, and interior
    NUL.
  - Cover that scrollback copy returns `false` and writes no clipboard data when
    there is no scrollback history.
  - Cover false paths for null surfaces, detached surfaces, missing workers, and
    missing clipboard callbacks.
  - Keep existing `write_screen_file` and `write_selection_file` tests passing.

## Verification

Run:

- `cargo fmt -p roastty`
- `cargo test -p roastty write_scrollback_file -- --nocapture --test-threads=1`
- `cargo test -p roastty write_screen_file -- --nocapture --test-threads=1`
- `cargo test -p roastty write_selection_file -- --nocapture --test-threads=1`
- `cargo test -p roastty binding_action -- --nocapture --test-threads=1`
- `cargo test -p roastty --test abi_harness`
- `cargo fmt -p roastty -- --check`
- `git diff --check`

## Design Review

Codex reviewed the Experiment 738 design and found two real test-plan gaps.
First, comparing written files only against the new scrollback formatter helper
would be circular, so the plan now requires a known history-plus-visible-screen
fixture and assertions that scrollback output includes only history above the
active screen while excluding visible rows. Second, parser rejection coverage
needed the same explicit malformed cases as prior write-file experiments; the
plan now names missing/empty parameters, empty components, unknown action,
unsupported paste/open, unknown format, whitespace, extra fields, and interior
NUL.

The review also required recording `[review.design]` frontmatter, this review
section, and the README tuple before the plan commit. With those changes, the
plan is ready for the reviewed plan commit.

## Result

**Result:** Pass

Experiment 738 added `write_scrollback_file:copy` support. Roastty now has a
small internal history formatter path: PageList exposes the history selection
bounded by the history top-left and history bottom-right pins, Screen exposes
that selection, and Terminal formats it through the existing formatter with
unwrap enabled and trim disabled.

The write-file target enum now includes `Scrollback`. Scrollback copy writes
`scrollback.txt` for plain/vt output and `scrollback.html` for html output,
retains the temp directory, and copies the canonical path to the standard
clipboard as `text/plain` without confirmation.

The parser accepts `write_scrollback_file:copy`, `copy,plain`, `copy,vt`, and
`copy,html`. Malformed forms, unsupported formats, `paste`, and `open` remain
rejected for the scrollback target.

Tests cover a terminal fixture with both scrollback history and visible active
rows, asserting the scrollback output contains `history-red` and `history-two`
while excluding `visible-one`, `visible-two`, and `visible-three`. They also
cover no-history false paths, missing worker/callback false paths, clipboard
path metadata, filename extensions, and existing screen/selection write-file
regressions.

Verification passed:

- `cargo fmt -p roastty`
- `cargo test -p roastty write_scrollback_file -- --nocapture --test-threads=1`
  - 2 passed
- `cargo test -p roastty write_screen_file -- --nocapture --test-threads=1`
  - 4 passed
- `cargo test -p roastty write_selection_file -- --nocapture --test-threads=1`
  - 5 passed
- `cargo test -p roastty binding_action -- --nocapture --test-threads=1`
  - 123 passed
- `cargo test -p roastty --test abi_harness`
  - 1 passed
- `cargo fmt -p roastty -- --check`
- `git diff --check`

## Conclusion

Roastty now supports the copy action for all three write-file targets:
selection, screen, and scrollback. The remaining write-file surface is paste for
scrollback and the `open` action for each target. The `open` action still needs
runtime URL/open action plumbing before it can be ported faithfully.

## Completion Review

Codex reviewed the completed Experiment 738 result and implementation diff. It
found no implementation blockers.

The review confirmed that `write_scrollback_file:copy` is scoped to copy only,
uses a scrollback-only formatter selection, rejects paste/open and malformed
forms, writes retained `scrollback.txt` / `scrollback.html` files, copies the
canonical path as `text/plain` without confirmation, and includes tests for
history-only output versus visible rows plus existing write-file regression
coverage.
