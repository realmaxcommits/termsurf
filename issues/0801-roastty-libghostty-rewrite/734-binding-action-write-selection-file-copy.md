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

# Experiment 734: Binding Action Write Selection File Copy

## Description

Upstream Ghostty's `write_selection_file` binding writes the currently selected
text to a temporary file, then performs one of three actions with the file path:
copy it to the clipboard, paste it into the terminal, or open it with the OS.

Roastty already has the selection formatter, standard clipboard write callback,
and a `TempDir` helper. This experiment adds the smallest complete write-file
action: `write_selection_file:copy` and `write_selection_file:copy,<format>`.

The `paste` and `open` actions, plus `write_screen_file` and
`write_scrollback_file`, remain out of scope. This keeps the first file-writing
slice focused on temp-file creation, selection formatting, parser grammar, path
clipboard writes, and temp-directory lifetime. Those are the shared foundations
the other write-file actions need.

## Changes

- `roastty/src/lib.rs`
  - Add parsed binding-action support for `write_selection_file`.
  - Parse `copy`, `copy,plain`, `copy,vt`, and `copy,html`.
  - Reject missing parameters, empty action/format components, unknown actions,
    unknown formats, whitespace-padded values, extra comma-separated fields, and
    interior-NUL action text.
  - Add a surface helper that:
    - returns `false` for null/detached surfaces, missing workers, missing
      clipboard callbacks, no active selection, selection formatting failures,
      temp-directory/file failures, invalid path string conversion, or invalid
      clipboard C-string conversion;
    - formats the active selection with unwrap enabled and trim disabled,
      matching upstream `writeScreenFile`;
    - writes the formatted selection to a temporary file named `selection.txt`
      for plain/vt output or `selection.html` for HTML output;
    - retains every successful temporary directory on the surface so each copied
      file path remains valid after the binding returns and after later
      write-file actions create additional paths;
    - writes the file path to the standard clipboard as one `text/plain` item
      with `confirm = false`.
  - Keep `write_screen_file`, `write_scrollback_file`, `write_selection_file`
    `paste`, and `write_selection_file` `open` unsupported for now.

- `roastty/tests/abi_harness.c`
  - Add malformed `write_selection_file` parser rejection checks.
  - Add valid no-callback coverage returning `false`.

- Tests in `roastty/src/lib.rs`
  - Cover parser false paths for missing, empty, unknown, whitespace-padded,
    extra-field, interior-NUL, and unsupported action forms.
  - Cover null, detached, no-worker, no-selection, and missing-callback cases
    returning `false`.
  - Cover `write_selection_file:copy`, `copy,plain`, `copy,vt`, and `copy,html`
    writing a temp file, copying its path as `text/plain`, using the correct
    `.txt`/`.html` extension, and keeping the file readable after the binding
    returns.
  - Cover that an earlier copied file path remains readable after a later
    `write_selection_file:copy` call creates another retained temp directory.
  - Cover that the written file contents match the existing selection formatter
    for the requested output format, with unwrap enabled and trim disabled.
  - Cover existing `copy_to_clipboard` and `binding_action` suites still pass.

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

Codex reviewed the Experiment 734 design and found three real plan gaps. The
original temp-directory plan said directories would be retained on the surface,
but it did not specify whether repeated writes would preserve earlier copied
paths. The plan now requires retaining every successful temp directory and
testing that an earlier copied path remains readable after a subsequent
write-file action.

The review also noted that the parser false-path plan claimed interior-NUL
rejection without explicitly testing it; the test plan now includes that case.
Finally, this review result is recorded in the frontmatter and this section, and
the README tuple is updated to `Codex/Codex/-`.

With those fixes, the scoped first pass remains: `write_selection_file:copy`
only, with paste/open and screen/scrollback file actions deferred.

## Result

**Result:** Pass

Experiment 734 added `write_selection_file:copy` support. Roastty now parses
`write_selection_file:copy`, `write_selection_file:copy,plain`,
`write_selection_file:copy,vt`, and `write_selection_file:copy,html`. The
binding formats the active selection with unwrap enabled and trim disabled,
writes it to `selection.txt` or `selection.html` inside a new temporary
directory, retains every successful temporary directory on the surface, and
copies the canonical file path to the standard clipboard as one `text/plain`
item with `confirm = false`.

The action returns `false` for null surfaces, detached surfaces, missing
workers, no active selection, missing clipboard callbacks, unsupported
write-file actions, malformed parser inputs, and file/path/C-string failures.
The parser rejects missing parameters, empty fields, unknown actions, unknown
formats, whitespace-padded values, extra comma-separated fields, and interior
NULs.

`write_selection_file:paste`, `write_selection_file:open`, `write_screen_file`,
and `write_scrollback_file` remain future work.

Verification passed:

- `cargo fmt -p roastty`
- `cargo test -p roastty write_selection_file -- --nocapture --test-threads=1`
  - 3 passed
- `cargo test -p roastty copy_to_clipboard -- --nocapture --test-threads=1`
  - 2 passed
- `cargo test -p roastty binding_action -- --nocapture --test-threads=1`
  - 115 passed
- `cargo test -p roastty --test abi_harness`
  - 1 passed
- `cargo fmt -p roastty -- --check`
- `git diff --check`

## Conclusion

The first file-writing binding path is now live and keeps copied file paths
valid across later write-file actions by retaining all successful temporary
directories for the surface lifetime. The next write-file experiments can build
on the parser and temp-file lifetime model to add paste/open behavior and the
screen/scrollback variants.

## Completion Review

Codex reviewed the completed Experiment 734 result and implementation diff. It
found no implementation blockers. The review confirmed that the parser covers
the planned malformed forms, including missing and empty fields, unsupported
actions, unknown formats, extra fields, whitespace padding, and interior NULs.

The review also confirmed that selection formatting uses unwrap enabled and trim
disabled, temporary directories are retained across multiple writes, clipboard
writes use one standard `text/plain` path with `confirm = false`, and tests
cover format output, file readability, prior path retention, false paths, and
the recorded verification.
