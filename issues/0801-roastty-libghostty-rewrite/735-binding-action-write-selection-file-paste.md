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

# Experiment 735: Binding Action Write Selection File Paste

## Description

Experiment 734 added the first `write_selection_file` path:
`write_selection_file:copy`. Upstream Ghostty also supports
`write_selection_file:paste`, which writes the current selection to a temporary
file and queues the resulting file path into the terminal.

Roastty now has the temp-file lifetime model, parser foundation, selection
formatter, and worker `queue_write` path needed for this next action. This
experiment extends the existing selection-file helper to support the `paste`
action while keeping `open`, `write_screen_file`, and `write_scrollback_file`
out of scope.

## Changes

- `roastty/src/lib.rs`
  - Extend the write-file parsed action representation to include a copy/paste
    action plus the existing plain/vt/html format.
  - Parse `write_selection_file:paste`, `paste,plain`, `paste,vt`, and
    `paste,html`.
  - Keep rejecting `write_selection_file:open` and all malformed forms covered
    by Experiment 734.
  - Reuse the existing selection-file creation path so copy and paste share:
    - active-selection lookup;
    - unwrap-enabled, trim-disabled selection formatting;
    - `selection.txt` / `selection.html` naming;
    - successful temp-directory retention on the surface.
  - For the paste action, queue the canonical file path bytes to the terminal
    worker with no trailing newline or NUL.
  - Honor readonly mode for paste by returning `false` without creating a temp
    file or queueing path bytes.
  - Return `false` if queueing the path to the worker fails.

- `roastty/tests/abi_harness.c`
  - Add valid no-callback / no-worker coverage for the new paste forms returning
    `false`.
  - Keep malformed paste/open parser rejection coverage.

- Tests in `roastty/src/lib.rs`
  - Cover `write_selection_file:paste`, `paste,plain`, `paste,vt`, and
    `paste,html` writing the selected text to the expected temp-file extension
    and queueing exactly the copied path bytes to the child process.
  - Cover that paste retains the temporary directory, so the queued path remains
    readable after the binding returns.
  - Cover that paste returns `false` for null/detached surfaces, missing
    workers, no active selection, and worker queue failures.
  - Cover that paste returns `false` while readonly and that no path bytes reach
    the worker in readonly mode.
  - Cover copy still writes to the clipboard after the helper refactor.

## Verification

Run:

- `cargo fmt -p roastty`
- `cargo test -p roastty write_selection_file -- --nocapture --test-threads=1`
- `cargo test -p roastty copy_to_clipboard -- --nocapture --test-threads=1`
- `cargo test -p roastty binding_action -- --nocapture --test-threads=1`
- `cargo test -p roastty --test abi_harness`
- `cargo fmt -p roastty -- --check`
- `git diff --check`

## Design Review

Codex reviewed the Experiment 735 design and found one real behavior gap: the
paste action writes bytes into the PTY, so it must honor the readonly gate added
by Experiment 726. The plan now requires returning `false` without creating a
temp file or queueing bytes when the surface is readonly, and testing that no
path bytes reach the worker in readonly mode.

The review also required recording the design review result before the plan
commit. This section and the `[review.design]` frontmatter now record the
review, and the README tuple is updated to `Codex/Codex/-`.

With the readonly gate added, the review found the parser scope, temp-directory
lifetime model, exact queued path bytes requirement, queue-failure false path,
and verification plan coherent.

## Result

**Result:** Pass

Experiment 735 added `write_selection_file:paste` support. The
`write_selection_file` parser now accepts `paste`, `paste,plain`, `paste,vt`,
and `paste,html` alongside the existing copy forms. Both actions share the same
selection-file creation path: active selection lookup, unwrap-enabled and
trim-disabled formatting, `selection.txt` / `selection.html` naming, canonical
path resolution, and successful temporary-directory retention on the surface.

For paste, Roastty queues exactly the canonical file path bytes to the terminal
worker with no trailing newline or NUL. Paste returns `false` in readonly mode
before creating a temp file or queueing bytes, and it returns `false` if the
worker queue is disconnected.

`write_selection_file:open`, `write_screen_file`, and `write_scrollback_file`
remain future work.

Verification passed:

- `cargo fmt -p roastty`
- `cargo test -p roastty write_selection_file -- --nocapture --test-threads=1`
  - 5 passed
- `cargo test -p roastty copy_to_clipboard -- --nocapture --test-threads=1`
  - 2 passed
- `cargo test -p roastty binding_action -- --nocapture --test-threads=1`
  - 117 passed
- `cargo test -p roastty --test abi_harness`
  - 1 passed
- `cargo fmt -p roastty -- --check`
- `git diff --check`

## Conclusion

The selection-file binding now supports both copy and paste actions while
preserving the same temp-file lifetime model. The remaining write-file surface
is the OS open action and the screen/scrollback variants, which can reuse this
parser and file-creation foundation.

## Completion Review

Codex reviewed the completed Experiment 735 result and implementation diff. It
found no implementation blockers.

The review confirmed that the paste parser accepts the intended forms while
preserving malformed rejection coverage, readonly returns `false` before
temp-file creation or queueing, queue failures return `false`, queued bytes are
exactly the canonical path bytes, temporary directories are retained after
successful paste, copy still uses the shared file path and clipboard behavior,
and the verification record matches the implemented coverage.
