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

# Experiment 716: Binding Action Adjust Selection

## Description

Experiment 715 added `select_all` binding-action support. Upstream Ghostty's
`performBindingAction` also supports `adjust_selection:<direction>`, which
mutates the active selection endpoint and scrolls that adjusted endpoint into
view.

Roastty already has the core terminal selection-adjustment behavior:

- `Terminal::active_selection()` returns the active selection;
- `Terminal::selection_adjust(selection, adjustment)` adjusts a supplied
  selection and returns the adjusted selection;
- `Terminal::set_selection(Some(selection))` installs the adjusted selection;
- `roastty_terminal_selection_adjust` exposes terminal-level C ABI coverage.

This experiment wires the existing terminal behavior into
`roastty_surface_binding_action("adjust_selection:<direction>")` and adds the
surface/terminal viewport logic needed to keep the adjusted endpoint visible.

This does not implement copy/paste actions, search actions, write-file actions,
keybind storage/lookup, frontend selection routing, or clipboard integration.

## Changes

- `roastty/src/lib.rs`
  - Extend the internal parsed binding-action enum with
    `AdjustSelection(TerminalSelectionAdjustment)`.
  - Add a parser for Ghostty's adjustment names:
    - `left`
    - `right`
    - `up`
    - `down`
    - `page_up`
    - `page_down`
    - `home`
    - `end`
    - `beginning_of_line`
    - `end_of_line`
  - Extend `parse_binding_action` to accept `adjust_selection:<direction>` and
    reject missing, empty, unknown, whitespace-padded, or extra-colon
    parameters.
  - Add a surface helper that:
    - returns `false` for null, detached, and no-worker surfaces;
    - returns `false` when the worker-backed terminal has no active selection,
      matching upstream fall-through behavior;
    - adjusts the existing active selection with `Terminal::selection_adjust`;
    - installs the adjusted selection with `Terminal::set_selection`;
    - scrolls the adjusted selection end point into view;
    - requests a render and returns `true` after a successful adjustment.
  - Keep split, close, `text:`, `csi:`, `esc:`, `reset`, `clear_screen`, scroll,
    prompt-jump, and select-all action semantics unchanged.

- `roastty/src/terminal/screen.rs`
  - Add a helper that scrolls a supplied selection endpoint into view using the
    same rule as upstream Ghostty:
    - if the endpoint is already between viewport top-left and bottom-right, do
      not move the viewport;
    - if the endpoint is above the viewport, scroll to the endpoint pin;
    - if the endpoint is below the viewport, scroll to `endpoint - (rows - 1)`
      where possible so the endpoint lands on the bottom visible row.
    - if `rows <= 1` or walking up `rows - 1` rows from the endpoint cannot
      produce a valid pin, scroll to the endpoint pin instead and let the
      existing viewport clamping preserve integrity.

- `roastty/src/terminal/terminal.rs`
  - Add a terminal-level forwarding helper for the endpoint-scroll behavior.

- `roastty/tests/abi_harness.c`
  - Add C ABI smoke coverage that malformed `adjust_selection` forms are
    rejected.
  - Add no-worker coverage that representative valid adjustment forms return
    `false` without crashing.

- Tests in `roastty/src/lib.rs`
  - Cover parser false paths for missing, empty, unknown, whitespace-padded, and
    extra-colon `adjust_selection` forms.
  - Cover null, detached, no-worker, and no-active-selection surfaces returning
    `false`.
  - Cover all valid parser forms in a no-worker table proving they parse and
    return `false` without crashing: `left`, `right`, `up`, `down`, `page_up`,
    `page_down`, `home`, `end`, `beginning_of_line`, and `end_of_line`.
  - Cover worker-backed adjustments for representative horizontal, vertical,
    page, home/end, and beginning/end-of-line directions by comparing the active
    selection after the binding action to `Terminal::selection_adjust`.
  - Cover upward and downward endpoint scroll behavior.
  - Cover the below-viewport fallback for a one-row viewport or otherwise
    invalid upward adjustment.
  - Cover the already-visible endpoint case leaving the viewport unchanged while
    still requesting render.
  - Re-run existing binding-action tests to prove previous action semantics did
    not change.

## Verification

Run:

- `cargo fmt -p roastty`
- `cargo test -p roastty adjust_selection -- --nocapture --test-threads=1`
- `cargo test -p roastty binding_action -- --nocapture --test-threads=1`
- `cargo test -p roastty --test abi_harness`
- `cargo fmt -p roastty -- --check`
- `git diff --check`

## Design Review

Codex reviewed the Experiment 716 design and found the scope otherwise matches
upstream behavior: return `false` when there is no active selection, adjust and
install the active selection endpoint, scroll that endpoint into view, request
render, and return `true` on success.

The review raised two technical blockers before plan commit. First, parser
coverage needed to be explicit for every accepted direction rather than only
representative groups. The plan now requires a no-worker table covering all ten
valid direction names. Second, the endpoint-scroll fallback for below-viewport
targets needed a deterministic rule. The plan now states that `rows <= 1` or
failed `endpoint - (rows - 1)` movement falls back to scrolling to the endpoint
pin, relying on existing viewport clamping for integrity.

The review also raised the normal workflow provenance requirement. Design-review
frontmatter and this review section are now present, and the README provenance
tuple will be updated to `Codex/Codex/-` before the plan commit. Result-review
provenance will be added only after implementation and completion review.
