+++
[implementer]
agent = "codex"
model = "gpt-5"
reasoning = "high"

[review.design]
agent = "codex"
model = "gpt-5"
reasoning = "medium"
+++

# Experiment 741: Binding Action Write File Open

## Description

Experiments 734 through 739 completed copy and paste support for
`write_selection_file`, `write_screen_file`, and `write_scrollback_file`.
Experiment 740 added the runtime `open_url` action ABI foundation needed by the
remaining `open` action.

This experiment wires `write_*_file:open` through the existing write-file helper
for all three targets: selection, screen, and scrollback. It creates the same
temporary formatted file as copy/paste, then forwards the canonical path through
`ROASTTY_ACTION_OPEN_URL` with upstream-compatible open kinds. The upstream
default OS opener remains out of scope; if the runtime action callback is
missing or rejects the URL, Roastty returns `false`.

## Changes

- `roastty/src/lib.rs`
  - Extend `WriteFileAction` and `write_file_action_from_str` to accept `open`.
  - Extend `Surface::write_file` with an `Open` branch that:
    - creates the target file with the existing target-aware formatter;
    - maps `plain` and `vt` formats to `ROASTTY_ACTION_OPEN_URL_KIND_TEXT`;
    - maps `html` format to `ROASTTY_ACTION_OPEN_URL_KIND_HTML`;
    - calls `Surface::perform_open_url_result(kind, path.as_bytes())`;
    - retains the temporary directory only when the runtime callback accepts the
      open-url action;
    - returns `false` for missing app/callback, detached surfaces, no selection
      or no scrollback content, write-file creation failure, and callback
      rejection.
  - Keep readonly behavior unchanged: `open` is not terminal input and should
    not use the paste-only readonly gate.
  - Keep OS fallback opener behavior out of scope.

- `roastty/tests/abi_harness.c`
  - Move `write_selection_file:open`, `write_screen_file:open`, and
    `write_scrollback_file:open` from rejected parser coverage to valid
    no-callback false-path coverage.
  - Add `open,plain`, `open,vt`, and `open,html` valid no-callback false-path
    coverage for each target.
  - Keep malformed open forms rejected: empty formats, unsupported formats,
    extra components, whitespace-padded action names, and NUL-containing
    parameters.

- Tests in `roastty/src/lib.rs`
  - Cover `write_selection_file:open`, `open,plain`, `open,vt`, and `open,html`
    creating `selection.txt` or `selection.html`, forwarding the canonical path
    through the open-url callback, and writing file contents that match the
    selected plain/vt/html formatter output.
  - Cover `write_screen_file:open`, `open,plain`, `open,vt`, and `open,html`
    creating `screen.txt` or `screen.html`, forwarding through the open-url
    callback, and writing full-screen formatter output.
  - Cover `write_scrollback_file:open`, `open,plain`, `open,vt`, and `open,html`
    creating `scrollback.txt` or `scrollback.html`, forwarding through the
    open-url callback, and writing only history above the active screen.
  - Assert open-url kind mapping: plain/vt use text, html uses html.
  - Cover callback rejection returning `false` and not retaining the temporary
    directory.
  - Cover no-selection and no-history false paths returning `false` without
    retaining a temporary directory.
  - Keep existing copy, paste, parser, open-url ABI, and ABI harness tests
    passing.

## Verification

Run:

- `cargo fmt -p roastty`
- `cargo test -p roastty write_selection_file -- --nocapture --test-threads=1`
- `cargo test -p roastty write_screen_file -- --nocapture --test-threads=1`
- `cargo test -p roastty write_scrollback_file -- --nocapture --test-threads=1`
- `cargo test -p roastty open_url -- --nocapture --test-threads=1`
- `cargo test -p roastty binding_action -- --nocapture --test-threads=1`
- `cargo test -p roastty --test abi_harness`
- `cargo fmt -p roastty -- --check`
- `git diff --check`

## Design Review

Codex reviewed the Experiment 741 design and approved the technical scope. The
review confirmed that the plan is coherent once Experiment 740's process gates
are complete: route `write_*_file:open` through `ROASTTY_ACTION_OPEN_URL`, map
plain/vt formats to text and html to html, retain temporary directories only on
callback acceptance, return `false` without an OS fallback when the runtime does
not handle the action, and cover all three targets plus parser and ABI
regressions.

The review initially raised a stale process concern that Experiment 740 still
needed completion-review metadata and a result commit. Current git history shows
Experiment 740 has both required commits:
`7e594cd602904 Plan doors for paper paths` and
`b265c8606c76d Let doors learn borrowed names`. No Experiment 740 blocker
remains.

The remaining workflow requirement from the review was to record
`[review.design]`, this review section, and the README tuple before the
Experiment 741 plan commit; those records are now present.
